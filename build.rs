fn main() {
    println!("cargo:rustc-link-lib=static=traildb");

    // Judy does not support querying by the pkg-config
    println!("cargo:rustc-link-lib=dylib=Judy");

    let lib = pkg_config::probe_library("libarchive").expect("libarchive is missing");
    println!("cargo:rustc-link-lib=dylib={}", lib.libs[0]);
}
