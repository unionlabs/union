//! Parser for format descriptions.

use alloc::boxed::Box;
use alloc::vec::Vec;

/// A helper macro to make version restrictions simpler to read and write.
macro_rules! version {
    ($range:expr) => {
        $range.contains(&VERSION)
    };
}

/// A helper macro to statically validate the version (when used as a const parameter).
macro_rules! validate_version {
    ($version:ident) => {
        #[allow(clippy::let_unit_value)]
        let _ = $crate::format_description::parse::Version::<$version>::IS_VALID;
    };
}

mod ast;
mod format_item;
mod lexer;

/// A struct that is used to ensure that the version is valid.
struct Version<const N: usize>;
impl<const N: usize> Version<N> {
    /// A constant that panics if the version is not valid. This results in a post-monomorphization
    /// error.
    const IS_VALID: () = assert!(N >= 1 && N <= 2);
}

/// Parse a sequence of items from the format description.
///
/// The syntax for the format description can be found in [the
/// book](https://time-rs.github.io/book/api/format-description.html).
///
/// This function exists for backward compatibility reasons. It is equivalent to calling
/// `parse_borrowed::<1>(s)`. In the future, this function will be deprecated in favor of
/// `parse_borrowed`.
pub fn parse(
    s: &str,
) -> Result<Vec<crate::format_description::FormatItem<'_>>, crate::error::InvalidFormatDescription>
{
    parse_borrowed::<1>(s)
}

/// Parse a sequence of items from the format description.
///
/// The syntax for the format description can be found in [the
/// book](https://time-rs.github.io/book/api/format-description.html). The version of the format
/// description is provided as the const parameter. **It is recommended to use version 2.**
pub fn parse_borrowed<const VERSION: usize>(
    s: &str,
) -> Result<Vec<crate::format_description::FormatItem<'_>>, crate::error::InvalidFormatDescription>
{
    validate_version!(VERSION);
    let mut lexed = lexer::lex::<VERSION>(s.as_bytes());
    let ast = ast::parse::<_, VERSION>(&mut lexed);
    let format_items = format_item::parse(ast);
    Ok(format_items
        .map(|res| res.and_then(TryInto::try_into))
        .collect::<Result<_, _>>()?)
}

/// Parse a sequence of items from the format description.
///
/// The syntax for the format description can be found in [the
/// book](https://time-rs.github.io/book/api/format-description.html). The version of the format
/// description is provided as the const parameter.
///
/// Unlike [`parse`], this function returns [`OwnedFormatItem`], which owns its contents. This means
/// that there is no lifetime that needs to be handled. **It is recommended to use version 2.**
///
/// [`OwnedFormatItem`]: crate::format_description::OwnedFormatItem
pub fn parse_owned<const VERSION: usize>(
    s: &str,
) -> Result<crate::format_description::OwnedFormatItem, crate::error::InvalidFormatDescription> {
    validate_version!(VERSION);
    let mut lexed = lexer::lex::<VERSION>(s.as_bytes());
    let ast = ast::parse::<_, VERSION>(&mut lexed);
    let format_items = format_item::parse(ast);
    let items = format_items
        .map(|res| res.map(Into::into))
        .collect::<Result<Box<_>, _>>()?;
    Ok(items.into())
}

/// A location within a string.
#[derive(Clone, Copy)]
struct Location {
    /// The zero-indexed byte of the string.
    byte: u32,
}

impl Location {
    /// Create a new [`Span`] from `self` to `other`.
    const fn to(self, end: Self) -> Span {
        Span { start: self, end }
    }

    /// Offset the location by the provided amount.
    ///
    /// Note that this assumes the resulting location is on the same line as the original location.
    #[must_use = "this does not modify the original value"]
    const fn offset(&self, offset: u32) -> Self {
        Self {
            byte: self.byte + offset,
        }
    }

    /// Create an error with the provided message at this location.
    const fn error(self, message: &'static str) -> ErrorInner {
        ErrorInner {
            _message: message,
            _span: Span {
                start: self,
                end: self,
            },
        }
    }
}

/// A start and end point within a string.
#[derive(Clone, Copy)]
struct Span {
    #[allow(clippy::missing_docs_in_private_items)]
    start: Location,
    #[allow(clippy::missing_docs_in_private_items)]
    end: Location,
}

impl Span {
    /// Obtain a `Span` pointing at the start of the pre-existing span.
    #[must_use = "this does not modify the original value"]
    const fn shrink_to_start(&self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    /// Obtain a `Span` pointing at the end of the pre-existing span.
    #[must_use = "this does not modify the original value"]
    const fn shrink_to_end(&self) -> Self {
        Self {
            start: self.end,
            end: self.end,
        }
    }

    /// Obtain a `Span` that ends before the provided position of the pre-existing span.
    #[must_use = "this does not modify the original value"]
    const fn shrink_to_before(&self, pos: u32) -> Self {
        Self {
            start: self.start,
            end: Location {
                byte: self.start.byte + pos - 1,
            },
        }
    }

    /// Obtain a `Span` that starts after provided position to the end of the pre-existing span.
    #[must_use = "this does not modify the original value"]
    const fn shrink_to_after(&self, pos: u32) -> Self {
        Self {
            start: Location {
                byte: self.start.byte + pos + 1,
            },
            end: self.end,
        }
    }

    /// Create an error with the provided message at this span.
    const fn error(self, message: &'static str) -> ErrorInner {
        ErrorInner {
            _message: message,
            _span: self,
        }
    }
}

/// A value with an associated [`Span`].
#[derive(Clone, Copy)]
struct Spanned<T> {
    /// The value.
    value: T,
    /// Where the value was in the format string.
    span: Span,
}

impl<T> core::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Helper trait to attach a [`Span`] to a value.
trait SpannedValue: Sized {
    /// Attach a [`Span`] to a value.
    fn spanned(self, span: Span) -> Spanned<Self>;
}

impl<T> SpannedValue for T {
    fn spanned(self, span: Span) -> Spanned<Self> {
        Spanned { value: self, span }
    }
}

/// The internal error type.
struct ErrorInner {
    /// The message displayed to the user.
    _message: &'static str,
    /// Where the error originated.
    _span: Span,
}

/// A complete error description.
struct Error {
    /// The internal error.
    _inner: Unused<ErrorInner>,
    /// The error needed for interoperability with the rest of `time`.
    public: crate::error::InvalidFormatDescription,
}

impl From<Error> for crate::error::InvalidFormatDescription {
    fn from(error: Error) -> Self {
        error.public
    }
}

/// A value that may be used in the future, but currently is not.
///
/// This struct exists so that data can semantically be passed around without _actually_ passing it
/// around. This way the data still exists if it is needed in the future.
// `PhantomData` is not used directly because we don't want to introduce any trait implementations.
struct Unused<T>(core::marker::PhantomData<T>);

/// Indicate that a value is currently unused.
#[allow(clippy::missing_const_for_fn)] // false positive
fn unused<T>(_: T) -> Unused<T> {
    Unused(core::marker::PhantomData)
}
