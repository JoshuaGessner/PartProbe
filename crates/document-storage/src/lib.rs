//! Immutable blob, governed derivative-manifest, and storage-port contracts.

use std::fmt::Write as _;

use partprobe_domain::{
    ActorId, DataClassificationId, DomainError, RecordId, RecordVersionId, RecordedAt,
    SchemaVersion,
};
use partprobe_security::{AuditCorrelationId, SecurityPolicyRef};
use sha2::{Digest, Sha256};

macro_rules! storage_token {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Validates a bounded machine-readable storage token.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 256
                    || !value.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'-' | b'_' | b'.' | b':' | b'+' | b'/')
                    })
                {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must be a 1-256 byte ASCII machine token",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated token.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

storage_token!(
    /// Stable schema-family identity for derivative payload bytes.
    ArtifactSchemaId,
    "artifact schema ID"
);
storage_token!(
    /// Opaque reference that links a derivative to the producing analysis result.
    DerivativeReference,
    "derivative reference"
);
storage_token!(
    /// Stable retention-policy identity selected by the application.
    RetentionPolicyId,
    "retention policy ID"
);
storage_token!(
    /// Opaque adapter-owned locator; never a user-visible filesystem path.
    DocumentLocator,
    "document locator"
);

/// Canonical lowercase type/subtype for derivative payload bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactMediaType(String);

impl ArtifactMediaType {
    /// Validates a bounded media type without parameters.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let mut components = value.split('/');
        let Some(top_level) = components.next() else {
            return Err(invalid_media_type());
        };
        let Some(subtype) = components.next() else {
            return Err(invalid_media_type());
        };
        if components.next().is_some()
            || top_level.is_empty()
            || subtype.is_empty()
            || value.len() > 127
            || !top_level.bytes().chain(subtype.bytes()).all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(
                        byte,
                        b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-'
                    )
            })
        {
            return Err(invalid_media_type());
        }
        Ok(Self(value))
    }

    /// Returns the canonical media type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn invalid_media_type() -> DomainError {
    DomainError::InvalidValue {
        field: "artifact media type",
        reason: "must be a lowercase ASCII type/subtype without parameters",
    }
}

/// Lowercase SHA-256 digest and canonical content address.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContentAddress(String);

impl ContentAddress {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let mut value = String::with_capacity(64);
        for byte in hasher.finalize() {
            write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Self(value)
    }

    /// Returns the lowercase SHA-256 digest used as the content address.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.0
    }
}

/// Nonempty immutable bytes whose content address and length were independently computed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmutableBlob {
    content_address: ContentAddress,
    bytes: Box<[u8]>,
}

impl ImmutableBlob {
    /// Revalidates claimed SHA-256 and byte length before accepting immutable bytes.
    pub fn from_claimed_sha256(
        claimed_sha256: &str,
        claimed_byte_length: u64,
        bytes: Box<[u8]>,
    ) -> Result<Self, DomainError> {
        if bytes.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "immutable document blob",
                reason: "must not be empty",
            });
        }
        let actual_byte_length =
            u64::try_from(bytes.len()).map_err(|_| DomainError::InvalidValue {
                field: "immutable document blob",
                reason: "byte length exceeds the supported range",
            })?;
        let content_address = ContentAddress::from_bytes(&bytes);
        if claimed_byte_length != actual_byte_length || claimed_sha256 != content_address.sha256() {
            return Err(DomainError::InvalidValue {
                field: "immutable document blob",
                reason: "claimed hash and length must match the supplied bytes",
            });
        }
        Ok(Self {
            content_address,
            bytes,
        })
    }

    /// Returns the computed content address.
    #[must_use]
    pub const fn content_address(&self) -> &ContentAddress {
        &self.content_address
    }

    /// Returns the verified byte length.
    #[must_use]
    pub fn byte_length(&self) -> u64 {
        u64::try_from(self.bytes.len()).expect("validated blob length must fit in u64")
    }

    /// Returns the immutable payload.
    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Versioned retention policy selected for one derivative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicyRef {
    id: RetentionPolicyId,
    version: u64,
}

impl RetentionPolicyRef {
    /// Creates a nonzero retention-policy reference.
    pub fn new(id: RetentionPolicyId, version: u64) -> Result<Self, DomainError> {
        if version == 0 {
            return Err(DomainError::InvalidValue {
                field: "retention policy version",
                reason: "must be greater than zero",
            });
        }
        Ok(Self { id, version })
    }

    /// Returns the policy identity.
    #[must_use]
    pub const fn id(&self) -> &RetentionPolicyId {
        &self.id
    }

    /// Returns the exact policy version.
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
}

/// Versioned payload schema identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactSchemaRef {
    id: ArtifactSchemaId,
    version: SchemaVersion,
}

impl ArtifactSchemaRef {
    /// Creates a schema reference.
    #[must_use]
    pub const fn new(id: ArtifactSchemaId, version: SchemaVersion) -> Self {
        Self { id, version }
    }

    /// Returns the schema family.
    #[must_use]
    pub const fn id(&self) -> &ArtifactSchemaId {
        &self.id
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn version(&self) -> SchemaVersion {
        self.version
    }
}

/// Stable artifact/source identities and producer reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivativeIdentity {
    artifact_id: RecordId,
    artifact_version_id: RecordVersionId,
    source_record_id: RecordId,
    source_record_version_id: RecordVersionId,
    derivative_reference: DerivativeReference,
}

impl DerivativeIdentity {
    /// Creates explicit immutable derivative lineage.
    #[must_use]
    pub const fn new(
        artifact_id: RecordId,
        artifact_version_id: RecordVersionId,
        source_record_id: RecordId,
        source_record_version_id: RecordVersionId,
        derivative_reference: DerivativeReference,
    ) -> Self {
        Self {
            artifact_id,
            artifact_version_id,
            source_record_id,
            source_record_version_id,
            derivative_reference,
        }
    }

    /// Returns the derivative record identity.
    #[must_use]
    pub const fn artifact_id(&self) -> &RecordId {
        &self.artifact_id
    }

    /// Returns the immutable derivative version identity.
    #[must_use]
    pub const fn artifact_version_id(&self) -> &RecordVersionId {
        &self.artifact_version_id
    }

    /// Returns the source record identity.
    #[must_use]
    pub const fn source_record_id(&self) -> &RecordId {
        &self.source_record_id
    }

    /// Returns the source record version.
    #[must_use]
    pub const fn source_record_version_id(&self) -> &RecordVersionId {
        &self.source_record_version_id
    }

    /// Returns the producing analysis reference.
    #[must_use]
    pub const fn derivative_reference(&self) -> &DerivativeReference {
        &self.derivative_reference
    }
}

/// Classification, access, retention, and creation evidence for one derivative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivativeGovernance {
    classification_id: DataClassificationId,
    access_policy: SecurityPolicyRef,
    retention_policy: RetentionPolicyRef,
    authorization_correlation_id: AuditCorrelationId,
    created_by: ActorId,
    created_at: RecordedAt,
}

impl DerivativeGovernance {
    /// Creates complete governance evidence without inferring defaults.
    #[must_use]
    pub const fn new(
        classification_id: DataClassificationId,
        access_policy: SecurityPolicyRef,
        retention_policy: RetentionPolicyRef,
        authorization_correlation_id: AuditCorrelationId,
        created_by: ActorId,
        created_at: RecordedAt,
    ) -> Self {
        Self {
            classification_id,
            access_policy,
            retention_policy,
            authorization_correlation_id,
            created_by,
            created_at,
        }
    }

    /// Returns the inherited classification.
    #[must_use]
    pub const fn classification_id(&self) -> &DataClassificationId {
        &self.classification_id
    }

    /// Returns the exact access policy selected at persistence time.
    #[must_use]
    pub const fn access_policy(&self) -> &SecurityPolicyRef {
        &self.access_policy
    }

    /// Returns the exact retention policy selected at persistence time.
    #[must_use]
    pub const fn retention_policy(&self) -> &RetentionPolicyRef {
        &self.retention_policy
    }

    /// Returns the correlation identity linking authorization and audit evidence.
    #[must_use]
    pub const fn authorization_correlation_id(&self) -> &AuditCorrelationId {
        &self.authorization_correlation_id
    }

    /// Returns the actor creating the manifest.
    #[must_use]
    pub const fn created_by(&self) -> &ActorId {
        &self.created_by
    }

    /// Returns the trusted application timestamp.
    #[must_use]
    pub const fn created_at(&self) -> &RecordedAt {
        &self.created_at
    }
}

/// Integrity state derivable at the application/store handoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DerivativeIntegrityState {
    /// Hash and byte length were independently recomputed from the immutable bytes.
    Verified,
}

/// Immutable manifest committed by a controlled derivative store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivativeManifest {
    identity: DerivativeIdentity,
    governance: DerivativeGovernance,
    schema: ArtifactSchemaRef,
    media_type: ArtifactMediaType,
    content_address: ContentAddress,
    byte_length: u64,
    integrity_state: DerivativeIntegrityState,
}

impl DerivativeManifest {
    fn from_write(
        identity: DerivativeIdentity,
        governance: DerivativeGovernance,
        schema: ArtifactSchemaRef,
        media_type: ArtifactMediaType,
        blob: &ImmutableBlob,
    ) -> Self {
        Self {
            identity,
            governance,
            schema,
            media_type,
            content_address: blob.content_address().clone(),
            byte_length: blob.byte_length(),
            integrity_state: DerivativeIntegrityState::Verified,
        }
    }

    /// Returns artifact/source lineage.
    #[must_use]
    pub const fn identity(&self) -> &DerivativeIdentity {
        &self.identity
    }

    /// Returns classification and policy evidence.
    #[must_use]
    pub const fn governance(&self) -> &DerivativeGovernance {
        &self.governance
    }

    /// Returns the payload schema.
    #[must_use]
    pub const fn schema(&self) -> &ArtifactSchemaRef {
        &self.schema
    }

    /// Returns the declared payload media type.
    #[must_use]
    pub const fn media_type(&self) -> &ArtifactMediaType {
        &self.media_type
    }

    /// Returns the content address.
    #[must_use]
    pub const fn content_address(&self) -> &ContentAddress {
        &self.content_address
    }

    /// Returns the verified byte length.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Returns the integrity state derived during independent blob validation.
    #[must_use]
    pub const fn integrity_state(&self) -> DerivativeIntegrityState {
        self.integrity_state
    }
}

/// Fully governed immutable bytes presented to a storage adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlledDerivativeWrite {
    manifest: DerivativeManifest,
    blob: ImmutableBlob,
}

impl ControlledDerivativeWrite {
    /// Creates a manifest whose integrity evidence is derived from the verified blob.
    #[must_use]
    pub fn new(
        identity: DerivativeIdentity,
        governance: DerivativeGovernance,
        schema: ArtifactSchemaRef,
        media_type: ArtifactMediaType,
        blob: ImmutableBlob,
    ) -> Self {
        let manifest =
            DerivativeManifest::from_write(identity, governance, schema, media_type, &blob);
        Self { manifest, blob }
    }

    /// Returns the immutable manifest.
    #[must_use]
    pub const fn manifest(&self) -> &DerivativeManifest {
        &self.manifest
    }

    /// Returns the independently verified blob.
    #[must_use]
    pub const fn blob(&self) -> &ImmutableBlob {
        &self.blob
    }
}

/// Adapter receipt for a committed immutable blob and manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredDerivative {
    manifest: DerivativeManifest,
    locator: DocumentLocator,
}

impl StoredDerivative {
    /// Creates an adapter receipt. Application services revalidate it against the write request.
    #[must_use]
    pub const fn new(manifest: DerivativeManifest, locator: DocumentLocator) -> Self {
        Self { manifest, locator }
    }

    /// Returns the committed manifest.
    #[must_use]
    pub const fn manifest(&self) -> &DerivativeManifest {
        &self.manifest
    }

    /// Returns the opaque storage locator.
    #[must_use]
    pub const fn locator(&self) -> &DocumentLocator {
        &self.locator
    }

    /// Returns whether the adapter receipt exactly matches the requested manifest.
    #[must_use]
    pub fn matches(&self, write: &ControlledDerivativeWrite) -> bool {
        self.manifest == write.manifest
    }
}

/// Controlled persistence port; implementations must commit blob and manifest before success.
pub trait ControlledDerivativeStore {
    /// Persists one governed immutable derivative.
    fn persist(
        &self,
        write: &ControlledDerivativeWrite,
    ) -> Result<StoredDerivative, DerivativeStoreError>;
}

/// Content-free storage failure category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DerivativeStoreError {
    /// The backing store could not complete the operation.
    Unavailable,
    /// Existing or returned storage evidence conflicted with the requested manifest.
    IntegrityConflict,
}
