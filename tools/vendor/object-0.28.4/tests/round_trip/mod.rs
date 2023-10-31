#![cfg(all(feature = "read", feature = "write"))]

use object::read::{Object, ObjectSection, ObjectSymbol};
use object::{read, write};
use object::{
    Architecture, BinaryFormat, Endianness, RelocationEncoding, RelocationKind, SectionKind,
    SymbolFlags, SymbolKind, SymbolScope, SymbolSection,
};

mod bss;
mod coff;
mod comdat;
mod common;
mod elf;
mod macho;
mod section_flags;
mod tls;

#[test]
fn coff_x86_64() {
    let mut object =
        write::Object::new(BinaryFormat::Coff, Architecture::X86_64, Endianness::Little);

    object.add_file_symbol(b"file.c".to_vec());

    let text = object.section_id(write::StandardSection::Text);
    object.append_section_data(text, &[1; 30], 4);

    let func1_offset = object.append_section_data(text, &[1; 30], 4);
    assert_eq!(func1_offset, 32);
    let func1_symbol = object.add_symbol(write::Symbol {
        name: b"func1".to_vec(),
        value: func1_offset,
        size: 32,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: write::SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
    object
        .add_relocation(
            text,
            write::Relocation {
                offset: 8,
                size: 64,
                kind: RelocationKind::Absolute,
                encoding: RelocationEncoding::Generic,
                symbol: func1_symbol,
                addend: 0,
            },
        )
        .unwrap();

    let bytes = object.write().unwrap();
    let object = read::File::parse(&*bytes).unwrap();
    assert_eq!(object.format(), BinaryFormat::Coff);
    assert_eq!(object.architecture(), Architecture::X86_64);
    assert_eq!(object.endianness(), Endianness::Little);

    let mut sections = object.sections();

    let text = sections.next().unwrap();
    println!("{:?}", text);
    let text_index = text.index();
    assert_eq!(text.name(), Ok(".text"));
    assert_eq!(text.kind(), SectionKind::Text);
    assert_eq!(text.address(), 0);
    assert_eq!(text.size(), 62);
    assert_eq!(&text.data().unwrap()[..30], &[1; 30]);
    assert_eq!(&text.data().unwrap()[32..62], &[1; 30]);

    let mut symbols = object.symbols();

    let symbol = symbols.next().unwrap();
    println!("{:?}", symbol);
    assert_eq!(symbol.name(), Ok("file.c"));
    assert_eq!(symbol.address(), 0);
    assert_eq!(symbol.kind(), SymbolKind::File);
    assert_eq!(symbol.section(), SymbolSection::None);
    assert_eq!(symbol.scope(), SymbolScope::Compilation);
    assert_eq!(symbol.is_weak(), false);

    let symbol = symbols.next().unwrap();
    println!("{:?}", symbol);
    let func1_symbol = symbol.index();
    assert_eq!(symbol.name(), Ok("func1"));
    assert_eq!(symbol.address(), func1_offset);
    assert_eq!(symbol.kind(), SymbolKind::Text);
    assert_eq!(symbol.section_index(), Some(text_index));
    assert_eq!(symbol.scope(), SymbolScope::Linkage);
    assert_eq!(symbol.is_weak(), false);
    assert_eq!(symbol.is_undefined(), false);

    let mut relocations = text.relocations();

    let (offset, relocation) = relocations.next().unwrap();
    println!("{:?}", relocation);
    assert_eq!(offset, 8);
    assert_eq!(relocation.kind(), RelocationKind::Absolute);
    assert_eq!(relocation.encoding(), RelocationEncoding::Generic);
    assert_eq!(relocation.size(), 64);
    assert_eq!(
        relocation.target(),
        read::RelocationTarget::Symbol(func1_symbol)
    );
    assert_eq!(relocation.addend(), 0);

    let map = object.symbol_map();
    let symbol = map.get(func1_offset + 1).unwrap();
    assert_eq!(symbol.address(), func1_offset);
    assert_eq!(symbol.name(), "func1");
    assert_eq!(map.get(func1_offset - 1), None);
}

#[test]
fn elf_x86_64() {
    let mut object =
        write::Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    object.add_file_symbol(b"file.c".to_vec());

    let text = object.section_id(write::StandardSection::Text);
    object.append_section_data(text, &[1; 30], 4);

    let func1_offset = object.append_section_data(text, &[1; 30], 4);
    assert_eq!(func1_offset, 32);
    let func1_symbol = object.add_symbol(write::Symbol {
        name: b"func1".to_vec(),
        value: func1_offset,
        size: 32,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: write::SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
    object
        .add_relocation(
            text,
            write::Relocation {
                offset: 8,
                size: 64,
                kind: RelocationKind::Absolute,
                encoding: RelocationEncoding::Generic,
                symbol: func1_symbol,
                addend: 0,
            },
        )
        .unwrap();

    let bytes = object.write().unwrap();
    let object = read::File::parse(&*bytes).unwrap();
    assert_eq!(object.format(), BinaryFormat::Elf);
    assert_eq!(object.architecture(), Architecture::X86_64);
    assert_eq!(object.endianness(), Endianness::Little);

    let mut sections = object.sections();

    let section = sections.next().unwrap();
    println!("{:?}", section);
    assert_eq!(section.name(), Ok(""));
    assert_eq!(section.kind(), SectionKind::Metadata);
    assert_eq!(section.address(), 0);
    assert_eq!(section.size(), 0);

    let text = sections.next().unwrap();
    println!("{:?}", text);
    let text_index = text.index();
    assert_eq!(text.name(), Ok(".text"));
    assert_eq!(text.kind(), SectionKind::Text);
    assert_eq!(text.address(), 0);
    assert_eq!(text.size(), 62);
    assert_eq!(&text.data().unwrap()[..30], &[1; 30]);
    assert_eq!(&text.data().unwrap()[32..62], &[1; 30]);

    let mut symbols = object.symbols();

    let symbol = symbols.next().unwrap();
    println!("{:?}", symbol);
    assert_eq!(symbol.name(), Ok(""));
    assert_eq!(symbol.address(), 0);
    assert_eq!(symbol.kind(), SymbolKind::Null);
    assert_eq!(symbol.section_index(), None);
    assert_eq!(symbol.scope(), SymbolScope::Unknown);
    assert_eq!(symbol.is_weak(), false);
    assert_eq!(symbol.is_undefined(), true);

    let symbol = symbols.next().unwrap();
    println!("{:?}", symbol);
    assert_eq!(symbol.name(), Ok("file.c"));
    assert_eq!(symbol.address(), 0);
    assert_eq!(symbol.kind(), SymbolKind::File);
    assert_eq!(symbol.section(), SymbolSection::None);
    assert_eq!(symbol.scope(), SymbolScope::Compilation);
    assert_eq!(symbol.is_weak(), false);

    let symbol = symbols.next().unwrap();
    println!("{:?}", symbol);
    let func1_symbol = symbol.index();
    assert_eq!(symbol.name(), Ok("func1"));
    assert_eq!(symbol.address(), func1_offset);
    assert_eq!(symbol.kind(), SymbolKind::Text);
    assert_eq!(symbol.section_index(), Some(text_index));
    assert_eq!(symbol.scope(), SymbolScope::Linkage);
    assert_eq!(symbol.is_weak(), false);
    assert_eq!(symbol.is_undefined(), false);

    let mut relocations = text.relocations();

    let (offset, relocation) = relocations.next().unwrap();
    println!("{:?}", relocation);
    assert_eq!(offset, 8);
    assert_eq!(relocation.kind(), RelocationKind::Absolute);
    assert_eq!(relocation.encoding(), RelocationEncoding::Generic);
    assert_eq!(relocation.size(), 64);
    assert_eq!(
        relocation.target(),
        read::RelocationTarget::Symbol(func1_symbol)
    );
    assert_eq!(relocation.addend(), 0);

    let map = object.symbol_map();
    let symbol = map.get(func1_offset + 1).unwrap();
    assert_eq!(symbol.address(), func1_offset);
    assert_eq!(symbol.name(), "func1");
    assert_eq!(map.get(func1_offset - 1), None);
}

#[test]
fn elf_any() {
    for (arch, endian) in [
        (Architecture::Aarch64, Endianness::Little),
        (Architecture::Arm, Endianness::Little),
        (Architecture::Avr, Endianness::Little),
        (Architecture::Bpf, Endianness::Little),
        (Architecture::I386, Endianness::Little),
        (Architecture::X86_64, Endianness::Little),
        (Architecture::X86_64_X32, Endianness::Little),
        (Architecture::Hexagon, Endianness::Little),
        (Architecture::LoongArch64, Endianness::Little),
        (Architecture::Mips, Endianness::Little),
        (Architecture::Mips64, Endianness::Little),
        (Architecture::Msp430, Endianness::Little),
        (Architecture::PowerPc, Endianness::Big),
        (Architecture::PowerPc64, Endianness::Big),
        (Architecture::Riscv32, Endianness::Little),
        (Architecture::Riscv64, Endianness::Little),
        (Architecture::S390x, Endianness::Big),
        (Architecture::Sparc64, Endianness::Big),
    ]
    .iter()
    .copied()
    {
        let mut object = write::Object::new(BinaryFormat::Elf, arch, endian);

        let section = object.section_id(write::StandardSection::Data);
        object.append_section_data(section, &[1; 30], 4);
        let symbol = object.section_symbol(section);

        object
            .add_relocation(
                section,
                write::Relocation {
                    offset: 8,
                    size: 32,
                    kind: RelocationKind::Absolute,
                    encoding: RelocationEncoding::Generic,
                    symbol,
                    addend: 0,
                },
            )
            .unwrap();
        if arch.address_size().unwrap().bytes() >= 8 {
            object
                .add_relocation(
                    section,
                    write::Relocation {
                        offset: 16,
                        size: 64,
                        kind: RelocationKind::Absolute,
                        encoding: RelocationEncoding::Generic,
                        symbol,
                        addend: 0,
                    },
                )
                .unwrap();
        }

        let bytes = object.write().unwrap();
        let object = read::File::parse(&*bytes).unwrap();
        println!("{:?}", object.architecture());
        assert_eq!(object.format(), BinaryFormat::Elf);
        assert_eq!(object.architecture(), arch);
        assert_eq!(object.endianness(), endian);

        let mut sections = object.sections();

        let section = sections.next().unwrap();
        println!("{:?}", section);
        assert_eq!(section.name(), Ok(""));
        assert_eq!(section.kind(), SectionKind::Metadata);
        assert_eq!(section.address(), 0);
        assert_eq!(section.size(), 0);

        let data = sections.next().unwrap();
        println!("{:?}", data);
        assert_eq!(data.name(), Ok(".data"));
        assert_eq!(data.kind(), SectionKind::Data);

        let mut relocations = data.relocations();

        let (offset, relocation) = relocations.next().unwrap();
        println!("{:?}", relocation);
        assert_eq!(offset, 8);
        assert_eq!(relocation.kind(), RelocationKind::Absolute);
        assert_eq!(relocation.encoding(), RelocationEncoding::Generic);
        assert_eq!(relocation.size(), 32);
        assert_eq!(relocation.addend(), 0);

        if arch.address_size().unwrap().bytes() >= 8 {
            let (offset, relocation) = relocations.next().unwrap();
            println!("{:?}", relocation);
            assert_eq!(offset, 16);
            assert_eq!(relocation.kind(), RelocationKind::Absolute);
            assert_eq!(relocation.encoding(), RelocationEncoding::Generic);
            assert_eq!(relocation.size(), 64);
            assert_eq!(relocation.addend(), 0);
        }
    }
}

#[test]
fn macho_x86_64() {
    let mut object = write::Object::new(
        BinaryFormat::MachO,
        Architecture::X86_64,
        Endianness::Little,
    );

    object.add_file_symbol(b"file.c".to_vec());

    let text = object.section_id(write::StandardSection::Text);
    object.append_section_data(text, &[1; 30], 4);

    let func1_offset = object.append_section_data(text, &[1; 30], 4);
    assert_eq!(func1_offset, 32);
    let func1_symbol = object.add_symbol(write::Symbol {
        name: b"func1".to_vec(),
        value: func1_offset,
        size: 32,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: write::SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
    object
        .add_relocation(
            text,
            write::Relocation {
                offset: 8,
                size: 64,
                kind: RelocationKind::Absolute,
                encoding: RelocationEncoding::Generic,
                symbol: func1_symbol,
                addend: 0,
            },
        )
        .unwrap();
    object
        .add_relocation(
            text,
            write::Relocation {
                offset: 16,
                size: 32,
                kind: RelocationKind::Relative,
                encoding: RelocationEncoding::Generic,
                symbol: func1_symbol,
                addend: -4,
            },
        )
        .unwrap();

    let bytes = object.write().unwrap();
    let object = read::File::parse(&*bytes).unwrap();
    assert_eq!(object.format(), BinaryFormat::MachO);
    assert_eq!(object.architecture(), Architecture::X86_64);
    assert_eq!(object.endianness(), Endianness::Little);

    let mut sections = object.sections();

    let text = sections.next().unwrap();
    println!("{:?}", text);
    let text_index = text.index();
    assert_eq!(text.name(), Ok("__text"));
    assert_eq!(text.segment_name(), Ok(Some("__TEXT")));
    assert_eq!(text.kind(), SectionKind::Text);
    assert_eq!(text.address(), 0);
    assert_eq!(text.size(), 62);
    assert_eq!(&text.data().unwrap()[..30], &[1; 30]);
    assert_eq!(&text.data().unwrap()[32..62], &[1; 30]);

    let mut symbols = object.symbols();

    let symbol = symbols.next().unwrap();
    println!("{:?}", symbol);
    let func1_symbol = symbol.index();
    assert_eq!(symbol.name(), Ok("_func1"));
    assert_eq!(symbol.address(), func1_offset);
    assert_eq!(symbol.kind(), SymbolKind::Text);
    assert_eq!(symbol.section_index(), Some(text_index));
    assert_eq!(symbol.scope(), SymbolScope::Linkage);
    assert_eq!(symbol.is_weak(), false);
    assert_eq!(symbol.is_undefined(), false);

    let mut relocations = text.relocations();

    let (offset, relocation) = relocations.next().unwrap();
    println!("{:?}", relocation);
    assert_eq!(offset, 8);
    assert_eq!(relocation.kind(), RelocationKind::Absolute);
    assert_eq!(relocation.encoding(), RelocationEncoding::Generic);
    assert_eq!(relocation.size(), 64);
    assert_eq!(
        relocation.target(),
        read::RelocationTarget::Symbol(func1_symbol)
    );
    assert_eq!(relocation.addend(), 0);

    let (offset, relocation) = relocations.next().unwrap();
    println!("{:?}", relocation);
    assert_eq!(offset, 16);
    assert_eq!(relocation.kind(), RelocationKind::Relative);
    assert_eq!(relocation.encoding(), RelocationEncoding::X86RipRelative);
    assert_eq!(relocation.size(), 32);
    assert_eq!(
        relocation.target(),
        read::RelocationTarget::Symbol(func1_symbol)
    );
    assert_eq!(relocation.addend(), -4);

    let map = object.symbol_map();
    let symbol = map.get(func1_offset + 1).unwrap();
    assert_eq!(symbol.address(), func1_offset);
    assert_eq!(symbol.name(), "_func1");
    assert_eq!(map.get(func1_offset - 1), None);
}
