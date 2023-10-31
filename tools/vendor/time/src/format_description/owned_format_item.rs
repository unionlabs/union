//! A format item with owned data.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::error;
use crate::format_description::{Component, FormatItem};

/// A complete description of how to format and parse a type.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq)]
pub enum OwnedFormatItem {
    /// Bytes that are formatted as-is.
    ///
    /// **Note**: If you call the `format` method that returns a `String`, these bytes will be
    /// passed through `String::from_utf8_lossy`.
    Literal(Box<[u8]>),
    /// A minimal representation of a single non-literal item.
    Component(Component),
    /// A series of literals or components that collectively form a partial or complete
    /// description.
    Compound(Box<[Self]>),
    /// A `FormatItem` that may or may not be present when parsing. If parsing fails, there
    /// will be no effect on the resulting `struct`.
    ///
    /// This variant has no effect on formatting, as the value is guaranteed to be present.
    Optional(Box<Self>),
    /// A series of `FormatItem`s where, when parsing, the first successful parse is used. When
    /// formatting, the first element of the [`Vec`] is used. An empty [`Vec`] is a no-op when
    /// formatting or parsing.
    First(Box<[Self]>),
}

impl fmt::Debug for OwnedFormatItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(literal) => f.write_str(&String::from_utf8_lossy(literal)),
            Self::Component(component) => component.fmt(f),
            Self::Compound(compound) => compound.fmt(f),
            Self::Optional(item) => f.debug_tuple("Optional").field(item).finish(),
            Self::First(items) => f.debug_tuple("First").field(items).finish(),
        }
    }
}

// region: conversions from FormatItem
impl From<FormatItem<'_>> for OwnedFormatItem {
    fn from(item: FormatItem<'_>) -> Self {
        (&item).into()
    }
}

impl From<&FormatItem<'_>> for OwnedFormatItem {
    fn from(item: &FormatItem<'_>) -> Self {
        match item {
            FormatItem::Literal(literal) => Self::Literal(literal.to_vec().into_boxed_slice()),
            FormatItem::Component(component) => Self::Component(*component),
            FormatItem::Compound(compound) => Self::Compound(
                compound
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            FormatItem::Optional(item) => Self::Optional(Box::new((*item).into())),
            FormatItem::First(items) => Self::First(
                items
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        }
    }
}

impl From<Vec<FormatItem<'_>>> for OwnedFormatItem {
    fn from(items: Vec<FormatItem<'_>>) -> Self {
        items.as_slice().into()
    }
}

impl<'a, T: AsRef<[FormatItem<'a>]> + ?Sized> From<&T> for OwnedFormatItem {
    fn from(items: &T) -> Self {
        Self::Compound(
            items
                .as_ref()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}
// endregion conversions from FormatItem

// region: from variants
impl From<Component> for OwnedFormatItem {
    fn from(component: Component) -> Self {
        Self::Component(component)
    }
}

impl TryFrom<OwnedFormatItem> for Component {
    type Error = error::DifferentVariant;

    fn try_from(value: OwnedFormatItem) -> Result<Self, Self::Error> {
        match value {
            OwnedFormatItem::Component(component) => Ok(component),
            _ => Err(error::DifferentVariant),
        }
    }
}

impl From<Vec<Self>> for OwnedFormatItem {
    fn from(items: Vec<Self>) -> Self {
        Self::Compound(items.into_boxed_slice())
    }
}

impl TryFrom<OwnedFormatItem> for Vec<OwnedFormatItem> {
    type Error = error::DifferentVariant;

    fn try_from(value: OwnedFormatItem) -> Result<Self, Self::Error> {
        match value {
            OwnedFormatItem::Compound(items) => Ok(items.into_vec()),
            _ => Err(error::DifferentVariant),
        }
    }
}
// endregion from variants

// region: equality
impl PartialEq<Component> for OwnedFormatItem {
    fn eq(&self, rhs: &Component) -> bool {
        matches!(self, Self::Component(component) if component == rhs)
    }
}

impl PartialEq<OwnedFormatItem> for Component {
    fn eq(&self, rhs: &OwnedFormatItem) -> bool {
        rhs == self
    }
}

impl PartialEq<&[Self]> for OwnedFormatItem {
    fn eq(&self, rhs: &&[Self]) -> bool {
        matches!(self, Self::Compound(compound) if &&**compound == rhs)
    }
}

impl PartialEq<OwnedFormatItem> for &[OwnedFormatItem] {
    fn eq(&self, rhs: &OwnedFormatItem) -> bool {
        rhs == self
    }
}
// endregion equality
