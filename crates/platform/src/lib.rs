//! Narrow operating-system process/resource adapters.

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

/// A prepared direct worker asset kept alive until the child is spawned.
#[derive(Debug)]
#[must_use = "prepared worker assets must configure exactly one worker command"]
pub struct DirectWorkerAsset {
    resource_id: u64,
    #[cfg(unix)]
    descriptor: std::os::fd::OwnedFd,
    #[cfg(windows)]
    handle: std::os::windows::io::OwnedHandle,
    #[cfg(windows)]
    _launch_guard: std::sync::MutexGuard<'static, ()>,
}

impl DirectWorkerAsset {
    /// Returns the process-local resource identifier carried by the worker manifest.
    #[must_use]
    pub const fn resource_id(&self) -> u64 {
        self.resource_id
    }
}

/// A worker command with a cleared environment and fixed piped-input/piped-output contract.
#[derive(Debug)]
pub struct WorkerCommand {
    program: PathBuf,
    current_directory: Option<PathBuf>,
    environment: Vec<(OsString, OsString)>,
    direct_asset: Option<DirectWorkerAsset>,
}

impl WorkerCommand {
    /// Creates a path-explicit worker command with no inherited environment.
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            program: PathBuf::from(program.as_ref()),
            current_directory: None,
            environment: Vec::new(),
            direct_asset: None,
        }
    }

    /// Selects the worker's controlled current directory.
    pub fn current_dir(&mut self, directory: impl AsRef<Path>) -> &mut Self {
        self.current_directory = Some(directory.as_ref().to_owned());
        self
    }

    /// Adds one explicit environment entry to the otherwise-cleared worker environment.
    pub fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        let key = key.as_ref().to_owned();
        self.environment.retain(|(existing, _)| existing != &key);
        self.environment.push((key, value.as_ref().to_owned()));
        self
    }

    /// Selects one already-prepared direct asset for this launch.
    pub fn direct_asset(&mut self, asset: DirectWorkerAsset) -> &mut Self {
        self.direct_asset = Some(asset);
        self
    }

    /// Spawns the worker with piped stdin/stdout and null stderr.
    pub fn spawn(self) -> io::Result<WorkerChild> {
        #[cfg(windows)]
        if self.direct_asset.is_some() {
            return windows::spawn_direct(self);
        }
        spawn_standard(self)
    }
}

fn spawn_standard(mut worker: WorkerCommand) -> io::Result<WorkerChild> {
    let mut command = Command::new(worker.program);
    command
        .env_clear()
        .envs(worker.environment)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(directory) = worker.current_directory {
        command.current_dir(directory);
    }
    if let Some(asset) = worker.direct_asset.take() {
        #[cfg(unix)]
        unix::configure(&mut command, asset.descriptor)?;
        #[cfg(windows)]
        {
            let _ = asset;
            return Err(io::Error::other(
                "a Windows direct asset requires the restricted worker launcher",
            ));
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = asset;
            return Err(direct_transport_unavailable());
        }
    }
    command.spawn().map(WorkerChild::standard)
}

/// Owned input pipe for a supervised worker.
pub enum WorkerStdin {
    Standard(ChildStdin),
    #[cfg(windows)]
    Restricted(File),
}

impl Write for WorkerStdin {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self {
            Self::Standard(stdin) => stdin.write(buffer),
            #[cfg(windows)]
            Self::Restricted(stdin) => stdin.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Standard(stdin) => stdin.flush(),
            #[cfg(windows)]
            Self::Restricted(stdin) => stdin.flush(),
        }
    }
}

/// Owned output pipe for a supervised worker.
pub enum WorkerStdout {
    Standard(ChildStdout),
    #[cfg(windows)]
    Restricted(File),
}

impl Read for WorkerStdout {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Standard(stdout) => stdout.read(buffer),
            #[cfg(windows)]
            Self::Restricted(stdout) => stdout.read(buffer),
        }
    }
}

/// The process and protocol pipes needed by the geometry supervisor.
pub struct WorkerChild {
    inner: WorkerChildInner,
}

enum WorkerChildInner {
    Standard(Child),
    #[cfg(windows)]
    Restricted(windows::RestrictedChild),
}

impl WorkerChild {
    fn standard(child: Child) -> Self {
        Self {
            inner: WorkerChildInner::Standard(child),
        }
    }

    /// Takes the single writer for worker stdin.
    pub fn take_stdin(&mut self) -> Option<WorkerStdin> {
        match &mut self.inner {
            WorkerChildInner::Standard(child) => child.stdin.take().map(WorkerStdin::Standard),
            #[cfg(windows)]
            WorkerChildInner::Restricted(child) => child.stdin.take().map(WorkerStdin::Restricted),
        }
    }

    /// Takes the single reader for worker stdout.
    pub fn take_stdout(&mut self) -> Option<WorkerStdout> {
        match &mut self.inner {
            WorkerChildInner::Standard(child) => child.stdout.take().map(WorkerStdout::Standard),
            #[cfg(windows)]
            WorkerChildInner::Restricted(child) => {
                child.stdout.take().map(WorkerStdout::Restricted)
            }
        }
    }

    /// Returns the exit status when available without blocking.
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match &mut self.inner {
            WorkerChildInner::Standard(child) => child.try_wait(),
            #[cfg(windows)]
            WorkerChildInner::Restricted(child) => child.try_wait(),
        }
    }

    /// Requests immediate process termination.
    pub fn kill(&mut self) -> io::Result<()> {
        match &mut self.inner {
            WorkerChildInner::Standard(child) => child.kill(),
            #[cfg(windows)]
            WorkerChildInner::Restricted(child) => child.kill(),
        }
    }

    /// Waits for and reaps the worker process.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        match &mut self.inner {
            WorkerChildInner::Standard(child) => child.wait(),
            #[cfg(windows)]
            WorkerChildInner::Restricted(child) => child.wait(),
        }
    }
}

/// Returns whether this target has an implemented direct worker-asset launcher.
#[must_use]
pub const fn direct_worker_asset_supported() -> bool {
    cfg!(any(unix, windows))
}

/// Duplicates one read-only source into a prepared direct resource.
pub fn prepare_direct_worker_asset(source: &File) -> io::Result<DirectWorkerAsset> {
    #[cfg(unix)]
    {
        unix::prepare(source)
    }
    #[cfg(windows)]
    {
        windows::prepare(source)
    }
    #[cfg(not(any(unix, windows)))]
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
    #[cfg(windows)]
    {
        windows::take(resource_id)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = resource_id;
        Err(direct_transport_unavailable())
    }
}

#[cfg(not(any(unix, windows)))]
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
    pub(super) fn prepare_inheritable_sentinel(source: &File) -> io::Result<OwnedFd> {
        let descriptor =
            fcntl_dupfd_cloexec(source, FIRST_NON_STDIO_DESCRIPTOR).map_err(io::Error::from)?;
        fcntl_setfd(&descriptor, FdFlags::empty()).map_err(io::Error::from)?;
        Ok(descriptor)
    }

    #[cfg(test)]
    pub(super) fn sentinel_resource_id(descriptor: &OwnedFd) -> u64 {
        u64::try_from(descriptor.as_raw_fd()).expect("sentinel descriptor must be nonnegative")
    }

    #[cfg(test)]
    #[allow(
        unsafe_code,
        reason = "test-only non-owning probe checks whether a numeric sentinel survived exec"
    )]
    pub(super) fn inherited_sentinel_is_absent(resource_id: u64) -> bool {
        let Ok(descriptor_id) = RawFd::try_from(resource_id) else {
            return true;
        };
        // SAFETY: The borrowed descriptor is used only for F_GETFD and is never closed or owned.
        fcntl_getfd(unsafe { BorrowedFd::borrow_raw(descriptor_id) }).is_err()
    }
}

#[cfg(windows)]
mod windows {
    use std::cmp::Ordering as Comparison;
    use std::ffi::OsString;
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::os::windows::process::ExitStatusExt;
    use std::process::ExitStatus;
    use std::ptr::{null, null_mut};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    use windows_sys::Win32::Foundation::{
        DUPLICATE_SAME_ACCESS, DuplicateHandle, GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT,
        INVALID_HANDLE_VALUE, SetHandleInformation, WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DeleteProcThreadAttributeList,
        EXTENDED_STARTUPINFO_PRESENT, GetCurrentProcess, GetExitCodeProcess, INFINITE,
        InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
        PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION, STARTF_USESTDHANDLES,
        STARTUPINFOEXW, TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject,
    };
    #[cfg(test)]
    use windows_sys::Win32::System::Threading::{CreateEventW, SetEvent};

    use super::{DirectWorkerAsset, WorkerChild, WorkerChildInner, WorkerCommand};

    static WORKER_ASSET_TAKEN: AtomicBool = AtomicBool::new(false);
    static DIRECT_LAUNCH: Mutex<()> = Mutex::new(());

    pub(super) struct RestrictedChild {
        process: OwnedHandle,
        pub(super) stdin: Option<File>,
        pub(super) stdout: Option<File>,
        status: Option<ExitStatus>,
    }

    impl RestrictedChild {
        pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            if let Some(status) = self.status {
                return Ok(Some(status));
            }
            match wait_for_process(&self.process, 0)? {
                Some(status) => {
                    self.status = Some(status);
                    Ok(Some(status))
                }
                None => Ok(None),
            }
        }

        pub(super) fn kill(&mut self) -> io::Result<()> {
            terminate_process(&self.process)
        }

        pub(super) fn wait(&mut self) -> io::Result<ExitStatus> {
            if let Some(status) = self.status {
                return Ok(status);
            }
            let status = wait_for_process(&self.process, INFINITE)?.ok_or_else(|| {
                io::Error::other("infinite process wait returned without an exit status")
            })?;
            self.status = Some(status);
            Ok(status)
        }
    }

    pub(super) fn prepare(source: &File) -> io::Result<DirectWorkerAsset> {
        let launch_guard = DIRECT_LAUNCH
            .lock()
            .map_err(|_| io::Error::other("the Windows direct-worker launch lock is poisoned"))?;
        let handle = duplicate_inheritable(source)?;
        Ok(DirectWorkerAsset {
            resource_id: handle_resource_id(&handle),
            handle,
            _launch_guard: launch_guard,
        })
    }

    #[allow(
        unsafe_code,
        reason = "the manifest names one validated inherited HANDLE that has no Rust owner yet"
    )]
    pub(super) fn take(resource_id: u64) -> io::Result<File> {
        if WORKER_ASSET_TAKEN.swap(true, Ordering::AcqRel) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "the inherited worker asset was already claimed",
            ));
        }
        let raw = usize::try_from(resource_id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker asset HANDLE identifier is out of range",
            )
        })? as HANDLE;
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker asset HANDLE identifier is invalid",
            ));
        }
        let mut flags = 0_u32;
        // SAFETY: GetHandleInformation validates the non-owning manifest value before ownership is
        // constructed. The process-global one-shot guard prevents a second Rust owner.
        if unsafe { GetHandleInformation(raw, &mut flags) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: The validated HANDLE belongs to this process. Clearing inheritance prevents the
        // worker from leaking the source to any descendant it might create.
        if unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: The HANDLE was validated, is uniquely claimed, and will now be closed by File.
        Ok(File::from(unsafe {
            OwnedHandle::from_raw_handle(raw.cast())
        }))
    }

    pub(super) fn spawn_direct(worker: WorkerCommand) -> io::Result<WorkerChild> {
        let WorkerCommand {
            program,
            current_directory,
            environment,
            direct_asset,
        } = worker;
        let asset = direct_asset.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "direct worker asset is missing",
            )
        })?;
        let (child_stdin, parent_stdin) = create_pipe()?;
        clear_inheritance(&parent_stdin)?;
        let (parent_stdout, child_stdout) = create_pipe()?;
        clear_inheritance(&parent_stdout)?;
        let null_stderr = OpenOptions::new().write(true).open("NUL")?;
        let child_stderr = duplicate_inheritable(&null_stderr)?;
        let handles = [
            raw_handle(&child_stdin),
            raw_handle(&child_stdout),
            raw_handle(&child_stderr),
            raw_handle(&asset.handle),
        ];
        let attributes = AttributeList::new(&handles)?;
        let program_wide = wide_null_terminated(program.as_os_str(), "worker executable")?;
        let mut command_line = quoted_program_command_line(&program_wide)?;
        let directory_wide = current_directory
            .as_deref()
            .map(|directory| wide_null_terminated(directory.as_os_str(), "worker directory"))
            .transpose()?;
        let environment_block = environment_block(environment)?;
        let mut startup = STARTUPINFOEXW::default();
        startup.StartupInfo.cb = u32::try_from(size_of::<STARTUPINFOEXW>()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "STARTUPINFOEXW size is out of range",
            )
        })?;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = handles[0];
        startup.StartupInfo.hStdOutput = handles[1];
        startup.StartupInfo.hStdError = handles[2];
        startup.lpAttributeList = attributes.pointer;
        let current_directory_pointer = directory_wide
            .as_ref()
            .map_or(null(), |directory| directory.as_ptr());
        let process_information = create_process(
            &program_wide,
            &mut command_line,
            &environment_block,
            current_directory_pointer,
            &startup,
        )?;
        let process = owned_created_handle(process_information.hProcess);
        let thread = owned_created_handle(process_information.hThread);
        drop(thread);
        drop(attributes);
        drop(child_stdin);
        drop(child_stdout);
        drop(child_stderr);
        drop(asset);

        Ok(WorkerChild {
            inner: WorkerChildInner::Restricted(RestrictedChild {
                process,
                stdin: Some(File::from(parent_stdin)),
                stdout: Some(File::from(parent_stdout)),
                status: None,
            }),
        })
    }

    struct AttributeList {
        _storage: Vec<usize>,
        pointer: LPPROC_THREAD_ATTRIBUTE_LIST,
    }

    impl AttributeList {
        #[allow(
            unsafe_code,
            reason = "Win32 requires caller-owned aligned storage for the process attribute list"
        )]
        fn new(handles: &[HANDLE]) -> io::Result<Self> {
            let mut byte_length = 0_usize;
            // SAFETY: A null first call is the documented size query; byte_length is valid output.
            let _ =
                unsafe { InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut byte_length) };
            if byte_length == 0 {
                return Err(io::Error::last_os_error());
            }
            let word_length = byte_length.div_ceil(size_of::<usize>());
            let mut storage = vec![0_usize; word_length];
            let pointer = storage.as_mut_ptr().cast();
            // SAFETY: storage is aligned, writable, and at least the queried byte length.
            if unsafe { InitializeProcThreadAttributeList(pointer, 1, 0, &mut byte_length) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let attributes = Self {
                _storage: storage,
                pointer,
            };
            let handle_bytes = std::mem::size_of_val(handles);
            // SAFETY: Both the initialized list and handle slice remain alive through CreateProcess.
            if unsafe {
                UpdateProcThreadAttribute(
                    attributes.pointer,
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                    handles.as_ptr().cast(),
                    handle_bytes,
                    null_mut(),
                    null(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(attributes)
        }
    }

    impl Drop for AttributeList {
        #[allow(
            unsafe_code,
            reason = "every successfully initialized Win32 attribute list must be destroyed"
        )]
        fn drop(&mut self) {
            // SAFETY: pointer names the initialized list owned by this value and is dropped once.
            unsafe { DeleteProcThreadAttributeList(self.pointer) };
        }
    }

    #[allow(
        unsafe_code,
        reason = "CreateProcessW is the required Windows boundary for an exact HANDLE allowlist"
    )]
    fn create_process(
        program: &[u16],
        command_line: &mut [u16],
        environment: &[u16],
        current_directory: *const u16,
        startup: &STARTUPINFOEXW,
    ) -> io::Result<PROCESS_INFORMATION> {
        let mut process_information = PROCESS_INFORMATION::default();
        // SAFETY: All pointers name live, correctly terminated buffers; STARTUPINFOEXW embeds the
        // STARTUPINFOW prefix expected by CreateProcessW and owns a live attribute list.
        let succeeded = unsafe {
            CreateProcessW(
                program.as_ptr(),
                command_line.as_mut_ptr(),
                null(),
                null(),
                1,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
                environment.as_ptr().cast(),
                current_directory,
                std::ptr::from_ref(&startup.StartupInfo),
                &mut process_information,
            )
        };
        if succeeded == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(process_information)
        }
    }

    #[allow(
        unsafe_code,
        reason = "CreatePipe returns two new owned HANDLE values through output pointers"
    )]
    fn create_pipe() -> io::Result<(OwnedHandle, OwnedHandle)> {
        let mut attributes = inheritable_security_attributes();
        let mut read = null_mut();
        let mut write = null_mut();
        // SAFETY: Output pointers and SECURITY_ATTRIBUTES are initialized and valid for the call.
        if unsafe { CreatePipe(&mut read, &mut write, &mut attributes, 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((owned_created_handle(read), owned_created_handle(write)))
    }

    #[allow(
        unsafe_code,
        reason = "DuplicateHandle creates one independently owned inheritable duplicate"
    )]
    fn duplicate_inheritable(source: &impl AsRawHandle) -> io::Result<OwnedHandle> {
        // SAFETY: GetCurrentProcess returns a pseudo handle used only as DuplicateHandle context.
        let process = unsafe { GetCurrentProcess() };
        let mut duplicate = null_mut();
        // SAFETY: source is live, duplicate is a valid output pointer, and same-process duplication
        // with DUPLICATE_SAME_ACCESS preserves the source access mask.
        if unsafe {
            DuplicateHandle(
                process,
                source.as_raw_handle().cast(),
                process,
                &mut duplicate,
                0,
                1,
                DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(owned_created_handle(duplicate))
    }

    #[allow(
        unsafe_code,
        reason = "SetHandleInformation narrows inheritance on one live parent pipe HANDLE"
    )]
    fn clear_inheritance(handle: &OwnedHandle) -> io::Result<()> {
        // SAFETY: handle is live and owned for the duration of this call.
        if unsafe { SetHandleInformation(raw_handle(handle), HANDLE_FLAG_INHERIT, 0) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[allow(
        unsafe_code,
        reason = "WaitForSingleObject and GetExitCodeProcess observe one owned process HANDLE"
    )]
    fn wait_for_process(process: &OwnedHandle, timeout: u32) -> io::Result<Option<ExitStatus>> {
        // SAFETY: process is a live owned process HANDLE.
        match unsafe { WaitForSingleObject(raw_handle(process), timeout) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0_u32;
                // SAFETY: process is signaled and code is a valid output pointer.
                if unsafe { GetExitCodeProcess(raw_handle(process), &mut code) } == 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(Some(ExitStatus::from_raw(code)))
                }
            }
            _ => Err(io::Error::last_os_error()),
        }
    }

    #[allow(
        unsafe_code,
        reason = "TerminateProcess is the Windows hard-stop primitive for the owned worker process"
    )]
    fn terminate_process(process: &OwnedHandle) -> io::Result<()> {
        // SAFETY: process is a live owned process HANDLE; the supervisor always follows with wait.
        if unsafe { TerminateProcess(raw_handle(process), 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn inheritable_security_attributes() -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: u32::try_from(size_of::<SECURITY_ATTRIBUTES>())
                .expect("SECURITY_ATTRIBUTES size must fit u32"),
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        }
    }

    fn wide_null_terminated(value: &std::ffi::OsStr, field: &'static str) -> io::Result<Vec<u16>> {
        let mut wide = value.encode_wide().collect::<Vec<_>>();
        if wide.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{field} contains an interior NUL"),
            ));
        }
        wide.push(0);
        Ok(wide)
    }

    fn quoted_program_command_line(program: &[u16]) -> io::Result<Vec<u16>> {
        let program = program.strip_suffix(&[0]).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker executable is not terminated",
            )
        })?;
        if program.contains(&(b'"' as u16)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker executable contains an invalid quote",
            ));
        }
        let mut command_line = Vec::with_capacity(program.len() + 3);
        command_line.push(b'"' as u16);
        command_line.extend_from_slice(program);
        command_line.push(b'"' as u16);
        command_line.push(0);
        Ok(command_line)
    }

    fn environment_block(environment: Vec<(OsString, OsString)>) -> io::Result<Vec<u16>> {
        let mut entries = environment
            .into_iter()
            .map(|(key, value)| {
                let key_text = key.to_string_lossy();
                if key_text.is_empty()
                    || !key_text.is_ascii()
                    || key_text.contains('=')
                    || key.encode_wide().any(|unit| unit == 0)
                    || value.encode_wide().any(|unit| unit == 0)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "worker environment entry is invalid",
                    ));
                }
                Ok((key_text.to_ascii_uppercase(), key, value))
            })
            .collect::<io::Result<Vec<_>>>()?;
        entries.sort_by(|left, right| {
            let ordering = left.0.cmp(&right.0);
            if ordering == Comparison::Equal {
                left.1.cmp(&right.1)
            } else {
                ordering
            }
        });
        let mut block = Vec::new();
        for (_, key, value) in entries {
            block.extend(key.encode_wide());
            block.push(b'=' as u16);
            block.extend(value.encode_wide());
            block.push(0);
        }
        block.push(0);
        if block.len() == 1 {
            block.push(0);
        }
        Ok(block)
    }

    fn raw_handle(handle: &OwnedHandle) -> HANDLE {
        handle.as_raw_handle().cast()
    }

    fn handle_resource_id(handle: &OwnedHandle) -> u64 {
        raw_handle(handle) as usize as u64
    }

    #[allow(
        unsafe_code,
        reason = "a successful Win32 creation call transfers one raw HANDLE into OwnedHandle"
    )]
    fn owned_created_handle(raw: HANDLE) -> OwnedHandle {
        debug_assert!(!raw.is_null() && raw != INVALID_HANDLE_VALUE);
        // SAFETY: The caller passes a newly created HANDLE and transfers unique close ownership.
        unsafe { OwnedHandle::from_raw_handle(raw.cast()) }
    }

    #[cfg(test)]
    #[allow(
        unsafe_code,
        reason = "test-only inheritable event proves an unrelated HANDLE is excluded"
    )]
    pub(super) fn prepare_inheritable_sentinel() -> io::Result<OwnedHandle> {
        let mut attributes = inheritable_security_attributes();
        // SAFETY: SECURITY_ATTRIBUTES is initialized; the unnamed event is returned uniquely owned.
        let event = unsafe { CreateEventW(&mut attributes, 1, 0, null()) };
        if event.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(owned_created_handle(event))
        }
    }

    #[cfg(test)]
    pub(super) fn sentinel_resource_id(event: &OwnedHandle) -> u64 {
        handle_resource_id(event)
    }

    #[cfg(test)]
    #[allow(
        unsafe_code,
        reason = "test-only SetEvent probe does not take ownership of the manifest-named sentinel"
    )]
    pub(super) fn inherited_sentinel_is_absent(resource_id: u64) -> bool {
        let Ok(raw) = usize::try_from(resource_id) else {
            return true;
        };
        let raw = raw as HANDLE;
        // SAFETY: SetEvent validates the non-owning test value; success means the sentinel leaked.
        (unsafe { SetEvent(raw) }) == 0
    }

    #[cfg(test)]
    #[allow(
        unsafe_code,
        reason = "test-only zero-time wait verifies the parent sentinel remains unsignaled"
    )]
    pub(super) fn sentinel_remains_unsignaled(event: &OwnedHandle) -> bool {
        // SAFETY: event is a live owned event HANDLE.
        (unsafe { WaitForSingleObject(raw_handle(event), 0) }) == WAIT_TIMEOUT
    }
}

#[cfg(all(test, any(unix, windows)))]
mod tests {
    use std::io::{Read, Write};

    use super::*;

    const CHILD_ASSET_ID: &str = "PARTPROBE_TEST_DIRECT_ASSET_ID";
    const CHILD_SENTINEL_ID: &str = "PARTPROBE_TEST_INHERITABLE_SENTINEL_ID";

    #[test]
    fn direct_asset_allowlists_the_source_and_excludes_an_inheritable_sentinel() {
        if let Some(asset_id) = std::env::var_os(CHILD_ASSET_ID) {
            let asset_id = asset_id
                .to_string_lossy()
                .parse::<u64>()
                .expect("child asset resource ID must parse");
            let sentinel_id = std::env::var(CHILD_SENTINEL_ID)
                .expect("child sentinel resource ID must exist")
                .parse::<u64>()
                .expect("child sentinel resource ID must parse");
            let mut asset = take_inherited_worker_asset(asset_id)
                .expect("allowlisted asset resource must be inherited");
            let mut contents = String::new();
            asset
                .read_to_string(&mut contents)
                .expect("allowlisted asset must be readable");
            assert!(contents.contains("partprobe-platform"));
            assert!(inherited_sentinel_is_absent(sentinel_id));
            assert!(std::env::var_os("PATH").is_none());
            assert_eq!(
                std::fs::canonicalize(std::env::current_dir().expect("child cwd must resolve"))
                    .expect("child cwd must canonicalize"),
                std::fs::canonicalize(std::env::temp_dir())
                    .expect("temp directory must canonicalize")
            );
            let mut control = String::new();
            std::io::stdin()
                .read_to_string(&mut control)
                .expect("child stdin must be readable");
            assert_eq!(control, "partprobe-control");
            std::io::stdout()
                .write_all(b"partprobe-ready")
                .expect("child stdout must be writable");
            return;
        }

        let source = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .expect("platform manifest must open");
        let sentinel = prepare_inheritable_sentinel(&source)
            .expect("inheritable sentinel resource must be created");
        let sentinel_id = sentinel_resource_id(&sentinel);
        let direct = prepare_direct_worker_asset(&source).expect("direct asset must prepare");
        let asset_id = direct.resource_id();
        let mut command =
            WorkerCommand::new(std::env::current_exe().expect("test path must resolve"));
        command
            .current_dir(std::env::temp_dir())
            .env(CHILD_ASSET_ID, asset_id.to_string())
            .env(CHILD_SENTINEL_ID, sentinel_id.to_string())
            .direct_asset(direct);

        let mut child = command.spawn().expect("probe child must launch");
        let mut stdin = child.take_stdin().expect("probe stdin must exist");
        stdin
            .write_all(b"partprobe-control")
            .expect("probe stdin must accept control bytes");
        drop(stdin);
        let mut output = Vec::new();
        child
            .take_stdout()
            .expect("probe stdout must exist")
            .read_to_end(&mut output)
            .expect("probe stdout must drain");
        let status = child.wait().expect("probe child must be reaped");

        assert!(status.success(), "probe child must verify the resource set");
        assert!(
            output
                .windows(b"partprobe-ready".len())
                .any(|window| window == b"partprobe-ready"),
            "probe child must write its readiness token"
        );
        assert_parent_sentinel_unsignaled(&sentinel);
    }

    #[cfg(unix)]
    fn prepare_inheritable_sentinel(source: &File) -> io::Result<std::os::fd::OwnedFd> {
        unix::prepare_inheritable_sentinel(source)
    }

    #[cfg(windows)]
    fn prepare_inheritable_sentinel(
        _source: &File,
    ) -> io::Result<std::os::windows::io::OwnedHandle> {
        windows::prepare_inheritable_sentinel()
    }

    #[cfg(unix)]
    fn sentinel_resource_id(sentinel: &std::os::fd::OwnedFd) -> u64 {
        unix::sentinel_resource_id(sentinel)
    }

    #[cfg(windows)]
    fn sentinel_resource_id(sentinel: &std::os::windows::io::OwnedHandle) -> u64 {
        windows::sentinel_resource_id(sentinel)
    }

    #[cfg(unix)]
    fn inherited_sentinel_is_absent(resource_id: u64) -> bool {
        unix::inherited_sentinel_is_absent(resource_id)
    }

    #[cfg(windows)]
    fn inherited_sentinel_is_absent(resource_id: u64) -> bool {
        windows::inherited_sentinel_is_absent(resource_id)
    }

    #[cfg(unix)]
    fn assert_parent_sentinel_unsignaled(_sentinel: &std::os::fd::OwnedFd) {}

    #[cfg(windows)]
    fn assert_parent_sentinel_unsignaled(sentinel: &std::os::windows::io::OwnedHandle) {
        assert!(
            windows::sentinel_remains_unsignaled(sentinel),
            "excluded sentinel event must remain unsignaled in the parent"
        );
    }
}
