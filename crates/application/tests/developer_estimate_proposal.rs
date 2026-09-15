use partprobe_application::{
    AnalyzedGeometryEvidence, DeveloperEstimateProposalAdoptionError,
    DeveloperEstimateProposalApplication, DeveloperEstimateProposalReview, DraftQuantityInputs,
};
use partprobe_application::{
    AssetReadSubject, DeveloperStockCandidateSessionError, DraftEstimateApplication,
    DraftEstimateSession, GeometryAnalysisFailure, GeometryAnalysisPort, LocalAssetReadService,
    MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS, StockCandidateRevisionExpectation,
};
use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CurrencyCode, DensityKilogramsPerCubicMeter, EffectiveDate,
    LibraryRecordState, MachineEnvelopeMillimeters, MachineProfile, MachineProfileId,
    MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId, Money, ProcessClass,
    RecordedAt, RemovalRateCubicMillimetersPerMinute, RuntimeMinutes, RuntimeProfileId,
    ShopProfileId, ShopResourceLibrary, ShopResourceLibraryId, ShopResourceVersion,
    ShopSettingsDraft, ShopSettingsRevision, SourceKind, SourceRef, StockAllowanceMillimeters,
    StockAllowanceProfile, StockAllowanceProfileId, StockForm, ValueState,
};
use partprobe_geometry_core::{
    ExactStepEnvelopeDerivative, ProvisionalGeometryDecimal, ProvisionalGeometrySnapshot,
    Sha256Digest, StageStatus,
};
use partprobe_geometry_import::{
    ControlledGeometryResult, PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE,
    PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE, ProvisionalExactStepAnalysis, SnapshotReference,
};
use partprobe_setup_planner::{
    AxisAlignedEnvelopeMillimeters,
    stock_candidate::{StockCandidateEdit, StockCandidateError},
};
use partprobe_setup_planner::{
    DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_ID, DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_VERSION,
    DeveloperEstimateProposalReadiness, DeveloperEstimateProposalReasonCode,
    EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION,
};
use rust_decimal::Decimal;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct CandidateTestAnalyzer(AnalyzedGeometryEvidence);

impl GeometryAnalysisPort for CandidateTestAnalyzer {
    fn analyze(
        &self,
        _request: &partprobe_geometry_import::GeometryWorkerRequest,
        _grant: partprobe_geometry_import::AssetReadGrant,
        _cancellation: &AtomicBool,
    ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure> {
        Ok(self.0.clone())
    }
}

#[derive(Debug)]
struct CandidateTestPolicy;

impl partprobe_security::AuthorizationPolicy for CandidateTestPolicy {
    fn evaluate(
        &self,
        _context: &partprobe_security::AuthorizationContext,
    ) -> partprobe_security::AuthorizationDecision {
        partprobe_security::AuthorizationDecision::allow(
            partprobe_security::SecurityPolicyRef::new(
                partprobe_security::SecurityPolicyId::new("synthetic-candidate-test").unwrap(),
                partprobe_security::SecurityPolicyVersion::new(1).unwrap(),
            ),
            partprobe_security::AuthorizationReasonCode::new("TEST_ONLY").unwrap(),
        )
    }
}

#[derive(Debug)]
struct CandidateTestAudit;

impl partprobe_security::AuthorizationAuditSink for CandidateTestAudit {
    fn append(
        &self,
        _event: partprobe_security::AuthorizationAuditEvent,
    ) -> Result<(), partprobe_security::AuditAppendError> {
        Ok(())
    }
}

fn candidate_session(geometry: AnalyzedGeometryEvidence) -> DraftEstimateSession {
    let sequence = SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "partprobe-stock-candidate-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(
        directory.join("synthetic.step"),
        b"public synthetic test input",
    )
    .unwrap();
    let root = partprobe_geometry_import::LocalAssetRoot::open(
        partprobe_domain::AssetRootId::new("candidate-test-root").unwrap(),
        &directory,
    )
    .unwrap();
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(CandidateTestPolicy, CandidateTestAudit),
        CandidateTestAnalyzer(geometry),
    );
    let subject = AssetReadSubject::new(
        ActorId::new("test-estimator").unwrap(),
        partprobe_domain::ProjectId::new("test-project").unwrap(),
        partprobe_domain::RecordId::new("test-model").unwrap(),
        partprobe_domain::RecordVersionId::new("test-revision").unwrap(),
        partprobe_domain::DataClassificationId::new("public-synthetic").unwrap(),
        partprobe_domain::RecordStateId::new("draft").unwrap(),
        partprobe_security::AuditCorrelationId::new("candidate-test-audit").unwrap(),
        RecordedAt::new("2026-09-15T12:00:00Z").unwrap(),
    );
    let request = partprobe_geometry_import::GeometryWorkerRequest::new(
        partprobe_domain::SchemaVersion::new(1).unwrap(),
        partprobe_geometry_import::GeometryJobId::new("test-job").unwrap(),
        partprobe_geometry_import::CorrelationId::new("test-correlation").unwrap(),
        partprobe_geometry_import::AssetCapability::new("test-capability").unwrap(),
        digest(SOURCE_HASH),
        vec![partprobe_geometry_core::GeometryStage::BasicProperties],
        partprobe_geometry_core::AnalysisProfile {
            id: partprobe_geometry_core::AnalysisProfileId::new("test-profile").unwrap(),
            version: partprobe_domain::RuleVersion::new(1, 0, 0),
        },
        partprobe_geometry_import::ResourceQuotas::new(1_000_000, 1_000_000, 10_000, 5_000)
            .unwrap(),
    )
    .unwrap();
    let session = application
        .start_session(
            subject,
            &root,
            &request,
            Path::new("synthetic.step"),
            &AtomicBool::new(false),
        )
        .unwrap();
    drop(root);
    std::fs::remove_file(directory.join("synthetic.step")).unwrap();
    std::fs::remove_dir(&directory).unwrap();
    session
}

fn prism_session() -> DraftEstimateSession {
    candidate_session(exact_geometry(
        ["12", "8", "5"],
        "392",
        "480",
        ["6", "4", "2.5"],
    ))
}

fn candidate_id() -> partprobe_domain::RecordId {
    partprobe_domain::RecordId::new("test-stock").unwrap()
}

fn expected_candidate(revision: u32) -> StockCandidateRevisionExpectation {
    StockCandidateRevisionExpectation {
        candidate_id: candidate_id(),
        revision,
    }
}

fn candidate_edit(x: &str) -> StockCandidateEdit {
    StockCandidateEdit::new(
        AxisAlignedEnvelopeMillimeters::new(
            partprobe_setup_planner::CanonicalMillimeterLength::new(decimal(x)).unwrap(),
            partprobe_setup_planner::CanonicalMillimeterLength::new(decimal("12")).unwrap(),
            partprobe_setup_planner::CanonicalMillimeterLength::new(decimal("8")).unwrap(),
        ),
        ActorId::new("test-estimator").unwrap(),
        RecordedAt::new("2026-09-15T12:00:00Z").unwrap(),
        "review saw blank dimensions",
    )
    .unwrap()
}

fn candidate_review() -> DeveloperEstimateProposalReview {
    DeveloperEstimateProposalReview {
        proposal_values_reviewed: true,
        coarse_limitations_accepted: true,
        actor: ActorId::new("test-reviewer").unwrap(),
        recorded_at: RecordedAt::new("2026-09-15T13:00:00Z").unwrap(),
        reason: "reviewed edited blank and coarse exclusions".to_owned(),
    }
}

#[test]
fn session_retains_original_and_reviewed_predecessor_without_inheriting_review() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    let original_geometry = session.geometry().clone();
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    let first = session.stock_candidate_revisions()[0].candidate().clone();
    assert_eq!(first.blank_volume_mm3().value(), decimal("1920"));
    session
        .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
        .unwrap();
    let reviewed_first = session.stock_candidate_revisions()[0].clone();
    assert_eq!(
        reviewed_first.session_rule_id(),
        "partprobe-developer-stock-candidate-session"
    );
    assert_eq!(reviewed_first.session_rule_version(), (1, 0, 0));
    session
        .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("18"))
        .unwrap();
    assert_eq!(session.stock_candidate_revisions().len(), 2);
    assert_eq!(session.stock_candidate_revisions()[0], reviewed_first);
    let second = &session.stock_candidate_revisions()[1];
    assert_eq!(second.candidate().revision(), 2);
    assert_eq!(
        second.candidate().previous_dimensions_mm(),
        first.edit().dimensions_mm()
    );
    assert_eq!(
        second.candidate().original_proposal(),
        first.original_proposal()
    );
    assert!(second.review().is_none());
    assert_eq!(
        second.candidate().readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
    assert_eq!(session.geometry(), &original_geometry);
    assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));
}

#[test]
fn session_rejects_stale_or_wrong_identity_edits_reviews_and_reinitialization() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    session
        .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("18"))
        .unwrap();
    let before = session.clone();
    for expectation in [
        expected_candidate(1),
        expected_candidate(3),
        StockCandidateRevisionExpectation {
            candidate_id: partprobe_domain::RecordId::new("other-stock").unwrap(),
            revision: 2,
        },
    ] {
        assert_eq!(
            session
                .revise_stock_candidate(&expectation, &settings, candidate_edit("22"))
                .unwrap_err(),
            DeveloperStockCandidateSessionError::StaleRevision
        );
        assert_eq!(
            session
                .review_stock_candidate(&expectation, &settings, candidate_review())
                .unwrap_err(),
            DeveloperStockCandidateSessionError::StaleRevision
        );
        assert_eq!(session, before);
    }
    assert_eq!(
        session
            .create_stock_candidate(candidate_id(), &settings, candidate_edit("22"))
            .unwrap_err(),
        DeveloperStockCandidateSessionError::AlreadyStarted
    );
    assert_eq!(session, before);
}

#[test]
fn changed_settings_bytes_even_under_reused_versions_reject_edits_and_reviews() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    let changed = ShopSettingsDraft::new_with_resources(
        settings.profile_id().clone(),
        settings.revision(),
        settings.currency().clone(),
        None,
        None,
        Some(library(
            LibraryRecordState::Draft,
            StockForm::Rectangular,
            ProcessClass::Milling,
            ["400", "500", "500"],
        )),
        ActorId::new("test-editor").unwrap(),
        RecordedAt::new("2026-09-10T13:00:00Z").unwrap(),
        "persisted developer-demo resource draft",
    )
    .unwrap();
    let before = session.clone();
    assert_eq!(
        session
            .revise_stock_candidate(&expected_candidate(1), &changed, candidate_edit("22"))
            .unwrap_err(),
        DeveloperStockCandidateSessionError::StaleSettings
    );
    assert_eq!(
        session
            .review_stock_candidate(&expected_candidate(1), &changed, candidate_review())
            .unwrap_err(),
        DeveloperStockCandidateSessionError::StaleSettings
    );
    assert_eq!(session, before);
}

#[test]
fn invalid_candidate_review_preserves_state_and_review_is_append_once() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    let before = session.clone();
    let mut bad = candidate_review();
    bad.proposal_values_reviewed = false;
    assert_eq!(
        session
            .review_stock_candidate(&expected_candidate(1), &settings, bad)
            .unwrap_err(),
        DeveloperStockCandidateSessionError::NotReviewed
    );
    let mut bad = candidate_review();
    bad.coarse_limitations_accepted = false;
    assert_eq!(
        session
            .review_stock_candidate(&expected_candidate(1), &settings, bad)
            .unwrap_err(),
        DeveloperStockCandidateSessionError::LimitationsNotAccepted
    );
    for reason in [" ".to_owned(), "x".repeat(1025), "bad\0reason".to_owned()] {
        let mut bad = candidate_review();
        bad.reason = reason;
        assert_eq!(
            session
                .review_stock_candidate(&expected_candidate(1), &settings, bad)
                .unwrap_err(),
            DeveloperStockCandidateSessionError::InvalidReviewReason
        );
    }
    assert_eq!(session, before);
    let review = candidate_review();
    session
        .review_stock_candidate(&expected_candidate(1), &settings, review.clone())
        .unwrap();
    assert_eq!(
        session.stock_candidate_revisions()[0].review(),
        Some(&review)
    );
    let reviewed = session.clone();
    assert_eq!(
        session
            .review_stock_candidate(&expected_candidate(1), &settings, review)
            .unwrap_err(),
        DeveloperStockCandidateSessionError::AlreadyReviewed
    );
    assert_eq!(session, reviewed);
}

#[test]
fn candidate_history_limit_rejects_new_edits_without_dropping_evidence() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    for revision in 1..MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS as u32 {
        session
            .revise_stock_candidate(
                &expected_candidate(revision),
                &settings,
                candidate_edit(if revision % 2 == 1 { "18" } else { "20" }),
            )
            .unwrap();
    }
    let before = session.clone();
    assert_eq!(
        session
            .revise_stock_candidate(&expected_candidate(32), &settings, candidate_edit("22"))
            .unwrap_err(),
        DeveloperStockCandidateSessionError::HistoryLimit
    );
    assert_eq!(session, before);
    session
        .review_stock_candidate(&expected_candidate(32), &settings, candidate_review())
        .unwrap();
    assert_eq!(
        session.stock_candidate_revisions().len(),
        MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS
    );
}

#[test]
fn missing_resources_legacy_analysis_and_uninitialized_review_remain_unavailable() {
    for (mut session, settings) in [
        (prism_session(), settings(None)),
        (
            candidate_session(legacy_geometry()),
            settings(Some(valid_library())),
        ),
    ] {
        let before = session.clone();
        assert_eq!(
            session
                .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
                .unwrap_err(),
            DeveloperStockCandidateSessionError::Unavailable
        );
        assert_eq!(
            session
                .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
                .unwrap_err(),
            DeveloperStockCandidateSessionError::Unavailable
        );
        assert_eq!(session, before);
    }
}

#[test]
fn failed_candidate_calculation_preserves_latest_revision_and_existing_review() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    session
        .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
        .unwrap();
    let before = session.clone();
    assert_eq!(
        session
            .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("20"))
            .unwrap_err(),
        DeveloperStockCandidateSessionError::Candidate(StockCandidateError::NoStockChange)
    );
    assert_eq!(
        session
            .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("11"))
            .unwrap_err(),
        DeveloperStockCandidateSessionError::Candidate(
            StockCandidateError::BlankDoesNotEncloseModel
        )
    );
    assert_eq!(session, before);
}

#[test]
fn edited_and_reviewed_stock_does_not_replace_the_accepted_deterministic_baseline() {
    let mut session = prism_session();
    let settings = settings(Some(valid_library()));
    let ValueState::Available { value: proposal } =
        DeveloperEstimateProposalApplication.propose(session.geometry(), &settings)
    else {
        panic!("valid original proposal")
    };
    let adopted = DeveloperEstimateProposalApplication
        .adopt(
            proposal,
            DraftQuantityInputs {
                deliver: partprobe_domain::ItemQuantity::new(2),
                planned_spares: partprobe_domain::ItemQuantity::new(1),
                destructive_samples: partprobe_domain::ItemQuantity::new(0),
            },
            candidate_review(),
        )
        .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../estimation-engine/tests/fixtures/task_002/golden_estimates.json"
    ))
    .unwrap();
    let card: partprobe_domain::RateCard =
        serde_json::from_value(fixture["rate_card"].clone()).unwrap();
    let policy = serde_json::from_value(fixture["pricing_policy"].clone()).unwrap();
    session.set_geometry_review(partprobe_application::DraftGeometryReview::new(true, true));
    session.set_inputs(adopted.inputs);
    session.set_rate_context(
        partprobe_application::DraftRateContext::new(
            card,
            EffectiveDate::new("2026-07-29").unwrap(),
            vec![partprobe_domain::RateScope::organization()],
        )
        .unwrap(),
    );
    session.set_pricing_policy(policy);
    let baseline = session.evaluate();
    assert!(matches!(baseline, ValueState::Available { .. }));
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    session
        .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
        .unwrap();
    assert_eq!(session.evaluate(), baseline);
    session
        .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("18"))
        .unwrap();
    assert_eq!(session.evaluate(), baseline);
}

#[test]
fn new_analysis_session_does_not_inherit_candidate_history_or_review() {
    let settings = settings(Some(valid_library()));
    let mut previous = prism_session();
    previous
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    previous
        .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
        .unwrap();
    let before = previous.clone();
    let mut current = candidate_session(exact_geometry(
        ["10", "10", "10"],
        "600",
        "1000",
        ["5", "5", "5"],
    ));
    assert!(current.stock_candidate_revisions().is_empty());
    assert_eq!(
        current
            .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
            .unwrap_err(),
        DeveloperStockCandidateSessionError::Unavailable
    );
    assert_eq!(
        current
            .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("22"))
            .unwrap_err(),
        DeveloperStockCandidateSessionError::Unavailable
    );
    assert_eq!(previous, before);
}

fn adoption_decision() -> partprobe_application::StockCandidateAdoptionDecision {
    partprobe_application::StockCandidateAdoptionDecision {
        actor: ActorId::new("test-adopter").unwrap(),
        recorded_at: RecordedAt::new("2026-09-15T14:00:00Z").unwrap(),
        reason: "use this exact reviewed stock revision for the coarse test estimate".to_owned(),
    }
}

fn quantity_three() -> DraftQuantityInputs {
    DraftQuantityInputs {
        deliver: partprobe_domain::ItemQuantity::new(2),
        planned_spares: partprobe_domain::ItemQuantity::new(1),
        destructive_samples: partprobe_domain::ItemQuantity::new(0),
    }
}

fn candidate_rate_and_policy() -> (
    partprobe_application::DraftRateContext,
    partprobe_domain::PricingPolicy,
) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../estimation-engine/tests/fixtures/task_002/golden_estimates.json"
    ))
    .unwrap();
    (
        partprobe_application::DraftRateContext::new(
            serde_json::from_value(fixture["rate_card"].clone()).unwrap(),
            EffectiveDate::new("2026-07-29").unwrap(),
            vec![partprobe_domain::RateScope::organization()],
        )
        .unwrap(),
        serde_json::from_value(fixture["pricing_policy"].clone()).unwrap(),
    )
}

fn configure_candidate_context(session: &mut DraftEstimateSession) {
    let (rates, policy) = candidate_rate_and_policy();
    session.set_geometry_review(partprobe_application::DraftGeometryReview::new(true, true));
    session.set_rate_context(rates);
    session.set_pricing_policy(policy);
}

fn reviewed_stock_session(settings: &ShopSettingsDraft) -> DraftEstimateSession {
    let mut session = prism_session();
    session
        .create_stock_candidate(candidate_id(), settings, candidate_edit("20"))
        .unwrap();
    session
        .review_stock_candidate(&expected_candidate(1), settings, candidate_review())
        .unwrap();
    session
}

#[test]
fn exact_reviewed_stock_adoption_maps_quantity_and_pins_distinct_evidence() {
    let settings = settings(Some(valid_library()));
    let session = reviewed_stock_session(&settings);
    let before = session.clone();
    let decision = adoption_decision();
    let adoption = session
        .adopt_stock_candidate(
            &expected_candidate(1),
            &settings,
            quantity_three(),
            decision.clone(),
        )
        .unwrap();
    assert_eq!(session, before);
    assert_eq!(
        adoption.rule_id(),
        "partprobe-developer-stock-candidate-adoption"
    );
    assert_eq!(adoption.rule_version(), (1, 0, 0));
    assert_eq!(adoption.decision(), &decision);
    assert_ne!(
        adoption.decision().actor,
        adoption.revision().review().unwrap().actor
    );
    assert_eq!(adoption.revision(), &session.stock_candidate_revisions()[0]);
    let inputs = adoption.inputs();
    assert_eq!(inputs.stock.stock_volume.value(), decimal("1920"));
    assert_eq!(inputs.stock.density.value(), decimal("0.0000027"));
    assert_eq!(inputs.material.purchased.amount(), decimal("0.132192"));
    assert_eq!(inputs.quantities, quantity_three());
    assert_eq!(inputs.times.cutting_hours_per_item, decimal("0.0015"));
    assert_eq!(inputs.times.setup_hours, Decimal::ONE);
    assert_eq!(inputs.times.programming_hours, decimal("0.75"));
    assert_eq!(inputs.times.load_unload_hours_per_item, decimal("0.05"));
    assert_eq!(inputs.times.quality_inspection_hours, decimal("0.25"));
    assert_eq!(inputs.times.non_cutting_hours_per_item, Decimal::ZERO);
    assert_eq!(inputs.operation.fixture.amount(), Decimal::ZERO);
    assert_eq!(inputs.base.overhead.amount(), Decimal::ZERO);
    assert_eq!(adoption.excluded_inputs().len(), 6);
    assert!(adoption.excluded_inputs().contains(&"risk_and_rework"));
    assert_eq!(
        adoption.revision().candidate().readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
}

#[test]
fn adoption_requires_recorded_review_valid_reason_and_bounded_quantity_arithmetic() {
    let settings = settings(Some(valid_library()));
    let mut session = prism_session();
    session
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    assert_eq!(
        session
            .adopt_stock_candidate(
                &expected_candidate(1),
                &settings,
                quantity_three(),
                adoption_decision()
            )
            .unwrap_err(),
        DeveloperStockCandidateSessionError::NotReviewed
    );
    session
        .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
        .unwrap();
    let before = session.clone();
    for reason in [" ".to_owned(), "x".repeat(1025), "bad\0reason".to_owned()] {
        let mut decision = adoption_decision();
        decision.reason = reason;
        assert_eq!(
            session
                .adopt_stock_candidate(
                    &expected_candidate(1),
                    &settings,
                    quantity_three(),
                    decision
                )
                .unwrap_err(),
            DeveloperStockCandidateSessionError::InvalidAdoptionReason
        );
    }
    let overflow = DraftQuantityInputs {
        deliver: partprobe_domain::ItemQuantity::new(u64::MAX),
        planned_spares: partprobe_domain::ItemQuantity::new(1),
        destructive_samples: partprobe_domain::ItemQuantity::new(0),
    };
    assert_eq!(
        session
            .adopt_stock_candidate(
                &expected_candidate(1),
                &settings,
                overflow,
                adoption_decision()
            )
            .unwrap_err(),
        DeveloperStockCandidateSessionError::AdoptionArithmeticFailure
    );
    assert_eq!(session, before);
}

#[test]
fn candidate_evaluation_preserves_baseline_and_uses_exact_stock_sensitive_inputs() {
    let settings = settings(Some(valid_library()));
    let mut session = reviewed_stock_session(&settings);
    let ValueState::Available { value: original } =
        DeveloperEstimateProposalApplication.propose(session.geometry(), &settings)
    else {
        panic!("original proposal")
    };
    session.set_inputs(
        DeveloperEstimateProposalApplication
            .adopt(original, quantity_three(), candidate_review())
            .unwrap()
            .inputs,
    );
    configure_candidate_context(&mut session);
    let before = session.clone();
    let baseline = session.evaluate();
    let adoption = session
        .adopt_stock_candidate(
            &expected_candidate(1),
            &settings,
            quantity_three(),
            adoption_decision(),
        )
        .unwrap();
    let ValueState::Available { value: layer } =
        session.evaluate_stock_candidate(&settings, &adoption)
    else {
        panic!("reviewed candidate with complete context must evaluate")
    };
    assert_eq!(layer.baseline(), &baseline);
    assert_eq!(layer.adoption(), &adoption);
    assert_eq!(layer.result().stock_mass.value(), decimal("0.005184"));
    assert_eq!(layer.result().material_cost.amount(), decimal("0.132192"));
    assert_eq!(layer.result().trace.inputs, *adoption.inputs());
    let ValueState::Available {
        value: original_result,
    } = &baseline
    else {
        panic!("original baseline")
    };
    assert!(
        layer.result().total_internal_cost.amount() > original_result.total_internal_cost.amount()
    );
    assert_eq!(
        layer.result().net_part_volume,
        original_result.net_part_volume
    );
    assert_eq!(layer.result().setup_cost, original_result.setup_cost);
    assert_eq!(
        layer.result().programming_cost,
        original_result.programming_cost
    );
    assert_eq!(
        layer.result().trace.pricing_policy,
        original_result.trace.pricing_policy
    );
    assert_eq!(layer.rate_context(), &candidate_rate_and_policy().0);
    assert_eq!(session, before);
    assert_eq!(session.evaluate(), baseline);
}

#[test]
fn old_stock_adoption_is_retained_but_cannot_evaluate_after_revision_or_settings_change() {
    let settings = settings(Some(valid_library()));
    let mut session = reviewed_stock_session(&settings);
    configure_candidate_context(&mut session);
    let adoption = session
        .adopt_stock_candidate(
            &expected_candidate(1),
            &settings,
            quantity_three(),
            adoption_decision(),
        )
        .unwrap();
    let old = adoption.clone();
    session
        .revise_stock_candidate(&expected_candidate(1), &settings, candidate_edit("18"))
        .unwrap();
    assert_eq!(
        session
            .adopt_stock_candidate(
                &expected_candidate(1),
                &settings,
                quantity_three(),
                adoption_decision()
            )
            .unwrap_err(),
        DeveloperStockCandidateSessionError::StaleRevision
    );
    assert!(matches!(
        session.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Blocked { .. }
    ));
    assert_eq!(
        session
            .adopt_stock_candidate(
                &expected_candidate(2),
                &settings,
                quantity_three(),
                adoption_decision()
            )
            .unwrap_err(),
        DeveloperStockCandidateSessionError::NotReviewed
    );
    session
        .review_stock_candidate(&expected_candidate(2), &settings, candidate_review())
        .unwrap();
    let current = session
        .adopt_stock_candidate(
            &expected_candidate(2),
            &settings,
            quantity_three(),
            adoption_decision(),
        )
        .unwrap();
    let changed_settings = self::settings(None);
    assert_eq!(
        session
            .adopt_stock_candidate(
                &expected_candidate(2),
                &changed_settings,
                quantity_three(),
                adoption_decision()
            )
            .unwrap_err(),
        DeveloperStockCandidateSessionError::StaleSettings
    );
    assert!(matches!(
        session.evaluate_stock_candidate(&changed_settings, &current),
        ValueState::Blocked { .. }
    ));
    assert_eq!(adoption, old);
    assert!(matches!(
        session.evaluate_stock_candidate(&settings, &current),
        ValueState::Available { .. }
    ));
}

#[test]
fn candidate_from_another_analysis_with_reused_identity_is_not_a_current_adoption() {
    let settings = settings(Some(valid_library()));
    let session = reviewed_stock_session(&settings);
    let adoption = session
        .adopt_stock_candidate(
            &expected_candidate(1),
            &settings,
            quantity_three(),
            adoption_decision(),
        )
        .unwrap();
    let mut other = candidate_session(exact_geometry(
        ["12", "7", "5"],
        "358",
        "420",
        ["6", "3.5", "2.5"],
    ));
    other
        .create_stock_candidate(candidate_id(), &settings, candidate_edit("20"))
        .unwrap();
    other
        .review_stock_candidate(&expected_candidate(1), &settings, candidate_review())
        .unwrap();
    configure_candidate_context(&mut other);
    let before = other.clone();
    assert!(matches!(
        other.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Blocked { .. }
    ));
    assert_eq!(other, before);
}

#[test]
fn candidate_evaluation_requires_units_warnings_rates_and_policy_without_defaulting_baseline() {
    let settings = settings(Some(valid_library()));
    let mut session = reviewed_stock_session(&settings);
    let adoption = session
        .adopt_stock_candidate(
            &expected_candidate(1),
            &settings,
            quantity_three(),
            adoption_decision(),
        )
        .unwrap();
    assert!(matches!(
        session.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Unavailable { .. }
    ));
    let (rates, policy) = candidate_rate_and_policy();
    session.set_rate_context(rates.clone());
    session.set_pricing_policy(policy.clone());
    for (units, warnings) in [(false, true), (true, false)] {
        session.set_geometry_review(partprobe_application::DraftGeometryReview::new(
            units, warnings,
        ));
        assert!(matches!(
            session.evaluate_stock_candidate(&settings, &adoption),
            ValueState::Unavailable { .. }
        ));
    }
    session.set_geometry_review(partprobe_application::DraftGeometryReview::new(true, true));
    let mut missing_rates = reviewed_stock_session(&settings);
    missing_rates.set_geometry_review(partprobe_application::DraftGeometryReview::new(true, true));
    missing_rates.set_pricing_policy(policy);
    assert!(matches!(
        missing_rates.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Unavailable { .. }
    ));
    let mut missing_policy = reviewed_stock_session(&settings);
    missing_policy.set_geometry_review(partprobe_application::DraftGeometryReview::new(true, true));
    missing_policy.set_rate_context(rates.clone());
    assert!(matches!(
        missing_policy.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Unavailable { .. }
    ));
    let ValueState::Available { value: layer } =
        session.evaluate_stock_candidate(&settings, &adoption)
    else {
        panic!("complete candidate context")
    };
    assert!(matches!(layer.baseline(), ValueState::Unavailable { .. }));
    let empty = partprobe_domain::RateCard::empty(
        rates.rate_card.id().clone(),
        rates.rate_card.version(),
        rates.rate_card.currency().clone(),
    );
    session.set_rate_context(
        partprobe_application::DraftRateContext::new(
            empty,
            rates.effective_on,
            rates.ordered_scopes,
        )
        .unwrap(),
    );
    assert!(matches!(
        session.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Unavailable { .. }
    ));
}

#[test]
fn conflicting_rates_block_candidate_evaluation_and_do_not_mutate_retained_adoption() {
    let settings = settings(Some(valid_library()));
    let mut session = reviewed_stock_session(&settings);
    configure_candidate_context(&mut session);
    let adoption = session
        .adopt_stock_candidate(
            &expected_candidate(1),
            &settings,
            quantity_three(),
            adoption_decision(),
        )
        .unwrap();
    let retained = session.evaluate_stock_candidate(&settings, &adoption);
    assert!(matches!(retained, ValueState::Available { .. }));
    let original_layer = retained.clone();
    let (rates, _) = candidate_rate_and_policy();
    let setup = rates
        .rate_card
        .entries()
        .iter()
        .find(|entry| entry.id().as_str() == "setup-labor")
        .unwrap();
    let mut duplicate = serde_json::to_value(setup).unwrap();
    duplicate["id"] = serde_json::Value::String("conflicting-setup-rate".to_owned());
    let mut entries = rates.rate_card.entries().to_vec();
    entries.push(serde_json::from_value(duplicate).unwrap());
    let conflicting = partprobe_domain::RateCard::new(
        rates.rate_card.id().clone(),
        rates.rate_card.version(),
        rates.rate_card.currency().clone(),
        entries,
    )
    .unwrap();
    session.set_rate_context(
        partprobe_application::DraftRateContext::new(
            conflicting,
            rates.effective_on,
            rates.ordered_scopes,
        )
        .unwrap(),
    );
    let before = session.clone();
    assert!(matches!(
        session.evaluate_stock_candidate(&settings, &adoption),
        ValueState::Blocked { .. }
    ));
    assert_eq!(session, before);
    assert_eq!(retained, original_layer);
}

const SOURCE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const OUTPUT_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).expect("test decimal must be exact")
}

fn geometry_decimal(value: &str) -> ProvisionalGeometryDecimal {
    ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be valid")
}

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::new(value).expect("test hash must be valid")
}

fn source(label: &str) -> SourceRef {
    SourceRef::new(
        SourceKind::Manual,
        label,
        Some("test-v1".to_owned()),
        Some(RecordedAt::new("2026-09-10T12:00:00Z").expect("time must be valid")),
    )
    .expect("source must be valid")
}

fn snapshot(
    surface_area_mm2: &str,
    volume_mm3: &str,
    center_of_mass_mm: [&str; 3],
) -> ProvisionalGeometrySnapshot {
    ProvisionalGeometrySnapshot::new(
        digest(SOURCE_HASH),
        "8.0.0",
        3,
        1,
        1,
        geometry_decimal(surface_area_mm2),
        geometry_decimal(volume_mm3),
        center_of_mass_mm.map(geometry_decimal),
    )
    .expect("snapshot must be valid")
}

fn exact_geometry(
    extents_mm: [&str; 3],
    surface_area_mm2: &str,
    volume_mm3: &str,
    center_of_mass_mm: [&str; 3],
) -> AnalyzedGeometryEvidence {
    let snapshot = snapshot(surface_area_mm2, volume_mm3, center_of_mass_mm);
    let envelope =
        ExactStepEnvelopeDerivative::new(digest(SOURCE_HASH), extents_mm.map(geometry_decimal))
            .expect("envelope must be valid");
    let analysis = ProvisionalExactStepAnalysis::new(snapshot, envelope)
        .expect("analysis must be source-bound");
    AnalyzedGeometryEvidence::new(
        StageStatus::Succeeded,
        Vec::new(),
        SnapshotReference::new(PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE)
            .expect("reference must be valid"),
        digest(OUTPUT_HASH),
        512,
        ControlledGeometryResult::ExactBrepWithEnvelope(Box::new(analysis)),
        None,
        None,
    )
    .expect("analyzed evidence must be valid")
}

fn legacy_geometry() -> AnalyzedGeometryEvidence {
    AnalyzedGeometryEvidence::new(
        StageStatus::Succeeded,
        Vec::new(),
        SnapshotReference::new(PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE)
            .expect("reference must be valid"),
        digest(OUTPUT_HASH),
        256,
        ControlledGeometryResult::ExactBrep(Box::new(snapshot("600", "1000", ["5", "5", "5"]))),
        None,
        None,
    )
    .expect("legacy evidence must be valid")
}

fn library(
    record_state: LibraryRecordState,
    stock_form: StockForm,
    process_class: ProcessClass,
    machine_envelope_mm: [&str; 3],
) -> ShopResourceLibrary {
    let material_version = ShopResourceVersion::new(2).expect("version must be valid");
    let offer_version = ShopResourceVersion::new(3).expect("version must be valid");
    let allowance_version = ShopResourceVersion::new(4).expect("version must be valid");
    let machine_version = ShopResourceVersion::new(5).expect("version must be valid");
    let runtime_version = ShopResourceVersion::new(6).expect("version must be valid");
    let material_id = MaterialDefinitionId::new("al-6061-t6").expect("ID must be valid");
    let machine_id = MachineProfileId::new("vmc-1").expect("ID must be valid");
    let material = MaterialDefinition::new(
        material_id.clone(),
        material_version,
        "Aluminum",
        "6061",
        None,
        Some("T6".to_owned()),
        DensityKilogramsPerCubicMeter::new(decimal("2700")).expect("density must be valid"),
        source("material-test"),
        record_state,
    )
    .expect("material must be valid");
    let offer = MaterialOffer::new(
        MaterialOfferId::new("al-offer").expect("ID must be valid"),
        offer_version,
        material_id.clone(),
        material_version,
        "test supplier",
        Money::new(
            decimal("8.50"),
            CurrencyCode::new("USD").expect("currency must be valid"),
        ),
        EffectiveDate::new("2026-09-10").expect("date must be valid"),
        source("offer-test"),
        record_state,
    )
    .expect("offer must be valid");
    let allowance = StockAllowanceProfile::new(
        StockAllowanceProfileId::new("rectangular-default").expect("ID must be valid"),
        allowance_version,
        stock_form,
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("2")).expect("allowance must be valid"),
        source("allowance-test"),
        record_state,
    );
    let machine = MachineProfile::new(
        machine_id.clone(),
        machine_version,
        "Test VMC",
        process_class,
        MachineEnvelopeMillimeters::new(decimal(machine_envelope_mm[0]))
            .expect("envelope must be valid"),
        MachineEnvelopeMillimeters::new(decimal(machine_envelope_mm[1]))
            .expect("envelope must be valid"),
        MachineEnvelopeMillimeters::new(decimal(machine_envelope_mm[2]))
            .expect("envelope must be valid"),
        source("machine-test"),
        record_state,
    )
    .expect("machine must be valid");
    let runtime = CoarseRuntimeProfile::new(
        RuntimeProfileId::new("vmc-al").expect("ID must be valid"),
        runtime_version,
        machine_id,
        machine_version,
        material_id,
        material_version,
        RemovalRateCubicMillimetersPerMinute::new(decimal("16000"))
            .expect("removal rate must be valid"),
        RuntimeMinutes::new(decimal("60")).expect("setup must be valid"),
        RuntimeMinutes::new(decimal("45")).expect("programming must be valid"),
        RuntimeMinutes::new(decimal("3")).expect("handling must be valid"),
        RuntimeMinutes::new(decimal("15")).expect("quality must be valid"),
        source("runtime-test"),
        record_state,
    );
    ShopResourceLibrary::new(
        ShopResourceLibraryId::new("starter-library").expect("ID must be valid"),
        ShopResourceVersion::new(9).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        material,
        offer,
        allowance,
        machine,
        runtime,
    )
    .expect("library must be internally consistent")
}

fn settings(resources: Option<ShopResourceLibrary>) -> ShopSettingsDraft {
    ShopSettingsDraft::new_with_resources(
        ShopProfileId::new("test-shop").expect("profile must be valid"),
        ShopSettingsRevision::new(7).expect("revision must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        None,
        None,
        resources,
        ActorId::new("test-editor").expect("actor must be valid"),
        RecordedAt::new("2026-09-10T13:00:00Z").expect("time must be valid"),
        "persisted developer-demo resource draft",
    )
    .expect("settings must be valid")
}

fn valid_library() -> ShopResourceLibrary {
    library(
        LibraryRecordState::Draft,
        StockForm::Rectangular,
        ProcessClass::Milling,
        ["500", "500", "500"],
    )
}

#[test]
fn developer_proposal_pins_evidence_and_derives_exact_cost_and_coarse_times() {
    let result = DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings(Some(valid_library())),
    );
    let ValueState::Available { value } = result else {
        panic!("valid exact evidence and persisted draft must produce a proposal")
    };

    assert_eq!(
        value.evidence_schema_version(),
        EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION
    );
    assert_eq!(value.rule_id(), DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_ID);
    assert_eq!(
        value.rule_version(),
        DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_VERSION
    );
    assert_eq!(value.source_hash().as_str(), SOURCE_HASH);
    assert_eq!(value.analysis_output_hash().as_str(), OUTPUT_HASH);
    assert_eq!(value.library_id().as_str(), "starter-library");
    assert_eq!(value.library_version().value(), 9);
    assert_eq!(value.material_id().as_str(), "al-6061-t6");
    assert_eq!(value.material_version().value(), 2);
    assert_eq!(value.material_offer_id().as_str(), "al-offer");
    assert_eq!(value.material_offer_version().value(), 3);
    assert_eq!(value.stock_allowance_id().as_str(), "rectangular-default");
    assert_eq!(value.stock_allowance_version().value(), 4);
    assert_eq!(value.machine_id().as_str(), "vmc-1");
    assert_eq!(value.machine_version().value(), 5);
    assert_eq!(value.runtime_id().as_str(), "vmc-al");
    assert_eq!(value.runtime_version().value(), 6);

    assert_eq!(
        value
            .total_allowance_mm()
            .map(|allowance| allowance.value()),
        [decimal("3"), decimal("3"), decimal("2")]
    );
    assert_eq!(value.blank_dimensions_mm().x().value(), decimal("15"));
    assert_eq!(value.blank_dimensions_mm().y().value(), decimal("11"));
    assert_eq!(value.blank_dimensions_mm().z().value(), decimal("7"));
    assert_eq!(value.blank_volume_mm3().value(), decimal("1155"));
    assert_eq!(value.removed_volume_mm3().value(), decimal("675"));
    assert_eq!(value.material_density_kg_per_m3().value(), decimal("2700"));
    assert_eq!(value.blank_mass_kg().value(), decimal("0.0031185"));
    assert_eq!(value.material_price_per_kg().amount(), decimal("8.50"));
    assert_eq!(
        value.unit_stock_material_cost().amount(),
        decimal("0.026507250")
    );
    assert_eq!(value.unit_stock_material_cost().currency().as_str(), "USD");
    assert_eq!(
        value.removal_rate_mm3_per_minute().value(),
        decimal("16000")
    );
    assert_eq!(value.coarse_times().setup().value(), decimal("60"));
    assert_eq!(value.coarse_times().programming().value(), decimal("45"));
    assert_eq!(value.coarse_times().cutting().value(), decimal("0.0421875"));
    assert_eq!(value.coarse_times().load_unload().value(), decimal("3"));
    assert_eq!(value.coarse_times().quality().value(), decimal("15"));
    assert_eq!(
        value.readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
    assert!(
        value
            .reason_codes()
            .contains(&DeveloperEstimateProposalReasonCode::CoarseRuntimeNotCam)
    );
    assert!(
        value
            .reason_codes()
            .contains(&DeveloperEstimateProposalReasonCode::HumanReviewAndAdoptionRequired)
    );
}

#[test]
fn distinct_cube_and_prism_geometry_produce_distinct_cost_and_cutting_evidence() {
    let settings = settings(Some(valid_library()));
    let cube = DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["10", "10", "10"], "600", "1000", ["5", "5", "5"]),
        &settings,
    );
    let prism = DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings,
    );
    let (ValueState::Available { value: cube }, ValueState::Available { value: prism }) =
        (cube, prism)
    else {
        panic!("both analytic solids must produce proposals")
    };

    assert_eq!(cube.blank_volume_mm3().value(), decimal("2028"));
    assert_eq!(cube.removed_volume_mm3().value(), decimal("1028"));
    assert_eq!(cube.blank_mass_kg().value(), decimal("0.0054756"));
    assert_eq!(
        cube.unit_stock_material_cost().amount(),
        decimal("0.046542600")
    );
    assert_eq!(cube.coarse_times().cutting().value(), decimal("0.06425"));
    assert_ne!(cube.blank_volume_mm3(), prism.blank_volume_mm3());
    assert_ne!(
        cube.unit_stock_material_cost(),
        prism.unit_stock_material_cost()
    );
    assert_ne!(
        cube.coarse_times().cutting(),
        prism.coarse_times().cutting()
    );
}

#[test]
fn explicit_review_adopts_model_sensitive_inputs_and_records_excluded_categories_as_zero() {
    let proposal = match DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings(Some(valid_library())),
    ) {
        ValueState::Available { value } => value,
        _ => panic!("valid proposal must be available"),
    };
    let adopted = DeveloperEstimateProposalApplication
        .adopt(
            proposal,
            DraftQuantityInputs {
                deliver: partprobe_domain::ItemQuantity::new(2),
                planned_spares: partprobe_domain::ItemQuantity::new(1),
                destructive_samples: partprobe_domain::ItemQuantity::new(0),
            },
            DeveloperEstimateProposalReview {
                proposal_values_reviewed: true,
                coarse_limitations_accepted: true,
                actor: ActorId::new("test-estimator").expect("actor must be valid"),
                recorded_at: RecordedAt::new("2026-09-10T14:00:00Z").expect("time must be valid"),
                reason: "reviewed for one coarse developer estimate".to_owned(),
            },
        )
        .expect("explicit review must adopt the proposal");

    assert_eq!(adopted.inputs.stock.stock_volume.value(), decimal("1155"));
    assert_eq!(adopted.inputs.stock.density.value(), decimal("0.0000027"));
    assert_eq!(
        adopted.inputs.material.purchased.amount(),
        decimal("0.079521750")
    );
    assert_eq!(adopted.inputs.times.setup_hours, decimal("1"));
    assert_eq!(adopted.inputs.times.programming_hours, decimal("0.75"));
    assert_eq!(
        adopted.inputs.times.cutting_hours_per_item,
        decimal("0.000703125")
    );
    assert_eq!(
        adopted.inputs.times.load_unload_hours_per_item,
        decimal("0.05")
    );
    assert_eq!(
        adopted.inputs.times.quality_inspection_hours,
        decimal("0.25")
    );
    assert_eq!(adopted.inputs.operation.tooling.amount(), Decimal::ZERO);
    assert_eq!(adopted.inputs.base.overhead.amount(), Decimal::ZERO);
    assert_eq!(adopted.review.actor.as_str(), "test-estimator");
}

#[test]
fn adoption_requires_both_review_decisions_and_a_reason() {
    let proposal = match DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings(Some(valid_library())),
    ) {
        ValueState::Available { value } => value,
        _ => panic!("valid proposal must be available"),
    };
    let quantities = DraftQuantityInputs {
        deliver: partprobe_domain::ItemQuantity::new(1),
        planned_spares: partprobe_domain::ItemQuantity::new(0),
        destructive_samples: partprobe_domain::ItemQuantity::new(0),
    };
    let review = |values, limits, reason: &str| DeveloperEstimateProposalReview {
        proposal_values_reviewed: values,
        coarse_limitations_accepted: limits,
        actor: ActorId::new("test-estimator").expect("actor must be valid"),
        recorded_at: RecordedAt::new("2026-09-10T14:00:00Z").expect("time must be valid"),
        reason: reason.to_owned(),
    };

    assert_eq!(
        DeveloperEstimateProposalApplication.adopt(
            proposal.clone(),
            quantities.clone(),
            review(false, true, "reason")
        ),
        Err(DeveloperEstimateProposalAdoptionError::ProposalNotReviewed)
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.adopt(
            proposal.clone(),
            quantities.clone(),
            review(true, false, "reason")
        ),
        Err(DeveloperEstimateProposalAdoptionError::LimitationsNotAccepted)
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.adopt(proposal, quantities, review(true, true, "")),
        Err(DeveloperEstimateProposalAdoptionError::InvalidReviewReason)
    );
}

#[test]
fn missing_library_and_legacy_exact_geometry_remain_visibly_unavailable() {
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &exact_geometry(["10", "10", "10"], "600", "1000", ["5", "5", "5"]),
            &settings(None),
        ),
        ValueState::Unavailable {
            reason: "persisted starter resource-library draft is unavailable".to_owned(),
        }
    );
    assert_eq!(
        DeveloperEstimateProposalApplication
            .propose(&legacy_geometry(), &settings(Some(valid_library()))),
        ValueState::Unavailable {
            reason: "exact STEP envelope evidence is unavailable".to_owned(),
        }
    );
}

#[test]
fn non_draft_resource_and_unsupported_stock_form_fail_visibly() {
    let geometry = exact_geometry(["10", "10", "10"], "600", "1000", ["5", "5", "5"]);
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Reviewed,
                StockForm::Rectangular,
                ProcessClass::Milling,
                ["500", "500", "500"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: developer proposal requires draft material evidence"
                .to_owned(),
        }
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Draft,
                StockForm::Round,
                ProcessClass::Milling,
                ["500", "500", "500"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: stock-envelope rule supports only rectangular stock"
                .to_owned(),
        }
    );
}

#[test]
fn process_and_machine_mismatches_fail_instead_of_filling_estimate_inputs() {
    let geometry = exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]);
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Draft,
                StockForm::Rectangular,
                ProcessClass::Turning,
                ["500", "500", "500"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: developer proposal currently supports only coarse milling profiles"
                .to_owned(),
        }
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Draft,
                StockForm::Rectangular,
                ProcessClass::Milling,
                ["14", "10", "6"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: proposed rectangular blank exceeds the pinned machine envelope"
                .to_owned(),
        }
    );
}
