use std::path::Path;
#[cfg(feature = "desktop-host")]
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
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
    GeometryConfidenceLevel as DesktopConfidenceLevel, GeometryEvidenceState, GeometryStageSummary,
    HostCommandError, MeshMeasurementBasis,
    MeshSelfIntersectionState as DesktopSelfIntersectionState,
    MeshTopologyIdentity as DesktopTopologyIdentity, MeshWeldingStatus as DesktopWeldingStatus,
    ModelAnalysisResult, ModelLengthUnit as DesktopLengthUnit, ModelSourceFormat,
    PersistenceAvailability, ProvisionalExactBrepFacts, ProvisionalGeometryFacts,
    ProvisionalMeshFacts, StlEncoding as DesktopStlEncoding,
    UnitResolution as DesktopUnitResolution,
};
use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId,
    RecordVersionId, RecordedAt, RuleVersion, SchemaVersion, ValueState,
};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryConfidence, GeometryConfidenceLevel, GeometryStage,
    ModelLengthUnit, StageStatus, UnitResolutionMethod,
};
use partprobe_geometry_import::{
    AssetCapability, CorrelationId, GeometryJobId, LocalAssetRoot, MeshSelfIntersectionState,
    MeshTopologyIdentity, MeshVector3, MeshWeldingStatus, ProvisionalMeshEvidence, ResourceQuotas,
    StlEncoding,
};
#[cfg(feature = "desktop-host")]
use partprobe_geometry_import::{GeometryWorkerSupervisor, SupervisorPolicy};
#[cfg(feature = "desktop-host")]
use partprobe_native_runtime::VerifiedNativeRuntime;
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationAuditEvent, AuthorizationAuditSink,
    AuthorizationContext, AuthorizationDecision, AuthorizationPolicy, AuthorizationReasonCode,
    ProtectedOperation, SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion,
};

pub(crate) const DEVELOPER_ACTOR_ID: &str = "local-developer-session";
const DEVELOPER_PROJECT_ID: &str = "gui-4-developer-slice";
const DEVELOPER_CLASSIFICATION_ID: &str = "local-test-data";
const DEVELOPER_RECORD_STATE_ID: &str = "ephemeral-draft";
const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 1024 * 1024;
const MAX_ENTITIES: u64 = 2_000_000;
const WALL_TIME_MILLIS: u64 = 30_000;
#[cfg(feature = "desktop-host")]
const BUNDLED_NATIVE_RUNTIME_DIRECTORY: &str = "partprobe-native-runtime";

#[cfg(feature = "desktop-host")]
#[derive(Debug)]
pub struct DesktopAnalysisConfiguration {
    worker_executable: PathBuf,
    worker_workspace: PathBuf,
    native_library_directory: PathBuf,
}

#[cfg(feature = "desktop-host")]
impl DesktopAnalysisConfiguration {
    #[cfg(test)]
    pub fn from_environment() -> Result<Self, HostCommandError> {
        let runtime_root = required_path("PARTPROBE_NATIVE_RUNTIME")?;
        Self::from_runtime_root(runtime_root)
    }

    pub fn from_deployment_resource_directory(
        resource_directory: &Path,
    ) -> Result<Self, HostCommandError> {
        let runtime_root = deployment_runtime_root(
            resource_directory,
            optional_path("PARTPROBE_NATIVE_RUNTIME"),
        );
        Self::from_runtime_root(runtime_root)
    }

    fn from_runtime_root(runtime_root: PathBuf) -> Result<Self, HostCommandError> {
        let worker_workspace = required_path("PARTPROBE_GEOMETRY_WORKSPACE")?;
        let runtime = VerifiedNativeRuntime::verify(runtime_root)
            .map_err(|_| HostCommandError::analysis_unavailable("GUI5-NATIVE-RUNTIME-VERIFY"))?;
        Self::new(
            runtime.worker_executable().to_path_buf(),
            worker_workspace,
            runtime.native_library_directory().to_path_buf(),
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
fn deployment_runtime_root(
    resource_directory: &Path,
    explicit_runtime_root: Option<PathBuf>,
) -> PathBuf {
    explicit_runtime_root
        .unwrap_or_else(|| resource_directory.join(BUNDLED_NATIVE_RUNTIME_DIRECTORY))
}

#[cfg(feature = "desktop-host")]
fn optional_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(feature = "desktop-host")]
fn required_path(name: &str) -> Result<PathBuf, HostCommandError> {
    optional_path(name)
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
        selected_format: ModelSourceFormat,
        source_path: &Path,
        analysis_number: u64,
        cancellation: &AtomicBool,
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
                cancellation,
            )
            .map_err(|error| map_application_error(error, cancellation.load(Ordering::Acquire)))?;
        let result = analysis_result(selection_id, &ids.analysis_id, selected_format, &session)?;
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
            id: AnalysisProfileId::new("governed-geometry-spike")
                .map_err(|_| internal_contract_error())?,
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

pub(crate) fn trusted_recorded_at() -> Result<RecordedAt, HostCommandError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| internal_contract_error())?
        .as_secs();
    RecordedAt::new(format!("unix-seconds:{seconds}")).map_err(|_| internal_contract_error())
}

fn map_application_error(
    error: DraftEstimateApplicationError,
    cancellation_requested: bool,
) -> HostCommandError {
    if cancellation_requested {
        return HostCommandError::analysis_cancelled("GUI4-ANALYSIS-CANCELLED");
    }
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
    selected_format: ModelSourceFormat,
    session: &DraftEstimateSession,
) -> Result<ModelAnalysisResult, HostCommandError> {
    let evidence = session.geometry();
    let estimate_reason = match session.evaluate() {
        ValueState::Unavailable { reason }
        | ValueState::Blocked { reason }
        | ValueState::Unknown { reason } => reason,
        ValueState::Stale { reason, .. } => reason,
        ValueState::Available { .. } => {
            "estimate state requires explicit review before display".to_owned()
        }
    };
    let (detected_format, evidence_state, geometry) =
        if let Some(snapshot) = evidence.exact_snapshot() {
            (
                ModelSourceFormat::Step,
                GeometryEvidenceState::ProvisionalExactBrepSpike,
                ProvisionalGeometryFacts::ExactBrep(ProvisionalExactBrepFacts {
                    canonical_units: CanonicalLengthUnit::Millimeter,
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
                }),
            )
        } else {
            let snapshot = evidence
                .mesh_snapshot()
                .expect("validated application evidence must contain exact or mesh geometry");
            (
                match snapshot.evidence() {
                    ProvisionalMeshEvidence::Stl(_) => ModelSourceFormat::Stl,
                    ProvisionalMeshEvidence::ThreeMf(_) => ModelSourceFormat::ThreeMf,
                },
                GeometryEvidenceState::ProvisionalMeshSpike,
                ProvisionalGeometryFacts::Mesh(map_mesh_facts(
                    snapshot.evidence(),
                    snapshot.source_hash().as_str(),
                    evidence.output_hash.as_str(),
                    evidence.output_byte_length,
                )),
            )
        };
    if detected_format != selected_format {
        return Err(HostCommandError::analysis_failed(
            "GUI4-ANALYSIS-FORMAT-MISMATCH",
        ));
    }
    Ok(ModelAnalysisResult {
        selection_id: selection_id.to_owned(),
        analysis_id: analysis_id.to_owned(),
        analysis_status: AnalysisStatus::ProvisionalAvailable,
        evidence_state,
        persistence: PersistenceAvailability::SessionOnly,
        geometry,
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
    })
}

fn map_mesh_facts(
    evidence: &ProvisionalMeshEvidence,
    source_hash: &str,
    output_hash: &str,
    output_byte_length: u64,
) -> ProvisionalMeshFacts {
    match evidence {
        ProvisionalMeshEvidence::Stl(stl) => ProvisionalMeshFacts {
            detected_format: ModelSourceFormat::Stl,
            stl_encoding: Some(map_stl_encoding(stl.encoding())),
            source_units: map_length_unit(stl.source_units()),
            unit_resolution: map_unit_resolution(stl.unit_resolution()),
            unit_was_explicit: None,
            measurement_basis: MeshMeasurementBasis::SourceCoordinates,
            aabb_extents: map_vector(stl.aabb_extents_source_units()),
            surface_area: format_mesh_decimal(stl.surface_area_source_units_squared()),
            enclosed_volume: stl
                .enclosed_volume_source_units_cubed()
                .map(format_mesh_decimal),
            center_of_mass: stl.center_of_mass_source_units().map(map_vector),
            triangle_count: u64::try_from(stl.triangle_count())
                .expect("bounded STL triangle count must fit u64"),
            manifold: stl.manifold(),
            watertight: stl.watertight(),
            consistently_wound: stl.consistently_wound(),
            self_intersection: map_self_intersection(stl.self_intersection()),
            confidence_level: map_confidence_level(stl.confidence().level()),
            confidence_reason_codes: confidence_reasons(stl.confidence()),
            algorithm_version: stl.algorithm_version().to_owned(),
            self_intersection_algorithm_version: stl
                .self_intersection_algorithm_version()
                .to_owned(),
            confidence_policy_version: stl.confidence_policy_version().to_owned(),
            topology_policy_version: stl.topology_policy_version().to_owned(),
            topology_identity: map_topology_identity(stl.topology_identity()),
            welding_status: map_welding_status(stl.welding_status()),
            warning_codes: stl
                .warnings()
                .iter()
                .map(|warning| warning.as_str().to_owned())
                .collect(),
            source_hash_sha256: source_hash.to_owned(),
            output_hash_sha256: output_hash.to_owned(),
            output_byte_length,
        },
        ProvisionalMeshEvidence::ThreeMf(three_mf) => ProvisionalMeshFacts {
            detected_format: ModelSourceFormat::ThreeMf,
            stl_encoding: None,
            source_units: map_length_unit(three_mf.source_units()),
            unit_resolution: map_unit_resolution(three_mf.unit_resolution()),
            unit_was_explicit: Some(three_mf.unit_was_explicit()),
            measurement_basis: MeshMeasurementBasis::CanonicalMillimeters,
            aabb_extents: map_vector(three_mf.aabb_extents_mm()),
            surface_area: format_mesh_decimal(three_mf.surface_area_mm2()),
            enclosed_volume: three_mf.enclosed_volume_mm3().map(format_mesh_decimal),
            center_of_mass: three_mf.center_of_mass_mm().map(map_vector),
            triangle_count: u64::try_from(three_mf.triangle_count())
                .expect("bounded 3MF triangle count must fit u64"),
            manifold: three_mf.manifold(),
            watertight: three_mf.watertight(),
            consistently_wound: three_mf.consistently_wound(),
            self_intersection: map_self_intersection(three_mf.self_intersection()),
            confidence_level: map_confidence_level(three_mf.confidence().level()),
            confidence_reason_codes: confidence_reasons(three_mf.confidence()),
            algorithm_version: three_mf.algorithm_version().to_owned(),
            self_intersection_algorithm_version: three_mf
                .self_intersection_algorithm_version()
                .to_owned(),
            confidence_policy_version: three_mf.confidence_policy_version().to_owned(),
            topology_policy_version: three_mf.topology_policy_version().to_owned(),
            topology_identity: map_topology_identity(three_mf.topology_identity()),
            welding_status: map_welding_status(three_mf.welding_status()),
            warning_codes: three_mf
                .warnings()
                .iter()
                .map(|warning| warning.as_str().to_owned())
                .collect(),
            source_hash_sha256: source_hash.to_owned(),
            output_hash_sha256: output_hash.to_owned(),
            output_byte_length,
        },
    }
}

fn format_mesh_decimal(value: f64) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

fn map_vector(vector: MeshVector3) -> [String; 3] {
    vector.components().map(format_mesh_decimal)
}

fn confidence_reasons(confidence: &GeometryConfidence) -> Vec<String> {
    confidence
        .reasons()
        .iter()
        .map(|reason| reason.as_str().to_owned())
        .collect()
}

const fn map_stl_encoding(encoding: StlEncoding) -> DesktopStlEncoding {
    match encoding {
        StlEncoding::Ascii => DesktopStlEncoding::Ascii,
        StlEncoding::Binary => DesktopStlEncoding::Binary,
    }
}

const fn map_length_unit(unit: ModelLengthUnit) -> DesktopLengthUnit {
    match unit {
        ModelLengthUnit::Micrometer => DesktopLengthUnit::Micrometer,
        ModelLengthUnit::Millimeter => DesktopLengthUnit::Millimeter,
        ModelLengthUnit::Centimeter => DesktopLengthUnit::Centimeter,
        ModelLengthUnit::Meter => DesktopLengthUnit::Meter,
        ModelLengthUnit::Inch => DesktopLengthUnit::Inch,
        ModelLengthUnit::Foot => DesktopLengthUnit::Foot,
        ModelLengthUnit::Unknown => DesktopLengthUnit::Unknown,
    }
}

const fn map_unit_resolution(method: UnitResolutionMethod) -> DesktopUnitResolution {
    match method {
        UnitResolutionMethod::Declared => DesktopUnitResolution::Declared,
        UnitResolutionMethod::Confirmed => DesktopUnitResolution::Confirmed,
        UnitResolutionMethod::Inferred => DesktopUnitResolution::Inferred,
        UnitResolutionMethod::Unresolved => DesktopUnitResolution::Unresolved,
    }
}

const fn map_confidence_level(level: GeometryConfidenceLevel) -> DesktopConfidenceLevel {
    match level {
        GeometryConfidenceLevel::High => DesktopConfidenceLevel::High,
        GeometryConfidenceLevel::Medium => DesktopConfidenceLevel::Medium,
        GeometryConfidenceLevel::Low => DesktopConfidenceLevel::Low,
        GeometryConfidenceLevel::NeedsReview => DesktopConfidenceLevel::NeedsReview,
    }
}

const fn map_topology_identity(identity: MeshTopologyIdentity) -> DesktopTopologyIdentity {
    match identity {
        MeshTopologyIdentity::ExactSourceCoordinates => {
            DesktopTopologyIdentity::ExactSourceCoordinates
        }
        MeshTopologyIdentity::SourceVertexIndices => DesktopTopologyIdentity::SourceVertexIndices,
    }
}

const fn map_welding_status(status: MeshWeldingStatus) -> DesktopWeldingStatus {
    match status {
        MeshWeldingStatus::NotApplied => DesktopWeldingStatus::NotApplied,
    }
}

const fn map_self_intersection(state: MeshSelfIntersectionState) -> DesktopSelfIntersectionState {
    match state {
        MeshSelfIntersectionState::NotDetected => DesktopSelfIntersectionState::NotDetected,
        MeshSelfIntersectionState::Detected => DesktopSelfIntersectionState::Detected,
        MeshSelfIntersectionState::Indeterminate => DesktopSelfIntersectionState::Indeterminate,
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
    use partprobe_desktop_contract::DraftEstimateEvaluationState;
    use partprobe_geometry_core::{GeometryStageReport, ProvisionalGeometryDecimal, StageStatus};
    use partprobe_geometry_import::{
        AssetReadGrant, ControlledGeometryResult, GeometryWorkerRequest,
        PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE, PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE,
        ProvisionalGeometrySnapshot, ProvisionalMeshGeometrySnapshot, Sha256Digest,
        SnapshotReference, StlLimits, ThreeMfLimits, analyze_3mf, analyze_stl,
    };

    use super::*;

    #[derive(Debug, Default)]
    struct StaticAnalyzer {
        calls: AtomicU64,
    }

    #[derive(Debug, Default)]
    struct StaticMeshAnalyzer;

    #[derive(Debug, Default)]
    struct StaticThreeMfAnalyzer;

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
                SnapshotReference::new(PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE)
                    .expect("snapshot reference must be valid"),
                Sha256Digest::new("b".repeat(64)).expect("output hash must be valid"),
                256,
                ControlledGeometryResult::ExactBrep(Box::new(snapshot)),
                None,
                None,
            )
            .map_err(|_| GeometryAnalysisFailure::ControlledOutputInvalid)
        }
    }

    impl GeometryAnalysisPort for StaticMeshAnalyzer {
        fn analyze(
            &self,
            request: &GeometryWorkerRequest,
            grant: AssetReadGrant,
            _cancellation: &AtomicBool,
        ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure> {
            assert_eq!(grant.asset_capability(), request.asset_capability());
            let mesh = analyze_stl(
                include_bytes!("../../../../fixtures/models/cube_10mm_ascii.stl"),
                StlLimits::new(1024 * 1024, 10_000).expect("test limits must be valid"),
            )
            .expect("governed STL fixture must analyze");
            let snapshot = ProvisionalMeshGeometrySnapshot::from_stl(
                request.expected_source_hash().clone(),
                mesh,
            );
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
                SnapshotReference::new(PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE)
                    .expect("snapshot reference must be valid"),
                Sha256Digest::new("c".repeat(64)).expect("output hash must be valid"),
                512,
                ControlledGeometryResult::Mesh(Box::new(snapshot)),
                None,
                None,
            )
            .map_err(|_| GeometryAnalysisFailure::ControlledOutputInvalid)
        }
    }

    impl GeometryAnalysisPort for StaticThreeMfAnalyzer {
        fn analyze(
            &self,
            request: &GeometryWorkerRequest,
            grant: AssetReadGrant,
            _cancellation: &AtomicBool,
        ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure> {
            assert_eq!(grant.asset_capability(), request.asset_capability());
            let mesh = analyze_3mf(
                include_bytes!("../../../../fixtures/models/cube_1cm_translated.3mf"),
                ThreeMfLimits::new(
                    64 * 1024,
                    16,
                    64 * 1024,
                    32 * 1024,
                    100,
                    1_000,
                    4,
                    3,
                    8,
                    100,
                )
                .expect("test limits must be valid"),
            )
            .expect("governed 3MF fixture must analyze");
            let snapshot = ProvisionalMeshGeometrySnapshot::from_three_mf(
                request.expected_source_hash().clone(),
                mesh,
            );
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
                SnapshotReference::new(PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE)
                    .expect("snapshot reference must be valid"),
                Sha256Digest::new("d".repeat(64)).expect("output hash must be valid"),
                768,
                ControlledGeometryResult::Mesh(Box::new(snapshot)),
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

        let (mut session, result) = adapter
            .analyze(
                "selection-1",
                ModelSourceFormat::Step,
                &source,
                1,
                &AtomicBool::new(false),
            )
            .expect("authorized fixture must produce provisional evidence");

        assert_eq!(adapter.audit_event_count(), 1);
        assert_eq!(result.selection_id, "selection-1");
        assert_eq!(result.analysis_id, "analysis-1");
        let ProvisionalGeometryFacts::ExactBrep(geometry) = &result.geometry else {
            panic!("STEP fixture must retain exact-B-rep evidence");
        };
        assert_eq!(geometry.surface_area_mm2, "392");
        assert_eq!(geometry.enclosed_volume_mm3, "480");
        assert_eq!(geometry.center_of_mass_mm, ["6", "4", "2.5"]);
        assert_eq!(result.analysis_status, AnalysisStatus::ProvisionalAvailable);
        assert_eq!(result.estimate.state, EstimateAvailability::Unavailable);
        assert!(matches!(session.evaluate(), ValueState::Unavailable { .. }));
        let serialized = serde_json::to_string(&result).expect("result must serialize");
        assert!(!serialized.contains(source.to_string_lossy().as_ref()));
        assert!(!serialized.contains("fixtures/models"));

        let request = crate::estimate::complete_test_request("selection-1", "analysis-1");
        let evaluation = crate::estimate::evaluate_draft_estimate(&mut session, &request)
            .expect("complete explicit developer inputs must evaluate");
        assert_eq!(evaluation.state, DraftEstimateEvaluationState::Available);
        assert_eq!(
            evaluation
                .result
                .expect("available result")
                .rounded_selling_price,
            "702"
        );
    }

    #[test]
    fn nonabsolute_source_is_rejected_before_policy_audit_or_analysis() {
        let adapter = DesktopAnalysisAdapter::new(StaticAnalyzer::default());

        let error = adapter
            .analyze(
                "selection-1",
                ModelSourceFormat::Step,
                Path::new("relative.step"),
                1,
                &AtomicBool::new(false),
            )
            .expect_err("relative ambient path must be rejected");

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::InvalidSelection
        );
        assert_eq!(adapter.audit_event_count(), 0);
    }

    #[test]
    fn mesh_analysis_is_path_free_and_cannot_authorize_a_draft_estimate() {
        let adapter = DesktopAnalysisAdapter::new(StaticMeshAnalyzer);
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/cube_10mm_ascii.stl");

        let (mut session, result) = adapter
            .analyze(
                "selection-1",
                ModelSourceFormat::Stl,
                &source,
                1,
                &AtomicBool::new(false),
            )
            .expect("governed mesh fixture must produce provisional evidence");

        assert_eq!(
            result.evidence_state,
            GeometryEvidenceState::ProvisionalMeshSpike
        );
        let ProvisionalGeometryFacts::Mesh(geometry) = &result.geometry else {
            panic!("STL fixture must retain mesh evidence");
        };
        assert_eq!(geometry.detected_format, ModelSourceFormat::Stl);
        assert_eq!(geometry.source_units, DesktopLengthUnit::Unknown);
        assert_eq!(
            geometry.measurement_basis,
            MeshMeasurementBasis::SourceCoordinates
        );
        assert_eq!(geometry.welding_status, DesktopWeldingStatus::NotApplied);
        assert!(geometry.enclosed_volume.is_some());
        assert!(geometry.center_of_mass.is_some());
        assert!(
            geometry
                .warning_codes
                .iter()
                .any(|code| code == "UNITS_MISSING_REQUIRES_CONFIRMATION")
        );
        assert!(result.estimate.reason.contains("not authorized"));

        let serialized = serde_json::to_string(&result).expect("result must serialize");
        assert!(!serialized.contains(source.to_string_lossy().as_ref()));
        assert!(!serialized.contains("fixtures/models"));

        let request = crate::estimate::complete_test_request("selection-1", "analysis-1");
        let evaluation = crate::estimate::evaluate_draft_estimate(&mut session, &request)
            .expect("mesh evaluation must return an explicit state");
        assert_eq!(evaluation.state, DraftEstimateEvaluationState::Unavailable);
        assert!(evaluation.reason.unwrap().contains("not authorized"));
    }

    #[test]
    fn three_mf_mapping_preserves_canonical_units_and_mesh_authority() {
        let adapter = DesktopAnalysisAdapter::new(StaticThreeMfAnalyzer);
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/cube_1cm_translated.3mf");

        let (_session, result) = adapter
            .analyze(
                "selection-1",
                ModelSourceFormat::ThreeMf,
                &source,
                1,
                &AtomicBool::new(false),
            )
            .expect("governed 3MF fixture must produce provisional evidence");

        let ProvisionalGeometryFacts::Mesh(geometry) = &result.geometry else {
            panic!("3MF fixture must retain mesh evidence");
        };
        assert_eq!(geometry.detected_format, ModelSourceFormat::ThreeMf);
        assert_eq!(geometry.source_units, DesktopLengthUnit::Centimeter);
        assert_eq!(
            geometry.measurement_basis,
            MeshMeasurementBasis::CanonicalMillimeters
        );
        assert_eq!(
            geometry.topology_identity,
            DesktopTopologyIdentity::SourceVertexIndices
        );
        assert!(geometry.enclosed_volume.is_some());
        assert!(geometry.center_of_mass.is_some());
        assert!(result.estimate.reason.contains("not authorized"));
    }

    #[test]
    fn mesh_numeric_text_never_rounds_a_nonzero_measurement_to_zero() {
        assert_eq!(format_mesh_decimal(0.0), "0");
        assert_eq!(format_mesh_decimal(-0.0), "0");
        assert_ne!(format_mesh_decimal(0.000_000_1), "0");
        assert_ne!(format_mesh_decimal(-0.000_000_1), "0");
    }

    #[test]
    fn selected_extension_and_content_detected_format_must_match() {
        let adapter = DesktopAnalysisAdapter::new(StaticMeshAnalyzer);
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/cube_10mm_ascii.stl");

        let error = adapter
            .analyze(
                "selection-1",
                ModelSourceFormat::ThreeMf,
                &source,
                1,
                &AtomicBool::new(false),
            )
            .expect_err("selected extension and content-derived format must not diverge");

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::AnalysisFailed
        );
        assert_eq!(error.diagnostic_id, "GUI4-ANALYSIS-FORMAT-MISMATCH");
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

    #[cfg(feature = "desktop-host")]
    #[test]
    fn deployment_runtime_location_is_fixed_and_explicit_override_wins() {
        let resource_directory = Path::new("bundle-resources");

        assert_eq!(
            deployment_runtime_root(resource_directory, None),
            resource_directory.join("partprobe-native-runtime")
        );
        assert_eq!(
            deployment_runtime_root(
                resource_directory,
                Some(PathBuf::from("explicit-developer-runtime")),
            ),
            PathBuf::from("explicit-developer-runtime")
        );
    }

    fn decimal(value: &str) -> ProvisionalGeometryDecimal {
        ProvisionalGeometryDecimal::new(value).expect("fixture decimal must be valid")
    }
}
