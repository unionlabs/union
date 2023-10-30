use super::*;

/// Specialized `bool` prompter that supports yes/no (y/n) values
impl Promptable for bool {
    /// Prompt for `bool` represented as `true/false`, `yes/no`, or `y/n` input
    ///
    /// The prompt will display the options: `(y/n)`
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// bool::prompt("Do you accept the terms?")?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt<S: AsRef<str>>(msg: S) -> Result<Self> {
        prompt_bool(msg)
    }

    /// Prompt for optional `bool` input. Empty input returns `None`.
    ///
    /// The prompt will display the options: `(y/n)`
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// bool::prompt_opt("Did you even read this question?")?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt_opt<S: AsRef<str>>(msg: S) -> Result<Option<Self>> {
        prompt_bool_opt(msg)
    }

    /// Prompt for optional `bool` input. Empty input returns `None`.
    ///
    /// The prompt will also display the options: `(Y/n)` or `(y/N)` depending on the default
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// bool::prompt_default("Would you like to send us money?", true)?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt_default<S: AsRef<str>>(msg: S, default: Self) -> Result<Self> {
        let msg = if default {
            format!("{} (Y/n)", msg.as_ref())
        } else {
            format!("{} (y/N)", msg.as_ref())
        };
        Ok(prompt_bool_opt(msg)?.unwrap_or(default))
    }
}

fn prompt_bool<S: AsRef<str>>(msg: S) -> Result<bool> {
    Prompter::new().prompt_then(msg, |s| match &*s.to_lowercase() {
        "true" | "yes" | "y" => Ok(true),
        "false" | "no" | "n" => Ok(false),
        s => Err(format!("Could not parse {} as bool.", s)),
    })
}

fn prompt_bool_opt<S: AsRef<str>>(msg: S) -> Result<Option<bool>> {
    Prompter::new().prompt_then(msg, |s| match &*s.to_lowercase().trim() {
        "" => Ok(None),
        "true" | "yes" | "y" => Ok(Some(true)),
        "false" | "no" | "n" => Ok(Some(false)),
        s => Err(format!("Could not parse {} as bool.", s)),
    })
}
