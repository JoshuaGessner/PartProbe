use partprobe_domain::{
    ActorId, DomainError, RecordedAt, ResourceSelection, ResourceSelectionId,
    ResourceSelectionState, ShopProfileId, ShopResourceCatalog, ShopResourceCatalogId,
    ShopResourceVersion, ShopSettingsDraft, ShopSettingsRevision,
};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationDecision, AuthorizationOutcome,
    AuthorizationReasonCode, SecurityPolicyRef,
};

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

/// The single protected catalog mutation currently exposed by the application layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShopResourceCatalogOperation {
    /// Makes one exact reviewed selection eligible for future proposal generation only.
    ActivateForProposals,
}

/// Exact, content-minimized policy facts for one catalog activation attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShopResourceCatalogAuthorizationContext {
    profile_id: ShopProfileId,
    settings_revision: ShopSettingsRevision,
    catalog_id: ShopResourceCatalogId,
    catalog_version: ShopResourceVersion,
    selection_id: ResourceSelectionId,
    selection_version: ShopResourceVersion,
    actor_id: ActorId,
    recorded_at: RecordedAt,
    correlation_id: AuditCorrelationId,
    operation: ShopResourceCatalogOperation,
}

impl ShopResourceCatalogAuthorizationContext {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        profile_id: ShopProfileId,
        settings_revision: ShopSettingsRevision,
        catalog_id: ShopResourceCatalogId,
        catalog_version: ShopResourceVersion,
        selection_id: ResourceSelectionId,
        selection_version: ShopResourceVersion,
        actor_id: ActorId,
        recorded_at: RecordedAt,
        correlation_id: AuditCorrelationId,
    ) -> Self {
        Self {
            profile_id,
            settings_revision,
            catalog_id,
            catalog_version,
            selection_id,
            selection_version,
            actor_id,
            recorded_at,
            correlation_id,
            operation: ShopResourceCatalogOperation::ActivateForProposals,
        }
    }

    #[must_use]
    pub const fn profile_id(&self) -> &ShopProfileId {
        &self.profile_id
    }

    #[must_use]
    pub const fn settings_revision(&self) -> ShopSettingsRevision {
        self.settings_revision
    }

    #[must_use]
    pub const fn catalog_id(&self) -> &ShopResourceCatalogId {
        &self.catalog_id
    }

    #[must_use]
    pub const fn catalog_version(&self) -> ShopResourceVersion {
        self.catalog_version
    }

    #[must_use]
    pub const fn selection_id(&self) -> &ResourceSelectionId {
        &self.selection_id
    }

    #[must_use]
    pub const fn selection_version(&self) -> ShopResourceVersion {
        self.selection_version
    }

    #[must_use]
    pub const fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    #[must_use]
    pub const fn recorded_at(&self) -> &RecordedAt {
        &self.recorded_at
    }

    #[must_use]
    pub const fn correlation_id(&self) -> &AuditCorrelationId {
        &self.correlation_id
    }

    #[must_use]
    pub const fn operation(&self) -> ShopResourceCatalogOperation {
        self.operation
    }
}

/// Deployment policy port for catalog proposal-eligibility decisions.
pub trait ShopResourceCatalogAuthorizationPolicy {
    /// Evaluates one exact current catalog/selection version without mutating it.
    fn evaluate(&self, context: &ShopResourceCatalogAuthorizationContext) -> AuthorizationDecision;
}

/// Fail-closed baseline for hosts that have not configured catalog authorization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DenyAllShopResourceCatalogAuthorizationPolicy {
    policy: SecurityPolicyRef,
    reason_code: AuthorizationReasonCode,
}

impl DenyAllShopResourceCatalogAuthorizationPolicy {
    #[must_use]
    pub const fn new(policy: SecurityPolicyRef, reason_code: AuthorizationReasonCode) -> Self {
        Self {
            policy,
            reason_code,
        }
    }
}

impl ShopResourceCatalogAuthorizationPolicy for DenyAllShopResourceCatalogAuthorizationPolicy {
    fn evaluate(
        &self,
        _context: &ShopResourceCatalogAuthorizationContext,
    ) -> AuthorizationDecision {
        AuthorizationDecision::deny(self.policy.clone(), self.reason_code.clone())
    }
}

/// Append-preserving evidence for one catalog authorization decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShopResourceCatalogAuthorizationEvent {
    context: ShopResourceCatalogAuthorizationContext,
    decision: AuthorizationDecision,
}

impl ShopResourceCatalogAuthorizationEvent {
    #[must_use]
    pub const fn new(
        context: ShopResourceCatalogAuthorizationContext,
        decision: AuthorizationDecision,
    ) -> Self {
        Self { context, decision }
    }

    #[must_use]
    pub const fn context(&self) -> &ShopResourceCatalogAuthorizationContext {
        &self.context
    }

    #[must_use]
    pub const fn decision(&self) -> &AuthorizationDecision {
        &self.decision
    }
}

/// Audit port that must accept the decision before catalog state may change.
pub trait ShopResourceCatalogAuthorizationAuditSink {
    fn append(&self, event: ShopResourceCatalogAuthorizationEvent) -> Result<(), AuditAppendError>;
}

/// Exact optimistic-concurrency and human-decision evidence for catalog activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivateShopResourceSelectionRequest {
    profile_id: ShopProfileId,
    expected_settings_revision: ShopSettingsRevision,
    expected_catalog_id: ShopResourceCatalogId,
    expected_catalog_version: ShopResourceVersion,
    selection_id: ResourceSelectionId,
    selection_version: ShopResourceVersion,
    actor_id: ActorId,
    recorded_at: RecordedAt,
    reason: String,
    correlation_id: AuditCorrelationId,
}

impl ActivateShopResourceSelectionRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile_id: ShopProfileId,
        expected_settings_revision: ShopSettingsRevision,
        expected_catalog_id: ShopResourceCatalogId,
        expected_catalog_version: ShopResourceVersion,
        selection_id: ResourceSelectionId,
        selection_version: ShopResourceVersion,
        actor_id: ActorId,
        recorded_at: RecordedAt,
        reason: impl Into<String>,
        correlation_id: AuditCorrelationId,
    ) -> Result<Self, DomainError> {
        let reason = reason.into();
        if reason.trim().is_empty() || reason.len() > 1_024 || reason.contains('\0') {
            return Err(DomainError::InvalidValue {
                field: "resource-selection activation reason",
                reason: "must be 1-1024 bytes, nonblank, and contain no NUL",
            });
        }
        Ok(Self {
            profile_id,
            expected_settings_revision,
            expected_catalog_id,
            expected_catalog_version,
            selection_id,
            selection_version,
            actor_id,
            recorded_at,
            reason,
            correlation_id,
        })
    }
}

/// Content-free failures from the governed catalog activation use case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShopResourceCatalogActivationError {
    NotConfigured,
    CatalogUnavailable,
    StaleSettings,
    StaleCatalog,
    SelectionUnavailable,
    SelectionNotReviewed,
    VersionOverflow,
    Denied(AuthorizationReasonCode),
    AuditUnavailable,
    InvalidTransition,
    Store(ShopSettingsStoreError),
}

/// Application-owned service for one policy-evaluated, audited catalog decision.
pub struct ShopResourceCatalogApplication<R, P, A> {
    repository: R,
    policy: P,
    audit: A,
}

impl<R, P, A> ShopResourceCatalogApplication<R, P, A>
where
    R: ShopSettingsDraftRepository,
    P: ShopResourceCatalogAuthorizationPolicy,
    A: ShopResourceCatalogAuthorizationAuditSink,
{
    #[must_use]
    pub const fn new(repository: R, policy: P, audit: A) -> Self {
        Self {
            repository,
            policy,
            audit,
        }
    }

    /// Activates one reviewed selection for proposal generation only.
    ///
    /// The exact current settings/catalog/selection versions are checked before policy
    /// evaluation. The policy decision must be appended before any durable mutation.
    pub fn activate_for_proposals(
        &mut self,
        request: &ActivateShopResourceSelectionRequest,
    ) -> Result<ShopSettingsDraft, ShopResourceCatalogActivationError> {
        let current = self
            .repository
            .current(&request.profile_id)
            .map_err(ShopResourceCatalogActivationError::Store)?
            .ok_or(ShopResourceCatalogActivationError::NotConfigured)?;
        if current.revision() != request.expected_settings_revision {
            return Err(ShopResourceCatalogActivationError::StaleSettings);
        }
        let catalog = current
            .resource_catalog()
            .ok_or(ShopResourceCatalogActivationError::CatalogUnavailable)?;
        if catalog.id() != &request.expected_catalog_id
            || catalog.version() != request.expected_catalog_version
        {
            return Err(ShopResourceCatalogActivationError::StaleCatalog);
        }
        let selected = catalog
            .selections()
            .iter()
            .find(|selection| {
                selection.id() == &request.selection_id
                    && selection.version() == request.selection_version
            })
            .ok_or(ShopResourceCatalogActivationError::SelectionUnavailable)?;
        if selected.state() != ResourceSelectionState::Reviewed {
            return Err(ShopResourceCatalogActivationError::SelectionNotReviewed);
        }

        let next_settings_revision = increment_settings_revision(current.revision())?;
        let next_catalog_version = increment_resource_version(catalog.version())?;
        let selections = successor_selections(catalog, selected, request)?;
        let next_catalog = ShopResourceCatalog::new(
            catalog.id().clone(),
            next_catalog_version,
            catalog.currency().clone(),
            catalog.materials().to_vec(),
            catalog.material_offers().to_vec(),
            catalog.stock_allowances().to_vec(),
            catalog.machines().to_vec(),
            catalog.runtimes().to_vec(),
            selections,
        )
        .map_err(|_| ShopResourceCatalogActivationError::InvalidTransition)?;
        let next = ShopSettingsDraft::new_with_catalog(
            current.profile_id().clone(),
            next_settings_revision,
            current.currency().clone(),
            current.rate_card().cloned(),
            current.pricing_policy().cloned(),
            Some(next_catalog),
            request.actor_id.clone(),
            request.recorded_at.clone(),
            &request.reason,
        )
        .map_err(|_| ShopResourceCatalogActivationError::InvalidTransition)?;

        let context = ShopResourceCatalogAuthorizationContext::new(
            request.profile_id.clone(),
            request.expected_settings_revision,
            request.expected_catalog_id.clone(),
            request.expected_catalog_version,
            request.selection_id.clone(),
            request.selection_version,
            request.actor_id.clone(),
            request.recorded_at.clone(),
            request.correlation_id.clone(),
        );
        let decision = self.policy.evaluate(&context);
        self.audit
            .append(ShopResourceCatalogAuthorizationEvent::new(
                context,
                decision.clone(),
            ))
            .map_err(|_| ShopResourceCatalogActivationError::AuditUnavailable)?;
        if decision.outcome() == AuthorizationOutcome::Denied {
            return Err(ShopResourceCatalogActivationError::Denied(
                decision.reason_code().clone(),
            ));
        }

        self.repository
            .save(&next, Some(request.expected_settings_revision))
            .map_err(ShopResourceCatalogActivationError::Store)?;
        Ok(next)
    }

    #[must_use]
    pub fn into_parts(self) -> (R, P, A) {
        (self.repository, self.policy, self.audit)
    }
}

fn successor_selections(
    catalog: &ShopResourceCatalog,
    selected: &ResourceSelection,
    request: &ActivateShopResourceSelectionRequest,
) -> Result<Vec<ResourceSelection>, ShopResourceCatalogActivationError> {
    catalog
        .selections()
        .iter()
        .map(|selection| {
            let next_state =
                if selection.id() == selected.id() && selection.version() == selected.version() {
                    Some(ResourceSelectionState::ActiveForProposals)
                } else if selection.state() == ResourceSelectionState::ActiveForProposals {
                    Some(ResourceSelectionState::Reviewed)
                } else {
                    None
                };
            next_state.map_or_else(
                || Ok(selection.clone()),
                |state| {
                    ResourceSelection::new(
                        selection.id().clone(),
                        increment_resource_version(selection.version())?,
                        selection.material_id().clone(),
                        selection.material_version(),
                        selection.material_offer_id().clone(),
                        selection.material_offer_version(),
                        selection.stock_allowance_id().clone(),
                        selection.stock_allowance_version(),
                        selection.machine_id().clone(),
                        selection.machine_version(),
                        selection.runtime_id().clone(),
                        selection.runtime_version(),
                        state,
                        request.actor_id.clone(),
                        request.recorded_at.clone(),
                        &request.reason,
                    )
                    .map_err(|_| ShopResourceCatalogActivationError::InvalidTransition)
                },
            )
        })
        .collect()
}

fn increment_settings_revision(
    version: ShopSettingsRevision,
) -> Result<ShopSettingsRevision, ShopResourceCatalogActivationError> {
    version
        .value()
        .checked_add(1)
        .ok_or(ShopResourceCatalogActivationError::VersionOverflow)
        .and_then(|value| {
            ShopSettingsRevision::new(value)
                .map_err(|_| ShopResourceCatalogActivationError::VersionOverflow)
        })
}

fn increment_resource_version(
    version: ShopResourceVersion,
) -> Result<ShopResourceVersion, ShopResourceCatalogActivationError> {
    version
        .value()
        .checked_add(1)
        .ok_or(ShopResourceCatalogActivationError::VersionOverflow)
        .and_then(|value| {
            ShopResourceVersion::new(value)
                .map_err(|_| ShopResourceCatalogActivationError::VersionOverflow)
        })
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use partprobe_security::{SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion};
    use partprobe_test_support::resource_catalog_fixture;

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

    #[derive(Clone)]
    struct MemoryRepository {
        current: Option<ShopSettingsDraft>,
    }

    impl ShopSettingsDraftRepository for MemoryRepository {
        fn save(
            &mut self,
            draft: &ShopSettingsDraft,
            expected_current_revision: Option<ShopSettingsRevision>,
        ) -> Result<(), ShopSettingsStoreError> {
            if self.current.as_ref().map(ShopSettingsDraft::revision) != expected_current_revision {
                return Err(ShopSettingsStoreError::RevisionConflict);
            }
            self.current = Some(draft.clone());
            Ok(())
        }

        fn current(
            &self,
            _profile_id: &ShopProfileId,
        ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
            Ok(self.current.clone())
        }

        fn revision(
            &self,
            _profile_id: &ShopProfileId,
            revision: ShopSettingsRevision,
        ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
            Ok(self
                .current
                .as_ref()
                .filter(|draft| draft.revision() == revision)
                .cloned())
        }
    }

    #[derive(Clone)]
    struct FixedPolicy {
        decision: AuthorizationDecision,
    }

    impl ShopResourceCatalogAuthorizationPolicy for FixedPolicy {
        fn evaluate(
            &self,
            _context: &ShopResourceCatalogAuthorizationContext,
        ) -> AuthorizationDecision {
            self.decision.clone()
        }
    }

    #[derive(Clone, Default)]
    struct RecordingAudit {
        events: Rc<RefCell<Vec<ShopResourceCatalogAuthorizationEvent>>>,
        fail: bool,
    }

    impl ShopResourceCatalogAuthorizationAuditSink for RecordingAudit {
        fn append(
            &self,
            event: ShopResourceCatalogAuthorizationEvent,
        ) -> Result<(), AuditAppendError> {
            if self.fail {
                return Err(AuditAppendError);
            }
            self.events.borrow_mut().push(event);
            Ok(())
        }
    }

    fn profile() -> ShopProfileId {
        ShopProfileId::new("shop-1").expect("profile")
    }

    fn settings_with_catalog(state: ResourceSelectionState) -> ShopSettingsDraft {
        ShopSettingsDraft::new_with_catalog(
            profile(),
            ShopSettingsRevision::new(1).expect("settings revision"),
            partprobe_domain::CurrencyCode::new("USD").expect("currency"),
            None,
            None,
            Some(resource_catalog_fixture(1, 1, "2700", 1, state)),
            ActorId::new("catalog-author").expect("actor"),
            RecordedAt::new("2026-09-09T12:00:00Z").expect("time"),
            "initial reviewed catalog",
        )
        .expect("settings draft")
    }

    fn settings_with_active_and_reviewed_candidate() -> ShopSettingsDraft {
        let catalog =
            resource_catalog_fixture(1, 1, "2700", 1, ResourceSelectionState::ActiveForProposals);
        let active = &catalog.selections()[0];
        let candidate = ResourceSelection::new(
            ResourceSelectionId::new("reviewed-candidate").expect("selection ID"),
            ShopResourceVersion::new(1).expect("selection version"),
            active.material_id().clone(),
            active.material_version(),
            active.material_offer_id().clone(),
            active.material_offer_version(),
            active.stock_allowance_id().clone(),
            active.stock_allowance_version(),
            active.machine_id().clone(),
            active.machine_version(),
            active.runtime_id().clone(),
            active.runtime_version(),
            ResourceSelectionState::Reviewed,
            ActorId::new("catalog-author").expect("actor"),
            RecordedAt::new("2026-09-09T13:00:00Z").expect("time"),
            "reviewed alternative",
        )
        .expect("candidate");
        let catalog = ShopResourceCatalog::new(
            catalog.id().clone(),
            catalog.version(),
            catalog.currency().clone(),
            catalog.materials().to_vec(),
            catalog.material_offers().to_vec(),
            catalog.stock_allowances().to_vec(),
            catalog.machines().to_vec(),
            catalog.runtimes().to_vec(),
            vec![active.clone(), candidate],
        )
        .expect("catalog");
        ShopSettingsDraft::new_with_catalog(
            profile(),
            ShopSettingsRevision::new(1).expect("settings revision"),
            partprobe_domain::CurrencyCode::new("USD").expect("currency"),
            None,
            None,
            Some(catalog),
            ActorId::new("catalog-author").expect("actor"),
            RecordedAt::new("2026-09-09T14:00:00Z").expect("time"),
            "catalog with reviewed alternative",
        )
        .expect("settings")
    }

    fn request(selection_id: &str) -> ActivateShopResourceSelectionRequest {
        ActivateShopResourceSelectionRequest::new(
            profile(),
            ShopSettingsRevision::new(1).expect("settings revision"),
            ShopResourceCatalogId::new("test-resource-catalog").expect("catalog ID"),
            ShopResourceVersion::new(1).expect("catalog version"),
            ResourceSelectionId::new(selection_id).expect("selection ID"),
            ShopResourceVersion::new(1).expect("selection version"),
            ActorId::new("catalog-reviewer").expect("actor"),
            RecordedAt::new("2026-09-09T16:00:00Z").expect("time"),
            "adopt reviewed resources for proposal generation",
            AuditCorrelationId::new("catalog-activation-1").expect("correlation"),
        )
        .expect("activation request")
    }

    fn policy_ref() -> SecurityPolicyRef {
        SecurityPolicyRef::new(
            SecurityPolicyId::new("catalog-policy").expect("policy ID"),
            SecurityPolicyVersion::new(1).expect("policy version"),
        )
    }

    fn allowed_policy_decision() -> AuthorizationDecision {
        let policy = policy_ref();
        let reason = AuthorizationReasonCode::new("CATALOG_REVIEW_ALLOWED").expect("reason code");
        AuthorizationDecision::allow(policy, reason)
    }

    fn assert_activation_rejected_before_audit(
        request: &ActivateShopResourceSelectionRequest,
        expected: ShopResourceCatalogActivationError,
    ) {
        let initial = settings_with_catalog(ResourceSelectionState::Reviewed);
        let audit = RecordingAudit::default();
        let audit_view = audit.clone();
        let mut application = ShopResourceCatalogApplication::new(
            MemoryRepository {
                current: Some(initial.clone()),
            },
            FixedPolicy {
                decision: allowed_policy_decision(),
            },
            audit,
        );
        assert_eq!(application.activate_for_proposals(request), Err(expected));
        let (repository, _, _) = application.into_parts();
        assert_eq!(repository.current, Some(initial));
        assert!(audit_view.events.borrow().is_empty());
    }

    #[test]
    fn allowed_activation_revisions_selection_catalog_and_settings_after_audit() {
        let repository = MemoryRepository {
            current: Some(settings_with_catalog(ResourceSelectionState::Reviewed)),
        };
        let audit = RecordingAudit::default();
        let audit_view = audit.clone();
        let mut application = ShopResourceCatalogApplication::new(
            repository,
            FixedPolicy {
                decision: allowed_policy_decision(),
            },
            audit,
        );

        let activated = application
            .activate_for_proposals(&request("test-resource-selection"))
            .expect("activate reviewed selection");

        assert_eq!(activated.revision().value(), 2);
        assert_eq!(activated.changed_by().as_str(), "catalog-reviewer");
        let catalog = activated.resource_catalog().expect("catalog");
        assert_eq!(catalog.version().value(), 2);
        let selected = catalog.active_selection().expect("active selection");
        assert_eq!(selected.version().value(), 2);
        assert_eq!(selected.decided_by().as_str(), "catalog-reviewer");
        assert_eq!(
            selected.reason(),
            "adopt reviewed resources for proposal generation"
        );
        let events = audit_view.events.borrow();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].context().operation(),
            ShopResourceCatalogOperation::ActivateForProposals
        );
        assert_eq!(
            events[0].context().correlation_id().as_str(),
            "catalog-activation-1"
        );
        assert_eq!(
            events[0].decision().outcome(),
            AuthorizationOutcome::Allowed
        );
    }

    #[test]
    fn denied_activation_is_audited_without_changing_current_state() {
        let initial = settings_with_catalog(ResourceSelectionState::Reviewed);
        let repository = MemoryRepository {
            current: Some(initial.clone()),
        };
        let audit = RecordingAudit::default();
        let audit_view = audit.clone();
        let mut application = ShopResourceCatalogApplication::new(
            repository,
            DenyAllShopResourceCatalogAuthorizationPolicy::new(
                policy_ref(),
                AuthorizationReasonCode::new("CATALOG_REVIEW_DENIED").expect("reason"),
            ),
            audit,
        );

        assert_eq!(
            application.activate_for_proposals(&request("test-resource-selection")),
            Err(ShopResourceCatalogActivationError::Denied(
                AuthorizationReasonCode::new("CATALOG_REVIEW_DENIED").expect("reason")
            ))
        );
        let (repository, _, _) = application.into_parts();
        assert_eq!(repository.current, Some(initial));
        assert_eq!(audit_view.events.borrow().len(), 1);
        assert_eq!(
            audit_view.events.borrow()[0].decision().outcome(),
            AuthorizationOutcome::Denied
        );
    }

    #[test]
    fn audit_failure_blocks_an_allowed_activation() {
        let initial = settings_with_catalog(ResourceSelectionState::Reviewed);
        let repository = MemoryRepository {
            current: Some(initial.clone()),
        };
        let mut application = ShopResourceCatalogApplication::new(
            repository,
            FixedPolicy {
                decision: allowed_policy_decision(),
            },
            RecordingAudit {
                fail: true,
                ..RecordingAudit::default()
            },
        );

        assert_eq!(
            application.activate_for_proposals(&request("test-resource-selection")),
            Err(ShopResourceCatalogActivationError::AuditUnavailable)
        );
        let (repository, _, _) = application.into_parts();
        assert_eq!(repository.current, Some(initial));
    }

    #[test]
    fn already_active_selection_cannot_be_silently_reauthorized() {
        let initial = settings_with_catalog(ResourceSelectionState::ActiveForProposals);
        let repository = MemoryRepository {
            current: Some(initial.clone()),
        };
        let audit = RecordingAudit::default();
        let audit_view = audit.clone();
        let mut application = ShopResourceCatalogApplication::new(
            repository,
            FixedPolicy {
                decision: allowed_policy_decision(),
            },
            audit,
        );

        assert_eq!(
            application.activate_for_proposals(&request("test-resource-selection")),
            Err(ShopResourceCatalogActivationError::SelectionNotReviewed)
        );
        let (repository, _, _) = application.into_parts();
        assert_eq!(repository.current, Some(initial));
        assert!(audit_view.events.borrow().is_empty());
    }

    #[test]
    fn stale_settings_catalog_and_selection_versions_fail_before_audit() {
        let mut stale_settings = request("test-resource-selection");
        stale_settings.expected_settings_revision =
            ShopSettingsRevision::new(2).expect("settings revision");
        assert_activation_rejected_before_audit(
            &stale_settings,
            ShopResourceCatalogActivationError::StaleSettings,
        );

        let mut stale_catalog = request("test-resource-selection");
        stale_catalog.expected_catalog_version =
            ShopResourceVersion::new(2).expect("catalog version");
        assert_activation_rejected_before_audit(
            &stale_catalog,
            ShopResourceCatalogActivationError::StaleCatalog,
        );

        let mut stale_selection = request("test-resource-selection");
        stale_selection.selection_version = ShopResourceVersion::new(2).expect("selection version");
        assert_activation_rejected_before_audit(
            &stale_selection,
            ShopResourceCatalogActivationError::SelectionUnavailable,
        );
    }

    #[test]
    fn activating_an_alternative_revisions_the_prior_active_decision_to_reviewed() {
        let repository = MemoryRepository {
            current: Some(settings_with_active_and_reviewed_candidate()),
        };
        let mut application = ShopResourceCatalogApplication::new(
            repository,
            FixedPolicy {
                decision: allowed_policy_decision(),
            },
            RecordingAudit::default(),
        );

        let activated = application
            .activate_for_proposals(&request("reviewed-candidate"))
            .expect("activate reviewed alternative");
        let selections = activated.resource_catalog().expect("catalog").selections();
        assert_eq!(selections.len(), 2);
        let prior = selections
            .iter()
            .find(|selection| selection.id().as_str() == "test-resource-selection")
            .expect("prior selection");
        assert_eq!(prior.version().value(), 2);
        assert_eq!(prior.state(), ResourceSelectionState::Reviewed);
        let active = activated
            .resource_catalog()
            .expect("catalog")
            .active_selection()
            .expect("new active selection");
        assert_eq!(active.id().as_str(), "reviewed-candidate");
        assert_eq!(active.version().value(), 2);
    }
}
