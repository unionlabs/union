use std::sync::Arc;

use super::*;

fn int_to_hash256(int: u128) -> Hash256 {
    let mut bytes = [0; HASHSIZE];
    bytes[0..16].copy_from_slice(&int.to_le_bytes());
    bytes
}

// REVIEW: Use T::BITS instead of passing $bit_size?
macro_rules! impl_for_bitsize {
    ($type: ident, $bit_size: expr) => {
        impl TreeHash for $type {
            fn tree_hash_type() -> TreeHashType {
                TreeHashType::Basic
            }

            fn tree_hash_packed_encoding(&self) -> PackedEncoding {
                PackedEncoding::from_slice(&self.to_le_bytes())
            }

            fn tree_hash_packing_factor() -> usize {
                HASHSIZE / ($bit_size / 8)
            }

            #[allow(clippy::cast_lossless)] // Lint does not apply to all uses of this macro.
            fn tree_hash_root(&self) -> Hash256 {
                int_to_hash256(*self as u128)
            }
        }
    };
}

impl_for_bitsize!(u8, 8);
impl_for_bitsize!(u16, 16);
impl_for_bitsize!(u32, 32);
impl_for_bitsize!(u64, 64);
impl_for_bitsize!(u128, 128);
impl_for_bitsize!(usize, 64);

impl TreeHash for bool {
    fn tree_hash_type() -> TreeHashType {
        TreeHashType::Basic
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        (*self as u8).tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        u8::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> Hash256 {
        int_to_hash256(*self as u128)
    }
}

/// Only valid for byte types less than 32 bytes.
macro_rules! impl_for_lt_32byte_u8_array {
    ($len: expr) => {
        impl TreeHash for [u8; $len] {
            fn tree_hash_type() -> TreeHashType {
                TreeHashType::Vector
            }

            fn tree_hash_packed_encoding(&self) -> PackedEncoding {
                unreachable!("bytesN should never be packed.")
            }

            fn tree_hash_packing_factor() -> usize {
                unreachable!("bytesN should never be packed.")
            }

            fn tree_hash_root(&self) -> Hash256 {
                let mut result = [0; 32];
                result[0..$len].copy_from_slice(&self[..]);
                result
            }
        }
    };
}

impl_for_lt_32byte_u8_array!(4);
impl_for_lt_32byte_u8_array!(32);

impl TreeHash for [u8; 48] {
    fn tree_hash_type() -> TreeHashType {
        TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        unreachable!("Vector should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("Vector should never be packed.")
    }

    fn tree_hash_root(&self) -> Hash256 {
        let values_per_chunk = BYTES_PER_CHUNK;
        let minimum_chunk_count = (48 + values_per_chunk - 1) / values_per_chunk;
        merkle_root(self, minimum_chunk_count)
    }
}

impl<T: TreeHash> TreeHash for Arc<T> {
    fn tree_hash_type() -> TreeHashType {
        T::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        self.as_ref().tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        T::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> Hash256 {
        self.as_ref().tree_hash_root()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bool() {
        let mut true_bytes: Vec<u8> = vec![1];
        true_bytes.append(&mut vec![0; 31]);

        let false_bytes: Vec<u8> = vec![0; 32];

        assert_eq!(true.tree_hash_root(), true_bytes.as_slice());
        assert_eq!(false.tree_hash_root(), false_bytes.as_slice());
    }

    #[test]
    fn int_to_bytes() {
        assert_eq!(int_to_hash256(0), [0; 32]);
        assert_eq!(
            int_to_hash256(1),
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
        assert_eq!(
            int_to_hash256(u64::max_value() as u128),
            [
                255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
