use std::mem;
use super::{str_as_ptr, str_as_mut_ptr, str_from_raw_parts, str_from_raw_parts_mut};

macro_rules! str_group_by_key {
    (struct $name:ident, $elem:ty, $as_ptr:ident, $as_str:ident) => {
        impl<'a, F> $name<'a, F> {
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

        impl<'a, F, K> std::iter::Iterator for $name<'a, F>
        where F: FnMut(char) -> K,
              K: PartialEq,
        {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.inner.is_empty() { return None }

                let mut iter = self.inner.char_indices().peekable();
                while let (Some((_, ac)), Some((bi, bc))) = (iter.next(), iter.peek().cloned())
                {
                    if (self.func)(ac) != (self.func)(bc) {
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

        impl<'a, F, K> std::iter::DoubleEndedIterator for $name<'a, F>
        where F: FnMut(char) -> K,
              K: PartialEq,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.inner.is_empty() { return None }

                let mut iter = self.inner.char_indices().rev().peekable();
                while let (Some((ai, ac)), Some((_, bc))) = (iter.next(), iter.peek().cloned())
                {
                    if (self.func)(ac) != (self.func)(bc) {
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

        impl<'a, F, K> std::iter::FusedIterator for $name<'a, F>
        where F: FnMut(char) -> K,
              K: PartialEq,
        { }
    }
}

/// An iterator that will return non-overlapping groups in the `str`
/// using *linear/sequential search*.
///
/// It will give an element to the given function, producing a key and comparing
/// the keys to determine groups.
pub struct LinearStrGroupByKey<'a, F> {
    inner: &'a str,
    func: F,
}

impl<'a, F> LinearStrGroupByKey<'a, F> {
    pub fn new(string: &'a str, func: F) -> Self {
        Self { inner: string, func }
    }
}

str_group_by_key!{ struct LinearStrGroupByKey, &'a str, str_as_ptr, str_from_raw_parts }

/// An iterator that will return non-overlapping *mutable* groups in the `str`
/// using *linear/sequential search*.
///
/// It will give an element to the given function, producing a key and comparing
/// the keys to determine groups.
pub struct LinearStrGroupByKeyMut<'a, F> {
    inner: &'a mut str,
    func: F,
}

impl<'a, F> LinearStrGroupByKeyMut<'a, F> {
    pub fn new(string: &'a mut str, func: F) -> Self {
        Self { inner: string, func }
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        &mut self.inner
    }
}

str_group_by_key!{ struct LinearStrGroupByKeyMut, &'a mut str, str_as_mut_ptr, str_from_raw_parts_mut }
