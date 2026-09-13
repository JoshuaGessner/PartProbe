use std::path::PathBuf;

use partprobe_model_viewer::{StandardView, render_synthetic_model_and_stock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/vis-1-model-stock.ppm"));
    let frame = render_synthetic_model_and_stock(960, 640, StandardView::Isometric)?;
    frame.write_ppm(&output)?;
    println!(
        "rendered {}x{} {} via {} to {}",
        frame.report.width,
        frame.report.height,
        frame.report.scene_reference,
        frame.report.backend,
        output.display()
    );
    Ok(())
}
