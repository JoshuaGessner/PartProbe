use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/occt_bridge.cpp");
    println!("cargo:rerun-if-env-changed=PARTPROBE_OCCT_ROOT");

    if env::var_os("CARGO_FEATURE_NATIVE_OCCT").is_none() {
        return;
    }

    let root = env::var_os("PARTPROBE_OCCT_ROOT")
        .map(PathBuf::from)
        .expect("native-occt requires PARTPROBE_OCCT_ROOT");
    let include = root.join("include").join("opencascade");
    let library = root.join("lib");
    assert!(
        include.is_dir() && library.is_dir(),
        "PARTPROBE_OCCT_ROOT must contain include/opencascade and lib"
    );

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .include(&include)
        .file("src/occt_bridge.cpp")
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .compile("partprobe_occt_bridge");

    println!("cargo:rustc-link-search=native={}", library.display());
    for library_name in [
        "TKDESTEP",
        "TKXSBase",
        "TKShHealing",
        "TKMesh",
        "TKTopAlgo",
        "TKBRep",
        "TKGeomAlgo",
        "TKGeomBase",
        "TKG3d",
        "TKG2d",
        "TKMath",
        "TKernel",
    ] {
        println!("cargo:rustc-link-lib=dylib={library_name}");
    }
}
