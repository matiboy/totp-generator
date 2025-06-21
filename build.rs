fn main() {
    if std::env::var("CARGO_FEATURE_CONFIGURE").is_ok() {
        println!("cargo:rerun-if-changed=wrapper.h");

        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Failed to generate bindings");

        bindings
            .write_to_file("src/bindings.rs")
            .expect("Failed to write bindings");

        println!("cargo:rustc-link-lib=dylib=zbar");
    }
}
