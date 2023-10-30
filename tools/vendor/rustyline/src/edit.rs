//! Command processor

use log::debug;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use super::{Context, Helper, Result};
use crate::highlight::Highlighter;
use crate::hint::Hint;
use crate::history::SearchDirection;
use crate::keymap::{Anchor, At, CharSearch, Cmd, Movement, RepeatCount, Word};
use crate::keymap::{InputState, Invoke, Refresher};
use crate::layout::{Layout, Position};
use crate::line_buffer::{LineBuffer, WordAction, MAX_LINE};
use crate::tty::{Renderer, Term, Terminal};
use crate::undo::Changeset;
use crate::validate::{ValidationContext, ValidationResult};

/// Represent the state during line editing.
/// Implement rendering.
pub struct State<'out, 'prompt, H: Helper> {
    pub out: &'out mut <Terminal as Term>::Writer,
    prompt: &'prompt str,  // Prompt to display (rl_prompt)
    prompt_size: Position, // Prompt Unicode/visible width and height
    pub line: LineBuffer,  // Edited line buffer
    pub layout: Layout,
    saved_line_for_history: LineBuffer, // Current edited line before history browsing
    byte_buffer: [u8; 4],
    pub changes: Rc<RefCell<Changeset>>, // changes to line, for undo/redo
    pub helper: Option<&'out H>,
    pub ctx: Context<'out>,          // Give access to history for `hinter`
    pub hint: Option<Box<dyn Hint>>, // last hint displayed
    highlight_char: bool,            // `true` if a char has been highlighted
}

enum Info<'m> {
    NoHint,
    Hint,
    Msg(Option<&'m str>),
}

impl<'out, 'prompt, H: Helper> State<'out, 'prompt, H> {
    pub fn new(
        out: &'out mut <Terminal as Term>::Writer,
        prompt: &'prompt str,
        helper: Option<&'out H>,
        ctx: Context<'out>,
    ) -> State<'out, 'prompt, H> {
        let prompt_size = out.calculate_position(prompt, Position::default());
        State {
            out,
            prompt,
            prompt_size,
            line: LineBuffer::with_capacity(MAX_LINE).can_growth(true),
            layout: Layout::default(),
            saved_line_for_history: LineBuffer::with_capacity(MAX_LINE).can_growth(true),
            byte_buffer: [0; 4],
            changes: Rc::new(RefCell::new(Changeset::new())),
            helper,
            ctx,
            hint: None,
            highlight_char: false,
        }
    }

    pub fn highlighter(&self) -> Option<&dyn Highlighter> {
        if self.out.colors_enabled() {
            self.helper.map(|h| h as &dyn Highlighter)
        } else {
            None
        }
    }

    pub fn next_cmd(
        &mut self,
        input_state: &mut InputState,
        rdr: &mut <Terminal as Term>::Reader,
        single_esc_abort: bool,
    ) -> Result<Cmd> {
        loop {
            let rc = input_state.next_cmd(rdr, self, single_esc_abort);
            if rc.is_err() && self.out.sigwinch() {
                self.out.update_size();
                self.prompt_size = self
                    .out
                    .calculate_position(self.prompt, Position::default());
                self.refresh_line()?;
                continue;
            }
            if let Ok(Cmd::Replace(..)) = rc {
                self.changes.borrow_mut().begin();
            }
            return rc;
        }
    }

    pub fn backup(&mut self) {
        self.saved_line_for_history
            .update(self.line.as_str(), self.line.pos());
    }

    pub fn restore(&mut self) {
        self.line.update(
            self.saved_line_for_history.as_str(),
            self.saved_line_for_history.pos(),
        );
    }

    pub fn move_cursor(&mut self) -> Result<()> {
        // calculate the desired position of the cursor
        let cursor = self
            .out
            .calculate_position(&self.line[..self.line.pos()], self.prompt_size);
        if self.layout.cursor == cursor {
            return Ok(());
        }
        if self.highlight_char() {
            let prompt_size = self.prompt_size;
            self.refresh(self.prompt, prompt_size, true, Info::NoHint)?;
        } else {
            self.out.move_cursor(self.layout.cursor, cursor)?;
            self.layout.prompt_size = self.prompt_size;
            self.layout.cursor = cursor;
            debug_assert!(self.layout.prompt_size <= self.layout.cursor);
            debug_assert!(self.layout.cursor <= self.layout.end);
        }
        Ok(())
    }

    pub fn move_cursor_to_end(&mut self) -> Result<()> {
        if self.layout.cursor == self.layout.end {
            return Ok(());
        }
        self.out.move_cursor(self.layout.cursor, self.layout.end)?;
        self.layout.cursor = self.layout.end;
        Ok(())
    }

    pub fn move_cursor_at_leftmost(&mut self, rdr: &mut <Terminal as Term>::Reader) -> Result<()> {
        self.out.move_cursor_at_leftmost(rdr)
    }

    fn refresh(
        &mut self,
        prompt: &str,
        prompt_size: Position,
        default_prompt: bool,
        info: Info<'_>,
    ) -> Result<()> {
        let info = match info {
            Info::NoHint => None,
            Info::Hint => self.hint.as_ref().map(|h| h.display()),
            Info::Msg(msg) => msg,
        };
        let highlighter = if self.out.colors_enabled() {
            self.helper.map(|h| h as &dyn Highlighter)
        } else {
            None
        };

        let new_layout = self
            .out
            .compute_layout(prompt_size, default_prompt, &self.line, info);

        debug!(target: "rustyline", "old layout: {:?}", self.layout);
        debug!(target: "rustyline", "new layout: {:?}", new_layout);
        self.out.refresh_line(
            prompt,
            &self.line,
            info,
            &self.layout,
            &new_layout,
            highlighter,
        )?;
        self.layout = new_layout;

        Ok(())
    }

    pub fn hint(&mut self) {
        if let Some(hinter) = self.helper {
            let hint = hinter.hint(self.line.as_str(), self.line.pos(), &self.ctx);
            self.hint = match hint {
                Some(val) if !val.display().is_empty() => Some(Box::new(val) as Box<dyn Hint>),
                _ => None,
            };
        } else {
            self.hint = None;
        }
    }

    fn highlight_char(&mut self) -> bool {
        if let Some(highlighter) = self.highlighter() {
            let highlight_char = highlighter.highlight_char(&self.line, self.line.pos());
            if highlight_char {
                self.highlight_char = true;
                true
            } else if self.highlight_char {
                // previously highlighted => force a full refresh
                self.highlight_char = false;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn is_default_prompt(&self) -> bool {
        self.layout.default_prompt
    }

    pub fn validate(&mut self) -> Result<ValidationResult> {
        if let Some(validator) = self.helper {
            self.changes.borrow_mut().begin();
            let result = validator.validate(&mut ValidationContext::new(self))?;
            let corrected = self.changes.borrow_mut().end();
            match result {
                ValidationResult::Incomplete => {}
                ValidationResult::Valid(ref msg) => {
                    // Accept the line regardless of where the cursor is.
                    if corrected || self.has_hint() || msg.is_some() {
                        // Force a refresh without hints to leave the previous
                        // line as the user typed it after a newline.
                        self.refresh_line_with_msg(msg.as_deref())?;
                    }
                }
                ValidationResult::Invalid(ref msg) => {
                    if corrected || self.has_hint() || msg.is_some() {
                        self.refresh_line_with_msg(msg.as_deref())?;
                    }
                }
            }
            Ok(result)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl<'out, 'prompt, H: Helper> Invoke for State<'out, 'prompt, H> {
    fn input(&self) -> &str {
        self.line.as_str()
    }
}

impl<'out, 'prompt, H: Helper> Refresher for State<'out, 'prompt, H> {
    fn refresh_line(&mut self) -> Result<()> {
        let prompt_size = self.prompt_size;
        self.hint();
        self.highlight_char();
        self.refresh(self.prompt, prompt_size, true, Info::Hint)
    }

    fn refresh_line_with_msg(&mut self, msg: Option<&str>) -> Result<()> {
        let prompt_size = self.prompt_size;
        self.hint = None;
        self.highlight_char();
        self.refresh(self.prompt, prompt_size, true, Info::Msg(msg))
    }

    fn refresh_prompt_and_line(&mut self, prompt: &str) -> Result<()> {
        let prompt_size = self.out.calculate_position(prompt, Position::default());
        self.hint();
        self.highlight_char();
        self.refresh(prompt, prompt_size, false, Info::Hint)
    }

    fn doing_insert(&mut self) {
        self.changes.borrow_mut().begin();
    }

    fn done_inserting(&mut self) {
        self.changes.borrow_mut().end();
    }

    fn last_insert(&self) -> Option<String> {
        self.changes.borrow().last_insert()
    }

    fn is_cursor_at_end(&self) -> bool {
        self.line.pos() == self.line.len()
    }

    fn has_hint(&self) -> bool {
        self.hint.is_some()
    }

    fn hint_text(&self) -> Option<&str> {
        self.hint.as_ref().and_then(|hint| hint.completion())
    }

    fn line(&self) -> &str {
        self.line.as_str()
    }

    fn pos(&self) -> usize {
        self.line.pos()
    }
}

impl<'out, 'prompt, H: Helper> fmt::Debug for State<'out, 'prompt, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("prompt", &self.prompt)
            .field("prompt_size", &self.prompt_size)
            .field("buf", &self.line)
            .field("cols", &self.out.get_columns())
            .field("layout", &self.layout)
            .field("saved_line_for_history", &self.saved_line_for_history)
            .finish()
    }
}

impl<'out, 'prompt, H: Helper> State<'out, 'prompt, H> {
    pub fn clear_screen(&mut self) -> Result<()> {
        self.out.clear_screen()?;
        self.layout.cursor = Position::default();
        self.layout.end = Position::default();
        Ok(())
    }

    /// Insert the character `ch` at cursor current position.
    pub fn edit_insert(&mut self, ch: char, n: RepeatCount) -> Result<()> {
        if let Some(push) = self.line.insert(ch, n) {
            if push {
                let prompt_size = self.prompt_size;
                let no_previous_hint = self.hint.is_none();
                self.hint();
                let width = ch.width().unwrap_or(0);
                if n == 1
                    && width != 0 // Ctrl-V + \t or \n ...
                    && self.layout.cursor.col + width < self.out.get_columns()
                    && (self.hint.is_none() && no_previous_hint) // TODO refresh only current line
                    && !self.highlight_char()
                {
                    // Avoid a full update of the line in the trivial case.
                    self.layout.cursor.col += width;
                    self.layout.end.col += width;
                    debug_assert!(self.layout.prompt_size <= self.layout.cursor);
                    debug_assert!(self.layout.cursor <= self.layout.end);
                    let bits = ch.encode_utf8(&mut self.byte_buffer);
                    let bits = bits.as_bytes();
                    self.out.write_and_flush(bits)
                } else {
                    self.refresh(self.prompt, prompt_size, true, Info::Hint)
                }
            } else {
                self.refresh_line()
            }
        } else {
            Ok(())
        }
    }

    /// Replace a single (or n) character(s) under the cursor (Vi mode)
    pub fn edit_replace_char(&mut self, ch: char, n: RepeatCount) -> Result<()> {
        self.changes.borrow_mut().begin();
        let succeed = if let Some(chars) = self.line.delete(n) {
            let count = chars.graphemes(true).count();
            self.line.insert(ch, count);
            self.line.move_backward(1);
            true
        } else {
            false
        };
        self.changes.borrow_mut().end();
        if succeed {
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    /// Overwrite the character under the cursor (Vi mode)
    pub fn edit_overwrite_char(&mut self, ch: char) -> Result<()> {
        if let Some(end) = self.line.next_pos(1) {
            {
                let text = ch.encode_utf8(&mut self.byte_buffer);
                let start = self.line.pos();
                self.line.replace(start..end, text);
            }
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    // Yank/paste `text` at current position.
    pub fn edit_yank(
        &mut self,
        input_state: &InputState,
        text: &str,
        anchor: Anchor,
        n: RepeatCount,
    ) -> Result<()> {
        if let Anchor::After = anchor {
            self.line.move_forward(1);
        }
        if self.line.yank(text, n).is_some() {
            if !input_state.is_emacs_mode() {
                self.line.move_backward(1);
            }
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    // Delete previously yanked text and yank/paste `text` at current position.
    pub fn edit_yank_pop(&mut self, yank_size: usize, text: &str) -> Result<()> {
        self.changes.borrow_mut().begin();
        let result = if self.line.yank_pop(yank_size, text).is_some() {
            self.refresh_line()
        } else {
            Ok(())
        };
        self.changes.borrow_mut().end();
        result
    }

    /// Move cursor on the left.
    pub fn edit_move_backward(&mut self, n: RepeatCount) -> Result<()> {
        if self.line.move_backward(n) {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    /// Move cursor on the right.
    pub fn edit_move_forward(&mut self, n: RepeatCount) -> Result<()> {
        if self.line.move_forward(n) {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    /// Move cursor to the start of the line.
    pub fn edit_move_home(&mut self) -> Result<()> {
        if self.line.move_home() {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    /// Move cursor to the end of the line.
    pub fn edit_move_end(&mut self) -> Result<()> {
        if self.line.move_end() {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    /// Move cursor to the start of the buffer.
    pub fn edit_move_buffer_start(&mut self) -> Result<()> {
        if self.line.move_buffer_start() {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    /// Move cursor to the end of the buffer.
    pub fn edit_move_buffer_end(&mut self) -> Result<()> {
        if self.line.move_buffer_end() {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    pub fn edit_kill(&mut self, mvt: &Movement) -> Result<()> {
        if self.line.kill(mvt) {
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    pub fn edit_insert_text(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let cursor = self.line.pos();
        self.line.insert_str(cursor, text);
        self.refresh_line()
    }

    /// Exchange the char before cursor with the character at cursor.
    pub fn edit_transpose_chars(&mut self) -> Result<()> {
        self.changes.borrow_mut().begin();
        let succeed = self.line.transpose_chars();
        self.changes.borrow_mut().end();
        if succeed {
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    pub fn edit_move_to_prev_word(&mut self, word_def: Word, n: RepeatCount) -> Result<()> {
        if self.line.move_to_prev_word(word_def, n) {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    pub fn edit_move_to_next_word(&mut self, at: At, word_def: Word, n: RepeatCount) -> Result<()> {
        if self.line.move_to_next_word(at, word_def, n) {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    /// Moves the cursor to the same column in the line above
    pub fn edit_move_line_up(&mut self, n: RepeatCount) -> Result<bool> {
        if self.line.move_to_line_up(n) {
            self.move_cursor()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Moves the cursor to the same column in the line above
    pub fn edit_move_line_down(&mut self, n: RepeatCount) -> Result<bool> {
        if self.line.move_to_line_down(n) {
            self.move_cursor()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn edit_move_to(&mut self, cs: CharSearch, n: RepeatCount) -> Result<()> {
        if self.line.move_to(cs, n) {
            self.move_cursor()
        } else {
            Ok(())
        }
    }

    pub fn edit_word(&mut self, a: WordAction) -> Result<()> {
        self.changes.borrow_mut().begin();
        let succeed = self.line.edit_word(a);
        self.changes.borrow_mut().end();
        if succeed {
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    pub fn edit_transpose_words(&mut self, n: RepeatCount) -> Result<()> {
        self.changes.borrow_mut().begin();
        let succeed = self.line.transpose_words(n);
        self.changes.borrow_mut().end();
        if succeed {
            self.refresh_line()
        } else {
            Ok(())
        }
    }

    /// Substitute the currently edited line with the next or previous history
    /// entry.
    pub fn edit_history_next(&mut self, prev: bool) -> Result<()> {
        let history = self.ctx.history;
        if history.is_empty() {
            return Ok(());
        }
        if self.ctx.history_index == history.len() {
            if prev {
                // Save the current edited line before overwriting it
                self.backup();
            } else {
                return Ok(());
            }
        } else if self.ctx.history_index == 0 && prev {
            return Ok(());
        }
        if prev {
            self.ctx.history_index -= 1;
        } else {
            self.ctx.history_index += 1;
        }
        if self.ctx.history_index < history.len() {
            let buf = history.get(self.ctx.history_index).unwrap();
            self.changes.borrow_mut().begin();
            self.line.update(buf, buf.len());
            self.changes.borrow_mut().end();
        } else {
            // Restore current edited line
            self.restore();
        }
        self.refresh_line()
    }

    // Non-incremental, anchored search
    pub fn edit_history_search(&mut self, dir: SearchDirection) -> Result<()> {
        let history = self.ctx.history;
        if history.is_empty() {
            return self.out.beep();
        }
        if self.ctx.history_index == history.len() && dir == SearchDirection::Forward
            || self.ctx.history_index == 0 && dir == SearchDirection::Reverse
        {
            return self.out.beep();
        }
        if dir == SearchDirection::Reverse {
            self.ctx.history_index -= 1;
        } else {
            self.ctx.history_index += 1;
        }
        if let Some(sr) = history.starts_with(
            &self.line.as_str()[..self.line.pos()],
            self.ctx.history_index,
            dir,
        ) {
            self.ctx.history_index = sr.idx;
            self.changes.borrow_mut().begin();
            self.line.update(sr.entry, sr.pos);
            self.changes.borrow_mut().end();
            self.refresh_line()
        } else {
            self.out.beep()
        }
    }

    /// Substitute the currently edited line with the first/last history entry.
    pub fn edit_history(&mut self, first: bool) -> Result<()> {
        let history = self.ctx.history;
        if history.is_empty() {
            return Ok(());
        }
        if self.ctx.history_index == history.len() {
            if first {
                // Save the current edited line before overwriting it
                self.backup();
            } else {
                return Ok(());
            }
        } else if self.ctx.history_index == 0 && first {
            return Ok(());
        }
        if first {
            self.ctx.history_index = 0;
            let buf = history.get(self.ctx.history_index).unwrap();
            self.changes.borrow_mut().begin();
            self.line.update(buf, buf.len());
            self.changes.borrow_mut().end();
        } else {
            self.ctx.history_index = history.len();
            // Restore current edited line
            self.restore();
        }
        self.refresh_line()
    }

    /// Change the indentation of the lines covered by movement
    pub fn edit_indent(&mut self, mvt: &Movement, amount: usize, dedent: bool) -> Result<()> {
        if self.line.indent(mvt, amount, dedent) {
            self.refresh_line()
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub fn init_state<'out, H: Helper>(
    out: &'out mut <Terminal as Term>::Writer,
    line: &str,
    pos: usize,
    helper: Option<&'out H>,
    history: &'out crate::history::History,
) -> State<'out, 'static, H> {
    State {
        out,
        prompt: "",
        prompt_size: Position::default(),
        line: LineBuffer::init(line, pos, None),
        layout: Layout::default(),
        saved_line_for_history: LineBuffer::with_capacity(100),
        byte_buffer: [0; 4],
        changes: Rc::new(RefCell::new(Changeset::new())),
        helper,
        ctx: Context::new(history),
        hint: Some(Box::new("hint".to_owned())),
        highlight_char: false,
    }
}

#[cfg(test)]
mod test {
    use super::init_state;
    use crate::history::History;
    use crate::tty::Sink;

    #[test]
    fn edit_history_next() {
        let mut out = Sink::new();
        let mut history = History::new();
        history.add("line0");
        history.add("line1");
        let line = "current edited line";
        let helper: Option<()> = None;
        let mut s = init_state(&mut out, line, 6, helper.as_ref(), &history);
        s.ctx.history_index = history.len();

        for _ in 0..2 {
            s.edit_history_next(false).unwrap();
            assert_eq!(line, s.line.as_str());
        }

        s.edit_history_next(true).unwrap();
        assert_eq!(line, s.saved_line_for_history.as_str());
        assert_eq!(1, s.ctx.history_index);
        assert_eq!("line1", s.line.as_str());

        for _ in 0..2 {
            s.edit_history_next(true).unwrap();
            assert_eq!(line, s.saved_line_for_history.as_str());
            assert_eq!(0, s.ctx.history_index);
            assert_eq!("line0", s.line.as_str());
        }

        s.edit_history_next(false).unwrap();
        assert_eq!(line, s.saved_line_for_history.as_str());
        assert_eq!(1, s.ctx.history_index);
        assert_eq!("line1", s.line.as_str());

        s.edit_history_next(false).unwrap();
        // assert_eq!(line, s.saved_line_for_history);
        assert_eq!(2, s.ctx.history_index);
        assert_eq!(line, s.line.as_str());
    }
}
