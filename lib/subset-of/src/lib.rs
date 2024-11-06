pub use subset_of_derive::SubsetOf;

/// Defines a "subset" relationship between two types. This is similar to `TryFrom<T, Error = T> + Into<T>` except that there is no blanket `impl<T> SubsetOf<T> for T`, which allows for much more useful generic implementations outside of this crate.
pub trait SubsetOf<T>: Sized {
    fn try_from_super(t: T) -> Result<Self, T>;

    fn into_super(self) -> T;
}

pub trait Superset: Sized {
    fn try_into_sub<T: SubsetOf<Self>>(self) -> Result<T, Self> {
        T::try_from_super(self)
    }

    fn from_sub<T: SubsetOf<Self>>(t: T) -> Self {
        t.into_super()
    }
}

impl<T> Superset for T {}
