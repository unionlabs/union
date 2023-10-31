use std::env;
use std::path::Path;

/// Tells whether we're building for Windows. This is more suitable than a plain
/// `cfg!(windows)`, since the latter does not properly handle cross-compilation
///
/// Note that there is no way to know at compile-time which system we'll be
/// targetting, and this test must be made at run-time (of the build script) See
/// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
fn win_target() -> bool {
    env::var("CARGO_CFG_WINDOWS").is_ok()
}

/// Tells whether we're building for Android.
/// See [`win_target`]
#[cfg(any(feature = "bundled", feature = "bundled-windows"))]
fn android_target() -> bool {
    env::var("CARGO_CFG_TARGET_OS").map_or(false, |v| v == "android")
}

/// Tells whether a given compiler will be used `compiler_name` is compared to
/// the content of `CARGO_CFG_TARGET_ENV` (and is always lowercase)
///
/// See [`win_target`]
fn is_compiler(compiler_name: &str) -> bool {
    env::var("CARGO_CFG_TARGET_ENV").map_or(false, |v| v == compiler_name)
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("bindgen.rs");
    if cfg!(feature = "in_gecko") {
        // When inside mozilla-central, we are included into the build with
        // sqlite3.o directly, so we don't want to provide any linker arguments.
        std::fs::copy("sqlite3/bindgen_bundled_version.rs", out_path)
            .expect("Could not copy bindings to output directory");
        return;
    }
    if cfg!(all(
        feature = "sqlcipher",
        not(feature = "bundled-sqlcipher")
    )) {
        if cfg!(feature = "bundled") || (win_target() && cfg!(feature = "bundled-windows")) {
            println!(
                "cargo:warning=For backwards compatibility, feature 'sqlcipher' overrides
                features 'bundled' and 'bundled-windows'. If you want a bundled build of
                SQLCipher (available for the moment only on Unix), use feature 'bundled-sqlcipher'
                or 'bundled-sqlcipher-vendored-openssl' to also bundle OpenSSL crypto."
            );
        }
        build_linked::main(&out_dir, &out_path);
    } else if cfg!(feature = "bundled")
        || (win_target() && cfg!(feature = "bundled-windows"))
        || cfg!(feature = "bundled-sqlcipher")
    {
        #[cfg(any(
            feature = "bundled",
            feature = "bundled-windows",
            feature = "bundled-sqlcipher"
        ))]
        build_bundled::main(&out_dir, &out_path);
        #[cfg(not(any(
            feature = "bundled",
            feature = "bundled-windows",
            feature = "bundled-sqlcipher"
        )))]
        panic!("The runtime test should not run this branch, which has not compiled any logic.")
    } else {
        build_linked::main(&out_dir, &out_path);
    }
}

#[cfg(any(
    feature = "bundled",
    feature = "bundled-windows",
    feature = "bundled-sqlcipher"
))]
mod build_bundled {
    use std::env;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{is_compiler, win_target};

    pub fn main(out_dir: &str, out_path: &Path) {
        let lib_name = super::lib_name();

        // This is just a sanity check, the top level `main` should ensure this.
        assert!(!(cfg!(feature = "bundled-windows") && !cfg!(feature = "bundled") && !win_target()),
            "This module should not be used: we're not on Windows and the bundled feature has not been enabled");

        #[cfg(feature = "buildtime_bindgen")]
        {
            use super::{bindings, HeaderLocation};
            let header = HeaderLocation::FromPath(format!("{}/sqlite3.h", lib_name));
            bindings::write_to_out_dir(header, out_path);
        }
        #[cfg(not(feature = "buildtime_bindgen"))]
        {
            use std::fs;
            fs::copy(format!("{lib_name}/bindgen_bundled_version.rs"), out_path)
                .expect("Could not copy bindings to output directory");
        }
        // println!("cargo:rerun-if-changed=sqlite3/sqlite3.c");
        // println!("cargo:rerun-if-changed=sqlcipher/sqlite3.c");
        println!("cargo:rerun-if-changed={lib_name}/sqlite3.c");
        println!("cargo:rerun-if-changed=sqlite3/wasm32-wasi-vfs.c");
        let mut cfg = cc::Build::new();
        cfg.file(format!("{lib_name}/sqlite3.c"))
            .flag("-DSQLITE_CORE")
            .flag("-DSQLITE_DEFAULT_FOREIGN_KEYS=1")
            .flag("-DSQLITE_ENABLE_API_ARMOR")
            .flag("-DSQLITE_ENABLE_COLUMN_METADATA")
            .flag("-DSQLITE_ENABLE_DBSTAT_VTAB")
            .flag("-DSQLITE_ENABLE_FTS3")
            .flag("-DSQLITE_ENABLE_FTS3_PARENTHESIS")
            .flag("-DSQLITE_ENABLE_FTS5")
            .flag("-DSQLITE_ENABLE_JSON1")
            .flag("-DSQLITE_ENABLE_LOAD_EXTENSION=1")
            .flag("-DSQLITE_ENABLE_MEMORY_MANAGEMENT")
            .flag("-DSQLITE_ENABLE_RTREE")
            .flag("-DSQLITE_ENABLE_STAT2")
            .flag("-DSQLITE_ENABLE_STAT4")
            .flag("-DSQLITE_SOUNDEX")
            .flag("-DSQLITE_THREADSAFE=1")
            .flag("-DSQLITE_USE_URI")
            .flag("-DHAVE_USLEEP=1")
            .flag("-D_POSIX_THREAD_SAFE_FUNCTIONS") // cross compile with MinGW
            .warnings(false);

        if cfg!(feature = "bundled-sqlcipher") {
            cfg.flag("-DSQLITE_HAS_CODEC").flag("-DSQLITE_TEMP_STORE=2");

            let target = env::var("TARGET").unwrap();
            let host = env::var("HOST").unwrap();

            let is_windows = host.contains("windows") && target.contains("windows");
            let is_apple = host.contains("apple") && target.contains("apple");

            let lib_dir = env("OPENSSL_LIB_DIR").map(PathBuf::from);
            let inc_dir = env("OPENSSL_INCLUDE_DIR").map(PathBuf::from);
            let mut use_openssl = false;

            let (lib_dir, inc_dir) = match (lib_dir, inc_dir) {
                (Some(lib_dir), Some(inc_dir)) => {
                    use_openssl = true;
                    (lib_dir, inc_dir)
                }
                (lib_dir, inc_dir) => match find_openssl_dir(&host, &target) {
                    None => {
                        if is_windows && !cfg!(feature = "bundled-sqlcipher-vendored-openssl") {
                            panic!("Missing environment variable OPENSSL_DIR or OPENSSL_DIR is not set")
                        } else {
                            (PathBuf::new(), PathBuf::new())
                        }
                    }
                    Some(openssl_dir) => {
                        let lib_dir = lib_dir.unwrap_or_else(|| openssl_dir.join("lib"));
                        let inc_dir = inc_dir.unwrap_or_else(|| openssl_dir.join("include"));

                        assert!(
                            Path::new(&lib_dir).exists(),
                            "OpenSSL library directory does not exist: {}",
                            lib_dir.to_string_lossy()
                        );

                        if !Path::new(&inc_dir).exists() {
                            panic!(
                                "OpenSSL include directory does not exist: {}",
                                inc_dir.to_string_lossy()
                            );
                        }

                        use_openssl = true;
                        (lib_dir, inc_dir)
                    }
                },
            };

            if cfg!(feature = "bundled-sqlcipher-vendored-openssl") {
                cfg.include(env::var("DEP_OPENSSL_INCLUDE").unwrap());
                // cargo will resolve downstream to the static lib in
                // openssl-sys
            } else if use_openssl {
                cfg.include(inc_dir.to_string_lossy().as_ref());
                let lib_name = if is_windows { "libcrypto" } else { "crypto" };
                println!("cargo:rustc-link-lib=dylib={}", lib_name);
                println!("cargo:rustc-link-search={}", lib_dir.to_string_lossy());
            } else if is_apple {
                cfg.flag("-DSQLCIPHER_CRYPTO_CC");
                println!("cargo:rustc-link-lib=framework=Security");
                println!("cargo:rustc-link-lib=framework=CoreFoundation");
            } else {
                // branch not taken on Windows, just `crypto` is fine.
                println!("cargo:rustc-link-lib=dylib=crypto");
            }
        }

        // on android sqlite can't figure out where to put the temp files.
        // the bundled sqlite on android also uses `SQLITE_TEMP_STORE=3`.
        // https://android.googlesource.com/platform/external/sqlite/+/2c8c9ae3b7e6f340a19a0001c2a889a211c9d8b2/dist/Android.mk
        if super::android_target() {
            cfg.flag("-DSQLITE_TEMP_STORE=3");
        }

        if cfg!(feature = "with-asan") {
            cfg.flag("-fsanitize=address");
        }

        // If explicitly requested: enable static linking against the Microsoft Visual
        // C++ Runtime to avoid dependencies on vcruntime140.dll and similar libraries.
        if cfg!(target_feature = "crt-static") && is_compiler("msvc") {
            cfg.static_crt(true);
        }

        // Older versions of visual studio don't support c99 (including isnan), which
        // causes a build failure when the linker fails to find the `isnan`
        // function. `sqlite` provides its own implementation, using the fact
        // that x != x when x is NaN.
        //
        // There may be other platforms that don't support `isnan`, they should be
        // tested for here.
        if is_compiler("msvc") {
            use cc::windows_registry::{find_vs_version, VsVers};
            let vs_has_nan = match find_vs_version() {
                Ok(ver) => ver != VsVers::Vs12,
                Err(_msg) => false,
            };
            if vs_has_nan {
                cfg.flag("-DHAVE_ISNAN");
            }
        } else {
            cfg.flag("-DHAVE_ISNAN");
        }
        if !win_target() {
            cfg.flag("-DHAVE_LOCALTIME_R");
        }
        // Target wasm32-wasi can't compile the default VFS
        if is_compiler("wasm32-wasi") {
            cfg.flag("-DSQLITE_OS_OTHER")
                // https://github.com/rust-lang/rust/issues/74393
                .flag("-DLONGDOUBLE_TYPE=double");
            if cfg!(feature = "wasm32-wasi-vfs") {
                cfg.file("sqlite3/wasm32-wasi-vfs.c");
            }
        }
        if cfg!(feature = "unlock_notify") {
            cfg.flag("-DSQLITE_ENABLE_UNLOCK_NOTIFY");
        }
        if cfg!(feature = "preupdate_hook") {
            cfg.flag("-DSQLITE_ENABLE_PREUPDATE_HOOK");
        }
        if cfg!(feature = "session") {
            cfg.flag("-DSQLITE_ENABLE_SESSION");
        }

        if let Ok(limit) = env::var("SQLITE_MAX_VARIABLE_NUMBER") {
            cfg.flag(&format!("-DSQLITE_MAX_VARIABLE_NUMBER={limit}"));
        }
        println!("cargo:rerun-if-env-changed=SQLITE_MAX_VARIABLE_NUMBER");

        if let Ok(limit) = env::var("SQLITE_MAX_EXPR_DEPTH") {
            cfg.flag(&format!("-DSQLITE_MAX_EXPR_DEPTH={limit}"));
        }
        println!("cargo:rerun-if-env-changed=SQLITE_MAX_EXPR_DEPTH");

        if let Ok(extras) = env::var("LIBSQLITE3_FLAGS") {
            for extra in extras.split_whitespace() {
                if extra.starts_with("-D") || extra.starts_with("-U") {
                    cfg.flag(extra);
                } else if extra.starts_with("SQLITE_") {
                    cfg.flag(&format!("-D{extra}"));
                } else {
                    panic!("Don't understand {} in LIBSQLITE3_FLAGS", extra);
                }
            }
        }
        println!("cargo:rerun-if-env-changed=LIBSQLITE3_FLAGS");

        cfg.compile(lib_name);

        println!("cargo:lib_dir={out_dir}");
    }

    fn env(name: &str) -> Option<OsString> {
        let prefix = env::var("TARGET").unwrap().to_uppercase().replace('-', "_");
        let prefixed = format!("{prefix}_{name}");
        let var = env::var_os(prefixed);

        match var {
            None => env::var_os(name),
            _ => var,
        }
    }

    fn find_openssl_dir(_host: &str, _target: &str) -> Option<PathBuf> {
        let openssl_dir = env("OPENSSL_DIR");
        openssl_dir.map(PathBuf::from)
    }
}

fn env_prefix() -> &'static str {
    if cfg!(any(feature = "sqlcipher", feature = "bundled-sqlcipher")) {
        "SQLCIPHER"
    } else {
        "SQLITE3"
    }
}

fn lib_name() -> &'static str {
    if cfg!(any(feature = "sqlcipher", feature = "bundled-sqlcipher")) {
        "sqlcipher"
    } else if cfg!(all(windows, feature = "winsqlite3")) {
        "winsqlite3"
    } else {
        "sqlite3"
    }
}

pub enum HeaderLocation {
    FromEnvironment,
    Wrapper,
    FromPath(String),
}

impl From<HeaderLocation> for String {
    fn from(header: HeaderLocation) -> String {
        match header {
            HeaderLocation::FromEnvironment => {
                let prefix = env_prefix();
                let mut header = env::var(format!("{prefix}_INCLUDE_DIR")).unwrap_or_else(|_| {
                    panic!(
                        "{}_INCLUDE_DIR must be set if {}_LIB_DIR is set",
                        prefix, prefix
                    )
                });
                header.push_str("/sqlite3.h");
                header
            }
            HeaderLocation::Wrapper => "wrapper.h".into(),
            HeaderLocation::FromPath(path) => path,
        }
    }
}

mod build_linked {
    #[cfg(feature = "vcpkg")]
    extern crate vcpkg;

    use super::{bindings, env_prefix, is_compiler, lib_name, win_target, HeaderLocation};
    use std::env;
    use std::path::Path;

    pub fn main(_out_dir: &str, out_path: &Path) {
        let header = find_sqlite();
        if (cfg!(any(
            feature = "bundled_bindings",
            feature = "bundled",
            feature = "bundled-sqlcipher"
        )) || (win_target() && cfg!(feature = "bundled-windows")))
            && !cfg!(feature = "buildtime_bindgen")
        {
            // Generally means the `bundled_bindings` feature is enabled.
            // Most users are better off with turning
            // on buildtime_bindgen instead, but this is still supported as we
            // have runtime version checks and there are good reasons to not
            // want to run bindgen.
            std::fs::copy(
                format!("{}/bindgen_bundled_version.rs", lib_name()),
                out_path,
            )
            .expect("Could not copy bindings to output directory");
        } else {
            bindings::write_to_out_dir(header, out_path);
        }
    }

    fn find_link_mode() -> &'static str {
        // If the user specifies SQLITE3_STATIC (or SQLCIPHER_STATIC), do static
        // linking, unless it's explicitly set to 0.
        match &env::var(format!("{}_STATIC", env_prefix())) {
            Ok(v) if v != "0" => "static",
            _ => "dylib",
        }
    }
    // Prints the necessary cargo link commands and returns the path to the header.
    fn find_sqlite() -> HeaderLocation {
        let link_lib = lib_name();

        println!("cargo:rerun-if-env-changed={}_INCLUDE_DIR", env_prefix());
        println!("cargo:rerun-if-env-changed={}_LIB_DIR", env_prefix());
        println!("cargo:rerun-if-env-changed={}_STATIC", env_prefix());
        if cfg!(feature = "vcpkg") && is_compiler("msvc") {
            println!("cargo:rerun-if-env-changed=VCPKGRS_DYNAMIC");
        }

        // dependents can access `DEP_SQLITE3_LINK_TARGET` (`sqlite3` being the
        // `links=` value in our Cargo.toml) to get this value. This might be
        // useful if you need to ensure whatever crypto library sqlcipher relies
        // on is available, for example.
        println!("cargo:link-target={link_lib}");

        if win_target() && cfg!(feature = "winsqlite3") {
            println!("cargo:rustc-link-lib=dylib={link_lib}");
            return HeaderLocation::Wrapper;
        }

        // Allow users to specify where to find SQLite.
        if let Ok(dir) = env::var(format!("{}_LIB_DIR", env_prefix())) {
            // Try to use pkg-config to determine link commands
            let pkgconfig_path = Path::new(&dir).join("pkgconfig");
            env::set_var("PKG_CONFIG_PATH", pkgconfig_path);
            if pkg_config::Config::new().probe(link_lib).is_err() {
                // Otherwise just emit the bare minimum link commands.
                println!("cargo:rustc-link-lib={}={link_lib}", find_link_mode());
                println!("cargo:rustc-link-search={dir}");
            }
            return HeaderLocation::FromEnvironment;
        }

        if let Some(header) = try_vcpkg() {
            return header;
        }

        // See if pkg-config can do everything for us.
        if let Ok(mut lib) = pkg_config::Config::new()
            .print_system_libs(false)
            .probe(link_lib)
        {
            if let Some(mut header) = lib.include_paths.pop() {
                header.push("sqlite3.h");
                HeaderLocation::FromPath(header.to_string_lossy().into())
            } else {
                HeaderLocation::Wrapper
            }
        } else {
            // No env var set and pkg-config couldn't help; just output the link-lib
            // request and hope that the library exists on the system paths. We used to
            // output /usr/lib explicitly, but that can introduce other linking problems;
            // see https://github.com/rusqlite/rusqlite/issues/207.
            println!("cargo:rustc-link-lib={}={link_lib}", find_link_mode());
            HeaderLocation::Wrapper
        }
    }

    fn try_vcpkg() -> Option<HeaderLocation> {
        if cfg!(feature = "vcpkg") && is_compiler("msvc") {
            // See if vcpkg can find it.
            if let Ok(mut lib) = vcpkg::Config::new().probe(lib_name()) {
                if let Some(mut header) = lib.include_paths.pop() {
                    header.push("sqlite3.h");
                    return Some(HeaderLocation::FromPath(header.to_string_lossy().into()));
                }
            }
            None
        } else {
            None
        }
    }
}

#[cfg(not(feature = "buildtime_bindgen"))]
mod bindings {
    #![allow(dead_code)]
    use super::HeaderLocation;

    use std::fs;
    use std::path::Path;

    static PREBUILT_BINDGEN_PATHS: &[&str] = &["bindgen-bindings/bindgen_3.14.0.rs"];

    pub fn write_to_out_dir(_header: HeaderLocation, out_path: &Path) {
        let in_path = PREBUILT_BINDGEN_PATHS[PREBUILT_BINDGEN_PATHS.len() - 1];
        fs::copy(in_path, out_path).expect("Could not copy bindings to output directory");
    }
}

#[cfg(feature = "buildtime_bindgen")]
mod bindings {
    use super::HeaderLocation;
    use bindgen::callbacks::{IntKind, ParseCallbacks};

    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::Path;

    use super::win_target;

    #[derive(Debug)]
    struct SqliteTypeChooser;

    impl ParseCallbacks for SqliteTypeChooser {
        fn int_macro(&self, _name: &str, value: i64) -> Option<IntKind> {
            if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                Some(IntKind::I32)
            } else {
                None
            }
        }
        fn item_name(&self, original_item_name: &str) -> Option<String> {
            original_item_name
                .strip_prefix("sqlite3_index_info_")
                .map(|s| s.to_owned())
        }
    }

    // Are we generating the bundled bindings? Used to avoid emitting things
    // that would be problematic in bundled builds. This env var is set by
    // `upgrade.sh`.
    fn generating_bundled_bindings() -> bool {
        // Hacky way to know if we're generating the bundled bindings
        println!("cargo:rerun-if-env-changed=LIBSQLITE3_SYS_BUNDLING");
        match std::env::var("LIBSQLITE3_SYS_BUNDLING") {
            Ok(v) => v != "0",
            Err(_) => false,
        }
    }

    pub fn write_to_out_dir(header: HeaderLocation, out_path: &Path) {
        let header: String = header.into();
        let mut output = Vec::new();
        let mut bindings = bindgen::builder()
            .trust_clang_mangling(false)
            .header(header.clone())
            .parse_callbacks(Box::new(SqliteTypeChooser))
            .rustfmt_bindings(true);

        if cfg!(any(feature = "sqlcipher", feature = "bundled-sqlcipher")) {
            bindings = bindings.clang_arg("-DSQLITE_HAS_CODEC");
        }
        if cfg!(feature = "unlock_notify") {
            bindings = bindings.clang_arg("-DSQLITE_ENABLE_UNLOCK_NOTIFY");
        }
        if cfg!(feature = "preupdate_hook") {
            bindings = bindings.clang_arg("-DSQLITE_ENABLE_PREUPDATE_HOOK");
        }
        if cfg!(feature = "session") {
            bindings = bindings.clang_arg("-DSQLITE_ENABLE_SESSION");
        }
        if win_target() && cfg!(feature = "winsqlite3") {
            bindings = bindings
                .clang_arg("-DBINDGEN_USE_WINSQLITE3")
                .blocklist_item("NTDDI_.+")
                .blocklist_item("WINAPI_FAMILY.*")
                .blocklist_item("_WIN32_.+")
                .blocklist_item("_VCRT_COMPILER_PREPROCESSOR")
                .blocklist_item("_SAL_VERSION")
                .blocklist_item("__SAL_H_VERSION")
                .blocklist_item("_USE_DECLSPECS_FOR_SAL")
                .blocklist_item("_USE_ATTRIBUTES_FOR_SAL")
                .blocklist_item("_CRT_PACKING")
                .blocklist_item("_HAS_EXCEPTIONS")
                .blocklist_item("_STL_LANG")
                .blocklist_item("_HAS_CXX17")
                .blocklist_item("_HAS_CXX20")
                .blocklist_item("_HAS_NODISCARD")
                .blocklist_item("WDK_NTDDI_VERSION")
                .blocklist_item("OSVERSION_MASK")
                .blocklist_item("SPVERSION_MASK")
                .blocklist_item("SUBVERSION_MASK")
                .blocklist_item("WINVER")
                .blocklist_item("__security_cookie")
                .blocklist_type("size_t")
                .blocklist_type("__vcrt_bool")
                .blocklist_type("wchar_t")
                .blocklist_function("__security_init_cookie")
                .blocklist_function("__report_gsfailure")
                .blocklist_function("__va_start");
        }

        // When cross compiling unless effort is taken to fix the issue, bindgen
        // will find the wrong headers. There's only one header included by the
        // amalgamated `sqlite.h`: `stdarg.h`.
        //
        // Thankfully, there's almost no case where rust code needs to use
        // functions taking `va_list` (It's nearly impossible to get a `va_list`
        // in Rust unless you get passed it by C code for some reason).
        //
        // Arguably, we should never be including these, but we include them for
        // the cases where they aren't totally broken...
        let target_arch = std::env::var("TARGET").unwrap();
        let host_arch = std::env::var("HOST").unwrap();
        let is_cross_compiling = target_arch != host_arch;

        // Note that when generating the bundled file, we're essentially always
        // cross compiling.
        if generating_bundled_bindings() || is_cross_compiling {
            // Get rid of va_list, as it's not
            bindings = bindings
                .blocklist_function("sqlite3_vmprintf")
                .blocklist_function("sqlite3_vsnprintf")
                .blocklist_function("sqlite3_str_vappendf")
                .blocklist_type("va_list")
                .blocklist_type("__builtin_va_list")
                .blocklist_type("__gnuc_va_list")
                .blocklist_type("__va_list_tag")
                .blocklist_item("__GNUC_VA_LIST");
        }

        bindings
            .layout_tests(false)
            .generate()
            .unwrap_or_else(|_| panic!("could not run bindgen on header {}", header))
            .write(Box::new(&mut output))
            .expect("could not write output of bindgen");
        let mut output = String::from_utf8(output).expect("bindgen output was not UTF-8?!");

        // rusqlite's functions feature ors in the SQLITE_DETERMINISTIC flag when it
        // can. This flag was added in SQLite 3.8.3, but oring it in in prior
        // versions of SQLite is harmless. We don't want to not build just
        // because this flag is missing (e.g., if we're linking against
        // SQLite 3.7.x), so append the flag manually if it isn't present in bindgen's
        // output.
        if !output.contains("pub const SQLITE_DETERMINISTIC") {
            output.push_str("\npub const SQLITE_DETERMINISTIC: i32 = 2048;\n");
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(out_path)
            .unwrap_or_else(|_| panic!("Could not write to {:?}", out_path));

        file.write_all(output.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write to {:?}", out_path));
    }
}
