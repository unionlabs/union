#[cfg(feature = "read")]
use alloc::vec::Vec;

use core::cmp::{Ord, Ordering};
use core::fmt::{self, Debug};
use core::iter::FromIterator;
use core::mem;
use core::num::Wrapping;

use super::util::{ArrayLike, ArrayVec};
use crate::common::{DebugFrameOffset, EhFrameOffset, Encoding, Format, Register, SectionId};
use crate::constants::{self, DwEhPe};
use crate::endianity::Endianity;
use crate::read::{
    EndianSlice, Error, Expression, Reader, ReaderOffset, Result, Section, StoreOnHeap,
};

/// `DebugFrame` contains the `.debug_frame` section's frame unwinding
/// information required to unwind to and recover registers from older frames on
/// the stack. For example, this is useful for a debugger that wants to print
/// locals in a backtrace.
///
/// Most interesting methods are defined in the
/// [`UnwindSection`](trait.UnwindSection.html) trait.
///
/// ### Differences between `.debug_frame` and `.eh_frame`
///
/// While the `.debug_frame` section's information has a lot of overlap with the
/// `.eh_frame` section's information, the `.eh_frame` information tends to only
/// encode the subset of information needed for exception handling. Often, only
/// one of `.eh_frame` or `.debug_frame` will be present in an object file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugFrame<R: Reader> {
    section: R,
    address_size: u8,
    segment_size: u8,
}

impl<R: Reader> DebugFrame<R> {
    /// Set the size of a target address in bytes.
    ///
    /// This defaults to the native word size.
    /// This is only used if the CIE version is less than 4.
    pub fn set_address_size(&mut self, address_size: u8) {
        self.address_size = address_size
    }

    /// Set the size of a segment selector in bytes.
    ///
    /// This defaults to 0.
    /// This is only used if the CIE version is less than 4.
    pub fn set_segment_size(&mut self, segment_size: u8) {
        self.segment_size = segment_size
    }
}

impl<'input, Endian> DebugFrame<EndianSlice<'input, Endian>>
where
    Endian: Endianity,
{
    /// Construct a new `DebugFrame` instance from the data in the
    /// `.debug_frame` section.
    ///
    /// It is the caller's responsibility to read the section and present it as
    /// a `&[u8]` slice. That means using some ELF loader on Linux, a Mach-O
    /// loader on macOS, etc.
    ///
    /// ```
    /// use gimli::{DebugFrame, NativeEndian};
    ///
    /// // Use with `.debug_frame`
    /// # let buf = [0x00, 0x01, 0x02, 0x03];
    /// # let read_debug_frame_section_somehow = || &buf;
    /// let debug_frame = DebugFrame::new(read_debug_frame_section_somehow(), NativeEndian);
    /// ```
    pub fn new(section: &'input [u8], endian: Endian) -> Self {
        Self::from(EndianSlice::new(section, endian))
    }
}

impl<R: Reader> Section<R> for DebugFrame<R> {
    fn id() -> SectionId {
        SectionId::DebugFrame
    }

    fn reader(&self) -> &R {
        &self.section
    }
}

impl<R: Reader> From<R> for DebugFrame<R> {
    fn from(section: R) -> Self {
        // Default to no segments and native word size.
        DebugFrame {
            section,
            address_size: mem::size_of::<usize>() as u8,
            segment_size: 0,
        }
    }
}

/// `EhFrameHdr` contains the information about the `.eh_frame_hdr` section.
///
/// A pointer to the start of the `.eh_frame` data, and optionally, a binary
/// search table of pointers to the `.eh_frame` records that are found in this section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EhFrameHdr<R: Reader>(R);

/// `ParsedEhFrameHdr` contains the parsed information from the `.eh_frame_hdr` section.
#[derive(Clone, Debug)]
pub struct ParsedEhFrameHdr<R: Reader> {
    address_size: u8,
    section: R,

    eh_frame_ptr: Pointer,
    fde_count: u64,
    table_enc: DwEhPe,
    table: R,
}

impl<'input, Endian> EhFrameHdr<EndianSlice<'input, Endian>>
where
    Endian: Endianity,
{
    /// Constructs a new `EhFrameHdr` instance from the data in the `.eh_frame_hdr` section.
    pub fn new(section: &'input [u8], endian: Endian) -> Self {
        Self::from(EndianSlice::new(section, endian))
    }
}

impl<R: Reader> EhFrameHdr<R> {
    /// Parses this `EhFrameHdr` to a `ParsedEhFrameHdr`.
    pub fn parse(&self, bases: &BaseAddresses, address_size: u8) -> Result<ParsedEhFrameHdr<R>> {
        let mut reader = self.0.clone();
        let version = reader.read_u8()?;
        if version != 1 {
            return Err(Error::UnknownVersion(u64::from(version)));
        }

        let eh_frame_ptr_enc = parse_pointer_encoding(&mut reader)?;
        let fde_count_enc = parse_pointer_encoding(&mut reader)?;
        let table_enc = parse_pointer_encoding(&mut reader)?;

        let parameters = PointerEncodingParameters {
            bases: &bases.eh_frame_hdr,
            func_base: None,
            address_size,
            section: &self.0,
        };

        // Omitting this pointer is not valid (defeats the purpose of .eh_frame_hdr entirely)
        if eh_frame_ptr_enc == constants::DW_EH_PE_omit {
            return Err(Error::CannotParseOmitPointerEncoding);
        }
        let eh_frame_ptr = parse_encoded_pointer(eh_frame_ptr_enc, &parameters, &mut reader)?;

        let fde_count;
        if fde_count_enc == constants::DW_EH_PE_omit || table_enc == constants::DW_EH_PE_omit {
            fde_count = 0
        } else {
            let ptr = parse_encoded_pointer(fde_count_enc, &parameters, &mut reader)?;
            fde_count = match ptr {
                Pointer::Direct(c) => c,
                Pointer::Indirect(_) => return Err(Error::UnsupportedPointerEncoding),
            }
        }

        Ok(ParsedEhFrameHdr {
            address_size,
            section: self.0.clone(),

            eh_frame_ptr,
            fde_count,
            table_enc,
            table: reader,
        })
    }
}

impl<R: Reader> Section<R> for EhFrameHdr<R> {
    fn id() -> SectionId {
        SectionId::EhFrameHdr
    }

    fn reader(&self) -> &R {
        &self.0
    }
}

impl<R: Reader> From<R> for EhFrameHdr<R> {
    fn from(section: R) -> Self {
        EhFrameHdr(section)
    }
}

impl<R: Reader> ParsedEhFrameHdr<R> {
    /// Returns the address of the binary's `.eh_frame` section.
    pub fn eh_frame_ptr(&self) -> Pointer {
        self.eh_frame_ptr
    }

    /// Retrieves the CFI binary search table, if there is one.
    pub fn table(&self) -> Option<EhHdrTable<R>> {
        // There are two big edge cases here:
        // * You search the table for an invalid address. As this is just a binary
        //   search table, we always have to return a valid result for that (unless
        //   you specify an address that is lower than the first address in the
        //   table). Since this means that you have to recheck that the FDE contains
        //   your address anyways, we just return the first FDE even when the address
        //   is too low. After all, we're just doing a normal binary search.
        // * This falls apart when the table is empty - there is no entry we could
        //   return. We conclude that an empty table is not really a table at all.
        if self.fde_count == 0 {
            None
        } else {
            Some(EhHdrTable { hdr: self })
        }
    }
}

/// An iterator for `.eh_frame_hdr` section's binary search table.
///
/// Each table entry consists of a tuple containing an  `initial_location` and `address`.
/// The `initial location` represents the first address that the targeted FDE
/// is able to decode. The `address` is the address of the FDE in the `.eh_frame` section.
/// The `address` can be converted with `EhHdrTable::pointer_to_offset` and `EhFrame::fde_from_offset` to an FDE.
#[derive(Debug)]
pub struct EhHdrTableIter<'a, 'bases, R: Reader> {
    hdr: &'a ParsedEhFrameHdr<R>,
    table: R,
    bases: &'bases BaseAddresses,
    remain: u64,
}

impl<'a, 'bases, R: Reader> EhHdrTableIter<'a, 'bases, R> {
    /// Yield the next entry in the `EhHdrTableIter`.
    pub fn next(&mut self) -> Result<Option<(Pointer, Pointer)>> {
        if self.remain == 0 {
            return Ok(None);
        }

        let parameters = PointerEncodingParameters {
            bases: &self.bases.eh_frame_hdr,
            func_base: None,
            address_size: self.hdr.address_size,
            section: &self.hdr.section,
        };

        self.remain -= 1;
        let from = parse_encoded_pointer(self.hdr.table_enc, &parameters, &mut self.table)?;
        let to = parse_encoded_pointer(self.hdr.table_enc, &parameters, &mut self.table)?;
        Ok(Some((from, to)))
    }
    /// Yield the nth entry in the `EhHdrTableIter`
    pub fn nth(&mut self, n: usize) -> Result<Option<(Pointer, Pointer)>> {
        use core::convert::TryFrom;
        let size = match self.hdr.table_enc.format() {
            constants::DW_EH_PE_uleb128 | constants::DW_EH_PE_sleb128 => {
                return Err(Error::VariableLengthSearchTable);
            }
            constants::DW_EH_PE_sdata2 | constants::DW_EH_PE_udata2 => 2,
            constants::DW_EH_PE_sdata4 | constants::DW_EH_PE_udata4 => 4,
            constants::DW_EH_PE_sdata8 | constants::DW_EH_PE_udata8 => 8,
            _ => return Err(Error::UnknownPointerEncoding),
        };

        let row_size = size * 2;
        let n = u64::try_from(n).map_err(|_| Error::UnsupportedOffset)?;
        self.remain = self.remain.saturating_sub(n);
        self.table.skip(R::Offset::from_u64(n * row_size)?)?;
        self.next()
    }
}

#[cfg(feature = "fallible-iterator")]
impl<'a, 'bases, R: Reader> fallible_iterator::FallibleIterator for EhHdrTableIter<'a, 'bases, R> {
    type Item = (Pointer, Pointer);
    type Error = Error;
    fn next(&mut self) -> Result<Option<Self::Item>> {
        EhHdrTableIter::next(self)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        use core::convert::TryInto;
        (
            self.remain.try_into().unwrap_or(0),
            self.remain.try_into().ok(),
        )
    }

    fn nth(&mut self, n: usize) -> Result<Option<Self::Item>> {
        EhHdrTableIter::nth(self, n)
    }
}

/// The CFI binary search table that is an optional part of the `.eh_frame_hdr` section.
#[derive(Debug, Clone)]
pub struct EhHdrTable<'a, R: Reader> {
    hdr: &'a ParsedEhFrameHdr<R>,
}

impl<'a, R: Reader + 'a> EhHdrTable<'a, R> {
    /// Return an iterator that can walk the `.eh_frame_hdr` table.
    ///
    /// Each table entry consists of a tuple containing an `initial_location` and `address`.
    /// The `initial location` represents the first address that the targeted FDE
    /// is able to decode. The `address` is the address of the FDE in the `.eh_frame` section.
    /// The `address` can be converted with `EhHdrTable::pointer_to_offset` and `EhFrame::fde_from_offset` to an FDE.
    pub fn iter<'bases>(&self, bases: &'bases BaseAddresses) -> EhHdrTableIter<'_, 'bases, R> {
        EhHdrTableIter {
            hdr: self.hdr,
            bases,
            remain: self.hdr.fde_count,
            table: self.hdr.table.clone(),
        }
    }
    /// *Probably* returns a pointer to the FDE for the given address.
    ///
    /// This performs a binary search, so if there is no FDE for the given address,
    /// this function **will** return a pointer to any other FDE that's close by.
    ///
    /// To be sure, you **must** call `contains` on the FDE.
    pub fn lookup(&self, address: u64, bases: &BaseAddresses) -> Result<Pointer> {
        let size = match self.hdr.table_enc.format() {
            constants::DW_EH_PE_uleb128 | constants::DW_EH_PE_sleb128 => {
                return Err(Error::VariableLengthSearchTable);
            }
            constants::DW_EH_PE_sdata2 | constants::DW_EH_PE_udata2 => 2,
            constants::DW_EH_PE_sdata4 | constants::DW_EH_PE_udata4 => 4,
            constants::DW_EH_PE_sdata8 | constants::DW_EH_PE_udata8 => 8,
            _ => return Err(Error::UnknownPointerEncoding),
        };

        let row_size = size * 2;

        let mut len = self.hdr.fde_count;

        let mut reader = self.hdr.table.clone();

        let parameters = PointerEncodingParameters {
            bases: &bases.eh_frame_hdr,
            func_base: None,
            address_size: self.hdr.address_size,
            section: &self.hdr.section,
        };

        while len > 1 {
            let head = reader.split(R::Offset::from_u64((len / 2) * row_size)?)?;
            let tail = reader.clone();

            let pivot = parse_encoded_pointer(self.hdr.table_enc, &parameters, &mut reader)?;
            let pivot = match pivot {
                Pointer::Direct(x) => x,
                Pointer::Indirect(_) => return Err(Error::UnsupportedPointerEncoding),
            };

            match pivot.cmp(&address) {
                Ordering::Equal => {
                    reader = tail;
                    break;
                }
                Ordering::Less => {
                    reader = tail;
                    len = len - (len / 2);
                }
                Ordering::Greater => {
                    reader = head;
                    len /= 2;
                }
            }
        }

        reader.skip(R::Offset::from_u64(size)?)?;

        parse_encoded_pointer(self.hdr.table_enc, &parameters, &mut reader)
    }

    /// Convert a `Pointer` to a section offset.
    ///
    /// This does not support indirect pointers.
    pub fn pointer_to_offset(&self, ptr: Pointer) -> Result<EhFrameOffset<R::Offset>> {
        let ptr = match ptr {
            Pointer::Direct(x) => x,
            _ => return Err(Error::UnsupportedPointerEncoding),
        };

        let eh_frame_ptr = match self.hdr.eh_frame_ptr() {
            Pointer::Direct(x) => x,
            _ => return Err(Error::UnsupportedPointerEncoding),
        };

        // Calculate the offset in the EhFrame section
        R::Offset::from_u64(ptr - eh_frame_ptr).map(EhFrameOffset)
    }

    /// Returns a parsed FDE for the given address, or `NoUnwindInfoForAddress`
    /// if there are none.
    ///
    /// You must provide a function to get its associated CIE. See
    /// `PartialFrameDescriptionEntry::parse` for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use gimli::{BaseAddresses, EhFrame, ParsedEhFrameHdr, EndianSlice, NativeEndian, Error, UnwindSection};
    /// # fn foo() -> Result<(), Error> {
    /// # let eh_frame: EhFrame<EndianSlice<NativeEndian>> = unreachable!();
    /// # let eh_frame_hdr: ParsedEhFrameHdr<EndianSlice<NativeEndian>> = unimplemented!();
    /// # let addr = 0;
    /// # let bases = unimplemented!();
    /// let table = eh_frame_hdr.table().unwrap();
    /// let fde = table.fde_for_address(&eh_frame, &bases, addr, EhFrame::cie_from_offset)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fde_for_address<F>(
        &self,
        frame: &EhFrame<R>,
        bases: &BaseAddresses,
        address: u64,
        get_cie: F,
    ) -> Result<FrameDescriptionEntry<R>>
    where
        F: FnMut(
            &EhFrame<R>,
            &BaseAddresses,
            EhFrameOffset<R::Offset>,
        ) -> Result<CommonInformationEntry<R>>,
    {
        let fdeptr = self.lookup(address, bases)?;
        let offset = self.pointer_to_offset(fdeptr)?;
        let entry = frame.fde_from_offset(bases, offset, get_cie)?;
        if entry.contains(address) {
            Ok(entry)
        } else {
            Err(Error::NoUnwindInfoForAddress)
        }
    }

    #[inline]
    #[doc(hidden)]
    #[deprecated(note = "Method renamed to fde_for_address; use that instead.")]
    pub fn lookup_and_parse<F>(
        &self,
        address: u64,
        bases: &BaseAddresses,
        frame: EhFrame<R>,
        get_cie: F,
    ) -> Result<FrameDescriptionEntry<R>>
    where
        F: FnMut(
            &EhFrame<R>,
            &BaseAddresses,
            EhFrameOffset<R::Offset>,
        ) -> Result<CommonInformationEntry<R>>,
    {
        self.fde_for_address(&frame, bases, address, get_cie)
    }

    /// Returns the frame unwind information for the given address,
    /// or `NoUnwindInfoForAddress` if there are none.
    ///
    /// You must provide a function to get the associated CIE. See
    /// `PartialFrameDescriptionEntry::parse` for more information.
    pub fn unwind_info_for_address<'ctx, F, A: UnwindContextStorage<R>>(
        &self,
        frame: &EhFrame<R>,
        bases: &BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
        address: u64,
        get_cie: F,
    ) -> Result<&'ctx UnwindTableRow<R, A>>
    where
        F: FnMut(
            &EhFrame<R>,
            &BaseAddresses,
            EhFrameOffset<R::Offset>,
        ) -> Result<CommonInformationEntry<R>>,
    {
        let fde = self.fde_for_address(frame, bases, address, get_cie)?;
        fde.unwind_info_for_address(frame, bases, ctx, address)
    }
}

/// `EhFrame` contains the frame unwinding information needed during exception
/// handling found in the `.eh_frame` section.
///
/// Most interesting methods are defined in the
/// [`UnwindSection`](trait.UnwindSection.html) trait.
///
/// See
/// [`DebugFrame`](./struct.DebugFrame.html#differences-between-debug_frame-and-eh_frame)
/// for some discussion on the differences between `.debug_frame` and
/// `.eh_frame`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EhFrame<R: Reader> {
    section: R,
    address_size: u8,
}

impl<R: Reader> EhFrame<R> {
    /// Set the size of a target address in bytes.
    ///
    /// This defaults to the native word size.
    pub fn set_address_size(&mut self, address_size: u8) {
        self.address_size = address_size
    }
}

impl<'input, Endian> EhFrame<EndianSlice<'input, Endian>>
where
    Endian: Endianity,
{
    /// Construct a new `EhFrame` instance from the data in the
    /// `.eh_frame` section.
    ///
    /// It is the caller's responsibility to read the section and present it as
    /// a `&[u8]` slice. That means using some ELF loader on Linux, a Mach-O
    /// loader on macOS, etc.
    ///
    /// ```
    /// use gimli::{EhFrame, EndianSlice, NativeEndian};
    ///
    /// // Use with `.eh_frame`
    /// # let buf = [0x00, 0x01, 0x02, 0x03];
    /// # let read_eh_frame_section_somehow = || &buf;
    /// let eh_frame = EhFrame::new(read_eh_frame_section_somehow(), NativeEndian);
    /// ```
    pub fn new(section: &'input [u8], endian: Endian) -> Self {
        Self::from(EndianSlice::new(section, endian))
    }
}

impl<R: Reader> Section<R> for EhFrame<R> {
    fn id() -> SectionId {
        SectionId::EhFrame
    }

    fn reader(&self) -> &R {
        &self.section
    }
}

impl<R: Reader> From<R> for EhFrame<R> {
    fn from(section: R) -> Self {
        // Default to native word size.
        EhFrame {
            section,
            address_size: mem::size_of::<usize>() as u8,
        }
    }
}

// This has to be `pub` to silence a warning (that is deny(..)'d by default) in
// rustc. Eventually, not having this `pub` will become a hard error.
#[doc(hidden)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CieOffsetEncoding {
    U32,
    U64,
}

/// An offset into an `UnwindSection`.
//
// Needed to avoid conflicting implementations of `Into<T>`.
pub trait UnwindOffset<T = usize>: Copy + Debug + Eq + From<T>
where
    T: ReaderOffset,
{
    /// Convert an `UnwindOffset<T>` into a `T`.
    fn into(self) -> T;
}

impl<T> UnwindOffset<T> for DebugFrameOffset<T>
where
    T: ReaderOffset,
{
    #[inline]
    fn into(self) -> T {
        self.0
    }
}

impl<T> UnwindOffset<T> for EhFrameOffset<T>
where
    T: ReaderOffset,
{
    #[inline]
    fn into(self) -> T {
        self.0
    }
}

/// This trait completely encapsulates everything that is different between
/// `.eh_frame` and `.debug_frame`, as well as all the bits that can change
/// between DWARF versions.
#[doc(hidden)]
pub trait _UnwindSectionPrivate<R: Reader> {
    /// Get the underlying section data.
    fn section(&self) -> &R;

    /// Returns true if the given length value should be considered an
    /// end-of-entries sentinel.
    fn length_value_is_end_of_entries(length: R::Offset) -> bool;

    /// Return true if the given offset if the CIE sentinel, false otherwise.
    fn is_cie(format: Format, id: u64) -> bool;

    /// Return the CIE offset/ID encoding used by this unwind section with the
    /// given DWARF format.
    fn cie_offset_encoding(format: Format) -> CieOffsetEncoding;

    /// For `.eh_frame`, CIE offsets are relative to the current position. For
    /// `.debug_frame`, they are relative to the start of the section. We always
    /// internally store them relative to the section, so we handle translating
    /// `.eh_frame`'s relative offsets in this method. If the offset calculation
    /// underflows, return `None`.
    fn resolve_cie_offset(&self, base: R::Offset, offset: R::Offset) -> Option<R::Offset>;

    /// Does this version of this unwind section encode address and segment
    /// sizes in its CIEs?
    fn has_address_and_segment_sizes(version: u8) -> bool;

    /// The address size to use if `has_address_and_segment_sizes` returns false.
    fn address_size(&self) -> u8;

    /// The segment size to use if `has_address_and_segment_sizes` returns false.
    fn segment_size(&self) -> u8;
}

/// A section holding unwind information: either `.debug_frame` or
/// `.eh_frame`. See [`DebugFrame`](./struct.DebugFrame.html) and
/// [`EhFrame`](./struct.EhFrame.html) respectively.
pub trait UnwindSection<R: Reader>: Clone + Debug + _UnwindSectionPrivate<R> {
    /// The offset type associated with this CFI section. Either
    /// `DebugFrameOffset` or `EhFrameOffset`.
    type Offset: UnwindOffset<R::Offset>;

    /// Iterate over the `CommonInformationEntry`s and `FrameDescriptionEntry`s
    /// in this `.debug_frame` section.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    fn entries<'bases>(&self, bases: &'bases BaseAddresses) -> CfiEntriesIter<'bases, Self, R> {
        CfiEntriesIter {
            section: self.clone(),
            bases,
            input: self.section().clone(),
        }
    }

    /// Parse the `CommonInformationEntry` at the given offset.
    fn cie_from_offset(
        &self,
        bases: &BaseAddresses,
        offset: Self::Offset,
    ) -> Result<CommonInformationEntry<R>> {
        let offset = UnwindOffset::into(offset);
        let input = &mut self.section().clone();
        input.skip(offset)?;
        CommonInformationEntry::parse(bases, self, input)
    }

    /// Parse the `PartialFrameDescriptionEntry` at the given offset.
    fn partial_fde_from_offset<'bases>(
        &self,
        bases: &'bases BaseAddresses,
        offset: Self::Offset,
    ) -> Result<PartialFrameDescriptionEntry<'bases, Self, R>> {
        let offset = UnwindOffset::into(offset);
        let input = &mut self.section().clone();
        input.skip(offset)?;
        PartialFrameDescriptionEntry::parse_partial(self, bases, input)
    }

    /// Parse the `FrameDescriptionEntry` at the given offset.
    fn fde_from_offset<F>(
        &self,
        bases: &BaseAddresses,
        offset: Self::Offset,
        get_cie: F,
    ) -> Result<FrameDescriptionEntry<R>>
    where
        F: FnMut(&Self, &BaseAddresses, Self::Offset) -> Result<CommonInformationEntry<R>>,
    {
        let partial = self.partial_fde_from_offset(bases, offset)?;
        partial.parse(get_cie)
    }

    /// Find the `FrameDescriptionEntry` for the given address.
    ///
    /// If found, the FDE is returned.  If not found,
    /// `Err(gimli::Error::NoUnwindInfoForAddress)` is returned.
    /// If parsing fails, the error is returned.
    ///
    /// You must provide a function to get its associated CIE. See
    /// `PartialFrameDescriptionEntry::parse` for more information.
    ///
    /// Note: this iterates over all FDEs. If available, it is possible
    /// to do a binary search with `EhFrameHdr::fde_for_address` instead.
    fn fde_for_address<F>(
        &self,
        bases: &BaseAddresses,
        address: u64,
        mut get_cie: F,
    ) -> Result<FrameDescriptionEntry<R>>
    where
        F: FnMut(&Self, &BaseAddresses, Self::Offset) -> Result<CommonInformationEntry<R>>,
    {
        let mut entries = self.entries(bases);
        while let Some(entry) = entries.next()? {
            match entry {
                CieOrFde::Cie(_) => {}
                CieOrFde::Fde(partial) => {
                    let fde = partial.parse(&mut get_cie)?;
                    if fde.contains(address) {
                        return Ok(fde);
                    }
                }
            }
        }
        Err(Error::NoUnwindInfoForAddress)
    }

    /// Find the frame unwind information for the given address.
    ///
    /// If found, the unwind information is returned.  If not found,
    /// `Err(gimli::Error::NoUnwindInfoForAddress)` is returned. If parsing or
    /// CFI evaluation fails, the error is returned.
    ///
    /// ```
    /// use gimli::{BaseAddresses, EhFrame, EndianSlice, NativeEndian, UnwindContext,
    ///             UnwindSection};
    ///
    /// # fn foo() -> gimli::Result<()> {
    /// # let read_eh_frame_section = || unimplemented!();
    /// // Get the `.eh_frame` section from the object file. Alternatively,
    /// // use `EhFrame` with the `.eh_frame` section of the object file.
    /// let eh_frame = EhFrame::new(read_eh_frame_section(), NativeEndian);
    ///
    /// # let get_frame_pc = || unimplemented!();
    /// // Get the address of the PC for a frame you'd like to unwind.
    /// let address = get_frame_pc();
    ///
    /// // This context is reusable, which cuts down on heap allocations.
    /// let ctx = UnwindContext::new();
    ///
    /// // Optionally provide base addresses for any relative pointers. If a
    /// // base address isn't provided and a pointer is found that is relative to
    /// // it, we will return an `Err`.
    /// # let address_of_text_section_in_memory = unimplemented!();
    /// # let address_of_got_section_in_memory = unimplemented!();
    /// let bases = BaseAddresses::default()
    ///     .set_text(address_of_text_section_in_memory)
    ///     .set_got(address_of_got_section_in_memory);
    ///
    /// let unwind_info = eh_frame.unwind_info_for_address(
    ///     &bases,
    ///     &mut ctx,
    ///     address,
    ///     EhFrame::cie_from_offset,
    /// )?;
    ///
    /// # let do_stuff_with = |_| unimplemented!();
    /// do_stuff_with(unwind_info);
    /// # let _ = ctx;
    /// # unreachable!()
    /// # }
    /// ```
    #[inline]
    fn unwind_info_for_address<'ctx, F, A: UnwindContextStorage<R>>(
        &self,
        bases: &BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
        address: u64,
        get_cie: F,
    ) -> Result<&'ctx UnwindTableRow<R, A>>
    where
        F: FnMut(&Self, &BaseAddresses, Self::Offset) -> Result<CommonInformationEntry<R>>,
    {
        let fde = self.fde_for_address(bases, address, get_cie)?;
        fde.unwind_info_for_address(self, bases, ctx, address)
    }
}

impl<R: Reader> _UnwindSectionPrivate<R> for DebugFrame<R> {
    fn section(&self) -> &R {
        &self.section
    }

    fn length_value_is_end_of_entries(_: R::Offset) -> bool {
        false
    }

    fn is_cie(format: Format, id: u64) -> bool {
        match format {
            Format::Dwarf32 => id == 0xffff_ffff,
            Format::Dwarf64 => id == 0xffff_ffff_ffff_ffff,
        }
    }

    fn cie_offset_encoding(format: Format) -> CieOffsetEncoding {
        match format {
            Format::Dwarf32 => CieOffsetEncoding::U32,
            Format::Dwarf64 => CieOffsetEncoding::U64,
        }
    }

    fn resolve_cie_offset(&self, _: R::Offset, offset: R::Offset) -> Option<R::Offset> {
        Some(offset)
    }

    fn has_address_and_segment_sizes(version: u8) -> bool {
        version == 4
    }

    fn address_size(&self) -> u8 {
        self.address_size
    }

    fn segment_size(&self) -> u8 {
        self.segment_size
    }
}

impl<R: Reader> UnwindSection<R> for DebugFrame<R> {
    type Offset = DebugFrameOffset<R::Offset>;
}

impl<R: Reader> _UnwindSectionPrivate<R> for EhFrame<R> {
    fn section(&self) -> &R {
        &self.section
    }

    fn length_value_is_end_of_entries(length: R::Offset) -> bool {
        length.into_u64() == 0
    }

    fn is_cie(_: Format, id: u64) -> bool {
        id == 0
    }

    fn cie_offset_encoding(_format: Format) -> CieOffsetEncoding {
        // `.eh_frame` offsets are always 4 bytes, regardless of the DWARF
        // format.
        CieOffsetEncoding::U32
    }

    fn resolve_cie_offset(&self, base: R::Offset, offset: R::Offset) -> Option<R::Offset> {
        base.checked_sub(offset)
    }

    fn has_address_and_segment_sizes(_version: u8) -> bool {
        false
    }

    fn address_size(&self) -> u8 {
        self.address_size
    }

    fn segment_size(&self) -> u8 {
        0
    }
}

impl<R: Reader> UnwindSection<R> for EhFrame<R> {
    type Offset = EhFrameOffset<R::Offset>;
}

/// Optional base addresses for the relative `DW_EH_PE_*` encoded pointers.
///
/// During CIE/FDE parsing, if a relative pointer is encountered for a base
/// address that is unknown, an Err will be returned.
///
/// ```
/// use gimli::BaseAddresses;
///
/// # fn foo() {
/// # let address_of_eh_frame_hdr_section_in_memory = unimplemented!();
/// # let address_of_eh_frame_section_in_memory = unimplemented!();
/// # let address_of_text_section_in_memory = unimplemented!();
/// # let address_of_got_section_in_memory = unimplemented!();
/// # let address_of_the_start_of_current_func = unimplemented!();
/// let bases = BaseAddresses::default()
///     .set_eh_frame_hdr(address_of_eh_frame_hdr_section_in_memory)
///     .set_eh_frame(address_of_eh_frame_section_in_memory)
///     .set_text(address_of_text_section_in_memory)
///     .set_got(address_of_got_section_in_memory);
/// # let _ = bases;
/// # }
/// ```
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct BaseAddresses {
    /// The base addresses to use for pointers in the `.eh_frame_hdr` section.
    pub eh_frame_hdr: SectionBaseAddresses,

    /// The base addresses to use for pointers in the `.eh_frame` section.
    pub eh_frame: SectionBaseAddresses,
}

/// Optional base addresses for the relative `DW_EH_PE_*` encoded pointers
/// in a particular section.
///
/// See `BaseAddresses` for methods that are helpful in setting these addresses.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct SectionBaseAddresses {
    /// The address of the section containing the pointer.
    pub section: Option<u64>,

    /// The base address for text relative pointers.
    /// This is generally the address of the `.text` section.
    pub text: Option<u64>,

    /// The base address for data relative pointers.
    ///
    /// For pointers in the `.eh_frame_hdr` section, this is the address
    /// of the `.eh_frame_hdr` section
    ///
    /// For pointers in the `.eh_frame` section, this is generally the
    /// global pointer, such as the address of the `.got` section.
    pub data: Option<u64>,
}

impl BaseAddresses {
    /// Set the `.eh_frame_hdr` section base address.
    #[inline]
    pub fn set_eh_frame_hdr(mut self, addr: u64) -> Self {
        self.eh_frame_hdr.section = Some(addr);
        self.eh_frame_hdr.data = Some(addr);
        self
    }

    /// Set the `.eh_frame` section base address.
    #[inline]
    pub fn set_eh_frame(mut self, addr: u64) -> Self {
        self.eh_frame.section = Some(addr);
        self
    }

    /// Set the `.text` section base address.
    #[inline]
    pub fn set_text(mut self, addr: u64) -> Self {
        self.eh_frame_hdr.text = Some(addr);
        self.eh_frame.text = Some(addr);
        self
    }

    /// Set the `.got` section base address.
    #[inline]
    pub fn set_got(mut self, addr: u64) -> Self {
        self.eh_frame.data = Some(addr);
        self
    }
}

/// An iterator over CIE and FDE entries in a `.debug_frame` or `.eh_frame`
/// section.
///
/// Some pointers may be encoded relative to various base addresses. Use the
/// [`BaseAddresses`](./struct.BaseAddresses.html) parameter to provide them. By
/// default, none are provided. If a relative pointer is encountered for a base
/// address that is unknown, an `Err` will be returned and iteration will abort.
///
/// Can be [used with
/// `FallibleIterator`](./index.html#using-with-fallibleiterator).
///
/// ```
/// use gimli::{BaseAddresses, EhFrame, EndianSlice, NativeEndian, UnwindSection};
///
/// # fn foo() -> gimli::Result<()> {
/// # let read_eh_frame_somehow = || unimplemented!();
/// let eh_frame = EhFrame::new(read_eh_frame_somehow(), NativeEndian);
///
/// # let address_of_eh_frame_hdr_section_in_memory = unimplemented!();
/// # let address_of_eh_frame_section_in_memory = unimplemented!();
/// # let address_of_text_section_in_memory = unimplemented!();
/// # let address_of_got_section_in_memory = unimplemented!();
/// # let address_of_the_start_of_current_func = unimplemented!();
/// // Provide base addresses for relative pointers.
/// let bases = BaseAddresses::default()
///     .set_eh_frame_hdr(address_of_eh_frame_hdr_section_in_memory)
///     .set_eh_frame(address_of_eh_frame_section_in_memory)
///     .set_text(address_of_text_section_in_memory)
///     .set_got(address_of_got_section_in_memory);
///
/// let mut entries = eh_frame.entries(&bases);
///
/// # let do_stuff_with = |_| unimplemented!();
/// while let Some(entry) = entries.next()? {
///     do_stuff_with(entry)
/// }
/// # unreachable!()
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct CfiEntriesIter<'bases, Section, R>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    section: Section,
    bases: &'bases BaseAddresses,
    input: R,
}

impl<'bases, Section, R> CfiEntriesIter<'bases, Section, R>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    /// Advance the iterator to the next entry.
    pub fn next(&mut self) -> Result<Option<CieOrFde<'bases, Section, R>>> {
        if self.input.is_empty() {
            return Ok(None);
        }

        match parse_cfi_entry(self.bases, &self.section, &mut self.input) {
            Err(e) => {
                self.input.empty();
                Err(e)
            }
            Ok(None) => {
                self.input.empty();
                Ok(None)
            }
            Ok(Some(entry)) => Ok(Some(entry)),
        }
    }
}

#[cfg(feature = "fallible-iterator")]
impl<'bases, Section, R> fallible_iterator::FallibleIterator for CfiEntriesIter<'bases, Section, R>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    type Item = CieOrFde<'bases, Section, R>;
    type Error = Error;

    fn next(&mut self) -> ::core::result::Result<Option<Self::Item>, Self::Error> {
        CfiEntriesIter::next(self)
    }
}

/// Either a `CommonInformationEntry` (CIE) or a `FrameDescriptionEntry` (FDE).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CieOrFde<'bases, Section, R>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    /// This CFI entry is a `CommonInformationEntry`.
    Cie(CommonInformationEntry<R>),
    /// This CFI entry is a `FrameDescriptionEntry`, however fully parsing it
    /// requires parsing its CIE first, so it is left in a partially parsed
    /// state.
    Fde(PartialFrameDescriptionEntry<'bases, Section, R>),
}

#[allow(clippy::type_complexity)]
fn parse_cfi_entry<'bases, Section, R>(
    bases: &'bases BaseAddresses,
    section: &Section,
    input: &mut R,
) -> Result<Option<CieOrFde<'bases, Section, R>>>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    let (offset, length, format) = loop {
        let offset = input.offset_from(section.section());
        let (length, format) = input.read_initial_length()?;

        if Section::length_value_is_end_of_entries(length) {
            return Ok(None);
        }

        // Hack: skip zero padding inserted by buggy compilers/linkers.
        // We require that the padding is a multiple of 32-bits, otherwise
        // there is no reliable way to determine when the padding ends. This
        // should be okay since CFI entries must be aligned to the address size.

        if length.into_u64() != 0 || format != Format::Dwarf32 {
            break (offset, length, format);
        }
    };

    let mut rest = input.split(length)?;
    let cie_offset_base = rest.offset_from(section.section());
    let cie_id_or_offset = match Section::cie_offset_encoding(format) {
        CieOffsetEncoding::U32 => rest.read_u32().map(u64::from)?,
        CieOffsetEncoding::U64 => rest.read_u64()?,
    };

    if Section::is_cie(format, cie_id_or_offset) {
        let cie = CommonInformationEntry::parse_rest(offset, length, format, bases, section, rest)?;
        Ok(Some(CieOrFde::Cie(cie)))
    } else {
        let cie_offset = R::Offset::from_u64(cie_id_or_offset)?;
        let cie_offset = match section.resolve_cie_offset(cie_offset_base, cie_offset) {
            None => return Err(Error::OffsetOutOfBounds),
            Some(cie_offset) => cie_offset,
        };

        let fde = PartialFrameDescriptionEntry {
            offset,
            length,
            format,
            cie_offset: cie_offset.into(),
            rest,
            section: section.clone(),
            bases,
        };

        Ok(Some(CieOrFde::Fde(fde)))
    }
}

/// We support the z-style augmentation [defined by `.eh_frame`][ehframe].
///
/// [ehframe]: https://refspecs.linuxfoundation.org/LSB_3.0.0/LSB-Core-generic/LSB-Core-generic/ehframechpt.html
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Augmentation {
    /// > A 'L' may be present at any position after the first character of the
    /// > string. This character may only be present if 'z' is the first character
    /// > of the string. If present, it indicates the presence of one argument in
    /// > the Augmentation Data of the CIE, and a corresponding argument in the
    /// > Augmentation Data of the FDE. The argument in the Augmentation Data of
    /// > the CIE is 1-byte and represents the pointer encoding used for the
    /// > argument in the Augmentation Data of the FDE, which is the address of a
    /// > language-specific data area (LSDA). The size of the LSDA pointer is
    /// > specified by the pointer encoding used.
    lsda: Option<constants::DwEhPe>,

    /// > A 'P' may be present at any position after the first character of the
    /// > string. This character may only be present if 'z' is the first character
    /// > of the string. If present, it indicates the presence of two arguments in
    /// > the Augmentation Data of the CIE. The first argument is 1-byte and
    /// > represents the pointer encoding used for the second argument, which is
    /// > the address of a personality routine handler. The size of the
    /// > personality routine pointer is specified by the pointer encoding used.
    personality: Option<(constants::DwEhPe, Pointer)>,

    /// > A 'R' may be present at any position after the first character of the
    /// > string. This character may only be present if 'z' is the first character
    /// > of the string. If present, The Augmentation Data shall include a 1 byte
    /// > argument that represents the pointer encoding for the address pointers
    /// > used in the FDE.
    fde_address_encoding: Option<constants::DwEhPe>,

    /// True if this CIE's FDEs are trampolines for signal handlers.
    is_signal_trampoline: bool,
}

impl Augmentation {
    fn parse<Section, R>(
        augmentation_str: &mut R,
        bases: &BaseAddresses,
        address_size: u8,
        section: &Section,
        input: &mut R,
    ) -> Result<Augmentation>
    where
        R: Reader,
        Section: UnwindSection<R>,
    {
        debug_assert!(
            !augmentation_str.is_empty(),
            "Augmentation::parse should only be called if we have an augmentation"
        );

        let mut augmentation = Augmentation::default();

        let mut parsed_first = false;
        let mut data = None;

        while !augmentation_str.is_empty() {
            let ch = augmentation_str.read_u8()?;
            match ch {
                b'z' => {
                    if parsed_first {
                        return Err(Error::UnknownAugmentation);
                    }

                    let augmentation_length = input.read_uleb128().and_then(R::Offset::from_u64)?;
                    data = Some(input.split(augmentation_length)?);
                }
                b'L' => {
                    let rest = data.as_mut().ok_or(Error::UnknownAugmentation)?;
                    let encoding = parse_pointer_encoding(rest)?;
                    augmentation.lsda = Some(encoding);
                }
                b'P' => {
                    let rest = data.as_mut().ok_or(Error::UnknownAugmentation)?;
                    let encoding = parse_pointer_encoding(rest)?;
                    let parameters = PointerEncodingParameters {
                        bases: &bases.eh_frame,
                        func_base: None,
                        address_size,
                        section: section.section(),
                    };

                    let personality = parse_encoded_pointer(encoding, &parameters, rest)?;
                    augmentation.personality = Some((encoding, personality));
                }
                b'R' => {
                    let rest = data.as_mut().ok_or(Error::UnknownAugmentation)?;
                    let encoding = parse_pointer_encoding(rest)?;
                    augmentation.fde_address_encoding = Some(encoding);
                }
                b'S' => augmentation.is_signal_trampoline = true,
                _ => return Err(Error::UnknownAugmentation),
            }

            parsed_first = true;
        }

        Ok(augmentation)
    }
}

/// Parsed augmentation data for a `FrameDescriptEntry`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AugmentationData {
    lsda: Option<Pointer>,
}

impl AugmentationData {
    fn parse<R: Reader>(
        augmentation: &Augmentation,
        encoding_parameters: &PointerEncodingParameters<R>,
        input: &mut R,
    ) -> Result<AugmentationData> {
        // In theory, we should be iterating over the original augmentation
        // string, interpreting each character, and reading the appropriate bits
        // out of the augmentation data as we go. However, the only character
        // that defines augmentation data in the FDE is the 'L' character, so we
        // can just check for its presence directly.

        let aug_data_len = input.read_uleb128().and_then(R::Offset::from_u64)?;
        let rest = &mut input.split(aug_data_len)?;
        let mut augmentation_data = AugmentationData::default();
        if let Some(encoding) = augmentation.lsda {
            let lsda = parse_encoded_pointer(encoding, encoding_parameters, rest)?;
            augmentation_data.lsda = Some(lsda);
        }
        Ok(augmentation_data)
    }
}

/// > A Common Information Entry holds information that is shared among many
/// > Frame Description Entries. There is at least one CIE in every non-empty
/// > `.debug_frame` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommonInformationEntry<R, Offset = <R as Reader>::Offset>
where
    R: Reader<Offset = Offset>,
    Offset: ReaderOffset,
{
    /// The offset of this entry from the start of its containing section.
    offset: Offset,

    /// > A constant that gives the number of bytes of the CIE structure, not
    /// > including the length field itself (see Section 7.2.2). The size of the
    /// > length field plus the value of length must be an integral multiple of
    /// > the address size.
    length: Offset,

    format: Format,

    /// > A version number (see Section 7.23). This number is specific to the
    /// > call frame information and is independent of the DWARF version number.
    version: u8,

    /// The parsed augmentation, if any.
    augmentation: Option<Augmentation>,

    /// > The size of a target address in this CIE and any FDEs that use it, in
    /// > bytes. If a compilation unit exists for this frame, its address size
    /// > must match the address size here.
    address_size: u8,

    /// "The size of a segment selector in this CIE and any FDEs that use it, in
    /// bytes."
    segment_size: u8,

    /// "A constant that is factored out of all advance location instructions
    /// (see Section 6.4.2.1)."
    code_alignment_factor: u64,

    /// > A constant that is factored out of certain offset instructions (see
    /// > below). The resulting value is (operand * data_alignment_factor).
    data_alignment_factor: i64,

    /// > An unsigned LEB128 constant that indicates which column in the rule
    /// > table represents the return address of the function. Note that this
    /// > column might not correspond to an actual machine register.
    return_address_register: Register,

    /// > A sequence of rules that are interpreted to create the initial setting
    /// > of each column in the table.
    ///
    /// > The default rule for all columns before interpretation of the initial
    /// > instructions is the undefined rule. However, an ABI authoring body or a
    /// > compilation system authoring body may specify an alternate default
    /// > value for any or all columns.
    ///
    /// This is followed by `DW_CFA_nop` padding until the end of `length` bytes
    /// in the input.
    initial_instructions: R,
}

impl<R: Reader> CommonInformationEntry<R> {
    fn parse<Section: UnwindSection<R>>(
        bases: &BaseAddresses,
        section: &Section,
        input: &mut R,
    ) -> Result<CommonInformationEntry<R>> {
        match parse_cfi_entry(bases, section, input)? {
            Some(CieOrFde::Cie(cie)) => Ok(cie),
            Some(CieOrFde::Fde(_)) => Err(Error::NotCieId),
            None => Err(Error::NoEntryAtGivenOffset),
        }
    }

    fn parse_rest<Section: UnwindSection<R>>(
        offset: R::Offset,
        length: R::Offset,
        format: Format,
        bases: &BaseAddresses,
        section: &Section,
        mut rest: R,
    ) -> Result<CommonInformationEntry<R>> {
        let version = rest.read_u8()?;

        // Version 1 of `.debug_frame` corresponds to DWARF 2, and then for
        // DWARF 3 and 4, I think they decided to just match the standard's
        // version.
        match version {
            1 | 3 | 4 => (),
            _ => return Err(Error::UnknownVersion(u64::from(version))),
        }

        let mut augmentation_string = rest.read_null_terminated_slice()?;

        let (address_size, segment_size) = if Section::has_address_and_segment_sizes(version) {
            let address_size = rest.read_u8()?;
            let segment_size = rest.read_u8()?;
            (address_size, segment_size)
        } else {
            (section.address_size(), section.segment_size())
        };

        let code_alignment_factor = rest.read_uleb128()?;
        let data_alignment_factor = rest.read_sleb128()?;

        let return_address_register = if version == 1 {
            Register(rest.read_u8()?.into())
        } else {
            rest.read_uleb128().and_then(Register::from_u64)?
        };

        let augmentation = if augmentation_string.is_empty() {
            None
        } else {
            Some(Augmentation::parse(
                &mut augmentation_string,
                bases,
                address_size,
                section,
                &mut rest,
            )?)
        };

        let entry = CommonInformationEntry {
            offset,
            length,
            format,
            version,
            augmentation,
            address_size,
            segment_size,
            code_alignment_factor,
            data_alignment_factor,
            return_address_register,
            initial_instructions: rest,
        };

        Ok(entry)
    }
}

/// # Signal Safe Methods
///
/// These methods are guaranteed not to allocate, acquire locks, or perform any
/// other signal-unsafe operations.
impl<R: Reader> CommonInformationEntry<R> {
    /// Get the offset of this entry from the start of its containing section.
    pub fn offset(&self) -> R::Offset {
        self.offset
    }

    /// Return the encoding parameters for this CIE.
    pub fn encoding(&self) -> Encoding {
        Encoding {
            format: self.format,
            version: u16::from(self.version),
            address_size: self.address_size,
        }
    }

    /// The size of addresses (in bytes) in this CIE.
    pub fn address_size(&self) -> u8 {
        self.address_size
    }

    /// Iterate over this CIE's initial instructions.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    pub fn instructions<'a, Section>(
        &self,
        section: &'a Section,
        bases: &'a BaseAddresses,
    ) -> CallFrameInstructionIter<'a, R>
    where
        Section: UnwindSection<R>,
    {
        CallFrameInstructionIter {
            input: self.initial_instructions.clone(),
            address_encoding: None,
            parameters: PointerEncodingParameters {
                bases: &bases.eh_frame,
                func_base: None,
                address_size: self.address_size,
                section: section.section(),
            },
        }
    }

    /// > A constant that gives the number of bytes of the CIE structure, not
    /// > including the length field itself (see Section 7.2.2). The size of the
    /// > length field plus the value of length must be an integral multiple of
    /// > the address size.
    pub fn entry_len(&self) -> R::Offset {
        self.length
    }

    /// > A version number (see Section 7.23). This number is specific to the
    /// > call frame information and is independent of the DWARF version number.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Get the augmentation data, if any exists.
    ///
    /// The only augmentation understood by `gimli` is that which is defined by
    /// `.eh_frame`.
    pub fn augmentation(&self) -> Option<&Augmentation> {
        self.augmentation.as_ref()
    }

    /// True if this CIE's FDEs have a LSDA.
    pub fn has_lsda(&self) -> bool {
        self.augmentation.map_or(false, |a| a.lsda.is_some())
    }

    /// Return the encoding of the LSDA address for this CIE's FDEs.
    pub fn lsda_encoding(&self) -> Option<constants::DwEhPe> {
        self.augmentation.and_then(|a| a.lsda)
    }

    /// Return the encoding and address of the personality routine handler
    /// for this CIE's FDEs.
    pub fn personality_with_encoding(&self) -> Option<(constants::DwEhPe, Pointer)> {
        self.augmentation.as_ref().and_then(|a| a.personality)
    }

    /// Return the address of the personality routine handler
    /// for this CIE's FDEs.
    pub fn personality(&self) -> Option<Pointer> {
        self.augmentation
            .as_ref()
            .and_then(|a| a.personality)
            .map(|(_, p)| p)
    }

    /// Return the encoding of the addresses for this CIE's FDEs.
    pub fn fde_address_encoding(&self) -> Option<constants::DwEhPe> {
        self.augmentation.and_then(|a| a.fde_address_encoding)
    }

    /// True if this CIE's FDEs are trampolines for signal handlers.
    pub fn is_signal_trampoline(&self) -> bool {
        self.augmentation.map_or(false, |a| a.is_signal_trampoline)
    }

    /// > A constant that is factored out of all advance location instructions
    /// > (see Section 6.4.2.1).
    pub fn code_alignment_factor(&self) -> u64 {
        self.code_alignment_factor
    }

    /// > A constant that is factored out of certain offset instructions (see
    /// > below). The resulting value is (operand * data_alignment_factor).
    pub fn data_alignment_factor(&self) -> i64 {
        self.data_alignment_factor
    }

    /// > An unsigned ... constant that indicates which column in the rule
    /// > table represents the return address of the function. Note that this
    /// > column might not correspond to an actual machine register.
    pub fn return_address_register(&self) -> Register {
        self.return_address_register
    }
}

/// A partially parsed `FrameDescriptionEntry`.
///
/// Fully parsing this FDE requires first parsing its CIE.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialFrameDescriptionEntry<'bases, Section, R>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    offset: R::Offset,
    length: R::Offset,
    format: Format,
    cie_offset: Section::Offset,
    rest: R,
    section: Section,
    bases: &'bases BaseAddresses,
}

impl<'bases, Section, R> PartialFrameDescriptionEntry<'bases, Section, R>
where
    R: Reader,
    Section: UnwindSection<R>,
{
    fn parse_partial(
        section: &Section,
        bases: &'bases BaseAddresses,
        input: &mut R,
    ) -> Result<PartialFrameDescriptionEntry<'bases, Section, R>> {
        match parse_cfi_entry(bases, section, input)? {
            Some(CieOrFde::Cie(_)) => Err(Error::NotFdePointer),
            Some(CieOrFde::Fde(partial)) => Ok(partial),
            None => Err(Error::NoEntryAtGivenOffset),
        }
    }

    /// Fully parse this FDE.
    ///
    /// You must provide a function get its associated CIE (either by parsing it
    /// on demand, or looking it up in some table mapping offsets to CIEs that
    /// you've already parsed, etc.)
    pub fn parse<F>(&self, get_cie: F) -> Result<FrameDescriptionEntry<R>>
    where
        F: FnMut(&Section, &BaseAddresses, Section::Offset) -> Result<CommonInformationEntry<R>>,
    {
        FrameDescriptionEntry::parse_rest(
            self.offset,
            self.length,
            self.format,
            self.cie_offset,
            self.rest.clone(),
            &self.section,
            self.bases,
            get_cie,
        )
    }
}

/// A `FrameDescriptionEntry` is a set of CFA instructions for an address range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameDescriptionEntry<R, Offset = <R as Reader>::Offset>
where
    R: Reader<Offset = Offset>,
    Offset: ReaderOffset,
{
    /// The start of this entry within its containing section.
    offset: Offset,

    /// > A constant that gives the number of bytes of the header and
    /// > instruction stream for this function, not including the length field
    /// > itself (see Section 7.2.2). The size of the length field plus the value
    /// > of length must be an integral multiple of the address size.
    length: Offset,

    format: Format,

    /// "A constant offset into the .debug_frame section that denotes the CIE
    /// that is associated with this FDE."
    ///
    /// This is the CIE at that offset.
    cie: CommonInformationEntry<R, Offset>,

    /// > The address of the first location associated with this table entry. If
    /// > the segment_size field of this FDE's CIE is non-zero, the initial
    /// > location is preceded by a segment selector of the given length.
    initial_segment: u64,
    initial_address: u64,

    /// "The number of bytes of program instructions described by this entry."
    address_range: u64,

    /// The parsed augmentation data, if we have any.
    augmentation: Option<AugmentationData>,

    /// "A sequence of table defining instructions that are described below."
    ///
    /// This is followed by `DW_CFA_nop` padding until `length` bytes of the
    /// input are consumed.
    instructions: R,
}

impl<R: Reader> FrameDescriptionEntry<R> {
    #[allow(clippy::too_many_arguments)]
    fn parse_rest<Section, F>(
        offset: R::Offset,
        length: R::Offset,
        format: Format,
        cie_pointer: Section::Offset,
        mut rest: R,
        section: &Section,
        bases: &BaseAddresses,
        mut get_cie: F,
    ) -> Result<FrameDescriptionEntry<R>>
    where
        Section: UnwindSection<R>,
        F: FnMut(&Section, &BaseAddresses, Section::Offset) -> Result<CommonInformationEntry<R>>,
    {
        let cie = get_cie(section, bases, cie_pointer)?;

        let initial_segment = if cie.segment_size > 0 {
            rest.read_address(cie.segment_size)?
        } else {
            0
        };

        let mut parameters = PointerEncodingParameters {
            bases: &bases.eh_frame,
            func_base: None,
            address_size: cie.address_size,
            section: section.section(),
        };

        let (initial_address, address_range) = Self::parse_addresses(&mut rest, &cie, &parameters)?;
        parameters.func_base = Some(initial_address);

        let aug_data = if let Some(ref augmentation) = cie.augmentation {
            Some(AugmentationData::parse(
                augmentation,
                &parameters,
                &mut rest,
            )?)
        } else {
            None
        };

        let entry = FrameDescriptionEntry {
            offset,
            length,
            format,
            cie,
            initial_segment,
            initial_address,
            address_range,
            augmentation: aug_data,
            instructions: rest,
        };

        Ok(entry)
    }

    fn parse_addresses(
        input: &mut R,
        cie: &CommonInformationEntry<R>,
        parameters: &PointerEncodingParameters<R>,
    ) -> Result<(u64, u64)> {
        let encoding = cie.augmentation().and_then(|a| a.fde_address_encoding);
        if let Some(encoding) = encoding {
            let initial_address = parse_encoded_pointer(encoding, parameters, input)?;

            // Ignore indirection.
            let initial_address = initial_address.into();

            // Address ranges cannot be relative to anything, so just grab the
            // data format bits from the encoding.
            let address_range = parse_encoded_pointer(encoding.format(), parameters, input)?;
            Ok((initial_address, address_range.into()))
        } else {
            let initial_address = input.read_address(cie.address_size)?;
            let address_range = input.read_address(cie.address_size)?;
            Ok((initial_address, address_range))
        }
    }

    /// Return the table of unwind information for this FDE.
    #[inline]
    pub fn rows<'a, 'ctx, Section: UnwindSection<R>, A: UnwindContextStorage<R>>(
        &self,
        section: &'a Section,
        bases: &'a BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
    ) -> Result<UnwindTable<'a, 'ctx, R, A>> {
        UnwindTable::new(section, bases, ctx, self)
    }

    /// Find the frame unwind information for the given address.
    ///
    /// If found, the unwind information is returned along with the reset
    /// context in the form `Ok((unwind_info, context))`. If not found,
    /// `Err(gimli::Error::NoUnwindInfoForAddress)` is returned. If parsing or
    /// CFI evaluation fails, the error is returned.
    pub fn unwind_info_for_address<'ctx, Section: UnwindSection<R>, A: UnwindContextStorage<R>>(
        &self,
        section: &Section,
        bases: &BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
        address: u64,
    ) -> Result<&'ctx UnwindTableRow<R, A>> {
        let mut table = self.rows(section, bases, ctx)?;
        while let Some(row) = table.next_row()? {
            if row.contains(address) {
                return Ok(table.ctx.row());
            }
        }
        Err(Error::NoUnwindInfoForAddress)
    }
}

/// # Signal Safe Methods
///
/// These methods are guaranteed not to allocate, acquire locks, or perform any
/// other signal-unsafe operations.
#[allow(clippy::len_without_is_empty)]
impl<R: Reader> FrameDescriptionEntry<R> {
    /// Get the offset of this entry from the start of its containing section.
    pub fn offset(&self) -> R::Offset {
        self.offset
    }

    /// Get a reference to this FDE's CIE.
    pub fn cie(&self) -> &CommonInformationEntry<R> {
        &self.cie
    }

    /// > A constant that gives the number of bytes of the header and
    /// > instruction stream for this function, not including the length field
    /// > itself (see Section 7.2.2). The size of the length field plus the value
    /// > of length must be an integral multiple of the address size.
    pub fn entry_len(&self) -> R::Offset {
        self.length
    }

    /// Iterate over this FDE's instructions.
    ///
    /// Will not include the CIE's initial instructions, if you want those do
    /// `fde.cie().instructions()` first.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    pub fn instructions<'a, Section>(
        &self,
        section: &'a Section,
        bases: &'a BaseAddresses,
    ) -> CallFrameInstructionIter<'a, R>
    where
        Section: UnwindSection<R>,
    {
        CallFrameInstructionIter {
            input: self.instructions.clone(),
            address_encoding: self.cie.augmentation().and_then(|a| a.fde_address_encoding),
            parameters: PointerEncodingParameters {
                bases: &bases.eh_frame,
                func_base: None,
                address_size: self.cie.address_size,
                section: section.section(),
            },
        }
    }

    /// The first address for which this entry has unwind information for.
    pub fn initial_address(&self) -> u64 {
        self.initial_address
    }

    /// The number of bytes of instructions that this entry has unwind
    /// information for.
    pub fn len(&self) -> u64 {
        self.address_range
    }

    /// Return `true` if the given address is within this FDE, `false`
    /// otherwise.
    ///
    /// This is equivalent to `entry.initial_address() <= address <
    /// entry.initial_address() + entry.len()`.
    pub fn contains(&self, address: u64) -> bool {
        let start = self.initial_address();
        let end = start + self.len();
        start <= address && address < end
    }

    /// The address of this FDE's language-specific data area (LSDA), if it has
    /// any.
    pub fn lsda(&self) -> Option<Pointer> {
        self.augmentation.as_ref().and_then(|a| a.lsda)
    }

    /// Return true if this FDE's function is a trampoline for a signal handler.
    #[inline]
    pub fn is_signal_trampoline(&self) -> bool {
        self.cie().is_signal_trampoline()
    }

    /// Return the address of the FDE's function's personality routine
    /// handler. The personality routine does language-specific clean up when
    /// unwinding the stack frames with the intent to not run them again.
    #[inline]
    pub fn personality(&self) -> Option<Pointer> {
        self.cie().personality()
    }
}

/// Specification of what storage should be used for [`UnwindContext`].
///
#[cfg_attr(
    feature = "read",
    doc = "
Normally you would only need to use [`StoreOnHeap`], which places the stack
on the heap using [`Vec`]. This is the default storage type parameter for [`UnwindContext`].
"
)]
///
/// If you need to avoid [`UnwindContext`] from allocating memory, e.g. for signal safety,
/// you can provide you own storage specification:
/// ```rust,no_run
/// # use gimli::*;
/// #
/// # fn foo<'a>(some_fde: gimli::FrameDescriptionEntry<gimli::EndianSlice<'a, gimli::LittleEndian>>)
/// #            -> gimli::Result<()> {
/// # let eh_frame: gimli::EhFrame<_> = unreachable!();
/// # let bases = unimplemented!();
/// #
/// struct StoreOnStack;
///
/// impl<R: Reader> UnwindContextStorage<R> for StoreOnStack {
///     type Rules = [(Register, RegisterRule<R>); 192];
///     type Stack = [UnwindTableRow<R, Self>; 4];
/// }
///
/// let mut ctx = UnwindContext::<_, StoreOnStack>::new_in();
///
/// // Initialize the context by evaluating the CIE's initial instruction program,
/// // and generate the unwind table.
/// let mut table = some_fde.rows(&eh_frame, &bases, &mut ctx)?;
/// while let Some(row) = table.next_row()? {
///     // Do stuff with each row...
/// #   let _ = row;
/// }
/// # unreachable!()
/// # }
/// ```
pub trait UnwindContextStorage<R: Reader>: Sized {
    /// The storage used for register rules in a unwind table row.
    ///
    /// Note that this is nested within the stack.
    type Rules: ArrayLike<Item = (Register, RegisterRule<R>)>;

    /// The storage used for unwind table row stack.
    type Stack: ArrayLike<Item = UnwindTableRow<R, Self>>;
}

#[cfg(feature = "read")]
const MAX_RULES: usize = 192;

#[cfg(feature = "read")]
impl<R: Reader> UnwindContextStorage<R> for StoreOnHeap {
    type Rules = [(Register, RegisterRule<R>); MAX_RULES];
    type Stack = Vec<UnwindTableRow<R, Self>>;
}

/// Common context needed when evaluating the call frame unwinding information.
///
/// This structure can be large so it is advisable to place it on the heap.
/// To avoid re-allocating the context multiple times when evaluating multiple
/// CFI programs, it can be reused.
///
/// ```
/// use gimli::{UnwindContext, UnwindTable};
///
/// # fn foo<'a>(some_fde: gimli::FrameDescriptionEntry<gimli::EndianSlice<'a, gimli::LittleEndian>>)
/// #            -> gimli::Result<()> {
/// # let eh_frame: gimli::EhFrame<_> = unreachable!();
/// # let bases = unimplemented!();
/// // An uninitialized context.
/// let mut ctx = Box::new(UnwindContext::new());
///
/// // Initialize the context by evaluating the CIE's initial instruction program,
/// // and generate the unwind table.
/// let mut table = some_fde.rows(&eh_frame, &bases, &mut ctx)?;
/// while let Some(row) = table.next_row()? {
///     // Do stuff with each row...
/// #   let _ = row;
/// }
/// # unreachable!()
/// # }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct UnwindContext<R: Reader, A: UnwindContextStorage<R> = StoreOnHeap> {
    // Stack of rows. The last row is the row currently being built by the
    // program. There is always at least one row. The vast majority of CFI
    // programs will only ever have one row on the stack.
    stack: ArrayVec<A::Stack>,

    // If we are evaluating an FDE's instructions, then `is_initialized` will be
    // `true`. If `initial_rule` is `Some`, then the initial register rules are either
    // all default rules or have just 1 non-default rule, stored in `initial_rule`.
    // If it's `None`, `stack[0]` will contain the initial register rules
    // described by the CIE's initial instructions. These rules are used by
    // `DW_CFA_restore`. Otherwise, when we are currently evaluating a CIE's
    // initial instructions, `is_initialized` will be `false` and initial rules
    // cannot be read.
    initial_rule: Option<(Register, RegisterRule<R>)>,

    is_initialized: bool,
}

impl<R: Reader, S: UnwindContextStorage<R>> Debug for UnwindContext<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnwindContext")
            .field("stack", &self.stack)
            .field("initial_rule", &self.initial_rule)
            .field("is_initialized", &self.is_initialized)
            .finish()
    }
}

impl<R: Reader, A: UnwindContextStorage<R>> Default for UnwindContext<R, A> {
    fn default() -> Self {
        Self::new_in()
    }
}

#[cfg(feature = "read")]
impl<R: Reader> UnwindContext<R> {
    /// Construct a new call frame unwinding context.
    pub fn new() -> Self {
        Self::new_in()
    }
}

/// # Signal Safe Methods
///
/// These methods are guaranteed not to allocate, acquire locks, or perform any
/// other signal-unsafe operations, if an non-allocating storage is used.
impl<R: Reader, A: UnwindContextStorage<R>> UnwindContext<R, A> {
    /// Construct a new call frame unwinding context.
    pub fn new_in() -> Self {
        let mut ctx = UnwindContext {
            stack: Default::default(),
            initial_rule: None,
            is_initialized: false,
        };
        ctx.reset();
        ctx
    }

    /// Run the CIE's initial instructions and initialize this `UnwindContext`.
    fn initialize<Section: UnwindSection<R>>(
        &mut self,
        section: &Section,
        bases: &BaseAddresses,
        cie: &CommonInformationEntry<R>,
    ) -> Result<()> {
        if self.is_initialized {
            self.reset();
        }

        let mut table = UnwindTable::new_for_cie(section, bases, self, cie);
        while let Some(_) = table.next_row()? {}

        self.save_initial_rules()?;
        Ok(())
    }

    fn reset(&mut self) {
        self.stack.clear();
        self.stack.try_push(UnwindTableRow::default()).unwrap();
        debug_assert!(self.stack[0].is_default());
        self.initial_rule = None;
        self.is_initialized = false;
    }

    fn row(&self) -> &UnwindTableRow<R, A> {
        self.stack.last().unwrap()
    }

    fn row_mut(&mut self) -> &mut UnwindTableRow<R, A> {
        self.stack.last_mut().unwrap()
    }

    fn save_initial_rules(&mut self) -> Result<()> {
        assert_eq!(self.is_initialized, false);
        self.initial_rule = match *self.stack.last().unwrap().registers.rules {
            // All rules are default (undefined). In this case just synthesize
            // an undefined rule.
            [] => Some((Register(0), RegisterRule::Undefined)),
            [ref rule] => Some(rule.clone()),
            _ => {
                let rules = self.stack.last().unwrap().clone();
                self.stack
                    .try_insert(0, rules)
                    .map_err(|_| Error::StackFull)?;
                None
            }
        };
        self.is_initialized = true;
        Ok(())
    }

    fn start_address(&self) -> u64 {
        self.row().start_address
    }

    fn set_start_address(&mut self, start_address: u64) {
        let row = self.row_mut();
        row.start_address = start_address;
    }

    fn set_register_rule(&mut self, register: Register, rule: RegisterRule<R>) -> Result<()> {
        let row = self.row_mut();
        row.registers.set(register, rule)
    }

    /// Returns `None` if we have not completed evaluation of a CIE's initial
    /// instructions.
    fn get_initial_rule(&self, register: Register) -> Option<RegisterRule<R>> {
        if !self.is_initialized {
            return None;
        }
        Some(match self.initial_rule {
            None => self.stack[0].registers.get(register),
            Some((r, ref rule)) if r == register => rule.clone(),
            _ => RegisterRule::Undefined,
        })
    }

    fn set_cfa(&mut self, cfa: CfaRule<R>) {
        self.row_mut().cfa = cfa;
    }

    fn cfa_mut(&mut self) -> &mut CfaRule<R> {
        &mut self.row_mut().cfa
    }

    fn push_row(&mut self) -> Result<()> {
        let new_row = self.row().clone();
        self.stack.try_push(new_row).map_err(|_| Error::StackFull)
    }

    fn pop_row(&mut self) -> Result<()> {
        let min_size = if self.is_initialized && self.initial_rule.is_none() {
            2
        } else {
            1
        };
        if self.stack.len() <= min_size {
            return Err(Error::PopWithEmptyStack);
        }
        self.stack.pop().unwrap();
        Ok(())
    }
}

/// The `UnwindTable` iteratively evaluates a `FrameDescriptionEntry`'s
/// `CallFrameInstruction` program, yielding the each row one at a time.
///
/// > 6.4.1 Structure of Call Frame Information
/// >
/// > DWARF supports virtual unwinding by defining an architecture independent
/// > basis for recording how procedures save and restore registers during their
/// > lifetimes. This basis must be augmented on some machines with specific
/// > information that is defined by an architecture specific ABI authoring
/// > committee, a hardware vendor, or a compiler producer. The body defining a
/// > specific augmentation is referred to below as the “augmenter.”
/// >
/// > Abstractly, this mechanism describes a very large table that has the
/// > following structure:
/// >
/// > <table>
/// >   <tr>
/// >     <th>LOC</th><th>CFA</th><th>R0</th><th>R1</th><td>...</td><th>RN</th>
/// >   </tr>
/// >   <tr>
/// >     <th>L0</th> <td></td>   <td></td>  <td></td>  <td></td>   <td></td>
/// >   </tr>
/// >   <tr>
/// >     <th>L1</th> <td></td>   <td></td>  <td></td>  <td></td>   <td></td>
/// >   </tr>
/// >   <tr>
/// >     <td>...</td><td></td>   <td></td>  <td></td>  <td></td>   <td></td>
/// >   </tr>
/// >   <tr>
/// >     <th>LN</th> <td></td>   <td></td>  <td></td>  <td></td>   <td></td>
/// >   </tr>
/// > </table>
/// >
/// > The first column indicates an address for every location that contains code
/// > in a program. (In shared objects, this is an object-relative offset.) The
/// > remaining columns contain virtual unwinding rules that are associated with
/// > the indicated location.
/// >
/// > The CFA column defines the rule which computes the Canonical Frame Address
/// > value; it may be either a register and a signed offset that are added
/// > together, or a DWARF expression that is evaluated.
/// >
/// > The remaining columns are labeled by register number. This includes some
/// > registers that have special designation on some architectures such as the PC
/// > and the stack pointer register. (The actual mapping of registers for a
/// > particular architecture is defined by the augmenter.) The register columns
/// > contain rules that describe whether a given register has been saved and the
/// > rule to find the value for the register in the previous frame.
/// >
/// > ...
/// >
/// > This table would be extremely large if actually constructed as
/// > described. Most of the entries at any point in the table are identical to
/// > the ones above them. The whole table can be represented quite compactly by
/// > recording just the differences starting at the beginning address of each
/// > subroutine in the program.
#[derive(Debug)]
pub struct UnwindTable<'a, 'ctx, R: Reader, A: UnwindContextStorage<R> = StoreOnHeap> {
    code_alignment_factor: Wrapping<u64>,
    data_alignment_factor: Wrapping<i64>,
    next_start_address: u64,
    last_end_address: u64,
    returned_last_row: bool,
    current_row_valid: bool,
    instructions: CallFrameInstructionIter<'a, R>,
    ctx: &'ctx mut UnwindContext<R, A>,
}

/// # Signal Safe Methods
///
/// These methods are guaranteed not to allocate, acquire locks, or perform any
/// other signal-unsafe operations.
impl<'a, 'ctx, R: Reader, A: UnwindContextStorage<R>> UnwindTable<'a, 'ctx, R, A> {
    /// Construct a new `UnwindTable` for the given
    /// `FrameDescriptionEntry`'s CFI unwinding program.
    pub fn new<Section: UnwindSection<R>>(
        section: &'a Section,
        bases: &'a BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
        fde: &FrameDescriptionEntry<R>,
    ) -> Result<Self> {
        ctx.initialize(section, bases, fde.cie())?;
        Ok(Self::new_for_fde(section, bases, ctx, fde))
    }

    fn new_for_fde<Section: UnwindSection<R>>(
        section: &'a Section,
        bases: &'a BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
        fde: &FrameDescriptionEntry<R>,
    ) -> Self {
        assert!(ctx.stack.len() >= 1);
        UnwindTable {
            code_alignment_factor: Wrapping(fde.cie().code_alignment_factor()),
            data_alignment_factor: Wrapping(fde.cie().data_alignment_factor()),
            next_start_address: fde.initial_address(),
            last_end_address: fde.initial_address().wrapping_add(fde.len()),
            returned_last_row: false,
            current_row_valid: false,
            instructions: fde.instructions(section, bases),
            ctx,
        }
    }

    fn new_for_cie<Section: UnwindSection<R>>(
        section: &'a Section,
        bases: &'a BaseAddresses,
        ctx: &'ctx mut UnwindContext<R, A>,
        cie: &CommonInformationEntry<R>,
    ) -> Self {
        assert!(ctx.stack.len() >= 1);
        UnwindTable {
            code_alignment_factor: Wrapping(cie.code_alignment_factor()),
            data_alignment_factor: Wrapping(cie.data_alignment_factor()),
            next_start_address: 0,
            last_end_address: 0,
            returned_last_row: false,
            current_row_valid: false,
            instructions: cie.instructions(section, bases),
            ctx,
        }
    }

    /// Evaluate call frame instructions until the next row of the table is
    /// completed, and return it.
    ///
    /// Unfortunately, this cannot be used with `FallibleIterator` because of
    /// the restricted lifetime of the yielded item.
    pub fn next_row(&mut self) -> Result<Option<&UnwindTableRow<R, A>>> {
        assert!(self.ctx.stack.len() >= 1);
        self.ctx.set_start_address(self.next_start_address);
        self.current_row_valid = false;

        loop {
            match self.instructions.next() {
                Err(e) => return Err(e),

                Ok(None) => {
                    if self.returned_last_row {
                        return Ok(None);
                    }

                    let row = self.ctx.row_mut();
                    row.end_address = self.last_end_address;

                    self.returned_last_row = true;
                    self.current_row_valid = true;
                    return Ok(Some(row));
                }

                Ok(Some(instruction)) => {
                    if self.evaluate(instruction)? {
                        self.current_row_valid = true;
                        return Ok(Some(self.ctx.row()));
                    }
                }
            };
        }
    }

    /// Returns the current row with the lifetime of the context.
    pub fn into_current_row(self) -> Option<&'ctx UnwindTableRow<R, A>> {
        if self.current_row_valid {
            Some(self.ctx.row())
        } else {
            None
        }
    }

    /// Evaluate one call frame instruction. Return `Ok(true)` if the row is
    /// complete, `Ok(false)` otherwise.
    fn evaluate(&mut self, instruction: CallFrameInstruction<R>) -> Result<bool> {
        use crate::CallFrameInstruction::*;

        match instruction {
            // Instructions that complete the current row and advance the
            // address for the next row.
            SetLoc { address } => {
                if address < self.ctx.start_address() {
                    return Err(Error::InvalidAddressRange);
                }

                self.next_start_address = address;
                self.ctx.row_mut().end_address = self.next_start_address;
                return Ok(true);
            }
            AdvanceLoc { delta } => {
                let delta = Wrapping(u64::from(delta)) * self.code_alignment_factor;
                self.next_start_address = (Wrapping(self.ctx.start_address()) + delta).0;
                self.ctx.row_mut().end_address = self.next_start_address;
                return Ok(true);
            }

            // Instructions that modify the CFA.
            DefCfa { register, offset } => {
                self.ctx.set_cfa(CfaRule::RegisterAndOffset {
                    register,
                    offset: offset as i64,
                });
            }
            DefCfaSf {
                register,
                factored_offset,
            } => {
                let data_align = self.data_alignment_factor;
                self.ctx.set_cfa(CfaRule::RegisterAndOffset {
                    register,
                    offset: (Wrapping(factored_offset) * data_align).0,
                });
            }
            DefCfaRegister { register } => {
                if let CfaRule::RegisterAndOffset {
                    register: ref mut reg,
                    ..
                } = *self.ctx.cfa_mut()
                {
                    *reg = register;
                } else {
                    return Err(Error::CfiInstructionInInvalidContext);
                }
            }
            DefCfaOffset { offset } => {
                if let CfaRule::RegisterAndOffset {
                    offset: ref mut off,
                    ..
                } = *self.ctx.cfa_mut()
                {
                    *off = offset as i64;
                } else {
                    return Err(Error::CfiInstructionInInvalidContext);
                }
            }
            DefCfaOffsetSf { factored_offset } => {
                if let CfaRule::RegisterAndOffset {
                    offset: ref mut off,
                    ..
                } = *self.ctx.cfa_mut()
                {
                    let data_align = self.data_alignment_factor;
                    *off = (Wrapping(factored_offset) * data_align).0;
                } else {
                    return Err(Error::CfiInstructionInInvalidContext);
                }
            }
            DefCfaExpression { expression } => {
                self.ctx.set_cfa(CfaRule::Expression(expression));
            }

            // Instructions that define register rules.
            Undefined { register } => {
                self.ctx
                    .set_register_rule(register, RegisterRule::Undefined)?;
            }
            SameValue { register } => {
                self.ctx
                    .set_register_rule(register, RegisterRule::SameValue)?;
            }
            Offset {
                register,
                factored_offset,
            } => {
                let offset = Wrapping(factored_offset as i64) * self.data_alignment_factor;
                self.ctx
                    .set_register_rule(register, RegisterRule::Offset(offset.0))?;
            }
            OffsetExtendedSf {
                register,
                factored_offset,
            } => {
                let offset = Wrapping(factored_offset) * self.data_alignment_factor;
                self.ctx
                    .set_register_rule(register, RegisterRule::Offset(offset.0))?;
            }
            ValOffset {
                register,
                factored_offset,
            } => {
                let offset = Wrapping(factored_offset as i64) * self.data_alignment_factor;
                self.ctx
                    .set_register_rule(register, RegisterRule::ValOffset(offset.0))?;
            }
            ValOffsetSf {
                register,
                factored_offset,
            } => {
                let offset = Wrapping(factored_offset) * self.data_alignment_factor;
                self.ctx
                    .set_register_rule(register, RegisterRule::ValOffset(offset.0))?;
            }
            Register {
                dest_register,
                src_register,
            } => {
                self.ctx
                    .set_register_rule(dest_register, RegisterRule::Register(src_register))?;
            }
            Expression {
                register,
                expression,
            } => {
                let expression = RegisterRule::Expression(expression);
                self.ctx.set_register_rule(register, expression)?;
            }
            ValExpression {
                register,
                expression,
            } => {
                let expression = RegisterRule::ValExpression(expression);
                self.ctx.set_register_rule(register, expression)?;
            }
            Restore { register } => {
                let initial_rule = if let Some(rule) = self.ctx.get_initial_rule(register) {
                    rule
                } else {
                    // Can't restore the initial rule when we are
                    // evaluating the initial rules!
                    return Err(Error::CfiInstructionInInvalidContext);
                };

                self.ctx.set_register_rule(register, initial_rule)?;
            }

            // Row push and pop instructions.
            RememberState => {
                self.ctx.push_row()?;
            }
            RestoreState => {
                // Pop state while preserving current location.
                let start_address = self.ctx.start_address();
                self.ctx.pop_row()?;
                self.ctx.set_start_address(start_address);
            }

            // GNU Extension. Save the size somewhere so the unwinder can use
            // it when restoring IP
            ArgsSize { size } => {
                self.ctx.row_mut().saved_args_size = size;
            }

            // No operation.
            Nop => {}
        };

        Ok(false)
    }
}

// We tend to have very few register rules: usually only a couple. Even if we
// have a rule for every register, on x86-64 with SSE and everything we're
// talking about ~100 rules. So rather than keeping the rules in a hash map, or
// a vector indexed by register number (which would lead to filling lots of
// empty entries), we store them as a vec of (register number, register rule)
// pairs.
//
// Additionally, because every register's default rule is implicitly
// `RegisterRule::Undefined`, we never store a register's rule in this vec if it
// is undefined and save a little bit more space and do a little fewer
// comparisons that way.
//
// The maximum number of rules preallocated by libunwind is 97 for AArch64, 128
// for ARM, and even 188 for MIPS. It is extremely unlikely to encounter this
// many register rules in practice.
//
// See:
// - https://github.com/libunwind/libunwind/blob/11fd461095ea98f4b3e3a361f5a8a558519363fa/include/tdep-x86_64/dwarf-config.h#L36
// - https://github.com/libunwind/libunwind/blob/11fd461095ea98f4b3e3a361f5a8a558519363fa/include/tdep-aarch64/dwarf-config.h#L32
// - https://github.com/libunwind/libunwind/blob/11fd461095ea98f4b3e3a361f5a8a558519363fa/include/tdep-arm/dwarf-config.h#L31
// - https://github.com/libunwind/libunwind/blob/11fd461095ea98f4b3e3a361f5a8a558519363fa/include/tdep-mips/dwarf-config.h#L31
struct RegisterRuleMap<R: Reader, S: UnwindContextStorage<R> = StoreOnHeap> {
    rules: ArrayVec<S::Rules>,
}

impl<R: Reader, S: UnwindContextStorage<R>> Debug for RegisterRuleMap<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RegisterRuleMap")
            .field("rules", &self.rules)
            .finish()
    }
}

impl<R: Reader, S: UnwindContextStorage<R>> Clone for RegisterRuleMap<R, S> {
    fn clone(&self) -> Self {
        Self {
            rules: self.rules.clone(),
        }
    }
}

impl<R: Reader, S: UnwindContextStorage<R>> Default for RegisterRuleMap<R, S> {
    fn default() -> Self {
        RegisterRuleMap {
            rules: Default::default(),
        }
    }
}

/// # Signal Safe Methods
///
/// These methods are guaranteed not to allocate, acquire locks, or perform any
/// other signal-unsafe operations.
impl<R: Reader, S: UnwindContextStorage<R>> RegisterRuleMap<R, S> {
    fn is_default(&self) -> bool {
        self.rules.is_empty()
    }

    fn get(&self, register: Register) -> RegisterRule<R> {
        self.rules
            .iter()
            .find(|rule| rule.0 == register)
            .map(|r| {
                debug_assert!(r.1.is_defined());
                r.1.clone()
            })
            .unwrap_or(RegisterRule::Undefined)
    }

    fn set(&mut self, register: Register, rule: RegisterRule<R>) -> Result<()> {
        if !rule.is_defined() {
            let idx = self
                .rules
                .iter()
                .enumerate()
                .find(|&(_, r)| r.0 == register)
                .map(|(i, _)| i);
            if let Some(idx) = idx {
                self.rules.swap_remove(idx);
            }
            return Ok(());
        }

        for &mut (reg, ref mut old_rule) in &mut *self.rules {
            debug_assert!(old_rule.is_defined());
            if reg == register {
                *old_rule = rule;
                return Ok(());
            }
        }

        self.rules
            .try_push((register, rule))
            .map_err(|_| Error::TooManyRegisterRules)
    }

    fn iter(&self) -> RegisterRuleIter<R> {
        RegisterRuleIter(self.rules.iter())
    }
}

impl<'a, R, S: UnwindContextStorage<R>> FromIterator<&'a (Register, RegisterRule<R>)>
    for RegisterRuleMap<R, S>
where
    R: 'a + Reader,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a (Register, RegisterRule<R>)>,
    {
        let iter = iter.into_iter();
        let mut rules = RegisterRuleMap::default();
        for &(reg, ref rule) in iter.filter(|r| r.1.is_defined()) {
            rules.set(reg, rule.clone()).expect(
                "This is only used in tests, impl isn't exposed publicly.
                         If you trip this, fix your test",
            );
        }
        rules
    }
}

impl<R, S: UnwindContextStorage<R>> PartialEq for RegisterRuleMap<R, S>
where
    R: Reader + PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        for &(reg, ref rule) in &*self.rules {
            debug_assert!(rule.is_defined());
            if *rule != rhs.get(reg) {
                return false;
            }
        }

        for &(reg, ref rhs_rule) in &*rhs.rules {
            debug_assert!(rhs_rule.is_defined());
            if *rhs_rule != self.get(reg) {
                return false;
            }
        }

        true
    }
}

impl<R, S: UnwindContextStorage<R>> Eq for RegisterRuleMap<R, S> where R: Reader + Eq {}

/// An unordered iterator for register rules.
#[derive(Debug, Clone)]
pub struct RegisterRuleIter<'iter, R>(::core::slice::Iter<'iter, (Register, RegisterRule<R>)>)
where
    R: Reader;

impl<'iter, R: Reader> Iterator for RegisterRuleIter<'iter, R> {
    type Item = &'iter (Register, RegisterRule<R>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A row in the virtual unwind table that describes how to find the values of
/// the registers in the *previous* frame for a range of PC addresses.
#[derive(PartialEq, Eq)]
pub struct UnwindTableRow<R: Reader, S: UnwindContextStorage<R> = StoreOnHeap> {
    start_address: u64,
    end_address: u64,
    saved_args_size: u64,
    cfa: CfaRule<R>,
    registers: RegisterRuleMap<R, S>,
}

impl<R: Reader, S: UnwindContextStorage<R>> Debug for UnwindTableRow<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnwindTableRow")
            .field("start_address", &self.start_address)
            .field("end_address", &self.end_address)
            .field("saved_args_size", &self.saved_args_size)
            .field("cfa", &self.cfa)
            .field("registers", &self.registers)
            .finish()
    }
}

impl<R: Reader, S: UnwindContextStorage<R>> Clone for UnwindTableRow<R, S> {
    fn clone(&self) -> Self {
        Self {
            start_address: self.start_address,
            end_address: self.end_address,
            saved_args_size: self.saved_args_size,
            cfa: self.cfa.clone(),
            registers: self.registers.clone(),
        }
    }
}

impl<R: Reader, S: UnwindContextStorage<R>> Default for UnwindTableRow<R, S> {
    fn default() -> Self {
        UnwindTableRow {
            start_address: 0,
            end_address: 0,
            saved_args_size: 0,
            cfa: Default::default(),
            registers: Default::default(),
        }
    }
}

impl<R: Reader, S: UnwindContextStorage<R>> UnwindTableRow<R, S> {
    fn is_default(&self) -> bool {
        self.start_address == 0
            && self.end_address == 0
            && self.cfa.is_default()
            && self.registers.is_default()
    }

    /// Get the starting PC address that this row applies to.
    pub fn start_address(&self) -> u64 {
        self.start_address
    }

    /// Get the end PC address where this row's register rules become
    /// unapplicable.
    ///
    /// In other words, this row describes how to recover the last frame's
    /// registers for all PCs where `row.start_address() <= PC <
    /// row.end_address()`. This row does NOT describe how to recover registers
    /// when `PC == row.end_address()`.
    pub fn end_address(&self) -> u64 {
        self.end_address
    }

    /// Return `true` if the given `address` is within this row's address range,
    /// `false` otherwise.
    pub fn contains(&self, address: u64) -> bool {
        self.start_address <= address && address < self.end_address
    }

    /// Returns the amount of args currently on the stack.
    ///
    /// When unwinding, if the personality function requested a change in IP,
    /// the SP needs to be adjusted by saved_args_size.
    pub fn saved_args_size(&self) -> u64 {
        self.saved_args_size
    }

    /// Get the canonical frame address (CFA) recovery rule for this row.
    pub fn cfa(&self) -> &CfaRule<R> {
        &self.cfa
    }

    /// Get the register recovery rule for the given register number.
    ///
    /// The register number mapping is architecture dependent. For example, in
    /// the x86-64 ABI the register number mapping is defined in Figure 3.36:
    ///
    /// > Figure 3.36: DWARF Register Number Mapping
    /// >
    /// > <table>
    /// >   <tr><th>Register Name</th>                    <th>Number</th>  <th>Abbreviation</th></tr>
    /// >   <tr><td>General Purpose Register RAX</td>     <td>0</td>       <td>%rax</td></tr>
    /// >   <tr><td>General Purpose Register RDX</td>     <td>1</td>       <td>%rdx</td></tr>
    /// >   <tr><td>General Purpose Register RCX</td>     <td>2</td>       <td>%rcx</td></tr>
    /// >   <tr><td>General Purpose Register RBX</td>     <td>3</td>       <td>%rbx</td></tr>
    /// >   <tr><td>General Purpose Register RSI</td>     <td>4</td>       <td>%rsi</td></tr>
    /// >   <tr><td>General Purpose Register RDI</td>     <td>5</td>       <td>%rdi</td></tr>
    /// >   <tr><td>General Purpose Register RBP</td>     <td>6</td>       <td>%rbp</td></tr>
    /// >   <tr><td>Stack Pointer Register RSP</td>       <td>7</td>       <td>%rsp</td></tr>
    /// >   <tr><td>Extended Integer Registers 8-15</td>  <td>8-15</td>    <td>%r8-%r15</td></tr>
    /// >   <tr><td>Return Address RA</td>                <td>16</td>      <td></td></tr>
    /// >   <tr><td>Vector Registers 0–7</td>             <td>17-24</td>   <td>%xmm0–%xmm7</td></tr>
    /// >   <tr><td>Extended Vector Registers 8–15</td>   <td>25-32</td>   <td>%xmm8–%xmm15</td></tr>
    /// >   <tr><td>Floating Point Registers 0–7</td>     <td>33-40</td>   <td>%st0–%st7</td></tr>
    /// >   <tr><td>MMX Registers 0–7</td>                <td>41-48</td>   <td>%mm0–%mm7</td></tr>
    /// >   <tr><td>Flag Register</td>                    <td>49</td>      <td>%rFLAGS</td></tr>
    /// >   <tr><td>Segment Register ES</td>              <td>50</td>      <td>%es</td></tr>
    /// >   <tr><td>Segment Register CS</td>              <td>51</td>      <td>%cs</td></tr>
    /// >   <tr><td>Segment Register SS</td>              <td>52</td>      <td>%ss</td></tr>
    /// >   <tr><td>Segment Register DS</td>              <td>53</td>      <td>%ds</td></tr>
    /// >   <tr><td>Segment Register FS</td>              <td>54</td>      <td>%fs</td></tr>
    /// >   <tr><td>Segment Register GS</td>              <td>55</td>      <td>%gs</td></tr>
    /// >   <tr><td>Reserved</td>                         <td>56-57</td>   <td></td></tr>
    /// >   <tr><td>FS Base address</td>                  <td>58</td>      <td>%fs.base</td></tr>
    /// >   <tr><td>GS Base address</td>                  <td>59</td>      <td>%gs.base</td></tr>
    /// >   <tr><td>Reserved</td>                         <td>60-61</td>   <td></td></tr>
    /// >   <tr><td>Task Register</td>                    <td>62</td>      <td>%tr</td></tr>
    /// >   <tr><td>LDT Register</td>                     <td>63</td>      <td>%ldtr</td></tr>
    /// >   <tr><td>128-bit Media Control and Status</td> <td>64</td>      <td>%mxcsr</td></tr>
    /// >   <tr><td>x87 Control Word</td>                 <td>65</td>      <td>%fcw</td></tr>
    /// >   <tr><td>x87 Status Word</td>                  <td>66</td>      <td>%fsw</td></tr>
    /// >   <tr><td>Upper Vector Registers 16–31</td>     <td>67-82</td>   <td>%xmm16–%xmm31</td></tr>
    /// >   <tr><td>Reserved</td>                         <td>83-117</td>  <td></td></tr>
    /// >   <tr><td>Vector Mask Registers 0–7</td>        <td>118-125</td> <td>%k0–%k7</td></tr>
    /// >   <tr><td>Reserved</td>                         <td>126-129</td> <td></td></tr>
    /// > </table>
    pub fn register(&self, register: Register) -> RegisterRule<R> {
        self.registers.get(register)
    }

    /// Iterate over all defined register `(number, rule)` pairs.
    ///
    /// The rules are not iterated in any guaranteed order. Any register that
    /// does not make an appearance in the iterator implicitly has the rule
    /// `RegisterRule::Undefined`.
    ///
    /// ```
    /// # use gimli::{EndianSlice, LittleEndian, UnwindTableRow};
    /// # fn foo<'input>(unwind_table_row: UnwindTableRow<EndianSlice<'input, LittleEndian>>) {
    /// for &(register, ref rule) in unwind_table_row.registers() {
    ///     // ...
    ///     # drop(register); drop(rule);
    /// }
    /// # }
    /// ```
    pub fn registers(&self) -> RegisterRuleIter<R> {
        self.registers.iter()
    }
}

/// The canonical frame address (CFA) recovery rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CfaRule<R: Reader> {
    /// The CFA is given offset from the given register's value.
    RegisterAndOffset {
        /// The register containing the base value.
        register: Register,
        /// The offset from the register's base value.
        offset: i64,
    },
    /// The CFA is obtained by evaluating this `Reader` as a DWARF expression
    /// program.
    Expression(Expression<R>),
}

impl<R: Reader> Default for CfaRule<R> {
    fn default() -> Self {
        CfaRule::RegisterAndOffset {
            register: Register(0),
            offset: 0,
        }
    }
}

impl<R: Reader> CfaRule<R> {
    fn is_default(&self) -> bool {
        match *self {
            CfaRule::RegisterAndOffset { register, offset } => {
                register == Register(0) && offset == 0
            }
            _ => false,
        }
    }
}

/// An entry in the abstract CFI table that describes how to find the value of a
/// register.
///
/// "The register columns contain rules that describe whether a given register
/// has been saved and the rule to find the value for the register in the
/// previous frame."
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterRule<R: Reader> {
    /// > A register that has this rule has no recoverable value in the previous
    /// > frame. (By convention, it is not preserved by a callee.)
    Undefined,

    /// > This register has not been modified from the previous frame. (By
    /// > convention, it is preserved by the callee, but the callee has not
    /// > modified it.)
    SameValue,

    /// "The previous value of this register is saved at the address CFA+N where
    /// CFA is the current CFA value and N is a signed offset."
    Offset(i64),

    /// "The previous value of this register is the value CFA+N where CFA is the
    /// current CFA value and N is a signed offset."
    ValOffset(i64),

    /// "The previous value of this register is stored in another register
    /// numbered R."
    Register(Register),

    /// "The previous value of this register is located at the address produced
    /// by executing the DWARF expression."
    Expression(Expression<R>),

    /// "The previous value of this register is the value produced by executing
    /// the DWARF expression."
    ValExpression(Expression<R>),

    /// "The rule is defined externally to this specification by the augmenter."
    Architectural,
}

impl<R: Reader> RegisterRule<R> {
    fn is_defined(&self) -> bool {
        match *self {
            RegisterRule::Undefined => false,
            _ => true,
        }
    }
}

/// A parsed call frame instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallFrameInstruction<R: Reader> {
    // 6.4.2.1 Row Creation Methods
    /// > 1. DW_CFA_set_loc
    /// >
    /// > The DW_CFA_set_loc instruction takes a single operand that represents
    /// > a target address. The required action is to create a new table row
    /// > using the specified address as the location. All other values in the
    /// > new row are initially identical to the current row. The new location
    /// > value is always greater than the current one. If the segment_size
    /// > field of this FDE's CIE is non- zero, the initial location is preceded
    /// > by a segment selector of the given length.
    SetLoc {
        /// The target address.
        address: u64,
    },

    /// The `AdvanceLoc` instruction is used for all of `DW_CFA_advance_loc` and
    /// `DW_CFA_advance_loc{1,2,4}`.
    ///
    /// > 2. DW_CFA_advance_loc
    /// >
    /// > The DW_CFA_advance instruction takes a single operand (encoded with
    /// > the opcode) that represents a constant delta. The required action is
    /// > to create a new table row with a location value that is computed by
    /// > taking the current entry’s location value and adding the value of
    /// > delta * code_alignment_factor. All other values in the new row are
    /// > initially identical to the current row.
    AdvanceLoc {
        /// The delta to be added to the current address.
        delta: u32,
    },

    // 6.4.2.2 CFA Definition Methods
    /// > 1. DW_CFA_def_cfa
    /// >
    /// > The DW_CFA_def_cfa instruction takes two unsigned LEB128 operands
    /// > representing a register number and a (non-factored) offset. The
    /// > required action is to define the current CFA rule to use the provided
    /// > register and offset.
    DefCfa {
        /// The target register's number.
        register: Register,
        /// The non-factored offset.
        offset: u64,
    },

    /// > 2. DW_CFA_def_cfa_sf
    /// >
    /// > The DW_CFA_def_cfa_sf instruction takes two operands: an unsigned
    /// > LEB128 value representing a register number and a signed LEB128
    /// > factored offset. This instruction is identical to DW_CFA_def_cfa
    /// > except that the second operand is signed and factored. The resulting
    /// > offset is factored_offset * data_alignment_factor.
    DefCfaSf {
        /// The target register's number.
        register: Register,
        /// The factored offset.
        factored_offset: i64,
    },

    /// > 3. DW_CFA_def_cfa_register
    /// >
    /// > The DW_CFA_def_cfa_register instruction takes a single unsigned LEB128
    /// > operand representing a register number. The required action is to
    /// > define the current CFA rule to use the provided register (but to keep
    /// > the old offset). This operation is valid only if the current CFA rule
    /// > is defined to use a register and offset.
    DefCfaRegister {
        /// The target register's number.
        register: Register,
    },

    /// > 4. DW_CFA_def_cfa_offset
    /// >
    /// > The DW_CFA_def_cfa_offset instruction takes a single unsigned LEB128
    /// > operand representing a (non-factored) offset. The required action is
    /// > to define the current CFA rule to use the provided offset (but to keep
    /// > the old register). This operation is valid only if the current CFA
    /// > rule is defined to use a register and offset.
    DefCfaOffset {
        /// The non-factored offset.
        offset: u64,
    },

    /// > 5. DW_CFA_def_cfa_offset_sf
    /// >
    /// > The DW_CFA_def_cfa_offset_sf instruction takes a signed LEB128 operand
    /// > representing a factored offset. This instruction is identical to
    /// > DW_CFA_def_cfa_offset except that the operand is signed and
    /// > factored. The resulting offset is factored_offset *
    /// > data_alignment_factor. This operation is valid only if the current CFA
    /// > rule is defined to use a register and offset.
    DefCfaOffsetSf {
        /// The factored offset.
        factored_offset: i64,
    },

    /// > 6. DW_CFA_def_cfa_expression
    /// >
    /// > The DW_CFA_def_cfa_expression instruction takes a single operand
    /// > encoded as a DW_FORM_exprloc value representing a DWARF
    /// > expression. The required action is to establish that expression as the
    /// > means by which the current CFA is computed.
    DefCfaExpression {
        /// The DWARF expression.
        expression: Expression<R>,
    },

    // 6.4.2.3 Register Rule Instructions
    /// > 1. DW_CFA_undefined
    /// >
    /// > The DW_CFA_undefined instruction takes a single unsigned LEB128
    /// > operand that represents a register number. The required action is to
    /// > set the rule for the specified register to “undefined.”
    Undefined {
        /// The target register's number.
        register: Register,
    },

    /// > 2. DW_CFA_same_value
    /// >
    /// > The DW_CFA_same_value instruction takes a single unsigned LEB128
    /// > operand that represents a register number. The required action is to
    /// > set the rule for the specified register to “same value.”
    SameValue {
        /// The target register's number.
        register: Register,
    },

    /// The `Offset` instruction represents both `DW_CFA_offset` and
    /// `DW_CFA_offset_extended`.
    ///
    /// > 3. DW_CFA_offset
    /// >
    /// > The DW_CFA_offset instruction takes two operands: a register number
    /// > (encoded with the opcode) and an unsigned LEB128 constant representing
    /// > a factored offset. The required action is to change the rule for the
    /// > register indicated by the register number to be an offset(N) rule
    /// > where the value of N is factored offset * data_alignment_factor.
    Offset {
        /// The target register's number.
        register: Register,
        /// The factored offset.
        factored_offset: u64,
    },

    /// > 5. DW_CFA_offset_extended_sf
    /// >
    /// > The DW_CFA_offset_extended_sf instruction takes two operands: an
    /// > unsigned LEB128 value representing a register number and a signed
    /// > LEB128 factored offset. This instruction is identical to
    /// > DW_CFA_offset_extended except that the second operand is signed and
    /// > factored. The resulting offset is factored_offset *
    /// > data_alignment_factor.
    OffsetExtendedSf {
        /// The target register's number.
        register: Register,
        /// The factored offset.
        factored_offset: i64,
    },

    /// > 6. DW_CFA_val_offset
    /// >
    /// > The DW_CFA_val_offset instruction takes two unsigned LEB128 operands
    /// > representing a register number and a factored offset. The required
    /// > action is to change the rule for the register indicated by the
    /// > register number to be a val_offset(N) rule where the value of N is
    /// > factored_offset * data_alignment_factor.
    ValOffset {
        /// The target register's number.
        register: Register,
        /// The factored offset.
        factored_offset: u64,
    },

    /// > 7. DW_CFA_val_offset_sf
    /// >
    /// > The DW_CFA_val_offset_sf instruction takes two operands: an unsigned
    /// > LEB128 value representing a register number and a signed LEB128
    /// > factored offset. This instruction is identical to DW_CFA_val_offset
    /// > except that the second operand is signed and factored. The resulting
    /// > offset is factored_offset * data_alignment_factor.
    ValOffsetSf {
        /// The target register's number.
        register: Register,
        /// The factored offset.
        factored_offset: i64,
    },

    /// > 8. DW_CFA_register
    /// >
    /// > The DW_CFA_register instruction takes two unsigned LEB128 operands
    /// > representing register numbers. The required action is to set the rule
    /// > for the first register to be register(R) where R is the second
    /// > register.
    Register {
        /// The number of the register whose rule is being changed.
        dest_register: Register,
        /// The number of the register where the other register's value can be
        /// found.
        src_register: Register,
    },

    /// > 9. DW_CFA_expression
    /// >
    /// > The DW_CFA_expression instruction takes two operands: an unsigned
    /// > LEB128 value representing a register number, and a DW_FORM_block value
    /// > representing a DWARF expression. The required action is to change the
    /// > rule for the register indicated by the register number to be an
    /// > expression(E) rule where E is the DWARF expression. That is, the DWARF
    /// > expression computes the address. The value of the CFA is pushed on the
    /// > DWARF evaluation stack prior to execution of the DWARF expression.
    Expression {
        /// The target register's number.
        register: Register,
        /// The DWARF expression.
        expression: Expression<R>,
    },

    /// > 10. DW_CFA_val_expression
    /// >
    /// > The DW_CFA_val_expression instruction takes two operands: an unsigned
    /// > LEB128 value representing a register number, and a DW_FORM_block value
    /// > representing a DWARF expression. The required action is to change the
    /// > rule for the register indicated by the register number to be a
    /// > val_expression(E) rule where E is the DWARF expression. That is, the
    /// > DWARF expression computes the value of the given register. The value
    /// > of the CFA is pushed on the DWARF evaluation stack prior to execution
    /// > of the DWARF expression.
    ValExpression {
        /// The target register's number.
        register: Register,
        /// The DWARF expression.
        expression: Expression<R>,
    },

    /// The `Restore` instruction represents both `DW_CFA_restore` and
    /// `DW_CFA_restore_extended`.
    ///
    /// > 11. DW_CFA_restore
    /// >
    /// > The DW_CFA_restore instruction takes a single operand (encoded with
    /// > the opcode) that represents a register number. The required action is
    /// > to change the rule for the indicated register to the rule assigned it
    /// > by the initial_instructions in the CIE.
    Restore {
        /// The register to be reset.
        register: Register,
    },

    // 6.4.2.4 Row State Instructions
    /// > 1. DW_CFA_remember_state
    /// >
    /// > The DW_CFA_remember_state instruction takes no operands. The required
    /// > action is to push the set of rules for every register onto an implicit
    /// > stack.
    RememberState,

    /// > 2. DW_CFA_restore_state
    /// >
    /// > The DW_CFA_restore_state instruction takes no operands. The required
    /// > action is to pop the set of rules off the implicit stack and place
    /// > them in the current row.
    RestoreState,

    /// > DW_CFA_GNU_args_size
    /// >
    /// > GNU Extension
    /// >
    /// > The DW_CFA_GNU_args_size instruction takes an unsigned LEB128 operand
    /// > representing an argument size. This instruction specifies the total of
    /// > the size of the arguments which have been pushed onto the stack.
    ArgsSize {
        /// The size of the arguments which have been pushed onto the stack
        size: u64,
    },

    // 6.4.2.5 Padding Instruction
    /// > 1. DW_CFA_nop
    /// >
    /// > The DW_CFA_nop instruction has no operands and no required actions. It
    /// > is used as padding to make a CIE or FDE an appropriate size.
    Nop,
}

const CFI_INSTRUCTION_HIGH_BITS_MASK: u8 = 0b1100_0000;
const CFI_INSTRUCTION_LOW_BITS_MASK: u8 = !CFI_INSTRUCTION_HIGH_BITS_MASK;

impl<R: Reader> CallFrameInstruction<R> {
    fn parse(
        input: &mut R,
        address_encoding: Option<DwEhPe>,
        parameters: &PointerEncodingParameters<R>,
    ) -> Result<CallFrameInstruction<R>> {
        let instruction = input.read_u8()?;
        let high_bits = instruction & CFI_INSTRUCTION_HIGH_BITS_MASK;

        if high_bits == constants::DW_CFA_advance_loc.0 {
            let delta = instruction & CFI_INSTRUCTION_LOW_BITS_MASK;
            return Ok(CallFrameInstruction::AdvanceLoc {
                delta: u32::from(delta),
            });
        }

        if high_bits == constants::DW_CFA_offset.0 {
            let register = Register((instruction & CFI_INSTRUCTION_LOW_BITS_MASK).into());
            let offset = input.read_uleb128()?;
            return Ok(CallFrameInstruction::Offset {
                register,
                factored_offset: offset,
            });
        }

        if high_bits == constants::DW_CFA_restore.0 {
            let register = Register((instruction & CFI_INSTRUCTION_LOW_BITS_MASK).into());
            return Ok(CallFrameInstruction::Restore { register });
        }

        debug_assert_eq!(high_bits, 0);
        let instruction = constants::DwCfa(instruction);

        match instruction {
            constants::DW_CFA_nop => Ok(CallFrameInstruction::Nop),

            constants::DW_CFA_set_loc => {
                let address = if let Some(encoding) = address_encoding {
                    match parse_encoded_pointer(encoding, parameters, input)? {
                        Pointer::Direct(x) => x,
                        _ => return Err(Error::UnsupportedPointerEncoding),
                    }
                } else {
                    input.read_address(parameters.address_size)?
                };
                Ok(CallFrameInstruction::SetLoc { address })
            }

            constants::DW_CFA_advance_loc1 => {
                let delta = input.read_u8()?;
                Ok(CallFrameInstruction::AdvanceLoc {
                    delta: u32::from(delta),
                })
            }

            constants::DW_CFA_advance_loc2 => {
                let delta = input.read_u16()?;
                Ok(CallFrameInstruction::AdvanceLoc {
                    delta: u32::from(delta),
                })
            }

            constants::DW_CFA_advance_loc4 => {
                let delta = input.read_u32()?;
                Ok(CallFrameInstruction::AdvanceLoc { delta })
            }

            constants::DW_CFA_offset_extended => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let offset = input.read_uleb128()?;
                Ok(CallFrameInstruction::Offset {
                    register,
                    factored_offset: offset,
                })
            }

            constants::DW_CFA_restore_extended => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                Ok(CallFrameInstruction::Restore { register })
            }

            constants::DW_CFA_undefined => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                Ok(CallFrameInstruction::Undefined { register })
            }

            constants::DW_CFA_same_value => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                Ok(CallFrameInstruction::SameValue { register })
            }

            constants::DW_CFA_register => {
                let dest = input.read_uleb128().and_then(Register::from_u64)?;
                let src = input.read_uleb128().and_then(Register::from_u64)?;
                Ok(CallFrameInstruction::Register {
                    dest_register: dest,
                    src_register: src,
                })
            }

            constants::DW_CFA_remember_state => Ok(CallFrameInstruction::RememberState),

            constants::DW_CFA_restore_state => Ok(CallFrameInstruction::RestoreState),

            constants::DW_CFA_def_cfa => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let offset = input.read_uleb128()?;
                Ok(CallFrameInstruction::DefCfa { register, offset })
            }

            constants::DW_CFA_def_cfa_register => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                Ok(CallFrameInstruction::DefCfaRegister { register })
            }

            constants::DW_CFA_def_cfa_offset => {
                let offset = input.read_uleb128()?;
                Ok(CallFrameInstruction::DefCfaOffset { offset })
            }

            constants::DW_CFA_def_cfa_expression => {
                let len = input.read_uleb128().and_then(R::Offset::from_u64)?;
                let expression = input.split(len)?;
                Ok(CallFrameInstruction::DefCfaExpression {
                    expression: Expression(expression),
                })
            }

            constants::DW_CFA_expression => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let len = input.read_uleb128().and_then(R::Offset::from_u64)?;
                let expression = input.split(len)?;
                Ok(CallFrameInstruction::Expression {
                    register,
                    expression: Expression(expression),
                })
            }

            constants::DW_CFA_offset_extended_sf => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let offset = input.read_sleb128()?;
                Ok(CallFrameInstruction::OffsetExtendedSf {
                    register,
                    factored_offset: offset,
                })
            }

            constants::DW_CFA_def_cfa_sf => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let offset = input.read_sleb128()?;
                Ok(CallFrameInstruction::DefCfaSf {
                    register,
                    factored_offset: offset,
                })
            }

            constants::DW_CFA_def_cfa_offset_sf => {
                let offset = input.read_sleb128()?;
                Ok(CallFrameInstruction::DefCfaOffsetSf {
                    factored_offset: offset,
                })
            }

            constants::DW_CFA_val_offset => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let offset = input.read_uleb128()?;
                Ok(CallFrameInstruction::ValOffset {
                    register,
                    factored_offset: offset,
                })
            }

            constants::DW_CFA_val_offset_sf => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let offset = input.read_sleb128()?;
                Ok(CallFrameInstruction::ValOffsetSf {
                    register,
                    factored_offset: offset,
                })
            }

            constants::DW_CFA_val_expression => {
                let register = input.read_uleb128().and_then(Register::from_u64)?;
                let len = input.read_uleb128().and_then(R::Offset::from_u64)?;
                let expression = input.split(len)?;
                Ok(CallFrameInstruction::ValExpression {
                    register,
                    expression: Expression(expression),
                })
            }

            constants::DW_CFA_GNU_args_size => {
                let size = input.read_uleb128()?;
                Ok(CallFrameInstruction::ArgsSize { size })
            }

            otherwise => Err(Error::UnknownCallFrameInstruction(otherwise)),
        }
    }
}

/// A lazy iterator parsing call frame instructions.
///
/// Can be [used with
/// `FallibleIterator`](./index.html#using-with-fallibleiterator).
#[derive(Clone, Debug)]
pub struct CallFrameInstructionIter<'a, R: Reader> {
    input: R,
    address_encoding: Option<constants::DwEhPe>,
    parameters: PointerEncodingParameters<'a, R>,
}

impl<'a, R: Reader> CallFrameInstructionIter<'a, R> {
    /// Parse the next call frame instruction.
    pub fn next(&mut self) -> Result<Option<CallFrameInstruction<R>>> {
        if self.input.is_empty() {
            return Ok(None);
        }

        match CallFrameInstruction::parse(&mut self.input, self.address_encoding, &self.parameters)
        {
            Ok(instruction) => Ok(Some(instruction)),
            Err(e) => {
                self.input.empty();
                Err(e)
            }
        }
    }
}

#[cfg(feature = "fallible-iterator")]
impl<'a, R: Reader> fallible_iterator::FallibleIterator for CallFrameInstructionIter<'a, R> {
    type Item = CallFrameInstruction<R>;
    type Error = Error;

    fn next(&mut self) -> ::core::result::Result<Option<Self::Item>, Self::Error> {
        CallFrameInstructionIter::next(self)
    }
}

/// Parse a `DW_EH_PE_*` pointer encoding.
#[doc(hidden)]
#[inline]
fn parse_pointer_encoding<R: Reader>(input: &mut R) -> Result<constants::DwEhPe> {
    let eh_pe = input.read_u8()?;
    let eh_pe = constants::DwEhPe(eh_pe);

    if eh_pe.is_valid_encoding() {
        Ok(eh_pe)
    } else {
        Err(Error::UnknownPointerEncoding)
    }
}

/// A decoded pointer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Pointer {
    /// This value is the decoded pointer value.
    Direct(u64),

    /// This value is *not* the pointer value, but points to the address of
    /// where the real pointer value lives. In other words, deref this pointer
    /// to get the real pointer value.
    ///
    /// Chase this pointer at your own risk: do you trust the DWARF data it came
    /// from?
    Indirect(u64),
}

impl Default for Pointer {
    #[inline]
    fn default() -> Self {
        Pointer::Direct(0)
    }
}

impl Into<u64> for Pointer {
    #[inline]
    fn into(self) -> u64 {
        match self {
            Pointer::Direct(p) | Pointer::Indirect(p) => p,
        }
    }
}

impl Pointer {
    #[inline]
    fn new(encoding: constants::DwEhPe, address: u64) -> Pointer {
        if encoding.is_indirect() {
            Pointer::Indirect(address)
        } else {
            Pointer::Direct(address)
        }
    }
}

#[derive(Clone, Debug)]
struct PointerEncodingParameters<'a, R: Reader> {
    bases: &'a SectionBaseAddresses,
    func_base: Option<u64>,
    address_size: u8,
    section: &'a R,
}

fn parse_encoded_pointer<R: Reader>(
    encoding: constants::DwEhPe,
    parameters: &PointerEncodingParameters<R>,
    input: &mut R,
) -> Result<Pointer> {
    // TODO: check this once only in parse_pointer_encoding
    if !encoding.is_valid_encoding() {
        return Err(Error::UnknownPointerEncoding);
    }

    if encoding == constants::DW_EH_PE_omit {
        return Err(Error::CannotParseOmitPointerEncoding);
    }

    let base = match encoding.application() {
        constants::DW_EH_PE_absptr => 0,
        constants::DW_EH_PE_pcrel => {
            if let Some(section_base) = parameters.bases.section {
                let offset_from_section = input.offset_from(parameters.section);
                section_base.wrapping_add(offset_from_section.into_u64())
            } else {
                return Err(Error::PcRelativePointerButSectionBaseIsUndefined);
            }
        }
        constants::DW_EH_PE_textrel => {
            if let Some(text) = parameters.bases.text {
                text
            } else {
                return Err(Error::TextRelativePointerButTextBaseIsUndefined);
            }
        }
        constants::DW_EH_PE_datarel => {
            if let Some(data) = parameters.bases.data {
                data
            } else {
                return Err(Error::DataRelativePointerButDataBaseIsUndefined);
            }
        }
        constants::DW_EH_PE_funcrel => {
            if let Some(func) = parameters.func_base {
                func
            } else {
                return Err(Error::FuncRelativePointerInBadContext);
            }
        }
        constants::DW_EH_PE_aligned => return Err(Error::UnsupportedPointerEncoding),
        _ => unreachable!(),
    };

    let offset = match encoding.format() {
        // Unsigned variants.
        constants::DW_EH_PE_absptr => input.read_address(parameters.address_size),
        constants::DW_EH_PE_uleb128 => input.read_uleb128(),
        constants::DW_EH_PE_udata2 => input.read_u16().map(u64::from),
        constants::DW_EH_PE_udata4 => input.read_u32().map(u64::from),
        constants::DW_EH_PE_udata8 => input.read_u64(),

        // Signed variants. Here we sign extend the values (happens by
        // default when casting a signed integer to a larger range integer
        // in Rust), return them as u64, and rely on wrapping addition to do
        // the right thing when adding these offsets to their bases.
        constants::DW_EH_PE_sleb128 => input.read_sleb128().map(|a| a as u64),
        constants::DW_EH_PE_sdata2 => input.read_i16().map(|a| a as u64),
        constants::DW_EH_PE_sdata4 => input.read_i32().map(|a| a as u64),
        constants::DW_EH_PE_sdata8 => input.read_i64().map(|a| a as u64),

        // That was all of the valid encoding formats.
        _ => unreachable!(),
    }?;

    Ok(Pointer::new(encoding, base.wrapping_add(offset)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{parse_cfi_entry, AugmentationData, RegisterRuleMap, UnwindContext};
    use crate::common::Format;
    use crate::constants;
    use crate::endianity::{BigEndian, Endianity, LittleEndian, NativeEndian};
    use crate::read::{
        EndianSlice, Error, Expression, Pointer, ReaderOffsetId, Result, Section as ReadSection,
    };
    use crate::test_util::GimliSectionMethods;
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use core::mem;
    use core::u64;
    use test_assembler::{Endian, Label, LabelMaker, LabelOrNum, Section, ToLabelOrNum};

    // Ensure each test tries to read the same section kind that it wrote.
    #[derive(Clone, Copy)]
    struct SectionKind<Section>(PhantomData<Section>);

    impl<T> SectionKind<T> {
        fn endian<'input, E>(self) -> Endian
        where
            E: Endianity,
            T: UnwindSection<EndianSlice<'input, E>>,
            T::Offset: UnwindOffset<usize>,
        {
            if E::default().is_big_endian() {
                Endian::Big
            } else {
                Endian::Little
            }
        }

        fn section<'input, E>(self, contents: &'input [u8]) -> T
        where
            E: Endianity,
            T: UnwindSection<EndianSlice<'input, E>> + ReadSection<EndianSlice<'input, E>>,
            T::Offset: UnwindOffset<usize>,
        {
            EndianSlice::new(contents, E::default()).into()
        }
    }

    fn debug_frame_le<'a>() -> SectionKind<DebugFrame<EndianSlice<'a, LittleEndian>>> {
        SectionKind(PhantomData)
    }

    fn debug_frame_be<'a>() -> SectionKind<DebugFrame<EndianSlice<'a, BigEndian>>> {
        SectionKind(PhantomData)
    }

    fn eh_frame_le<'a>() -> SectionKind<EhFrame<EndianSlice<'a, LittleEndian>>> {
        SectionKind(PhantomData)
    }

    fn parse_fde<Section, O, F, R>(
        section: Section,
        input: &mut R,
        get_cie: F,
    ) -> Result<FrameDescriptionEntry<R>>
    where
        R: Reader,
        Section: UnwindSection<R, Offset = O>,
        O: UnwindOffset<R::Offset>,
        F: FnMut(&Section, &BaseAddresses, O) -> Result<CommonInformationEntry<R>>,
    {
        let bases = Default::default();
        match parse_cfi_entry(&bases, &section, input) {
            Ok(Some(CieOrFde::Fde(partial))) => partial.parse(get_cie),
            Ok(_) => Err(Error::NoEntryAtGivenOffset),
            Err(e) => Err(e),
        }
    }

    // Mixin methods for `Section` to help define binary test data.

    trait CfiSectionMethods: GimliSectionMethods {
        fn cie<'aug, 'input, E, T>(
            self,
            _kind: SectionKind<T>,
            augmentation: Option<&'aug str>,
            cie: &mut CommonInformationEntry<EndianSlice<'input, E>>,
        ) -> Self
        where
            E: Endianity,
            T: UnwindSection<EndianSlice<'input, E>>,
            T::Offset: UnwindOffset;
        fn fde<'a, 'input, E, T, L>(
            self,
            _kind: SectionKind<T>,
            cie_offset: L,
            fde: &mut FrameDescriptionEntry<EndianSlice<'input, E>>,
        ) -> Self
        where
            E: Endianity,
            T: UnwindSection<EndianSlice<'input, E>>,
            T::Offset: UnwindOffset,
            L: ToLabelOrNum<'a, u64>;
    }

    impl CfiSectionMethods for Section {
        fn cie<'aug, 'input, E, T>(
            self,
            _kind: SectionKind<T>,
            augmentation: Option<&'aug str>,
            cie: &mut CommonInformationEntry<EndianSlice<'input, E>>,
        ) -> Self
        where
            E: Endianity,
            T: UnwindSection<EndianSlice<'input, E>>,
            T::Offset: UnwindOffset,
        {
            cie.offset = self.size() as _;
            let length = Label::new();
            let start = Label::new();
            let end = Label::new();

            let section = match cie.format {
                Format::Dwarf32 => self.D32(&length).mark(&start).D32(0xffff_ffff),
                Format::Dwarf64 => {
                    let section = self.D32(0xffff_ffff);
                    section.D64(&length).mark(&start).D64(0xffff_ffff_ffff_ffff)
                }
            };

            let mut section = section.D8(cie.version);

            if let Some(augmentation) = augmentation {
                section = section.append_bytes(augmentation.as_bytes());
            }

            // Null terminator for augmentation string.
            let section = section.D8(0);

            let section = if T::has_address_and_segment_sizes(cie.version) {
                section.D8(cie.address_size).D8(cie.segment_size)
            } else {
                section
            };

            let section = section
                .uleb(cie.code_alignment_factor)
                .sleb(cie.data_alignment_factor)
                .uleb(cie.return_address_register.0.into())
                .append_bytes(cie.initial_instructions.into())
                .mark(&end);

            cie.length = (&end - &start) as usize;
            length.set_const(cie.length as u64);

            section
        }

        fn fde<'a, 'input, E, T, L>(
            self,
            _kind: SectionKind<T>,
            cie_offset: L,
            fde: &mut FrameDescriptionEntry<EndianSlice<'input, E>>,
        ) -> Self
        where
            E: Endianity,
            T: UnwindSection<EndianSlice<'input, E>>,
            T::Offset: UnwindOffset,
            L: ToLabelOrNum<'a, u64>,
        {
            fde.offset = self.size() as _;
            let length = Label::new();
            let start = Label::new();
            let end = Label::new();

            assert_eq!(fde.format, fde.cie.format);

            let section = match T::cie_offset_encoding(fde.format) {
                CieOffsetEncoding::U32 => {
                    let section = self.D32(&length).mark(&start);
                    match cie_offset.to_labelornum() {
                        LabelOrNum::Label(ref l) => section.D32(l),
                        LabelOrNum::Num(o) => section.D32(o as u32),
                    }
                }
                CieOffsetEncoding::U64 => {
                    let section = self.D32(0xffff_ffff);
                    section.D64(&length).mark(&start).D64(cie_offset)
                }
            };

            let section = match fde.cie.segment_size {
                0 => section,
                4 => section.D32(fde.initial_segment as u32),
                8 => section.D64(fde.initial_segment),
                x => panic!("Unsupported test segment size: {}", x),
            };

            let section = match fde.cie.address_size {
                4 => section
                    .D32(fde.initial_address() as u32)
                    .D32(fde.len() as u32),
                8 => section.D64(fde.initial_address()).D64(fde.len()),
                x => panic!("Unsupported address size: {}", x),
            };

            let section = if let Some(ref augmentation) = fde.augmentation {
                let cie_aug = fde
                    .cie
                    .augmentation
                    .expect("FDE has augmentation, but CIE doesn't");

                if let Some(lsda) = augmentation.lsda {
                    // We only support writing `DW_EH_PE_absptr` here.
                    assert_eq!(
                        cie_aug
                            .lsda
                            .expect("FDE has lsda, but CIE doesn't")
                            .format(),
                        constants::DW_EH_PE_absptr
                    );

                    // Augmentation data length
                    let section = section.uleb(u64::from(fde.cie.address_size));
                    match fde.cie.address_size {
                        4 => section.D32({
                            let x: u64 = lsda.into();
                            x as u32
                        }),
                        8 => section.D64({
                            let x: u64 = lsda.into();
                            x
                        }),
                        x => panic!("Unsupported address size: {}", x),
                    }
                } else {
                    // Even if we don't have any augmentation data, if there is
                    // an augmentation defined, we need to put the length in.
                    section.uleb(0)
                }
            } else {
                section
            };

            let section = section.append_bytes(fde.instructions.into()).mark(&end);

            fde.length = (&end - &start) as usize;
            length.set_const(fde.length as u64);

            section
        }
    }

    trait ResultExt {
        fn map_eof(self, input: &[u8]) -> Self;
    }

    impl<T> ResultExt for Result<T> {
        fn map_eof(self, input: &[u8]) -> Self {
            match self {
                Err(Error::UnexpectedEof(id)) => {
                    let id = ReaderOffsetId(id.0 - input.as_ptr() as u64);
                    Err(Error::UnexpectedEof(id))
                }
                r => r,
            }
        }
    }

    #[allow(clippy::type_complexity)]
    #[allow(clippy::needless_pass_by_value)]
    fn assert_parse_cie<'input, E>(
        kind: SectionKind<DebugFrame<EndianSlice<'input, E>>>,
        section: Section,
        address_size: u8,
        expected: Result<(
            EndianSlice<'input, E>,
            CommonInformationEntry<EndianSlice<'input, E>>,
        )>,
    ) where
        E: Endianity,
    {
        let section = section.get_contents().unwrap();
        let mut debug_frame = kind.section(&section);
        debug_frame.set_address_size(address_size);
        let input = &mut EndianSlice::new(&section, E::default());
        let bases = Default::default();
        let result = CommonInformationEntry::parse(&bases, &debug_frame, input);
        let result = result.map(|cie| (*input, cie)).map_eof(&section);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_cie_incomplete_length_32() {
        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian()).L16(5);
        assert_parse_cie(
            kind,
            section,
            8,
            Err(Error::UnexpectedEof(ReaderOffsetId(0))),
        );
    }

    #[test]
    fn test_parse_cie_incomplete_length_64() {
        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .L32(0xffff_ffff)
            .L32(12345);
        assert_parse_cie(
            kind,
            section,
            8,
            Err(Error::UnexpectedEof(ReaderOffsetId(4))),
        );
    }

    #[test]
    fn test_parse_cie_incomplete_id_32() {
        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            // The length is not large enough to contain the ID.
            .B32(3)
            .B32(0xffff_ffff);
        assert_parse_cie(
            kind,
            section,
            8,
            Err(Error::UnexpectedEof(ReaderOffsetId(4))),
        );
    }

    #[test]
    fn test_parse_cie_bad_id_32() {
        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            // Initial length
            .B32(4)
            // Not the CIE Id.
            .B32(0xbad1_bad2);
        assert_parse_cie(kind, section, 8, Err(Error::NotCieId));
    }

    #[test]
    fn test_parse_cie_32_bad_version() {
        let mut cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 99,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 2,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&[], LittleEndian),
        };

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian()).cie(kind, None, &mut cie);
        assert_parse_cie(kind, section, 4, Err(Error::UnknownVersion(99)));
    }

    #[test]
    fn test_parse_cie_unknown_augmentation() {
        let length = Label::new();
        let start = Label::new();
        let end = Label::new();

        let augmentation = Some("replicant");
        let expected_rest = [1, 2, 3];

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            // Initial length
            .L32(&length)
            .mark(&start)
            // CIE Id
            .L32(0xffff_ffff)
            // Version
            .D8(4)
            // Augmentation
            .append_bytes(augmentation.unwrap().as_bytes())
            // Null terminator
            .D8(0)
            // Extra augmented data that we can't understand.
            .L32(1)
            .L32(2)
            .L32(3)
            .L32(4)
            .L32(5)
            .L32(6)
            .mark(&end)
            .append_bytes(&expected_rest);

        let expected_length = (&end - &start) as u64;
        length.set_const(expected_length);

        assert_parse_cie(kind, section, 8, Err(Error::UnknownAugmentation));
    }

    fn test_parse_cie(format: Format, version: u8, address_size: u8) {
        let expected_rest = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected_instrs: Vec<_> = (0..4).map(|_| constants::DW_CFA_nop.0).collect();

        let mut cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format,
            version,
            augmentation: None,
            address_size,
            segment_size: 0,
            code_alignment_factor: 16,
            data_alignment_factor: 32,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&expected_instrs, LittleEndian),
        };

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .cie(kind, None, &mut cie)
            .append_bytes(&expected_rest);

        assert_parse_cie(
            kind,
            section,
            address_size,
            Ok((EndianSlice::new(&expected_rest, LittleEndian), cie)),
        );
    }

    #[test]
    fn test_parse_cie_32_ok() {
        test_parse_cie(Format::Dwarf32, 1, 4);
        test_parse_cie(Format::Dwarf32, 1, 8);
        test_parse_cie(Format::Dwarf32, 4, 4);
        test_parse_cie(Format::Dwarf32, 4, 8);
    }

    #[test]
    fn test_parse_cie_64_ok() {
        test_parse_cie(Format::Dwarf64, 1, 4);
        test_parse_cie(Format::Dwarf64, 1, 8);
        test_parse_cie(Format::Dwarf64, 4, 4);
        test_parse_cie(Format::Dwarf64, 4, 8);
    }

    #[test]
    fn test_parse_cie_length_too_big() {
        let expected_instrs: Vec<_> = (0..13).map(|_| constants::DW_CFA_nop.0).collect();

        let mut cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 0,
            data_alignment_factor: 0,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&expected_instrs, LittleEndian),
        };

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian()).cie(kind, None, &mut cie);

        let mut contents = section.get_contents().unwrap();

        // Overwrite the length to be too big.
        contents[0] = 0;
        contents[1] = 0;
        contents[2] = 0;
        contents[3] = 255;

        let debug_frame = DebugFrame::new(&contents, LittleEndian);
        let bases = Default::default();
        assert_eq!(
            CommonInformationEntry::parse(
                &bases,
                &debug_frame,
                &mut EndianSlice::new(&contents, LittleEndian)
            )
            .map_eof(&contents),
            Err(Error::UnexpectedEof(ReaderOffsetId(4)))
        );
    }

    #[test]
    fn test_parse_fde_incomplete_length_32() {
        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian()).L16(5);
        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, LittleEndian);
        assert_eq!(
            parse_fde(debug_frame, rest, UnwindSection::cie_from_offset).map_eof(&section),
            Err(Error::UnexpectedEof(ReaderOffsetId(0)))
        );
    }

    #[test]
    fn test_parse_fde_incomplete_length_64() {
        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .L32(0xffff_ffff)
            .L32(12345);
        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, LittleEndian);
        assert_eq!(
            parse_fde(debug_frame, rest, UnwindSection::cie_from_offset).map_eof(&section),
            Err(Error::UnexpectedEof(ReaderOffsetId(4)))
        );
    }

    #[test]
    fn test_parse_fde_incomplete_cie_pointer_32() {
        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            // The length is not large enough to contain the CIE pointer.
            .B32(3)
            .B32(1994);
        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, BigEndian);
        assert_eq!(
            parse_fde(debug_frame, rest, UnwindSection::cie_from_offset).map_eof(&section),
            Err(Error::UnexpectedEof(ReaderOffsetId(4)))
        );
    }

    #[test]
    fn test_parse_fde_32_ok() {
        let expected_rest = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cie_offset = 0xbad0_bad1;
        let expected_instrs: Vec<_> = (0..7).map(|_| constants::DW_CFA_nop.0).collect();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 100,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            // DWARF32 with a 64 bit address size! Holy moly!
            address_size: 8,
            segment_size: 0,
            code_alignment_factor: 3,
            data_alignment_factor: 2,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&[], LittleEndian),
        };

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 39,
            augmentation: None,
            instructions: EndianSlice::new(&expected_instrs, LittleEndian),
        };

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&expected_rest);

        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, LittleEndian);

        let get_cie = |_: &_, _: &_, offset| {
            assert_eq!(offset, DebugFrameOffset(cie_offset as usize));
            Ok(cie.clone())
        };

        assert_eq!(parse_fde(debug_frame, rest, get_cie), Ok(fde));
        assert_eq!(*rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_fde_32_with_segment_ok() {
        let expected_rest = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cie_offset = 0xbad0_bad1;
        let expected_instrs: Vec<_> = (0..92).map(|_| constants::DW_CFA_nop.0).collect();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 100,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 4,
            code_alignment_factor: 3,
            data_alignment_factor: 2,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&[], LittleEndian),
        };

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0xbadb_ad11,
            initial_address: 0xfeed_beef,
            address_range: 999,
            augmentation: None,
            instructions: EndianSlice::new(&expected_instrs, LittleEndian),
        };

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&expected_rest);

        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, LittleEndian);

        let get_cie = |_: &_, _: &_, offset| {
            assert_eq!(offset, DebugFrameOffset(cie_offset as usize));
            Ok(cie.clone())
        };

        assert_eq!(parse_fde(debug_frame, rest, get_cie), Ok(fde));
        assert_eq!(*rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_fde_64_ok() {
        let expected_rest = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cie_offset = 0xbad0_bad1;
        let expected_instrs: Vec<_> = (0..7).map(|_| constants::DW_CFA_nop.0).collect();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 100,
            format: Format::Dwarf64,
            version: 4,
            augmentation: None,
            address_size: 8,
            segment_size: 0,
            code_alignment_factor: 3,
            data_alignment_factor: 2,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&[], LittleEndian),
        };

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf64,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 999,
            augmentation: None,
            instructions: EndianSlice::new(&expected_instrs, LittleEndian),
        };

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&expected_rest);

        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, LittleEndian);

        let get_cie = |_: &_, _: &_, offset| {
            assert_eq!(offset, DebugFrameOffset(cie_offset as usize));
            Ok(cie.clone())
        };

        assert_eq!(parse_fde(debug_frame, rest, get_cie), Ok(fde));
        assert_eq!(*rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_entry_on_cie_32_ok() {
        let expected_rest = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected_instrs: Vec<_> = (0..4).map(|_| constants::DW_CFA_nop.0).collect();

        let mut cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 16,
            data_alignment_factor: 32,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&expected_instrs, BigEndian),
        };

        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            .cie(kind, None, &mut cie)
            .append_bytes(&expected_rest);
        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, BigEndian);

        let bases = Default::default();
        assert_eq!(
            parse_cfi_entry(&bases, &debug_frame, rest),
            Ok(Some(CieOrFde::Cie(cie)))
        );
        assert_eq!(*rest, EndianSlice::new(&expected_rest, BigEndian));
    }

    #[test]
    fn test_parse_cfi_entry_on_fde_32_ok() {
        let cie_offset = 0x1234_5678;
        let expected_rest = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected_instrs: Vec<_> = (0..4).map(|_| constants::DW_CFA_nop.0).collect();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 16,
            data_alignment_factor: 32,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&[], BigEndian),
        };

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 39,
            augmentation: None,
            instructions: EndianSlice::new(&expected_instrs, BigEndian),
        };

        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&expected_rest);

        let section = section.get_contents().unwrap();
        let debug_frame = kind.section(&section);
        let rest = &mut EndianSlice::new(&section, BigEndian);

        let bases = Default::default();
        match parse_cfi_entry(&bases, &debug_frame, rest) {
            Ok(Some(CieOrFde::Fde(partial))) => {
                assert_eq!(*rest, EndianSlice::new(&expected_rest, BigEndian));

                assert_eq!(partial.length, fde.length);
                assert_eq!(partial.format, fde.format);
                assert_eq!(partial.cie_offset, DebugFrameOffset(cie_offset as usize));

                let get_cie = |_: &_, _: &_, offset| {
                    assert_eq!(offset, DebugFrameOffset(cie_offset as usize));
                    Ok(cie.clone())
                };

                assert_eq!(partial.parse(get_cie), Ok(fde));
            }
            otherwise => panic!("Unexpected result: {:#?}", otherwise),
        }
    }

    #[test]
    fn test_cfi_entries_iter() {
        let expected_instrs1: Vec<_> = (0..4).map(|_| constants::DW_CFA_nop.0).collect();

        let expected_instrs2: Vec<_> = (0..8).map(|_| constants::DW_CFA_nop.0).collect();

        let expected_instrs3: Vec<_> = (0..12).map(|_| constants::DW_CFA_nop.0).collect();

        let expected_instrs4: Vec<_> = (0..16).map(|_| constants::DW_CFA_nop.0).collect();

        let mut cie1 = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 2,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&expected_instrs1, BigEndian),
        };

        let mut cie2 = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 3,
            data_alignment_factor: 2,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&expected_instrs2, BigEndian),
        };

        let cie1_location = Label::new();
        let cie2_location = Label::new();

        // Write the CIEs first so that their length gets set before we clone
        // them into the FDEs and our equality assertions down the line end up
        // with all the CIEs always having he correct length.
        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            .mark(&cie1_location)
            .cie(kind, None, &mut cie1)
            .mark(&cie2_location)
            .cie(kind, None, &mut cie2);

        let mut fde1 = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie1.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 39,
            augmentation: None,
            instructions: EndianSlice::new(&expected_instrs3, BigEndian),
        };

        let mut fde2 = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie2.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_face,
            address_range: 9000,
            augmentation: None,
            instructions: EndianSlice::new(&expected_instrs4, BigEndian),
        };

        let section =
            section
                .fde(kind, &cie1_location, &mut fde1)
                .fde(kind, &cie2_location, &mut fde2);

        section.start().set_const(0);

        let cie1_offset = cie1_location.value().unwrap() as usize;
        let cie2_offset = cie2_location.value().unwrap() as usize;

        let contents = section.get_contents().unwrap();
        let debug_frame = kind.section(&contents);

        let bases = Default::default();
        let mut entries = debug_frame.entries(&bases);

        assert_eq!(entries.next(), Ok(Some(CieOrFde::Cie(cie1.clone()))));
        assert_eq!(entries.next(), Ok(Some(CieOrFde::Cie(cie2.clone()))));

        match entries.next() {
            Ok(Some(CieOrFde::Fde(partial))) => {
                assert_eq!(partial.length, fde1.length);
                assert_eq!(partial.format, fde1.format);
                assert_eq!(partial.cie_offset, DebugFrameOffset(cie1_offset));

                let get_cie = |_: &_, _: &_, offset| {
                    assert_eq!(offset, DebugFrameOffset(cie1_offset));
                    Ok(cie1.clone())
                };
                assert_eq!(partial.parse(get_cie), Ok(fde1));
            }
            otherwise => panic!("Unexpected result: {:#?}", otherwise),
        }

        match entries.next() {
            Ok(Some(CieOrFde::Fde(partial))) => {
                assert_eq!(partial.length, fde2.length);
                assert_eq!(partial.format, fde2.format);
                assert_eq!(partial.cie_offset, DebugFrameOffset(cie2_offset));

                let get_cie = |_: &_, _: &_, offset| {
                    assert_eq!(offset, DebugFrameOffset(cie2_offset));
                    Ok(cie2.clone())
                };
                assert_eq!(partial.parse(get_cie), Ok(fde2));
            }
            otherwise => panic!("Unexpected result: {:#?}", otherwise),
        }

        assert_eq!(entries.next(), Ok(None));
    }

    #[test]
    fn test_parse_cie_from_offset() {
        let filler = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let instrs: Vec<_> = (0..5).map(|_| constants::DW_CFA_nop.0).collect();

        let mut cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf64,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 4,
            data_alignment_factor: 8,
            return_address_register: Register(12),
            initial_instructions: EndianSlice::new(&instrs, LittleEndian),
        };

        let cie_location = Label::new();

        let kind = debug_frame_le();
        let section = Section::with_endian(kind.endian())
            .append_bytes(&filler)
            .mark(&cie_location)
            .cie(kind, None, &mut cie)
            .append_bytes(&filler);

        section.start().set_const(0);

        let cie_offset = DebugFrameOffset(cie_location.value().unwrap() as usize);

        let contents = section.get_contents().unwrap();
        let debug_frame = kind.section(&contents);
        let bases = Default::default();

        assert_eq!(debug_frame.cie_from_offset(&bases, cie_offset), Ok(cie));
    }

    fn parse_cfi_instruction<R: Reader + Default>(
        input: &mut R,
        address_size: u8,
    ) -> Result<CallFrameInstruction<R>> {
        let parameters = &PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size,
            section: &R::default(),
        };
        CallFrameInstruction::parse(input, None, parameters)
    }

    #[test]
    fn test_parse_cfi_instruction_advance_loc() {
        let expected_rest = [1, 2, 3, 4];
        let expected_delta = 42;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_advance_loc.0 | expected_delta)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::AdvanceLoc {
                delta: u32::from(expected_delta),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_offset() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 3;
        let expected_offset = 1997;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_offset.0 | expected_reg)
            .uleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Offset {
                register: Register(expected_reg.into()),
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_restore() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 3;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_restore.0 | expected_reg)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Restore {
                register: Register(expected_reg.into()),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_nop() {
        let expected_rest = [1, 2, 3, 4];
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_nop.0)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Nop)
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_set_loc() {
        let expected_rest = [1, 2, 3, 4];
        let expected_addr = 0xdead_beef;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_set_loc.0)
            .L64(expected_addr)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::SetLoc {
                address: expected_addr,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_set_loc_encoding() {
        let text_base = 0xfeed_face;
        let addr_offset = 0xbeef;
        let expected_addr = text_base + addr_offset;
        let expected_rest = [1, 2, 3, 4];
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_set_loc.0)
            .L64(addr_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        let parameters = &PointerEncodingParameters {
            bases: &BaseAddresses::default().set_text(text_base).eh_frame,
            func_base: None,
            address_size: 8,
            section: &EndianSlice::new(&[], LittleEndian),
        };
        assert_eq!(
            CallFrameInstruction::parse(input, Some(constants::DW_EH_PE_textrel), parameters),
            Ok(CallFrameInstruction::SetLoc {
                address: expected_addr,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_advance_loc1() {
        let expected_rest = [1, 2, 3, 4];
        let expected_delta = 8;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_advance_loc1.0)
            .D8(expected_delta)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::AdvanceLoc {
                delta: u32::from(expected_delta),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_advance_loc2() {
        let expected_rest = [1, 2, 3, 4];
        let expected_delta = 500;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_advance_loc2.0)
            .L16(expected_delta)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::AdvanceLoc {
                delta: u32::from(expected_delta),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_advance_loc4() {
        let expected_rest = [1, 2, 3, 4];
        let expected_delta = 1 << 20;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_advance_loc4.0)
            .L32(expected_delta)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::AdvanceLoc {
                delta: expected_delta,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_offset_extended() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 7;
        let expected_offset = 33;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_offset_extended.0)
            .uleb(expected_reg.into())
            .uleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Offset {
                register: Register(expected_reg),
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_restore_extended() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 7;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_restore_extended.0)
            .uleb(expected_reg.into())
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Restore {
                register: Register(expected_reg),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_undefined() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 7;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_undefined.0)
            .uleb(expected_reg.into())
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Undefined {
                register: Register(expected_reg),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_same_value() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 7;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_same_value.0)
            .uleb(expected_reg.into())
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::SameValue {
                register: Register(expected_reg),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_register() {
        let expected_rest = [1, 2, 3, 4];
        let expected_dest_reg = 7;
        let expected_src_reg = 8;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_register.0)
            .uleb(expected_dest_reg.into())
            .uleb(expected_src_reg.into())
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Register {
                dest_register: Register(expected_dest_reg),
                src_register: Register(expected_src_reg),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_remember_state() {
        let expected_rest = [1, 2, 3, 4];
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_remember_state.0)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::RememberState)
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_restore_state() {
        let expected_rest = [1, 2, 3, 4];
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_restore_state.0)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::RestoreState)
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_def_cfa() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 2;
        let expected_offset = 0;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_def_cfa.0)
            .uleb(expected_reg.into())
            .uleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::DefCfa {
                register: Register(expected_reg),
                offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_def_cfa_register() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 2;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_def_cfa_register.0)
            .uleb(expected_reg.into())
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::DefCfaRegister {
                register: Register(expected_reg),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_def_cfa_offset() {
        let expected_rest = [1, 2, 3, 4];
        let expected_offset = 23;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_def_cfa_offset.0)
            .uleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::DefCfaOffset {
                offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_def_cfa_expression() {
        let expected_rest = [1, 2, 3, 4];
        let expected_expr = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

        let length = Label::new();
        let start = Label::new();
        let end = Label::new();

        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_def_cfa_expression.0)
            .D8(&length)
            .mark(&start)
            .append_bytes(&expected_expr)
            .mark(&end)
            .append_bytes(&expected_rest);

        length.set_const((&end - &start) as u64);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);

        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::DefCfaExpression {
                expression: Expression(EndianSlice::new(&expected_expr, LittleEndian)),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_expression() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 99;
        let expected_expr = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

        let length = Label::new();
        let start = Label::new();
        let end = Label::new();

        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_expression.0)
            .uleb(expected_reg.into())
            .D8(&length)
            .mark(&start)
            .append_bytes(&expected_expr)
            .mark(&end)
            .append_bytes(&expected_rest);

        length.set_const((&end - &start) as u64);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);

        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::Expression {
                register: Register(expected_reg),
                expression: Expression(EndianSlice::new(&expected_expr, LittleEndian)),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_offset_extended_sf() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 7;
        let expected_offset = -33;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_offset_extended_sf.0)
            .uleb(expected_reg.into())
            .sleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::OffsetExtendedSf {
                register: Register(expected_reg),
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_def_cfa_sf() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 2;
        let expected_offset = -9999;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_def_cfa_sf.0)
            .uleb(expected_reg.into())
            .sleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::DefCfaSf {
                register: Register(expected_reg),
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_def_cfa_offset_sf() {
        let expected_rest = [1, 2, 3, 4];
        let expected_offset = -123;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_def_cfa_offset_sf.0)
            .sleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::DefCfaOffsetSf {
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_val_offset() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 50;
        let expected_offset = 23;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_val_offset.0)
            .uleb(expected_reg.into())
            .uleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::ValOffset {
                register: Register(expected_reg),
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_val_offset_sf() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 50;
        let expected_offset = -23;
        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_val_offset_sf.0)
            .uleb(expected_reg.into())
            .sleb(expected_offset)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::ValOffsetSf {
                register: Register(expected_reg),
                factored_offset: expected_offset,
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_val_expression() {
        let expected_rest = [1, 2, 3, 4];
        let expected_reg = 50;
        let expected_expr = [2, 2, 1, 1, 5, 5];

        let length = Label::new();
        let start = Label::new();
        let end = Label::new();

        let section = Section::with_endian(Endian::Little)
            .D8(constants::DW_CFA_val_expression.0)
            .uleb(expected_reg.into())
            .D8(&length)
            .mark(&start)
            .append_bytes(&expected_expr)
            .mark(&end)
            .append_bytes(&expected_rest);

        length.set_const((&end - &start) as u64);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);

        assert_eq!(
            parse_cfi_instruction(input, 8),
            Ok(CallFrameInstruction::ValExpression {
                register: Register(expected_reg),
                expression: Expression(EndianSlice::new(&expected_expr, LittleEndian)),
            })
        );
        assert_eq!(*input, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_cfi_instruction_unknown_instruction() {
        let expected_rest = [1, 2, 3, 4];
        let unknown_instr = constants::DwCfa(0b0011_1111);
        let section = Section::with_endian(Endian::Little)
            .D8(unknown_instr.0)
            .append_bytes(&expected_rest);
        let contents = section.get_contents().unwrap();
        let input = &mut EndianSlice::new(&contents, LittleEndian);
        assert_eq!(
            parse_cfi_instruction(input, 8),
            Err(Error::UnknownCallFrameInstruction(unknown_instr))
        );
    }

    #[test]
    fn test_call_frame_instruction_iter_ok() {
        let expected_reg = 50;
        let expected_expr = [2, 2, 1, 1, 5, 5];
        let expected_delta = 230;

        let length = Label::new();
        let start = Label::new();
        let end = Label::new();

        let section = Section::with_endian(Endian::Big)
            .D8(constants::DW_CFA_val_expression.0)
            .uleb(expected_reg.into())
            .D8(&length)
            .mark(&start)
            .append_bytes(&expected_expr)
            .mark(&end)
            .D8(constants::DW_CFA_advance_loc1.0)
            .D8(expected_delta);

        length.set_const((&end - &start) as u64);
        let contents = section.get_contents().unwrap();
        let input = EndianSlice::new(&contents, BigEndian);
        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 8,
            section: &EndianSlice::default(),
        };
        let mut iter = CallFrameInstructionIter {
            input,
            address_encoding: None,
            parameters,
        };

        assert_eq!(
            iter.next(),
            Ok(Some(CallFrameInstruction::ValExpression {
                register: Register(expected_reg),
                expression: Expression(EndianSlice::new(&expected_expr, BigEndian)),
            }))
        );

        assert_eq!(
            iter.next(),
            Ok(Some(CallFrameInstruction::AdvanceLoc {
                delta: u32::from(expected_delta),
            }))
        );

        assert_eq!(iter.next(), Ok(None));
    }

    #[test]
    fn test_call_frame_instruction_iter_err() {
        // DW_CFA_advance_loc1 without an operand.
        let section = Section::with_endian(Endian::Big).D8(constants::DW_CFA_advance_loc1.0);

        let contents = section.get_contents().unwrap();
        let input = EndianSlice::new(&contents, BigEndian);
        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 8,
            section: &EndianSlice::default(),
        };
        let mut iter = CallFrameInstructionIter {
            input,
            address_encoding: None,
            parameters,
        };

        assert_eq!(
            iter.next().map_eof(&contents),
            Err(Error::UnexpectedEof(ReaderOffsetId(1)))
        );
        assert_eq!(iter.next(), Ok(None));
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_eval<'a, I>(
        mut initial_ctx: UnwindContext<EndianSlice<'a, LittleEndian>>,
        expected_ctx: UnwindContext<EndianSlice<'a, LittleEndian>>,
        cie: CommonInformationEntry<EndianSlice<'a, LittleEndian>>,
        fde: Option<FrameDescriptionEntry<EndianSlice<'a, LittleEndian>>>,
        instructions: I,
    ) where
        I: AsRef<
            [(
                Result<bool>,
                CallFrameInstruction<EndianSlice<'a, LittleEndian>>,
            )],
        >,
    {
        {
            let section = &DebugFrame::from(EndianSlice::default());
            let bases = &BaseAddresses::default();
            let mut table = match fde {
                Some(fde) => UnwindTable::new_for_fde(section, bases, &mut initial_ctx, &fde),
                None => UnwindTable::new_for_cie(section, bases, &mut initial_ctx, &cie),
            };
            for &(ref expected_result, ref instruction) in instructions.as_ref() {
                assert_eq!(*expected_result, table.evaluate(instruction.clone()));
            }
        }

        assert_eq!(expected_ctx, initial_ctx);
    }

    fn make_test_cie<'a>() -> CommonInformationEntry<EndianSlice<'a, LittleEndian>> {
        CommonInformationEntry {
            offset: 0,
            format: Format::Dwarf64,
            length: 0,
            return_address_register: Register(0),
            version: 4,
            address_size: mem::size_of::<usize>() as u8,
            initial_instructions: EndianSlice::new(&[], LittleEndian),
            augmentation: None,
            segment_size: 0,
            data_alignment_factor: 2,
            code_alignment_factor: 3,
        }
    }

    #[test]
    fn test_eval_set_loc() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected.row_mut().end_address = 42;
        let instructions = [(Ok(true), CallFrameInstruction::SetLoc { address: 42 })];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_set_loc_backwards() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.row_mut().start_address = 999;
        let expected = ctx.clone();
        let instructions = [(
            Err(Error::InvalidAddressRange),
            CallFrameInstruction::SetLoc { address: 42 },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_advance_loc() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.row_mut().start_address = 3;
        let mut expected = ctx.clone();
        expected.row_mut().end_address = 3 + 2 * cie.code_alignment_factor;
        let instructions = [(Ok(true), CallFrameInstruction::AdvanceLoc { delta: 2 })];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_advance_loc_overflow() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.row_mut().start_address = u64::MAX;
        let mut expected = ctx.clone();
        expected.row_mut().end_address = 42 * cie.code_alignment_factor - 1;
        let instructions = [(Ok(true), CallFrameInstruction::AdvanceLoc { delta: 42 })];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected.set_cfa(CfaRule::RegisterAndOffset {
            register: Register(42),
            offset: 36,
        });
        let instructions = [(
            Ok(false),
            CallFrameInstruction::DefCfa {
                register: Register(42),
                offset: 36,
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa_sf() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected.set_cfa(CfaRule::RegisterAndOffset {
            register: Register(42),
            offset: 36 * cie.data_alignment_factor as i64,
        });
        let instructions = [(
            Ok(false),
            CallFrameInstruction::DefCfaSf {
                register: Register(42),
                factored_offset: 36,
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa_register() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.set_cfa(CfaRule::RegisterAndOffset {
            register: Register(3),
            offset: 8,
        });
        let mut expected = ctx.clone();
        expected.set_cfa(CfaRule::RegisterAndOffset {
            register: Register(42),
            offset: 8,
        });
        let instructions = [(
            Ok(false),
            CallFrameInstruction::DefCfaRegister {
                register: Register(42),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa_register_invalid_context() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.set_cfa(CfaRule::Expression(Expression(EndianSlice::new(
            &[],
            LittleEndian,
        ))));
        let expected = ctx.clone();
        let instructions = [(
            Err(Error::CfiInstructionInInvalidContext),
            CallFrameInstruction::DefCfaRegister {
                register: Register(42),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa_offset() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.set_cfa(CfaRule::RegisterAndOffset {
            register: Register(3),
            offset: 8,
        });
        let mut expected = ctx.clone();
        expected.set_cfa(CfaRule::RegisterAndOffset {
            register: Register(3),
            offset: 42,
        });
        let instructions = [(Ok(false), CallFrameInstruction::DefCfaOffset { offset: 42 })];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa_offset_invalid_context() {
        let cie = make_test_cie();
        let mut ctx = UnwindContext::new();
        ctx.set_cfa(CfaRule::Expression(Expression(EndianSlice::new(
            &[],
            LittleEndian,
        ))));
        let expected = ctx.clone();
        let instructions = [(
            Err(Error::CfiInstructionInInvalidContext),
            CallFrameInstruction::DefCfaOffset { offset: 1993 },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_def_cfa_expression() {
        let expr = [1, 2, 3, 4];
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected.set_cfa(CfaRule::Expression(Expression(EndianSlice::new(
            &expr,
            LittleEndian,
        ))));
        let instructions = [(
            Ok(false),
            CallFrameInstruction::DefCfaExpression {
                expression: Expression(EndianSlice::new(&expr, LittleEndian)),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_undefined() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(Register(5), RegisterRule::Undefined)
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::Undefined {
                register: Register(5),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_same_value() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(Register(0), RegisterRule::SameValue)
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::SameValue {
                register: Register(0),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_offset() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(
                Register(2),
                RegisterRule::Offset(3 * cie.data_alignment_factor),
            )
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::Offset {
                register: Register(2),
                factored_offset: 3,
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_offset_extended_sf() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(
                Register(4),
                RegisterRule::Offset(-3 * cie.data_alignment_factor),
            )
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::OffsetExtendedSf {
                register: Register(4),
                factored_offset: -3,
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_val_offset() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(
                Register(5),
                RegisterRule::ValOffset(7 * cie.data_alignment_factor),
            )
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::ValOffset {
                register: Register(5),
                factored_offset: 7,
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_val_offset_sf() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(
                Register(5),
                RegisterRule::ValOffset(-7 * cie.data_alignment_factor),
            )
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::ValOffsetSf {
                register: Register(5),
                factored_offset: -7,
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_expression() {
        let expr = [1, 2, 3, 4];
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(
                Register(9),
                RegisterRule::Expression(Expression(EndianSlice::new(&expr, LittleEndian))),
            )
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::Expression {
                register: Register(9),
                expression: Expression(EndianSlice::new(&expr, LittleEndian)),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_val_expression() {
        let expr = [1, 2, 3, 4];
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected
            .set_register_rule(
                Register(9),
                RegisterRule::ValExpression(Expression(EndianSlice::new(&expr, LittleEndian))),
            )
            .unwrap();
        let instructions = [(
            Ok(false),
            CallFrameInstruction::ValExpression {
                register: Register(9),
                expression: Expression(EndianSlice::new(&expr, LittleEndian)),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_restore() {
        let cie = make_test_cie();
        let fde = FrameDescriptionEntry {
            offset: 0,
            format: Format::Dwarf64,
            length: 0,
            address_range: 0,
            augmentation: None,
            initial_address: 0,
            initial_segment: 0,
            cie: cie.clone(),
            instructions: EndianSlice::new(&[], LittleEndian),
        };

        let mut ctx = UnwindContext::new();
        ctx.set_register_rule(Register(0), RegisterRule::Offset(1))
            .unwrap();
        ctx.save_initial_rules().unwrap();
        let expected = ctx.clone();
        ctx.set_register_rule(Register(0), RegisterRule::Offset(2))
            .unwrap();

        let instructions = [(
            Ok(false),
            CallFrameInstruction::Restore {
                register: Register(0),
            },
        )];
        assert_eval(ctx, expected, cie, Some(fde), instructions);
    }

    #[test]
    fn test_eval_restore_havent_saved_initial_context() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let expected = ctx.clone();
        let instructions = [(
            Err(Error::CfiInstructionInInvalidContext),
            CallFrameInstruction::Restore {
                register: Register(0),
            },
        )];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_remember_state() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let mut expected = ctx.clone();
        expected.push_row().unwrap();
        let instructions = [(Ok(false), CallFrameInstruction::RememberState)];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_restore_state() {
        let cie = make_test_cie();

        let mut ctx = UnwindContext::new();
        ctx.set_start_address(1);
        ctx.set_register_rule(Register(0), RegisterRule::SameValue)
            .unwrap();
        let mut expected = ctx.clone();
        ctx.push_row().unwrap();
        ctx.set_start_address(2);
        ctx.set_register_rule(Register(0), RegisterRule::Offset(16))
            .unwrap();

        // Restore state should preserve current location.
        expected.set_start_address(2);

        let instructions = [
            // First one pops just fine.
            (Ok(false), CallFrameInstruction::RestoreState),
            // Second pop would try to pop out of bounds.
            (
                Err(Error::PopWithEmptyStack),
                CallFrameInstruction::RestoreState,
            ),
        ];

        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_eval_nop() {
        let cie = make_test_cie();
        let ctx = UnwindContext::new();
        let expected = ctx.clone();
        let instructions = [(Ok(false), CallFrameInstruction::Nop)];
        assert_eval(ctx, expected, cie, None, instructions);
    }

    #[test]
    fn test_unwind_table_cie_no_rule() {
        #[allow(clippy::identity_op)]
        let initial_instructions = Section::with_endian(Endian::Little)
            // The CFA is -12 from register 4.
            .D8(constants::DW_CFA_def_cfa_sf.0)
            .uleb(4)
            .sleb(-12)
            .append_repeated(constants::DW_CFA_nop.0, 4);
        let initial_instructions = initial_instructions.get_contents().unwrap();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 8,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 1,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&initial_instructions, LittleEndian),
        };

        let instructions = Section::with_endian(Endian::Little)
            // A bunch of nop padding.
            .append_repeated(constants::DW_CFA_nop.0, 8);
        let instructions = instructions.get_contents().unwrap();

        let fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0,
            address_range: 100,
            augmentation: None,
            instructions: EndianSlice::new(&instructions, LittleEndian),
        };

        let section = &DebugFrame::from(EndianSlice::default());
        let bases = &BaseAddresses::default();
        let mut ctx = Box::new(UnwindContext::new());

        let mut table = fde
            .rows(section, bases, &mut ctx)
            .expect("Should run initial program OK");
        assert!(table.ctx.is_initialized);
        let expected_initial_rule = (Register(0), RegisterRule::Undefined);
        assert_eq!(table.ctx.initial_rule, Some(expected_initial_rule));

        {
            let row = table.next_row().expect("Should evaluate first row OK");
            let expected = UnwindTableRow {
                start_address: 0,
                end_address: 100,
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [].iter().collect(),
            };
            assert_eq!(Some(&expected), row);
        }

        // All done!
        assert_eq!(Ok(None), table.next_row());
        assert_eq!(Ok(None), table.next_row());
    }

    #[test]
    fn test_unwind_table_cie_single_rule() {
        #[allow(clippy::identity_op)]
        let initial_instructions = Section::with_endian(Endian::Little)
            // The CFA is -12 from register 4.
            .D8(constants::DW_CFA_def_cfa_sf.0)
            .uleb(4)
            .sleb(-12)
            // Register 3 is 4 from the CFA.
            .D8(constants::DW_CFA_offset.0 | 3)
            .uleb(4)
            .append_repeated(constants::DW_CFA_nop.0, 4);
        let initial_instructions = initial_instructions.get_contents().unwrap();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 8,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 1,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&initial_instructions, LittleEndian),
        };

        let instructions = Section::with_endian(Endian::Little)
            // A bunch of nop padding.
            .append_repeated(constants::DW_CFA_nop.0, 8);
        let instructions = instructions.get_contents().unwrap();

        let fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0,
            address_range: 100,
            augmentation: None,
            instructions: EndianSlice::new(&instructions, LittleEndian),
        };

        let section = &DebugFrame::from(EndianSlice::default());
        let bases = &BaseAddresses::default();
        let mut ctx = Box::new(UnwindContext::new());

        let mut table = fde
            .rows(section, bases, &mut ctx)
            .expect("Should run initial program OK");
        assert!(table.ctx.is_initialized);
        let expected_initial_rule = (Register(3), RegisterRule::Offset(4));
        assert_eq!(table.ctx.initial_rule, Some(expected_initial_rule));

        {
            let row = table.next_row().expect("Should evaluate first row OK");
            let expected = UnwindTableRow {
                start_address: 0,
                end_address: 100,
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [(Register(3), RegisterRule::Offset(4))].iter().collect(),
            };
            assert_eq!(Some(&expected), row);
        }

        // All done!
        assert_eq!(Ok(None), table.next_row());
        assert_eq!(Ok(None), table.next_row());
    }

    #[test]
    fn test_unwind_table_next_row() {
        #[allow(clippy::identity_op)]
        let initial_instructions = Section::with_endian(Endian::Little)
            // The CFA is -12 from register 4.
            .D8(constants::DW_CFA_def_cfa_sf.0)
            .uleb(4)
            .sleb(-12)
            // Register 0 is 8 from the CFA.
            .D8(constants::DW_CFA_offset.0 | 0)
            .uleb(8)
            // Register 3 is 4 from the CFA.
            .D8(constants::DW_CFA_offset.0 | 3)
            .uleb(4)
            .append_repeated(constants::DW_CFA_nop.0, 4);
        let initial_instructions = initial_instructions.get_contents().unwrap();

        let cie = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 8,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 1,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&initial_instructions, LittleEndian),
        };

        let instructions = Section::with_endian(Endian::Little)
            // Initial instructions form a row, advance the address by 1.
            .D8(constants::DW_CFA_advance_loc1.0)
            .D8(1)
            // Register 0 is -16 from the CFA.
            .D8(constants::DW_CFA_offset_extended_sf.0)
            .uleb(0)
            .sleb(-16)
            // Finish this row, advance the address by 32.
            .D8(constants::DW_CFA_advance_loc1.0)
            .D8(32)
            // Register 3 is -4 from the CFA.
            .D8(constants::DW_CFA_offset_extended_sf.0)
            .uleb(3)
            .sleb(-4)
            // Finish this row, advance the address by 64.
            .D8(constants::DW_CFA_advance_loc1.0)
            .D8(64)
            // Register 5 is 4 from the CFA.
            .D8(constants::DW_CFA_offset.0 | 5)
            .uleb(4)
            // A bunch of nop padding.
            .append_repeated(constants::DW_CFA_nop.0, 8);
        let instructions = instructions.get_contents().unwrap();

        let fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0,
            address_range: 100,
            augmentation: None,
            instructions: EndianSlice::new(&instructions, LittleEndian),
        };

        let section = &DebugFrame::from(EndianSlice::default());
        let bases = &BaseAddresses::default();
        let mut ctx = Box::new(UnwindContext::new());

        let mut table = fde
            .rows(section, bases, &mut ctx)
            .expect("Should run initial program OK");
        assert!(table.ctx.is_initialized);
        assert!(table.ctx.initial_rule.is_none());
        let expected_initial_rules: RegisterRuleMap<_> = [
            (Register(0), RegisterRule::Offset(8)),
            (Register(3), RegisterRule::Offset(4)),
        ]
        .iter()
        .collect();
        assert_eq!(table.ctx.stack[0].registers, expected_initial_rules);

        {
            let row = table.next_row().expect("Should evaluate first row OK");
            let expected = UnwindTableRow {
                start_address: 0,
                end_address: 1,
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [
                    (Register(0), RegisterRule::Offset(8)),
                    (Register(3), RegisterRule::Offset(4)),
                ]
                .iter()
                .collect(),
            };
            assert_eq!(Some(&expected), row);
        }

        {
            let row = table.next_row().expect("Should evaluate second row OK");
            let expected = UnwindTableRow {
                start_address: 1,
                end_address: 33,
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [
                    (Register(0), RegisterRule::Offset(-16)),
                    (Register(3), RegisterRule::Offset(4)),
                ]
                .iter()
                .collect(),
            };
            assert_eq!(Some(&expected), row);
        }

        {
            let row = table.next_row().expect("Should evaluate third row OK");
            let expected = UnwindTableRow {
                start_address: 33,
                end_address: 97,
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [
                    (Register(0), RegisterRule::Offset(-16)),
                    (Register(3), RegisterRule::Offset(-4)),
                ]
                .iter()
                .collect(),
            };
            assert_eq!(Some(&expected), row);
        }

        {
            let row = table.next_row().expect("Should evaluate fourth row OK");
            let expected = UnwindTableRow {
                start_address: 97,
                end_address: 100,
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [
                    (Register(0), RegisterRule::Offset(-16)),
                    (Register(3), RegisterRule::Offset(-4)),
                    (Register(5), RegisterRule::Offset(4)),
                ]
                .iter()
                .collect(),
            };
            assert_eq!(Some(&expected), row);
        }

        // All done!
        assert_eq!(Ok(None), table.next_row());
        assert_eq!(Ok(None), table.next_row());
    }

    #[test]
    fn test_unwind_info_for_address_ok() {
        let instrs1 = Section::with_endian(Endian::Big)
            // The CFA is -12 from register 4.
            .D8(constants::DW_CFA_def_cfa_sf.0)
            .uleb(4)
            .sleb(-12);
        let instrs1 = instrs1.get_contents().unwrap();

        let instrs2: Vec<_> = (0..8).map(|_| constants::DW_CFA_nop.0).collect();

        let instrs3 = Section::with_endian(Endian::Big)
            // Initial instructions form a row, advance the address by 100.
            .D8(constants::DW_CFA_advance_loc1.0)
            .D8(100)
            // Register 0 is -16 from the CFA.
            .D8(constants::DW_CFA_offset_extended_sf.0)
            .uleb(0)
            .sleb(-16);
        let instrs3 = instrs3.get_contents().unwrap();

        let instrs4: Vec<_> = (0..16).map(|_| constants::DW_CFA_nop.0).collect();

        let mut cie1 = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 8,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 1,
            return_address_register: Register(3),
            initial_instructions: EndianSlice::new(&instrs1, BigEndian),
        };

        let mut cie2 = CommonInformationEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            version: 4,
            augmentation: None,
            address_size: 4,
            segment_size: 0,
            code_alignment_factor: 1,
            data_alignment_factor: 1,
            return_address_register: Register(1),
            initial_instructions: EndianSlice::new(&instrs2, BigEndian),
        };

        let cie1_location = Label::new();
        let cie2_location = Label::new();

        // Write the CIEs first so that their length gets set before we clone
        // them into the FDEs and our equality assertions down the line end up
        // with all the CIEs always having he correct length.
        let kind = debug_frame_be();
        let section = Section::with_endian(kind.endian())
            .mark(&cie1_location)
            .cie(kind, None, &mut cie1)
            .mark(&cie2_location)
            .cie(kind, None, &mut cie2);

        let mut fde1 = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie1.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 200,
            augmentation: None,
            instructions: EndianSlice::new(&instrs3, BigEndian),
        };

        let mut fde2 = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie2.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_face,
            address_range: 9000,
            augmentation: None,
            instructions: EndianSlice::new(&instrs4, BigEndian),
        };

        let section =
            section
                .fde(kind, &cie1_location, &mut fde1)
                .fde(kind, &cie2_location, &mut fde2);
        section.start().set_const(0);

        let contents = section.get_contents().unwrap();
        let debug_frame = kind.section(&contents);

        // Get the second row of the unwind table in `instrs3`.
        let bases = Default::default();
        let mut ctx = Box::new(UnwindContext::new());
        let result = debug_frame.unwind_info_for_address(
            &bases,
            &mut ctx,
            0xfeed_beef + 150,
            DebugFrame::cie_from_offset,
        );
        assert!(result.is_ok());
        let unwind_info = result.unwrap();

        assert_eq!(
            *unwind_info,
            UnwindTableRow {
                start_address: fde1.initial_address() + 100,
                end_address: fde1.initial_address() + fde1.len(),
                saved_args_size: 0,
                cfa: CfaRule::RegisterAndOffset {
                    register: Register(4),
                    offset: -12,
                },
                registers: [(Register(0), RegisterRule::Offset(-16))].iter().collect(),
            }
        );
    }

    #[test]
    fn test_unwind_info_for_address_not_found() {
        let debug_frame = DebugFrame::new(&[], NativeEndian);
        let bases = Default::default();
        let mut ctx = Box::new(UnwindContext::new());
        let result = debug_frame.unwind_info_for_address(
            &bases,
            &mut ctx,
            0xbadb_ad99,
            DebugFrame::cie_from_offset,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::NoUnwindInfoForAddress);
    }

    #[test]
    fn test_eh_frame_hdr_unknown_version() {
        let bases = BaseAddresses::default();
        let buf = &[42];
        let result = EhFrameHdr::new(buf, NativeEndian).parse(&bases, 8);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::UnknownVersion(42));
    }

    #[test]
    fn test_eh_frame_hdr_omit_ehptr() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0xff)
            .L8(0x03)
            .L8(0x0b)
            .L32(2)
            .L32(10)
            .L32(1)
            .L32(20)
            .L32(2)
            .L32(0);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::CannotParseOmitPointerEncoding);
    }

    #[test]
    fn test_eh_frame_hdr_omit_count() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0x0b)
            .L8(0xff)
            .L8(0x0b)
            .L32(0x12345);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.eh_frame_ptr(), Pointer::Direct(0x12345));
        assert!(result.table().is_none());
    }

    #[test]
    fn test_eh_frame_hdr_omit_table() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0x0b)
            .L8(0x03)
            .L8(0xff)
            .L32(0x12345)
            .L32(2);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.eh_frame_ptr(), Pointer::Direct(0x12345));
        assert!(result.table().is_none());
    }

    #[test]
    fn test_eh_frame_hdr_varlen_table() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0x0b)
            .L8(0x03)
            .L8(0x01)
            .L32(0x12345)
            .L32(2);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.eh_frame_ptr(), Pointer::Direct(0x12345));
        let table = result.table();
        assert!(table.is_some());
        let table = table.unwrap();
        assert_eq!(
            table.lookup(0, &bases),
            Err(Error::VariableLengthSearchTable)
        );
    }

    #[test]
    fn test_eh_frame_hdr_indirect_length() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0x0b)
            .L8(0x83)
            .L8(0x0b)
            .L32(0x12345)
            .L32(2);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::UnsupportedPointerEncoding);
    }

    #[test]
    fn test_eh_frame_hdr_indirect_ptrs() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0x8b)
            .L8(0x03)
            .L8(0x8b)
            .L32(0x12345)
            .L32(2)
            .L32(10)
            .L32(1)
            .L32(20)
            .L32(2);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.eh_frame_ptr(), Pointer::Indirect(0x12345));
        let table = result.table();
        assert!(table.is_some());
        let table = table.unwrap();
        assert_eq!(
            table.lookup(0, &bases),
            Err(Error::UnsupportedPointerEncoding)
        );
    }

    #[test]
    fn test_eh_frame_hdr_good() {
        let section = Section::with_endian(Endian::Little)
            .L8(1)
            .L8(0x0b)
            .L8(0x03)
            .L8(0x0b)
            .L32(0x12345)
            .L32(2)
            .L32(10)
            .L32(1)
            .L32(20)
            .L32(2);
        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let result = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.eh_frame_ptr(), Pointer::Direct(0x12345));
        let table = result.table();
        assert!(table.is_some());
        let table = table.unwrap();
        assert_eq!(table.lookup(0, &bases), Ok(Pointer::Direct(1)));
        assert_eq!(table.lookup(9, &bases), Ok(Pointer::Direct(1)));
        assert_eq!(table.lookup(10, &bases), Ok(Pointer::Direct(1)));
        assert_eq!(table.lookup(11, &bases), Ok(Pointer::Direct(1)));
        assert_eq!(table.lookup(19, &bases), Ok(Pointer::Direct(1)));
        assert_eq!(table.lookup(20, &bases), Ok(Pointer::Direct(2)));
        assert_eq!(table.lookup(21, &bases), Ok(Pointer::Direct(2)));
        assert_eq!(table.lookup(100_000, &bases), Ok(Pointer::Direct(2)));
    }

    #[test]
    fn test_eh_frame_fde_for_address_good() {
        // First, setup eh_frame
        // Write the CIE first so that its length gets set before we clone it
        // into the FDE.
        let mut cie = make_test_cie();
        cie.format = Format::Dwarf32;
        cie.version = 1;

        let start_of_cie = Label::new();
        let end_of_cie = Label::new();

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .append_repeated(0, 16)
            .mark(&start_of_cie)
            .cie(kind, None, &mut cie)
            .mark(&end_of_cie);

        let mut fde1 = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 9,
            address_range: 4,
            augmentation: None,
            instructions: EndianSlice::new(&[], LittleEndian),
        };
        let mut fde2 = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 20,
            address_range: 8,
            augmentation: None,
            instructions: EndianSlice::new(&[], LittleEndian),
        };

        let start_of_fde1 = Label::new();
        let start_of_fde2 = Label::new();

        let section = section
            // +4 for the FDE length before the CIE offset.
            .mark(&start_of_fde1)
            .fde(kind, (&start_of_fde1 - &start_of_cie + 4) as u64, &mut fde1)
            .mark(&start_of_fde2)
            .fde(kind, (&start_of_fde2 - &start_of_cie + 4) as u64, &mut fde2);

        section.start().set_const(0);
        let section = section.get_contents().unwrap();
        let section = EndianSlice::new(&section, LittleEndian);
        let eh_frame = kind.section(&section);

        // Setup eh_frame_hdr
        let section = Section::with_endian(kind.endian())
            .L8(1)
            .L8(0x0b)
            .L8(0x03)
            .L8(0x0b)
            .L32(0x12345)
            .L32(2)
            .L32(10)
            .L32(0x12345 + start_of_fde1.value().unwrap() as u32)
            .L32(20)
            .L32(0x12345 + start_of_fde2.value().unwrap() as u32);

        let section = section.get_contents().unwrap();
        let bases = BaseAddresses::default();
        let eh_frame_hdr = EhFrameHdr::new(&section, LittleEndian).parse(&bases, 8);
        assert!(eh_frame_hdr.is_ok());
        let eh_frame_hdr = eh_frame_hdr.unwrap();

        let table = eh_frame_hdr.table();
        assert!(table.is_some());
        let table = table.unwrap();

        let bases = Default::default();
        let mut iter = table.iter(&bases);
        assert_eq!(
            iter.next(),
            Ok(Some((
                Pointer::Direct(10),
                Pointer::Direct(0x12345 + start_of_fde1.value().unwrap() as u64)
            )))
        );
        assert_eq!(
            iter.next(),
            Ok(Some((
                Pointer::Direct(20),
                Pointer::Direct(0x12345 + start_of_fde2.value().unwrap() as u64)
            )))
        );
        assert_eq!(iter.next(), Ok(None));

        assert_eq!(
            table.iter(&bases).nth(0),
            Ok(Some((
                Pointer::Direct(10),
                Pointer::Direct(0x12345 + start_of_fde1.value().unwrap() as u64)
            )))
        );

        assert_eq!(
            table.iter(&bases).nth(1),
            Ok(Some((
                Pointer::Direct(20),
                Pointer::Direct(0x12345 + start_of_fde2.value().unwrap() as u64)
            )))
        );
        assert_eq!(table.iter(&bases).nth(2), Ok(None));

        let f = |_: &_, _: &_, o: EhFrameOffset| {
            assert_eq!(o, EhFrameOffset(start_of_cie.value().unwrap() as usize));
            Ok(cie.clone())
        };
        assert_eq!(
            table.fde_for_address(&eh_frame, &bases, 9, f),
            Ok(fde1.clone())
        );
        assert_eq!(
            table.fde_for_address(&eh_frame, &bases, 10, f),
            Ok(fde1.clone())
        );
        assert_eq!(table.fde_for_address(&eh_frame, &bases, 11, f), Ok(fde1));
        assert_eq!(
            table.fde_for_address(&eh_frame, &bases, 19, f),
            Err(Error::NoUnwindInfoForAddress)
        );
        assert_eq!(
            table.fde_for_address(&eh_frame, &bases, 20, f),
            Ok(fde2.clone())
        );
        assert_eq!(table.fde_for_address(&eh_frame, &bases, 21, f), Ok(fde2));
        assert_eq!(
            table.fde_for_address(&eh_frame, &bases, 100_000, f),
            Err(Error::NoUnwindInfoForAddress)
        );
    }

    #[test]
    fn test_eh_frame_stops_at_zero_length() {
        let section = Section::with_endian(Endian::Little).L32(0);
        let section = section.get_contents().unwrap();
        let rest = &mut EndianSlice::new(&section, LittleEndian);
        let bases = Default::default();

        assert_eq!(
            parse_cfi_entry(&bases, &EhFrame::new(&*section, LittleEndian), rest),
            Ok(None)
        );

        assert_eq!(
            EhFrame::new(&section, LittleEndian).cie_from_offset(&bases, EhFrameOffset(0)),
            Err(Error::NoEntryAtGivenOffset)
        );
    }

    fn resolve_cie_offset(buf: &[u8], cie_offset: usize) -> Result<usize> {
        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf64,
            cie: make_test_cie(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 39,
            augmentation: None,
            instructions: EndianSlice::new(&[], LittleEndian),
        };

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .append_bytes(&buf)
            .fde(kind, cie_offset as u64, &mut fde)
            .append_bytes(&buf);

        let section = section.get_contents().unwrap();
        let eh_frame = kind.section(&section);
        let input = &mut EndianSlice::new(&section[buf.len()..], LittleEndian);

        let bases = Default::default();
        match parse_cfi_entry(&bases, &eh_frame, input) {
            Ok(Some(CieOrFde::Fde(partial))) => Ok(partial.cie_offset.0),
            Err(e) => Err(e),
            otherwise => panic!("Unexpected result: {:#?}", otherwise),
        }
    }

    #[test]
    fn test_eh_frame_resolve_cie_offset_ok() {
        let buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cie_offset = 2;
        // + 4 for size of length field
        assert_eq!(
            resolve_cie_offset(&buf, buf.len() + 4 - cie_offset),
            Ok(cie_offset)
        );
    }

    #[test]
    fn test_eh_frame_resolve_cie_offset_out_of_bounds() {
        let buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(
            resolve_cie_offset(&buf, buf.len() + 4 + 2),
            Err(Error::OffsetOutOfBounds)
        );
    }

    #[test]
    fn test_eh_frame_resolve_cie_offset_underflow() {
        let buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(
            resolve_cie_offset(&buf, ::core::usize::MAX),
            Err(Error::OffsetOutOfBounds)
        );
    }

    #[test]
    fn test_eh_frame_fde_ok() {
        let mut cie = make_test_cie();
        cie.format = Format::Dwarf32;
        cie.version = 1;

        let start_of_cie = Label::new();
        let end_of_cie = Label::new();

        // Write the CIE first so that its length gets set before we clone it
        // into the FDE.
        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .append_repeated(0, 16)
            .mark(&start_of_cie)
            .cie(kind, None, &mut cie)
            .mark(&end_of_cie);

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 999,
            augmentation: None,
            instructions: EndianSlice::new(&[], LittleEndian),
        };

        let section = section
            // +4 for the FDE length before the CIE offset.
            .fde(kind, (&end_of_cie - &start_of_cie + 4) as u64, &mut fde);

        section.start().set_const(0);
        let section = section.get_contents().unwrap();
        let eh_frame = kind.section(&section);
        let section = EndianSlice::new(&section, LittleEndian);

        let mut offset = None;
        match parse_fde(
            eh_frame,
            &mut section.range_from(end_of_cie.value().unwrap() as usize..),
            |_, _, o| {
                offset = Some(o);
                assert_eq!(o, EhFrameOffset(start_of_cie.value().unwrap() as usize));
                Ok(cie.clone())
            },
        ) {
            Ok(actual) => assert_eq!(actual, fde),
            otherwise => panic!("Unexpected result {:?}", otherwise),
        }
        assert!(offset.is_some());
    }

    #[test]
    fn test_eh_frame_fde_out_of_bounds() {
        let mut cie = make_test_cie();
        cie.version = 1;

        let end_of_cie = Label::new();

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf64,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_beef,
            address_range: 999,
            augmentation: None,
            instructions: EndianSlice::new(&[], LittleEndian),
        };

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .cie(kind, None, &mut cie)
            .mark(&end_of_cie)
            .fde(kind, 99_999_999_999_999, &mut fde);

        section.start().set_const(0);
        let section = section.get_contents().unwrap();
        let eh_frame = kind.section(&section);
        let section = EndianSlice::new(&section, LittleEndian);

        let result = parse_fde(
            eh_frame,
            &mut section.range_from(end_of_cie.value().unwrap() as usize..),
            UnwindSection::cie_from_offset,
        );
        assert_eq!(result, Err(Error::OffsetOutOfBounds));
    }

    #[test]
    fn test_augmentation_parse_not_z_augmentation() {
        let augmentation = &mut EndianSlice::new(b"wtf", NativeEndian);
        let bases = Default::default();
        let address_size = 8;
        let section = EhFrame::new(&[], NativeEndian);
        let input = &mut EndianSlice::new(&[], NativeEndian);
        assert_eq!(
            Augmentation::parse(augmentation, &bases, address_size, &section, input),
            Err(Error::UnknownAugmentation)
        );
    }

    #[test]
    fn test_augmentation_parse_just_signal_trampoline() {
        let aug_str = &mut EndianSlice::new(b"S", LittleEndian);
        let bases = Default::default();
        let address_size = 8;
        let section = EhFrame::new(&[], LittleEndian);
        let input = &mut EndianSlice::new(&[], LittleEndian);

        let mut augmentation = Augmentation::default();
        augmentation.is_signal_trampoline = true;

        assert_eq!(
            Augmentation::parse(aug_str, &bases, address_size, &section, input),
            Ok(augmentation)
        );
    }

    #[test]
    fn test_augmentation_parse_unknown_part_of_z_augmentation() {
        // The 'Z' character is not defined by the z-style augmentation.
        let bases = Default::default();
        let address_size = 8;
        let section = Section::with_endian(Endian::Little)
            .uleb(4)
            .append_repeated(4, 4)
            .get_contents()
            .unwrap();
        let section = EhFrame::new(&section, LittleEndian);
        let input = &mut section.section().clone();
        let augmentation = &mut EndianSlice::new(b"zZ", LittleEndian);
        assert_eq!(
            Augmentation::parse(augmentation, &bases, address_size, &section, input),
            Err(Error::UnknownAugmentation)
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_augmentation_parse_L() {
        let bases = Default::default();
        let address_size = 8;
        let rest = [9, 8, 7, 6, 5, 4, 3, 2, 1];

        let section = Section::with_endian(Endian::Little)
            .uleb(1)
            .D8(constants::DW_EH_PE_uleb128.0)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = EhFrame::new(&section, LittleEndian);
        let input = &mut section.section().clone();
        let aug_str = &mut EndianSlice::new(b"zL", LittleEndian);

        let mut augmentation = Augmentation::default();
        augmentation.lsda = Some(constants::DW_EH_PE_uleb128);

        assert_eq!(
            Augmentation::parse(aug_str, &bases, address_size, &section, input),
            Ok(augmentation)
        );
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_augmentation_parse_P() {
        let bases = Default::default();
        let address_size = 8;
        let rest = [9, 8, 7, 6, 5, 4, 3, 2, 1];

        let section = Section::with_endian(Endian::Little)
            .uleb(9)
            .D8(constants::DW_EH_PE_udata8.0)
            .L64(0xf00d_f00d)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = EhFrame::new(&section, LittleEndian);
        let input = &mut section.section().clone();
        let aug_str = &mut EndianSlice::new(b"zP", LittleEndian);

        let mut augmentation = Augmentation::default();
        augmentation.personality = Some((constants::DW_EH_PE_udata8, Pointer::Direct(0xf00d_f00d)));

        assert_eq!(
            Augmentation::parse(aug_str, &bases, address_size, &section, input),
            Ok(augmentation)
        );
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_augmentation_parse_R() {
        let bases = Default::default();
        let address_size = 8;
        let rest = [9, 8, 7, 6, 5, 4, 3, 2, 1];

        let section = Section::with_endian(Endian::Little)
            .uleb(1)
            .D8(constants::DW_EH_PE_udata4.0)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = EhFrame::new(&section, LittleEndian);
        let input = &mut section.section().clone();
        let aug_str = &mut EndianSlice::new(b"zR", LittleEndian);

        let mut augmentation = Augmentation::default();
        augmentation.fde_address_encoding = Some(constants::DW_EH_PE_udata4);

        assert_eq!(
            Augmentation::parse(aug_str, &bases, address_size, &section, input),
            Ok(augmentation)
        );
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_augmentation_parse_S() {
        let bases = Default::default();
        let address_size = 8;
        let rest = [9, 8, 7, 6, 5, 4, 3, 2, 1];

        let section = Section::with_endian(Endian::Little)
            .uleb(0)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = EhFrame::new(&section, LittleEndian);
        let input = &mut section.section().clone();
        let aug_str = &mut EndianSlice::new(b"zS", LittleEndian);

        let mut augmentation = Augmentation::default();
        augmentation.is_signal_trampoline = true;

        assert_eq!(
            Augmentation::parse(aug_str, &bases, address_size, &section, input),
            Ok(augmentation)
        );
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    fn test_augmentation_parse_all() {
        let bases = Default::default();
        let address_size = 8;
        let rest = [9, 8, 7, 6, 5, 4, 3, 2, 1];

        let section = Section::with_endian(Endian::Little)
            .uleb(1 + 9 + 1)
            // L
            .D8(constants::DW_EH_PE_uleb128.0)
            // P
            .D8(constants::DW_EH_PE_udata8.0)
            .L64(0x1bad_f00d)
            // R
            .D8(constants::DW_EH_PE_uleb128.0)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = EhFrame::new(&section, LittleEndian);
        let input = &mut section.section().clone();
        let aug_str = &mut EndianSlice::new(b"zLPRS", LittleEndian);

        let augmentation = Augmentation {
            lsda: Some(constants::DW_EH_PE_uleb128),
            personality: Some((constants::DW_EH_PE_udata8, Pointer::Direct(0x1bad_f00d))),
            fde_address_encoding: Some(constants::DW_EH_PE_uleb128),
            is_signal_trampoline: true,
        };

        assert_eq!(
            Augmentation::parse(aug_str, &bases, address_size, &section, input),
            Ok(augmentation)
        );
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    fn test_eh_frame_fde_no_augmentation() {
        let instrs = [1, 2, 3, 4];
        let cie_offset = 1;

        let mut cie = make_test_cie();
        cie.format = Format::Dwarf32;
        cie.version = 1;

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_face,
            address_range: 9000,
            augmentation: None,
            instructions: EndianSlice::new(&instrs, LittleEndian),
        };

        let rest = [1, 2, 3, 4];

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = kind.section(&section);
        let input = &mut section.section().clone();

        let result = parse_fde(section, input, |_, _, _| Ok(cie.clone()));
        assert_eq!(result, Ok(fde));
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    fn test_eh_frame_fde_empty_augmentation() {
        let instrs = [1, 2, 3, 4];
        let cie_offset = 1;

        let mut cie = make_test_cie();
        cie.format = Format::Dwarf32;
        cie.version = 1;
        cie.augmentation = Some(Augmentation::default());

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_face,
            address_range: 9000,
            augmentation: Some(AugmentationData::default()),
            instructions: EndianSlice::new(&instrs, LittleEndian),
        };

        let rest = [1, 2, 3, 4];

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = kind.section(&section);
        let input = &mut section.section().clone();

        let result = parse_fde(section, input, |_, _, _| Ok(cie.clone()));
        assert_eq!(result, Ok(fde));
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    fn test_eh_frame_fde_lsda_augmentation() {
        let instrs = [1, 2, 3, 4];
        let cie_offset = 1;

        let mut cie = make_test_cie();
        cie.format = Format::Dwarf32;
        cie.version = 1;
        cie.augmentation = Some(Augmentation::default());
        cie.augmentation.as_mut().unwrap().lsda = Some(constants::DW_EH_PE_absptr);

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_face,
            address_range: 9000,
            augmentation: Some(AugmentationData {
                lsda: Some(Pointer::Direct(0x1122_3344)),
            }),
            instructions: EndianSlice::new(&instrs, LittleEndian),
        };

        let rest = [1, 2, 3, 4];

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = kind.section(&section);
        let input = &mut section.section().clone();

        let result = parse_fde(section, input, |_, _, _| Ok(cie.clone()));
        assert_eq!(result, Ok(fde));
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    fn test_eh_frame_fde_lsda_function_relative() {
        let instrs = [1, 2, 3, 4];
        let cie_offset = 1;

        let mut cie = make_test_cie();
        cie.format = Format::Dwarf32;
        cie.version = 1;
        cie.augmentation = Some(Augmentation::default());
        cie.augmentation.as_mut().unwrap().lsda = Some(constants::DwEhPe(
            constants::DW_EH_PE_funcrel.0 | constants::DW_EH_PE_absptr.0,
        ));

        let mut fde = FrameDescriptionEntry {
            offset: 0,
            length: 0,
            format: Format::Dwarf32,
            cie: cie.clone(),
            initial_segment: 0,
            initial_address: 0xfeed_face,
            address_range: 9000,
            augmentation: Some(AugmentationData {
                lsda: Some(Pointer::Direct(0xbeef)),
            }),
            instructions: EndianSlice::new(&instrs, LittleEndian),
        };

        let rest = [1, 2, 3, 4];

        let kind = eh_frame_le();
        let section = Section::with_endian(kind.endian())
            .append_repeated(10, 10)
            .fde(kind, cie_offset, &mut fde)
            .append_bytes(&rest)
            .get_contents()
            .unwrap();
        let section = kind.section(&section);
        let input = &mut section.section().range_from(10..);

        // Adjust the FDE's augmentation to be relative to the function.
        fde.augmentation.as_mut().unwrap().lsda = Some(Pointer::Direct(0xfeed_face + 0xbeef));

        let result = parse_fde(section, input, |_, _, _| Ok(cie.clone()));
        assert_eq!(result, Ok(fde));
        assert_eq!(*input, EndianSlice::new(&rest, LittleEndian));
    }

    #[test]
    fn test_eh_frame_cie_personality_function_relative_bad_context() {
        let instrs = [1, 2, 3, 4];

        let length = Label::new();
        let start = Label::new();
        let end = Label::new();

        let aug_len = Label::new();
        let aug_start = Label::new();
        let aug_end = Label::new();

        let section = Section::with_endian(Endian::Little)
            // Length
            .L32(&length)
            .mark(&start)
            // CIE ID
            .L32(0)
            // Version
            .D8(1)
            // Augmentation
            .append_bytes(b"zP\0")
            // Code alignment factor
            .uleb(1)
            // Data alignment factor
            .sleb(1)
            // Return address register
            .uleb(1)
            // Augmentation data length. This is a uleb, be we rely on the value
            // being less than 2^7 and therefore a valid uleb (can't use Label
            // with uleb).
            .D8(&aug_len)
            .mark(&aug_start)
            // Augmentation data. Personality encoding and then encoded pointer.
            .D8(constants::DW_EH_PE_funcrel.0 | constants::DW_EH_PE_uleb128.0)
            .uleb(1)
            .mark(&aug_end)
            // Initial instructions
            .append_bytes(&instrs)
            .mark(&end);

        length.set_const((&end - &start) as u64);
        aug_len.set_const((&aug_end - &aug_start) as u64);

        let section = section.get_contents().unwrap();
        let section = EhFrame::new(&section, LittleEndian);

        let bases = BaseAddresses::default();
        let mut iter = section.entries(&bases);
        assert_eq!(iter.next(), Err(Error::FuncRelativePointerInBadContext));
    }

    #[test]
    fn register_rule_map_eq() {
        // Different order, but still equal.
        let map1: RegisterRuleMap<EndianSlice<LittleEndian>> = [
            (Register(0), RegisterRule::SameValue),
            (Register(3), RegisterRule::Offset(1)),
        ]
        .iter()
        .collect();
        let map2: RegisterRuleMap<EndianSlice<LittleEndian>> = [
            (Register(3), RegisterRule::Offset(1)),
            (Register(0), RegisterRule::SameValue),
        ]
        .iter()
        .collect();
        assert_eq!(map1, map2);
        assert_eq!(map2, map1);

        // Not equal.
        let map3: RegisterRuleMap<EndianSlice<LittleEndian>> = [
            (Register(0), RegisterRule::SameValue),
            (Register(2), RegisterRule::Offset(1)),
        ]
        .iter()
        .collect();
        let map4: RegisterRuleMap<EndianSlice<LittleEndian>> = [
            (Register(3), RegisterRule::Offset(1)),
            (Register(0), RegisterRule::SameValue),
        ]
        .iter()
        .collect();
        assert!(map3 != map4);
        assert!(map4 != map3);

        // One has undefined explicitly set, other implicitly has undefined.
        let mut map5 = RegisterRuleMap::<EndianSlice<LittleEndian>>::default();
        map5.set(Register(0), RegisterRule::SameValue).unwrap();
        map5.set(Register(0), RegisterRule::Undefined).unwrap();
        let map6 = RegisterRuleMap::<EndianSlice<LittleEndian>>::default();
        assert_eq!(map5, map6);
        assert_eq!(map6, map5);
    }

    #[test]
    fn iter_register_rules() {
        let mut row = UnwindTableRow::<EndianSlice<LittleEndian>>::default();
        row.registers = [
            (Register(0), RegisterRule::SameValue),
            (Register(1), RegisterRule::Offset(1)),
            (Register(2), RegisterRule::ValOffset(2)),
        ]
        .iter()
        .collect();

        let mut found0 = false;
        let mut found1 = false;
        let mut found2 = false;

        for &(register, ref rule) in row.registers() {
            match register.0 {
                0 => {
                    assert_eq!(found0, false);
                    found0 = true;
                    assert_eq!(*rule, RegisterRule::SameValue);
                }
                1 => {
                    assert_eq!(found1, false);
                    found1 = true;
                    assert_eq!(*rule, RegisterRule::Offset(1));
                }
                2 => {
                    assert_eq!(found2, false);
                    found2 = true;
                    assert_eq!(*rule, RegisterRule::ValOffset(2));
                }
                x => panic!("Unexpected register rule: ({}, {:?})", x, rule),
            }
        }

        assert_eq!(found0, true);
        assert_eq!(found1, true);
        assert_eq!(found2, true);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_of_unwind_ctx() {
        use core::mem;
        let size = mem::size_of::<UnwindContext<EndianSlice<NativeEndian>>>();
        let max_size = 30968;
        if size > max_size {
            assert_eq!(size, max_size);
        }
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_of_register_rule_map() {
        use core::mem;
        let size = mem::size_of::<RegisterRuleMap<EndianSlice<NativeEndian>>>();
        let max_size = 6152;
        if size > max_size {
            assert_eq!(size, max_size);
        }
    }

    #[test]
    fn test_parse_pointer_encoding_ok() {
        use crate::endianity::NativeEndian;
        let expected =
            constants::DwEhPe(constants::DW_EH_PE_uleb128.0 | constants::DW_EH_PE_pcrel.0);
        let input = [expected.0, 1, 2, 3, 4];
        let input = &mut EndianSlice::new(&input, NativeEndian);
        assert_eq!(parse_pointer_encoding(input), Ok(expected));
        assert_eq!(*input, EndianSlice::new(&[1, 2, 3, 4], NativeEndian));
    }

    #[test]
    fn test_parse_pointer_encoding_bad_encoding() {
        use crate::endianity::NativeEndian;
        let expected =
            constants::DwEhPe((constants::DW_EH_PE_sdata8.0 + 1) | constants::DW_EH_PE_pcrel.0);
        let input = [expected.0, 1, 2, 3, 4];
        let input = &mut EndianSlice::new(&input, NativeEndian);
        assert_eq!(
            Err(Error::UnknownPointerEncoding),
            parse_pointer_encoding(input)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_absptr() {
        let encoding = constants::DW_EH_PE_absptr;
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L32(0xf00d_f00d)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0xf00d_f00d))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_pcrel() {
        let encoding = constants::DW_EH_PE_pcrel;
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .append_repeated(0, 0x10)
            .L32(0x1)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input.range_from(0x10..);

        let parameters = PointerEncodingParameters {
            bases: &BaseAddresses::default().set_eh_frame(0x100).eh_frame,
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x111))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_pcrel_undefined() {
        let encoding = constants::DW_EH_PE_pcrel;

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::PcRelativePointerButSectionBaseIsUndefined)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_textrel() {
        let encoding = constants::DW_EH_PE_textrel;
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L32(0x1)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &BaseAddresses::default().set_text(0x10).eh_frame,
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x11))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_textrel_undefined() {
        let encoding = constants::DW_EH_PE_textrel;

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::TextRelativePointerButTextBaseIsUndefined)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_datarel() {
        let encoding = constants::DW_EH_PE_datarel;
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L32(0x1)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &BaseAddresses::default().set_got(0x10).eh_frame,
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x11))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_datarel_undefined() {
        let encoding = constants::DW_EH_PE_datarel;

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::DataRelativePointerButDataBaseIsUndefined)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_funcrel() {
        let encoding = constants::DW_EH_PE_funcrel;
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L32(0x1)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: Some(0x10),
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x11))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_funcrel_undefined() {
        let encoding = constants::DW_EH_PE_funcrel;

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::FuncRelativePointerInBadContext)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_uleb128() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_uleb128.0);
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .uleb(0x12_3456)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x12_3456))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_udata2() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_udata2.0);
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L16(0x1234)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x1234))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_udata4() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_udata4.0);
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L32(0x1234_5678)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x1234_5678))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_udata8() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_udata8.0);
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .L64(0x1234_5678_1234_5678)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x1234_5678_1234_5678))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_sleb128() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_textrel.0 | constants::DW_EH_PE_sleb128.0);
        let expected_rest = [1, 2, 3, 4];

        let input = Section::with_endian(Endian::Little)
            .sleb(-0x1111)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &BaseAddresses::default().set_text(0x1111_1111).eh_frame,
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(0x1111_0000))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_sdata2() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_sdata2.0);
        let expected_rest = [1, 2, 3, 4];
        let expected = 0x111 as i16;

        let input = Section::with_endian(Endian::Little)
            .L16(expected as u16)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(expected as u64))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_sdata4() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_sdata4.0);
        let expected_rest = [1, 2, 3, 4];
        let expected = 0x111_1111 as i32;

        let input = Section::with_endian(Endian::Little)
            .L32(expected as u32)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(expected as u64))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_sdata8() {
        let encoding =
            constants::DwEhPe(constants::DW_EH_PE_absptr.0 | constants::DW_EH_PE_sdata8.0);
        let expected_rest = [1, 2, 3, 4];
        let expected = -0x11_1111_1222_2222 as i64;

        let input = Section::with_endian(Endian::Little)
            .L64(expected as u64)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Direct(expected as u64))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }

    #[test]
    fn test_parse_encoded_pointer_omit() {
        let encoding = constants::DW_EH_PE_omit;

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::CannotParseOmitPointerEncoding)
        );
        assert_eq!(rest, input);
    }

    #[test]
    fn test_parse_encoded_pointer_bad_encoding() {
        let encoding = constants::DwEhPe(constants::DW_EH_PE_sdata8.0 + 1);

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::UnknownPointerEncoding)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_aligned() {
        // FIXME: support this encoding!

        let encoding = constants::DW_EH_PE_aligned;

        let input = Section::with_endian(Endian::Little).L32(0x1);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Err(Error::UnsupportedPointerEncoding)
        );
    }

    #[test]
    fn test_parse_encoded_pointer_indirect() {
        let expected_rest = [1, 2, 3, 4];
        let encoding = constants::DW_EH_PE_indirect;

        let input = Section::with_endian(Endian::Little)
            .L32(0x1234_5678)
            .append_bytes(&expected_rest);
        let input = input.get_contents().unwrap();
        let input = EndianSlice::new(&input, LittleEndian);
        let mut rest = input;

        let parameters = PointerEncodingParameters {
            bases: &SectionBaseAddresses::default(),
            func_base: None,
            address_size: 4,
            section: &input,
        };
        assert_eq!(
            parse_encoded_pointer(encoding, &parameters, &mut rest),
            Ok(Pointer::Indirect(0x1234_5678))
        );
        assert_eq!(rest, EndianSlice::new(&expected_rest, LittleEndian));
    }
}
