use alloc::string::String;
use alloc::sync::Arc;

use crate::common::{
    DebugAddrBase, DebugAddrIndex, DebugInfoOffset, DebugLineStrOffset, DebugLocListsBase,
    DebugLocListsIndex, DebugRngListsBase, DebugRngListsIndex, DebugStrOffset, DebugStrOffsetsBase,
    DebugStrOffsetsIndex, DebugTypeSignature, DebugTypesOffset, DwarfFileType, DwoId, Encoding,
    LocationListsOffset, RangeListsOffset, RawRangeListsOffset, SectionId, UnitSectionOffset,
};
use crate::constants;
use crate::read::{
    Abbreviations, AttributeValue, DebugAbbrev, DebugAddr, DebugAranges, DebugCuIndex, DebugInfo,
    DebugInfoUnitHeadersIter, DebugLine, DebugLineStr, DebugLoc, DebugLocLists, DebugRngLists,
    DebugStr, DebugStrOffsets, DebugTuIndex, DebugTypes, DebugTypesUnitHeadersIter,
    DebuggingInformationEntry, EntriesCursor, EntriesRaw, EntriesTree, Error,
    IncompleteLineProgram, LocListIter, LocationLists, Range, RangeLists, RawLocListIter,
    RawRngListIter, Reader, ReaderOffset, ReaderOffsetId, Result, RngListIter, Section, UnitHeader,
    UnitIndex, UnitIndexSectionIterator, UnitOffset, UnitType,
};

/// All of the commonly used DWARF sections, and other common information.
#[derive(Debug, Default)]
pub struct Dwarf<R> {
    /// The `.debug_abbrev` section.
    pub debug_abbrev: DebugAbbrev<R>,

    /// The `.debug_addr` section.
    pub debug_addr: DebugAddr<R>,

    /// The `.debug_aranges` section.
    pub debug_aranges: DebugAranges<R>,

    /// The `.debug_info` section.
    pub debug_info: DebugInfo<R>,

    /// The `.debug_line` section.
    pub debug_line: DebugLine<R>,

    /// The `.debug_line_str` section.
    pub debug_line_str: DebugLineStr<R>,

    /// The `.debug_str` section.
    pub debug_str: DebugStr<R>,

    /// The `.debug_str_offsets` section.
    pub debug_str_offsets: DebugStrOffsets<R>,

    /// The `.debug_types` section.
    pub debug_types: DebugTypes<R>,

    /// The location lists in the `.debug_loc` and `.debug_loclists` sections.
    pub locations: LocationLists<R>,

    /// The range lists in the `.debug_ranges` and `.debug_rnglists` sections.
    pub ranges: RangeLists<R>,

    /// The type of this file.
    pub file_type: DwarfFileType,

    /// The DWARF sections for a supplementary object file.
    pub sup: Option<Arc<Dwarf<R>>>,
}

impl<T> Dwarf<T> {
    /// Try to load the DWARF sections using the given loader function.
    ///
    /// `section` loads a DWARF section from the object file.
    /// It should return an empty section if the section does not exist.
    ///
    /// `section` may either directly return a `Reader` instance (such as
    /// `EndianSlice`), or it may return some other type and then convert
    /// that type into a `Reader` using `Dwarf::borrow`.
    ///
    /// After loading, the user should set the `file_type` field and
    /// call `load_sup` if required.
    pub fn load<F, E>(mut section: F) -> core::result::Result<Self, E>
    where
        F: FnMut(SectionId) -> core::result::Result<T, E>,
    {
        // Section types are inferred.
        let debug_loc = Section::load(&mut section)?;
        let debug_loclists = Section::load(&mut section)?;
        let debug_ranges = Section::load(&mut section)?;
        let debug_rnglists = Section::load(&mut section)?;
        Ok(Dwarf {
            debug_abbrev: Section::load(&mut section)?,
            debug_addr: Section::load(&mut section)?,
            debug_aranges: Section::load(&mut section)?,
            debug_info: Section::load(&mut section)?,
            debug_line: Section::load(&mut section)?,
            debug_line_str: Section::load(&mut section)?,
            debug_str: Section::load(&mut section)?,
            debug_str_offsets: Section::load(&mut section)?,
            debug_types: Section::load(&mut section)?,
            locations: LocationLists::new(debug_loc, debug_loclists),
            ranges: RangeLists::new(debug_ranges, debug_rnglists),
            file_type: DwarfFileType::Main,
            sup: None,
        })
    }

    /// Load the DWARF sections from the supplementary object file.
    ///
    /// `section` operates the same as for `load`.
    ///
    /// Sets `self.sup`, replacing any previous value.
    pub fn load_sup<F, E>(&mut self, section: F) -> core::result::Result<(), E>
    where
        F: FnMut(SectionId) -> core::result::Result<T, E>,
    {
        self.sup = Some(Arc::new(Self::load(section)?));
        Ok(())
    }

    /// Create a `Dwarf` structure that references the data in `self`.
    ///
    /// This is useful when `R` implements `Reader` but `T` does not.
    ///
    /// ## Example Usage
    ///
    /// It can be useful to load DWARF sections into owned data structures,
    /// such as `Vec`. However, we do not implement the `Reader` trait
    /// for `Vec`, because it would be very inefficient, but this trait
    /// is required for all of the methods that parse the DWARF data.
    /// So we first load the DWARF sections into `Vec`s, and then use
    /// `borrow` to create `Reader`s that reference the data.
    ///
    /// ```rust,no_run
    /// # fn example() -> Result<(), gimli::Error> {
    /// # let loader = |name| -> Result<_, gimli::Error> { unimplemented!() };
    /// # let sup_loader = |name| -> Result<_, gimli::Error> { unimplemented!() };
    /// // Read the DWARF sections into `Vec`s with whatever object loader you're using.
    /// let mut owned_dwarf: gimli::Dwarf<Vec<u8>> = gimli::Dwarf::load(loader)?;
    /// owned_dwarf.load_sup(sup_loader)?;
    /// // Create references to the DWARF sections.
    /// let dwarf = owned_dwarf.borrow(|section| {
    ///     gimli::EndianSlice::new(&section, gimli::LittleEndian)
    /// });
    /// # unreachable!()
    /// # }
    /// ```
    pub fn borrow<'a, F, R>(&'a self, mut borrow: F) -> Dwarf<R>
    where
        F: FnMut(&'a T) -> R,
    {
        Dwarf {
            debug_abbrev: self.debug_abbrev.borrow(&mut borrow),
            debug_addr: self.debug_addr.borrow(&mut borrow),
            debug_aranges: self.debug_aranges.borrow(&mut borrow),
            debug_info: self.debug_info.borrow(&mut borrow),
            debug_line: self.debug_line.borrow(&mut borrow),
            debug_line_str: self.debug_line_str.borrow(&mut borrow),
            debug_str: self.debug_str.borrow(&mut borrow),
            debug_str_offsets: self.debug_str_offsets.borrow(&mut borrow),
            debug_types: self.debug_types.borrow(&mut borrow),
            locations: self.locations.borrow(&mut borrow),
            ranges: self.ranges.borrow(&mut borrow),
            file_type: self.file_type,
            sup: self.sup().map(|sup| Arc::new(sup.borrow(borrow))),
        }
    }

    /// Return a reference to the DWARF sections for supplementary object file.
    pub fn sup(&self) -> Option<&Dwarf<T>> {
        self.sup.as_ref().map(Arc::as_ref)
    }
}

impl<R: Reader> Dwarf<R> {
    /// Iterate the unit headers in the `.debug_info` section.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    #[inline]
    pub fn units(&self) -> DebugInfoUnitHeadersIter<R> {
        self.debug_info.units()
    }

    /// Construct a new `Unit` from the given unit header.
    #[inline]
    pub fn unit(&self, header: UnitHeader<R>) -> Result<Unit<R>> {
        Unit::new(self, header)
    }

    /// Iterate the type-unit headers in the `.debug_types` section.
    ///
    /// Can be [used with
    /// `FallibleIterator`](./index.html#using-with-fallibleiterator).
    #[inline]
    pub fn type_units(&self) -> DebugTypesUnitHeadersIter<R> {
        self.debug_types.units()
    }

    /// Parse the abbreviations for a compilation unit.
    // TODO: provide caching of abbreviations
    #[inline]
    pub fn abbreviations(&self, unit: &UnitHeader<R>) -> Result<Abbreviations> {
        unit.abbreviations(&self.debug_abbrev)
    }

    /// Return the string offset at the given index.
    #[inline]
    pub fn string_offset(
        &self,
        unit: &Unit<R>,
        index: DebugStrOffsetsIndex<R::Offset>,
    ) -> Result<DebugStrOffset<R::Offset>> {
        self.debug_str_offsets
            .get_str_offset(unit.header.format(), unit.str_offsets_base, index)
    }

    /// Return the string at the given offset in `.debug_str`.
    #[inline]
    pub fn string(&self, offset: DebugStrOffset<R::Offset>) -> Result<R> {
        self.debug_str.get_str(offset)
    }

    /// Return the string at the given offset in `.debug_line_str`.
    #[inline]
    pub fn line_string(&self, offset: DebugLineStrOffset<R::Offset>) -> Result<R> {
        self.debug_line_str.get_str(offset)
    }

    /// Return an attribute value as a string slice.
    ///
    /// If the attribute value is one of:
    ///
    /// - an inline `DW_FORM_string` string
    /// - a `DW_FORM_strp` reference to an offset into the `.debug_str` section
    /// - a `DW_FORM_strp_sup` reference to an offset into a supplementary
    /// object file
    /// - a `DW_FORM_line_strp` reference to an offset into the `.debug_line_str`
    /// section
    /// - a `DW_FORM_strx` index into the `.debug_str_offsets` entries for the unit
    ///
    /// then return the attribute's string value. Returns an error if the attribute
    /// value does not have a string form, or if a string form has an invalid value.
    pub fn attr_string(&self, unit: &Unit<R>, attr: AttributeValue<R>) -> Result<R> {
        match attr {
            AttributeValue::String(string) => Ok(string),
            AttributeValue::DebugStrRef(offset) => self.debug_str.get_str(offset),
            AttributeValue::DebugStrRefSup(offset) => {
                if let Some(sup) = self.sup() {
                    sup.debug_str.get_str(offset)
                } else {
                    Err(Error::ExpectedStringAttributeValue)
                }
            }
            AttributeValue::DebugLineStrRef(offset) => self.debug_line_str.get_str(offset),
            AttributeValue::DebugStrOffsetsIndex(index) => {
                let offset = self.debug_str_offsets.get_str_offset(
                    unit.header.format(),
                    unit.str_offsets_base,
                    index,
                )?;
                self.debug_str.get_str(offset)
            }
            _ => Err(Error::ExpectedStringAttributeValue),
        }
    }

    /// Return the address at the given index.
    pub fn address(&self, unit: &Unit<R>, index: DebugAddrIndex<R::Offset>) -> Result<u64> {
        self.debug_addr
            .get_address(unit.encoding().address_size, unit.addr_base, index)
    }

    /// Try to return an attribute value as an address.
    ///
    /// If the attribute value is one of:
    ///
    /// - a `DW_FORM_addr`
    /// - a `DW_FORM_addrx` index into the `.debug_addr` entries for the unit
    ///
    /// then return the address.
    /// Returns `None` for other forms.
    pub fn attr_address(&self, unit: &Unit<R>, attr: AttributeValue<R>) -> Result<Option<u64>> {
        match attr {
            AttributeValue::Addr(addr) => Ok(Some(addr)),
            AttributeValue::DebugAddrIndex(index) => self.address(unit, index).map(Some),
            _ => Ok(None),
        }
    }

    /// Return the range list offset for the given raw offset.
    ///
    /// This handles adding `DW_AT_GNU_ranges_base` if required.
    pub fn ranges_offset_from_raw(
        &self,
        unit: &Unit<R>,
        offset: RawRangeListsOffset<R::Offset>,
    ) -> RangeListsOffset<R::Offset> {
        if self.file_type == DwarfFileType::Dwo && unit.header.version() < 5 {
            RangeListsOffset(offset.0.wrapping_add(unit.rnglists_base.0))
        } else {
            RangeListsOffset(offset.0)
        }
    }

    /// Return the range list offset at the given index.
    pub fn ranges_offset(
        &self,
        unit: &Unit<R>,
        index: DebugRngListsIndex<R::Offset>,
    ) -> Result<RangeListsOffset<R::Offset>> {
        self.ranges
            .get_offset(unit.encoding(), unit.rnglists_base, index)
    }

    /// Iterate over the `RangeListEntry`s starting at the given offset.
    pub fn ranges(
        &self,
        unit: &Unit<R>,
        offset: RangeListsOffset<R::Offset>,
    ) -> Result<RngListIter<R>> {
        self.ranges.ranges(
            offset,
            unit.encoding(),
            unit.low_pc,
            &self.debug_addr,
            unit.addr_base,
        )
    }

    /// Iterate over the `RawRngListEntry`ies starting at the given offset.
    pub fn raw_ranges(
        &self,
        unit: &Unit<R>,
        offset: RangeListsOffset<R::Offset>,
    ) -> Result<RawRngListIter<R>> {
        self.ranges.raw_ranges(offset, unit.encoding())
    }

    /// Try to return an attribute value as a range list offset.
    ///
    /// If the attribute value is one of:
    ///
    /// - a `DW_FORM_sec_offset` reference to the `.debug_ranges` or `.debug_rnglists` sections
    /// - a `DW_FORM_rnglistx` index into the `.debug_rnglists` entries for the unit
    ///
    /// then return the range list offset of the range list.
    /// Returns `None` for other forms.
    pub fn attr_ranges_offset(
        &self,
        unit: &Unit<R>,
        attr: AttributeValue<R>,
    ) -> Result<Option<RangeListsOffset<R::Offset>>> {
        match attr {
            AttributeValue::RangeListsRef(offset) => {
                Ok(Some(self.ranges_offset_from_raw(unit, offset)))
            }
            AttributeValue::DebugRngListsIndex(index) => self.ranges_offset(unit, index).map(Some),
            _ => Ok(None),
        }
    }

    /// Try to return an attribute value as a range list entry iterator.
    ///
    /// If the attribute value is one of:
    ///
    /// - a `DW_FORM_sec_offset` reference to the `.debug_ranges` or `.debug_rnglists` sections
    /// - a `DW_FORM_rnglistx` index into the `.debug_rnglists` entries for the unit
    ///
    /// then return an iterator over the entries in the range list.
    /// Returns `None` for other forms.
    pub fn attr_ranges(
        &self,
        unit: &Unit<R>,
        attr: AttributeValue<R>,
    ) -> Result<Option<RngListIter<R>>> {
        match self.attr_ranges_offset(unit, attr)? {
            Some(offset) => Ok(Some(self.ranges(unit, offset)?)),
            None => Ok(None),
        }
    }

    /// Return an iterator for the address ranges of a `DebuggingInformationEntry`.
    ///
    /// This uses `DW_AT_low_pc`, `DW_AT_high_pc` and `DW_AT_ranges`.
    pub fn die_ranges(
        &self,
        unit: &Unit<R>,
        entry: &DebuggingInformationEntry<R>,
    ) -> Result<RangeIter<R>> {
        let mut low_pc = None;
        let mut high_pc = None;
        let mut size = None;
        let mut attrs = entry.attrs();
        while let Some(attr) = attrs.next()? {
            match attr.name() {
                constants::DW_AT_low_pc => {
                    low_pc = Some(
                        self.attr_address(unit, attr.value())?
                            .ok_or(Error::UnsupportedAttributeForm)?,
                    );
                }
                constants::DW_AT_high_pc => match attr.value() {
                    AttributeValue::Udata(val) => size = Some(val),
                    attr => {
                        high_pc = Some(
                            self.attr_address(unit, attr)?
                                .ok_or(Error::UnsupportedAttributeForm)?,
                        );
                    }
                },
                constants::DW_AT_ranges => {
                    if let Some(list) = self.attr_ranges(unit, attr.value())? {
                        return Ok(RangeIter(RangeIterInner::List(list)));
                    }
                }
                _ => {}
            }
        }
        let range = low_pc.and_then(|begin| {
            let end = size.map(|size| begin + size).or(high_pc);
            // TODO: perhaps return an error if `end` is `None`
            end.map(|end| Range { begin, end })
        });
        Ok(RangeIter(RangeIterInner::Single(range)))
    }

    /// Return an iterator for the address ranges of a `Unit`.
    ///
    /// This uses `DW_AT_low_pc`, `DW_AT_high_pc` and `DW_AT_ranges` of the
    /// root `DebuggingInformationEntry`.
    pub fn unit_ranges(&self, unit: &Unit<R>) -> Result<RangeIter<R>> {
        let mut cursor = unit.header.entries(&unit.abbreviations);
        cursor.next_dfs()?;
        let root = cursor.current().ok_or(Error::MissingUnitDie)?;
        self.die_ranges(unit, root)
    }

    /// Return the location list offset at the given index.
    pub fn locations_offset(
        &self,
        unit: &Unit<R>,
        index: DebugLocListsIndex<R::Offset>,
    ) -> Result<LocationListsOffset<R::Offset>> {
        self.locations
            .get_offset(unit.encoding(), unit.loclists_base, index)
    }

    /// Iterate over the `LocationListEntry`s starting at the given offset.
    pub fn locations(
        &self,
        unit: &Unit<R>,
        offset: LocationListsOffset<R::Offset>,
    ) -> Result<LocListIter<R>> {
        match self.file_type {
            DwarfFileType::Main => self.locations.locations(
                offset,
                unit.encoding(),
                unit.low_pc,
                &self.debug_addr,
                unit.addr_base,
            ),
            DwarfFileType::Dwo => self.locations.locations_dwo(
                offset,
                unit.encoding(),
                unit.low_pc,
                &self.debug_addr,
                unit.addr_base,
            ),
        }
    }

    /// Iterate over the raw `LocationListEntry`s starting at the given offset.
    pub fn raw_locations(
        &self,
        unit: &Unit<R>,
        offset: LocationListsOffset<R::Offset>,
    ) -> Result<RawLocListIter<R>> {
        match self.file_type {
            DwarfFileType::Main => self.locations.raw_locations(offset, unit.encoding()),
            DwarfFileType::Dwo => self.locations.raw_locations_dwo(offset, unit.encoding()),
        }
    }

    /// Try to return an attribute value as a location list offset.
    ///
    /// If the attribute value is one of:
    ///
    /// - a `DW_FORM_sec_offset` reference to the `.debug_loc` or `.debug_loclists` sections
    /// - a `DW_FORM_loclistx` index into the `.debug_loclists` entries for the unit
    ///
    /// then return the location list offset of the location list.
    /// Returns `None` for other forms.
    pub fn attr_locations_offset(
        &self,
        unit: &Unit<R>,
        attr: AttributeValue<R>,
    ) -> Result<Option<LocationListsOffset<R::Offset>>> {
        match attr {
            AttributeValue::LocationListsRef(offset) => Ok(Some(offset)),
            AttributeValue::DebugLocListsIndex(index) => {
                self.locations_offset(unit, index).map(Some)
            }
            _ => Ok(None),
        }
    }

    /// Try to return an attribute value as a location list entry iterator.
    ///
    /// If the attribute value is one of:
    ///
    /// - a `DW_FORM_sec_offset` reference to the `.debug_loc` or `.debug_loclists` sections
    /// - a `DW_FORM_loclistx` index into the `.debug_loclists` entries for the unit
    ///
    /// then return an iterator over the entries in the location list.
    /// Returns `None` for other forms.
    pub fn attr_locations(
        &self,
        unit: &Unit<R>,
        attr: AttributeValue<R>,
    ) -> Result<Option<LocListIter<R>>> {
        match self.attr_locations_offset(unit, attr)? {
            Some(offset) => Ok(Some(self.locations(unit, offset)?)),
            None => Ok(None),
        }
    }

    /// Call `Reader::lookup_offset_id` for each section, and return the first match.
    ///
    /// The first element of the tuple is `true` for supplementary sections.
    pub fn lookup_offset_id(&self, id: ReaderOffsetId) -> Option<(bool, SectionId, R::Offset)> {
        None.or_else(|| self.debug_abbrev.lookup_offset_id(id))
            .or_else(|| self.debug_addr.lookup_offset_id(id))
            .or_else(|| self.debug_aranges.lookup_offset_id(id))
            .or_else(|| self.debug_info.lookup_offset_id(id))
            .or_else(|| self.debug_line.lookup_offset_id(id))
            .or_else(|| self.debug_line_str.lookup_offset_id(id))
            .or_else(|| self.debug_str.lookup_offset_id(id))
            .or_else(|| self.debug_str_offsets.lookup_offset_id(id))
            .or_else(|| self.debug_types.lookup_offset_id(id))
            .or_else(|| self.locations.lookup_offset_id(id))
            .or_else(|| self.ranges.lookup_offset_id(id))
            .map(|(id, offset)| (false, id, offset))
            .or_else(|| {
                self.sup()
                    .and_then(|sup| sup.lookup_offset_id(id))
                    .map(|(_, id, offset)| (true, id, offset))
            })
    }

    /// Returns a string representation of the given error.
    ///
    /// This uses information from the DWARF sections to provide more information in some cases.
    pub fn format_error(&self, err: Error) -> String {
        #[allow(clippy::single_match)]
        match err {
            Error::UnexpectedEof(id) => match self.lookup_offset_id(id) {
                Some((sup, section, offset)) => {
                    return format!(
                        "{} at {}{}+0x{:x}",
                        err,
                        section.name(),
                        if sup { "(sup)" } else { "" },
                        offset.into_u64(),
                    );
                }
                None => {}
            },
            _ => {}
        }
        err.description().into()
    }
}

/// The sections from a `.dwp` file.
#[derive(Debug)]
pub struct DwarfPackage<R: Reader> {
    /// The compilation unit index in the `.debug_cu_index` section.
    pub cu_index: UnitIndex<R>,

    /// The type unit index in the `.debug_tu_index` section.
    pub tu_index: UnitIndex<R>,

    /// The `.debug_abbrev.dwo` section.
    pub debug_abbrev: DebugAbbrev<R>,

    /// The `.debug_info.dwo` section.
    pub debug_info: DebugInfo<R>,

    /// The `.debug_line.dwo` section.
    pub debug_line: DebugLine<R>,

    /// The `.debug_str.dwo` section.
    pub debug_str: DebugStr<R>,

    /// The `.debug_str_offsets.dwo` section.
    pub debug_str_offsets: DebugStrOffsets<R>,

    /// The `.debug_loc.dwo` section.
    ///
    /// Only present when using GNU split-dwarf extension to DWARF 4.
    pub debug_loc: DebugLoc<R>,

    /// The `.debug_loclists.dwo` section.
    pub debug_loclists: DebugLocLists<R>,

    /// The `.debug_rnglists.dwo` section.
    pub debug_rnglists: DebugRngLists<R>,

    /// The `.debug_types.dwo` section.
    ///
    /// Only present when using GNU split-dwarf extension to DWARF 4.
    pub debug_types: DebugTypes<R>,

    /// An empty section.
    ///
    /// Used when creating `Dwarf<R>`.
    pub empty: R,
}

impl<R: Reader> DwarfPackage<R> {
    /// Try to load the `.dwp` sections using the given loader function.
    ///
    /// `section` loads a DWARF section from the object file.
    /// It should return an empty section if the section does not exist.
    pub fn load<F, E>(mut section: F, empty: R) -> core::result::Result<Self, E>
    where
        F: FnMut(SectionId) -> core::result::Result<R, E>,
        E: From<Error>,
    {
        Ok(DwarfPackage {
            cu_index: DebugCuIndex::load(&mut section)?.index()?,
            tu_index: DebugTuIndex::load(&mut section)?.index()?,
            // Section types are inferred.
            debug_abbrev: Section::load(&mut section)?,
            debug_info: Section::load(&mut section)?,
            debug_line: Section::load(&mut section)?,
            debug_str: Section::load(&mut section)?,
            debug_str_offsets: Section::load(&mut section)?,
            debug_loc: Section::load(&mut section)?,
            debug_loclists: Section::load(&mut section)?,
            debug_rnglists: Section::load(&mut section)?,
            debug_types: Section::load(&mut section)?,
            empty,
        })
    }

    /// Find the compilation unit with the given DWO identifier and return its section
    /// contributions.
    pub fn find_cu(&self, id: DwoId, parent: &Dwarf<R>) -> Result<Option<Dwarf<R>>> {
        let row = match self.cu_index.find(id.0) {
            Some(row) => row,
            None => return Ok(None),
        };
        self.cu_sections(row, parent).map(Some)
    }

    /// Find the type unit with the given type signature and return its section
    /// contributions.
    pub fn find_tu(
        &self,
        signature: DebugTypeSignature,
        parent: &Dwarf<R>,
    ) -> Result<Option<Dwarf<R>>> {
        let row = match self.tu_index.find(signature.0) {
            Some(row) => row,
            None => return Ok(None),
        };
        self.tu_sections(row, parent).map(Some)
    }

    /// Return the section contributions of the compilation unit at the given index.
    ///
    /// The index must be in the range `1..cu_index.unit_count`.
    ///
    /// This function should only be needed by low level parsers.
    pub fn cu_sections(&self, index: u32, parent: &Dwarf<R>) -> Result<Dwarf<R>> {
        self.sections(self.cu_index.sections(index)?, parent)
    }

    /// Return the section contributions of the compilation unit at the given index.
    ///
    /// The index must be in the range `1..tu_index.unit_count`.
    ///
    /// This function should only be needed by low level parsers.
    pub fn tu_sections(&self, index: u32, parent: &Dwarf<R>) -> Result<Dwarf<R>> {
        self.sections(self.tu_index.sections(index)?, parent)
    }

    /// Return the section contributions of a unit.
    ///
    /// This function should only be needed by low level parsers.
    pub fn sections(
        &self,
        sections: UnitIndexSectionIterator<R>,
        parent: &Dwarf<R>,
    ) -> Result<Dwarf<R>> {
        let mut abbrev_offset = 0;
        let mut abbrev_size = 0;
        let mut info_offset = 0;
        let mut info_size = 0;
        let mut line_offset = 0;
        let mut line_size = 0;
        let mut loc_offset = 0;
        let mut loc_size = 0;
        let mut loclists_offset = 0;
        let mut loclists_size = 0;
        let mut str_offsets_offset = 0;
        let mut str_offsets_size = 0;
        let mut rnglists_offset = 0;
        let mut rnglists_size = 0;
        let mut types_offset = 0;
        let mut types_size = 0;
        for section in sections {
            match section.section {
                SectionId::DebugAbbrev => {
                    abbrev_offset = section.offset;
                    abbrev_size = section.size;
                }
                SectionId::DebugInfo => {
                    info_offset = section.offset;
                    info_size = section.size;
                }
                SectionId::DebugLine => {
                    line_offset = section.offset;
                    line_size = section.size;
                }
                SectionId::DebugLoc => {
                    loc_offset = section.offset;
                    loc_size = section.size;
                }
                SectionId::DebugLocLists => {
                    loclists_offset = section.offset;
                    loclists_size = section.size;
                }
                SectionId::DebugStrOffsets => {
                    str_offsets_offset = section.offset;
                    str_offsets_size = section.size;
                }
                SectionId::DebugRngLists => {
                    rnglists_offset = section.offset;
                    rnglists_size = section.size;
                }
                SectionId::DebugTypes => {
                    types_offset = section.offset;
                    types_size = section.size;
                }
                SectionId::DebugMacro | SectionId::DebugMacinfo => {
                    // These are valid but we can't parse these yet.
                }
                _ => return Err(Error::UnknownIndexSection),
            }
        }

        let debug_abbrev = self.debug_abbrev.dwp_range(abbrev_offset, abbrev_size)?;
        let debug_info = self.debug_info.dwp_range(info_offset, info_size)?;
        let debug_line = self.debug_line.dwp_range(line_offset, line_size)?;
        let debug_loc = self.debug_loc.dwp_range(loc_offset, loc_size)?;
        let debug_loclists = self
            .debug_loclists
            .dwp_range(loclists_offset, loclists_size)?;
        let debug_str_offsets = self
            .debug_str_offsets
            .dwp_range(str_offsets_offset, str_offsets_size)?;
        let debug_rnglists = self
            .debug_rnglists
            .dwp_range(rnglists_offset, rnglists_size)?;
        let debug_types = self.debug_types.dwp_range(types_offset, types_size)?;

        let debug_str = self.debug_str.clone();

        let debug_addr = parent.debug_addr.clone();
        let debug_ranges = parent.ranges.debug_ranges().clone();

        let debug_aranges = self.empty.clone().into();
        let debug_line_str = self.empty.clone().into();

        Ok(Dwarf {
            debug_abbrev,
            debug_addr,
            debug_aranges,
            debug_info,
            debug_line,
            debug_line_str,
            debug_str,
            debug_str_offsets,
            debug_types,
            locations: LocationLists::new(debug_loc, debug_loclists),
            ranges: RangeLists::new(debug_ranges, debug_rnglists),
            file_type: DwarfFileType::Dwo,
            sup: None,
        })
    }
}

/// All of the commonly used information for a unit in the `.debug_info` or `.debug_types`
/// sections.
#[derive(Debug)]
pub struct Unit<R, Offset = <R as Reader>::Offset>
where
    R: Reader<Offset = Offset>,
    Offset: ReaderOffset,
{
    /// The header of the unit.
    pub header: UnitHeader<R, Offset>,

    /// The parsed abbreviations for the unit.
    pub abbreviations: Abbreviations,

    /// The `DW_AT_name` attribute of the unit.
    pub name: Option<R>,

    /// The `DW_AT_comp_dir` attribute of the unit.
    pub comp_dir: Option<R>,

    /// The `DW_AT_low_pc` attribute of the unit. Defaults to 0.
    pub low_pc: u64,

    /// The `DW_AT_str_offsets_base` attribute of the unit. Defaults to 0.
    pub str_offsets_base: DebugStrOffsetsBase<Offset>,

    /// The `DW_AT_addr_base` attribute of the unit. Defaults to 0.
    pub addr_base: DebugAddrBase<Offset>,

    /// The `DW_AT_loclists_base` attribute of the unit. Defaults to 0.
    pub loclists_base: DebugLocListsBase<Offset>,

    /// The `DW_AT_rnglists_base` attribute of the unit. Defaults to 0.
    pub rnglists_base: DebugRngListsBase<Offset>,

    /// The line number program of the unit.
    pub line_program: Option<IncompleteLineProgram<R, Offset>>,

    /// The DWO ID of a skeleton unit or split compilation unit.
    pub dwo_id: Option<DwoId>,
}

impl<R: Reader> Unit<R> {
    /// Construct a new `Unit` from the given unit header.
    #[inline]
    pub fn new(dwarf: &Dwarf<R>, header: UnitHeader<R>) -> Result<Self> {
        let abbreviations = header.abbreviations(&dwarf.debug_abbrev)?;
        let mut unit = Unit {
            abbreviations,
            name: None,
            comp_dir: None,
            low_pc: 0,
            str_offsets_base: DebugStrOffsetsBase::default_for_encoding_and_file(
                header.encoding(),
                dwarf.file_type,
            ),
            // NB: Because the .debug_addr section never lives in a .dwo, we can assume its base is always 0 or provided.
            addr_base: DebugAddrBase(R::Offset::from_u8(0)),
            loclists_base: DebugLocListsBase::default_for_encoding_and_file(
                header.encoding(),
                dwarf.file_type,
            ),
            rnglists_base: DebugRngListsBase::default_for_encoding_and_file(
                header.encoding(),
                dwarf.file_type,
            ),
            line_program: None,
            dwo_id: match header.type_() {
                UnitType::Skeleton(dwo_id) | UnitType::SplitCompilation(dwo_id) => Some(dwo_id),
                _ => None,
            },
            header,
        };
        let mut name = None;
        let mut comp_dir = None;
        let mut line_program_offset = None;
        let mut low_pc_attr = None;

        {
            let mut cursor = unit.header.entries(&unit.abbreviations);
            cursor.next_dfs()?;
            let root = cursor.current().ok_or(Error::MissingUnitDie)?;
            let mut attrs = root.attrs();
            while let Some(attr) = attrs.next()? {
                match attr.name() {
                    constants::DW_AT_name => {
                        name = Some(attr.value());
                    }
                    constants::DW_AT_comp_dir => {
                        comp_dir = Some(attr.value());
                    }
                    constants::DW_AT_low_pc => {
                        low_pc_attr = Some(attr.value());
                    }
                    constants::DW_AT_stmt_list => {
                        if let AttributeValue::DebugLineRef(offset) = attr.value() {
                            line_program_offset = Some(offset);
                        }
                    }
                    constants::DW_AT_str_offsets_base => {
                        if let AttributeValue::DebugStrOffsetsBase(base) = attr.value() {
                            unit.str_offsets_base = base;
                        }
                    }
                    constants::DW_AT_addr_base | constants::DW_AT_GNU_addr_base => {
                        if let AttributeValue::DebugAddrBase(base) = attr.value() {
                            unit.addr_base = base;
                        }
                    }
                    constants::DW_AT_loclists_base => {
                        if let AttributeValue::DebugLocListsBase(base) = attr.value() {
                            unit.loclists_base = base;
                        }
                    }
                    constants::DW_AT_rnglists_base | constants::DW_AT_GNU_ranges_base => {
                        if let AttributeValue::DebugRngListsBase(base) = attr.value() {
                            unit.rnglists_base = base;
                        }
                    }
                    constants::DW_AT_GNU_dwo_id => {
                        if unit.dwo_id.is_none() {
                            if let AttributeValue::DwoId(dwo_id) = attr.value() {
                                unit.dwo_id = Some(dwo_id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        unit.name = match name {
            Some(val) => dwarf.attr_string(&unit, val).ok(),
            None => None,
        };
        unit.comp_dir = match comp_dir {
            Some(val) => dwarf.attr_string(&unit, val).ok(),
            None => None,
        };
        unit.line_program = match line_program_offset {
            Some(offset) => Some(dwarf.debug_line.program(
                offset,
                unit.header.address_size(),
                unit.comp_dir.clone(),
                unit.name.clone(),
            )?),
            None => None,
        };
        if let Some(low_pc_attr) = low_pc_attr {
            if let Some(addr) = dwarf.attr_address(&unit, low_pc_attr)? {
                unit.low_pc = addr;
            }
        }
        Ok(unit)
    }

    /// Return the encoding parameters for this unit.
    #[inline]
    pub fn encoding(&self) -> Encoding {
        self.header.encoding()
    }

    /// Read the `DebuggingInformationEntry` at the given offset.
    pub fn entry(&self, offset: UnitOffset<R::Offset>) -> Result<DebuggingInformationEntry<R>> {
        self.header.entry(&self.abbreviations, offset)
    }

    /// Navigate this unit's `DebuggingInformationEntry`s.
    #[inline]
    pub fn entries(&self) -> EntriesCursor<R> {
        self.header.entries(&self.abbreviations)
    }

    /// Navigate this unit's `DebuggingInformationEntry`s
    /// starting at the given offset.
    #[inline]
    pub fn entries_at_offset(&self, offset: UnitOffset<R::Offset>) -> Result<EntriesCursor<R>> {
        self.header.entries_at_offset(&self.abbreviations, offset)
    }

    /// Navigate this unit's `DebuggingInformationEntry`s as a tree
    /// starting at the given offset.
    #[inline]
    pub fn entries_tree(&self, offset: Option<UnitOffset<R::Offset>>) -> Result<EntriesTree<R>> {
        self.header.entries_tree(&self.abbreviations, offset)
    }

    /// Read the raw data that defines the Debugging Information Entries.
    #[inline]
    pub fn entries_raw(&self, offset: Option<UnitOffset<R::Offset>>) -> Result<EntriesRaw<R>> {
        self.header.entries_raw(&self.abbreviations, offset)
    }

    /// Copy attributes that are subject to relocation from another unit. This is intended
    /// to be used to copy attributes from a skeleton compilation unit to the corresponding
    /// split compilation unit.
    pub fn copy_relocated_attributes(&mut self, other: &Unit<R>) {
        self.low_pc = other.low_pc;
        self.addr_base = other.addr_base;
        if self.header.version() < 5 {
            self.rnglists_base = other.rnglists_base;
        }
    }
}

impl<T: ReaderOffset> UnitSectionOffset<T> {
    /// Convert an offset to be relative to the start of the given unit,
    /// instead of relative to the start of the section.
    /// Returns `None` if the offset is not within the unit entries.
    pub fn to_unit_offset<R>(&self, unit: &Unit<R>) -> Option<UnitOffset<T>>
    where
        R: Reader<Offset = T>,
    {
        let (offset, unit_offset) = match (self, unit.header.offset()) {
            (
                UnitSectionOffset::DebugInfoOffset(offset),
                UnitSectionOffset::DebugInfoOffset(unit_offset),
            ) => (offset.0, unit_offset.0),
            (
                UnitSectionOffset::DebugTypesOffset(offset),
                UnitSectionOffset::DebugTypesOffset(unit_offset),
            ) => (offset.0, unit_offset.0),
            _ => return None,
        };
        let offset = match offset.checked_sub(unit_offset) {
            Some(offset) => UnitOffset(offset),
            None => return None,
        };
        if !unit.header.is_valid_offset(offset) {
            return None;
        }
        Some(offset)
    }
}

impl<T: ReaderOffset> UnitOffset<T> {
    /// Convert an offset to be relative to the start of the .debug_info section,
    /// instead of relative to the start of the given compilation unit.
    ///
    /// Does not check that the offset is valid.
    pub fn to_unit_section_offset<R>(&self, unit: &Unit<R>) -> UnitSectionOffset<T>
    where
        R: Reader<Offset = T>,
    {
        match unit.header.offset() {
            UnitSectionOffset::DebugInfoOffset(unit_offset) => {
                DebugInfoOffset(unit_offset.0 + self.0).into()
            }
            UnitSectionOffset::DebugTypesOffset(unit_offset) => {
                DebugTypesOffset(unit_offset.0 + self.0).into()
            }
        }
    }
}

/// An iterator for the address ranges of a `DebuggingInformationEntry`.
///
/// Returned by `Dwarf::die_ranges` and `Dwarf::unit_ranges`.
#[derive(Debug)]
pub struct RangeIter<R: Reader>(RangeIterInner<R>);

#[derive(Debug)]
enum RangeIterInner<R: Reader> {
    Single(Option<Range>),
    List(RngListIter<R>),
}

impl<R: Reader> Default for RangeIter<R> {
    fn default() -> Self {
        RangeIter(RangeIterInner::Single(None))
    }
}

impl<R: Reader> RangeIter<R> {
    /// Advance the iterator to the next range.
    pub fn next(&mut self) -> Result<Option<Range>> {
        match self.0 {
            RangeIterInner::Single(ref mut range) => Ok(range.take()),
            RangeIterInner::List(ref mut list) => list.next(),
        }
    }
}

#[cfg(feature = "fallible-iterator")]
impl<R: Reader> fallible_iterator::FallibleIterator for RangeIter<R> {
    type Item = Range;
    type Error = Error;

    #[inline]
    fn next(&mut self) -> ::core::result::Result<Option<Self::Item>, Self::Error> {
        RangeIter::next(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::EndianSlice;
    use crate::{Endianity, LittleEndian};

    /// Ensure that `Dwarf<R>` is covariant wrt R.
    #[test]
    fn test_dwarf_variance() {
        /// This only needs to compile.
        fn _f<'a: 'b, 'b, E: Endianity>(x: Dwarf<EndianSlice<'a, E>>) -> Dwarf<EndianSlice<'b, E>> {
            x
        }
    }

    /// Ensure that `Unit<R>` is covariant wrt R.
    #[test]
    fn test_dwarf_unit_variance() {
        /// This only needs to compile.
        fn _f<'a: 'b, 'b, E: Endianity>(x: Unit<EndianSlice<'a, E>>) -> Unit<EndianSlice<'b, E>> {
            x
        }
    }

    #[test]
    fn test_send() {
        fn assert_is_send<T: Send>() {}
        assert_is_send::<Dwarf<EndianSlice<LittleEndian>>>();
        assert_is_send::<Unit<EndianSlice<LittleEndian>>>();
    }

    #[test]
    fn test_format_error() {
        let mut owned_dwarf = Dwarf::load(|_| -> Result<_> { Ok(vec![1, 2]) }).unwrap();
        owned_dwarf
            .load_sup(|_| -> Result<_> { Ok(vec![1, 2]) })
            .unwrap();
        let dwarf = owned_dwarf.borrow(|section| EndianSlice::new(&section, LittleEndian));

        match dwarf.debug_str.get_str(DebugStrOffset(1)) {
            Ok(r) => panic!("Unexpected str {:?}", r),
            Err(e) => {
                assert_eq!(
                    dwarf.format_error(e),
                    "Hit the end of input before it was expected at .debug_str+0x1"
                );
            }
        }
        match dwarf.sup().unwrap().debug_str.get_str(DebugStrOffset(1)) {
            Ok(r) => panic!("Unexpected str {:?}", r),
            Err(e) => {
                assert_eq!(
                    dwarf.format_error(e),
                    "Hit the end of input before it was expected at .debug_str(sup)+0x1"
                );
            }
        }
        assert_eq!(dwarf.format_error(Error::Io), Error::Io.description());
    }
}
