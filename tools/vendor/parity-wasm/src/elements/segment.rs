use super::{CountedList, CountedListWriter, Deserialize, Error, InitExpr, Serialize, VarUint32};
use crate::io;
use alloc::vec::Vec;

#[cfg(feature = "bulk")]
const FLAG_MEMZERO: u32 = 0;
#[cfg(feature = "bulk")]
const FLAG_PASSIVE: u32 = 1;
#[cfg(feature = "bulk")]
const FLAG_MEM_NONZERO: u32 = 2;

#[cfg(feature = "reduced-stack-buffer")]
const VALUES_BUFFER_LENGTH: usize = 256;

#[cfg(not(feature = "reduced-stack-buffer"))]
const VALUES_BUFFER_LENGTH: usize = 16384;

/// Entry in the element section.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementSegment {
	index: u32,
	offset: Option<InitExpr>,
	members: Vec<u32>,

	#[cfg(feature = "bulk")]
	passive: bool,
}

impl ElementSegment {
	/// New element segment.
	pub fn new(index: u32, offset: Option<InitExpr>, members: Vec<u32>) -> Self {
		ElementSegment {
			index,
			offset,
			members,

			#[cfg(feature = "bulk")]
			passive: false,
		}
	}

	/// Sequence of function indices.
	pub fn members(&self) -> &[u32] {
		&self.members
	}

	/// Sequence of function indices (mutable)
	pub fn members_mut(&mut self) -> &mut Vec<u32> {
		&mut self.members
	}

	/// Table index (currently valid only value of `0`)
	pub fn index(&self) -> u32 {
		self.index
	}

	/// An i32 initializer expression that computes the offset at which to place the elements.
	///
	/// Note that this return `None` if the segment is `passive`.
	pub fn offset(&self) -> &Option<InitExpr> {
		&self.offset
	}

	/// An i32 initializer expression that computes the offset at which to place the elements (mutable)
	///
	/// Note that this return `None` if the segment is `passive`.
	pub fn offset_mut(&mut self) -> &mut Option<InitExpr> {
		&mut self.offset
	}
}

#[cfg(feature = "bulk")]
impl ElementSegment {
	/// Whether or not this table segment is "passive"
	pub fn passive(&self) -> bool {
		self.passive
	}

	/// Whether or not this table segment is "passive"
	pub fn passive_mut(&mut self) -> &mut bool {
		&mut self.passive
	}

	/// Set whether or not this table segment is "passive"
	pub fn set_passive(&mut self, passive: bool) {
		self.passive = passive;
	}
}

impl Deserialize for ElementSegment {
	type Error = Error;

	#[cfg(not(feature = "bulk"))]
	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let index: u32 = VarUint32::deserialize(reader)?.into();
		let offset = InitExpr::deserialize(reader)?;
		let members: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
			.into_inner()
			.into_iter()
			.map(Into::into)
			.collect();

		Ok(ElementSegment { index, offset: Some(offset), members })
	}

	#[cfg(feature = "bulk")]
	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		// This piece of data was treated as `index` [of the table], but was repurposed
		// for flags in bulk-memory operations proposal.
		let flags: u32 = VarUint32::deserialize(reader)?.into();
		let index = if flags == FLAG_MEMZERO || flags == FLAG_PASSIVE {
			0u32
		} else if flags == FLAG_MEM_NONZERO {
			VarUint32::deserialize(reader)?.into()
		} else {
			return Err(Error::InvalidSegmentFlags(flags))
		};
		let offset =
			if flags == FLAG_PASSIVE { None } else { Some(InitExpr::deserialize(reader)?) };

		let members: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
			.into_inner()
			.into_iter()
			.map(Into::into)
			.collect();

		Ok(ElementSegment { index, offset, members, passive: flags == FLAG_PASSIVE })
	}
}

impl Serialize for ElementSegment {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		#[cfg(feature = "bulk")]
		{
			if self.passive {
				VarUint32::from(FLAG_PASSIVE).serialize(writer)?;
			} else if self.index != 0 {
				VarUint32::from(FLAG_MEM_NONZERO).serialize(writer)?;
				VarUint32::from(self.index).serialize(writer)?;
			} else {
				VarUint32::from(FLAG_MEMZERO).serialize(writer)?;
			}
		}
		#[cfg(not(feature = "bulk"))]
		VarUint32::from(self.index).serialize(writer)?;

		if let Some(offset) = self.offset {
			offset.serialize(writer)?;
		}
		let data = self.members;
		let counted_list =
			CountedListWriter::<VarUint32, _>(data.len(), data.into_iter().map(Into::into));
		counted_list.serialize(writer)?;
		Ok(())
	}
}

/// Data segment definition.
#[derive(Clone, Debug, PartialEq)]
pub struct DataSegment {
	index: u32,
	offset: Option<InitExpr>,
	value: Vec<u8>,

	#[cfg(feature = "bulk")]
	passive: bool,
}

impl DataSegment {
	/// New data segments.
	pub fn new(index: u32, offset: Option<InitExpr>, value: Vec<u8>) -> Self {
		DataSegment {
			index,
			offset,
			value,

			#[cfg(feature = "bulk")]
			passive: false,
		}
	}

	/// Linear memory index (currently the only valid value is `0`).
	pub fn index(&self) -> u32 {
		self.index
	}

	/// An i32 initializer expression that computes the offset at which to place the data.
	///
	/// Note that this return `None` if the segment is `passive`.
	pub fn offset(&self) -> &Option<InitExpr> {
		&self.offset
	}

	/// An i32 initializer expression that computes the offset at which to place the data (mutable)
	///
	/// Note that this return `None` if the segment is `passive`.
	pub fn offset_mut(&mut self) -> &mut Option<InitExpr> {
		&mut self.offset
	}

	/// Initial value of the data segment.
	pub fn value(&self) -> &[u8] {
		&self.value
	}

	/// Initial value of the data segment (mutable).
	pub fn value_mut(&mut self) -> &mut Vec<u8> {
		&mut self.value
	}
}

#[cfg(feature = "bulk")]
impl DataSegment {
	/// Whether or not this data segment is "passive".
	pub fn passive(&self) -> bool {
		self.passive
	}

	/// Whether or not this data segment is "passive" (mutable).
	pub fn passive_mut(&mut self) -> &mut bool {
		&mut self.passive
	}

	/// Set whether or not this table segment is "passive"
	pub fn set_passive(&mut self, passive: bool) {
		self.passive = passive;
	}
}

impl Deserialize for DataSegment {
	type Error = Error;

	#[cfg(not(feature = "bulk"))]
	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let index = VarUint32::deserialize(reader)?;
		let offset = InitExpr::deserialize(reader)?;
		let value_len = u32::from(VarUint32::deserialize(reader)?) as usize;
		let value = buffered_read!(VALUES_BUFFER_LENGTH, value_len, reader);

		Ok(DataSegment { index: index.into(), offset: Some(offset), value })
	}

	#[cfg(feature = "bulk")]
	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let flags: u32 = VarUint32::deserialize(reader)?.into();
		let index = if flags == FLAG_MEMZERO || flags == FLAG_PASSIVE {
			0u32
		} else if flags == FLAG_MEM_NONZERO {
			VarUint32::deserialize(reader)?.into()
		} else {
			return Err(Error::InvalidSegmentFlags(flags))
		};
		let offset =
			if flags == FLAG_PASSIVE { None } else { Some(InitExpr::deserialize(reader)?) };
		let value_len = u32::from(VarUint32::deserialize(reader)?) as usize;
		let value = buffered_read!(VALUES_BUFFER_LENGTH, value_len, reader);

		Ok(DataSegment { index, offset, value, passive: flags == FLAG_PASSIVE })
	}
}

impl Serialize for DataSegment {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		#[cfg(feature = "bulk")]
		{
			if self.passive {
				VarUint32::from(FLAG_PASSIVE).serialize(writer)?;
			} else if self.index != 0 {
				VarUint32::from(FLAG_MEM_NONZERO).serialize(writer)?;
				VarUint32::from(self.index).serialize(writer)?;
			} else {
				VarUint32::from(FLAG_MEMZERO).serialize(writer)?;
			}
		}
		#[cfg(not(feature = "bulk"))]
		VarUint32::from(self.index).serialize(writer)?;

		if let Some(offset) = self.offset {
			offset.serialize(writer)?;
		}

		let value = self.value;
		VarUint32::from(value.len()).serialize(writer)?;
		writer.write(&value[..])?;
		Ok(())
	}
}
