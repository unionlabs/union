pub use subset_of_derive::SubsetOf;

/// Defines a "subset" relationship between two types. This is similar to `TryFrom<T, Error = T> + Into<T>` except that there is no blanket `impl<T> SubsetOf<T> for T`, which allows for much more useful generic implementations outside of this crate.
pub trait SubsetOf<T>: Sized {
    fn try_from_super(t: T) -> Result<Self, T>;

    fn into_super(self) -> T;
}
