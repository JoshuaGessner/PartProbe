#![forbid(unsafe_code)]

#[cfg(target_arch = "wasm32")]
fn main() {
    partprobe_estimator_desktop_ui::mount();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
