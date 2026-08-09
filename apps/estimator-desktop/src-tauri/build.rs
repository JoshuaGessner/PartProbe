#[cfg(feature = "desktop-host")]
fn main() {
    const COMMANDS: &[&str] = &[
        "desktop_contract",
        "select_model_source",
        "analyze_model_source",
        "cancel_model_analysis",
        "evaluate_draft_estimate",
    ];

    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("Tauri desktop configuration must be valid");
}

#[cfg(not(feature = "desktop-host"))]
fn main() {}
