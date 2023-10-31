use super::Internal;
use crate::{fill::Fill, ValueBag};

impl<'v> ValueBag<'v> {
    /// Get a value from a fillable slot.
    pub fn from_fill<T>(value: &'v T) -> Self
    where
        T: Fill,
    {
        ValueBag {
            inner: Internal::Fill(value),
        }
    }
}
