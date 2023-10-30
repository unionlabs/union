fn generate_tests() {
    use std::env;
    use std::ffi::OsStr;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut dst = File::create(Path::new(&out_dir).join("tests.rs")).unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests_dir = manifest_dir.join("tests").join("rust");
    let tests = fs::read_dir(&tests_dir).unwrap();

    let entries = tests.map(|t| t.expect("Couldn't read test file"));

    println!("cargo:rerun-if-changed={}", tests_dir.display());

    // Try to make a decent guess at where our binary will end up in.
    //
    // TODO(emilio): Ideally running tests will just use the library-version of
    // cbindgen instead of the built binary.
    let cbindgen_path = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("cbindgen");

    for entry in entries {
        let path_segment = if entry.file_type().unwrap().is_file() {
            match entry.path().extension().and_then(OsStr::to_str) {
                Some("rs") => {}
                _ => continue,
            };

            entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        } else {
            entry.file_name().to_str().unwrap().to_owned()
        };

        let identifier = path_segment
            .replace(|c| !char::is_alphanumeric(c), "_")
            .replace("__", "_");

        writeln!(
            dst,
            "test_file!({:?}, test_{}, {:?}, {:?});",
            cbindgen_path,
            identifier,
            path_segment,
            entry.path(),
        )
        .unwrap();
    }

    dst.flush().unwrap();
}

fn main() {
    generate_tests();
}
