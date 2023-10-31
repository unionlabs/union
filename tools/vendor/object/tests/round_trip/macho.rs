use object::read::macho::MachHeader;
use object::read::{Object, ObjectSection};
use object::{macho, read, write, Architecture, BinaryFormat, Endianness};

// Test that segment size is valid when the first section needs alignment.
#[test]
fn issue_286_segment_file_size() {
    let mut object = write::Object::new(
        BinaryFormat::MachO,
        Architecture::X86_64,
        Endianness::Little,
    );

    let text = object.section_id(write::StandardSection::Text);
    object.append_section_data(text, &[1; 30], 0x1000);

    let bytes = &*object.write().unwrap();
    let header = macho::MachHeader64::parse(bytes, 0).unwrap();
    let endian: Endianness = header.endian().unwrap();
    let mut commands = header.load_commands(endian, bytes, 0).unwrap();
    let command = commands.next().unwrap().unwrap();
    let (segment, _) = command.segment_64().unwrap().unwrap();
    assert_eq!(segment.vmsize.get(endian), 30);
    assert_eq!(segment.filesize.get(endian), 30);
}

// We were emitting section file alignment padding that didn't match the address alignment padding.
#[test]
fn issue_552_section_file_alignment() {
    let mut object = write::Object::new(
        BinaryFormat::MachO,
        Architecture::X86_64,
        Endianness::Little,
    );

    // Odd number of sections ensures that the starting file offset is not a multiple of 32.
    // Length of 32 ensures that the file offset of the end of this section is still not a
    // multiple of 32.
    let section = object.add_section(vec![], vec![], object::SectionKind::ReadOnlyDataWithRel);
    object.append_section_data(section, &vec![0u8; 32], 1);

    // Address is already aligned correctly, so there must not any padding,
    // even though file offset is not aligned.
    let section = object.add_section(vec![], vec![], object::SectionKind::ReadOnlyData);
    object.append_section_data(section, &vec![0u8; 1], 32);

    let section = object.add_section(vec![], vec![], object::SectionKind::Text);
    object.append_section_data(section, &vec![0u8; 1], 1);

    let bytes = &*object.write().unwrap();
    let object = read::File::parse(bytes).unwrap();
    let mut sections = object.sections();

    let section = sections.next().unwrap();
    assert_eq!(section.file_range(), Some((368, 32)));
    assert_eq!(section.address(), 0);
    assert_eq!(section.size(), 32);

    let section = sections.next().unwrap();
    assert_eq!(section.file_range(), Some((400, 1)));
    assert_eq!(section.address(), 32);
    assert_eq!(section.size(), 1);
}
