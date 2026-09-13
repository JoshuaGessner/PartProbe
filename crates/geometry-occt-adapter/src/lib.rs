//! Optional, narrow C ABI boundary for the out-of-process OCCT adapter.

/// Native adapter ABI version implemented by this crate.
pub const OCCT_ADAPTER_ABI_VERSION: u32 = 4;
/// First additive native display-tessellation ABI implemented by this crate.
pub const OCCT_DISPLAY_TESSELLATION_ABI_VERSION: u32 = 1;

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
    /// Precise source-axis-aligned bounding extents in millimetres.
    pub aabb_extents_mm: [f64; 3],
}

/// Caller-supplied ceilings enforced before native tessellation crosses the ABI.
#[cfg(feature = "native-occt")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeTessellationLimits {
    /// Maximum flattened display vertices.
    max_vertices: u64,
    /// Maximum flattened display triangles.
    max_triangles: u64,
    /// Maximum combined position and index bytes.
    max_bytes: u64,
}

#[cfg(feature = "native-occt")]
impl NativeTessellationLimits {
    /// Creates positive display-tessellation ceilings.
    pub fn new(
        max_vertices: u64,
        max_triangles: u64,
        max_bytes: u64,
    ) -> Result<Self, NativeAdapterError> {
        if max_vertices == 0 || max_triangles == 0 || max_bytes == 0 {
            return Err(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_LIMITS",
            });
        }
        Ok(Self {
            max_vertices,
            max_triangles,
            max_bytes,
        })
    }
}

/// Native display-only triangle mesh copied into Rust-owned finite arrays.
#[cfg(feature = "native-occt")]
#[derive(Debug, PartialEq)]
pub struct NativeDisplayMesh {
    positions_mm: Vec<[f32; 3]>,
    triangle_indices: Vec<u32>,
}

#[cfg(feature = "native-occt")]
impl NativeDisplayMesh {
    /// Returns canonical-millimetre display positions.
    #[must_use]
    pub fn positions_mm(&self) -> &[[f32; 3]] {
        &self.positions_mm
    }

    /// Returns a flattened triangle-list index buffer.
    #[must_use]
    pub fn triangle_indices(&self) -> &[u32] {
        &self.triangle_indices
    }
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
    use std::ffi::{CStr, CString, c_char, c_int, c_void};
    use std::mem::size_of;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::path::Path;

    use super::{
        NativeAdapterError, NativeBasicProperties, NativeDisplayMesh, NativeTessellationLimits,
        OCCT_ADAPTER_ABI_VERSION, OCCT_DISPLAY_TESSELLATION_ABI_VERSION,
    };

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
        aabb_extent_x_mm: f64,
        aabb_extent_y_mm: f64,
        aabb_extent_z_mm: f64,
        diagnostic_code: [c_char; DIAGNOSTIC_CAPACITY],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NativeDisplayResult {
        abi_version: u32,
        reserved: u32,
        vertex_count: u64,
        triangle_count: u64,
        positions: *const f32,
        triangle_indices: *const u32,
        ownership: *mut c_void,
        diagnostic_code: [c_char; DIAGNOSTIC_CAPACITY],
    }

    unsafe extern "C" {
        fn partprobe_occt_abi_version() -> u32;
        fn partprobe_occt_analyze_step_bytes(
            bytes: *const u8,
            byte_count: usize,
            result: *mut NativeResult,
            result_size: usize,
            cancellation_probe: extern "C" fn(*const c_void) -> u8,
            cancellation_context: *const c_void,
        ) -> c_int;
        fn partprobe_occt_analyze_step(
            path: *const c_char,
            result: *mut NativeResult,
            result_size: usize,
            cancellation_probe: extern "C" fn(*const c_void) -> u8,
            cancellation_context: *const c_void,
        ) -> c_int;
        fn partprobe_occt_tessellate_step_bytes(
            bytes: *const u8,
            byte_count: usize,
            linear_deflection_mm: f64,
            angular_deflection_degrees: f64,
            max_vertices: u64,
            max_triangles: u64,
            max_bytes: u64,
            result: *mut NativeDisplayResult,
            result_size: usize,
            cancellation_probe: extern "C" fn(*const c_void) -> u8,
            cancellation_context: *const c_void,
        ) -> c_int;
        fn partprobe_occt_free_display_mesh(ownership: *mut c_void);
        #[cfg(feature = "fixture-tools")]
        fn partprobe_occt_write_step_cube(path: *const c_char, size_mm: f64) -> c_int;
    }

    pub fn abi_version() -> u32 {
        // SAFETY: the function has no arguments or mutable state exposed across the ABI.
        unsafe { partprobe_occt_abi_version() }
    }

    pub fn analyze_step(path: &Path) -> Result<NativeBasicProperties, NativeAdapterError> {
        analyze_step_with_cancellation(path, &|| false)
    }

    pub fn analyze_step_bytes(
        step_bytes: &[u8],
    ) -> Result<NativeBasicProperties, NativeAdapterError> {
        analyze_step_bytes_with_cancellation(step_bytes, &|| false)
    }

    pub fn analyze_step_bytes_with_cancellation<P>(
        step_bytes: &[u8],
        cancellation_probe: &P,
    ) -> Result<NativeBasicProperties, NativeAdapterError>
    where
        P: Fn() -> bool + Sync,
    {
        analyze_with_cancellation(cancellation_probe, |probe, context, result| {
            // SAFETY: the slice pointer and exact length remain valid for the blocking call;
            // `result` and cancellation callback/context satisfy `analyze_with_cancellation`.
            unsafe {
                partprobe_occt_analyze_step_bytes(
                    step_bytes.as_ptr(),
                    step_bytes.len(),
                    result,
                    size_of::<NativeResult>(),
                    probe,
                    context,
                )
            }
        })
    }

    pub fn analyze_step_with_cancellation<P>(
        path: &Path,
        cancellation_probe: &P,
    ) -> Result<NativeBasicProperties, NativeAdapterError>
    where
        P: Fn() -> bool + Sync,
    {
        let path = path.to_str().ok_or(NativeAdapterError {
            diagnostic_code: "ASSET_PATH_ENCODING_UNSUPPORTED",
        })?;
        let path = CString::new(path).map_err(|_| NativeAdapterError {
            diagnostic_code: "ASSET_PATH_ENCODING_UNSUPPORTED",
        })?;
        analyze_with_cancellation(cancellation_probe, |probe, context, result| {
            // SAFETY: `path` is NUL-terminated and lives through the call; `result` and the
            // cancellation callback/context satisfy `analyze_with_cancellation`.
            unsafe {
                partprobe_occt_analyze_step(
                    path.as_ptr(),
                    result,
                    size_of::<NativeResult>(),
                    probe,
                    context,
                )
            }
        })
    }

    pub fn tessellate_step_bytes_with_cancellation<P>(
        step_bytes: &[u8],
        linear_deflection_mm: f64,
        angular_deflection_degrees: f64,
        limits: NativeTessellationLimits,
        cancellation_probe: &P,
    ) -> Result<NativeDisplayMesh, NativeAdapterError>
    where
        P: Fn() -> bool + Sync,
    {
        extern "C" fn probe<P>(context: *const c_void) -> u8
        where
            P: Fn() -> bool + Sync,
        {
            if context.is_null() {
                return 1;
            }
            // SAFETY: the context points to the borrowed probe for the duration of the blocking
            // native call. `P: Sync` permits concurrent read-only callback invocation.
            let probe = unsafe { &*context.cast::<P>() };
            u8::from(catch_unwind(AssertUnwindSafe(probe)).unwrap_or(true))
        }

        if !linear_deflection_mm.is_finite()
            || linear_deflection_mm <= 0.0
            || !angular_deflection_degrees.is_finite()
            || angular_deflection_degrees <= 0.0
        {
            return Err(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_PROFILE",
            });
        }

        let mut result = NativeDisplayResult {
            abi_version: OCCT_DISPLAY_TESSELLATION_ABI_VERSION,
            reserved: 0,
            vertex_count: 0,
            triangle_count: 0,
            positions: std::ptr::null(),
            triangle_indices: std::ptr::null(),
            ownership: std::ptr::null_mut(),
            diagnostic_code: [0; DIAGNOSTIC_CAPACITY],
        };
        let status = unsafe {
            partprobe_occt_tessellate_step_bytes(
                step_bytes.as_ptr(),
                step_bytes.len(),
                linear_deflection_mm,
                angular_deflection_degrees,
                limits.max_vertices,
                limits.max_triangles,
                limits.max_bytes,
                &mut result,
                size_of::<NativeDisplayResult>(),
                probe::<P>,
                std::ptr::from_ref(cancellation_probe).cast::<c_void>(),
            )
        };
        let ownership = DisplayMeshOwnership(result.ownership);
        if status != 0 {
            return Err(NativeAdapterError {
                diagnostic_code: display_diagnostic_code(&result),
            });
        }
        let position_value_count = result
            .vertex_count
            .checked_mul(3)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
            })?;
        let index_count = result
            .triangle_count
            .checked_mul(3)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
            })?;
        let byte_count = result
            .vertex_count
            .checked_mul(12)
            .and_then(|value| result.triangle_count.checked_mul(12)?.checked_add(value))
            .ok_or(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
            })?;
        if result.abi_version != OCCT_DISPLAY_TESSELLATION_ABI_VERSION
            || result.reserved != 0
            || result.vertex_count == 0
            || result.triangle_count == 0
            || result.vertex_count > limits.max_vertices
            || result.triangle_count > limits.max_triangles
            || byte_count > limits.max_bytes
            || result.positions.is_null()
            || result.triangle_indices.is_null()
            || ownership.0.is_null()
        {
            return Err(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
            });
        }

        // SAFETY: a successful native call owns arrays with the exact validated counts until the
        // guard is dropped. Both pointers are non-null and the values are copied before that drop.
        let native_positions =
            unsafe { std::slice::from_raw_parts(result.positions, position_value_count) };
        // SAFETY: same ownership and count proof as the position slice above.
        let native_indices =
            unsafe { std::slice::from_raw_parts(result.triangle_indices, index_count) };
        let mut positions_mm = Vec::new();
        positions_mm
            .try_reserve_exact(usize::try_from(result.vertex_count).map_err(|_| {
                NativeAdapterError {
                    diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
                }
            })?)
            .map_err(|_| NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_ALLOCATION_FAILED",
            })?;
        for coordinates in native_positions.chunks_exact(3) {
            let position = [coordinates[0], coordinates[1], coordinates[2]];
            if position.iter().any(|coordinate| !coordinate.is_finite()) {
                return Err(NativeAdapterError {
                    diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
                });
            }
            positions_mm.push(position);
        }
        if native_indices
            .iter()
            .any(|index| u64::from(*index) >= result.vertex_count)
        {
            return Err(NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_INVALID_RESULT",
            });
        }
        let mut triangle_indices = Vec::new();
        triangle_indices
            .try_reserve_exact(index_count)
            .map_err(|_| NativeAdapterError {
                diagnostic_code: "OCCT_TESSELLATION_ALLOCATION_FAILED",
            })?;
        triangle_indices.extend_from_slice(native_indices);
        drop(ownership);
        Ok(NativeDisplayMesh {
            positions_mm,
            triangle_indices,
        })
    }

    struct DisplayMeshOwnership(*mut c_void);

    impl Drop for DisplayMeshOwnership {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: the opaque allocation came from the matching OCCT bridge function and
                // this guard is its sole Rust owner.
                unsafe { partprobe_occt_free_display_mesh(self.0) };
            }
        }
    }

    fn analyze_with_cancellation<P>(
        cancellation_probe: &P,
        analyze: impl FnOnce(
            extern "C" fn(*const c_void) -> u8,
            *const c_void,
            *mut NativeResult,
        ) -> c_int,
    ) -> Result<NativeBasicProperties, NativeAdapterError>
    where
        P: Fn() -> bool + Sync,
    {
        extern "C" fn probe<P>(context: *const c_void) -> u8
        where
            P: Fn() -> bool + Sync,
        {
            if context.is_null() {
                return 1;
            }
            // SAFETY: the context points to the borrowed probe for the duration of the blocking
            // native call. `P: Sync` permits concurrent read-only callback invocation.
            let probe = unsafe { &*context.cast::<P>() };
            u8::from(catch_unwind(AssertUnwindSafe(probe)).unwrap_or(true))
        }

        let mut result = NativeResult {
            abi_version: OCCT_ADAPTER_ABI_VERSION,
            transferred_roots: 0,
            solid_body_count: 0,
            surface_area_mm2: 0.0,
            enclosed_volume_mm3: 0.0,
            center_of_mass_x_mm: 0.0,
            center_of_mass_y_mm: 0.0,
            center_of_mass_z_mm: 0.0,
            aabb_extent_x_mm: 0.0,
            aabb_extent_y_mm: 0.0,
            aabb_extent_z_mm: 0.0,
            diagnostic_code: [0; DIAGNOSTIC_CAPACITY],
        };
        let status = analyze(
            probe::<P>,
            std::ptr::from_ref(cancellation_probe).cast::<c_void>(),
            &mut result,
        );
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
                result.aabb_extent_x_mm,
                result.aabb_extent_y_mm,
                result.aabb_extent_z_mm,
            ]
            .iter()
            .any(|value| !value.is_finite())
            || result.surface_area_mm2 < 0.0
            || result.enclosed_volume_mm3 < 0.0
            || result.aabb_extent_x_mm <= 0.0
            || result.aabb_extent_y_mm <= 0.0
            || result.aabb_extent_z_mm <= 0.0
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
            aabb_extents_mm: [
                result.aabb_extent_x_mm,
                result.aabb_extent_y_mm,
                result.aabb_extent_z_mm,
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
            "OCCT_CANCELLED" => "OCCT_CANCELLED",
            "STEP_READ_FAILED" => "STEP_READ_FAILED",
            "STEP_TRANSFER_FAILED" => "STEP_TRANSFER_FAILED",
            "STEP_NO_SHAPE" => "STEP_NO_SHAPE",
            "OCCT_INVALID_BOUNDS" => "OCCT_INVALID_BOUNDS",
            "OCCT_STANDARD_FAILURE" => "OCCT_STANDARD_FAILURE",
            "OCCT_UNKNOWN_FAILURE" => "OCCT_UNKNOWN_FAILURE",
            _ => "OCCT_UNKNOWN_FAILURE",
        }
    }

    fn display_diagnostic_code(result: &NativeDisplayResult) -> &'static str {
        // SAFETY: C++ always zero-initializes the fixed buffer and writes bounded static codes.
        let code = unsafe { CStr::from_ptr(result.diagnostic_code.as_ptr()) }
            .to_str()
            .unwrap_or("");
        match code {
            "OCCT_CANCELLED" => "OCCT_CANCELLED",
            "OCCT_INVALID_ARGUMENT" => "OCCT_INVALID_ARGUMENT",
            "OCCT_TESSELLATION_INVALID_PROFILE" => "OCCT_TESSELLATION_INVALID_PROFILE",
            "OCCT_TESSELLATION_INVALID_LIMITS" => "OCCT_TESSELLATION_INVALID_LIMITS",
            "OCCT_TESSELLATION_FAILED" => "OCCT_TESSELLATION_FAILED",
            "OCCT_TESSELLATION_EMPTY" => "OCCT_TESSELLATION_EMPTY",
            "OCCT_TESSELLATION_LIMIT_EXCEEDED" => "OCCT_TESSELLATION_LIMIT_EXCEEDED",
            "OCCT_TESSELLATION_ALLOCATION_FAILED" => "OCCT_TESSELLATION_ALLOCATION_FAILED",
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
        fn invalid_step_entity_failure_exposes_only_a_stable_code() {
            let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/models/invalid_entity.step");
            let error = analyze_step(&fixture).expect_err("invalid STEP entity must fail");
            assert_eq!(error.diagnostic_code(), "STEP_TRANSFER_FAILED");
        }

        #[test]
        fn invalid_step_bytes_expose_only_a_stable_code() {
            let bytes = include_bytes!("../../../fixtures/models/invalid_entity.step");
            let error = analyze_step_bytes(bytes).expect_err("invalid STEP bytes must fail");
            assert_eq!(error.diagnostic_code(), "STEP_TRANSFER_FAILED");
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
            for (actual, expected) in properties.aabb_extents_mm.into_iter().zip([10.0; 3]) {
                assert!((actual - expected).abs() <= 0.000_001);
            }
        }

        #[test]
        fn analytic_step_cube_bytes_match_reviewable_properties() {
            let bytes = include_bytes!("../../../fixtures/models/cube_10mm.step");
            let properties =
                analyze_step_bytes(bytes).expect("analytic STEP cube bytes must import");

            assert_eq!(properties.transferred_roots, 1);
            assert_eq!(properties.solid_body_count, 1);
            assert!((properties.surface_area_mm2 - 600.0).abs() <= 0.000_001);
            assert!((properties.enclosed_volume_mm3 - 1000.0).abs() <= 0.000_001);
            for component in properties.center_of_mass_mm {
                assert!((component - 5.0).abs() <= 0.000_001);
            }
            for (actual, expected) in properties.aabb_extents_mm.into_iter().zip([10.0; 3]) {
                assert!((actual - expected).abs() <= 0.000_001);
            }
        }

        #[test]
        fn independently_authored_step_prism_matches_analytic_properties() {
            let bytes = include_bytes!("../../../fixtures/models/rectangular_prism_12x8x5.step");
            let properties =
                analyze_step_bytes(bytes).expect("independently authored STEP prism must import");

            assert_eq!(properties.transferred_roots, 1);
            assert_eq!(properties.solid_body_count, 1);
            assert!((properties.surface_area_mm2 - 392.0).abs() <= 0.000_001);
            assert!((properties.enclosed_volume_mm3 - 480.0).abs() <= 0.000_001);
            for (actual, expected) in properties
                .center_of_mass_mm
                .into_iter()
                .zip([6.0, 4.0, 2.5])
            {
                assert!((actual - expected).abs() <= 0.000_001);
            }
            for (actual, expected) in properties.aabb_extents_mm.into_iter().zip([12.0, 8.0, 5.0]) {
                assert!((actual - expected).abs() <= 0.000_001);
            }
        }

        #[test]
        fn cancellation_probe_stops_native_analysis_with_a_stable_code() {
            use std::sync::atomic::{AtomicUsize, Ordering};

            let bytes = include_bytes!("../../../fixtures/models/cube_10mm.step");
            let calls = AtomicUsize::new(0);
            let error = analyze_step_bytes_with_cancellation(bytes, &|| {
                calls.fetch_add(1, Ordering::Relaxed);
                true
            })
            .expect_err("pre-requested cancellation must stop native analysis");

            assert_eq!(error.diagnostic_code(), "OCCT_CANCELLED");
            assert!(calls.load(Ordering::Relaxed) > 0);
        }

        #[test]
        fn cancellation_probe_panic_is_contained_at_the_native_boundary() {
            let bytes = include_bytes!("../../../fixtures/models/cube_10mm.step");
            let error = analyze_step_bytes_with_cancellation(bytes, &|| {
                panic!("probe failure must not unwind through C++")
            })
            .expect_err("a failed probe must stop native analysis");

            assert_eq!(error.diagnostic_code(), "OCCT_CANCELLED");
        }

        fn display_limits() -> NativeTessellationLimits {
            NativeTessellationLimits::new(1_000_000, 2_000_000, 32 * 1024 * 1024)
                .expect("governed display limits must be valid")
        }

        #[test]
        fn analytic_step_cube_tessellates_into_bounded_canonical_arrays() {
            let bytes = include_bytes!("../../../fixtures/models/cube_10mm.step");
            let mesh = tessellate_step_bytes_with_cancellation(
                bytes,
                0.1,
                12.0,
                display_limits(),
                &|| false,
            )
            .expect("analytic cube must tessellate");

            assert_eq!(mesh.positions_mm().len(), 24);
            assert_eq!(mesh.triangle_indices().len(), 36);
            assert!(mesh.positions_mm().iter().flatten().all(|value| {
                value.is_finite() && *value >= -0.000_001 && *value <= 10.000_001
            }));
            assert!(
                mesh.triangle_indices()
                    .iter()
                    .all(|index| (*index as usize) < mesh.positions_mm().len())
            );
        }

        #[test]
        fn independently_authored_step_prism_tessellation_is_deterministic() {
            let bytes = include_bytes!("../../../fixtures/models/rectangular_prism_12x8x5.step");
            let first = tessellate_step_bytes_with_cancellation(
                bytes,
                0.1,
                12.0,
                display_limits(),
                &|| false,
            )
            .expect("independent prism must tessellate");
            let second = tessellate_step_bytes_with_cancellation(
                bytes,
                0.1,
                12.0,
                display_limits(),
                &|| false,
            )
            .expect("repeated prism tessellation must succeed");

            assert_eq!(first, second);
            assert_eq!(first.positions_mm().len(), 24);
            assert_eq!(first.triangle_indices().len(), 36);
            let maximums =
                first
                    .positions_mm()
                    .iter()
                    .fold([f32::NEG_INFINITY; 3], |mut bounds, position| {
                        for axis in 0..3 {
                            bounds[axis] = bounds[axis].max(position[axis]);
                        }
                        bounds
                    });
            assert_eq!(maximums, [12.0, 8.0, 5.0]);
        }

        #[test]
        fn tessellation_fails_closed_at_the_caller_limits() {
            let bytes = include_bytes!("../../../fixtures/models/cube_10mm.step");
            let limits = NativeTessellationLimits::new(23, 12, 1_000)
                .expect("positive test limits must be valid");
            let error =
                tessellate_step_bytes_with_cancellation(bytes, 0.1, 12.0, limits, &|| false)
                    .expect_err("cube must not cross a 23-vertex limit");

            assert_eq!(error.diagnostic_code(), "OCCT_TESSELLATION_LIMIT_EXCEEDED");
        }

        #[test]
        fn tessellation_cancellation_is_sanitized() {
            let bytes = include_bytes!("../../../fixtures/models/cube_10mm.step");
            let error = tessellate_step_bytes_with_cancellation(
                bytes,
                0.1,
                12.0,
                display_limits(),
                &|| true,
            )
            .expect_err("pre-requested cancellation must stop tessellation");

            assert_eq!(error.diagnostic_code(), "OCCT_CANCELLED");
        }
    }
}

/// Returns the linked native adapter ABI version.
#[cfg(feature = "native-occt")]
#[must_use]
pub fn linked_abi_version() -> u32 {
    native::abi_version()
}

/// Imports one caller-bounded in-memory STEP asset and returns basic unrounded measurements.
///
/// The bytes must already have passed the worker's capability, length, quota, and hash checks.
/// Values remain non-authoritative until unit, tolerance, fixture, and replay validation succeeds.
#[cfg(feature = "native-occt")]
pub fn analyze_step_bytes(step_bytes: &[u8]) -> Result<NativeBasicProperties, NativeAdapterError> {
    native::analyze_step_bytes(step_bytes)
}

/// Imports caller-bounded STEP bytes while polling cancellation where OCCT supports progress.
///
/// OCCT 8.0 polls the probe before and after stream parsing and during STEP root transfer. Stream
/// parsing and the current property calculations do not expose a progress range, so callers still
/// require an external deadline and force-termination boundary for those phases. A probe panic is
/// contained and treated as a cancellation request rather than unwinding through the native ABI.
#[cfg(feature = "native-occt")]
pub fn analyze_step_bytes_with_cancellation<P>(
    step_bytes: &[u8],
    cancellation_probe: &P,
) -> Result<NativeBasicProperties, NativeAdapterError>
where
    P: Fn() -> bool + Sync,
{
    native::analyze_step_bytes_with_cancellation(step_bytes, cancellation_probe)
}

/// Tessellates caller-bounded STEP bytes into one flattened display-only triangle mesh.
///
/// The additive display ABI enforces the caller's vertex, triangle, and combined-byte ceilings
/// before any native array is exposed. Rust revalidates the ABI, counts, finite positions, and
/// index bounds while copying into Rust-owned arrays. This output is visualization evidence only;
/// it does not replace exact B-rep measurements or imply CAM/topology authority.
#[cfg(feature = "native-occt")]
pub fn tessellate_step_bytes_with_cancellation<P>(
    step_bytes: &[u8],
    linear_deflection_mm: f64,
    angular_deflection_degrees: f64,
    limits: NativeTessellationLimits,
    cancellation_probe: &P,
) -> Result<NativeDisplayMesh, NativeAdapterError>
where
    P: Fn() -> bool + Sync,
{
    native::tessellate_step_bytes_with_cancellation(
        step_bytes,
        linear_deflection_mm,
        angular_deflection_degrees,
        limits,
        cancellation_probe,
    )
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

/// Imports one controlled STEP asset while polling cancellation where OCCT supports progress.
///
/// OCCT 8.0 polls the probe during STEP root transfer. File parsing and the current property
/// calculations do not expose a progress range, so callers still require an external deadline and
/// force-termination boundary for those phases. A probe panic is contained and treated as a
/// cancellation request rather than unwinding through the native ABI.
#[cfg(feature = "native-occt")]
pub fn analyze_step_with_cancellation<P>(
    worker_local_path: &std::path::Path,
    cancellation_probe: &P,
) -> Result<NativeBasicProperties, NativeAdapterError>
where
    P: Fn() -> bool + Sync,
{
    native::analyze_step_with_cancellation(worker_local_path, cancellation_probe)
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
