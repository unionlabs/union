use std::mem;
use super::{str_as_ptr, str_as_mut_ptr, str_from_raw_parts, str_from_raw_parts_mut};

macro_rules! str_group_by {
    (struct $name:ident, $elem:ty, $as_ptr:ident, $as_str:ident) => {
        impl<'a, P> $name<'a, P> {
            #[inline]
            pub fn as_str(&self) -> &str {
                self.inner
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }

            #[inline]
            pub fn remainder_len(&self) -> usize {
                self.inner.len()
            }
        }

        impl<'a, P> std::iter::Iterator for $name<'a, P>
        where P: FnMut(char, char) -> bool,
        {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.inner.is_empty() { return None }

                let mut iter = self.inner.char_indices().peekable();
                while let (Some((_, ac)), Some((bi, bc))) = (iter.next(), iter.peek().cloned())
                {
                    if !(self.predicate)(ac, bc) {
                        let len = self.inner.len();
                        let ptr = $as_ptr(self.inner);

                        let left = unsafe { $as_str(ptr, bi) };
                        let right = unsafe { $as_str(ptr.add(bi), len - bi) };

                        self.inner = right;
                        return Some(left);
                    }
                }

                let output = mem::replace(&mut self.inner, Default::default());
                return Some(output);
            }

            fn last(mut self) -> Option<Self::Item> {
                self.next_back()
            }
        }

        impl<'a, P> std::iter::DoubleEndedIterator for $name<'a, P>
        where P: FnMut(char, char) -> bool,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.inner.is_empty() { return None }

                let mut iter = self.inner.char_indices().rev().peekable();
                while let (Some((ai, ac)), Some((_, bc))) = (iter.next(), iter.peek().cloned())
                {
                    if !(self.predicate)(ac, bc) {
                        let len = self.inner.len();
                        let ptr = $as_ptr(self.inner);

                        let left = unsafe { $as_str(ptr, ai) };
                        let right = unsafe { $as_str(ptr.add(ai), len - ai) };

                        self.inner = left;
                        return Some(right);
                    }
                }

                let output = mem::replace(&mut self.inner, Default::default());
                return Some(output);
            }
        }

        impl<'a, P> std::iter::FusedIterator for $name<'a, P>
        where P: FnMut(char, char) -> bool,
        { }
    }
}

/// An iterator that will return non-overlapping groups in the `str`
/// using *linear/sequential search*.
///
/// It will give two contiguous `char` to the predicate function.
pub struct LinearStrGroupBy<'a, P> {
    inner: &'a str,
    predicate: P,
}

impl<'a, P> LinearStrGroupBy<'a, P> {
    pub fn new(string: &'a str, predicate: P) -> Self {
        Self { inner: string, predicate }
    }
}

str_group_by!{ struct LinearStrGroupBy, &'a str, str_as_ptr, str_from_raw_parts }

/// An iterator that will return non-overlapping *mutable* groups in the `str`
/// using *linear/sequential search*.
///
/// It will give two contiguous `char` to the predicate function.
pub struct LinearStrGroupByMut<'a, P> {
    inner: &'a mut str,
    predicate: P,
}

impl<'a, P> LinearStrGroupByMut<'a, P> {
    pub fn new(string: &'a mut str, predicate: P) -> Self {
        Self { inner: string, predicate }
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        &mut self.inner
    }
}

str_group_by!{ struct LinearStrGroupByMut, &'a mut str, str_as_mut_ptr, str_from_raw_parts_mut }
