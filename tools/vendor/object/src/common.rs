/// A CPU architecture.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Architecture {
    Unknown,
    Aarch64,
    #[allow(non_camel_case_types)]
    Aarch64_Ilp32,
    Arm,
    Avr,
    Bpf,
    I386,
    X86_64,
    #[allow(non_camel_case_types)]
    X86_64_X32,
    Hexagon,
    LoongArch64,
    Mips,
    Mips64,
    Msp430,
    PowerPc,
    PowerPc64,
    Riscv32,
    Riscv64,
    S390x,
    Sbf,
    Sparc64,
    Wasm32,
    Xtensa,
}

impl Architecture {
    /// The size of an address value for this architecture.
    ///
    /// Returns `None` for unknown architectures.
    pub fn address_size(self) -> Option<AddressSize> {
        match self {
            Architecture::Unknown => None,
            Architecture::Aarch64 => Some(AddressSize::U64),
            Architecture::Aarch64_Ilp32 => Some(AddressSize::U32),
            Architecture::Arm => Some(AddressSize::U32),
            Architecture::Avr => Some(AddressSize::U8),
            Architecture::Bpf => Some(AddressSize::U64),
            Architecture::I386 => Some(AddressSize::U32),
            Architecture::X86_64 => Some(AddressSize::U64),
            Architecture::X86_64_X32 => Some(AddressSize::U32),
            Architecture::Hexagon => Some(AddressSize::U32),
            Architecture::LoongArch64 => Some(AddressSize::U64),
            Architecture::Mips => Some(AddressSize::U32),
            Architecture::Mips64 => Some(AddressSize::U64),
            Architecture::Msp430 => Some(AddressSize::U16),
            Architecture::PowerPc => Some(AddressSize::U32),
            Architecture::PowerPc64 => Some(AddressSize::U64),
            Architecture::Riscv32 => Some(AddressSize::U32),
            Architecture::Riscv64 => Some(AddressSize::U64),
            Architecture::S390x => Some(AddressSize::U64),
            Architecture::Sbf => Some(AddressSize::U64),
            Architecture::Sparc64 => Some(AddressSize::U64),
            Architecture::Wasm32 => Some(AddressSize::U32),
            Architecture::Xtensa => Some(AddressSize::U32),
        }
    }
}

/// The size of an address value for an architecture.
///
/// This may differ from the address size supported by the file format (such as for COFF).
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum AddressSize {
    U8 = 1,
    U16 = 2,
    U32 = 4,
    U64 = 8,
}

impl AddressSize {
    /// The size in bytes of an address value.
    #[inline]
    pub fn bytes(self) -> u8 {
        self as u8
    }
}

/// A binary file format.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BinaryFormat {
    Coff,
    Elf,
    MachO,
    Pe,
    Wasm,
    Xcoff,
}

/// The kind of a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SectionKind {
    /// The section kind is unknown.
    Unknown,
    /// An executable code section.
    ///
    /// Example ELF sections: `.text`
    ///
    /// Example Mach-O sections: `__TEXT/__text`
    Text,
    /// A data section.
    ///
    /// Example ELF sections: `.data`
    ///
    /// Example Mach-O sections: `__DATA/__data`
    Data,
    /// A read only data section.
    ///
    /// Example ELF sections: `.rodata`
    ///
    /// Example Mach-O sections: `__TEXT/__const`, `__DATA/__const`, `__TEXT/__literal4`
    ReadOnlyData,
    /// A read only data section with relocations.
    ///
    /// This is the same as either `Data` or `ReadOnlyData`, depending on the file format.
    /// This value is only used in the API for writing files. It is never returned when reading files.
    ReadOnlyDataWithRel,
    /// A loadable string section.
    ///
    /// Example ELF sections: `.rodata.str`
    ///
    /// Example Mach-O sections: `__TEXT/__cstring`
    ReadOnlyString,
    /// An uninitialized data section.
    ///
    /// Example ELF sections: `.bss`
    ///
    /// Example Mach-O sections: `__DATA/__bss`
    UninitializedData,
    /// An uninitialized common data section.
    ///
    /// Example Mach-O sections: `__DATA/__common`
    Common,
    /// A TLS data section.
    ///
    /// Example ELF sections: `.tdata`
    ///
    /// Example Mach-O sections: `__DATA/__thread_data`
    Tls,
    /// An uninitialized TLS data section.
    ///
    /// Example ELF sections: `.tbss`
    ///
    /// Example Mach-O sections: `__DATA/__thread_bss`
    UninitializedTls,
    /// A TLS variables section.
    ///
    /// This contains TLS variable structures, rather than the variable initializers.
    ///
    /// Example Mach-O sections: `__DATA/__thread_vars`
    TlsVariables,
    /// A non-loadable string section.
    ///
    /// Example ELF sections: `.comment`, `.debug_str`
    OtherString,
    /// Some other non-loadable section.
    ///
    /// Example ELF sections: `.debug_info`
    Other,
    /// Debug information.
    ///
    /// Example Mach-O sections: `__DWARF/__debug_info`
    Debug,
    /// Information for the linker.
    ///
    /// Example COFF sections: `.drectve`
    Linker,
    /// ELF note section.
    Note,
    /// Metadata such as symbols or relocations.
    ///
    /// Example ELF sections: `.symtab`, `.strtab`, `.group`
    Metadata,
    /// Some other ELF section type.
    ///
    /// This is the `sh_type` field in the section header.
    /// The meaning may be dependent on the architecture.
    Elf(u32),
}

impl SectionKind {
    /// Return true if this section contains zerofill data.
    pub fn is_bss(self) -> bool {
        self == SectionKind::UninitializedData
            || self == SectionKind::UninitializedTls
            || self == SectionKind::Common
    }
}

/// The selection kind for a COMDAT section group.
///
/// This determines the way in which the linker resolves multiple definitions of the COMDAT
/// sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ComdatKind {
    /// The selection kind is unknown.
    Unknown,
    /// Multiple definitions are allowed.
    ///
    /// An arbitrary definition is selected, and the rest are removed.
    ///
    /// This is the only supported selection kind for ELF.
    Any,
    /// Multiple definitions are not allowed.
    ///
    /// This is used to group sections without allowing duplicates.
    NoDuplicates,
    /// Multiple definitions must have the same size.
    ///
    /// An arbitrary definition is selected, and the rest are removed.
    SameSize,
    /// Multiple definitions must match exactly.
    ///
    /// An arbitrary definition is selected, and the rest are removed.
    ExactMatch,
    /// Multiple definitions are allowed, and the largest is selected.
    ///
    /// An arbitrary definition with the largest size is selected, and the rest are removed.
    Largest,
    /// Multiple definitions are allowed, and the newest is selected.
    Newest,
}

/// The kind of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SymbolKind {
    /// The symbol kind is unknown.
    Unknown,
    /// The symbol is a null placeholder.
    Null,
    /// The symbol is for executable code.
    Text,
    /// The symbol is for a data object.
    Data,
    /// The symbol is for a section.
    Section,
    /// The symbol is the name of a file. It precedes symbols within that file.
    File,
    /// The symbol is for a code label.
    Label,
    /// The symbol is for a thread local storage entity.
    Tls,
}

/// A symbol scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolScope {
    /// Unknown scope.
    Unknown,
    /// Symbol is visible to the compilation unit.
    Compilation,
    /// Symbol is visible to the static linkage unit.
    Linkage,
    /// Symbol is visible to dynamically linked objects.
    Dynamic,
}

/// The operation used to calculate the result of the relocation.
///
/// The relocation descriptions use the following definitions. Note that
/// these definitions probably don't match any ELF ABI.
///
/// * A - The value of the addend.
/// * G - The address of the symbol's entry within the global offset table.
/// * L - The address of the symbol's entry within the procedure linkage table.
/// * P - The address of the place of the relocation.
/// * S - The address of the symbol.
/// * GotBase - The address of the global offset table.
/// * Image - The base address of the image.
/// * Section - The address of the section containing the symbol.
///
/// 'XxxRelative' means 'Xxx + A - P'.  'XxxOffset' means 'S + A - Xxx'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RelocationKind {
    /// S + A
    Absolute,
    /// S + A - P
    Relative,
    /// G + A - GotBase
    Got,
    /// G + A - P
    GotRelative,
    /// GotBase + A - P
    GotBaseRelative,
    /// S + A - GotBase
    GotBaseOffset,
    /// L + A - P
    PltRelative,
    /// S + A - Image
    ImageOffset,
    /// S + A - Section
    SectionOffset,
    /// The index of the section containing the symbol.
    SectionIndex,
    /// Some other ELF relocation. The value is dependent on the architecture.
    Elf(u32),
    /// Some other Mach-O relocation. The value is dependent on the architecture.
    MachO {
        /// The relocation type.
        value: u8,
        /// Whether the relocation is relative to the place.
        relative: bool,
    },
    /// Some other COFF relocation. The value is dependent on the architecture.
    Coff(u16),
    /// Some other XCOFF relocation.
    Xcoff(u8),
}

/// Information about how the result of the relocation operation is encoded in the place.
///
/// This is usually architecture specific, such as specifying an addressing mode or
/// a specific instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RelocationEncoding {
    /// Generic encoding.
    Generic,

    /// x86 sign extension at runtime.
    ///
    /// Used with `RelocationKind::Absolute`.
    X86Signed,
    /// x86 rip-relative addressing.
    ///
    /// The `RelocationKind` must be PC relative.
    X86RipRelative,
    /// x86 rip-relative addressing in movq instruction.
    ///
    /// The `RelocationKind` must be PC relative.
    X86RipRelativeMovq,
    /// x86 branch instruction.
    ///
    /// The `RelocationKind` must be PC relative.
    X86Branch,

    /// s390x PC-relative offset shifted right by one bit.
    ///
    /// The `RelocationKind` must be PC relative.
    S390xDbl,

    /// AArch64 call target.
    ///
    /// The `RelocationKind` must be PC relative.
    AArch64Call,

    /// LoongArch branch offset with two trailing zeros.
    ///
    /// The `RelocationKind` must be PC relative.
    LoongArchBranch,
}

/// File flags that are specific to each file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FileFlags {
    /// No file flags.
    None,
    /// ELF file flags.
    Elf {
        /// `os_abi` field in the ELF file header.
        os_abi: u8,
        /// `abi_version` field in the ELF file header.
        abi_version: u8,
        /// `e_flags` field in the ELF file header.
        e_flags: u32,
    },
    /// Mach-O file flags.
    MachO {
        /// `flags` field in the Mach-O file header.
        flags: u32,
    },
    /// COFF file flags.
    Coff {
        /// `Characteristics` field in the COFF file header.
        characteristics: u16,
    },
    /// XCOFF file flags.
    Xcoff {
        /// `f_flags` field in the XCOFF file header.
        f_flags: u16,
    },
}

/// Segment flags that are specific to each file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SegmentFlags {
    /// No segment flags.
    None,
    /// ELF segment flags.
    Elf {
        /// `p_flags` field in the segment header.
        p_flags: u32,
    },
    /// Mach-O segment flags.
    MachO {
        /// `flags` field in the segment header.
        flags: u32,
        /// `maxprot` field in the segment header.
        maxprot: u32,
        /// `initprot` field in the segment header.
        initprot: u32,
    },
    /// COFF segment flags.
    Coff {
        /// `Characteristics` field in the segment header.
        characteristics: u32,
    },
}

/// Section flags that are specific to each file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SectionFlags {
    /// No section flags.
    None,
    /// ELF section flags.
    Elf {
        /// `sh_flags` field in the section header.
        sh_flags: u64,
    },
    /// Mach-O section flags.
    MachO {
        /// `flags` field in the section header.
        flags: u32,
    },
    /// COFF section flags.
    Coff {
        /// `Characteristics` field in the section header.
        characteristics: u32,
    },
    /// XCOFF section flags.
    Xcoff {
        /// `s_flags` field in the section header.
        s_flags: u32,
    },
}

/// Symbol flags that are specific to each file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SymbolFlags<Section> {
    /// No symbol flags.
    None,
    /// ELF symbol flags.
    Elf {
        /// `st_info` field in the ELF symbol.
        st_info: u8,
        /// `st_other` field in the ELF symbol.
        st_other: u8,
    },
    /// Mach-O symbol flags.
    MachO {
        /// `n_desc` field in the Mach-O symbol.
        n_desc: u16,
    },
    /// COFF flags for a section symbol.
    CoffSection {
        /// `Selection` field in the auxiliary symbol for the section.
        selection: u8,
        /// `Number` field in the auxiliary symbol for the section.
        associative_section: Option<Section>,
    },
}
