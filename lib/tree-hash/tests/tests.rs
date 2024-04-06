use ssz::{
    types::{typenum::U32, VariableList},
    Encode,
};
use tree_hash::{hash_concat, Hash256, MerkleHasher, PackedEncoding, TreeHash, BYTES_PER_CHUNK};

#[derive(Encode)]
struct HashVec {
    vec: VariableList<u8, U32>,
}

impl HashVec {
    fn new(vec: VariableList<u8, U32>) -> Self {
        Self { vec }
    }
}

impl tree_hash::TreeHash for HashVec {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_root(&self) -> Hash256 {
        let mut hasher =
            MerkleHasher::with_leaves((self.vec.len() + BYTES_PER_CHUNK - 1) / BYTES_PER_CHUNK);

        for item in &self.vec {
            hasher.write(&item.tree_hash_packed_encoding()).unwrap()
        }

        let root = hasher.finish().unwrap();

        tree_hash::mix_in_length(&root, self.vec.len())
    }
}

fn mix_in_selector(a: Hash256, selector: u8) -> Hash256 {
    let mut b = [0; 32];
    b[0] = selector;

    hash_concat(&a, &b)
}

fn u8_hash_concat(v1: u8, v2: u8) -> Hash256 {
    let mut a = [0; 32];
    let mut b = [0; 32];

    a[0] = v1;
    b[0] = v2;

    hash_concat(&a, &b)
}

fn u8_hash(x: u8) -> Hash256 {
    let mut a = [0; 32];
    a[0] = x;
    a
}

#[derive(TreeHash)]
#[tree_hash(enum_behaviour = "transparent")]
enum FixedTrans {
    A(u8),
    B(u8),
}

#[test]
fn fixed_trans() {
    assert_eq!(FixedTrans::A(2).tree_hash_root(), u8_hash(2));
    assert_eq!(FixedTrans::B(2).tree_hash_root(), u8_hash(2));
}

#[derive(TreeHash)]
#[tree_hash(enum_behaviour = "union")]
enum FixedUnion {
    A(u8),
    B(u8),
}

#[test]
fn fixed_union() {
    assert_eq!(FixedUnion::A(2).tree_hash_root(), u8_hash_concat(2, 0));
    assert_eq!(FixedUnion::B(2).tree_hash_root(), u8_hash_concat(2, 1));
}

#[derive(TreeHash)]
#[tree_hash(enum_behaviour = "transparent")]
enum VariableTrans {
    A(HashVec),
    B(HashVec),
}

#[test]
fn variable_trans() {
    assert_eq!(
        VariableTrans::A(HashVec::new(vec![2_u8].try_into().unwrap())).tree_hash_root(),
        u8_hash_concat(2, 1)
    );
    assert_eq!(
        VariableTrans::B(HashVec::new(vec![2_u8].try_into().unwrap())).tree_hash_root(),
        u8_hash_concat(2, 1)
    );
}

#[derive(TreeHash)]
#[tree_hash(enum_behaviour = "union")]
enum VariableUnion {
    A(HashVec),
    B(HashVec),
}

#[test]
fn variable_union() {
    assert_eq!(
        VariableUnion::A(HashVec::new(vec![2].try_into().unwrap())).tree_hash_root(),
        mix_in_selector(u8_hash_concat(2, 1), 0)
    );
    assert_eq!(
        VariableUnion::B(HashVec::new(vec![2].try_into().unwrap())).tree_hash_root(),
        mix_in_selector(u8_hash_concat(2, 1), 1)
    );
}
