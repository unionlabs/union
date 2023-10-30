use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt;
use std::hash;
use std::iter::FromIterator;
use std::ops;
use std::rc::Rc;

use crate::cdsl::types::{LaneType, ReferenceType, SpecialType, ValueType};

const MAX_LANES: u16 = 256;
const MAX_BITS: u16 = 128;
const MAX_FLOAT_BITS: u16 = 64;

/// Type variables can be used in place of concrete types when defining
/// instructions. This makes the instructions *polymorphic*.
///
/// A type variable is restricted to vary over a subset of the value types.
/// This subset is specified by a set of flags that control the permitted base
/// types and whether the type variable can assume scalar or vector types, or
/// both.
#[derive(Debug)]
pub(crate) struct TypeVarContent {
    /// Short name of type variable used in instruction descriptions.
    pub name: String,

    /// Documentation string.
    pub doc: String,

    /// Type set associated to the type variable.
    /// This field must remain private; use `get_typeset()` or `get_raw_typeset()` to get the
    /// information you want.
    type_set: TypeSet,

    pub base: Option<TypeVarParent>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypeVar {
    content: Rc<RefCell<TypeVarContent>>,
}

impl TypeVar {
    pub fn new(name: impl Into<String>, doc: impl Into<String>, type_set: TypeSet) -> Self {
        Self {
            content: Rc::new(RefCell::new(TypeVarContent {
                name: name.into(),
                doc: doc.into(),
                type_set,
                base: None,
            })),
        }
    }

    pub fn new_singleton(value_type: ValueType) -> Self {
        let (name, doc) = (value_type.to_string(), value_type.doc());
        let mut builder = TypeSetBuilder::new();

        let (scalar_type, num_lanes) = match value_type {
            ValueType::Special(special_type) => {
                return TypeVar::new(name, doc, builder.specials(vec![special_type]).build());
            }
            ValueType::Reference(ReferenceType(reference_type)) => {
                let bits = reference_type as RangeBound;
                return TypeVar::new(name, doc, builder.refs(bits..bits).build());
            }
            ValueType::Lane(lane_type) => (lane_type, 1),
            ValueType::Vector(vec_type) => {
                (vec_type.lane_type(), vec_type.lane_count() as RangeBound)
            }
        };

        builder = builder.simd_lanes(num_lanes..num_lanes);

        let builder = match scalar_type {
            LaneType::Int(int_type) => {
                let bits = int_type as RangeBound;
                builder.ints(bits..bits)
            }
            LaneType::Float(float_type) => {
                let bits = float_type as RangeBound;
                builder.floats(bits..bits)
            }
            LaneType::Bool(bool_type) => {
                let bits = bool_type as RangeBound;
                builder.bools(bits..bits)
            }
        };
        TypeVar::new(name, doc, builder.build())
    }

    /// Get a fresh copy of self, named after `name`. Can only be called on non-derived typevars.
    pub fn copy_from(other: &TypeVar, name: String) -> TypeVar {
        assert!(
            other.base.is_none(),
            "copy_from() can only be called on non-derived type variables"
        );
        TypeVar {
            content: Rc::new(RefCell::new(TypeVarContent {
                name,
                doc: "".into(),
                type_set: other.type_set.clone(),
                base: None,
            })),
        }
    }

    /// Returns the typeset for this TV. If the TV is derived, computes it recursively from the
    /// derived function and the base's typeset.
    /// Note this can't be done non-lazily in the constructor, because the TypeSet of the base may
    /// change over time.
    pub fn get_typeset(&self) -> TypeSet {
        match &self.base {
            Some(base) => base.type_var.get_typeset().image(base.derived_func),
            None => self.type_set.clone(),
        }
    }

    /// Returns this typevar's type set, assuming this type var has no parent.
    pub fn get_raw_typeset(&self) -> &TypeSet {
        assert_eq!(self.type_set, self.get_typeset());
        &self.type_set
    }

    /// If the associated typeset has a single type return it. Otherwise return None.
    pub fn singleton_type(&self) -> Option<ValueType> {
        let type_set = self.get_typeset();
        if type_set.size() == 1 {
            Some(type_set.get_singleton())
        } else {
            None
        }
    }

    /// Get the free type variable controlling this one.
    pub fn free_typevar(&self) -> Option<TypeVar> {
        match &self.base {
            Some(base) => base.type_var.free_typevar(),
            None => {
                match self.singleton_type() {
                    // A singleton type isn't a proper free variable.
                    Some(_) => None,
                    None => Some(self.clone()),
                }
            }
        }
    }

    /// Create a type variable that is a function of another.
    pub fn derived(&self, derived_func: DerivedFunc) -> TypeVar {
        let ts = self.get_typeset();

        // Safety checks to avoid over/underflows.
        assert!(ts.specials.is_empty(), "can't derive from special types");
        match derived_func {
            DerivedFunc::HalfWidth => {
                assert!(
                    ts.ints.is_empty() || *ts.ints.iter().min().unwrap() > 8,
                    "can't halve all integer types"
                );
                assert!(
                    ts.floats.is_empty() || *ts.floats.iter().min().unwrap() > 32,
                    "can't halve all float types"
                );
                assert!(
                    ts.bools.is_empty() || *ts.bools.iter().min().unwrap() > 8,
                    "can't halve all boolean types"
                );
            }
            DerivedFunc::DoubleWidth => {
                assert!(
                    ts.ints.is_empty() || *ts.ints.iter().max().unwrap() < MAX_BITS,
                    "can't double all integer types"
                );
                assert!(
                    ts.floats.is_empty() || *ts.floats.iter().max().unwrap() < MAX_FLOAT_BITS,
                    "can't double all float types"
                );
                assert!(
                    ts.bools.is_empty() || *ts.bools.iter().max().unwrap() < MAX_BITS,
                    "can't double all boolean types"
                );
            }
            DerivedFunc::HalfVector => {
                assert!(
                    *ts.lanes.iter().min().unwrap() > 1,
                    "can't halve a scalar type"
                );
            }
            DerivedFunc::DoubleVector => {
                assert!(
                    *ts.lanes.iter().max().unwrap() < MAX_LANES,
                    "can't double 256 lanes"
                );
            }
            DerivedFunc::SplitLanes => {
                assert!(
                    ts.ints.is_empty() || *ts.ints.iter().min().unwrap() > 8,
                    "can't halve all integer types"
                );
                assert!(
                    ts.floats.is_empty() || *ts.floats.iter().min().unwrap() > 32,
                    "can't halve all float types"
                );
                assert!(
                    ts.bools.is_empty() || *ts.bools.iter().min().unwrap() > 8,
                    "can't halve all boolean types"
                );
                assert!(
                    *ts.lanes.iter().max().unwrap() < MAX_LANES,
                    "can't double 256 lanes"
                );
            }
            DerivedFunc::MergeLanes => {
                assert!(
                    ts.ints.is_empty() || *ts.ints.iter().max().unwrap() < MAX_BITS,
                    "can't double all integer types"
                );
                assert!(
                    ts.floats.is_empty() || *ts.floats.iter().max().unwrap() < MAX_FLOAT_BITS,
                    "can't double all float types"
                );
                assert!(
                    ts.bools.is_empty() || *ts.bools.iter().max().unwrap() < MAX_BITS,
                    "can't double all boolean types"
                );
                assert!(
                    *ts.lanes.iter().min().unwrap() > 1,
                    "can't halve a scalar type"
                );
            }
            DerivedFunc::LaneOf | DerivedFunc::AsBool => { /* no particular assertions */ }
        }

        TypeVar {
            content: Rc::new(RefCell::new(TypeVarContent {
                name: format!("{}({})", derived_func.name(), self.name),
                doc: "".into(),
                type_set: ts,
                base: Some(TypeVarParent {
                    type_var: self.clone(),
                    derived_func,
                }),
            })),
        }
    }

    pub fn lane_of(&self) -> TypeVar {
        self.derived(DerivedFunc::LaneOf)
    }
    pub fn as_bool(&self) -> TypeVar {
        self.derived(DerivedFunc::AsBool)
    }
    pub fn half_width(&self) -> TypeVar {
        self.derived(DerivedFunc::HalfWidth)
    }
    pub fn double_width(&self) -> TypeVar {
        self.derived(DerivedFunc::DoubleWidth)
    }
    pub fn half_vector(&self) -> TypeVar {
        self.derived(DerivedFunc::HalfVector)
    }
    pub fn double_vector(&self) -> TypeVar {
        self.derived(DerivedFunc::DoubleVector)
    }
    pub fn split_lanes(&self) -> TypeVar {
        self.derived(DerivedFunc::SplitLanes)
    }
    pub fn merge_lanes(&self) -> TypeVar {
        self.derived(DerivedFunc::MergeLanes)
    }
}

impl Into<TypeVar> for &TypeVar {
    fn into(self) -> TypeVar {
        self.clone()
    }
}
impl Into<TypeVar> for ValueType {
    fn into(self) -> TypeVar {
        TypeVar::new_singleton(self)
    }
}

// Hash TypeVars by pointers.
// There might be a better way to do this, but since TypeVar's content (namely TypeSet) can be
// mutated, it makes sense to use pointer equality/hashing here.
impl hash::Hash for TypeVar {
    fn hash<H: hash::Hasher>(&self, h: &mut H) {
        match &self.base {
            Some(base) => {
                base.type_var.hash(h);
                base.derived_func.hash(h);
            }
            None => {
                (&**self as *const TypeVarContent).hash(h);
            }
        }
    }
}

impl PartialEq for TypeVar {
    fn eq(&self, other: &TypeVar) -> bool {
        match (&self.base, &other.base) {
            (Some(base1), Some(base2)) => {
                base1.type_var.eq(&base2.type_var) && base1.derived_func == base2.derived_func
            }
            (None, None) => Rc::ptr_eq(&self.content, &other.content),
            _ => false,
        }
    }
}

// Allow TypeVar as map keys, based on pointer equality (see also above PartialEq impl).
impl Eq for TypeVar {}

impl ops::Deref for TypeVar {
    type Target = TypeVarContent;
    fn deref(&self) -> &Self::Target {
        unsafe { self.content.as_ptr().as_ref().unwrap() }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) enum DerivedFunc {
    LaneOf,
    AsBool,
    HalfWidth,
    DoubleWidth,
    HalfVector,
    DoubleVector,
    SplitLanes,
    MergeLanes,
}

impl DerivedFunc {
    pub fn name(self) -> &'static str {
        match self {
            DerivedFunc::LaneOf => "lane_of",
            DerivedFunc::AsBool => "as_bool",
            DerivedFunc::HalfWidth => "half_width",
            DerivedFunc::DoubleWidth => "double_width",
            DerivedFunc::HalfVector => "half_vector",
            DerivedFunc::DoubleVector => "double_vector",
            DerivedFunc::SplitLanes => "split_lanes",
            DerivedFunc::MergeLanes => "merge_lanes",
        }
    }
}

#[derive(Debug, Hash)]
pub(crate) struct TypeVarParent {
    pub type_var: TypeVar,
    pub derived_func: DerivedFunc,
}

/// A set of types.
///
/// We don't allow arbitrary subsets of types, but use a parametrized approach
/// instead.
///
/// Objects of this class can be used as dictionary keys.
///
/// Parametrized type sets are specified in terms of ranges:
/// - The permitted range of vector lanes, where 1 indicates a scalar type.
/// - The permitted range of integer types.
/// - The permitted range of floating point types, and
/// - The permitted range of boolean types.
///
/// The ranges are inclusive from smallest bit-width to largest bit-width.
///
/// Finally, a type set can contain special types (derived from `SpecialType`)
/// which can't appear as lane types.

type RangeBound = u16;
type Range = ops::Range<RangeBound>;
type NumSet = BTreeSet<RangeBound>;

macro_rules! num_set {
    ($($expr:expr),*) => {
        NumSet::from_iter(vec![$($expr),*])
    };
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypeSet {
    pub lanes: NumSet,
    pub ints: NumSet,
    pub floats: NumSet,
    pub bools: NumSet,
    pub refs: NumSet,
    pub specials: Vec<SpecialType>,
}

impl TypeSet {
    fn new(
        lanes: NumSet,
        ints: NumSet,
        floats: NumSet,
        bools: NumSet,
        refs: NumSet,
        specials: Vec<SpecialType>,
    ) -> Self {
        Self {
            lanes,
            ints,
            floats,
            bools,
            refs,
            specials,
        }
    }

    /// Return the number of concrete types represented by this typeset.
    pub fn size(&self) -> usize {
        self.lanes.len()
            * (self.ints.len() + self.floats.len() + self.bools.len() + self.refs.len())
            + self.specials.len()
    }

    /// Return the image of self across the derived function func.
    fn image(&self, derived_func: DerivedFunc) -> TypeSet {
        match derived_func {
            DerivedFunc::LaneOf => self.lane_of(),
            DerivedFunc::AsBool => self.as_bool(),
            DerivedFunc::HalfWidth => self.half_width(),
            DerivedFunc::DoubleWidth => self.double_width(),
            DerivedFunc::HalfVector => self.half_vector(),
            DerivedFunc::DoubleVector => self.double_vector(),
            DerivedFunc::SplitLanes => self.half_width().double_vector(),
            DerivedFunc::MergeLanes => self.double_width().half_vector(),
        }
    }

    /// Return a TypeSet describing the image of self across lane_of.
    fn lane_of(&self) -> TypeSet {
        let mut copy = self.clone();
        copy.lanes = num_set![1];
        copy
    }

    /// Return a TypeSet describing the image of self across as_bool.
    fn as_bool(&self) -> TypeSet {
        let mut copy = self.clone();
        copy.ints = NumSet::new();
        copy.floats = NumSet::new();
        copy.refs = NumSet::new();
        if !(&self.lanes - &num_set![1]).is_empty() {
            copy.bools = &self.ints | &self.floats;
            copy.bools = &copy.bools | &self.bools;
        }
        if self.lanes.contains(&1) {
            copy.bools.insert(1);
        }
        copy
    }

    /// Return a TypeSet describing the image of self across halfwidth.
    fn half_width(&self) -> TypeSet {
        let mut copy = self.clone();
        copy.ints = NumSet::from_iter(self.ints.iter().filter(|&&x| x > 8).map(|&x| x / 2));
        copy.floats = NumSet::from_iter(self.floats.iter().filter(|&&x| x > 32).map(|&x| x / 2));
        copy.bools = NumSet::from_iter(self.bools.iter().filter(|&&x| x > 8).map(|&x| x / 2));
        copy.specials = Vec::new();
        copy
    }

    /// Return a TypeSet describing the image of self across doublewidth.
    fn double_width(&self) -> TypeSet {
        let mut copy = self.clone();
        copy.ints = NumSet::from_iter(self.ints.iter().filter(|&&x| x < MAX_BITS).map(|&x| x * 2));
        copy.floats = NumSet::from_iter(
            self.floats
                .iter()
                .filter(|&&x| x < MAX_FLOAT_BITS)
                .map(|&x| x * 2),
        );
        copy.bools = NumSet::from_iter(
            self.bools
                .iter()
                .filter(|&&x| x < MAX_BITS)
                .map(|&x| x * 2)
                .filter(|x| legal_bool(*x)),
        );
        copy.specials = Vec::new();
        copy
    }

    /// Return a TypeSet describing the image of self across halfvector.
    fn half_vector(&self) -> TypeSet {
        let mut copy = self.clone();
        copy.lanes = NumSet::from_iter(self.lanes.iter().filter(|&&x| x > 1).map(|&x| x / 2));
        copy.specials = Vec::new();
        copy
    }

    /// Return a TypeSet describing the image of self across doublevector.
    fn double_vector(&self) -> TypeSet {
        let mut copy = self.clone();
        copy.lanes = NumSet::from_iter(
            self.lanes
                .iter()
                .filter(|&&x| x < MAX_LANES)
                .map(|&x| x * 2),
        );
        copy.specials = Vec::new();
        copy
    }

    fn concrete_types(&self) -> Vec<ValueType> {
        let mut ret = Vec::new();
        for &num_lanes in &self.lanes {
            for &bits in &self.ints {
                ret.push(LaneType::int_from_bits(bits).by(num_lanes));
            }
            for &bits in &self.floats {
                ret.push(LaneType::float_from_bits(bits).by(num_lanes));
            }
            for &bits in &self.bools {
                ret.push(LaneType::bool_from_bits(bits).by(num_lanes));
            }
            for &bits in &self.refs {
                ret.push(ReferenceType::ref_from_bits(bits).into());
            }
        }
        for &special in &self.specials {
            ret.push(special.into());
        }
        ret
    }

    /// Return the singleton type represented by self. Can only call on typesets containing 1 type.
    fn get_singleton(&self) -> ValueType {
        let mut types = self.concrete_types();
        assert_eq!(types.len(), 1);
        types.remove(0)
    }
}

impl fmt::Debug for TypeSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "TypeSet(")?;

        let mut subsets = Vec::new();
        if !self.lanes.is_empty() {
            subsets.push(format!(
                "lanes={{{}}}",
                Vec::from_iter(self.lanes.iter().map(|x| x.to_string())).join(", ")
            ));
        }
        if !self.ints.is_empty() {
            subsets.push(format!(
                "ints={{{}}}",
                Vec::from_iter(self.ints.iter().map(|x| x.to_string())).join(", ")
            ));
        }
        if !self.floats.is_empty() {
            subsets.push(format!(
                "floats={{{}}}",
                Vec::from_iter(self.floats.iter().map(|x| x.to_string())).join(", ")
            ));
        }
        if !self.bools.is_empty() {
            subsets.push(format!(
                "bools={{{}}}",
                Vec::from_iter(self.bools.iter().map(|x| x.to_string())).join(", ")
            ));
        }
        if !self.refs.is_empty() {
            subsets.push(format!(
                "refs={{{}}}",
                Vec::from_iter(self.refs.iter().map(|x| x.to_string())).join(", ")
            ));
        }
        if !self.specials.is_empty() {
            subsets.push(format!(
                "specials={{{}}}",
                Vec::from_iter(self.specials.iter().map(|x| x.to_string())).join(", ")
            ));
        }

        write!(fmt, "{})", subsets.join(", "))?;
        Ok(())
    }
}

pub(crate) struct TypeSetBuilder {
    ints: Interval,
    floats: Interval,
    bools: Interval,
    refs: Interval,
    includes_scalars: bool,
    simd_lanes: Interval,
    specials: Vec<SpecialType>,
}

impl TypeSetBuilder {
    pub fn new() -> Self {
        Self {
            ints: Interval::None,
            floats: Interval::None,
            bools: Interval::None,
            refs: Interval::None,
            includes_scalars: true,
            simd_lanes: Interval::None,
            specials: Vec::new(),
        }
    }

    pub fn ints(mut self, interval: impl Into<Interval>) -> Self {
        assert!(self.ints == Interval::None);
        self.ints = interval.into();
        self
    }
    pub fn floats(mut self, interval: impl Into<Interval>) -> Self {
        assert!(self.floats == Interval::None);
        self.floats = interval.into();
        self
    }
    pub fn bools(mut self, interval: impl Into<Interval>) -> Self {
        assert!(self.bools == Interval::None);
        self.bools = interval.into();
        self
    }
    pub fn refs(mut self, interval: impl Into<Interval>) -> Self {
        assert!(self.refs == Interval::None);
        self.refs = interval.into();
        self
    }
    pub fn includes_scalars(mut self, includes_scalars: bool) -> Self {
        self.includes_scalars = includes_scalars;
        self
    }
    pub fn simd_lanes(mut self, interval: impl Into<Interval>) -> Self {
        assert!(self.simd_lanes == Interval::None);
        self.simd_lanes = interval.into();
        self
    }
    pub fn specials(mut self, specials: Vec<SpecialType>) -> Self {
        assert!(self.specials.is_empty());
        self.specials = specials;
        self
    }

    pub fn build(self) -> TypeSet {
        let min_lanes = if self.includes_scalars { 1 } else { 2 };

        let bools = range_to_set(self.bools.to_range(1..MAX_BITS, None))
            .into_iter()
            .filter(|x| legal_bool(*x))
            .collect();

        TypeSet::new(
            range_to_set(self.simd_lanes.to_range(min_lanes..MAX_LANES, Some(1))),
            range_to_set(self.ints.to_range(8..MAX_BITS, None)),
            range_to_set(self.floats.to_range(32..64, None)),
            bools,
            range_to_set(self.refs.to_range(32..64, None)),
            self.specials,
        )
    }
}

#[derive(PartialEq)]
pub(crate) enum Interval {
    None,
    All,
    Range(Range),
}

impl Interval {
    fn to_range(&self, full_range: Range, default: Option<RangeBound>) -> Option<Range> {
        match self {
            Interval::None => {
                if let Some(default_val) = default {
                    Some(default_val..default_val)
                } else {
                    None
                }
            }

            Interval::All => Some(full_range),

            Interval::Range(range) => {
                let (low, high) = (range.start, range.end);
                assert!(low.is_power_of_two());
                assert!(high.is_power_of_two());
                assert!(low <= high);
                assert!(low >= full_range.start);
                assert!(high <= full_range.end);
                Some(low..high)
            }
        }
    }
}

impl Into<Interval> for Range {
    fn into(self) -> Interval {
        Interval::Range(self)
    }
}

fn legal_bool(bits: RangeBound) -> bool {
    // Only allow legal bit widths for bool types.
    bits == 1 || (bits >= 8 && bits <= MAX_BITS && bits.is_power_of_two())
}

/// Generates a set with all the powers of two included in the range.
fn range_to_set(range: Option<Range>) -> NumSet {
    let mut set = NumSet::new();

    let (low, high) = match range {
        Some(range) => (range.start, range.end),
        None => return set,
    };

    assert!(low.is_power_of_two());
    assert!(high.is_power_of_two());
    assert!(low <= high);

    for i in low.trailing_zeros()..=high.trailing_zeros() {
        assert!(1 << i <= RangeBound::max_value());
        set.insert(1 << i);
    }
    set
}

#[test]
fn test_typevar_builder() {
    let type_set = TypeSetBuilder::new().ints(Interval::All).build();
    assert_eq!(type_set.lanes, num_set![1]);
    assert!(type_set.floats.is_empty());
    assert_eq!(type_set.ints, num_set![8, 16, 32, 64, 128]);
    assert!(type_set.bools.is_empty());
    assert!(type_set.specials.is_empty());

    let type_set = TypeSetBuilder::new().bools(Interval::All).build();
    assert_eq!(type_set.lanes, num_set![1]);
    assert!(type_set.floats.is_empty());
    assert!(type_set.ints.is_empty());
    assert_eq!(type_set.bools, num_set![1, 8, 16, 32, 64, 128]);
    assert!(type_set.specials.is_empty());

    let type_set = TypeSetBuilder::new().floats(Interval::All).build();
    assert_eq!(type_set.lanes, num_set![1]);
    assert_eq!(type_set.floats, num_set![32, 64]);
    assert!(type_set.ints.is_empty());
    assert!(type_set.bools.is_empty());
    assert!(type_set.specials.is_empty());

    let type_set = TypeSetBuilder::new()
        .floats(Interval::All)
        .simd_lanes(Interval::All)
        .includes_scalars(false)
        .build();
    assert_eq!(type_set.lanes, num_set![2, 4, 8, 16, 32, 64, 128, 256]);
    assert_eq!(type_set.floats, num_set![32, 64]);
    assert!(type_set.ints.is_empty());
    assert!(type_set.bools.is_empty());
    assert!(type_set.specials.is_empty());

    let type_set = TypeSetBuilder::new()
        .floats(Interval::All)
        .simd_lanes(Interval::All)
        .includes_scalars(true)
        .build();
    assert_eq!(type_set.lanes, num_set![1, 2, 4, 8, 16, 32, 64, 128, 256]);
    assert_eq!(type_set.floats, num_set![32, 64]);
    assert!(type_set.ints.is_empty());
    assert!(type_set.bools.is_empty());
    assert!(type_set.specials.is_empty());

    let type_set = TypeSetBuilder::new().ints(16..64).build();
    assert_eq!(type_set.lanes, num_set![1]);
    assert_eq!(type_set.ints, num_set![16, 32, 64]);
    assert!(type_set.floats.is_empty());
    assert!(type_set.bools.is_empty());
    assert!(type_set.specials.is_empty());
}

#[test]
#[should_panic]
fn test_typevar_builder_too_high_bound_panic() {
    TypeSetBuilder::new().ints(16..2 * MAX_BITS).build();
}

#[test]
#[should_panic]
fn test_typevar_builder_inverted_bounds_panic() {
    TypeSetBuilder::new().ints(32..16).build();
}

#[test]
fn test_as_bool() {
    let a = TypeSetBuilder::new()
        .simd_lanes(2..8)
        .ints(8..8)
        .floats(32..32)
        .build();
    assert_eq!(
        a.lane_of(),
        TypeSetBuilder::new().ints(8..8).floats(32..32).build()
    );

    // Test as_bool with disjoint intervals.
    let mut a_as_bool = TypeSetBuilder::new().simd_lanes(2..8).build();
    a_as_bool.bools = num_set![8, 32];
    assert_eq!(a.as_bool(), a_as_bool);

    let b = TypeSetBuilder::new()
        .simd_lanes(1..8)
        .ints(8..8)
        .floats(32..32)
        .build();
    let mut b_as_bool = TypeSetBuilder::new().simd_lanes(1..8).build();
    b_as_bool.bools = num_set![1, 8, 32];
    assert_eq!(b.as_bool(), b_as_bool);
}

#[test]
fn test_forward_images() {
    let empty_set = TypeSetBuilder::new().build();

    // Half vector.
    assert_eq!(
        TypeSetBuilder::new()
            .simd_lanes(1..32)
            .build()
            .half_vector(),
        TypeSetBuilder::new().simd_lanes(1..16).build()
    );

    // Double vector.
    assert_eq!(
        TypeSetBuilder::new()
            .simd_lanes(1..32)
            .build()
            .double_vector(),
        TypeSetBuilder::new().simd_lanes(2..64).build()
    );
    assert_eq!(
        TypeSetBuilder::new()
            .simd_lanes(128..256)
            .build()
            .double_vector(),
        TypeSetBuilder::new().simd_lanes(256..256).build()
    );

    // Half width.
    assert_eq!(
        TypeSetBuilder::new().ints(8..32).build().half_width(),
        TypeSetBuilder::new().ints(8..16).build()
    );
    assert_eq!(
        TypeSetBuilder::new().floats(32..32).build().half_width(),
        empty_set
    );
    assert_eq!(
        TypeSetBuilder::new().floats(32..64).build().half_width(),
        TypeSetBuilder::new().floats(32..32).build()
    );
    assert_eq!(
        TypeSetBuilder::new().bools(1..8).build().half_width(),
        empty_set
    );
    assert_eq!(
        TypeSetBuilder::new().bools(1..32).build().half_width(),
        TypeSetBuilder::new().bools(8..16).build()
    );

    // Double width.
    assert_eq!(
        TypeSetBuilder::new().ints(8..32).build().double_width(),
        TypeSetBuilder::new().ints(16..64).build()
    );
    assert_eq!(
        TypeSetBuilder::new().ints(32..64).build().double_width(),
        TypeSetBuilder::new().ints(64..128).build()
    );
    assert_eq!(
        TypeSetBuilder::new().floats(32..32).build().double_width(),
        TypeSetBuilder::new().floats(64..64).build()
    );
    assert_eq!(
        TypeSetBuilder::new().floats(32..64).build().double_width(),
        TypeSetBuilder::new().floats(64..64).build()
    );
    assert_eq!(
        TypeSetBuilder::new().bools(1..16).build().double_width(),
        TypeSetBuilder::new().bools(16..32).build()
    );
    assert_eq!(
        TypeSetBuilder::new().bools(32..64).build().double_width(),
        TypeSetBuilder::new().bools(64..128).build()
    );
}

#[test]
#[should_panic]
fn test_typeset_singleton_panic_nonsingleton_types() {
    TypeSetBuilder::new()
        .ints(8..8)
        .floats(32..32)
        .build()
        .get_singleton();
}

#[test]
#[should_panic]
fn test_typeset_singleton_panic_nonsingleton_lanes() {
    TypeSetBuilder::new()
        .simd_lanes(1..2)
        .floats(32..32)
        .build()
        .get_singleton();
}

#[test]
fn test_typeset_singleton() {
    use crate::shared::types as shared_types;
    assert_eq!(
        TypeSetBuilder::new().ints(16..16).build().get_singleton(),
        ValueType::Lane(shared_types::Int::I16.into())
    );
    assert_eq!(
        TypeSetBuilder::new().floats(64..64).build().get_singleton(),
        ValueType::Lane(shared_types::Float::F64.into())
    );
    assert_eq!(
        TypeSetBuilder::new().bools(1..1).build().get_singleton(),
        ValueType::Lane(shared_types::Bool::B1.into())
    );
    assert_eq!(
        TypeSetBuilder::new()
            .simd_lanes(4..4)
            .ints(32..32)
            .build()
            .get_singleton(),
        LaneType::from(shared_types::Int::I32).by(4)
    );
}

#[test]
fn test_typevar_functions() {
    let x = TypeVar::new(
        "x",
        "i16 and up",
        TypeSetBuilder::new().ints(16..64).build(),
    );
    assert_eq!(x.half_width().name, "half_width(x)");
    assert_eq!(
        x.half_width().double_width().name,
        "double_width(half_width(x))"
    );

    let x = TypeVar::new("x", "up to i32", TypeSetBuilder::new().ints(8..32).build());
    assert_eq!(x.double_width().name, "double_width(x)");
}

#[test]
fn test_typevar_singleton() {
    use crate::cdsl::types::VectorType;
    use crate::shared::types as shared_types;

    // Test i32.
    let typevar = TypeVar::new_singleton(ValueType::Lane(LaneType::Int(shared_types::Int::I32)));
    assert_eq!(typevar.name, "i32");
    assert_eq!(typevar.type_set.ints, num_set![32]);
    assert!(typevar.type_set.floats.is_empty());
    assert!(typevar.type_set.bools.is_empty());
    assert!(typevar.type_set.specials.is_empty());
    assert_eq!(typevar.type_set.lanes, num_set![1]);

    // Test f32x4.
    let typevar = TypeVar::new_singleton(ValueType::Vector(VectorType::new(
        LaneType::Float(shared_types::Float::F32),
        4,
    )));
    assert_eq!(typevar.name, "f32x4");
    assert!(typevar.type_set.ints.is_empty());
    assert_eq!(typevar.type_set.floats, num_set![32]);
    assert_eq!(typevar.type_set.lanes, num_set![4]);
    assert!(typevar.type_set.bools.is_empty());
    assert!(typevar.type_set.specials.is_empty());
}
