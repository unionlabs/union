use crate::{
    fill::Slot,
    std::{any::Any, error},
    ValueBag,
};

use super::Internal;

impl<'v> ValueBag<'v> {
    /// Get a value from an error.
    pub fn capture_error<T>(value: &'v T) -> Self
    where
        T: error::Error + 'static,
    {
        ValueBag {
            inner: Internal::Error(value),
        }
    }

    /// Get a value from an erased value.
    #[inline]
    pub fn from_dyn_error(value: &'v (dyn error::Error + 'static)) -> Self {
        ValueBag {
            inner: Internal::AnonError(value),
        }
    }

    /// Try get an error from this value.
    #[inline]
    pub fn to_borrowed_error(&self) -> Option<&(dyn Error + 'static)> {
        match self.inner {
            Internal::Error(value) => Some(value.as_super()),
            Internal::AnonError(value) => Some(value),
            _ => None,
        }
    }
}

#[cfg(feature = "error")]
pub(crate) trait DowncastError {
    fn as_any(&self) -> &dyn Any;
    fn as_super(&self) -> &(dyn error::Error + 'static);
}

#[cfg(feature = "error")]
impl<T: error::Error + 'static> DowncastError for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_super(&self) -> &(dyn error::Error + 'static) {
        self
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with an error.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_error<T>(self, value: T) -> Result<(), crate::Error>
    where
        T: error::Error + 'static,
    {
        self.fill(|visitor| visitor.error(&value))
    }

    /// Fill the slot with an error.
    pub fn fill_dyn_error(self, value: &(dyn error::Error + 'static)) -> Result<(), crate::Error> {
        self.fill(|visitor| visitor.error(value))
    }
}

pub use self::error::Error;

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;

    use crate::{
        std::{io, string::ToString},
        test::*,
    };

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_capture() {
        let err = io::Error::from(io::ErrorKind::Other);

        assert_eq!(
            err.to_string(),
            ValueBag::capture_error(&err)
                .to_borrowed_error()
                .expect("invalid value")
                .to_string()
        );

        assert_eq!(
            err.to_string(),
            ValueBag::from_dyn_error(&err)
                .to_borrowed_error()
                .expect("invalid value")
                .to_string()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_downcast() {
        let err = io::Error::from(io::ErrorKind::Other);

        assert!(ValueBag::capture_error(&err)
            .downcast_ref::<io::Error>()
            .is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_visit() {
        let err = io::Error::from(io::ErrorKind::Other);

        ValueBag::from_dyn_error(&err)
            .visit(TestVisit)
            .expect("failed to visit value");
    }
}
