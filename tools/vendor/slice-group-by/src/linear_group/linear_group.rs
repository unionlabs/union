use crate::{LinearGroupBy, LinearGroupByMut};

/// An iterator that will return non-overlapping groups of equal elements
/// in the slice using *linear/sequential search*.
///
/// It will give two contiguous elements to the [`PartialEq::eq`] function
/// therefore the slice must not be necessarily sorted.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
pub struct LinearGroup<'a, T: 'a>(LinearGroupBy<'a, T, fn(&T, &T) -> bool>);

impl<'a, T: 'a> LinearGroup<'a, T>
where T: PartialEq,
{
    pub fn new(slice: &'a [T]) -> LinearGroup<'a, T> {
        LinearGroup(LinearGroupBy::new(slice, PartialEq::eq))
    }
}

group_by_wrapped!{ struct LinearGroup, &'a [T] }

/// An iterator that will return non-overlapping *mutable* groups of equal elements
/// in the slice using *linear/sequential search*.
///
/// It will give two contiguous elements to the [`PartialEq::eq`] function
/// therefore the slice must not be necessarily sorted.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
pub struct LinearGroupMut<'a, T: 'a>(LinearGroupByMut<'a, T, fn(&T, &T) -> bool>);

impl<'a, T: 'a> LinearGroupMut<'a, T>
where T: PartialEq,
{
    pub fn new(slice: &'a mut [T]) -> LinearGroupMut<'a, T> {
        LinearGroupMut(LinearGroupByMut::new(slice, PartialEq::eq))
    }
}

group_by_wrapped!{ struct LinearGroupMut, &'a mut [T] }
