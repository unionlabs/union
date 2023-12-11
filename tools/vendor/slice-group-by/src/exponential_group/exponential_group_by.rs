use crate::{exponential_search_by, offset_from};
use std::cmp::Ordering::{Greater, Less};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::{fmt, marker};

macro_rules! exponential_group_by {
    (struct $name:ident, $elem:ty, $mkslice:ident) => {
        impl<'a, T: 'a, P> $name<'a, T, P> {
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.ptr == self.end
            }

            #[inline]
            pub fn remainder_len(&self) -> usize {
                unsafe { offset_from(self.end, self.ptr) }
            }
        }

        impl<'a, T: 'a, P> std::iter::Iterator for $name<'a, T, P>
        where
            P: FnMut(&T, &T) -> bool,
        {
            type Item = $elem;

            fn next(&mut self) -> Option<Self::Item> {
                if self.is_empty() {
                    return None;
                }

                let first = unsafe { &*self.ptr };

                let len = self.remainder_len();
                let tail = unsafe { $mkslice(self.ptr.add(1), len - 1) };

                let predicate = |x: &T| {
                    if (self.predicate)(first, x) {
                        Less
                    } else {
                        Greater
                    }
                };
                let index = exponential_search_by(tail, predicate).unwrap_err();

                let left = unsafe { $mkslice(self.ptr, index + 1) };
                self.ptr = unsafe { self.ptr.add(index + 1) };

                Some(left)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                if self.is_empty() {
                    return (0, Some(0));
                }

                let len = self.remainder_len();
                (1, Some(len))
            }

            fn last(mut self) -> Option<Self::Item> {
                self.next_back()
            }
        }

        impl<'a, T: 'a, P> std::iter::DoubleEndedIterator for $name<'a, T, P>
        where
            P: FnMut(&T, &T) -> bool,
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.is_empty() {
                    return None;
                }

                let last = unsafe { &*self.end.sub(1) };

                let len = self.remainder_len();
                let head = unsafe { $mkslice(self.ptr, len - 1) };

                let predicate = |x: &T| {
                    if (self.predicate)(last, x) {
                        Greater
                    } else {
                        Less
                    }
                };
                let index = exponential_search_by(head, predicate).unwrap_err();

                let right = unsafe { $mkslice(self.ptr.add(index), len - index) };
                self.end = unsafe { self.end.sub(len - index) };

                Some(right)
            }
        }

        impl<'a, T: 'a, P> std::iter::FusedIterator for $name<'a, T, P> where
            P: FnMut(&T, &T) -> bool
        {
        }
    };
}

/// An iterator that will reutrn non-overlapping groups in the slice using *exponential search*.
///
/// It will not necessarily gives contiguous elements to the predicate function.
/// The predicate function should implement an order consistent with the sort order of the slice.
pub struct ExponentialGroupBy<'a, T, P> {
    ptr: *const T,
    end: *const T,
    predicate: P,
    _phantom: marker::PhantomData<&'a T>,
}

impl<'a, T: 'a, P> ExponentialGroupBy<'a, T, P>
where
    P: FnMut(&T, &T) -> bool,
{
    pub fn new(slice: &'a [T], predicate: P) -> Self {
        ExponentialGroupBy {
            ptr: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            predicate,
            _phantom: marker::PhantomData,
        }
    }
}

impl<'a, T: 'a, P> ExponentialGroupBy<'a, T, P> {
    /// Returns the remainder of the original slice that is going to be
    /// returned by the iterator.
    pub fn remainder(&self) -> &[T] {
        let len = self.remainder_len();
        unsafe { from_raw_parts(self.ptr, len) }
    }
}

impl<'a, T: 'a + fmt::Debug, P> fmt::Debug for ExponentialGroupBy<'a, T, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExponentialGroupBy")
            .field("remainder", &self.remainder())
            .finish()
    }
}

exponential_group_by! { struct ExponentialGroupBy, &'a [T], from_raw_parts }

/// An iterator that will reutrn non-overlapping *mutable* groups
/// in the slice using *exponential search*.
///
/// It will not necessarily gives contiguous elements to the predicate function.
/// The predicate function should implement an order consistent with the sort order of the slice.
pub struct ExponentialGroupByMut<'a, T, P> {
    ptr: *mut T,
    end: *mut T,
    predicate: P,
    _phantom: marker::PhantomData<&'a mut T>,
}

impl<'a, T: 'a, P> ExponentialGroupByMut<'a, T, P>
where
    P: FnMut(&T, &T) -> bool,
{
    pub fn new(slice: &'a mut [T], predicate: P) -> Self {
        let ptr = slice.as_mut_ptr();
        let end = unsafe { ptr.add(slice.len()) };
        ExponentialGroupByMut {
            ptr,
            end,
            predicate,
            _phantom: marker::PhantomData,
        }
    }
}

impl<'a, T: 'a, P> ExponentialGroupByMut<'a, T, P> {
    /// Returns the remainder of the original slice that is going to be
    /// returned by the iterator.
    pub fn into_remainder(self) -> &'a mut [T] {
        let len = self.remainder_len();
        unsafe { from_raw_parts_mut(self.ptr, len) }
    }
}

impl<'a, T: 'a + fmt::Debug, P> fmt::Debug for ExponentialGroupByMut<'a, T, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.remainder_len();
        let remainder = unsafe { from_raw_parts(self.ptr, len) };

        f.debug_struct("ExponentialGroupByMut")
            .field("remaining", &remainder)
            .finish()
    }
}

exponential_group_by! { struct ExponentialGroupByMut, &'a mut [T], from_raw_parts_mut }
