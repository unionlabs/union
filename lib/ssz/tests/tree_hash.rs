// use ssz::{
//     tree_hash::{hash_concat, Hash256, MerkleHasher, BYTES_PER_CHUNK},
//     types::{typenum::U32, List},
//     Ssz,
// };

// #[derive(Ssz)]
// struct HashVec {
//     vec: List<u8, U32>,
// }

// impl HashVec {
//     fn new(vec: List<u8, U32>) -> Self {
//         Self { vec }
//     }
// }

// // REVIEW: I think this is wrong? I don't see it anywhere in the ssz docs: https://github.com/ethereum/consensus-specs/blob/v0.12.1/ssz/simple-serialize.md
// fn mix_in_selector(a: Hash256, selector: u8) -> Hash256 {
//     let mut b = [0; 32];
//     b[0] = selector;

//     hash_concat(&a, &b)
// }

// fn u8_hash_concat(v1: u8, v2: u8) -> Hash256 {
//     let mut a = [0; 32];
//     let mut b = [0; 32];

//     a[0] = v1;
//     b[0] = v2;

//     hash_concat(&a, &b)
// }

// fn u8_hash(x: u8) -> Hash256 {
//     let mut a = [0; 32];
//     a[0] = x;
//     a
// }

// #[derive(TreeHash)]
// #[tree_hash(enum_behaviour = "transparent")]
// enum FixedTrans {
//     A(u8),
//     B(u8),
// }

// #[test]
// fn fixed_trans() {
//     assert_eq!(FixedTrans::A(2).tree_hash_root(), u8_hash(2));
//     assert_eq!(FixedTrans::B(2).tree_hash_root(), u8_hash(2));
// }

// #[derive(TreeHash)]
// #[tree_hash(enum_behaviour = "union")]
// enum FixedUnion {
//     A(u8),
//     B(u8),
// }

// #[test]
// fn fixed_union() {
//     assert_eq!(FixedUnion::A(2).tree_hash_root(), u8_hash_concat(2, 0));
//     assert_eq!(FixedUnion::B(2).tree_hash_root(), u8_hash_concat(2, 1));
// }

// #[derive(TreeHash)]
// #[tree_hash(enum_behaviour = "transparent")]
// enum VariableTrans {
//     A(HashVec),
//     B(HashVec),
// }

// #[test]
// fn variable_trans() {
//     assert_eq!(
//         VariableTrans::A(HashVec::new(vec![2_u8].try_into().unwrap())).tree_hash_root(),
//         u8_hash_concat(2, 1)
//     );
//     assert_eq!(
//         VariableTrans::B(HashVec::new(vec![2_u8].try_into().unwrap())).tree_hash_root(),
//         u8_hash_concat(2, 1)
//     );
// }

// #[derive(TreeHash)]
// #[tree_hash(enum_behaviour = "union")]
// enum VariableUnion {
//     A(HashVec),
//     B(HashVec),
// }

// #[test]
// fn variable_union() {
//     assert_eq!(
//         VariableUnion::A(HashVec::new(vec![2].try_into().unwrap())).tree_hash_root(),
//         mix_in_selector(u8_hash_concat(2, 1), 0)
//     );
//     assert_eq!(
//         VariableUnion::B(HashVec::new(vec![2].try_into().unwrap())).tree_hash_root(),
//         mix_in_selector(u8_hash_concat(2, 1), 1)
//     );
// }

// mod fixed_vector {
//     use ssz::{tree_hash::merkle_root, types::Vector};
//     use typenum::{U1, U13, U16, U8};

//     #[derive(Debug, Clone, Copy, Default)]
//     struct A {
//         a: u32,
//         b: u32,
//     }

//     fn repeat(input: &[u8], n: usize) -> Vec<u8> {
//         let mut output = vec![];

//         for _ in 0..n {
//             output.append(&mut input.to_vec());
//         }

//         output
//     }

//     #[test]
//     fn tree_hash_composite() {
//         let a = A { a: 0, b: 1 };

//         let fixed: Vector<A, U1> = (vec![a]).try_into().unwrap();
//         assert_eq!(fixed.tree_hash_root(), merkle_root(&a.tree_hash_root(), 0));

//         let fixed: Vector<A, U8> = (vec![a; 8]).try_into().unwrap();
//         assert_eq!(
//             fixed.tree_hash_root(),
//             merkle_root(&repeat(&a.tree_hash_root(), 8), 0)
//         );

//         let fixed: Vector<A, U13> = (vec![a; 13]).try_into().unwrap();
//         assert_eq!(
//             fixed.tree_hash_root(),
//             merkle_root(&repeat(&a.tree_hash_root(), 13), 0)
//         );

//         let fixed: Vector<A, U16> = (vec![a; 16]).try_into().unwrap();
//         assert_eq!(
//             fixed.tree_hash_root(),
//             merkle_root(&repeat(&a.tree_hash_root(), 16), 0)
//         );
//     }
// }

// mod variable_list {
//     use ssz::{
//         tree_hash::{merkle_root, Hash256},
//         types::List,
//         TreeHash,
//     };
//     use typenum::{U1, U13, U16, U8};

//     #[derive(Debug, Clone, Copy, Default)]
//     struct A {
//         a: u32,
//         b: u32,
//     }

//     fn repeat(input: &[u8], n: usize) -> Vec<u8> {
//         let mut output = vec![];

//         for _ in 0..n {
//             output.append(&mut input.to_vec());
//         }

//         output
//     }

//     fn padded_root_with_length(bytes: &[u8], len: usize, min_nodes: usize) -> Hash256 {
//         let root = merkle_root(bytes, min_nodes);
//         ssz::tree_hash::mix_in_length(&root, len)
//     }

//     #[test]
//     fn tree_hash_composite() {
//         let a = A { a: 0, b: 1 };

//         for i in 1..=1 {
//             let fixed: List<A, U1> = vec![a; i].try_into().unwrap();
//             assert_eq!(
//                 fixed.tree_hash_root(),
//                 padded_root_with_length(&repeat(&a.tree_hash_root(), i), i, 1),
//                 "U1 {}",
//                 i
//             );
//         }

//         for i in 1..=8 {
//             let fixed: List<A, U8> = vec![a; i].try_into().unwrap();
//             assert_eq!(
//                 fixed.tree_hash_root(),
//                 padded_root_with_length(&repeat(&a.tree_hash_root(), i), i, 8),
//                 "U8 {}",
//                 i
//             );
//         }

//         for i in 1..=13 {
//             let fixed: List<A, U13> = vec![a; i].try_into().unwrap();
//             assert_eq!(
//                 fixed.tree_hash_root(),
//                 padded_root_with_length(&repeat(&a.tree_hash_root(), i), i, 13),
//                 "U13 {}",
//                 i
//             );
//         }

//         for i in 1..=16 {
//             let fixed: List<A, U16> = (vec![a; i]).try_into().unwrap();
//             assert_eq!(
//                 fixed.tree_hash_root(),
//                 padded_root_with_length(&repeat(&a.tree_hash_root(), i), i, 16),
//                 "U16 {}",
//                 i
//             );
//         }
//     }
// }
