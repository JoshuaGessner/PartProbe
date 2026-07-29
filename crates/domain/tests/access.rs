use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId, RecordVersionId,
};

#[test]
fn access_control_ids_preserve_opaque_deployment_values() {
    assert_eq!(
        ActorId::new("identity-provider:actor-1")
            .expect("actor ID must be valid")
            .as_str(),
        "identity-provider:actor-1"
    );
    assert!(ProjectId::new("project-1").is_ok());
    assert!(RecordId::new("asset-1").is_ok());
    assert!(RecordVersionId::new("revision-1").is_ok());
    assert!(DataClassificationId::new("organization-defined").is_ok());
    assert!(RecordStateId::new("approved").is_ok());
    assert!(AssetRootId::new("controlled-root-1").is_ok());
}

#[test]
fn access_control_ids_reject_invalid_construction_and_deserialization() {
    assert!(ActorId::new("").is_err());
    assert!(ProjectId::new("x".repeat(257)).is_err());
    assert!(RecordId::new("asset\0id").is_err());
    assert!(serde_json::from_str::<AssetRootId>("\"\"").is_err());
    assert!(serde_json::from_str::<DataClassificationId>("\"class\\u0000ification\"").is_err());
}
