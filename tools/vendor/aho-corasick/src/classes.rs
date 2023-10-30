use std::fmt;

/// A representation of byte oriented equivalence classes.
///
/// This is used in an FSM to reduce the size of the transition table. This can
/// have a particularly large impact not only on the total size of an FSM, but
/// also on compile times.
#[derive(Clone, Copy)]
pub struct ByteClasses([u8; 256]);

impl ByteClasses {
    /// Creates a new set of equivalence classes where all bytes are mapped to
    /// the same class.
    pub fn empty() -> ByteClasses {
        ByteClasses([0; 256])
    }

    /// Creates a new set of equivalence classes where each byte belongs to
    /// its own equivalence class.
    pub fn singletons() -> ByteClasses {
        let mut classes = ByteClasses::empty();
        for i in 0..256 {
            classes.set(i as u8, i as u8);
        }
        classes
    }

    /// Set the equivalence class for the given byte.
    #[inline]
    pub fn set(&mut self, byte: u8, class: u8) {
        self.0[byte as usize] = class;
    }

    /// Get the equivalence class for the given byte.
    #[inline]
    pub fn get(&self, byte: u8) -> u8 {
        // SAFETY: This is safe because all dense transitions have
        // exactly 256 elements, so all u8 values are valid indices.
        self.0[byte as usize]
    }

    /// Return the total number of elements in the alphabet represented by
    /// these equivalence classes. Equivalently, this returns the total number
    /// of equivalence classes.
    #[inline]
    pub fn alphabet_len(&self) -> usize {
        self.0[255] as usize + 1
    }

    /// Returns true if and only if every byte in this class maps to its own
    /// equivalence class. Equivalently, there are 256 equivalence classes
    /// and each class contains exactly one byte.
    #[inline]
    pub fn is_singleton(&self) -> bool {
        self.alphabet_len() == 256
    }

    /// Returns an iterator over a sequence of representative bytes from each
    /// equivalence class. Namely, this yields exactly N items, where N is
    /// equivalent to the number of equivalence classes. Each item is an
    /// arbitrary byte drawn from each equivalence class.
    ///
    /// This is useful when one is determinizing an NFA and the NFA's alphabet
    /// hasn't been converted to equivalence classes yet. Picking an arbitrary
    /// byte from each equivalence class then permits a full exploration of
    /// the NFA instead of using every possible byte value.
    pub fn representatives(&self) -> ByteClassRepresentatives<'_> {
        ByteClassRepresentatives { classes: self, byte: 0, last_class: None }
    }

    /// Returns all of the bytes in the given equivalence class.
    ///
    /// The second element in the tuple indicates the number of elements in
    /// the array.
    fn elements(&self, equiv: u8) -> ([u8; 256], usize) {
        let (mut array, mut len) = ([0; 256], 0);
        for b in 0..256 {
            if self.get(b as u8) == equiv {
                array[len] = b as u8;
                len += 1;
            }
        }
        (array, len)
    }
}

impl fmt::Debug for ByteClasses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_singleton() {
            write!(f, "ByteClasses({{singletons}})")
        } else {
            write!(f, "ByteClasses(")?;
            for equiv in 0..self.alphabet_len() {
                let (members, len) = self.elements(equiv as u8);
                write!(f, " {} => {:?}", equiv, &members[..len])?;
            }
            write!(f, ")")
        }
    }
}

/// An iterator over representative bytes from each equivalence class.
#[derive(Debug)]
pub struct ByteClassRepresentatives<'a> {
    classes: &'a ByteClasses,
    byte: usize,
    last_class: Option<u8>,
}

impl<'a> Iterator for ByteClassRepresentatives<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        while self.byte < 256 {
            let byte = self.byte as u8;
            let class = self.classes.get(byte);
            self.byte += 1;

            if self.last_class != Some(class) {
                self.last_class = Some(class);
                return Some(byte);
            }
        }
        None
    }
}

/// A byte class builder keeps track of an *approximation* of equivalence
/// classes of bytes during NFA construction. That is, every byte in an
/// equivalence class cannot discriminate between a match and a non-match.
///
/// For example, in the literals `abc` and `xyz`, the bytes [\x00-`], [d-w]
/// and [{-\xFF] never discriminate between a match and a non-match, precisely
/// because they never occur in the literals anywhere.
///
/// Note though that this does not necessarily compute the minimal set of
/// equivalence classes. For example, in the literals above, the byte ranges
/// [\x00-`], [d-w] and [{-\xFF] are all treated as distinct equivalence
/// classes even though they could be treated a single class. The reason for
/// this is implementation complexity. In the future, we should endeavor to
/// compute the minimal equivalence classes since they can have a rather large
/// impact on the size of the DFA.
///
/// The representation here is 256 booleans, all initially set to false. Each
/// boolean maps to its corresponding byte based on position. A `true` value
/// indicates the end of an equivalence class, where its corresponding byte
/// and all of the bytes corresponding to all previous contiguous `false`
/// values are in the same equivalence class.
///
/// This particular representation only permits contiguous ranges of bytes to
/// be in the same equivalence class, which means that we can never discover
/// the true minimal set of equivalence classes.
#[derive(Debug)]
pub struct ByteClassBuilder(Vec<bool>);

impl ByteClassBuilder {
    /// Create a new builder of byte classes where all bytes are part of the
    /// same equivalence class.
    pub fn new() -> ByteClassBuilder {
        ByteClassBuilder(vec![false; 256])
    }

    /// Indicate the the range of byte given (inclusive) can discriminate a
    /// match between it and all other bytes outside of the range.
    pub fn set_range(&mut self, start: u8, end: u8) {
        debug_assert!(start <= end);
        if start > 0 {
            self.0[start as usize - 1] = true;
        }
        self.0[end as usize] = true;
    }

    /// Build byte classes that map all byte values to their corresponding
    /// equivalence class. The last mapping indicates the largest equivalence
    /// class identifier (which is never bigger than 255).
    pub fn build(&self) -> ByteClasses {
        let mut classes = ByteClasses::empty();
        let mut class = 0u8;
        let mut i = 0;
        loop {
            classes.set(i as u8, class as u8);
            if i >= 255 {
                break;
            }
            if self.0[i] {
                class = class.checked_add(1).unwrap();
            }
            i += 1;
        }
        classes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_classes() {
        let mut set = ByteClassBuilder::new();
        set.set_range(b'a', b'z');

        let classes = set.build();
        assert_eq!(classes.get(0), 0);
        assert_eq!(classes.get(1), 0);
        assert_eq!(classes.get(2), 0);
        assert_eq!(classes.get(b'a' - 1), 0);
        assert_eq!(classes.get(b'a'), 1);
        assert_eq!(classes.get(b'm'), 1);
        assert_eq!(classes.get(b'z'), 1);
        assert_eq!(classes.get(b'z' + 1), 2);
        assert_eq!(classes.get(254), 2);
        assert_eq!(classes.get(255), 2);

        let mut set = ByteClassBuilder::new();
        set.set_range(0, 2);
        set.set_range(4, 6);
        let classes = set.build();
        assert_eq!(classes.get(0), 0);
        assert_eq!(classes.get(1), 0);
        assert_eq!(classes.get(2), 0);
        assert_eq!(classes.get(3), 1);
        assert_eq!(classes.get(4), 2);
        assert_eq!(classes.get(5), 2);
        assert_eq!(classes.get(6), 2);
        assert_eq!(classes.get(7), 3);
        assert_eq!(classes.get(255), 3);
    }

    #[test]
    fn full_byte_classes() {
        let mut set = ByteClassBuilder::new();
        for i in 0..256u16 {
            set.set_range(i as u8, i as u8);
        }
        assert_eq!(set.build().alphabet_len(), 256);
    }
}
