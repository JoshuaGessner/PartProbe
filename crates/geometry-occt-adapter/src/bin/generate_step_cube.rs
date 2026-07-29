use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    let Some(output) = arguments.next() else {
        return ExitCode::from(64);
    };
    if arguments.next().is_some() {
        return ExitCode::from(64);
    }
    match partprobe_geometry_occt_adapter::write_synthetic_step_cube(Path::new(&output), 10.0) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}
