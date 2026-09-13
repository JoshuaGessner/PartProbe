#[cfg(feature = "desktop-host")]
fn main() {
    const COMMANDS: &[&str] = &[
        "desktop_contract",
        "select_model_source",
        "analyze_model_source",
        "cancel_model_analysis",
        "evaluate_draft_estimate",
        "prepare_draft_estimate_proposal",
        "load_shop_settings",
        "save_shop_settings",
        "activate_shop_resource_selection",
        "save_shop_resource_catalog_draft",
        "set_model_viewer_workspace",
    ];

    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("Tauri desktop configuration must be valid");
}

#[cfg(not(feature = "desktop-host"))]
fn main() {}
