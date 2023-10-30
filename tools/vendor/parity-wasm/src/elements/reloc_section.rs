use crate::io;
use alloc::{string::String, vec::Vec};

use super::{
	CountedList, CountedListWriter, CountedWriter, Deserialize, Error, Serialize, VarInt32,
	VarUint32, VarUint7,
};

const FUNCTION_INDEX_LEB: u8 = 0;
const TABLE_INDEX_SLEB: u8 = 1;
const TABLE_INDEX_I32: u8 = 2;
const MEMORY_ADDR_LEB: u8 = 3;
const MEMORY_ADDR_SLEB: u8 = 4;
const MEMORY_ADDR_I32: u8 = 5;
const TYPE_INDEX_LEB: u8 = 6;
const GLOBAL_INDEX_LEB: u8 = 7;

/// Relocation information.
#[derive(Clone, Debug, PartialEq)]
pub struct RelocSection {
	/// Name of this section.
	name: String,

	/// ID of the section containing the relocations described in this section.
	section_id: u32,

	/// Name of the section containing the relocations described in this section. Only set if section_id is 0.
	relocation_section_name: Option<String>,

	/// Relocation entries.
	entries: Vec<RelocationEntry>,
}

impl RelocSection {
	/// Name of this section.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Name of this section (mutable).
	pub fn name_mut(&mut self) -> &mut String {
		&mut self.name
	}

	/// ID of the section containing the relocations described in this section.
	pub fn section_id(&self) -> u32 {
		self.section_id
	}

	/// ID of the section containing the relocations described in this section (mutable).
	pub fn section_id_mut(&mut self) -> &mut u32 {
		&mut self.section_id
	}

	/// Name of the section containing the relocations described in this section.
	pub fn relocation_section_name(&self) -> Option<&str> {
		self.relocation_section_name.as_deref()
	}

	/// Name of the section containing the relocations described in this section (mutable).
	pub fn relocation_section_name_mut(&mut self) -> &mut Option<String> {
		&mut self.relocation_section_name
	}

	/// List of relocation entries.
	pub fn entries(&self) -> &[RelocationEntry] {
		&self.entries
	}

	/// List of relocation entries (mutable).
	pub fn entries_mut(&mut self) -> &mut Vec<RelocationEntry> {
		&mut self.entries
	}
}

impl RelocSection {
	/// Deserialize a reloc section.
	pub fn deserialize<R: io::Read>(name: String, rdr: &mut R) -> Result<Self, Error> {
		let section_id = VarUint32::deserialize(rdr)?.into();

		let relocation_section_name =
			if section_id == 0 { Some(String::deserialize(rdr)?) } else { None };

		let entries = CountedList::deserialize(rdr)?.into_inner();

		Ok(RelocSection { name, section_id, relocation_section_name, entries })
	}
}

impl Serialize for RelocSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Error> {
		let mut counted_writer = CountedWriter::new(wtr);

		self.name.serialize(&mut counted_writer)?;

		VarUint32::from(self.section_id).serialize(&mut counted_writer)?;

		if let Some(relocation_section_name) = self.relocation_section_name {
			relocation_section_name.serialize(&mut counted_writer)?;
		}

		let counted_list = CountedListWriter(self.entries.len(), self.entries.into_iter());
		counted_list.serialize(&mut counted_writer)?;

		counted_writer.done()?;

		Ok(())
	}
}

/// Relocation entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RelocationEntry {
	/// Function index.
	FunctionIndexLeb {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the function symbol in the symbol table.
		index: u32,
	},

	/// Function table index.
	TableIndexSleb {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the function symbol in the symbol table.
		index: u32,
	},

	/// Function table index.
	TableIndexI32 {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the function symbol in the symbol table.
		index: u32,
	},

	/// Linear memory index.
	MemoryAddressLeb {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the data symbol in the symbol table.
		index: u32,

		/// Addend to add to the address.
		addend: i32,
	},

	/// Linear memory index.
	MemoryAddressSleb {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the data symbol in the symbol table.
		index: u32,

		/// Addend to add to the address.
		addend: i32,
	},

	/// Linear memory index.
	MemoryAddressI32 {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the data symbol in the symbol table.
		index: u32,

		/// Addend to add to the address.
		addend: i32,
	},

	/// Type table index.
	TypeIndexLeb {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the type used.
		index: u32,
	},

	/// Global index.
	GlobalIndexLeb {
		/// Offset of the value to rewrite.
		offset: u32,

		/// Index of the global symbol in the symbol table.
		index: u32,
	},
}

impl Deserialize for RelocationEntry {
	type Error = Error;

	fn deserialize<R: io::Read>(rdr: &mut R) -> Result<Self, Self::Error> {
		match VarUint7::deserialize(rdr)?.into() {
			FUNCTION_INDEX_LEB => Ok(RelocationEntry::FunctionIndexLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			TABLE_INDEX_SLEB => Ok(RelocationEntry::TableIndexSleb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			TABLE_INDEX_I32 => Ok(RelocationEntry::TableIndexI32 {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			MEMORY_ADDR_LEB => Ok(RelocationEntry::MemoryAddressLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
				addend: VarInt32::deserialize(rdr)?.into(),
			}),

			MEMORY_ADDR_SLEB => Ok(RelocationEntry::MemoryAddressSleb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
				addend: VarInt32::deserialize(rdr)?.into(),
			}),

			MEMORY_ADDR_I32 => Ok(RelocationEntry::MemoryAddressI32 {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
				addend: VarInt32::deserialize(rdr)?.into(),
			}),

			TYPE_INDEX_LEB => Ok(RelocationEntry::TypeIndexLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			GLOBAL_INDEX_LEB => Ok(RelocationEntry::GlobalIndexLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			entry_type => Err(Error::UnknownValueType(entry_type as i8)),
		}
	}
}

impl Serialize for RelocationEntry {
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Error> {
		match self {
			RelocationEntry::FunctionIndexLeb { offset, index } => {
				VarUint7::from(FUNCTION_INDEX_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::TableIndexSleb { offset, index } => {
				VarUint7::from(TABLE_INDEX_SLEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::TableIndexI32 { offset, index } => {
				VarUint7::from(TABLE_INDEX_I32).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::MemoryAddressLeb { offset, index, addend } => {
				VarUint7::from(MEMORY_ADDR_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
				VarInt32::from(addend).serialize(wtr)?;
			},

			RelocationEntry::MemoryAddressSleb { offset, index, addend } => {
				VarUint7::from(MEMORY_ADDR_SLEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
				VarInt32::from(addend).serialize(wtr)?;
			},

			RelocationEntry::MemoryAddressI32 { offset, index, addend } => {
				VarUint7::from(MEMORY_ADDR_I32).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
				VarInt32::from(addend).serialize(wtr)?;
			},

			RelocationEntry::TypeIndexLeb { offset, index } => {
				VarUint7::from(TYPE_INDEX_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::GlobalIndexLeb { offset, index } => {
				VarUint7::from(GLOBAL_INDEX_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::{
		super::{deserialize_file, Section},
		RelocationEntry,
	};

	#[test]
	fn reloc_section() {
		let module = deserialize_file("./res/cases/v1/relocatable.wasm")
			.expect("Module should be deserialized")
			.parse_reloc()
			.expect("Reloc section should be deserialized");
		let mut found = false;
		for section in module.sections() {
			if let Section::Reloc(ref reloc_section) = *section {
				assert_eq!(
					vec![
						RelocationEntry::MemoryAddressSleb { offset: 4, index: 0, addend: 0 },
						RelocationEntry::FunctionIndexLeb { offset: 12, index: 0 },
					],
					reloc_section.entries()
				);
				found = true
			}
		}
		assert!(found, "There should be a reloc section in relocatable.wasm");
	}
}
