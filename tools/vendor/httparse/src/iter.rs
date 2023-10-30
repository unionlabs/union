use core::slice;
use core::convert::TryInto;
use core::convert::TryFrom;

pub struct Bytes<'a> {
    slice: &'a [u8],
    pos: usize
}

impl<'a> Bytes<'a> {
    #[inline]
    pub fn new(slice: &'a [u8]) -> Bytes<'a> {
        Bytes {
            slice,
            pos: 0
        }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.peek_ahead(0)
    }

    #[inline]
    pub fn peek_ahead(&self, n: usize) -> Option<u8> {
        self.slice.get(self.pos + n).copied()
    }

    #[inline]
    pub fn peek_n<U: TryFrom<&'a[u8]>>(&self, n: usize) -> Option<U> {
        self.slice.get(self.pos..self.pos + n)?.try_into().ok()
    }

    #[inline]
    pub unsafe fn bump(&mut self) {
        debug_assert!(self.pos < self.slice.len(), "overflow");
        self.pos += 1;
    }

    #[allow(unused)]
    #[inline]
    pub unsafe fn advance(&mut self, n: usize) {
        debug_assert!(self.pos + n <= self.slice.len(), "overflow");
        self.pos += n;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    pub fn slice(&mut self) -> &'a [u8] {
        // not moving position at all, so it's safe
        unsafe {
            self.slice_skip(0)
        }
    }

    #[inline]
    pub unsafe fn slice_skip(&mut self, skip: usize) -> &'a [u8] {
        debug_assert!(self.pos >= skip);
        let head_pos = self.pos - skip;
        let ptr = self.slice.as_ptr();
        let head = slice::from_raw_parts(ptr, head_pos);
        let tail = slice::from_raw_parts(ptr.add(self.pos), self.slice.len() - self.pos);
        self.pos = 0;
        self.slice = tail;
        head
    }

    #[inline]
    pub unsafe fn advance_and_commit(&mut self, n: usize) {
        debug_assert!(self.pos + n <= self.slice.len(), "overflow");
        self.pos += n;
        let ptr = self.slice.as_ptr();
        let tail = slice::from_raw_parts(ptr.add(n), self.slice.len() - n);
        self.pos = 0;
        self.slice = tail;
    }
}

impl<'a> AsRef<[u8]> for Bytes<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.slice[self.pos..]
    }
}

impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.slice.len() > self.pos {
            let b = unsafe { *self.slice.get_unchecked(self.pos) };
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }
}
