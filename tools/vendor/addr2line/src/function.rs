use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::iter;

use crate::lazy::LazyCell;
use crate::maybe_small;
use crate::{Error, RangeAttributes, ResDwarf};

pub(crate) struct Functions<R: gimli::Reader> {
    /// List of all `DW_TAG_subprogram` details in the unit.
    pub(crate) functions: Box<
        [(
            gimli::UnitOffset<R::Offset>,
            LazyCell<Result<Function<R>, Error>>,
        )],
    >,
    /// List of `DW_TAG_subprogram` address ranges in the unit.
    pub(crate) addresses: Box<[FunctionAddress]>,
}

/// A single address range for a function.
///
/// It is possible for a function to have multiple address ranges; this
/// is handled by having multiple `FunctionAddress` entries with the same
/// `function` field.
pub(crate) struct FunctionAddress {
    range: gimli::Range,
    /// An index into `Functions::functions`.
    pub(crate) function: usize,
}

pub(crate) struct Function<R: gimli::Reader> {
    pub(crate) dw_die_offset: gimli::UnitOffset<R::Offset>,
    pub(crate) name: Option<R>,
    /// List of all `DW_TAG_inlined_subroutine` details in this function.
    inlined_functions: Box<[InlinedFunction<R>]>,
    /// List of `DW_TAG_inlined_subroutine` address ranges in this function.
    inlined_addresses: Box<[InlinedFunctionAddress]>,
}

pub(crate) struct InlinedFunctionAddress {
    range: gimli::Range,
    call_depth: usize,
    /// An index into `Function::inlined_functions`.
    function: usize,
}

pub(crate) struct InlinedFunction<R: gimli::Reader> {
    pub(crate) dw_die_offset: gimli::UnitOffset<R::Offset>,
    pub(crate) name: Option<R>,
    pub(crate) call_file: u64,
    pub(crate) call_line: u32,
    pub(crate) call_column: u32,
}

impl<R: gimli::Reader> Functions<R> {
    pub(crate) fn parse(unit: &gimli::Unit<R>, dwarf: &ResDwarf<R>) -> Result<Functions<R>, Error> {
        let mut functions = Vec::new();
        let mut addresses = Vec::new();
        let mut entries = unit.entries_raw(None)?;
        while !entries.is_empty() {
            let dw_die_offset = entries.next_offset();
            if let Some(abbrev) = entries.read_abbreviation()? {
                if abbrev.tag() == gimli::DW_TAG_subprogram {
                    let mut ranges = RangeAttributes::default();
                    for spec in abbrev.attributes() {
                        match entries.read_attribute(*spec) {
                            Ok(ref attr) => {
                                match attr.name() {
                                    gimli::DW_AT_low_pc => match attr.value() {
                                        gimli::AttributeValue::Addr(val) => {
                                            ranges.low_pc = Some(val)
                                        }
                                        gimli::AttributeValue::DebugAddrIndex(index) => {
                                            ranges.low_pc =
                                                Some(dwarf.sections.address(unit, index)?);
                                        }
                                        _ => {}
                                    },
                                    gimli::DW_AT_high_pc => match attr.value() {
                                        gimli::AttributeValue::Addr(val) => {
                                            ranges.high_pc = Some(val)
                                        }
                                        gimli::AttributeValue::DebugAddrIndex(index) => {
                                            ranges.high_pc =
                                                Some(dwarf.sections.address(unit, index)?);
                                        }
                                        gimli::AttributeValue::Udata(val) => {
                                            ranges.size = Some(val)
                                        }
                                        _ => {}
                                    },
                                    gimli::DW_AT_ranges => {
                                        ranges.ranges_offset = dwarf
                                            .sections
                                            .attr_ranges_offset(unit, attr.value())?;
                                    }
                                    _ => {}
                                };
                            }
                            Err(e) => return Err(e),
                        }
                    }

                    let function_index = functions.len();
                    if ranges.for_each_range(&dwarf.sections, unit, |range| {
                        addresses.push(FunctionAddress {
                            range,
                            function: function_index,
                        });
                    })? {
                        functions.push((dw_die_offset, LazyCell::new()));
                    }
                } else {
                    entries.skip_attributes(abbrev.attributes())?;
                }
            }
        }

        // The binary search requires the addresses to be sorted.
        //
        // It also requires them to be non-overlapping.  In practice, overlapping
        // function ranges are unlikely, so we don't try to handle that yet.
        //
        // It's possible for multiple functions to have the same address range if the
        // compiler can detect and remove functions with identical code.  In that case
        // we'll nondeterministically return one of them.
        addresses.sort_by_key(|x| x.range.begin);

        Ok(Functions {
            functions: functions.into_boxed_slice(),
            addresses: addresses.into_boxed_slice(),
        })
    }

    pub(crate) fn find_address(&self, probe: u64) -> Option<usize> {
        self.addresses
            .binary_search_by(|address| {
                if probe < address.range.begin {
                    Ordering::Greater
                } else if probe >= address.range.end {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .ok()
    }

    pub(crate) fn parse_inlined_functions(
        &self,
        unit: &gimli::Unit<R>,
        dwarf: &ResDwarf<R>,
    ) -> Result<(), Error> {
        for function in &*self.functions {
            function
                .1
                .borrow_with(|| Function::parse(function.0, unit, dwarf))
                .as_ref()
                .map_err(Error::clone)?;
        }
        Ok(())
    }
}

impl<R: gimli::Reader> Function<R> {
    pub(crate) fn parse(
        dw_die_offset: gimli::UnitOffset<R::Offset>,
        unit: &gimli::Unit<R>,
        dwarf: &ResDwarf<R>,
    ) -> Result<Self, Error> {
        let mut entries = unit.entries_raw(Some(dw_die_offset))?;
        let depth = entries.next_depth();
        let abbrev = entries.read_abbreviation()?.unwrap();
        debug_assert_eq!(abbrev.tag(), gimli::DW_TAG_subprogram);

        let mut name = None;
        for spec in abbrev.attributes() {
            match entries.read_attribute(*spec) {
                Ok(ref attr) => {
                    match attr.name() {
                        gimli::DW_AT_linkage_name | gimli::DW_AT_MIPS_linkage_name => {
                            if let Ok(val) = dwarf.sections.attr_string(unit, attr.value()) {
                                name = Some(val);
                            }
                        }
                        gimli::DW_AT_name => {
                            if name.is_none() {
                                name = dwarf.sections.attr_string(unit, attr.value()).ok();
                            }
                        }
                        gimli::DW_AT_abstract_origin | gimli::DW_AT_specification => {
                            if name.is_none() {
                                name = name_attr(attr.value(), unit, dwarf, 16)?;
                            }
                        }
                        _ => {}
                    };
                }
                Err(e) => return Err(e),
            }
        }

        let mut inlined_functions = Vec::new();
        let mut inlined_addresses = Vec::new();
        Function::parse_children(
            &mut entries,
            depth,
            unit,
            dwarf,
            &mut inlined_functions,
            &mut inlined_addresses,
            0,
        )?;

        // Sort ranges in "breadth-first traversal order", i.e. first by call_depth
        // and then by range.begin. This allows finding the range containing an
        // address at a certain depth using binary search.
        // Note: Using DFS order, i.e. ordering by range.begin first and then by
        // call_depth, would not work! Consider the two examples
        // "[0..10 at depth 0], [0..2 at depth 1], [6..8 at depth 1]"  and
        // "[0..5 at depth 0], [0..2 at depth 1], [5..10 at depth 0], [6..8 at depth 1]".
        // In this example, if you want to look up address 7 at depth 0, and you
        // encounter [0..2 at depth 1], are you before or after the target range?
        // You don't know.
        inlined_addresses.sort_by(|r1, r2| {
            if r1.call_depth < r2.call_depth {
                Ordering::Less
            } else if r1.call_depth > r2.call_depth {
                Ordering::Greater
            } else if r1.range.begin < r2.range.begin {
                Ordering::Less
            } else if r1.range.begin > r2.range.begin {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        Ok(Function {
            dw_die_offset,
            name,
            inlined_functions: inlined_functions.into_boxed_slice(),
            inlined_addresses: inlined_addresses.into_boxed_slice(),
        })
    }

    fn parse_children(
        entries: &mut gimli::EntriesRaw<R>,
        depth: isize,
        unit: &gimli::Unit<R>,
        dwarf: &ResDwarf<R>,
        inlined_functions: &mut Vec<InlinedFunction<R>>,
        inlined_addresses: &mut Vec<InlinedFunctionAddress>,
        inlined_depth: usize,
    ) -> Result<(), Error> {
        loop {
            let dw_die_offset = entries.next_offset();
            let next_depth = entries.next_depth();
            if next_depth <= depth {
                return Ok(());
            }
            if let Some(abbrev) = entries.read_abbreviation()? {
                match abbrev.tag() {
                    gimli::DW_TAG_subprogram => {
                        Function::skip(entries, abbrev, next_depth)?;
                    }
                    gimli::DW_TAG_inlined_subroutine => {
                        InlinedFunction::parse(
                            dw_die_offset,
                            entries,
                            abbrev,
                            next_depth,
                            unit,
                            dwarf,
                            inlined_functions,
                            inlined_addresses,
                            inlined_depth,
                        )?;
                    }
                    _ => {
                        entries.skip_attributes(abbrev.attributes())?;
                    }
                }
            }
        }
    }

    fn skip(
        entries: &mut gimli::EntriesRaw<R>,
        abbrev: &gimli::Abbreviation,
        depth: isize,
    ) -> Result<(), Error> {
        // TODO: use DW_AT_sibling
        entries.skip_attributes(abbrev.attributes())?;
        while entries.next_depth() > depth {
            if let Some(abbrev) = entries.read_abbreviation()? {
                entries.skip_attributes(abbrev.attributes())?;
            }
        }
        Ok(())
    }

    /// Build the list of inlined functions that contain `probe`.
    pub(crate) fn find_inlined_functions(
        &self,
        probe: u64,
    ) -> iter::Rev<maybe_small::IntoIter<&InlinedFunction<R>>> {
        // `inlined_functions` is ordered from outside to inside.
        let mut inlined_functions = maybe_small::Vec::new();
        let mut inlined_addresses = &self.inlined_addresses[..];
        loop {
            let current_depth = inlined_functions.len();
            // Look up (probe, current_depth) in inline_ranges.
            // `inlined_addresses` is sorted in "breadth-first traversal order", i.e.
            // by `call_depth` first, and then by `range.begin`. See the comment at
            // the sort call for more information about why.
            let search = inlined_addresses.binary_search_by(|range| {
                if range.call_depth > current_depth {
                    Ordering::Greater
                } else if range.call_depth < current_depth {
                    Ordering::Less
                } else if range.range.begin > probe {
                    Ordering::Greater
                } else if range.range.end <= probe {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            });
            if let Ok(index) = search {
                let function_index = inlined_addresses[index].function;
                inlined_functions.push(&self.inlined_functions[function_index]);
                inlined_addresses = &inlined_addresses[index + 1..];
            } else {
                break;
            }
        }
        inlined_functions.into_iter().rev()
    }
}

impl<R: gimli::Reader> InlinedFunction<R> {
    fn parse(
        dw_die_offset: gimli::UnitOffset<R::Offset>,
        entries: &mut gimli::EntriesRaw<R>,
        abbrev: &gimli::Abbreviation,
        depth: isize,
        unit: &gimli::Unit<R>,
        dwarf: &ResDwarf<R>,
        inlined_functions: &mut Vec<InlinedFunction<R>>,
        inlined_addresses: &mut Vec<InlinedFunctionAddress>,
        inlined_depth: usize,
    ) -> Result<(), Error> {
        let mut ranges = RangeAttributes::default();
        let mut name = None;
        let mut call_file = 0;
        let mut call_line = 0;
        let mut call_column = 0;
        for spec in abbrev.attributes() {
            match entries.read_attribute(*spec) {
                Ok(ref attr) => match attr.name() {
                    gimli::DW_AT_low_pc => match attr.value() {
                        gimli::AttributeValue::Addr(val) => ranges.low_pc = Some(val),
                        gimli::AttributeValue::DebugAddrIndex(index) => {
                            ranges.low_pc = Some(dwarf.sections.address(unit, index)?);
                        }
                        _ => {}
                    },
                    gimli::DW_AT_high_pc => match attr.value() {
                        gimli::AttributeValue::Addr(val) => ranges.high_pc = Some(val),
                        gimli::AttributeValue::DebugAddrIndex(index) => {
                            ranges.high_pc = Some(dwarf.sections.address(unit, index)?);
                        }
                        gimli::AttributeValue::Udata(val) => ranges.size = Some(val),
                        _ => {}
                    },
                    gimli::DW_AT_ranges => {
                        ranges.ranges_offset =
                            dwarf.sections.attr_ranges_offset(unit, attr.value())?;
                    }
                    gimli::DW_AT_linkage_name | gimli::DW_AT_MIPS_linkage_name => {
                        if let Ok(val) = dwarf.sections.attr_string(unit, attr.value()) {
                            name = Some(val);
                        }
                    }
                    gimli::DW_AT_name => {
                        if name.is_none() {
                            name = dwarf.sections.attr_string(unit, attr.value()).ok();
                        }
                    }
                    gimli::DW_AT_abstract_origin | gimli::DW_AT_specification => {
                        if name.is_none() {
                            name = name_attr(attr.value(), unit, dwarf, 16)?;
                        }
                    }
                    gimli::DW_AT_call_file => {
                        if let gimli::AttributeValue::FileIndex(fi) = attr.value() {
                            call_file = fi;
                        }
                    }
                    gimli::DW_AT_call_line => {
                        call_line = attr.udata_value().unwrap_or(0) as u32;
                    }
                    gimli::DW_AT_call_column => {
                        call_column = attr.udata_value().unwrap_or(0) as u32;
                    }
                    _ => {}
                },
                Err(e) => return Err(e),
            }
        }

        let function_index = inlined_functions.len();
        inlined_functions.push(InlinedFunction {
            dw_die_offset,
            name,
            call_file,
            call_line,
            call_column,
        });

        ranges.for_each_range(&dwarf.sections, unit, |range| {
            inlined_addresses.push(InlinedFunctionAddress {
                range,
                call_depth: inlined_depth,
                function: function_index,
            });
        })?;

        Function::parse_children(
            entries,
            depth,
            unit,
            dwarf,
            inlined_functions,
            inlined_addresses,
            inlined_depth + 1,
        )
    }
}

fn name_attr<R>(
    attr: gimli::AttributeValue<R>,
    unit: &gimli::Unit<R>,
    dwarf: &ResDwarf<R>,
    recursion_limit: usize,
) -> Result<Option<R>, Error>
where
    R: gimli::Reader,
{
    if recursion_limit == 0 {
        return Ok(None);
    }

    match attr {
        gimli::AttributeValue::UnitRef(offset) => name_entry(unit, offset, dwarf, recursion_limit),
        gimli::AttributeValue::DebugInfoRef(dr) => {
            let res_unit = dwarf.find_unit(dr)?;
            name_entry(
                &res_unit.dw_unit,
                gimli::UnitOffset(dr.0 - res_unit.offset.0),
                dwarf,
                recursion_limit,
            )
        }
        gimli::AttributeValue::DebugInfoRefSup(dr) => {
            if let Some(sup_dwarf) = dwarf.sup.as_ref() {
                let res_unit = sup_dwarf.find_unit(dr)?;
                name_entry(
                    &res_unit.dw_unit,
                    gimli::UnitOffset(dr.0 - res_unit.offset.0),
                    sup_dwarf,
                    recursion_limit,
                )
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

fn name_entry<R>(
    unit: &gimli::Unit<R>,
    offset: gimli::UnitOffset<R::Offset>,
    dwarf: &ResDwarf<R>,
    recursion_limit: usize,
) -> Result<Option<R>, Error>
where
    R: gimli::Reader,
{
    let mut entries = unit.entries_raw(Some(offset))?;
    let abbrev = if let Some(abbrev) = entries.read_abbreviation()? {
        abbrev
    } else {
        return Err(gimli::Error::NoEntryAtGivenOffset);
    };

    let mut name = None;
    let mut next = None;
    for spec in abbrev.attributes() {
        match entries.read_attribute(*spec) {
            Ok(ref attr) => match attr.name() {
                gimli::DW_AT_linkage_name | gimli::DW_AT_MIPS_linkage_name => {
                    if let Ok(val) = dwarf.sections.attr_string(unit, attr.value()) {
                        return Ok(Some(val));
                    }
                }
                gimli::DW_AT_name => {
                    if let Ok(val) = dwarf.sections.attr_string(unit, attr.value()) {
                        name = Some(val);
                    }
                }
                gimli::DW_AT_abstract_origin | gimli::DW_AT_specification => {
                    next = Some(attr.value());
                }
                _ => {}
            },
            Err(e) => return Err(e),
        }
    }

    if name.is_some() {
        return Ok(name);
    }

    if let Some(next) = next {
        return name_attr(next, unit, dwarf, recursion_limit - 1);
    }

    Ok(None)
}
