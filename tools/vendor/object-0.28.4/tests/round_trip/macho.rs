use object::read::macho::MachHeader;
use object::{macho, write, Architecture, BinaryFormat, Endianness};

#[test]
// Test that segment size is valid when the first section needs alignment.
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
