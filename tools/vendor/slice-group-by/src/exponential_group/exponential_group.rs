use crate::{ExponentialGroupBy, ExponentialGroupByMut};

/// An iterator that will return non-overlapping groups of equal elements, according to
/// the [`PartialEq::eq`] function in the slice using *exponential search*.
///
/// It will not necessarily gives contiguous elements to the predicate function.
/// The predicate function should implement an order consistent with the sort order of the slice.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
pub struct ExponentialGroup<'a, T: 'a>(ExponentialGroupBy<'a, T, fn(&T, &T) -> bool>);

impl<'a, T: 'a> ExponentialGroup<'a, T>
where T: PartialEq,
{
    pub fn new(slice: &'a [T]) -> ExponentialGroup<'a, T> {
        ExponentialGroup(ExponentialGroupBy::new(slice, PartialEq::eq))
    }
}

group_by_wrapped!{ struct ExponentialGroup, &'a [T] }

/// An iterator that will return non-overlapping *mutable* groups of equal elements, according to
/// the [`PartialEq::eq`] function in the slice using *exponential search*.
///
/// It will not necessarily gives contiguous elements to the predicate function.
/// The predicate function should implement an order consistent with the sort order of the slice.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
pub struct ExponentialGroupMut<'a, T: 'a>(ExponentialGroupByMut<'a, T, fn(&T, &T) -> bool>);

impl<'a, T: 'a> ExponentialGroupMut<'a, T>
where T: PartialEq,
{
    pub fn new(slice: &'a mut [T]) -> ExponentialGroupMut<'a, T> {
        ExponentialGroupMut(ExponentialGroupByMut::new(slice, PartialEq::eq))
    }
}

group_by_wrapped!{ struct ExponentialGroupMut, &'a mut [T] }
