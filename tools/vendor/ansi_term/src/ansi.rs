use style::{Colour, Style};

use std::fmt;

use write::AnyWrite;


// ---- generating ANSI codes ----

impl Style {

    /// Write any bytes that go *before* a piece of text to the given writer.
    fn write_prefix<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {

        // If there are actually no styles here, then don’t write *any* codes
        // as the prefix. An empty ANSI code may not affect the terminal
        // output at all, but a user may just want a code-free string.
        if self.is_plain() {
            return Ok(());
        }

        // Write the codes’ prefix, then write numbers, separated by
        // semicolons, for each text style we want to apply.
        write!(f, "\x1B[")?;
        let mut written_anything = false;

        {
            let mut write_char = |c| {
                if written_anything { write!(f, ";")?; }
                written_anything = true;
                write!(f, "{}", c)?;
                Ok(())
            };

            if self.is_bold           { write_char('1')? }
            if self.is_dimmed         { write_char('2')? }
            if self.is_italic         { write_char('3')? }
            if self.is_underline      { write_char('4')? }
            if self.is_blink          { write_char('5')? }
            if self.is_reverse        { write_char('7')? }
            if self.is_hidden         { write_char('8')? }
            if self.is_strikethrough  { write_char('9')? }
        }

        // The foreground and background colours, if specified, need to be
        // handled specially because the number codes are more complicated.
        // (see `write_background_code` and `write_foreground_code`)
        if let Some(bg) = self.background {
            if written_anything { write!(f, ";")?; }
            written_anything = true;
            bg.write_background_code(f)?;
        }

        if let Some(fg) = self.foreground {
            if written_anything { write!(f, ";")?; }
            fg.write_foreground_code(f)?;
        }

        // All the codes end with an `m`, because reasons.
        write!(f, "m")?;

        Ok(())
    }

    /// Write any bytes that go *after* a piece of text to the given writer.
    fn write_suffix<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
        if self.is_plain() {
            Ok(())
        }
        else {
            write!(f, "{}", RESET)
        }
    }
}


/// The code to send to reset all styles and return to `Style::default()`.
pub static RESET: &str = "\x1B[0m";



impl Colour {
    fn write_foreground_code<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
        match *self {
            Colour::Black      => write!(f, "30"),
            Colour::Red        => write!(f, "31"),
            Colour::Green      => write!(f, "32"),
            Colour::Yellow     => write!(f, "33"),
            Colour::Blue       => write!(f, "34"),
            Colour::Purple     => write!(f, "35"),
            Colour::Cyan       => write!(f, "36"),
            Colour::White      => write!(f, "37"),
            Colour::Fixed(num) => write!(f, "38;5;{}", &num),
            Colour::RGB(r,g,b) => write!(f, "38;2;{};{};{}", &r, &g, &b),
        }
    }

    fn write_background_code<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
        match *self {
            Colour::Black      => write!(f, "40"),
            Colour::Red        => write!(f, "41"),
            Colour::Green      => write!(f, "42"),
            Colour::Yellow     => write!(f, "43"),
            Colour::Blue       => write!(f, "44"),
            Colour::Purple     => write!(f, "45"),
            Colour::Cyan       => write!(f, "46"),
            Colour::White      => write!(f, "47"),
            Colour::Fixed(num) => write!(f, "48;5;{}", &num),
            Colour::RGB(r,g,b) => write!(f, "48;2;{};{};{}", &r, &g, &b),
        }
    }
}


/// Like `ANSIString`, but only displays the style prefix.
///
/// This type implements the `Display` trait, meaning it can be written to a
/// `std::fmt` formatting without doing any extra allocation, and written to a
/// string with the `.to_string()` method. For examples, see
/// [`Style::prefix`](struct.Style.html#method.prefix).
#[derive(Clone, Copy, Debug)]
pub struct Prefix(Style);

/// Like `ANSIString`, but only displays the difference between two
/// styles.
///
/// This type implements the `Display` trait, meaning it can be written to a
/// `std::fmt` formatting without doing any extra allocation, and written to a
/// string with the `.to_string()` method. For examples, see
/// [`Style::infix`](struct.Style.html#method.infix).
#[derive(Clone, Copy, Debug)]
pub struct Infix(Style, Style);

/// Like `ANSIString`, but only displays the style suffix.
///
/// This type implements the `Display` trait, meaning it can be written to a
/// `std::fmt` formatting without doing any extra allocation, and written to a
/// string with the `.to_string()` method. For examples, see
/// [`Style::suffix`](struct.Style.html#method.suffix).
#[derive(Clone, Copy, Debug)]
pub struct Suffix(Style);


impl Style {

    /// The prefix bytes for this style. These are the bytes that tell the
    /// terminal to use a different colour or font style.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour::Blue};
    ///
    /// let style = Style::default().bold();
    /// assert_eq!("\x1b[1m",
    ///            style.prefix().to_string());
    ///
    /// let style = Blue.bold();
    /// assert_eq!("\x1b[1;34m",
    ///            style.prefix().to_string());
    ///
    /// let style = Style::default();
    /// assert_eq!("",
    ///            style.prefix().to_string());
    /// ```
    pub fn prefix(self) -> Prefix {
        Prefix(self)
    }

    /// The infix bytes between this style and `next` style. These are the bytes
    /// that tell the terminal to change the style to `next`. These may include
    /// a reset followed by the next colour and style, depending on the two styles.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour::Green};
    ///
    /// let style = Style::default().bold();
    /// assert_eq!("\x1b[32m",
    ///            style.infix(Green.bold()).to_string());
    ///
    /// let style = Green.normal();
    /// assert_eq!("\x1b[1m",
    ///            style.infix(Green.bold()).to_string());
    ///
    /// let style = Style::default();
    /// assert_eq!("",
    ///            style.infix(style).to_string());
    /// ```
    pub fn infix(self, next: Style) -> Infix {
        Infix(self, next)
    }

    /// The suffix for this style. These are the bytes that tell the terminal
    /// to reset back to its normal colour and font style.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour::Green};
    ///
    /// let style = Style::default().bold();
    /// assert_eq!("\x1b[0m",
    ///            style.suffix().to_string());
    ///
    /// let style = Green.normal().bold();
    /// assert_eq!("\x1b[0m",
    ///            style.suffix().to_string());
    ///
    /// let style = Style::default();
    /// assert_eq!("",
    ///            style.suffix().to_string());
    /// ```
    pub fn suffix(self) -> Suffix {
        Suffix(self)
    }
}


impl Colour {

    /// The prefix bytes for this colour as a `Style`. These are the bytes
    /// that tell the terminal to use a different colour or font style.
    ///
    /// See also [`Style::prefix`](struct.Style.html#method.prefix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour::Green;
    ///
    /// assert_eq!("\x1b[0m",
    ///            Green.suffix().to_string());
    /// ```
    pub fn prefix(self) -> Prefix {
        Prefix(self.normal())
    }

    /// The infix bytes between this colour and `next` colour. These are the bytes
    /// that tell the terminal to use the `next` colour, or to do nothing if
    /// the two colours are equal.
    ///
    /// See also [`Style::infix`](struct.Style.html#method.infix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour::{Red, Yellow};
    ///
    /// assert_eq!("\x1b[33m",
    ///            Red.infix(Yellow).to_string());
    /// ```
    pub fn infix(self, next: Colour) -> Infix {
        Infix(self.normal(), next.normal())
    }

    /// The suffix for this colour as a `Style`. These are the bytes that
    /// tell the terminal to reset back to its normal colour and font style.
    ///
    /// See also [`Style::suffix`](struct.Style.html#method.suffix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour::Purple;
    ///
    /// assert_eq!("\x1b[0m",
    ///            Purple.suffix().to_string());
    /// ```
    pub fn suffix(self) -> Suffix {
        Suffix(self.normal())
    }
}


impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        self.0.write_prefix(f)
    }
}


impl fmt::Display for Infix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use difference::Difference;

        match Difference::between(&self.0, &self.1) {
            Difference::ExtraStyles(style) => {
                let f: &mut fmt::Write = f;
                style.write_prefix(f)
            },
            Difference::Reset => {
                let f: &mut fmt::Write = f;
                write!(f, "{}{}", RESET, self.1.prefix())
            },
            Difference::NoDifference => {
                Ok(())   // nothing to write
            },
        }
    }
}


impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        self.0.write_suffix(f)
    }
}



#[cfg(test)]
mod test {
    use style::Style;
    use style::Colour::*;

    macro_rules! test {
        ($name: ident: $style: expr; $input: expr => $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($style.paint($input).to_string(), $result.to_string());

                let mut v = Vec::new();
                $style.paint($input.as_bytes()).write_to(&mut v).unwrap();
                assert_eq!(v.as_slice(), $result.as_bytes());
            }
        };
    }

    test!(plain:                 Style::default();                  "text/plain" => "text/plain");
    test!(red:                   Red;                               "hi" => "\x1B[31mhi\x1B[0m");
    test!(black:                 Black.normal();                    "hi" => "\x1B[30mhi\x1B[0m");
    test!(yellow_bold:           Yellow.bold();                     "hi" => "\x1B[1;33mhi\x1B[0m");
    test!(yellow_bold_2:         Yellow.normal().bold();            "hi" => "\x1B[1;33mhi\x1B[0m");
    test!(blue_underline:        Blue.underline();                  "hi" => "\x1B[4;34mhi\x1B[0m");
    test!(green_bold_ul:         Green.bold().underline();          "hi" => "\x1B[1;4;32mhi\x1B[0m");
    test!(green_bold_ul_2:       Green.underline().bold();          "hi" => "\x1B[1;4;32mhi\x1B[0m");
    test!(purple_on_white:       Purple.on(White);                  "hi" => "\x1B[47;35mhi\x1B[0m");
    test!(purple_on_white_2:     Purple.normal().on(White);         "hi" => "\x1B[47;35mhi\x1B[0m");
    test!(yellow_on_blue:        Style::new().on(Blue).fg(Yellow);  "hi" => "\x1B[44;33mhi\x1B[0m");
    test!(yellow_on_blue_2:      Cyan.on(Blue).fg(Yellow);          "hi" => "\x1B[44;33mhi\x1B[0m");
    test!(cyan_bold_on_white:    Cyan.bold().on(White);             "hi" => "\x1B[1;47;36mhi\x1B[0m");
    test!(cyan_ul_on_white:      Cyan.underline().on(White);        "hi" => "\x1B[4;47;36mhi\x1B[0m");
    test!(cyan_bold_ul_on_white: Cyan.bold().underline().on(White); "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(cyan_ul_bold_on_white: Cyan.underline().bold().on(White); "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(fixed:                 Fixed(100);                        "hi" => "\x1B[38;5;100mhi\x1B[0m");
    test!(fixed_on_purple:       Fixed(100).on(Purple);             "hi" => "\x1B[45;38;5;100mhi\x1B[0m");
    test!(fixed_on_fixed:        Fixed(100).on(Fixed(200));         "hi" => "\x1B[48;5;200;38;5;100mhi\x1B[0m");
    test!(rgb:                   RGB(70,130,180);                   "hi" => "\x1B[38;2;70;130;180mhi\x1B[0m");
    test!(rgb_on_blue:           RGB(70,130,180).on(Blue);          "hi" => "\x1B[44;38;2;70;130;180mhi\x1B[0m");
    test!(blue_on_rgb:           Blue.on(RGB(70,130,180));          "hi" => "\x1B[48;2;70;130;180;34mhi\x1B[0m");
    test!(rgb_on_rgb:            RGB(70,130,180).on(RGB(5,10,15));  "hi" => "\x1B[48;2;5;10;15;38;2;70;130;180mhi\x1B[0m");
    test!(bold:                  Style::new().bold();               "hi" => "\x1B[1mhi\x1B[0m");
    test!(underline:             Style::new().underline();          "hi" => "\x1B[4mhi\x1B[0m");
    test!(bunderline:            Style::new().bold().underline();   "hi" => "\x1B[1;4mhi\x1B[0m");
    test!(dimmed:                Style::new().dimmed();             "hi" => "\x1B[2mhi\x1B[0m");
    test!(italic:                Style::new().italic();             "hi" => "\x1B[3mhi\x1B[0m");
    test!(blink:                 Style::new().blink();              "hi" => "\x1B[5mhi\x1B[0m");
    test!(reverse:               Style::new().reverse();            "hi" => "\x1B[7mhi\x1B[0m");
    test!(hidden:                Style::new().hidden();             "hi" => "\x1B[8mhi\x1B[0m");
    test!(stricken:              Style::new().strikethrough();      "hi" => "\x1B[9mhi\x1B[0m");

    #[test]
    fn test_infix() {
        assert_eq!(Style::new().dimmed().infix(Style::new()).to_string(), "\x1B[0m");
        assert_eq!(White.dimmed().infix(White.normal()).to_string(), "\x1B[0m\x1B[37m");
        assert_eq!(White.normal().infix(White.bold()).to_string(), "\x1B[1m");
        assert_eq!(White.normal().infix(Blue.normal()).to_string(), "\x1B[34m");
        assert_eq!(Blue.bold().infix(Blue.bold()).to_string(), "");
    }
}
