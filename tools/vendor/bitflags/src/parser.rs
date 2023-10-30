//! Parsing flags from text.
//!
//! `bitflags` defines the following *whitespace-insensitive*, *case-sensitive* grammar for flags formatted
//! as text:
//!
//! - _Flags:_ (_Flag_)`|`*
//! - _Flag:_ _Identifier_ | _HexNumber_
//! - _Identifier:_ Any Rust identifier
//! - _HexNumber_: `0x`([0-9a-fA-F])*
//!
//! As an example, this is how `Flags::A | Flags::B | 0x0c` can be represented as text:
//!
//! ```text
//! A | B | 0x0c
//! ```
//!
//! Alternatively, it could be represented without whitespace:
//!
//! ```text
//! A|B|0x0C
//! ```
//!
//! Note that identifiers are *case-sensitive*, so the following is *not equivalent*:
//!
//! ```text
//! a | b | 0x0c
//! ```

#![allow(clippy::let_unit_value)]

use core::fmt::{self, Write};

use crate::{Bits, Flags};

/// Write a set of flags to a writer.
///
/// Any bits that don't correspond to a valid flag will be formatted
/// as a hex number.
pub fn to_writer<B: Flags>(flags: &B, mut writer: impl Write) -> Result<(), fmt::Error>
where
    B::Bits: WriteHex,
{
    // A formatter for bitflags that produces text output like:
    //
    // A | B | 0xf6
    //
    // The names of set flags are written in a bar-separated-format,
    // followed by a hex number of any remaining bits that are set
    // but don't correspond to any flags.

    // Iterate over known flag values
    let mut first = true;
    let mut iter = flags.iter_names();
    for (name, _) in &mut iter {
        if !first {
            writer.write_str(" | ")?;
        }

        first = false;
        writer.write_str(name)?;
    }

    // Append any extra bits that correspond to flags to the end of the format
    let remaining = iter.remaining().bits();
    if remaining != B::Bits::EMPTY {
        if !first {
            writer.write_str(" | ")?;
        }

        writer.write_str("0x")?;
        remaining.write_hex(writer)?;
    }

    fmt::Result::Ok(())
}

pub(crate) struct AsDisplay<'a, B>(pub(crate) &'a B);

impl<'a, B: Flags> fmt::Display for AsDisplay<'a, B>
where
    B::Bits: WriteHex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        to_writer(self.0, f)
    }
}

/// Parse a set of flags from text.
///
/// This function will fail on unknown flags rather than ignore them.
pub fn from_str<B: Flags>(input: &str) -> Result<B, ParseError>
where
    B::Bits: ParseHex,
{
    let mut parsed_flags = B::empty();

    // If the input is empty then return an empty set of flags
    if input.trim().is_empty() {
        return Ok(parsed_flags);
    }

    for flag in input.split('|') {
        let flag = flag.trim();

        // If the flag is empty then we've got missing input
        if flag.is_empty() {
            return Err(ParseError::empty_flag());
        }

        // If the flag starts with `0x` then it's a hex number
        // Parse it directly to the underlying bits type
        let parsed_flag = if let Some(flag) = flag.strip_prefix("0x") {
            let bits =
                <B::Bits>::parse_hex(flag).map_err(|_| ParseError::invalid_hex_flag(flag))?;

            B::from_bits_retain(bits)
        }
        // Otherwise the flag is a name
        // The generated flags type will determine whether
        // or not it's a valid identifier
        else {
            B::from_name(flag).ok_or_else(|| ParseError::invalid_named_flag(flag))?
        };

        parsed_flags.insert(parsed_flag);
    }

    Ok(parsed_flags)
}

/// Encode a value as a hex number.
pub trait WriteHex {
    /// Write the value as hex.
    fn write_hex<W: fmt::Write>(&self, writer: W) -> fmt::Result;
}

/// Parse a value from a number encoded as a hex string.
pub trait ParseHex {
    /// Parse the value from hex.
    fn parse_hex(input: &str) -> Result<Self, ParseError>
    where
        Self: Sized;
}

/// An error encountered while parsing flags from text.
#[derive(Debug)]
pub struct ParseError(ParseErrorKind);

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum ParseErrorKind {
    EmptyFlag,
    InvalidNamedFlag {
        #[cfg(not(feature = "std"))]
        got: (),
        #[cfg(feature = "std")]
        got: String,
    },
    InvalidHexFlag {
        #[cfg(not(feature = "std"))]
        got: (),
        #[cfg(feature = "std")]
        got: String,
    },
}

impl ParseError {
    /// An invalid hex flag was encountered.
    pub fn invalid_hex_flag(flag: impl fmt::Display) -> Self {
        let _flag = flag;

        let got = {
            #[cfg(feature = "std")]
            {
                _flag.to_string()
            }
        };

        ParseError(ParseErrorKind::InvalidHexFlag { got })
    }

    /// A named flag that doesn't correspond to any on the flags type was encountered.
    pub fn invalid_named_flag(flag: impl fmt::Display) -> Self {
        let _flag = flag;

        let got = {
            #[cfg(feature = "std")]
            {
                _flag.to_string()
            }
        };

        ParseError(ParseErrorKind::InvalidNamedFlag { got })
    }

    /// A hex or named flag wasn't found between separators.
    pub const fn empty_flag() -> Self {
        ParseError(ParseErrorKind::EmptyFlag)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ParseErrorKind::InvalidNamedFlag { got } => {
                let _got = got;

                write!(f, "unrecognized named flag")?;

                #[cfg(feature = "std")]
                {
                    write!(f, " `{}`", _got)?;
                }
            }
            ParseErrorKind::InvalidHexFlag { got } => {
                let _got = got;

                write!(f, "invalid hex flag")?;

                #[cfg(feature = "std")]
                {
                    write!(f, " `{}`", _got)?;
                }
            }
            ParseErrorKind::EmptyFlag => {
                write!(f, "encountered empty flag")?;
            }
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
