use crate::bitset::BitSet;
use alloc::vec::Vec;

type Num = u32;
const NUM_BITS: usize = core::mem::size_of::<Num>() * 8;

/// Stack maps record which words in a stack frame contain live GC references at
/// a given instruction pointer.
///
/// Logically, a set of stack maps for a function record a table of the form:
///
/// ```text
/// +---------------------+-------------------------------------------+
/// | Instruction Pointer | SP-Relative Offsets of Live GC References |
/// +---------------------+-------------------------------------------+
/// | 0x12345678          | 2, 6, 12                                  |
/// | 0x1234abcd          | 2, 6                                      |
/// | ...                 | ...                                       |
/// +---------------------+-------------------------------------------+
/// ```
///
/// Where "instruction pointer" is an instruction pointer within the function,
/// and "offsets of live GC references" contains the offsets (in units of words)
/// from the frame's stack pointer where live GC references are stored on the
/// stack. Instruction pointers within the function that do not have an entry in
/// this table are not GC safepoints.
///
/// Because
///
/// * offsets of live GC references are relative from the stack pointer, and
/// * stack frames grow down from higher addresses to lower addresses,
///
/// to get a pointer to a live reference at offset `x` within a stack frame, you
/// add `x` from the frame's stack pointer.
///
/// For example, to calculate the pointer to the live GC reference inside "frame
/// 1" below, you would do `frame_1_sp + x`:
///
/// ```text
///           Stack
///         +-------------------+
///         | Frame 0           |
///         |                   |
///    |    |                   |
///    |    +-------------------+ <--- Frame 0's SP
///    |    | Frame 1           |
///  Grows  |                   |
///  down   |                   |
///    |    | Live GC reference | --+--
///    |    |                   |   |
///    |    |                   |   |
///    V    |                   |   x = offset of live GC reference
///         |                   |   |
///         |                   |   |
///         +-------------------+ --+--  <--- Frame 1's SP
///         | Frame 2           |
///         | ...               |
/// ```
///
/// An individual `StackMap` is associated with just one instruction pointer
/// within the function, contains the size of the stack frame, and represents
/// the stack frame as a bitmap. There is one bit per word in the stack frame,
/// and if the bit is set, then the word contains a live GC reference.
///
/// Note that a caller's `OutgoingArg` stack slots and callee's `IncomingArg`
/// stack slots overlap, so we must choose which function's stack maps record
/// live GC references in these slots. We record the `IncomingArg`s in the
/// callee's stack map.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "enable-serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StackMap {
    bitmap: Vec<BitSet<Num>>,
    mapped_words: u32,
}

impl StackMap {
    /// Create a vec of Bitsets from a slice of bools.
    pub fn from_slice(vec: &[bool]) -> Self {
        let len = vec.len();
        let num_word = len / NUM_BITS + (len % NUM_BITS != 0) as usize;
        let mut bitmap = Vec::with_capacity(num_word);

        for segment in vec.chunks(NUM_BITS) {
            let mut curr_word = 0;
            for (i, set) in segment.iter().enumerate() {
                if *set {
                    curr_word |= 1 << i;
                }
            }
            bitmap.push(BitSet(curr_word));
        }
        Self {
            mapped_words: len as u32,
            bitmap,
        }
    }

    /// Returns a specified bit.
    pub fn get_bit(&self, bit_index: usize) -> bool {
        assert!(bit_index < NUM_BITS * self.bitmap.len());
        let word_index = bit_index / NUM_BITS;
        let word_offset = bit_index % NUM_BITS;
        self.bitmap[word_index].contains(word_offset as u32)
    }

    /// Returns the raw bitmap that represents this stack map.
    pub fn as_slice(&self) -> &[BitSet<u32>] {
        &self.bitmap
    }

    /// Returns the number of words represented by this stack map.
    pub fn mapped_words(&self) -> u32 {
        self.mapped_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_maps() {
        let vec: Vec<bool> = Vec::new();
        assert!(StackMap::from_slice(&vec).bitmap.is_empty());

        let mut vec: [bool; NUM_BITS] = Default::default();
        let set_true_idx = [5, 7, 24, 31];

        for &idx in &set_true_idx {
            vec[idx] = true;
        }

        let mut vec = vec.to_vec();
        assert_eq!(
            vec![BitSet::<Num>(2164261024)],
            StackMap::from_slice(&vec).bitmap
        );

        vec.push(false);
        vec.push(true);
        let res = StackMap::from_slice(&vec);
        assert_eq!(
            vec![BitSet::<Num>(2164261024), BitSet::<Num>(2)],
            res.bitmap
        );

        assert!(res.get_bit(5));
        assert!(res.get_bit(31));
        assert!(res.get_bit(33));
        assert!(!res.get_bit(1));
    }
}
