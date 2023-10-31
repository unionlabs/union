/// Incrementally convert to wincon calls for non-contiguous data
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct WinconBytes {
    parser: anstyle_parse::Parser,
    capture: WinconCapture,
}

impl WinconBytes {
    /// Initial state
    pub fn new() -> Self {
        Default::default()
    }

    /// Strip the next segment of data
    pub fn extract_next<'s>(&'s mut self, bytes: &'s [u8]) -> WinconBytesIter<'s> {
        self.capture.reset();
        self.capture.printable.reserve(bytes.len());
        WinconBytesIter {
            bytes,
            parser: &mut self.parser,
            capture: &mut self.capture,
        }
    }
}

/// See [`WinconBytes`]
#[derive(Debug, PartialEq, Eq)]
pub struct WinconBytesIter<'s> {
    bytes: &'s [u8],
    parser: &'s mut anstyle_parse::Parser,
    capture: &'s mut WinconCapture,
}

impl<'s> Iterator for WinconBytesIter<'s> {
    type Item = (anstyle::Style, String);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        next_bytes(&mut self.bytes, self.parser, self.capture)
    }
}

#[inline]
fn next_bytes(
    bytes: &mut &[u8],
    parser: &mut anstyle_parse::Parser,
    capture: &mut WinconCapture,
) -> Option<(anstyle::Style, String)> {
    capture.reset();
    while capture.ready.is_none() {
        let byte = if let Some((byte, remainder)) = (*bytes).split_first() {
            *bytes = remainder;
            *byte
        } else {
            break;
        };
        parser.advance(capture, byte);
    }
    if capture.printable.is_empty() {
        return None;
    }

    let style = capture.ready.unwrap_or(capture.style);
    Some((style, std::mem::take(&mut capture.printable)))
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct WinconCapture {
    style: anstyle::Style,
    printable: String,
    ready: Option<anstyle::Style>,
}

impl WinconCapture {
    fn reset(&mut self) {
        self.ready = None;
    }
}

impl anstyle_parse::Perform for WinconCapture {
    /// Draw a character to the screen and update states.
    fn print(&mut self, c: char) {
        self.printable.push(c);
    }

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, byte: u8) {
        if byte.is_ascii_whitespace() {
            self.printable.push(byte as char);
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &anstyle_parse::Params,
        _intermediates: &[u8],
        ignore: bool,
        action: u8,
    ) {
        if ignore {
            return;
        }
        if action != b'm' {
            return;
        }

        let mut style = self.style;
        for param in params {
            let mut state = State::Normal;
            let mut r = None;
            let mut g = None;
            let mut is_bg = false;
            for value in param {
                match (state, *value) {
                    (State::Normal, 0) => {
                        style = anstyle::Style::default();
                        break;
                    }
                    (State::Normal, 1) => {
                        style = style.bold();
                        break;
                    }
                    (State::Normal, 4) => {
                        style = style.underline();
                        break;
                    }
                    (State::Normal, 30..=37) => {
                        let color = to_ansi_color(value - 30).unwrap();
                        style = style.fg_color(Some(color.into()));
                        break;
                    }
                    (State::Normal, 38) => {
                        is_bg = false;
                        state = State::PrepareCustomColor;
                    }
                    (State::Normal, 39) => {
                        style = style.fg_color(None);
                        break;
                    }
                    (State::Normal, 40..=47) => {
                        let color = to_ansi_color(value - 40).unwrap();
                        style = style.bg_color(Some(color.into()));
                        break;
                    }
                    (State::Normal, 48) => {
                        is_bg = true;
                        state = State::PrepareCustomColor;
                    }
                    (State::Normal, 49) => {
                        style = style.bg_color(None);
                        break;
                    }
                    (State::Normal, 90..=97) => {
                        let color = to_ansi_color(value - 90).unwrap().bright(true);
                        style = style.fg_color(Some(color.into()));
                        break;
                    }
                    (State::Normal, 100..=107) => {
                        let color = to_ansi_color(value - 100).unwrap().bright(true);
                        style = style.bg_color(Some(color.into()));
                        break;
                    }
                    (State::PrepareCustomColor, 5) => {
                        state = State::Ansi256;
                    }
                    (State::PrepareCustomColor, 2) => {
                        state = State::Rgb;
                        r = None;
                        g = None;
                    }
                    (State::Ansi256, n) => {
                        let color = anstyle::Ansi256Color(n as u8);
                        if is_bg {
                            style = style.bg_color(Some(color.into()));
                        } else {
                            style = style.fg_color(Some(color.into()));
                        }
                        break;
                    }
                    (State::Rgb, b) => match (r, g) {
                        (None, _) => {
                            r = Some(b);
                        }
                        (Some(_), None) => {
                            g = Some(b);
                        }
                        (Some(r), Some(g)) => {
                            let color = anstyle::RgbColor(r as u8, g as u8, b as u8);
                            if is_bg {
                                style = style.bg_color(Some(color.into()));
                            } else {
                                style = style.fg_color(Some(color.into()));
                            }
                            break;
                        }
                    },
                    _ => {
                        break;
                    }
                }
            }
        }

        if style != self.style && !self.printable.is_empty() {
            self.ready = Some(self.style);
        }
        self.style = style;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum State {
    Normal,
    PrepareCustomColor,
    Ansi256,
    Rgb,
}

fn to_ansi_color(digit: u16) -> Option<anstyle::AnsiColor> {
    match digit {
        0 => Some(anstyle::AnsiColor::Black),
        1 => Some(anstyle::AnsiColor::Red),
        2 => Some(anstyle::AnsiColor::Green),
        3 => Some(anstyle::AnsiColor::Yellow),
        4 => Some(anstyle::AnsiColor::Blue),
        5 => Some(anstyle::AnsiColor::Magenta),
        6 => Some(anstyle::AnsiColor::Cyan),
        7 => Some(anstyle::AnsiColor::White),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use owo_colors::OwoColorize as _;
    use proptest::prelude::*;

    #[track_caller]
    fn verify(input: &str, expected: Vec<(anstyle::Style, &str)>) {
        let expected = expected
            .into_iter()
            .map(|(style, value)| (style, value.to_owned()))
            .collect::<Vec<_>>();
        let mut state = WinconBytes::new();
        let actual = state.extract_next(input.as_bytes()).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn start() {
        let input = format!("{} world!", "Hello".green().on_red());
        let expected = vec![
            (
                anstyle::AnsiColor::Green.on(anstyle::AnsiColor::Red),
                "Hello",
            ),
            (anstyle::Style::default(), " world!"),
        ];
        verify(&input, expected);
    }

    #[test]
    fn middle() {
        let input = format!("Hello {}!", "world".green().on_red());
        let expected = vec![
            (anstyle::Style::default(), "Hello "),
            (
                anstyle::AnsiColor::Green.on(anstyle::AnsiColor::Red),
                "world",
            ),
            (anstyle::Style::default(), "!"),
        ];
        verify(&input, expected);
    }

    #[test]
    fn end() {
        let input = format!("Hello {}", "world!".green().on_red());
        let expected = vec![
            (anstyle::Style::default(), "Hello "),
            (
                anstyle::AnsiColor::Green.on(anstyle::AnsiColor::Red),
                "world!",
            ),
        ];
        verify(&input, expected);
    }

    proptest! {
        #[test]
        #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
        fn wincon_no_escapes(s in "\\PC*") {
            let expected = if s.is_empty() {
                vec![]
            } else {
                vec![(anstyle::Style::default(), s.clone())]
            };
            let mut state = WinconBytes::new();
            let actual = state.extract_next(s.as_bytes()).collect::<Vec<_>>();
            assert_eq!(expected, actual);
        }
    }
}
