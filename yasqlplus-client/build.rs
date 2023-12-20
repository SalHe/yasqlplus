use std::env;
use std::path::PathBuf;

fn main() {
    let src_dir = env!("CARGO_MANIFEST_DIR");

    println!("cargo:rustc-link-search={src_dir}/yas-client/lib");
    println!("cargo:rustc-link-lib=yascli");
    println!("cargo:rustc-link-lib=yas_infra");
    println!("cargo:rerun-if-changed={src_dir}/yas-client/include/yacli.h");
    let bindings = bindgen::Builder::default()
        .header(format!("{src_dir}/yas-client/include/yacli.h"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
