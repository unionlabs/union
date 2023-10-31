// We only use AVX when we can detect at runtime whether it's available, which
// requires std.
#[cfg(feature = "std")]
pub(crate) mod avx;
pub(crate) mod sse;
