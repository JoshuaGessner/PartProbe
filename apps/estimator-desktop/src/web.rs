use leptos::prelude::*;
use partprobe_desktop_contract::{COMMAND_SELECT_MODEL_SOURCE, SelectedModelSource};
use wasm_bindgen::prelude::*;

use crate::ModelPanelState;

#[wasm_bindgen(inline_js = r#"
export async function invokePartProbe(command) {
  return await window.__TAURI__.core.invoke(command);
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = invokePartProbe)]
    async fn invoke_partprobe(command: &str) -> Result<JsValue, JsValue>;
}

pub fn mount() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (model_state, set_model_state) = signal(ModelPanelState::Empty);
    let (is_selecting, set_is_selecting) = signal(false);

    let select_model = move |_| {
        set_is_selecting.set(true);
        leptos::task::spawn_local(async move {
            let next = invoke_partprobe(COMMAND_SELECT_MODEL_SOURCE)
                .await
                .ok()
                .and_then(|value| serde_wasm_bindgen::from_value(value).ok());

            match next {
                Some(selection) => set_model_state.update(|state| state.apply_selection(selection)),
                None => set_model_state.set(ModelPanelState::Failed),
            }
            set_is_selecting.set(false);
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
                        class="primary-action"
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
            </section>

            <aside class="estimate-panel" aria-labelledby="estimate-heading">
                <div class="panel-heading">
                    <div>
                        <p class="section-index">"02 / ESTIMATE"</p>
                        <h2 id="estimate-heading">"Draft estimate"</h2>
                    </div>
                    <span class="status-chip blocked">"Unavailable"</span>
                </div>
                <div class="blocked-state">
                    <p class="blocked-title">"Analysis inputs are not ready"</p>
                    <p>
                        "This checkpoint proves the desktop security boundary and native source selection. It does not analyze geometry or calculate price yet."
                    </p>
                    <dl>
                        <div><dt>"Geometry"</dt><dd>"Not analyzed"</dd></div>
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
