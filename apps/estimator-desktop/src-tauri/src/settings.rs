use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use partprobe_application::{
    ActivateShopResourceSelectionRequest as ApplicationActivationRequest,
    SaveShopResourceCatalogDraftRequest as ApplicationCatalogDraftRequest,
    ShopResourceCatalogActivationError, ShopResourceCatalogApplication,
    ShopResourceCatalogAuthorizationContext, ShopResourceCatalogAuthorizationPolicy,
    ShopResourceCatalogDraftApplication,
    ShopResourceCatalogDraftContents as ApplicationCatalogDraftContents,
    ShopResourceCatalogDraftError, ShopSettingsApplication, ShopSettingsLoadState,
    ShopSettingsStoreError,
};
use partprobe_desktop_contract::{
    ActivateShopResourceSelectionRequest, DeveloperPricingInputFields, DeveloperRateInputFields,
    ExpectedShopResourceCatalogRevision, HostCommandError, SaveShopResourceCatalogDraftRequest,
    SaveShopSettingsRequest, ShopMachineSnapshot, ShopMaterialOfferSnapshot, ShopMaterialSnapshot,
    ShopResourceCatalogActivationResult, ShopResourceCatalogActivationUnavailableReason,
    ShopResourceCatalogDraftSaveResult, ShopResourceCatalogDraftUnavailableReason,
    ShopResourceCatalogSnapshot, ShopResourceInputFields, ShopResourceRecordState,
    ShopResourceSelectionSnapshot, ShopResourceSelectionState, ShopRuntimeSnapshot,
    ShopSettingsSnapshot, ShopSettingsState, ShopStockAllowanceSnapshot,
};
use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CostCategory, CurrencyCode, DensityKilogramsPerCubicMeter,
    EffectiveDate, LibraryRecordState, MAX_SHOP_RESOURCE_RECORDS_PER_KIND,
    MachineEnvelopeMillimeters, MachineProfile, MachineProfileId, MaterialDefinition,
    MaterialDefinitionId, MaterialOffer, MaterialOfferId, Money, PricingMethod, ProcessClass,
    RateCard, RecordedAt, RemovalRateCubicMillimetersPerMinute, ResourceSelectionState,
    RuntimeMinutes, RuntimeProfileId, ShopProfileId, ShopResourceCatalog, ShopResourceCatalogId,
    ShopResourceLibrary, ShopResourceLibraryId, ShopResourceVersion, ShopSettingsDraft,
    ShopSettingsRevision, SourceKind, SourceRef, StockAllowanceMillimeters, StockAllowanceProfile,
    StockAllowanceProfileId, StockForm,
};
use partprobe_persistence_sqlite::{
    SqliteShopResourceCatalogAuthorizationAudit, SqliteShopSettingsRepository,
};
use partprobe_security::{
    AuditCorrelationId, AuthorizationDecision, AuthorizationReasonCode, SecurityPolicyId,
    SecurityPolicyRef, SecurityPolicyVersion,
};
use rust_decimal::Decimal;

use crate::analysis::trusted_recorded_at;
use crate::estimate::{pricing_policy, rate_card};

const SHOP_PROFILE_ID: &str = "default-shop";
const CATALOG_POLICY_ID: &str = "partprobe.desktop.catalog-activation";
const CATALOG_POLICY_VERSION: u64 = 1;
const CATALOG_POLICY_NOT_CONFIGURED: &str = "CATALOG_ACTIVATION_NOT_CONFIGURED";

#[derive(Clone, Debug, Eq, PartialEq)]
enum DesktopCatalogAuthorizationRule {
    DenyAll,
    ExactOperator {
        profile_id: ShopProfileId,
        actor_id: ActorId,
        allowed_reason: AuthorizationReasonCode,
    },
}

/// Native-only catalog policy adapter. An exact-operator rule is safe only when the actor
/// identity is supplied by a trusted host authentication/session boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopCatalogAuthorizationPolicy {
    policy: SecurityPolicyRef,
    rule: DesktopCatalogAuthorizationRule,
    denied_reason: AuthorizationReasonCode,
}

impl DesktopCatalogAuthorizationPolicy {
    /// Creates an explicit deny-all deployment baseline.
    #[must_use]
    pub fn deny_all(policy: SecurityPolicyRef, denied_reason: AuthorizationReasonCode) -> Self {
        Self {
            policy,
            rule: DesktopCatalogAuthorizationRule::DenyAll,
            denied_reason,
        }
    }

    /// Allows only one exact profile/operator pair from a trusted native identity source.
    #[must_use]
    pub const fn exact_operator(
        policy: SecurityPolicyRef,
        profile_id: ShopProfileId,
        actor_id: ActorId,
        allowed_reason: AuthorizationReasonCode,
        denied_reason: AuthorizationReasonCode,
    ) -> Self {
        Self {
            policy,
            rule: DesktopCatalogAuthorizationRule::ExactOperator {
                profile_id,
                actor_id,
                allowed_reason,
            },
            denied_reason,
        }
    }
}

impl ShopResourceCatalogAuthorizationPolicy for DesktopCatalogAuthorizationPolicy {
    fn evaluate(&self, context: &ShopResourceCatalogAuthorizationContext) -> AuthorizationDecision {
        match &self.rule {
            DesktopCatalogAuthorizationRule::ExactOperator {
                profile_id,
                actor_id,
                allowed_reason,
            } if profile_id == context.profile_id() && actor_id == context.actor_id() => {
                AuthorizationDecision::allow(self.policy.clone(), allowed_reason.clone())
            }
            DesktopCatalogAuthorizationRule::DenyAll
            | DesktopCatalogAuthorizationRule::ExactOperator { .. } => {
                AuthorizationDecision::deny(self.policy.clone(), self.denied_reason.clone())
            }
        }
    }
}

type DesktopCatalogApplication = ShopResourceCatalogApplication<
    SqliteShopSettingsRepository,
    DesktopCatalogAuthorizationPolicy,
    SqliteShopResourceCatalogAuthorizationAudit,
>;
type DesktopCatalogDraftApplication =
    ShopResourceCatalogDraftApplication<SqliteShopSettingsRepository>;

/// Host-owned Settings service. Paths and SQLite handles never cross the desktop contract.
pub struct DesktopSettingsState {
    application: Mutex<Option<ShopSettingsApplication<SqliteShopSettingsRepository>>>,
    catalog_draft_application: Mutex<Option<DesktopCatalogDraftApplication>>,
    catalog_application: Mutex<Option<DesktopCatalogApplication>>,
    catalog_actor: Option<ActorId>,
    next_catalog_correlation: AtomicU64,
}

impl DesktopSettingsState {
    pub fn open(database_path: &Path) -> Result<Self, HostCommandError> {
        Self::open_with_catalog_configuration(database_path, unconfigured_catalog_policy()?, None)
    }

    /// Composes the native repository, deployment policy, and durable decision audit without
    /// supplying actor authority.
    pub fn open_with_catalog_policy(
        database_path: &Path,
        catalog_policy: DesktopCatalogAuthorizationPolicy,
    ) -> Result<Self, HostCommandError> {
        Self::open_with_catalog_configuration(database_path, catalog_policy, None)
    }

    /// Controlled native composition seam for a future authenticated session adapter.
    /// Supplying an actor here does not authenticate it; deployment code must do that before
    /// constructing this state. Ordinary runtime startup deliberately supplies no actor.
    pub fn open_with_catalog_configuration(
        database_path: &Path,
        catalog_policy: DesktopCatalogAuthorizationPolicy,
        catalog_actor: Option<ActorId>,
    ) -> Result<Self, HostCommandError> {
        let repository = SqliteShopSettingsRepository::open(database_path)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-OPEN"))?;
        let catalog_repository = SqliteShopSettingsRepository::open(database_path)
            .map_err(|error| map_store_error(error, "USE2-CATALOG-REPOSITORY-OPEN"))?;
        let catalog_draft_repository = SqliteShopSettingsRepository::open(database_path)
            .map_err(|error| map_store_error(error, "USE2-CATALOG-DRAFT-REPOSITORY-OPEN"))?;
        let catalog_audit = SqliteShopResourceCatalogAuthorizationAudit::open(database_path)
            .map_err(|error| map_store_error(error, "USE2-CATALOG-AUDIT-OPEN"))?;
        Ok(Self {
            application: Mutex::new(Some(ShopSettingsApplication::new(repository))),
            catalog_draft_application: Mutex::new(Some(ShopResourceCatalogDraftApplication::new(
                catalog_draft_repository,
            ))),
            catalog_application: Mutex::new(Some(ShopResourceCatalogApplication::new(
                catalog_repository,
                catalog_policy,
                catalog_audit,
            ))),
            catalog_actor,
            next_catalog_correlation: AtomicU64::new(1),
        })
    }

    #[must_use]
    pub const fn unavailable() -> Self {
        Self {
            application: Mutex::new(None),
            catalog_draft_application: Mutex::new(None),
            catalog_application: Mutex::new(None),
            catalog_actor: None,
            next_catalog_correlation: AtomicU64::new(1),
        }
    }

    fn activate_catalog_for_proposals(
        &self,
        request: &ApplicationActivationRequest,
    ) -> Result<ShopSettingsDraft, ShopResourceCatalogActivationError> {
        let mut application = self.catalog_application.lock().map_err(|_| {
            ShopResourceCatalogActivationError::Store(ShopSettingsStoreError::Unavailable)
        })?;
        application
            .as_mut()
            .ok_or(ShopResourceCatalogActivationError::Store(
                ShopSettingsStoreError::Unavailable,
            ))?
            .activate_for_proposals(request)
    }

    /// Converts the path-free contract request into the governed application request using only
    /// native actor, time, profile, and correlation evidence.
    pub fn activate_shop_resource_selection(
        &self,
        request: &ActivateShopResourceSelectionRequest,
    ) -> Result<ShopResourceCatalogActivationResult, HostCommandError> {
        let Some(actor) = self.catalog_actor.clone() else {
            return Ok(ShopResourceCatalogActivationResult::Unavailable {
                reason: ShopResourceCatalogActivationUnavailableReason::NativeIdentityUnavailable,
            });
        };
        let expected_settings_revision =
            ShopSettingsRevision::new(request.expected_settings_revision)
                .map_err(|_| invalid_settings("USE2-CATALOG-EXPECTED-SETTINGS-REVISION"))?;
        let expected_catalog_id = ShopResourceCatalogId::new(&request.expected_catalog_id)
            .map_err(|_| invalid_settings("USE2-CATALOG-EXPECTED-CATALOG-ID"))?;
        let expected_catalog_version =
            ShopResourceVersion::new(request.expected_catalog_version)
                .map_err(|_| invalid_settings("USE2-CATALOG-EXPECTED-CATALOG-VERSION"))?;
        let selection_id = partprobe_domain::ResourceSelectionId::new(&request.selection_id)
            .map_err(|_| invalid_settings("USE2-CATALOG-SELECTION-ID"))?;
        let selection_version = ShopResourceVersion::new(request.selection_version)
            .map_err(|_| invalid_settings("USE2-CATALOG-SELECTION-VERSION"))?;
        let (recorded_at, correlation_id) = self.catalog_operation_evidence()?;
        let application_request = ApplicationActivationRequest::new(
            profile_id()?,
            expected_settings_revision,
            expected_catalog_id,
            expected_catalog_version,
            selection_id,
            selection_version,
            actor,
            recorded_at,
            &request.reason,
            correlation_id,
        )
        .map_err(|_| invalid_settings("USE2-CATALOG-ACTIVATION-REQUEST"))?;

        match self.activate_catalog_for_proposals(&application_request) {
            Ok(draft) => Ok(ShopResourceCatalogActivationResult::Activated {
                settings: Box::new(snapshot(&draft)?),
            }),
            Err(ShopResourceCatalogActivationError::Denied(reason_code)) => {
                Ok(ShopResourceCatalogActivationResult::Denied {
                    reason_code: reason_code.as_str().to_owned(),
                })
            }
            Err(error) => Err(map_catalog_activation_error(error)),
        }
    }

    /// Maps one complete path-free catalog draft through native identity/time/profile evidence.
    pub fn save_shop_resource_catalog_draft(
        &self,
        request: &SaveShopResourceCatalogDraftRequest,
    ) -> Result<ShopResourceCatalogDraftSaveResult, HostCommandError> {
        let Some(actor) = self.catalog_actor.clone() else {
            return Ok(ShopResourceCatalogDraftSaveResult::Unavailable {
                reason: ShopResourceCatalogDraftUnavailableReason::NativeIdentityUnavailable,
            });
        };
        let current = self.current_settings_draft()?;
        let expected_settings_revision =
            ShopSettingsRevision::new(request.expected_settings_revision)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-SETTINGS-REVISION"))?;
        let expected_catalog = request
            .expected_catalog
            .as_ref()
            .map(expected_catalog_revision)
            .transpose()?;
        let recorded_at = trusted_recorded_at()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-DRAFT-TIME"))?;
        let contents = catalog_draft_contents(
            request,
            current.resource_catalog(),
            current.currency(),
            &recorded_at,
        )?;
        let application_request = ApplicationCatalogDraftRequest::new(
            profile_id()?,
            expected_settings_revision,
            expected_catalog,
            contents,
            actor,
            recorded_at,
            &request.reason,
        )
        .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-REQUEST"))?;
        let mut application = self
            .catalog_draft_application
            .lock()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-DRAFT-LOCK"))?;
        let saved = application
            .as_mut()
            .ok_or_else(|| {
                HostCommandError::settings_unavailable("USE2-CATALOG-DRAFT-NOT-CONFIGURED")
            })?
            .save_draft(&application_request)
            .map_err(map_catalog_draft_error)?;
        Ok(ShopResourceCatalogDraftSaveResult::Saved {
            settings: Box::new(snapshot(&saved)?),
        })
    }

    pub(crate) fn current_settings_draft(&self) -> Result<ShopSettingsDraft, HostCommandError> {
        let application = self
            .application
            .lock()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-LOCK"))?;
        let application = application.as_ref().ok_or_else(|| {
            HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED")
        })?;
        match application
            .load(&profile_id()?)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-LOAD"))?
        {
            ShopSettingsLoadState::Available(draft) => Ok(*draft),
            ShopSettingsLoadState::NotConfigured => Err(HostCommandError::settings_unavailable(
                "USE2-SETTINGS-NOT-CONFIGURED",
            )),
        }
    }

    fn catalog_operation_evidence(
        &self,
    ) -> Result<(RecordedAt, AuditCorrelationId), HostCommandError> {
        let sequence = self
            .next_catalog_correlation
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .map_err(|_| invalid_settings("USE2-CATALOG-CORRELATION-OVERFLOW"))?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-TIME"))?
            .as_nanos();
        let recorded_at = RecordedAt::new(format!("unix-nanos:{nanos}"))
            .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-TIME"))?;
        let correlation_id = AuditCorrelationId::new(format!(
            "desktop-catalog-{}-{nanos}-{sequence}",
            std::process::id()
        ))
        .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-CORRELATION"))?;
        Ok((recorded_at, correlation_id))
    }

    pub fn load(&self) -> Result<ShopSettingsState, HostCommandError> {
        let profile = profile_id()?;
        let application = self
            .application
            .lock()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-LOCK"))?;
        let application = application.as_ref().ok_or_else(|| {
            HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED")
        })?;
        match application
            .load(&profile)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-LOAD"))?
        {
            ShopSettingsLoadState::NotConfigured => Ok(ShopSettingsState::NotConfigured),
            ShopSettingsLoadState::Available(draft) => Ok(ShopSettingsState::Available {
                settings: Box::new(snapshot(&draft)?),
            }),
        }
    }

    pub fn save(
        &self,
        request: &SaveShopSettingsRequest,
    ) -> Result<ShopSettingsState, HostCommandError> {
        if !request.rates.confirmed_for_session || !request.pricing.confirmed_for_session {
            return Err(HostCommandError::invalid_settings_input(
                "USE2-SETTINGS-CONFIRMATION",
            ));
        }
        if request.rates.currency != "USD" {
            return Err(HostCommandError::invalid_settings_input(
                "USE2-SETTINGS-CURRENCY",
            ));
        }
        let expected_revision = request
            .expected_revision
            .map(ShopSettingsRevision::new)
            .transpose()
            .map_err(|_| invalid_settings("USE2-SETTINGS-EXPECTED-REVISION"))?;
        let next_revision = request
            .expected_revision
            .map_or(Some(1), |value| value.checked_add(1))
            .ok_or_else(|| invalid_settings("USE2-SETTINGS-REVISION-OVERFLOW"))?;
        let profile = profile_id()?;
        let recorded_at = trusted_recorded_at()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-TIME"))?;
        let actor = ActorId::new(&request.changed_by)
            .map_err(|_| invalid_settings("USE2-SETTINGS-ACTOR"))?;
        let (mut rates, _) = rate_card(
            &request.rates,
            SHOP_PROFILE_ID,
            &recorded_at,
            actor.as_str(),
            "entered in a durable non-authoritative shop-settings draft",
            "confirmed for the saved draft; calculation use still requires session review",
        )
        .map_err(|_| invalid_settings("USE2-SETTINGS-RATES"))?;
        let mut pricing = pricing_policy(&request.pricing, &request.rates.currency)
            .map_err(|_| invalid_settings("USE2-SETTINGS-PRICING"))?;
        let mut resources = request
            .resources
            .as_ref()
            .map(|fields| resource_library(fields, &recorded_at, &request.rates.currency))
            .transpose()?;

        let mut application = self
            .application
            .lock()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-LOCK"))?;
        let application_ref = application.as_ref().ok_or_else(|| {
            HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED")
        })?;
        if let ShopSettingsLoadState::Available(current) = application_ref
            .load(&profile)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-LOAD"))?
        {
            if current.resource_catalog().is_some() {
                return Err(HostCommandError::settings_unavailable(
                    "USE2-CATALOG-SAVE-NOT-AUTHORIZED",
                ));
            }
            if let Some(existing) = current.rate_card()
                && rate_fields(existing)? == rate_fields(&rates)?
            {
                rates = existing.clone();
            }
            if let Some(existing) = current.pricing_policy()
                && pricing_fields(existing)? == pricing_fields(&pricing)?
            {
                pricing = existing.clone();
            }
            if let (Some(existing), Some(candidate)) =
                (current.resource_library(), resources.as_ref())
                && resource_fields(existing) == resource_fields(candidate)
            {
                resources = Some(existing.clone());
            }
        }
        let draft = ShopSettingsDraft::new_with_resources(
            profile,
            ShopSettingsRevision::new(next_revision)
                .map_err(|_| invalid_settings("USE2-SETTINGS-REVISION"))?,
            CurrencyCode::new("USD").map_err(|_| invalid_settings("USE2-SETTINGS-CURRENCY"))?,
            Some(rates),
            Some(pricing),
            resources,
            actor,
            recorded_at,
            &request.change_reason,
        )
        .map_err(|_| invalid_settings("USE2-SETTINGS-DRAFT"))?;
        application
            .as_mut()
            .ok_or_else(|| HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED"))?
            .save(&draft, expected_revision)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-SAVE"))?;
        Ok(ShopSettingsState::Available {
            settings: Box::new(snapshot(&draft)?),
        })
    }
}

fn unconfigured_catalog_policy() -> Result<DesktopCatalogAuthorizationPolicy, HostCommandError> {
    let policy = SecurityPolicyRef::new(
        SecurityPolicyId::new(CATALOG_POLICY_ID)
            .map_err(|_| invalid_settings("USE2-CATALOG-POLICY-ID"))?,
        SecurityPolicyVersion::new(CATALOG_POLICY_VERSION)
            .map_err(|_| invalid_settings("USE2-CATALOG-POLICY-VERSION"))?,
    );
    let reason = AuthorizationReasonCode::new(CATALOG_POLICY_NOT_CONFIGURED)
        .map_err(|_| invalid_settings("USE2-CATALOG-POLICY-REASON"))?;
    Ok(DesktopCatalogAuthorizationPolicy::deny_all(policy, reason))
}

fn profile_id() -> Result<ShopProfileId, HostCommandError> {
    ShopProfileId::new(SHOP_PROFILE_ID)
        .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-PROFILE"))
}

fn snapshot(draft: &ShopSettingsDraft) -> Result<ShopSettingsSnapshot, HostCommandError> {
    Ok(ShopSettingsSnapshot {
        revision: draft.revision().value(),
        currency: draft.currency().as_str().to_owned(),
        rates: draft.rate_card().map(rate_fields).transpose()?,
        pricing: draft.pricing_policy().map(pricing_fields).transpose()?,
        resources: draft.resource_library().map(resource_fields),
        resource_catalog: draft.resource_catalog().map(resource_catalog_snapshot),
        changed_by: draft.changed_by().as_str().to_owned(),
        changed_at: draft.changed_at().as_str().to_owned(),
        change_reason: draft.change_reason().to_owned(),
    })
}

fn resource_catalog_snapshot(catalog: &ShopResourceCatalog) -> ShopResourceCatalogSnapshot {
    ShopResourceCatalogSnapshot {
        catalog_id: catalog.id().as_str().to_owned(),
        catalog_version: catalog.version().value(),
        currency: catalog.currency().as_str().to_owned(),
        materials: catalog
            .materials()
            .iter()
            .map(|material| ShopMaterialSnapshot {
                material_id: material.id().as_str().to_owned(),
                material_version: material.version().value(),
                family: material.family().to_owned(),
                grade: material.grade().to_owned(),
                specification: material.specification().map(str::to_owned),
                condition: material.condition().map(str::to_owned),
                density_kg_per_m3: decimal_text(material.density_kg_per_m3().value()),
                source_id: material.source().source_id().to_owned(),
                state: record_state(material.state()),
            })
            .collect(),
        material_offers: catalog
            .material_offers()
            .iter()
            .map(|offer| ShopMaterialOfferSnapshot {
                offer_id: offer.id().as_str().to_owned(),
                offer_version: offer.version().value(),
                material_id: offer.material_id().as_str().to_owned(),
                material_version: offer.material_version().value(),
                supplier: offer.supplier().to_owned(),
                price_per_kg: decimal_text(offer.price_per_kg().amount()),
                currency: offer.price_per_kg().currency().as_str().to_owned(),
                effective_from: offer.effective_from().as_str().to_owned(),
                source_id: offer.source().source_id().to_owned(),
                state: record_state(offer.state()),
            })
            .collect(),
        stock_allowances: catalog
            .stock_allowances()
            .iter()
            .map(|stock| ShopStockAllowanceSnapshot {
                stock_allowance_id: stock.id().as_str().to_owned(),
                stock_allowance_version: stock.version().value(),
                stock_form: stock_form_text(stock.stock_form()).to_owned(),
                x_allowance_mm: decimal_text(stock.x_allowance_mm().value()),
                y_allowance_mm: decimal_text(stock.y_allowance_mm().value()),
                z_allowance_mm: decimal_text(stock.z_allowance_mm().value()),
                source_id: stock.source().source_id().to_owned(),
                state: record_state(stock.state()),
            })
            .collect(),
        machines: catalog
            .machines()
            .iter()
            .map(|machine| ShopMachineSnapshot {
                machine_id: machine.id().as_str().to_owned(),
                machine_version: machine.version().value(),
                name: machine.name().to_owned(),
                process_class: process_class_text(machine.process_class()).to_owned(),
                envelope_x_mm: decimal_text(machine.envelope_x_mm().value()),
                envelope_y_mm: decimal_text(machine.envelope_y_mm().value()),
                envelope_z_mm: decimal_text(machine.envelope_z_mm().value()),
                source_id: machine.source().source_id().to_owned(),
                state: record_state(machine.state()),
            })
            .collect(),
        runtimes: catalog
            .runtimes()
            .iter()
            .map(|runtime| ShopRuntimeSnapshot {
                runtime_id: runtime.id().as_str().to_owned(),
                runtime_version: runtime.version().value(),
                machine_id: runtime.machine_id().as_str().to_owned(),
                machine_version: runtime.machine_version().value(),
                material_id: runtime.material_id().as_str().to_owned(),
                material_version: runtime.material_version().value(),
                removal_rate_mm3_per_minute: decimal_text(
                    runtime.removal_rate_mm3_per_minute().value(),
                ),
                setup_minutes: decimal_text(runtime.setup_minutes().value()),
                programming_minutes: decimal_text(runtime.programming_minutes().value()),
                load_unload_minutes: decimal_text(runtime.load_unload_minutes().value()),
                inspection_minutes: decimal_text(runtime.inspection_minutes().value()),
                source_id: runtime.source().source_id().to_owned(),
                state: record_state(runtime.state()),
            })
            .collect(),
        selections: catalog
            .selections()
            .iter()
            .map(|selection| ShopResourceSelectionSnapshot {
                selection_id: selection.id().as_str().to_owned(),
                selection_version: selection.version().value(),
                material_id: selection.material_id().as_str().to_owned(),
                material_version: selection.material_version().value(),
                material_offer_id: selection.material_offer_id().as_str().to_owned(),
                material_offer_version: selection.material_offer_version().value(),
                stock_allowance_id: selection.stock_allowance_id().as_str().to_owned(),
                stock_allowance_version: selection.stock_allowance_version().value(),
                machine_id: selection.machine_id().as_str().to_owned(),
                machine_version: selection.machine_version().value(),
                runtime_id: selection.runtime_id().as_str().to_owned(),
                runtime_version: selection.runtime_version().value(),
                state: selection_state(selection.state()),
                decided_by: selection.decided_by().as_str().to_owned(),
                decided_at: selection.decided_at().as_str().to_owned(),
                reason: selection.reason().to_owned(),
            })
            .collect(),
    }
}

fn expected_catalog_revision(
    expected: &ExpectedShopResourceCatalogRevision,
) -> Result<(ShopResourceCatalogId, ShopResourceVersion), HostCommandError> {
    Ok((
        ShopResourceCatalogId::new(&expected.catalog_id)
            .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-EXPECTED-ID"))?,
        ShopResourceVersion::new(expected.catalog_version)
            .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-EXPECTED-VERSION"))?,
    ))
}

fn catalog_draft_contents(
    request: &SaveShopResourceCatalogDraftRequest,
    current: Option<&ShopResourceCatalog>,
    currency: &CurrencyCode,
    recorded_at: &RecordedAt,
) -> Result<ApplicationCatalogDraftContents, HostCommandError> {
    let input = &request.contents;
    if [
        input.materials.len(),
        input.material_offers.len(),
        input.stock_allowances.len(),
        input.machines.len(),
        input.runtimes.len(),
    ]
    .into_iter()
    .any(|count| count > MAX_SHOP_RESOURCE_RECORDS_PER_KIND)
    {
        return Err(invalid_settings("USE2-CATALOG-DRAFT-RECORD-LIMIT"));
    }
    let materials = input
        .materials
        .iter()
        .map(|candidate| {
            let id = MaterialDefinitionId::new(&candidate.material_id)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MATERIAL-ID"))?;
            let version = ShopResourceVersion::new(candidate.material_version)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MATERIAL-VERSION"))?;
            let source = current
                .and_then(|catalog| {
                    catalog.materials().iter().find(|existing| {
                        existing.id() == &id
                            && existing.version() == version
                            && existing.source().source_id() == candidate.source_id
                    })
                })
                .map(|existing| existing.source().clone())
                .map_or_else(
                    || manual_source(&candidate.source_id, recorded_at),
                    Result::Ok,
                )?;
            MaterialDefinition::new(
                id,
                version,
                &candidate.family,
                &candidate.grade,
                candidate.specification.clone(),
                candidate.condition.clone(),
                DensityKilogramsPerCubicMeter::new(decimal(&candidate.density_kg_per_m3)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-DENSITY"))?,
                source,
                domain_record_state(candidate.state),
            )
            .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MATERIAL"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let material_offers = input
        .material_offers
        .iter()
        .map(|candidate| {
            if candidate.currency != currency.as_str() {
                return Err(invalid_settings("USE2-CATALOG-DRAFT-OFFER-CURRENCY"));
            }
            let id = MaterialOfferId::new(&candidate.offer_id)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-OFFER-ID"))?;
            let version = ShopResourceVersion::new(candidate.offer_version)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-OFFER-VERSION"))?;
            let source = current
                .and_then(|catalog| {
                    catalog.material_offers().iter().find(|existing| {
                        existing.id() == &id
                            && existing.version() == version
                            && existing.source().source_id() == candidate.source_id
                    })
                })
                .map(|existing| existing.source().clone())
                .map_or_else(
                    || manual_source(&candidate.source_id, recorded_at),
                    Result::Ok,
                )?;
            MaterialOffer::new(
                id,
                version,
                MaterialDefinitionId::new(&candidate.material_id)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-OFFER-MATERIAL-ID"))?,
                ShopResourceVersion::new(candidate.material_version)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-OFFER-MATERIAL-VERSION"))?,
                &candidate.supplier,
                Money::new(decimal(&candidate.price_per_kg)?, currency.clone()),
                EffectiveDate::new(&candidate.effective_from)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-OFFER-DATE"))?,
                source,
                domain_record_state(candidate.state),
            )
            .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-OFFER"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let stock_allowances = input
        .stock_allowances
        .iter()
        .map(|candidate| {
            let id = StockAllowanceProfileId::new(&candidate.stock_allowance_id)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-STOCK-ID"))?;
            let version = ShopResourceVersion::new(candidate.stock_allowance_version)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-STOCK-VERSION"))?;
            let source = current
                .and_then(|catalog| {
                    catalog.stock_allowances().iter().find(|existing| {
                        existing.id() == &id
                            && existing.version() == version
                            && existing.source().source_id() == candidate.source_id
                    })
                })
                .map(|existing| existing.source().clone())
                .map_or_else(
                    || manual_source(&candidate.source_id, recorded_at),
                    Result::Ok,
                )?;
            Ok(StockAllowanceProfile::new(
                id,
                version,
                stock_form(&candidate.stock_form)?,
                StockAllowanceMillimeters::new(decimal(&candidate.x_allowance_mm)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-STOCK-X"))?,
                StockAllowanceMillimeters::new(decimal(&candidate.y_allowance_mm)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-STOCK-Y"))?,
                StockAllowanceMillimeters::new(decimal(&candidate.z_allowance_mm)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-STOCK-Z"))?,
                source,
                domain_record_state(candidate.state),
            ))
        })
        .collect::<Result<Vec<_>, HostCommandError>>()?;
    let machines = input
        .machines
        .iter()
        .map(|candidate| {
            let id = MachineProfileId::new(&candidate.machine_id)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MACHINE-ID"))?;
            let version = ShopResourceVersion::new(candidate.machine_version)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MACHINE-VERSION"))?;
            let source = current
                .and_then(|catalog| {
                    catalog.machines().iter().find(|existing| {
                        existing.id() == &id
                            && existing.version() == version
                            && existing.source().source_id() == candidate.source_id
                    })
                })
                .map(|existing| existing.source().clone())
                .map_or_else(
                    || manual_source(&candidate.source_id, recorded_at),
                    Result::Ok,
                )?;
            MachineProfile::new(
                id,
                version,
                &candidate.name,
                process_class(&candidate.process_class)?,
                MachineEnvelopeMillimeters::new(decimal(&candidate.envelope_x_mm)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MACHINE-X"))?,
                MachineEnvelopeMillimeters::new(decimal(&candidate.envelope_y_mm)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MACHINE-Y"))?,
                MachineEnvelopeMillimeters::new(decimal(&candidate.envelope_z_mm)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MACHINE-Z"))?,
                source,
                domain_record_state(candidate.state),
            )
            .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-MACHINE"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let runtimes = input
        .runtimes
        .iter()
        .map(|candidate| {
            let id = RuntimeProfileId::new(&candidate.runtime_id)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-RUNTIME-ID"))?;
            let version = ShopResourceVersion::new(candidate.runtime_version)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-RUNTIME-VERSION"))?;
            let source = current
                .and_then(|catalog| {
                    catalog.runtimes().iter().find(|existing| {
                        existing.id() == &id
                            && existing.version() == version
                            && existing.source().source_id() == candidate.source_id
                    })
                })
                .map(|existing| existing.source().clone())
                .map_or_else(
                    || manual_source(&candidate.source_id, recorded_at),
                    Result::Ok,
                )?;
            Ok(CoarseRuntimeProfile::new(
                id,
                version,
                MachineProfileId::new(&candidate.machine_id)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-RUNTIME-MACHINE-ID"))?,
                ShopResourceVersion::new(candidate.machine_version)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-RUNTIME-MACHINE-VERSION"))?,
                MaterialDefinitionId::new(&candidate.material_id)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-RUNTIME-MATERIAL-ID"))?,
                ShopResourceVersion::new(candidate.material_version)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-RUNTIME-MATERIAL-VERSION"))?,
                RemovalRateCubicMillimetersPerMinute::new(decimal(
                    &candidate.removal_rate_mm3_per_minute,
                )?)
                .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-REMOVAL-RATE"))?,
                RuntimeMinutes::new(decimal(&candidate.setup_minutes)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-SETUP-TIME"))?,
                RuntimeMinutes::new(decimal(&candidate.programming_minutes)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-PROGRAMMING-TIME"))?,
                RuntimeMinutes::new(decimal(&candidate.load_unload_minutes)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-LOAD-TIME"))?,
                RuntimeMinutes::new(decimal(&candidate.inspection_minutes)?)
                    .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-INSPECTION-TIME"))?,
                source,
                domain_record_state(candidate.state),
            ))
        })
        .collect::<Result<Vec<_>, HostCommandError>>()?;

    Ok(ApplicationCatalogDraftContents::new(
        ShopResourceCatalogId::new(&input.catalog_id)
            .map_err(|_| invalid_settings("USE2-CATALOG-DRAFT-ID"))?,
        materials,
        material_offers,
        stock_allowances,
        machines,
        runtimes,
    ))
}

const fn domain_record_state(state: ShopResourceRecordState) -> LibraryRecordState {
    match state {
        ShopResourceRecordState::Draft => LibraryRecordState::Draft,
        ShopResourceRecordState::Reviewed => LibraryRecordState::Reviewed,
        ShopResourceRecordState::Approved => LibraryRecordState::Approved,
        ShopResourceRecordState::Retired => LibraryRecordState::Retired,
        ShopResourceRecordState::Superseded => LibraryRecordState::Superseded,
    }
}

const fn record_state(state: LibraryRecordState) -> ShopResourceRecordState {
    match state {
        LibraryRecordState::Draft => ShopResourceRecordState::Draft,
        LibraryRecordState::Reviewed => ShopResourceRecordState::Reviewed,
        LibraryRecordState::Approved => ShopResourceRecordState::Approved,
        LibraryRecordState::Retired => ShopResourceRecordState::Retired,
        LibraryRecordState::Superseded => ShopResourceRecordState::Superseded,
    }
}

const fn selection_state(state: ResourceSelectionState) -> ShopResourceSelectionState {
    match state {
        ResourceSelectionState::Reviewed => ShopResourceSelectionState::Reviewed,
        ResourceSelectionState::ActiveForProposals => {
            ShopResourceSelectionState::ActiveForProposals
        }
        ResourceSelectionState::Retired => ShopResourceSelectionState::Retired,
    }
}

fn resource_library(
    fields: &ShopResourceInputFields,
    recorded_at: &RecordedAt,
    currency: &str,
) -> Result<ShopResourceLibrary, HostCommandError> {
    if !fields.confirmed_for_draft {
        return Err(invalid_settings("USE2-RESOURCES-CONFIRMATION"));
    }
    let library_version = resource_version(&fields.library_version)?;
    let material_version = resource_version(&fields.material_version)?;
    let offer_version = resource_version(&fields.offer_version)?;
    let stock_version = resource_version(&fields.stock_profile_version)?;
    let machine_version = resource_version(&fields.machine_version)?;
    let runtime_version = resource_version(&fields.runtime_profile_version)?;
    let material_id = MaterialDefinitionId::new(&fields.material_id)
        .map_err(|_| invalid_settings("USE2-RESOURCES-MATERIAL-ID"))?;
    let machine_id = MachineProfileId::new(&fields.machine_id)
        .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-ID"))?;
    let currency =
        CurrencyCode::new(currency).map_err(|_| invalid_settings("USE2-RESOURCES-CURRENCY"))?;
    let material = MaterialDefinition::new(
        material_id.clone(),
        material_version,
        &fields.material_family,
        &fields.material_grade,
        optional_text(&fields.optional_material_specification),
        optional_text(&fields.optional_material_condition),
        DensityKilogramsPerCubicMeter::new(decimal(&fields.density_kg_per_m3)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-DENSITY"))?,
        manual_source(&fields.material_source, recorded_at)?,
        LibraryRecordState::Draft,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-MATERIAL"))?;
    let offer = MaterialOffer::new(
        MaterialOfferId::new(&fields.offer_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-OFFER-ID"))?,
        offer_version,
        material_id.clone(),
        material_version,
        &fields.supplier,
        Money::new(decimal(&fields.material_price_per_kg)?, currency.clone()),
        EffectiveDate::new(&fields.offer_effective_on)
            .map_err(|_| invalid_settings("USE2-RESOURCES-OFFER-DATE"))?,
        manual_source(&fields.offer_source, recorded_at)?,
        LibraryRecordState::Draft,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-OFFER"))?;
    let stock = StockAllowanceProfile::new(
        StockAllowanceProfileId::new(&fields.stock_profile_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-ID"))?,
        stock_version,
        stock_form(&fields.stock_form)?,
        StockAllowanceMillimeters::new(decimal(&fields.stock_allowance_x_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-X"))?,
        StockAllowanceMillimeters::new(decimal(&fields.stock_allowance_y_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-Y"))?,
        StockAllowanceMillimeters::new(decimal(&fields.stock_allowance_z_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-Z"))?,
        manual_source(&fields.stock_source, recorded_at)?,
        LibraryRecordState::Draft,
    );
    let machine = MachineProfile::new(
        machine_id.clone(),
        machine_version,
        &fields.machine_name,
        process_class(&fields.process_class)?,
        MachineEnvelopeMillimeters::new(decimal(&fields.machine_envelope_x_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-X"))?,
        MachineEnvelopeMillimeters::new(decimal(&fields.machine_envelope_y_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-Y"))?,
        MachineEnvelopeMillimeters::new(decimal(&fields.machine_envelope_z_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-Z"))?,
        manual_source(&fields.machine_source, recorded_at)?,
        LibraryRecordState::Draft,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE"))?;
    let runtime = CoarseRuntimeProfile::new(
        RuntimeProfileId::new(&fields.runtime_profile_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-RUNTIME-ID"))?,
        runtime_version,
        machine_id,
        machine_version,
        material_id,
        material_version,
        RemovalRateCubicMillimetersPerMinute::new(decimal(&fields.removal_rate_mm3_per_minute)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-REMOVAL-RATE"))?,
        RuntimeMinutes::new(decimal(&fields.setup_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-SETUP-TIME"))?,
        RuntimeMinutes::new(decimal(&fields.programming_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-PROGRAMMING-TIME"))?,
        RuntimeMinutes::new(decimal(&fields.load_unload_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-LOAD-TIME"))?,
        RuntimeMinutes::new(decimal(&fields.inspection_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-INSPECTION-TIME"))?,
        manual_source(&fields.runtime_source, recorded_at)?,
        LibraryRecordState::Draft,
    );
    ShopResourceLibrary::new(
        ShopResourceLibraryId::new(&fields.library_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-LIBRARY-ID"))?,
        library_version,
        currency,
        material,
        offer,
        stock,
        machine,
        runtime,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-LIBRARY"))
}

fn resource_fields(library: &ShopResourceLibrary) -> ShopResourceInputFields {
    let material = library.material();
    let offer = library.material_offer();
    let stock = library.stock_allowance();
    let machine = library.machine();
    let runtime = library.runtime();
    ShopResourceInputFields {
        confirmed_for_draft: false,
        library_id: library.id().as_str().to_owned(),
        library_version: library.version().value().to_string(),
        material_id: material.id().as_str().to_owned(),
        material_version: material.version().value().to_string(),
        material_family: material.family().to_owned(),
        material_grade: material.grade().to_owned(),
        optional_material_specification: material.specification().unwrap_or_default().to_owned(),
        optional_material_condition: material.condition().unwrap_or_default().to_owned(),
        density_kg_per_m3: decimal_text(material.density_kg_per_m3().value()),
        material_source: material.source().source_id().to_owned(),
        offer_id: offer.id().as_str().to_owned(),
        offer_version: offer.version().value().to_string(),
        supplier: offer.supplier().to_owned(),
        material_price_per_kg: decimal_text(offer.price_per_kg().amount()),
        offer_effective_on: offer.effective_from().as_str().to_owned(),
        offer_source: offer.source().source_id().to_owned(),
        stock_profile_id: stock.id().as_str().to_owned(),
        stock_profile_version: stock.version().value().to_string(),
        stock_form: stock_form_text(stock.stock_form()).to_owned(),
        stock_allowance_x_mm: decimal_text(stock.x_allowance_mm().value()),
        stock_allowance_y_mm: decimal_text(stock.y_allowance_mm().value()),
        stock_allowance_z_mm: decimal_text(stock.z_allowance_mm().value()),
        stock_source: stock.source().source_id().to_owned(),
        machine_id: machine.id().as_str().to_owned(),
        machine_version: machine.version().value().to_string(),
        machine_name: machine.name().to_owned(),
        process_class: process_class_text(machine.process_class()).to_owned(),
        machine_envelope_x_mm: decimal_text(machine.envelope_x_mm().value()),
        machine_envelope_y_mm: decimal_text(machine.envelope_y_mm().value()),
        machine_envelope_z_mm: decimal_text(machine.envelope_z_mm().value()),
        machine_source: machine.source().source_id().to_owned(),
        runtime_profile_id: runtime.id().as_str().to_owned(),
        runtime_profile_version: runtime.version().value().to_string(),
        removal_rate_mm3_per_minute: decimal_text(runtime.removal_rate_mm3_per_minute().value()),
        setup_minutes: decimal_text(runtime.setup_minutes().value()),
        programming_minutes: decimal_text(runtime.programming_minutes().value()),
        load_unload_minutes: decimal_text(runtime.load_unload_minutes().value()),
        inspection_minutes: decimal_text(runtime.inspection_minutes().value()),
        runtime_source: runtime.source().source_id().to_owned(),
    }
}

fn decimal(value: &str) -> Result<Decimal, HostCommandError> {
    Decimal::from_str(value).map_err(|_| invalid_settings("USE2-RESOURCES-DECIMAL"))
}

fn resource_version(value: &str) -> Result<ShopResourceVersion, HostCommandError> {
    value
        .parse::<u32>()
        .map_err(|_| invalid_settings("USE2-RESOURCES-VERSION"))
        .and_then(|value| {
            ShopResourceVersion::new(value).map_err(|_| invalid_settings("USE2-RESOURCES-VERSION"))
        })
}

fn optional_text(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_owned())
}

fn manual_source(value: &str, recorded_at: &RecordedAt) -> Result<SourceRef, HostCommandError> {
    SourceRef::new(SourceKind::Manual, value, None, Some(recorded_at.clone()))
        .map_err(|_| invalid_settings("USE2-RESOURCES-SOURCE"))
}

fn stock_form(value: &str) -> Result<StockForm, HostCommandError> {
    match value {
        "rectangular" => Ok(StockForm::Rectangular),
        "round" => Ok(StockForm::Round),
        "plate" => Ok(StockForm::Plate),
        _ => Err(invalid_settings("USE2-RESOURCES-STOCK-FORM")),
    }
}

const fn stock_form_text(value: StockForm) -> &'static str {
    match value {
        StockForm::Rectangular => "rectangular",
        StockForm::Round => "round",
        StockForm::Plate => "plate",
    }
}

fn process_class(value: &str) -> Result<ProcessClass, HostCommandError> {
    match value {
        "milling" => Ok(ProcessClass::Milling),
        "turning" => Ok(ProcessClass::Turning),
        "sawing" => Ok(ProcessClass::Sawing),
        "inspection" => Ok(ProcessClass::Inspection),
        _ => Err(invalid_settings("USE2-RESOURCES-PROCESS")),
    }
}

const fn process_class_text(value: ProcessClass) -> &'static str {
    match value {
        ProcessClass::Milling => "milling",
        ProcessClass::Turning => "turning",
        ProcessClass::Sawing => "sawing",
        ProcessClass::Inspection => "inspection",
    }
}

fn rate_fields(card: &RateCard) -> Result<DeveloperRateInputFields, HostCommandError> {
    let effective_on = common_effective_date(card)?;
    Ok(DeveloperRateInputFields {
        confirmed_for_session: false,
        rate_card_id: card.id().as_str().to_owned(),
        rate_card_version: card.version().value().to_string(),
        effective_on,
        currency: card.currency().as_str().to_owned(),
        setup_labor_per_hour: rate_amount(card, CostCategory::SetupLabor)?,
        programming_per_hour: rate_amount(card, CostCategory::Programming)?,
        run_labor_per_hour: rate_amount(card, CostCategory::RunLabor)?,
        machine_per_hour: rate_amount(card, CostCategory::Machine)?,
        quality_inspection_per_hour: rate_amount(card, CostCategory::QualityInspection)?,
    })
}

fn pricing_fields(
    pricing: &partprobe_domain::PricingPolicy,
) -> Result<DeveloperPricingInputFields, HostCommandError> {
    let markup_rate = match pricing.method() {
        PricingMethod::Markup { rate } => decimal_text(*rate),
        PricingMethod::TargetMargin { .. } => {
            return Err(HostCommandError::settings_unavailable(
                "USE2-SETTINGS-UNSUPPORTED-PRICING",
            ));
        }
    };
    Ok(DeveloperPricingInputFields {
        confirmed_for_session: false,
        pricing_policy_id: pricing.id().as_str().to_owned(),
        pricing_policy_version: pricing.version().value().to_string(),
        markup_rate,
        optional_price_floor: pricing
            .floor()
            .map_or_else(String::new, |value| decimal_text(value.amount())),
        optional_minimum_order: pricing
            .minimum_order()
            .map_or_else(String::new, |value| decimal_text(value.amount())),
        rounding_decimal_places: pricing.quote_total_rounding().scale().to_string(),
    })
}

fn common_effective_date(card: &RateCard) -> Result<String, HostCommandError> {
    let first = card
        .entries()
        .first()
        .ok_or_else(|| HostCommandError::settings_unavailable("USE2-SETTINGS-EMPTY-RATES"))?
        .effective_from();
    if card
        .entries()
        .iter()
        .any(|entry| entry.effective_from() != first)
    {
        return Err(HostCommandError::settings_unavailable(
            "USE2-SETTINGS-MIXED-EFFECTIVE-DATES",
        ));
    }
    Ok(first.as_str().to_owned())
}

fn rate_amount(card: &RateCard, category: CostCategory) -> Result<String, HostCommandError> {
    let mut matches = card
        .entries()
        .iter()
        .filter(|entry| entry.category() == category);
    let amount = matches
        .next()
        .ok_or_else(|| HostCommandError::settings_unavailable("USE2-SETTINGS-MISSING-RATE"))?
        .amount()
        .amount();
    if matches.next().is_some() {
        return Err(HostCommandError::settings_unavailable(
            "USE2-SETTINGS-AMBIGUOUS-RATE",
        ));
    }
    Ok(decimal_text(amount))
}

fn decimal_text(value: Decimal) -> String {
    partprobe_domain::decimal_serde::canonical(value).to_string()
}

fn map_store_error(error: ShopSettingsStoreError, diagnostic: &str) -> HostCommandError {
    match error {
        ShopSettingsStoreError::RevisionConflict => HostCommandError::settings_conflict(diagnostic),
        ShopSettingsStoreError::IntegrityConflict => {
            HostCommandError::settings_version_conflict(diagnostic)
        }
        ShopSettingsStoreError::InvalidStoredRecord
        | ShopSettingsStoreError::UnsupportedSchema
        | ShopSettingsStoreError::Unavailable => HostCommandError::settings_unavailable(diagnostic),
    }
}

fn map_catalog_activation_error(error: ShopResourceCatalogActivationError) -> HostCommandError {
    match error {
        ShopResourceCatalogActivationError::NotConfigured
        | ShopResourceCatalogActivationError::CatalogUnavailable
        | ShopResourceCatalogActivationError::AuditUnavailable => {
            HostCommandError::settings_unavailable("USE2-CATALOG-ACTIVATION-UNAVAILABLE")
        }
        ShopResourceCatalogActivationError::StaleSettings
        | ShopResourceCatalogActivationError::StaleCatalog => {
            HostCommandError::settings_conflict("USE2-CATALOG-ACTIVATION-STALE")
        }
        ShopResourceCatalogActivationError::SelectionUnavailable
        | ShopResourceCatalogActivationError::SelectionNotReviewed
        | ShopResourceCatalogActivationError::VersionOverflow
        | ShopResourceCatalogActivationError::InvalidTransition => {
            invalid_settings("USE2-CATALOG-ACTIVATION-INVALID")
        }
        ShopResourceCatalogActivationError::Store(error) => {
            map_store_error(error, "USE2-CATALOG-ACTIVATION-STORE")
        }
        ShopResourceCatalogActivationError::Denied(_) => {
            HostCommandError::settings_unavailable("USE2-CATALOG-ACTIVATION-DENIED-MAPPING")
        }
    }
}

fn map_catalog_draft_error(error: ShopResourceCatalogDraftError) -> HostCommandError {
    match error {
        ShopResourceCatalogDraftError::NotConfigured
        | ShopResourceCatalogDraftError::StarterResourceMigrationRequired => {
            HostCommandError::settings_unavailable("USE2-CATALOG-DRAFT-UNAVAILABLE")
        }
        ShopResourceCatalogDraftError::StaleSettings
        | ShopResourceCatalogDraftError::CatalogExpectationMismatch => {
            HostCommandError::settings_conflict("USE2-CATALOG-DRAFT-STALE")
        }
        ShopResourceCatalogDraftError::InvalidRecordLifecycle => {
            invalid_settings("USE2-CATALOG-DRAFT-LIFECYCLE")
        }
        ShopResourceCatalogDraftError::InvalidCatalog => {
            invalid_settings("USE2-CATALOG-DRAFT-CONTENTS")
        }
        ShopResourceCatalogDraftError::VersionOverflow => {
            invalid_settings("USE2-CATALOG-DRAFT-VERSION-OVERFLOW")
        }
        ShopResourceCatalogDraftError::Store(error) => {
            map_store_error(error, "USE2-CATALOG-DRAFT-STORE")
        }
    }
}

fn invalid_settings(diagnostic: &str) -> HostCommandError {
    HostCommandError::invalid_settings_input(diagnostic)
}

#[cfg(test)]
mod tests {
    use partprobe_application::ShopSettingsDraftRepository;
    use partprobe_desktop_contract::{
        HostErrorCode, ShopResourceCatalogDraftContents as ContractCatalogDraftContents,
    };
    use partprobe_domain::ShopResourceCatalogId;
    use partprobe_security::AuditCorrelationId;
    use partprobe_test_support::{TestDirectory, resource_catalog_fixture};

    use super::*;

    fn request(expected_revision: Option<u32>) -> SaveShopSettingsRequest {
        SaveShopSettingsRequest {
            expected_revision,
            changed_by: "test-operator".to_owned(),
            change_reason: "reviewed test values".to_owned(),
            rates: DeveloperRateInputFields {
                confirmed_for_session: true,
                rate_card_id: "test-shop-rates".to_owned(),
                rate_card_version: "1".to_owned(),
                effective_on: "2026-09-04".to_owned(),
                currency: "USD".to_owned(),
                setup_labor_per_hour: "75".to_owned(),
                programming_per_hour: "90".to_owned(),
                run_labor_per_hour: "65".to_owned(),
                machine_per_hour: "110".to_owned(),
                quality_inspection_per_hour: "80".to_owned(),
            },
            pricing: DeveloperPricingInputFields {
                confirmed_for_session: true,
                pricing_policy_id: "test-shop-pricing".to_owned(),
                pricing_policy_version: "1".to_owned(),
                markup_rate: "0.25".to_owned(),
                optional_price_floor: String::new(),
                optional_minimum_order: "100".to_owned(),
                rounding_decimal_places: "2".to_owned(),
            },
            resources: None,
        }
    }

    fn resources() -> ShopResourceInputFields {
        ShopResourceInputFields {
            confirmed_for_draft: true,
            library_id: "test-resources".to_owned(),
            library_version: "1".to_owned(),
            material_id: "al-6061-t6".to_owned(),
            material_version: "1".to_owned(),
            material_family: "Aluminum".to_owned(),
            material_grade: "6061".to_owned(),
            optional_material_specification: "ASTM B221".to_owned(),
            optional_material_condition: "T6".to_owned(),
            density_kg_per_m3: "2700".to_owned(),
            material_source: "test-material-handbook".to_owned(),
            offer_id: "test-al-offer".to_owned(),
            offer_version: "1".to_owned(),
            supplier: "Synthetic supplier".to_owned(),
            material_price_per_kg: "8.50".to_owned(),
            offer_effective_on: "2026-09-09".to_owned(),
            offer_source: "test-supplier-quote".to_owned(),
            stock_profile_id: "test-rectangular-stock".to_owned(),
            stock_profile_version: "1".to_owned(),
            stock_form: "rectangular".to_owned(),
            stock_allowance_x_mm: "3".to_owned(),
            stock_allowance_y_mm: "3".to_owned(),
            stock_allowance_z_mm: "2".to_owned(),
            stock_source: "test-shop-policy".to_owned(),
            machine_id: "test-vmc".to_owned(),
            machine_version: "1".to_owned(),
            machine_name: "Synthetic VMC".to_owned(),
            process_class: "milling".to_owned(),
            machine_envelope_x_mm: "762".to_owned(),
            machine_envelope_y_mm: "508".to_owned(),
            machine_envelope_z_mm: "508".to_owned(),
            machine_source: "test-machine-manual".to_owned(),
            runtime_profile_id: "test-vmc-al-runtime".to_owned(),
            runtime_profile_version: "1".to_owned(),
            removal_rate_mm3_per_minute: "16000".to_owned(),
            setup_minutes: "60".to_owned(),
            programming_minutes: "45".to_owned(),
            load_unload_minutes: "3".to_owned(),
            inspection_minutes: "15".to_owned(),
            runtime_source: "test-estimator-review".to_owned(),
        }
    }

    fn reviewed_catalog_draft() -> ShopSettingsDraft {
        ShopSettingsDraft::new_with_catalog(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            Some(resource_catalog_fixture(
                1,
                1,
                "2700",
                1,
                ResourceSelectionState::Reviewed,
            )),
            ActorId::new("test-catalog-author").unwrap(),
            RecordedAt::new("2026-09-10T13:00:00Z").unwrap(),
            "reviewed synthetic catalog awaiting governed activation",
        )
        .unwrap()
    }

    fn seed_reviewed_catalog(database: &Path) -> ShopSettingsDraft {
        let draft = reviewed_catalog_draft();
        let mut repository = SqliteShopSettingsRepository::open(database).unwrap();
        repository.save(&draft, None).unwrap();
        drop(repository);
        draft
    }

    fn activation_request(actor: &str, correlation: &str) -> ApplicationActivationRequest {
        ApplicationActivationRequest::new(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            ShopResourceCatalogId::new("test-resource-catalog").unwrap(),
            ShopResourceVersion::new(1).unwrap(),
            partprobe_domain::ResourceSelectionId::new("test-resource-selection").unwrap(),
            ShopResourceVersion::new(1).unwrap(),
            ActorId::new(actor).unwrap(),
            RecordedAt::new("2026-09-10T14:00:00Z").unwrap(),
            "activate exact reviewed resources for proposal eligibility",
            AuditCorrelationId::new(correlation).unwrap(),
        )
        .unwrap()
    }

    fn activation_command_request() -> ActivateShopResourceSelectionRequest {
        ActivateShopResourceSelectionRequest {
            expected_settings_revision: 1,
            expected_catalog_id: "test-resource-catalog".to_owned(),
            expected_catalog_version: 1,
            selection_id: "test-resource-selection".to_owned(),
            selection_version: 1,
            reason: "activate exact reviewed resources for proposal eligibility".to_owned(),
        }
    }

    fn catalog_draft_command_request(
        draft: &ShopSettingsDraft,
    ) -> SaveShopResourceCatalogDraftRequest {
        let catalog = draft.resource_catalog().expect("catalog");
        let snapshot = resource_catalog_snapshot(catalog);
        SaveShopResourceCatalogDraftRequest {
            expected_settings_revision: draft.revision().value(),
            expected_catalog: Some(ExpectedShopResourceCatalogRevision {
                catalog_id: snapshot.catalog_id.clone(),
                catalog_version: snapshot.catalog_version,
            }),
            contents: ContractCatalogDraftContents {
                catalog_id: snapshot.catalog_id,
                materials: snapshot.materials,
                material_offers: snapshot.material_offers,
                stock_allowances: snapshot.stock_allowances,
                machines: snapshot.machines,
                runtimes: snapshot.runtimes,
            },
            reason: "save reviewed catalog draft changes".to_owned(),
        }
    }

    fn exact_operator_policy(actor: &str) -> DesktopCatalogAuthorizationPolicy {
        DesktopCatalogAuthorizationPolicy::exact_operator(
            SecurityPolicyRef::new(
                SecurityPolicyId::new("test.desktop.catalog-policy").unwrap(),
                SecurityPolicyVersion::new(7).unwrap(),
            ),
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ActorId::new(actor).unwrap(),
            AuthorizationReasonCode::new("CATALOG_ACTIVATION_EXACT_OPERATOR_ALLOWED").unwrap(),
            AuthorizationReasonCode::new("CATALOG_ACTIVATION_IDENTITY_MISMATCH").unwrap(),
        )
    }

    #[test]
    fn first_run_save_reopen_and_stale_writer_are_explicit() {
        let directory = TestDirectory::create("desktop-settings").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let state = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(state.load().unwrap(), ShopSettingsState::NotConfigured);

        let saved = state.save(&request(None)).unwrap();
        let ShopSettingsState::Available { settings } = saved else {
            panic!("saved settings must be available");
        };
        assert_eq!(settings.revision, 1);
        assert_eq!(settings.rates.as_ref().unwrap().machine_per_hour, "110");
        assert!(!settings.rates.as_ref().unwrap().confirmed_for_session);
        assert!(!settings.pricing.as_ref().unwrap().confirmed_for_session);
        assert_eq!(settings.changed_by, "test-operator");
        assert_eq!(
            state.save(&request(None)).unwrap_err().code,
            HostErrorCode::SettingsConflict
        );
        let mut changed_without_version = request(Some(1));
        changed_without_version.rates.machine_per_hour = "111".to_owned();
        assert_eq!(
            state.save(&changed_without_version).unwrap_err().code,
            HostErrorCode::SettingsVersionConflict
        );

        drop(state);
        let reopened = DesktopSettingsState::open(&database).unwrap();
        let loaded = reopened.load().unwrap();
        assert_eq!(
            serde_json::to_string(&loaded).unwrap(),
            serde_json::to_string(&ShopSettingsState::Available { settings }).unwrap()
        );
        drop(reopened);
        directory.cleanup().unwrap();
    }

    #[test]
    fn save_rejects_unconfirmed_or_non_usd_values_without_mutation() {
        let directory = TestDirectory::create("desktop-settings-invalid").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let state = DesktopSettingsState::open(&database).unwrap();

        let mut unconfirmed = request(None);
        unconfirmed.rates.confirmed_for_session = false;
        assert_eq!(
            state.save(&unconfirmed).unwrap_err().code,
            HostErrorCode::InvalidSettingsInput
        );
        let mut wrong_currency = request(None);
        wrong_currency.rates.currency = "EUR".to_owned();
        assert_eq!(
            state.save(&wrong_currency).unwrap_err().code,
            HostErrorCode::InvalidSettingsInput
        );
        let mut unconfirmed_resources = request(None);
        let mut resource_fields = resources();
        resource_fields.confirmed_for_draft = false;
        unconfirmed_resources.resources = Some(resource_fields);
        assert_eq!(
            state.save(&unconfirmed_resources).unwrap_err().code,
            HostErrorCode::InvalidSettingsInput
        );
        assert_eq!(state.load().unwrap(), ShopSettingsState::NotConfigured);

        drop(state);
        directory.cleanup().unwrap();
    }

    #[test]
    fn path_free_snapshot_preserves_missing_libraries_as_missing() {
        let draft = ShopSettingsDraft::new(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            ActorId::new("test-operator").unwrap(),
            partprobe_domain::RecordedAt::new("2026-09-04T12:00:00Z").unwrap(),
            "partial first-run draft",
        )
        .unwrap();

        let value = snapshot(&draft).unwrap();
        assert_eq!(value.currency, "USD");
        assert!(value.rates.is_none());
        assert!(value.pricing.is_none());
        assert!(value.resource_catalog.is_none());
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(serialized.contains("\"rates\":null"));
        assert!(serialized.contains("\"pricing\":null"));
        assert!(!serialized.contains("path"));
    }

    #[test]
    fn contract_v7_exposes_a_complete_path_free_read_only_catalog() {
        let draft = ShopSettingsDraft::new_with_catalog(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            Some(resource_catalog_fixture(
                1,
                1,
                "2700",
                1,
                partprobe_domain::ResourceSelectionState::ActiveForProposals,
            )),
            ActorId::new("test-operator").unwrap(),
            partprobe_domain::RecordedAt::new("2026-09-09T16:00:00Z").unwrap(),
            "persisted catalog is read-only in desktop contract v7",
        )
        .unwrap();

        let value = snapshot(&draft).unwrap();
        let catalog = value.resource_catalog.as_ref().unwrap();
        assert_eq!(catalog.catalog_id, "test-resource-catalog");
        assert_eq!(catalog.catalog_version, 1);
        assert_eq!(catalog.currency, "USD");
        assert_eq!(catalog.materials.len(), 1);
        assert_eq!(catalog.material_offers.len(), 1);
        assert_eq!(catalog.stock_allowances.len(), 1);
        assert_eq!(catalog.machines.len(), 1);
        assert_eq!(catalog.runtimes.len(), 1);
        assert_eq!(catalog.selections.len(), 1);
        assert_eq!(catalog.materials[0].material_id, "test-al-6061-t6");
        assert_eq!(catalog.materials[0].density_kg_per_m3, "2700");
        assert_eq!(
            catalog.materials[0].state,
            ShopResourceRecordState::Reviewed
        );
        assert_eq!(catalog.material_offers[0].price_per_kg, "8.5");
        assert_eq!(catalog.material_offers[0].currency, "USD");
        assert_eq!(catalog.stock_allowances[0].stock_form, "rectangular");
        assert_eq!(catalog.stock_allowances[0].z_allowance_mm, "2");
        assert_eq!(catalog.machines[0].process_class, "milling");
        assert_eq!(catalog.machines[0].envelope_x_mm, "762");
        assert_eq!(catalog.runtimes[0].removal_rate_mm3_per_minute, "16000");
        assert_eq!(catalog.runtimes[0].setup_minutes, "60");
        assert_eq!(
            catalog.selections[0].state,
            ShopResourceSelectionState::ActiveForProposals
        );
        assert_eq!(catalog.selections[0].decided_by, "test-resource-reviewer");
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(serialized.contains("active_for_proposals"));
        assert!(!serialized.contains("path"));
        assert!(!serialized.contains("sqlite"));
    }

    #[test]
    fn contract_v7_loads_but_cannot_overwrite_a_persisted_catalog_revision() {
        let directory = TestDirectory::create("desktop-settings-catalog-guard").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let draft = ShopSettingsDraft::new_with_catalog(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            Some(resource_catalog_fixture(
                1,
                1,
                "2700",
                1,
                partprobe_domain::ResourceSelectionState::ActiveForProposals,
            )),
            ActorId::new("test-operator").unwrap(),
            partprobe_domain::RecordedAt::new("2026-09-09T16:00:00Z").unwrap(),
            "persisted catalog is read-only in desktop contract v7",
        )
        .unwrap();
        let mut repository = SqliteShopSettingsRepository::open(&database).unwrap();
        repository.save(&draft, None).unwrap();
        drop(repository);

        let state = DesktopSettingsState::open(&database).unwrap();
        let ShopSettingsState::Available { settings } = state.load().unwrap() else {
            panic!("catalog-backed settings must load");
        };
        assert!(settings.resource_catalog.is_some());
        assert_eq!(
            state.save(&request(Some(1))).unwrap_err().code,
            HostErrorCode::SettingsUnavailable
        );
        drop(state);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(draft)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn native_default_policy_denies_and_durably_replays_the_exact_decision() {
        let directory = TestDirectory::create("desktop-catalog-default-deny").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = seed_reviewed_catalog(&database);
        let request = activation_request("test-catalog-operator", "default-denial-1");

        let state = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(
            state.activate_catalog_for_proposals(&request),
            Err(ShopResourceCatalogActivationError::Denied(
                AuthorizationReasonCode::new(CATALOG_POLICY_NOT_CONFIGURED).unwrap(),
            ))
        );
        drop(state);

        let reopened = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(
            reopened.activate_catalog_for_proposals(&request),
            Err(ShopResourceCatalogActivationError::Denied(
                AuthorizationReasonCode::new(CATALOG_POLICY_NOT_CONFIGURED).unwrap(),
            ))
        );
        assert_eq!(
            reopened.activate_catalog_for_proposals(&activation_request(
                "another-catalog-operator",
                "default-denial-1",
            )),
            Err(ShopResourceCatalogActivationError::AuditUnavailable)
        );
        drop(reopened);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(initial)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn native_exact_operator_policy_denies_mismatch_then_activates_and_reopens() {
        let directory = TestDirectory::create("desktop-catalog-exact-operator").unwrap();
        let database = directory.path().join("settings.sqlite3");
        seed_reviewed_catalog(&database);
        let state = DesktopSettingsState::open_with_catalog_policy(
            &database,
            exact_operator_policy("trusted-catalog-operator"),
        )
        .unwrap();

        assert_eq!(
            state.activate_catalog_for_proposals(&activation_request(
                "untrusted-catalog-operator",
                "configured-denial-1",
            )),
            Err(ShopResourceCatalogActivationError::Denied(
                AuthorizationReasonCode::new("CATALOG_ACTIVATION_IDENTITY_MISMATCH").unwrap(),
            ))
        );
        let activated = state
            .activate_catalog_for_proposals(&activation_request(
                "trusted-catalog-operator",
                "configured-allow-1",
            ))
            .unwrap();
        assert_eq!(activated.revision().value(), 2);
        assert_eq!(
            activated
                .resource_catalog()
                .and_then(ShopResourceCatalog::active_selection)
                .map(partprobe_domain::ResourceSelection::state),
            Some(ResourceSelectionState::ActiveForProposals)
        );
        drop(state);

        let reopened = DesktopSettingsState::open(&database).unwrap();
        let ShopSettingsState::Available { settings } = reopened.load().unwrap() else {
            panic!("activated catalog settings must reopen");
        };
        assert_eq!(settings.revision, 2);
        assert_eq!(
            settings
                .resource_catalog
                .as_ref()
                .and_then(|catalog| catalog.selections.iter().find(|selection| {
                    selection.state == ShopResourceSelectionState::ActiveForProposals
                }))
                .map(|selection| selection.decided_by.as_str()),
            Some("trusted-catalog-operator")
        );
        drop(reopened);
        directory.cleanup().unwrap();
    }

    #[test]
    fn desktop_activation_request_stays_unavailable_without_native_identity() {
        let directory = TestDirectory::create("desktop-catalog-no-identity").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = seed_reviewed_catalog(&database);
        let state = DesktopSettingsState::open(&database).unwrap();

        assert_eq!(
            state
                .activate_shop_resource_selection(&activation_command_request())
                .unwrap(),
            ShopResourceCatalogActivationResult::Unavailable {
                reason: ShopResourceCatalogActivationUnavailableReason::NativeIdentityUnavailable,
            }
        );
        drop(state);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(initial)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn desktop_activation_request_uses_native_identity_and_shipped_policy_denies() {
        let directory = TestDirectory::create("desktop-catalog-command-deny").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = seed_reviewed_catalog(&database);
        let state = DesktopSettingsState::open_with_catalog_configuration(
            &database,
            unconfigured_catalog_policy().unwrap(),
            Some(ActorId::new("native-session-operator").unwrap()),
        )
        .unwrap();

        assert_eq!(
            state
                .activate_shop_resource_selection(&activation_command_request())
                .unwrap(),
            ShopResourceCatalogActivationResult::Denied {
                reason_code: CATALOG_POLICY_NOT_CONFIGURED.to_owned(),
            }
        );
        drop(state);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(initial)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn controlled_desktop_activation_derives_actor_time_and_correlation_natively() {
        let directory = TestDirectory::create("desktop-catalog-command-allow").unwrap();
        let database = directory.path().join("settings.sqlite3");
        seed_reviewed_catalog(&database);
        let actor = ActorId::new("trusted-native-session-operator").unwrap();
        let state = DesktopSettingsState::open_with_catalog_configuration(
            &database,
            exact_operator_policy(actor.as_str()),
            Some(actor.clone()),
        )
        .unwrap();

        let ShopResourceCatalogActivationResult::Activated { settings } = state
            .activate_shop_resource_selection(&activation_command_request())
            .unwrap()
        else {
            panic!("matching controlled native identity must activate the reviewed selection");
        };
        assert_eq!(settings.revision, 2);
        assert_eq!(settings.changed_by, actor.as_str());
        assert!(settings.changed_at.starts_with("unix-nanos:"));
        assert_eq!(
            settings
                .resource_catalog
                .as_ref()
                .and_then(|catalog| catalog.selections.iter().find(|selection| {
                    selection.state == ShopResourceSelectionState::ActiveForProposals
                }))
                .map(|selection| selection.decided_by.as_str()),
            Some(actor.as_str())
        );
        let serialized = serde_json::to_string(&settings).unwrap();
        assert!(!serialized.contains("correlation"));
        assert!(!serialized.contains("path"));
        drop(state);

        let reopened = DesktopSettingsState::open(&database).unwrap();
        assert!(matches!(
            reopened.load().unwrap(),
            ShopSettingsState::Available { .. }
        ));
        drop(reopened);
        directory.cleanup().unwrap();
    }

    #[test]
    fn desktop_catalog_draft_request_is_unavailable_without_native_identity() {
        let directory = TestDirectory::create("desktop-catalog-draft-no-identity").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = seed_reviewed_catalog(&database);
        let request = catalog_draft_command_request(&initial);
        let state = DesktopSettingsState::open(&database).unwrap();

        assert_eq!(
            state.save_shop_resource_catalog_draft(&request).unwrap(),
            ShopResourceCatalogDraftSaveResult::Unavailable {
                reason: ShopResourceCatalogDraftUnavailableReason::NativeIdentityUnavailable,
            }
        );
        drop(state);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(initial)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn controlled_catalog_draft_save_uses_native_identity_and_downgrades_activation() {
        let directory = TestDirectory::create("desktop-catalog-draft-save").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = ShopSettingsDraft::new_with_catalog(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            Some(resource_catalog_fixture(
                1,
                1,
                "2700",
                1,
                ResourceSelectionState::ActiveForProposals,
            )),
            ActorId::new("test-catalog-author").unwrap(),
            RecordedAt::new("2026-09-10T13:00:00Z").unwrap(),
            "active synthetic catalog awaiting an edit",
        )
        .unwrap();
        let mut repository = SqliteShopSettingsRepository::open(&database).unwrap();
        repository.save(&initial, None).unwrap();
        drop(repository);
        let request = catalog_draft_command_request(&initial);
        let actor = ActorId::new("trusted-native-catalog-editor").unwrap();
        let state = DesktopSettingsState::open_with_catalog_configuration(
            &database,
            unconfigured_catalog_policy().unwrap(),
            Some(actor.clone()),
        )
        .unwrap();
        let loaded = state.current_settings_draft().unwrap();
        let loaded_catalog = loaded.resource_catalog().unwrap();
        let converted = catalog_draft_contents(
            &request,
            Some(loaded_catalog),
            loaded.currency(),
            &RecordedAt::new("2026-09-10T18:00:00Z").unwrap(),
        )
        .unwrap();
        assert_eq!(
            converted,
            ApplicationCatalogDraftContents::new(
                loaded_catalog.id().clone(),
                loaded_catalog.materials().to_vec(),
                loaded_catalog.material_offers().to_vec(),
                loaded_catalog.stock_allowances().to_vec(),
                loaded_catalog.machines().to_vec(),
                loaded_catalog.runtimes().to_vec(),
            )
        );

        let ShopResourceCatalogDraftSaveResult::Saved { settings } =
            state.save_shop_resource_catalog_draft(&request).unwrap()
        else {
            panic!("trusted native editor must save the governed draft")
        };
        assert_eq!(settings.revision, 2);
        assert_eq!(settings.changed_by, actor.as_str());
        let catalog = settings.resource_catalog.as_ref().expect("catalog");
        assert_eq!(catalog.catalog_version, 2);
        assert!(
            catalog
                .selections
                .iter()
                .all(|selection| selection.state != ShopResourceSelectionState::ActiveForProposals)
        );
        assert_eq!(
            catalog.selections[0].state,
            ShopResourceSelectionState::Reviewed
        );
        assert_eq!(catalog.selections[0].selection_version, 2);
        let serialized = serde_json::to_string(&settings).unwrap();
        assert!(!serialized.contains("path"));
        assert!(!serialized.contains("sqlite"));
        drop(state);

        let reopened = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(
            reopened.load().unwrap(),
            ShopSettingsState::Available { settings }
        );
        drop(reopened);
        directory.cleanup().unwrap();
    }

    #[test]
    fn controlled_catalog_draft_save_rejects_changed_record_without_new_version() {
        let directory = TestDirectory::create("desktop-catalog-draft-version-reuse").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = seed_reviewed_catalog(&database);
        let mut request = catalog_draft_command_request(&initial);
        request.contents.material_offers[0].price_per_kg = "9.25".to_owned();
        let actor = ActorId::new("trusted-native-catalog-editor").unwrap();
        let state = DesktopSettingsState::open_with_catalog_configuration(
            &database,
            unconfigured_catalog_policy().unwrap(),
            Some(actor),
        )
        .unwrap();

        let error = state
            .save_shop_resource_catalog_draft(&request)
            .unwrap_err();
        assert_eq!(error.code, HostErrorCode::InvalidSettingsInput);
        assert_eq!(error.diagnostic_id, "USE2-CATALOG-DRAFT-LIFECYCLE");
        drop(state);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(initial)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn controlled_catalog_draft_save_rejects_over_limit_collection_before_mapping() {
        let directory = TestDirectory::create("desktop-catalog-draft-record-limit").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let initial = seed_reviewed_catalog(&database);
        let mut request = catalog_draft_command_request(&initial);
        request.contents.materials =
            vec![request.contents.materials[0].clone(); MAX_SHOP_RESOURCE_RECORDS_PER_KIND + 1];
        let actor = ActorId::new("trusted-native-catalog-editor").unwrap();
        let state = DesktopSettingsState::open_with_catalog_configuration(
            &database,
            unconfigured_catalog_policy().unwrap(),
            Some(actor),
        )
        .unwrap();

        let error = state
            .save_shop_resource_catalog_draft(&request)
            .unwrap_err();
        assert_eq!(error.code, HostErrorCode::InvalidSettingsInput);
        assert_eq!(error.diagnostic_id, "USE2-CATALOG-DRAFT-RECORD-LIMIT");
        drop(state);

        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        assert_eq!(
            repository.current(&profile_id().unwrap()).unwrap(),
            Some(initial)
        );
        drop(repository);
        directory.cleanup().unwrap();
    }

    #[test]
    fn resource_bundle_reuses_unchanged_evidence_and_rejects_changed_child_version() {
        let directory = TestDirectory::create("desktop-settings-resources").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let state = DesktopSettingsState::open(&database).unwrap();
        let mut initial = request(None);
        initial.resources = Some(resources());
        let saved = state.save(&initial).unwrap();
        let ShopSettingsState::Available {
            settings: first_settings,
        } = saved
        else {
            panic!("saved settings must be available");
        };
        let resource_fields = first_settings.resources.as_ref().unwrap();
        assert_eq!(resource_fields.material_grade, "6061");
        assert_eq!(resource_fields.stock_form, "rectangular");
        assert_eq!(resource_fields.process_class, "milling");
        assert!(!resource_fields.confirmed_for_draft);

        let mut unchanged = request(Some(1));
        unchanged.change_reason = "reviewed unchanged values for a new draft".to_owned();
        unchanged.resources = Some(resources());
        let unchanged_saved = state.save(&unchanged).unwrap();
        let ShopSettingsState::Available { settings } = unchanged_saved else {
            panic!("saved settings must be available");
        };
        assert_eq!(settings.revision, 2);

        let mut divergent = request(Some(2));
        let mut changed_resources = resources();
        changed_resources.library_version = "2".to_owned();
        changed_resources.density_kg_per_m3 = "2710".to_owned();
        divergent.resources = Some(changed_resources);
        assert_eq!(
            state.save(&divergent).unwrap_err().code,
            HostErrorCode::SettingsVersionConflict
        );

        drop(state);
        let reopened = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(
            reopened.load().unwrap(),
            ShopSettingsState::Available { settings }
        );
        drop(reopened);
        directory.cleanup().unwrap();
    }
}
