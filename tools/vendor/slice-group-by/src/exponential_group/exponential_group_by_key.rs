use crate::{exponential_search_by, offset_from};
use std::cmp::Ordering::{Greater, Less};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::{fmt, marker};

macro_rules! exponential_group_by_key {
    (struct $name:ident, $elem:ty, $mkslice:ident) => {
        impl<'a, T: 'a, F> $name<'a, T, F> {
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.ptr == self.end
            }

            #[inline]
            pub fn remainder_len(&self) -> usize {
                unsafe { offset_from(self.end, self.ptr) }
            }
        }

        impl<'a, T: 'a, F, K> std::iter::Iterator for $name<'a, T, F>
        where
            F: FnMut(&T) -> K,
            K: PartialEq,
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
                    if (self.func)(first) == (self.func)(x) {
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

        impl<'a, T: 'a, F, K> std::iter::DoubleEndedIterator for $name<'a, T, F>
        where
            F: FnMut(&T) -> K,
            K: PartialEq,
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.is_empty() {
                    return None;
                }

                let last = unsafe { &*self.end.sub(1) };

                let len = self.remainder_len();
                let head = unsafe { $mkslice(self.ptr, len - 1) };

                let predicate = |x: &T| {
                    if (self.func)(last) == (self.func)(x) {
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

        impl<'a, T: 'a, F, K> std::iter::FusedIterator for $name<'a, T, F>
        where
            F: FnMut(&T) -> K,
            K: PartialEq,
        {
        }
    };
}

/// An iterator that will reutrn non-overlapping groups in the slice using *exponential search*.
///
/// It will give an element to the given function, producing a key and comparing
/// the keys to determine groups.
pub struct ExponentialGroupByKey<'a, T, F> {
    ptr: *const T,
    end: *const T,
    func: F,
    _phantom: marker::PhantomData<&'a T>,
}

impl<'a, T: 'a, F> ExponentialGroupByKey<'a, T, F> {
    pub fn new(slice: &'a [T], func: F) -> Self {
        ExponentialGroupByKey {
            ptr: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            func,
            _phantom: marker::PhantomData,
        }
    }
}

impl<'a, T: 'a, F> ExponentialGroupByKey<'a, T, F> {
    /// Returns the remainder of the original slice that is going to be
    /// returned by the iterator.
    pub fn remainder(&self) -> &[T] {
        let len = self.remainder_len();
        unsafe { from_raw_parts(self.ptr, len) }
    }
}

impl<'a, T: 'a + fmt::Debug, F> fmt::Debug for ExponentialGroupByKey<'a, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExponentialGroupByKey")
            .field("remainder", &self.remainder())
            .finish()
    }
}

exponential_group_by_key! { struct ExponentialGroupByKey, &'a [T], from_raw_parts }

/// An iterator that will reutrn non-overlapping *mutable* groups
/// in the slice using *exponential search*.
///
/// It will give an element to the given function, producing a key and comparing
/// the keys to determine groups.
pub struct ExponentialGroupByKeyMut<'a, T, F> {
    ptr: *mut T,
    end: *mut T,
    func: F,
    _phantom: marker::PhantomData<&'a mut T>,
}

impl<'a, T: 'a, F> ExponentialGroupByKeyMut<'a, T, F> {
    pub fn new(slice: &'a mut [T], func: F) -> Self {
        let ptr = slice.as_mut_ptr();
        let end = unsafe { ptr.add(slice.len()) };
        ExponentialGroupByKeyMut {
            ptr,
            end,
            func,
            _phantom: marker::PhantomData,
        }
    }
}

impl<'a, T: 'a, F> ExponentialGroupByKeyMut<'a, T, F> {
    /// Returns the remainder of the original slice that is going to be
    /// returned by the iterator.
    pub fn into_remainder(self) -> &'a mut [T] {
        let len = self.remainder_len();
        unsafe { from_raw_parts_mut(self.ptr, len) }
    }
}

impl<'a, T: 'a + fmt::Debug, F> fmt::Debug for ExponentialGroupByKeyMut<'a, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.remainder_len();
        let remainder = unsafe { from_raw_parts(self.ptr, len) };

        f.debug_struct("ExponentialGroupByKeyMut")
            .field("remaining", &remainder)
            .finish()
    }
}

exponential_group_by_key! { struct ExponentialGroupByKeyMut, &'a mut [T], from_raw_parts_mut }
