use core::fmt;

/// The error type for variable hasher initialization
#[derive(Clone, Copy, Debug, Default)]
pub struct InvalidOutputSize;

impl fmt::Display for InvalidOutputSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid output size")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidOutputSize {}
