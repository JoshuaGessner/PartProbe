use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize};

use crate::DomainError;

macro_rules! non_empty_string_wrapper {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and creates the value.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must not be empty",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

non_empty_string_wrapper!(
    /// Stable identifier of a calculation rule.
    RuleId,
    "rule ID"
);

non_empty_string_wrapper!(
    /// Timestamp text preserved exactly from a trusted application boundary.
    RecordedAt,
    "recorded-at timestamp"
);

/// Semantic version of rule behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RuleVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl RuleVersion {
    /// Creates a semantic rule version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns the major component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Returns the patch component.
    #[must_use]
    pub const fn patch(self) -> u16 {
        self.patch
    }
}

impl Display for RuleVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A stable rule ID paired with the exact behavior version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuleRef {
    id: RuleId,
    version: RuleVersion,
}

impl RuleRef {
    /// Creates a rule reference.
    #[must_use]
    pub const fn new(id: RuleId, version: RuleVersion) -> Self {
        Self { id, version }
    }

    /// Returns the stable rule ID.
    #[must_use]
    pub const fn id(&self) -> &RuleId {
        &self.id
    }

    /// Returns the behavior version.
    #[must_use]
    pub const fn version(&self) -> RuleVersion {
        self.version
    }
}

/// Version of a serialized schema contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    /// Creates a nonzero schema version.
    pub fn new(value: u16) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidValue {
                field: "schema version",
                reason: "must be greater than zero",
            });
        }
        Ok(Self(value))
    }

    /// Returns the numeric schema version.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Origin category for a significant value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// Directly measured evidence.
    Measured,
    /// Imported evidence.
    Imported,
    /// Deterministic calculated evidence.
    Calculated,
    /// System-proposed evidence requiring review.
    Suggested,
    /// Historical evidence.
    Historical,
    /// Human-entered evidence.
    Manual,
}

/// Versioned provenance for a value or snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceRef {
    kind: SourceKind,
    source_id: String,
    revision: Option<String>,
    recorded_at: Option<RecordedAt>,
}

#[derive(Deserialize)]
struct SourceRefWire {
    kind: SourceKind,
    source_id: String,
    revision: Option<String>,
    recorded_at: Option<RecordedAt>,
}

impl<'de> Deserialize<'de> for SourceRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SourceRefWire::deserialize(deserializer)?;
        Self::new(wire.kind, wire.source_id, wire.revision, wire.recorded_at)
            .map_err(serde::de::Error::custom)
    }
}

impl SourceRef {
    /// Validates and creates a source reference.
    pub fn new(
        kind: SourceKind,
        source_id: impl Into<String>,
        revision: Option<String>,
        recorded_at: Option<RecordedAt>,
    ) -> Result<Self, DomainError> {
        let source_id = source_id.into();
        if source_id.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "source ID",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            kind,
            source_id,
            revision,
            recorded_at,
        })
    }

    /// Returns the source category.
    #[must_use]
    pub const fn kind(&self) -> SourceKind {
        self.kind
    }

    /// Returns the source identifier.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Returns the optional source revision.
    #[must_use]
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    /// Returns the optional recorded-at timestamp.
    #[must_use]
    pub const fn recorded_at(&self) -> Option<&RecordedAt> {
        self.recorded_at.as_ref()
    }
}
