use super::{
	serialize, CountedList, CountedListWriter, CountedWriter, DataSegment, Deserialize,
	ElementSegment, Error, ExportEntry, External, Func, FuncBody, GlobalEntry, ImportEntry,
	MemoryType, Serialize, TableType, VarUint32, VarUint7,
};
use crate::{elements, io};
use alloc::{borrow::ToOwned, string::String, vec::Vec};

use super::{name_section::NameSection, reloc_section::RelocSection, types::Type};

#[cfg(feature = "reduced-stack-buffer")]
const ENTRIES_BUFFER_LENGTH: usize = 256;

#[cfg(not(feature = "reduced-stack-buffer"))]
const ENTRIES_BUFFER_LENGTH: usize = 16384;

/// Section in the WebAssembly module.
#[derive(Debug, Clone, PartialEq)]
pub enum Section {
	/// Section is unparsed.
	Unparsed {
		/// id of the unparsed section.
		id: u8,
		/// raw bytes of the unparsed section.
		payload: Vec<u8>,
	},
	/// Custom section (`id=0`).
	Custom(CustomSection),
	/// Types section.
	Type(TypeSection),
	/// Import section.
	Import(ImportSection),
	/// Function signatures section.
	Function(FunctionSection),
	/// Table definition section.
	Table(TableSection),
	/// Memory definition section.
	Memory(MemorySection),
	/// Global entries section.
	Global(GlobalSection),
	/// Export definitions.
	Export(ExportSection),
	/// Entry reference of the module.
	Start(u32),
	/// Elements section.
	Element(ElementSection),
	/// Number of passive data entries in the data section
	DataCount(u32),
	/// Function bodies section.
	Code(CodeSection),
	/// Data definition section.
	Data(DataSection),
	/// Name section.
	///
	/// Note that initially it is not parsed until `parse_names` is called explicitly.
	Name(NameSection),
	/// Relocation section.
	///
	/// Note that initially it is not parsed until `parse_reloc` is called explicitly.
	/// Also note that currently there are serialization (but not de-serialization)
	///   issues with this section (#198).
	Reloc(RelocSection),
}

impl Deserialize for Section {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let id = match VarUint7::deserialize(reader) {
			// todo: be more selective detecting no more section
			Err(_) => return Err(Error::UnexpectedEof),
			Ok(id) => id,
		};

		Ok(match id.into() {
			0 => Section::Custom(CustomSection::deserialize(reader)?),
			1 => Section::Type(TypeSection::deserialize(reader)?),
			2 => Section::Import(ImportSection::deserialize(reader)?),
			3 => Section::Function(FunctionSection::deserialize(reader)?),
			4 => Section::Table(TableSection::deserialize(reader)?),
			5 => Section::Memory(MemorySection::deserialize(reader)?),
			6 => Section::Global(GlobalSection::deserialize(reader)?),
			7 => Section::Export(ExportSection::deserialize(reader)?),
			8 => {
				let mut section_reader = SectionReader::new(reader)?;
				let start_idx = VarUint32::deserialize(&mut section_reader)?;
				section_reader.close()?;
				Section::Start(start_idx.into())
			},
			9 => Section::Element(ElementSection::deserialize(reader)?),
			10 => Section::Code(CodeSection::deserialize(reader)?),
			11 => Section::Data(DataSection::deserialize(reader)?),
			12 => {
				let mut section_reader = SectionReader::new(reader)?;
				let count = VarUint32::deserialize(&mut section_reader)?;
				section_reader.close()?;
				Section::DataCount(count.into())
			},
			invalid_id => return Err(Error::InvalidSectionId(invalid_id)),
		})
	}
}

impl Serialize for Section {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		match self {
			Section::Custom(custom_section) => {
				VarUint7::from(0x00).serialize(writer)?;
				custom_section.serialize(writer)?;
			},
			Section::Unparsed { id, payload } => {
				VarUint7::from(id).serialize(writer)?;
				writer.write(&payload[..])?;
			},
			Section::Type(type_section) => {
				VarUint7::from(0x01).serialize(writer)?;
				type_section.serialize(writer)?;
			},
			Section::Import(import_section) => {
				VarUint7::from(0x02).serialize(writer)?;
				import_section.serialize(writer)?;
			},
			Section::Function(function_section) => {
				VarUint7::from(0x03).serialize(writer)?;
				function_section.serialize(writer)?;
			},
			Section::Table(table_section) => {
				VarUint7::from(0x04).serialize(writer)?;
				table_section.serialize(writer)?;
			},
			Section::Memory(memory_section) => {
				VarUint7::from(0x05).serialize(writer)?;
				memory_section.serialize(writer)?;
			},
			Section::Global(global_section) => {
				VarUint7::from(0x06).serialize(writer)?;
				global_section.serialize(writer)?;
			},
			Section::Export(export_section) => {
				VarUint7::from(0x07).serialize(writer)?;
				export_section.serialize(writer)?;
			},
			Section::Start(index) => {
				VarUint7::from(0x08).serialize(writer)?;
				let mut counted_writer = CountedWriter::new(writer);
				VarUint32::from(index).serialize(&mut counted_writer)?;
				counted_writer.done()?;
			},
			Section::DataCount(count) => {
				VarUint7::from(0x0c).serialize(writer)?;
				let mut counted_writer = CountedWriter::new(writer);
				VarUint32::from(count).serialize(&mut counted_writer)?;
				counted_writer.done()?;
			},
			Section::Element(element_section) => {
				VarUint7::from(0x09).serialize(writer)?;
				element_section.serialize(writer)?;
			},
			Section::Code(code_section) => {
				VarUint7::from(0x0a).serialize(writer)?;
				code_section.serialize(writer)?;
			},
			Section::Data(data_section) => {
				VarUint7::from(0x0b).serialize(writer)?;
				data_section.serialize(writer)?;
			},
			Section::Name(name_section) => {
				VarUint7::from(0x00).serialize(writer)?;
				let custom =
					CustomSection { name: "name".to_owned(), payload: serialize(name_section)? };
				custom.serialize(writer)?;
			},
			Section::Reloc(reloc_section) => {
				VarUint7::from(0x00).serialize(writer)?;
				reloc_section.serialize(writer)?;
			},
		}
		Ok(())
	}
}

impl Section {
	pub(crate) fn order(&self) -> u8 {
		match *self {
			Section::Custom(_) => 0x00,
			Section::Unparsed { .. } => 0x00,
			Section::Type(_) => 0x1,
			Section::Import(_) => 0x2,
			Section::Function(_) => 0x3,
			Section::Table(_) => 0x4,
			Section::Memory(_) => 0x5,
			Section::Global(_) => 0x6,
			Section::Export(_) => 0x7,
			Section::Start(_) => 0x8,
			Section::Element(_) => 0x9,
			Section::DataCount(_) => 0x0a,
			Section::Code(_) => 0x0b,
			Section::Data(_) => 0x0c,
			Section::Name(_) => 0x00,
			Section::Reloc(_) => 0x00,
		}
	}
}

pub(crate) struct SectionReader {
	cursor: io::Cursor<Vec<u8>>,
	declared_length: usize,
}

impl SectionReader {
	pub fn new<R: io::Read>(reader: &mut R) -> Result<Self, elements::Error> {
		let length = u32::from(VarUint32::deserialize(reader)?) as usize;
		let inner_buffer = buffered_read!(ENTRIES_BUFFER_LENGTH, length, reader);
		let declared_length = inner_buffer.len();
		let cursor = io::Cursor::new(inner_buffer);

		Ok(SectionReader { cursor, declared_length })
	}

	pub fn close(self) -> Result<(), io::Error> {
		let cursor = self.cursor;
		let buf_length = self.declared_length;

		if cursor.position() != buf_length {
			Err(io::Error::InvalidData)
		} else {
			Ok(())
		}
	}
}

impl io::Read for SectionReader {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<()> {
		self.cursor.read(buf)?;
		Ok(())
	}
}

fn read_entries<R: io::Read, T: Deserialize<Error = elements::Error>>(
	reader: &mut R,
) -> Result<Vec<T>, elements::Error> {
	let mut section_reader = SectionReader::new(reader)?;
	let result = CountedList::<T>::deserialize(&mut section_reader)?.into_inner();
	section_reader.close()?;
	Ok(result)
}

/// Custom section.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CustomSection {
	name: String,
	payload: Vec<u8>,
}

impl CustomSection {
	/// Creates a new custom section with the given name and payload.
	pub fn new(name: String, payload: Vec<u8>) -> CustomSection {
		CustomSection { name, payload }
	}

	/// Name of the custom section.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Payload of the custom section.
	pub fn payload(&self) -> &[u8] {
		&self.payload
	}

	/// Name of the custom section (mutable).
	pub fn name_mut(&mut self) -> &mut String {
		&mut self.name
	}

	/// Payload of the custom section (mutable).
	pub fn payload_mut(&mut self) -> &mut Vec<u8> {
		&mut self.payload
	}
}

impl Deserialize for CustomSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let section_length: usize = u32::from(VarUint32::deserialize(reader)?) as usize;
		let buf = buffered_read!(ENTRIES_BUFFER_LENGTH, section_length, reader);
		let mut cursor = io::Cursor::new(&buf[..]);
		let name = String::deserialize(&mut cursor)?;
		let payload = buf[cursor.position() as usize..].to_vec();
		Ok(CustomSection { name, payload })
	}
}

impl Serialize for CustomSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use io::Write;

		let mut counted_writer = CountedWriter::new(writer);
		self.name.serialize(&mut counted_writer)?;
		counted_writer.write(&self.payload[..])?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Section with type declarations.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TypeSection(Vec<Type>);

impl TypeSection {
	///  New type section with provided types.
	pub fn with_types(types: Vec<Type>) -> Self {
		TypeSection(types)
	}

	/// List of type declarations.
	pub fn types(&self) -> &[Type] {
		&self.0
	}

	/// List of type declarations (mutable).
	pub fn types_mut(&mut self) -> &mut Vec<Type> {
		&mut self.0
	}
}

impl Deserialize for TypeSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(TypeSection(read_entries(reader)?))
	}
}

impl Serialize for TypeSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<Type, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Section of the imports definition.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ImportSection(Vec<ImportEntry>);

impl ImportSection {
	///  New import section with provided types.
	pub fn with_entries(entries: Vec<ImportEntry>) -> Self {
		ImportSection(entries)
	}

	/// List of import entries.
	pub fn entries(&self) -> &[ImportEntry] {
		&self.0
	}

	/// List of import entries (mutable).
	pub fn entries_mut(&mut self) -> &mut Vec<ImportEntry> {
		&mut self.0
	}

	/// Returns number of functions.
	pub fn functions(&self) -> usize {
		self.0
			.iter()
			.filter(|entry| matches!(*entry.external(), External::Function(_)))
			.count()
	}

	/// Returns number of globals
	pub fn globals(&self) -> usize {
		self.0
			.iter()
			.filter(|entry| matches!(entry.external(), &External::Global(_)))
			.count()
	}
}

impl Deserialize for ImportSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(ImportSection(read_entries(reader)?))
	}
}

impl Serialize for ImportSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<ImportEntry, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Section with function signatures definition.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct FunctionSection(Vec<Func>);

impl FunctionSection {
	///  New function signatures section with provided entries.
	pub fn with_entries(entries: Vec<Func>) -> Self {
		FunctionSection(entries)
	}

	/// List of all functions in the section, mutable.
	pub fn entries_mut(&mut self) -> &mut Vec<Func> {
		&mut self.0
	}

	/// List of all functions in the section.
	pub fn entries(&self) -> &[Func] {
		&self.0
	}
}

impl Deserialize for FunctionSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(FunctionSection(read_entries(reader)?))
	}
}

impl Serialize for FunctionSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list = CountedListWriter::<VarUint32, _>(
			data.len(),
			data.into_iter().map(|func| func.type_ref().into()),
		);
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Section with table definition (currently only one is allowed).
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TableSection(Vec<TableType>);

impl TableSection {
	/// Table entries.
	pub fn entries(&self) -> &[TableType] {
		&self.0
	}

	///  New table section with provided table entries.
	pub fn with_entries(entries: Vec<TableType>) -> Self {
		TableSection(entries)
	}

	/// Mutable table entries.
	pub fn entries_mut(&mut self) -> &mut Vec<TableType> {
		&mut self.0
	}
}

impl Deserialize for TableSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(TableSection(read_entries(reader)?))
	}
}

impl Serialize for TableSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<TableType, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Section with table definition (currently only one entry is allowed).
#[derive(Default, Debug, Clone, PartialEq)]
pub struct MemorySection(Vec<MemoryType>);

impl MemorySection {
	/// List of all memory entries in the section
	pub fn entries(&self) -> &[MemoryType] {
		&self.0
	}

	///  New memory section with memory types.
	pub fn with_entries(entries: Vec<MemoryType>) -> Self {
		MemorySection(entries)
	}

	/// Mutable list of all memory entries in the section.
	pub fn entries_mut(&mut self) -> &mut Vec<MemoryType> {
		&mut self.0
	}
}

impl Deserialize for MemorySection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(MemorySection(read_entries(reader)?))
	}
}

impl Serialize for MemorySection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<MemoryType, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Globals definition section.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct GlobalSection(Vec<GlobalEntry>);

impl GlobalSection {
	/// List of all global entries in the section.
	pub fn entries(&self) -> &[GlobalEntry] {
		&self.0
	}

	/// New global section from list of global entries.
	pub fn with_entries(entries: Vec<GlobalEntry>) -> Self {
		GlobalSection(entries)
	}

	/// List of all global entries in the section (mutable).
	pub fn entries_mut(&mut self) -> &mut Vec<GlobalEntry> {
		&mut self.0
	}
}

impl Deserialize for GlobalSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(GlobalSection(read_entries(reader)?))
	}
}

impl Serialize for GlobalSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<GlobalEntry, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// List of exports definition.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ExportSection(Vec<ExportEntry>);

impl ExportSection {
	/// List of all export entries in the section.
	pub fn entries(&self) -> &[ExportEntry] {
		&self.0
	}

	/// New export section from list of export entries.
	pub fn with_entries(entries: Vec<ExportEntry>) -> Self {
		ExportSection(entries)
	}

	/// List of all export entries in the section (mutable).
	pub fn entries_mut(&mut self) -> &mut Vec<ExportEntry> {
		&mut self.0
	}
}

impl Deserialize for ExportSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(ExportSection(read_entries(reader)?))
	}
}

impl Serialize for ExportSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<ExportEntry, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Section with function bodies of the module.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CodeSection(Vec<FuncBody>);

impl CodeSection {
	/// New code section with specified function bodies.
	pub fn with_bodies(bodies: Vec<FuncBody>) -> Self {
		CodeSection(bodies)
	}

	/// All function bodies in the section.
	pub fn bodies(&self) -> &[FuncBody] {
		&self.0
	}

	/// All function bodies in the section, mutable.
	pub fn bodies_mut(&mut self) -> &mut Vec<FuncBody> {
		&mut self.0
	}
}

impl Deserialize for CodeSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(CodeSection(read_entries(reader)?))
	}
}

impl Serialize for CodeSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<FuncBody, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Element entries section.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct ElementSection(Vec<ElementSegment>);

impl ElementSection {
	/// New elements section.
	pub fn with_entries(entries: Vec<ElementSegment>) -> Self {
		ElementSection(entries)
	}

	/// New elements entries in the section.
	pub fn entries(&self) -> &[ElementSegment] {
		&self.0
	}

	/// List of all data entries in the section (mutable).
	pub fn entries_mut(&mut self) -> &mut Vec<ElementSegment> {
		&mut self.0
	}
}

impl Deserialize for ElementSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(ElementSection(read_entries(reader)?))
	}
}

impl Serialize for ElementSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<ElementSegment, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

/// Data entries definitions.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct DataSection(Vec<DataSegment>);

impl DataSection {
	/// New data section.
	pub fn with_entries(entries: Vec<DataSegment>) -> Self {
		DataSection(entries)
	}

	/// List of all data entries in the section.
	pub fn entries(&self) -> &[DataSegment] {
		&self.0
	}

	/// List of all data entries in the section (mutable).
	pub fn entries_mut(&mut self) -> &mut Vec<DataSegment> {
		&mut self.0
	}
}

impl Deserialize for DataSection {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(DataSection(read_entries(reader)?))
	}
}

impl Serialize for DataSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut counted_writer = CountedWriter::new(writer);
		let data = self.0;
		let counted_list =
			CountedListWriter::<DataSegment, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(&mut counted_writer)?;
		counted_writer.done()?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {

	use super::{
		super::{
			deserialize_buffer, deserialize_file, serialize, BlockType, DataSegment,
			ElementSegment, FuncBody, InitExpr, Instructions, Local, ValueType,
		},
		CodeSection, DataSection, ElementSection, Section, Type, TypeSection,
	};

	#[test]
	fn import_section() {
		let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		let mut found = false;
		for section in module.sections() {
			if let Section::Import(ref import_section) = *section {
				assert_eq!(25, import_section.entries().len());
				found = true
			}
		}
		assert!(found, "There should be import section in test5.wasm");
	}

	fn functions_test_payload() -> &'static [u8] {
		&[
			// functions section id
			0x03u8, // functions section length
			0x87, 0x80, 0x80, 0x80, 0x0,  // number of functions
			0x04, // type reference 1
			0x01, // type reference 2
			0x86, 0x80, 0x00, // type reference 3
			0x09, // type reference 4
			0x33,
		]
	}

	#[test]
	fn fn_section_detect() {
		let section: Section =
			deserialize_buffer(functions_test_payload()).expect("section to be deserialized");

		match section {
			Section::Function(_) => {},
			_ => {
				panic!("Payload should be recognized as functions section")
			},
		}
	}

	#[test]
	fn fn_section_number() {
		let section: Section =
			deserialize_buffer(functions_test_payload()).expect("section to be deserialized");

		if let Section::Function(fn_section) = section {
			assert_eq!(4, fn_section.entries().len(), "There should be 4 functions total");
		}
	}

	#[test]
	fn fn_section_ref() {
		let section: Section =
			deserialize_buffer(functions_test_payload()).expect("section to be deserialized");

		if let Section::Function(fn_section) = section {
			assert_eq!(6, fn_section.entries()[1].type_ref());
		}
	}

	fn types_test_payload() -> &'static [u8] {
		&[
			// section length
			11,   // 2 functions
			2,    // func 1, form =1
			0x60, // param_count=1
			1,    // first param
			0x7e, // i64
			// no return params
			0x00, // func 2, form=1
			0x60, // param_count=2
			2,    // first param
			0x7e, // second param
			0x7d, // return param (is_present, param_type)
			0x01, 0x7e,
		]
	}

	#[test]
	fn type_section_len() {
		let type_section: TypeSection =
			deserialize_buffer(types_test_payload()).expect("type_section be deserialized");

		assert_eq!(type_section.types().len(), 2);
	}

	#[test]
	fn type_section_infer() {
		let type_section: TypeSection =
			deserialize_buffer(types_test_payload()).expect("type_section be deserialized");

		let Type::Function(ref t1) = type_section.types()[1];
		assert_eq!(vec![ValueType::I64], t1.results());
		assert_eq!(2, t1.params().len());
	}

	fn export_payload() -> &'static [u8] {
		&[
			// section id
			0x07, // section length
			28,   // 6 entries
			6,
			// func "A", index 6
			// [name_len(1-5 bytes), name_bytes(name_len, internal_kind(1byte), internal_index(1-5 bytes)])
			0x01, 0x41, 0x01, 0x86, 0x80, 0x00, // func "B", index 8
			0x01, 0x42, 0x01, 0x86, 0x00, // func "C", index 7
			0x01, 0x43, 0x01, 0x07, // memory "D", index 0
			0x01, 0x44, 0x02, 0x00, // func "E", index 1
			0x01, 0x45, 0x01, 0x01, // func "F", index 2
			0x01, 0x46, 0x01, 0x02,
		]
	}

	#[test]
	fn export_detect() {
		let section: Section =
			deserialize_buffer(export_payload()).expect("section to be deserialized");

		match section {
			Section::Export(_) => {},
			_ => {
				panic!("Payload should be recognized as export section")
			},
		}
	}

	fn code_payload() -> &'static [u8] {
		&[
			// sectionid
			0x0Au8, // section length, 32
			0x20,   // body count
			0x01,   // body 1, length 30
			0x1E, 0x01, 0x01, 0x7F, // local i32 (one collection of length one of type i32)
			0x02, 0x7F, // block i32
			0x23, 0x00, // get_global 0
			0x21, 0x01, // set_local 1
			0x23, 0x00, // get_global 0
			0x20, 0x00, // get_local 0
			0x6A, // i32.add
			0x24, 0x00, // set_global 0
			0x23, 0x00, // get_global 0
			0x41, 0x0F, // i32.const 15
			0x6A, // i32.add
			0x41, 0x70, // i32.const -16
			0x71, // i32.and
			0x24, 0x00, // set_global 0
			0x20, 0x01, // get_local 1
			0x0B, 0x0B,
		]
	}

	#[test]
	fn code_detect() {
		let section: Section =
			deserialize_buffer(code_payload()).expect("section to be deserialized");

		match section {
			Section::Code(_) => {},
			_ => {
				panic!("Payload should be recognized as a code section")
			},
		}
	}

	fn data_payload() -> &'static [u8] {
		&[
			0x0bu8, // section id
			20,     // 20 bytes overall
			0x01,   // number of segments
			0x00,   // index
			0x0b,   // just `end` op
			0x10,   // 16x 0x00
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00,
		]
	}

	#[test]
	fn data_section_ser() {
		let data_section = DataSection::with_entries(vec![DataSegment::new(
			0u32,
			Some(InitExpr::empty()),
			vec![0u8; 16],
		)]);

		let buf = serialize(data_section).expect("Data section to be serialized");

		assert_eq!(
			buf,
			vec![
				20u8, // 19 bytes overall
				0x01, // number of segments
				0x00, // index
				0x0b, // just `end` op
				16,   // value of length 16
				0x00, 0x00, 0x00, 0x00, // 16x 0x00 as in initialization
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
			]
		);
	}

	#[test]
	fn data_section_detect() {
		let section: Section =
			deserialize_buffer(data_payload()).expect("section to be deserialized");

		match section {
			Section::Data(_) => {},
			_ => {
				panic!("Payload should be recognized as a data section")
			},
		}
	}

	#[test]
	fn element_section_ser() {
		let element_section = ElementSection::with_entries(vec![ElementSegment::new(
			0u32,
			Some(InitExpr::empty()),
			vec![0u32; 4],
		)]);

		let buf = serialize(element_section).expect("Element section to be serialized");

		assert_eq!(
			buf,
			vec![
				8u8,  // 8 bytes overall
				0x01, // number of segments
				0x00, // index
				0x0b, // just `end` op
				0x04, // 4 elements
				0x00, 0x00, 0x00, 0x00 // 4x 0x00 as in initialization
			]
		);
	}

	#[test]
	fn code_section_ser() {
		use super::super::Instruction::*;

		let code_section = CodeSection::with_bodies(vec![FuncBody::new(
			vec![Local::new(1, ValueType::I32)],
			Instructions::new(vec![
				Block(BlockType::Value(ValueType::I32)),
				GetGlobal(0),
				End,
				End,
			]),
		)]);

		let buf = serialize(code_section).expect("Code section to be serialized");

		assert_eq!(
			buf,
			vec![
				11u8, // 11 bytes total section size
				0x01, // 1 function
				9,    //   function #1 total code size
				1,    //   1 local variable declaration
				1,    //      amount of variables
				0x7f, //      type of variable (7-bit, -0x01), negative
				0x02, //   block
				0x7f, //      block return type (7-bit, -0x01), negative
				0x23, 0x00, //      get_global(0)
				0x0b, //   block end
				0x0b, // function end
			]
		);
	}

	#[test]
	fn start_section() {
		let section: Section =
			deserialize_buffer(&[8u8, 1u8, 0u8]).expect("Start section to deserialize");
		if let Section::Start(_) = section {
		} else {
			panic!("Payload should be a start section");
		}

		let serialized = serialize(section).expect("Start section to successfully serializen");

		assert_eq!(serialized, vec![8u8, 1u8, 0u8]);
	}
}
