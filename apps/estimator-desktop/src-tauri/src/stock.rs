//! Native composition of the durable developer stock workflow.
//!
//! This adapter owns the only desktop path to `DeveloperStockApplication`. Every operation is
//! validated against the exact retained selection/analysis session, authorized by a visibly
//! labeled developer-session policy (not production identity or roles), appended to the durable
//! reference-only decision audit, and given one fresh correlation ID per attempt. Uncertain
//! attempts must be reconciled by the operator and retried with a new attempt, never replayed.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use partprobe_application::{
    AssetReadSubject, AxisAlignedEnvelopeMillimeters, CanonicalMillimeterLength,
    DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_ID,
    DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_VERSION, DEVELOPER_STOCK_WORKFLOW_RULE_ID,
    DEVELOPER_STOCK_WORKFLOW_RULE_VERSION, DeveloperEstimateProposalReview,
    DeveloperStockApplication, DeveloperStockAuthorizationContext,
    DeveloperStockAuthorizationPolicy, DeveloperStockCandidateRevision,
    DeveloperStockCandidateSessionError, DeveloperStockOutcome, DeveloperStockRequest,
    DeveloperStockWorkflowError, DraftEstimateSession, DraftQuantityInputs,
    STOCK_CANDIDATE_RULE_ID, STOCK_CANDIDATE_RULE_VERSION, StockCandidateAdoptionDecision,
    StockCandidateEdit, StockCandidateReasonCode, StockCandidateRevisionExpectation,
};
use partprobe_desktop_contract::{
    AdoptStockCandidateEstimateRequest, DraftEstimateEvaluationState, HostCommandError,
    ProposeStockCandidateRequest, ReviewStockCandidateRequest, StockCandidateEstimateEvaluation,
    StockCandidateEstimateSummary, StockCandidateRevisionRef, StockCandidateSnapshot,
};
use partprobe_domain::{ItemQuantity, RecordId, RecordedAt, ShopSettingsDraft, ValueState};
use partprobe_persistence_sqlite::SqliteDeveloperStockAuthorizationAudit;
use partprobe_security::{
    AuditCorrelationId, AuthorizationDecision, AuthorizationReasonCode, SecurityPolicyId,
    SecurityPolicyRef, SecurityPolicyVersion,
};
use rust_decimal::Decimal;

use crate::analysis::{
    DEVELOPER_ACTOR_ID, DEVELOPER_CLASSIFICATION_ID, DEVELOPER_PROJECT_ID,
    DEVELOPER_RECORD_STATE_ID, trusted_recorded_at,
};
use crate::proposal::{decimal_text, minutes_to_hours, proposal_reason_code};

const STOCK_SESSION_POLICY_ID: &str = "partprobe-desktop-developer-stock-session";
const STOCK_SESSION_POLICY_VERSION: u64 = 1;
const STOCK_SESSION_ALLOWED: &str = "DEVELOPER_SESSION_SUBJECT";
const STOCK_SESSION_DENIED: &str = "NOT_A_DEVELOPER_SESSION_SUBJECT";

/// Allows exactly the visibly labeled local developer-session identity used for analysis
/// intake. It is not authenticated production identity, a role model, or shop approval; any
/// other subject is denied and the decision is still recorded.
#[derive(Debug)]
pub(crate) struct DeveloperSessionStockPolicy {
    policy: SecurityPolicyRef,
}

impl DeveloperSessionStockPolicy {
    fn new() -> Result<Self, HostCommandError> {
        Ok(Self {
            policy: SecurityPolicyRef::new(
                SecurityPolicyId::new(STOCK_SESSION_POLICY_ID)
                    .map_err(|_| internal_stock_error())?,
                SecurityPolicyVersion::new(STOCK_SESSION_POLICY_VERSION)
                    .map_err(|_| internal_stock_error())?,
            ),
        })
    }
}

impl DeveloperStockAuthorizationPolicy for DeveloperSessionStockPolicy {
    fn evaluate(&self, context: &DeveloperStockAuthorizationContext) -> AuthorizationDecision {
        let allowed = context.actor_id().as_str() == DEVELOPER_ACTOR_ID
            && context.project_id().as_str() == DEVELOPER_PROJECT_ID
            && context.classification_id().as_str() == DEVELOPER_CLASSIFICATION_ID
            && context.record_state_id().as_str() == DEVELOPER_RECORD_STATE_ID;
        let reason = if allowed {
            AuthorizationReasonCode::new(STOCK_SESSION_ALLOWED)
        } else {
            AuthorizationReasonCode::new(STOCK_SESSION_DENIED)
        };
        let Ok(reason) = reason else {
            return AuthorizationDecision::deny(
                self.policy.clone(),
                AuthorizationReasonCode::new("INTERNAL").expect("static reason code must be valid"),
            );
        };
        if allowed {
            AuthorizationDecision::allow(self.policy.clone(), reason)
        } else {
            AuthorizationDecision::deny(self.policy.clone(), reason)
        }
    }
}

struct StockWorkflowRuntime {
    application: DeveloperStockApplication<
        DeveloperSessionStockPolicy,
        SqliteDeveloperStockAuthorizationAudit,
    >,
    attempt_sequence: u64,
    candidate_sequence: u64,
}

/// Host-owned stock workflow state. Without a verified durable audit store every stock
/// candidate command stays visibly unavailable; nothing falls back to an in-memory sink.
pub struct DesktopStockWorkflowState {
    runtime: Mutex<Option<StockWorkflowRuntime>>,
}

impl DesktopStockWorkflowState {
    pub fn open(database_path: &Path) -> Result<Self, HostCommandError> {
        let audit = SqliteDeveloperStockAuthorizationAudit::open(database_path)
            .map_err(|_| HostCommandError::stock_workflow_unavailable("VIS4-STOCK-AUDIT-OPEN"))?;
        Ok(Self {
            runtime: Mutex::new(Some(StockWorkflowRuntime {
                application: DeveloperStockApplication::new(
                    DeveloperSessionStockPolicy::new()?,
                    audit,
                ),
                attempt_sequence: 1,
                candidate_sequence: 1,
            })),
        })
    }

    #[must_use]
    pub const fn unavailable() -> Self {
        Self {
            runtime: Mutex::new(None),
        }
    }

    pub fn propose(
        &self,
        session: &mut DraftEstimateSession,
        settings: &ShopSettingsDraft,
        request: &ProposeStockCandidateRequest,
    ) -> Result<StockCandidateSnapshot, HostCommandError> {
        let mut guard = self.lock_runtime()?;
        let runtime = Self::require_runtime(&mut guard)?;
        let recorded_at = trusted_recorded_at()?;
        let edit = StockCandidateEdit::new(
            parse_dimensions(&request.final_dimensions_mm)?,
            session.source_actor().clone(),
            recorded_at.clone(),
            request.reason.clone(),
        )
        .map_err(|_| HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-EDIT"))?;
        let workflow_request = match &request.expected {
            None => DeveloperStockRequest::Create {
                candidate_id: runtime.next_candidate_id()?,
                edit,
            },
            Some(expected) => DeveloperStockRequest::Revise {
                expected: expectation(expected)?,
                edit,
            },
        };
        let outcome = runtime.execute(session, settings, recorded_at, workflow_request)?;
        let DeveloperStockOutcome::Revision(revision) = outcome else {
            return Err(internal_stock_error());
        };
        Ok(snapshot(
            &request.selection_id,
            &request.analysis_id,
            settings,
            &revision,
        ))
    }

    pub fn review(
        &self,
        session: &mut DraftEstimateSession,
        settings: &ShopSettingsDraft,
        request: &ReviewStockCandidateRequest,
    ) -> Result<StockCandidateSnapshot, HostCommandError> {
        let mut guard = self.lock_runtime()?;
        let runtime = Self::require_runtime(&mut guard)?;
        let recorded_at = trusted_recorded_at()?;
        let review = DeveloperEstimateProposalReview {
            proposal_values_reviewed: request.proposal_values_reviewed,
            coarse_limitations_accepted: request.coarse_limitations_accepted,
            actor: session.source_actor().clone(),
            recorded_at: recorded_at.clone(),
            reason: request.reason.clone(),
        };
        let workflow_request = DeveloperStockRequest::Review {
            expected: expectation(&request.expected)?,
            review,
        };
        let outcome = runtime.execute(session, settings, recorded_at, workflow_request)?;
        let DeveloperStockOutcome::Revision(revision) = outcome else {
            return Err(internal_stock_error());
        };
        Ok(snapshot(
            &request.selection_id,
            &request.analysis_id,
            settings,
            &revision,
        ))
    }

    pub fn adopt(
        &self,
        session: &mut DraftEstimateSession,
        settings: &ShopSettingsDraft,
        request: &AdoptStockCandidateEstimateRequest,
    ) -> Result<StockCandidateEstimateEvaluation, HostCommandError> {
        let mut guard = self.lock_runtime()?;
        let runtime = Self::require_runtime(&mut guard)?;
        let recorded_at = trusted_recorded_at()?;
        let decision = StockCandidateAdoptionDecision {
            actor: session.source_actor().clone(),
            recorded_at: recorded_at.clone(),
            reason: request.reason.clone(),
        };
        let workflow_request = DeveloperStockRequest::AdoptAndEvaluate {
            expected: expectation(&request.expected)?,
            quantities: DraftQuantityInputs {
                deliver: quantity(&request.deliver_quantity)?,
                planned_spares: quantity(&request.planned_spares)?,
                destructive_samples: quantity(&request.destructive_samples)?,
            },
            decision,
        };
        let outcome = runtime.execute(session, settings, recorded_at, workflow_request)?;
        let DeveloperStockOutcome::Estimate(state) = outcome else {
            return Err(internal_stock_error());
        };
        Ok(match *state {
            ValueState::Available { value } => {
                let candidate = snapshot(
                    &request.selection_id,
                    &request.analysis_id,
                    settings,
                    value.adoption().revision(),
                );
                let (baseline_state, baseline_rounded_selling_price) =
                    baseline_trace(value.baseline());
                let input_trace = crate::estimate::input_trace(value.adoption().inputs());
                let excluded_inputs = value
                    .adoption()
                    .excluded_inputs()
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect();
                let adoption_reason = value.adoption().decision().reason.clone();
                let result =
                    crate::estimate::result_summary(value.result().clone(), input_trace, None);
                StockCandidateEstimateEvaluation::Available {
                    estimate: Box::new(StockCandidateEstimateSummary {
                        selection_id: request.selection_id.clone(),
                        analysis_id: request.analysis_id.clone(),
                        candidate,
                        adoption_rule_id: DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_ID.to_owned(),
                        adoption_rule_version: rule_version_text(
                            DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_VERSION,
                        ),
                        adoption_reason,
                        excluded_inputs,
                        baseline_state,
                        baseline_rounded_selling_price,
                        result: Box::new(result),
                    }),
                }
            }
            ValueState::Unavailable { reason } => {
                StockCandidateEstimateEvaluation::Unavailable { reason }
            }
            ValueState::Blocked { reason }
            | ValueState::Unknown { reason }
            | ValueState::Stale { reason, .. } => {
                StockCandidateEstimateEvaluation::Blocked { reason }
            }
        })
    }

    fn lock_runtime(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Option<StockWorkflowRuntime>>, HostCommandError> {
        self.runtime.lock().map_err(|_| internal_stock_error())
    }

    fn require_runtime<'a>(
        guard: &'a mut std::sync::MutexGuard<'_, Option<StockWorkflowRuntime>>,
    ) -> Result<&'a mut StockWorkflowRuntime, HostCommandError> {
        guard.as_mut().ok_or_else(|| {
            HostCommandError::stock_workflow_unavailable("VIS4-STOCK-NOT-CONFIGURED")
        })
    }
}

impl StockWorkflowRuntime {
    /// Runs one attempt with a fresh correlation ID bound to the retained source subject.
    fn execute(
        &mut self,
        session: &mut DraftEstimateSession,
        settings: &ShopSettingsDraft,
        recorded_at: RecordedAt,
        request: DeveloperStockRequest,
    ) -> Result<DeveloperStockOutcome, HostCommandError> {
        let subject = self.fresh_subject(session, recorded_at)?;
        self.application
            .execute(session, settings, subject, request)
            .map_err(map_workflow_error)
    }

    fn fresh_subject(
        &mut self,
        session: &DraftEstimateSession,
        recorded_at: RecordedAt,
    ) -> Result<AssetReadSubject, HostCommandError> {
        let sequence = self.attempt_sequence;
        self.attempt_sequence = self.attempt_sequence.checked_add(1).unwrap_or(1);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| internal_stock_error())?
            .as_nanos();
        let correlation =
            AuditCorrelationId::new(format!("desktop-stock-attempt-{nanos}-{sequence}"))
                .map_err(|_| internal_stock_error())?;
        Ok(session.stock_workflow_subject(correlation, recorded_at))
    }

    fn next_candidate_id(&mut self) -> Result<RecordId, HostCommandError> {
        let value = self.candidate_sequence;
        self.candidate_sequence = self.candidate_sequence.checked_add(1).unwrap_or(1);
        RecordId::new(format!("stock-candidate-{value}")).map_err(|_| internal_stock_error())
    }
}

fn snapshot(
    selection_id: &str,
    analysis_id: &str,
    settings: &ShopSettingsDraft,
    revision: &DeveloperStockCandidateRevision,
) -> StockCandidateSnapshot {
    let candidate = revision.candidate();
    let original = candidate.original_proposal().blank_dimensions_mm();
    let previous = candidate.previous_dimensions_mm();
    let edited = candidate.edit().dimensions_mm();
    let allowance = candidate.total_allowance_mm();
    let mut reason_codes: Vec<String> = candidate
        .original_proposal()
        .reason_codes()
        .iter()
        .map(|reason| proposal_reason_code(*reason).to_owned())
        .collect();
    reason_codes.extend(
        candidate
            .reason_codes()
            .iter()
            .map(|reason| candidate_reason_code(*reason).to_owned()),
    );
    StockCandidateSnapshot {
        selection_id: selection_id.to_owned(),
        analysis_id: analysis_id.to_owned(),
        candidate_id: candidate.id().as_str().to_owned(),
        revision: candidate.revision(),
        previous_revision: candidate.previous_revision(),
        settings_revision: settings.revision().value(),
        original_blank_dimensions_mm: [
            decimal_text(original.x().value()),
            decimal_text(original.y().value()),
            decimal_text(original.z().value()),
        ],
        previous_blank_dimensions_mm: [
            decimal_text(previous.x().value()),
            decimal_text(previous.y().value()),
            decimal_text(previous.z().value()),
        ],
        blank_dimensions_mm: [
            decimal_text(edited.x().value()),
            decimal_text(edited.y().value()),
            decimal_text(edited.z().value()),
        ],
        total_allowance_mm: [
            decimal_text(allowance[0].value()),
            decimal_text(allowance[1].value()),
            decimal_text(allowance[2].value()),
        ],
        blank_volume_mm3: decimal_text(candidate.blank_volume_mm3().value()),
        removed_volume_mm3: decimal_text(candidate.removed_volume_mm3().value()),
        blank_mass_kg: decimal_text(candidate.blank_mass_kg().value()),
        unit_stock_material_cost: decimal_text(candidate.unit_stock_material_cost().amount()),
        currency: candidate
            .unit_stock_material_cost()
            .currency()
            .as_str()
            .to_owned(),
        cutting_hours_per_item: minutes_to_hours(candidate.coarse_times().cutting().value()),
        reviewed: revision.review().is_some(),
        session_rule_id: revision.session_rule_id().to_owned(),
        session_rule_version: rule_version_text(revision.session_rule_version()),
        candidate_rule_id: STOCK_CANDIDATE_RULE_ID.to_owned(),
        candidate_rule_version: rule_version_text(STOCK_CANDIDATE_RULE_VERSION),
        workflow_rule_id: DEVELOPER_STOCK_WORKFLOW_RULE_ID.to_owned(),
        workflow_rule_version: rule_version_text(DEVELOPER_STOCK_WORKFLOW_RULE_VERSION),
        reason_codes,
    }
}

fn baseline_trace(
    baseline: &ValueState<partprobe_application::DraftEstimateResult>,
) -> (DraftEstimateEvaluationState, Option<String>) {
    match baseline {
        ValueState::Available { value } => (
            DraftEstimateEvaluationState::Available,
            Some(decimal_text(value.pricing.rounded_price.rounded.amount())),
        ),
        ValueState::Unavailable { .. } | ValueState::Unknown { .. } => {
            (DraftEstimateEvaluationState::Unavailable, None)
        }
        ValueState::Blocked { .. } | ValueState::Stale { .. } => {
            (DraftEstimateEvaluationState::Blocked, None)
        }
    }
}

fn expectation(
    reference: &StockCandidateRevisionRef,
) -> Result<StockCandidateRevisionExpectation, HostCommandError> {
    Ok(StockCandidateRevisionExpectation {
        candidate_id: RecordId::new(reference.candidate_id.clone())
            .map_err(|_| HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-REFERENCE"))?,
        revision: reference.revision,
    })
}

fn parse_dimensions(
    values: &[String; 3],
) -> Result<AxisAlignedEnvelopeMillimeters, HostCommandError> {
    let mut lengths = values.iter().map(|value| {
        Decimal::from_str_exact(value.trim())
            .ok()
            .and_then(|decimal| CanonicalMillimeterLength::new(decimal).ok())
            .ok_or_else(|| HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-DIMENSION"))
    });
    let x = lengths.next().expect("three dimensions")?;
    let y = lengths.next().expect("three dimensions")?;
    let z = lengths.next().expect("three dimensions")?;
    Ok(AxisAlignedEnvelopeMillimeters::new(x, y, z))
}

fn quantity(value: &str) -> Result<ItemQuantity, HostCommandError> {
    value
        .trim()
        .parse::<u64>()
        .map(ItemQuantity::new)
        .map_err(|_| HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-QUANTITY"))
}

fn rule_version_text((major, minor, patch): (u16, u16, u16)) -> String {
    format!("{major}.{minor}.{patch}")
}

const fn candidate_reason_code(reason: StockCandidateReasonCode) -> &'static str {
    match reason {
        StockCandidateReasonCode::UserEditedBlankDimensions => "user_edited_blank_dimensions",
        StockCandidateReasonCode::StockAllowanceProfileOverridden => {
            "stock_allowance_profile_overridden"
        }
    }
}

fn map_workflow_error(error: DeveloperStockWorkflowError) -> HostCommandError {
    match error {
        DeveloperStockWorkflowError::SubjectMismatch => {
            HostCommandError::host_state_unavailable("VIS4-STOCK-SUBJECT")
        }
        DeveloperStockWorkflowError::AuditUnavailable => {
            HostCommandError::stock_workflow_unavailable("VIS4-STOCK-AUDIT-APPEND")
        }
        DeveloperStockWorkflowError::Denied(_) => {
            HostCommandError::stock_authorization_denied("VIS4-STOCK-DENIED")
        }
        DeveloperStockWorkflowError::Session(session) => map_session_error(session),
    }
}

fn map_session_error(error: DeveloperStockCandidateSessionError) -> HostCommandError {
    use DeveloperStockCandidateSessionError as SessionError;
    match error {
        SessionError::Unavailable => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-EVIDENCE-UNAVAILABLE")
        }
        SessionError::Blocked => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-EVIDENCE-BLOCKED")
        }
        SessionError::Unknown => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-EVIDENCE-UNKNOWN")
        }
        SessionError::StaleBasis => {
            HostCommandError::stale_stock_candidate("VIS4-STOCK-STALE-BASIS")
        }
        SessionError::AlreadyStarted => {
            HostCommandError::stale_stock_candidate("VIS4-STOCK-ALREADY-STARTED")
        }
        SessionError::StaleRevision => {
            HostCommandError::stale_stock_candidate("VIS4-STOCK-STALE-REVISION")
        }
        SessionError::StaleSettings => {
            HostCommandError::stale_stock_candidate("VIS4-STOCK-STALE-SETTINGS")
        }
        SessionError::HistoryLimit => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-HISTORY-LIMIT")
        }
        SessionError::AlreadyReviewed => {
            HostCommandError::stale_stock_candidate("VIS4-STOCK-ALREADY-REVIEWED")
        }
        SessionError::NotReviewed => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-NOT-REVIEWED")
        }
        SessionError::LimitationsNotAccepted => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-LIMITATIONS")
        }
        SessionError::InvalidReviewReason => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-REVIEW-REASON")
        }
        SessionError::InvalidAdoptionReason => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-ADOPTION-REASON")
        }
        SessionError::AdoptionArithmeticFailure => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-ADOPTION-ARITHMETIC")
        }
        SessionError::Candidate(_) => {
            HostCommandError::invalid_stock_candidate_input("VIS4-STOCK-CANDIDATE-RULE")
        }
    }
}

fn internal_stock_error() -> HostCommandError {
    HostCommandError::host_state_unavailable("VIS4-STOCK-INTERNAL")
}
