use super::*;
use rustyline::{completion::FilenameCompleter, Context, Helper};
use std::env;
use std::path::PathBuf;

/// PathBuf prompting will use a path autocompleter
impl Promptable for PathBuf {
    /// Prompt until you get a non-empty path
    fn prompt<S: AsRef<str>>(msg: S) -> Result<Self> {
        prompt_path(msg)
    }
    /// Prompt for an optional path
    fn prompt_opt<S: AsRef<str>>(msg: S) -> Result<Option<Self>> {
        prompt_path_opt(msg)
    }
    /// Prompt for a path with a provided fallback value if empty
    fn prompt_default<S: AsRef<str>>(msg: S, default: Self) -> Result<Self> {
        let msg = format!("{} (default={})", msg.as_ref(), default.display());
        Ok(prompt_path_opt(msg)?.unwrap_or(default))
    }
}

struct FilenameHelper {
    completer: FilenameCompleter,
}

impl Helper for FilenameHelper {}
impl rustyline::hint::Hinter for FilenameHelper {
    type Hint = String;
}
impl rustyline::validate::Validator for FilenameHelper {}
impl rustyline::highlight::Highlighter for FilenameHelper {}
impl Completer for FilenameHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

fn prompt_path<S: AsRef<str>>(msg: S) -> Result<PathBuf> {
    let helper = FilenameHelper {
        completer: FilenameCompleter::new(),
    };

    let s = Prompter::with_helper(helper).prompt_nonempty(msg)?;
    Ok(PathBuf::from(path_expand(s)))
}

fn prompt_path_opt<S: AsRef<str>>(msg: S) -> Result<Option<PathBuf>> {
    let helper = FilenameHelper {
        completer: FilenameCompleter::new(),
    };

    Ok(Prompter::with_helper(helper)
        .prompt_opt(msg)?
        .map(path_expand)
        .map(PathBuf::from))
}

fn path_expand(s: String) -> String {
    if s.starts_with('~') {
        if let Ok(home) = env::var("HOME") {
            return s.replacen('~', &home, 1);
        }
    }
    s
}
