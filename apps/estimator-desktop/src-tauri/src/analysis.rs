use std::path::Path;
#[cfg(feature = "desktop-host")]
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "desktop-host", test))]
use partprobe_application::LocalAssetReadService;
use partprobe_application::{
    AssetReadSubject, DraftEstimateApplication, DraftEstimateApplicationError,
    DraftEstimateSession, DraftGeometryRequestTemplate, GeometryAnalysisPort,
};
use partprobe_desktop_contract::{
    AnalysisStatus, CanonicalLengthUnit, EstimateAvailability, EstimateReadiness,
    GeometryEvidenceState, GeometryRepresentation, GeometryStageSummary, HostCommandError,
    ModelAnalysisResult, PersistenceAvailability, ProvisionalGeometryFacts,
};
use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId,
    RecordVersionId, RecordedAt, RuleVersion, SchemaVersion, ValueState,
};
use partprobe_geometry_core::{AnalysisProfile, AnalysisProfileId, GeometryStage, StageStatus};
use partprobe_geometry_import::{
    AssetCapability, CorrelationId, GeometryJobId, LocalAssetRoot, ResourceQuotas,
};
#[cfg(feature = "desktop-host")]
use partprobe_geometry_import::{GeometryWorkerSupervisor, SupervisorPolicy};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationAuditEvent, AuthorizationAuditSink,
    AuthorizationContext, AuthorizationDecision, AuthorizationPolicy, AuthorizationReasonCode,
    ProtectedOperation, SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion,
};

const DEVELOPER_ACTOR_ID: &str = "local-developer-session";
const DEVELOPER_PROJECT_ID: &str = "gui-4-developer-slice";
const DEVELOPER_CLASSIFICATION_ID: &str = "local-test-data";
const DEVELOPER_RECORD_STATE_ID: &str = "ephemeral-draft";
const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 1024 * 1024;
const MAX_ENTITIES: u64 = 2_000_000;
const WALL_TIME_MILLIS: u64 = 30_000;

#[cfg(feature = "desktop-host")]
#[derive(Debug)]
pub struct DesktopAnalysisConfiguration {
    worker_executable: PathBuf,
    worker_workspace: PathBuf,
    native_library_directory: PathBuf,
}

#[cfg(feature = "desktop-host")]
impl DesktopAnalysisConfiguration {
    pub fn from_environment() -> Result<Self, HostCommandError> {
        let worker_executable = required_path("PARTPROBE_GEOMETRY_WORKER")?;
        let worker_workspace = required_path("PARTPROBE_GEOMETRY_WORKSPACE")?;
        let occt_root = required_path("PARTPROBE_OCCT_ROOT")?;
        let native_library_directory = occt_root.join("lib");
        Self::new(
            worker_executable,
            worker_workspace,
            native_library_directory,
        )
    }

    pub fn new(
        worker_executable: PathBuf,
        worker_workspace: PathBuf,
        native_library_directory: PathBuf,
    ) -> Result<Self, HostCommandError> {
        if !worker_executable.is_file()
            || !worker_workspace.is_dir()
            || !native_library_directory.is_dir()
        {
            return Err(HostCommandError::analysis_unavailable(
                "GUI4-ANALYSIS-CONFIG-PATHS",
            ));
        }
        Ok(Self {
            worker_executable,
            worker_workspace,
            native_library_directory,
        })
    }

    pub fn build_adapter(
        self,
    ) -> Result<DesktopAnalysisAdapter<GeometryWorkerSupervisor>, HostCommandError> {
        let policy = SupervisorPolicy::new(
            1024 * 1024,
            10,
            250,
            2 * 1024 * 1024 * 1024,
            WALL_TIME_MILLIS,
        )
        .map_err(|_| HostCommandError::analysis_unavailable("GUI4-ANALYSIS-POLICY"))?;
        let supervisor =
            GeometryWorkerSupervisor::new(self.worker_executable, self.worker_workspace, policy)
                .and_then(|supervisor| {
                    supervisor.with_native_library_directory(self.native_library_directory)
                })
                .map_err(|_| HostCommandError::analysis_unavailable("GUI4-ANALYSIS-SUPERVISOR"))?;
        Ok(DesktopAnalysisAdapter::new(supervisor))
    }
}

#[cfg(feature = "desktop-host")]
fn required_path(name: &str) -> Result<PathBuf, HostCommandError> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| HostCommandError::analysis_unavailable("GUI4-ANALYSIS-CONFIG-MISSING"))
}

#[derive(Debug)]
pub struct DesktopAnalysisAdapter<G> {
    application: DraftEstimateApplication<
        DeveloperSessionAuthorizationPolicy,
        InMemoryAuthorizationAudit,
        G,
    >,
}

impl<G> DesktopAnalysisAdapter<G>
where
    G: GeometryAnalysisPort,
{
    #[cfg(any(feature = "desktop-host", test))]
    pub fn new(geometry_analysis: G) -> Self {
        Self {
            application: DraftEstimateApplication::new(
                LocalAssetReadService::new(
                    DeveloperSessionAuthorizationPolicy,
                    InMemoryAuthorizationAudit::default(),
                ),
                geometry_analysis,
            ),
        }
    }

    pub fn analyze(
        &self,
        selection_id: &str,
        source_path: &Path,
        analysis_number: u64,
    ) -> Result<(DraftEstimateSession, ModelAnalysisResult), HostCommandError> {
        if !source_path.is_absolute() {
            return Err(HostCommandError::invalid_selection(
                "GUI4-ANALYSIS-SOURCE-NONABSOLUTE",
            ));
        }
        let parent = source_path
            .parent()
            .ok_or_else(|| HostCommandError::invalid_selection("GUI4-ANALYSIS-SOURCE-PARENT"))?;
        let relative_path = source_path
            .file_name()
            .map(Path::new)
            .ok_or_else(|| HostCommandError::invalid_selection("GUI4-ANALYSIS-SOURCE-NAME"))?;
        let ids = AnalysisIdentifiers::new(analysis_number)?;
        let root = LocalAssetRoot::open(ids.asset_root_id.clone(), parent)
            .map_err(|_| HostCommandError::invalid_selection("GUI4-ANALYSIS-SOURCE-ROOT"))?;
        let template = request_template(&ids)?;
        let subject = asset_subject(&ids)?;
        let session = self
            .application
            .start_session_from_unfingerprinted_source(
                subject,
                &root,
                &template,
                relative_path,
                &AtomicBool::new(false),
            )
            .map_err(map_application_error)?;
        let result = analysis_result(selection_id, &ids.analysis_id, &session);
        Ok((session, result))
    }

    #[cfg(test)]
    pub fn audit_event_count(&self) -> usize {
        self.application.asset_reads().audit().event_count()
    }
}

#[derive(Debug)]
struct AnalysisIdentifiers {
    analysis_id: String,
    asset_root_id: AssetRootId,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    audit_correlation_id: AuditCorrelationId,
    asset_capability: AssetCapability,
    record_id: RecordId,
    record_version_id: RecordVersionId,
}

impl AnalysisIdentifiers {
    fn new(number: u64) -> Result<Self, HostCommandError> {
        let analysis_id = format!("analysis-{number}");
        Ok(Self {
            asset_root_id: AssetRootId::new(format!("desktop-root-{number}"))
                .map_err(|_| internal_contract_error())?,
            job_id: GeometryJobId::new(format!("desktop-job-{number}"))
                .map_err(|_| internal_contract_error())?,
            correlation_id: CorrelationId::new(format!("desktop-correlation-{number}"))
                .map_err(|_| internal_contract_error())?,
            audit_correlation_id: AuditCorrelationId::new(format!(
                "desktop-audit-correlation-{number}"
            ))
            .map_err(|_| internal_contract_error())?,
            asset_capability: AssetCapability::new(format!("desktop-capability-{number}"))
                .map_err(|_| internal_contract_error())?,
            record_id: RecordId::new(format!("desktop-model-{number}"))
                .map_err(|_| internal_contract_error())?,
            record_version_id: RecordVersionId::new(format!("desktop-revision-{number}"))
                .map_err(|_| internal_contract_error())?,
            analysis_id,
        })
    }
}

fn request_template(
    ids: &AnalysisIdentifiers,
) -> Result<DraftGeometryRequestTemplate, HostCommandError> {
    DraftGeometryRequestTemplate::new(
        SchemaVersion::new(1).map_err(|_| internal_contract_error())?,
        ids.job_id.clone(),
        ids.correlation_id.clone(),
        ids.asset_capability.clone(),
        vec![
            GeometryStage::Intake,
            GeometryStage::Identify,
            GeometryStage::Parse,
            GeometryStage::UnitResolution,
            GeometryStage::Validation,
            GeometryStage::BasicProperties,
        ],
        AnalysisProfile {
            id: AnalysisProfileId::new("occt-step-spike").map_err(|_| internal_contract_error())?,
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(
            MAX_INPUT_BYTES,
            MAX_OUTPUT_BYTES,
            MAX_ENTITIES,
            WALL_TIME_MILLIS,
        )
        .map_err(|_| internal_contract_error())?,
    )
    .map_err(|_| internal_contract_error())
}

fn asset_subject(ids: &AnalysisIdentifiers) -> Result<AssetReadSubject, HostCommandError> {
    Ok(AssetReadSubject::new(
        ActorId::new(DEVELOPER_ACTOR_ID).map_err(|_| internal_contract_error())?,
        ProjectId::new(DEVELOPER_PROJECT_ID).map_err(|_| internal_contract_error())?,
        ids.record_id.clone(),
        ids.record_version_id.clone(),
        DataClassificationId::new(DEVELOPER_CLASSIFICATION_ID)
            .map_err(|_| internal_contract_error())?,
        RecordStateId::new(DEVELOPER_RECORD_STATE_ID).map_err(|_| internal_contract_error())?,
        ids.audit_correlation_id.clone(),
        trusted_recorded_at()?,
    ))
}

fn trusted_recorded_at() -> Result<RecordedAt, HostCommandError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| internal_contract_error())?
        .as_secs();
    RecordedAt::new(format!("unix-seconds:{seconds}")).map_err(|_| internal_contract_error())
}

fn map_application_error(error: DraftEstimateApplicationError) -> HostCommandError {
    match error {
        DraftEstimateApplicationError::AssetRead(_) => {
            HostCommandError::analysis_failed("GUI4-ANALYSIS-ASSET-READ")
        }
        DraftEstimateApplicationError::SourceFingerprint => {
            HostCommandError::analysis_failed("GUI4-ANALYSIS-FINGERPRINT")
        }
        DraftEstimateApplicationError::GeometryAnalysis(_) => {
            HostCommandError::analysis_failed("GUI4-ANALYSIS-WORKER")
        }
    }
}

fn internal_contract_error() -> HostCommandError {
    HostCommandError::analysis_unavailable("GUI4-ANALYSIS-INTERNAL-CONTRACT")
}

fn analysis_result(
    selection_id: &str,
    analysis_id: &str,
    session: &DraftEstimateSession,
) -> ModelAnalysisResult {
    let evidence = session.geometry();
    let snapshot = &evidence.snapshot;
    let estimate_reason = match session.evaluate() {
        ValueState::Unavailable { reason }
        | ValueState::Blocked { reason }
        | ValueState::Unknown { reason } => reason,
        ValueState::Stale { reason, .. } => reason,
        ValueState::Available { .. } => {
            "estimate state requires explicit review before display".to_owned()
        }
    };
    ModelAnalysisResult {
        selection_id: selection_id.to_owned(),
        analysis_id: analysis_id.to_owned(),
        analysis_status: AnalysisStatus::ProvisionalAvailable,
        evidence_state: GeometryEvidenceState::ProvisionalSpike,
        persistence: PersistenceAvailability::SessionOnly,
        geometry: ProvisionalGeometryFacts {
            canonical_units: CanonicalLengthUnit::Millimeter,
            representation: GeometryRepresentation::ExactBrep,
            surface_area_mm2: snapshot.surface_area_mm2().to_owned(),
            enclosed_volume_mm3: snapshot.enclosed_volume_mm3().to_owned(),
            center_of_mass_mm: snapshot.center_of_mass_mm().map(str::to_owned),
            solid_body_count: snapshot.solid_body_count(),
            transferred_roots: snapshot.transferred_roots(),
            source_hash_sha256: snapshot.source_hash().as_str().to_owned(),
            output_hash_sha256: evidence.output_hash.as_str().to_owned(),
            output_byte_length: evidence.output_byte_length,
            geometry_engine: format!("OCCT {}", snapshot.occt_version()),
            adapter_abi_version: snapshot.adapter_abi_version(),
        },
        stages: evidence
            .stage_reports
            .iter()
            .map(|report| GeometryStageSummary {
                stage: stage_name(report.stage()).to_owned(),
                status: stage_status_name(report.status()).to_owned(),
                warning_codes: report
                    .warnings()
                    .iter()
                    .map(|warning| warning.code.as_str().to_owned())
                    .collect(),
            })
            .collect(),
        estimate: EstimateReadiness {
            state: EstimateAvailability::Unavailable,
            reason: estimate_reason,
        },
    }
}

fn stage_name(stage: GeometryStage) -> &'static str {
    match stage {
        GeometryStage::Intake => "intake",
        GeometryStage::Identify => "identify",
        GeometryStage::Parse => "parse",
        GeometryStage::UnitResolution => "unit_resolution",
        GeometryStage::Healing => "healing",
        GeometryStage::Validation => "validation",
        GeometryStage::BasicProperties => "basic_properties",
        GeometryStage::Tessellation => "tessellation",
    }
}

fn stage_status_name(status: StageStatus) -> &'static str {
    match status {
        StageStatus::Succeeded => "succeeded",
        StageStatus::SucceededWithWarnings => "succeeded_with_warnings",
        StageStatus::NeedsUserInput => "needs_user_input",
        StageStatus::FailedRecoverable => "failed_recoverable",
        StageStatus::FailedTerminal => "failed_terminal",
    }
}

#[derive(Clone, Debug)]
struct DeveloperSessionAuthorizationPolicy;

impl AuthorizationPolicy for DeveloperSessionAuthorizationPolicy {
    fn evaluate(&self, context: &AuthorizationContext) -> AuthorizationDecision {
        let policy = SecurityPolicyRef::new(
            SecurityPolicyId::new("gui-4-developer-session")
                .expect("static policy ID must be valid"),
            SecurityPolicyVersion::new(1).expect("static policy version must be valid"),
        );
        let allowed = context.actor_id().as_str() == DEVELOPER_ACTOR_ID
            && context.project_id().as_str() == DEVELOPER_PROJECT_ID
            && context.classification_id().as_str() == DEVELOPER_CLASSIFICATION_ID
            && context.record_state_id().as_str() == DEVELOPER_RECORD_STATE_ID
            && context.operation() == ProtectedOperation::ReadGeometryAsset;
        let reason = if allowed {
            "LOCAL_DEVELOPER_SESSION"
        } else {
            "DEVELOPER_SESSION_SCOPE_MISMATCH"
        };
        let reason = AuthorizationReasonCode::new(reason)
            .expect("static authorization reason must be valid");
        if allowed {
            AuthorizationDecision::allow(policy, reason)
        } else {
            AuthorizationDecision::deny(policy, reason)
        }
    }
}

#[derive(Clone, Debug, Default)]
struct InMemoryAuthorizationAudit {
    events: Arc<Mutex<Vec<AuthorizationAuditEvent>>>,
}

impl InMemoryAuthorizationAudit {
    #[cfg(test)]
    fn event_count(&self) -> usize {
        self.events.lock().map_or(0, |events| events.len())
    }
}

impl AuthorizationAuditSink for InMemoryAuthorizationAudit {
    fn append(&self, event: AuthorizationAuditEvent) -> Result<(), AuditAppendError> {
        self.events
            .lock()
            .map_err(|_| AuditAppendError)?
            .push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use partprobe_application::{
        AnalyzedGeometryEvidence, GeometryAnalysisFailure, GeometryAnalysisPort,
    };
    use partprobe_geometry_core::{GeometryStageReport, ProvisionalGeometryDecimal, StageStatus};
    use partprobe_geometry_import::{
        AssetReadGrant, GeometryWorkerRequest, ProvisionalGeometrySnapshot, Sha256Digest,
        SnapshotReference,
    };

    use super::*;

    #[derive(Debug, Default)]
    struct StaticAnalyzer {
        calls: AtomicU64,
    }

    impl GeometryAnalysisPort for StaticAnalyzer {
        fn analyze(
            &self,
            request: &GeometryWorkerRequest,
            grant: AssetReadGrant,
            _cancellation: &AtomicBool,
        ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure> {
            assert_eq!(grant.asset_capability(), request.asset_capability());
            self.calls.fetch_add(1, Ordering::Relaxed);
            let snapshot = ProvisionalGeometrySnapshot::new(
                request.expected_source_hash().clone(),
                "8.0.0",
                3,
                1,
                1,
                decimal("392"),
                decimal("480"),
                [decimal("6"), decimal("4"), decimal("2.5")],
            )
            .expect("fixture snapshot must be valid");
            AnalyzedGeometryEvidence::new(
                StageStatus::Succeeded,
                vec![
                    GeometryStageReport::new(
                        GeometryStage::BasicProperties,
                        StageStatus::Succeeded,
                        Vec::new(),
                    )
                    .expect("stage report must be valid"),
                ],
                SnapshotReference::new("desktop-analysis-snapshot")
                    .expect("snapshot reference must be valid"),
                Sha256Digest::new("b".repeat(64)).expect("output hash must be valid"),
                256,
                snapshot,
                None,
                None,
            )
            .map_err(|_| GeometryAnalysisFailure::ControlledOutputInvalid)
        }
    }

    #[test]
    fn authorized_selected_source_returns_path_free_provisional_evidence_and_audit() {
        let adapter = DesktopAnalysisAdapter::new(StaticAnalyzer::default());
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/rectangular_prism_12x8x5.step");

        let (session, result) = adapter
            .analyze("selection-1", &source, 1)
            .expect("authorized fixture must produce provisional evidence");

        assert_eq!(adapter.audit_event_count(), 1);
        assert_eq!(result.selection_id, "selection-1");
        assert_eq!(result.analysis_id, "analysis-1");
        assert_eq!(result.geometry.surface_area_mm2, "392");
        assert_eq!(result.geometry.enclosed_volume_mm3, "480");
        assert_eq!(result.geometry.center_of_mass_mm, ["6", "4", "2.5"]);
        assert_eq!(result.analysis_status, AnalysisStatus::ProvisionalAvailable);
        assert_eq!(result.estimate.state, EstimateAvailability::Unavailable);
        assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));
        let serialized = serde_json::to_string(&result).expect("result must serialize");
        assert!(!serialized.contains(source.to_string_lossy().as_ref()));
        assert!(!serialized.contains("fixtures/models"));
    }

    #[test]
    fn nonabsolute_source_is_rejected_before_policy_audit_or_analysis() {
        let adapter = DesktopAnalysisAdapter::new(StaticAnalyzer::default());

        let error = adapter
            .analyze("selection-1", Path::new("relative.step"), 1)
            .expect_err("relative ambient path must be rejected");

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::InvalidSelection
        );
        assert_eq!(adapter.audit_event_count(), 0);
    }

    #[cfg(feature = "desktop-host")]
    #[test]
    fn desktop_worker_configuration_fails_closed_when_paths_are_missing() {
        let error = DesktopAnalysisConfiguration::new(
            PathBuf::from("missing-worker"),
            PathBuf::from("missing-workspace"),
            PathBuf::from("missing-native-libraries"),
        )
        .expect_err("missing deployment paths must keep analysis unavailable");

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::AnalysisUnavailable
        );
        assert!(!error.message.contains("missing-worker"));
    }

    fn decimal(value: &str) -> ProvisionalGeometryDecimal {
        ProvisionalGeometryDecimal::new(value).expect("fixture decimal must be valid")
    }
}
