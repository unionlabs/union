# `object` Change Log

--------------------------------------------------------------------------------

## 0.28.4

Released 2022/05/09.

### Added

* Added `read::pe::DataDirectories::resource_directory`.
  [#425](https://github.com/gimli-rs/object/pull/425)
  [#427](https://github.com/gimli-rs/object/pull/427)

* Added PE support for more ARM relocations.
  [#428](https://github.com/gimli-rs/object/pull/428)

* Added support for `Architecture::LoongArch64`.
  [#430](https://github.com/gimli-rs/object/pull/430)
  [#432](https://github.com/gimli-rs/object/pull/432)

* Added `elf::EF_MIPS_ABI` and associated constants.
  [#433](https://github.com/gimli-rs/object/pull/433)

--------------------------------------------------------------------------------

## 0.28.3

Released 2022/01/19.

### Changed

* For the Mach-O support in `write::Object`, accept `RelocationKind::MachO` for all
  architecures, and accept `RelocationKind::Absolute` for ARM64.
  [#422](https://github.com/gimli-rs/object/pull/422)

### Added

* Added `pe::ImageDataDirectory::file_range`, `read::pe::SectionTable::pe_file_range_at`
  and `pe::ImageSectionHeader::pe_file_range_at`.
  [#421](https://github.com/gimli-rs/object/pull/421)

* Added `write::Object::add_coff_exports`.
  [#423](https://github.com/gimli-rs/object/pull/423)

--------------------------------------------------------------------------------

## 0.28.2

Released 2022/01/09.

### Changed

* Ignored errors for the Wasm extended name section in `read::WasmFile::parse`.
  [#408](https://github.com/gimli-rs/object/pull/408)

* Ignored errors for the COFF symbol table in `read::PeFile::parse`.
  [#410](https://github.com/gimli-rs/object/pull/410)

* Fixed handling of `SectionFlags::Coff` in `write::Object::coff_write`.
  [#412](https://github.com/gimli-rs/object/pull/412)

### Added

* Added `read::ObjectSegment::flags`.
  [#416](https://github.com/gimli-rs/object/pull/416)
  [#418](https://github.com/gimli-rs/object/pull/418)

--------------------------------------------------------------------------------

## 0.28.1

Released 2021/12/12.

### Changed

* Fixed `read::elf::SymbolTable::shndx_section`.
  [#405](https://github.com/gimli-rs/object/pull/405)

* Fixed build warnings.
  [#405](https://github.com/gimli-rs/object/pull/405)
  [#406](https://github.com/gimli-rs/object/pull/406)

--------------------------------------------------------------------------------

## 0.28.0

Released 2021/12/12.

### Breaking changes

* `write_core` feature no longer enables `std` support. Use `write_std` instead.
  [#400](https://github.com/gimli-rs/object/pull/400)

* Multiple changes related to Mach-O split dyld cache support.
  [#398](https://github.com/gimli-rs/object/pull/398)

### Added

* Added `write::pe::Writer::write_file_align`.
  [#397](https://github.com/gimli-rs/object/pull/397)

* Added support for Mach-O split dyld cache.
  [#398](https://github.com/gimli-rs/object/pull/398)

* Added support for `IMAGE_SCN_LNK_NRELOC_OVFL` when reading and writing COFF.
  [#399](https://github.com/gimli-rs/object/pull/399)

* Added `write::elf::Writer::reserve_null_symbol_index`.
  [#402](https://github.com/gimli-rs/object/pull/402)

--------------------------------------------------------------------------------

## 0.27.1

Released 2021/10/22.

### Changed

* Fixed build error with older Rust versions due to cargo resolver version.

--------------------------------------------------------------------------------

## 0.27.0

Released 2021/10/17.

### Breaking changes

* Changed `read::elf` to use `SectionIndex` instead of `usize` in more places.
  [#341](https://github.com/gimli-rs/object/pull/341)

* Changed some `read::elf` section methods to additionally return the linked section index.
  [#341](https://github.com/gimli-rs/object/pull/341)

* Changed `read::pe::ImageNtHeaders::parse` to return `DataDirectories` instead of a slice.
  [#357](https://github.com/gimli-rs/object/pull/357)

* Deleted `value` parameter for `write:WritableBuffer::resize`.
  [#369](https://github.com/gimli-rs/object/pull/369)

* Changed `write::Object` and `write::Section` to use `Cow` for section data.
  This added a lifetime parameter, which existing users can set to `'static`.
  [#370](https://github.com/gimli-rs/object/pull/370)

### Changed

* Fixed parsing when PE import directory has zero size.
  [#341](https://github.com/gimli-rs/object/pull/341)

* Fixed parsing when PE import directory has zero for original first thunk.
  [#385](https://github.com/gimli-rs/object/pull/385)
  [#387](https://github.com/gimli-rs/object/pull/387)

* Fixed parsing when PE export directory has zero number of names.
  [#353](https://github.com/gimli-rs/object/pull/353)

* Fixed parsing when PE export directory has zero number of names and addresses.
  [#362](https://github.com/gimli-rs/object/pull/362)

* Fixed parsing when PE sections are contiguous.
  [#354](https://github.com/gimli-rs/object/pull/354)

* Fixed `std` feature for `indexmap` dependency.
  [#374](https://github.com/gimli-rs/object/pull/374)

* Fixed overflow in COFF section name offset parsing.
  [#390](https://github.com/gimli-rs/object/pull/390)

### Added

* Added `name_bytes` methods to unified `read` traits.
  [#351](https://github.com/gimli-rs/object/pull/351)

* Added `read::Object::kind`.
  [#352](https://github.com/gimli-rs/object/pull/352)

* Added `read::elf::VersionTable` and related helpers.
  [#341](https://github.com/gimli-rs/object/pull/341)

* Added `read::elf::SectionTable::dynamic` and related helpers.
  [#345](https://github.com/gimli-rs/object/pull/345)

* Added `read::coff::SectionTable::max_section_file_offset`.
  [#344](https://github.com/gimli-rs/object/pull/344)

* Added `read::pe::ExportTable` and related helpers.
  [#349](https://github.com/gimli-rs/object/pull/349)
  [#353](https://github.com/gimli-rs/object/pull/353)

* Added `read::pe::ImportTable` and related helpers.
  [#357](https://github.com/gimli-rs/object/pull/357)

* Added `read::pe::DataDirectories` and related helpers.
  [#357](https://github.com/gimli-rs/object/pull/357)
  [#384](https://github.com/gimli-rs/object/pull/384)

* Added `read::pe::RichHeaderInfo` and related helpers.
  [#375](https://github.com/gimli-rs/object/pull/375)
  [#379](https://github.com/gimli-rs/object/pull/379)

* Added `read::pe::RelocationBlocks` and related helpers.
  [#378](https://github.com/gimli-rs/object/pull/378)

* Added `write::elf::Writer`.
  [#350](https://github.com/gimli-rs/object/pull/350)

* Added `write::pe::Writer`.
  [#382](https://github.com/gimli-rs/object/pull/382)
  [#388](https://github.com/gimli-rs/object/pull/388)

* Added `write::Section::data/data_mut`.
  [#367](https://github.com/gimli-rs/object/pull/367)

* Added `write::Object::write_stream`.
  [#369](https://github.com/gimli-rs/object/pull/369)

* Added MIPSr6 ELF header flag definitions.
  [#372](https://github.com/gimli-rs/object/pull/372)

--------------------------------------------------------------------------------

## 0.26.2

Released 2021/08/28.

### Added

* Added support for 64-bit symbol table names to `read::archive`.
  [#366](https://github.com/gimli-rs/object/pull/366)

--------------------------------------------------------------------------------

## 0.26.1

Released 2021/08/19.

### Changed

* Activate `memchr`'s `rustc-dep-of-std` feature
  [#356](https://github.com/gimli-rs/object/pull/356)

--------------------------------------------------------------------------------

## 0.26.0

Released 2021/07/26.

### Breaking changes

* Changed `ReadRef::read_bytes_at_until` to accept a range parameter.
  [#326](https://github.com/gimli-rs/object/pull/326)

* Added `ReadRef` type parameter to `read::StringTable` and types that
  contain it. String table entries are now only read as required.
  [#326](https://github.com/gimli-rs/object/pull/326)

* Changed result type of `read::elf::SectionHeader::data` and `data_as_array`.
  [#332](https://github.com/gimli-rs/object/pull/332)

* Moved `pod::WritableBuffer` to `write::WritableBuffer`.
  Renamed `WritableBuffer::extend` to `write_bytes`.
  Added more provided methods to `WritableBuffer`.
  [#335](https://github.com/gimli-rs/object/pull/335)

* Moved `pod::Bytes` to `read::Bytes`.
  [#336](https://github.com/gimli-rs/object/pull/336)

* Added `is_mips64el` parameter to `elf::Rela64::r_info/set_r_info`.
  [#337](https://github.com/gimli-rs/object/pull/337)

### Changed

* Removed `alloc` dependency when no features are enabled.
  [#336](https://github.com/gimli-rs/object/pull/336)

### Added

* Added `read::pe::PeFile` methods: `section_table`, `data_directory`, and `data`.
  [#324](https://github.com/gimli-rs/object/pull/324)

* Added more ELF definitions.
  [#332](https://github.com/gimli-rs/object/pull/332)

* Added `read::elf::SectionTable` methods for hash tables and symbol version
  information.
  [#332](https://github.com/gimli-rs/object/pull/332)

* Added PE RISC-V definitions.
  [#333](https://github.com/gimli-rs/object/pull/333)

* Added `WritableBuffer` implementation for `Vec`.
  [#335](https://github.com/gimli-rs/object/pull/335)

--------------------------------------------------------------------------------

## 0.25.3

Released 2021/06/12.

### Added

* Added `RelocationEncoding::AArch64Call`.
  [#322](https://github.com/gimli-rs/object/pull/322)

--------------------------------------------------------------------------------

## 0.25.2

Released 2021/06/04.

### Added

* Added `Architecture::X86_64_X32`.
  [#320](https://github.com/gimli-rs/object/pull/320)

--------------------------------------------------------------------------------

## 0.25.1

Released 2021/06/03.

### Changed

* write: Fix choice of `SHT_REL` or `SHT_RELA` for most architectures.
  [#318](https://github.com/gimli-rs/object/pull/318)

* write: Fix relocation encoding for MIPS64EL.
  [#318](https://github.com/gimli-rs/object/pull/318)

--------------------------------------------------------------------------------

## 0.25.0

Released 2021/06/02.

### Breaking changes

* Added `non_exhaustive` to most public enums.
  [#306](https://github.com/gimli-rs/object/pull/306)

* `MachHeader::parse` and `MachHeader::load_commands` now require a header offset.
  [#304](https://github.com/gimli-rs/object/pull/304)

* Added `ReadRef::read_bytes_at_until`.
  [#308](https://github.com/gimli-rs/object/pull/308)

* `PeFile::entry`, `PeSection::address` and `PeSegment::address` now return a
  virtual address instead of a RVA.
  [#315](https://github.com/gimli-rs/object/pull/315)

### Added

* Added `pod::from_bytes_mut`, `pod::slice_from_bytes_mut`, `pod::bytes_of_mut`,
  and `pod::bytes_of_slice_mut`.
  [#296](https://github.com/gimli-rs/object/pull/296)
  [#297](https://github.com/gimli-rs/object/pull/297)

* Added `Object::pdb_info`.
  [#298](https://github.com/gimli-rs/object/pull/298)

* Added `read::macho::DyldCache`, other associated definitions,
  and support for these in the examples.
  [#308](https://github.com/gimli-rs/object/pull/308)

* Added more architecture support.
  [#303](https://github.com/gimli-rs/object/pull/303)
  [#309](https://github.com/gimli-rs/object/pull/309)

* Derive more traits for enums.
  [#311](https://github.com/gimli-rs/object/pull/311)

* Added `Object::relative_address_base`.
  [#315](https://github.com/gimli-rs/object/pull/315)

### Changed

* Improved performance for string parsing.
  [#302](https://github.com/gimli-rs/object/pull/302)

* `objdump` example allows selecting container members.
  [#308](https://github.com/gimli-rs/object/pull/308)
