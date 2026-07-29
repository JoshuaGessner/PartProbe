//! Optional, narrow C ABI boundary for the out-of-process OCCT adapter.

/// Native adapter ABI version implemented by this crate.
pub const OCCT_ADAPTER_ABI_VERSION: u32 = 1;

/// Returns whether this build contains the optional OCCT bridge.
#[must_use]
pub const fn native_occt_enabled() -> bool {
    cfg!(feature = "native-occt")
}

/// Sanitized basic properties returned by the native adapter.
#[cfg(feature = "native-occt")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeBasicProperties {
    /// Number of STEP roots transferred.
    pub transferred_roots: u64,
    /// Number of solid bodies found in the translated shape.
    pub solid_body_count: u64,
    /// Surface area in square millimetres.
    pub surface_area_mm2: f64,
    /// Enclosed volume in cubic millimetres.
    pub enclosed_volume_mm3: f64,
    /// Center of mass in millimetres.
    pub center_of_mass_mm: [f64; 3],
}

/// Sanitized adapter failure without native exception text or source paths.
#[cfg(feature = "native-occt")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeAdapterError {
    diagnostic_code: &'static str,
}

#[cfg(feature = "native-occt")]
impl NativeAdapterError {
    /// Returns a stable, content-free diagnostic code.
    #[must_use]
    pub const fn diagnostic_code(self) -> &'static str {
        self.diagnostic_code
    }
}

#[cfg(feature = "native-occt")]
#[allow(unsafe_code)]
mod native {
    use std::ffi::{CStr, CString, c_char, c_int};
    use std::mem::size_of;
    use std::path::Path;

    use super::{NativeAdapterError, NativeBasicProperties, OCCT_ADAPTER_ABI_VERSION};

    const DIAGNOSTIC_CAPACITY: usize = 64;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NativeResult {
        abi_version: u32,
        transferred_roots: u64,
        solid_body_count: u64,
        surface_area_mm2: f64,
        enclosed_volume_mm3: f64,
        center_of_mass_x_mm: f64,
        center_of_mass_y_mm: f64,
        center_of_mass_z_mm: f64,
        diagnostic_code: [c_char; DIAGNOSTIC_CAPACITY],
    }

    unsafe extern "C" {
        fn partprobe_occt_abi_version() -> u32;
        fn partprobe_occt_analyze_step(
            path: *const c_char,
            result: *mut NativeResult,
            result_size: usize,
        ) -> c_int;
        #[cfg(feature = "fixture-tools")]
        fn partprobe_occt_write_step_cube(path: *const c_char, size_mm: f64) -> c_int;
    }

    pub fn abi_version() -> u32 {
        // SAFETY: the function has no arguments or mutable state exposed across the ABI.
        unsafe { partprobe_occt_abi_version() }
    }

    pub fn analyze_step(path: &Path) -> Result<NativeBasicProperties, NativeAdapterError> {
        let path = path.to_str().ok_or(NativeAdapterError {
            diagnostic_code: "ASSET_PATH_ENCODING_UNSUPPORTED",
        })?;
        let path = CString::new(path).map_err(|_| NativeAdapterError {
            diagnostic_code: "ASSET_PATH_ENCODING_UNSUPPORTED",
        })?;
        let mut result = NativeResult {
            abi_version: OCCT_ADAPTER_ABI_VERSION,
            transferred_roots: 0,
            solid_body_count: 0,
            surface_area_mm2: 0.0,
            enclosed_volume_mm3: 0.0,
            center_of_mass_x_mm: 0.0,
            center_of_mass_y_mm: 0.0,
            center_of_mass_z_mm: 0.0,
            diagnostic_code: [0; DIAGNOSTIC_CAPACITY],
        };
        // SAFETY: `path` is NUL-terminated and lives through the call; `result` is writable,
        // correctly aligned, and paired with its exact size. C++ catches exceptions internally.
        let status = unsafe {
            partprobe_occt_analyze_step(path.as_ptr(), &mut result, size_of::<NativeResult>())
        };
        if status != 0 {
            return Err(NativeAdapterError {
                diagnostic_code: diagnostic_code(&result),
            });
        }
        if result.abi_version != OCCT_ADAPTER_ABI_VERSION
            || [
                result.surface_area_mm2,
                result.enclosed_volume_mm3,
                result.center_of_mass_x_mm,
                result.center_of_mass_y_mm,
                result.center_of_mass_z_mm,
            ]
            .iter()
            .any(|value| !value.is_finite())
            || result.surface_area_mm2 < 0.0
            || result.enclosed_volume_mm3 < 0.0
        {
            return Err(NativeAdapterError {
                diagnostic_code: "OCCT_INVALID_RESULT",
            });
        }
        Ok(NativeBasicProperties {
            transferred_roots: result.transferred_roots,
            solid_body_count: result.solid_body_count,
            surface_area_mm2: result.surface_area_mm2,
            enclosed_volume_mm3: result.enclosed_volume_mm3,
            center_of_mass_mm: [
                result.center_of_mass_x_mm,
                result.center_of_mass_y_mm,
                result.center_of_mass_z_mm,
            ],
        })
    }

    #[cfg(feature = "fixture-tools")]
    pub fn write_synthetic_step_cube(path: &Path, size_mm: f64) -> Result<(), NativeAdapterError> {
        if !size_mm.is_finite() || size_mm <= 0.0 {
            return Err(NativeAdapterError {
                diagnostic_code: "OCCT_INVALID_ARGUMENT",
            });
        }
        let path = path.to_str().ok_or(NativeAdapterError {
            diagnostic_code: "ASSET_PATH_ENCODING_UNSUPPORTED",
        })?;
        let path = CString::new(path).map_err(|_| NativeAdapterError {
            diagnostic_code: "ASSET_PATH_ENCODING_UNSUPPORTED",
        })?;
        // SAFETY: `path` is NUL-terminated and valid for the call; C++ catches all exceptions.
        let status = unsafe { partprobe_occt_write_step_cube(path.as_ptr(), size_mm) };
        if status == 0 {
            Ok(())
        } else {
            Err(NativeAdapterError {
                diagnostic_code: "STEP_FIXTURE_WRITE_FAILED",
            })
        }
    }

    fn diagnostic_code(result: &NativeResult) -> &'static str {
        // SAFETY: C++ always zero-initializes the fixed buffer and writes bounded static codes.
        let code = unsafe { CStr::from_ptr(result.diagnostic_code.as_ptr()) }
            .to_str()
            .unwrap_or("");
        match code {
            "OCCT_ABI_MISMATCH" => "OCCT_ABI_MISMATCH",
            "OCCT_INVALID_ARGUMENT" => "OCCT_INVALID_ARGUMENT",
            "STEP_READ_FAILED" => "STEP_READ_FAILED",
            "STEP_TRANSFER_FAILED" => "STEP_TRANSFER_FAILED",
            "STEP_NO_SHAPE" => "STEP_NO_SHAPE",
            "OCCT_STANDARD_FAILURE" => "OCCT_STANDARD_FAILURE",
            "OCCT_UNKNOWN_FAILURE" => "OCCT_UNKNOWN_FAILURE",
            _ => "OCCT_UNKNOWN_FAILURE",
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn linked_adapter_reports_the_expected_abi() {
            assert_eq!(abi_version(), OCCT_ADAPTER_ABI_VERSION);
        }

        #[test]
        fn missing_asset_failure_exposes_only_a_stable_code() {
            let error = analyze_step(Path::new("partprobe-missing-fixture.step"))
                .expect_err("missing STEP asset must fail");
            assert_eq!(error.diagnostic_code(), "STEP_READ_FAILED");
        }

        #[test]
        fn analytic_step_cube_matches_reviewable_properties() {
            let fixture =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm.step");
            let properties = analyze_step(&fixture).expect("analytic STEP cube must import");

            assert_eq!(properties.transferred_roots, 1);
            assert_eq!(properties.solid_body_count, 1);
            assert!((properties.surface_area_mm2 - 600.0).abs() <= 0.000_001);
            assert!((properties.enclosed_volume_mm3 - 1000.0).abs() <= 0.000_001);
            for component in properties.center_of_mass_mm {
                assert!((component - 5.0).abs() <= 0.000_001);
            }
        }
    }
}

/// Returns the linked native adapter ABI version.
#[cfg(feature = "native-occt")]
#[must_use]
pub fn linked_abi_version() -> u32 {
    native::abi_version()
}

/// Imports one controlled STEP asset and returns basic unrounded native measurements.
///
/// The caller must resolve the opaque asset capability to this worker-local path. Values remain
/// non-authoritative until unit, tolerance, fixture, and replay validation succeeds.
#[cfg(feature = "native-occt")]
pub fn analyze_step(
    worker_local_path: &std::path::Path,
) -> Result<NativeBasicProperties, NativeAdapterError> {
    native::analyze_step(worker_local_path)
}

/// Writes a synthetic analytic cube used only to regenerate the public STEP fixture.
#[cfg(feature = "fixture-tools")]
pub fn write_synthetic_step_cube(
    output_path: &std::path::Path,
    size_mm: f64,
) -> Result<(), NativeAdapterError> {
    native::write_synthetic_step_cube(output_path, size_mm)?;
    normalize_step_timestamp(output_path)
}

#[cfg(feature = "fixture-tools")]
fn normalize_step_timestamp(output_path: &std::path::Path) -> Result<(), NativeAdapterError> {
    const PREFIX: &str = "FILE_NAME('Open CASCADE Shape Model','";
    const TIMESTAMP_LENGTH: usize = 19;
    const FIXED_TIMESTAMP: &str = "2000-01-01T00:00:00";
    let error = || NativeAdapterError {
        diagnostic_code: "STEP_FIXTURE_NORMALIZATION_FAILED",
    };

    let mut contents = std::fs::read_to_string(output_path).map_err(|_| error())?;
    let start = contents.find(PREFIX).ok_or_else(error)? + PREFIX.len();
    let end = start.checked_add(TIMESTAMP_LENGTH).ok_or_else(error)?;
    if !contents.is_char_boundary(start)
        || !contents.is_char_boundary(end)
        || contents.as_bytes().get(end) != Some(&b'\'')
    {
        return Err(error());
    }
    contents.replace_range(start..end, FIXED_TIMESTAMP);
    std::fs::write(output_path, contents).map_err(|_| error())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_native_adapter_is_not_a_default_capability() {
        assert_eq!(
            native_occt_enabled(),
            cfg!(feature = "native-occt"),
            "capability must reflect the explicit Cargo feature"
        );
    }
}

#[cfg(all(test, feature = "fixture-tools"))]
mod fixture_tool_tests {
    use super::*;

    #[test]
    fn generated_step_cube_reproduces_the_committed_fixture() {
        let output = std::env::temp_dir().join(format!(
            "partprobe-generated-cube-{}.step",
            std::process::id()
        ));
        write_synthetic_step_cube(&output, 10.0).expect("fixture generation must succeed");
        let committed = include_bytes!("../../../fixtures/models/cube_10mm.step");
        assert_eq!(
            std::fs::read(&output).expect("generated fixture must be readable"),
            committed
        );
        std::fs::remove_file(output).expect("generated fixture must be removable");
    }
}
