// Allow clippy lints when building without clippy.
#![allow(unknown_lints)]

use gimli::{AttributeValue, UnitHeader};
use object::{Object, ObjectSection};
use rayon::prelude::*;
use std::borrow::{Borrow, Cow};
use std::env;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;
use typed_arena::Arena;

trait Reader: gimli::Reader<Offset = usize> + Send + Sync {
    type SyncSendEndian: gimli::Endianity + Send + Sync;
}

impl<'input, Endian> Reader for gimli::EndianSlice<'input, Endian>
where
    Endian: gimli::Endianity + Send + Sync,
{
    type SyncSendEndian = Endian;
}

struct ErrorWriter<W: Write + Send> {
    inner: Mutex<(W, usize)>,
    path: PathBuf,
}

impl<W: Write + Send> ErrorWriter<W> {
    #[allow(clippy::needless_pass_by_value)]
    fn error(&self, s: String) {
        let mut lock = self.inner.lock().unwrap();
        writeln!(&mut lock.0, "DWARF error in {}: {}", self.path.display(), s).unwrap();
        lock.1 += 1;
    }
}

fn main() {
    let mut w = BufWriter::new(io::stdout());
    let mut errors = 0;
    for arg in env::args_os().skip(1) {
        let path = Path::new(&arg);
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open file '{}': {}", path.display(), err);
                errors += 1;
                continue;
            }
        };
        let file = match unsafe { memmap2::Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(err) => {
                eprintln!("Failed to map file '{}': {}", path.display(), &err);
                errors += 1;
                continue;
            }
        };
        let file = match object::File::parse(&*file) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to parse file '{}': {}", path.display(), err);
                errors += 1;
                continue;
            }
        };

        let endian = if file.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };
        let mut error_writer = ErrorWriter {
            inner: Mutex::new((&mut w, 0)),
            path: path.to_owned(),
        };
        validate_file(&mut error_writer, &file, endian);
        errors += error_writer.inner.into_inner().unwrap().1;
    }
    // Flush any errors.
    drop(w);
    if errors > 0 {
        process::exit(1);
    }
}

fn validate_file<W, Endian>(w: &mut ErrorWriter<W>, file: &object::File, endian: Endian)
where
    W: Write + Send,
    Endian: gimli::Endianity + Send + Sync,
{
    let arena = Arena::new();

    fn load_section<'a, 'file, 'input, S, Endian>(
        arena: &'a Arena<Cow<'file, [u8]>>,
        file: &'file object::File<'input>,
        endian: Endian,
    ) -> S
    where
        S: gimli::Section<gimli::EndianSlice<'a, Endian>>,
        Endian: gimli::Endianity + Send + Sync,
        'file: 'input,
        'a: 'file,
    {
        let data = match file.section_by_name(S::section_name()) {
            Some(ref section) => section
                .uncompressed_data()
                .unwrap_or(Cow::Borrowed(&[][..])),
            None => Cow::Borrowed(&[][..]),
        };
        let data_ref = (*arena.alloc(data)).borrow();
        S::from(gimli::EndianSlice::new(data_ref, endian))
    }

    // Variables representing sections of the file. The type of each is inferred from its use in the
    // validate_info function below.
    let debug_abbrev = &load_section(&arena, file, endian);
    let debug_info = &load_section(&arena, file, endian);

    validate_info(w, debug_info, debug_abbrev);
}

struct UnitSummary {
    // True if we successfully parsed all the DIEs and attributes in the compilation unit
    internally_valid: bool,
    offset: gimli::DebugInfoOffset,
    die_offsets: Vec<gimli::UnitOffset>,
    global_die_references: Vec<(gimli::UnitOffset, gimli::DebugInfoOffset)>,
}

fn validate_info<W, R>(
    w: &mut ErrorWriter<W>,
    debug_info: &gimli::DebugInfo<R>,
    debug_abbrev: &gimli::DebugAbbrev<R>,
) where
    W: Write + Send,
    R: Reader,
{
    let mut units = Vec::new();
    let mut units_iter = debug_info.units();
    let mut last_offset = 0;
    loop {
        let u = match units_iter.next() {
            Err(err) => {
                w.error(format!(
                    "Can't read unit header at offset {:#x}, stopping reading units: {}",
                    last_offset, err
                ));
                break;
            }
            Ok(None) => break,
            Ok(Some(u)) => u,
        };
        last_offset = u.offset().as_debug_info_offset().unwrap().0 + u.length_including_self();
        units.push(u);
    }
    let process_unit = |unit: UnitHeader<R>| -> UnitSummary {
        let unit_offset = unit.offset().as_debug_info_offset().unwrap();
        let mut ret = UnitSummary {
            internally_valid: false,
            offset: unit_offset,
            die_offsets: Vec::new(),
            global_die_references: Vec::new(),
        };
        let abbrevs = match unit.abbreviations(debug_abbrev) {
            Ok(abbrevs) => abbrevs,
            Err(err) => {
                w.error(format!(
                    "Invalid abbrevs for unit {:#x}: {}",
                    unit_offset.0, &err
                ));
                return ret;
            }
        };
        let mut entries = unit.entries(&abbrevs);
        let mut unit_refs = Vec::new();
        loop {
            let (_, entry) = match entries.next_dfs() {
                Err(err) => {
                    w.error(format!(
                        "Invalid DIE for unit {:#x}: {}",
                        unit_offset.0, &err
                    ));
                    return ret;
                }
                Ok(None) => break,
                Ok(Some(entry)) => entry,
            };
            ret.die_offsets.push(entry.offset());

            let mut attrs = entry.attrs();
            loop {
                let attr = match attrs.next() {
                    Err(err) => {
                        w.error(format!(
                            "Invalid attribute for unit {:#x} at DIE {:#x}: {}",
                            unit_offset.0,
                            entry.offset().0,
                            &err
                        ));
                        return ret;
                    }
                    Ok(None) => break,
                    Ok(Some(attr)) => attr,
                };
                match attr.value() {
                    AttributeValue::UnitRef(offset) => {
                        unit_refs.push((entry.offset(), offset));
                    }
                    AttributeValue::DebugInfoRef(offset) => {
                        ret.global_die_references.push((entry.offset(), offset));
                    }
                    _ => (),
                }
            }
        }
        ret.internally_valid = true;
        ret.die_offsets.shrink_to_fit();
        ret.global_die_references.shrink_to_fit();

        // Check intra-unit references
        for (from, to) in unit_refs {
            if ret.die_offsets.binary_search(&to).is_err() {
                w.error(format!(
                    "Invalid intra-unit reference in unit {:#x} from DIE {:#x} to {:#x}",
                    unit_offset.0, from.0, to.0
                ));
            }
        }

        ret
    };
    let processed_units = units.into_par_iter().map(process_unit).collect::<Vec<_>>();

    let check_unit = |summary: &UnitSummary| {
        if !summary.internally_valid {
            return;
        }
        for &(from, to) in summary.global_die_references.iter() {
            let u = match processed_units.binary_search_by_key(&to, |v| v.offset) {
                Ok(i) => &processed_units[i],
                Err(i) => {
                    if i > 0 {
                        &processed_units[i - 1]
                    } else {
                        w.error(format!("Invalid cross-unit reference in unit {:#x} from DIE {:#x} to global DIE {:#x}: no unit found",
                                        summary.offset.0, from.0, to.0));
                        continue;
                    }
                }
            };
            if !u.internally_valid {
                continue;
            }
            let to_offset = gimli::UnitOffset(to.0 - u.offset.0);
            if u.die_offsets.binary_search(&to_offset).is_err() {
                w.error(format!("Invalid cross-unit reference in unit {:#x} from DIE {:#x} to global DIE {:#x}: unit at {:#x} contains no DIE {:#x}",
                                summary.offset.0, from.0, to.0, u.offset.0, to_offset.0));
            }
        }
    };
    processed_units.par_iter().for_each(check_unit);
}
