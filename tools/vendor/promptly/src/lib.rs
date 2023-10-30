#![cfg_attr(feature = "nightly", feature(specialization))]

//! Simply call `prompt`, `prompt_default`, or `prompt_opt` to prompt for any `Promptable` type:
//!
//! ## Examples
//!
//! ```no_run
//! # use std::path::PathBuf;
//! use promptly::{prompt, prompt_opt, prompt_default};
//!
//! // Prompt until a non-empty string is provided
//! let name: String = prompt("Enter your name")?;
//!
//! // Prompt for other `FromStr` types
//! let age: u32 = prompt("Enter your age")?;
//!
//! // Prompt for optional paths with path completion. Returns `None` if empty input.
//! let photo: Option<PathBuf> = prompt_opt("Enter a path to a profile picture")?;
//!
//! // Prompt Y/n with a default value when input is empty
//! let fallback = prompt_default("Would you like to receive marketing emails", true)?;
//!
//! # Result::<_,Box<std::error::Error>>::Ok(())
//! ```

use rustyline::completion::{Completer, Pair};
use rustyline::{Editor, Helper};

pub use rustyline::error::ReadlineError;
type Result<T> = std::result::Result<T, ReadlineError>;

#[cfg(feature = "nightly")]
use std::fmt::Display;

mod from_str;
mod path;
mod yes_no;

/// Prompt until input can be parsed as `T`.
///
/// Empty string input causes a re-prompt (including for `String`)
///
/// ## Examples
///
/// ```no_run
/// # use std::path::PathBuf;
/// use promptly::prompt;
///
/// // Prompt until a non-empty string is provided
/// let name: String = prompt("Enter your name")?;
///
/// // Prompt for other `FromStr` types
/// let age: u32 = prompt("Enter your age")?;
///
/// # Result::<_,Box<std::error::Error>>::Ok(())
/// ```
///
/// ## Errors
/// Returns a `ReadlineError` if readline fails.
/// Input that can't be coerced into the specified type results in re-prompting.
pub fn prompt<T, S>(msg: S) -> Result<T>
where
    T: Promptable,
    S: AsRef<str>,
{
    T::prompt(msg)
}

/// Prompt until input can be parsed as `T`, returning `None` for empty input.
///
/// Empty string input results in `None`
///
/// ## Examples
///
/// ```no_run
/// # use std::path::PathBuf;
/// use promptly::prompt_opt;
///
/// // Prompt for an optional string
/// let name: Option<String> = prompt_opt("Enter your name (optional)")?;
///
/// // Prompt for optional paths with path completion. Returns `None` if empty input.
/// let photo: Option<PathBuf> = prompt_opt("Enter a path to a profile picture")?;
///
/// # Result::<_,Box<std::error::Error>>::Ok(())
/// ```
///
/// ## Errors
/// Returns a `ReadlineError` if readline fails.
/// Input that can't be coerced into the specified type results in re-prompting.
pub fn prompt_opt<T, S>(msg: S) -> Result<Option<T>>
where
    T: Promptable,
    S: AsRef<str>,
{
    T::prompt_opt(msg)
}

/// Prompt until input can be parsed as `T`, returning the `default` for empty input.
///
/// ## Examples
///
/// ```no_run
/// # use std::net::Ipv4Addr;
/// # use std::path::PathBuf;
/// use promptly::prompt_default;
///
/// // Prompt Y/n with a default value when input is empty
/// let fallback = prompt_default("Would you like to receive marketing emails", true)?;
///
/// // Prompt for a string with default
/// let fav_lang = prompt_default("Enter you favorite programming language", "Rust".to_string())?;
///
/// // Prompt for other `FromStr` types
/// let local_ip = prompt_default("Enter your local IP", Ipv4Addr::new(127, 0, 0, 1))?;
///
/// # Result::<_,Box<std::error::Error>>::Ok(())
/// ```
///
/// ## Errors
/// Returns a `ReadlineError` if readline fails.
/// Input that can't be coerced into the specified type results in re-prompting.
pub fn prompt_default<T, S>(msg: S, default: T) -> Result<T>
where
    T: Promptable,
    S: AsRef<str>,
{
    T::prompt_default(msg, default)
}

/// A trait for convenient, opinionated prompting
pub trait Promptable: Sized {
    /// Prompts for a value. Re-prompts on invalid and empty input.
    fn prompt<S: AsRef<str>>(msg: S) -> Result<Self>;

    /// Prompts for a value, returning `None` for empty input. Re-prompts on invalid input.
    fn prompt_opt<S: AsRef<str>>(msg: S) -> Result<Option<Self>>;

    /// Prompts for a value with a default value for empty input. Re-prompts on invalid input.
    ///
    /// The default value will be mentioned in the prompt message
    fn prompt_default<S: AsRef<str>>(msg: S, default: Self) -> Result<Self>;
}

/// Optinionated wrapper around rustyline to prompt for strings
struct Prompter<H: Helper> {
    editor: Editor<H>,
}

impl Prompter<()> {
    pub fn new() -> Prompter<()> {
        Prompter::default()
    }
}

impl Default for Prompter<()> {
    fn default() -> Self {
        Prompter {
            editor: Editor::new(),
        }
    }
}

impl<H> Prompter<H>
where
    H: Helper,
{
    pub fn with_helper(helper: H) -> Prompter<H> {
        let mut editor = Editor::new();
        editor.set_helper(Some(helper));
        Prompter { editor }
    }

    pub fn prompt_once<S: AsRef<str>>(&mut self, msg: S) -> Result<String> {
        self.editor
            .readline(&format!("{}: ", msg.as_ref()))
            .map(|line| line.trim().to_owned())
    }

    /// Prompts once but returns `None` for empty input
    pub fn prompt_opt<S: AsRef<str>>(&mut self, msg: S) -> Result<Option<String>> {
        let val = self.prompt_once(msg)?;
        if val.is_empty() {
            return Ok(None);
        }
        Ok(Some(val))
    }

    /// Prompts until a non-empty value is provided
    pub fn prompt_nonempty<S: AsRef<str>>(&mut self, msg: S) -> Result<String> {
        let mut val = self.prompt_opt(&msg)?;
        while val.is_none() {
            eprintln!("Value is required.");
            val = self.prompt_opt(&msg)?;
        }
        Ok(val.unwrap())
    }

    /// Prompts with custom handler to transform input
    pub fn prompt_then<S, F, U>(&mut self, msg: S, handler: F) -> Result<U>
    where
        S: AsRef<str>,
        F: Fn(String) -> ::std::result::Result<U, String>,
    {
        let mut val = handler(self.prompt_once(&msg)?);
        while let Err(e) = val {
            eprintln!("{}", e);
            val = handler(self.prompt_once(&msg)?);
        }
        Ok(val.unwrap())
    }
}
