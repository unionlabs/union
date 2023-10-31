//! Readline for Rust
//!
//! This implementation is based on [Antirez's
//! Linenoise](https://github.com/antirez/linenoise)
//!
//! # Example
//!
//! Usage
//!
//! ```
//! let mut rl = rustyline::Editor::<()>::new();
//! let readline = rl.readline(">> ");
//! match readline {
//!     Ok(line) => println!("Line: {:?}", line),
//!     Err(_) => println!("No input"),
//! }
//! ```
#![warn(missing_docs)]

mod binding;
mod command;
pub mod completion;
pub mod config;
mod edit;
pub mod error;
pub mod highlight;
pub mod hint;
pub mod history;
mod keymap;
mod keys;
mod kill_ring;
mod layout;
pub mod line_buffer;
mod tty;
mod undo;
pub mod validate;

use std::fmt;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::result;
use std::sync::{Arc, Mutex, RwLock};

use log::debug;
use radix_trie::Trie;
use unicode_width::UnicodeWidthStr;

use crate::tty::{RawMode, Renderer, Term, Terminal};

pub use crate::binding::{ConditionalEventHandler, Event, EventContext, EventHandler};
use crate::completion::{longest_common_prefix, Candidate, Completer};
pub use crate::config::{
    ColorMode, CompletionType, Config, EditMode, HistoryDuplicates, OutputStreamType,
};
use crate::edit::State;
use crate::highlight::Highlighter;
use crate::hint::Hinter;
use crate::history::{History, SearchDirection};
pub use crate::keymap::{Anchor, At, CharSearch, Cmd, InputMode, Movement, RepeatCount, Word};
use crate::keymap::{InputState, Refresher};
pub use crate::keys::{KeyCode, KeyEvent, Modifiers};
use crate::kill_ring::KillRing;

use crate::validate::Validator;

/// The error type for I/O and Linux Syscalls (Errno)
pub type Result<T> = result::Result<T, error::ReadlineError>;

/// Completes the line/word
fn complete_line<H: Helper>(
    rdr: &mut <Terminal as Term>::Reader,
    s: &mut State<'_, '_, H>,
    input_state: &mut InputState,
    config: &Config,
) -> Result<Option<Cmd>> {
    #[cfg(all(unix, feature = "with-fuzzy"))]
    use skim::prelude::{
        unbounded, Skim, SkimItem, SkimItemReceiver, SkimItemSender, SkimOptionsBuilder,
    };

    let completer = s.helper.unwrap();
    // get a list of completions
    let (start, candidates) = completer.complete(&s.line, s.line.pos(), &s.ctx)?;
    // if no completions, we are done
    if candidates.is_empty() {
        s.out.beep()?;
        Ok(None)
    } else if CompletionType::Circular == config.completion_type() {
        let mark = s.changes.borrow_mut().begin();
        // Save the current edited line before overwriting it
        let backup = s.line.as_str().to_owned();
        let backup_pos = s.line.pos();
        let mut cmd;
        let mut i = 0;
        loop {
            // Show completion or original buffer
            if i < candidates.len() {
                let candidate = candidates[i].replacement();
                // TODO we can't highlight the line buffer directly
                /*let candidate = if let Some(highlighter) = s.highlighter {
                    highlighter.highlight_candidate(candidate, CompletionType::Circular)
                } else {
                    Borrowed(candidate)
                };*/
                completer.update(&mut s.line, start, candidate);
            } else {
                // Restore current edited line
                s.line.update(&backup, backup_pos);
            }
            s.refresh_line()?;

            cmd = s.next_cmd(input_state, rdr, true)?;
            match cmd {
                Cmd::Complete => {
                    i = (i + 1) % (candidates.len() + 1); // Circular
                    if i == candidates.len() {
                        s.out.beep()?;
                    }
                }
                Cmd::CompleteBackward => {
                    if i == 0 {
                        i = candidates.len(); // Circular
                        s.out.beep()?;
                    } else {
                        i = (i - 1) % (candidates.len() + 1); // Circular
                    }
                }
                Cmd::Abort => {
                    // Re-show original buffer
                    if i < candidates.len() {
                        s.line.update(&backup, backup_pos);
                        s.refresh_line()?;
                    }
                    s.changes.borrow_mut().truncate(mark);
                    return Ok(None);
                }
                _ => {
                    s.changes.borrow_mut().end();
                    break;
                }
            }
        }
        Ok(Some(cmd))
    } else if CompletionType::List == config.completion_type() {
        if let Some(lcp) = longest_common_prefix(&candidates) {
            // if we can extend the item, extend it
            if lcp.len() > s.line.pos() - start {
                completer.update(&mut s.line, start, lcp);
                s.refresh_line()?;
            }
        }
        // beep if ambiguous
        if candidates.len() > 1 {
            s.out.beep()?;
        } else {
            return Ok(None);
        }
        // we can't complete any further, wait for second tab
        let mut cmd = s.next_cmd(input_state, rdr, true)?;
        // if any character other than tab, pass it to the main loop
        if cmd != Cmd::Complete {
            return Ok(Some(cmd));
        }
        // move cursor to EOL to avoid overwriting the command line
        let save_pos = s.line.pos();
        s.edit_move_end()?;
        s.line.set_pos(save_pos);
        // we got a second tab, maybe show list of possible completions
        let show_completions = if candidates.len() > config.completion_prompt_limit() {
            let msg = format!("\nDisplay all {} possibilities? (y or n)", candidates.len());
            s.out.write_and_flush(msg.as_bytes())?;
            s.layout.end.row += 1;
            while cmd != Cmd::SelfInsert(1, 'y')
                && cmd != Cmd::SelfInsert(1, 'Y')
                && cmd != Cmd::SelfInsert(1, 'n')
                && cmd != Cmd::SelfInsert(1, 'N')
                && cmd != Cmd::Kill(Movement::BackwardChar(1))
            {
                cmd = s.next_cmd(input_state, rdr, false)?;
            }
            matches!(cmd, Cmd::SelfInsert(1, 'y') | Cmd::SelfInsert(1, 'Y'))
        } else {
            true
        };
        if show_completions {
            page_completions(rdr, s, input_state, &candidates)
        } else {
            s.refresh_line()?;
            Ok(None)
        }
    } else {
        // if fuzzy feature is enabled and on unix based systems check for the
        // corresponding completion_type
        #[cfg(all(unix, feature = "with-fuzzy"))]
        {
            use std::borrow::Cow;
            if CompletionType::Fuzzy == config.completion_type() {
                struct Candidate {
                    index: usize,
                    text: String,
                }
                impl SkimItem for Candidate {
                    fn text(&self) -> Cow<str> {
                        Cow::Borrowed(&self.text)
                    }
                }

                let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

                candidates
                    .iter()
                    .enumerate()
                    .map(|(i, c)| Candidate {
                        index: i,
                        text: c.display().to_owned(),
                    })
                    .for_each(|c| {
                        let _ = tx_item.send(Arc::new(c));
                    });
                drop(tx_item); // so that skim could know when to stop waiting for more items.

                // setup skim and run with input options
                // will display UI for fuzzy search and return selected results
                // by default skim multi select is off so only expect one selection

                let options = SkimOptionsBuilder::default()
                    .height(Some("20%"))
                    .prompt(Some("? "))
                    .reverse(true)
                    .build()
                    .unwrap();

                let selected_items = Skim::run_with(&options, Some(rx_item))
                    .map(|out| out.selected_items)
                    .unwrap_or_else(Vec::new);

                // match the first (and only) returned option with the candidate and update the
                // line otherwise only refresh line to clear the skim UI changes
                if let Some(item) = selected_items.first() {
                    let item: &Candidate = (*item).as_any() // cast to Any
                        .downcast_ref::<Candidate>() // downcast to concrete type
                        .expect("something wrong with downcast");
                    if let Some(candidate) = candidates.get(item.index) {
                        completer.update(&mut s.line, start, candidate.replacement());
                    }
                }
                s.refresh_line()?;
            }
        };
        Ok(None)
    }
}

/// Completes the current hint
fn complete_hint_line<H: Helper>(s: &mut State<'_, '_, H>) -> Result<()> {
    let hint = match s.hint.as_ref() {
        Some(hint) => hint,
        None => return Ok(()),
    };
    s.line.move_end();
    if let Some(text) = hint.completion() {
        if s.line.yank(text, 1).is_none() {
            s.out.beep()?;
        }
    } else {
        s.out.beep()?;
    }
    s.refresh_line()
}

fn page_completions<C: Candidate, H: Helper>(
    rdr: &mut <Terminal as Term>::Reader,
    s: &mut State<'_, '_, H>,
    input_state: &mut InputState,
    candidates: &[C],
) -> Result<Option<Cmd>> {
    use std::cmp;

    let min_col_pad = 2;
    let cols = s.out.get_columns();
    let max_width = cmp::min(
        cols,
        candidates
            .iter()
            .map(|s| s.display().width())
            .max()
            .unwrap()
            + min_col_pad,
    );
    let num_cols = cols / max_width;

    let mut pause_row = s.out.get_rows() - 1;
    let num_rows = (candidates.len() + num_cols - 1) / num_cols;
    let mut ab = String::new();
    for row in 0..num_rows {
        if row == pause_row {
            s.out.write_and_flush(b"\n--More--")?;
            let mut cmd = Cmd::Noop;
            while cmd != Cmd::SelfInsert(1, 'y')
                && cmd != Cmd::SelfInsert(1, 'Y')
                && cmd != Cmd::SelfInsert(1, 'n')
                && cmd != Cmd::SelfInsert(1, 'N')
                && cmd != Cmd::SelfInsert(1, 'q')
                && cmd != Cmd::SelfInsert(1, 'Q')
                && cmd != Cmd::SelfInsert(1, ' ')
                && cmd != Cmd::Kill(Movement::BackwardChar(1))
                && cmd != Cmd::AcceptLine
                && cmd != Cmd::Newline
                && !matches!(cmd, Cmd::AcceptOrInsertLine { .. })
            {
                cmd = s.next_cmd(input_state, rdr, false)?;
            }
            match cmd {
                Cmd::SelfInsert(1, 'y') | Cmd::SelfInsert(1, 'Y') | Cmd::SelfInsert(1, ' ') => {
                    pause_row += s.out.get_rows() - 1;
                }
                Cmd::AcceptLine | Cmd::Newline | Cmd::AcceptOrInsertLine { .. } => {
                    pause_row += 1;
                }
                _ => break,
            }
        }
        s.out.write_and_flush(b"\n")?;
        ab.clear();
        for col in 0..num_cols {
            let i = (col * num_rows) + row;
            if i < candidates.len() {
                let candidate = &candidates[i].display();
                let width = candidate.width();
                if let Some(highlighter) = s.highlighter() {
                    ab.push_str(&highlighter.highlight_candidate(candidate, CompletionType::List));
                } else {
                    ab.push_str(candidate);
                }
                if ((col + 1) * num_rows) + row < candidates.len() {
                    for _ in width..max_width {
                        ab.push(' ');
                    }
                }
            }
        }
        s.out.write_and_flush(ab.as_bytes())?;
    }
    s.out.write_and_flush(b"\n")?;
    s.layout.end.row = 0; // dirty way to make clear_old_rows do nothing
    s.layout.cursor.row = 0;
    s.refresh_line()?;
    Ok(None)
}

/// Incremental search
fn reverse_incremental_search<H: Helper>(
    rdr: &mut <Terminal as Term>::Reader,
    s: &mut State<'_, '_, H>,
    input_state: &mut InputState,
    history: &History,
) -> Result<Option<Cmd>> {
    if history.is_empty() {
        return Ok(None);
    }
    let mark = s.changes.borrow_mut().begin();
    // Save the current edited line (and cursor position) before overwriting it
    let backup = s.line.as_str().to_owned();
    let backup_pos = s.line.pos();

    let mut search_buf = String::new();
    let mut history_idx = history.len() - 1;
    let mut direction = SearchDirection::Reverse;
    let mut success = true;

    let mut cmd;
    // Display the reverse-i-search prompt and process chars
    loop {
        let prompt = if success {
            format!("(reverse-i-search)`{}': ", search_buf)
        } else {
            format!("(failed reverse-i-search)`{}': ", search_buf)
        };
        s.refresh_prompt_and_line(&prompt)?;

        cmd = s.next_cmd(input_state, rdr, true)?;
        if let Cmd::SelfInsert(_, c) = cmd {
            search_buf.push(c);
        } else {
            match cmd {
                Cmd::Kill(Movement::BackwardChar(_)) => {
                    search_buf.pop();
                    continue;
                }
                Cmd::ReverseSearchHistory => {
                    direction = SearchDirection::Reverse;
                    if history_idx > 0 {
                        history_idx -= 1;
                    } else {
                        success = false;
                        continue;
                    }
                }
                Cmd::ForwardSearchHistory => {
                    direction = SearchDirection::Forward;
                    if history_idx < history.len() - 1 {
                        history_idx += 1;
                    } else {
                        success = false;
                        continue;
                    }
                }
                Cmd::Abort => {
                    // Restore current edited line (before search)
                    s.line.update(&backup, backup_pos);
                    s.refresh_line()?;
                    s.changes.borrow_mut().truncate(mark);
                    return Ok(None);
                }
                Cmd::Move(_) => {
                    s.refresh_line()?; // restore prompt
                    break;
                }
                _ => break,
            }
        }
        success = match history.search(&search_buf, history_idx, direction) {
            Some(sr) => {
                history_idx = sr.idx;
                s.line.update(sr.entry, sr.pos);
                true
            }
            _ => false,
        };
    }
    s.changes.borrow_mut().end();
    Ok(Some(cmd))
}

/// Handles reading and editing the readline buffer.
/// It will also handle special inputs in an appropriate fashion
/// (e.g., C-c will exit readline)
fn readline_edit<H: Helper>(
    prompt: &str,
    initial: Option<(&str, &str)>,
    editor: &mut Editor<H>,
    original_mode: &tty::Mode,
    term_key_map: tty::KeyMap,
) -> Result<String> {
    let mut stdout = editor.term.create_writer();

    editor.reset_kill_ring(); // TODO recreate a new kill ring vs Arc<Mutex<KillRing>>
    let ctx = Context::new(&editor.history);
    let mut s = State::new(&mut stdout, prompt, editor.helper.as_ref(), ctx);

    let mut input_state = InputState::new(&editor.config, Arc::clone(&editor.custom_bindings));

    s.line.set_delete_listener(editor.kill_ring.clone());
    s.line.set_change_listener(s.changes.clone());

    if let Some((left, right)) = initial {
        s.line
            .update((left.to_owned() + right).as_ref(), left.len());
    }

    let mut rdr = editor.term.create_reader(&editor.config, term_key_map)?;
    if editor.term.is_output_tty() && editor.config.check_cursor_position() {
        if let Err(e) = s.move_cursor_at_leftmost(&mut rdr) {
            if s.out.sigwinch() {
                s.out.update_size();
            } else {
                return Err(e);
            }
        }
    }
    s.refresh_line()?;

    loop {
        let mut cmd = s.next_cmd(&mut input_state, &mut rdr, false)?;

        if cmd.should_reset_kill_ring() {
            editor.reset_kill_ring();
        }

        // First trigger commands that need extra input

        if cmd == Cmd::Complete && s.helper.is_some() {
            let next = complete_line(&mut rdr, &mut s, &mut input_state, &editor.config)?;
            if let Some(next) = next {
                cmd = next;
            } else {
                continue;
            }
        }

        if cmd == Cmd::ReverseSearchHistory {
            // Search history backward
            let next =
                reverse_incremental_search(&mut rdr, &mut s, &mut input_state, &editor.history)?;
            if let Some(next) = next {
                cmd = next;
            } else {
                continue;
            }
        }

        #[cfg(unix)]
        if cmd == Cmd::Suspend {
            original_mode.disable_raw_mode()?;
            tty::suspend()?;
            let _ = editor.term.enable_raw_mode()?; // TODO original_mode may have changed
            s.refresh_line()?;
            continue;
        }

        #[cfg(unix)]
        if cmd == Cmd::QuotedInsert {
            // Quoted insert
            use crate::tty::RawReader;
            let c = rdr.next_char()?;
            s.edit_insert(c, 1)?;
            continue;
        }

        #[cfg(windows)]
        if cmd == Cmd::PasteFromClipboard {
            use crate::tty::RawReader;
            let clipboard = rdr.read_pasted_text()?;
            s.edit_yank(&input_state, &clipboard[..], Anchor::Before, 1)?;
        }

        // Tiny test quirk
        #[cfg(test)]
        if matches!(
            cmd,
            Cmd::AcceptLine | Cmd::Newline | Cmd::AcceptOrInsertLine { .. }
        ) {
            editor.term.cursor = s.layout.cursor.col;
        }

        // Execute things can be done solely on a state object
        match command::execute(cmd, &mut s, &input_state, &editor.kill_ring, &editor.config)? {
            command::Status::Proceed => continue,
            command::Status::Submit => break,
        }
    }

    // Move to end, in case cursor was in the middle of the line, so that
    // next thing application prints goes after the input
    s.edit_move_buffer_end()?;

    if cfg!(windows) {
        let _ = original_mode; // silent warning
    }
    Ok(s.line.into_string())
}

struct Guard<'m>(&'m tty::Mode);

#[allow(unused_must_use)]
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        let Guard(mode) = *self;
        mode.disable_raw_mode();
    }
}

/// Readline method that will enable RAW mode, call the `readline_edit()`
/// method and disable raw mode
fn readline_raw<H: Helper>(
    prompt: &str,
    initial: Option<(&str, &str)>,
    editor: &mut Editor<H>,
) -> Result<String> {
    let (original_mode, term_key_map) = editor.term.enable_raw_mode()?;
    let guard = Guard(&original_mode);
    let user_input = readline_edit(prompt, initial, editor, &original_mode, term_key_map);
    if editor.config.auto_add_history() {
        if let Ok(ref line) = user_input {
            editor.add_history_entry(line.as_str());
        }
    }
    drop(guard); // disable_raw_mode(original_mode)?;
    match editor.config.output_stream() {
        OutputStreamType::Stdout => writeln!(io::stdout())?,
        OutputStreamType::Stderr => writeln!(io::stderr())?,
    };
    user_input
}

// Helper to handle backspace characters in a direct input
fn apply_backspace_direct(input: &str) -> String {
    // Setup the output buffer
    // No '\b' in the input in the common case, so set the capacity to the input
    // length
    let mut out = String::with_capacity(input.len());

    // Keep track of the size of each grapheme from the input
    // As many graphemes as input bytes in the common case
    let mut grapheme_sizes: Vec<u8> = Vec::with_capacity(input.len());

    for g in unicode_segmentation::UnicodeSegmentation::graphemes(input, true) {
        match g {
            // backspace char
            "\u{0008}" => {
                if let Some(n) = grapheme_sizes.pop() {
                    // Remove the last grapheme
                    out.truncate(out.len() - n as usize);
                }
            }
            _ => {
                out.push_str(g);
                grapheme_sizes.push(g.len() as u8);
            }
        }
    }

    out
}

fn readline_direct(
    mut reader: impl BufRead,
    mut writer: impl Write,
    validator: &Option<impl Validator>,
) -> Result<String> {
    let mut input = String::new();

    loop {
        match reader.read_line(&mut input)? {
            0 => return Err(error::ReadlineError::Eof),
            _ => {
                // Remove trailing newline
                let trailing_n = input.ends_with('\n');
                let trailing_r;

                if trailing_n {
                    input.pop();
                    trailing_r = input.ends_with('\r');
                    if trailing_r {
                        input.pop();
                    }
                } else {
                    trailing_r = false;
                }

                input = apply_backspace_direct(&input);

                match validator.as_ref() {
                    None => return Ok(input),
                    Some(v) => {
                        let mut ctx = input.as_str();
                        let mut ctx = validate::ValidationContext::new(&mut ctx);

                        match v.validate(&mut ctx)? {
                            validate::ValidationResult::Valid(msg) => {
                                if let Some(msg) = msg {
                                    writer.write_all(msg.as_bytes())?;
                                }
                                return Ok(input);
                            }
                            validate::ValidationResult::Invalid(Some(msg)) => {
                                writer.write_all(msg.as_bytes())?;
                            }
                            validate::ValidationResult::Incomplete => {
                                // Add newline and keep on taking input
                                if trailing_r {
                                    input.push('\r');
                                }
                                if trailing_n {
                                    input.push('\n');
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Syntax specific helper.
///
/// TODO Tokenizer/parser used for both completion, suggestion, highlighting.
/// (parse current line once)
pub trait Helper
where
    Self: Completer + Hinter + Highlighter + Validator,
{
}

impl Helper for () {}

impl<'h, H: ?Sized + Helper> Helper for &'h H {}

/// Completion/suggestion context
pub struct Context<'h> {
    history: &'h History,
    history_index: usize,
}

impl<'h> Context<'h> {
    /// Constructor. Visible for testing.
    pub fn new(history: &'h History) -> Self {
        Context {
            history,
            history_index: history.len(),
        }
    }

    /// Return an immutable reference to the history object.
    pub fn history(&self) -> &History {
        self.history
    }

    /// The history index we are currently editing
    pub fn history_index(&self) -> usize {
        self.history_index
    }
}

/// Line editor
pub struct Editor<H: Helper> {
    term: Terminal,
    history: History,
    helper: Option<H>,
    kill_ring: Arc<Mutex<KillRing>>,
    config: Config,
    custom_bindings: Arc<RwLock<Trie<Event, EventHandler>>>,
}

#[allow(clippy::new_without_default)]
impl<H: Helper> Editor<H> {
    /// Create an editor with the default configuration
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    /// Create an editor with a specific configuration.
    pub fn with_config(config: Config) -> Self {
        let term = Terminal::new(
            config.color_mode(),
            config.output_stream(),
            config.tab_stop(),
            config.bell_style(),
            config.enable_bracketed_paste(),
        );
        Self {
            term,
            history: History::with_config(config),
            helper: None,
            kill_ring: Arc::new(Mutex::new(KillRing::new(60))),
            config,
            custom_bindings: Arc::new(RwLock::new(Trie::new())),
        }
    }

    /// This method will read a line from STDIN and will display a `prompt`.
    ///
    /// It uses terminal-style interaction if `stdin` is connected to a
    /// terminal.
    /// Otherwise (e.g., if `stdin` is a pipe or the terminal is not supported),
    /// it uses file-style interaction.
    pub fn readline(&mut self, prompt: &str) -> Result<String> {
        self.readline_with(prompt, None)
    }

    /// This function behaves in the exact same manner as `readline`, except
    /// that it pre-populates the input area.
    ///
    /// The text that resides in the input area is given as a 2-tuple.
    /// The string on the left of the tuple is what will appear to the left of
    /// the cursor and the string on the right is what will appear to the
    /// right of the cursor.
    pub fn readline_with_initial(&mut self, prompt: &str, initial: (&str, &str)) -> Result<String> {
        self.readline_with(prompt, Some(initial))
    }

    fn readline_with(&mut self, prompt: &str, initial: Option<(&str, &str)>) -> Result<String> {
        if self.term.is_unsupported() {
            debug!(target: "rustyline", "unsupported terminal");
            // Write prompt and flush it to stdout
            let mut stdout = io::stdout();
            stdout.write_all(prompt.as_bytes())?;
            stdout.flush()?;

            readline_direct(io::stdin().lock(), io::stderr(), &self.helper)
        } else if self.term.is_stdin_tty() {
            readline_raw(prompt, initial, self)
        } else {
            debug!(target: "rustyline", "stdin is not a tty");
            // Not a tty: read from file / pipe.
            readline_direct(io::stdin().lock(), io::stderr(), &self.helper)
        }
    }

    /// Load the history from the specified file.
    pub fn load_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.history.load(path)
    }

    /// Save the history in the specified file.
    pub fn save_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.history.save(path)
    }

    /// Append new entries in the specified file.
    pub fn append_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.history.append(path)
    }

    /// Add a new entry in the history.
    pub fn add_history_entry<S: AsRef<str> + Into<String>>(&mut self, line: S) -> bool {
        self.history.add(line)
    }

    /// Clear history.
    pub fn clear_history(&mut self) {
        self.history.clear()
    }

    /// Return a mutable reference to the history object.
    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }

    /// Return an immutable reference to the history object.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// Register a callback function to be called for tab-completion
    /// or to show hints to the user at the right of the prompt.
    pub fn set_helper(&mut self, helper: Option<H>) {
        self.helper = helper;
    }

    /// Return a mutable reference to the helper.
    pub fn helper_mut(&mut self) -> Option<&mut H> {
        self.helper.as_mut()
    }

    /// Return an immutable reference to the helper.
    pub fn helper(&self) -> Option<&H> {
        self.helper.as_ref()
    }

    /// Bind a sequence to a command.
    pub fn bind_sequence<E: Into<Event>, R: Into<EventHandler>>(
        &mut self,
        key_seq: E,
        handler: R,
    ) -> Option<EventHandler> {
        if let Ok(mut bindings) = self.custom_bindings.write() {
            bindings.insert(Event::normalize(key_seq.into()), handler.into())
        } else {
            None
        }
    }

    /// Remove a binding for the given sequence.
    pub fn unbind_sequence<E: Into<Event>>(&mut self, key_seq: E) -> Option<EventHandler> {
        if let Ok(mut bindings) = self.custom_bindings.write() {
            bindings.remove(&Event::normalize(key_seq.into()))
        } else {
            None
        }
    }

    /// Returns an iterator over edited lines
    /// ```
    /// let mut rl = rustyline::Editor::<()>::new();
    /// for readline in rl.iter("> ") {
    ///     match readline {
    ///         Ok(line) => {
    ///             println!("Line: {}", line);
    ///         }
    ///         Err(err) => {
    ///             println!("Error: {:?}", err);
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    pub fn iter<'a>(&'a mut self, prompt: &'a str) -> impl Iterator<Item = Result<String>> + 'a {
        Iter {
            editor: self,
            prompt,
        }
    }

    fn reset_kill_ring(&self) {
        let mut kill_ring = self.kill_ring.lock().unwrap();
        kill_ring.reset();
    }

    /// If output stream is a tty, this function returns its width and height as
    /// a number of characters.
    pub fn dimensions(&mut self) -> Option<(usize, usize)> {
        if self.term.is_output_tty() {
            let out = self.term.create_writer();
            Some((out.get_columns(), out.get_rows()))
        } else {
            None
        }
    }
}

impl<H: Helper> config::Configurer for Editor<H> {
    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    fn set_max_history_size(&mut self, max_size: usize) {
        self.config_mut().set_max_history_size(max_size);
        self.history.set_max_len(max_size);
    }

    fn set_history_ignore_dups(&mut self, yes: bool) {
        self.config_mut().set_history_ignore_dups(yes);
        self.history.ignore_dups = yes;
    }

    fn set_history_ignore_space(&mut self, yes: bool) {
        self.config_mut().set_history_ignore_space(yes);
        self.history.ignore_space = yes;
    }

    fn set_color_mode(&mut self, color_mode: ColorMode) {
        self.config_mut().set_color_mode(color_mode);
        self.term.color_mode = color_mode;
    }
}

impl<H: Helper> fmt::Debug for Editor<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Editor")
            .field("term", &self.term)
            .field("config", &self.config)
            .finish()
    }
}

struct Iter<'a, H: Helper> {
    editor: &'a mut Editor<H>,
    prompt: &'a str,
}

impl<'a, H: Helper> Iterator for Iter<'a, H> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let readline = self.editor.readline(self.prompt);
        match readline {
            Ok(l) => Some(Ok(l)),
            Err(error::ReadlineError::Eof) => None,
            e @ Err(_) => Some(e),
        }
    }
}

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
#[cfg(test)]
mod test;

#[cfg(doctest)]
doc_comment::doctest!("../README.md");
