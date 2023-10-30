//! The Rust standard collections
//!
//! This library provides efficient implementations of the most common general purpose programming
//! data structures.

pub mod binary_heap;
pub mod btree_map;
pub mod btree_set;
pub mod hash_map;
pub mod hash_set;
pub mod linked_list;
pub mod vec_deque;

pub use binary_heap::BinaryHeap;
pub use btree_map::BTreeMap;
pub use btree_set::BTreeSet;
pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use linked_list::LinkedList;
pub use vec_deque::VecDeque;
