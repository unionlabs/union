// Build script.
//
// This program is run by Cargo when building cranelift-codegen. It is used to generate Rust code from
// the language definitions in the cranelift-codegen/meta directory.
//
// Environment:
//
// OUT_DIR
//     Directory where generated files should be placed.
//
// TARGET
//     Target triple provided by Cargo.
//
// The build script expects to be run from the directory where this build.rs file lives. The
// current directory is used to find the sources.

use cranelift_codegen_meta as meta;

use std::env;
use std::io::Read;
use std::process;
use std::time::Instant;

fn main() {
    let start_time = Instant::now();

    let out_dir = env::var("OUT_DIR").expect("The OUT_DIR environment variable must be set");
    let target_triple = env::var("TARGET").expect("The TARGET environment variable must be set");

    let isa_targets = meta::isa::Isa::all()
        .iter()
        .cloned()
        .filter(|isa| {
            let env_key = format!("CARGO_FEATURE_{}", isa.to_string().to_uppercase());
            env::var(env_key).is_ok()
        })
        .collect::<Vec<_>>();

    let isas = if isa_targets.is_empty() {
        // Try to match native target.
        let target_name = target_triple.split('-').next().unwrap();
        let isa = meta::isa_from_arch(&target_name).expect("error when identifying target");
        println!("cargo:rustc-cfg=feature=\"{}\"", isa);
        vec![isa]
    } else {
        isa_targets
    };

    let cur_dir = env::current_dir().expect("Can't access current working directory");
    let crate_dir = cur_dir.as_path();

    println!("cargo:rerun-if-changed=build.rs");

    if let Err(err) = meta::generate(&isas, &out_dir, crate_dir) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }

    if env::var("CRANELIFT_VERBOSE").is_ok() {
        for isa in &isas {
            println!("cargo:warning=Includes support for {} ISA", isa.to_string());
        }
        println!(
            "cargo:warning=Build step took {:?}.",
            Instant::now() - start_time
        );
        println!("cargo:warning=Generated files are in {}", out_dir);
    }

    // The "Meta deterministic check" CI job runs this build script N
    // times to ensure it produces the same output
    // consistently. However, it runs the script in a fresh directory,
    // without any of the source tree present; this breaks our
    // manifest check (we need the ISLE source to be present). To keep
    // things simple, we just disable all ISLE-related logic for this
    // specific CI job.
    #[cfg(not(feature = "completely-skip-isle-for-ci-deterministic-check"))]
    {
        maybe_rebuild_isle(crate_dir).expect("Unhandled failure in ISLE rebuild");
    }

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap();
    let mut cmd = std::process::Command::new("git");
    cmd.arg("rev-parse")
        .arg("HEAD")
        .stdout(std::process::Stdio::piped())
        .current_dir(env::var("CARGO_MANIFEST_DIR").unwrap());
    let version = if let Ok(mut child) = cmd.spawn() {
        let mut git_rev = String::new();
        child
            .stdout
            .as_mut()
            .unwrap()
            .read_to_string(&mut git_rev)
            .unwrap();
        let status = child.wait().unwrap();
        if status.success() {
            let git_rev = git_rev.trim().chars().take(9).collect::<String>();
            format!("{}-{}", pkg_version, git_rev)
        } else {
            // not a git repo
            pkg_version
        }
    } else {
        // git not available
        pkg_version
    };
    std::fs::write(
        std::path::Path::new(&out_dir).join("version.rs"),
        format!(
            "/// Version number of this crate. \n\
            pub const VERSION: &str = \"{}\";",
            version
        ),
    )
    .unwrap();
}

/// Strip the current directory from the file paths, because `islec`
/// includes them in the generated source, and this helps us maintain
/// deterministic builds that don't include those local file paths.
fn make_isle_source_path_relative(
    cur_dir: &std::path::PathBuf,
    filename: std::path::PathBuf,
) -> std::path::PathBuf {
    if let Ok(suffix) = filename.strip_prefix(&cur_dir) {
        suffix.to_path_buf()
    } else {
        filename
    }
}

/// A list of compilations (transformations from ISLE source to
/// generated Rust source) that exist in the repository.
///
/// This list is used either to regenerate the Rust source in-tree (if
/// the `rebuild-isle` feature is enabled), or to verify that the ISLE
/// source in-tree corresponds to the ISLE source that was last used
/// to rebuild the Rust source (if the `rebuild-isle` feature is not
/// enabled).
#[derive(Clone, Debug)]
struct IsleCompilations {
    items: Vec<IsleCompilation>,
}

#[derive(Clone, Debug)]
struct IsleCompilation {
    output: std::path::PathBuf,
    inputs: Vec<std::path::PathBuf>,
}

impl IsleCompilation {
    /// Compute the manifest filename for the given generated Rust file.
    fn manifest_filename(&self) -> std::path::PathBuf {
        self.output.with_extension("manifest")
    }

    /// Compute the content of the source manifest for all ISLE source
    /// files that go into the compilation of one Rust file.
    ///
    /// We store this alongside the `<generated_filename>.rs` file as
    /// `<generated_filename>.manifest` and use it to verify that a
    /// rebuild was done if necessary.
    fn compute_manifest(&self) -> Result<String, Box<dyn std::error::Error + 'static>> {
        // We use the deprecated SipHasher from std::hash in order to verify
        // that ISLE sources haven't changed since the generated source was
        // last regenerated.
        //
        // We use this instead of a stronger and more usual content hash, like
        // SHA-{160,256,512}, because it's built into the standard library and
        // we don't want to pull in a separate crate. We try to keep Cranelift
        // crate dependencies as intentionally small as possible. In fact, we
        // used to use the `sha2` crate for SHA-512 and this turns out to be
        // undesirable for downstream consumers (see #3609).
        //
        // Why not the recommended replacement
        // `std::collections::hash_map::DefaultHasher`? Because we need the
        // hash to be deterministic, both between runs (i.e., not seeded with
        // random state) and across Rust versions.
        //
        // If `SipHasher` is ever actually removed from `std`, we'll need to
        // find a new option, either a very small crate or something else
        // that's built-in.
        #![allow(deprecated)]
        use std::fmt::Write;
        use std::hash::{Hasher, SipHasher};

        let mut manifest = String::new();

        for filename in &self.inputs {
            // Our source must be valid UTF-8 for this to work, else user
            // will get an error on build. This is not expected to be an
            // issue.
            let content = std::fs::read_to_string(filename)?;
            // On Windows, source is checked out with line-endings changed
            // to `\r\n`; canonicalize the source that we hash to
            // Unix-style (`\n`) so hashes will match.
            let content = content.replace("\r\n", "\n");
            // One line in the manifest: <filename> <siphash>.
            let mut hasher = SipHasher::new_with_keys(0, 0); // fixed keys for determinism
            hasher.write(content.as_bytes());
            let filename = format!("{}", filename.display()).replace("\\", "/");
            writeln!(&mut manifest, "{} {:x}", filename, hasher.finish())?;
        }

        Ok(manifest)
    }
}

/// Construct the list of compilations (transformations from ISLE
/// source to generated Rust source) that exist in the repository.
fn get_isle_compilations(crate_dir: &std::path::Path) -> Result<IsleCompilations, std::io::Error> {
    let cur_dir = std::env::current_dir()?;

    let clif_isle =
        make_isle_source_path_relative(&cur_dir, crate_dir.join("src").join("clif.isle"));
    let prelude_isle =
        make_isle_source_path_relative(&cur_dir, crate_dir.join("src").join("prelude.isle"));
    let src_isa_x64 =
        make_isle_source_path_relative(&cur_dir, crate_dir.join("src").join("isa").join("x64"));
    let src_isa_aarch64 =
        make_isle_source_path_relative(&cur_dir, crate_dir.join("src").join("isa").join("aarch64"));
    let src_isa_s390x =
        make_isle_source_path_relative(&cur_dir, crate_dir.join("src").join("isa").join("s390x"));

    // This is a set of ISLE compilation units.
    //
    // The format of each entry is:
    //
    //     (output Rust code file, input ISLE source files)
    //
    // There should be one entry for each backend that uses ISLE for lowering,
    // and if/when we replace our peephole optimization passes with ISLE, there
    // should be an entry for each of those as well.
    Ok(IsleCompilations {
        items: vec![
            // The x86-64 instruction selector.
            IsleCompilation {
                output: src_isa_x64
                    .join("lower")
                    .join("isle")
                    .join("generated_code.rs"),
                inputs: vec![
                    clif_isle.clone(),
                    prelude_isle.clone(),
                    src_isa_x64.join("inst.isle"),
                    src_isa_x64.join("lower.isle"),
                ],
            },
            // The aarch64 instruction selector.
            IsleCompilation {
                output: src_isa_aarch64
                    .join("lower")
                    .join("isle")
                    .join("generated_code.rs"),
                inputs: vec![
                    clif_isle.clone(),
                    prelude_isle.clone(),
                    src_isa_aarch64.join("inst.isle"),
                    src_isa_aarch64.join("lower.isle"),
                ],
            },
            // The s390x instruction selector.
            IsleCompilation {
                output: src_isa_s390x
                    .join("lower")
                    .join("isle")
                    .join("generated_code.rs"),
                inputs: vec![
                    clif_isle.clone(),
                    prelude_isle.clone(),
                    src_isa_s390x.join("inst.isle"),
                    src_isa_s390x.join("lower.isle"),
                ],
            },
        ],
    })
}

/// Check the manifest for the ISLE generated code, which documents
/// what ISLE source went into generating the Rust, and if there is a
/// mismatch, either invoke the ISLE compiler (if we have the
/// `rebuild-isle` feature) or exit with an error (if not).
///
/// We do this by computing a hash of the ISLE source and checking it
/// against a "manifest" that is also checked into git, alongside the
/// generated Rust.
///
/// (Why not include the `rebuild-isle` feature by default? Because
/// the build process must not modify the checked-in source by
/// default; any checked-in source is a human-managed bit of data, and
/// we can only act as an agent of the human developer when explicitly
/// requested to do so. This manifest check is a middle ground that
/// ensures this explicit control while also avoiding the easy footgun
/// of "I changed the ISLE, why isn't the compiler updated?!".)
fn maybe_rebuild_isle(
    crate_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let isle_compilations = get_isle_compilations(crate_dir)?;
    let mut rebuild_compilations = vec![];

    for compilation in &isle_compilations.items {
        for file in &compilation.inputs {
            println!("cargo:rerun-if-changed={}", file.display());
        }

        let manifest =
            std::fs::read_to_string(compilation.manifest_filename()).unwrap_or(String::new());
        // Canonicalize Windows line-endings into Unix line-endings in
        // the manifest text itself.
        let manifest = manifest.replace("\r\n", "\n");
        let expected_manifest = compilation.compute_manifest()?.replace("\r\n", "\n");
        if manifest != expected_manifest {
            rebuild_compilations.push((compilation, expected_manifest));
        }
    }

    #[cfg(feature = "rebuild-isle")]
    {
        if !rebuild_compilations.is_empty() {
            set_miette_hook();
        }
        let mut had_error = false;
        for (compilation, expected_manifest) in rebuild_compilations {
            if let Err(e) = rebuild_isle(compilation, &expected_manifest) {
                eprintln!("Error building ISLE files: {:?}", e);
                let mut source = e.source();
                while let Some(e) = source {
                    eprintln!("{:?}", e);
                    source = e.source();
                }
                had_error = true;
            }
        }

        if had_error {
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "rebuild-isle"))]
    {
        if !rebuild_compilations.is_empty() {
            for (compilation, _) in rebuild_compilations {
                eprintln!("");
                eprintln!(
                    "Error: the ISLE source files that resulted in the generated Rust source"
                );
                eprintln!("");
                eprintln!("      * {}", compilation.output.display());
                eprintln!("");
                eprintln!(
                    "have changed but the generated source was not rebuilt! These ISLE source"
                );
                eprintln!("files are:");
                eprintln!("");
                for file in &compilation.inputs {
                    eprintln!("       * {}", file.display());
                }
            }

            eprintln!("");
            eprintln!("Please add `--features rebuild-isle` to your `cargo build` command");
            eprintln!("if you wish to rebuild the generated source, then include these changes");
            eprintln!("in any git commits you make that include the changes to the ISLE.");
            eprintln!("");
            eprintln!("For example:");
            eprintln!("");
            eprintln!("  $ cargo build -p cranelift-codegen --features rebuild-isle");
            eprintln!("");
            eprintln!("(This build script cannot do this for you by default because we cannot");
            eprintln!("modify checked-into-git source without your explicit opt-in.)");
            eprintln!("");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(feature = "rebuild-isle")]
fn set_miette_hook() {
    use std::sync::Once;
    static SET_MIETTE_HOOK: Once = Once::new();
    SET_MIETTE_HOOK.call_once(|| {
        let _ = miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    // This is necessary for `miette` to properly display errors
                    // until https://github.com/zkat/miette/issues/93 is fixed.
                    .force_graphical(true)
                    .build(),
            )
        }));
    });
}

/// Rebuild ISLE DSL source text into generated Rust code.
///
/// NB: This must happen *after* the `cranelift-codegen-meta` functions, since
/// it consumes files generated by them.
#[cfg(feature = "rebuild-isle")]
fn rebuild_isle(
    compilation: &IsleCompilation,
    manifest: &str,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    use cranelift_isle as isle;

    println!("Rebuilding {}", compilation.output.display());

    // First, remove the manifest, if any; we will recreate it
    // below if the compilation is successful. Ignore error if no
    // manifest was present.
    let manifest_filename = compilation.manifest_filename();
    let _ = std::fs::remove_file(&manifest_filename);

    let code = (|| {
        let lexer = isle::lexer::Lexer::from_files(&compilation.inputs[..])?;
        let defs = isle::parser::parse(lexer)?;
        isle::compile::compile(&defs)
    })()
    .map_err(|e| {
        // Make sure to include the source snippets location info along with
        // the error messages.

        let report = miette::Report::new(e);
        return DebugReport(report);

        struct DebugReport(miette::Report);

        impl std::fmt::Display for DebugReport {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.handler().debug(&*self.0, f)
            }
        }

        impl std::fmt::Debug for DebugReport {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }

        impl std::error::Error for DebugReport {}
    })?;

    let code = rustfmt(&code).unwrap_or_else(|e| {
        println!(
            "cargo:warning=Failed to run `rustfmt` on ISLE-generated code: {:?}",
            e
        );
        code
    });

    println!(
        "Writing ISLE-generated Rust code to {}",
        compilation.output.display()
    );
    std::fs::write(&compilation.output, code)?;

    // Write the manifest so that, in the default build configuration
    // without the `rebuild-isle` feature, we can at least verify that
    // no changes were made that will not be picked up. Note that we
    // only write this *after* we write the source above, so no
    // manifest is produced if there was an error.
    std::fs::write(&manifest_filename, manifest)?;

    return Ok(());

    fn rustfmt(code: &str) -> std::io::Result<String> {
        use std::io::Write;

        let mut rustfmt = std::process::Command::new("rustfmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let mut stdin = rustfmt.stdin.take().unwrap();
        stdin.write_all(code.as_bytes())?;
        drop(stdin);

        let mut stdout = rustfmt.stdout.take().unwrap();
        let mut data = vec![];
        stdout.read_to_end(&mut data)?;

        let status = rustfmt.wait()?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("`rustfmt` exited with status {}", status),
            ));
        }

        Ok(String::from_utf8(data).expect("rustfmt always writs utf-8 to stdout"))
    }
}
