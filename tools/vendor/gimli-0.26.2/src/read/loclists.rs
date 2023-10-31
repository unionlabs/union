use crate::common::{
    DebugAddrBase, DebugAddrIndex, DebugLocListsBase, DebugLocListsIndex, DwarfFileType, Encoding,
    LocationListsOffset, SectionId,
};
use crate::constants;
use crate::endianity::Endianity;
use crate::read::{
    lists::ListsHeader, DebugAddr, EndianSlice, Error, Expression, Range, RawRange, Reader,
    ReaderOffset, ReaderOffsetId, Result, Section,
};

/// The raw contents of the `.debug_loc` section.
#[derive(Debug, Default, Clone, Copy)]
pub struct DebugLoc<R> {
    pub(crate) section: R,
}

impl<'input, Endian> DebugLoc<EndianSlice<'input, Endian>>
where
    Endian: Endianity,
{
    /// Construct a new `DebugLoc` instance from the data in the `.debug_loc`
    /// section.
    ///
    /// It is the caller's responsibility to read the `.debug_loc` section and
    /// present it as a `&[u8]` slice. That means using some ELF loader on
    /// Linux, a Mach-O loader on macOS, etc.
    ///
    /// ```
    /// use gimli::{DebugLoc, LittleEndian};
    ///
    /// # let buf = [0x00, 0x01, 0x02, 0x03];
    /// # let read_debug_loc_section_somehow = || &buf;
    /// let debug_loc = DebugLoc::new(read_debug_loc_section_somehow(), LittleEndian);
    /// ```
    pub fn new(section: &'input [u8], endian: Endian) -> Self {
        Self::from(EndianSlice::new(section, endian))
    }
}

impl<R> Section<R> for DebugLoc<R> {
    fn id() -> SectionId {
        SectionId::DebugLoc
    }

    fn reader(&self) -> &R {
        &self.section
    }
}

impl<R> From<R> for DebugLoc<R> {
    fn from(section: R) -> Self {
        DebugLoc { section }
    }
}

/// The `DebugLocLists` struct represents the DWARF data
/// found in the `.debug_loclists` section.
#[derive(Debug, Default, Clone, Copy)]
pub struct DebugLocLists<R> {
    section: R,
}

impl<'input, Endian> DebugLocLists<EndianSlice<'input, Endian>>
where
    Endian: Endianity,
{
    /// Construct a new `DebugLocLists` instance from the data in the `.debug_loclists`
    /// section.
    ///
    /// It is the caller's responsibility to read the `.debug_loclists` section and
    /// present it as a `&[u8]` slice. That means using some ELF loader on
    /// Linux, a Mach-O loader on macOS, etc.
    ///
    /// ```
    /// use gimli::{DebugLocLists, LittleEndian};
    ///
    /// # let buf = [0x00, 0x01, 0x02, 0x03];
    /// # let read_debug_loclists_section_somehow = || &buf;
    /// let debug_loclists = DebugLocLists::new(read_debug_loclists_section_somehow(), LittleEndian);
    /// ```
    pub fn new(section: &'input [u8], endian: Endian) -> Self {
        Self::from(EndianSlice::new(section, endian))
    }
}

impl<R> Section<R> for DebugLocLists<R> {
    fn id() -> SectionId {
        SectionId::DebugLocLists
    }

    fn reader(&self) -> &R {
        &self.section
    }
}

impl<R> From<R> for DebugLocLists<R> {
    fn from(section: R) -> Self {
        DebugLocLists { section }
    }
}

pub(crate) type LocListsHeader = ListsHeader;

impl<Offset> DebugLocListsBase<Offset>
where
    Offset: ReaderOffset,
{
    /// Returns a `DebugLocListsBase` with the default value of DW_AT_loclists_base
    /// for the given `Encoding` and `DwarfFileType`.
    pub fn default_for_encoding_and_file(
        encoding: Encoding,
        file_type: DwarfFileType,
    ) -> DebugLocListsBase<Offset> {
        if encoding.version >= 5 && file_type == DwarfFileType::Dwo {
            // In .dwo files, the compiler omits the DW_AT_loclists_base attribute (because there is
            // only a single unit in the file) but we must skip past the header, which the attribute
            // would normally do for us.
            DebugLocListsBase(Offset::from_u8(LocListsHeader::size_for_encoding(encoding)))
        } else {
            DebugLocListsBase(Offset::from_u8(0))
        }
    }
}

/// The DWARF data found in `.debug_loc` and `.debug_loclists` sections.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocationLists<R> {
    debug_loc: DebugLoc<R>,
    debug_loclists: DebugLocLists<R>,
}

impl<R> LocationLists<R> {
    /// Construct a new `LocationLists` instance from the data in the `.debug_loc` and
    /// `.debug_loclists` sections.
    pub fn new(debug_loc: DebugLoc<R>, debug_loclists: DebugLocLists<R>) -> LocationLists<R> {
        LocationLists {
            debug_loc,
            debug_loclists,
        }
    }
}

impl<T> LocationLists<T> {
    /// Create a `LocationLists` that references the data in `self`.
    ///
    /// This is useful when `R` implements `Reader` but `T` does not.
    ///
    /// ## Example Usage
    ///
    /// ```rust,no_run
    /// # let load_section = || unimplemented!();
    /// // Read the DWARF section into a `Vec` with whatever object loader you're using.
    /// let owned_section: gimli::LocationLists<Vec<u8>> = load_section();
    /// // Create a reference to the DWARF section.
    /// let section = owned_section.borrow(|section| {
    ///     gimli::EndianSlice::new(&section, gimli::LittleEndian)
    /// });
    /// ```
    pub fn borrow<'a, F, R>(&'a self, mut borrow: F) -> LocationLists<R>
    where
        F: FnMut(&'a T) -> R,
    {
        LocationLists {
            debug_loc: borrow(&self.debug_loc.section).into(),
            debug_loclists: borrow(&self.debug_loclists.section).into(),
        }
    }
}

impl<R: Reader> LocationLists<R> {
    /// Iterate over the `LocationListEntry`s starting at the given offset.
    ///
    /// The `unit_encoding` must match the compilation unit that the
    /// offset was contained in.
    ///
    /// The `base_address` should be obtained from the `DW_AT_low_pc` attribute in the
    /// `DW_TAG_compile_unit` entry for the compilation unit that contains this location
    /// list.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    pub fn locations(
        &self,
        offset: LocationListsOffset<R::Offset>,
        unit_encoding: Encoding,
        base_address: u64,
        debug_addr: &DebugAddr<R>,
        debug_addr_base: DebugAddrBase<R::Offset>,
    ) -> Result<LocListIter<R>> {
        Ok(LocListIter::new(
            self.raw_locations(offset, unit_encoding)?,
            base_address,
            debug_addr.clone(),
            debug_addr_base,
        ))
    }

    /// Similar to `locations`, but with special handling for .dwo files.
    /// This should only been used when this `LocationLists` was loaded from a
    /// .dwo file.
    pub fn locations_dwo(
        &self,
        offset: LocationListsOffset<R::Offset>,
        unit_encoding: Encoding,
        base_address: u64,
        debug_addr: &DebugAddr<R>,
        debug_addr_base: DebugAddrBase<R::Offset>,
    ) -> Result<LocListIter<R>> {
        Ok(LocListIter::new(
            self.raw_locations_dwo(offset, unit_encoding)?,
            base_address,
            debug_addr.clone(),
            debug_addr_base,
        ))
    }

    /// Iterate over the raw `LocationListEntry`s starting at the given offset.
    ///
    /// The `unit_encoding` must match the compilation unit that the
    /// offset was contained in.
    ///
    /// This iterator does not perform any processing of the location entries,
    /// such as handling base addresses.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    pub fn raw_locations(
        &self,
        offset: LocationListsOffset<R::Offset>,
        unit_encoding: Encoding,
    ) -> Result<RawLocListIter<R>> {
        let (mut input, format) = if unit_encoding.version <= 4 {
            (self.debug_loc.section.clone(), LocListsFormat::Bare)
        } else {
            (self.debug_loclists.section.clone(), LocListsFormat::LLE)
        };
        input.skip(offset.0)?;
        Ok(RawLocListIter::new(input, unit_encoding, format))
    }

    /// Similar to `raw_locations`, but with special handling for .dwo files.
    /// This should only been used when this `LocationLists` was loaded from a
    /// .dwo file.
    pub fn raw_locations_dwo(
        &self,
        offset: LocationListsOffset<R::Offset>,
        unit_encoding: Encoding,
    ) -> Result<RawLocListIter<R>> {
        let mut input = if unit_encoding.version <= 4 {
            // In the GNU split dwarf extension the locations are present in the
            // .debug_loc section but are encoded with the DW_LLE values used
            // for the DWARF 5 .debug_loclists section.
            self.debug_loc.section.clone()
        } else {
            self.debug_loclists.section.clone()
        };
        input.skip(offset.0)?;
        Ok(RawLocListIter::new(
            input,
            unit_encoding,
            LocListsFormat::LLE,
        ))
    }

    /// Returns the `.debug_loclists` offset at the given `base` and `index`.
    ///
    /// The `base` must be the `DW_AT_loclists_base` value from the compilation unit DIE.
    /// This is an offset that points to the first entry following the header.
    ///
    /// The `index` is the value of a `DW_FORM_loclistx` attribute.
    pub fn get_offset(
        &self,
        unit_encoding: Encoding,
        base: DebugLocListsBase<R::Offset>,
        index: DebugLocListsIndex<R::Offset>,
    ) -> Result<LocationListsOffset<R::Offset>> {
        let format = unit_encoding.format;
        let input = &mut self.debug_loclists.section.clone();
        input.skip(base.0)?;
        input.skip(R::Offset::from_u64(
            index.0.into_u64() * u64::from(format.word_size()),
        )?)?;
        input
            .read_offset(format)
            .map(|x| LocationListsOffset(base.0 + x))
    }

    /// Call `Reader::lookup_offset_id` for each section, and return the first match.
    pub fn lookup_offset_id(&self, id: ReaderOffsetId) -> Option<(SectionId, R::Offset)> {
        self.debug_loc
            .lookup_offset_id(id)
            .or_else(|| self.debug_loclists.lookup_offset_id(id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocListsFormat {
    /// The bare location list format used before DWARF 5.
    Bare,
    /// The DW_LLE encoded range list format used in DWARF 5 and the non-standard GNU
    /// split dwarf extension.
    LLE,
}

/// A raw iterator over a location list.
///
/// This iterator does not perform any processing of the location entries,
/// such as handling base addresses.
#[derive(Debug)]
pub struct RawLocListIter<R: Reader> {
    input: R,
    encoding: Encoding,
    format: LocListsFormat,
}

/// A raw entry in .debug_loclists.
#[derive(Clone, Debug)]
pub enum RawLocListEntry<R: Reader> {
    /// A location from DWARF version <= 4.
    AddressOrOffsetPair {
        /// Start of range. May be an address or an offset.
        begin: u64,
        /// End of range. May be an address or an offset.
        end: u64,
        /// expression
        data: Expression<R>,
    },
    /// DW_LLE_base_address
    BaseAddress {
        /// base address
        addr: u64,
    },
    /// DW_LLE_base_addressx
    BaseAddressx {
        /// base address
        addr: DebugAddrIndex<R::Offset>,
    },
    /// DW_LLE_startx_endx
    StartxEndx {
        /// start of range
        begin: DebugAddrIndex<R::Offset>,
        /// end of range
        end: DebugAddrIndex<R::Offset>,
        /// expression
        data: Expression<R>,
    },
    /// DW_LLE_startx_length
    StartxLength {
        /// start of range
        begin: DebugAddrIndex<R::Offset>,
        /// length of range
        length: u64,
        /// expression
        data: Expression<R>,
    },
    /// DW_LLE_offset_pair
    OffsetPair {
        /// start of range
        begin: u64,
        /// end of range
        end: u64,
        /// expression
        data: Expression<R>,
    },
    /// DW_LLE_default_location
    DefaultLocation {
        /// expression
        data: Expression<R>,
    },
    /// DW_LLE_start_end
    StartEnd {
        /// start of range
        begin: u64,
        /// end of range
        end: u64,
        /// expression
        data: Expression<R>,
    },
    /// DW_LLE_start_length
    StartLength {
        /// start of range
        begin: u64,
        /// length of range
        length: u64,
        /// expression
        data: Expression<R>,
    },
}

fn parse_data<R: Reader>(input: &mut R, encoding: Encoding) -> Result<Expression<R>> {
    if encoding.version >= 5 {
        let len = R::Offset::from_u64(input.read_uleb128()?)?;
        Ok(Expression(input.split(len)?))
    } else {
        // In the GNU split-dwarf extension this is a fixed 2 byte value.
        let len = R::Offset::from_u16(input.read_u16()?);
        Ok(Expression(input.split(len)?))
    }
}

impl<R: Reader> RawLocListEntry<R> {
    /// Parse a location list entry from `.debug_loclists`
    fn parse(input: &mut R, encoding: Encoding, format: LocListsFormat) -> Result<Option<Self>> {
        match format {
            LocListsFormat::Bare => {
                let range = RawRange::parse(input, encoding.address_size)?;
                return Ok(if range.is_end() {
                    None
                } else if range.is_base_address(encoding.address_size) {
                    Some(RawLocListEntry::BaseAddress { addr: range.end })
                } else {
                    let len = R::Offset::from_u16(input.read_u16()?);
                    let data = Expression(input.split(len)?);
                    Some(RawLocListEntry::AddressOrOffsetPair {
                        begin: range.begin,
                        end: range.end,
                        data,
                    })
                });
            }
            LocListsFormat::LLE => Ok(match constants::DwLle(input.read_u8()?) {
                constants::DW_LLE_end_of_list => None,
                constants::DW_LLE_base_addressx => Some(RawLocListEntry::BaseAddressx {
                    addr: DebugAddrIndex(input.read_uleb128().and_then(R::Offset::from_u64)?),
                }),
                constants::DW_LLE_startx_endx => Some(RawLocListEntry::StartxEndx {
                    begin: DebugAddrIndex(input.read_uleb128().and_then(R::Offset::from_u64)?),
                    end: DebugAddrIndex(input.read_uleb128().and_then(R::Offset::from_u64)?),
                    data: parse_data(input, encoding)?,
                }),
                constants::DW_LLE_startx_length => Some(RawLocListEntry::StartxLength {
                    begin: DebugAddrIndex(input.read_uleb128().and_then(R::Offset::from_u64)?),
                    length: if encoding.version >= 5 {
                        input.read_uleb128()?
                    } else {
                        // In the GNU split-dwarf extension this is a fixed 4 byte value.
                        input.read_u32()? as u64
                    },
                    data: parse_data(input, encoding)?,
                }),
                constants::DW_LLE_offset_pair => Some(RawLocListEntry::OffsetPair {
                    begin: input.read_uleb128()?,
                    end: input.read_uleb128()?,
                    data: parse_data(input, encoding)?,
                }),
                constants::DW_LLE_default_location => Some(RawLocListEntry::DefaultLocation {
                    data: parse_data(input, encoding)?,
                }),
                constants::DW_LLE_base_address => Some(RawLocListEntry::BaseAddress {
                    addr: input.read_address(encoding.address_size)?,
                }),
                constants::DW_LLE_start_end => Some(RawLocListEntry::StartEnd {
                    begin: input.read_address(encoding.address_size)?,
                    end: input.read_address(encoding.address_size)?,
                    data: parse_data(input, encoding)?,
                }),
                constants::DW_LLE_start_length => Some(RawLocListEntry::StartLength {
                    begin: input.read_address(encoding.address_size)?,
                    length: input.read_uleb128()?,
                    data: parse_data(input, encoding)?,
                }),
                _ => {
                    return Err(Error::InvalidAddressRange);
                }
            }),
        }
    }
}

impl<R: Reader> RawLocListIter<R> {
    /// Construct a `RawLocListIter`.
    fn new(input: R, encoding: Encoding, format: LocListsFormat) -> RawLocListIter<R> {
        RawLocListIter {
            input,
            encoding,
            format,
        }
    }

    /// Advance the iterator to the next location.
    pub fn next(&mut self) -> Result<Option<RawLocListEntry<R>>> {
        if self.input.is_empty() {
            return Ok(None);
        }

        match RawLocListEntry::parse(&mut self.input, self.encoding, self.format) {
            Ok(entry) => {
                if entry.is_none() {
                    self.input.empty();
                }
                Ok(entry)
            }
            Err(e) => {
                self.input.empty();
                Err(e)
            }
        }
    }
}

#[cfg(feature = "fallible-iterator")]
impl<R: Reader> fallible_iterator::FallibleIterator for RawLocListIter<R> {
    type Item = RawLocListEntry<R>;
    type Error = Error;

    fn next(&mut self) -> ::core::result::Result<Option<Self::Item>, Self::Error> {
        RawLocListIter::next(self)
    }
}

/// An iterator over a location list.
///
/// This iterator internally handles processing of base address selection entries
/// and list end entries.  Thus, it only returns location entries that are valid
/// and already adjusted for the base address.
#[derive(Debug)]
pub struct LocListIter<R: Reader> {
    raw: RawLocListIter<R>,
    base_address: u64,
    debug_addr: DebugAddr<R>,
    debug_addr_base: DebugAddrBase<R::Offset>,
}

impl<R: Reader> LocListIter<R> {
    /// Construct a `LocListIter`.
    fn new(
        raw: RawLocListIter<R>,
        base_address: u64,
        debug_addr: DebugAddr<R>,
        debug_addr_base: DebugAddrBase<R::Offset>,
    ) -> LocListIter<R> {
        LocListIter {
            raw,
            base_address,
            debug_addr,
            debug_addr_base,
        }
    }

    #[inline]
    fn get_address(&self, index: DebugAddrIndex<R::Offset>) -> Result<u64> {
        self.debug_addr
            .get_address(self.raw.encoding.address_size, self.debug_addr_base, index)
    }

    /// Advance the iterator to the next location.
    pub fn next(&mut self) -> Result<Option<LocationListEntry<R>>> {
        loop {
            let raw_loc = match self.raw.next()? {
                Some(loc) => loc,
                None => return Ok(None),
            };

            let (range, data) = match raw_loc {
                RawLocListEntry::BaseAddress { addr } => {
                    self.base_address = addr;
                    continue;
                }
                RawLocListEntry::BaseAddressx { addr } => {
                    self.base_address = self.get_address(addr)?;
                    continue;
                }
                RawLocListEntry::StartxEndx { begin, end, data } => {
                    let begin = self.get_address(begin)?;
                    let end = self.get_address(end)?;
                    (Range { begin, end }, data)
                }
                RawLocListEntry::StartxLength {
                    begin,
                    length,
                    data,
                } => {
                    let begin = self.get_address(begin)?;
                    let end = begin + length;
                    (Range { begin, end }, data)
                }
                RawLocListEntry::DefaultLocation { data } => (
                    Range {
                        begin: 0,
                        end: u64::max_value(),
                    },
                    data,
                ),
                RawLocListEntry::AddressOrOffsetPair { begin, end, data }
                | RawLocListEntry::OffsetPair { begin, end, data } => {
                    let mut range = Range { begin, end };
                    range.add_base_address(self.base_address, self.raw.encoding.address_size);
                    (range, data)
                }
                RawLocListEntry::StartEnd { begin, end, data } => (Range { begin, end }, data),
                RawLocListEntry::StartLength {
                    begin,
                    length,
                    data,
                } => (
                    Range {
                        begin,
                        end: begin + length,
                    },
                    data,
                ),
            };

            if range.begin > range.end {
                self.raw.input.empty();
                return Err(Error::InvalidLocationAddressRange);
            }

            return Ok(Some(LocationListEntry { range, data }));
        }
    }
}

#[cfg(feature = "fallible-iterator")]
impl<R: Reader> fallible_iterator::FallibleIterator for LocListIter<R> {
    type Item = LocationListEntry<R>;
    type Error = Error;

    fn next(&mut self) -> ::core::result::Result<Option<Self::Item>, Self::Error> {
        LocListIter::next(self)
    }
}

/// A location list entry from the `.debug_loc` or `.debug_loclists` sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationListEntry<R: Reader> {
    /// The address range that this location is valid for.
    pub range: Range,

    /// The data containing a single location description.
    pub data: Expression<R>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Format;
    use crate::endianity::LittleEndian;
    use crate::read::{EndianSlice, Range};
    use crate::test_util::GimliSectionMethods;
    use test_assembler::{Endian, Label, LabelMaker, Section};

    #[test]
    fn test_loclists_32() {
        let encoding = Encoding {
            format: Format::Dwarf32,
            version: 5,
            address_size: 4,
        };

        let section = Section::with_endian(Endian::Little)
            .L32(0x0300_0000)
            .L32(0x0301_0300)
            .L32(0x0301_0400)
            .L32(0x0301_0500);
        let buf = section.get_contents().unwrap();
        let debug_addr = &DebugAddr::from(EndianSlice::new(&buf, LittleEndian));
        let debug_addr_base = DebugAddrBase(0);

        let start = Label::new();
        let first = Label::new();
        let size = Label::new();
        #[rustfmt::skip]
        let section = Section::with_endian(Endian::Little)
            // Header
            .mark(&start)
            .L32(&size)
            .L16(encoding.version)
            .L8(encoding.address_size)
            .L8(0)
            .L32(0)
            .mark(&first)
            // OffsetPair
            .L8(4).uleb(0x10200).uleb(0x10300).uleb(4).L32(2)
            // A base address selection followed by an OffsetPair.
            .L8(6).L32(0x0200_0000)
            .L8(4).uleb(0x10400).uleb(0x10500).uleb(4).L32(3)
            // An empty OffsetPair followed by a normal OffsetPair.
            .L8(4).uleb(0x10600).uleb(0x10600).uleb(4).L32(4)
            .L8(4).uleb(0x10800).uleb(0x10900).uleb(4).L32(5)
            // A StartEnd
            .L8(7).L32(0x201_0a00).L32(0x201_0b00).uleb(4).L32(6)
            // A StartLength
            .L8(8).L32(0x201_0c00).uleb(0x100).uleb(4).L32(7)
            // An OffsetPair that starts at 0.
            .L8(4).uleb(0).uleb(1).uleb(4).L32(8)
            // An OffsetPair that ends at -1.
            .L8(6).L32(0)
            .L8(4).uleb(0).uleb(0xffff_ffff).uleb(4).L32(9)
            // A DefaultLocation
            .L8(5).uleb(4).L32(10)
            // A BaseAddressx + OffsetPair
            .L8(1).uleb(0)
            .L8(4).uleb(0x10100).uleb(0x10200).uleb(4).L32(11)
            // A StartxEndx
            .L8(2).uleb(1).uleb(2).uleb(4).L32(12)
            // A StartxLength
            .L8(3).uleb(3).uleb(0x100).uleb(4).L32(13)
            // A range end.
            .L8(0)
            // Some extra data.
            .L32(0xffff_ffff);
        size.set_const((&section.here() - &start - 4) as u64);

        let buf = section.get_contents().unwrap();
        let debug_loc = DebugLoc::new(&[], LittleEndian);
        let debug_loclists = DebugLocLists::new(&buf, LittleEndian);
        let loclists = LocationLists::new(debug_loc, debug_loclists);
        let offset = LocationListsOffset((&first - &start) as usize);
        let mut locations = loclists
            .locations(offset, encoding, 0x0100_0000, debug_addr, debug_addr_base)
            .unwrap();

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0101_0200,
                    end: 0x0101_0300,
                },
                data: Expression(EndianSlice::new(&[2, 0, 0, 0], LittleEndian)),
            }))
        );

        // A base address selection followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0400,
                    end: 0x0201_0500,
                },
                data: Expression(EndianSlice::new(&[3, 0, 0, 0], LittleEndian)),
            }))
        );

        // An empty location range followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0600,
                    end: 0x0201_0600,
                },
                data: Expression(EndianSlice::new(&[4, 0, 0, 0], LittleEndian)),
            }))
        );
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0800,
                    end: 0x0201_0900,
                },
                data: Expression(EndianSlice::new(&[5, 0, 0, 0], LittleEndian)),
            }))
        );

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0a00,
                    end: 0x0201_0b00,
                },
                data: Expression(EndianSlice::new(&[6, 0, 0, 0], LittleEndian)),
            }))
        );

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0c00,
                    end: 0x0201_0d00,
                },
                data: Expression(EndianSlice::new(&[7, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that starts at 0.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0200_0000,
                    end: 0x0200_0001,
                },
                data: Expression(EndianSlice::new(&[8, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that ends at -1.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0000_0000,
                    end: 0xffff_ffff,
                },
                data: Expression(EndianSlice::new(&[9, 0, 0, 0], LittleEndian)),
            }))
        );

        // A DefaultLocation.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0,
                    end: u64::max_value(),
                },
                data: Expression(EndianSlice::new(&[10, 0, 0, 0], LittleEndian)),
            }))
        );

        // A BaseAddressx + OffsetPair
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0301_0100,
                    end: 0x0301_0200,
                },
                data: Expression(EndianSlice::new(&[11, 0, 0, 0], LittleEndian)),
            }))
        );

        // A StartxEndx
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0301_0300,
                    end: 0x0301_0400,
                },
                data: Expression(EndianSlice::new(&[12, 0, 0, 0], LittleEndian)),
            }))
        );

        // A StartxLength
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0301_0500,
                    end: 0x0301_0600,
                },
                data: Expression(EndianSlice::new(&[13, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location list end.
        assert_eq!(locations.next(), Ok(None));

        // An offset at the end of buf.
        let mut locations = loclists
            .locations(
                LocationListsOffset(buf.len()),
                encoding,
                0x0100_0000,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(locations.next(), Ok(None));
    }

    #[test]
    fn test_loclists_64() {
        let encoding = Encoding {
            format: Format::Dwarf64,
            version: 5,
            address_size: 8,
        };

        let section = Section::with_endian(Endian::Little)
            .L64(0x0300_0000)
            .L64(0x0301_0300)
            .L64(0x0301_0400)
            .L64(0x0301_0500);
        let buf = section.get_contents().unwrap();
        let debug_addr = &DebugAddr::from(EndianSlice::new(&buf, LittleEndian));
        let debug_addr_base = DebugAddrBase(0);

        let start = Label::new();
        let first = Label::new();
        let size = Label::new();
        #[rustfmt::skip]
        let section = Section::with_endian(Endian::Little)
            // Header
            .mark(&start)
            .L32(0xffff_ffff)
            .L64(&size)
            .L16(encoding.version)
            .L8(encoding.address_size)
            .L8(0)
            .L32(0)
            .mark(&first)
            // OffsetPair
            .L8(4).uleb(0x10200).uleb(0x10300).uleb(4).L32(2)
            // A base address selection followed by an OffsetPair.
            .L8(6).L64(0x0200_0000)
            .L8(4).uleb(0x10400).uleb(0x10500).uleb(4).L32(3)
            // An empty OffsetPair followed by a normal OffsetPair.
            .L8(4).uleb(0x10600).uleb(0x10600).uleb(4).L32(4)
            .L8(4).uleb(0x10800).uleb(0x10900).uleb(4).L32(5)
            // A StartEnd
            .L8(7).L64(0x201_0a00).L64(0x201_0b00).uleb(4).L32(6)
            // A StartLength
            .L8(8).L64(0x201_0c00).uleb(0x100).uleb(4).L32(7)
            // An OffsetPair that starts at 0.
            .L8(4).uleb(0).uleb(1).uleb(4).L32(8)
            // An OffsetPair that ends at -1.
            .L8(6).L64(0)
            .L8(4).uleb(0).uleb(0xffff_ffff).uleb(4).L32(9)
            // A DefaultLocation
            .L8(5).uleb(4).L32(10)
            // A BaseAddressx + OffsetPair
            .L8(1).uleb(0)
            .L8(4).uleb(0x10100).uleb(0x10200).uleb(4).L32(11)
            // A StartxEndx
            .L8(2).uleb(1).uleb(2).uleb(4).L32(12)
            // A StartxLength
            .L8(3).uleb(3).uleb(0x100).uleb(4).L32(13)
            // A range end.
            .L8(0)
            // Some extra data.
            .L32(0xffff_ffff);
        size.set_const((&section.here() - &start - 12) as u64);

        let buf = section.get_contents().unwrap();
        let debug_loc = DebugLoc::new(&[], LittleEndian);
        let debug_loclists = DebugLocLists::new(&buf, LittleEndian);
        let loclists = LocationLists::new(debug_loc, debug_loclists);
        let offset = LocationListsOffset((&first - &start) as usize);
        let mut locations = loclists
            .locations(offset, encoding, 0x0100_0000, debug_addr, debug_addr_base)
            .unwrap();

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0101_0200,
                    end: 0x0101_0300,
                },
                data: Expression(EndianSlice::new(&[2, 0, 0, 0], LittleEndian)),
            }))
        );

        // A base address selection followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0400,
                    end: 0x0201_0500,
                },
                data: Expression(EndianSlice::new(&[3, 0, 0, 0], LittleEndian)),
            }))
        );

        // An empty location range followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0600,
                    end: 0x0201_0600,
                },
                data: Expression(EndianSlice::new(&[4, 0, 0, 0], LittleEndian)),
            }))
        );
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0800,
                    end: 0x0201_0900,
                },
                data: Expression(EndianSlice::new(&[5, 0, 0, 0], LittleEndian)),
            }))
        );

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0a00,
                    end: 0x0201_0b00,
                },
                data: Expression(EndianSlice::new(&[6, 0, 0, 0], LittleEndian)),
            }))
        );

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0c00,
                    end: 0x0201_0d00,
                },
                data: Expression(EndianSlice::new(&[7, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that starts at 0.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0200_0000,
                    end: 0x0200_0001,
                },
                data: Expression(EndianSlice::new(&[8, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that ends at -1.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0000_0000,
                    end: 0xffff_ffff,
                },
                data: Expression(EndianSlice::new(&[9, 0, 0, 0], LittleEndian)),
            }))
        );

        // A DefaultLocation.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0,
                    end: u64::max_value(),
                },
                data: Expression(EndianSlice::new(&[10, 0, 0, 0], LittleEndian)),
            }))
        );

        // A BaseAddressx + OffsetPair
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0301_0100,
                    end: 0x0301_0200,
                },
                data: Expression(EndianSlice::new(&[11, 0, 0, 0], LittleEndian)),
            }))
        );

        // A StartxEndx
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0301_0300,
                    end: 0x0301_0400,
                },
                data: Expression(EndianSlice::new(&[12, 0, 0, 0], LittleEndian)),
            }))
        );

        // A StartxLength
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0301_0500,
                    end: 0x0301_0600,
                },
                data: Expression(EndianSlice::new(&[13, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location list end.
        assert_eq!(locations.next(), Ok(None));

        // An offset at the end of buf.
        let mut locations = loclists
            .locations(
                LocationListsOffset(buf.len()),
                encoding,
                0x0100_0000,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(locations.next(), Ok(None));
    }

    #[test]
    fn test_location_list_32() {
        let start = Label::new();
        let first = Label::new();
        #[rustfmt::skip]
        let section = Section::with_endian(Endian::Little)
            // A location before the offset.
            .mark(&start)
            .L32(0x10000).L32(0x10100).L16(4).L32(1)
            .mark(&first)
            // A normal location.
            .L32(0x10200).L32(0x10300).L16(4).L32(2)
            // A base address selection followed by a normal location.
            .L32(0xffff_ffff).L32(0x0200_0000)
            .L32(0x10400).L32(0x10500).L16(4).L32(3)
            // An empty location range followed by a normal location.
            .L32(0x10600).L32(0x10600).L16(4).L32(4)
            .L32(0x10800).L32(0x10900).L16(4).L32(5)
            // A location range that starts at 0.
            .L32(0).L32(1).L16(4).L32(6)
            // A location range that ends at -1.
            .L32(0xffff_ffff).L32(0x0000_0000)
            .L32(0).L32(0xffff_ffff).L16(4).L32(7)
            // A location list end.
            .L32(0).L32(0)
            // Some extra data.
            .L32(0);

        let buf = section.get_contents().unwrap();
        let debug_loc = DebugLoc::new(&buf, LittleEndian);
        let debug_loclists = DebugLocLists::new(&[], LittleEndian);
        let loclists = LocationLists::new(debug_loc, debug_loclists);
        let offset = LocationListsOffset((&first - &start) as usize);
        let debug_addr = &DebugAddr::from(EndianSlice::new(&[], LittleEndian));
        let debug_addr_base = DebugAddrBase(0);
        let encoding = Encoding {
            format: Format::Dwarf32,
            version: 4,
            address_size: 4,
        };
        let mut locations = loclists
            .locations(offset, encoding, 0x0100_0000, debug_addr, debug_addr_base)
            .unwrap();

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0101_0200,
                    end: 0x0101_0300,
                },
                data: Expression(EndianSlice::new(&[2, 0, 0, 0], LittleEndian)),
            }))
        );

        // A base address selection followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0400,
                    end: 0x0201_0500,
                },
                data: Expression(EndianSlice::new(&[3, 0, 0, 0], LittleEndian)),
            }))
        );

        // An empty location range followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0600,
                    end: 0x0201_0600,
                },
                data: Expression(EndianSlice::new(&[4, 0, 0, 0], LittleEndian)),
            }))
        );
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0800,
                    end: 0x0201_0900,
                },
                data: Expression(EndianSlice::new(&[5, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that starts at 0.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0200_0000,
                    end: 0x0200_0001,
                },
                data: Expression(EndianSlice::new(&[6, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that ends at -1.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0000_0000,
                    end: 0xffff_ffff,
                },
                data: Expression(EndianSlice::new(&[7, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location list end.
        assert_eq!(locations.next(), Ok(None));

        // An offset at the end of buf.
        let mut locations = loclists
            .locations(
                LocationListsOffset(buf.len()),
                encoding,
                0x0100_0000,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(locations.next(), Ok(None));
    }

    #[test]
    fn test_location_list_64() {
        let start = Label::new();
        let first = Label::new();
        #[rustfmt::skip]
        let section = Section::with_endian(Endian::Little)
            // A location before the offset.
            .mark(&start)
            .L64(0x10000).L64(0x10100).L16(4).L32(1)
            .mark(&first)
            // A normal location.
            .L64(0x10200).L64(0x10300).L16(4).L32(2)
            // A base address selection followed by a normal location.
            .L64(0xffff_ffff_ffff_ffff).L64(0x0200_0000)
            .L64(0x10400).L64(0x10500).L16(4).L32(3)
            // An empty location range followed by a normal location.
            .L64(0x10600).L64(0x10600).L16(4).L32(4)
            .L64(0x10800).L64(0x10900).L16(4).L32(5)
            // A location range that starts at 0.
            .L64(0).L64(1).L16(4).L32(6)
            // A location range that ends at -1.
            .L64(0xffff_ffff_ffff_ffff).L64(0x0000_0000)
            .L64(0).L64(0xffff_ffff_ffff_ffff).L16(4).L32(7)
            // A location list end.
            .L64(0).L64(0)
            // Some extra data.
            .L64(0);

        let buf = section.get_contents().unwrap();
        let debug_loc = DebugLoc::new(&buf, LittleEndian);
        let debug_loclists = DebugLocLists::new(&[], LittleEndian);
        let loclists = LocationLists::new(debug_loc, debug_loclists);
        let offset = LocationListsOffset((&first - &start) as usize);
        let debug_addr = &DebugAddr::from(EndianSlice::new(&[], LittleEndian));
        let debug_addr_base = DebugAddrBase(0);
        let encoding = Encoding {
            format: Format::Dwarf64,
            version: 4,
            address_size: 8,
        };
        let mut locations = loclists
            .locations(offset, encoding, 0x0100_0000, debug_addr, debug_addr_base)
            .unwrap();

        // A normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0101_0200,
                    end: 0x0101_0300,
                },
                data: Expression(EndianSlice::new(&[2, 0, 0, 0], LittleEndian)),
            }))
        );

        // A base address selection followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0400,
                    end: 0x0201_0500,
                },
                data: Expression(EndianSlice::new(&[3, 0, 0, 0], LittleEndian)),
            }))
        );

        // An empty location range followed by a normal location.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0600,
                    end: 0x0201_0600,
                },
                data: Expression(EndianSlice::new(&[4, 0, 0, 0], LittleEndian)),
            }))
        );
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0201_0800,
                    end: 0x0201_0900,
                },
                data: Expression(EndianSlice::new(&[5, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that starts at 0.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0200_0000,
                    end: 0x0200_0001,
                },
                data: Expression(EndianSlice::new(&[6, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location range that ends at -1.
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0,
                    end: 0xffff_ffff_ffff_ffff,
                },
                data: Expression(EndianSlice::new(&[7, 0, 0, 0], LittleEndian)),
            }))
        );

        // A location list end.
        assert_eq!(locations.next(), Ok(None));

        // An offset at the end of buf.
        let mut locations = loclists
            .locations(
                LocationListsOffset(buf.len()),
                encoding,
                0x0100_0000,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(locations.next(), Ok(None));
    }

    #[test]
    fn test_locations_invalid() {
        #[rustfmt::skip]
        let section = Section::with_endian(Endian::Little)
            // An invalid location range.
            .L32(0x20000).L32(0x10000).L16(4).L32(1)
            // An invalid range after wrapping.
            .L32(0x20000).L32(0xff01_0000).L16(4).L32(2);

        let buf = section.get_contents().unwrap();
        let debug_loc = DebugLoc::new(&buf, LittleEndian);
        let debug_loclists = DebugLocLists::new(&[], LittleEndian);
        let loclists = LocationLists::new(debug_loc, debug_loclists);
        let debug_addr = &DebugAddr::from(EndianSlice::new(&[], LittleEndian));
        let debug_addr_base = DebugAddrBase(0);
        let encoding = Encoding {
            format: Format::Dwarf32,
            version: 4,
            address_size: 4,
        };

        // An invalid location range.
        let mut locations = loclists
            .locations(
                LocationListsOffset(0x0),
                encoding,
                0x0100_0000,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(locations.next(), Err(Error::InvalidLocationAddressRange));

        // An invalid location range after wrapping.
        let mut locations = loclists
            .locations(
                LocationListsOffset(14),
                encoding,
                0x0100_0000,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(locations.next(), Err(Error::InvalidLocationAddressRange));

        // An invalid offset.
        match loclists.locations(
            LocationListsOffset(buf.len() + 1),
            encoding,
            0x0100_0000,
            debug_addr,
            debug_addr_base,
        ) {
            Err(Error::UnexpectedEof(_)) => {}
            otherwise => panic!("Unexpected result: {:?}", otherwise),
        }
    }

    #[test]
    fn test_get_offset() {
        for format in vec![Format::Dwarf32, Format::Dwarf64] {
            let encoding = Encoding {
                format,
                version: 5,
                address_size: 4,
            };

            let zero = Label::new();
            let length = Label::new();
            let start = Label::new();
            let first = Label::new();
            let end = Label::new();
            let mut section = Section::with_endian(Endian::Little)
                .mark(&zero)
                .initial_length(format, &length, &start)
                .D16(encoding.version)
                .D8(encoding.address_size)
                .D8(0)
                .D32(20)
                .mark(&first);
            for i in 0..20 {
                section = section.word(format.word_size(), 1000 + i);
            }
            section = section.mark(&end);
            length.set_const((&end - &start) as u64);
            let section = section.get_contents().unwrap();

            let debug_loc = DebugLoc::from(EndianSlice::new(&[], LittleEndian));
            let debug_loclists = DebugLocLists::from(EndianSlice::new(&section, LittleEndian));
            let locations = LocationLists::new(debug_loc, debug_loclists);

            let base = DebugLocListsBase((&first - &zero) as usize);
            assert_eq!(
                locations.get_offset(encoding, base, DebugLocListsIndex(0)),
                Ok(LocationListsOffset(base.0 + 1000))
            );
            assert_eq!(
                locations.get_offset(encoding, base, DebugLocListsIndex(19)),
                Ok(LocationListsOffset(base.0 + 1019))
            );
        }
    }

    #[test]
    fn test_loclists_gnu_v4_split_dwarf() {
        #[rustfmt::skip]
        let buf = [
            0x03, // DW_LLE_startx_length
            0x00, // ULEB encoded b7
            0x08, 0x00, 0x00, 0x00, // Fixed 4 byte length of 8
            0x03, 0x00, // Fixed two byte length of the location
            0x11, 0x00, // DW_OP_constu 0
            0x9f, // DW_OP_stack_value
            // Padding data
            //0x99, 0x99, 0x99, 0x99
        ];
        let data_buf = [0x11, 0x00, 0x9f];
        let expected_data = EndianSlice::new(&data_buf, LittleEndian);
        let debug_loc = DebugLoc::new(&buf, LittleEndian);
        let debug_loclists = DebugLocLists::new(&[], LittleEndian);
        let loclists = LocationLists::new(debug_loc, debug_loclists);
        let debug_addr =
            &DebugAddr::from(EndianSlice::new(&[0x01, 0x02, 0x03, 0x04], LittleEndian));
        let debug_addr_base = DebugAddrBase(0);
        let encoding = Encoding {
            format: Format::Dwarf32,
            version: 4,
            address_size: 4,
        };

        // An invalid location range.
        let mut locations = loclists
            .locations_dwo(
                LocationListsOffset(0x0),
                encoding,
                0,
                debug_addr,
                debug_addr_base,
            )
            .unwrap();
        assert_eq!(
            locations.next(),
            Ok(Some(LocationListEntry {
                range: Range {
                    begin: 0x0403_0201,
                    end: 0x0403_0209
                },
                data: Expression(expected_data),
            }))
        );
    }
}
