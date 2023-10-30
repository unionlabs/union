//! Target configuration
use crate::error::ParseCpuFeatureError;
use crate::lib::std::str::FromStr;
use crate::lib::std::string::{String, ToString};
use enumset::{EnumSet, EnumSetType};
use loupe::MemoryUsage;
pub use target_lexicon::{
    Architecture, BinaryFormat, CallingConvention, Endianness, OperatingSystem, PointerWidth,
    Triple,
};

/// The nomenclature is inspired by the [`cpuid` crate].
/// The list of supported features was initially retrieved from
/// [`cranelift-native`].
///
/// The `CpuFeature` enum values are likely to grow closer to the
/// original `cpuid`. However, we prefer to start small and grow from there.
///
/// If you would like to use a flag that doesn't exist yet here, please
/// open a PR.
///
/// [`cpuid` crate]: https://docs.rs/cpuid/0.1.1/cpuid/enum.CpuFeature.html
/// [`cranelift-native`]: https://github.com/bytecodealliance/cranelift/blob/6988545fd20249b084c53f4761b8c861266f5d31/cranelift-native/src/lib.rs#L51-L92
#[allow(missing_docs, clippy::derive_hash_xor_eq)]
#[derive(EnumSetType, Debug, Hash)]
pub enum CpuFeature {
    // X86 features
    SSE2,
    SSE3,
    SSSE3,
    SSE41,
    SSE42,
    POPCNT,
    AVX,
    BMI1,
    BMI2,
    AVX2,
    AVX512DQ,
    AVX512VL,
    AVX512F,
    LZCNT,
    // ARM features
    // Risc-V features
}

impl CpuFeature {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// Retrieves the features for the current Host
    pub fn for_host() -> EnumSet<Self> {
        let mut features = EnumSet::new();

        if std::is_x86_feature_detected!("sse2") {
            features.insert(Self::SSE2);
        }
        if std::is_x86_feature_detected!("sse3") {
            features.insert(Self::SSE3);
        }
        if std::is_x86_feature_detected!("ssse3") {
            features.insert(Self::SSSE3);
        }
        if std::is_x86_feature_detected!("sse4.1") {
            features.insert(Self::SSE41);
        }
        if std::is_x86_feature_detected!("sse4.2") {
            features.insert(Self::SSE42);
        }
        if std::is_x86_feature_detected!("popcnt") {
            features.insert(Self::POPCNT);
        }
        if std::is_x86_feature_detected!("avx") {
            features.insert(Self::AVX);
        }
        if std::is_x86_feature_detected!("bmi1") {
            features.insert(Self::BMI1);
        }
        if std::is_x86_feature_detected!("bmi2") {
            features.insert(Self::BMI2);
        }
        if std::is_x86_feature_detected!("avx2") {
            features.insert(Self::AVX2);
        }
        if std::is_x86_feature_detected!("avx512dq") {
            features.insert(Self::AVX512DQ);
        }
        if std::is_x86_feature_detected!("avx512vl") {
            features.insert(Self::AVX512VL);
        }
        if std::is_x86_feature_detected!("avx512f") {
            features.insert(Self::AVX512F);
        }
        if std::is_x86_feature_detected!("lzcnt") {
            features.insert(Self::LZCNT);
        }
        features
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    /// Retrieves the features for the current Host
    pub fn for_host() -> EnumSet<Self> {
        // We default to an empty hash set
        EnumSet::new()
    }

    /// Retrieves an empty set of `CpuFeature`s.
    pub fn set() -> EnumSet<Self> {
        // We default to an empty hash set
        EnumSet::new()
    }
}

// This options should map exactly the GCC options indicated
// here by architectures:
//
// X86: https://gcc.gnu.org/onlinedocs/gcc/x86-Options.html
// ARM: https://gcc.gnu.org/onlinedocs/gcc/ARM-Options.html
// Aarch64: https://gcc.gnu.org/onlinedocs/gcc/AArch64-Options.html
impl FromStr for CpuFeature {
    type Err = ParseCpuFeatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sse2" => Ok(Self::SSE2),
            "sse3" => Ok(Self::SSE3),
            "ssse3" => Ok(Self::SSSE3),
            "sse4.1" => Ok(Self::SSE41),
            "sse4.2" => Ok(Self::SSE42),
            "popcnt" => Ok(Self::POPCNT),
            "avx" => Ok(Self::AVX),
            "bmi" => Ok(Self::BMI1),
            "bmi2" => Ok(Self::BMI2),
            "avx2" => Ok(Self::AVX2),
            "avx512dq" => Ok(Self::AVX512DQ),
            "avx512vl" => Ok(Self::AVX512VL),
            "avx512f" => Ok(Self::AVX512F),
            "lzcnt" => Ok(Self::LZCNT),
            _ => Err(ParseCpuFeatureError::Missing(s.to_string())),
        }
    }
}

impl ToString for CpuFeature {
    fn to_string(&self) -> String {
        match self {
            Self::SSE2 => "sse2",
            Self::SSE3 => "sse3",
            Self::SSSE3 => "ssse3",
            Self::SSE41 => "sse4.1",
            Self::SSE42 => "sse4.2",
            Self::POPCNT => "popcnt",
            Self::AVX => "avx",
            Self::BMI1 => "bmi",
            Self::BMI2 => "bmi2",
            Self::AVX2 => "avx2",
            Self::AVX512DQ => "avx512dq",
            Self::AVX512VL => "avx512vl",
            Self::AVX512F => "avx512f",
            Self::LZCNT => "lzcnt",
        }
        .to_string()
    }
}

/// This is the target that we will use for compiling
/// the WebAssembly ModuleInfo, and then run it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, MemoryUsage)]
pub struct Target {
    #[loupe(skip)]
    triple: Triple,
    #[loupe(skip)]
    cpu_features: EnumSet<CpuFeature>,
}

impl Target {
    /// Creates a new target given a triple
    pub fn new(triple: Triple, cpu_features: EnumSet<CpuFeature>) -> Self {
        Self {
            triple,
            cpu_features,
        }
    }

    /// The triple associated for the target.
    pub fn triple(&self) -> &Triple {
        &self.triple
    }

    /// The triple associated for the target.
    pub fn cpu_features(&self) -> &EnumSet<CpuFeature> {
        &self.cpu_features
    }
}

/// The default for the Target will use the HOST as the triple
impl Default for Target {
    fn default() -> Self {
        Self {
            triple: Triple::host(),
            cpu_features: CpuFeature::for_host(),
        }
    }
}
