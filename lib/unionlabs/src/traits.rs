use core::fmt::Debug;

#[cfg(feature = "serde")]
pub trait Serde = ::serde::Serialize + for<'de> ::serde::Deserialize<'de>;
#[cfg(not(feature = "serde"))]
pub trait Serde =;

/// Trait alias for traits commonly used together throughout this crate.
pub trait Member = Debug + Clone + PartialEq + Serde + Send + Sync + Unpin + 'static;
