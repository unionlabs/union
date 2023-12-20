use std::iter::FusedIterator;
use std::{mem, fmt, slice};

unsafe fn split_at_unchecked<T>(slice: &[T], mid: usize) -> (&[T], &[T]) {
    (slice.get_unchecked(..mid), slice.get_unchecked(mid..))
}

unsafe fn split_at_mut_unchecked<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    // split_at_mut_unchecked
    let len = slice.len();
    let ptr = slice.as_mut_ptr();

    // SAFETY: Caller has to check that `0 <= mid <= slice.len()`.
    //
    // `[ptr; mid]` and `[mid; len]` are not overlapping, so returning a mutable reference
    // is fine.
    (slice::from_raw_parts_mut(ptr, mid), slice::from_raw_parts_mut(ptr.add(mid), len - mid))
}

pub struct LinearGroupBy<'a, T: 'a, P> {
    slice: &'a [T],
    predicate: P,
}

impl<'a, T: 'a, P> LinearGroupBy<'a, T, P> {
    pub(crate) fn new(slice: &'a [T], predicate: P) -> Self {
        LinearGroupBy { slice, predicate }
    }
}

impl<'a, T: 'a, P> Iterator for LinearGroupBy<'a, T, P>
where
    P: FnMut(&T, &T) -> bool,
{
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let mut len = 1;
            let mut iter = self.slice.windows(2);
            while let Some([l, r]) = iter.next() {
                if (self.predicate)(l, r) { len += 1 } else { break }
            }
            let (head, tail) = unsafe { split_at_unchecked(self.slice, len) };
            self.slice = tail;
            Some(head)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.slice.is_empty() { (0, Some(0)) } else { (1, Some(self.slice.len())) }
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T: 'a, P> DoubleEndedIterator for LinearGroupBy<'a, T, P>
where
    P: FnMut(&T, &T) -> bool,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let mut len = 1;
            let mut iter = self.slice.windows(2);
            while let Some([l, r]) = iter.next_back() {
                if (self.predicate)(l, r) { len += 1 } else { break }
            }
            let (head, tail) = unsafe { split_at_unchecked(self.slice, self.slice.len() - len) };
            self.slice = head;
            Some(tail)
        }
    }
}

impl<'a, T: 'a, P> FusedIterator for LinearGroupBy<'a, T, P> where P: FnMut(&T, &T) -> bool {}

impl<'a, T: 'a + fmt::Debug, P> fmt::Debug for LinearGroupBy<'a, T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinearGroupBy").field("slice", &self.slice).finish()
    }
}

pub struct LinearGroupByMut<'a, T: 'a, P> {
    slice: &'a mut [T],
    predicate: P,
}

impl<'a, T: 'a, P> LinearGroupByMut<'a, T, P> {
    pub(crate) fn new(slice: &'a mut [T], predicate: P) -> Self {
        LinearGroupByMut { slice, predicate }
    }
}

impl<'a, T: 'a, P> Iterator for LinearGroupByMut<'a, T, P>
where
    P: FnMut(&T, &T) -> bool,
{
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let mut len = 1;
            let mut iter = self.slice.windows(2);
            while let Some([l, r]) = iter.next() {
                if (self.predicate)(l, r) { len += 1 } else { break }
            }
            let slice = mem::take(&mut self.slice);
            let (head, tail) = unsafe { split_at_mut_unchecked(slice, len) };
            self.slice = tail;
            Some(head)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.slice.is_empty() { (0, Some(0)) } else { (1, Some(self.slice.len())) }
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T: 'a, P> DoubleEndedIterator for LinearGroupByMut<'a, T, P>
where
    P: FnMut(&T, &T) -> bool,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let mut len = 1;
            let mut iter = self.slice.windows(2);
            while let Some([l, r]) = iter.next_back() {
                if (self.predicate)(l, r) { len += 1 } else { break }
            }
            let slice = mem::take(&mut self.slice);
            let (head, tail) = unsafe { split_at_mut_unchecked(slice, slice.len() - len) };
            self.slice = head;
            Some(tail)
        }
    }
}

impl<'a, T: 'a, P> FusedIterator for LinearGroupByMut<'a, T, P> where P: FnMut(&T, &T) -> bool {}

impl<'a, T: 'a + fmt::Debug, P> fmt::Debug for LinearGroupByMut<'a, T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinearGroupByMut").field("slice", &self.slice).finish()
    }
}
