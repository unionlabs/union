use super::{
	Deserialize, Error, Serialize, TableElementType, Uint8, ValueType, VarInt7, VarUint1,
	VarUint32, VarUint7,
};
use crate::io;
use alloc::string::String;

const FLAG_HAS_MAX: u8 = 0x01;
#[cfg(feature = "atomics")]
const FLAG_SHARED: u8 = 0x02;

/// Global definition struct
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalType {
	content_type: ValueType,
	is_mutable: bool,
}

impl GlobalType {
	/// New global type
	pub fn new(content_type: ValueType, is_mutable: bool) -> Self {
		GlobalType { content_type, is_mutable }
	}

	/// Type of the global entry
	pub fn content_type(&self) -> ValueType {
		self.content_type
	}

	/// Is global entry is declared as mutable
	pub fn is_mutable(&self) -> bool {
		self.is_mutable
	}
}

impl Deserialize for GlobalType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let content_type = ValueType::deserialize(reader)?;
		let is_mutable = VarUint1::deserialize(reader)?;
		Ok(GlobalType { content_type, is_mutable: is_mutable.into() })
	}
}

impl Serialize for GlobalType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.content_type.serialize(writer)?;
		VarUint1::from(self.is_mutable).serialize(writer)?;
		Ok(())
	}
}

/// Table entry
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TableType {
	elem_type: TableElementType,
	limits: ResizableLimits,
}

impl TableType {
	/// New table definition
	pub fn new(min: u32, max: Option<u32>) -> Self {
		TableType { elem_type: TableElementType::AnyFunc, limits: ResizableLimits::new(min, max) }
	}

	/// Table memory specification
	pub fn limits(&self) -> &ResizableLimits {
		&self.limits
	}

	/// Table element type
	pub fn elem_type(&self) -> TableElementType {
		self.elem_type
	}
}

impl Deserialize for TableType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let elem_type = TableElementType::deserialize(reader)?;
		let limits = ResizableLimits::deserialize(reader)?;
		Ok(TableType { elem_type, limits })
	}
}

impl Serialize for TableType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.elem_type.serialize(writer)?;
		self.limits.serialize(writer)
	}
}

/// Memory and table limits.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ResizableLimits {
	initial: u32,
	maximum: Option<u32>,
	#[cfg(feature = "atomics")]
	shared: bool,
}

impl ResizableLimits {
	/// New memory limits definition.
	pub fn new(min: u32, max: Option<u32>) -> Self {
		ResizableLimits {
			initial: min,
			maximum: max,
			#[cfg(feature = "atomics")]
			shared: false,
		}
	}
	/// Initial size.
	pub fn initial(&self) -> u32 {
		self.initial
	}
	/// Maximum size.
	pub fn maximum(&self) -> Option<u32> {
		self.maximum
	}

	#[cfg(feature = "atomics")]
	/// Whether or not this is a shared array buffer.
	pub fn shared(&self) -> bool {
		self.shared
	}
}

impl Deserialize for ResizableLimits {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let flags: u8 = Uint8::deserialize(reader)?.into();
		match flags {
			// Default flags are always supported. This is simply: FLAG_HAS_MAX={true, false}.
			0x00 | 0x01 => {},

			// Atomics proposal introduce FLAG_SHARED (0x02). Shared memories can be used only
			// together with FLAG_HAS_MAX (0x01), hence 0x03.
			#[cfg(feature = "atomics")]
			0x03 => {},

			_ => return Err(Error::InvalidLimitsFlags(flags)),
		}

		let initial = VarUint32::deserialize(reader)?;
		let maximum = if flags & FLAG_HAS_MAX != 0 {
			Some(VarUint32::deserialize(reader)?.into())
		} else {
			None
		};

		Ok(ResizableLimits {
			initial: initial.into(),
			maximum,

			#[cfg(feature = "atomics")]
			shared: flags & FLAG_SHARED != 0,
		})
	}
}

impl Serialize for ResizableLimits {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let mut flags: u8 = 0;
		if self.maximum.is_some() {
			flags |= FLAG_HAS_MAX;
		}

		#[cfg(feature = "atomics")]
		{
			// If the atomics feature is enabled and if the shared flag is set, add logically
			// it to the flags.
			if self.shared {
				flags |= FLAG_SHARED;
			}
		}
		Uint8::from(flags).serialize(writer)?;
		VarUint32::from(self.initial).serialize(writer)?;
		if let Some(max) = self.maximum {
			VarUint32::from(max).serialize(writer)?;
		}
		Ok(())
	}
}

/// Memory entry.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MemoryType(ResizableLimits);

impl MemoryType {
	/// New memory definition
	pub fn new(min: u32, max: Option<u32>) -> Self {
		let r = ResizableLimits::new(min, max);
		MemoryType(r)
	}

	/// Set the `shared` flag that denotes a memory that can be shared between threads.
	///
	/// `false` by default. This is only available if the `atomics` feature is enabled.
	#[cfg(feature = "atomics")]
	pub fn set_shared(&mut self, shared: bool) {
		self.0.shared = shared;
	}

	/// Limits of the memory entry.
	pub fn limits(&self) -> &ResizableLimits {
		&self.0
	}
}

impl Deserialize for MemoryType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(MemoryType(ResizableLimits::deserialize(reader)?))
	}
}

impl Serialize for MemoryType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.0.serialize(writer)
	}
}

/// External to local binding.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum External {
	/// Binds to a function whose type is associated with the given index in the
	/// type section.
	Function(u32),
	/// Describes local table definition to be imported as.
	Table(TableType),
	/// Describes local memory definition to be imported as.
	Memory(MemoryType),
	/// Describes local global entry to be imported as.
	Global(GlobalType),
}

impl Deserialize for External {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let kind = VarUint7::deserialize(reader)?;
		match kind.into() {
			0x00 => Ok(External::Function(VarUint32::deserialize(reader)?.into())),
			0x01 => Ok(External::Table(TableType::deserialize(reader)?)),
			0x02 => Ok(External::Memory(MemoryType::deserialize(reader)?)),
			0x03 => Ok(External::Global(GlobalType::deserialize(reader)?)),
			_ => Err(Error::UnknownExternalKind(kind.into())),
		}
	}
}

impl Serialize for External {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::External::*;

		match self {
			Function(index) => {
				VarUint7::from(0x00).serialize(writer)?;
				VarUint32::from(index).serialize(writer)?;
			},
			Table(tt) => {
				VarInt7::from(0x01).serialize(writer)?;
				tt.serialize(writer)?;
			},
			Memory(mt) => {
				VarInt7::from(0x02).serialize(writer)?;
				mt.serialize(writer)?;
			},
			Global(gt) => {
				VarInt7::from(0x03).serialize(writer)?;
				gt.serialize(writer)?;
			},
		}

		Ok(())
	}
}

/// Import entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportEntry {
	module_str: String,
	field_str: String,
	external: External,
}

impl ImportEntry {
	/// New import entry.
	pub fn new(module_str: String, field_str: String, external: External) -> Self {
		ImportEntry { module_str, field_str, external }
	}

	/// Module reference of the import entry.
	pub fn module(&self) -> &str {
		&self.module_str
	}

	/// Module reference of the import entry (mutable).
	pub fn module_mut(&mut self) -> &mut String {
		&mut self.module_str
	}

	/// Field reference of the import entry.
	pub fn field(&self) -> &str {
		&self.field_str
	}

	/// Field reference of the import entry (mutable)
	pub fn field_mut(&mut self) -> &mut String {
		&mut self.field_str
	}

	/// Local binidng of the import entry.
	pub fn external(&self) -> &External {
		&self.external
	}

	/// Local binidng of the import entry (mutable)
	pub fn external_mut(&mut self) -> &mut External {
		&mut self.external
	}
}

impl Deserialize for ImportEntry {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let module_str = String::deserialize(reader)?;
		let field_str = String::deserialize(reader)?;
		let external = External::deserialize(reader)?;

		Ok(ImportEntry { module_str, field_str, external })
	}
}

impl Serialize for ImportEntry {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.module_str.serialize(writer)?;
		self.field_str.serialize(writer)?;
		self.external.serialize(writer)
	}
}
