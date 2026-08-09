use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use partprobe_application::{
    AnalyzedGeometryEvidence, AssetReadSubject, DraftBaseCostInputs, DraftEstimateApplication,
    DraftEstimateApplicationError, DraftEstimateInputs, DraftGeometryRequestTemplate,
    DraftGeometryReview, DraftMaterialCostInputs, DraftOperationCostInputs, DraftQuantityInputs,
    DraftRateContext, DraftStockInputs, DraftTimeInputs, GeometryAnalysisFailure,
    GeometryAnalysisPort, LocalAssetReadService,
};
use partprobe_domain::{
    ActorId, AssetRootId, CurrencyCode, DataClassificationId, DensityKilogramsPerCubicMillimeter,
    EffectiveDate, ItemQuantity, Money, PricingPolicy, ProjectId, RateCard, RateEntry, RateScope,
    RecordId, RecordStateId, RecordVersionId, RecordedAt, RuleVersion, SchemaVersion, ValueState,
    VolumeCubicMillimeters,
};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, ProvisionalGeometryDecimal,
};
use partprobe_geometry_import::{
    AssetCapability, AssetReadGrant, CorrelationId, GeometryJobId, GeometryWorkerRequest,
    LocalAssetRoot, ProvisionalGeometrySnapshot, ResourceQuotas, Sha256Digest, SnapshotReference,
};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationAuditEvent, AuthorizationAuditSink,
    AuthorizationDecision, AuthorizationPolicy, AuthorizationReasonCode,
    DenyAllAuthorizationPolicy, SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion,
};
use rust_decimal::Decimal;

const TASK_002_FIXTURE: &str =
    include_str!("../../estimation-engine/tests/fixtures/task_002/golden_estimates.json");
const SOURCE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SYNTHETIC_SOURCE_HASH: &str =
    "72806c44cd993f89a56810474fac5ae65710a5d72232f095b5f959b6d84f20e4";
const OUTPUT_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";

static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
struct StaticAnalyzer {
    calls: Rc<Cell<u64>>,
    expected_source_hash: &'static str,
    evidence: AnalyzedGeometryEvidence,
}

impl GeometryAnalysisPort for StaticAnalyzer {
    fn analyze(
        &self,
        request: &GeometryWorkerRequest,
        grant: AssetReadGrant,
        _cancellation: &AtomicBool,
    ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure> {
        assert_eq!(
            request.expected_source_hash().as_str(),
            self.expected_source_hash
        );
        assert_eq!(grant.asset_capability(), request.asset_capability());
        self.calls.set(self.calls.get() + 1);
        Ok(self.evidence.clone())
    }
}

#[derive(Debug)]
struct AllowPolicy;

impl AuthorizationPolicy for AllowPolicy {
    fn evaluate(
        &self,
        _context: &partprobe_security::AuthorizationContext,
    ) -> AuthorizationDecision {
        AuthorizationDecision::allow(
            policy_ref(),
            AuthorizationReasonCode::new("ASSIGNED_SCOPE").expect("reason must be valid"),
        )
    }
}

#[derive(Debug, Default)]
struct RecordingAudit {
    events: RefCell<Vec<AuthorizationAuditEvent>>,
}

impl AuthorizationAuditSink for RecordingAudit {
    fn append(&self, event: AuthorizationAuditEvent) -> Result<(), AuditAppendError> {
        self.events.borrow_mut().push(event);
        Ok(())
    }
}

#[test]
fn authorized_analysis_starts_ephemeral_session_with_no_numeric_defaults() {
    let test_directory = create_test_root("start");
    let root = local_root(&test_directory, "root-start");
    let calls = Rc::new(Cell::new(0));
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(AllowPolicy, RecordingAudit::default()),
        StaticAnalyzer {
            calls: Rc::clone(&calls),
            expected_source_hash: SOURCE_HASH,
            evidence: geometry_evidence(),
        },
    );

    let mut session = application
        .start_session(
            subject(),
            &root,
            &request(),
            Path::new("part.step"),
            &AtomicBool::new(false),
        )
        .expect("authorized model must start a session");

    assert_eq!(calls.get(), 1);
    assert_eq!(application.asset_reads().audit().events.borrow().len(), 1);
    assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));
    session.set_geometry_review(DraftGeometryReview::new(true, true));
    assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));
    assert_eq!(
        session.geometry().snapshot.source_hash().as_str(),
        SOURCE_HASH
    );
    drop(session);
    drop(root);
    remove_test_root(&test_directory);
}

#[test]
fn authorized_source_is_fingerprinted_before_the_pathless_worker_request_is_built() {
    let test_directory = create_test_root("fingerprint");
    let root = local_root(&test_directory, "root-fingerprint");
    let calls = Rc::new(Cell::new(0));
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(AllowPolicy, RecordingAudit::default()),
        StaticAnalyzer {
            calls: Rc::clone(&calls),
            expected_source_hash: SYNTHETIC_SOURCE_HASH,
            evidence: geometry_evidence_for(SYNTHETIC_SOURCE_HASH),
        },
    );

    let session = application
        .start_session_from_unfingerprinted_source(
            subject(),
            &root,
            &request_template(),
            Path::new("part.step"),
            &AtomicBool::new(false),
        )
        .expect("authorized source must produce a pathless analyzed session");

    assert_eq!(calls.get(), 1);
    assert_eq!(application.asset_reads().audit().events.borrow().len(), 1);
    assert_eq!(
        session.geometry().snapshot.source_hash().as_str(),
        SYNTHETIC_SOURCE_HASH
    );
    assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));
    drop(session);
    drop(root);
    remove_test_root(&test_directory);
}

#[test]
fn denied_source_never_reaches_geometry_analysis() {
    let test_directory = create_test_root("deny");
    let root = local_root(&test_directory, "root-deny");
    let calls = Rc::new(Cell::new(0));
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(
            DenyAllAuthorizationPolicy::new(
                policy_ref(),
                AuthorizationReasonCode::new("POLICY_NOT_CONFIGURED")
                    .expect("reason must be valid"),
            ),
            RecordingAudit::default(),
        ),
        StaticAnalyzer {
            calls: Rc::clone(&calls),
            expected_source_hash: SOURCE_HASH,
            evidence: geometry_evidence(),
        },
    );

    let result = application.start_session(
        subject(),
        &root,
        &request(),
        Path::new("part.step"),
        &AtomicBool::new(false),
    );

    assert!(matches!(
        result,
        Err(DraftEstimateApplicationError::AssetRead(_))
    ));
    assert_eq!(calls.get(), 0);
    assert_eq!(application.asset_reads().audit().events.borrow().len(), 1);
    drop(root);
    remove_test_root(&test_directory);
}

#[test]
fn denied_unfingerprinted_source_is_not_opened_or_hashed() {
    let test_directory = create_test_root("deny-fingerprint");
    let root = local_root(&test_directory, "root-deny-fingerprint");
    let calls = Rc::new(Cell::new(0));
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(
            DenyAllAuthorizationPolicy::new(
                policy_ref(),
                AuthorizationReasonCode::new("POLICY_NOT_CONFIGURED")
                    .expect("reason must be valid"),
            ),
            RecordingAudit::default(),
        ),
        StaticAnalyzer {
            calls: Rc::clone(&calls),
            expected_source_hash: SYNTHETIC_SOURCE_HASH,
            evidence: geometry_evidence_for(SYNTHETIC_SOURCE_HASH),
        },
    );

    let result = application.start_session_from_unfingerprinted_source(
        subject(),
        &root,
        &request_template(),
        Path::new("missing.step"),
        &AtomicBool::new(false),
    );

    assert!(matches!(
        result,
        Err(DraftEstimateApplicationError::AssetRead(_))
    ));
    assert_eq!(calls.get(), 0);
    assert_eq!(application.asset_reads().audit().events.borrow().len(), 1);
    drop(root);
    remove_test_root(&test_directory);
}

#[test]
fn missing_and_conflicting_rates_never_collapse_to_zero() {
    let (card, pricing_policy) = rate_card_and_policy();
    let mut session = configured_session();
    session.set_rate_context(
        DraftRateContext::new(
            RateCard::empty(card.id().clone(), card.version(), card.currency().clone()),
            effective_date(),
            vec![RateScope::organization()],
        )
        .expect("scope context must be valid"),
    );
    session.set_pricing_policy(pricing_policy.clone());
    assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));

    let setup = card
        .entries()
        .iter()
        .find(|entry| entry.id().as_str() == "setup-labor")
        .expect("fixture setup rate");
    let mut duplicate_value = serde_json::to_value(setup).expect("serialize setup rate");
    duplicate_value["id"] = serde_json::Value::String("setup-labor-conflict".to_owned());
    let duplicate: RateEntry =
        serde_json::from_value(duplicate_value).expect("duplicate rate must remain valid");
    let mut entries = card.entries().to_vec();
    entries.push(duplicate);
    let conflicting_card = RateCard::new(
        card.id().clone(),
        card.version(),
        card.currency().clone(),
        entries,
    )
    .expect("conflicting applicability is retained for explicit resolution");
    session.set_rate_context(
        DraftRateContext::new(
            conflicting_card,
            effective_date(),
            vec![RateScope::organization()],
        )
        .expect("scope context must be valid"),
    );

    assert!(matches!(session.evaluate(), ValueState::Blocked { .. }));
}

#[test]
fn deterministic_recalculation_matches_golden_engine_trace_and_tracks_edits() {
    let (card, pricing_policy) = rate_card_and_policy();
    let mut session = configured_session();
    session.set_rate_context(
        DraftRateContext::new(card, effective_date(), vec![RateScope::organization()])
            .expect("scope context must be valid"),
    );
    session.set_pricing_policy(pricing_policy);

    let first = session.evaluate();
    let replay = session.evaluate();
    assert_eq!(first, replay);
    let first = available(first);
    assert_eq!(first.removed_volume.value.value(), decimal("1000"));
    assert_eq!(first.make_quantity.value(), 1);
    assert_eq!(first.material_cost, usd("100"));
    assert_eq!(first.operation_cost, usd("260"));
    assert_eq!(first.total_internal_cost, usd("520"));
    assert_eq!(first.pricing.rounded_price.rounded, usd("702"));
    assert_eq!(
        first.trace.resolved_rates.setup_labor.card_version.value(),
        1
    );
    assert_eq!(first.trace.geometry.snapshot.occt_version(), "8.0.0");
    assert!(first.trace.calculation_rule_ids.contains(&"CALC-018"));

    let mut edited = explicit_inputs();
    edited.times.setup_hours = decimal("4");
    session.set_inputs(edited);
    let recalculated = available(session.evaluate());
    assert_eq!(recalculated.setup_cost, usd("100"));
    assert_eq!(recalculated.total_internal_cost, usd("545"));
    assert_eq!(recalculated.pricing.rounded_price.rounded, usd("735.75"));
    assert_eq!(recalculated.trace.inputs.times.setup_hours, decimal("4"));

    let mut zero_delivery = explicit_inputs();
    zero_delivery.quantities.deliver = ItemQuantity::new(0);
    session.set_inputs(zero_delivery);
    assert!(matches!(session.evaluate(), ValueState::Blocked { .. }));
}

fn configured_session() -> partprobe_application::DraftEstimateSession {
    let test_directory = create_test_root("configured");
    let root = local_root(&test_directory, "root-configured");
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(AllowPolicy, RecordingAudit::default()),
        StaticAnalyzer {
            calls: Rc::new(Cell::new(0)),
            expected_source_hash: SOURCE_HASH,
            evidence: geometry_evidence(),
        },
    );
    let mut session = application
        .start_session(
            subject(),
            &root,
            &request(),
            Path::new("part.step"),
            &AtomicBool::new(false),
        )
        .expect("session must start");
    session.set_geometry_review(DraftGeometryReview::new(true, true));
    session.set_inputs(explicit_inputs());
    drop(root);
    remove_test_root(&test_directory);
    session
}

fn explicit_inputs() -> DraftEstimateInputs {
    DraftEstimateInputs {
        stock: DraftStockInputs {
            stock_volume: VolumeCubicMillimeters::new(decimal("2000"))
                .expect("stock volume must be valid"),
            density: DensityKilogramsPerCubicMillimeter::new(decimal("0.00000785"))
                .expect("density must be valid"),
        },
        quantities: DraftQuantityInputs {
            deliver: ItemQuantity::new(1),
            planned_spares: ItemQuantity::new(0),
            destructive_samples: ItemQuantity::new(0),
        },
        times: DraftTimeInputs {
            setup_hours: decimal("3"),
            programming_hours: decimal("2"),
            cutting_hours_per_item: decimal("0.42"),
            non_cutting_hours_per_item: decimal("0.18"),
            load_unload_hours_per_item: Decimal::ZERO,
            in_cycle_inspection_hours_per_item: Decimal::ZERO,
            quality_inspection_hours: decimal("1"),
        },
        material: DraftMaterialCostInputs {
            purchased: usd("90"),
            cut: usd("5"),
            certificate: usd("0"),
            inbound_freight: usd("5"),
            approved_remnant_credit: usd("0"),
        },
        operation: DraftOperationCostInputs {
            prove_out: usd("20"),
            tooling: usd("25"),
            consumables: usd("10"),
            fixture: usd("20"),
            outside: usd("0"),
            freight: usd("5"),
        },
        base: DraftBaseCostInputs {
            nonrecurring_engineering: usd("50"),
            administration: usd("25"),
            overhead: usd("50"),
            accepted_risk_impacts: vec![usd("35")],
            expected_rework: usd("0"),
        },
    }
}

fn geometry_evidence() -> AnalyzedGeometryEvidence {
    geometry_evidence_for(SOURCE_HASH)
}

fn geometry_evidence_for(source_hash: &str) -> AnalyzedGeometryEvidence {
    let snapshot = ProvisionalGeometrySnapshot::new(
        digest(source_hash),
        "8.0.0",
        1,
        1,
        1,
        geometry_decimal("600"),
        geometry_decimal("1000"),
        [
            geometry_decimal("5"),
            geometry_decimal("5"),
            geometry_decimal("5"),
        ],
    )
    .expect("synthetic geometry evidence must be valid");
    AnalyzedGeometryEvidence::new(
        partprobe_geometry_core::StageStatus::Succeeded,
        Vec::new(),
        SnapshotReference::new("geometry-snapshot-v1").expect("reference must be valid"),
        digest(OUTPUT_HASH),
        256,
        snapshot,
        None,
        None,
    )
    .expect("analyzed geometry evidence must be valid")
}

fn request() -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new("gui-2-job").expect("job ID must be valid"),
        CorrelationId::new("gui-2-correlation").expect("correlation must be valid"),
        AssetCapability::new("gui-2-capability").expect("capability must be valid"),
        digest(SOURCE_HASH),
        vec![GeometryStage::BasicProperties],
        AnalysisProfile {
            id: AnalysisProfileId::new("gui-2-profile").expect("profile must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 1_000_000, 10_000, 5_000).expect("quotas must be valid"),
    )
    .expect("request must be valid")
}

fn request_template() -> DraftGeometryRequestTemplate {
    DraftGeometryRequestTemplate::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new("gui-4-job").expect("job ID must be valid"),
        CorrelationId::new("gui-4-correlation").expect("correlation must be valid"),
        AssetCapability::new("gui-4-capability").expect("capability must be valid"),
        vec![GeometryStage::BasicProperties],
        AnalysisProfile {
            id: AnalysisProfileId::new("gui-4-profile").expect("profile must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 1_000_000, 10_000, 5_000).expect("quotas must be valid"),
    )
    .expect("request template must be valid")
}

fn rate_card_and_policy() -> (RateCard, PricingPolicy) {
    let fixture: serde_json::Value =
        serde_json::from_str(TASK_002_FIXTURE).expect("fixture must be valid JSON");
    (
        serde_json::from_value(fixture["rate_card"].clone()).expect("rate card must be valid"),
        serde_json::from_value(fixture["pricing_policy"].clone())
            .expect("pricing policy must be valid"),
    )
}

fn subject() -> AssetReadSubject {
    AssetReadSubject::new(
        ActorId::new("actor-1").expect("actor ID must be valid"),
        ProjectId::new("project-1").expect("project ID must be valid"),
        RecordId::new("asset-1").expect("record ID must be valid"),
        RecordVersionId::new("revision-1").expect("record version must be valid"),
        DataClassificationId::new("organization-defined").expect("classification must be valid"),
        RecordStateId::new("draft").expect("record state must be valid"),
        AuditCorrelationId::new("audit-correlation-1").expect("correlation must be valid"),
        RecordedAt::new("2026-08-01T12:00:00-05:00").expect("timestamp must be valid"),
    )
}

fn policy_ref() -> SecurityPolicyRef {
    SecurityPolicyRef::new(
        SecurityPolicyId::new("test-policy").expect("policy ID must be valid"),
        SecurityPolicyVersion::new(1).expect("policy version must be valid"),
    )
}

fn effective_date() -> EffectiveDate {
    EffectiveDate::new("2026-07-29").expect("date must be valid")
}

fn decimal(value: &str) -> Decimal {
    Decimal::from_str(value).expect("decimal must be valid")
}

fn usd(value: &str) -> Money {
    Money::new(
        decimal(value),
        CurrencyCode::new("USD").expect("currency must be valid"),
    )
}

fn available(
    state: ValueState<partprobe_application::DraftEstimateResult>,
) -> partprobe_application::DraftEstimateResult {
    match state {
        ValueState::Available { value } => value,
        other => panic!("expected available draft estimate, got {other:?}"),
    }
}

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::new(value).expect("digest must be valid")
}

fn geometry_decimal(value: &str) -> ProvisionalGeometryDecimal {
    ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be valid")
}

fn local_root(directory: &Path, id: &str) -> LocalAssetRoot {
    LocalAssetRoot::open(
        AssetRootId::new(id).expect("root ID must be valid"),
        directory,
    )
    .expect("asset root must open")
}

fn create_test_root(label: &str) -> std::path::PathBuf {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "partprobe-draft-estimate-{label}-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("test directory must be created");
    std::fs::write(directory.join("part.step"), b"synthetic-step-source")
        .expect("test asset must be written");
    directory
}

fn remove_test_root(directory: &Path) {
    std::fs::remove_file(directory.join("part.step")).expect("test asset must be removable");
    std::fs::remove_dir(directory).expect("test directory must be removable");
}
