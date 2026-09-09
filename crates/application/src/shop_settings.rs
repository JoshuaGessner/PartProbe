use partprobe_domain::{ShopProfileId, ShopSettingsDraft, ShopSettingsRevision};

/// Content-free failures at the durable shop-settings boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShopSettingsStoreError {
    /// The caller's expected current revision did not match durable state.
    RevisionConflict,
    /// Immutable identity or payload evidence conflicted with an existing record.
    IntegrityConflict,
    /// Stored bytes failed schema, hash, or domain validation.
    InvalidStoredRecord,
    /// The database schema is unknown, newer, or has changed checksums.
    UnsupportedSchema,
    /// Local durable storage could not complete the operation.
    Unavailable,
}

/// Application-owned persistence port for immutable shop-settings drafts.
pub trait ShopSettingsDraftRepository {
    /// Appends one revision and atomically advances the current pointer.
    fn save(
        &mut self,
        draft: &ShopSettingsDraft,
        expected_current_revision: Option<ShopSettingsRevision>,
    ) -> Result<(), ShopSettingsStoreError>;

    /// Loads the current draft, if a profile has one.
    fn current(
        &self,
        profile_id: &ShopProfileId,
    ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError>;

    /// Loads an immutable historical revision for deterministic replay.
    fn revision(
        &self,
        profile_id: &ShopProfileId,
        revision: ShopSettingsRevision,
    ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError>;
}

/// Explicit first-run state returned to Settings callers without inventing defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShopSettingsLoadState {
    /// No durable draft exists for this profile.
    NotConfigured,
    /// The current immutable draft was loaded and revalidated.
    Available(Box<ShopSettingsDraft>),
}

/// Typed application service that is the only intended desktop entry to shop-settings storage.
pub struct ShopSettingsApplication<R> {
    repository: R,
}

impl<R> ShopSettingsApplication<R>
where
    R: ShopSettingsDraftRepository,
{
    /// Creates the service around one deployment-owned repository adapter.
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Loads the current revision while preserving first-run absence explicitly.
    pub fn load(
        &self,
        profile_id: &ShopProfileId,
    ) -> Result<ShopSettingsLoadState, ShopSettingsStoreError> {
        self.repository.current(profile_id).map(|draft| {
            draft.map_or(ShopSettingsLoadState::NotConfigured, |value| {
                ShopSettingsLoadState::Available(Box::new(value))
            })
        })
    }

    /// Saves a validated immutable revision with an explicit optimistic-concurrency expectation.
    pub fn save(
        &mut self,
        draft: &ShopSettingsDraft,
        expected_current_revision: Option<ShopSettingsRevision>,
    ) -> Result<(), ShopSettingsStoreError> {
        self.repository.save(draft, expected_current_revision)
    }

    /// Loads one immutable prior revision for replay or review.
    pub fn revision(
        &self,
        profile_id: &ShopProfileId,
        revision: ShopSettingsRevision,
    ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
        self.repository.revision(profile_id, revision)
    }

    /// Returns the adapter for deployment lifecycle or test inspection.
    #[must_use]
    pub fn into_repository(self) -> R {
        self.repository
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct EmptyRepository;

    impl ShopSettingsDraftRepository for EmptyRepository {
        fn save(
            &mut self,
            _draft: &ShopSettingsDraft,
            _expected_current_revision: Option<ShopSettingsRevision>,
        ) -> Result<(), ShopSettingsStoreError> {
            Err(ShopSettingsStoreError::Unavailable)
        }

        fn current(
            &self,
            _profile_id: &ShopProfileId,
        ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
            Ok(None)
        }

        fn revision(
            &self,
            _profile_id: &ShopProfileId,
            _revision: ShopSettingsRevision,
        ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
            Ok(None)
        }
    }

    #[test]
    fn first_run_is_not_configured_instead_of_defaulted() {
        let application = ShopSettingsApplication::new(EmptyRepository);
        let state = application
            .load(&ShopProfileId::new("shop-1").expect("valid profile"))
            .expect("load state");
        assert_eq!(state, ShopSettingsLoadState::NotConfigured);
    }
}
