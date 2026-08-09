use leptos::prelude::*;
use partprobe_desktop_contract::{
    AnalyzeModelSourceRequest, COMMAND_ANALYZE_MODEL_SOURCE, COMMAND_SELECT_MODEL_SOURCE,
    HostCommandError, ModelAnalysisResult, ModelSourceSelection, SelectedModelSource,
};
use wasm_bindgen::prelude::*;

use crate::{AnalysisPanelState, ModelPanelState};

#[wasm_bindgen(inline_js = r#"
export async function invokePartProbe(command, args) {
  return await window.__TAURI__.core.invoke(command, args ?? {});
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = invokePartProbe)]
    async fn invoke_partprobe(command: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(serde::Serialize)]
struct AnalyzeModelSourceArgs {
    request: AnalyzeModelSourceRequest,
}

pub fn mount() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (model_state, set_model_state) = signal(ModelPanelState::Empty);
    let (analysis_state, set_analysis_state) = signal(AnalysisPanelState::NotStarted);
    let (is_selecting, set_is_selecting) = signal(false);

    let select_model = move |_| {
        set_is_selecting.set(true);
        leptos::task::spawn_local(async move {
            let next = invoke_partprobe(COMMAND_SELECT_MODEL_SOURCE, JsValue::UNDEFINED)
                .await
                .ok()
                .and_then(|value| serde_wasm_bindgen::from_value(value).ok());

            match next {
                Some(selection @ ModelSourceSelection::Selected { .. }) => {
                    set_model_state.update(|state| state.apply_selection(selection));
                    set_analysis_state.set(AnalysisPanelState::NotStarted);
                }
                Some(ModelSourceSelection::Cancelled) => {}
                None => set_model_state.set(ModelPanelState::Failed),
            }
            set_is_selecting.set(false);
        });
    };

    let analyze_model = move |_| {
        let Some(selection_id) = model_state
            .get_untracked()
            .selected_source()
            .map(|source| source.selection_id.clone())
        else {
            return;
        };
        set_analysis_state.set(AnalysisPanelState::Running);
        leptos::task::spawn_local(async move {
            let request = AnalyzeModelSourceRequest { selection_id };
            let args = serde_wasm_bindgen::to_value(&AnalyzeModelSourceArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_ANALYZE_MODEL_SOURCE, args).await {
                Ok(value) => {
                    let result = serde_wasm_bindgen::from_value::<ModelAnalysisResult>(value)
                        .map(Box::new)
                        .map(AnalysisPanelState::Available)
                        .unwrap_or_else(|_| {
                            AnalysisPanelState::Failed(HostCommandError::analysis_failed(
                                "GUI4-ANALYSIS-RESULT",
                            ))
                        });
                    set_analysis_state.set(result);
                }
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::analysis_failed("GUI4-ANALYSIS-INVOKE")
                        });
                    set_analysis_state.set(AnalysisPanelState::Failed(error));
                }
            }
        });
    };

    view! {
        <a class="skip-link" href="#workspace">"Skip to workspace"</a>
        <header class="app-header">
            <div>
                <p class="eyebrow">"PARTPROBE / DEVELOPER ALPHA"</p>
                <h1>"Estimate workspace"</h1>
            </div>
            <div class="session-state" aria-label="Application state">
                <span class="state-dot" aria-hidden="true"></span>
                <span>"Session only · not saved"</span>
            </div>
        </header>

        <main id="workspace" class="workspace">
            <section class="model-panel" aria-labelledby="model-heading">
                <div class="panel-heading">
                    <div>
                        <p class="section-index">"01 / SOURCE"</p>
                        <h2 id="model-heading">"Model intake"</h2>
                    </div>
                    <span class="status-chip">"STEP only"</span>
                </div>

                <div class="drop-zone">
                    <div class="model-mark" aria-hidden="true">"P"</div>
                    <p class="status-heading" aria-live="polite">
                        {move || if is_selecting.get() {
                            "Waiting for model selection"
                        } else {
                            model_state.get().status_heading()
                        }}
                    </p>
                    <p class="status-detail">
                        {move || if is_selecting.get() {
                            "PartProbe is showing the native file picker."
                        } else {
                            model_state.get().status_detail()
                        }}
                    </p>
                    <button
                        type="button"
                        class="secondary-action"
                        disabled=move || is_selecting.get()
                        on:click=select_model
                    >
                        {move || if is_selecting.get() {
                            "Picker open"
                        } else {
                            "Choose STEP model"
                        }}
                    </button>
                </div>

                <SelectedSource state=model_state />
                <button
                    type="button"
                    class="primary-action analyze-action"
                    disabled=move || {
                        model_state.get().selected_source().is_none()
                            || matches!(analysis_state.get(), AnalysisPanelState::Running)
                    }
                    on:click=analyze_model
                >
                    {move || if matches!(analysis_state.get(), AnalysisPanelState::Running) {
                        "Analyzing model"
                    } else {
                        "Analyze provisional geometry"
                    }}
                </button>
                <AnalysisEvidence state=analysis_state />
            </section>

            <aside class="estimate-panel" aria-labelledby="estimate-heading">
                <div class="panel-heading">
                    <div>
                        <p class="section-index">"02 / ESTIMATE"</p>
                        <h2 id="estimate-heading">"Draft estimate"</h2>
                    </div>
                    <span class="status-chip blocked">"Unavailable"</span>
                </div>
                <div class="blocked-state" aria-live="polite">
                    <p class="blocked-title">{move || analysis_state.get().status_heading()}</p>
                    <p>
                        {move || analysis_state.get().status_detail().to_owned()}
                    </p>
                    <dl>
                        <div>
                            <dt>"Geometry"</dt>
                            <dd>{move || match analysis_state.get() {
                                AnalysisPanelState::Available(_) => "Provisional facts available",
                                AnalysisPanelState::Running => "Analysis running",
                                AnalysisPanelState::Failed(_) => "Failed safely",
                                AnalysisPanelState::NotStarted => "Not analyzed",
                            }}</dd>
                        </div>
                        <div><dt>"Units"</dt><dd>"Not reviewed"</dd></div>
                        <div><dt>"Rate basis"</dt><dd>"Not selected"</dd></div>
                        <div><dt>"Selling price"</dt><dd>"Unavailable"</dd></div>
                    </dl>
                </div>
            </aside>
        </main>
    }
}

#[component]
fn SelectedSource(state: ReadSignal<ModelPanelState>) -> impl IntoView {
    move || match state.get() {
        ModelPanelState::Selected(source) => view! { <SourceSummary source /> }.into_any(),
        _ => ().into_any(),
    }
}

#[component]
fn SourceSummary(source: SelectedModelSource) -> impl IntoView {
    view! {
        <dl class="source-summary" aria-label="Selected model source">
            <div><dt>"File"</dt><dd>{source.display_name}</dd></div>
            <div><dt>"Format"</dt><dd>"STEP"</dd></div>
            <div><dt>"Analysis"</dt><dd>"Not started"</dd></div>
            <div><dt>"Storage"</dt><dd>"Session only"</dd></div>
        </dl>
    }
}

#[component]
fn AnalysisEvidence(state: ReadSignal<AnalysisPanelState>) -> impl IntoView {
    move || {
        match state.get() {
        AnalysisPanelState::Available(result) => {
            let geometry = result.geometry;
            let centroid = geometry.center_of_mass_mm.join(", ");
            let warning_count = result
                .stages
                .iter()
                .map(|stage| stage.warning_codes.len())
                .sum::<usize>();
            view! {
                <section class="analysis-evidence" aria-labelledby="analysis-evidence-heading">
                    <div class="analysis-evidence-heading">
                        <div>
                            <p class="section-index">"PROVISIONAL / SESSION ONLY"</p>
                            <h3 id="analysis-evidence-heading">"Geometry evidence"</h3>
                        </div>
                        <span class="status-chip">"Review required"</span>
                    </div>
                    <dl class="geometry-facts">
                        <div><dt>"Surface area"</dt><dd>{geometry.surface_area_mm2}" mm²"</dd></div>
                        <div><dt>"Enclosed volume"</dt><dd>{geometry.enclosed_volume_mm3}" mm³"</dd></div>
                        <div><dt>"Centroid"</dt><dd>{centroid}" mm"</dd></div>
                        <div><dt>"Solid bodies"</dt><dd>{geometry.solid_body_count}</dd></div>
                        <div><dt>"Canonical units"</dt><dd>"Millimeter"</dd></div>
                        <div><dt>"Warnings"</dt><dd>{warning_count}</dd></div>
                        <div><dt>"Engine"</dt><dd>{geometry.geometry_engine}</dd></div>
                        <div><dt>"Analysis ID"</dt><dd>{result.analysis_id}</dd></div>
                    </dl>
                    <p class="evidence-note">
                        "These exact-B-rep measurements are provisional spike evidence. They are not a supported importer result or an approved estimate."
                    </p>
                </section>
            }
            .into_any()
        }
        AnalysisPanelState::Failed(error) => view! {
            <section class="analysis-error" role="alert">
                <p class="blocked-title">"Analysis failed safely"</p>
                <p>{error.message}</p>
                <p class="diagnostic-id">"Diagnostic: " {error.diagnostic_id}</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::Running => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Isolated worker analysis in progress"</p>
                <p>"The selected source remains session-only. Cancellation is not available in this checkpoint."</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::NotStarted => ().into_any(),
    }
    }
}
