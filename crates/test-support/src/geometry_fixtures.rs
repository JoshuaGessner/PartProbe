//! Validated expectation records for geometry fixtures.

use std::collections::BTreeSet;

use partprobe_domain::{DomainError, SchemaVersion};
use partprobe_geometry_core::{
    GeometryConfidence, GeometryConfidenceLevel, GeometryWarningCode, ModelLengthUnit,
    RepresentationBasis, Sha256Digest, StageStatus,
};
use partprobe_geometry_import::DiagnosticCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

const FIXTURE_EXPECTATION_SCHEMA_VERSION: u16 = 4;
const IMPORT_FAILURE_EXPECTATION_SCHEMA_VERSION: u16 = 1;

/// Explicit expectation state; unavailable and inapplicable never become zero.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ExpectedEvidence<T> {
    /// A reviewed expected value exists.
    Available {
        /// Expected value.
        value: T,
    },
    /// Evidence is unavailable for a stable reason.
    Unavailable {
        /// Stable reason code.
        reason_code: GeometryWarningCode,
    },
    /// The field does not apply to this representation.
    NotApplicable {
        /// Stable reason code.
        reason_code: GeometryWarningCode,
    },
}

/// Explicit mesh self-intersection state retained by successful fixture evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedMeshSelfIntersectionState {
    /// The governed detector found no intersection.
    NotDetected,
    /// The governed detector found an intersection.
    Detected,
    /// The governed detector cannot decide the coplanar case without a tolerance policy.
    Indeterminate,
}

impl<T> ExpectedEvidence<T> {
    /// Returns the available value when one exists.
    #[must_use]
    pub const fn available_value(&self) -> Option<&T> {
        match self {
            Self::Available { value } => Some(value),
            Self::Unavailable { .. } | Self::NotApplicable { .. } => None,
        }
    }
}

/// Three-dimensional value in canonical millimetres.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExpectedVector3 {
    /// X component.
    #[serde(with = "partprobe_domain::decimal_serde")]
    pub x: Decimal,
    /// Y component.
    #[serde(with = "partprobe_domain::decimal_serde")]
    pub y: Decimal,
    /// Z component.
    #[serde(with = "partprobe_domain::decimal_serde")]
    pub z: Decimal,
}

/// Geometry expectation record shared by fixture tests and future adapters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryFixtureExpectation {
    schema_version: SchemaVersion,
    fixture_id: String,
    representation: RepresentationBasis,
    imported_units: Option<ModelLengthUnit>,
    confirmed_units: ModelLengthUnit,
    body_count: u64,
    triangle_count: Option<u64>,
    watertight: ExpectedEvidence<bool>,
    manifold: ExpectedEvidence<bool>,
    self_intersection: ExpectedEvidence<ExpectedMeshSelfIntersectionState>,
    confidence: GeometryConfidence,
    aabb_mm: ExpectedEvidence<ExpectedVector3>,
    surface_area_mm2: ExpectedEvidence<Decimal>,
    enclosed_volume_mm3: ExpectedEvidence<Decimal>,
    center_of_mass_mm: ExpectedEvidence<ExpectedVector3>,
    #[serde(with = "partprobe_domain::decimal_serde")]
    absolute_tolerance: Decimal,
    required_warnings: Vec<GeometryWarningCode>,
}

#[derive(Deserialize)]
struct GeometryFixtureExpectationWire {
    schema_version: SchemaVersion,
    fixture_id: String,
    representation: RepresentationBasis,
    imported_units: Option<ModelLengthUnit>,
    confirmed_units: ModelLengthUnit,
    body_count: u64,
    triangle_count: Option<u64>,
    watertight: ExpectedEvidence<bool>,
    manifold: ExpectedEvidence<bool>,
    self_intersection: ExpectedEvidence<ExpectedMeshSelfIntersectionState>,
    confidence: GeometryConfidence,
    aabb_mm: ExpectedEvidence<ExpectedVector3>,
    surface_area_mm2: ExpectedEvidence<Decimal>,
    enclosed_volume_mm3: ExpectedEvidence<Decimal>,
    center_of_mass_mm: ExpectedEvidence<ExpectedVector3>,
    #[serde(with = "partprobe_domain::decimal_serde")]
    absolute_tolerance: Decimal,
    required_warnings: Vec<GeometryWarningCode>,
}

impl<'de> Deserialize<'de> for GeometryFixtureExpectation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryFixtureExpectationWire::deserialize(deserializer)?;
        Self::new(
            wire.schema_version,
            wire.fixture_id,
            wire.representation,
            wire.imported_units,
            wire.confirmed_units,
            wire.body_count,
            wire.triangle_count,
            wire.watertight,
            wire.manifold,
            wire.self_intersection,
            wire.confidence,
            wire.aabb_mm,
            wire.surface_area_mm2,
            wire.enclosed_volume_mm3,
            wire.center_of_mass_mm,
            wire.absolute_tolerance,
            wire.required_warnings,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl GeometryFixtureExpectation {
    /// Validates an exact fixture expectation without collapsing state.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_version: SchemaVersion,
        fixture_id: impl Into<String>,
        representation: RepresentationBasis,
        imported_units: Option<ModelLengthUnit>,
        confirmed_units: ModelLengthUnit,
        body_count: u64,
        triangle_count: Option<u64>,
        watertight: ExpectedEvidence<bool>,
        manifold: ExpectedEvidence<bool>,
        self_intersection: ExpectedEvidence<ExpectedMeshSelfIntersectionState>,
        confidence: GeometryConfidence,
        aabb_mm: ExpectedEvidence<ExpectedVector3>,
        surface_area_mm2: ExpectedEvidence<Decimal>,
        enclosed_volume_mm3: ExpectedEvidence<Decimal>,
        center_of_mass_mm: ExpectedEvidence<ExpectedVector3>,
        absolute_tolerance: Decimal,
        required_warnings: Vec<GeometryWarningCode>,
    ) -> Result<Self, DomainError> {
        let fixture_id = fixture_id.into();
        if schema_version.value() != FIXTURE_EXPECTATION_SCHEMA_VERSION {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture schema version",
                reason: "unsupported fixture expectation schema version",
            });
        }
        if fixture_id.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture ID",
                reason: "must not be empty",
            });
        }
        if confirmed_units == ModelLengthUnit::Unknown {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture units",
                reason: "confirmed units must be explicit",
            });
        }
        if body_count == 0 {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture body count",
                reason: "must be greater than zero",
            });
        }
        if representation == RepresentationBasis::Mesh && triangle_count.is_none() {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture triangle count",
                reason: "mesh expectations require a triangle count",
            });
        }
        if representation == RepresentationBasis::Mesh
            && self_intersection.available_value().is_none()
        {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture self-intersection",
                reason: "mesh expectations require an explicit self-intersection state",
            });
        }
        if representation != RepresentationBasis::Mesh
            && self_intersection.available_value().is_some()
        {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture self-intersection",
                reason: "non-mesh expectations cannot claim mesh self-intersection evidence",
            });
        }
        if representation == RepresentationBasis::Mesh
            && !matches!(
                confidence.level(),
                GeometryConfidenceLevel::Low | GeometryConfidenceLevel::NeedsReview
            )
        {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture confidence",
                reason: "mesh confidence cannot exceed low",
            });
        }
        match self_intersection.available_value().copied() {
            Some(ExpectedMeshSelfIntersectionState::Detected) => {
                require_intersection_review_evidence(
                    &confidence,
                    &required_warnings,
                    "SELF_INTERSECTION_DETECTED",
                )?;
            }
            Some(ExpectedMeshSelfIntersectionState::Indeterminate) => {
                require_intersection_review_evidence(
                    &confidence,
                    &required_warnings,
                    "SELF_INTERSECTION_INDETERMINATE",
                )?;
            }
            Some(ExpectedMeshSelfIntersectionState::NotDetected) => {
                if has_intersection_code(&confidence, &required_warnings) {
                    return Err(DomainError::InvalidValue {
                        field: "geometry fixture self-intersection evidence",
                        reason: "not-detected state cannot carry detected or indeterminate evidence",
                    });
                }
            }
            None => {}
        }
        if absolute_tolerance <= Decimal::ZERO {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture tolerance",
                reason: "must be greater than zero",
            });
        }
        for value in [
            surface_area_mm2.available_value(),
            enclosed_volume_mm3.available_value(),
        ]
        .into_iter()
        .flatten()
        {
            if value.is_sign_negative() {
                return Err(DomainError::InvalidValue {
                    field: "geometry fixture measurement",
                    reason: "area and volume must be nonnegative",
                });
            }
        }
        if matches!(watertight.available_value(), Some(false))
            && enclosed_volume_mm3.available_value().is_some()
        {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture volume",
                reason: "a non-watertight expectation cannot carry enclosed volume",
            });
        }
        if matches!(
            self_intersection.available_value(),
            Some(
                ExpectedMeshSelfIntersectionState::Detected
                    | ExpectedMeshSelfIntersectionState::Indeterminate
            )
        ) && (enclosed_volume_mm3.available_value().is_some()
            || center_of_mass_mm.available_value().is_some())
        {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture closed measurements",
                reason: "detected or indeterminate intersection cannot carry closed measurements",
            });
        }
        let mut warning_codes = BTreeSet::new();
        if required_warnings
            .iter()
            .any(|code| !warning_codes.insert(code.clone()))
        {
            return Err(DomainError::InvalidValue {
                field: "geometry fixture warnings",
                reason: "warning codes must be unique",
            });
        }
        Ok(Self {
            schema_version,
            fixture_id,
            representation,
            imported_units,
            confirmed_units,
            body_count,
            triangle_count,
            watertight,
            manifold,
            self_intersection,
            confidence,
            aabb_mm,
            surface_area_mm2,
            enclosed_volume_mm3,
            center_of_mass_mm,
            absolute_tolerance,
            required_warnings,
        })
    }

    /// Returns the fixture ID.
    #[must_use]
    pub fn fixture_id(&self) -> &str {
        &self.fixture_id
    }

    /// Returns the expected representation.
    #[must_use]
    pub const fn representation(&self) -> RepresentationBasis {
        self.representation
    }

    /// Returns the expected watertight state.
    #[must_use]
    pub const fn watertight(&self) -> &ExpectedEvidence<bool> {
        &self.watertight
    }

    /// Returns the expected enclosed-volume state.
    #[must_use]
    pub const fn enclosed_volume_mm3(&self) -> &ExpectedEvidence<Decimal> {
        &self.enclosed_volume_mm3
    }

    /// Returns the expected mesh self-intersection state.
    #[must_use]
    pub const fn self_intersection(&self) -> &ExpectedEvidence<ExpectedMeshSelfIntersectionState> {
        &self.self_intersection
    }

    /// Returns the expected categorical confidence and reasons.
    #[must_use]
    pub const fn confidence(&self) -> &GeometryConfidence {
        &self.confidence
    }

    /// Returns required warning codes.
    #[must_use]
    pub fn required_warnings(&self) -> &[GeometryWarningCode] {
        &self.required_warnings
    }
}

fn require_intersection_review_evidence(
    confidence: &GeometryConfidence,
    required_warnings: &[GeometryWarningCode],
    expected_code: &str,
) -> Result<(), DomainError> {
    let has_confidence_reason = confidence
        .reasons()
        .iter()
        .any(|reason| reason.as_str() == expected_code);
    let has_warning = required_warnings
        .iter()
        .any(|warning| warning.as_str() == expected_code);
    if confidence.level() != GeometryConfidenceLevel::NeedsReview
        || !has_confidence_reason
        || !has_warning
    {
        return Err(DomainError::InvalidValue {
            field: "geometry fixture self-intersection evidence",
            reason: "detected or indeterminate state requires matching review evidence",
        });
    }
    Ok(())
}

fn has_intersection_code(
    confidence: &GeometryConfidence,
    required_warnings: &[GeometryWarningCode],
) -> bool {
    const INTERSECTION_CODES: [&str; 2] = [
        "SELF_INTERSECTION_DETECTED",
        "SELF_INTERSECTION_INDETERMINATE",
    ];
    confidence
        .reasons()
        .iter()
        .any(|reason| INTERSECTION_CODES.contains(&reason.as_str()))
        || required_warnings
            .iter()
            .any(|warning| INTERSECTION_CODES.contains(&warning.as_str()))
}

/// Expected controlled outcome for an import that must fail without producing geometry evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryImportFailureExpectation {
    schema_version: SchemaVersion,
    fixture_id: String,
    source_sha256: Sha256Digest,
    expected_status: StageStatus,
    expected_diagnostic_code: DiagnosticCode,
    snapshot_expected: bool,
    output_file_expected: bool,
    staged_input_retained: bool,
}

#[derive(Deserialize)]
struct GeometryImportFailureExpectationWire {
    schema_version: SchemaVersion,
    fixture_id: String,
    source_sha256: Sha256Digest,
    expected_status: StageStatus,
    expected_diagnostic_code: DiagnosticCode,
    snapshot_expected: bool,
    output_file_expected: bool,
    staged_input_retained: bool,
}

impl<'de> Deserialize<'de> for GeometryImportFailureExpectation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryImportFailureExpectationWire::deserialize(deserializer)?;
        Self::new(
            wire.schema_version,
            wire.fixture_id,
            wire.source_sha256,
            wire.expected_status,
            wire.expected_diagnostic_code,
            wire.snapshot_expected,
            wire.output_file_expected,
            wire.staged_input_retained,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl GeometryImportFailureExpectation {
    /// Builds a failure expectation that cannot masquerade as successful geometry evidence.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_version: SchemaVersion,
        fixture_id: impl Into<String>,
        source_sha256: Sha256Digest,
        expected_status: StageStatus,
        expected_diagnostic_code: DiagnosticCode,
        snapshot_expected: bool,
        output_file_expected: bool,
        staged_input_retained: bool,
    ) -> Result<Self, DomainError> {
        let fixture_id = fixture_id.into();
        if schema_version.value() != IMPORT_FAILURE_EXPECTATION_SCHEMA_VERSION {
            return Err(DomainError::InvalidValue {
                field: "geometry import failure schema version",
                reason: "unsupported failure expectation schema version",
            });
        }
        if fixture_id.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry import failure fixture ID",
                reason: "must not be empty",
            });
        }
        if expected_status != StageStatus::FailedRecoverable {
            return Err(DomainError::InvalidValue {
                field: "geometry import failure status",
                reason: "adversarial fixture must fail recoverably",
            });
        }
        if snapshot_expected || output_file_expected || staged_input_retained {
            return Err(DomainError::InvalidValue {
                field: "geometry import failure artifacts",
                reason: "failed import must not retain staged input or produce snapshot/output",
            });
        }
        Ok(Self {
            schema_version,
            fixture_id,
            source_sha256,
            expected_status,
            expected_diagnostic_code,
            snapshot_expected,
            output_file_expected,
            staged_input_retained,
        })
    }

    /// Returns the fixture ID.
    #[must_use]
    pub fn fixture_id(&self) -> &str {
        &self.fixture_id
    }

    /// Returns the expected source digest.
    #[must_use]
    pub const fn source_sha256(&self) -> &Sha256Digest {
        &self.source_sha256
    }

    /// Returns the required recoverable status.
    #[must_use]
    pub const fn expected_status(&self) -> StageStatus {
        self.expected_status
    }

    /// Returns the required sanitized diagnostic.
    #[must_use]
    pub const fn expected_diagnostic_code(&self) -> &DiagnosticCode {
        &self.expected_diagnostic_code
    }

    /// Returns whether a snapshot is expected.
    #[must_use]
    pub const fn snapshot_expected(&self) -> bool {
        self.snapshot_expected
    }

    /// Returns whether an output file is expected.
    #[must_use]
    pub const fn output_file_expected(&self) -> bool {
        self.output_file_expected
    }

    /// Returns whether the staged input may remain.
    #[must_use]
    pub const fn staged_input_retained(&self) -> bool {
        self.staged_input_retained
    }
}
