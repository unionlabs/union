#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//! An implementation of sets which aims to be fast for both large sets and
//! very small sets, even if the elements are sparse relative to the universe.

use rustc_hash::FxHashSet;
use std::fmt;
use std::hash::Hash;

//=============================================================================
// SparseSet

// Handy wrappers around `SparseSetU`, if you don't want to have to guess at an "optimal"
// in-line size.
pub type SparseSet<T> = SparseSetU<[T; 12]>;
//pub type SparseSetIter<'a, T> = SparseSetUIter<'a, [T; 12]>; // No use case yet

// Implementation: for small, unordered but no dups

use core::mem::MaybeUninit;
use core::ptr::{read, write};

// Types that can be used as the backing store for a SparseSet.
pub trait Array {
    // The type of the array's elements.
    type Item;
    // Returns the number of items the array can hold.
    fn size() -> usize;
}
macro_rules! impl_array(
  ($($size:expr),+) => {
    $(
      impl<T> Array for [T; $size] {
        type Item = T;
        fn size() -> usize { $size }
      }
    )+
  }
);
impl_array!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 24, 28, 32);

// The U here stands for "unordered".  It refers to the fact that the elements
// in `Small::arr` are in no particular order, although they are
// duplicate-free.
pub enum SparseSetU<A: Array> {
    Large { set: FxHashSet<A::Item> },
    Small { card: usize, arr: MaybeUninit<A> },
}

// ================ Admin (private) methods ================

impl<A> SparseSetU<A>
where
    A: Array,
    A::Item: Eq + Hash + Copy,
{
    #[cfg(test)]
    fn is_small(&self) -> bool {
        match self {
            SparseSetU::Large { .. } => false,
            SparseSetU::Small { .. } => true,
        }
    }

    #[cfg(test)]
    fn is_large(&self) -> bool {
        !self.is_small()
    }

    #[inline(never)]
    fn upgrade(&mut self) {
        match self {
            SparseSetU::Large { .. } => panic!("SparseSetU: upgrade"),
            SparseSetU::Small { card, arr } => {
                assert!(*card == A::size());
                let mut set = FxHashSet::<A::Item>::default();
                set.reserve(A::size());
                // Could this be done faster?
                let arr_p = arr.as_mut_ptr() as *mut A::Item;
                for i in 0..*card {
                    set.insert(unsafe { read(arr_p.add(i)) });
                }
                *self = SparseSetU::Large { set }
            }
        }
    }

    // A large set is only downgradeable if its card does not exceed this value.
    #[inline(always)]
    fn small_halfmax_card(&self) -> usize {
        let limit = A::size();
        //if limit >= 4 {
        //  limit / 2
        //} else {
        //  limit - 1
        //}
        if false {
            // Set the transition point as roughly half of the inline size
            match limit {
                0 | 1 => panic!("SparseSetU: small_halfmax_card"),
                2 => 1,
                3 => 2,
                4 => 2,
                5 => 3,
                6 => 3,
                _ => limit / 2,
            }
        } else {
            // Set the transition point as roughly two thirds of the inline size
            match limit {
                0 | 1 => panic!("SparseSetU: small_halfmax_card"),
                2 => 1,
                3 => 2,
                4 => 3,
                5 => 4,
                6 => 4,
                // FIXME JRS 2020Apr10 avoid possible integer overflow here:
                _ => (2 * limit) / 3,
            }
        }
    }

    // If we have a large-format set, but the cardinality has fallen below half
    // the size of a small format set, convert it to the small format.  This
    // isn't done at the point when the cardinality falls to the max capacity of
    // a small set in order to give some hysteresis -- we don't want to be
    // constantly converting back and forth for a set whose size repeatedly
    // crosses the border.
    #[inline(never)]
    fn maybe_downgrade(&mut self) {
        let small_halfmax_card = self.small_halfmax_card();
        match self {
            SparseSetU::Large { set } => {
                if set.len() <= small_halfmax_card {
                    let mut arr = MaybeUninit::<A>::uninit();
                    let arr_p = arr.as_mut_ptr() as *mut A::Item;
                    let mut i = 0;
                    for e in set.iter() {
                        unsafe { write(arr_p.add(i), *e) };
                        i += 1;
                    }
                    assert!(i <= small_halfmax_card);
                    *self = SparseSetU::Small { card: i, arr };
                }
            }
            SparseSetU::Small { .. } => {
                panic!("SparseSetU::maybe_downgrade: is already small");
            }
        }
    }

    #[inline(always)]
    fn insert_no_dup_check(&mut self, item: A::Item) {
        match self {
            SparseSetU::Large { set } => {
                set.insert(item);
            }
            SparseSetU::Small { card, arr } => {
                assert!(*card <= A::size());
                if *card < A::size() {
                    // Stay small
                    let arr_p = arr.as_mut_ptr() as *mut A::Item;
                    unsafe {
                        write(arr_p.add(*card), item);
                    }
                    *card += 1;
                } else {
                    // Transition up
                    self.upgrade();
                    match self {
                        SparseSetU::Large { set } => {
                            let _ = set.insert(item);
                        }
                        SparseSetU::Small { .. } => {
                            // Err, what?  Still Small after upgrade?
                            panic!("SparseSetU::insert_no_dup_check")
                        }
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn small_contains<A>(card: usize, arr: &MaybeUninit<A>, item: A::Item) -> bool
where
    A: Array,
    A::Item: Eq,
{
    let arr_p = arr.as_ptr() as *const A::Item;
    for i in 0..card {
        if unsafe { read(arr_p.add(i)) } == item {
            return true;
        }
    }
    false
}

// ================ Public methods ================

impl<A> SparseSetU<A>
where
    A: Array,
    A::Item: Eq + Hash + Copy,
{
    #[inline(always)]
    pub fn empty() -> Self {
        SparseSetU::Small {
            card: 0,
            arr: MaybeUninit::uninit(),
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        match self {
            SparseSetU::Small { card, .. } => *card == 0,
            SparseSetU::Large { set } => {
                // This holds because `maybe_downgrade` will always convert a
                // zero-sized large variant into a small variant.
                assert!(set.len() > 0);
                false
            }
        }
    }

    #[inline(never)]
    pub fn card(&self) -> usize {
        match self {
            SparseSetU::Large { set } => set.len(),
            SparseSetU::Small { card, .. } => *card,
        }
    }

    #[inline(never)]
    pub fn insert(&mut self, item: A::Item) {
        match self {
            SparseSetU::Large { set } => {
                set.insert(item);
            }
            SparseSetU::Small { card, arr } => {
                assert!(*card <= A::size());
                // Do we already have it?
                if small_contains(*card, arr, item) {
                    return;
                }
                // No.
                let arr_p = arr.as_mut_ptr() as *mut A::Item;
                if *card < A::size() {
                    // Stay small
                    unsafe {
                        write(arr_p.add(*card), item);
                    }
                    *card += 1;
                } else {
                    // Transition up
                    self.upgrade();
                    self.insert(item);
                }
            }
        }
    }

    #[inline(always)]
    pub fn contains(&self, item: A::Item) -> bool {
        match self {
            SparseSetU::Large { set } => set.contains(&item),
            SparseSetU::Small { card, arr } => small_contains(*card, arr, item),
        }
    }

    #[inline(never)]
    pub fn union(&mut self, other: &Self) {
        match self {
            SparseSetU::Large { set: set1 } => match other {
                SparseSetU::Large { set: set2 } => {
                    for item in set2.iter() {
                        set1.insert(*item);
                    }
                }
                SparseSetU::Small {
                    card: card2,
                    arr: arr2,
                } => {
                    let arr2_p = arr2.as_ptr() as *const A::Item;
                    for i in 0..*card2 {
                        let item = unsafe { read(arr2_p.add(i)) };
                        set1.insert(item);
                    }
                }
            },
            SparseSetU::Small {
                card: card1,
                arr: arr1,
            } => {
                let arr1_p = arr1.as_mut_ptr() as *mut A::Item;
                match other {
                    SparseSetU::Large { set: set2 } => {
                        let mut set2c = set2.clone();
                        for i in 0..*card1 {
                            let item = unsafe { read(arr1_p.add(i)) };
                            set2c.insert(item);
                        }
                        *self = SparseSetU::Large { set: set2c };
                    }
                    SparseSetU::Small {
                        card: card2,
                        arr: arr2,
                    } => {
                        let mut extras: MaybeUninit<A> = MaybeUninit::uninit();
                        let mut n_extras = 0;
                        let extras_p = extras.as_mut_ptr() as *mut A::Item;
                        let arr2_p = arr2.as_ptr() as *const A::Item;
                        // Iterate through the second set.  Add every item not in the
                        // first set to `extras`.
                        for i in 0..*card2 {
                            let item2 = unsafe { read(arr2_p.add(i)) };
                            let mut in1 = false;
                            for j in 0..*card1 {
                                let item1 = unsafe { read(arr1_p.add(j)) };
                                if item1 == item2 {
                                    in1 = true;
                                    break;
                                }
                            }
                            if !in1 {
                                debug_assert!(n_extras < A::size());
                                unsafe {
                                    write(extras_p.add(n_extras), item2);
                                }
                                n_extras += 1;
                            }
                        }
                        // The result is the concatenation of arr1 and extras.
                        for i in 0..n_extras {
                            let item = unsafe { read(extras_p.add(i)) };
                            self.insert_no_dup_check(item);
                        }
                    }
                }
            }
        }
    }

    #[inline(never)]
    pub fn remove(&mut self, other: &Self) {
        match self {
            SparseSetU::Large { set: set1 } => {
                match other {
                    SparseSetU::Large { set: set2 } => {
                        for item in set2.iter() {
                            set1.remove(item);
                        }
                    }
                    SparseSetU::Small {
                        card: card2,
                        arr: arr2,
                    } => {
                        let arr2_p = arr2.as_ptr() as *const A::Item;
                        for i in 0..*card2 {
                            let item = unsafe { read(arr2_p.add(i)) };
                            set1.remove(&item);
                        }
                    }
                }
                self.maybe_downgrade();
            }
            SparseSetU::Small {
                card: card1,
                arr: arr1,
            } => {
                let arr1_p = arr1.as_mut_ptr() as *mut A::Item;
                match other {
                    SparseSetU::Large { set: set2 } => {
                        let mut w = 0;
                        for r in 0..*card1 {
                            let item = unsafe { read(arr1_p.add(r)) };
                            let is_in2 = set2.contains(&item);
                            if !is_in2 {
                                // Keep it.
                                if r != w {
                                    unsafe {
                                        write(arr1_p.add(w), item);
                                    }
                                }
                                w += 1;
                            }
                        }
                        *card1 = w;
                    }
                    SparseSetU::Small {
                        card: card2,
                        arr: arr2,
                    } => {
                        let arr2_p = arr2.as_ptr() as *const A::Item;
                        let mut w = 0;
                        for r in 0..*card1 {
                            let item = unsafe { read(arr1_p.add(r)) };
                            let mut is_in2 = false;
                            for i in 0..*card2 {
                                if unsafe { read(arr2_p.add(i)) } == item {
                                    is_in2 = true;
                                    break;
                                }
                            }
                            if !is_in2 {
                                // Keep it.
                                if r != w {
                                    unsafe {
                                        write(arr1_p.add(w), item);
                                    }
                                }
                                w += 1;
                            }
                        }
                        *card1 = w;
                    }
                }
            }
        }
    }

    // return true if `self` is a subset of `other`
    #[inline(never)]
    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self.card() > other.card() {
            return false;
        }
        // Visit all items in `self`, and see if they are in `other`.  If so
        // return true.
        match self {
            SparseSetU::Large { set: set1 } => match other {
                SparseSetU::Large { set: set2 } => set1.is_subset(set2),
                SparseSetU::Small {
                    card: card2,
                    arr: arr2,
                } => {
                    for item in set1.iter() {
                        if !small_contains(*card2, arr2, *item) {
                            return false;
                        }
                    }
                    true
                }
            },
            SparseSetU::Small {
                card: card1,
                arr: arr1,
            } => {
                let arr1_p = arr1.as_ptr() as *const A::Item;
                match other {
                    SparseSetU::Large { set: set2 } => {
                        for i in 0..*card1 {
                            let item = unsafe { read(arr1_p.add(i)) };
                            if !set2.contains(&item) {
                                return false;
                            }
                        }
                        true
                    }
                    SparseSetU::Small {
                        card: card2,
                        arr: arr2,
                    } => {
                        for i in 0..*card1 {
                            let item = unsafe { read(arr1_p.add(i)) };
                            if !small_contains(*card2, arr2, item) {
                                return false;
                            }
                        }
                        true
                    }
                }
            }
        }
    }

    #[inline(never)]
    pub fn from_vec(vec: Vec<A::Item>) -> Self {
        let vec_len = vec.len();
        if vec_len <= A::size() {
            let mut card = 0;
            let mut arr: MaybeUninit<A> = MaybeUninit::uninit();
            for i in 0..vec_len {
                let item = vec[i];
                if small_contains(card, &arr, item) {
                    continue;
                }
                let arr_p = arr.as_mut_ptr() as *mut A::Item;
                unsafe { write(arr_p.add(card), item) }
                card += 1;
            }
            SparseSetU::Small { card, arr }
        } else {
            let mut set = FxHashSet::<A::Item>::default();
            for i in 0..vec_len {
                set.insert(vec[i]);
            }
            SparseSetU::Large { set }
        }
    }

    #[inline(never)]
    pub fn equals(&self, other: &Self) -> bool {
        if self.card() != other.card() {
            return false;
        }
        match (self, other) {
            (SparseSetU::Large { set: set1 }, SparseSetU::Large { set: set2 }) => set1 == set2,
            (
                SparseSetU::Small {
                    card: card1,
                    arr: arr1,
                },
                SparseSetU::Small {
                    card: card2,
                    arr: arr2,
                },
            ) => {
                assert!(*card1 == *card2);
                // Check to see that all items in arr1 are present in arr2.  Since the
                // arrays have the same length and are duplicate free, although
                // unordered, this is a sufficient equality test.
                let arr1_p = arr1.as_ptr() as *const A::Item;
                let arr2_p = arr2.as_ptr() as *const A::Item;
                for i1 in 0..*card1 {
                    let item1 = unsafe { read(arr1_p.add(i1)) };
                    let mut found1 = false;
                    for i2 in 0..*card2 {
                        let item2 = unsafe { read(arr2_p.add(i2)) };
                        if item1 == item2 {
                            found1 = true;
                            break;
                        }
                    }
                    if !found1 {
                        return false;
                    }
                }
                true
            }
            (SparseSetU::Small { card, arr }, SparseSetU::Large { set })
            | (SparseSetU::Large { set }, SparseSetU::Small { card, arr }) => {
                // Same rationale as above as to why this is a sufficient test.
                let arr_p = arr.as_ptr() as *const A::Item;
                for i in 0..*card {
                    let item = unsafe { read(arr_p.add(i)) };
                    if !set.contains(&item) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<A> SparseSetU<A>
where
    A: Array,
    A::Item: Eq + Ord + Hash + Copy + fmt::Debug,
{
    #[inline(never)]
    pub fn to_vec(&self) -> Vec<A::Item> {
        let mut res = Vec::<A::Item>::new();
        match self {
            SparseSetU::Large { set } => {
                for item in set.iter() {
                    res.push(*item);
                }
            }
            SparseSetU::Small { card, arr } => {
                let arr_p = arr.as_ptr() as *const A::Item;
                for i in 0..*card {
                    res.push(unsafe { read(arr_p.add(i)) });
                }
            }
        }
        // Don't delete this.  It is important.
        res.sort_unstable();
        res
    }
}

impl<A> fmt::Debug for SparseSetU<A>
where
    A: Array + Eq + Ord + Hash + Copy + fmt::Debug,
    A::Item: Eq + Ord + Hash + Copy + fmt::Debug,
{
    #[inline(never)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Print the elements in some way which depends only on what is
        // present in the set, and not on any other factor.  In particular,
        // <Debug for FxHashSet> has been observed to to print the elements
        // of a two element set in both orders on different occasions.
        let sorted_vec = self.to_vec();
        let mut s = "{".to_string();
        for i in 0..sorted_vec.len() {
            if i > 0 {
                s = s + &", ".to_string();
            }
            s = s + &format!("{:?}", &sorted_vec[i]);
        }
        s = s + &"}".to_string();
        write!(fmt, "{}", s)
    }
}

impl<A> Clone for SparseSetU<A>
where
    A: Array + Eq + Hash + Copy + Clone,
    A::Item: Eq + Hash + Copy + Clone,
{
    #[inline(never)]
    fn clone(&self) -> Self {
        match self {
            SparseSetU::Large { set } => SparseSetU::Large { set: set.clone() },
            SparseSetU::Small { card, arr } => {
                let arr2 = arr.clone();
                SparseSetU::Small {
                    card: *card,
                    arr: arr2,
                }
            }
        }
    }
}

pub enum SparseSetUIter<'a, A: Array> {
    Large {
        set_iter: std::collections::hash_set::Iter<'a, A::Item>,
    },
    Small {
        card: usize,
        arr: &'a MaybeUninit<A>,
        next: usize,
    },
}
impl<A: Array> SparseSetU<A> {
    pub fn iter(&self) -> SparseSetUIter<A> {
        match self {
            SparseSetU::Large { set } => SparseSetUIter::Large {
                set_iter: set.iter(),
            },
            SparseSetU::Small { card, arr } => SparseSetUIter::Small {
                card: *card,
                arr,
                next: 0,
            },
        }
    }
}
impl<'a, A: Array> Iterator for SparseSetUIter<'a, A> {
    type Item = &'a A::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SparseSetUIter::Large { set_iter } => set_iter.next(),
            SparseSetUIter::Small { card, arr, next } => {
                if next < card {
                    let arr_p = arr.as_ptr() as *const A::Item;
                    let item_p = unsafe { arr_p.add(*next) };
                    *next += 1;
                    Some(unsafe { &*item_p })
                } else {
                    None
                }
            }
        }
    }
}

// ================ Testing machinery for SparseSetU ================

#[cfg(test)]
mod sparse_set_test_utils {
    // As currently set up, each number (from rand, not rand_base) has a 1-in-4
    // chance of being a dup of the last 8 numbers produced.
    pub struct RNGwithDups {
        seed: u32,
        circ: [u32; 8],
        circC: usize, // the cursor for `circ`
    }
    impl RNGwithDups {
        pub fn new() -> Self {
            Self {
                seed: 0,
                circ: [0; 8],
                circC: 0,
            }
        }
        fn rand_base(&mut self) -> u32 {
            self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
            self.seed
        }
        pub fn rand(&mut self) -> u32 {
            let r = self.rand_base();
            let rlo = r & 0xFFFF;
            let rhi = (r >> 16) & 0xFF;
            if rhi < 64 {
                self.circ[(rhi % 8) as usize]
            } else {
                self.circ[self.circC as usize] = rlo;
                self.circC += 1;
                if self.circC == 8 {
                    self.circC = 0
                };
                rlo
            }
        }
        pub fn rand_vec(&mut self, len: usize) -> Vec<u32> {
            let mut res = vec![];
            for _ in 0..len {
                res.push(self.rand());
            }
            res
        }
    }
}

#[test]
fn test_sparse_set() {
    use crate::data_structures::Set;
    let mut set = SparseSetU::<[u32; 3]>::empty();
    assert!(set.is_small());
    set.insert(3);
    assert!(set.is_small());
    set.insert(1);
    assert!(set.is_small());
    set.insert(4);
    assert!(set.is_small());
    set.insert(7);
    assert!(set.is_large());

    let iters = 20;
    let mut rng = sparse_set_test_utils::RNGwithDups::new();

    // empty
    {
        let spa = SparseSetU::<[u32; 10]>::empty();
        assert!(spa.card() == 0);
    }

    // card, is_empty
    for _ in 0..iters * 3 {
        for n1 in 0..100 {
            let size1 = n1 % 25;
            let vec1a = rng.rand_vec(size1);
            let vec1b = vec1a.clone(); // This is very stupid.
            let spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
            let std1 = Set::<u32>::from_vec(vec1b);
            assert!(spa1.card() == std1.card());
            assert!(spa1.is_empty() == (size1 == 0));
        }
    }

    // insert
    for _ in 0..iters * 3 {
        for n1 in 0..100 {
            let size1 = n1 % 25;
            let vec1a = rng.rand_vec(size1);
            let vec1b = vec1a.clone();
            let tmp = if size1 == 0 { 0 } else { vec1a[0] };
            let mut spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
            let mut std1 = Set::<u32>::from_vec(vec1b);
            // Insert an item which is almost certainly not in the set.
            let n = rng.rand();
            spa1.insert(n);
            std1.insert(n);
            assert!(spa1.card() == std1.card());
            assert!(spa1.to_vec() == std1.to_vec());
            // Insert an item which is already in the set.
            if n1 > 0 {
                spa1.insert(tmp);
                std1.insert(tmp);
                assert!(spa1.card() == std1.card());
                assert!(spa1.to_vec() == std1.to_vec());
            }
        }
    }

    // contains
    for _ in 0..iters * 2 {
        for n1 in 0..100 {
            let size1 = n1 % 25;
            let vec1a = rng.rand_vec(size1);
            let vec1b = vec1a.clone();
            let tmp = if size1 == 0 { 0 } else { vec1a[0] };
            let spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
            let std1 = Set::<u32>::from_vec(vec1b);
            // Check for an item which is almost certainly not in the set.
            let n = rng.rand();
            assert!(spa1.contains(n) == std1.contains(n));
            // Check for an item which is already in the set.
            if n1 > 0 {
                assert!(spa1.contains(tmp) == std1.contains(tmp));
            }
        }
    }

    // union
    for _ in 0..iters * 2 {
        for size1 in 0..25 {
            for size2 in 0..25 {
                let vec1a = rng.rand_vec(size1);
                let vec2a = rng.rand_vec(size2);
                let vec1b = vec1a.clone();
                let vec2b = vec2a.clone();
                let mut spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
                let spa2 = SparseSetU::<[u32; 10]>::from_vec(vec2a);
                let mut std1 = Set::<u32>::from_vec(vec1b);
                let std2 = Set::<u32>::from_vec(vec2b);
                spa1.union(&spa2);
                std1.union(&std2);
                assert!(spa1.to_vec() == std1.to_vec());
            }
        }
    }

    // remove
    for _ in 0..iters * 2 {
        for size1 in 0..25 {
            for size2 in 0..25 {
                let vec1a = rng.rand_vec(size1);
                let vec2a = rng.rand_vec(size2);
                let vec1b = vec1a.clone();
                let vec2b = vec2a.clone();
                let mut spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
                let spa2 = SparseSetU::<[u32; 10]>::from_vec(vec2a);
                let mut std1 = Set::<u32>::from_vec(vec1b);
                let std2 = Set::<u32>::from_vec(vec2b);
                spa1.remove(&spa2);
                std1.remove(&std2);
                assert!(spa1.to_vec() == std1.to_vec());
            }
        }
    }

    // is_subset_of
    for _ in 0..iters * 2 {
        for size1 in 0..25 {
            for size2 in 0..25 {
                let vec1a = rng.rand_vec(size1);
                let vec2a = rng.rand_vec(size2);
                let vec1b = vec1a.clone();
                let vec2b = vec2a.clone();
                let spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
                let spa2 = SparseSetU::<[u32; 10]>::from_vec(vec2a);
                let std1 = Set::<u32>::from_vec(vec1b);
                let std2 = Set::<u32>::from_vec(vec2b);
                assert!(spa1.is_subset_of(&spa2) == std1.is_subset_of(&std2));
            }
        }
    }

    // to_vec and from_vec are implicitly tested by the above; there's no way
    // they could be wrong and still have the above tests succeed.
    // (Famous last words!)

    // equals
    for _ in 0..iters * 2 {
        for size1 in 0..25 {
            for size2 in 0..25 {
                let vec1a = rng.rand_vec(size1);
                let vec2a = rng.rand_vec(size2);
                let vec1b = vec1a.clone();
                let vec2b = vec2a.clone();
                let spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
                let spa2 = SparseSetU::<[u32; 10]>::from_vec(vec2a);
                let std1 = Set::<u32>::from_vec(vec1b);
                let std2 = Set::<u32>::from_vec(vec2b);
                assert!(std1.equals(&std1)); // obviously
                assert!(std2.equals(&std2)); // obviously
                assert!(spa1.equals(&spa1)); // obviously
                assert!(spa2.equals(&spa2)); // obviously
                                             // More seriously
                assert!(spa1.equals(&spa2) == std1.equals(&std2));
            }
        }
    }

    // clone
    for _ in 0..iters * 3 {
        for n1 in 0..100 {
            let size1 = n1 % 25;
            let vec1a = rng.rand_vec(size1);
            let spa1 = SparseSetU::<[u32; 10]>::from_vec(vec1a);
            let spa2 = spa1.clone();
            assert!(spa1.equals(&spa2));
        }
    }
}
