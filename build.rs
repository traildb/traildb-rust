extern crate bindgen;
use std::path::Path;

fn main() {
    println!("cargo:rustc-link-lib=traildb");

    let _ = bindgen::builder()
        .header("src/ffi/include/traildb.h")
        .no_unstable_rust()
        .emit_builtins()
        .link("traildb")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new("src/ffi/mod.rs"));
}
