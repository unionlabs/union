#![cfg(all(feature = "read", feature = "write"))]

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use gimli::read;
use gimli::write::{self, Address, EndianVec};
use gimli::LittleEndian;

fn read_section(section: &str) -> Vec<u8> {
    let mut path = PathBuf::new();
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        path.push(dir);
    }
    path.push("fixtures/self");
    path.push(section);

    println!("Reading section \"{}\" at path {:?}", section, path);
    assert!(path.is_file());
    let mut file = File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

#[test]
fn test_convert_debug_info() {
    // Convert existing sections
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = read::DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_info = read_section("debug_info");
    let debug_info = read::DebugInfo::new(&debug_info, LittleEndian);

    let debug_line = read_section("debug_line");
    let debug_line = read::DebugLine::new(&debug_line, LittleEndian);

    let debug_str = read_section("debug_str");
    let debug_str = read::DebugStr::new(&debug_str, LittleEndian);

    let debug_ranges = read_section("debug_ranges");
    let debug_ranges = read::DebugRanges::new(&debug_ranges, LittleEndian);

    let debug_rnglists = read::DebugRngLists::new(&[], LittleEndian);

    let ranges = gimli::RangeLists::new(debug_ranges, debug_rnglists);

    let debug_loc = read_section("debug_loc");
    let debug_loc = read::DebugLoc::new(&debug_loc, LittleEndian);

    let debug_loclists = read::DebugLocLists::new(&[], LittleEndian);

    let locations = gimli::LocationLists::new(debug_loc, debug_loclists);

    let dwarf = read::Dwarf {
        debug_abbrev,
        debug_info,
        debug_line,
        debug_str,
        ranges,
        locations,
        ..Default::default()
    };

    let mut dwarf = write::Dwarf::from(&dwarf, &|address| Some(Address::Constant(address)))
        .expect("Should convert DWARF information");

    assert_eq!(dwarf.units.count(), 23);
    let entries: usize = (0..dwarf.units.count())
        .map(|i| dwarf.units.get(dwarf.units.id(i)).count())
        .sum();
    assert_eq!(entries, 29_560);
    assert_eq!(dwarf.line_strings.count(), 0);
    assert_eq!(dwarf.strings.count(), 3921);

    // Write to new sections
    let mut write_sections = write::Sections::new(EndianVec::new(LittleEndian));
    dwarf
        .write(&mut write_sections)
        .expect("Should write DWARF information");
    let debug_info_data = write_sections.debug_info.slice();
    let debug_abbrev_data = write_sections.debug_abbrev.slice();
    let debug_line_data = write_sections.debug_line.slice();
    let debug_ranges_data = write_sections.debug_ranges.slice();
    let debug_loc_data = write_sections.debug_loc.slice();
    let debug_str_data = write_sections.debug_str.slice();
    assert_eq!(debug_info_data.len(), 394_930);
    assert_eq!(debug_abbrev_data.len(), 9701);
    assert_eq!(debug_line_data.len(), 105_797);
    assert_eq!(debug_ranges_data.len(), 155_712);
    assert_eq!(debug_loc_data.len(), 245_168);
    assert_eq!(debug_str_data.len(), 144_731);

    // Convert new sections
    let debug_abbrev = read::DebugAbbrev::new(debug_abbrev_data, LittleEndian);
    let debug_info = read::DebugInfo::new(debug_info_data, LittleEndian);
    let debug_line = read::DebugLine::new(debug_line_data, LittleEndian);
    let debug_str = read::DebugStr::new(debug_str_data, LittleEndian);
    let debug_ranges = read::DebugRanges::new(debug_ranges_data, LittleEndian);
    let debug_rnglists = read::DebugRngLists::new(&[], LittleEndian);
    let debug_loc = read::DebugLoc::new(debug_loc_data, LittleEndian);
    let debug_loclists = read::DebugLocLists::new(&[], LittleEndian);

    let ranges = gimli::RangeLists::new(debug_ranges, debug_rnglists);
    let locations = gimli::LocationLists::new(debug_loc, debug_loclists);

    let dwarf = read::Dwarf {
        debug_abbrev,
        debug_info,
        debug_line,
        debug_str,
        ranges,
        locations,
        ..Default::default()
    };

    let dwarf = write::Dwarf::from(&dwarf, &|address| Some(Address::Constant(address)))
        .expect("Should convert DWARF information");

    assert_eq!(dwarf.units.count(), 23);
    let entries: usize = (0..dwarf.units.count())
        .map(|i| dwarf.units.get(dwarf.units.id(i)).count())
        .sum();
    assert_eq!(entries, 29_560);
    assert_eq!(dwarf.strings.count(), 3921);
}

#[test]
fn test_convert_eh_frame() {
    // Convert existing section
    let eh_frame = read_section("eh_frame");
    let mut eh_frame = read::EhFrame::new(&eh_frame, LittleEndian);
    // The `.eh_frame` fixture data was created on a 64-bit machine.
    eh_frame.set_address_size(8);
    let frames = write::FrameTable::from(&eh_frame, &|address| Some(Address::Constant(address)))
        .expect("Should convert eh_frame information");
    assert_eq!(frames.cie_count(), 2);
    assert_eq!(frames.fde_count(), 3482);

    // Write to new section
    let mut write_eh_frame = write::EhFrame(EndianVec::new(LittleEndian));
    frames
        .write_eh_frame(&mut write_eh_frame)
        .expect("Should write eh_frame information");
    let eh_frame = write_eh_frame.slice();
    assert_eq!(eh_frame.len(), 147144);

    // Convert new section
    let mut eh_frame = read::EhFrame::new(&eh_frame, LittleEndian);
    eh_frame.set_address_size(8);
    let frames = write::FrameTable::from(&eh_frame, &|address| Some(Address::Constant(address)))
        .expect("Should convert eh_frame information");
    assert_eq!(frames.cie_count(), 2);
    assert_eq!(frames.fde_count(), 3482);
}
