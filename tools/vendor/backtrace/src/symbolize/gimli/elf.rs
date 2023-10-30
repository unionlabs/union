use super::mystd::ffi::{OsStr, OsString};
use super::mystd::fs;
use super::mystd::os::unix::ffi::{OsStrExt, OsStringExt};
use super::mystd::path::{Path, PathBuf};
use super::Either;
use super::{Context, Mapping, Stash, Vec};
use core::convert::{TryFrom, TryInto};
use core::str;
use object::elf::{ELFCOMPRESS_ZLIB, ELF_NOTE_GNU, NT_GNU_BUILD_ID, SHF_COMPRESSED};
use object::read::elf::{CompressionHeader, FileHeader, SectionHeader, SectionTable, Sym};
use object::read::StringTable;
use object::{BigEndian, Bytes, NativeEndian};

#[cfg(target_pointer_width = "32")]
type Elf = object::elf::FileHeader32<NativeEndian>;
#[cfg(target_pointer_width = "64")]
type Elf = object::elf::FileHeader64<NativeEndian>;

impl Mapping {
    pub fn new(path: &Path) -> Option<Mapping> {
        let map = super::mmap(path)?;
        Mapping::mk_or_other(map, |map, stash| {
            let object = Object::parse(&map)?;

            // Try to locate an external debug file using the build ID.
            if let Some(path_debug) = object.build_id().and_then(locate_build_id) {
                if let Some(mapping) = Mapping::new_debug(path_debug, None) {
                    return Some(Either::A(mapping));
                }
            }

            // Try to locate an external debug file using the GNU debug link section.
            if let Some((path_debug, crc)) = object.gnu_debuglink_path(path) {
                if let Some(mapping) = Mapping::new_debug(path_debug, Some(crc)) {
                    return Some(Either::A(mapping));
                }
            }

            Context::new(stash, object, None).map(Either::B)
        })
    }

    /// Load debuginfo from an external debug file.
    fn new_debug(path: PathBuf, crc: Option<u32>) -> Option<Mapping> {
        let map = super::mmap(&path)?;
        Mapping::mk(map, |map, stash| {
            let object = Object::parse(&map)?;

            if let Some(_crc) = crc {
                // TODO: check crc
            }

            // Try to locate a supplementary object file.
            if let Some((path_sup, build_id_sup)) = object.gnu_debugaltlink_path(&path) {
                if let Some(map_sup) = super::mmap(&path_sup) {
                    let map_sup = stash.set_mmap_aux(map_sup);
                    if let Some(sup) = Object::parse(map_sup) {
                        if sup.build_id() == Some(build_id_sup) {
                            return Context::new(stash, object, Some(sup));
                        }
                    }
                }
            }

            Context::new(stash, object, None)
        })
    }
}

struct ParsedSym {
    address: u64,
    size: u64,
    name: u32,
}

pub struct Object<'a> {
    /// Zero-sized type representing the native endianness.
    ///
    /// We could use a literal instead, but this helps ensure correctness.
    endian: NativeEndian,
    /// The entire file data.
    data: &'a [u8],
    sections: SectionTable<'a, Elf>,
    strings: StringTable<'a>,
    /// List of pre-parsed and sorted symbols by base address.
    syms: Vec<ParsedSym>,
}

impl<'a> Object<'a> {
    fn parse(data: &'a [u8]) -> Option<Object<'a>> {
        let elf = Elf::parse(data).ok()?;
        let endian = elf.endian().ok()?;
        let sections = elf.sections(endian, data).ok()?;
        let mut syms = sections
            .symbols(endian, data, object::elf::SHT_SYMTAB)
            .ok()?;
        if syms.is_empty() {
            syms = sections
                .symbols(endian, data, object::elf::SHT_DYNSYM)
                .ok()?;
        }
        let strings = syms.strings();

        let mut syms = syms
            .iter()
            // Only look at function/object symbols. This mirrors what
            // libbacktrace does and in general we're only symbolicating
            // function addresses in theory. Object symbols correspond
            // to data, and maybe someone's crazy enough to have a
            // function go into static data?
            .filter(|sym| {
                let st_type = sym.st_type();
                st_type == object::elf::STT_FUNC || st_type == object::elf::STT_OBJECT
            })
            // skip anything that's in an undefined section header,
            // since it means it's an imported function and we're only
            // symbolicating with locally defined functions.
            .filter(|sym| sym.st_shndx(endian) != object::elf::SHN_UNDEF)
            .map(|sym| {
                let address = sym.st_value(endian).into();
                let size = sym.st_size(endian).into();
                let name = sym.st_name(endian);
                ParsedSym {
                    address,
                    size,
                    name,
                }
            })
            .collect::<Vec<_>>();
        syms.sort_unstable_by_key(|s| s.address);
        Some(Object {
            endian,
            data,
            sections,
            strings,
            syms,
        })
    }

    pub fn section(&self, stash: &'a Stash, name: &str) -> Option<&'a [u8]> {
        if let Some(section) = self.section_header(name) {
            let mut data = Bytes(section.data(self.endian, self.data).ok()?);

            // Check for DWARF-standard (gABI) compression, i.e., as generated
            // by ld's `--compress-debug-sections=zlib-gabi` flag.
            let flags: u64 = section.sh_flags(self.endian).into();
            if (flags & u64::from(SHF_COMPRESSED)) == 0 {
                // Not compressed.
                return Some(data.0);
            }

            let header = data.read::<<Elf as FileHeader>::CompressionHeader>().ok()?;
            if header.ch_type(self.endian) != ELFCOMPRESS_ZLIB {
                // Zlib compression is the only known type.
                return None;
            }
            let size = usize::try_from(header.ch_size(self.endian)).ok()?;
            let buf = stash.allocate(size);
            decompress_zlib(data.0, buf)?;
            return Some(buf);
        }

        // Check for the nonstandard GNU compression format, i.e., as generated
        // by ld's `--compress-debug-sections=zlib-gnu` flag. This means that if
        // we're actually asking for `.debug_info` then we need to look up a
        // section named `.zdebug_info`.
        if !name.starts_with(".debug_") {
            return None;
        }
        let debug_name = name[7..].as_bytes();
        let compressed_section = self
            .sections
            .iter()
            .filter_map(|header| {
                let name = self.sections.section_name(self.endian, header).ok()?;
                if name.starts_with(b".zdebug_") && &name[8..] == debug_name {
                    Some(header)
                } else {
                    None
                }
            })
            .next()?;
        let mut data = Bytes(compressed_section.data(self.endian, self.data).ok()?);
        if data.read_bytes(8).ok()?.0 != b"ZLIB\0\0\0\0" {
            return None;
        }
        let size = usize::try_from(data.read::<object::U32Bytes<_>>().ok()?.get(BigEndian)).ok()?;
        let buf = stash.allocate(size);
        decompress_zlib(data.0, buf)?;
        Some(buf)
    }

    fn section_header(&self, name: &str) -> Option<&<Elf as FileHeader>::SectionHeader> {
        self.sections
            .section_by_name(self.endian, name.as_bytes())
            .map(|(_index, section)| section)
    }

    pub fn search_symtab<'b>(&'b self, addr: u64) -> Option<&'b [u8]> {
        // Same sort of binary search as Windows above
        let i = match self.syms.binary_search_by_key(&addr, |sym| sym.address) {
            Ok(i) => i,
            Err(i) => i.checked_sub(1)?,
        };
        let sym = self.syms.get(i)?;
        if sym.address <= addr && addr <= sym.address + sym.size {
            self.strings.get(sym.name).ok()
        } else {
            None
        }
    }

    pub(super) fn search_object_map(&self, _addr: u64) -> Option<(&Context<'_>, u64)> {
        None
    }

    fn build_id(&self) -> Option<&'a [u8]> {
        for section in self.sections.iter() {
            if let Ok(Some(mut notes)) = section.notes(self.endian, self.data) {
                while let Ok(Some(note)) = notes.next() {
                    if note.name() == ELF_NOTE_GNU && note.n_type(self.endian) == NT_GNU_BUILD_ID {
                        return Some(note.desc());
                    }
                }
            }
        }
        None
    }

    // The contents of the ".gnu_debuglink" section is documented at:
    // https://sourceware.org/gdb/onlinedocs/gdb/Separate-Debug-Files.html
    fn gnu_debuglink_path(&self, path: &Path) -> Option<(PathBuf, u32)> {
        let section = self.section_header(".gnu_debuglink")?;
        let data = section.data(self.endian, self.data).ok()?;
        let len = data.iter().position(|x| *x == 0)?;
        let filename = &data[..len];
        let offset = (len + 1 + 3) & !3;
        let crc_bytes = data
            .get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())?;
        let crc = u32::from_ne_bytes(crc_bytes);
        let path_debug = locate_debuglink(path, filename)?;
        Some((path_debug, crc))
    }

    // The format of the ".gnu_debugaltlink" section is based on gdb.
    fn gnu_debugaltlink_path(&self, path: &Path) -> Option<(PathBuf, &'a [u8])> {
        let section = self.section_header(".gnu_debugaltlink")?;
        let data = section.data(self.endian, self.data).ok()?;
        let len = data.iter().position(|x| *x == 0)?;
        let filename = &data[..len];
        let build_id = &data[len + 1..];
        let path_sup = locate_debugaltlink(path, filename, build_id)?;
        Some((path_sup, build_id))
    }
}

fn decompress_zlib(input: &[u8], output: &mut [u8]) -> Option<()> {
    use miniz_oxide::inflate::core::inflate_flags::{
        TINFL_FLAG_PARSE_ZLIB_HEADER, TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
    };
    use miniz_oxide::inflate::core::{decompress, DecompressorOxide};
    use miniz_oxide::inflate::TINFLStatus;

    let (status, in_read, out_read) = decompress(
        &mut DecompressorOxide::new(),
        input,
        output,
        0,
        TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF | TINFL_FLAG_PARSE_ZLIB_HEADER,
    );
    if status == TINFLStatus::Done && in_read == input.len() && out_read == output.len() {
        Some(())
    } else {
        None
    }
}

const DEBUG_PATH: &[u8] = b"/usr/lib/debug";

fn debug_path_exists() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "freebsd", target_os = "linux"))] {
            use core::sync::atomic::{AtomicU8, Ordering};
            static DEBUG_PATH_EXISTS: AtomicU8 = AtomicU8::new(0);

            let mut exists = DEBUG_PATH_EXISTS.load(Ordering::Relaxed);
            if exists == 0 {
                exists = if Path::new(OsStr::from_bytes(DEBUG_PATH)).is_dir() {
                    1
                } else {
                    2
                };
                DEBUG_PATH_EXISTS.store(exists, Ordering::Relaxed);
            }
            exists == 1
        } else {
            false
        }
    }
}

/// Locate a debug file based on its build ID.
///
/// The format of build id paths is documented at:
/// https://sourceware.org/gdb/onlinedocs/gdb/Separate-Debug-Files.html
fn locate_build_id(build_id: &[u8]) -> Option<PathBuf> {
    const BUILD_ID_PATH: &[u8] = b"/usr/lib/debug/.build-id/";
    const BUILD_ID_SUFFIX: &[u8] = b".debug";

    if build_id.len() < 2 {
        return None;
    }

    if !debug_path_exists() {
        return None;
    }

    let mut path =
        Vec::with_capacity(BUILD_ID_PATH.len() + BUILD_ID_SUFFIX.len() + build_id.len() * 2 + 1);
    path.extend(BUILD_ID_PATH);
    path.push(hex(build_id[0] >> 4));
    path.push(hex(build_id[0] & 0xf));
    path.push(b'/');
    for byte in &build_id[1..] {
        path.push(hex(byte >> 4));
        path.push(hex(byte & 0xf));
    }
    path.extend(BUILD_ID_SUFFIX);
    Some(PathBuf::from(OsString::from_vec(path)))
}

fn hex(byte: u8) -> u8 {
    if byte < 10 {
        b'0' + byte
    } else {
        b'a' + byte - 10
    }
}

/// Locate a file specified in a `.gnu_debuglink` section.
///
/// `path` is the file containing the section.
/// `filename` is from the contents of the section.
///
/// Search order is based on gdb, documented at:
/// https://sourceware.org/gdb/onlinedocs/gdb/Separate-Debug-Files.html
///
/// gdb also allows the user to customize the debug search path, but we don't.
///
/// gdb also supports debuginfod, but we don't yet.
fn locate_debuglink(path: &Path, filename: &[u8]) -> Option<PathBuf> {
    let path = fs::canonicalize(path).ok()?;
    let parent = path.parent()?;
    let mut f = PathBuf::from(OsString::with_capacity(
        DEBUG_PATH.len() + parent.as_os_str().len() + filename.len() + 2,
    ));
    let filename = Path::new(OsStr::from_bytes(filename));

    // Try "/parent/filename" if it differs from "path"
    f.push(parent);
    f.push(filename);
    if f != path && f.is_file() {
        return Some(f);
    }

    // Try "/parent/.debug/filename"
    let mut s = OsString::from(f);
    s.clear();
    f = PathBuf::from(s);
    f.push(parent);
    f.push(".debug");
    f.push(filename);
    if f.is_file() {
        return Some(f);
    }

    if debug_path_exists() {
        // Try "/usr/lib/debug/parent/filename"
        let mut s = OsString::from(f);
        s.clear();
        f = PathBuf::from(s);
        f.push(OsStr::from_bytes(DEBUG_PATH));
        f.push(parent.strip_prefix("/").unwrap());
        f.push(filename);
        if f.is_file() {
            return Some(f);
        }
    }

    None
}

/// Locate a file specified in a `.gnu_debugaltlink` section.
///
/// `path` is the file containing the section.
/// `filename` and `build_id` are the contents of the section.
///
/// Search order is based on gdb:
/// - filename, which is either absolute or relative to `path`
/// - the build ID path under `BUILD_ID_PATH`
///
/// gdb also allows the user to customize the debug search path, but we don't.
///
/// gdb also supports debuginfod, but we don't yet.
fn locate_debugaltlink(path: &Path, filename: &[u8], build_id: &[u8]) -> Option<PathBuf> {
    let filename = Path::new(OsStr::from_bytes(filename));
    if filename.is_absolute() {
        if filename.is_file() {
            return Some(filename.into());
        }
    } else {
        let path = fs::canonicalize(path).ok()?;
        let parent = path.parent()?;
        let mut f = PathBuf::from(parent);
        f.push(filename);
        if f.is_file() {
            return Some(f);
        }
    }

    locate_build_id(build_id)
}
