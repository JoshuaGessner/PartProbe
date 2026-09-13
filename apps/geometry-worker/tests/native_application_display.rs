#![cfg(feature = "native-occt")]

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use partprobe_application::{
    AssetReadSubject, DraftEstimateApplication, DraftGeometryRequestTemplate, LocalAssetReadService,
};
use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId,
    RecordVersionId, RecordedAt, RuleVersion, SchemaVersion,
};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, DisplayTessellationProfile, GeometryStage,
    ProvisionalGeometryDecimal,
};
use partprobe_geometry_import::{
    AssetCapability, CorrelationId, DisplaySceneRequest, GEOMETRY_WORKER_SCHEMA_VERSION,
    GeometryJobId, GeometryWorkerSupervisor, LocalAssetRoot, ResourceQuotas, SupervisorPolicy,
};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationAuditEvent, AuthorizationAuditSink,
    AuthorizationDecision, AuthorizationPolicy, AuthorizationReasonCode, SecurityPolicyId,
    SecurityPolicyRef, SecurityPolicyVersion,
};

#[derive(Debug)]
struct AllowFixturePolicy;

impl AuthorizationPolicy for AllowFixturePolicy {
    fn evaluate(
        &self,
        _context: &partprobe_security::AuthorizationContext,
    ) -> AuthorizationDecision {
        AuthorizationDecision::allow(
            SecurityPolicyRef::new(
                SecurityPolicyId::new("native-display-fixture-policy")
                    .expect("policy ID must be valid"),
                SecurityPolicyVersion::new(1).expect("policy version must be valid"),
            ),
            AuthorizationReasonCode::new("PUBLIC_SYNTHETIC_FIXTURE")
                .expect("reason code must be valid"),
        )
    }
}

#[derive(Debug)]
struct RecordingAudit;

impl AuthorizationAuditSink for RecordingAudit {
    fn append(&self, _event: AuthorizationAuditEvent) -> Result<(), AuditAppendError> {
        Ok(())
    }
}

#[test]
fn native_display_scene_survives_the_authorized_application_session_boundary() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-native-application-display-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let occt_root =
        PathBuf::from(std::env::var_os("PARTPROBE_OCCT_ROOT").expect("OCCT root must be set"));
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_native_library_directory(native_library_directory(&occt_root))
    .expect("native library directory must be valid");
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models");
    let root = LocalAssetRoot::open(
        AssetRootId::new("public-native-display-fixtures").expect("root ID must be valid"),
        &fixture_root,
    )
    .expect("fixture capability root must open");
    let application = DraftEstimateApplication::new(
        LocalAssetReadService::new(AllowFixturePolicy, RecordingAudit),
        supervisor,
    );
    let template = DraftGeometryRequestTemplate::new(
        SchemaVersion::new(GEOMETRY_WORKER_SCHEMA_VERSION)
            .expect("worker schema version must be valid"),
        GeometryJobId::new("native-application-display-job").expect("job ID must be valid"),
        CorrelationId::new("native-application-display-correlation")
            .expect("correlation ID must be valid"),
        AssetCapability::new("native-application-display-capability")
            .expect("capability must be valid"),
        vec![
            GeometryStage::Intake,
            GeometryStage::Identify,
            GeometryStage::Parse,
            GeometryStage::UnitResolution,
            GeometryStage::Validation,
            GeometryStage::BasicProperties,
        ],
        AnalysisProfile {
            id: AnalysisProfileId::new("occt-step-spike").expect("profile ID must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 65_536, 100_000, 5_000)
            .expect("worker quotas must be valid"),
    )
    .expect("request template must be valid")
    .with_display_scene(DisplaySceneRequest::new(
        DisplayTessellationProfile::new(decimal("0.1"), decimal("12"))
            .expect("display profile must be valid"),
    ))
    .expect("schema-v2 display request must be valid");

    let session = application
        .start_session_from_unfingerprinted_source(
            subject(),
            &root,
            &template,
            Path::new("rectangular_prism_12x8x5.step"),
            &AtomicBool::new(false),
        )
        .expect("authorized prism must reach a retained native application session");

    let geometry = session.geometry();
    assert!(geometry.diagnostic_codes().is_empty());
    assert_eq!(
        geometry
            .exact_step_envelope()
            .expect("authoritative analysis must remain available")
            .aabb_extents_mm()
            .each_ref()
            .map(|extent| extent.as_str()),
        ["12", "8", "5"]
    );
    let scene = geometry
        .display_scene()
        .expect("validated display derivative must survive in native session state");
    assert_eq!(scene.manifest().total_vertex_count(), 24);
    assert_eq!(scene.manifest().total_triangle_count(), 12);
    assert_eq!(scene.chunks()[0].positions().len(), 24);
    assert_eq!(scene.chunks()[0].triangle_indices().len(), 36);
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    drop(session);
    drop(application);
    drop(root);
    std::fs::remove_dir(job_directory).expect("empty job directory must be removable");
}

fn decimal(value: &str) -> ProvisionalGeometryDecimal {
    ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be valid")
}

fn native_library_directory(occt_root: &Path) -> PathBuf {
    occt_root.join(if cfg!(windows) { "bin" } else { "lib" })
}

fn subject() -> AssetReadSubject {
    AssetReadSubject::new(
        ActorId::new("native-display-fixture-actor").expect("actor ID must be valid"),
        ProjectId::new("native-display-fixture-project").expect("project ID must be valid"),
        RecordId::new("native-display-fixture-record").expect("record ID must be valid"),
        RecordVersionId::new("native-display-fixture-revision")
            .expect("record version must be valid"),
        DataClassificationId::new("public-synthetic").expect("classification ID must be valid"),
        RecordStateId::new("draft").expect("record state must be valid"),
        AuditCorrelationId::new("native-display-fixture-audit")
            .expect("audit correlation must be valid"),
        RecordedAt::new("2026-09-12T21:30:00-05:00").expect("timestamp must be valid"),
    )
}
