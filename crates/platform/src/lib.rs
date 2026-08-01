//! Narrow operating-system process/resource adapters.

use std::fs::File;
use std::io;
use std::process::Command;

/// A prepared direct worker asset kept alive until the child is spawned.
#[derive(Debug)]
#[must_use = "prepared worker assets must configure exactly one worker command"]
pub struct DirectWorkerAsset {
    resource_id: u64,
    #[cfg(unix)]
    descriptor: std::os::fd::OwnedFd,
}

impl DirectWorkerAsset {
    /// Returns the process-local resource identifier carried by the worker manifest.
    #[must_use]
    pub const fn resource_id(&self) -> u64 {
        self.resource_id
    }

    /// Configures one command to inherit only stdio and this prepared asset resource.
    pub fn configure(self, command: &mut Command) -> io::Result<()> {
        #[cfg(unix)]
        {
            unix::configure(command, self.descriptor)
        }
        #[cfg(not(unix))]
        {
            let _ = (self, command);
            Err(direct_transport_unavailable())
        }
    }
}

/// Returns whether this target has an implemented direct worker-asset launcher.
#[must_use]
pub const fn direct_worker_asset_supported() -> bool {
    cfg!(unix)
}

/// Duplicates one read-only source into a close-on-exec prepared direct resource.
pub fn prepare_direct_worker_asset(source: &File) -> io::Result<DirectWorkerAsset> {
    #[cfg(unix)]
    {
        unix::prepare(source)
    }
    #[cfg(not(unix))]
    {
        let _ = source;
        Err(direct_transport_unavailable())
    }
}

/// Takes ownership of the single direct asset resource named by the worker manifest.
///
/// The call is process-global and one-shot so a safe caller cannot create multiple owners for the
/// same inherited operating-system resource.
pub fn take_inherited_worker_asset(resource_id: u64) -> io::Result<File> {
    #[cfg(unix)]
    {
        unix::take(resource_id)
    }
    #[cfg(not(unix))]
    {
        let _ = resource_id;
        Err(direct_transport_unavailable())
    }
}

#[cfg(not(unix))]
fn direct_transport_unavailable() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "direct worker asset transport is unavailable on this target",
    )
}

#[cfg(unix)]
mod unix {
    use std::fs::File;
    use std::io;
    use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    use std::sync::atomic::{AtomicBool, Ordering};

    use rustix::io::{FdFlags, fcntl_dupfd_cloexec, fcntl_getfd, fcntl_setfd};

    use super::DirectWorkerAsset;

    const FIRST_NON_STDIO_DESCRIPTOR: RawFd = 3;
    static WORKER_ASSET_TAKEN: AtomicBool = AtomicBool::new(false);

    pub(super) fn prepare(source: &File) -> io::Result<DirectWorkerAsset> {
        let descriptor =
            fcntl_dupfd_cloexec(source, FIRST_NON_STDIO_DESCRIPTOR).map_err(io::Error::from)?;
        let resource_id = u64::try_from(descriptor.as_raw_fd()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "prepared descriptor identifier is out of range",
            )
        })?;
        Ok(DirectWorkerAsset {
            resource_id,
            descriptor,
        })
    }

    #[allow(
        unsafe_code,
        reason = "pre_exec is the stable Unix hook for changing descriptor inheritance after fork"
    )]
    pub(super) fn configure(command: &mut Command, descriptor: OwnedFd) -> io::Result<()> {
        let descriptor_id = descriptor.as_raw_fd();
        // SAFETY: The callback performs only fcntl operations and close_fds' documented
        // async-signal-safe CLOEXEC sweep. It does not allocate, lock, log, or resolve paths.
        unsafe {
            command.pre_exec(move || {
                fcntl_setfd(&descriptor, FdFlags::empty()).map_err(io::Error::from)?;
                // SAFETY: The sorted keep list contains the one valid descriptor owned by this
                // closure. The crate documents this operation as async-signal-safe after fork.
                close_fds::set_fds_cloexec(
                    FIRST_NON_STDIO_DESCRIPTOR,
                    std::slice::from_ref(&descriptor_id),
                );
                Ok(())
            });
        }
        Ok(())
    }

    #[allow(
        unsafe_code,
        reason = "the manifest names an inherited raw descriptor that has no Rust owner yet"
    )]
    pub(super) fn take(resource_id: u64) -> io::Result<File> {
        if WORKER_ASSET_TAKEN.swap(true, Ordering::AcqRel) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "the inherited worker asset was already claimed",
            ));
        }
        let descriptor_id = RawFd::try_from(resource_id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker asset descriptor identifier is out of range",
            )
        })?;
        if descriptor_id < FIRST_NON_STDIO_DESCRIPTOR {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker asset descriptor must not alias stdio",
            ));
        }

        // SAFETY: fcntl validates that the manifest-named descriptor is open before ownership is
        // constructed. This process has exactly one claim path, enforced by WORKER_ASSET_TAKEN.
        let borrowed = unsafe { BorrowedFd::borrow_raw(descriptor_id) };
        fcntl_getfd(borrowed).map_err(io::Error::from)?;
        fcntl_setfd(borrowed, FdFlags::CLOEXEC).map_err(io::Error::from)?;
        // SAFETY: The validated descriptor is open, uniquely claimed above, and is now CLOEXEC.
        Ok(File::from(unsafe { OwnedFd::from_raw_fd(descriptor_id) }))
    }

    #[cfg(test)]
    #[allow(
        unsafe_code,
        reason = "test-only non-owning probe checks whether a numeric sentinel survived exec"
    )]
    fn descriptor_is_open(descriptor_id: RawFd) -> bool {
        // SAFETY: The borrowed descriptor is used only for F_GETFD and is never closed or owned.
        fcntl_getfd(unsafe { BorrowedFd::borrow_raw(descriptor_id) }).is_ok()
    }

    #[cfg(test)]
    mod tests {
        use std::io::Read;
        use std::os::fd::AsRawFd;
        use std::process::Stdio;

        use rustix::io::{FdFlags, fcntl_dupfd_cloexec, fcntl_setfd};

        use super::*;

        const CHILD_ASSET_ID: &str = "PARTPROBE_TEST_DIRECT_ASSET_ID";
        const CHILD_SENTINEL_ID: &str = "PARTPROBE_TEST_INHERITABLE_SENTINEL_ID";

        #[test]
        fn direct_asset_allowlists_the_source_and_excludes_an_inheritable_sentinel() {
            if let Some(asset_id) = std::env::var_os(CHILD_ASSET_ID) {
                let asset_id = asset_id
                    .to_string_lossy()
                    .parse::<u64>()
                    .expect("child asset descriptor ID must parse");
                let sentinel_id = std::env::var(CHILD_SENTINEL_ID)
                    .expect("child sentinel descriptor ID must exist")
                    .parse::<RawFd>()
                    .expect("child sentinel descriptor ID must parse");
                let mut asset = take(asset_id).expect("allowlisted asset must be inherited");
                let mut contents = String::new();
                asset
                    .read_to_string(&mut contents)
                    .expect("allowlisted asset must be readable");
                assert!(contents.contains("partprobe-platform"));
                assert!(
                    !descriptor_is_open(sentinel_id),
                    "unrelated inheritable sentinel must be closed by exec"
                );
                return;
            }

            let source = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
                .expect("platform manifest must open");
            let sentinel = fcntl_dupfd_cloexec(&source, FIRST_NON_STDIO_DESCRIPTOR)
                .expect("inheritable sentinel must duplicate");
            fcntl_setfd(&sentinel, FdFlags::empty())
                .expect("sentinel must be intentionally inheritable");
            let sentinel_id = sentinel.as_raw_fd();
            let direct = prepare(&source).expect("direct asset must prepare");
            let asset_id = direct.resource_id();
            let mut command =
                Command::new(std::env::current_exe().expect("test path must resolve"));
            command
                .arg("--exact")
                .arg("unix::tests::direct_asset_allowlists_the_source_and_excludes_an_inheritable_sentinel")
                .arg("--nocapture")
                .env_clear()
                .env(CHILD_ASSET_ID, asset_id.to_string())
                .env(CHILD_SENTINEL_ID, sentinel_id.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            configure(&mut command, direct.descriptor).expect("command must configure");

            let status = command.status().expect("probe child must launch");

            assert!(
                status.success(),
                "probe child must verify the descriptor set"
            );
        }
    }
}
