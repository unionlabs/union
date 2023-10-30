use alloc::vec::Vec;
use core::mem;

use crate::endian::{LittleEndian as LE, U16Bytes, U32Bytes, U16, U32};
use crate::pe as coff;
use crate::write::string::*;
use crate::write::util::*;
use crate::write::*;

#[derive(Default, Clone, Copy)]
struct SectionOffsets {
    offset: usize,
    str_id: Option<StringId>,
    reloc_offset: usize,
    selection: u8,
    associative_section: u16,
}

#[derive(Default, Clone, Copy)]
struct SymbolOffsets {
    index: usize,
    str_id: Option<StringId>,
    aux_count: u8,
}

/// Internal format to use for the `.drectve` section containing linker
/// directives for symbol exports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoffExportStyle {
    /// MSVC format supported by link.exe and LLD.
    Msvc,
    /// Gnu format supported by GNU LD and LLD.
    Gnu,
}

impl<'a> Object<'a> {
    pub(crate) fn coff_section_info(
        &self,
        section: StandardSection,
    ) -> (&'static [u8], &'static [u8], SectionKind) {
        match section {
            StandardSection::Text => (&[], &b".text"[..], SectionKind::Text),
            StandardSection::Data => (&[], &b".data"[..], SectionKind::Data),
            StandardSection::ReadOnlyData
            | StandardSection::ReadOnlyDataWithRel
            | StandardSection::ReadOnlyString => (&[], &b".rdata"[..], SectionKind::ReadOnlyData),
            StandardSection::UninitializedData => {
                (&[], &b".bss"[..], SectionKind::UninitializedData)
            }
            // TLS sections are data sections with a special name.
            StandardSection::Tls => (&[], &b".tls$"[..], SectionKind::Data),
            StandardSection::UninitializedTls => {
                // Unsupported section.
                (&[], &[], SectionKind::UninitializedTls)
            }
            StandardSection::TlsVariables => {
                // Unsupported section.
                (&[], &[], SectionKind::TlsVariables)
            }
            StandardSection::Common => {
                // Unsupported section.
                (&[], &[], SectionKind::Common)
            }
        }
    }

    pub(crate) fn coff_subsection_name(&self, section: &[u8], value: &[u8]) -> Vec<u8> {
        let mut name = section.to_vec();
        name.push(b'$');
        name.extend_from_slice(value);
        name
    }

    pub(crate) fn coff_fixup_relocation(&mut self, mut relocation: &mut Relocation) -> i64 {
        if relocation.kind == RelocationKind::GotRelative {
            // Use a stub symbol for the relocation instead.
            // This isn't really a GOT, but it's a similar purpose.
            // TODO: need to handle DLL imports differently?
            relocation.kind = RelocationKind::Relative;
            relocation.symbol = self.coff_add_stub_symbol(relocation.symbol);
        } else if relocation.kind == RelocationKind::PltRelative {
            // Windows doesn't need a separate relocation type for
            // references to functions in import libraries.
            // For convenience, treat this the same as Relative.
            relocation.kind = RelocationKind::Relative;
        }

        let constant = match self.architecture {
            Architecture::I386 | Architecture::Arm | Architecture::Aarch64 => match relocation.kind
            {
                RelocationKind::Relative => {
                    // IMAGE_REL_I386_REL32, IMAGE_REL_ARM_REL32, IMAGE_REL_ARM64_REL32
                    relocation.addend + 4
                }
                _ => relocation.addend,
            },
            Architecture::X86_64 => match relocation.kind {
                RelocationKind::Relative => {
                    // IMAGE_REL_AMD64_REL32 through to IMAGE_REL_AMD64_REL32_5
                    if relocation.addend <= -4 && relocation.addend >= -9 {
                        0
                    } else {
                        relocation.addend + 4
                    }
                }
                _ => relocation.addend,
            },
            _ => unimplemented!(),
        };
        relocation.addend -= constant;
        constant
    }

    fn coff_add_stub_symbol(&mut self, symbol_id: SymbolId) -> SymbolId {
        if let Some(stub_id) = self.stub_symbols.get(&symbol_id) {
            return *stub_id;
        }
        let stub_size = self.architecture.address_size().unwrap().bytes();

        let mut name = b".rdata$.refptr.".to_vec();
        name.extend_from_slice(&self.symbols[symbol_id.0].name);
        let section_id = self.add_section(Vec::new(), name, SectionKind::ReadOnlyData);
        let section = self.section_mut(section_id);
        section.set_data(vec![0; stub_size as usize], u64::from(stub_size));
        section.relocations = vec![Relocation {
            offset: 0,
            size: stub_size * 8,
            kind: RelocationKind::Absolute,
            encoding: RelocationEncoding::Generic,
            symbol: symbol_id,
            addend: 0,
        }];

        let mut name = b".refptr.".to_vec();
        name.extend_from_slice(&self.symbol(symbol_id).name);
        let stub_id = self.add_raw_symbol(Symbol {
            name,
            value: 0,
            size: u64::from(stub_size),
            kind: SymbolKind::Data,
            scope: SymbolScope::Compilation,
            weak: false,
            section: SymbolSection::Section(section_id),
            flags: SymbolFlags::None,
        });
        self.stub_symbols.insert(symbol_id, stub_id);

        stub_id
    }

    /// Appends linker directives to the `.drectve` section to tell the linker
    /// to export all symbols with `SymbolScope::Dynamic`.
    ///
    /// This must be called after all symbols have been defined.
    pub fn add_coff_exports(&mut self, style: CoffExportStyle) {
        assert_eq!(self.format, BinaryFormat::Coff);

        let mut directives = vec![];
        for symbol in &self.symbols {
            if symbol.scope == SymbolScope::Dynamic {
                match style {
                    CoffExportStyle::Msvc => directives.extend(b" /EXPORT:\""),
                    CoffExportStyle::Gnu => directives.extend(b" -export:\""),
                }
                directives.extend(&symbol.name);
                directives.extend(b"\"");
                if symbol.kind != SymbolKind::Text {
                    match style {
                        CoffExportStyle::Msvc => directives.extend(b",DATA"),
                        CoffExportStyle::Gnu => directives.extend(b",data"),
                    }
                }
            }
        }
        let drectve = self.add_section(vec![], b".drectve".to_vec(), SectionKind::Linker);
        self.append_section_data(drectve, &directives, 1);
    }

    pub(crate) fn coff_write(&self, buffer: &mut dyn WritableBuffer) -> Result<()> {
        // Calculate offsets of everything, and build strtab.
        let mut offset = 0;
        let mut strtab = StringTable::default();

        // COFF header.
        offset += mem::size_of::<coff::ImageFileHeader>();

        // Section headers.
        offset += self.sections.len() * mem::size_of::<coff::ImageSectionHeader>();

        // Calculate size of section data and add section strings to strtab.
        let mut section_offsets = vec![SectionOffsets::default(); self.sections.len()];
        for (index, section) in self.sections.iter().enumerate() {
            if section.name.len() > 8 {
                section_offsets[index].str_id = Some(strtab.add(&section.name));
            }

            let len = section.data.len();
            if len != 0 {
                // TODO: not sure what alignment is required here, but this seems to match LLVM
                offset = align(offset, 4);
                section_offsets[index].offset = offset;
                offset += len;
            } else {
                section_offsets[index].offset = 0;
            }

            // Calculate size of relocations.
            let mut count = section.relocations.len();
            if count != 0 {
                section_offsets[index].reloc_offset = offset;
                if count > 0xffff {
                    count += 1;
                }
                offset += count * mem::size_of::<coff::ImageRelocation>();
            }
        }

        // Set COMDAT flags.
        for comdat in &self.comdats {
            let symbol = &self.symbols[comdat.symbol.0];
            let comdat_section = match symbol.section {
                SymbolSection::Section(id) => id.0,
                _ => {
                    return Err(Error(format!(
                        "unsupported COMDAT symbol `{}` section {:?}",
                        symbol.name().unwrap_or(""),
                        symbol.section
                    )));
                }
            };
            section_offsets[comdat_section].selection = match comdat.kind {
                ComdatKind::NoDuplicates => coff::IMAGE_COMDAT_SELECT_NODUPLICATES,
                ComdatKind::Any => coff::IMAGE_COMDAT_SELECT_ANY,
                ComdatKind::SameSize => coff::IMAGE_COMDAT_SELECT_SAME_SIZE,
                ComdatKind::ExactMatch => coff::IMAGE_COMDAT_SELECT_EXACT_MATCH,
                ComdatKind::Largest => coff::IMAGE_COMDAT_SELECT_LARGEST,
                ComdatKind::Newest => coff::IMAGE_COMDAT_SELECT_NEWEST,
                ComdatKind::Unknown => {
                    return Err(Error(format!(
                        "unsupported COMDAT symbol `{}` kind {:?}",
                        symbol.name().unwrap_or(""),
                        comdat.kind
                    )));
                }
            };
            for id in &comdat.sections {
                let section = &self.sections[id.0];
                if section.symbol.is_none() {
                    return Err(Error(format!(
                        "missing symbol for COMDAT section `{}`",
                        section.name().unwrap_or(""),
                    )));
                }
                if id.0 != comdat_section {
                    section_offsets[id.0].selection = coff::IMAGE_COMDAT_SELECT_ASSOCIATIVE;
                    section_offsets[id.0].associative_section = comdat_section as u16 + 1;
                }
            }
        }

        // Calculate size of symbols and add symbol strings to strtab.
        let mut symbol_offsets = vec![SymbolOffsets::default(); self.symbols.len()];
        let mut symtab_count = 0;
        for (index, symbol) in self.symbols.iter().enumerate() {
            symbol_offsets[index].index = symtab_count;
            symtab_count += 1;
            match symbol.kind {
                SymbolKind::File => {
                    // Name goes in auxilary symbol records.
                    let aux_count = (symbol.name.len() + coff::IMAGE_SIZEOF_SYMBOL - 1)
                        / coff::IMAGE_SIZEOF_SYMBOL;
                    symbol_offsets[index].aux_count = aux_count as u8;
                    symtab_count += aux_count;
                    // Don't add name to strtab.
                    continue;
                }
                SymbolKind::Section => {
                    symbol_offsets[index].aux_count = 1;
                    symtab_count += 1;
                }
                _ => {}
            }
            if symbol.name.len() > 8 {
                symbol_offsets[index].str_id = Some(strtab.add(&symbol.name));
            }
        }

        // Calculate size of symtab.
        let symtab_offset = offset;
        let symtab_len = symtab_count * coff::IMAGE_SIZEOF_SYMBOL;
        offset += symtab_len;

        // Calculate size of strtab.
        let strtab_offset = offset;
        let mut strtab_data = Vec::new();
        // First 4 bytes of strtab are the length.
        strtab.write(4, &mut strtab_data);
        let strtab_len = strtab_data.len() + 4;
        offset += strtab_len;

        // Start writing.
        buffer
            .reserve(offset)
            .map_err(|_| Error(String::from("Cannot allocate buffer")))?;

        // Write file header.
        let header = coff::ImageFileHeader {
            machine: U16::new(
                LE,
                match self.architecture {
                    Architecture::Arm => coff::IMAGE_FILE_MACHINE_ARMNT,
                    Architecture::Aarch64 => coff::IMAGE_FILE_MACHINE_ARM64,
                    Architecture::I386 => coff::IMAGE_FILE_MACHINE_I386,
                    Architecture::X86_64 => coff::IMAGE_FILE_MACHINE_AMD64,
                    _ => {
                        return Err(Error(format!(
                            "unimplemented architecture {:?}",
                            self.architecture
                        )));
                    }
                },
            ),
            number_of_sections: U16::new(LE, self.sections.len() as u16),
            time_date_stamp: U32::default(),
            pointer_to_symbol_table: U32::new(LE, symtab_offset as u32),
            number_of_symbols: U32::new(LE, symtab_count as u32),
            size_of_optional_header: U16::default(),
            characteristics: match self.flags {
                FileFlags::Coff { characteristics } => U16::new(LE, characteristics),
                _ => U16::default(),
            },
        };
        buffer.write(&header);

        // Write section headers.
        for (index, section) in self.sections.iter().enumerate() {
            let mut characteristics = if let SectionFlags::Coff {
                characteristics, ..
            } = section.flags
            {
                characteristics
            } else {
                match section.kind {
                    SectionKind::Text => {
                        coff::IMAGE_SCN_CNT_CODE
                            | coff::IMAGE_SCN_MEM_EXECUTE
                            | coff::IMAGE_SCN_MEM_READ
                    }
                    SectionKind::Data => {
                        coff::IMAGE_SCN_CNT_INITIALIZED_DATA
                            | coff::IMAGE_SCN_MEM_READ
                            | coff::IMAGE_SCN_MEM_WRITE
                    }
                    SectionKind::UninitializedData => {
                        coff::IMAGE_SCN_CNT_UNINITIALIZED_DATA
                            | coff::IMAGE_SCN_MEM_READ
                            | coff::IMAGE_SCN_MEM_WRITE
                    }
                    SectionKind::ReadOnlyData | SectionKind::ReadOnlyString => {
                        coff::IMAGE_SCN_CNT_INITIALIZED_DATA | coff::IMAGE_SCN_MEM_READ
                    }
                    SectionKind::Debug | SectionKind::Other | SectionKind::OtherString => {
                        coff::IMAGE_SCN_CNT_INITIALIZED_DATA
                            | coff::IMAGE_SCN_MEM_READ
                            | coff::IMAGE_SCN_MEM_DISCARDABLE
                    }
                    SectionKind::Linker => coff::IMAGE_SCN_LNK_INFO | coff::IMAGE_SCN_LNK_REMOVE,
                    SectionKind::Common
                    | SectionKind::Tls
                    | SectionKind::UninitializedTls
                    | SectionKind::TlsVariables
                    | SectionKind::Note
                    | SectionKind::Unknown
                    | SectionKind::Metadata
                    | SectionKind::Elf(_) => {
                        return Err(Error(format!(
                            "unimplemented section `{}` kind {:?}",
                            section.name().unwrap_or(""),
                            section.kind
                        )));
                    }
                }
            };
            if section_offsets[index].selection != 0 {
                characteristics |= coff::IMAGE_SCN_LNK_COMDAT;
            };
            if section.relocations.len() > 0xffff {
                characteristics |= coff::IMAGE_SCN_LNK_NRELOC_OVFL;
            }
            characteristics |= match section.align {
                1 => coff::IMAGE_SCN_ALIGN_1BYTES,
                2 => coff::IMAGE_SCN_ALIGN_2BYTES,
                4 => coff::IMAGE_SCN_ALIGN_4BYTES,
                8 => coff::IMAGE_SCN_ALIGN_8BYTES,
                16 => coff::IMAGE_SCN_ALIGN_16BYTES,
                32 => coff::IMAGE_SCN_ALIGN_32BYTES,
                64 => coff::IMAGE_SCN_ALIGN_64BYTES,
                128 => coff::IMAGE_SCN_ALIGN_128BYTES,
                256 => coff::IMAGE_SCN_ALIGN_256BYTES,
                512 => coff::IMAGE_SCN_ALIGN_512BYTES,
                1024 => coff::IMAGE_SCN_ALIGN_1024BYTES,
                2048 => coff::IMAGE_SCN_ALIGN_2048BYTES,
                4096 => coff::IMAGE_SCN_ALIGN_4096BYTES,
                8192 => coff::IMAGE_SCN_ALIGN_8192BYTES,
                _ => {
                    return Err(Error(format!(
                        "unimplemented section `{}` align {}",
                        section.name().unwrap_or(""),
                        section.align
                    )));
                }
            };
            let mut coff_section = coff::ImageSectionHeader {
                name: [0; 8],
                virtual_size: U32::default(),
                virtual_address: U32::default(),
                size_of_raw_data: U32::new(LE, section.size as u32),
                pointer_to_raw_data: U32::new(LE, section_offsets[index].offset as u32),
                pointer_to_relocations: U32::new(LE, section_offsets[index].reloc_offset as u32),
                pointer_to_linenumbers: U32::default(),
                number_of_relocations: if section.relocations.len() > 0xffff {
                    U16::new(LE, 0xffff)
                } else {
                    U16::new(LE, section.relocations.len() as u16)
                },
                number_of_linenumbers: U16::default(),
                characteristics: U32::new(LE, characteristics),
            };
            if section.name.len() <= 8 {
                coff_section.name[..section.name.len()].copy_from_slice(&section.name);
            } else {
                let mut str_offset = strtab.get_offset(section_offsets[index].str_id.unwrap());
                if str_offset <= 9_999_999 {
                    let mut name = [0; 7];
                    let mut len = 0;
                    if str_offset == 0 {
                        name[6] = b'0';
                        len = 1;
                    } else {
                        while str_offset != 0 {
                            let rem = (str_offset % 10) as u8;
                            str_offset /= 10;
                            name[6 - len] = b'0' + rem;
                            len += 1;
                        }
                    }
                    coff_section.name = [0; 8];
                    coff_section.name[0] = b'/';
                    coff_section.name[1..][..len].copy_from_slice(&name[7 - len..]);
                } else if str_offset as u64 <= 0xf_ffff_ffff {
                    coff_section.name[0] = b'/';
                    coff_section.name[1] = b'/';
                    for i in 0..6 {
                        let rem = (str_offset % 64) as u8;
                        str_offset /= 64;
                        let c = match rem {
                            0..=25 => b'A' + rem,
                            26..=51 => b'a' + rem - 26,
                            52..=61 => b'0' + rem - 52,
                            62 => b'+',
                            63 => b'/',
                            _ => unreachable!(),
                        };
                        coff_section.name[7 - i] = c;
                    }
                } else {
                    return Err(Error(format!("invalid section name offset {}", str_offset)));
                }
            }
            buffer.write(&coff_section);
        }

        // Write section data and relocations.
        for (index, section) in self.sections.iter().enumerate() {
            let len = section.data.len();
            if len != 0 {
                write_align(buffer, 4);
                debug_assert_eq!(section_offsets[index].offset, buffer.len());
                buffer.write_bytes(&section.data);
            }

            if !section.relocations.is_empty() {
                debug_assert_eq!(section_offsets[index].reloc_offset, buffer.len());
                if section.relocations.len() > 0xffff {
                    let coff_relocation = coff::ImageRelocation {
                        virtual_address: U32Bytes::new(LE, section.relocations.len() as u32 + 1),
                        symbol_table_index: U32Bytes::new(LE, 0),
                        typ: U16Bytes::new(LE, 0),
                    };
                    buffer.write(&coff_relocation);
                }
                for reloc in &section.relocations {
                    //assert!(reloc.implicit_addend);
                    let typ = match self.architecture {
                        Architecture::I386 => match (reloc.kind, reloc.size, reloc.addend) {
                            (RelocationKind::Absolute, 16, 0) => coff::IMAGE_REL_I386_DIR16,
                            (RelocationKind::Relative, 16, 0) => coff::IMAGE_REL_I386_REL16,
                            (RelocationKind::Absolute, 32, 0) => coff::IMAGE_REL_I386_DIR32,
                            (RelocationKind::ImageOffset, 32, 0) => coff::IMAGE_REL_I386_DIR32NB,
                            (RelocationKind::SectionIndex, 16, 0) => coff::IMAGE_REL_I386_SECTION,
                            (RelocationKind::SectionOffset, 32, 0) => coff::IMAGE_REL_I386_SECREL,
                            (RelocationKind::SectionOffset, 7, 0) => coff::IMAGE_REL_I386_SECREL7,
                            (RelocationKind::Relative, 32, -4) => coff::IMAGE_REL_I386_REL32,
                            (RelocationKind::Coff(x), _, _) => x,
                            _ => {
                                return Err(Error(format!("unimplemented relocation {:?}", reloc)));
                            }
                        },
                        Architecture::X86_64 => match (reloc.kind, reloc.size, reloc.addend) {
                            (RelocationKind::Absolute, 64, 0) => coff::IMAGE_REL_AMD64_ADDR64,
                            (RelocationKind::Absolute, 32, 0) => coff::IMAGE_REL_AMD64_ADDR32,
                            (RelocationKind::ImageOffset, 32, 0) => coff::IMAGE_REL_AMD64_ADDR32NB,
                            (RelocationKind::Relative, 32, -4) => coff::IMAGE_REL_AMD64_REL32,
                            (RelocationKind::Relative, 32, -5) => coff::IMAGE_REL_AMD64_REL32_1,
                            (RelocationKind::Relative, 32, -6) => coff::IMAGE_REL_AMD64_REL32_2,
                            (RelocationKind::Relative, 32, -7) => coff::IMAGE_REL_AMD64_REL32_3,
                            (RelocationKind::Relative, 32, -8) => coff::IMAGE_REL_AMD64_REL32_4,
                            (RelocationKind::Relative, 32, -9) => coff::IMAGE_REL_AMD64_REL32_5,
                            (RelocationKind::SectionIndex, 16, 0) => coff::IMAGE_REL_AMD64_SECTION,
                            (RelocationKind::SectionOffset, 32, 0) => coff::IMAGE_REL_AMD64_SECREL,
                            (RelocationKind::SectionOffset, 7, 0) => coff::IMAGE_REL_AMD64_SECREL7,
                            (RelocationKind::Coff(x), _, _) => x,
                            _ => {
                                return Err(Error(format!("unimplemented relocation {:?}", reloc)));
                            }
                        },
                        Architecture::Arm => match (reloc.kind, reloc.size, reloc.addend) {
                            (RelocationKind::Absolute, 32, 0) => coff::IMAGE_REL_ARM_ADDR32,
                            (RelocationKind::ImageOffset, 32, 0) => coff::IMAGE_REL_ARM_ADDR32NB,
                            (RelocationKind::Relative, 32, -4) => coff::IMAGE_REL_ARM_REL32,
                            (RelocationKind::SectionIndex, 16, 0) => coff::IMAGE_REL_ARM_SECTION,
                            (RelocationKind::SectionOffset, 32, 0) => coff::IMAGE_REL_ARM_SECREL,
                            (RelocationKind::Coff(x), _, _) => x,
                            _ => {
                                return Err(Error(format!("unimplemented relocation {:?}", reloc)));
                            }
                        },
                        Architecture::Aarch64 => match (reloc.kind, reloc.size, reloc.addend) {
                            (RelocationKind::Absolute, 32, 0) => coff::IMAGE_REL_ARM64_ADDR32,
                            (RelocationKind::ImageOffset, 32, 0) => coff::IMAGE_REL_ARM64_ADDR32NB,
                            (RelocationKind::SectionIndex, 16, 0) => coff::IMAGE_REL_ARM64_SECTION,
                            (RelocationKind::SectionOffset, 32, 0) => coff::IMAGE_REL_ARM64_SECREL,
                            (RelocationKind::Absolute, 64, 0) => coff::IMAGE_REL_ARM64_ADDR64,
                            (RelocationKind::Relative, 32, -4) => coff::IMAGE_REL_ARM64_REL32,
                            (RelocationKind::Coff(x), _, _) => x,
                            _ => {
                                return Err(Error(format!("unimplemented relocation {:?}", reloc)));
                            }
                        },
                        _ => {
                            return Err(Error(format!(
                                "unimplemented architecture {:?}",
                                self.architecture
                            )));
                        }
                    };
                    let coff_relocation = coff::ImageRelocation {
                        virtual_address: U32Bytes::new(LE, reloc.offset as u32),
                        symbol_table_index: U32Bytes::new(
                            LE,
                            symbol_offsets[reloc.symbol.0].index as u32,
                        ),
                        typ: U16Bytes::new(LE, typ),
                    };
                    buffer.write(&coff_relocation);
                }
            }
        }

        // Write symbols.
        debug_assert_eq!(symtab_offset, buffer.len());
        for (index, symbol) in self.symbols.iter().enumerate() {
            let mut name = &symbol.name[..];
            let section_number = match symbol.section {
                SymbolSection::None => {
                    debug_assert_eq!(symbol.kind, SymbolKind::File);
                    coff::IMAGE_SYM_DEBUG
                }
                SymbolSection::Undefined => coff::IMAGE_SYM_UNDEFINED,
                SymbolSection::Absolute => coff::IMAGE_SYM_ABSOLUTE,
                SymbolSection::Common => coff::IMAGE_SYM_UNDEFINED,
                SymbolSection::Section(id) => id.0 as u16 + 1,
            };
            let typ = if symbol.kind == SymbolKind::Text {
                coff::IMAGE_SYM_DTYPE_FUNCTION << coff::IMAGE_SYM_DTYPE_SHIFT
            } else {
                coff::IMAGE_SYM_TYPE_NULL
            };
            let storage_class = match symbol.kind {
                SymbolKind::File => {
                    // Name goes in auxilary symbol records.
                    name = b".file";
                    coff::IMAGE_SYM_CLASS_FILE
                }
                SymbolKind::Section => coff::IMAGE_SYM_CLASS_STATIC,
                SymbolKind::Label => coff::IMAGE_SYM_CLASS_LABEL,
                SymbolKind::Text | SymbolKind::Data | SymbolKind::Tls => {
                    match symbol.section {
                        SymbolSection::None => {
                            return Err(Error(format!(
                                "missing section for symbol `{}`",
                                symbol.name().unwrap_or("")
                            )));
                        }
                        SymbolSection::Undefined | SymbolSection::Common => {
                            coff::IMAGE_SYM_CLASS_EXTERNAL
                        }
                        SymbolSection::Absolute | SymbolSection::Section(_) => {
                            match symbol.scope {
                                // TODO: does this need aux symbol records too?
                                _ if symbol.weak => coff::IMAGE_SYM_CLASS_WEAK_EXTERNAL,
                                SymbolScope::Unknown => {
                                    return Err(Error(format!(
                                        "unimplemented symbol `{}` scope {:?}",
                                        symbol.name().unwrap_or(""),
                                        symbol.scope
                                    )));
                                }
                                SymbolScope::Compilation => coff::IMAGE_SYM_CLASS_STATIC,
                                SymbolScope::Linkage | SymbolScope::Dynamic => {
                                    coff::IMAGE_SYM_CLASS_EXTERNAL
                                }
                            }
                        }
                    }
                }
                SymbolKind::Unknown | SymbolKind::Null => {
                    return Err(Error(format!(
                        "unimplemented symbol `{}` kind {:?}",
                        symbol.name().unwrap_or(""),
                        symbol.kind
                    )));
                }
            };
            let number_of_aux_symbols = symbol_offsets[index].aux_count;
            let value = if symbol.section == SymbolSection::Common {
                symbol.size as u32
            } else {
                symbol.value as u32
            };
            let mut coff_symbol = coff::ImageSymbol {
                name: [0; 8],
                value: U32Bytes::new(LE, value),
                section_number: U16Bytes::new(LE, section_number as u16),
                typ: U16Bytes::new(LE, typ),
                storage_class,
                number_of_aux_symbols,
            };
            if name.len() <= 8 {
                coff_symbol.name[..name.len()].copy_from_slice(name);
            } else {
                let str_offset = strtab.get_offset(symbol_offsets[index].str_id.unwrap());
                coff_symbol.name[4..8].copy_from_slice(&u32::to_le_bytes(str_offset as u32));
            }
            buffer.write(&coff_symbol);

            // Write auxiliary symbols.
            match symbol.kind {
                SymbolKind::File => {
                    let aux_len = number_of_aux_symbols as usize * coff::IMAGE_SIZEOF_SYMBOL;
                    debug_assert!(aux_len >= symbol.name.len());
                    let old_len = buffer.len();
                    buffer.write_bytes(&symbol.name);
                    buffer.resize(old_len + aux_len);
                }
                SymbolKind::Section => {
                    debug_assert_eq!(number_of_aux_symbols, 1);
                    let section_index = symbol.section.id().unwrap().0;
                    let section = &self.sections[section_index];
                    let aux = coff::ImageAuxSymbolSection {
                        length: U32Bytes::new(LE, section.size as u32),
                        number_of_relocations: if section.relocations.len() > 0xffff {
                            U16Bytes::new(LE, 0xffff)
                        } else {
                            U16Bytes::new(LE, section.relocations.len() as u16)
                        },
                        number_of_linenumbers: U16Bytes::default(),
                        check_sum: U32Bytes::new(LE, checksum(section.data())),
                        number: U16Bytes::new(
                            LE,
                            section_offsets[section_index].associative_section,
                        ),
                        selection: section_offsets[section_index].selection,
                        reserved: 0,
                        // TODO: bigobj
                        high_number: U16Bytes::default(),
                    };
                    buffer.write(&aux);
                }
                _ => {
                    debug_assert_eq!(number_of_aux_symbols, 0);
                }
            }
        }

        // Write strtab section.
        debug_assert_eq!(strtab_offset, buffer.len());
        buffer.write_bytes(&u32::to_le_bytes(strtab_len as u32));
        buffer.write_bytes(&strtab_data);

        debug_assert_eq!(offset, buffer.len());

        Ok(())
    }
}

// JamCRC
fn checksum(data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new_with_initial(0xffff_ffff);
    hasher.update(data);
    !hasher.finalize()
}
