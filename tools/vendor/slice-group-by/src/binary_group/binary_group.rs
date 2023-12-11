use crate::{BinaryGroupBy, BinaryGroupByMut};

/// An iterator that will return non-overlapping groups of equal elements, according to
/// the [`PartialEq::eq`] function in the slice using *binary search*.
///
/// It will not necessarily gives contiguous elements to the predicate function.
/// The predicate function should implement an order consistent with the sort order of the slice.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
pub struct BinaryGroup<'a, T: 'a>(BinaryGroupBy<'a, T, fn(&T, &T) -> bool>);

impl<'a, T: 'a> BinaryGroup<'a, T>
where T: PartialEq,
{
    pub fn new(slice: &'a [T]) -> BinaryGroup<'a, T> {
        BinaryGroup(BinaryGroupBy::new(slice, PartialEq::eq))
    }
}

group_by_wrapped!{ struct BinaryGroup, &'a [T] }

/// An iterator that will return non-overlapping *mutable* groups of equal elements, according to
/// the [`PartialEq::eq`] function in the slice using *binary search*.
///
/// It will not necessarily gives contiguous elements to the predicate function.
/// The predicate function should implement an order consistent with the sort order of the slice.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
pub struct BinaryGroupMut<'a, T: 'a>(BinaryGroupByMut<'a, T, fn(&T, &T) -> bool>);

impl<'a, T: 'a> BinaryGroupMut<'a, T>
where T: PartialEq,
{
    pub fn new(slice: &'a mut [T]) -> BinaryGroupMut<'a, T> {
        BinaryGroupMut(BinaryGroupByMut::new(slice, PartialEq::eq))
    }
}

group_by_wrapped!{ struct BinaryGroupMut, &'a mut [T] }
