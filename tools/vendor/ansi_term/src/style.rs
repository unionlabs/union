/// A style is a collection of properties that can format a string
/// using ANSI escape codes.
///
/// # Examples
///
/// ```
/// use ansi_term::{Style, Colour};
///
/// let style = Style::new().bold().on(Colour::Black);
/// println!("{}", style.paint("Bold on black"));
/// ```
#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "derive_serde_style", derive(serde::Deserialize, serde::Serialize))]
pub struct Style {

    /// The style's foreground colour, if it has one.
    pub foreground: Option<Colour>,

    /// The style's background colour, if it has one.
    pub background: Option<Colour>,

    /// Whether this style is bold.
    pub is_bold: bool,

    /// Whether this style is dimmed.
    pub is_dimmed: bool,

    /// Whether this style is italic.
    pub is_italic: bool,

    /// Whether this style is underlined.
    pub is_underline: bool,

    /// Whether this style is blinking.
    pub is_blink: bool,

    /// Whether this style has reverse colours.
    pub is_reverse: bool,

    /// Whether this style is hidden.
    pub is_hidden: bool,

    /// Whether this style is struckthrough.
    pub is_strikethrough: bool
}

impl Style {

    /// Creates a new Style with no properties set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new();
    /// println!("{}", style.paint("hi"));
    /// ```
    pub fn new() -> Style {
        Style::default()
    }

    /// Returns a `Style` with the bold property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().bold();
    /// println!("{}", style.paint("hey"));
    /// ```
    pub fn bold(&self) -> Style {
        Style { is_bold: true, .. *self }
    }

    /// Returns a `Style` with the dimmed property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().dimmed();
    /// println!("{}", style.paint("sup"));
    /// ```
    pub fn dimmed(&self) -> Style {
        Style { is_dimmed: true, .. *self }
    }

    /// Returns a `Style` with the italic property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().italic();
    /// println!("{}", style.paint("greetings"));
    /// ```
    pub fn italic(&self) -> Style {
        Style { is_italic: true, .. *self }
    }

    /// Returns a `Style` with the underline property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().underline();
    /// println!("{}", style.paint("salutations"));
    /// ```
    pub fn underline(&self) -> Style {
        Style { is_underline: true, .. *self }
    }

    /// Returns a `Style` with the blink property set.
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().blink();
    /// println!("{}", style.paint("wazzup"));
    /// ```
    pub fn blink(&self) -> Style {
        Style { is_blink: true, .. *self }
    }

    /// Returns a `Style` with the reverse property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().reverse();
    /// println!("{}", style.paint("aloha"));
    /// ```
    pub fn reverse(&self) -> Style {
        Style { is_reverse: true, .. *self }
    }

    /// Returns a `Style` with the hidden property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().hidden();
    /// println!("{}", style.paint("ahoy"));
    /// ```
    pub fn hidden(&self) -> Style {
        Style { is_hidden: true, .. *self }
    }

    /// Returns a `Style` with the strikethrough property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// let style = Style::new().strikethrough();
    /// println!("{}", style.paint("yo"));
    /// ```
    pub fn strikethrough(&self) -> Style {
        Style { is_strikethrough: true, .. *self }
    }

    /// Returns a `Style` with the foreground colour property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour};
    ///
    /// let style = Style::new().fg(Colour::Yellow);
    /// println!("{}", style.paint("hi"));
    /// ```
    pub fn fg(&self, foreground: Colour) -> Style {
        Style { foreground: Some(foreground), .. *self }
    }

    /// Returns a `Style` with the background colour property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour};
    ///
    /// let style = Style::new().on(Colour::Blue);
    /// println!("{}", style.paint("eyyyy"));
    /// ```
    pub fn on(&self, background: Colour) -> Style {
        Style { background: Some(background), .. *self }
    }

    /// Return true if this `Style` has no actual styles, and can be written
    /// without any control characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Style;
    ///
    /// assert_eq!(true,  Style::default().is_plain());
    /// assert_eq!(false, Style::default().bold().is_plain());
    /// ```
    pub fn is_plain(self) -> bool {
        self == Style::default()
    }
}

impl Default for Style {

    /// Returns a style with *no* properties set. Formatting text using this
    /// style returns the exact same text.
    ///
    /// ```
    /// use ansi_term::Style;
    /// assert_eq!(None,  Style::default().foreground);
    /// assert_eq!(None,  Style::default().background);
    /// assert_eq!(false, Style::default().is_bold);
    /// assert_eq!("txt", Style::default().paint("txt").to_string());
    /// ```
    fn default() -> Style {
        Style {
            foreground: None,
            background: None,
            is_bold: false,
            is_dimmed: false,
            is_italic: false,
            is_underline: false,
            is_blink: false,
            is_reverse: false,
            is_hidden: false,
            is_strikethrough: false,
        }
    }
}


// ---- colours ----

/// A colour is one specific type of ANSI escape code, and can refer
/// to either the foreground or background colour.
///
/// These use the standard numeric sequences.
/// See <http://invisible-island.net/xterm/ctlseqs/ctlseqs.html>
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "derive_serde_style", derive(serde::Deserialize, serde::Serialize))]
pub enum Colour {

    /// Colour #0 (foreground code `30`, background code `40`).
    ///
    /// This is not necessarily the background colour, and using it as one may
    /// render the text hard to read on terminals with dark backgrounds.
    Black,

    /// Colour #1 (foreground code `31`, background code `41`).
    Red,

    /// Colour #2 (foreground code `32`, background code `42`).
    Green,

    /// Colour #3 (foreground code `33`, background code `43`).
    Yellow,

    /// Colour #4 (foreground code `34`, background code `44`).
    Blue,

    /// Colour #5 (foreground code `35`, background code `45`).
    Purple,

    /// Colour #6 (foreground code `36`, background code `46`).
    Cyan,

    /// Colour #7 (foreground code `37`, background code `47`).
    ///
    /// As above, this is not necessarily the foreground colour, and may be
    /// hard to read on terminals with light backgrounds.
    White,

    /// A colour number from 0 to 255, for use in 256-colour terminal
    /// environments.
    ///
    /// - Colours 0 to 7 are the `Black` to `White` variants respectively.
    ///   These colours can usually be changed in the terminal emulator.
    /// - Colours 8 to 15 are brighter versions of the eight colours above.
    ///   These can also usually be changed in the terminal emulator, or it
    ///   could be configured to use the original colours and show the text in
    ///   bold instead. It varies depending on the program.
    /// - Colours 16 to 231 contain several palettes of bright colours,
    ///   arranged in six squares measuring six by six each.
    /// - Colours 232 to 255 are shades of grey from black to white.
    ///
    /// It might make more sense to look at a [colour chart][cc].
    ///
    /// [cc]: https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
    Fixed(u8),

    /// A 24-bit RGB color, as specified by ISO-8613-3.
    RGB(u8, u8, u8),
}


impl Colour {

    /// Returns a `Style` with the foreground colour set to this colour.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Red.normal();
    /// println!("{}", style.paint("hi"));
    /// ```
    pub fn normal(self) -> Style {
        Style { foreground: Some(self), .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// bold property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Green.bold();
    /// println!("{}", style.paint("hey"));
    /// ```
    pub fn bold(self) -> Style {
        Style { foreground: Some(self), is_bold: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// dimmed property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Yellow.dimmed();
    /// println!("{}", style.paint("sup"));
    /// ```
    pub fn dimmed(self) -> Style {
        Style { foreground: Some(self), is_dimmed: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// italic property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Blue.italic();
    /// println!("{}", style.paint("greetings"));
    /// ```
    pub fn italic(self) -> Style {
        Style { foreground: Some(self), is_italic: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// underline property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Purple.underline();
    /// println!("{}", style.paint("salutations"));
    /// ```
    pub fn underline(self) -> Style {
        Style { foreground: Some(self), is_underline: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// blink property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Cyan.blink();
    /// println!("{}", style.paint("wazzup"));
    /// ```
    pub fn blink(self) -> Style {
        Style { foreground: Some(self), is_blink: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// reverse property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Black.reverse();
    /// println!("{}", style.paint("aloha"));
    /// ```
    pub fn reverse(self) -> Style {
        Style { foreground: Some(self), is_reverse: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// hidden property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::White.hidden();
    /// println!("{}", style.paint("ahoy"));
    /// ```
    pub fn hidden(self) -> Style {
        Style { foreground: Some(self), is_hidden: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// strikethrough property set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::Fixed(244).strikethrough();
    /// println!("{}", style.paint("yo"));
    /// ```
    pub fn strikethrough(self) -> Style {
        Style { foreground: Some(self), is_strikethrough: true, .. Style::default() }
    }

    /// Returns a `Style` with the foreground colour set to this colour and the
    /// background colour property set to the given colour.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// let style = Colour::RGB(31, 31, 31).on(Colour::White);
    /// println!("{}", style.paint("eyyyy"));
    /// ```
    pub fn on(self, background: Colour) -> Style {
        Style { foreground: Some(self), background: Some(background), .. Style::default() }
    }
}

impl From<Colour> for Style {

    /// You can turn a `Colour` into a `Style` with the foreground colour set
    /// with the `From` trait.
    ///
    /// ```
    /// use ansi_term::{Style, Colour};
    /// let green_foreground = Style::default().fg(Colour::Green);
    /// assert_eq!(green_foreground, Colour::Green.normal());
    /// assert_eq!(green_foreground, Colour::Green.into());
    /// assert_eq!(green_foreground, Style::from(Colour::Green));
    /// ```
    fn from(colour: Colour) -> Style {
        colour.normal()
    }
}

#[cfg(test)]
#[cfg(feature = "derive_serde_style")]
mod serde_json_tests {
    use super::{Style, Colour};

    #[test]
    fn colour_serialization() {

        let colours = &[
            Colour::Red,
            Colour::Blue,
            Colour::RGB(123, 123, 123),
            Colour::Fixed(255),
        ];

        assert_eq!(serde_json::to_string(&colours).unwrap(), String::from("[\"Red\",\"Blue\",{\"RGB\":[123,123,123]},{\"Fixed\":255}]"));
    }

    #[test]
    fn colour_deserialization() {
        let colours = &[
            Colour::Red,
            Colour::Blue,
            Colour::RGB(123, 123, 123),
            Colour::Fixed(255),
        ];

        for colour in colours.into_iter() {
            let serialized = serde_json::to_string(&colour).unwrap();
            let deserialized: Colour = serde_json::from_str(&serialized).unwrap();

            assert_eq!(colour, &deserialized);
        }
    }

    #[test]
    fn style_serialization() {
        let style = Style::default();

        assert_eq!(serde_json::to_string(&style).unwrap(), "{\"foreground\":null,\"background\":null,\"is_bold\":false,\"is_dimmed\":false,\"is_italic\":false,\"is_underline\":false,\"is_blink\":false,\"is_reverse\":false,\"is_hidden\":false,\"is_strikethrough\":false}".to_string());
    }
}
