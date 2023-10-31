use super::{
	CountedList, CountedListWriter, CountedWriter, Deserialize, Error, Instructions, Serialize,
	ValueType, VarUint32,
};
use crate::{elements::section::SectionReader, io};
use alloc::vec::Vec;

/// Function signature (type reference)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Func(u32);

impl Func {
	/// New function signature
	pub fn new(type_ref: u32) -> Self {
		Func(type_ref)
	}

	/// Function signature type reference.
	pub fn type_ref(&self) -> u32 {
		self.0
	}

	/// Function signature type reference (mutable).
	pub fn type_ref_mut(&mut self) -> &mut u32 {
		&mut self.0
	}
}

impl Serialize for Func {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		VarUint32::from(self.0).serialize(writer)
	}
}

impl Deserialize for Func {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(Func(VarUint32::deserialize(reader)?.into()))
	}
}

/// Local definition inside the function body.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Local {
	count: u32,
	value_type: ValueType,
}

impl Local {
	/// New local with `count` and `value_type`.
	pub fn new(count: u32, value_type: ValueType) -> Self {
		Local { count, value_type }
	}

	/// Number of locals with the shared type.
	pub fn count(&self) -> u32 {
		self.count
	}

	/// Type of the locals.
	pub fn value_type(&self) -> ValueType {
		self.value_type
	}
}

impl Deserialize for Local {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let count = VarUint32::deserialize(reader)?;
		let value_type = ValueType::deserialize(reader)?;
		Ok(Local { count: count.into(), value_type })
	}
}

impl Serialize for Local {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		VarUint32::from(self.count).serialize(writer)?;
		self.value_type.serialize(writer)?;
		Ok(())
	}
}

/// Function body definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FuncBody {
	locals: Vec<Local>,
	instructions: Instructions,
}

impl FuncBody {
	/// New function body with given `locals` and `instructions`.
	pub fn new(locals: Vec<Local>, instructions: Instructions) -> Self {
		FuncBody { locals, instructions }
	}

	/// List of individual instructions.
	pub fn empty() -> Self {
		FuncBody { locals: Vec::new(), instructions: Instructions::empty() }
	}

	/// Locals declared in function body.
	pub fn locals(&self) -> &[Local] {
		&self.locals
	}

	/// Instruction list of the function body. Minimal instruction list
	///
	/// is just `&[Instruction::End]`
	pub fn code(&self) -> &Instructions {
		&self.instructions
	}

	/// Locals declared in function body (mutable).
	pub fn locals_mut(&mut self) -> &mut Vec<Local> {
		&mut self.locals
	}

	/// Instruction list of the function body (mutable).
	pub fn code_mut(&mut self) -> &mut Instructions {
		&mut self.instructions
	}
}

impl Deserialize for FuncBody {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut body_reader = SectionReader::new(reader)?;
		let locals: Vec<Local> = CountedList::<Local>::deserialize(&mut body_reader)?.into_inner();

		// The specification obliges us to count the total number of local variables while
		// decoding the binary format.
		locals
			.iter()
			.try_fold(0u32, |acc, &Local { count, .. }| acc.checked_add(count))
			.ok_or(Error::TooManyLocals)?;

		let instructions = Instructions::deserialize(&mut body_reader)?;
		body_reader.close()?;
		Ok(FuncBody { locals, instructions })
	}
}

impl Serialize for FuncBody {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);

		let data = self.locals;
		let counted_list =
			CountedListWriter::<Local, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;

		let code = self.instructions;
		code.serialize(&mut counted_writer)?;

		counted_writer.done()?;

		Ok(())
	}
}
