use std::path::Path;

fn bindgen_traildb() {
    let _ = bindgen::builder()
        .header("src/ffi/include/traildb.h")
        .no_unstable_rust()
        .emit_builtins()
        .link("traildb")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new("src/ffi/mod.rs"));
}

fn dep_libs() {
    println!("cargo:rustc-link-lib=static=traildb");

    // Judy does not support querying by the pkg-config
    println!("cargo:rustc-link-lib=dylib=Judy");

    let lib = pkg_config::probe_library("libarchive").expect("libarchive is missing");
    println!("cargo:rustc-link-lib=dylib={}", lib.libs[0]);
}

fn main() {
    dep_libs();
    bindgen_traildb();
}
