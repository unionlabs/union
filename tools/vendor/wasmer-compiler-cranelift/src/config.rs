use crate::compiler::CraneliftCompiler;
use cranelift_codegen::isa::{lookup, TargetIsa};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::CodegenResult;
use std::sync::Arc;
use wasmer_compiler::{Compiler, CompilerConfig, Engine, EngineBuilder, ModuleMiddleware};
use wasmer_types::{Architecture, CpuFeature, Target};

// Runtime Environment

/// Possible optimization levels for the Cranelift codegen backend.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CraneliftOptLevel {
    /// No optimizations performed, minimizes compilation time by disabling most
    /// optimizations.
    None,
    /// Generates the fastest possible code, but may take longer.
    Speed,
    /// Similar to `speed`, but also performs transformations aimed at reducing
    /// code size.
    SpeedAndSize,
}

/// Global configuration options used to create an
/// `wasmer_engine::Engine` and customize its behavior.
///
/// This structure exposes a builder-like interface and is primarily
/// consumed by `wasmer_engine::Engine::new`.
#[derive(Debug, Clone)]
pub struct Cranelift {
    enable_nan_canonicalization: bool,
    enable_verifier: bool,
    enable_pic: bool,
    opt_level: CraneliftOptLevel,
    /// The middleware chain.
    pub(crate) middlewares: Vec<Arc<dyn ModuleMiddleware>>,
}

impl Cranelift {
    /// Creates a new configuration object with the default configuration
    /// specified.
    pub fn new() -> Self {
        Self {
            enable_nan_canonicalization: false,
            enable_verifier: false,
            opt_level: CraneliftOptLevel::Speed,
            enable_pic: false,
            middlewares: vec![],
        }
    }

    /// Enable NaN canonicalization.
    ///
    /// NaN canonicalization is useful when trying to run WebAssembly
    /// deterministically across different architectures.
    pub fn canonicalize_nans(&mut self, enable: bool) -> &mut Self {
        self.enable_nan_canonicalization = enable;
        self
    }

    /// The optimization levels when optimizing the IR.
    pub fn opt_level(&mut self, opt_level: CraneliftOptLevel) -> &mut Self {
        self.opt_level = opt_level;
        self
    }

    /// Generates the ISA for the provided target
    pub fn isa(&self, target: &Target) -> CodegenResult<Box<dyn TargetIsa>> {
        let mut builder =
            lookup(target.triple().clone()).expect("construct Cranelift ISA for triple");
        // Cpu Features
        let cpu_features = target.cpu_features();
        if target.triple().architecture == Architecture::X86_64
            && !cpu_features.contains(CpuFeature::SSE2)
        {
            panic!("x86 support requires SSE2");
        }
        if cpu_features.contains(CpuFeature::SSE3) {
            builder.enable("has_sse3").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::SSSE3) {
            builder.enable("has_ssse3").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::SSE41) {
            builder.enable("has_sse41").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::SSE42) {
            builder.enable("has_sse42").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::POPCNT) {
            builder.enable("has_popcnt").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::AVX) {
            builder.enable("has_avx").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::BMI1) {
            builder.enable("has_bmi1").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::BMI2) {
            builder.enable("has_bmi2").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::AVX2) {
            builder.enable("has_avx2").expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::AVX512DQ) {
            builder
                .enable("has_avx512dq")
                .expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::AVX512VL) {
            builder
                .enable("has_avx512vl")
                .expect("should be valid flag");
        }
        if cpu_features.contains(CpuFeature::LZCNT) {
            builder.enable("has_lzcnt").expect("should be valid flag");
        }

        builder.finish(self.flags(target))
    }

    /// Generates the flags for the compiler
    pub fn flags(&self, target: &Target) -> settings::Flags {
        let is_riscv = matches!(target.triple().architecture, Architecture::Riscv64(_));
        let mut flags = settings::builder();

        // Enable probestack
        flags
            .enable("enable_probestack")
            .expect("should be valid flag");

        // Only inline probestack is supported on AArch64
        if matches!(target.triple().architecture, Architecture::Aarch64(_)) {
            flags
                .set("probestack_strategy", "inline")
                .expect("should be valid flag");
        }

        // There are two possible traps for division, and this way
        // we get the proper one if code traps.
        flags
            .enable("avoid_div_traps")
            .expect("should be valid flag");

        if self.enable_pic {
            flags.enable("is_pic").expect("should be a valid flag");
        }

        // We set up libcall trampolines in engine-universal.
        // These trampolines are always reachable through short jumps.
        flags
            .enable("use_colocated_libcalls")
            .expect("should be a valid flag");

        // Invert cranelift's default-on verification to instead default off.
        let enable_verifier = if self.enable_verifier {
            "true"
        } else {
            "false"
        };
        flags
            .set("enable_verifier", enable_verifier)
            .expect("should be valid flag");
        flags
            .set("enable_safepoints", "true")
            .expect("should be valid flag");

        flags
            .set(
                "opt_level",
                match self.opt_level {
                    CraneliftOptLevel::None => "none",
                    CraneliftOptLevel::Speed => "speed",
                    CraneliftOptLevel::SpeedAndSize => "speed_and_size",
                },
            )
            .expect("should be valid flag");

        if is_riscv {
            flags
                .set("enable_simd", "false")
                .expect("should be valid flag");
        } else {
            flags
                .set("enable_simd", "true")
                .expect("should be valid flag");
        }

        let enable_nan_canonicalization = if self.enable_nan_canonicalization {
            "true"
        } else {
            "false"
        };
        flags
            .set("enable_nan_canonicalization", enable_nan_canonicalization)
            .expect("should be valid flag");

        settings::Flags::new(flags)
    }
}

impl CompilerConfig for Cranelift {
    fn enable_pic(&mut self) {
        self.enable_pic = true;
    }

    fn enable_verifier(&mut self) {
        self.enable_verifier = true;
    }

    fn canonicalize_nans(&mut self, enable: bool) {
        self.enable_nan_canonicalization = enable;
    }

    /// Transform it into the compiler
    fn compiler(self: Box<Self>) -> Box<dyn Compiler> {
        Box::new(CraneliftCompiler::new(*self))
    }

    /// Pushes a middleware onto the back of the middleware chain.
    fn push_middleware(&mut self, middleware: Arc<dyn ModuleMiddleware>) {
        self.middlewares.push(middleware);
    }
}

impl Default for Cranelift {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Cranelift> for Engine {
    fn from(config: Cranelift) -> Self {
        EngineBuilder::new(config).engine()
    }
}
