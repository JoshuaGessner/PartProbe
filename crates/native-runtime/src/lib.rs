#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const MANIFEST_NAME: &str = "partprobe-native-runtime.json";
const RUNTIME_KIND: &str = "partprobe_developer_native_runtime";
const SUPPORT_STATUS: &str = "internal_developer_checkpoint";
const EXPECTED_OCCT_VERSION: &str = "8.0.0";
const EXPECTED_OCCT_COMMIT: &str = "d3056ef80c9668f395da40f5fd7be186cae4501f";
const EXPECTED_OCCT_TREE: &str = "b3ffb8a91468845b63675057957209032b5806b1";
const VERSION_HEADER_PATH: &str = "include/opencascade/Standard_Version.hxx";
const BUILD_MANIFEST_PATH: &str = "provenance/partprobe-occt-build-manifest.json";
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_ARTIFACTS: usize = 4096;
const MAX_DIRECTORIES: usize = 64;
const REQUIRED_LIBRARIES: [&str; 13] = [
    "TKDESTEP",
    "TKXSBase",
    "TKShHealing",
    "TKMesh",
    "TKTopAlgo",
    "TKPrim",
    "TKBRep",
    "TKGeomAlgo",
    "TKGeomBase",
    "TKG3d",
    "TKG2d",
    "TKMath",
    "TKernel",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeRuntimeError {
    RootUnavailable,
    ManifestUnreadable,
    ManifestInvalid,
    UnsupportedHost,
    ArtifactInvalid,
    ProvenanceInvalid,
}

impl fmt::Display for NativeRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RootUnavailable => "native runtime root is unavailable",
            Self::ManifestUnreadable => "native runtime manifest is unreadable",
            Self::ManifestInvalid => "native runtime manifest is invalid",
            Self::UnsupportedHost => "native runtime does not match this host",
            Self::ArtifactInvalid => "native runtime artifact verification failed",
            Self::ProvenanceInvalid => "native runtime provenance is invalid",
        })
    }
}

impl std::error::Error for NativeRuntimeError {}

#[derive(Debug)]
pub struct VerifiedNativeRuntime {
    root: PathBuf,
    worker_executable: PathBuf,
    native_library_directory: PathBuf,
}

impl VerifiedNativeRuntime {
    pub fn verify(runtime_root: impl AsRef<Path>) -> Result<Self, NativeRuntimeError> {
        let root = runtime_root
            .as_ref()
            .canonicalize()
            .map_err(|_| NativeRuntimeError::RootUnavailable)?;
        if !root.is_dir() {
            return Err(NativeRuntimeError::RootUnavailable);
        }
        let manifest_path = root.join(MANIFEST_NAME);
        let metadata = fs::symlink_metadata(&manifest_path)
            .map_err(|_| NativeRuntimeError::ManifestUnreadable)?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_MANIFEST_BYTES {
            return Err(NativeRuntimeError::ManifestUnreadable);
        }
        let manifest_bytes =
            fs::read(&manifest_path).map_err(|_| NativeRuntimeError::ManifestUnreadable)?;
        let manifest: RuntimeManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| NativeRuntimeError::ManifestInvalid)?;
        manifest.validate_header()?;

        let worker_relative = validate_relative_path(&manifest.worker.path)?;
        let expected_worker = if host_platform() == "windows" {
            "bin/partprobe-geometry-worker.exe"
        } else {
            "bin/partprobe-geometry-worker"
        };
        if manifest.worker.path != expected_worker
            || manifest.version_header.path != VERSION_HEADER_PATH
            || manifest.build_provenance.path != BUILD_MANIFEST_PATH
            || manifest.configuration.worker != manifest.worker.path
            || manifest.configuration.occt_root != "."
            || manifest.configuration.workspace != "external_directory_required"
        {
            return Err(NativeRuntimeError::ManifestInvalid);
        }

        let mut expected_paths = BTreeSet::from([MANIFEST_NAME.to_owned()]);
        verify_artifact(&root, &manifest.worker, &mut expected_paths)?;
        verify_artifact(&root, &manifest.version_header, &mut expected_paths)?;
        verify_artifact(&root, &manifest.build_provenance, &mut expected_paths)?;
        for required in REQUIRED_LIBRARIES {
            if !manifest.libraries.contains_key(required) {
                return Err(NativeRuntimeError::ManifestInvalid);
            }
        }
        for (family, entries) in &manifest.libraries {
            if !valid_library_family(family) || entries.is_empty() {
                return Err(NativeRuntimeError::ManifestInvalid);
            }
            for entry in entries {
                if !entry_matches_library_family(entry, family) {
                    return Err(NativeRuntimeError::ManifestInvalid);
                }
                verify_artifact(&root, entry, &mut expected_paths)?;
            }
        }

        let actual_paths = collect_runtime_artifacts(&root)?;
        if actual_paths != expected_paths {
            return Err(NativeRuntimeError::ArtifactInvalid);
        }

        let worker_executable = root.join(worker_relative);
        verify_worker_executable(&worker_executable)?;
        validate_build_provenance(&root.join(BUILD_MANIFEST_PATH), &manifest)?;
        validate_install_fingerprint(&root, &manifest)?;
        let native_library_directory = root.join(if host_platform() == "windows" {
            "bin"
        } else {
            "lib"
        });
        if !native_library_directory.is_dir() {
            return Err(NativeRuntimeError::ArtifactInvalid);
        }
        Ok(Self {
            root,
            worker_executable,
            native_library_directory,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn worker_executable(&self) -> &Path {
        &self.worker_executable
    }

    #[must_use]
    pub fn native_library_directory(&self) -> &Path {
        &self.native_library_directory
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeManifest {
    schema_version: u64,
    kind: String,
    support_status: String,
    platform: String,
    machine: String,
    source_policy: SourcePolicy,
    occt_install_fingerprint: InstallFingerprint,
    worker: Artifact,
    version_header: Artifact,
    build_provenance: Artifact,
    libraries: BTreeMap<String, Vec<Artifact>>,
    configuration: RuntimeConfiguration,
}

impl RuntimeManifest {
    fn validate_header(&self) -> Result<(), NativeRuntimeError> {
        if self.schema_version != 1
            || self.kind != RUNTIME_KIND
            || self.support_status != SUPPORT_STATUS
            || self.source_policy.occt_version != EXPECTED_OCCT_VERSION
            || self.source_policy.occt_commit != EXPECTED_OCCT_COMMIT
            || self.source_policy.occt_tree != EXPECTED_OCCT_TREE
        {
            return Err(NativeRuntimeError::ManifestInvalid);
        }
        if self.platform != host_platform() || !machines_match(&self.machine, host_machine()) {
            return Err(NativeRuntimeError::UnsupportedHost);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePolicy {
    occt_version: String,
    occt_commit: String,
    occt_tree: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeConfiguration {
    #[serde(rename = "PARTPROBE_GEOMETRY_WORKER")]
    worker: String,
    #[serde(rename = "PARTPROBE_OCCT_ROOT")]
    occt_root: String,
    #[serde(rename = "PARTPROBE_GEOMETRY_WORKSPACE")]
    workspace: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstallFingerprint {
    schema_version: u64,
    occt_version: String,
    platform: String,
    machine: String,
    version_header_sha256: String,
    libraries: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ArtifactKind {
    File,
    Symlink,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    path: String,
    #[serde(rename = "type")]
    kind: ArtifactKind,
    size_bytes: Option<u64>,
    sha256: Option<String>,
    target: Option<String>,
}

fn validate_relative_path(value: &str) -> Result<PathBuf, NativeRuntimeError> {
    if value.is_empty()
        || value.len() > 512
        || value.contains('\\')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(NativeRuntimeError::ManifestInvalid);
    }
    let path = PathBuf::from(value);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(NativeRuntimeError::ManifestInvalid);
    }
    Ok(path)
}

fn verify_artifact(
    root: &Path,
    entry: &Artifact,
    expected_paths: &mut BTreeSet<String>,
) -> Result<(), NativeRuntimeError> {
    let relative = validate_relative_path(&entry.path)?;
    if !expected_paths.insert(entry.path.clone()) {
        return Err(NativeRuntimeError::ManifestInvalid);
    }
    let artifact = root.join(relative);
    let metadata =
        fs::symlink_metadata(&artifact).map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
    match entry.kind {
        ArtifactKind::File => {
            if !metadata.file_type().is_file()
                || entry.target.is_some()
                || entry.size_bytes != Some(metadata.len())
            {
                return Err(NativeRuntimeError::ArtifactInvalid);
            }
            let actual_sha256 = sha256(&artifact)?;
            if entry.sha256.as_deref() != Some(actual_sha256.as_str()) {
                return Err(NativeRuntimeError::ArtifactInvalid);
            }
        }
        ArtifactKind::Symlink => {
            if !metadata.file_type().is_symlink()
                || entry.size_bytes.is_some()
                || entry.sha256.is_some()
            {
                return Err(NativeRuntimeError::ArtifactInvalid);
            }
            let target = entry
                .target
                .as_deref()
                .ok_or(NativeRuntimeError::ManifestInvalid)?;
            if target.contains('/') || target.contains('\\') || target.is_empty() {
                return Err(NativeRuntimeError::ManifestInvalid);
            }
            let actual_target =
                fs::read_link(&artifact).map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
            if actual_target != Path::new(target) {
                return Err(NativeRuntimeError::ArtifactInvalid);
            }
            let resolved = artifact
                .canonicalize()
                .map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
            let parent = artifact
                .parent()
                .ok_or(NativeRuntimeError::ArtifactInvalid)?
                .canonicalize()
                .map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
            if resolved.parent() != Some(parent.as_path()) {
                return Err(NativeRuntimeError::ArtifactInvalid);
            }
        }
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<String, NativeRuntimeError> {
    let mut file = File::open(path).map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    let mut result = String::with_capacity(64);
    for byte in digest.finalize() {
        use std::fmt::Write as _;

        write!(&mut result, "{byte:02x}").expect("writing to String cannot fail");
    }
    Ok(result)
}

fn collect_runtime_artifacts(root: &Path) -> Result<BTreeSet<String>, NativeRuntimeError> {
    let mut result = BTreeSet::new();
    let mut directories = vec![root.to_path_buf()];
    let mut visited_directories = 0_usize;
    while let Some(directory) = directories.pop() {
        visited_directories += 1;
        if visited_directories > MAX_DIRECTORIES {
            return Err(NativeRuntimeError::ArtifactInvalid);
        }
        collect_directory(root, &directory, &mut result, &mut directories)?;
    }
    Ok(result)
}

fn collect_directory(
    root: &Path,
    directory: &Path,
    result: &mut BTreeSet<String>,
    directories: &mut Vec<PathBuf>,
) -> Result<(), NativeRuntimeError> {
    for entry in fs::read_dir(directory).map_err(|_| NativeRuntimeError::ArtifactInvalid)? {
        let entry = entry.map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
        if metadata.file_type().is_dir() {
            directories.push(path);
        } else if metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
            let portable = relative.to_string_lossy().replace('\\', "/");
            if !result.insert(portable) || result.len() > MAX_ARTIFACTS {
                return Err(NativeRuntimeError::ArtifactInvalid);
            }
        } else {
            return Err(NativeRuntimeError::ArtifactInvalid);
        }
    }
    Ok(())
}

fn valid_library_family(family: &str) -> bool {
    family.starts_with("TK")
        && family.len() <= 64
        && family
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn entry_matches_library_family(entry: &Artifact, family: &str) -> bool {
    let Ok(path) = validate_relative_path(&entry.path) else {
        return false;
    };
    let expected_parent = if host_platform() == "windows" {
        Path::new("bin")
    } else {
        Path::new("lib")
    };
    if path.parent() != Some(expected_parent) {
        return false;
    }
    let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if host_platform() == "windows" {
        filename.eq_ignore_ascii_case(&format!("{family}.dll"))
    } else {
        filename.starts_with(&format!("lib{family}."))
    }
}

fn validate_build_provenance(
    path: &Path,
    runtime: &RuntimeManifest,
) -> Result<(), NativeRuntimeError> {
    let metadata = fs::metadata(path).map_err(|_| NativeRuntimeError::ProvenanceInvalid)?;
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(NativeRuntimeError::ProvenanceInvalid);
    }
    let bytes = fs::read(path).map_err(|_| NativeRuntimeError::ProvenanceInvalid)?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| NativeRuntimeError::ProvenanceInvalid)?;
    let expected = [
        ("schema_version", serde_json::json!(1)),
        ("occt_version", serde_json::json!(EXPECTED_OCCT_VERSION)),
        ("occt_commit", serde_json::json!(EXPECTED_OCCT_COMMIT)),
        ("occt_tree", serde_json::json!(EXPECTED_OCCT_TREE)),
        ("build_type", serde_json::json!("Release")),
        ("library_type", serde_json::json!("Shared")),
    ];
    if expected
        .iter()
        .any(|(key, expected)| value.get(key) != Some(expected))
        || value.get("platform").and_then(serde_json::Value::as_str)
            != Some(runtime.platform.as_str())
        || (runtime.platform == "windows"
            && value
                .get("cmake_generator_platform")
                .and_then(serde_json::Value::as_str)
                != Some("x64"))
        || !value
            .get("machine")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|machine| machines_match(machine, &runtime.machine))
    {
        return Err(NativeRuntimeError::ProvenanceInvalid);
    }
    Ok(())
}

fn validate_install_fingerprint(
    root: &Path,
    runtime: &RuntimeManifest,
) -> Result<(), NativeRuntimeError> {
    let fingerprint = &runtime.occt_install_fingerprint;
    if fingerprint.schema_version != 1
        || fingerprint.occt_version != EXPECTED_OCCT_VERSION
        || fingerprint.platform != runtime.platform
        || !machines_match(&fingerprint.machine, &runtime.machine)
        || fingerprint.version_header_sha256 != sha256(&root.join(VERSION_HEADER_PATH))?
        || fingerprint.libraries.len() != REQUIRED_LIBRARIES.len()
    {
        return Err(NativeRuntimeError::ProvenanceInvalid);
    }
    for family in REQUIRED_LIBRARIES {
        let expected = fingerprint
            .libraries
            .get(family)
            .ok_or(NativeRuntimeError::ProvenanceInvalid)?;
        let filename = match host_platform() {
            "darwin" => format!("lib{family}.dylib"),
            "linux" => format!("lib{family}.so"),
            "windows" => format!("{family}.dll"),
            _ => return Err(NativeRuntimeError::UnsupportedHost),
        };
        let directory = if host_platform() == "windows" {
            "bin"
        } else {
            "lib"
        };
        if expected != &sha256(&root.join(directory).join(filename))? {
            return Err(NativeRuntimeError::ProvenanceInvalid);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn verify_worker_executable(path: &Path) -> Result<(), NativeRuntimeError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path).map_err(|_| NativeRuntimeError::ArtifactInvalid)?;
    if metadata.permissions().mode() & 0o100 == 0 {
        return Err(NativeRuntimeError::ArtifactInvalid);
    }
    Ok(())
}

#[cfg(windows)]
fn verify_worker_executable(path: &Path) -> Result<(), NativeRuntimeError> {
    if !path.is_file() {
        return Err(NativeRuntimeError::ArtifactInvalid);
    }
    Ok(())
}

fn host_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        value => value,
    }
}

fn host_machine() -> &'static str {
    std::env::consts::ARCH
}

fn machines_match(left: &str, right: &str) -> bool {
    normalize_machine(left) == normalize_machine(right)
}

fn normalize_machine(machine: &str) -> &str {
    match machine.to_ascii_lowercase().as_str() {
        "arm64" | "aarch64" => "aarch64",
        "amd64" | "x64" | "x86_64" => "x86_64",
        _ => machine,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::{Value, json};

    use super::*;

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    struct TempRuntime {
        root: PathBuf,
    }

    impl TempRuntime {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "partprobe-native-runtime-test-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).expect("unique runtime root");
            Self { root }
        }

        fn manifest_path(&self) -> PathBuf {
            self.root.join(MANIFEST_NAME)
        }

        fn read_manifest(&self) -> Value {
            serde_json::from_slice(&fs::read(self.manifest_path()).expect("manifest bytes"))
                .expect("manifest JSON")
        }

        fn write_manifest(&self, value: &Value) {
            fs::write(
                self.manifest_path(),
                serde_json::to_vec_pretty(value).expect("manifest JSON"),
            )
            .expect("write manifest");
        }
    }

    impl Drop for TempRuntime {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("remove owned test runtime");
        }
    }

    fn create_file(root: &Path, relative: &str, content: &[u8]) -> Artifact {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("artifact parent")).expect("create parent");
        fs::write(&path, content).expect("write artifact");
        Artifact {
            path: relative.to_owned(),
            kind: ArtifactKind::File,
            size_bytes: Some(content.len() as u64),
            sha256: Some(sha256(&path).expect("artifact hash")),
            target: None,
        }
    }

    fn artifact_json(artifact: &Artifact) -> Value {
        json!({
            "path": artifact.path,
            "type": "file",
            "size_bytes": artifact.size_bytes,
            "sha256": artifact.sha256,
        })
    }

    fn valid_runtime() -> TempRuntime {
        let runtime = TempRuntime::new();
        let worker_path = if host_platform() == "windows" {
            "bin/partprobe-geometry-worker.exe"
        } else {
            "bin/partprobe-geometry-worker"
        };
        let worker = create_file(&runtime.root, worker_path, b"native-worker");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(
                runtime.root.join(worker_path),
                fs::Permissions::from_mode(0o755),
            )
            .expect("worker executable mode");
        }
        let header = create_file(
            &runtime.root,
            VERSION_HEADER_PATH,
            b"#define OCC_VERSION_COMPLETE \"8.0.0\"\n",
        );
        let build_value = json!({
            "schema_version": 1,
            "occt_version": EXPECTED_OCCT_VERSION,
            "occt_commit": EXPECTED_OCCT_COMMIT,
            "occt_tree": EXPECTED_OCCT_TREE,
            "platform": host_platform(),
            "machine": host_machine(),
            "build_type": "Release",
            "library_type": "Shared",
            "cmake_generator_platform": if host_platform() == "windows" { "x64" } else { "" }
        });
        let build_bytes = serde_json::to_vec_pretty(&build_value).expect("build JSON");
        let build = create_file(&runtime.root, BUILD_MANIFEST_PATH, &build_bytes);

        let mut libraries = serde_json::Map::new();
        let mut fingerprints = serde_json::Map::new();
        for family in REQUIRED_LIBRARIES {
            let filename = match host_platform() {
                "darwin" => format!("lib{family}.dylib"),
                "linux" => format!("lib{family}.so"),
                "windows" => format!("{family}.dll"),
                _ => panic!("test host must be supported"),
            };
            let directory = if host_platform() == "windows" {
                "bin"
            } else {
                "lib"
            };
            let relative = format!("{directory}/{filename}");
            let artifact = create_file(&runtime.root, &relative, family.as_bytes());
            fingerprints.insert(
                family.to_owned(),
                Value::String(artifact.sha256.clone().expect("library hash")),
            );
            libraries.insert(family.to_owned(), json!([artifact_json(&artifact)]));
        }
        let manifest = json!({
            "schema_version": 1,
            "kind": RUNTIME_KIND,
            "support_status": SUPPORT_STATUS,
            "platform": host_platform(),
            "machine": host_machine(),
            "source_policy": {
                "occt_version": EXPECTED_OCCT_VERSION,
                "occt_commit": EXPECTED_OCCT_COMMIT,
                "occt_tree": EXPECTED_OCCT_TREE
            },
            "occt_install_fingerprint": {
                "schema_version": 1,
                "occt_version": EXPECTED_OCCT_VERSION,
                "platform": host_platform(),
                "machine": host_machine(),
                "version_header_sha256": header.sha256,
                "libraries": fingerprints
            },
            "worker": artifact_json(&worker),
            "version_header": artifact_json(&header),
            "build_provenance": artifact_json(&build),
            "libraries": libraries,
            "configuration": {
                "PARTPROBE_GEOMETRY_WORKER": worker_path,
                "PARTPROBE_OCCT_ROOT": ".",
                "PARTPROBE_GEOMETRY_WORKSPACE": "external_directory_required"
            }
        });
        runtime.write_manifest(&manifest);
        runtime
    }

    #[test]
    fn valid_runtime_resolves_reviewed_worker_and_library_paths() {
        let runtime = valid_runtime();

        let verified = VerifiedNativeRuntime::verify(&runtime.root).expect("verified runtime");
        let canonical_root = runtime.root.canonicalize().unwrap();

        assert_eq!(verified.root(), canonical_root);
        assert_eq!(
            verified.worker_executable(),
            canonical_root.join(if host_platform() == "windows" {
                "bin/partprobe-geometry-worker.exe"
            } else {
                "bin/partprobe-geometry-worker"
            })
        );
        assert_eq!(
            verified.native_library_directory(),
            canonical_root.join(if host_platform() == "windows" {
                "bin"
            } else {
                "lib"
            })
        );
    }

    #[test]
    fn changed_worker_and_unmanifested_file_fail_closed() {
        let runtime = valid_runtime();
        let worker = runtime.root.join(if host_platform() == "windows" {
            "bin/partprobe-geometry-worker.exe"
        } else {
            "bin/partprobe-geometry-worker"
        });
        fs::write(worker, b"tampered-worker").unwrap();
        assert_eq!(
            VerifiedNativeRuntime::verify(&runtime.root).unwrap_err(),
            NativeRuntimeError::ArtifactInvalid
        );

        let runtime = valid_runtime();
        let unmanifested_directory = if host_platform() == "windows" {
            "bin"
        } else {
            "lib"
        };
        fs::write(
            runtime
                .root
                .join(unmanifested_directory)
                .join("unmanifested.bin"),
            b"extra",
        )
        .unwrap();
        assert_eq!(
            VerifiedNativeRuntime::verify(&runtime.root).unwrap_err(),
            NativeRuntimeError::ArtifactInvalid
        );
    }

    #[test]
    fn traversal_and_unreviewed_launch_configuration_fail_closed() {
        let runtime = valid_runtime();
        let mut manifest = runtime.read_manifest();
        manifest["worker"]["path"] = json!("../partprobe-geometry-worker");
        manifest["configuration"]["PARTPROBE_GEOMETRY_WORKER"] =
            json!("../partprobe-geometry-worker");
        runtime.write_manifest(&manifest);

        assert_eq!(
            VerifiedNativeRuntime::verify(&runtime.root).unwrap_err(),
            NativeRuntimeError::ManifestInvalid
        );
    }

    #[test]
    fn wrong_build_provenance_fails_after_artifact_verification() {
        let runtime = valid_runtime();
        let build_path = runtime.root.join(BUILD_MANIFEST_PATH);
        let mut build: Value = serde_json::from_slice(&fs::read(&build_path).unwrap()).unwrap();
        build["occt_tree"] = json!("0000000000000000000000000000000000000000");
        let build_bytes = serde_json::to_vec_pretty(&build).unwrap();
        fs::write(&build_path, &build_bytes).unwrap();
        let mut manifest = runtime.read_manifest();
        manifest["build_provenance"]["size_bytes"] = json!(build_bytes.len());
        manifest["build_provenance"]["sha256"] = json!(sha256(&build_path).unwrap());
        runtime.write_manifest(&manifest);

        assert_eq!(
            VerifiedNativeRuntime::verify(&runtime.root).unwrap_err(),
            NativeRuntimeError::ProvenanceInvalid
        );
    }

    #[test]
    fn runtime_for_another_platform_fails_closed() {
        let runtime = valid_runtime();
        let mut manifest = runtime.read_manifest();
        manifest["platform"] = json!("unsupported");
        runtime.write_manifest(&manifest);

        assert_eq!(
            VerifiedNativeRuntime::verify(&runtime.root).unwrap_err(),
            NativeRuntimeError::UnsupportedHost
        );
    }
}
