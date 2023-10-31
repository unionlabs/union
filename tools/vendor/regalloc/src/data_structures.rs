//! Data structures for the whole crate.

use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::{Iter, IterMut};

use crate::{Function, RegUsageMapper};

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

//=============================================================================
// Queues

pub type Queue<T> = VecDeque<T>;

//=============================================================================
// Maps

// NOTE: plain HashMap is nondeterministic, even in a single-threaded
// scenario, which can make debugging code that uses it really confusing.  So
// we use FxHashMap instead, as it *is* deterministic, and, allegedly, faster
// too.
pub type Map<K, V> = FxHashMap<K, V>;

//=============================================================================
// Sets of things

// Same comment as above for FxHashMap.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Set<T: Eq + Hash> {
    set: FxHashSet<T>,
}

impl<T: Eq + Ord + Hash + Copy + fmt::Debug> Set<T> {
    #[inline(never)]
    pub fn empty() -> Self {
        Self {
            set: FxHashSet::<T>::default(),
        }
    }

    #[inline(never)]
    pub fn unit(item: T) -> Self {
        let mut s = Self::empty();
        s.insert(item);
        s
    }

    #[inline(never)]
    pub fn two(item1: T, item2: T) -> Self {
        let mut s = Self::empty();
        s.insert(item1);
        s.insert(item2);
        s
    }

    #[inline(never)]
    pub fn card(&self) -> usize {
        self.set.len()
    }

    #[inline(never)]
    pub fn insert(&mut self, item: T) {
        self.set.insert(item);
    }

    #[inline(never)]
    pub fn delete(&mut self, item: T) {
        self.set.remove(&item);
    }

    #[inline(never)]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    #[inline(never)]
    pub fn contains(&self, item: T) -> bool {
        self.set.contains(&item)
    }

    #[inline(never)]
    pub fn intersect(&mut self, other: &Self) {
        let mut res = FxHashSet::<T>::default();
        for item in self.set.iter() {
            if other.set.contains(item) {
                res.insert(*item);
            }
        }
        self.set = res;
    }

    #[inline(never)]
    pub fn union(&mut self, other: &Self) {
        for item in other.set.iter() {
            self.set.insert(*item);
        }
    }

    #[inline(never)]
    pub fn remove(&mut self, other: &Self) {
        for item in other.set.iter() {
            self.set.remove(item);
        }
    }

    #[inline(never)]
    pub fn intersects(&self, other: &Self) -> bool {
        !self.set.is_disjoint(&other.set)
    }

    #[inline(never)]
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.set.is_subset(&other.set)
    }

    #[inline(never)]
    pub fn to_vec(&self) -> Vec<T> {
        let mut res = Vec::<T>::new();
        for item in self.set.iter() {
            res.push(*item)
        }
        // Don't delete this.  It is important.
        res.sort_unstable();
        res
    }

    #[inline(never)]
    pub fn from_vec(vec: Vec<T>) -> Self {
        let mut res = Set::<T>::empty();
        for x in vec {
            res.insert(x);
        }
        res
    }

    #[inline(never)]
    pub fn equals(&self, other: &Self) -> bool {
        self.set == other.set
    }

    #[inline(never)]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.set.retain(f)
    }

    #[inline(never)]
    pub fn map<F, U>(&self, f: F) -> Set<U>
    where
        F: Fn(&T) -> U,
        U: Eq + Ord + Hash + Copy + fmt::Debug,
    {
        Set {
            set: self.set.iter().map(f).collect(),
        }
    }

    #[inline(never)]
    pub fn filter_map<F, U>(&self, f: F) -> Set<U>
    where
        F: Fn(&T) -> Option<U>,
        U: Eq + Ord + Hash + Copy + fmt::Debug,
    {
        Set {
            set: self.set.iter().filter_map(f).collect(),
        }
    }

    pub fn clear(&mut self) {
        self.set.clear();
    }
}

impl<T: Eq + Ord + Hash + Copy + fmt::Debug> fmt::Debug for Set<T> {
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

pub struct SetIter<'a, T> {
    set_iter: std::collections::hash_set::Iter<'a, T>,
}
impl<T: Eq + Hash> Set<T> {
    pub fn iter(&self) -> SetIter<T> {
        SetIter {
            set_iter: self.set.iter(),
        }
    }
}
impl<'a, T> Iterator for SetIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.set_iter.next()
    }
}

//=============================================================================
// Iteration boilerplate for entities.  The only purpose of this is to support
// constructions of the form
//
//   for ent in startEnt .dotdot( endPlus1Ent ) {
//   }
//
// until such time as `trait Step` is available in stable Rust.  At that point
// `fn dotdot` and all of the following can be removed, and the loops
// rewritten using the standard syntax:
//
//   for ent in startEnt .. endPlus1Ent {
//   }

pub trait Zero {
    fn zero() -> Self;
}

pub trait PlusN {
    fn plus_n(&self, n: usize) -> Self;
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Range<T> {
    first: T,
    len: usize,
}

impl<T: Copy + PartialOrd + PlusN> IntoIterator for Range<T> {
    type Item = T;
    type IntoIter = MyIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        MyIterator {
            range: self,
            next: self.first,
        }
    }
}

impl<T: Copy + Eq + Ord + PlusN> Range<T> {
    /// Create a new range object.
    pub fn new(from: T, len: usize) -> Range<T> {
        Range { first: from, len }
    }

    pub fn start(&self) -> T {
        self.first
    }

    pub fn first(&self) -> T {
        assert!(self.len() > 0);
        self.start()
    }

    pub fn last(&self) -> T {
        assert!(self.len() > 0);
        self.start().plus_n(self.len() - 1)
    }

    pub fn last_plus1(&self) -> T {
        self.start().plus_n(self.len())
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn contains(&self, t: T) -> bool {
        t >= self.first && t < self.first.plus_n(self.len)
    }
}

pub struct MyIterator<T> {
    range: Range<T>,
    next: T,
}
impl<T: Copy + PartialOrd + PlusN> Iterator for MyIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.range.first.plus_n(self.range.len) {
            None
        } else {
            let res = Some(self.next);
            self.next = self.next.plus_n(1);
            res
        }
    }
}

//=============================================================================
// Vectors where both the index and element types can be specified (and at
// most 2^32-1 elems can be stored.  What if this overflows?)

pub struct TypedIxVec<TyIx, Ty> {
    vek: Vec<Ty>,
    ty_ix: PhantomData<TyIx>,
}

impl<TyIx, Ty> TypedIxVec<TyIx, Ty>
where
    Ty: Clone,
    TyIx: Copy + Eq + Ord + Zero + PlusN + Into<u32>,
{
    pub fn new() -> Self {
        Self {
            vek: Vec::new(),
            ty_ix: PhantomData::<TyIx>,
        }
    }
    pub fn from_vec(vek: Vec<Ty>) -> Self {
        Self {
            vek,
            ty_ix: PhantomData::<TyIx>,
        }
    }
    pub fn append(&mut self, other: &mut TypedIxVec<TyIx, Ty>) {
        // FIXME what if this overflows?
        self.vek.append(&mut other.vek);
    }
    pub fn iter(&self) -> Iter<Ty> {
        self.vek.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<Ty> {
        self.vek.iter_mut()
    }
    pub fn len(&self) -> u32 {
        // FIXME what if this overflows?
        self.vek.len() as u32
    }
    pub fn push(&mut self, item: Ty) {
        // FIXME what if this overflows?
        self.vek.push(item);
    }
    pub fn resize(&mut self, new_len: u32, value: Ty) {
        self.vek.resize(new_len as usize, value);
    }
    pub fn reserve(&mut self, additional: usize) {
        self.vek.reserve(additional);
    }
    pub fn elems(&self) -> &[Ty] {
        &self.vek[..]
    }
    pub fn elems_mut(&mut self) -> &mut [Ty] {
        &mut self.vek[..]
    }
    pub fn range(&self) -> Range<TyIx> {
        Range::new(TyIx::zero(), self.len() as usize)
    }
    pub fn remove(&mut self, idx: TyIx) -> Ty {
        self.vek.remove(idx.into() as usize)
    }
    pub fn sort_by<F: FnMut(&Ty, &Ty) -> Ordering>(&mut self, compare: F) {
        self.vek.sort_by(compare)
    }
    pub fn sort_unstable_by<F: FnMut(&Ty, &Ty) -> Ordering>(&mut self, compare: F) {
        self.vek.sort_unstable_by(compare)
    }
    pub fn clear(&mut self) {
        self.vek.clear();
    }
}

impl<TyIx, Ty> Index<TyIx> for TypedIxVec<TyIx, Ty>
where
    TyIx: Into<u32>,
{
    type Output = Ty;
    fn index(&self, ix: TyIx) -> &Ty {
        &self.vek[ix.into() as usize]
    }
}

impl<TyIx, Ty> IndexMut<TyIx> for TypedIxVec<TyIx, Ty>
where
    TyIx: Into<u32>,
{
    fn index_mut(&mut self, ix: TyIx) -> &mut Ty {
        &mut self.vek[ix.into() as usize]
    }
}

impl<TyIx, Ty> Clone for TypedIxVec<TyIx, Ty>
where
    Ty: Clone,
{
    // This is only needed for debug printing.
    fn clone(&self) -> Self {
        Self {
            vek: self.vek.clone(),
            ty_ix: PhantomData::<TyIx>,
        }
    }
}

//=============================================================================

macro_rules! generate_boilerplate {
    ($TypeIx:ident, $Type:ident, $PrintingPrefix:expr) => {
        #[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
        // Firstly, the indexing type (TypeIx)
        pub enum $TypeIx {
            $TypeIx(u32),
        }
        impl $TypeIx {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn new(n: u32) -> Self {
                debug_assert!(n != u32::max_value());
                Self::$TypeIx(n)
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub const fn max_value() -> Self {
                Self::$TypeIx(u32::max_value() - 1)
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub const fn min_value() -> Self {
                Self::$TypeIx(u32::min_value())
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub const fn invalid_value() -> Self {
                Self::$TypeIx(u32::max_value())
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn is_valid(self) -> bool {
                self != Self::invalid_value()
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn is_invalid(self) -> bool {
                self == Self::invalid_value()
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn get(self) -> u32 {
                debug_assert!(self.is_valid());
                match self {
                    $TypeIx::$TypeIx(n) => n,
                }
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn plus(self, delta: u32) -> $TypeIx {
                debug_assert!(self.is_valid());
                $TypeIx::$TypeIx(self.get() + delta)
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn minus(self, delta: u32) -> $TypeIx {
                debug_assert!(self.is_valid());
                $TypeIx::$TypeIx(self.get() - delta)
            }
            #[allow(dead_code)]
            pub fn dotdot(&self, last_plus1: $TypeIx) -> Range<$TypeIx> {
                debug_assert!(self.is_valid());
                let len = (last_plus1.get() - self.get()) as usize;
                Range::new(*self, len)
            }
        }
        impl fmt::Debug for $TypeIx {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                if self.is_invalid() {
                    write!(fmt, "{}<NONE>", $PrintingPrefix)
                } else {
                    write!(fmt, "{}{}", $PrintingPrefix, &self.get())
                }
            }
        }
        impl PlusN for $TypeIx {
            #[inline(always)]
            fn plus_n(&self, n: usize) -> Self {
                debug_assert!(self.is_valid());
                self.plus(n as u32)
            }
        }
        impl Into<u32> for $TypeIx {
            #[inline(always)]
            fn into(self) -> u32 {
                debug_assert!(self.is_valid());
                self.get()
            }
        }
        impl Zero for $TypeIx {
            #[inline(always)]
            fn zero() -> Self {
                $TypeIx::new(0)
            }
        }
    };
}

generate_boilerplate!(InstIx, Inst, "i");

generate_boilerplate!(BlockIx, Block, "b");

generate_boilerplate!(RangeFragIx, RangeFrag, "f");

generate_boilerplate!(VirtualRangeIx, VirtualRange, "vr");

generate_boilerplate!(RealRangeIx, RealRange, "rr");

impl<TyIx, Ty: fmt::Debug> fmt::Debug for TypedIxVec<TyIx, Ty> {
    // This is something of a hack in the sense that it doesn't show the
    // indices, but oh well ..
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.vek)
    }
}

//=============================================================================
// Definitions of register classes, registers and stack slots, and printing
// thereof. Note that this register class definition is meant to be
// architecture-independent: it simply captures common integer/float/vector
// types that machines are likely to use. TODO: investigate whether we need a
// more flexible register-class definition mechanism.

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum RegClass {
    I32 = 0,
    F32 = 1,
    I64 = 2,
    F64 = 3,
    V128 = 4,
    INVALID = 5,
}

/// The number of register classes that exist.
/// N.B.: must be <= 7 (fit into 3 bits) for 32-bit VReg/RReg packed format!
pub const NUM_REG_CLASSES: usize = 5;

impl RegClass {
    /// Convert a register class to a u32 index.
    #[inline(always)]
    pub const fn rc_to_u32(self) -> u32 {
        self as u32
    }
    /// Convert a register class to a usize index.
    #[inline(always)]
    pub const fn rc_to_usize(self) -> usize {
        self as usize
    }
    /// Construct a register class from a u32.
    #[inline(always)]
    pub fn rc_from_u32(rc: u32) -> RegClass {
        match rc {
            0 => RegClass::I32,
            1 => RegClass::F32,
            2 => RegClass::I64,
            3 => RegClass::F64,
            4 => RegClass::V128,
            _ => panic!("RegClass::rc_from_u32"),
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            RegClass::I32 => "I",
            RegClass::I64 => "J",
            RegClass::F32 => "F",
            RegClass::F64 => "D",
            RegClass::V128 => "V",
            RegClass::INVALID => panic!("RegClass::short_name"),
        }
    }

    pub fn long_name(self) -> &'static str {
        match self {
            RegClass::I32 => "I32",
            RegClass::I64 => "I32",
            RegClass::F32 => "F32",
            RegClass::F64 => "F32",
            RegClass::V128 => "V128",
            RegClass::INVALID => panic!("RegClass::long_name"),
        }
    }
}

// Reg represents both real and virtual registers.  For compactness and speed,
// these fields are packed into a single u32.  The format is:
//
// Virtual Reg:   1  rc:3                index:28
// Real Reg:      0  rc:3  uu:12  enc:8  index:8
//
// `rc` is the register class.  `uu` means "unused".  `enc` is the hardware
// encoding for the reg.  `index` is a zero based index which has the
// following meanings:
//
// * for a Virtual Reg, `index` is just the virtual register number.
// * for a Real Reg, `index` is the entry number in the associated
//   `RealRegUniverse`.
//
// This scheme gives us:
//
// * a compact (32-bit) representation for registers
// * fast equality tests for registers
// * ability to handle up to 2^28 (268.4 million) virtual regs per function
// * ability to handle up to 8 register classes
// * ability to handle targets with up to 256 real registers
// * ability to emit instructions containing real regs without having to
//   look up encodings in any side tables, since a real reg carries its
//   encoding
// * efficient bitsets and arrays of virtual registers, since each has a
//   zero-based index baked in
// * efficient bitsets and arrays of real registers, for the same reason
//
// This scheme makes it impossible to represent overlapping register classes,
// but that doesn't seem important.  AFAIK only ARM32 VFP/Neon has that.

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Reg {
    bits: u32,
}

const INVALID_REG: u32 = 0xffffffff;

impl Reg {
    #[inline(always)]
    pub fn is_virtual(self) -> bool {
        self.is_valid() && (self.bits & 0x8000_0000) != 0
    }
    #[inline(always)]
    pub fn is_real(self) -> bool {
        self.is_valid() && (self.bits & 0x8000_0000) == 0
    }
    pub const fn new_real(rc: RegClass, enc: u8, index: u8) -> Self {
        let n = (0 << 31) | (rc.rc_to_u32() << 28) | ((enc as u32) << 8) | ((index as u32) << 0);
        Reg { bits: n }
    }
    pub fn new_virtual(rc: RegClass, index: u32) -> Self {
        if index >= (1 << 28) {
            panic!("new_virtual(): index too large");
        }
        let n = (1 << 31) | (rc.rc_to_u32() << 28) | (index << 0);
        Reg { bits: n }
    }
    pub const fn invalid() -> Reg {
        Reg { bits: INVALID_REG }
    }
    #[inline(always)]
    pub fn is_invalid(self) -> bool {
        self.bits == INVALID_REG
    }
    #[inline(always)]
    pub fn is_valid(self) -> bool {
        !self.is_invalid()
    }
    pub fn is_virtual_or_invalid(self) -> bool {
        self.is_virtual() || self.is_invalid()
    }
    pub fn is_real_or_invalid(self) -> bool {
        self.is_real() || self.is_invalid()
    }
    #[inline(always)]
    pub fn get_class(self) -> RegClass {
        debug_assert!(self.is_valid());
        RegClass::rc_from_u32((self.bits >> 28) & 0x7)
    }
    #[inline(always)]
    pub fn get_index(self) -> usize {
        debug_assert!(self.is_valid());
        // Return type is usize because typically we will want to use the
        // result for indexing into a Vec
        if self.is_virtual() {
            (self.bits & ((1 << 28) - 1)) as usize
        } else {
            (self.bits & ((1 << 8) - 1)) as usize
        }
    }
    #[inline(always)]
    pub fn get_index_u32(self) -> u32 {
        debug_assert!(self.is_valid());
        if self.is_virtual() {
            self.bits & ((1 << 28) - 1)
        } else {
            self.bits & ((1 << 8) - 1)
        }
    }
    pub fn get_hw_encoding(self) -> u8 {
        debug_assert!(self.is_valid());
        if self.is_virtual() {
            panic!("Virtual register does not have a hardware encoding")
        } else {
            ((self.bits >> 8) & ((1 << 8) - 1)) as u8
        }
    }
    pub fn as_virtual_reg(self) -> Option<VirtualReg> {
        // Allow invalid virtual regs as well.
        if self.is_virtual_or_invalid() {
            Some(VirtualReg { reg: self })
        } else {
            None
        }
    }
    pub fn as_real_reg(self) -> Option<RealReg> {
        // Allow invalid real regs as well.
        if self.is_real_or_invalid() {
            Some(RealReg { reg: self })
        } else {
            None
        }
    }
    pub fn show_with_rru(self, univ: &RealRegUniverse) -> String {
        if self.is_real() && self.get_index() < univ.regs.len() {
            univ.regs[self.get_index()].1.clone()
        } else if self.is_valid() {
            format!("{:?}", self)
        } else {
            "rINVALID".to_string()
        }
    }
}

impl fmt::Debug for Reg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.is_valid() {
            write!(
                fmt,
                "{}{}{}",
                if self.is_virtual() { "v" } else { "r" },
                self.get_index(),
                self.get_class().short_name(),
            )
        } else {
            write!(fmt, "rINVALID")
        }
    }
}

// RealReg and VirtualReg are merely wrappers around Reg, which try to
// dynamically ensure that they are really wrapping the correct flavour of
// register.

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct RealReg {
    reg: Reg,
}
impl Reg /* !!not RealReg!! */ {
    pub fn to_real_reg(self) -> RealReg {
        if self.is_virtual() {
            panic!("Reg::to_real_reg: this is a virtual register")
        } else {
            RealReg { reg: self }
        }
    }
}
impl RealReg {
    pub fn get_class(self) -> RegClass {
        self.reg.get_class()
    }
    #[inline(always)]
    pub fn get_index(self) -> usize {
        self.reg.get_index()
    }
    pub fn get_hw_encoding(self) -> usize {
        self.reg.get_hw_encoding() as usize
    }
    #[inline(always)]
    pub fn to_reg(self) -> Reg {
        self.reg
    }
    pub fn invalid() -> RealReg {
        RealReg {
            reg: Reg::invalid(),
        }
    }
    pub fn is_valid(self) -> bool {
        self.reg.is_valid()
    }
    pub fn is_invalid(self) -> bool {
        self.reg.is_invalid()
    }
    pub fn maybe_valid(self) -> Option<RealReg> {
        if self == RealReg::invalid() {
            None
        } else {
            Some(self)
        }
    }
}
impl fmt::Debug for RealReg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.reg)
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct VirtualReg {
    reg: Reg,
}
impl Reg /* !!not VirtualReg!! */ {
    #[inline(always)]
    pub fn to_virtual_reg(self) -> VirtualReg {
        if self.is_virtual() {
            VirtualReg { reg: self }
        } else {
            panic!("Reg::to_virtual_reg: this is a real register")
        }
    }
}
impl VirtualReg {
    pub fn get_class(self) -> RegClass {
        self.reg.get_class()
    }
    #[inline(always)]
    pub fn get_index(self) -> usize {
        self.reg.get_index()
    }
    #[inline(always)]
    pub fn to_reg(self) -> Reg {
        self.reg
    }
    pub fn invalid() -> VirtualReg {
        VirtualReg {
            reg: Reg::invalid(),
        }
    }
    pub fn is_valid(self) -> bool {
        self.reg.is_valid()
    }
    pub fn is_invalid(self) -> bool {
        self.reg.is_invalid()
    }
    pub fn maybe_valid(self) -> Option<VirtualReg> {
        if self == VirtualReg::invalid() {
            None
        } else {
            Some(self)
        }
    }
}
impl fmt::Debug for VirtualReg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.reg)
    }
}

impl Reg {
    /// Apply a vreg-rreg mapping to a Reg.  This is used for registers used in
    /// a read-role.
    pub fn apply_uses<RUM: RegUsageMapper>(&mut self, mapper: &RUM) {
        self.apply(|vreg| mapper.get_use(vreg));
    }

    /// Apply a vreg-rreg mapping to a Reg.  This is used for registers used in
    /// a write-role.
    pub fn apply_defs<RUM: RegUsageMapper>(&mut self, mapper: &RUM) {
        self.apply(|vreg| mapper.get_def(vreg));
    }

    /// Apply a vreg-rreg mapping to a Reg.  This is used for registers used in
    /// a modify-role.
    pub fn apply_mods<RUM: RegUsageMapper>(&mut self, mapper: &RUM) {
        self.apply(|vreg| mapper.get_mod(vreg));
    }

    fn apply<F: Fn(VirtualReg) -> Option<RealReg>>(&mut self, f: F) {
        if let Some(vreg) = self.as_virtual_reg() {
            if let Some(rreg) = f(vreg) {
                debug_assert!(rreg.get_class() == vreg.get_class());
                *self = rreg.to_reg();
            } else {
                panic!("Reg::apply: no mapping for {:?}", self);
            }
        }
    }
}

/// A "writable register". This is a zero-cost wrapper that can be used to
/// create a distinction, at the Rust type level, between a plain "register"
/// and a "writable register".
///
/// Writable<..> can be used by the client to ensure that, internally, it only
/// generates instructions that write to registers that should be written. The
/// `InstRegUses` below, which must be implemented for every instruction,
/// requires a `Writable<Reg>` (not just `Reg`) in its `defined` and
/// `modified` sets. While we cannot hide the constructor for `Writable<..>`
/// from certain parts of the client while exposing it to others, the client
/// *can* adopt conventions to e.g. only ever call the Writable<..>
/// constructor from its central vreg-management logic, and decide that any
/// invocation of this constructor in a machine backend (for example) is an
/// error.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Writable<R> {
    reg: R,
}

/// Set of requirements for types that can be wrapped in Writable.
pub trait WritableBase:
    Copy + Clone + PartialEq + Eq + Hash + PartialOrd + Ord + fmt::Debug
{
}

impl<T> WritableBase for T where
    T: Copy + Clone + PartialEq + Eq + Hash + PartialOrd + Ord + fmt::Debug
{
}

impl<R> Writable<R> {
    /// Create a Writable<R> from an R. The client should carefully audit where
    /// it calls this constructor to ensure correctness (see `Writable<..>`
    /// struct documentation).
    #[inline(always)]
    pub const fn from_reg(reg: R) -> Writable<R> {
        Writable { reg }
    }
}

impl<R: WritableBase> Writable<R> {
    /// Get the inner Reg.
    pub fn to_reg(&self) -> R {
        self.reg
    }

    pub fn map<F, U>(&self, f: F) -> Writable<U>
    where
        F: Fn(R) -> U,
        U: WritableBase,
    {
        Writable { reg: f(self.reg) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpillSlot(u32);

impl SpillSlot {
    #[inline(always)]
    pub fn new(n: u32) -> Self {
        Self(n)
    }
    #[inline(always)]
    pub fn get(self) -> u32 {
        self.0
    }
    #[inline(always)]
    pub fn get_usize(self) -> usize {
        self.get() as usize
    }
    pub fn round_up(self, num_slots: u32) -> SpillSlot {
        assert!(num_slots > 0);
        SpillSlot::new((self.get() + num_slots - 1) / num_slots * num_slots)
    }
    pub fn inc(self, num_slots: u32) -> SpillSlot {
        SpillSlot::new(self.get() + num_slots)
    }
}

impl fmt::Debug for SpillSlot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "S{}", self.get())
    }
}

//=============================================================================
// Register uses: low level interface

// This minimal struct is visible from outside the regalloc.rs interface.  It
// is intended to be a safe wrapper around `RegVecs`, which isn't externally
// visible.  It is used to collect unsanitized reg use info from client
// instructions.
pub struct RegUsageCollector<'a> {
    pub reg_vecs: &'a mut RegVecs,
}

impl<'a> RegUsageCollector<'a> {
    pub fn new(reg_vecs: &'a mut RegVecs) -> Self {
        Self { reg_vecs }
    }
    pub fn add_use(&mut self, r: Reg) {
        self.reg_vecs.uses.push(r);
    }
    pub fn add_uses(&mut self, regs: &[Reg]) {
        self.reg_vecs.uses.extend(regs.iter());
    }
    pub fn add_def(&mut self, r: Writable<Reg>) {
        self.reg_vecs.defs.push(r.to_reg());
    }
    pub fn add_defs(&mut self, regs: &[Writable<Reg>]) {
        self.reg_vecs.defs.reserve(regs.len());
        for r in regs {
            self.reg_vecs.defs.push(r.to_reg());
        }
    }
    pub fn add_mod(&mut self, r: Writable<Reg>) {
        self.reg_vecs.mods.push(r.to_reg());
    }
    pub fn add_mods(&mut self, regs: &[Writable<Reg>]) {
        self.reg_vecs.mods.reserve(regs.len());
        for r in regs {
            self.reg_vecs.mods.push(r.to_reg());
        }
    }

    // The presence of the following two is a hack, needed to support fuzzing
    // in the test framework.  Real clients should not call them.
    pub fn get_use_def_mod_vecs_test_framework_only(&self) -> (Vec<Reg>, Vec<Reg>, Vec<Reg>) {
        (
            self.reg_vecs.uses.clone(),
            self.reg_vecs.defs.clone(),
            self.reg_vecs.mods.clone(),
        )
    }

    pub fn get_empty_reg_vecs_test_framework_only(sanitized: bool) -> RegVecs {
        RegVecs::new(sanitized)
    }
}

// Everything else is not visible outside the regalloc.rs interface.

// There is one of these per function.  Note that `defs` and `mods` lose the
// `Writable` constraint at this point.  This is for convenience of having all
// three vectors be the same type, but comes at the cost of the loss of being
// able to differentiate readonly vs read/write registers in the Rust type
// system.
#[derive(Debug)]
pub struct RegVecs {
    pub uses: Vec<Reg>,
    pub defs: Vec<Reg>,
    pub mods: Vec<Reg>,
    sanitized: bool,
}

impl RegVecs {
    pub fn new(sanitized: bool) -> Self {
        Self {
            uses: vec![],
            defs: vec![],
            mods: vec![],
            sanitized,
        }
    }
    pub fn is_sanitized(&self) -> bool {
        self.sanitized
    }
    pub fn set_sanitized(&mut self, sanitized: bool) {
        self.sanitized = sanitized;
    }
    pub fn clear(&mut self) {
        self.uses.clear();
        self.defs.clear();
        self.mods.clear();
    }
}

// There is one of these per insn, so try and keep it as compact as possible.
// I think this should fit in 16 bytes.
#[derive(Clone, Debug)]
pub struct RegVecBounds {
    // These are the group start indices in RegVecs.{uses, defs, mods}.
    pub uses_start: u32,
    pub defs_start: u32,
    pub mods_start: u32,
    // And these are the group lengths.  This does limit each instruction to
    // mentioning only 256 registers in any group, but that does not seem like a
    // problem.
    pub uses_len: u8,
    pub defs_len: u8,
    pub mods_len: u8,
}

impl RegVecBounds {
    pub fn new() -> Self {
        Self {
            uses_start: 0,
            defs_start: 0,
            mods_start: 0,
            uses_len: 0,
            defs_len: 0,
            mods_len: 0,
        }
    }
}

// This is the primary structure.  We compute just one of these for an entire
// function.
pub struct RegVecsAndBounds {
    // The three vectors of registers.  These can be arbitrarily long.
    pub vecs: RegVecs,
    // Admin info which tells us the location, for each insn, of its register
    // groups in `vecs`.
    pub bounds: TypedIxVec<InstIx, RegVecBounds>,
}

impl RegVecsAndBounds {
    pub fn new(vecs: RegVecs, bounds: TypedIxVec<InstIx, RegVecBounds>) -> Self {
        Self { vecs, bounds }
    }
    pub fn is_sanitized(&self) -> bool {
        self.vecs.sanitized
    }
    #[allow(dead_code)] // XXX for some reason, Rustc 1.43.1 thinks this is currently unused.
    pub fn num_insns(&self) -> u32 {
        self.bounds.len()
    }
}

//=============================================================================
// Register uses: convenience interface

// Some call sites want to get reg use information as three Sets.  This is a
// "convenience facility" which is easier to use but much slower than working
// with a whole-function `RegVecsAndBounds`.  It shouldn't be used on critical
// paths.
#[derive(Debug)]
pub struct RegSets {
    pub uses: Set<Reg>, // registers that are read.
    pub defs: Set<Reg>, // registers that are written.
    pub mods: Set<Reg>, // registers that are modified.
    sanitized: bool,
}

impl RegSets {
    pub fn new(sanitized: bool) -> Self {
        Self {
            uses: Set::<Reg>::empty(),
            defs: Set::<Reg>::empty(),
            mods: Set::<Reg>::empty(),
            sanitized,
        }
    }

    pub fn is_sanitized(&self) -> bool {
        self.sanitized
    }
}

impl RegVecsAndBounds {
    /* !!not RegSets!! */
    #[inline(never)]
    // Convenience function.  Try to avoid using this.
    pub fn get_reg_sets_for_iix(&self, iix: InstIx) -> RegSets {
        let bounds = &self.bounds[iix];
        let mut regsets = RegSets::new(self.vecs.sanitized);
        for i in bounds.uses_start as usize..bounds.uses_start as usize + bounds.uses_len as usize {
            regsets.uses.insert(self.vecs.uses[i]);
        }
        for i in bounds.defs_start as usize..bounds.defs_start as usize + bounds.defs_len as usize {
            regsets.defs.insert(self.vecs.defs[i]);
        }
        for i in bounds.mods_start as usize..bounds.mods_start as usize + bounds.mods_len as usize {
            regsets.mods.insert(self.vecs.mods[i]);
        }
        regsets
    }
}

//=============================================================================
// Definitions of the "real register universe".

// A "Real Register Universe" is a read-only structure that contains all
// information about real registers on a given host.  It serves several
// purposes:
//
// * defines the mapping from real register indices to the registers
//   themselves
//
// * defines the size of the initial section of that mapping that is available
//   to the register allocator for use, so that it can treat the registers
//   under its control as a zero based, contiguous array.  This is important
//   for its efficiency.
//
// * gives meaning to Set<RealReg>, which otherwise would merely be a bunch of
//   bits.

#[derive(Clone, Debug)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct RealRegUniverse {
    // The registers themselves.  All must be real registers, and all must
    // have their index number (.get_index()) equal to the array index here,
    // since this is the only place where we map index numbers to actual
    // registers.
    pub regs: Vec<(RealReg, String)>,

    // This is the size of the initial section of `regs` that is available to
    // the allocator.  It must be <= `regs`.len().
    pub allocable: usize,

    // Information about groups of allocable registers. Used to quickly address
    // only a group of allocable registers belonging to the same register class.
    // Indexes into `allocable_by_class` are RegClass values, such as
    // RegClass::F32. If the resulting entry is `None` then there are no
    // registers in that class.  Otherwise the value is a `RegClassInfo`, which
    // provides a register range and possibly information about fixed uses.
    pub allocable_by_class: [Option<RegClassInfo>; NUM_REG_CLASSES],
}

/// Information about a single register class in the `RealRegUniverse`.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct RegClassInfo {
    // Range of allocatable registers in this register class, in terms of
    // register indices.
    //
    // A range (first, last) specifies the range of entries in
    // `RealRegUniverse.regs` corresponding to that class.  The range includes
    // both `first` and `last`.
    //
    // In all cases, `last` must be < `RealRegUniverse.allocable`.  In other
    // words, all ranges together in `allocable_by_class` must describe only the
    // allocable prefix of `regs`.
    //
    // For example, let's say
    //    allocable_by_class[RegClass::F32] ==
    //      Some(RegClassInfo { first: 10, last: 14, .. })
    // Then regs[10], regs[11], regs[12], regs[13], and regs[14] give all
    // registers of register class RegClass::F32.
    //
    // The effect of the above is that registers in `regs` must form
    // contiguous groups. This is checked by RealRegUniverse::check_is_sane().
    pub first: usize,
    pub last: usize,

    // A register, if any, that is *guaranteed* not to be used as a fixed use
    // in any code, and so that the register allocator can statically reserve
    // for its own use as a temporary. Some register allocators may need such
    // a register for various maneuvers, for example a spillslot-to-spillslot
    // move when no (other) registers are free.
    pub suggested_scratch: Option<usize>,
}

impl RealRegUniverse {
    /// Show it in a pretty way.
    pub fn show(&self) -> Vec<String> {
        let mut res = vec![];
        // Show the allocables
        for class_num in 0..NUM_REG_CLASSES {
            let class_info = match &self.allocable_by_class[class_num] {
                None => continue,
                Some(info) => info,
            };
            let class = RegClass::rc_from_u32(class_num as u32);
            let mut class_str = "class ".to_string()
                + &class.long_name().to_string()
                + &"(".to_string()
                + &class.short_name().to_string()
                + &") at ".to_string();
            class_str = class_str + &format!("[{} .. {}]: ", class_info.first, class_info.last);
            for ix in class_info.first..=class_info.last {
                class_str = class_str + &self.regs[ix].1;
                if let Some(suggested_ix) = class_info.suggested_scratch {
                    if ix == suggested_ix {
                        class_str = class_str + "*";
                    }
                }
                class_str = class_str + " ";
            }
            res.push(class_str);
        }
        // And the non-allocables
        if self.allocable < self.regs.len() {
            let mut stragglers = format!(
                "not allocable at [{} .. {}]: ",
                self.allocable,
                self.regs.len() - 1
            );
            for ix in self.allocable..self.regs.len() {
                stragglers = stragglers + &self.regs[ix].1 + &" ".to_string();
            }
            res.push(stragglers);
        }
        res
    }

    /// Check that the given universe satisfies various invariants, and panic
    /// if not.  All the invariants are important.
    pub fn check_is_sane(&self) {
        let regs_len = self.regs.len();
        let regs_allocable = self.allocable;
        // The universe must contain at most 256 registers.  That's because
        // `Reg` only has an 8-bit index value field, so if the universe
        // contained more than 256 registers, we'd never be able to index into
        // entries 256 and above.  This is no limitation in practice since all
        // targets we're interested in contain (many) fewer than 256 regs in
        // total.
        let mut ok = regs_len <= 256;
        // The number of allocable registers must not exceed the number of
        // `regs` presented.  In general it will be less, since the universe
        // will list some registers (stack pointer, etc) which are not
        // available for allocation.
        if ok {
            ok = regs_allocable <= regs_len;
        }
        // All registers must have an index value which points back at the
        // `regs` slot they are in.  Also they really must be real regs.
        if ok {
            for i in 0..regs_len {
                let (reg, _name) = &self.regs[i];
                if ok && (reg.to_reg().is_virtual() || reg.get_index() != i) {
                    ok = false;
                }
            }
        }
        // The allocatable regclass groupings defined by `allocable_first` and
        // `allocable_last` must be contiguous.
        if ok {
            let mut regclass_used = [false; NUM_REG_CLASSES];
            for rc in 0..NUM_REG_CLASSES {
                regclass_used[rc] = false;
            }
            for i in 0..regs_allocable {
                let (reg, _name) = &self.regs[i];
                let rc = reg.get_class().rc_to_u32() as usize;
                regclass_used[rc] = true;
            }
            // Scan forward through each grouping, checking that the listed
            // registers really are of the claimed class.  Also count the
            // total number visited.  This seems a fairly reliable way to
            // ensure that the groupings cover all allocated registers exactly
            // once, and that all classes are contiguous groups.
            let mut regs_visited = 0;
            for rc in 0..NUM_REG_CLASSES {
                match &self.allocable_by_class[rc] {
                    &None => {
                        if regclass_used[rc] {
                            ok = false;
                        }
                    }
                    &Some(RegClassInfo {
                        first,
                        last,
                        suggested_scratch,
                    }) => {
                        if !regclass_used[rc] {
                            ok = false;
                        }
                        if ok {
                            for i in first..last + 1 {
                                let (reg, _name) = &self.regs[i];
                                if ok && RegClass::rc_from_u32(rc as u32) != reg.get_class() {
                                    ok = false;
                                }
                                regs_visited += 1;
                            }
                        }
                        if ok {
                            if let Some(s) = suggested_scratch {
                                if s < first || s > last {
                                    ok = false;
                                }
                            }
                        }
                    }
                }
            }
            if ok && regs_visited != regs_allocable {
                ok = false;
            }
        }
        // So finally ..
        if !ok {
            panic!("RealRegUniverse::check_is_sane: invalid RealRegUniverse");
        }
    }
}

//=============================================================================
// Representing and printing of live range fragments.

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord)]
// There are four "points" within an instruction that are of interest, and
// these have a total ordering: R < U < D < S.  They are:
//
// * R(eload): this is where any reload insns for the insn itself are
//   considered to live.
//
// * U(se): this is where the insn is considered to use values from those of
//   its register operands that appear in a Read or Modify role.
//
// * D(ef): this is where the insn is considered to define new values for
//   those of its register operands that appear in a Write or Modify role.
//
// * S(pill): this is where any spill insns for the insn itself are considered
//   to live.
//
// Instructions in the incoming Func may only exist at the U and D points,
// and so their associated live range fragments will only mention the U and D
// points.  However, when adding spill code, we need a way to represent live
// ranges involving the added spill and reload insns, in which case R and S
// come into play:
//
// * A reload for instruction i is considered to be live from i.R to i.U.
//
// * A spill for instruction i is considered to be live from i.D to i.S.

pub enum Point {
    // The values here are important.  Don't change them.
    Reload = 0,
    Use = 1,
    Def = 2,
    Spill = 3,
}

impl Point {
    #[inline(always)]
    pub fn is_reload(self) -> bool {
        match self {
            Point::Reload => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn is_use(self) -> bool {
        match self {
            Point::Use => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn is_def(self) -> bool {
        match self {
            Point::Def => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn is_spill(self) -> bool {
        match self {
            Point::Spill => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn is_use_or_def(self) -> bool {
        self.is_use() || self.is_def()
    }
}

impl PartialOrd for Point {
    // In short .. R < U < D < S.  This is probably what would be #derive'd
    // anyway, but we need to be sure.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self as u32).partial_cmp(&(*other as u32))
    }
}

// See comments below on `RangeFrag` for the meaning of `InstPoint`.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstPoint {
    /// This is conceptually:
    ///   pub iix: InstIx,
    ///   pub pt: Point,
    ///
    /// but packed into a single 32 bit word, so as
    /// (1) to ensure it is only 32 bits (and hence to guarantee that `RangeFrag`
    ///     is 64 bits), and
    /// (2) to make it possible to implement `PartialOrd` using `PartialOrd`
    ///     directly on 32 bit words (and hence we let it be derived).
    ///
    /// This has the format:
    ///    InstIx as bits 31:2,  Point as bits 1:0.
    ///
    /// It does give the slight limitation that all InstIxs must be < 2^30, but
    /// that's hardly a big deal: the analysis module rejects any input with 2^24
    /// or more Insns.
    ///
    /// Do not access this directly:
    bits: u32,
}

impl InstPoint {
    #[inline(always)]
    pub fn new(iix: InstIx, pt: Point) -> Self {
        let iix_n = iix.get();
        assert!(iix_n < 0x4000_0000u32);
        let pt_n = pt as u32;
        InstPoint {
            bits: (iix_n << 2) | pt_n,
        }
    }
    #[inline(always)]
    pub fn iix(self) -> InstIx {
        InstIx::new(self.bits >> 2)
    }
    #[inline(always)]
    pub fn pt(self) -> Point {
        match self.bits & 3 {
            0 => Point::Reload,
            1 => Point::Use,
            2 => Point::Def,
            3 => Point::Spill,
            // This can never happen, but rustc doesn't seem to know that.
            _ => panic!("InstPt::pt: unreachable case"),
        }
    }
    #[inline(always)]
    pub fn set_iix(&mut self, iix: InstIx) {
        let iix_n = iix.get();
        assert!(iix_n < 0x4000_0000u32);
        self.bits = (iix_n << 2) | (self.bits & 3);
    }
    #[inline(always)]
    pub fn set_pt(&mut self, pt: Point) {
        self.bits = (self.bits & 0xFFFF_FFFCu32) | pt as u32;
    }
    #[inline(always)]
    pub fn new_reload(iix: InstIx) -> Self {
        InstPoint::new(iix, Point::Reload)
    }
    #[inline(always)]
    pub fn new_use(iix: InstIx) -> Self {
        InstPoint::new(iix, Point::Use)
    }
    #[inline(always)]
    pub fn new_def(iix: InstIx) -> Self {
        InstPoint::new(iix, Point::Def)
    }
    #[inline(always)]
    pub fn new_spill(iix: InstIx) -> Self {
        InstPoint::new(iix, Point::Spill)
    }
    #[inline(always)]
    pub fn invalid_value() -> Self {
        Self {
            bits: 0xFFFF_FFFFu32,
        }
    }
    #[inline(always)]
    pub fn max_value() -> Self {
        Self {
            bits: 0xFFFF_FFFFu32,
        }
    }
    #[inline(always)]
    pub fn min_value() -> Self {
        Self { bits: 0u32 }
    }
}

impl fmt::Debug for InstPoint {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{:?}{}",
            self.iix(),
            match self.pt() {
                Point::Reload => ".r",
                Point::Use => ".u",
                Point::Def => ".d",
                Point::Spill => ".s",
            }
        )
    }
}

//=============================================================================
// Live Range Fragments, and their metrics

// A Live Range Fragment (RangeFrag) describes a consecutive sequence of one or
// more instructions, in which a Reg is "live".  The sequence must exist
// entirely inside only one basic block.
//
// However, merely indicating the start and end instruction numbers isn't
// enough: we must also include a "Use or Def" indication.  These indicate two
// different "points" within each instruction: the Use position, where
// incoming registers are read, and the Def position, where outgoing registers
// are written.  The Use position is considered to come before the Def
// position, as described for `Point` above.
//
// When we come to generate spill/restore live ranges, Point::S and Point::R
// also come into play.  Live ranges (and hence, RangeFrags) that do not perform
// spills or restores should not use either of Point::S or Point::R.
//
// The set of positions denoted by
//
//    {0 .. #insns-1} x {Reload point, Use point, Def point, Spill point}
//
// is exactly the set of positions that we need to keep track of when mapping
// live ranges to registers.  This the reason for the type InstPoint.  Note
// that InstPoint values have a total ordering, at least within a single basic
// block: the insn number is used as the primary key, and the Point part is
// the secondary key, with Reload < Use < Def < Spill.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RangeFrag {
    pub first: InstPoint,
    pub last: InstPoint,
}

impl fmt::Debug for RangeFrag {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "(RF: {:?}-{:?})", self.first, self.last)
    }
}

impl RangeFrag {
    #[allow(dead_code)] // XXX for some reason, Rustc 1.43.1 thinks this is unused.
    pub fn new(first: InstPoint, last: InstPoint) -> Self {
        debug_assert!(first <= last);
        RangeFrag { first, last }
    }

    pub fn invalid_value() -> Self {
        Self {
            first: InstPoint::invalid_value(),
            last: InstPoint::invalid_value(),
        }
    }

    pub fn new_with_metrics<F: Function>(
        f: &F,
        bix: BlockIx,
        first: InstPoint,
        last: InstPoint,
        count: u16,
    ) -> (Self, RangeFragMetrics) {
        debug_assert!(f.block_insns(bix).len() >= 1);
        debug_assert!(f.block_insns(bix).contains(first.iix()));
        debug_assert!(f.block_insns(bix).contains(last.iix()));
        debug_assert!(first <= last);
        if first == last {
            debug_assert!(count == 1);
        }
        let first_iix_in_block = f.block_insns(bix).first();
        let last_iix_in_block = f.block_insns(bix).last();
        let first_pt_in_block = InstPoint::new_use(first_iix_in_block);
        let last_pt_in_block = InstPoint::new_def(last_iix_in_block);
        let kind = match (first == first_pt_in_block, last == last_pt_in_block) {
            (false, false) => RangeFragKind::Local,
            (false, true) => RangeFragKind::LiveOut,
            (true, false) => RangeFragKind::LiveIn,
            (true, true) => RangeFragKind::Thru,
        };
        (
            RangeFrag { first, last },
            RangeFragMetrics { bix, kind, count },
        )
    }
}

// Comparison of RangeFrags.  They form a partial order.

pub fn cmp_range_frags(f1: &RangeFrag, f2: &RangeFrag) -> Option<Ordering> {
    if f1.last < f2.first {
        return Some(Ordering::Less);
    }
    if f1.first > f2.last {
        return Some(Ordering::Greater);
    }
    if f1.first == f2.first && f1.last == f2.last {
        return Some(Ordering::Equal);
    }
    None
}

impl RangeFrag {
    pub fn contains(&self, ipt: &InstPoint) -> bool {
        self.first <= *ipt && *ipt <= self.last
    }
}

// A handy summary hint for a RangeFrag.  Note that none of these are correct
// if the RangeFrag has been extended so as to cover multiple basic blocks.
// But that ("RangeFrag compression") is something done locally within each
// algorithm (BT and LSRA).  The analysis-phase output will not include any
// such compressed RangeFrags.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RangeFragKind {
    Local,   // Fragment exists entirely inside one block
    LiveIn,  // Fragment is live in to a block, but ends inside it
    LiveOut, // Fragment is live out of a block, but starts inside it
    Thru,    // Fragment is live through the block (starts and ends outside it)
}

impl fmt::Debug for RangeFragKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RangeFragKind::Local => write!(fmt, "Local"),
            RangeFragKind::LiveIn => write!(fmt, "LiveIn"),
            RangeFragKind::LiveOut => write!(fmt, "LiveOut"),
            RangeFragKind::Thru => write!(fmt, "Thru"),
        }
    }
}

// `RangeFrags` resulting from the initial analysis phase (analysis_data_flow.rs)
// exist only within single basic blocks, and therefore have some associated
// metrics, held by `RangeFragMetrics`:
//
// * a `count` field, which is a u16 indicating how often the associated storage
//   unit (Reg) is mentioned inside the RangeFrag.  It is assumed that the RangeFrag
//   is associated with some Reg.  If not, the `count` field is meaningless.  This
//   field has no effect on the correctness of the resulting allocation.  It is used
//   however in the estimation of `VirtualRange` spill costs, which are important
//   for prioritising which `VirtualRange`s get assigned a register vs which have
//   to be spilled.
//
// * `bix` field, which indicates which `Block` the fragment exists in.  This
//   field is actually redundant, since the containing `Block` can be inferred,
//   laboriously, from the associated `RangeFrag`s `first` and `last` fields,
//   providing you have an `InstIxToBlockIx` mapping table to hand.  It is included
//   here for convenience.
//
// * `kind` is another convenience field, indicating how the range is included
//   within its owning block.
//
// The analysis phase (fn `deref_and_compress_sorted_range_frag_ixs`)
// compresses ranges and as a result breaks the invariant that a `RangeFrag`
// exists only within a single `Block`.  For a `RangeFrag` spanning multiple
// `Block`s, all three `RangeFragMetric` fields are meaningless.  This is the
// reason for separating `RangeFrag` and `RangeFragMetrics` -- so that it is
// possible to merge `RangeFrag`s without being forced to create fake values
// for the metrics fields.
#[derive(Clone, PartialEq)]
pub struct RangeFragMetrics {
    pub bix: BlockIx,
    pub kind: RangeFragKind,
    pub count: u16,
}

impl fmt::Debug for RangeFragMetrics {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "(RFM: {:?}, count={}, {:?})",
            self.kind, self.count, self.bix
        )
    }
}

//=============================================================================
// Vectors of RangeFragIxs, sorted so that the associated RangeFrags are in
// ascending order, per their InstPoint fields.  The associated RangeFrags may
// not overlap.
//
// The "fragment environment" (usually called "frag_env"), to which the
// RangeFragIxs refer, is not stored here.

#[derive(Clone)]
pub(crate) struct SortedRangeFragIxs(SmallVec<[RangeFragIx; 4]>);

impl fmt::Debug for SortedRangeFragIxs {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl Deref for SortedRangeFragIxs {
    type Target = SmallVec<[RangeFragIx; 4]>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SortedRangeFragIxs {
    fn check(&self, fenv: &TypedIxVec<RangeFragIx, RangeFrag>) {
        for i in 1..self.0.len() {
            let prev_frag = &fenv[self.0[i - 1]];
            let this_frag = &fenv[self.0[i]];
            if cmp_range_frags(prev_frag, this_frag) != Some(Ordering::Less) {
                panic!("SortedRangeFragIxs::check: vector not ok");
            }
        }
    }

    fn sort(&mut self, fenv: &TypedIxVec<RangeFragIx, RangeFrag>) {
        self.0.sort_unstable_by(|fix_a, fix_b| {
            match cmp_range_frags(&fenv[*fix_a], &fenv[*fix_b]) {
                Some(Ordering::Less) => Ordering::Less,
                Some(Ordering::Greater) => Ordering::Greater,
                Some(Ordering::Equal) | None => {
                    panic!("SortedRangeFragIxs::sort: overlapping Frags!")
                }
            }
        });
    }

    pub(crate) fn new(
        frag_ixs: SmallVec<[RangeFragIx; 4]>,
        fenv: &TypedIxVec<RangeFragIx, RangeFrag>,
    ) -> Self {
        let mut res = Self(frag_ixs);
        // Check the source is ordered, and clone (or sort it).
        res.sort(fenv);
        res.check(fenv);
        res
    }

    pub(crate) fn unit(fix: RangeFragIx, fenv: &TypedIxVec<RangeFragIx, RangeFrag>) -> Self {
        let mut res = Self(SmallVec::<[RangeFragIx; 4]>::new());
        res.0.push(fix);
        res.check(fenv);
        res
    }

    /// Does this sorted list of range fragments contain the given instruction point?
    pub(crate) fn contains_pt(
        &self,
        fenv: &TypedIxVec<RangeFragIx, RangeFrag>,
        pt: InstPoint,
    ) -> bool {
        self.0
            .binary_search_by(|&ix| {
                let frag = &fenv[ix];
                if pt < frag.first {
                    Ordering::Greater
                } else if pt >= frag.first && pt <= frag.last {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            })
            .is_ok()
    }
}

/// Vectors of RangeFrags, sorted so that they are in ascending order, per
/// their InstPoint fields.  The RangeFrags may not overlap.
#[derive(Clone)]
pub(crate) struct SortedRangeFrags(SmallVec<[RangeFrag; 4]>);

impl fmt::Debug for SortedRangeFrags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl Deref for SortedRangeFrags {
    type Target = SmallVec<[RangeFrag; 4]>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SortedRangeFrags {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SortedRangeFrags {
    pub(crate) fn unit(frag: RangeFrag) -> Self {
        let mut res = Self(SmallVec::<[RangeFrag; 4]>::new());
        res.0.push(frag);
        res
    }

    pub(crate) fn empty() -> Self {
        Self(SmallVec::<[RangeFrag; 4]>::new())
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        // Since both vectors are sorted and individually non-overlapping, we can establish that
        // they are mutually non-overlapping by walking them simultaneously and checking, at each
        // step, that there is a unique "next lowest" frag available.
        let frags1 = &self.0;
        let frags2 = &other.0;
        let n1 = frags1.len();
        let n2 = frags2.len();
        let mut c1 = 0;
        let mut c2 = 0;
        loop {
            if c1 >= n1 || c2 >= n2 {
                // We made it to the end of one (or both) vectors without finding any conflicts: no
                // overlap.
                return false;
            }
            let f1 = &frags1[c1];
            let f2 = &frags2[c2];
            match cmp_range_frags(f1, f2) {
                Some(Ordering::Less) => c1 += 1,
                Some(Ordering::Greater) => c2 += 1,
                _ => {
                    // There's no unique "next frag" -- either they are identical, or they overlap.
                    // So we're done.
                    return true;
                }
            }
        }
    }

    /// Does this sorted list of range fragments contain the given instruction point?
    pub(crate) fn contains_pt(&self, pt: InstPoint) -> bool {
        self.0
            .binary_search_by(|frag| {
                if pt < frag.first {
                    Ordering::Greater
                } else if pt >= frag.first && pt <= frag.last {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            })
            .is_ok()
    }
}

//=============================================================================
// Representing spill costs.  A spill cost can either be infinite, in which
// case the associated VirtualRange may not be spilled, because it's already a
// spill/reload range.  Or it can be finite, in which case it must be a 32-bit
// floating point number, which is (in the IEEE754 meaning of the terms)
// non-infinite, non-NaN and it must be non negative.  In fact it's
// meaningless for a VLR to have a zero spill cost (how could that really be
// the case?) but we allow it here for convenience.

#[derive(Copy, Clone)]
pub enum SpillCost {
    Infinite,    // Infinite, positive
    Finite(f32), // Finite, non-negative
}

impl fmt::Debug for SpillCost {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SpillCost::Infinite => write!(fmt, "INFINITY"),
            SpillCost::Finite(c) => write!(fmt, "{:<.3}", c),
        }
    }
}

impl SpillCost {
    #[inline(always)]
    pub fn zero() -> Self {
        SpillCost::Finite(0.0)
    }
    #[inline(always)]
    pub fn infinite() -> Self {
        SpillCost::Infinite
    }
    #[inline(always)]
    pub fn finite(cost: f32) -> Self {
        // "`is_normal` returns true if the number is neither zero, infinite,
        // subnormal, or NaN."
        assert!(cost.is_normal() || cost == 0.0);
        // And also it can't be negative.
        assert!(cost >= 0.0);
        // Somewhat arbitrarily ..
        assert!(cost < 1e18);
        SpillCost::Finite(cost)
    }
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        match self {
            SpillCost::Infinite => false,
            SpillCost::Finite(c) => *c == 0.0,
        }
    }
    #[inline(always)]
    pub fn is_infinite(&self) -> bool {
        match self {
            SpillCost::Infinite => true,
            SpillCost::Finite(_) => false,
        }
    }
    #[inline(always)]
    pub fn is_finite(&self) -> bool {
        !self.is_infinite()
    }
    #[inline(always)]
    pub fn is_less_than(&self, other: &Self) -> bool {
        match (self, other) {
            // Dubious .. both are infinity
            (SpillCost::Infinite, SpillCost::Infinite) => false,
            // finite < inf
            (SpillCost::Finite(_), SpillCost::Infinite) => true,
            // inf is not < finite
            (SpillCost::Infinite, SpillCost::Finite(_)) => false,
            // straightforward
            (SpillCost::Finite(c1), SpillCost::Finite(c2)) => c1 < c2,
        }
    }
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        match (*self, other) {
            (SpillCost::Finite(c1), SpillCost::Finite(c2)) => {
                // The 10^18 limit above gives us a lot of headroom here, since max
                // f32 is around 10^37.
                *self = SpillCost::Finite(c1 + c2);
            }
            (_, _) => {
                // All other cases produce an infinity.
                *self = SpillCost::Infinite;
            }
        }
    }
}

//=============================================================================
// Representing and printing live ranges.  These are represented by two
// different but closely related types, RealRange and VirtualRange.

// RealRanges are live ranges for real regs (RealRegs).  VirtualRanges are
// live ranges for virtual regs (VirtualRegs).  VirtualRanges are the
// fundamental unit of allocation.
//
// A RealRange pairs a RealReg with a vector of RangeFragIxs in which it is
// live.  The RangeFragIxs are indices into some vector of RangeFrags (a
// "fragment environment", 'fenv'), which is not specified here.  They are
// sorted so as to give ascending order to the RangeFrags which they refer to.
//
// A VirtualRange pairs a VirtualReg with a vector of RangeFrags in which it
// is live.  Same scheme as for a RealRange, except it avoids the overhead of
// having to indirect into the fragment environment.
//
// VirtualRanges also contain metrics:
//
// * `size` is the number of instructions in total spanned by the LR.  It must
//   not be zero.
//
// * `total cost` is an abstractified measure of the cost of the LR.  Each
//   basic block in which the range exists gives a contribution to the `total
//   cost`, which is the number of times the register is mentioned in this
//   block, multiplied by the estimated execution frequency for the block.
//
// * `spill_cost` is an abstractified measure of the cost of spilling the LR,
//   and is the `total cost` divided by the `size`. The only constraint
//   (w.r.t. correctness) is that normal LRs have a `Some` value, whilst
//   `None` is reserved for live ranges created for spills and reloads and
//   interpreted to mean "infinity".  This is needed to guarantee that
//   allocation can always succeed in the worst case, in which all of the
//   original live ranges of the program are spilled.
//
// RealRanges don't carry any metrics info since we are not trying to allocate
// them.  We merely need to work around them.
//
// I find it helpful to think of a live range, both RealRange and
// VirtualRange, as a "renaming equivalence class".  That is, if you rename
// `reg` at some point inside `sorted_frags`, then you must rename *all*
// occurrences of `reg` inside `sorted_frags`, since otherwise the program will
// no longer work.

#[derive(Clone)]
pub struct RealRange {
    pub rreg: RealReg,
    pub(crate) sorted_frags: SortedRangeFragIxs,
    pub is_ref: bool,
}

impl fmt::Debug for RealRange {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "(RR: {:?}{}, {:?})",
            self.rreg,
            if self.is_ref { " REF" } else { "" },
            self.sorted_frags
        )
    }
}

impl RealRange {
    pub fn show_with_rru(&self, univ: &RealRegUniverse) -> String {
        format!(
            "(RR: {}{}, {:?})",
            self.rreg.to_reg().show_with_rru(univ),
            if self.is_ref { " REF" } else { "" },
            self.sorted_frags
        )
    }
}

// VirtualRanges are live ranges for virtual regs (VirtualRegs).  This does
// carry metrics info and also the identity of the RealReg to which it
// eventually got allocated.  (Or in the backtracking allocator, the identity
// of the RealReg to which it is *currently* assigned; that may be undone at
// some later point.)

#[derive(Clone)]
pub struct VirtualRange {
    pub(crate) vreg: VirtualReg,
    pub(crate) rreg: Option<RealReg>,
    pub(crate) sorted_frags: SortedRangeFrags,
    pub(crate) is_ref: bool,
    pub(crate) size: u16,
    pub(crate) total_cost: u32,
    pub(crate) spill_cost: SpillCost, // == total_cost / size
}

impl VirtualRange {
    pub fn overlaps(&self, other: &Self) -> bool {
        self.sorted_frags.overlaps(&other.sorted_frags)
    }
}

impl fmt::Debug for VirtualRange {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "(VR: {:?}{},",
            self.vreg,
            if self.is_ref { " REF" } else { "" }
        )?;
        if self.rreg.is_some() {
            write!(fmt, " -> {:?}", self.rreg.unwrap())?;
        }
        write!(
            fmt,
            " sz={}, tc={}, sc={:?}, {:?})",
            self.size, self.total_cost, self.spill_cost, self.sorted_frags
        )
    }
}

//=============================================================================
// Some auxiliary/miscellaneous data structures that are useful: RegToRangesMaps

/// Mappings from RealRegs and VirtualRegs to the sets of RealRanges and VirtualRanges that belong
/// to them.  These are needed for BT's coalescing analysis and for the dataflow analysis that
/// supports reftype handling.
pub(crate) struct RegToRangesMaps {
    /// This maps RealReg indices to the set of RealRangeIxs for that RealReg. Valid indices are
    /// real register indices for all non-sanitised real regs; that is,
    /// 0..RealRegUniverse::allocable. The Vecs of RealRangeIxs are duplicate-free.
    ///
    /// The SmallVec capacity of 6 was chosen after quite some profiling, of Cranelift/x64/newBE
    /// compiling ZenGarden.wasm -- a huge input, with many relatively small functions. Profiling
    /// was performed in August 2020, using Valgrind/DHAT.
    pub(crate) rreg_to_rlrs_map: Vec</*real reg ix, */ SmallVec<[RealRangeIx; 6]>>,

    /// This maps VirtualReg indices to the set of VirtualRangeIxs for that VirtualReg. Valid
    /// indices are 0..Function::get_num_vregs(). For functions mostly translated from SSA,
    /// most VirtualRegs will have just one VirtualRange, and there are a lot of VirtualRegs in
    /// general. So SmallVec is a definite benefit here.
    pub(crate) vreg_to_vlrs_map: Vec</*virtual reg ix, */ SmallVec<[VirtualRangeIx; 3]>>,

    /// As an optimisation heuristic for BT's coalescing analysis, these indicate which
    /// real/virtual registers have "many" `RangeFrag`s in their live ranges. For some definition
    /// of "many", as defined by the `many_frags_thresh` field. This is not important for overall
    /// allocation result or correctness: it merely allows the coalescing analysis to switch
    /// between two search strategies, one of which is fast for regs with few `RangeFrag`s (the
    /// vast majority) and the other of which has better asymptotic behaviour for regs with many
    /// `RangeFrag`s (in order to keep out of trouble on some pathological inputs).  These vectors
    /// are duplicate-free but the elements may be in an arbitrary order.
    pub(crate) rregs_with_many_frags: Vec<u32 /*RealReg index*/>,
    /// Same as above, for virtual registers.
    pub(crate) vregs_with_many_frags: Vec<u32 /*VirtualReg index*/>,

    /// This indicates what the threshold is actually set to.  A frag will be in
    /// `{r,v}regs_with_many_frags` if it has `many_frags_thresh` or more RangeFrags.
    pub(crate) many_frags_thresh: usize,
}

/// `MoveInfoElem` holds info about the two registers connected a move: the source and destination
/// of the move, the instruction performing the move, and the estimated execution frequency of the
/// containing block.
pub(crate) struct MoveInfoElem {
    pub(crate) dst: Reg,
    pub(crate) src: Reg,
    pub(crate) iix: InstIx,
    pub(crate) est_freq: u32,
}

/// Vector of `MoveInfoElem`, presented in no particular order, but duplicate-free in that each
/// move instruction will be listed only once.
pub(crate) struct MoveInfo(Vec<MoveInfoElem>);

impl MoveInfo {
    pub(crate) fn new(move_info: Vec<MoveInfoElem>) -> Self {
        Self(move_info)
    }
}

impl Deref for MoveInfo {
    type Target = Vec<MoveInfoElem>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Something that can be either a VirtualRangeIx or a RealRangeIx, whilst still being 32 bits
// (by stealing one bit from those spaces).  Note that the resulting thing no longer denotes a
// contiguous index space, and so it has a name that indicates it is an identifier rather than
// an index.

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct RangeId {
    // 1 X--(31)--X is a RealRangeIx with value X--(31)--X
    // 0 X--(31)--X is a VirtualRangeIx with value X--(31)--X
    bits: u32,
}

impl RangeId {
    #[inline(always)]
    pub fn new_real(rlrix: RealRangeIx) -> Self {
        let n = rlrix.get();
        assert!(n <= 0x7FFF_FFFF);
        Self {
            bits: n | 0x8000_0000,
        }
    }
    #[inline(always)]
    pub fn new_virtual(vlrix: VirtualRangeIx) -> Self {
        let n = vlrix.get();
        assert!(n <= 0x7FFF_FFFF);
        Self { bits: n }
    }
    #[inline(always)]
    pub fn is_real(self) -> bool {
        self.bits & 0x8000_0000 != 0
    }
    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_virtual(self) -> bool {
        self.bits & 0x8000_0000 == 0
    }
    #[inline(always)]
    pub fn to_real(self) -> RealRangeIx {
        assert!(self.bits & 0x8000_0000 != 0);
        RealRangeIx::new(self.bits & 0x7FFF_FFFF)
    }
    #[inline(always)]
    pub fn to_virtual(self) -> VirtualRangeIx {
        assert!(self.bits & 0x8000_0000 == 0);
        VirtualRangeIx::new(self.bits)
    }
    #[inline(always)]
    pub fn invalid_value() -> Self {
        // Real, and inplausibly huge
        Self { bits: 0xFFFF_FFFF }
    }
}

impl fmt::Debug for RangeId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.is_real() {
            self.to_real().fmt(fmt)
        } else {
            self.to_virtual().fmt(fmt)
        }
    }
}

//=============================================================================
// Test cases

// sewardj 2020Mar04: these are commented out for now, as they no longer
// compile.  They may be useful later though, once BT acquires an interval
// tree implementation for its CommitmentMap.

/*
#[test]
fn test_sorted_frag_ranges() {
  // Create a RangeFrag and RangeFragIx from two InstPoints.
  fn gen_fix(
    fenv: &mut TypedIxVec<RangeFragIx, RangeFrag>, first: InstPoint,
    last: InstPoint,
  ) -> RangeFragIx {
    assert!(first <= last);
    let res = RangeFragIx::new(fenv.len() as u32);
    let frag = RangeFrag {
      bix: BlockIx::new(123),
      kind: RangeFragKind::Local,
      first,
      last,
      count: 0,
    };
    fenv.push(frag);
    res
  }

  fn get_range_frag(
    fenv: &TypedIxVec<RangeFragIx, RangeFrag>, fix: RangeFragIx,
  ) -> &RangeFrag {
    &fenv[fix]
  }

  // Structural equality, at least.  Not equality in the sense of
  // deferencing the contained RangeFragIxes.
  fn sorted_range_eq(
    fixs1: &SortedRangeFragIxs, fixs2: &SortedRangeFragIxs,
  ) -> bool {
    if fixs1.frag_ixs.len() != fixs2.frag_ixs.len() {
      return false;
    }
    for (mf1, mf2) in fixs1.frag_ixs.iter().zip(&fixs2.frag_ixs) {
      if mf1 != mf2 {
        return false;
      }
    }
    true
  }

  let iix3 = InstIx::new(3);
  let iix4 = InstIx::new(4);
  let iix5 = InstIx::new(5);
  let iix6 = InstIx::new(6);
  let iix7 = InstIx::new(7);
  let iix10 = InstIx::new(10);
  let iix12 = InstIx::new(12);

  let fp_3u = InstPoint::new_use(iix3);
  let fp_3d = InstPoint::new_def(iix3);

  let fp_4u = InstPoint::new_use(iix4);

  let fp_5u = InstPoint::new_use(iix5);
  let fp_5d = InstPoint::new_def(iix5);

  let fp_6u = InstPoint::new_use(iix6);
  let fp_6d = InstPoint::new_def(iix6);

  let fp_7u = InstPoint::new_use(iix7);
  let fp_7d = InstPoint::new_def(iix7);

  let fp_10u = InstPoint::new_use(iix10);
  let fp_12u = InstPoint::new_use(iix12);

  let mut fenv = TypedIxVec::<RangeFragIx, RangeFrag>::new();

  let fix_3u = gen_fix(&mut fenv, fp_3u, fp_3u);
  let fix_3d = gen_fix(&mut fenv, fp_3d, fp_3d);
  let fix_4u = gen_fix(&mut fenv, fp_4u, fp_4u);
  let fix_3u_5u = gen_fix(&mut fenv, fp_3u, fp_5u);
  let fix_3d_5d = gen_fix(&mut fenv, fp_3d, fp_5d);
  let fix_3d_5u = gen_fix(&mut fenv, fp_3d, fp_5u);
  let fix_3u_5d = gen_fix(&mut fenv, fp_3u, fp_5d);
  let fix_6u_6d = gen_fix(&mut fenv, fp_6u, fp_6d);
  let fix_7u_7d = gen_fix(&mut fenv, fp_7u, fp_7d);
  let fix_10u = gen_fix(&mut fenv, fp_10u, fp_10u);
  let fix_12u = gen_fix(&mut fenv, fp_12u, fp_12u);

  // Boundary checks for point ranges, 3u vs 3d
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3u),
      get_range_frag(&fenv, fix_3u)
    ) == Some(Ordering::Equal)
  );
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3u),
      get_range_frag(&fenv, fix_3d)
    ) == Some(Ordering::Less)
  );
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3d),
      get_range_frag(&fenv, fix_3u)
    ) == Some(Ordering::Greater)
  );

  // Boundary checks for point ranges, 3d vs 4u
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3d),
      get_range_frag(&fenv, fix_3d)
    ) == Some(Ordering::Equal)
  );
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3d),
      get_range_frag(&fenv, fix_4u)
    ) == Some(Ordering::Less)
  );
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_4u),
      get_range_frag(&fenv, fix_3d)
    ) == Some(Ordering::Greater)
  );

  // Partially overlapping
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3d_5d),
      get_range_frag(&fenv, fix_3u_5u)
    ) == None
  );
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3u_5u),
      get_range_frag(&fenv, fix_3d_5d)
    ) == None
  );

  // Completely overlapping: one contained within the other
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3d_5u),
      get_range_frag(&fenv, fix_3u_5d)
    ) == None
  );
  assert!(
    cmp_range_frags(
      get_range_frag(&fenv, fix_3u_5d),
      get_range_frag(&fenv, fix_3d_5u)
    ) == None
  );

  // Create a SortedRangeFragIxs from a bunch of RangeFrag indices
  fn new_sorted_frag_ranges(
    fenv: &TypedIxVec<RangeFragIx, RangeFrag>, frags: &Vec<RangeFragIx>,
  ) -> SortedRangeFragIxs {
    SortedRangeFragIxs::new(&frags, fenv)
  }

  // Construction tests
  // These fail due to overlap
  //let _ = new_sorted_frag_ranges(&fenv, &vec![fix_3u_3u, fix_3u_3u]);
  //let _ = new_sorted_frag_ranges(&fenv, &vec![fix_3u_5u, fix_3d_5d]);

  // These fail due to not being in order
  //let _ = new_sorted_frag_ranges(&fenv, &vec![fix_4u_4u, fix_3u_3u]);

  // Simple non-overlap tests for add()

  let smf_empty = new_sorted_frag_ranges(&fenv, &vec![]);
  let smf_6_7_10 =
    new_sorted_frag_ranges(&fenv, &vec![fix_6u_6d, fix_7u_7d, fix_10u]);
  let smf_3_12 = new_sorted_frag_ranges(&fenv, &vec![fix_3u, fix_12u]);
  let smf_3_6_7_10_12 = new_sorted_frag_ranges(
    &fenv,
    &vec![fix_3u, fix_6u_6d, fix_7u_7d, fix_10u, fix_12u],
  );
  let mut tmp;

  tmp = smf_empty.clone();
  tmp.add(&smf_empty, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_empty));

  tmp = smf_3_12.clone();
  tmp.add(&smf_empty, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_3_12));

  tmp = smf_empty.clone();
  tmp.add(&smf_3_12, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_3_12));

  tmp = smf_6_7_10.clone();
  tmp.add(&smf_3_12, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_3_6_7_10_12));

  tmp = smf_3_12.clone();
  tmp.add(&smf_6_7_10, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_3_6_7_10_12));

  // Tests for can_add()
  assert!(true == smf_empty.can_add(&smf_empty, &fenv));
  assert!(true == smf_empty.can_add(&smf_3_12, &fenv));
  assert!(true == smf_3_12.can_add(&smf_empty, &fenv));
  assert!(false == smf_3_12.can_add(&smf_3_12, &fenv));

  assert!(true == smf_6_7_10.can_add(&smf_3_12, &fenv));

  assert!(true == smf_3_12.can_add(&smf_6_7_10, &fenv));

  // Tests for del()
  let smf_6_7 = new_sorted_frag_ranges(&fenv, &vec![fix_6u_6d, fix_7u_7d]);
  let smf_6_10 = new_sorted_frag_ranges(&fenv, &vec![fix_6u_6d, fix_10u]);
  let smf_7 = new_sorted_frag_ranges(&fenv, &vec![fix_7u_7d]);
  let smf_10 = new_sorted_frag_ranges(&fenv, &vec![fix_10u]);

  tmp = smf_empty.clone();
  tmp.del(&smf_empty, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_empty));

  tmp = smf_3_12.clone();
  tmp.del(&smf_empty, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_3_12));

  tmp = smf_empty.clone();
  tmp.del(&smf_3_12, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_empty));

  tmp = smf_6_7_10.clone();
  tmp.del(&smf_3_12, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_6_7_10));

  tmp = smf_3_12.clone();
  tmp.del(&smf_6_7_10, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_3_12));

  tmp = smf_6_7_10.clone();
  tmp.del(&smf_6_7, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_10));

  tmp = smf_6_7_10.clone();
  tmp.del(&smf_10, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_6_7));

  tmp = smf_6_7_10.clone();
  tmp.del(&smf_7, &fenv);
  assert!(sorted_range_eq(&tmp, &smf_6_10));

  // Tests for can_add_if_we_first_del()
  let smf_10_12 = new_sorted_frag_ranges(&fenv, &vec![fix_10u, fix_12u]);

  assert!(
    true
      == smf_6_7_10
        .can_add_if_we_first_del(/*d=*/ &smf_10_12, /*a=*/ &smf_3_12, &fenv)
  );

  assert!(
    false
      == smf_6_7_10
        .can_add_if_we_first_del(/*d=*/ &smf_10_12, /*a=*/ &smf_7, &fenv)
  );
}
*/
