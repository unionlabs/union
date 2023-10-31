/// Reset terminal formatting
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reset;

impl Reset {
    /// Render the ANSI code
    #[inline]
    pub fn render(self) -> impl core::fmt::Display {
        ResetDisplay
    }
}

struct ResetDisplay;

impl core::fmt::Display for ResetDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        RESET.fmt(f)
    }
}

pub(crate) const RESET: &str = "\x1B[0m";
