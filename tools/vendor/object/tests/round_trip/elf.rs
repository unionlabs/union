use object::read::elf::{FileHeader, SectionHeader};
use object::read::{Object, ObjectSymbol};
use object::{
    elf, read, write, Architecture, BinaryFormat, Endianness, LittleEndian, SectionIndex,
    SectionKind, SymbolFlags, SymbolKind, SymbolScope, SymbolSection, U32,
};
use std::io::Write;

#[test]
fn symtab_shndx() {
    let mut object =
        write::Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    for i in 0..0x10000 {
        let name = format!("func{}", i).into_bytes();
        let (section, offset) =
            object.add_subsection(write::StandardSection::Text, &name, &[0xcc], 1);
        object.add_symbol(write::Symbol {
            name,
            value: offset,
            size: 1,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: write::SymbolSection::Section(section),
            flags: SymbolFlags::None,
        });
    }
    let bytes = object.write().unwrap();

    //std::fs::write(&"symtab_shndx.o", &bytes).unwrap();

    let object = read::File::parse(&*bytes).unwrap();
    assert_eq!(object.format(), BinaryFormat::Elf);
    assert_eq!(object.architecture(), Architecture::X86_64);

    for symbol in object.symbols().skip(1) {
        assert_eq!(
            symbol.section(),
            SymbolSection::Section(SectionIndex(symbol.index().0))
        );
    }
}

#[cfg(feature = "compression")]
#[test]
fn compression_zlib() {
    use object::read::ObjectSection;
    use object::LittleEndian as LE;

    let data = b"test data data data";
    let len = data.len() as u64;

    let mut ch = object::elf::CompressionHeader64::<LE>::default();
    ch.ch_type.set(LE, object::elf::ELFCOMPRESS_ZLIB);
    ch.ch_size.set(LE, len);
    ch.ch_addralign.set(LE, 1);

    let mut buf = Vec::new();
    buf.write(object::bytes_of(&ch)).unwrap();
    let mut encoder = flate2::write::ZlibEncoder::new(buf, flate2::Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();

    let mut object =
        write::Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let section = object.add_section(
        Vec::new(),
        b".debug_info".to_vec(),
        object::SectionKind::Other,
    );
    object.section_mut(section).set_data(compressed, 1);
    object.section_mut(section).flags = object::SectionFlags::Elf {
        sh_flags: object::elf::SHF_COMPRESSED.into(),
    };
    let bytes = object.write().unwrap();

    //std::fs::write(&"compression.o", &bytes).unwrap();

    let object = read::File::parse(&*bytes).unwrap();
    assert_eq!(object.format(), BinaryFormat::Elf);
    assert_eq!(object.architecture(), Architecture::X86_64);

    let section = object.section_by_name(".debug_info").unwrap();
    let uncompressed = section.uncompressed_data().unwrap();
    assert_eq!(data, &*uncompressed);
}

#[cfg(feature = "compression")]
#[test]
fn compression_gnu() {
    use object::read::ObjectSection;
    use std::io::Write;

    let data = b"test data data data";
    let len = data.len() as u32;

    let mut buf = Vec::new();
    buf.write_all(b"ZLIB\0\0\0\0").unwrap();
    buf.write_all(&len.to_be_bytes()).unwrap();
    let mut encoder = flate2::write::ZlibEncoder::new(buf, flate2::Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();

    let mut object =
        write::Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let section = object.add_section(
        Vec::new(),
        b".zdebug_info".to_vec(),
        object::SectionKind::Other,
    );
    object.section_mut(section).set_data(compressed, 1);
    let bytes = object.write().unwrap();

    //std::fs::write(&"compression.o", &bytes).unwrap();

    let object = read::File::parse(&*bytes).unwrap();
    assert_eq!(object.format(), BinaryFormat::Elf);
    assert_eq!(object.architecture(), Architecture::X86_64);

    let section = object.section_by_name(".zdebug_info").unwrap();
    let uncompressed = section.uncompressed_data().unwrap();
    assert_eq!(data, &*uncompressed);
}

#[test]
fn note() {
    let endian = Endianness::Little;
    let mut object = write::Object::new(BinaryFormat::Elf, Architecture::X86_64, endian);

    // Add note section with align = 4.
    let mut buffer = Vec::new();

    buffer
        .write(object::bytes_of(&elf::NoteHeader32 {
            n_namesz: U32::new(endian, 6),
            n_descsz: U32::new(endian, 11),
            n_type: U32::new(endian, 1),
        }))
        .unwrap();
    buffer.write(b"name1\0\0\0").unwrap();
    buffer.write(b"descriptor\0\0").unwrap();

    buffer
        .write(object::bytes_of(&elf::NoteHeader32 {
            n_namesz: U32::new(endian, 6),
            n_descsz: U32::new(endian, 11),
            n_type: U32::new(endian, 2),
        }))
        .unwrap();
    buffer.write(b"name2\0\0\0").unwrap();
    buffer.write(b"descriptor\0\0").unwrap();

    let section = object.add_section(Vec::new(), b".note4".to_vec(), SectionKind::Note);
    object.section_mut(section).set_data(buffer, 4);

    // Add note section with align = 8.
    let mut buffer = Vec::new();

    buffer
        .write(object::bytes_of(&elf::NoteHeader32 {
            n_namesz: U32::new(endian, 6),
            n_descsz: U32::new(endian, 11),
            n_type: U32::new(endian, 1),
        }))
        .unwrap();
    buffer.write(b"name1\0\0\0\0\0\0\0").unwrap();
    buffer.write(b"descriptor\0\0\0\0\0\0").unwrap();

    buffer
        .write(object::bytes_of(&elf::NoteHeader32 {
            n_namesz: U32::new(endian, 4),
            n_descsz: U32::new(endian, 11),
            n_type: U32::new(endian, 2),
        }))
        .unwrap();
    buffer.write(b"abc\0").unwrap();
    buffer.write(b"descriptor\0\0\0\0\0\0").unwrap();

    let section = object.add_section(Vec::new(), b".note8".to_vec(), SectionKind::Note);
    object.section_mut(section).set_data(buffer, 8);

    let bytes = &*object.write().unwrap();

    //std::fs::write(&"note.o", &bytes).unwrap();

    let header = elf::FileHeader64::parse(bytes).unwrap();
    let endian: LittleEndian = header.endian().unwrap();
    let sections = header.sections(endian, bytes).unwrap();

    let section = sections.section(SectionIndex(1)).unwrap();
    assert_eq!(sections.section_name(endian, section).unwrap(), b".note4");
    assert_eq!(section.sh_addralign(endian), 4);
    let mut notes = section.notes(endian, bytes).unwrap().unwrap();
    let note = notes.next().unwrap().unwrap();
    assert_eq!(note.name(), b"name1");
    assert_eq!(note.desc(), b"descriptor\0");
    assert_eq!(note.n_type(endian), 1);
    let note = notes.next().unwrap().unwrap();
    assert_eq!(note.name(), b"name2");
    assert_eq!(note.desc(), b"descriptor\0");
    assert_eq!(note.n_type(endian), 2);
    assert!(notes.next().unwrap().is_none());

    let section = sections.section(SectionIndex(2)).unwrap();
    assert_eq!(sections.section_name(endian, section).unwrap(), b".note8");
    assert_eq!(section.sh_addralign(endian), 8);
    let mut notes = section.notes(endian, bytes).unwrap().unwrap();
    let note = notes.next().unwrap().unwrap();
    assert_eq!(note.name(), b"name1");
    assert_eq!(note.desc(), b"descriptor\0");
    assert_eq!(note.n_type(endian), 1);
    let note = notes.next().unwrap().unwrap();
    assert_eq!(note.name(), b"abc");
    assert_eq!(note.desc(), b"descriptor\0");
    assert_eq!(note.n_type(endian), 2);
    assert!(notes.next().unwrap().is_none());
}
