//! Validated expectation records for geometry fixtures.

use std::collections::BTreeSet;

use partprobe_domain::{DomainError, SchemaVersion};
use partprobe_geometry_core::{GeometryWarningCode, ModelLengthUnit, RepresentationBasis};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

const FIXTURE_EXPECTATION_SCHEMA_VERSION: u16 = 2;

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

    /// Returns required warning codes.
    #[must_use]
    pub fn required_warnings(&self) -> &[GeometryWarningCode] {
        &self.required_warnings
    }
}
