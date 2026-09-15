//! Bounded stock edits, review, and distinct session-only coarse adoption snapshots.

use partprobe_domain::{ActorId, RecordId, RecordedAt, ShopSettingsDraft, ValueState};
use partprobe_geometry_import::ControlledGeometryResult;
use partprobe_setup_planner::{
    ExactStepEnvelopeEvidence,
    stock_candidate::{DeveloperStockCandidate, StockCandidateEdit, StockCandidateError},
};

use crate::{
    DeveloperEstimateProposalApplication, DeveloperEstimateProposalReview, DraftEstimateInputs,
    DraftEstimateResult, DraftEstimateSession, DraftQuantityInputs, DraftRateContext,
};

/// Fail closed rather than silently discard immutable session evidence.
pub const MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS: usize = 32;
/// Identity of the additive headless session sequencing/review policy.
pub const DEVELOPER_STOCK_CANDIDATE_SESSION_RULE_ID: &str =
    "partprobe-developer-stock-candidate-session";
pub const DEVELOPER_STOCK_CANDIDATE_SESSION_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);
/// Separate identity: edited stock is never relabeled as original proposal adoption v1.
pub const DEVELOPER_STOCK_CANDIDATE_ADOPTION_RULE_ID: &str =
    "partprobe-developer-stock-candidate-adoption";
pub const DEVELOPER_STOCK_CANDIDATE_ADOPTION_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);

/// Explicit native-supplied adoption evidence, distinct from the earlier candidate review.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockCandidateAdoptionDecision {
    pub actor: ActorId,
    pub recorded_at: RecordedAt,
    pub reason: String,
}

/// Sealed, nonserializable input snapshot for one exact reviewed candidate and quantity set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptedDeveloperStockCandidateInputs {
    revision: DeveloperStockCandidateRevision,
    inputs: DraftEstimateInputs,
    decision: StockCandidateAdoptionDecision,
}

impl AdoptedDeveloperStockCandidateInputs {
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        DEVELOPER_STOCK_CANDIDATE_ADOPTION_RULE_ID
    }
    #[must_use]
    pub const fn rule_version(&self) -> (u16, u16, u16) {
        DEVELOPER_STOCK_CANDIDATE_ADOPTION_RULE_VERSION
    }
    #[must_use]
    pub const fn revision(&self) -> &DeveloperStockCandidateRevision {
        &self.revision
    }
    #[must_use]
    pub const fn inputs(&self) -> &DraftEstimateInputs {
        &self.inputs
    }
    #[must_use]
    pub const fn decision(&self) -> &StockCandidateAdoptionDecision {
        &self.decision
    }
    /// Explicit coarse omissions; these are not evidence of absent manufacturing cost.
    #[must_use]
    pub const fn excluded_inputs(&self) -> [&'static str; 6] {
        [
            "non_cutting_time",
            "in_cycle_inspection_time",
            "cut_certificate_and_freight",
            "tooling_fixture_and_outside_processing",
            "administration_and_overhead",
            "risk_and_rework",
        ]
    }
}

/// Distinct evaluated layer, with the original session's baseline state preserved as-is.
#[derive(Clone, Debug, PartialEq)]
pub struct DeveloperStockCandidateEstimate {
    adoption: AdoptedDeveloperStockCandidateInputs,
    baseline: ValueState<DraftEstimateResult>,
    result: DraftEstimateResult,
    rate_context: DraftRateContext,
}

impl DeveloperStockCandidateEstimate {
    #[must_use]
    pub const fn rate_context(&self) -> &DraftRateContext {
        &self.rate_context
    }
    #[must_use]
    pub const fn adoption(&self) -> &AdoptedDeveloperStockCandidateInputs {
        &self.adoption
    }
    #[must_use]
    pub const fn baseline(&self) -> &ValueState<DraftEstimateResult> {
        &self.baseline
    }
    #[must_use]
    pub const fn result(&self) -> &DraftEstimateResult {
        &self.result
    }
}

/// Optimistic expectation supplied for every successor and review.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockCandidateRevisionExpectation {
    pub candidate_id: RecordId,
    pub revision: u32,
}

/// Read-only candidate and separately recorded human review. Neither is adoption authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperStockCandidateRevision {
    candidate: DeveloperStockCandidate,
    review: Option<DeveloperEstimateProposalReview>,
}

impl DeveloperStockCandidateRevision {
    #[must_use]
    pub const fn session_rule_id(&self) -> &'static str {
        DEVELOPER_STOCK_CANDIDATE_SESSION_RULE_ID
    }

    #[must_use]
    pub const fn session_rule_version(&self) -> (u16, u16, u16) {
        DEVELOPER_STOCK_CANDIDATE_SESSION_RULE_VERSION
    }

    #[must_use]
    pub const fn candidate(&self) -> &DeveloperStockCandidate {
        &self.candidate
    }

    #[must_use]
    pub const fn review(&self) -> Option<&DeveloperEstimateProposalReview> {
        self.review.as_ref()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DeveloperStockCandidateHistory {
    revisions: Vec<DeveloperStockCandidateRevision>,
}

impl DraftEstimateSession {
    /// Maps one latest reviewed candidate through the unchanged coarse exclusion policy.
    /// Never replaces the session's inputs, rates, pricing, reviews, or prior results.
    pub fn adopt_stock_candidate(
        &self,
        expected: &StockCandidateRevisionExpectation,
        settings: &ShopSettingsDraft,
        quantities: DraftQuantityInputs,
        decision: StockCandidateAdoptionDecision,
    ) -> Result<AdoptedDeveloperStockCandidateInputs, DeveloperStockCandidateSessionError> {
        let revision = self.require_latest_stock_candidate(expected, settings)?;
        if revision.review.is_none() {
            return Err(DeveloperStockCandidateSessionError::NotReviewed);
        }
        if decision.reason.trim().is_empty()
            || decision.reason.len() > 1_024
            || decision.reason.contains('\0')
        {
            return Err(DeveloperStockCandidateSessionError::InvalidAdoptionReason);
        }
        let candidate = &revision.candidate;
        let inputs = crate::stock_proposal::coarse_developer_inputs(
            candidate.blank_volume_mm3(),
            candidate.original_proposal().material_density_kg_per_m3(),
            candidate.unit_stock_material_cost(),
            candidate.coarse_times(),
            quantities,
        )
        .map_err(|_| DeveloperStockCandidateSessionError::AdoptionArithmeticFailure)?;
        Ok(AdoptedDeveloperStockCandidateInputs {
            revision: revision.clone(),
            inputs,
            decision,
        })
    }

    /// Evaluates the sealed adoption as a separate coarse layer with current pinned context.
    /// All ordinary geometry/rate/pricing gates remain enforced by `evaluate`.
    #[must_use]
    pub fn evaluate_stock_candidate(
        &self,
        settings: &ShopSettingsDraft,
        adoption: &AdoptedDeveloperStockCandidateInputs,
    ) -> ValueState<DeveloperStockCandidateEstimate> {
        let candidate = &adoption.revision.candidate;
        let expected = StockCandidateRevisionExpectation {
            candidate_id: candidate.id().clone(),
            revision: candidate.revision(),
        };
        let current = match self.require_latest_stock_candidate(&expected, settings) {
            Ok(current) => current,
            Err(_) => return ValueState::Blocked {
                reason:
                    "stock candidate adoption does not match the latest retained revision/settings"
                        .to_owned(),
            },
        };
        if current != &adoption.revision {
            return ValueState::Blocked {
                reason:
                    "stock candidate adoption does not match retained candidate/review evidence"
                        .to_owned(),
            };
        }
        let mut candidate_session = self.clone();
        candidate_session.set_inputs(adoption.inputs.clone());
        let snapshot = |result| DeveloperStockCandidateEstimate {
            adoption: adoption.clone(),
            baseline: self.evaluate(),
            result,
            rate_context: self
                .pinned_rate_context()
                .expect("evaluation requires pinned rates")
                .clone(),
        };
        match candidate_session.evaluate() {
            ValueState::Available { value } => ValueState::available(snapshot(value)),
            ValueState::Unavailable { reason } => ValueState::Unavailable { reason },
            ValueState::Blocked { reason } => ValueState::Blocked { reason },
            ValueState::Unknown { reason } => ValueState::Unknown { reason },
            ValueState::Stale {
                last_known,
                as_of,
                reason,
            } => ValueState::Stale {
                last_known: snapshot(last_known),
                as_of,
                reason,
            },
        }
    }

    /// Retains all bounded revisions, including reviews of predecessors, without modifying them.
    #[must_use]
    pub fn stock_candidate_revisions(&self) -> &[DeveloperStockCandidateRevision] {
        &self.stock_candidates.revisions
    }

    /// Creates the first candidate from this session's controlled analysis and current settings.
    ///
    /// The host supplies identity/time/reason evidence. This headless developer seam does not
    /// establish authorization, durable audit, estimate adoption, or desktop permission.
    pub fn create_stock_candidate(
        &mut self,
        candidate_id: RecordId,
        settings: &ShopSettingsDraft,
        edit: StockCandidateEdit,
    ) -> Result<&DeveloperStockCandidateRevision, DeveloperStockCandidateSessionError> {
        if !self.stock_candidates.revisions.is_empty() {
            return Err(DeveloperStockCandidateSessionError::AlreadyStarted);
        }
        let envelope = self.stock_candidate_envelope()?;
        let proposal = match DeveloperEstimateProposalApplication.propose(self.geometry(), settings)
        {
            ValueState::Available { value } => value,
            ValueState::Unavailable { .. } => {
                return Err(DeveloperStockCandidateSessionError::Unavailable);
            }
            ValueState::Blocked { .. } => return Err(DeveloperStockCandidateSessionError::Blocked),
            ValueState::Unknown { .. } => return Err(DeveloperStockCandidateSessionError::Unknown),
            ValueState::Stale { .. } => {
                return Err(DeveloperStockCandidateSessionError::StaleBasis);
            }
        };
        let candidate =
            DeveloperStockCandidate::new(candidate_id, &envelope, settings, &proposal, edit)?;
        self.stock_candidates
            .revisions
            .push(DeveloperStockCandidateRevision {
                candidate,
                review: None,
            });
        Ok(self
            .stock_candidates
            .revisions
            .last()
            .expect("one revision was just appended"))
    }

    /// Appends exactly one successor to the latest revision. Failures change nothing.
    pub fn revise_stock_candidate(
        &mut self,
        expected: &StockCandidateRevisionExpectation,
        settings: &ShopSettingsDraft,
        edit: StockCandidateEdit,
    ) -> Result<&DeveloperStockCandidateRevision, DeveloperStockCandidateSessionError> {
        let latest = self.require_latest_stock_candidate(expected, settings)?;
        if self.stock_candidates.revisions.len() >= MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS {
            return Err(DeveloperStockCandidateSessionError::HistoryLimit);
        }
        let candidate =
            latest
                .candidate
                .revise(&self.stock_candidate_envelope()?, settings, edit)?;
        self.stock_candidates
            .revisions
            .push(DeveloperStockCandidateRevision {
                candidate,
                review: None,
            });
        Ok(self
            .stock_candidates
            .revisions
            .last()
            .expect("one revision was just appended"))
    }

    /// Records affirmative review against one exact latest candidate/settings basis.
    ///
    /// Review is append-once per revision; edits do not inherit it. Candidate confidence remains
    /// `NeedsReview`, and this operation does not replace inputs or calculate an estimate.
    pub fn review_stock_candidate(
        &mut self,
        expected: &StockCandidateRevisionExpectation,
        settings: &ShopSettingsDraft,
        review: DeveloperEstimateProposalReview,
    ) -> Result<&DeveloperStockCandidateRevision, DeveloperStockCandidateSessionError> {
        let latest = self.require_latest_stock_candidate(expected, settings)?;
        if latest.review.is_some() {
            return Err(DeveloperStockCandidateSessionError::AlreadyReviewed);
        }
        if !review.proposal_values_reviewed {
            return Err(DeveloperStockCandidateSessionError::NotReviewed);
        }
        if !review.coarse_limitations_accepted {
            return Err(DeveloperStockCandidateSessionError::LimitationsNotAccepted);
        }
        if review.reason.trim().is_empty()
            || review.reason.len() > 1_024
            || review.reason.contains('\0')
        {
            return Err(DeveloperStockCandidateSessionError::InvalidReviewReason);
        }
        let latest = self
            .stock_candidates
            .revisions
            .last_mut()
            .expect("latest revision was checked");
        latest.review = Some(review);
        Ok(latest)
    }

    fn require_latest_stock_candidate(
        &self,
        expected: &StockCandidateRevisionExpectation,
        settings: &ShopSettingsDraft,
    ) -> Result<&DeveloperStockCandidateRevision, DeveloperStockCandidateSessionError> {
        let latest = self
            .stock_candidates
            .revisions
            .last()
            .ok_or(DeveloperStockCandidateSessionError::Unavailable)?;
        if latest.candidate.id() != &expected.candidate_id
            || latest.candidate.revision() != expected.revision
        {
            return Err(DeveloperStockCandidateSessionError::StaleRevision);
        }
        if latest.candidate.settings_basis() != settings {
            return Err(DeveloperStockCandidateSessionError::StaleSettings);
        }
        Ok(latest)
    }

    fn stock_candidate_envelope(
        &self,
    ) -> Result<ExactStepEnvelopeEvidence, DeveloperStockCandidateSessionError> {
        let ControlledGeometryResult::ExactBrepWithEnvelope(analysis) = &self.geometry().result
        else {
            return Err(DeveloperStockCandidateSessionError::Unavailable);
        };
        ExactStepEnvelopeEvidence::from_controlled_analysis(
            analysis,
            self.geometry().output_hash.clone(),
        )
        .map_err(|_| DeveloperStockCandidateSessionError::Blocked)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeveloperStockCandidateSessionError {
    Unavailable,
    Blocked,
    Unknown,
    StaleBasis,
    AlreadyStarted,
    StaleRevision,
    StaleSettings,
    HistoryLimit,
    AlreadyReviewed,
    NotReviewed,
    LimitationsNotAccepted,
    InvalidReviewReason,
    InvalidAdoptionReason,
    AdoptionArithmeticFailure,
    Candidate(StockCandidateError),
}

impl From<StockCandidateError> for DeveloperStockCandidateSessionError {
    fn from(error: StockCandidateError) -> Self {
        Self::Candidate(error)
    }
}
