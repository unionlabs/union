use super::{
	CountedList, CountedListWriter, Deserialize, Error, Serialize, VarInt32, VarInt7, VarUint7,
};
use crate::io;
use alloc::vec::Vec;
use core::fmt;

/// Type definition in types section. Currently can be only of the function type.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Type {
	/// Function type.
	Function(FunctionType),
}

impl Deserialize for Type {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(Type::Function(FunctionType::deserialize(reader)?))
	}
}

impl Serialize for Type {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		match self {
			Type::Function(fn_type) => fn_type.serialize(writer),
		}
	}
}

/// Value type.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ValueType {
	/// 32-bit signed integer
	I32,
	/// 64-bit signed integer
	I64,
	/// 32-bit float
	F32,
	/// 64-bit float
	F64,
	#[cfg(feature = "simd")]
	/// 128-bit SIMD register
	V128,
}

impl Deserialize for ValueType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let val = VarInt7::deserialize(reader)?;

		match val.into() {
			-0x01 => Ok(ValueType::I32),
			-0x02 => Ok(ValueType::I64),
			-0x03 => Ok(ValueType::F32),
			-0x04 => Ok(ValueType::F64),
			#[cfg(feature = "simd")]
			-0x05 => Ok(ValueType::V128),
			_ => Err(Error::UnknownValueType(val.into())),
		}
	}
}

impl Serialize for ValueType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let val: VarInt7 = match self {
			ValueType::I32 => -0x01,
			ValueType::I64 => -0x02,
			ValueType::F32 => -0x03,
			ValueType::F64 => -0x04,
			#[cfg(feature = "simd")]
			ValueType::V128 => -0x05,
		}
		.into();
		val.serialize(writer)?;
		Ok(())
	}
}

impl fmt::Display for ValueType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ValueType::I32 => write!(f, "i32"),
			ValueType::I64 => write!(f, "i64"),
			ValueType::F32 => write!(f, "f32"),
			ValueType::F64 => write!(f, "f64"),
			#[cfg(feature = "simd")]
			ValueType::V128 => write!(f, "v128"),
		}
	}
}

/// Block type which is basically `ValueType` + NoResult (to define blocks that have no return type)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockType {
	/// No specified block type
	NoResult,
	/// Inline value type.
	Value(ValueType),
	/// Reference to a signature.
	#[cfg(feature = "multi_value")]
	TypeIndex(u32),
}

impl Deserialize for BlockType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let val = VarInt32::deserialize(reader)?;

		match val.into() {
			-0x40 => Ok(BlockType::NoResult),
			-0x01 => Ok(BlockType::Value(ValueType::I32)),
			-0x02 => Ok(BlockType::Value(ValueType::I64)),
			-0x03 => Ok(BlockType::Value(ValueType::F32)),
			-0x04 => Ok(BlockType::Value(ValueType::F64)),
			#[cfg(feature = "simd")]
			-0x05 => Ok(BlockType::Value(ValueType::V128)),
			#[cfg(feature = "multi_value")]
			idx => {
				let idx = idx.try_into().map_err(|_| Error::UnknownBlockType(idx))?;
				Ok(BlockType::TypeIndex(idx))
			},
			#[cfg(not(feature = "multi_value"))]
			_ => Err(Error::UnknownBlockType(val.into())),
		}
	}
}

impl Serialize for BlockType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let val: VarInt32 = match self {
			BlockType::NoResult => -0x40,
			BlockType::Value(ValueType::I32) => -0x01,
			BlockType::Value(ValueType::I64) => -0x02,
			BlockType::Value(ValueType::F32) => -0x03,
			BlockType::Value(ValueType::F64) => -0x04,
			#[cfg(feature = "simd")]
			BlockType::Value(ValueType::V128) => -0x05,
			#[cfg(feature = "multi_value")]
			BlockType::TypeIndex(idx) => idx as i32,
		}
		.into();
		val.serialize(writer)?;
		Ok(())
	}
}

/// Function signature type.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FunctionType {
	form: u8,
	params: Vec<ValueType>,
	results: Vec<ValueType>,
}

impl Default for FunctionType {
	fn default() -> Self {
		FunctionType { form: 0x60, params: Vec::new(), results: Vec::new() }
	}
}

impl FunctionType {
	/// New function type given the params and results as vectors
	pub fn new(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
		FunctionType { form: 0x60, params, results }
	}
	/// Function form (currently only valid value is `0x60`)
	pub fn form(&self) -> u8 {
		self.form
	}
	/// Parameters in the function signature.
	pub fn params(&self) -> &[ValueType] {
		&self.params
	}
	/// Mutable parameters in the function signature.
	pub fn params_mut(&mut self) -> &mut Vec<ValueType> {
		&mut self.params
	}
	/// Results in the function signature, if any.
	pub fn results(&self) -> &[ValueType] {
		&self.results
	}
	/// Mutable type in the function signature, if any.
	pub fn results_mut(&mut self) -> &mut Vec<ValueType> {
		&mut self.results
	}
}

impl Deserialize for FunctionType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let form: u8 = VarUint7::deserialize(reader)?.into();

		if form != 0x60 {
			return Err(Error::UnknownFunctionForm(form))
		}

		let params: Vec<ValueType> = CountedList::deserialize(reader)?.into_inner();
		let results: Vec<ValueType> = CountedList::deserialize(reader)?.into_inner();

		#[cfg(not(feature = "multi_value"))]
		if results.len() > 1 {
			return Err(Error::Other(
				"Enable the multi_value feature to deserialize more than one function result",
			))
		}

		Ok(FunctionType { form, params, results })
	}
}

impl Serialize for FunctionType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		VarUint7::from(self.form).serialize(writer)?;

		let params_counted_list = CountedListWriter::<ValueType, _>(
			self.params.len(),
			self.params.into_iter().map(Into::into),
		);
		params_counted_list.serialize(writer)?;

		let results_counted_list = CountedListWriter::<ValueType, _>(
			self.results.len(),
			self.results.into_iter().map(Into::into),
		);
		results_counted_list.serialize(writer)?;

		Ok(())
	}
}

/// Table element type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableElementType {
	/// A reference to a function with any signature.
	AnyFunc,
}

impl Deserialize for TableElementType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let val = VarInt7::deserialize(reader)?;

		match val.into() {
			-0x10 => Ok(TableElementType::AnyFunc),
			_ => Err(Error::UnknownTableElementType(val.into())),
		}
	}
}

impl Serialize for TableElementType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let val: VarInt7 = match self {
			TableElementType::AnyFunc => -0x10,
		}
		.into();
		val.serialize(writer)?;
		Ok(())
	}
}
