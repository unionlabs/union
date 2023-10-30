use crate::{memchr, memchr2, memchr3, memrchr, memrchr2, memrchr3};

macro_rules! iter_next {
    // Common code for the memchr iterators:
    // update haystack and position and produce the index
    //
    // self: &mut Self where Self is the iterator
    // search_result: Option<usize> which is the result of the corresponding
    // memchr function.
    //
    // Returns Option<usize> (the next iterator element)
    ($self_:expr, $search_result:expr) => {
        $search_result.map(move |index| {
            // split and take the remaining back half
            $self_.haystack = $self_.haystack.split_at(index + 1).1;
            let found_position = $self_.position + index;
            $self_.position = found_position + 1;
            found_position
        })
    };
}

macro_rules! iter_next_back {
    ($self_:expr, $search_result:expr) => {
        $search_result.map(move |index| {
            // split and take the remaining front half
            $self_.haystack = $self_.haystack.split_at(index).0;
            $self_.position + index
        })
    };
}

/// An iterator for `memchr`.
pub struct Memchr<'a> {
    needle: u8,
    // The haystack to iterate over
    haystack: &'a [u8],
    // The index
    position: usize,
}

impl<'a> Memchr<'a> {
    /// Creates a new iterator that yields all positions of needle in haystack.
    #[inline]
    pub fn new(needle: u8, haystack: &[u8]) -> Memchr<'_> {
        Memchr { needle: needle, haystack: haystack, position: 0 }
    }
}

impl<'a> Iterator for Memchr<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        iter_next!(self, memchr(self.needle, self.haystack))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.haystack.len()))
    }
}

impl<'a> DoubleEndedIterator for Memchr<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        iter_next_back!(self, memrchr(self.needle, self.haystack))
    }
}

/// An iterator for `memchr2`.
pub struct Memchr2<'a> {
    needle1: u8,
    needle2: u8,
    // The haystack to iterate over
    haystack: &'a [u8],
    // The index
    position: usize,
}

impl<'a> Memchr2<'a> {
    /// Creates a new iterator that yields all positions of needle in haystack.
    #[inline]
    pub fn new(needle1: u8, needle2: u8, haystack: &[u8]) -> Memchr2<'_> {
        Memchr2 {
            needle1: needle1,
            needle2: needle2,
            haystack: haystack,
            position: 0,
        }
    }
}

impl<'a> Iterator for Memchr2<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        iter_next!(self, memchr2(self.needle1, self.needle2, self.haystack))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.haystack.len()))
    }
}

impl<'a> DoubleEndedIterator for Memchr2<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        iter_next_back!(
            self,
            memrchr2(self.needle1, self.needle2, self.haystack)
        )
    }
}

/// An iterator for `memchr3`.
pub struct Memchr3<'a> {
    needle1: u8,
    needle2: u8,
    needle3: u8,
    // The haystack to iterate over
    haystack: &'a [u8],
    // The index
    position: usize,
}

impl<'a> Memchr3<'a> {
    /// Create a new `Memchr3` that's initialized to zero with a haystack
    #[inline]
    pub fn new(
        needle1: u8,
        needle2: u8,
        needle3: u8,
        haystack: &[u8],
    ) -> Memchr3<'_> {
        Memchr3 {
            needle1: needle1,
            needle2: needle2,
            needle3: needle3,
            haystack: haystack,
            position: 0,
        }
    }
}

impl<'a> Iterator for Memchr3<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        iter_next!(
            self,
            memchr3(self.needle1, self.needle2, self.needle3, self.haystack)
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.haystack.len()))
    }
}

impl<'a> DoubleEndedIterator for Memchr3<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        iter_next_back!(
            self,
            memrchr3(self.needle1, self.needle2, self.needle3, self.haystack)
        )
    }
}
