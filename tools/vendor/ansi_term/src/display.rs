use std::borrow::Cow;
use std::fmt;
use std::io;
use std::ops::Deref;

use ansi::RESET;
use difference::Difference;
use style::{Style, Colour};
use write::AnyWrite;


/// An `ANSIGenericString` includes a generic string type and a `Style` to
/// display that string.  `ANSIString` and `ANSIByteString` are aliases for
/// this type on `str` and `\[u8]`, respectively.
#[derive(PartialEq, Debug)]
pub struct ANSIGenericString<'a, S: 'a + ToOwned + ?Sized>
where <S as ToOwned>::Owned: fmt::Debug {
    style: Style,
    string: Cow<'a, S>,
}


/// Cloning an `ANSIGenericString` will clone its underlying string.
///
/// # Examples
///
/// ```
/// use ansi_term::ANSIString;
///
/// let plain_string = ANSIString::from("a plain string");
/// let clone_string = plain_string.clone();
/// assert_eq!(clone_string, plain_string);
/// ```
impl<'a, S: 'a + ToOwned + ?Sized> Clone for ANSIGenericString<'a, S>
where <S as ToOwned>::Owned: fmt::Debug {
    fn clone(&self) -> ANSIGenericString<'a, S> {
        ANSIGenericString {
            style: self.style,
            string: self.string.clone(),
        }
    }
}

// You might think that the hand-written Clone impl above is the same as the
// one that gets generated with #[derive]. But it’s not *quite* the same!
//
// `str` is not Clone, and the derived Clone implementation puts a Clone
// constraint on the S type parameter (generated using --pretty=expanded):
//
//                  ↓_________________↓
//     impl <'a, S: ::std::clone::Clone + 'a + ToOwned + ?Sized> ::std::clone::Clone
//     for ANSIGenericString<'a, S> where
//     <S as ToOwned>::Owned: fmt::Debug { ... }
//
// This resulted in compile errors when you tried to derive Clone on a type
// that used it:
//
//     #[derive(PartialEq, Debug, Clone, Default)]
//     pub struct TextCellContents(Vec<ANSIString<'static>>);
//                                 ^^^^^^^^^^^^^^^^^^^^^^^^^
//     error[E0277]: the trait `std::clone::Clone` is not implemented for `str`
//
// The hand-written impl above can ignore that constraint and still compile.



/// An ANSI String is a string coupled with the `Style` to display it
/// in a terminal.
///
/// Although not technically a string itself, it can be turned into
/// one with the `to_string` method.
///
/// # Examples
///
/// ```
/// use ansi_term::ANSIString;
/// use ansi_term::Colour::Red;
///
/// let red_string = Red.paint("a red string");
/// println!("{}", red_string);
/// ```
///
/// ```
/// use ansi_term::ANSIString;
///
/// let plain_string = ANSIString::from("a plain string");
/// assert_eq!(&*plain_string, "a plain string");
/// ```
pub type ANSIString<'a> = ANSIGenericString<'a, str>;

/// An `ANSIByteString` represents a formatted series of bytes.  Use
/// `ANSIByteString` when styling text with an unknown encoding.
pub type ANSIByteString<'a> = ANSIGenericString<'a, [u8]>;

impl<'a, I, S: 'a + ToOwned + ?Sized> From<I> for ANSIGenericString<'a, S>
where I: Into<Cow<'a, S>>,
      <S as ToOwned>::Owned: fmt::Debug {
    fn from(input: I) -> ANSIGenericString<'a, S> {
        ANSIGenericString {
            string: input.into(),
            style:  Style::default(),
        }
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> ANSIGenericString<'a, S>
    where <S as ToOwned>::Owned: fmt::Debug {

    /// Directly access the style
    pub fn style_ref(&self) -> &Style {
        &self.style
    }

    /// Directly access the style mutably
    pub fn style_ref_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> Deref for ANSIGenericString<'a, S>
where <S as ToOwned>::Owned: fmt::Debug {
    type Target = S;

    fn deref(&self) -> &S {
        self.string.deref()
    }
}


/// A set of `ANSIGenericString`s collected together, in order to be
/// written with a minimum of control characters.
#[derive(Debug, PartialEq)]
pub struct ANSIGenericStrings<'a, S: 'a + ToOwned + ?Sized>
    (pub &'a [ANSIGenericString<'a, S>])
    where <S as ToOwned>::Owned: fmt::Debug, S: PartialEq;

/// A set of `ANSIString`s collected together, in order to be written with a
/// minimum of control characters.
pub type ANSIStrings<'a> = ANSIGenericStrings<'a, str>;

/// A function to construct an `ANSIStrings` instance.
#[allow(non_snake_case)]
pub fn ANSIStrings<'a>(arg: &'a [ANSIString<'a>]) -> ANSIStrings<'a> {
    ANSIGenericStrings(arg)
}

/// A set of `ANSIByteString`s collected together, in order to be
/// written with a minimum of control characters.
pub type ANSIByteStrings<'a> = ANSIGenericStrings<'a, [u8]>;

/// A function to construct an `ANSIByteStrings` instance.
#[allow(non_snake_case)]
pub fn ANSIByteStrings<'a>(arg: &'a [ANSIByteString<'a>]) -> ANSIByteStrings<'a> {
    ANSIGenericStrings(arg)
}


// ---- paint functions ----

impl Style {

    /// Paints the given text with this colour, returning an ANSI string.
    #[must_use]
    pub fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> ANSIGenericString<'a, S>
    where I: Into<Cow<'a, S>>,
          <S as ToOwned>::Owned: fmt::Debug {
        ANSIGenericString {
            string: input.into(),
            style:  self,
        }
    }
}


impl Colour {

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don’t have to use `Blue.normal()` just
    /// to get blue text.
    ///
    /// ```
    /// use ansi_term::Colour::Blue;
    /// println!("{}", Blue.paint("da ba dee"));
    /// ```
    #[must_use]
    pub fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> ANSIGenericString<'a, S>
    where I: Into<Cow<'a, S>>,
          <S as ToOwned>::Owned: fmt::Debug {
        ANSIGenericString {
            string: input.into(),
            style:  self.normal(),
        }
    }
}


// ---- writers for individual ANSI strings ----

impl<'a> fmt::Display for ANSIString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: &mut fmt::Write = f;
        self.write_to_any(w)
    }
}

impl<'a> ANSIByteString<'a> {
    /// Write an `ANSIByteString` to an `io::Write`.  This writes the escape
    /// sequences for the associated `Style` around the bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let w: &mut io::Write = w;
        self.write_to_any(w)
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> ANSIGenericString<'a, S>
where <S as ToOwned>::Owned: fmt::Debug, &'a S: AsRef<[u8]> {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        write!(w, "{}", self.style.prefix())?;
        w.write_str(self.string.as_ref())?;
        write!(w, "{}", self.style.suffix())
    }
}


// ---- writers for combined ANSI strings ----

impl<'a> fmt::Display for ANSIStrings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        self.write_to_any(f)
    }
}

impl<'a> ANSIByteStrings<'a> {
    /// Write `ANSIByteStrings` to an `io::Write`.  This writes the minimal
    /// escape sequences for the associated `Style`s around each set of
    /// bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let w: &mut io::Write = w;
        self.write_to_any(w)
    }
}

impl<'a, S: 'a + ToOwned + ?Sized + PartialEq> ANSIGenericStrings<'a, S>
where <S as ToOwned>::Owned: fmt::Debug, &'a S: AsRef<[u8]> {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        use self::Difference::*;

        let first = match self.0.first() {
            None => return Ok(()),
            Some(f) => f,
        };

        write!(w, "{}", first.style.prefix())?;
        w.write_str(first.string.as_ref())?;

        for window in self.0.windows(2) {
            match Difference::between(&window[0].style, &window[1].style) {
                ExtraStyles(style) => write!(w, "{}", style.prefix())?,
                Reset              => write!(w, "{}{}", RESET, window[1].style.prefix())?,
                NoDifference       => {/* Do nothing! */},
            }

            w.write_str(&window[1].string)?;
        }

        // Write the final reset string after all of the ANSIStrings have been
        // written, *except* if the last one has no styles, because it would
        // have already been written by this point.
        if let Some(last) = self.0.last() {
            if !last.style.is_plain() {
                write!(w, "{}", RESET)?;
            }
        }

        Ok(())
    }
}


// ---- tests ----

#[cfg(test)]
mod tests {
    pub use super::super::ANSIStrings;
    pub use style::Style;
    pub use style::Colour::*;

    #[test]
    fn no_control_codes_for_plain() {
        let one = Style::default().paint("one");
        let two = Style::default().paint("two");
        let output = format!("{}", ANSIStrings( &[ one, two ] ));
        assert_eq!(&*output, "onetwo");
    }
}
