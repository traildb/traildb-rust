use std::env;
use std::fs::File;
use std::path::PathBuf;

use libflate::gzip::Decoder;
use tar::Archive;

use bindgen;

const SOURCES_PREFIX: &str = "traildb-0.6";
const ARCHIVE_SUFFIX: &str = ".tar.gz";

fn extract_archive() {
    // Unpack the tarball
    let archive = File::open(
        SOURCES_PREFIX.to_owned() + ARCHIVE_SUFFIX
    ).unwrap();

    assert!(archive.metadata().unwrap().is_file());

    let gz_decoder = Decoder::new(archive).unwrap();

    let mut archive = Archive::new(gz_decoder);

    archive.unpack(
        PathBuf::from(env::var("OUT_DIR").unwrap())
    ).unwrap();
}

fn build_lib() {
    let target = env::var("TARGET").unwrap();

    let sources_path = PathBuf::from(
        env::var("OUT_DIR").unwrap()
    ).join(SOURCES_PREFIX);

    let mut compiler = cc::Build::new();

    compiler.include(&sources_path.join("src/dsfmt/"));
    compiler.include(&sources_path.join("src/pqueue/"));
    compiler.include(&sources_path.join("src/xxhash/"));
    compiler.include(&sources_path.join("src/"));

    let globs = &[format!("{}/src/**/*.c", &sources_path.to_str().unwrap())];

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

    // suppress warnings
    compiler.flag_if_supported("-Wno-unused-function");
    compiler.flag_if_supported("-Wno-unused-parameter");
    compiler.flag_if_supported("-Wno-unused-variable");
    compiler.flag_if_supported("-Wno-sign-compare");

    compiler.cpp(false);
    compiler.compile("traildb");
}

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let traildb_path = out_path.join(SOURCES_PREFIX);

    let _ = bindgen::builder()
        .header(traildb_path.join("src/traildb.h").to_str().unwrap())
        .emit_builtins()
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("ffi.rs"));
}

fn main() {
    extract_archive();
    build_lib();
    build_bindings();
}
