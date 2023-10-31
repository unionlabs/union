extern crate cbindgen;

use cbindgen::*;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::{env, fs, str};

fn style_str(style: Style) -> &'static str {
    match style {
        Style::Both => "both",
        Style::Tag => "tag",
        Style::Type => "type",
    }
}

fn run_cbindgen(
    cbindgen_path: &'static str,
    path: &Path,
    output: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
) -> Vec<u8> {
    let program = Path::new(cbindgen_path);
    let mut command = Command::new(program);
    match language {
        Language::Cxx => {}
        Language::C => {
            command.arg("--lang").arg("c");

            if cpp_compat {
                command.arg("--cpp-compat");
            }
        }
        Language::Cython => {
            command.arg("--lang").arg("cython");
        }
    }

    if let Some(style) = style {
        command.arg("--style").arg(style_str(style));
    }

    let config = path.with_extension("toml");
    if config.exists() {
        command.arg("--config").arg(config);
    }

    command.arg(path);

    println!("Running: {:?}", command);
    let cbindgen_output = command.output().expect("failed to execute process");
    assert!(
        cbindgen_output.status.success(),
        "cbindgen failed: {:?} with error: {}",
        output,
        str::from_utf8(&cbindgen_output.stderr).unwrap_or_default()
    );
    cbindgen_output.stdout
}

fn compile(
    cbindgen_output: &Path,
    tests_path: &Path,
    tmp_dir: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
) {
    let cc = match language {
        Language::Cxx => env::var("CXX").unwrap_or_else(|_| "g++".to_owned()),
        Language::C => env::var("CC").unwrap_or_else(|_| "gcc".to_owned()),
        Language::Cython => env::var("CYTHON").unwrap_or_else(|_| "cython".to_owned()),
    };

    let file_name = cbindgen_output
        .file_name()
        .expect("cbindgen output should be a file");
    let mut object = tmp_dir.join(file_name);
    object.set_extension("o");

    let mut command = Command::new(cc);
    match language {
        Language::Cxx | Language::C => {
            command.arg("-D").arg("DEFINED");
            command.arg("-I").arg(tests_path);
            command.arg("-Wall");
            if !skip_warning_as_error {
                command.arg("-Werror");
            }
            // `swift_name` is not recognzied by gcc.
            command.arg("-Wno-attributes");
            // clang warns about unused const variables.
            command.arg("-Wno-unused-const-variable");
            // clang also warns about returning non-instantiated templates (they could
            // be specialized, but they're not so it's fine).
            command.arg("-Wno-return-type-c-linkage");
            if let Language::Cxx = language {
                // enum class is a c++11 extension which makes g++ on macos 10.14 error out
                // inline variables are are a c++17 extension
                command.arg("-std=c++17");
                // Prevents warnings when compiling .c files as c++.
                command.arg("-x").arg("c++");
                if let Ok(extra_flags) = env::var("CXXFLAGS") {
                    command.args(extra_flags.split_whitespace());
                }
            } else if let Ok(extra_flags) = env::var("CFLAGS") {
                command.args(extra_flags.split_whitespace());
            }

            if let Some(style) = style {
                command.arg("-D");
                command.arg(format!(
                    "CBINDGEN_STYLE_{}",
                    style_str(style).to_uppercase()
                ));
            }

            command.arg("-o").arg(&object);
            command.arg("-c").arg(cbindgen_output);
        }
        Language::Cython => {
            command.arg("-Wextra");
            if !skip_warning_as_error {
                command.arg("-Werror");
            }
            command.arg("-3");
            command.arg("-o").arg(&object);
            command.arg(cbindgen_output);
        }
    }

    println!("Running: {:?}", command);
    let out = command.output().expect("failed to compile");
    assert!(out.status.success(), "Output failed to compile: {:?}", out);

    if object.exists() {
        fs::remove_file(object).unwrap();
    }
}

const SKIP_WARNING_AS_ERROR_SUFFIX: &str = ".skip_warning_as_error";

#[allow(clippy::too_many_arguments)]
fn run_compile_test(
    cbindgen_path: &'static str,
    name: &'static str,
    path: &Path,
    tmp_dir: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
    cbindgen_outputs: &mut HashSet<Vec<u8>>,
) {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tests_path = Path::new(&crate_dir).join("tests");
    let mut generated_file = tests_path.join("expectations");
    fs::create_dir_all(&generated_file).unwrap();

    let style_ext = style
        .map(|style| match style {
            Style::Both => ".both",
            Style::Tag => ".tag",
            Style::Type => "",
        })
        .unwrap_or_default();
    let lang_ext = match language {
        Language::Cxx => ".cpp",
        Language::C if cpp_compat => ".compat.c",
        Language::C => ".c",
        // cbindgen is supposed to generate declaration files (`.pxd`), but `cython` compiler
        // is extension-sensitive and won't work on them, so we use implementation files (`.pyx`)
        // in the test suite.
        Language::Cython => ".pyx",
    };

    let skip_warning_as_error = name.rfind(SKIP_WARNING_AS_ERROR_SUFFIX).is_some();

    let source_file =
        format!("{}{}{}", name, style_ext, lang_ext).replace(SKIP_WARNING_AS_ERROR_SUFFIX, "");

    generated_file.push(source_file);

    let cbindgen_output = run_cbindgen(
        cbindgen_path,
        path,
        &generated_file,
        language,
        cpp_compat,
        style,
    );

    if cbindgen_outputs.contains(&cbindgen_output) {
        // We already generated an identical file previously.
        if env::var_os("CBINDGEN_TEST_VERIFY").is_some() {
            assert!(!generated_file.exists());
        } else if generated_file.exists() {
            fs::remove_file(&generated_file).unwrap();
        }
    } else {
        if env::var_os("CBINDGEN_TEST_VERIFY").is_some() {
            let prev_cbindgen_output = fs::read(&generated_file).unwrap();
            assert_eq!(cbindgen_output, prev_cbindgen_output);
        } else {
            fs::write(&generated_file, &cbindgen_output).unwrap();
        }

        cbindgen_outputs.insert(cbindgen_output);

        if env::var_os("CBINDGEN_TEST_NO_COMPILE").is_some() {
            return;
        }

        compile(
            &generated_file,
            &tests_path,
            tmp_dir,
            language,
            style,
            skip_warning_as_error,
        );

        if language == Language::C && cpp_compat {
            compile(
                &generated_file,
                &tests_path,
                tmp_dir,
                Language::Cxx,
                style,
                skip_warning_as_error,
            );
        }
    }
}

fn test_file(cbindgen_path: &'static str, name: &'static str, filename: &'static str) {
    let test = Path::new(filename);
    let tmp_dir = tempfile::Builder::new()
        .prefix("cbindgen-test-output")
        .tempdir()
        .expect("Creating tmp dir failed");
    let tmp_dir = tmp_dir.path();
    // Run tests in deduplication priority order. C++ compatibility tests are run first,
    // otherwise we would lose the C++ compiler run if they were deduplicated.
    let mut cbindgen_outputs = HashSet::new();
    for cpp_compat in &[true, false] {
        for style in &[Style::Type, Style::Tag, Style::Both] {
            run_compile_test(
                cbindgen_path,
                name,
                test,
                tmp_dir,
                Language::C,
                *cpp_compat,
                Some(*style),
                &mut cbindgen_outputs,
            );
        }
    }

    run_compile_test(
        cbindgen_path,
        name,
        test,
        tmp_dir,
        Language::Cxx,
        /* cpp_compat = */ false,
        None,
        &mut HashSet::new(),
    );

    // `Style::Both` should be identical to `Style::Tag` for Cython.
    let mut cbindgen_outputs = HashSet::new();
    for style in &[Style::Type, Style::Tag] {
        run_compile_test(
            cbindgen_path,
            name,
            test,
            tmp_dir,
            Language::Cython,
            /* cpp_compat = */ false,
            Some(*style),
            &mut cbindgen_outputs,
        );
    }
}

macro_rules! test_file {
    ($cbindgen_path:expr, $test_function_name:ident, $name:expr, $file:tt) => {
        #[test]
        fn $test_function_name() {
            test_file($cbindgen_path, $name, $file);
        }
    };
}

// This file is generated by build.rs
include!(concat!(env!("OUT_DIR"), "/tests.rs"));
