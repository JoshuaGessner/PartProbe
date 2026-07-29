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
