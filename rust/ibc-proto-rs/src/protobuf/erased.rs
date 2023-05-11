use core::convert::{Into as CoreInto, TryFrom as CoreTryFrom};

pub trait CloneInto<T> {
    fn clone_into(&self) -> T;
}

impl<T, U: Clone + CoreInto<T>> CloneInto<T> for U {
    fn clone_into(&self) -> T {
        self.clone().into()
    }
}

pub trait TryFrom<T> {
    type Error;

    fn try_from(t: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<T, U: CoreTryFrom<T>> TryFrom<T> for U {
    type Error = <Self as CoreTryFrom<T>>::Error;

    fn try_from(t: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        <Self as CoreTryFrom<T>>::try_from(t)
    }
}
