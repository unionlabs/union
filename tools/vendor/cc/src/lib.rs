//! A library for build scripts to compile custom C code
//!
//! This library is intended to be used as a `build-dependencies` entry in
//! `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! cc = "1.0"
//! ```
//!
//! The purpose of this crate is to provide the utility functions necessary to
//! compile C code into a static archive which is then linked into a Rust crate.
//! Configuration is available through the `Build` struct.
//!
//! This crate will automatically detect situations such as cross compilation or
//! other environment variables set by Cargo and will build code appropriately.
//!
//! The crate is not limited to C code, it can accept any source code that can
//! be passed to a C or C++ compiler. As such, assembly files with extensions
//! `.s` (gcc/clang) and `.asm` (MSVC) can also be compiled.
//!
//! [`Build`]: struct.Build.html
//!
//! # Parallelism
//!
//! To parallelize computation, enable the `parallel` feature for the crate.
//!
//! ```toml
//! [build-dependencies]
//! cc = { version = "1.0", features = ["parallel"] }
//! ```
//! To specify the max number of concurrent compilation jobs, set the `NUM_JOBS`
//! environment variable to the desired amount.
//!
//! Cargo will also set this environment variable when executed with the `-jN` flag.
//!
//! If `NUM_JOBS` is not set, the `RAYON_NUM_THREADS` environment variable can
//! also specify the build parallelism.
//!
//! # Examples
//!
//! Use the `Build` struct to compile `src/foo.c`:
//!
//! ```no_run
//! fn main() {
//!     cc::Build::new()
//!         .file("src/foo.c")
//!         .define("FOO", Some("bar"))
//!         .include("src")
//!         .compile("foo");
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/cc/1.0")]
#![cfg_attr(test, deny(warnings))]
#![allow(deprecated)]
#![deny(missing_docs)]

use std::collections::{hash_map, HashMap};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::hash::Hasher;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

// These modules are all glue to support reading the MSVC version from
// the registry and from COM interfaces
#[cfg(windows)]
mod registry;
#[cfg(windows)]
#[macro_use]
mod winapi;
#[cfg(windows)]
mod com;
#[cfg(windows)]
mod setup_config;
#[cfg(windows)]
mod vs_instances;

pub mod windows_registry;

/// A builder for compilation of a native library.
///
/// A `Build` is the main type of the `cc` crate and is used to control all the
/// various configuration options and such of a compile. You'll find more
/// documentation on each method itself.
#[derive(Clone, Debug)]
pub struct Build {
    include_directories: Vec<PathBuf>,
    definitions: Vec<(String, Option<String>)>,
    objects: Vec<PathBuf>,
    flags: Vec<String>,
    flags_supported: Vec<String>,
    known_flag_support_status: Arc<Mutex<HashMap<String, bool>>>,
    ar_flags: Vec<String>,
    asm_flags: Vec<String>,
    no_default_flags: bool,
    files: Vec<PathBuf>,
    cpp: bool,
    cpp_link_stdlib: Option<Option<String>>,
    cpp_set_stdlib: Option<String>,
    cuda: bool,
    cudart: Option<String>,
    target: Option<String>,
    host: Option<String>,
    out_dir: Option<PathBuf>,
    opt_level: Option<String>,
    debug: Option<bool>,
    force_frame_pointer: Option<bool>,
    env: Vec<(OsString, OsString)>,
    compiler: Option<PathBuf>,
    archiver: Option<PathBuf>,
    ranlib: Option<PathBuf>,
    cargo_metadata: bool,
    link_lib_modifiers: Vec<String>,
    pic: Option<bool>,
    use_plt: Option<bool>,
    static_crt: Option<bool>,
    shared_flag: Option<bool>,
    static_flag: Option<bool>,
    warnings_into_errors: bool,
    warnings: Option<bool>,
    extra_warnings: Option<bool>,
    env_cache: Arc<Mutex<HashMap<String, Option<String>>>>,
    apple_sdk_root_cache: Arc<Mutex<HashMap<String, OsString>>>,
    emit_rerun_if_env_changed: bool,
}

/// Represents the types of errors that may occur while using cc-rs.
#[derive(Clone, Debug)]
enum ErrorKind {
    /// Error occurred while performing I/O.
    IOError,
    /// Invalid architecture supplied.
    ArchitectureInvalid,
    /// Environment variable not found, with the var in question as extra info.
    EnvVarNotFound,
    /// Error occurred while using external tools (ie: invocation of compiler).
    ToolExecError,
    /// Error occurred due to missing external tools.
    ToolNotFound,
    /// One of the function arguments failed validation.
    InvalidArgument,
}

/// Represents an internal error that occurred, with an explanation.
#[derive(Clone, Debug)]
pub struct Error {
    /// Describes the kind of error that occurred.
    kind: ErrorKind,
    /// More explanation of error that occurred.
    message: String,
}

impl Error {
    fn new(kind: ErrorKind, message: &str) -> Error {
        Error {
            kind: kind,
            message: message.to_owned(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(ErrorKind::IOError, &format!("{}", e))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}

/// Configuration used to represent an invocation of a C compiler.
///
/// This can be used to figure out what compiler is in use, what the arguments
/// to it are, and what the environment variables look like for the compiler.
/// This can be used to further configure other build systems (e.g. forward
/// along CC and/or CFLAGS) or the `to_command` method can be used to run the
/// compiler itself.
#[derive(Clone, Debug)]
pub struct Tool {
    path: PathBuf,
    cc_wrapper_path: Option<PathBuf>,
    cc_wrapper_args: Vec<OsString>,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
    family: ToolFamily,
    cuda: bool,
    removed_args: Vec<OsString>,
}

/// Represents the family of tools this tool belongs to.
///
/// Each family of tools differs in how and what arguments they accept.
///
/// Detection of a family is done on best-effort basis and may not accurately reflect the tool.
#[derive(Copy, Clone, Debug, PartialEq)]
enum ToolFamily {
    /// Tool is GNU Compiler Collection-like.
    Gnu,
    /// Tool is Clang-like. It differs from the GCC in a sense that it accepts superset of flags
    /// and its cross-compilation approach is different.
    Clang,
    /// Tool is the MSVC cl.exe.
    Msvc { clang_cl: bool },
}

impl ToolFamily {
    /// What the flag to request debug info for this family of tools look like
    fn add_debug_flags(&self, cmd: &mut Tool, dwarf_version: Option<u32>) {
        match *self {
            ToolFamily::Msvc { .. } => {
                cmd.push_cc_arg("-Z7".into());
            }
            ToolFamily::Gnu | ToolFamily::Clang => {
                cmd.push_cc_arg(
                    dwarf_version
                        .map_or_else(|| "-g".into(), |v| format!("-gdwarf-{}", v))
                        .into(),
                );
            }
        }
    }

    /// What the flag to force frame pointers.
    fn add_force_frame_pointer(&self, cmd: &mut Tool) {
        match *self {
            ToolFamily::Gnu | ToolFamily::Clang => {
                cmd.push_cc_arg("-fno-omit-frame-pointer".into());
            }
            _ => (),
        }
    }

    /// What the flags to enable all warnings
    fn warnings_flags(&self) -> &'static str {
        match *self {
            ToolFamily::Msvc { .. } => "-W4",
            ToolFamily::Gnu | ToolFamily::Clang => "-Wall",
        }
    }

    /// What the flags to enable extra warnings
    fn extra_warnings_flags(&self) -> Option<&'static str> {
        match *self {
            ToolFamily::Msvc { .. } => None,
            ToolFamily::Gnu | ToolFamily::Clang => Some("-Wextra"),
        }
    }

    /// What the flag to turn warning into errors
    fn warnings_to_errors_flag(&self) -> &'static str {
        match *self {
            ToolFamily::Msvc { .. } => "-WX",
            ToolFamily::Gnu | ToolFamily::Clang => "-Werror",
        }
    }

    fn verbose_stderr(&self) -> bool {
        *self == ToolFamily::Clang
    }
}

/// Represents an object.
///
/// This is a source file -> object file pair.
#[derive(Clone, Debug)]
struct Object {
    src: PathBuf,
    dst: PathBuf,
}

impl Object {
    /// Create a new source file -> object file pair.
    fn new(src: PathBuf, dst: PathBuf) -> Object {
        Object { src: src, dst: dst }
    }
}

impl Build {
    /// Construct a new instance of a blank set of configuration.
    ///
    /// This builder is finished with the [`compile`] function.
    ///
    /// [`compile`]: struct.Build.html#method.compile
    pub fn new() -> Build {
        Build {
            include_directories: Vec::new(),
            definitions: Vec::new(),
            objects: Vec::new(),
            flags: Vec::new(),
            flags_supported: Vec::new(),
            known_flag_support_status: Arc::new(Mutex::new(HashMap::new())),
            ar_flags: Vec::new(),
            asm_flags: Vec::new(),
            no_default_flags: false,
            files: Vec::new(),
            shared_flag: None,
            static_flag: None,
            cpp: false,
            cpp_link_stdlib: None,
            cpp_set_stdlib: None,
            cuda: false,
            cudart: None,
            target: None,
            host: None,
            out_dir: None,
            opt_level: None,
            debug: None,
            force_frame_pointer: None,
            env: Vec::new(),
            compiler: None,
            archiver: None,
            ranlib: None,
            cargo_metadata: true,
            link_lib_modifiers: Vec::new(),
            pic: None,
            use_plt: None,
            static_crt: None,
            warnings: None,
            extra_warnings: None,
            warnings_into_errors: false,
            env_cache: Arc::new(Mutex::new(HashMap::new())),
            apple_sdk_root_cache: Arc::new(Mutex::new(HashMap::new())),
            emit_rerun_if_env_changed: true,
        }
    }

    /// Add a directory to the `-I` or include path for headers
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// let library_path = Path::new("/path/to/library");
    ///
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .include(library_path)
    ///     .include("src")
    ///     .compile("foo");
    /// ```
    pub fn include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Build {
        self.include_directories.push(dir.as_ref().to_path_buf());
        self
    }

    /// Add multiple directories to the `-I` include path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::path::Path;
    /// # let condition = true;
    /// #
    /// let mut extra_dir = None;
    /// if condition {
    ///     extra_dir = Some(Path::new("/path/to"));
    /// }
    ///
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .includes(extra_dir)
    ///     .compile("foo");
    /// ```
    pub fn includes<P>(&mut self, dirs: P) -> &mut Build
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for dir in dirs {
            self.include(dir);
        }
        self
    }

    /// Specify a `-D` variable with an optional value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .define("FOO", "BAR")
    ///     .define("BAZ", None)
    ///     .compile("foo");
    /// ```
    pub fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Build {
        self.definitions
            .push((var.to_string(), val.into().map(|s| s.to_string())));
        self
    }

    /// Add an arbitrary object file to link in
    pub fn object<P: AsRef<Path>>(&mut self, obj: P) -> &mut Build {
        self.objects.push(obj.as_ref().to_path_buf());
        self
    }

    /// Add an arbitrary flag to the invocation of the compiler
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .flag("-ffunction-sections")
    ///     .compile("foo");
    /// ```
    pub fn flag(&mut self, flag: &str) -> &mut Build {
        self.flags.push(flag.to_string());
        self
    }

    /// Add a flag to the invocation of the ar
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .file("src/bar.c")
    ///     .ar_flag("/NODEFAULTLIB:libc.dll")
    ///     .compile("foo");
    /// ```
    pub fn ar_flag(&mut self, flag: &str) -> &mut Build {
        self.ar_flags.push(flag.to_string());
        self
    }

    /// Add a flag that will only be used with assembly files.
    ///
    /// The flag will be applied to input files with either a `.s` or
    /// `.asm` extension (case insensitive).
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .asm_flag("-Wa,-defsym,abc=1")
    ///     .file("src/foo.S")  // The asm flag will be applied here
    ///     .file("src/bar.c")  // The asm flag will not be applied here
    ///     .compile("foo");
    /// ```
    pub fn asm_flag(&mut self, flag: &str) -> &mut Build {
        self.asm_flags.push(flag.to_string());
        self
    }

    fn ensure_check_file(&self) -> Result<PathBuf, Error> {
        let out_dir = self.get_out_dir()?;
        let src = if self.cuda {
            assert!(self.cpp);
            out_dir.join("flag_check.cu")
        } else if self.cpp {
            out_dir.join("flag_check.cpp")
        } else {
            out_dir.join("flag_check.c")
        };

        if !src.exists() {
            let mut f = fs::File::create(&src)?;
            write!(f, "int main(void) {{ return 0; }}")?;
        }

        Ok(src)
    }

    /// Run the compiler to test if it accepts the given flag.
    ///
    /// For a convenience method for setting flags conditionally,
    /// see `flag_if_supported()`.
    ///
    /// It may return error if it's unable to run the compiler with a test file
    /// (e.g. the compiler is missing or a write to the `out_dir` failed).
    ///
    /// Note: Once computed, the result of this call is stored in the
    /// `known_flag_support` field. If `is_flag_supported(flag)`
    /// is called again, the result will be read from the hash table.
    pub fn is_flag_supported(&self, flag: &str) -> Result<bool, Error> {
        let mut known_status = self.known_flag_support_status.lock().unwrap();
        if let Some(is_supported) = known_status.get(flag).cloned() {
            return Ok(is_supported);
        }

        let out_dir = self.get_out_dir()?;
        let src = self.ensure_check_file()?;
        let obj = out_dir.join("flag_check");
        let target = self.get_target()?;
        let host = self.get_host()?;
        let mut cfg = Build::new();
        cfg.flag(flag)
            .target(&target)
            .opt_level(0)
            .host(&host)
            .debug(false)
            .cpp(self.cpp)
            .cuda(self.cuda);
        if let Some(ref c) = self.compiler {
            cfg.compiler(c.clone());
        }
        let mut compiler = cfg.try_get_compiler()?;

        // Clang uses stderr for verbose output, which yields a false positive
        // result if the CFLAGS/CXXFLAGS include -v to aid in debugging.
        if compiler.family.verbose_stderr() {
            compiler.remove_arg("-v".into());
        }

        let mut cmd = compiler.to_command();
        let is_arm = target.contains("aarch64") || target.contains("arm");
        let clang = compiler.family == ToolFamily::Clang;
        command_add_output_file(
            &mut cmd,
            &obj,
            self.cuda,
            target.contains("msvc"),
            clang,
            false,
            is_arm,
        );

        // We need to explicitly tell msvc not to link and create an exe
        // in the root directory of the crate
        if target.contains("msvc") && !self.cuda {
            cmd.arg("-c");
        }

        cmd.arg(&src);

        let output = cmd.output()?;
        let is_supported = output.status.success() && output.stderr.is_empty();

        known_status.insert(flag.to_owned(), is_supported);
        Ok(is_supported)
    }

    /// Add an arbitrary flag to the invocation of the compiler if it supports it
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .flag_if_supported("-Wlogical-op") // only supported by GCC
    ///     .flag_if_supported("-Wunreachable-code") // only supported by clang
    ///     .compile("foo");
    /// ```
    pub fn flag_if_supported(&mut self, flag: &str) -> &mut Build {
        self.flags_supported.push(flag.to_string());
        self
    }

    /// Set the `-shared` flag.
    ///
    /// When enabled, the compiler will produce a shared object which can
    /// then be linked with other objects to form an executable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .shared_flag(true)
    ///     .compile("libfoo.so");
    /// ```
    pub fn shared_flag(&mut self, shared_flag: bool) -> &mut Build {
        self.shared_flag = Some(shared_flag);
        self
    }

    /// Set the `-static` flag.
    ///
    /// When enabled on systems that support dynamic linking, this prevents
    /// linking with the shared libraries.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .shared_flag(true)
    ///     .static_flag(true)
    ///     .compile("foo");
    /// ```
    pub fn static_flag(&mut self, static_flag: bool) -> &mut Build {
        self.static_flag = Some(static_flag);
        self
    }

    /// Disables the generation of default compiler flags. The default compiler
    /// flags may cause conflicts in some cross compiling scenarios.
    ///
    /// Setting the `CRATE_CC_NO_DEFAULTS` environment variable has the same
    /// effect as setting this to `true`. The presence of the environment
    /// variable and the value of `no_default_flags` will be OR'd together.
    pub fn no_default_flags(&mut self, no_default_flags: bool) -> &mut Build {
        self.no_default_flags = no_default_flags;
        self
    }

    /// Add a file which will be compiled
    pub fn file<P: AsRef<Path>>(&mut self, p: P) -> &mut Build {
        self.files.push(p.as_ref().to_path_buf());
        self
    }

    /// Add files which will be compiled
    pub fn files<P>(&mut self, p: P) -> &mut Build
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for file in p.into_iter() {
            self.file(file);
        }
        self
    }

    /// Set C++ support.
    ///
    /// The other `cpp_*` options will only become active if this is set to
    /// `true`.
    pub fn cpp(&mut self, cpp: bool) -> &mut Build {
        self.cpp = cpp;
        self
    }

    /// Set CUDA C++ support.
    ///
    /// Enabling CUDA will pass the detected C/C++ toolchain as an argument to
    /// the CUDA compiler, NVCC. NVCC itself accepts some limited GNU-like args;
    /// any other arguments for the C/C++ toolchain will be redirected using
    /// "-Xcompiler" flags.
    ///
    /// If enabled, this also implicitly enables C++ support.
    pub fn cuda(&mut self, cuda: bool) -> &mut Build {
        self.cuda = cuda;
        if cuda {
            self.cpp = true;
            self.cudart = Some("static".to_string());
        }
        self
    }

    /// Link CUDA run-time.
    ///
    /// This option mimics the `--cudart` NVCC command-line option. Just like
    /// the original it accepts `{none|shared|static}`, with default being
    /// `static`. The method has to be invoked after `.cuda(true)`, or not
    /// at all, if the default is right for the project.
    pub fn cudart(&mut self, cudart: &str) -> &mut Build {
        if self.cuda {
            self.cudart = Some(cudart.to_string());
        }
        self
    }

    /// Set warnings into errors flag.
    ///
    /// Disabled by default.
    ///
    /// Warning: turning warnings into errors only make sense
    /// if you are a developer of the crate using cc-rs.
    /// Some warnings only appear on some architecture or
    /// specific version of the compiler. Any user of this crate,
    /// or any other crate depending on it, could fail during
    /// compile time.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .warnings_into_errors(true)
    ///     .compile("libfoo.a");
    /// ```
    pub fn warnings_into_errors(&mut self, warnings_into_errors: bool) -> &mut Build {
        self.warnings_into_errors = warnings_into_errors;
        self
    }

    /// Set warnings flags.
    ///
    /// Adds some flags:
    /// - "-Wall" for MSVC.
    /// - "-Wall", "-Wextra" for GNU and Clang.
    ///
    /// Enabled by default.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .warnings(false)
    ///     .compile("libfoo.a");
    /// ```
    pub fn warnings(&mut self, warnings: bool) -> &mut Build {
        self.warnings = Some(warnings);
        self.extra_warnings = Some(warnings);
        self
    }

    /// Set extra warnings flags.
    ///
    /// Adds some flags:
    /// - nothing for MSVC.
    /// - "-Wextra" for GNU and Clang.
    ///
    /// Enabled by default.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Disables -Wextra, -Wall remains enabled:
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .extra_warnings(false)
    ///     .compile("libfoo.a");
    /// ```
    pub fn extra_warnings(&mut self, warnings: bool) -> &mut Build {
        self.extra_warnings = Some(warnings);
        self
    }

    /// Set the standard library to link against when compiling with C++
    /// support.
    ///
    /// See [`get_cpp_link_stdlib`](cc::Build::get_cpp_link_stdlib) documentation
    /// for the default value.
    /// If the `CXXSTDLIB` environment variable is set, its value will
    /// override the default value, but not the value explicitly set by calling
    /// this function.
    ///
    /// A value of `None` indicates that no automatic linking should happen,
    /// otherwise cargo will link against the specified library.
    ///
    /// The given library name must not contain the `lib` prefix.
    ///
    /// Common values:
    /// - `stdc++` for GNU
    /// - `c++` for Clang
    /// - `c++_shared` or `c++_static` for Android
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .shared_flag(true)
    ///     .cpp_link_stdlib("stdc++")
    ///     .compile("libfoo.so");
    /// ```
    pub fn cpp_link_stdlib<'a, V: Into<Option<&'a str>>>(
        &mut self,
        cpp_link_stdlib: V,
    ) -> &mut Build {
        self.cpp_link_stdlib = Some(cpp_link_stdlib.into().map(|s| s.into()));
        self
    }

    /// Force the C++ compiler to use the specified standard library.
    ///
    /// Setting this option will automatically set `cpp_link_stdlib` to the same
    /// value.
    ///
    /// The default value of this option is always `None`.
    ///
    /// This option has no effect when compiling for a Visual Studio based
    /// target.
    ///
    /// This option sets the `-stdlib` flag, which is only supported by some
    /// compilers (clang, icc) but not by others (gcc). The library will not
    /// detect which compiler is used, as such it is the responsibility of the
    /// caller to ensure that this option is only used in conjunction with a
    /// compiler which supports the `-stdlib` flag.
    ///
    /// A value of `None` indicates that no specific C++ standard library should
    /// be used, otherwise `-stdlib` is added to the compile invocation.
    ///
    /// The given library name must not contain the `lib` prefix.
    ///
    /// Common values:
    /// - `stdc++` for GNU
    /// - `c++` for Clang
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .cpp_set_stdlib("c++")
    ///     .compile("libfoo.a");
    /// ```
    pub fn cpp_set_stdlib<'a, V: Into<Option<&'a str>>>(
        &mut self,
        cpp_set_stdlib: V,
    ) -> &mut Build {
        let cpp_set_stdlib = cpp_set_stdlib.into();
        self.cpp_set_stdlib = cpp_set_stdlib.map(|s| s.into());
        self.cpp_link_stdlib(cpp_set_stdlib);
        self
    }

    /// Configures the target this configuration will be compiling for.
    ///
    /// This option is automatically scraped from the `TARGET` environment
    /// variable by build scripts, so it's not required to call this function.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .target("aarch64-linux-android")
    ///     .compile("foo");
    /// ```
    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    /// Configures the host assumed by this configuration.
    ///
    /// This option is automatically scraped from the `HOST` environment
    /// variable by build scripts, so it's not required to call this function.
    ///
    /// # Example
    ///
    /// ```no_run
    /// cc::Build::new()
    ///     .file("src/foo.c")
    ///     .host("arm-linux-gnueabihf")
    ///     .compile("foo");
    /// ```
    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    /// Configures the optimization level of the generated object files.
    ///
    /// This option is automatically scraped from the `OPT_LEVEL` environment
    /// variable by build scripts, so it's not required to call this function.
    pub fn opt_level(&mut self, opt_level: u32) -> &mut Build {
        self.opt_level = Some(opt_level.to_string());
        self
    }

    /// Configures the optimization level of the generated object files.
    ///
    /// This option is automatically scraped from the `OPT_LEVEL` environment
    /// variable by build scripts, so it's not required to call this function.
    pub fn opt_level_str(&mut self, opt_level: &str) -> &mut Build {
        self.opt_level = Some(opt_level.to_string());
        self
    }

    /// Configures whether the compiler will emit debug information when
    /// generating object files.
    ///
    /// This option is automatically scraped from the `DEBUG` environment
    /// variable by build scripts, so it's not required to call this function.
    pub fn debug(&mut self, debug: bool) -> &mut Build {
        self.debug = Some(debug);
        self
    }

    /// Configures whether the compiler will emit instructions to store
    /// frame pointers during codegen.
    ///
    /// This option is automatically enabled when debug information is emitted.
    /// Otherwise the target platform compiler's default will be used.
    /// You can use this option to force a specific setting.
    pub fn force_frame_pointer(&mut self, force: bool) -> &mut Build {
        self.force_frame_pointer = Some(force);
        self
    }

    /// Configures the output directory where all object files and static
    /// libraries will be located.
    ///
    /// This option is automatically scraped from the `OUT_DIR` environment
    /// variable by build scripts, so it's not required to call this function.
    pub fn out_dir<P: AsRef<Path>>(&mut self, out_dir: P) -> &mut Build {
        self.out_dir = Some(out_dir.as_ref().to_owned());
        self
    }

    /// Configures the compiler to be used to produce output.
    ///
    /// This option is automatically determined from the target platform or a
    /// number of environment variables, so it's not required to call this
    /// function.
    pub fn compiler<P: AsRef<Path>>(&mut self, compiler: P) -> &mut Build {
        self.compiler = Some(compiler.as_ref().to_owned());
        self
    }

    /// Configures the tool used to assemble archives.
    ///
    /// This option is automatically determined from the target platform or a
    /// number of environment variables, so it's not required to call this
    /// function.
    pub fn archiver<P: AsRef<Path>>(&mut self, archiver: P) -> &mut Build {
        self.archiver = Some(archiver.as_ref().to_owned());
        self
    }

    /// Configures the tool used to index archives.
    ///
    /// This option is automatically determined from the target platform or a
    /// number of environment variables, so it's not required to call this
    /// function.
    pub fn ranlib<P: AsRef<Path>>(&mut self, ranlib: P) -> &mut Build {
        self.ranlib = Some(ranlib.as_ref().to_owned());
        self
    }

    /// Define whether metadata should be emitted for cargo allowing it to
    /// automatically link the binary. Defaults to `true`.
    ///
    /// The emitted metadata is:
    ///
    ///  - `rustc-link-lib=static=`*compiled lib*
    ///  - `rustc-link-search=native=`*target folder*
    ///  - When target is MSVC, the ATL-MFC libs are added via `rustc-link-search=native=`
    ///  - When C++ is enabled, the C++ stdlib is added via `rustc-link-lib`
    ///  - If `emit_rerun_if_env_changed` is not `false`, `rerun-if-env-changed=`*env*
    ///
    pub fn cargo_metadata(&mut self, cargo_metadata: bool) -> &mut Build {
        self.cargo_metadata = cargo_metadata;
        self
    }

    /// Adds a native library modifier that will be added to the
    /// `rustc-link-lib=static:MODIFIERS=LIBRARY_NAME` metadata line
    /// emitted for cargo if `cargo_metadata` is enabled.
    /// See https://doc.rust-lang.org/rustc/command-line-arguments.html#-l-link-the-generated-crate-to-a-native-library
    /// for the list of modifiers accepted by rustc.
    pub fn link_lib_modifier(&mut self, link_lib_modifier: &str) -> &mut Build {
        self.link_lib_modifiers.push(link_lib_modifier.to_string());
        self
    }

    /// Configures whether the compiler will emit position independent code.
    ///
    /// This option defaults to `false` for `windows-gnu` and bare metal targets and
    /// to `true` for all other targets.
    pub fn pic(&mut self, pic: bool) -> &mut Build {
        self.pic = Some(pic);
        self
    }

    /// Configures whether the Procedure Linkage Table is used for indirect
    /// calls into shared libraries.
    ///
    /// The PLT is used to provide features like lazy binding, but introduces
    /// a small performance loss due to extra pointer indirection. Setting
    /// `use_plt` to `false` can provide a small performance increase.
    ///
    /// Note that skipping the PLT requires a recent version of GCC/Clang.
    ///
    /// This only applies to ELF targets. It has no effect on other platforms.
    pub fn use_plt(&mut self, use_plt: bool) -> &mut Build {
        self.use_plt = Some(use_plt);
        self
    }

    /// Define whether metadata should be emitted for cargo to detect environment
    /// changes that should trigger a rebuild.
    ///
    /// This has no effect if the `cargo_metadata` option is `false`.
    ///
    /// This option defaults to `true`.
    pub fn emit_rerun_if_env_changed(&mut self, emit_rerun_if_env_changed: bool) -> &mut Build {
        self.emit_rerun_if_env_changed = emit_rerun_if_env_changed;
        self
    }

    /// Configures whether the /MT flag or the /MD flag will be passed to msvc build tools.
    ///
    /// This option defaults to `false`, and affect only msvc targets.
    pub fn static_crt(&mut self, static_crt: bool) -> &mut Build {
        self.static_crt = Some(static_crt);
        self
    }

    #[doc(hidden)]
    pub fn __set_env<A, B>(&mut self, a: A, b: B) -> &mut Build
    where
        A: AsRef<OsStr>,
        B: AsRef<OsStr>,
    {
        self.env
            .push((a.as_ref().to_owned(), b.as_ref().to_owned()));
        self
    }

    /// Run the compiler, generating the file `output`
    ///
    /// This will return a result instead of panicing; see compile() for the complete description.
    pub fn try_compile(&self, output: &str) -> Result<(), Error> {
        let mut output_components = Path::new(output).components();
        match (output_components.next(), output_components.next()) {
            (Some(Component::Normal(_)), None) => {}
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    "argument of `compile` must be a single normal path component",
                ));
            }
        }

        let (lib_name, gnu_lib_name) = if output.starts_with("lib") && output.ends_with(".a") {
            (&output[3..output.len() - 2], output.to_owned())
        } else {
            let mut gnu = String::with_capacity(5 + output.len());
            gnu.push_str("lib");
            gnu.push_str(&output);
            gnu.push_str(".a");
            (output, gnu)
        };
        let dst = self.get_out_dir()?;

        let mut objects = Vec::new();
        for file in self.files.iter() {
            let obj = if file.has_root() {
                // If `file` is an absolute path, prefix the `basename`
                // with the `dirname`'s hash to ensure name uniqueness.
                let basename = file
                    .file_name()
                    .ok_or_else(|| Error::new(ErrorKind::InvalidArgument, "file_name() failure"))?
                    .to_string_lossy();
                let dirname = file
                    .parent()
                    .ok_or_else(|| Error::new(ErrorKind::InvalidArgument, "parent() failure"))?
                    .to_string_lossy();
                let mut hasher = hash_map::DefaultHasher::new();
                hasher.write(dirname.to_string().as_bytes());
                dst.join(format!("{:016x}-{}", hasher.finish(), basename))
                    .with_extension("o")
            } else {
                dst.join(file).with_extension("o")
            };
            let obj = if !obj.starts_with(&dst) {
                dst.join(obj.file_name().ok_or_else(|| {
                    Error::new(ErrorKind::IOError, "Getting object file details failed.")
                })?)
            } else {
                obj
            };

            match obj.parent() {
                Some(s) => fs::create_dir_all(s)?,
                None => {
                    return Err(Error::new(
                        ErrorKind::IOError,
                        "Getting object file details failed.",
                    ));
                }
            };

            objects.push(Object::new(file.to_path_buf(), obj));
        }
        self.compile_objects(&objects)?;
        self.assemble(lib_name, &dst.join(gnu_lib_name), &objects)?;

        if self.get_target()?.contains("msvc") {
            let compiler = self.get_base_compiler()?;
            let atlmfc_lib = compiler
                .env()
                .iter()
                .find(|&&(ref var, _)| var.as_os_str() == OsStr::new("LIB"))
                .and_then(|&(_, ref lib_paths)| {
                    env::split_paths(lib_paths).find(|path| {
                        let sub = Path::new("atlmfc/lib");
                        path.ends_with(sub) || path.parent().map_or(false, |p| p.ends_with(sub))
                    })
                });

            if let Some(atlmfc_lib) = atlmfc_lib {
                self.print(&format!(
                    "cargo:rustc-link-search=native={}",
                    atlmfc_lib.display()
                ));
            }
        }

        if self.link_lib_modifiers.is_empty() {
            self.print(&format!("cargo:rustc-link-lib=static={}", lib_name));
        } else {
            let m = self.link_lib_modifiers.join(",");
            self.print(&format!("cargo:rustc-link-lib=static:{}={}", m, lib_name));
        }
        self.print(&format!("cargo:rustc-link-search=native={}", dst.display()));

        // Add specific C++ libraries, if enabled.
        if self.cpp {
            if let Some(stdlib) = self.get_cpp_link_stdlib()? {
                self.print(&format!("cargo:rustc-link-lib={}", stdlib));
            }
        }

        let cudart = match &self.cudart {
            Some(opt) => opt.as_str(), // {none|shared|static}
            None => "none",
        };
        if cudart != "none" {
            if let Some(nvcc) = which(&self.get_compiler().path) {
                // Try to figure out the -L search path. If it fails,
                // it's on user to specify one by passing it through
                // RUSTFLAGS environment variable.
                let mut libtst = false;
                let mut libdir = nvcc;
                libdir.pop(); // remove 'nvcc'
                libdir.push("..");
                let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
                if cfg!(target_os = "linux") {
                    libdir.push("targets");
                    libdir.push(target_arch.to_owned() + "-linux");
                    libdir.push("lib");
                    libtst = true;
                } else if cfg!(target_env = "msvc") {
                    libdir.push("lib");
                    match target_arch.as_str() {
                        "x86_64" => {
                            libdir.push("x64");
                            libtst = true;
                        }
                        "x86" => {
                            libdir.push("Win32");
                            libtst = true;
                        }
                        _ => libtst = false,
                    }
                }
                if libtst && libdir.is_dir() {
                    println!(
                        "cargo:rustc-link-search=native={}",
                        libdir.to_str().unwrap()
                    );
                }

                // And now the -l flag.
                let lib = match cudart {
                    "shared" => "cudart",
                    "static" => "cudart_static",
                    bad => panic!("unsupported cudart option: {}", bad),
                };
                println!("cargo:rustc-link-lib={}", lib);
            }
        }

        Ok(())
    }

    /// Run the compiler, generating the file `output`
    ///
    /// # Library name
    ///
    /// The `output` string argument determines the file name for the compiled
    /// library. The Rust compiler will create an assembly named "lib"+output+".a".
    /// MSVC will create a file named output+".lib".
    ///
    /// The choice of `output` is close to arbitrary, but:
    ///
    /// - must be nonempty,
    /// - must not contain a path separator (`/`),
    /// - must be unique across all `compile` invocations made by the same build
    ///   script.
    ///
    /// If your build script compiles a single source file, the base name of
    /// that source file would usually be reasonable:
    ///
    /// ```no_run
    /// cc::Build::new().file("blobstore.c").compile("blobstore");
    /// ```
    ///
    /// Compiling multiple source files, some people use their crate's name, or
    /// their crate's name + "-cc".
    ///
    /// Otherwise, please use your imagination.
    ///
    /// For backwards compatibility, if `output` starts with "lib" *and* ends
    /// with ".a", a second "lib" prefix and ".a" suffix do not get added on,
    /// but this usage is deprecated; please omit `lib` and `.a` in the argument
    /// that you pass.
    ///
    /// # Panics
    ///
    /// Panics if `output` is not formatted correctly or if one of the underlying
    /// compiler commands fails. It can also panic if it fails reading file names
    /// or creating directories.
    pub fn compile(&self, output: &str) {
        if let Err(e) = self.try_compile(output) {
            fail(&e.message);
        }
    }

    #[cfg(feature = "parallel")]
    fn compile_objects<'me>(&'me self, objs: &[Object]) -> Result<(), Error> {
        use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
        use std::sync::Once;

        // Limit our parallelism globally with a jobserver. Start off by
        // releasing our own token for this process so we can have a bit of an
        // easier to write loop below. If this fails, though, then we're likely
        // on Windows with the main implicit token, so we just have a bit extra
        // parallelism for a bit and don't reacquire later.
        let server = jobserver();
        let reacquire = server.release_raw().is_ok();

        // When compiling objects in parallel we do a few dirty tricks to speed
        // things up:
        //
        // * First is that we use the `jobserver` crate to limit the parallelism
        //   of this build script. The `jobserver` crate will use a jobserver
        //   configured by Cargo for build scripts to ensure that parallelism is
        //   coordinated across C compilations and Rust compilations. Before we
        //   compile anything we make sure to wait until we acquire a token.
        //
        //   Note that this jobserver is cached globally so we only used one per
        //   process and only worry about creating it once.
        //
        // * Next we use a raw `thread::spawn` per thread to actually compile
        //   objects in parallel. We only actually spawn a thread after we've
        //   acquired a token to perform some work
        //
        // * Finally though we want to keep the dependencies of this crate
        //   pretty light, so we avoid using a safe abstraction like `rayon` and
        //   instead rely on some bits of `unsafe` code. We know that this stack
        //   frame persists while everything is compiling so we use all the
        //   stack-allocated objects without cloning/reallocating. We use a
        //   transmute to `State` with a `'static` lifetime to persist
        //   everything we need across the boundary, and the join-on-drop
        //   semantics of `JoinOnDrop` should ensure that our stack frame is
        //   alive while threads are alive.
        //
        // With all that in mind we compile all objects in a loop here, after we
        // acquire the appropriate tokens, Once all objects have been compiled
        // we join on all the threads and propagate the results of compilation.
        //
        // Note that as a slight optimization we try to break out as soon as
        // possible as soon as any compilation fails to ensure that errors get
        // out to the user as fast as possible.
        let error = AtomicBool::new(false);
        let mut threads = Vec::new();
        for obj in objs {
            if error.load(SeqCst) {
                break;
            }
            let token = server.acquire()?;
            let state = State {
                build: self,
                obj,
                error: &error,
            };
            let state = unsafe { std::mem::transmute::<State, State<'static>>(state) };
            let thread = thread::spawn(|| {
                let state: State<'me> = state; // erase the `'static` lifetime
                let result = state.build.compile_object(state.obj);
                if result.is_err() {
                    state.error.store(true, SeqCst);
                }
                drop(token); // make sure our jobserver token is released after the compile
                return result;
            });
            threads.push(JoinOnDrop(Some(thread)));
        }

        for mut thread in threads {
            if let Some(thread) = thread.0.take() {
                thread.join().expect("thread should not panic")?;
            }
        }

        // Reacquire our process's token before we proceed, which we released
        // before entering the loop above.
        if reacquire {
            server.acquire_raw()?;
        }

        return Ok(());

        /// Shared state from the parent thread to the child thread. This
        /// package of pointers is temporarily transmuted to a `'static`
        /// lifetime to cross the thread boundary and then once the thread is
        /// running we erase the `'static` to go back to an anonymous lifetime.
        struct State<'a> {
            build: &'a Build,
            obj: &'a Object,
            error: &'a AtomicBool,
        }

        /// Returns a suitable `jobserver::Client` used to coordinate
        /// parallelism between build scripts.
        fn jobserver() -> &'static jobserver::Client {
            static INIT: Once = Once::new();
            static mut JOBSERVER: Option<jobserver::Client> = None;

            fn _assert_sync<T: Sync>() {}
            _assert_sync::<jobserver::Client>();

            unsafe {
                INIT.call_once(|| {
                    let server = default_jobserver();
                    JOBSERVER = Some(server);
                });
                JOBSERVER.as_ref().unwrap()
            }
        }

        unsafe fn default_jobserver() -> jobserver::Client {
            // Try to use the environmental jobserver which Cargo typically
            // initializes for us...
            if let Some(client) = jobserver::Client::from_env() {
                return client;
            }

            // ... but if that fails for whatever reason select something
            // reasonable and crate a new jobserver. Use `NUM_JOBS` if set (it's
            // configured by Cargo) and otherwise just fall back to a
            // semi-reasonable number. Note that we could use `num_cpus` here
            // but it's an extra dependency that will almost never be used, so
            // it's generally not too worth it.
            let mut parallelism = 4;
            if let Ok(amt) = env::var("NUM_JOBS") {
                if let Ok(amt) = amt.parse() {
                    parallelism = amt;
                }
            }

            // If we create our own jobserver then be sure to reserve one token
            // for ourselves.
            let client = jobserver::Client::new(parallelism).expect("failed to create jobserver");
            client.acquire_raw().expect("failed to acquire initial");
            return client;
        }

        struct JoinOnDrop(Option<thread::JoinHandle<Result<(), Error>>>);

        impl Drop for JoinOnDrop {
            fn drop(&mut self) {
                if let Some(thread) = self.0.take() {
                    drop(thread.join());
                }
            }
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn compile_objects(&self, objs: &[Object]) -> Result<(), Error> {
        for obj in objs {
            self.compile_object(obj)?;
        }
        Ok(())
    }

    fn compile_object(&self, obj: &Object) -> Result<(), Error> {
        let asm_ext = AsmFileExt::from_path(&obj.src);
        let is_asm = asm_ext.is_some();
        let target = self.get_target()?;
        let msvc = target.contains("msvc");
        let compiler = self.try_get_compiler()?;
        let clang = compiler.family == ToolFamily::Clang;

        let (mut cmd, name) = if msvc && asm_ext == Some(AsmFileExt::DotAsm) {
            self.msvc_macro_assembler()?
        } else {
            let mut cmd = compiler.to_command();
            for &(ref a, ref b) in self.env.iter() {
                cmd.env(a, b);
            }
            (
                cmd,
                compiler
                    .path
                    .file_name()
                    .ok_or_else(|| Error::new(ErrorKind::IOError, "Failed to get compiler path."))?
                    .to_string_lossy()
                    .into_owned(),
            )
        };
        let is_arm = target.contains("aarch64") || target.contains("arm");
        command_add_output_file(&mut cmd, &obj.dst, self.cuda, msvc, clang, is_asm, is_arm);
        // armasm and armasm64 don't requrie -c option
        if !msvc || !is_asm || !is_arm {
            cmd.arg("-c");
        }
        if self.cuda && self.cuda_file_count() > 1 {
            cmd.arg("--device-c");
        }
        if is_asm {
            cmd.args(&self.asm_flags);
        }
        if compiler.family == (ToolFamily::Msvc { clang_cl: true }) && !is_asm {
            // #513: For `clang-cl`, separate flags/options from the input file.
            // When cross-compiling macOS -> Windows, this avoids interpreting
            // common `/Users/...` paths as the `/U` flag and triggering
            // `-Wslash-u-filename` warning.
            cmd.arg("--");
        }
        cmd.arg(&obj.src);
        if cfg!(target_os = "macos") {
            self.fix_env_for_apple_os(&mut cmd)?;
        }

        run(&mut cmd, &name)?;
        Ok(())
    }

    /// This will return a result instead of panicing; see expand() for the complete description.
    pub fn try_expand(&self) -> Result<Vec<u8>, Error> {
        let compiler = self.try_get_compiler()?;
        let mut cmd = compiler.to_command();
        for &(ref a, ref b) in self.env.iter() {
            cmd.env(a, b);
        }
        cmd.arg("-E");

        assert!(
            self.files.len() <= 1,
            "Expand may only be called for a single file"
        );

        for file in self.files.iter() {
            cmd.arg(file);
        }

        let name = compiler
            .path
            .file_name()
            .ok_or_else(|| Error::new(ErrorKind::IOError, "Failed to get compiler path."))?
            .to_string_lossy()
            .into_owned();

        Ok(run_output(&mut cmd, &name)?)
    }

    /// Run the compiler, returning the macro-expanded version of the input files.
    ///
    /// This is only relevant for C and C++ files.
    ///
    /// # Panics
    /// Panics if more than one file is present in the config, or if compiler
    /// path has an invalid file name.
    ///
    /// # Example
    /// ```no_run
    /// let out = cc::Build::new().file("src/foo.c").expand();
    /// ```
    pub fn expand(&self) -> Vec<u8> {
        match self.try_expand() {
            Err(e) => fail(&e.message),
            Ok(v) => v,
        }
    }

    /// Get the compiler that's in use for this configuration.
    ///
    /// This function will return a `Tool` which represents the culmination
    /// of this configuration at a snapshot in time. The returned compiler can
    /// be inspected (e.g. the path, arguments, environment) to forward along to
    /// other tools, or the `to_command` method can be used to invoke the
    /// compiler itself.
    ///
    /// This method will take into account all configuration such as debug
    /// information, optimization level, include directories, defines, etc.
    /// Additionally, the compiler binary in use follows the standard
    /// conventions for this path, e.g. looking at the explicitly set compiler,
    /// environment variables (a number of which are inspected here), and then
    /// falling back to the default configuration.
    ///
    /// # Panics
    ///
    /// Panics if an error occurred while determining the architecture.
    pub fn get_compiler(&self) -> Tool {
        match self.try_get_compiler() {
            Ok(tool) => tool,
            Err(e) => fail(&e.message),
        }
    }

    /// Get the compiler that's in use for this configuration.
    ///
    /// This will return a result instead of panicing; see get_compiler() for the complete description.
    pub fn try_get_compiler(&self) -> Result<Tool, Error> {
        let opt_level = self.get_opt_level()?;
        let target = self.get_target()?;

        let mut cmd = self.get_base_compiler()?;
        let envflags = self.envflags(if self.cpp { "CXXFLAGS" } else { "CFLAGS" });

        // Disable default flag generation via `no_default_flags` or environment variable
        let no_defaults = self.no_default_flags || self.getenv("CRATE_CC_NO_DEFAULTS").is_some();

        if !no_defaults {
            self.add_default_flags(&mut cmd, &target, &opt_level)?;
        } else {
            println!("Info: default compiler flags are disabled");
        }

        for arg in envflags {
            cmd.push_cc_arg(arg.into());
        }

        for directory in self.include_directories.iter() {
            cmd.args.push("-I".into());
            cmd.args.push(directory.into());
        }

        // If warnings and/or extra_warnings haven't been explicitly set,
        // then we set them only if the environment doesn't already have
        // CFLAGS/CXXFLAGS, since those variables presumably already contain
        // the desired set of warnings flags.

        if self
            .warnings
            .unwrap_or(if self.has_flags() { false } else { true })
        {
            let wflags = cmd.family.warnings_flags().into();
            cmd.push_cc_arg(wflags);
        }

        if self
            .extra_warnings
            .unwrap_or(if self.has_flags() { false } else { true })
        {
            if let Some(wflags) = cmd.family.extra_warnings_flags() {
                cmd.push_cc_arg(wflags.into());
            }
        }

        for flag in self.flags.iter() {
            cmd.args.push(flag.into());
        }

        for flag in self.flags_supported.iter() {
            if self.is_flag_supported(flag).unwrap_or(false) {
                cmd.push_cc_arg(flag.into());
            }
        }

        for &(ref key, ref value) in self.definitions.iter() {
            if let Some(ref value) = *value {
                cmd.args.push(format!("-D{}={}", key, value).into());
            } else {
                cmd.args.push(format!("-D{}", key).into());
            }
        }

        if self.warnings_into_errors {
            let warnings_to_errors_flag = cmd.family.warnings_to_errors_flag().into();
            cmd.push_cc_arg(warnings_to_errors_flag);
        }

        Ok(cmd)
    }

    fn add_default_flags(
        &self,
        cmd: &mut Tool,
        target: &str,
        opt_level: &str,
    ) -> Result<(), Error> {
        // Non-target flags
        // If the flag is not conditioned on target variable, it belongs here :)
        match cmd.family {
            ToolFamily::Msvc { .. } => {
                cmd.push_cc_arg("-nologo".into());

                let crt_flag = match self.static_crt {
                    Some(true) => "-MT",
                    Some(false) => "-MD",
                    None => {
                        let features = self
                            .getenv("CARGO_CFG_TARGET_FEATURE")
                            .unwrap_or(String::new());
                        if features.contains("crt-static") {
                            "-MT"
                        } else {
                            "-MD"
                        }
                    }
                };
                cmd.push_cc_arg(crt_flag.into());

                match &opt_level[..] {
                    // Msvc uses /O1 to enable all optimizations that minimize code size.
                    "z" | "s" | "1" => cmd.push_opt_unless_duplicate("-O1".into()),
                    // -O3 is a valid value for gcc and clang compilers, but not msvc. Cap to /O2.
                    "2" | "3" => cmd.push_opt_unless_duplicate("-O2".into()),
                    _ => {}
                }
            }
            ToolFamily::Gnu | ToolFamily::Clang => {
                // arm-linux-androideabi-gcc 4.8 shipped with Android NDK does
                // not support '-Oz'
                if opt_level == "z" && cmd.family != ToolFamily::Clang {
                    cmd.push_opt_unless_duplicate("-Os".into());
                } else {
                    cmd.push_opt_unless_duplicate(format!("-O{}", opt_level).into());
                }

                if cmd.family == ToolFamily::Clang && target.contains("android") {
                    // For compatibility with code that doesn't use pre-defined `__ANDROID__` macro.
                    // If compiler used via ndk-build or cmake (officially supported build methods)
                    // this macros is defined.
                    // See https://android.googlesource.com/platform/ndk/+/refs/heads/ndk-release-r21/build/cmake/android.toolchain.cmake#456
                    // https://android.googlesource.com/platform/ndk/+/refs/heads/ndk-release-r21/build/core/build-binary.mk#141
                    cmd.push_opt_unless_duplicate("-DANDROID".into());
                }

                if !target.contains("apple-ios") && !target.contains("apple-watchos") {
                    cmd.push_cc_arg("-ffunction-sections".into());
                    cmd.push_cc_arg("-fdata-sections".into());
                }
                // Disable generation of PIC on bare-metal for now: rust-lld doesn't support this yet
                if self.pic.unwrap_or(
                    !target.contains("windows")
                        && !target.contains("-none-")
                        && !target.contains("uefi"),
                ) {
                    cmd.push_cc_arg("-fPIC".into());
                    // PLT only applies if code is compiled with PIC support,
                    // and only for ELF targets.
                    if target.contains("linux") && !self.use_plt.unwrap_or(true) {
                        cmd.push_cc_arg("-fno-plt".into());
                    }
                }
            }
        }

        if self.get_debug() {
            if self.cuda {
                // NVCC debug flag
                cmd.args.push("-G".into());
            }
            let family = cmd.family;
            family.add_debug_flags(cmd, self.get_dwarf_version());
        }

        if self.get_force_frame_pointer() {
            let family = cmd.family;
            family.add_force_frame_pointer(cmd);
        }

        // Target flags
        match cmd.family {
            ToolFamily::Clang => {
                if !(target.contains("android")
                    && android_clang_compiler_uses_target_arg_internally(&cmd.path))
                {
                    if target.contains("darwin") {
                        if let Some(arch) =
                            map_darwin_target_from_rust_to_compiler_architecture(target)
                        {
                            cmd.args
                                .push(format!("--target={}-apple-darwin", arch).into());
                        }
                    } else if target.contains("macabi") {
                        if let Some(arch) =
                            map_darwin_target_from_rust_to_compiler_architecture(target)
                        {
                            cmd.args
                                .push(format!("--target={}-apple-ios-macabi", arch).into());
                        }
                    } else if target.contains("ios-sim") {
                        if let Some(arch) =
                            map_darwin_target_from_rust_to_compiler_architecture(target)
                        {
                            let deployment_target = env::var("IPHONEOS_DEPLOYMENT_TARGET")
                                .unwrap_or_else(|_| "7.0".into());
                            cmd.args.push(
                                format!(
                                    "--target={}-apple-ios{}-simulator",
                                    arch, deployment_target
                                )
                                .into(),
                            );
                        }
                    } else if target.contains("watchos-sim") {
                        if let Some(arch) =
                            map_darwin_target_from_rust_to_compiler_architecture(target)
                        {
                            let deployment_target = env::var("WATCHOS_DEPLOYMENT_TARGET")
                                .unwrap_or_else(|_| "5.0".into());
                            cmd.args.push(
                                format!(
                                    "--target={}-apple-watchos{}-simulator",
                                    arch, deployment_target
                                )
                                .into(),
                            );
                        }
                    } else if target.starts_with("riscv64gc-") {
                        cmd.args.push(
                            format!("--target={}", target.replace("riscv64gc", "riscv64")).into(),
                        );
                    } else if target.starts_with("riscv32gc-") {
                        cmd.args.push(
                            format!("--target={}", target.replace("riscv32gc", "riscv32")).into(),
                        );
                    } else if target.contains("uefi") {
                        if target.contains("x86_64") {
                            cmd.args.push("--target=x86_64-unknown-windows-gnu".into());
                        } else if target.contains("i686") {
                            cmd.args.push("--target=i686-unknown-windows-gnu".into())
                        } else if target.contains("aarch64") {
                            cmd.args.push("--target=aarch64-unknown-windows-gnu".into())
                        }
                    } else {
                        cmd.push_cc_arg(format!("--target={}", target).into());
                    }
                }
            }
            ToolFamily::Msvc { clang_cl } => {
                // This is an undocumented flag from MSVC but helps with making
                // builds more reproducible by avoiding putting timestamps into
                // files.
                cmd.push_cc_arg("-Brepro".into());

                if clang_cl {
                    if target.contains("x86_64") {
                        cmd.push_cc_arg("-m64".into());
                    } else if target.contains("86") {
                        cmd.push_cc_arg("-m32".into());
                        cmd.push_cc_arg("-arch:IA32".into());
                    } else {
                        cmd.push_cc_arg(format!("--target={}", target).into());
                    }
                } else {
                    if target.contains("i586") {
                        cmd.push_cc_arg("-arch:IA32".into());
                    }
                }

                // There is a check in corecrt.h that will generate a
                // compilation error if
                // _ARM_WINAPI_PARTITION_DESKTOP_SDK_AVAILABLE is
                // not defined to 1. The check was added in Windows
                // 8 days because only store apps were allowed on ARM.
                // This changed with the release of Windows 10 IoT Core.
                // The check will be going away in future versions of
                // the SDK, but for all released versions of the
                // Windows SDK it is required.
                if target.contains("arm") || target.contains("thumb") {
                    cmd.args
                        .push("-D_ARM_WINAPI_PARTITION_DESKTOP_SDK_AVAILABLE=1".into());
                }
            }
            ToolFamily::Gnu => {
                if target.contains("i686") || target.contains("i586") {
                    cmd.args.push("-m32".into());
                } else if target == "x86_64-unknown-linux-gnux32" {
                    cmd.args.push("-mx32".into());
                } else if target.contains("x86_64") || target.contains("powerpc64") {
                    cmd.args.push("-m64".into());
                }

                if target.contains("darwin") {
                    if let Some(arch) = map_darwin_target_from_rust_to_compiler_architecture(target)
                    {
                        cmd.args.push("-arch".into());
                        cmd.args.push(arch.into());
                    }
                }

                if target.contains("-kmc-solid_") {
                    cmd.args.push("-finput-charset=utf-8".into());
                }

                if self.static_flag.is_none() {
                    let features = self
                        .getenv("CARGO_CFG_TARGET_FEATURE")
                        .unwrap_or(String::new());
                    if features.contains("crt-static") {
                        cmd.args.push("-static".into());
                    }
                }

                // armv7 targets get to use armv7 instructions
                if (target.starts_with("armv7") || target.starts_with("thumbv7"))
                    && (target.contains("-linux-") || target.contains("-kmc-solid_"))
                {
                    cmd.args.push("-march=armv7-a".into());

                    if target.ends_with("eabihf") {
                        // lowest common denominator FPU
                        cmd.args.push("-mfpu=vfpv3-d16".into());
                    }
                }

                // (x86 Android doesn't say "eabi")
                if target.contains("-androideabi") && target.contains("v7") {
                    // -march=armv7-a handled above
                    cmd.args.push("-mthumb".into());
                    if !target.contains("neon") {
                        // On android we can guarantee some extra float instructions
                        // (specified in the android spec online)
                        // NEON guarantees even more; see below.
                        cmd.args.push("-mfpu=vfpv3-d16".into());
                    }
                    cmd.args.push("-mfloat-abi=softfp".into());
                }

                if target.contains("neon") {
                    cmd.args.push("-mfpu=neon-vfpv4".into());
                }

                if target.starts_with("armv4t-unknown-linux-") {
                    cmd.args.push("-march=armv4t".into());
                    cmd.args.push("-marm".into());
                    cmd.args.push("-mfloat-abi=soft".into());
                }

                if target.starts_with("armv5te-unknown-linux-") {
                    cmd.args.push("-march=armv5te".into());
                    cmd.args.push("-marm".into());
                    cmd.args.push("-mfloat-abi=soft".into());
                }

                // For us arm == armv6 by default
                if target.starts_with("arm-unknown-linux-") {
                    cmd.args.push("-march=armv6".into());
                    cmd.args.push("-marm".into());
                    if target.ends_with("hf") {
                        cmd.args.push("-mfpu=vfp".into());
                    } else {
                        cmd.args.push("-mfloat-abi=soft".into());
                    }
                }

                // We can guarantee some settings for FRC
                if target.starts_with("arm-frc-") {
                    cmd.args.push("-march=armv7-a".into());
                    cmd.args.push("-mcpu=cortex-a9".into());
                    cmd.args.push("-mfpu=vfpv3".into());
                    cmd.args.push("-mfloat-abi=softfp".into());
                    cmd.args.push("-marm".into());
                }

                // Turn codegen down on i586 to avoid some instructions.
                if target.starts_with("i586-unknown-linux-") {
                    cmd.args.push("-march=pentium".into());
                }

                // Set codegen level for i686 correctly
                if target.starts_with("i686-unknown-linux-") {
                    cmd.args.push("-march=i686".into());
                }

                // Looks like `musl-gcc` makes it hard for `-m32` to make its way
                // all the way to the linker, so we need to actually instruct the
                // linker that we're generating 32-bit executables as well. This'll
                // typically only be used for build scripts which transitively use
                // these flags that try to compile executables.
                if target == "i686-unknown-linux-musl" || target == "i586-unknown-linux-musl" {
                    cmd.args.push("-Wl,-melf_i386".into());
                }

                if target.starts_with("thumb") {
                    cmd.args.push("-mthumb".into());

                    if target.ends_with("eabihf") {
                        cmd.args.push("-mfloat-abi=hard".into())
                    }
                }
                if target.starts_with("thumbv6m") {
                    cmd.args.push("-march=armv6s-m".into());
                }
                if target.starts_with("thumbv7em") {
                    cmd.args.push("-march=armv7e-m".into());

                    if target.ends_with("eabihf") {
                        cmd.args.push("-mfpu=fpv4-sp-d16".into())
                    }
                }
                if target.starts_with("thumbv7m") {
                    cmd.args.push("-march=armv7-m".into());
                }
                if target.starts_with("thumbv8m.base") {
                    cmd.args.push("-march=armv8-m.base".into());
                }
                if target.starts_with("thumbv8m.main") {
                    cmd.args.push("-march=armv8-m.main".into());

                    if target.ends_with("eabihf") {
                        cmd.args.push("-mfpu=fpv5-sp-d16".into())
                    }
                }
                if target.starts_with("armebv7r") | target.starts_with("armv7r") {
                    if target.starts_with("armeb") {
                        cmd.args.push("-mbig-endian".into());
                    } else {
                        cmd.args.push("-mlittle-endian".into());
                    }

                    // ARM mode
                    cmd.args.push("-marm".into());

                    // R Profile
                    cmd.args.push("-march=armv7-r".into());

                    if target.ends_with("eabihf") {
                        // Calling convention
                        cmd.args.push("-mfloat-abi=hard".into());

                        // lowest common denominator FPU
                        // (see Cortex-R4 technical reference manual)
                        cmd.args.push("-mfpu=vfpv3-d16".into())
                    } else {
                        // Calling convention
                        cmd.args.push("-mfloat-abi=soft".into());
                    }
                }
                if target.starts_with("armv7a") {
                    cmd.args.push("-march=armv7-a".into());

                    if target.ends_with("eabihf") {
                        // lowest common denominator FPU
                        cmd.args.push("-mfpu=vfpv3-d16".into());
                    }
                }
                if target.starts_with("riscv32") || target.starts_with("riscv64") {
                    // get the 32i/32imac/32imc/64gc/64imac/... part
                    let mut parts = target.split('-');
                    if let Some(arch) = parts.next() {
                        let arch = &arch[5..];
                        if target.contains("linux") && arch.starts_with("64") {
                            cmd.args.push(("-march=rv64gc").into());
                            cmd.args.push("-mabi=lp64d".into());
                        } else if target.contains("freebsd") && arch.starts_with("64") {
                            cmd.args.push(("-march=rv64gc").into());
                            cmd.args.push("-mabi=lp64d".into());
                        } else if target.contains("openbsd") && arch.starts_with("64") {
                            cmd.args.push(("-march=rv64gc").into());
                            cmd.args.push("-mabi=lp64d".into());
                        } else if target.contains("linux") && arch.starts_with("32") {
                            cmd.args.push(("-march=rv32gc").into());
                            cmd.args.push("-mabi=ilp32d".into());
                        } else if arch.starts_with("64") {
                            cmd.args.push(("-march=rv".to_owned() + arch).into());
                            cmd.args.push("-mabi=lp64".into());
                        } else {
                            cmd.args.push(("-march=rv".to_owned() + arch).into());
                            cmd.args.push("-mabi=ilp32".into());
                        }
                        cmd.args.push("-mcmodel=medany".into());
                    }
                }
            }
        }

        if target.contains("apple-ios") || target.contains("apple-watchos") {
            self.ios_watchos_flags(cmd)?;
        }

        if self.static_flag.unwrap_or(false) {
            cmd.args.push("-static".into());
        }
        if self.shared_flag.unwrap_or(false) {
            cmd.args.push("-shared".into());
        }

        if self.cpp {
            match (self.cpp_set_stdlib.as_ref(), cmd.family) {
                (None, _) => {}
                (Some(stdlib), ToolFamily::Gnu) | (Some(stdlib), ToolFamily::Clang) => {
                    cmd.push_cc_arg(format!("-stdlib=lib{}", stdlib).into());
                }
                _ => {
                    println!(
                        "cargo:warning=cpp_set_stdlib is specified, but the {:?} compiler \
                         does not support this option, ignored",
                        cmd.family
                    );
                }
            }
        }

        Ok(())
    }

    fn has_flags(&self) -> bool {
        let flags_env_var_name = if self.cpp { "CXXFLAGS" } else { "CFLAGS" };
        let flags_env_var_value = self.get_var(flags_env_var_name);
        if let Ok(_) = flags_env_var_value {
            true
        } else {
            false
        }
    }

    fn msvc_macro_assembler(&self) -> Result<(Command, String), Error> {
        let target = self.get_target()?;
        let tool = if target.contains("x86_64") {
            "ml64.exe"
        } else if target.contains("arm") {
            "armasm.exe"
        } else if target.contains("aarch64") {
            "armasm64.exe"
        } else {
            "ml.exe"
        };
        let mut cmd = windows_registry::find(&target, tool).unwrap_or_else(|| self.cmd(tool));
        cmd.arg("-nologo"); // undocumented, yet working with armasm[64]
        for directory in self.include_directories.iter() {
            cmd.arg("-I").arg(directory);
        }
        if target.contains("aarch64") || target.contains("arm") {
            if self.get_debug() {
                cmd.arg("-g");
            }

            println!("cargo:warning=The MSVC ARM assemblers do not support -D flags");
        } else {
            if self.get_debug() {
                cmd.arg("-Zi");
            }

            for &(ref key, ref value) in self.definitions.iter() {
                if let Some(ref value) = *value {
                    cmd.arg(&format!("-D{}={}", key, value));
                } else {
                    cmd.arg(&format!("-D{}", key));
                }
            }
        }

        if target.contains("i686") || target.contains("i586") {
            cmd.arg("-safeseh");
        }
        for flag in self.flags.iter() {
            cmd.arg(flag);
        }

        Ok((cmd, tool.to_string()))
    }

    fn assemble(&self, lib_name: &str, dst: &Path, objs: &[Object]) -> Result<(), Error> {
        // Delete the destination if it exists as we want to
        // create on the first iteration instead of appending.
        let _ = fs::remove_file(&dst);

        // Add objects to the archive in limited-length batches. This helps keep
        // the length of the command line within a reasonable length to avoid
        // blowing system limits on limiting platforms like Windows.
        let objs: Vec<_> = objs
            .iter()
            .map(|o| o.dst.clone())
            .chain(self.objects.clone())
            .collect();
        for chunk in objs.chunks(100) {
            self.assemble_progressive(dst, chunk)?;
        }

        if self.cuda && self.cuda_file_count() > 0 {
            // Link the device-side code and add it to the target library,
            // so that non-CUDA linker can link the final binary.

            let out_dir = self.get_out_dir()?;
            let dlink = out_dir.join(lib_name.to_owned() + "_dlink.o");
            let mut nvcc = self.get_compiler().to_command();
            nvcc.arg("--device-link")
                .arg("-o")
                .arg(dlink.clone())
                .arg(dst);
            run(&mut nvcc, "nvcc")?;
            self.assemble_progressive(dst, &[dlink])?;
        }

        let target = self.get_target()?;
        if target.contains("msvc") {
            // The Rust compiler will look for libfoo.a and foo.lib, but the
            // MSVC linker will also be passed foo.lib, so be sure that both
            // exist for now.

            let lib_dst = dst.with_file_name(format!("{}.lib", lib_name));
            let _ = fs::remove_file(&lib_dst);
            match fs::hard_link(&dst, &lib_dst).or_else(|_| {
                // if hard-link fails, just copy (ignoring the number of bytes written)
                fs::copy(&dst, &lib_dst).map(|_| ())
            }) {
                Ok(_) => (),
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::IOError,
                        "Could not copy or create a hard-link to the generated lib file.",
                    ));
                }
            };
        } else {
            // Non-msvc targets (those using `ar`) need a separate step to add
            // the symbol table to archives since our construction command of
            // `cq` doesn't add it for us.
            let (mut ar, cmd, _any_flags) = self.get_ar()?;

            // NOTE: We add `s` even if flags were passed using $ARFLAGS/ar_flag, because `s`
            // here represents a _mode_, not an arbitrary flag. Further discussion of this choice
            // can be seen in https://github.com/rust-lang/cc-rs/pull/763.
            run(ar.arg("s").arg(dst), &cmd)?;
        }

        Ok(())
    }

    fn assemble_progressive(&self, dst: &Path, objs: &[PathBuf]) -> Result<(), Error> {
        let target = self.get_target()?;

        if target.contains("msvc") {
            let (mut cmd, program, any_flags) = self.get_ar()?;
            // NOTE: -out: here is an I/O flag, and so must be included even if $ARFLAGS/ar_flag is
            // in use. -nologo on the other hand is just a regular flag, and one that we'll skip if
            // the caller has explicitly dictated the flags they want. See
            // https://github.com/rust-lang/cc-rs/pull/763 for further discussion.
            let mut out = OsString::from("-out:");
            out.push(dst);
            cmd.arg(out);
            if !any_flags {
                cmd.arg("-nologo");
            }
            // If the library file already exists, add the library name
            // as an argument to let lib.exe know we are appending the objs.
            if dst.exists() {
                cmd.arg(dst);
            }
            cmd.args(objs);
            run(&mut cmd, &program)?;
        } else {
            let (mut ar, cmd, _any_flags) = self.get_ar()?;

            // Set an environment variable to tell the OSX archiver to ensure
            // that all dates listed in the archive are zero, improving
            // determinism of builds. AFAIK there's not really official
            // documentation of this but there's a lot of references to it if
            // you search google.
            //
            // You can reproduce this locally on a mac with:
            //
            //      $ touch foo.c
            //      $ cc -c foo.c -o foo.o
            //
            //      # Notice that these two checksums are different
            //      $ ar crus libfoo1.a foo.o && sleep 2 && ar crus libfoo2.a foo.o
            //      $ md5sum libfoo*.a
            //
            //      # Notice that these two checksums are the same
            //      $ export ZERO_AR_DATE=1
            //      $ ar crus libfoo1.a foo.o && sleep 2 && touch foo.o && ar crus libfoo2.a foo.o
            //      $ md5sum libfoo*.a
            //
            // In any case if this doesn't end up getting read, it shouldn't
            // cause that many issues!
            ar.env("ZERO_AR_DATE", "1");

            // NOTE: We add cq here regardless of whether $ARFLAGS/ar_flag have been used because
            // it dictates the _mode_ ar runs in, which the setter of $ARFLAGS/ar_flag can't
            // dictate. See https://github.com/rust-lang/cc-rs/pull/763 for further discussion.
            run(ar.arg("cq").arg(dst).args(objs), &cmd)?;
        }

        Ok(())
    }

    fn ios_watchos_flags(&self, cmd: &mut Tool) -> Result<(), Error> {
        enum ArchSpec {
            Device(&'static str),
            Simulator(&'static str),
            Catalyst(&'static str),
        }

        enum Os {
            Ios,
            WatchOs,
        }
        impl Display for Os {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    Os::Ios => f.write_str("iOS"),
                    Os::WatchOs => f.write_str("WatchOS"),
                }
            }
        }

        let target = self.get_target()?;
        let os = if target.contains("-watchos") {
            Os::WatchOs
        } else {
            Os::Ios
        };

        let arch = target.split('-').nth(0).ok_or_else(|| {
            Error::new(
                ErrorKind::ArchitectureInvalid,
                format!("Unknown architecture for {} target.", os).as_str(),
            )
        })?;

        let is_catalyst = match target.split('-').nth(3) {
            Some(v) => v == "macabi",
            None => false,
        };

        let is_sim = match target.split('-').nth(3) {
            Some(v) => v == "sim",
            None => false,
        };

        let arch = if is_catalyst {
            match arch {
                "arm64e" => ArchSpec::Catalyst("arm64e"),
                "arm64" | "aarch64" => ArchSpec::Catalyst("arm64"),
                "x86_64" => ArchSpec::Catalyst("-m64"),
                _ => {
                    return Err(Error::new(
                        ErrorKind::ArchitectureInvalid,
                        "Unknown architecture for iOS target.",
                    ));
                }
            }
        } else if is_sim {
            match arch {
                "arm64" | "aarch64" => ArchSpec::Simulator("arm64"),
                "x86_64" => ArchSpec::Simulator("-m64"),
                _ => {
                    return Err(Error::new(
                        ErrorKind::ArchitectureInvalid,
                        "Unknown architecture for iOS simulator target.",
                    ));
                }
            }
        } else {
            match arch {
                "arm" | "armv7" | "thumbv7" => ArchSpec::Device("armv7"),
                "armv7k" => ArchSpec::Device("armv7k"),
                "armv7s" | "thumbv7s" => ArchSpec::Device("armv7s"),
                "arm64e" => ArchSpec::Device("arm64e"),
                "arm64" | "aarch64" => ArchSpec::Device("arm64"),
                "arm64_32" => ArchSpec::Device("arm64_32"),
                "i386" | "i686" => ArchSpec::Simulator("-m32"),
                "x86_64" => ArchSpec::Simulator("-m64"),
                _ => {
                    return Err(Error::new(
                        ErrorKind::ArchitectureInvalid,
                        format!("Unknown architecture for {} target.", os).as_str(),
                    ));
                }
            }
        };

        let (sdk_prefix, sim_prefix, min_version) = match os {
            Os::Ios => (
                "iphone",
                "ios-",
                std::env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "7.0".into()),
            ),
            Os::WatchOs => (
                "watch",
                "watch",
                std::env::var("WATCHOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "2.0".into()),
            ),
        };

        let sdk = match arch {
            ArchSpec::Device(arch) => {
                cmd.args.push("-arch".into());
                cmd.args.push(arch.into());
                cmd.args
                    .push(format!("-m{}os-version-min={}", sdk_prefix, min_version).into());
                format!("{}os", sdk_prefix)
            }
            ArchSpec::Simulator(arch) => {
                if arch.starts_with('-') {
                    // -m32 or -m64
                    cmd.args.push(arch.into());
                } else {
                    cmd.args.push("-arch".into());
                    cmd.args.push(arch.into());
                }
                cmd.args
                    .push(format!("-m{}simulator-version-min={}", sim_prefix, min_version).into());
                format!("{}simulator", sdk_prefix)
            }
            ArchSpec::Catalyst(_) => "macosx".to_owned(),
        };

        self.print(&format!("Detecting {} SDK path for {}", os, sdk));
        let sdk_path = if let Some(sdkroot) = env::var_os("SDKROOT") {
            sdkroot
        } else {
            self.apple_sdk_root(sdk.as_str())?
        };

        cmd.args.push("-isysroot".into());
        cmd.args.push(sdk_path);
        // TODO: Remove this once Apple stops accepting apps built with Xcode 13
        cmd.args.push("-fembed-bitcode".into());

        Ok(())
    }

    fn cmd<P: AsRef<OsStr>>(&self, prog: P) -> Command {
        let mut cmd = Command::new(prog);
        for &(ref a, ref b) in self.env.iter() {
            cmd.env(a, b);
        }
        cmd
    }

    fn get_base_compiler(&self) -> Result<Tool, Error> {
        if let Some(ref c) = self.compiler {
            return Ok(Tool::new(c.clone()));
        }
        let host = self.get_host()?;
        let target = self.get_target()?;
        let (env, msvc, gnu, traditional, clang) = if self.cpp {
            ("CXX", "cl.exe", "g++", "c++", "clang++")
        } else {
            ("CC", "cl.exe", "gcc", "cc", "clang")
        };

        // On historical Solaris systems, "cc" may have been Sun Studio, which
        // is not flag-compatible with "gcc".  This history casts a long shadow,
        // and many modern illumos distributions today ship GCC as "gcc" without
        // also making it available as "cc".
        let default = if host.contains("solaris") || host.contains("illumos") {
            gnu
        } else {
            traditional
        };

        let cl_exe = windows_registry::find_tool(&target, "cl.exe");

        let tool_opt: Option<Tool> = self
            .env_tool(env)
            .map(|(tool, wrapper, args)| {
                // find the driver mode, if any
                const DRIVER_MODE: &str = "--driver-mode=";
                let driver_mode = args
                    .iter()
                    .find(|a| a.starts_with(DRIVER_MODE))
                    .map(|a| &a[DRIVER_MODE.len()..]);
                // Chop off leading/trailing whitespace to work around
                // semi-buggy build scripts which are shared in
                // makefiles/configure scripts (where spaces are far more
                // lenient)
                let mut t = Tool::with_clang_driver(PathBuf::from(tool.trim()), driver_mode);
                if let Some(cc_wrapper) = wrapper {
                    t.cc_wrapper_path = Some(PathBuf::from(cc_wrapper));
                }
                for arg in args {
                    t.cc_wrapper_args.push(arg.into());
                }
                t
            })
            .or_else(|| {
                if target.contains("emscripten") {
                    let tool = if self.cpp { "em++" } else { "emcc" };
                    // Windows uses bat file so we have to be a bit more specific
                    if cfg!(windows) {
                        let mut t = Tool::new(PathBuf::from("cmd"));
                        t.args.push("/c".into());
                        t.args.push(format!("{}.bat", tool).into());
                        Some(t)
                    } else {
                        Some(Tool::new(PathBuf::from(tool)))
                    }
                } else {
                    None
                }
            })
            .or_else(|| cl_exe.clone());

        let tool = match tool_opt {
            Some(t) => t,
            None => {
                let compiler = if host.contains("windows") && target.contains("windows") {
                    if target.contains("msvc") {
                        msvc.to_string()
                    } else {
                        let cc = if target.contains("llvm") { clang } else { gnu };
                        format!("{}.exe", cc)
                    }
                } else if target.contains("apple-ios") {
                    clang.to_string()
                } else if target.contains("apple-watchos") {
                    clang.to_string()
                } else if target.contains("android") {
                    autodetect_android_compiler(&target, &host, gnu, clang)
                } else if target.contains("cloudabi") {
                    format!("{}-{}", target, traditional)
                } else if target == "wasm32-wasi"
                    || target == "wasm32-unknown-wasi"
                    || target == "wasm32-unknown-unknown"
                {
                    "clang".to_string()
                } else if target.contains("vxworks") {
                    if self.cpp {
                        "wr-c++".to_string()
                    } else {
                        "wr-cc".to_string()
                    }
                } else if target.starts_with("armv7a-kmc-solid_") {
                    format!("arm-kmc-eabi-{}", gnu)
                } else if target.starts_with("aarch64-kmc-solid_") {
                    format!("aarch64-kmc-elf-{}", gnu)
                } else if self.get_host()? != target {
                    let prefix = self.prefix_for_target(&target);
                    match prefix {
                        Some(prefix) => {
                            let cc = if target.contains("llvm") { clang } else { gnu };
                            format!("{}-{}", prefix, cc)
                        }
                        None => default.to_string(),
                    }
                } else {
                    default.to_string()
                };

                let mut t = Tool::new(PathBuf::from(compiler));
                if let Some(cc_wrapper) = Self::rustc_wrapper_fallback() {
                    t.cc_wrapper_path = Some(PathBuf::from(cc_wrapper));
                }
                t
            }
        };

        let mut tool = if self.cuda {
            assert!(
                tool.args.is_empty(),
                "CUDA compilation currently assumes empty pre-existing args"
            );
            let nvcc = match self.get_var("NVCC") {
                Err(_) => "nvcc".into(),
                Ok(nvcc) => nvcc,
            };
            let mut nvcc_tool = Tool::with_features(PathBuf::from(nvcc), None, self.cuda);
            nvcc_tool
                .args
                .push(format!("-ccbin={}", tool.path.display()).into());
            nvcc_tool.family = tool.family;
            nvcc_tool
        } else {
            tool
        };

        // New "standalone" C/C++ cross-compiler executables from recent Android NDK
        // are just shell scripts that call main clang binary (from Android NDK) with
        // proper `--target` argument.
        //
        // For example, armv7a-linux-androideabi16-clang passes
        // `--target=armv7a-linux-androideabi16` to clang.
        //
        // As the shell script calls the main clang binary, the command line limit length
        // on Windows is restricted to around 8k characters instead of around 32k characters.
        // To remove this limit, we call the main clang binary directly and construct the
        // `--target=` ourselves.
        if host.contains("windows") && android_clang_compiler_uses_target_arg_internally(&tool.path)
        {
            if let Some(path) = tool.path.file_name() {
                let file_name = path.to_str().unwrap().to_owned();
                let (target, clang) = file_name.split_at(file_name.rfind("-").unwrap());

                tool.path.set_file_name(clang.trim_start_matches("-"));
                tool.path.set_extension("exe");
                tool.args.push(format!("--target={}", target).into());

                // Additionally, shell scripts for target i686-linux-android versions 16 to 24
                // pass the `mstackrealign` option so we do that here as well.
                if target.contains("i686-linux-android") {
                    let (_, version) = target.split_at(target.rfind("d").unwrap() + 1);
                    if let Ok(version) = version.parse::<u32>() {
                        if version > 15 && version < 25 {
                            tool.args.push("-mstackrealign".into());
                        }
                    }
                }
            };
        }

        // If we found `cl.exe` in our environment, the tool we're returning is
        // an MSVC-like tool, *and* no env vars were set then set env vars for
        // the tool that we're returning.
        //
        // Env vars are needed for things like `link.exe` being put into PATH as
        // well as header include paths sometimes. These paths are automatically
        // included by default but if the `CC` or `CXX` env vars are set these
        // won't be used. This'll ensure that when the env vars are used to
        // configure for invocations like `clang-cl` we still get a "works out
        // of the box" experience.
        if let Some(cl_exe) = cl_exe {
            if tool.family == (ToolFamily::Msvc { clang_cl: true })
                && tool.env.len() == 0
                && target.contains("msvc")
            {
                for &(ref k, ref v) in cl_exe.env.iter() {
                    tool.env.push((k.to_owned(), v.to_owned()));
                }
            }
        }

        Ok(tool)
    }

    fn get_var(&self, var_base: &str) -> Result<String, Error> {
        let target = self.get_target()?;
        let host = self.get_host()?;
        let kind = if host == target { "HOST" } else { "TARGET" };
        let target_u = target.replace("-", "_");
        let res = self
            .getenv(&format!("{}_{}", var_base, target))
            .or_else(|| self.getenv(&format!("{}_{}", var_base, target_u)))
            .or_else(|| self.getenv(&format!("{}_{}", kind, var_base)))
            .or_else(|| self.getenv(var_base));

        match res {
            Some(res) => Ok(res),
            None => Err(Error::new(
                ErrorKind::EnvVarNotFound,
                &format!("Could not find environment variable {}.", var_base),
            )),
        }
    }

    fn envflags(&self, name: &str) -> Vec<String> {
        self.get_var(name)
            .unwrap_or(String::new())
            .split_ascii_whitespace()
            .map(|slice| slice.to_string())
            .collect()
    }

    /// Returns a fallback `cc_compiler_wrapper` by introspecting `RUSTC_WRAPPER`
    fn rustc_wrapper_fallback() -> Option<String> {
        // No explicit CC wrapper was detected, but check if RUSTC_WRAPPER
        // is defined and is a build accelerator that is compatible with
        // C/C++ compilers (e.g. sccache)
        const VALID_WRAPPERS: &[&'static str] = &["sccache", "cachepot"];

        let rustc_wrapper = std::env::var_os("RUSTC_WRAPPER")?;
        let wrapper_path = Path::new(&rustc_wrapper);
        let wrapper_stem = wrapper_path.file_stem()?;

        if VALID_WRAPPERS.contains(&wrapper_stem.to_str()?) {
            Some(rustc_wrapper.to_str()?.to_owned())
        } else {
            None
        }
    }

    /// Returns compiler path, optional modifier name from whitelist, and arguments vec
    fn env_tool(&self, name: &str) -> Option<(String, Option<String>, Vec<String>)> {
        let tool = match self.get_var(name) {
            Ok(tool) => tool,
            Err(_) => return None,
        };

        // If this is an exact path on the filesystem we don't want to do any
        // interpretation at all, just pass it on through. This'll hopefully get
        // us to support spaces-in-paths.
        if Path::new(&tool).exists() {
            return Some((tool, None, Vec::new()));
        }

        // Ok now we want to handle a couple of scenarios. We'll assume from
        // here on out that spaces are splitting separate arguments. Two major
        // features we want to support are:
        //
        //      CC='sccache cc'
        //
        // aka using `sccache` or any other wrapper/caching-like-thing for
        // compilations. We want to know what the actual compiler is still,
        // though, because our `Tool` API support introspection of it to see
        // what compiler is in use.
        //
        // additionally we want to support
        //
        //      CC='cc -flag'
        //
        // where the CC env var is used to also pass default flags to the C
        // compiler.
        //
        // It's true that everything here is a bit of a pain, but apparently if
        // you're not literally make or bash then you get a lot of bug reports.
        let known_wrappers = ["ccache", "distcc", "sccache", "icecc", "cachepot"];

        let mut parts = tool.split_whitespace();
        let maybe_wrapper = match parts.next() {
            Some(s) => s,
            None => return None,
        };

        let file_stem = Path::new(maybe_wrapper)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        if known_wrappers.contains(&file_stem) {
            if let Some(compiler) = parts.next() {
                return Some((
                    compiler.to_string(),
                    Some(maybe_wrapper.to_string()),
                    parts.map(|s| s.to_string()).collect(),
                ));
            }
        }

        Some((
            maybe_wrapper.to_string(),
            Self::rustc_wrapper_fallback(),
            parts.map(|s| s.to_string()).collect(),
        ))
    }

    /// Returns the C++ standard library:
    /// 1. If [cpp_link_stdlib](cc::Build::cpp_link_stdlib) is set, uses its value.
    /// 2. Else if the `CXXSTDLIB` environment variable is set, uses its value.
    /// 3. Else the default is `libc++` for OS X and BSDs, `libc++_shared` for Android,
    /// `None` for MSVC and `libstdc++` for anything else.
    fn get_cpp_link_stdlib(&self) -> Result<Option<String>, Error> {
        match self.cpp_link_stdlib.clone() {
            Some(s) => Ok(s),
            None => {
                if let Ok(stdlib) = self.get_var("CXXSTDLIB") {
                    if stdlib.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(stdlib))
                    }
                } else {
                    let target = self.get_target()?;
                    if target.contains("msvc") {
                        Ok(None)
                    } else if target.contains("apple") {
                        Ok(Some("c++".to_string()))
                    } else if target.contains("freebsd") {
                        Ok(Some("c++".to_string()))
                    } else if target.contains("openbsd") {
                        Ok(Some("c++".to_string()))
                    } else if target.contains("android") {
                        Ok(Some("c++_shared".to_string()))
                    } else {
                        Ok(Some("stdc++".to_string()))
                    }
                }
            }
        }
    }

    fn get_ar(&self) -> Result<(Command, String, bool), Error> {
        self.try_get_archiver_and_flags()
    }

    /// Get the archiver (ar) that's in use for this configuration.
    ///
    /// You can use [`Command::get_program`] to get just the path to the command.
    ///
    /// This method will take into account all configuration such as debug
    /// information, optimization level, include directories, defines, etc.
    /// Additionally, the compiler binary in use follows the standard
    /// conventions for this path, e.g. looking at the explicitly set compiler,
    /// environment variables (a number of which are inspected here), and then
    /// falling back to the default configuration.
    ///
    /// # Panics
    ///
    /// Panics if an error occurred while determining the architecture.
    pub fn get_archiver(&self) -> Command {
        match self.try_get_archiver() {
            Ok(tool) => tool,
            Err(e) => fail(&e.message),
        }
    }

    /// Get the archiver that's in use for this configuration.
    ///
    /// This will return a result instead of panicing;
    /// see [`get_archiver()`] for the complete description.
    pub fn try_get_archiver(&self) -> Result<Command, Error> {
        Ok(self.try_get_archiver_and_flags()?.0)
    }

    fn try_get_archiver_and_flags(&self) -> Result<(Command, String, bool), Error> {
        let (mut cmd, name) = self.get_base_archiver()?;
        let flags = self.envflags("ARFLAGS");
        let mut any_flags = !flags.is_empty();
        cmd.args(flags);
        for flag in &self.ar_flags {
            any_flags = true;
            cmd.arg(flag);
        }
        Ok((cmd, name, any_flags))
    }

    fn get_base_archiver(&self) -> Result<(Command, String), Error> {
        if let Some(ref a) = self.archiver {
            return Ok((self.cmd(a), a.to_string_lossy().into_owned()));
        }

        self.get_base_archiver_variant("AR", "ar")
    }

    /// Get the ranlib that's in use for this configuration.
    ///
    /// You can use [`Command::get_program`] to get just the path to the command.
    ///
    /// This method will take into account all configuration such as debug
    /// information, optimization level, include directories, defines, etc.
    /// Additionally, the compiler binary in use follows the standard
    /// conventions for this path, e.g. looking at the explicitly set compiler,
    /// environment variables (a number of which are inspected here), and then
    /// falling back to the default configuration.
    ///
    /// # Panics
    ///
    /// Panics if an error occurred while determining the architecture.
    pub fn get_ranlib(&self) -> Command {
        match self.try_get_ranlib() {
            Ok(tool) => tool,
            Err(e) => fail(&e.message),
        }
    }

    /// Get the ranlib that's in use for this configuration.
    ///
    /// This will return a result instead of panicing;
    /// see [`get_ranlib()`] for the complete description.
    pub fn try_get_ranlib(&self) -> Result<Command, Error> {
        let mut cmd = self.get_base_ranlib()?;
        cmd.args(self.envflags("RANLIBFLAGS"));
        Ok(cmd)
    }

    fn get_base_ranlib(&self) -> Result<Command, Error> {
        if let Some(ref r) = self.ranlib {
            return Ok(self.cmd(r));
        }

        Ok(self.get_base_archiver_variant("RANLIB", "ranlib")?.0)
    }

    fn get_base_archiver_variant(&self, env: &str, tool: &str) -> Result<(Command, String), Error> {
        let target = self.get_target()?;
        let mut name = String::new();
        let tool_opt: Option<Command> = self
            .env_tool(env)
            .map(|(tool, _wrapper, args)| {
                let mut cmd = self.cmd(tool);
                cmd.args(args);
                cmd
            })
            .or_else(|| {
                if target.contains("emscripten") {
                    // Windows use bat files so we have to be a bit more specific
                    if cfg!(windows) {
                        let mut cmd = self.cmd("cmd");
                        name = format!("em{}.bat", tool);
                        cmd.arg("/c").arg(&name);
                        Some(cmd)
                    } else {
                        name = format!("em{}", tool);
                        Some(self.cmd(&name))
                    }
                } else {
                    None
                }
            });

        let default = tool.to_string();
        let tool = match tool_opt {
            Some(t) => t,
            None => {
                if target.contains("android") {
                    name = format!("{}-{}", target.replace("armv7", "arm"), tool);
                    self.cmd(&name)
                } else if target.contains("msvc") {
                    // NOTE: There isn't really a ranlib on msvc, so arguably we should return
                    // `None` somehow here. But in general, callers will already have to be aware
                    // of not running ranlib on Windows anyway, so it feels okay to return lib.exe
                    // here.

                    let compiler = self.get_base_compiler()?;
                    let mut lib = String::new();
                    if compiler.family == (ToolFamily::Msvc { clang_cl: true }) {
                        // See if there is 'llvm-lib' next to 'clang-cl'
                        // Another possibility could be to see if there is 'clang'
                        // next to 'clang-cl' and use 'search_programs()' to locate
                        // 'llvm-lib'. This is because 'clang-cl' doesn't support
                        // the -print-search-dirs option.
                        if let Some(mut cmd) = which(&compiler.path) {
                            cmd.pop();
                            cmd.push("llvm-lib.exe");
                            if let Some(llvm_lib) = which(&cmd) {
                                lib = llvm_lib.to_str().unwrap().to_owned();
                            }
                        }
                    }

                    if lib.is_empty() {
                        name = String::from("lib.exe");
                        match windows_registry::find(&target, "lib.exe") {
                            Some(t) => t,
                            None => self.cmd("lib.exe"),
                        }
                    } else {
                        name = lib;
                        self.cmd(&name)
                    }
                } else if target.contains("illumos") {
                    // The default 'ar' on illumos uses a non-standard flags,
                    // but the OS comes bundled with a GNU-compatible variant.
                    //
                    // Use the GNU-variant to match other Unix systems.
                    name = format!("g{}", tool);
                    self.cmd(&name)
                } else if self.get_host()? != target {
                    match self.prefix_for_target(&target) {
                        Some(p) => {
                            // GCC uses $target-gcc-ar, whereas binutils uses $target-ar -- try both.
                            // Prefer -ar if it exists, as builds of `-gcc-ar` have been observed to be
                            // outright broken (such as when targetting freebsd with `--disable-lto`
                            // toolchain where the archiver attempts to load the LTO plugin anyway but
                            // fails to find one).
                            //
                            // The same applies to ranlib.
                            let mut chosen = default;
                            for &infix in &["", "-gcc"] {
                                let target_p = format!("{}{}-{}", p, infix, tool);
                                if Command::new(&target_p).output().is_ok() {
                                    chosen = target_p;
                                    break;
                                }
                            }
                            name = chosen;
                            self.cmd(&name)
                        }
                        None => {
                            name = default;
                            self.cmd(&name)
                        }
                    }
                } else {
                    name = default;
                    self.cmd(&name)
                }
            }
        };

        Ok((tool, name))
    }

    fn prefix_for_target(&self, target: &str) -> Option<String> {
        // Put aside RUSTC_LINKER's prefix to be used as last resort
        let rustc_linker = self.getenv("RUSTC_LINKER").unwrap_or("".to_string());
        // let linker_prefix = rustc_linker.strip_suffix("-gcc"); // >=1.45.0
        let linker_prefix = if rustc_linker.len() > 4 {
            let (prefix, suffix) = rustc_linker.split_at(rustc_linker.len() - 4);
            if suffix == "-gcc" {
                Some(prefix)
            } else {
                None
            }
        } else {
            None
        };
        // CROSS_COMPILE is of the form: "arm-linux-gnueabi-"
        let cc_env = self.getenv("CROSS_COMPILE");
        let cross_compile = cc_env.as_ref().map(|s| s.trim_end_matches('-').to_owned());
        cross_compile.or(match &target[..] {
            // Note: there is no `aarch64-pc-windows-gnu` target, only `-gnullvm`
            "aarch64-pc-windows-gnullvm" => Some("aarch64-w64-mingw32"),
            "aarch64-uwp-windows-gnu" => Some("aarch64-w64-mingw32"),
            "aarch64-unknown-linux-gnu" => Some("aarch64-linux-gnu"),
            "aarch64-unknown-linux-musl" => Some("aarch64-linux-musl"),
            "aarch64-unknown-netbsd" => Some("aarch64--netbsd"),
            "arm-unknown-linux-gnueabi" => Some("arm-linux-gnueabi"),
            "armv4t-unknown-linux-gnueabi" => Some("arm-linux-gnueabi"),
            "armv5te-unknown-linux-gnueabi" => Some("arm-linux-gnueabi"),
            "armv5te-unknown-linux-musleabi" => Some("arm-linux-gnueabi"),
            "arm-frc-linux-gnueabi" => Some("arm-frc-linux-gnueabi"),
            "arm-unknown-linux-gnueabihf" => Some("arm-linux-gnueabihf"),
            "arm-unknown-linux-musleabi" => Some("arm-linux-musleabi"),
            "arm-unknown-linux-musleabihf" => Some("arm-linux-musleabihf"),
            "arm-unknown-netbsd-eabi" => Some("arm--netbsdelf-eabi"),
            "armv6-unknown-netbsd-eabihf" => Some("armv6--netbsdelf-eabihf"),
            "armv7-unknown-linux-gnueabi" => Some("arm-linux-gnueabi"),
            "armv7-unknown-linux-gnueabihf" => Some("arm-linux-gnueabihf"),
            "armv7-unknown-linux-musleabihf" => Some("arm-linux-musleabihf"),
            "armv7neon-unknown-linux-gnueabihf" => Some("arm-linux-gnueabihf"),
            "armv7neon-unknown-linux-musleabihf" => Some("arm-linux-musleabihf"),
            "thumbv7-unknown-linux-gnueabihf" => Some("arm-linux-gnueabihf"),
            "thumbv7-unknown-linux-musleabihf" => Some("arm-linux-musleabihf"),
            "thumbv7neon-unknown-linux-gnueabihf" => Some("arm-linux-gnueabihf"),
            "thumbv7neon-unknown-linux-musleabihf" => Some("arm-linux-musleabihf"),
            "armv7-unknown-netbsd-eabihf" => Some("armv7--netbsdelf-eabihf"),
            "hexagon-unknown-linux-musl" => Some("hexagon-linux-musl"),
            "i586-unknown-linux-musl" => Some("musl"),
            "i686-pc-windows-gnu" => Some("i686-w64-mingw32"),
            "i686-uwp-windows-gnu" => Some("i686-w64-mingw32"),
            "i686-unknown-linux-gnu" => self.find_working_gnu_prefix(&[
                "i686-linux-gnu",
                "x86_64-linux-gnu", // transparently support gcc-multilib
            ]), // explicit None if not found, so caller knows to fall back
            "i686-unknown-linux-musl" => Some("musl"),
            "i686-unknown-netbsd" => Some("i486--netbsdelf"),
            "mips-unknown-linux-gnu" => Some("mips-linux-gnu"),
            "mips-unknown-linux-musl" => Some("mips-linux-musl"),
            "mipsel-unknown-linux-gnu" => Some("mipsel-linux-gnu"),
            "mipsel-unknown-linux-musl" => Some("mipsel-linux-musl"),
            "mips64-unknown-linux-gnuabi64" => Some("mips64-linux-gnuabi64"),
            "mips64el-unknown-linux-gnuabi64" => Some("mips64el-linux-gnuabi64"),
            "mipsisa32r6-unknown-linux-gnu" => Some("mipsisa32r6-linux-gnu"),
            "mipsisa32r6el-unknown-linux-gnu" => Some("mipsisa32r6el-linux-gnu"),
            "mipsisa64r6-unknown-linux-gnuabi64" => Some("mipsisa64r6-linux-gnuabi64"),
            "mipsisa64r6el-unknown-linux-gnuabi64" => Some("mipsisa64r6el-linux-gnuabi64"),
            "powerpc-unknown-linux-gnu" => Some("powerpc-linux-gnu"),
            "powerpc-unknown-linux-gnuspe" => Some("powerpc-linux-gnuspe"),
            "powerpc-unknown-netbsd" => Some("powerpc--netbsd"),
            "powerpc64-unknown-linux-gnu" => Some("powerpc-linux-gnu"),
            "powerpc64le-unknown-linux-gnu" => Some("powerpc64le-linux-gnu"),
            "riscv32i-unknown-none-elf" => self.find_working_gnu_prefix(&[
                "riscv32-unknown-elf",
                "riscv64-unknown-elf",
                "riscv-none-embed",
            ]),
            "riscv32imac-unknown-none-elf" => self.find_working_gnu_prefix(&[
                "riscv32-unknown-elf",
                "riscv64-unknown-elf",
                "riscv-none-embed",
            ]),
            "riscv32imac-unknown-xous-elf" => self.find_working_gnu_prefix(&[
                "riscv32-unknown-elf",
                "riscv64-unknown-elf",
                "riscv-none-embed",
            ]),
            "riscv32imc-esp-espidf" => Some("riscv32-esp-elf"),
            "riscv32imc-unknown-none-elf" => self.find_working_gnu_prefix(&[
                "riscv32-unknown-elf",
                "riscv64-unknown-elf",
                "riscv-none-embed",
            ]),
            "riscv64gc-unknown-none-elf" => self.find_working_gnu_prefix(&[
                "riscv64-unknown-elf",
                "riscv32-unknown-elf",
                "riscv-none-embed",
            ]),
            "riscv64imac-unknown-none-elf" => self.find_working_gnu_prefix(&[
                "riscv64-unknown-elf",
                "riscv32-unknown-elf",
                "riscv-none-embed",
            ]),
            "riscv64gc-unknown-linux-gnu" => Some("riscv64-linux-gnu"),
            "riscv32gc-unknown-linux-gnu" => Some("riscv32-linux-gnu"),
            "riscv64gc-unknown-linux-musl" => Some("riscv64-linux-musl"),
            "riscv32gc-unknown-linux-musl" => Some("riscv32-linux-musl"),
            "s390x-unknown-linux-gnu" => Some("s390x-linux-gnu"),
            "sparc-unknown-linux-gnu" => Some("sparc-linux-gnu"),
            "sparc64-unknown-linux-gnu" => Some("sparc64-linux-gnu"),
            "sparc64-unknown-netbsd" => Some("sparc64--netbsd"),
            "sparcv9-sun-solaris" => Some("sparcv9-sun-solaris"),
            "armv7a-none-eabi" => Some("arm-none-eabi"),
            "armv7a-none-eabihf" => Some("arm-none-eabi"),
            "armebv7r-none-eabi" => Some("arm-none-eabi"),
            "armebv7r-none-eabihf" => Some("arm-none-eabi"),
            "armv7r-none-eabi" => Some("arm-none-eabi"),
            "armv7r-none-eabihf" => Some("arm-none-eabi"),
            "thumbv6m-none-eabi" => Some("arm-none-eabi"),
            "thumbv7em-none-eabi" => Some("arm-none-eabi"),
            "thumbv7em-none-eabihf" => Some("arm-none-eabi"),
            "thumbv7m-none-eabi" => Some("arm-none-eabi"),
            "thumbv8m.base-none-eabi" => Some("arm-none-eabi"),
            "thumbv8m.main-none-eabi" => Some("arm-none-eabi"),
            "thumbv8m.main-none-eabihf" => Some("arm-none-eabi"),
            "x86_64-pc-windows-gnu" => Some("x86_64-w64-mingw32"),
            "x86_64-pc-windows-gnullvm" => Some("x86_64-w64-mingw32"),
            "x86_64-uwp-windows-gnu" => Some("x86_64-w64-mingw32"),
            "x86_64-rumprun-netbsd" => Some("x86_64-rumprun-netbsd"),
            "x86_64-unknown-linux-gnu" => self.find_working_gnu_prefix(&[
                "x86_64-linux-gnu", // rustfmt wrap
            ]), // explicit None if not found, so caller knows to fall back
            "x86_64-unknown-linux-musl" => Some("musl"),
            "x86_64-unknown-netbsd" => Some("x86_64--netbsd"),
            _ => linker_prefix,
        }
        .map(|x| x.to_owned()))
    }

    /// Some platforms have multiple, compatible, canonical prefixes. Look through
    /// each possible prefix for a compiler that exists and return it. The prefixes
    /// should be ordered from most-likely to least-likely.
    fn find_working_gnu_prefix(&self, prefixes: &[&'static str]) -> Option<&'static str> {
        let suffix = if self.cpp { "-g++" } else { "-gcc" };
        let extension = std::env::consts::EXE_SUFFIX;

        // Loop through PATH entries searching for each toolchain. This ensures that we
        // are more likely to discover the toolchain early on, because chances are good
        // that the desired toolchain is in one of the higher-priority paths.
        env::var_os("PATH")
            .as_ref()
            .and_then(|path_entries| {
                env::split_paths(path_entries).find_map(|path_entry| {
                    for prefix in prefixes {
                        let target_compiler = format!("{}{}{}", prefix, suffix, extension);
                        if path_entry.join(&target_compiler).exists() {
                            return Some(prefix);
                        }
                    }
                    None
                })
            })
            .map(|prefix| *prefix)
            .or_else(||
            // If no toolchain was found, provide the first toolchain that was passed in.
            // This toolchain has been shown not to exist, however it will appear in the
            // error that is shown to the user which should make it easier to search for
            // where it should be obtained.
            prefixes.first().map(|prefix| *prefix))
    }

    fn get_target(&self) -> Result<String, Error> {
        match self.target.clone() {
            Some(t) => Ok(t),
            None => Ok(self.getenv_unwrap("TARGET")?),
        }
    }

    fn get_host(&self) -> Result<String, Error> {
        match self.host.clone() {
            Some(h) => Ok(h),
            None => Ok(self.getenv_unwrap("HOST")?),
        }
    }

    fn get_opt_level(&self) -> Result<String, Error> {
        match self.opt_level.as_ref().cloned() {
            Some(ol) => Ok(ol),
            None => Ok(self.getenv_unwrap("OPT_LEVEL")?),
        }
    }

    fn get_debug(&self) -> bool {
        self.debug.unwrap_or_else(|| match self.getenv("DEBUG") {
            Some(s) => s != "false",
            None => false,
        })
    }

    fn get_dwarf_version(&self) -> Option<u32> {
        // Tentatively matches the DWARF version defaults as of rustc 1.62.
        let target = self.get_target().ok()?;
        if target.contains("android")
            || target.contains("apple")
            || target.contains("dragonfly")
            || target.contains("freebsd")
            || target.contains("netbsd")
            || target.contains("openbsd")
            || target.contains("windows-gnu")
        {
            Some(2)
        } else if target.contains("linux") {
            Some(4)
        } else {
            None
        }
    }

    fn get_force_frame_pointer(&self) -> bool {
        self.force_frame_pointer.unwrap_or_else(|| self.get_debug())
    }

    fn get_out_dir(&self) -> Result<PathBuf, Error> {
        match self.out_dir.clone() {
            Some(p) => Ok(p),
            None => Ok(env::var_os("OUT_DIR").map(PathBuf::from).ok_or_else(|| {
                Error::new(
                    ErrorKind::EnvVarNotFound,
                    "Environment variable OUT_DIR not defined.",
                )
            })?),
        }
    }

    fn getenv(&self, v: &str) -> Option<String> {
        // Returns true for environment variables cargo sets for build scripts:
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
        //
        // This handles more of the vars than we actually use (it tries to check
        // complete-ish set), just to avoid needing maintenance if/when new
        // calls to `getenv`/`getenv_unwrap` are added.
        fn provided_by_cargo(envvar: &str) -> bool {
            match envvar {
                v if v.starts_with("CARGO") || v.starts_with("RUSTC") => true,
                "HOST" | "TARGET" | "RUSTDOC" | "OUT_DIR" | "OPT_LEVEL" | "DEBUG" | "PROFILE"
                | "NUM_JOBS" | "RUSTFLAGS" => true,
                _ => false,
            }
        }
        let mut cache = self.env_cache.lock().unwrap();
        if let Some(val) = cache.get(v) {
            return val.clone();
        }
        if self.emit_rerun_if_env_changed && !provided_by_cargo(v) {
            self.print(&format!("cargo:rerun-if-env-changed={}", v));
        }
        let r = env::var(v).ok();
        self.print(&format!("{} = {:?}", v, r));
        cache.insert(v.to_string(), r.clone());
        r
    }

    fn getenv_unwrap(&self, v: &str) -> Result<String, Error> {
        match self.getenv(v) {
            Some(s) => Ok(s),
            None => Err(Error::new(
                ErrorKind::EnvVarNotFound,
                &format!("Environment variable {} not defined.", v.to_string()),
            )),
        }
    }

    fn print(&self, s: &str) {
        if self.cargo_metadata {
            println!("{}", s);
        }
    }

    fn fix_env_for_apple_os(&self, cmd: &mut Command) -> Result<(), Error> {
        let target = self.get_target()?;
        let host = self.get_host()?;
        if host.contains("apple-darwin") && target.contains("apple-darwin") {
            // If, for example, `cargo` runs during the build of an XCode project, then `SDKROOT` environment variable
            // would represent the current target, and this is the problem for us, if we want to compile something
            // for the host, when host != target.
            // We can not just remove `SDKROOT`, because, again, for example, XCode add to PATH
            // /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin
            // and `cc` from this path can not find system include files, like `pthread.h`, if `SDKROOT`
            // is not set
            if let Ok(sdkroot) = env::var("SDKROOT") {
                if !sdkroot.contains("MacOSX") {
                    let macos_sdk = self.apple_sdk_root("macosx")?;
                    cmd.env("SDKROOT", macos_sdk);
                }
            }
            // Additionally, `IPHONEOS_DEPLOYMENT_TARGET` must not be set when using the Xcode linker at
            // "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld",
            // although this is apparently ignored when using the linker at "/usr/bin/ld".
            cmd.env_remove("IPHONEOS_DEPLOYMENT_TARGET");
        }
        Ok(())
    }

    fn apple_sdk_root(&self, sdk: &str) -> Result<OsString, Error> {
        let mut cache = self
            .apple_sdk_root_cache
            .lock()
            .expect("apple_sdk_root_cache lock failed");
        if let Some(ret) = cache.get(sdk) {
            return Ok(ret.clone());
        }

        let sdk_path = run_output(
            self.cmd("xcrun")
                .arg("--show-sdk-path")
                .arg("--sdk")
                .arg(sdk),
            "xcrun",
        )?;

        let sdk_path = match String::from_utf8(sdk_path) {
            Ok(p) => p,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::IOError,
                    "Unable to determine Apple SDK path.",
                ));
            }
        };
        let ret: OsString = sdk_path.trim().into();
        cache.insert(sdk.into(), ret.clone());
        Ok(ret)
    }

    fn cuda_file_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.extension() == Some(OsStr::new("cu")))
            .count()
    }
}

impl Default for Build {
    fn default() -> Build {
        Build::new()
    }
}

impl Tool {
    fn new(path: PathBuf) -> Self {
        Tool::with_features(path, None, false)
    }

    fn with_clang_driver(path: PathBuf, clang_driver: Option<&str>) -> Self {
        Self::with_features(path, clang_driver, false)
    }

    #[cfg(windows)]
    /// Explicitly set the `ToolFamily`, skipping name-based detection.
    fn with_family(path: PathBuf, family: ToolFamily) -> Self {
        Self {
            path: path,
            cc_wrapper_path: None,
            cc_wrapper_args: Vec::new(),
            args: Vec::new(),
            env: Vec::new(),
            family: family,
            cuda: false,
            removed_args: Vec::new(),
        }
    }

    fn with_features(path: PathBuf, clang_driver: Option<&str>, cuda: bool) -> Self {
        // Try to detect family of the tool from its name, falling back to Gnu.
        let family = if let Some(fname) = path.file_name().and_then(|p| p.to_str()) {
            if fname.contains("clang-cl") {
                ToolFamily::Msvc { clang_cl: true }
            } else if fname.ends_with("cl") || fname == "cl.exe" {
                ToolFamily::Msvc { clang_cl: false }
            } else if fname.contains("clang") {
                match clang_driver {
                    Some("cl") => ToolFamily::Msvc { clang_cl: true },
                    _ => ToolFamily::Clang,
                }
            } else {
                ToolFamily::Gnu
            }
        } else {
            ToolFamily::Gnu
        };

        Tool {
            path: path,
            cc_wrapper_path: None,
            cc_wrapper_args: Vec::new(),
            args: Vec::new(),
            env: Vec::new(),
            family: family,
            cuda: cuda,
            removed_args: Vec::new(),
        }
    }

    /// Add an argument to be stripped from the final command arguments.
    fn remove_arg(&mut self, flag: OsString) {
        self.removed_args.push(flag);
    }

    /// Add a flag, and optionally prepend the NVCC wrapper flag "-Xcompiler".
    ///
    /// Currently this is only used for compiling CUDA sources, since NVCC only
    /// accepts a limited set of GNU-like flags, and the rest must be prefixed
    /// with a "-Xcompiler" flag to get passed to the underlying C++ compiler.
    fn push_cc_arg(&mut self, flag: OsString) {
        if self.cuda {
            self.args.push("-Xcompiler".into());
        }
        self.args.push(flag);
    }

    fn is_duplicate_opt_arg(&self, flag: &OsString) -> bool {
        let flag = flag.to_str().unwrap();
        let mut chars = flag.chars();

        // Only duplicate check compiler flags
        if self.is_like_msvc() {
            if chars.next() != Some('/') {
                return false;
            }
        } else if self.is_like_gnu() || self.is_like_clang() {
            if chars.next() != Some('-') {
                return false;
            }
        }

        // Check for existing optimization flags (-O, /O)
        if chars.next() == Some('O') {
            return self
                .args()
                .iter()
                .any(|ref a| a.to_str().unwrap_or("").chars().nth(1) == Some('O'));
        }

        // TODO Check for existing -m..., -m...=..., /arch:... flags
        return false;
    }

    /// Don't push optimization arg if it conflicts with existing args
    fn push_opt_unless_duplicate(&mut self, flag: OsString) {
        if self.is_duplicate_opt_arg(&flag) {
            println!("Info: Ignoring duplicate arg {:?}", &flag);
        } else {
            self.push_cc_arg(flag);
        }
    }

    /// Converts this compiler into a `Command` that's ready to be run.
    ///
    /// This is useful for when the compiler needs to be executed and the
    /// command returned will already have the initial arguments and environment
    /// variables configured.
    pub fn to_command(&self) -> Command {
        let mut cmd = match self.cc_wrapper_path {
            Some(ref cc_wrapper_path) => {
                let mut cmd = Command::new(&cc_wrapper_path);
                cmd.arg(&self.path);
                cmd
            }
            None => Command::new(&self.path),
        };
        cmd.args(&self.cc_wrapper_args);

        let value = self
            .args
            .iter()
            .filter(|a| !self.removed_args.contains(a))
            .collect::<Vec<_>>();
        cmd.args(&value);

        for &(ref k, ref v) in self.env.iter() {
            cmd.env(k, v);
        }
        cmd
    }

    /// Returns the path for this compiler.
    ///
    /// Note that this may not be a path to a file on the filesystem, e.g. "cc",
    /// but rather something which will be resolved when a process is spawned.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the default set of arguments to the compiler needed to produce
    /// executables for the target this compiler generates.
    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    /// Returns the set of environment variables needed for this compiler to
    /// operate.
    ///
    /// This is typically only used for MSVC compilers currently.
    pub fn env(&self) -> &[(OsString, OsString)] {
        &self.env
    }

    /// Returns the compiler command in format of CC environment variable.
    /// Or empty string if CC env was not present
    ///
    /// This is typically used by configure script
    pub fn cc_env(&self) -> OsString {
        match self.cc_wrapper_path {
            Some(ref cc_wrapper_path) => {
                let mut cc_env = cc_wrapper_path.as_os_str().to_owned();
                cc_env.push(" ");
                cc_env.push(self.path.to_path_buf().into_os_string());
                for arg in self.cc_wrapper_args.iter() {
                    cc_env.push(" ");
                    cc_env.push(arg);
                }
                cc_env
            }
            None => OsString::from(""),
        }
    }

    /// Returns the compiler flags in format of CFLAGS environment variable.
    /// Important here - this will not be CFLAGS from env, its internal gcc's flags to use as CFLAGS
    /// This is typically used by configure script
    pub fn cflags_env(&self) -> OsString {
        let mut flags = OsString::new();
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                flags.push(" ");
            }
            flags.push(arg);
        }
        flags
    }

    /// Whether the tool is GNU Compiler Collection-like.
    pub fn is_like_gnu(&self) -> bool {
        self.family == ToolFamily::Gnu
    }

    /// Whether the tool is Clang-like.
    pub fn is_like_clang(&self) -> bool {
        self.family == ToolFamily::Clang
    }

    /// Whether the tool is MSVC-like.
    pub fn is_like_msvc(&self) -> bool {
        match self.family {
            ToolFamily::Msvc { .. } => true,
            _ => false,
        }
    }
}

fn run(cmd: &mut Command, program: &str) -> Result<(), Error> {
    let (mut child, print) = spawn(cmd, program)?;
    let status = match child.wait() {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::ToolExecError,
                &format!(
                    "Failed to wait on spawned child process, command {:?} with args {:?}.",
                    cmd, program
                ),
            ));
        }
    };
    print.join().unwrap();
    println!("{}", status);

    if status.success() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::ToolExecError,
            &format!(
                "Command {:?} with args {:?} did not execute successfully (status code {}).",
                cmd, program, status
            ),
        ))
    }
}

fn run_output(cmd: &mut Command, program: &str) -> Result<Vec<u8>, Error> {
    cmd.stdout(Stdio::piped());
    let (mut child, print) = spawn(cmd, program)?;
    let mut stdout = vec![];
    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    let status = match child.wait() {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::ToolExecError,
                &format!(
                    "Failed to wait on spawned child process, command {:?} with args {:?}.",
                    cmd, program
                ),
            ));
        }
    };
    print.join().unwrap();
    println!("{}", status);

    if status.success() {
        Ok(stdout)
    } else {
        Err(Error::new(
            ErrorKind::ToolExecError,
            &format!(
                "Command {:?} with args {:?} did not execute successfully (status code {}).",
                cmd, program, status
            ),
        ))
    }
}

fn spawn(cmd: &mut Command, program: &str) -> Result<(Child, JoinHandle<()>), Error> {
    println!("running: {:?}", cmd);

    // Capture the standard error coming from these programs, and write it out
    // with cargo:warning= prefixes. Note that this is a bit wonky to avoid
    // requiring the output to be UTF-8, we instead just ship bytes from one
    // location to another.
    match cmd.stderr(Stdio::piped()).spawn() {
        Ok(mut child) => {
            let stderr = BufReader::new(child.stderr.take().unwrap());
            let print = thread::spawn(move || {
                for line in stderr.split(b'\n').filter_map(|l| l.ok()) {
                    print!("cargo:warning=");
                    std::io::stdout().write_all(&line).unwrap();
                    println!("");
                }
            });
            Ok((child, print))
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            let extra = if cfg!(windows) {
                " (see https://github.com/rust-lang/cc-rs#compile-time-requirements \
                 for help)"
            } else {
                ""
            };
            Err(Error::new(
                ErrorKind::ToolNotFound,
                &format!("Failed to find tool. Is `{}` installed?{}", program, extra),
            ))
        }
        Err(ref e) => Err(Error::new(
            ErrorKind::ToolExecError,
            &format!(
                "Command {:?} with args {:?} failed to start: {:?}",
                cmd, program, e
            ),
        )),
    }
}

fn fail(s: &str) -> ! {
    eprintln!("\n\nerror occurred: {}\n\n", s);
    std::process::exit(1);
}

fn command_add_output_file(
    cmd: &mut Command,
    dst: &Path,
    cuda: bool,
    msvc: bool,
    clang: bool,
    is_asm: bool,
    is_arm: bool,
) {
    if msvc && !clang && !cuda && !(is_asm && is_arm) {
        let mut s = OsString::from("-Fo");
        s.push(&dst);
        cmd.arg(s);
    } else {
        cmd.arg("-o").arg(&dst);
    }
}

// Use by default minimum available API level
// See note about naming here
// https://android.googlesource.com/platform/ndk/+/refs/heads/ndk-release-r21/docs/BuildSystemMaintainers.md#Clang
static NEW_STANDALONE_ANDROID_COMPILERS: [&str; 4] = [
    "aarch64-linux-android21-clang",
    "armv7a-linux-androideabi16-clang",
    "i686-linux-android16-clang",
    "x86_64-linux-android21-clang",
];

// New "standalone" C/C++ cross-compiler executables from recent Android NDK
// are just shell scripts that call main clang binary (from Android NDK) with
// proper `--target` argument.
//
// For example, armv7a-linux-androideabi16-clang passes
// `--target=armv7a-linux-androideabi16` to clang.
// So to construct proper command line check if
// `--target` argument would be passed or not to clang
fn android_clang_compiler_uses_target_arg_internally(clang_path: &Path) -> bool {
    if let Some(filename) = clang_path.file_name() {
        if let Some(filename_str) = filename.to_str() {
            filename_str.contains("android")
        } else {
            false
        }
    } else {
        false
    }
}

#[test]
fn test_android_clang_compiler_uses_target_arg_internally() {
    for version in 16..21 {
        assert!(android_clang_compiler_uses_target_arg_internally(
            &PathBuf::from(format!("armv7a-linux-androideabi{}-clang", version))
        ));
        assert!(android_clang_compiler_uses_target_arg_internally(
            &PathBuf::from(format!("armv7a-linux-androideabi{}-clang++", version))
        ));
    }
    assert!(!android_clang_compiler_uses_target_arg_internally(
        &PathBuf::from("clang")
    ));
    assert!(!android_clang_compiler_uses_target_arg_internally(
        &PathBuf::from("clang++")
    ));
}

fn autodetect_android_compiler(target: &str, host: &str, gnu: &str, clang: &str) -> String {
    let new_clang_key = match target {
        "aarch64-linux-android" => Some("aarch64"),
        "armv7-linux-androideabi" => Some("armv7a"),
        "i686-linux-android" => Some("i686"),
        "x86_64-linux-android" => Some("x86_64"),
        _ => None,
    };

    let new_clang = new_clang_key
        .map(|key| {
            NEW_STANDALONE_ANDROID_COMPILERS
                .iter()
                .find(|x| x.starts_with(key))
        })
        .unwrap_or(None);

    if let Some(new_clang) = new_clang {
        if Command::new(new_clang).output().is_ok() {
            return (*new_clang).into();
        }
    }

    let target = target
        .replace("armv7neon", "arm")
        .replace("armv7", "arm")
        .replace("thumbv7neon", "arm")
        .replace("thumbv7", "arm");
    let gnu_compiler = format!("{}-{}", target, gnu);
    let clang_compiler = format!("{}-{}", target, clang);

    // On Windows, the Android clang compiler is provided as a `.cmd` file instead
    // of a `.exe` file. `std::process::Command` won't run `.cmd` files unless the
    // `.cmd` is explicitly appended to the command name, so we do that here.
    let clang_compiler_cmd = format!("{}-{}.cmd", target, clang);

    // Check if gnu compiler is present
    // if not, use clang
    if Command::new(&gnu_compiler).output().is_ok() {
        gnu_compiler
    } else if host.contains("windows") && Command::new(&clang_compiler_cmd).output().is_ok() {
        clang_compiler_cmd
    } else {
        clang_compiler
    }
}

// Rust and clang/cc don't agree on how to name the target.
fn map_darwin_target_from_rust_to_compiler_architecture(target: &str) -> Option<&'static str> {
    if target.contains("x86_64") {
        Some("x86_64")
    } else if target.contains("arm64e") {
        Some("arm64e")
    } else if target.contains("aarch64") {
        Some("arm64")
    } else if target.contains("i686") {
        Some("i386")
    } else if target.contains("powerpc") {
        Some("ppc")
    } else if target.contains("powerpc64") {
        Some("ppc64")
    } else {
        None
    }
}

fn which(tool: &Path) -> Option<PathBuf> {
    fn check_exe(exe: &mut PathBuf) -> bool {
        let exe_ext = std::env::consts::EXE_EXTENSION;
        exe.exists() || (!exe_ext.is_empty() && exe.set_extension(exe_ext) && exe.exists())
    }

    // If |tool| is not just one "word," assume it's an actual path...
    if tool.components().count() > 1 {
        let mut exe = PathBuf::from(tool);
        return if check_exe(&mut exe) { Some(exe) } else { None };
    }

    // Loop through PATH entries searching for the |tool|.
    let path_entries = env::var_os("PATH")?;
    env::split_paths(&path_entries).find_map(|path_entry| {
        let mut exe = path_entry.join(tool);
        return if check_exe(&mut exe) { Some(exe) } else { None };
    })
}

#[derive(Clone, Copy, PartialEq)]
enum AsmFileExt {
    /// `.asm` files. On MSVC targets, we assume these should be passed to MASM
    /// (`ml{,64}.exe`).
    DotAsm,
    /// `.s` or `.S` files, which do not have the special handling on MSVC targets.
    DotS,
}

impl AsmFileExt {
    fn from_path(file: &Path) -> Option<Self> {
        if let Some(ext) = file.extension() {
            if let Some(ext) = ext.to_str() {
                let ext = ext.to_lowercase();
                match &*ext {
                    "asm" => return Some(AsmFileExt::DotAsm),
                    "s" => return Some(AsmFileExt::DotS),
                    _ => return None,
                }
            }
        }
        None
    }
}
