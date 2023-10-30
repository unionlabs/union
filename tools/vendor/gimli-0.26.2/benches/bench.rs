#![feature(test)]

extern crate test;

use gimli::{
    AttributeValue, DebugAbbrev, DebugAddr, DebugAddrBase, DebugAranges, DebugInfo, DebugLine,
    DebugLineOffset, DebugLoc, DebugLocLists, DebugPubNames, DebugPubTypes, DebugRanges,
    DebugRngLists, Encoding, EndianSlice, EntriesTreeNode, Expression, LittleEndian, LocationLists,
    Operation, RangeLists, RangeListsOffset, Reader, ReaderOffset,
};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

pub fn read_section(section: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    path.push("./fixtures/self/");
    path.push(section);

    assert!(path.is_file());
    let mut file = File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

#[bench]
fn bench_parsing_debug_abbrev(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);
    let unit = debug_info
        .units()
        .next()
        .expect("Should have at least one compilation unit")
        .expect("And it should parse OK");

    let debug_abbrev = read_section("debug_abbrev");

    b.iter(|| {
        let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);
        test::black_box(
            unit.abbreviations(&debug_abbrev)
                .expect("Should parse abbreviations"),
        );
    });
}

#[inline]
fn impl_bench_parsing_debug_info<R: Reader>(
    debug_info: DebugInfo<R>,
    debug_abbrev: DebugAbbrev<R>,
) {
    let mut iter = debug_info.units();
    while let Some(unit) = iter.next().expect("Should parse compilation unit") {
        let abbrevs = unit
            .abbreviations(&debug_abbrev)
            .expect("Should parse abbreviations");

        let mut cursor = unit.entries(&abbrevs);
        while let Some((_, entry)) = cursor.next_dfs().expect("Should parse next dfs") {
            let mut attrs = entry.attrs();
            loop {
                match attrs.next() {
                    Ok(Some(ref attr)) => {
                        test::black_box(attr);
                    }
                    Ok(None) => break,
                    e @ Err(_) => {
                        e.expect("Should parse entry's attribute");
                    }
                }
            }
        }
    }
}

#[bench]
fn bench_parsing_debug_info(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    b.iter(|| impl_bench_parsing_debug_info(debug_info, debug_abbrev));
}

#[bench]
fn bench_parsing_debug_info_with_endian_rc_slice(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = Rc::from(&debug_info[..]);
    let debug_info = gimli::EndianRcSlice::new(debug_info, LittleEndian);
    let debug_info = DebugInfo::from(debug_info);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = Rc::from(&debug_abbrev[..]);
    let debug_abbrev = gimli::EndianRcSlice::new(debug_abbrev, LittleEndian);
    let debug_abbrev = DebugAbbrev::from(debug_abbrev);

    b.iter(|| impl_bench_parsing_debug_info(debug_info.clone(), debug_abbrev.clone()));
}

#[bench]
fn bench_parsing_debug_info_tree(b: &mut test::Bencher) {
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_info = read_section("debug_info");

    b.iter(|| {
        let debug_info = DebugInfo::new(&debug_info, LittleEndian);

        let mut iter = debug_info.units();
        while let Some(unit) = iter.next().expect("Should parse compilation unit") {
            let abbrevs = unit
                .abbreviations(&debug_abbrev)
                .expect("Should parse abbreviations");

            let mut tree = unit
                .entries_tree(&abbrevs, None)
                .expect("Should have entries tree");
            let root = tree.root().expect("Should parse root entry");
            parse_debug_info_tree(root);
        }
    });
}

fn parse_debug_info_tree<R: Reader>(node: EntriesTreeNode<R>) {
    {
        let mut attrs = node.entry().attrs();
        loop {
            match attrs.next() {
                Ok(Some(ref attr)) => {
                    test::black_box(attr);
                }
                Ok(None) => break,
                e @ Err(_) => {
                    e.expect("Should parse entry's attribute");
                }
            }
        }
    }
    let mut children = node.children();
    while let Some(child) = children.next().expect("Should parse child entry") {
        parse_debug_info_tree(child);
    }
}

#[bench]
fn bench_parsing_debug_info_raw(b: &mut test::Bencher) {
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_info = read_section("debug_info");

    b.iter(|| {
        let debug_info = DebugInfo::new(&debug_info, LittleEndian);

        let mut iter = debug_info.units();
        while let Some(unit) = iter.next().expect("Should parse compilation unit") {
            let abbrevs = unit
                .abbreviations(&debug_abbrev)
                .expect("Should parse abbreviations");

            let mut raw = unit
                .entries_raw(&abbrevs, None)
                .expect("Should have entries");
            while !raw.is_empty() {
                if let Some(abbrev) = raw
                    .read_abbreviation()
                    .expect("Should parse abbreviation code")
                {
                    for spec in abbrev.attributes().iter().cloned() {
                        match raw.read_attribute(spec) {
                            Ok(ref attr) => {
                                test::black_box(attr);
                            }
                            e @ Err(_) => {
                                e.expect("Should parse attribute");
                            }
                        }
                    }
                }
            }
        }
    });
}

#[bench]
fn bench_parsing_debug_aranges(b: &mut test::Bencher) {
    let debug_aranges = read_section("debug_aranges");
    let debug_aranges = DebugAranges::new(&debug_aranges, LittleEndian);

    b.iter(|| {
        let mut headers = debug_aranges.headers();
        while let Some(header) = headers.next().expect("Should parse arange header OK") {
            let mut entries = header.entries();
            while let Some(arange) = entries.next().expect("Should parse arange entry OK") {
                test::black_box(arange);
            }
        }
    });
}

#[bench]
fn bench_parsing_debug_pubnames(b: &mut test::Bencher) {
    let debug_pubnames = read_section("debug_pubnames");
    let debug_pubnames = DebugPubNames::new(&debug_pubnames, LittleEndian);

    b.iter(|| {
        let mut pubnames = debug_pubnames.items();
        while let Some(pubname) = pubnames.next().expect("Should parse pubname OK") {
            test::black_box(pubname);
        }
    });
}

#[bench]
fn bench_parsing_debug_pubtypes(b: &mut test::Bencher) {
    let debug_pubtypes = read_section("debug_pubtypes");
    let debug_pubtypes = DebugPubTypes::new(&debug_pubtypes, LittleEndian);

    b.iter(|| {
        let mut pubtypes = debug_pubtypes.items();
        while let Some(pubtype) = pubtypes.next().expect("Should parse pubtype OK") {
            test::black_box(pubtype);
        }
    });
}

// We happen to know that there is a line number program and header at
// offset 0 and that address size is 8 bytes. No need to parse DIEs to grab
// this info off of the compilation units.
const OFFSET: DebugLineOffset = DebugLineOffset(0);
const ADDRESS_SIZE: u8 = 8;

#[bench]
fn bench_parsing_line_number_program_opcodes(b: &mut test::Bencher) {
    let debug_line = read_section("debug_line");
    let debug_line = DebugLine::new(&debug_line, LittleEndian);

    b.iter(|| {
        let program = debug_line
            .program(OFFSET, ADDRESS_SIZE, None, None)
            .expect("Should parse line number program header");
        let header = program.header();

        let mut instructions = header.instructions();
        while let Some(instruction) = instructions
            .next_instruction(header)
            .expect("Should parse instruction")
        {
            test::black_box(instruction);
        }
    });
}

#[bench]
fn bench_executing_line_number_programs(b: &mut test::Bencher) {
    let debug_line = read_section("debug_line");
    let debug_line = DebugLine::new(&debug_line, LittleEndian);

    b.iter(|| {
        let program = debug_line
            .program(OFFSET, ADDRESS_SIZE, None, None)
            .expect("Should parse line number program header");

        let mut rows = program.rows();
        while let Some(row) = rows
            .next_row()
            .expect("Should parse and execute all rows in the line number program")
        {
            test::black_box(row);
        }
    });
}

#[bench]
fn bench_parsing_debug_loc(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_addr = DebugAddr::from(EndianSlice::new(&[], LittleEndian));
    let debug_addr_base = DebugAddrBase(0);

    let debug_loc = read_section("debug_loc");
    let debug_loc = DebugLoc::new(&debug_loc, LittleEndian);
    let debug_loclists = DebugLocLists::new(&[], LittleEndian);
    let loclists = LocationLists::new(debug_loc, debug_loclists);

    let mut offsets = Vec::new();

    let mut iter = debug_info.units();
    while let Some(unit) = iter.next().expect("Should parse compilation unit") {
        let abbrevs = unit
            .abbreviations(&debug_abbrev)
            .expect("Should parse abbreviations");

        let mut cursor = unit.entries(&abbrevs);
        cursor.next_dfs().expect("Should parse next dfs");

        let mut low_pc = 0;

        {
            let unit_entry = cursor.current().expect("Should have a root entry");
            let low_pc_attr = unit_entry
                .attr_value(gimli::DW_AT_low_pc)
                .expect("Should parse low_pc");
            if let Some(gimli::AttributeValue::Addr(address)) = low_pc_attr {
                low_pc = address;
            }
        }

        while cursor.next_dfs().expect("Should parse next dfs").is_some() {
            let entry = cursor.current().expect("Should have a current entry");
            let mut attrs = entry.attrs();
            while let Some(attr) = attrs.next().expect("Should parse entry's attribute") {
                if let gimli::AttributeValue::LocationListsRef(offset) = attr.value() {
                    offsets.push((offset, unit.encoding(), low_pc));
                }
            }
        }
    }

    b.iter(|| {
        for &(offset, encoding, base_address) in &*offsets {
            let mut locs = loclists
                .locations(offset, encoding, base_address, &debug_addr, debug_addr_base)
                .expect("Should parse locations OK");
            while let Some(loc) = locs.next().expect("Should parse next location") {
                test::black_box(loc);
            }
        }
    });
}

#[bench]
fn bench_parsing_debug_ranges(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_addr = DebugAddr::from(EndianSlice::new(&[], LittleEndian));
    let debug_addr_base = DebugAddrBase(0);

    let debug_ranges = read_section("debug_ranges");
    let debug_ranges = DebugRanges::new(&debug_ranges, LittleEndian);
    let debug_rnglists = DebugRngLists::new(&[], LittleEndian);
    let rnglists = RangeLists::new(debug_ranges, debug_rnglists);

    let mut offsets = Vec::new();

    let mut iter = debug_info.units();
    while let Some(unit) = iter.next().expect("Should parse compilation unit") {
        let abbrevs = unit
            .abbreviations(&debug_abbrev)
            .expect("Should parse abbreviations");

        let mut cursor = unit.entries(&abbrevs);
        cursor.next_dfs().expect("Should parse next dfs");

        let mut low_pc = 0;

        {
            let unit_entry = cursor.current().expect("Should have a root entry");
            let low_pc_attr = unit_entry
                .attr_value(gimli::DW_AT_low_pc)
                .expect("Should parse low_pc");
            if let Some(gimli::AttributeValue::Addr(address)) = low_pc_attr {
                low_pc = address;
            }
        }

        while cursor.next_dfs().expect("Should parse next dfs").is_some() {
            let entry = cursor.current().expect("Should have a current entry");
            let mut attrs = entry.attrs();
            while let Some(attr) = attrs.next().expect("Should parse entry's attribute") {
                if let gimli::AttributeValue::RangeListsRef(offset) = attr.value() {
                    offsets.push((RangeListsOffset(offset.0), unit.encoding(), low_pc));
                }
            }
        }
    }

    b.iter(|| {
        for &(offset, encoding, base_address) in &*offsets {
            let mut ranges = rnglists
                .ranges(offset, encoding, base_address, &debug_addr, debug_addr_base)
                .expect("Should parse ranges OK");
            while let Some(range) = ranges.next().expect("Should parse next range") {
                test::black_box(range);
            }
        }
    });
}

fn debug_info_expressions<R: Reader>(
    debug_info: &DebugInfo<R>,
    debug_abbrev: &DebugAbbrev<R>,
) -> Vec<(Expression<R>, Encoding)> {
    let mut expressions = Vec::new();

    let mut iter = debug_info.units();
    while let Some(unit) = iter.next().expect("Should parse compilation unit") {
        let abbrevs = unit
            .abbreviations(debug_abbrev)
            .expect("Should parse abbreviations");

        let mut cursor = unit.entries(&abbrevs);
        while let Some((_, entry)) = cursor.next_dfs().expect("Should parse next dfs") {
            let mut attrs = entry.attrs();
            while let Some(attr) = attrs.next().expect("Should parse entry's attribute") {
                if let AttributeValue::Exprloc(expression) = attr.value() {
                    expressions.push((expression, unit.encoding()));
                }
            }
        }
    }

    expressions
}

#[bench]
fn bench_parsing_debug_info_expressions(b: &mut test::Bencher) {
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let expressions = debug_info_expressions(&debug_info, &debug_abbrev);

    b.iter(|| {
        for &(expression, encoding) in &*expressions {
            let mut pc = expression.0;
            while !pc.is_empty() {
                Operation::parse(&mut pc, encoding).expect("Should parse operation");
            }
        }
    });
}

#[bench]
fn bench_evaluating_debug_info_expressions(b: &mut test::Bencher) {
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let expressions = debug_info_expressions(&debug_info, &debug_abbrev);

    b.iter(|| {
        for &(expression, encoding) in &*expressions {
            let mut eval = expression.evaluation(encoding);
            eval.set_initial_value(0);
            let result = eval.evaluate().expect("Should evaluate expression");
            test::black_box(result);
        }
    });
}

fn debug_loc_expressions<R: Reader>(
    debug_info: &DebugInfo<R>,
    debug_abbrev: &DebugAbbrev<R>,
    debug_addr: &DebugAddr<R>,
    loclists: &LocationLists<R>,
) -> Vec<(Expression<R>, Encoding)> {
    let debug_addr_base = DebugAddrBase(R::Offset::from_u8(0));

    let mut expressions = Vec::new();

    let mut iter = debug_info.units();
    while let Some(unit) = iter.next().expect("Should parse compilation unit") {
        let abbrevs = unit
            .abbreviations(debug_abbrev)
            .expect("Should parse abbreviations");

        let mut cursor = unit.entries(&abbrevs);
        cursor.next_dfs().expect("Should parse next dfs");

        let mut low_pc = 0;

        {
            let unit_entry = cursor.current().expect("Should have a root entry");
            let low_pc_attr = unit_entry
                .attr_value(gimli::DW_AT_low_pc)
                .expect("Should parse low_pc");
            if let Some(gimli::AttributeValue::Addr(address)) = low_pc_attr {
                low_pc = address;
            }
        }

        while cursor.next_dfs().expect("Should parse next dfs").is_some() {
            let entry = cursor.current().expect("Should have a current entry");
            let mut attrs = entry.attrs();
            while let Some(attr) = attrs.next().expect("Should parse entry's attribute") {
                if let gimli::AttributeValue::LocationListsRef(offset) = attr.value() {
                    let mut locs = loclists
                        .locations(offset, unit.encoding(), low_pc, debug_addr, debug_addr_base)
                        .expect("Should parse locations OK");
                    while let Some(loc) = locs.next().expect("Should parse next location") {
                        expressions.push((loc.data, unit.encoding()));
                    }
                }
            }
        }
    }

    expressions
}

#[bench]
fn bench_parsing_debug_loc_expressions(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_addr = DebugAddr::from(EndianSlice::new(&[], LittleEndian));

    let debug_loc = read_section("debug_loc");
    let debug_loc = DebugLoc::new(&debug_loc, LittleEndian);
    let debug_loclists = DebugLocLists::new(&[], LittleEndian);
    let loclists = LocationLists::new(debug_loc, debug_loclists);

    let expressions = debug_loc_expressions(&debug_info, &debug_abbrev, &debug_addr, &loclists);

    b.iter(|| {
        for &(expression, encoding) in &*expressions {
            let mut pc = expression.0;
            while !pc.is_empty() {
                Operation::parse(&mut pc, encoding).expect("Should parse operation");
            }
        }
    });
}

#[bench]
fn bench_evaluating_debug_loc_expressions(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::new(&debug_info, LittleEndian);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_addr = DebugAddr::from(EndianSlice::new(&[], LittleEndian));

    let debug_loc = read_section("debug_loc");
    let debug_loc = DebugLoc::new(&debug_loc, LittleEndian);
    let debug_loclists = DebugLocLists::new(&[], LittleEndian);
    let loclists = LocationLists::new(debug_loc, debug_loclists);

    let expressions = debug_loc_expressions(&debug_info, &debug_abbrev, &debug_addr, &loclists);

    b.iter(|| {
        for &(expression, encoding) in &*expressions {
            let mut eval = expression.evaluation(encoding);
            eval.set_initial_value(0);
            let result = eval.evaluate().expect("Should evaluate expression");
            test::black_box(result);
        }
    });
}

// See comment above `test_parse_self_eh_frame`.
#[cfg(target_pointer_width = "64")]
mod cfi {
    use super::*;
    use fallible_iterator::FallibleIterator;

    use gimli::{
        BaseAddresses, CieOrFde, EhFrame, FrameDescriptionEntry, LittleEndian, UnwindContext,
        UnwindSection,
    };

    #[bench]
    fn iterate_entries_and_do_not_parse_any_fde(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);

        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);

        b.iter(|| {
            let mut entries = eh_frame.entries(&bases);
            while let Some(entry) = entries.next().expect("Should parse CFI entry OK") {
                test::black_box(entry);
            }
        });
    }

    #[bench]
    fn iterate_entries_and_parse_every_fde(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);

        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);

        b.iter(|| {
            let mut entries = eh_frame.entries(&bases);
            while let Some(entry) = entries.next().expect("Should parse CFI entry OK") {
                match entry {
                    CieOrFde::Cie(cie) => {
                        test::black_box(cie);
                    }
                    CieOrFde::Fde(partial) => {
                        let fde = partial
                            .parse(EhFrame::cie_from_offset)
                            .expect("Should be able to get CIE for FED");
                        test::black_box(fde);
                    }
                };
            }
        });
    }

    #[bench]
    fn iterate_entries_and_parse_every_fde_and_instructions(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);

        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);

        b.iter(|| {
            let mut entries = eh_frame.entries(&bases);
            while let Some(entry) = entries.next().expect("Should parse CFI entry OK") {
                match entry {
                    CieOrFde::Cie(cie) => {
                        let mut instrs = cie.instructions(&eh_frame, &bases);
                        while let Some(i) =
                            instrs.next().expect("Can parse next CFI instruction OK")
                        {
                            test::black_box(i);
                        }
                    }
                    CieOrFde::Fde(partial) => {
                        let fde = partial
                            .parse(EhFrame::cie_from_offset)
                            .expect("Should be able to get CIE for FED");
                        let mut instrs = fde.instructions(&eh_frame, &bases);
                        while let Some(i) =
                            instrs.next().expect("Can parse next CFI instruction OK")
                        {
                            test::black_box(i);
                        }
                    }
                };
            }
        });
    }

    #[bench]
    fn iterate_entries_evaluate_every_fde(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);

        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);

        let mut ctx = Box::new(UnwindContext::new());

        b.iter(|| {
            let mut entries = eh_frame.entries(&bases);
            while let Some(entry) = entries.next().expect("Should parse CFI entry OK") {
                match entry {
                    CieOrFde::Cie(_) => {}
                    CieOrFde::Fde(partial) => {
                        let fde = partial
                            .parse(EhFrame::cie_from_offset)
                            .expect("Should be able to get CIE for FED");
                        let mut table = fde
                            .rows(&eh_frame, &bases, &mut ctx)
                            .expect("Should be able to initialize ctx");
                        while let Some(row) =
                            table.next_row().expect("Should get next unwind table row")
                        {
                            test::black_box(row);
                        }
                    }
                };
            }
        });
    }

    fn instrs_len<R: Reader>(
        eh_frame: &EhFrame<R>,
        bases: &BaseAddresses,
        fde: &FrameDescriptionEntry<R>,
    ) -> usize {
        fde.instructions(eh_frame, bases)
            .fold(0, |count, _| Ok(count + 1))
            .expect("fold over instructions OK")
    }

    fn get_fde_with_longest_cfi_instructions<R: Reader>(
        eh_frame: &EhFrame<R>,
        bases: &BaseAddresses,
    ) -> FrameDescriptionEntry<R> {
        let mut longest: Option<(usize, FrameDescriptionEntry<_>)> = None;

        let mut entries = eh_frame.entries(bases);
        while let Some(entry) = entries.next().expect("Should parse CFI entry OK") {
            match entry {
                CieOrFde::Cie(_) => {}
                CieOrFde::Fde(partial) => {
                    let fde = partial
                        .parse(EhFrame::cie_from_offset)
                        .expect("Should be able to get CIE for FED");

                    let this_len = instrs_len(eh_frame, bases, &fde);

                    let found_new_longest = match longest {
                        None => true,
                        Some((longest_len, ref _fde)) => this_len > longest_len,
                    };

                    if found_new_longest {
                        longest = Some((this_len, fde));
                    }
                }
            };
        }

        longest.expect("At least one FDE in .eh_frame").1
    }

    #[bench]
    fn parse_longest_fde_instructions(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);
        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);
        let fde = get_fde_with_longest_cfi_instructions(&eh_frame, &bases);

        b.iter(|| {
            let mut instrs = fde.instructions(&eh_frame, &bases);
            while let Some(i) = instrs.next().expect("Should parse instruction OK") {
                test::black_box(i);
            }
        });
    }

    #[bench]
    fn eval_longest_fde_instructions_new_ctx_everytime(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);
        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);
        let fde = get_fde_with_longest_cfi_instructions(&eh_frame, &bases);

        b.iter(|| {
            let mut ctx = Box::new(UnwindContext::new());
            let mut table = fde
                .rows(&eh_frame, &bases, &mut ctx)
                .expect("Should initialize the ctx OK");
            while let Some(row) = table.next_row().expect("Should get next unwind table row") {
                test::black_box(row);
            }
        });
    }

    #[bench]
    fn eval_longest_fde_instructions_same_ctx(b: &mut test::Bencher) {
        let eh_frame = read_section("eh_frame");
        let eh_frame = EhFrame::new(&eh_frame, LittleEndian);
        let bases = BaseAddresses::default()
            .set_eh_frame(0)
            .set_got(0)
            .set_text(0);
        let fde = get_fde_with_longest_cfi_instructions(&eh_frame, &bases);

        let mut ctx = Box::new(UnwindContext::new());

        b.iter(|| {
            let mut table = fde
                .rows(&eh_frame, &bases, &mut ctx)
                .expect("Should initialize the ctx OK");
            while let Some(row) = table.next_row().expect("Should get next unwind table row") {
                test::black_box(row);
            }
        });
    }
}
