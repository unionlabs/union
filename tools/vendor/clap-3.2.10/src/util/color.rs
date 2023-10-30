/// Represents the color preferences for program output
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ColorChoice {
    /// Enables colored output only when the output is going to a terminal or TTY.
    ///
    /// **NOTE:** This is the default behavior of `clap`.
    ///
    /// # Platform Specific
    ///
    /// This setting only applies to Unix, Linux, and macOS (i.e. non-Windows platforms).
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "color"), doc = " ```ignore")]
    #[cfg_attr(feature = "color", doc = " ```no_run")]
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Auto)
    ///     .get_matches();
    /// ```
    Auto,

    /// Enables colored output regardless of whether or not the output is going to a terminal/TTY.
    ///
    /// # Platform Specific
    ///
    /// This setting only applies to Unix, Linux, and macOS (i.e. non-Windows platforms).
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "color"), doc = " ```ignore")]
    #[cfg_attr(feature = "color", doc = " ```no_run")]
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Always)
    ///     .get_matches();
    /// ```
    Always,

    /// Disables colored output no matter if the output is going to a terminal/TTY, or not.
    ///
    /// # Platform Specific
    ///
    /// This setting only applies to Unix, Linux, and macOS (i.e. non-Windows platforms)
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "color"), doc = " ```ignore")]
    #[cfg_attr(feature = "color", doc = " ```no_run")]
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Never)
    ///     .get_matches();
    /// ```
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}
