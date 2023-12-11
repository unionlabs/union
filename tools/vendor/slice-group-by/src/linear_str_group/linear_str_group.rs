use crate::{LinearStrGroupBy, LinearStrGroupByMut};

/// An iterator that will return non-overlapping groups of equal `char`
/// in the `str` using *linear/sequential search*.
///
/// It will use the `char` [`PartialEq::eq`] function.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/primitive.char.html#impl-PartialEq%3Cchar%3E
pub struct LinearStrGroup<'a>(LinearStrGroupBy<'a, fn(char, char) -> bool>);

impl<'a> LinearStrGroup<'a> {
    pub fn new(string: &'a str) -> Self {
        LinearStrGroup(LinearStrGroupBy::new(string, |a, b| a == b))
    }
}

str_group_by_wrapped!{ struct LinearStrGroup, &'a str }

/// An iterator that will return non-overlapping *mutable* groups of equal `char`
/// in the `str` using *linear/sequential search*.
///
/// It will use the `char` [`PartialEq::eq`] function.
///
/// [`PartialEq::eq`]: https://doc.rust-lang.org/std/primitive.char.html#impl-PartialEq%3Cchar%3E
pub struct LinearStrGroupMut<'a>(LinearStrGroupByMut<'a, fn(char, char) -> bool>);

impl<'a> LinearStrGroupMut<'a> {
    pub fn new(string: &'a mut str) -> LinearStrGroupMut {
        LinearStrGroupMut(LinearStrGroupByMut::new(string, |a, b| a == b))
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        self.0.as_str_mut()
    }
}

str_group_by_wrapped!{ struct LinearStrGroupMut, &'a mut str }
