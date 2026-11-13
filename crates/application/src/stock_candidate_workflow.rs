//! Fail-closed policy/audit sequencing for headless developer stock operations.

use partprobe_domain::{
    ActorId, DataClassificationId, ProjectId, RecordId, RecordStateId, RecordVersionId, RecordedAt,
    ShopProfileId, ShopSettingsDraft, ShopSettingsRevision, ValueState,
};
use partprobe_geometry_core::Sha256Digest;
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationDecision, AuthorizationOutcome,
    AuthorizationReasonCode, SecurityPolicyRef,
};
use partprobe_setup_planner::stock_candidate::StockCandidateEdit;

use crate::{
    AssetReadSubject, DeveloperEstimateProposalReview, DeveloperStockCandidateEstimate,
    DeveloperStockCandidateRevision, DeveloperStockCandidateSessionError, DraftEstimateSession,
    DraftQuantityInputs, StockCandidateAdoptionDecision, StockCandidateRevisionExpectation,
};

pub const DEVELOPER_STOCK_WORKFLOW_RULE_ID: &str = "partprobe-developer-stock-workflow";
pub const DEVELOPER_STOCK_WORKFLOW_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeveloperStockOperation {
    Create,
    Revise,
    Review,
    AdoptAndEvaluate,
}

/// Native-only typed intent. It is not a desktop DTO or an identity/permission source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeveloperStockRequest {
    Create {
        candidate_id: RecordId,
        edit: StockCandidateEdit,
    },
    Revise {
        expected: StockCandidateRevisionExpectation,
        edit: StockCandidateEdit,
    },
    Review {
        expected: StockCandidateRevisionExpectation,
        review: DeveloperEstimateProposalReview,
    },
    AdoptAndEvaluate {
        expected: StockCandidateRevisionExpectation,
        quantities: DraftQuantityInputs,
        decision: StockCandidateAdoptionDecision,
    },
}

/// Exact record/version references only; no paths, dimensions, prices, rates, or free text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperStockAuthorizationContext {
    subject: AssetReadSubject,
    operation: DeveloperStockOperation,
    profile_id: ShopProfileId,
    settings_revision: ShopSettingsRevision,
    candidate_id: RecordId,
    expected_revision: Option<u32>,
    resulting_revision: u32,
    source_hash: Sha256Digest,
    analysis_output_hash: Sha256Digest,
}

impl DeveloperStockAuthorizationContext {
    #[must_use]
    pub const fn actor_id(&self) -> &ActorId {
        &self.subject.actor_id
    }
    #[must_use]
    pub const fn project_id(&self) -> &ProjectId {
        &self.subject.project_id
    }
    #[must_use]
    pub const fn record_id(&self) -> &RecordId {
        &self.subject.record_id
    }
    #[must_use]
    pub const fn record_version_id(&self) -> &RecordVersionId {
        &self.subject.record_version_id
    }
    #[must_use]
    pub const fn classification_id(&self) -> &DataClassificationId {
        &self.subject.classification_id
    }
    #[must_use]
    pub const fn record_state_id(&self) -> &RecordStateId {
        &self.subject.record_state_id
    }
    #[must_use]
    pub const fn correlation_id(&self) -> &AuditCorrelationId {
        &self.subject.correlation_id
    }
    #[must_use]
    pub const fn recorded_at(&self) -> &RecordedAt {
        &self.subject.recorded_at
    }
    #[must_use]
    pub const fn operation(&self) -> DeveloperStockOperation {
        self.operation
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
    pub const fn candidate_id(&self) -> &RecordId {
        &self.candidate_id
    }
    #[must_use]
    pub const fn expected_revision(&self) -> Option<u32> {
        self.expected_revision
    }
    #[must_use]
    pub const fn resulting_revision(&self) -> u32 {
        self.resulting_revision
    }
    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }
    #[must_use]
    pub const fn analysis_output_hash(&self) -> &Sha256Digest {
        &self.analysis_output_hash
    }
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        DEVELOPER_STOCK_WORKFLOW_RULE_ID
    }
    #[must_use]
    pub const fn rule_version(&self) -> (u16, u16, u16) {
        DEVELOPER_STOCK_WORKFLOW_RULE_VERSION
    }
}

pub trait DeveloperStockAuthorizationPolicy {
    fn evaluate(&self, context: &DeveloperStockAuthorizationContext) -> AuthorizationDecision;
}

#[derive(Clone, Debug)]
pub struct DenyAllDeveloperStockAuthorizationPolicy {
    policy: SecurityPolicyRef,
    reason: AuthorizationReasonCode,
}

impl DenyAllDeveloperStockAuthorizationPolicy {
    #[must_use]
    pub const fn new(policy: SecurityPolicyRef, reason: AuthorizationReasonCode) -> Self {
        Self { policy, reason }
    }
}

impl DeveloperStockAuthorizationPolicy for DenyAllDeveloperStockAuthorizationPolicy {
    fn evaluate(&self, _: &DeveloperStockAuthorizationContext) -> AuthorizationDecision {
        AuthorizationDecision::deny(self.policy.clone(), self.reason.clone())
    }
}

/// A decision, not proof that mutation or calculation subsequently succeeded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperStockAuthorizationEvent {
    context: DeveloperStockAuthorizationContext,
    decision: AuthorizationDecision,
}

impl DeveloperStockAuthorizationEvent {
    #[must_use]
    pub const fn context(&self) -> &DeveloperStockAuthorizationContext {
        &self.context
    }
    #[must_use]
    pub const fn decision(&self) -> &AuthorizationDecision {
        &self.decision
    }
}

/// Append-preserving adapter seam. Must reject every reused correlation ID, even an exact retry;
/// decision references intentionally do not contain the full private operation payload.
/// Hosts must reconcile an uncertain attempt and supply a fresh ID, never replay an allow event.
pub trait DeveloperStockAuthorizationAuditSink {
    fn append(&self, event: DeveloperStockAuthorizationEvent) -> Result<(), AuditAppendError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeveloperStockOutcome {
    Revision(Box<DeveloperStockCandidateRevision>),
    Estimate(Box<ValueState<DeveloperStockCandidateEstimate>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeveloperStockWorkflowError {
    SubjectMismatch,
    Session(DeveloperStockCandidateSessionError),
    AuditUnavailable,
    Denied(AuthorizationReasonCode),
}

/// Future desktop entry point; does not configure authentication or provide a durable sink.
#[derive(Debug)]
pub struct DeveloperStockApplication<P, A> {
    policy: P,
    audit: A,
}

impl<P, A> DeveloperStockApplication<P, A>
where
    P: DeveloperStockAuthorizationPolicy,
    A: DeveloperStockAuthorizationAuditSink,
{
    #[must_use]
    pub const fn new(policy: P, audit: A) -> Self {
        Self { policy, audit }
    }
    #[must_use]
    pub const fn audit(&self) -> &A {
        &self.audit
    }

    pub fn execute(
        &self,
        session: &mut DraftEstimateSession,
        settings: &ShopSettingsDraft,
        subject: AssetReadSubject,
        request: DeveloperStockRequest,
    ) -> Result<DeveloperStockOutcome, DeveloperStockWorkflowError> {
        let source = session.source_subject();
        if subject.project_id != source.project_id
            || subject.record_id != source.record_id
            || subject.record_version_id != source.record_version_id
            || subject.classification_id != source.classification_id
            || subject.record_state_id != source.record_state_id
        {
            return Err(DeveloperStockWorkflowError::SubjectMismatch);
        }
        let (actor, time) = match &request {
            DeveloperStockRequest::Create { edit, .. }
            | DeveloperStockRequest::Revise { edit, .. } => (edit.actor(), edit.recorded_at()),
            DeveloperStockRequest::Review { review, .. } => (&review.actor, &review.recorded_at),
            DeveloperStockRequest::AdoptAndEvaluate { decision, .. } => {
                (&decision.actor, &decision.recorded_at)
            }
        };
        if actor != &subject.actor_id || time != &subject.recorded_at {
            return Err(DeveloperStockWorkflowError::SubjectMismatch);
        }

        // Validate/stage privately before policy. No candidate or authoritative result escapes.
        let mut staged = session.clone();
        let (operation, expected_revision, adoption) = match request {
            DeveloperStockRequest::Create { candidate_id, edit } => {
                staged
                    .create_stock_candidate(candidate_id, settings, edit)
                    .map_err(DeveloperStockWorkflowError::Session)?;
                (DeveloperStockOperation::Create, None, None)
            }
            DeveloperStockRequest::Revise { expected, edit } => {
                staged
                    .revise_stock_candidate(&expected, settings, edit)
                    .map_err(DeveloperStockWorkflowError::Session)?;
                (
                    DeveloperStockOperation::Revise,
                    Some(expected.revision),
                    None,
                )
            }
            DeveloperStockRequest::Review { expected, review } => {
                staged
                    .review_stock_candidate(&expected, settings, review)
                    .map_err(DeveloperStockWorkflowError::Session)?;
                (
                    DeveloperStockOperation::Review,
                    Some(expected.revision),
                    None,
                )
            }
            DeveloperStockRequest::AdoptAndEvaluate {
                expected,
                quantities,
                decision,
            } => {
                let adoption = staged
                    .adopt_stock_candidate(&expected, settings, quantities, decision)
                    .map_err(DeveloperStockWorkflowError::Session)?;
                (
                    DeveloperStockOperation::AdoptAndEvaluate,
                    Some(expected.revision),
                    Some(adoption),
                )
            }
        };
        let revision = staged
            .stock_candidate_revisions()
            .last()
            .expect("validated operation retains a candidate");
        let candidate = revision.candidate();
        let context = DeveloperStockAuthorizationContext {
            subject,
            operation,
            profile_id: settings.profile_id().clone(),
            settings_revision: settings.revision(),
            candidate_id: candidate.id().clone(),
            expected_revision,
            resulting_revision: candidate.revision(),
            source_hash: candidate.original_proposal().source_hash().clone(),
            analysis_output_hash: candidate.original_proposal().analysis_output_hash().clone(),
        };
        let decision = self.policy.evaluate(&context);
        self.audit
            .append(DeveloperStockAuthorizationEvent {
                context,
                decision: decision.clone(),
            })
            .map_err(|_| DeveloperStockWorkflowError::AuditUnavailable)?;
        if decision.outcome() == AuthorizationOutcome::Denied {
            return Err(DeveloperStockWorkflowError::Denied(
                decision.reason_code().clone(),
            ));
        }
        if let Some(adoption) = adoption {
            Ok(DeveloperStockOutcome::Estimate(Box::new(
                staged.evaluate_stock_candidate(settings, &adoption),
            )))
        } else {
            let result = revision.clone();
            *session = staged;
            Ok(DeveloperStockOutcome::Revision(Box::new(result)))
        }
    }
}
