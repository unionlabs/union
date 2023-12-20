//! Cranelift ValueType hierarchy

use std::fmt;

use crate::shared::types as shared_types;
use cranelift_codegen_shared::constants;

// Rust name prefix used for the `rust_name` method.
static RUST_NAME_PREFIX: &str = "ir::types::";

// ValueType variants (i8, i32, ...) are provided in `shared::types.rs`.

/// A concrete SSA value type.
///
/// All SSA values have a type that is described by an instance of `ValueType`
/// or one of its subclasses.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ValueType {
    Lane(LaneType),
    Reference(ReferenceType),
    Special(SpecialType),
    Vector(VectorType),
    DynamicVector(DynamicVectorType),
}

impl ValueType {
    /// Iterate through all of the lane types.
    pub fn all_lane_types() -> LaneTypeIterator {
        LaneTypeIterator::new()
    }

    /// Iterate through all of the special types (neither lanes nor vectors).
    pub fn all_special_types() -> SpecialTypeIterator {
        SpecialTypeIterator::new()
    }

    pub fn all_reference_types() -> ReferenceTypeIterator {
        ReferenceTypeIterator::new()
    }

    /// Return a string containing the documentation comment for this type.
    pub fn doc(&self) -> String {
        match *self {
            ValueType::Lane(l) => l.doc(),
            ValueType::Reference(r) => r.doc(),
            ValueType::Special(s) => s.doc(),
            ValueType::Vector(ref v) => v.doc(),
            ValueType::DynamicVector(ref v) => v.doc(),
        }
    }

    /// Return the number of bits in a lane.
    pub fn lane_bits(&self) -> u64 {
        match *self {
            ValueType::Lane(l) => l.lane_bits(),
            ValueType::Reference(r) => r.lane_bits(),
            ValueType::Special(s) => s.lane_bits(),
            ValueType::Vector(ref v) => v.lane_bits(),
            ValueType::DynamicVector(ref v) => v.lane_bits(),
        }
    }

    /// Return the number of lanes.
    pub fn lane_count(&self) -> u64 {
        match *self {
            ValueType::Vector(ref v) => v.lane_count(),
            _ => 1,
        }
    }

    /// Find the number of bytes that this type occupies in memory.
    pub fn membytes(&self) -> u64 {
        self.width() / 8
    }

    /// Find the unique number associated with this type.
    pub fn number(&self) -> u16 {
        match *self {
            ValueType::Lane(l) => l.number(),
            ValueType::Reference(r) => r.number(),
            ValueType::Special(s) => s.number(),
            ValueType::Vector(ref v) => v.number(),
            ValueType::DynamicVector(ref v) => v.number(),
        }
    }

    /// Return the name of this type for generated Rust source files.
    pub fn rust_name(&self) -> String {
        format!("{}{}", RUST_NAME_PREFIX, self.to_string().to_uppercase())
    }

    /// Return the total number of bits of an instance of this type.
    pub fn width(&self) -> u64 {
        self.lane_count() * self.lane_bits()
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValueType::Lane(l) => l.fmt(f),
            ValueType::Reference(r) => r.fmt(f),
            ValueType::Special(s) => s.fmt(f),
            ValueType::Vector(ref v) => v.fmt(f),
            ValueType::DynamicVector(ref v) => v.fmt(f),
        }
    }
}

/// Create a ValueType from a given lane type.
impl From<LaneType> for ValueType {
    fn from(lane: LaneType) -> Self {
        ValueType::Lane(lane)
    }
}

/// Create a ValueType from a given reference type.
impl From<ReferenceType> for ValueType {
    fn from(reference: ReferenceType) -> Self {
        ValueType::Reference(reference)
    }
}

/// Create a ValueType from a given special type.
impl From<SpecialType> for ValueType {
    fn from(spec: SpecialType) -> Self {
        ValueType::Special(spec)
    }
}

/// Create a ValueType from a given vector type.
impl From<VectorType> for ValueType {
    fn from(vector: VectorType) -> Self {
        ValueType::Vector(vector)
    }
}

/// Create a ValueType from a given dynamic vector type.
impl From<DynamicVectorType> for ValueType {
    fn from(vector: DynamicVectorType) -> Self {
        ValueType::DynamicVector(vector)
    }
}

/// A concrete scalar type that can appear as a vector lane too.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LaneType {
    Float(shared_types::Float),
    Int(shared_types::Int),
}

impl LaneType {
    /// Return a string containing the documentation comment for this lane type.
    pub fn doc(self) -> String {
        match self {
            LaneType::Float(shared_types::Float::F32) => String::from(
                "A 32-bit floating point type represented in the IEEE 754-2008
                *binary32* interchange format. This corresponds to the :c:type:`float`
                type in most C implementations.",
            ),
            LaneType::Float(shared_types::Float::F64) => String::from(
                "A 64-bit floating point type represented in the IEEE 754-2008
                *binary64* interchange format. This corresponds to the :c:type:`double`
                type in most C implementations.",
            ),
            LaneType::Int(_) if self.lane_bits() < 32 => format!(
                "An integer type with {} bits.
                WARNING: arithmetic on {}bit integers is incomplete",
                self.lane_bits(),
                self.lane_bits()
            ),
            LaneType::Int(_) => format!("An integer type with {} bits.", self.lane_bits()),
        }
    }

    /// Return the number of bits in a lane.
    pub fn lane_bits(self) -> u64 {
        match self {
            LaneType::Float(ref f) => *f as u64,
            LaneType::Int(ref i) => *i as u64,
        }
    }

    /// Find the unique number associated with this lane type.
    pub fn number(self) -> u16 {
        constants::LANE_BASE
            + match self {
                LaneType::Int(shared_types::Int::I8) => 6,
                LaneType::Int(shared_types::Int::I16) => 7,
                LaneType::Int(shared_types::Int::I32) => 8,
                LaneType::Int(shared_types::Int::I64) => 9,
                LaneType::Int(shared_types::Int::I128) => 10,
                LaneType::Float(shared_types::Float::F32) => 11,
                LaneType::Float(shared_types::Float::F64) => 12,
            }
    }

    pub fn int_from_bits(num_bits: u16) -> LaneType {
        LaneType::Int(match num_bits {
            8 => shared_types::Int::I8,
            16 => shared_types::Int::I16,
            32 => shared_types::Int::I32,
            64 => shared_types::Int::I64,
            128 => shared_types::Int::I128,
            _ => unreachable!("unxpected num bits for int"),
        })
    }

    pub fn float_from_bits(num_bits: u16) -> LaneType {
        LaneType::Float(match num_bits {
            32 => shared_types::Float::F32,
            64 => shared_types::Float::F64,
            _ => unreachable!("unxpected num bits for float"),
        })
    }

    pub fn by(self, lanes: u16) -> ValueType {
        if lanes == 1 {
            self.into()
        } else {
            ValueType::Vector(VectorType::new(self, lanes.into()))
        }
    }

    pub fn to_dynamic(self, lanes: u16) -> ValueType {
        ValueType::DynamicVector(DynamicVectorType::new(self, lanes.into()))
    }
}

impl fmt::Display for LaneType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LaneType::Float(_) => write!(f, "f{}", self.lane_bits()),
            LaneType::Int(_) => write!(f, "i{}", self.lane_bits()),
        }
    }
}

impl fmt::Debug for LaneType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner_msg = format!("bits={}", self.lane_bits());
        write!(
            f,
            "{}",
            match *self {
                LaneType::Float(_) => format!("FloatType({})", inner_msg),
                LaneType::Int(_) => format!("IntType({})", inner_msg),
            }
        )
    }
}

/// Create a LaneType from a given float variant.
impl From<shared_types::Float> for LaneType {
    fn from(f: shared_types::Float) -> Self {
        LaneType::Float(f)
    }
}

/// Create a LaneType from a given int variant.
impl From<shared_types::Int> for LaneType {
    fn from(i: shared_types::Int) -> Self {
        LaneType::Int(i)
    }
}

/// An iterator for different lane types.
pub(crate) struct LaneTypeIterator {
    int_iter: shared_types::IntIterator,
    float_iter: shared_types::FloatIterator,
}

impl LaneTypeIterator {
    /// Create a new lane type iterator.
    fn new() -> Self {
        Self {
            int_iter: shared_types::IntIterator::new(),
            float_iter: shared_types::FloatIterator::new(),
        }
    }
}

impl Iterator for LaneTypeIterator {
    type Item = LaneType;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.int_iter.next() {
            Some(LaneType::from(i))
        } else if let Some(f) = self.float_iter.next() {
            Some(LaneType::from(f))
        } else {
            None
        }
    }
}

/// A concrete SIMD vector type.
///
/// A vector type has a lane type which is an instance of `LaneType`,
/// and a positive number of lanes.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct VectorType {
    base: LaneType,
    lanes: u64,
}

impl VectorType {
    /// Initialize a new integer type with `n` bits.
    pub fn new(base: LaneType, lanes: u64) -> Self {
        Self { base, lanes }
    }

    /// Return a string containing the documentation comment for this vector type.
    pub fn doc(&self) -> String {
        format!(
            "A SIMD vector with {} lanes containing a `{}` each.",
            self.lane_count(),
            self.base
        )
    }

    /// Return the number of bits in a lane.
    pub fn lane_bits(&self) -> u64 {
        self.base.lane_bits()
    }

    /// Return the number of lanes.
    pub fn lane_count(&self) -> u64 {
        self.lanes
    }

    /// Return the lane type.
    pub fn lane_type(&self) -> LaneType {
        self.base
    }

    /// Find the unique number associated with this vector type.
    ///
    /// Vector types are encoded with the lane type in the low 4 bits and
    /// log2(lanes) in the high 4 bits, giving a range of 2-256 lanes.
    pub fn number(&self) -> u16 {
        let lanes_log_2: u32 = 63 - self.lane_count().leading_zeros();
        let base_num = u32::from(self.base.number());
        let num = (lanes_log_2 << 4) + base_num;
        num as u16
    }
}

impl fmt::Display for VectorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.base, self.lane_count())
    }
}

impl fmt::Debug for VectorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VectorType(base={}, lanes={})",
            self.base,
            self.lane_count()
        )
    }
}

/// A concrete dynamic SIMD vector type.
///
/// A vector type has a lane type which is an instance of `LaneType`,
/// and a positive number of lanes.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DynamicVectorType {
    base: LaneType,
    unscaled_lanes: u64,
}

impl DynamicVectorType {
    /// Initialize a new type with `base` lane type and a minimum number of lanes.
    pub fn new(base: LaneType, unscaled_lanes: u64) -> Self {
        Self {
            base,
            unscaled_lanes,
        }
    }

    /// Return a string containing the documentation comment for this vector type.
    pub fn doc(&self) -> String {
        format!(
            "A dynamically-scaled SIMD vector with a minimum of {} lanes containing `{}` bits each.",
            self.unscaled_lanes,
            self.base
        )
    }

    /// Return the number of bits in a lane.
    pub fn lane_bits(&self) -> u64 {
        self.base.lane_bits()
    }

    /// Return the number of lanes.
    pub fn minimum_lane_count(&self) -> u64 {
        self.unscaled_lanes
    }

    /// Return the lane type.
    pub fn lane_type(&self) -> LaneType {
        self.base
    }

    /// Find the unique number associated with this vector type.
    ///
    /// Dynamic vector types are encoded in the same manner as `VectorType`,
    /// with lane type in the low 4 bits and the log2(lane_count). We add the
    /// `VECTOR_BASE` to move these numbers into the range beyond the fixed
    /// SIMD types.
    pub fn number(&self) -> u16 {
        let base_num = u32::from(self.base.number());
        let lanes_log_2: u32 = 63 - self.minimum_lane_count().leading_zeros();
        let num = 0x80 + (lanes_log_2 << 4) + base_num;
        num as u16
    }
}

impl fmt::Display for DynamicVectorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}xN", self.base, self.minimum_lane_count())
    }
}

impl fmt::Debug for DynamicVectorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DynamicVectorType(base={}, lanes={})",
            self.base,
            self.minimum_lane_count(),
        )
    }
}

/// A concrete scalar type that is neither a vector nor a lane type.
///
/// Special types cannot be used to form vectors.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SpecialType {
    Flag(shared_types::Flag),
}

impl SpecialType {
    /// Return a string containing the documentation comment for this special type.
    pub fn doc(self) -> String {
        match self {
            SpecialType::Flag(shared_types::Flag::IFlags) => String::from(
                "CPU flags representing the result of an integer comparison. These flags
                can be tested with an :type:`intcc` condition code.",
            ),
            SpecialType::Flag(shared_types::Flag::FFlags) => String::from(
                "CPU flags representing the result of a floating point comparison. These
                flags can be tested with a :type:`floatcc` condition code.",
            ),
        }
    }

    /// Return the number of bits in a lane.
    pub fn lane_bits(self) -> u64 {
        match self {
            SpecialType::Flag(_) => 0,
        }
    }

    /// Find the unique number associated with this special type.
    pub fn number(self) -> u16 {
        match self {
            SpecialType::Flag(shared_types::Flag::IFlags) => 1,
            SpecialType::Flag(shared_types::Flag::FFlags) => 2,
        }
    }
}

impl fmt::Display for SpecialType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpecialType::Flag(shared_types::Flag::IFlags) => write!(f, "iflags"),
            SpecialType::Flag(shared_types::Flag::FFlags) => write!(f, "fflags"),
        }
    }
}

impl fmt::Debug for SpecialType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                SpecialType::Flag(_) => format!("FlagsType({})", self),
            }
        )
    }
}

impl From<shared_types::Flag> for SpecialType {
    fn from(f: shared_types::Flag) -> Self {
        SpecialType::Flag(f)
    }
}

pub(crate) struct SpecialTypeIterator {
    flag_iter: shared_types::FlagIterator,
}

impl SpecialTypeIterator {
    fn new() -> Self {
        Self {
            flag_iter: shared_types::FlagIterator::new(),
        }
    }
}

impl Iterator for SpecialTypeIterator {
    type Item = SpecialType;
    fn next(&mut self) -> Option<Self::Item> {
        self.flag_iter.next().map(SpecialType::from)
    }
}

/// Reference type is scalar type, but not lane type.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ReferenceType(pub shared_types::Reference);

impl ReferenceType {
    /// Return a string containing the documentation comment for this reference type.
    pub fn doc(self) -> String {
        format!("An opaque reference type with {} bits.", self.lane_bits())
    }

    /// Return the number of bits in a lane.
    pub fn lane_bits(self) -> u64 {
        match self.0 {
            shared_types::Reference::R32 => 32,
            shared_types::Reference::R64 => 64,
        }
    }

    /// Find the unique number associated with this reference type.
    pub fn number(self) -> u16 {
        constants::REFERENCE_BASE
            + match self {
                ReferenceType(shared_types::Reference::R32) => 0,
                ReferenceType(shared_types::Reference::R64) => 1,
            }
    }

    pub fn ref_from_bits(num_bits: u16) -> ReferenceType {
        ReferenceType(match num_bits {
            32 => shared_types::Reference::R32,
            64 => shared_types::Reference::R64,
            _ => unreachable!("unexpected number of bits for a reference type"),
        })
    }
}

impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r{}", self.lane_bits())
    }
}

impl fmt::Debug for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReferenceType(bits={})", self.lane_bits())
    }
}

/// Create a ReferenceType from a given reference variant.
impl From<shared_types::Reference> for ReferenceType {
    fn from(r: shared_types::Reference) -> Self {
        ReferenceType(r)
    }
}

/// An iterator for different reference types.
pub(crate) struct ReferenceTypeIterator {
    reference_iter: shared_types::ReferenceIterator,
}

impl ReferenceTypeIterator {
    /// Create a new reference type iterator.
    fn new() -> Self {
        Self {
            reference_iter: shared_types::ReferenceIterator::new(),
        }
    }
}

impl Iterator for ReferenceTypeIterator {
    type Item = ReferenceType;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.reference_iter.next() {
            Some(ReferenceType::from(r))
        } else {
            None
        }
    }
}
