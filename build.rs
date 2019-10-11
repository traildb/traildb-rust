use std::path::Path;
use std::env;

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

fn build_traildb() {
    let target = env::var("TARGET").unwrap();

    let mut compiler = cc::Build::new();
    compiler.include("traildb/src/dsfmt/");
    compiler.include("traildb/src/pqueue/");
    compiler.include("traildb/src/xxhash/");
    compiler.include("traildb/src/");

    let globs = &["traildb/src/**/*.c"];

    for pattern in globs {
        for path in glob::glob(pattern).unwrap() {
            let path = path.unwrap();
            compiler.file(path);
        }
    }
    if target.contains("x86_64") {
        // This is needed to enable hardware CRC32C. Technically, SSE 4.2 is
        // only available since Intel Nehalem (about 2010) and AMD Bulldozer
        // (about 2011).
        compiler.define("HAVE_PCLMUL", Some("1"));
        compiler.define("HAVE_SSE42", Some("1"));
        compiler.flag_if_supported("-msse2");
        compiler.flag_if_supported("-msse4.1");
        compiler.flag_if_supported("-msse4.2");
        compiler.flag_if_supported("-mpclmul");
    }

    compiler.define("DSFMT_MEXP", Some("521"));
    compiler.define("HAVE_ARCHIVE_H", Some("1"));

    compiler.flag("-std=c99");
    compiler.flag("-O3");
    compiler.flag("-g");
    compiler.flag_if_supported("-Wextra");
    compiler.flag_if_supported("-Wcast-qual");
    compiler.flag_if_supported("-Wformat-security");
    compiler.flag_if_supported("-Wformat");
    compiler.flag_if_supported("-Wmissing-declarations");
    compiler.flag_if_supported("-Wmissing-prototypes");
    compiler.flag_if_supported("-Wnested-externs");
    compiler.flag_if_supported("-Wpointer-arith");
    compiler.flag_if_supported("-Wshadow");
    compiler.flag_if_supported("-Wstrict-prototypes");

    compiler.cpp(false);
    compiler.compile("traildb");
}

fn dep_libs() {
    // Judy does not support querying by the pkg-config
    println!("cargo:rustc-link-lib=dylib=Judy");

    let lib = pkg_config::probe_library("libarchive").expect("libarchive is missing");
    println!("cargo:rustc-link-lib=dylib={}", lib.libs[0]);
}

fn main() {
    bindgen_traildb();
    match pkg_config::Config::new().atleast_version("0.7").statik(true).probe("traildb") {
        Ok(traildb) => println!("cargo:rustc-link-lib=static={}", traildb.libs[0]),
        Err(_) => build_traildb(),
    }
    dep_libs();
}
