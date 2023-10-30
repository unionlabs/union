//! Iterating over set flag values.

use crate::{Flag, Flags};

/// An iterator over a set of flags.
///
/// Any bits that don't correspond to a valid flag will be yielded
/// as a final item from the iterator.
pub struct Iter<B: 'static> {
    inner: IterNames<B>,
    done: bool,
}

impl<B: Flags> Iter<B> {
    /// Create a new iterator over the given set of flags.
    pub(crate) fn new(flags: &B) -> Self {
        Iter {
            inner: IterNames::new(flags),
            done: false,
        }
    }
}

impl<B: 'static> Iter<B> {
    #[doc(hidden)]
    pub const fn __private_const_new(flags: &'static [Flag<B>], source: B, remaining: B) -> Self {
        Iter {
            inner: IterNames::__private_const_new(flags, source, remaining),
            done: false,
        }
    }
}

impl<B: Flags> Iterator for Iter<B> {
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some((_, flag)) => Some(flag),
            None if !self.done => {
                self.done = true;

                // After iterating through valid names, if there are any bits left over
                // then return one final value that includes them. This makes `into_iter`
                // and `from_iter` roundtrip
                if !self.inner.remaining().is_empty() {
                    Some(B::from_bits_retain(self.inner.remaining.bits()))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

/// An iterator over a set of flags and their names.
///
/// Any bits that don't correspond to a valid flag will be ignored.
pub struct IterNames<B: 'static> {
    flags: &'static [Flag<B>],
    idx: usize,
    source: B,
    remaining: B,
}

impl<B: Flags> IterNames<B> {
    /// Create a new iterator over the given set of flags.
    pub(crate) fn new(flags: &B) -> Self {
        IterNames {
            flags: B::FLAGS,
            idx: 0,
            remaining: B::from_bits_retain(flags.bits()),
            source: B::from_bits_retain(flags.bits()),
        }
    }
}

impl<B: 'static> IterNames<B> {
    #[doc(hidden)]
    pub const fn __private_const_new(flags: &'static [Flag<B>], source: B, remaining: B) -> Self {
        IterNames {
            flags,
            idx: 0,
            remaining,
            source,
        }
    }

    /// Get the remaining (unyielded) flags.
    ///
    /// Once the iterator has finished, this method can be used to
    /// check whether or not there are any bits that didn't correspond
    /// to a valid flag remaining.
    pub fn remaining(&self) -> &B {
        &self.remaining
    }
}

impl<B: Flags> Iterator for IterNames<B> {
    type Item = (&'static str, B);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(flag) = self.flags.get(self.idx) {
            // Short-circuit if our state is empty
            if self.remaining.is_empty() {
                return None;
            }

            self.idx += 1;

            let bits = flag.value().bits();

            // If the flag is set in the original source _and_ it has bits that haven't
            // been covered by a previous flag yet then yield it. These conditions cover
            // two cases for multi-bit flags:
            //
            // 1. When flags partially overlap, such as `0b00000001` and `0b00000101`, we'll
            // yield both flags.
            // 2. When flags fully overlap, such as in convenience flags that are a shorthand for others,
            // we won't yield both flags.
            if self.source.contains(B::from_bits_retain(bits))
                && self.remaining.intersects(B::from_bits_retain(bits))
            {
                self.remaining.remove(B::from_bits_retain(bits));

                return Some((flag.name(), B::from_bits_retain(bits)));
            }
        }

        None
    }
}
