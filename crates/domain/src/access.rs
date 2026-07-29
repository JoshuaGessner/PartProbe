use serde::{Deserialize, Deserializer, Serialize};

use crate::DomainError;

macro_rules! access_id {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and creates an opaque access-control identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() || value.len() > 256 || value.contains('\0') {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must be nonempty, bounded to 256 bytes, and contain no null byte",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the opaque identifier.
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

access_id!(
    /// Stable identity of an authenticated or local-policy actor.
    ActorId,
    "actor ID"
);
access_id!(
    /// Stable project identity used by authorization policy.
    ProjectId,
    "project ID"
);
access_id!(
    /// Stable identity of a protected business record or asset.
    RecordId,
    "record ID"
);
access_id!(
    /// Immutable version identity of a protected record.
    RecordVersionId,
    "record version ID"
);
access_id!(
    /// Deployment-defined data-classification identity.
    DataClassificationId,
    "data classification ID"
);
access_id!(
    /// Deployment-defined record-state identity.
    RecordStateId,
    "record state ID"
);
access_id!(
    /// Stable identity of an application-approved local asset root.
    AssetRootId,
    "asset root ID"
);
