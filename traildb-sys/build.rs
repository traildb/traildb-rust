use std::env;
use std::fs::File;
use std::path::PathBuf;

use libflate::gzip::Decoder;
use tar::Archive;

const ARCHIVE_FILE: &str = "traildb-0.6.tar.gz";
const SOURCES_DIR: &str = "traildb-0.6";

fn extract_archive() {
    // Unpack the tarball
    let archive = File::open(ARCHIVE_FILE).unwrap();
    assert!(archive.metadata().unwrap().is_file());
    let gz_decoder = Decoder::new(archive).unwrap();
    let mut archive = Archive::new(gz_decoder);
    archive.unpack(".").unwrap();
}

fn build_lib() {
    let target = env::var("TARGET").unwrap();

    let sources_path = PathBuf::from(SOURCES_DIR);
    let mut compiler = cc::Build::new();
    compiler.include(&sources_path.join("src/dsfmt/"));
    compiler.include(&sources_path.join("src/pqueue/"));
    compiler.include(&sources_path.join("src/xxhash/"));
    compiler.include(&sources_path.join("src/"));

    let globs = &[format!("{}/src/**/*.c", SOURCES_DIR)];

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

fn main() {
    extract_archive();
    build_lib();
}
