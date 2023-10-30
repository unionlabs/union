//! Support for symbolication using the `gimli` crate on crates.io
//!
//! This is the default symbolication implementation for Rust.

use self::gimli::read::EndianSlice;
use self::gimli::NativeEndian as Endian;
use self::mmap::Mmap;
use self::stash::Stash;
use super::BytesOrWideString;
use super::ResolveWhat;
use super::SymbolName;
use addr2line::gimli;
use core::convert::TryInto;
use core::mem;
use core::u32;
use libc::c_void;
use mystd::ffi::OsString;
use mystd::fs::File;
use mystd::path::Path;
use mystd::prelude::v1::*;

#[cfg(backtrace_in_libstd)]
mod mystd {
    pub use crate::*;
}
#[cfg(not(backtrace_in_libstd))]
extern crate std as mystd;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[path = "gimli/mmap_windows.rs"]
        mod mmap;
    } else if #[cfg(any(
        target_os = "android",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "solaris",
        target_os = "illumos",
    ))] {
        #[path = "gimli/mmap_unix.rs"]
        mod mmap;
    } else {
        #[path = "gimli/mmap_fake.rs"]
        mod mmap;
    }
}

mod stash;

const MAPPINGS_CACHE_SIZE: usize = 4;

struct Mapping {
    // 'static lifetime is a lie to hack around lack of support for self-referential structs.
    cx: Context<'static>,
    _map: Mmap,
    _stash: Stash,
}

enum Either<A, B> {
    #[allow(dead_code)]
    A(A),
    B(B),
}

impl Mapping {
    /// Creates a `Mapping` by ensuring that the `data` specified is used to
    /// create a `Context` and it can only borrow from that or the `Stash` of
    /// decompressed sections or auxiliary data.
    fn mk<F>(data: Mmap, mk: F) -> Option<Mapping>
    where
        F: for<'a> FnOnce(&'a [u8], &'a Stash) -> Option<Context<'a>>,
    {
        Mapping::mk_or_other(data, move |data, stash| {
            let cx = mk(data, stash)?;
            Some(Either::B(cx))
        })
    }

    /// Creates a `Mapping` from `data`, or if the closure decides to, returns a
    /// different mapping.
    fn mk_or_other<F>(data: Mmap, mk: F) -> Option<Mapping>
    where
        F: for<'a> FnOnce(&'a [u8], &'a Stash) -> Option<Either<Mapping, Context<'a>>>,
    {
        let stash = Stash::new();
        let cx = match mk(&data, &stash)? {
            Either::A(mapping) => return Some(mapping),
            Either::B(cx) => cx,
        };
        Some(Mapping {
            // Convert to 'static lifetimes since the symbols should
            // only borrow `map` and `stash` and we're preserving them below.
            cx: unsafe { core::mem::transmute::<Context<'_>, Context<'static>>(cx) },
            _map: data,
            _stash: stash,
        })
    }
}

struct Context<'a> {
    dwarf: addr2line::Context<EndianSlice<'a, Endian>>,
    object: Object<'a>,
}

impl<'data> Context<'data> {
    fn new(
        stash: &'data Stash,
        object: Object<'data>,
        sup: Option<Object<'data>>,
    ) -> Option<Context<'data>> {
        let mut sections = gimli::Dwarf::load(|id| -> Result<_, ()> {
            let data = object.section(stash, id.name()).unwrap_or(&[]);
            Ok(EndianSlice::new(data, Endian))
        })
        .ok()?;

        if let Some(sup) = sup {
            sections
                .load_sup(|id| -> Result<_, ()> {
                    let data = sup.section(stash, id.name()).unwrap_or(&[]);
                    Ok(EndianSlice::new(data, Endian))
                })
                .ok()?;
        }
        let dwarf = addr2line::Context::from_dwarf(sections).ok()?;

        Some(Context { dwarf, object })
    }
}

fn mmap(path: &Path) -> Option<Mmap> {
    let file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len().try_into().ok()?;
    unsafe { Mmap::map(&file, len) }
}

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod coff;
        use self::coff::Object;
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
    ))] {
        mod macho;
        use self::macho::Object;
    } else {
        mod elf;
        use self::elf::Object;
    }
}

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod libs_windows;
        use libs_windows::native_libraries;
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
    ))] {
        mod libs_macos;
        use libs_macos::native_libraries;
    } else if #[cfg(target_os = "illumos")] {
        mod libs_illumos;
        use libs_illumos::native_libraries;
    } else if #[cfg(all(
        any(
            target_os = "linux",
            target_os = "fuchsia",
            target_os = "freebsd",
            target_os = "openbsd",
            all(target_os = "android", feature = "dl_iterate_phdr"),
        ),
        not(target_env = "uclibc"),
    ))] {
        mod libs_dl_iterate_phdr;
        use libs_dl_iterate_phdr::native_libraries;
        #[path = "gimli/parse_running_mmaps_unix.rs"]
        mod parse_running_mmaps;
    } else if #[cfg(target_env = "libnx")] {
        mod libs_libnx;
        use libs_libnx::native_libraries;
    } else if #[cfg(target_os = "haiku")] {
        mod libs_haiku;
        use libs_haiku::native_libraries;
    } else {
        // Everything else should doesn't know how to load native libraries.
        fn native_libraries() -> Vec<Library> {
            Vec::new()
        }
    }
}

#[derive(Default)]
struct Cache {
    /// All known shared libraries that have been loaded.
    libraries: Vec<Library>,

    /// Mappings cache where we retain parsed dwarf information.
    ///
    /// This list has a fixed capacity for its entire lifetime which never
    /// increases. The `usize` element of each pair is an index into `libraries`
    /// above where `usize::max_value()` represents the current executable. The
    /// `Mapping` is corresponding parsed dwarf information.
    ///
    /// Note that this is basically an LRU cache and we'll be shifting things
    /// around in here as we symbolize addresses.
    mappings: Vec<(usize, Mapping)>,
}

struct Library {
    name: OsString,
    /// Segments of this library loaded into memory, and where they're loaded.
    segments: Vec<LibrarySegment>,
    /// The "bias" of this library, typically where it's loaded into memory.
    /// This value is added to each segment's stated address to get the actual
    /// virtual memory address that the segment is loaded into. Additionally
    /// this bias is subtracted from real virtual memory addresses to index into
    /// debuginfo and the symbol table.
    bias: usize,
}

struct LibrarySegment {
    /// The stated address of this segment in the object file. This is not
    /// actually where the segment is loaded, but rather this address plus the
    /// containing library's `bias` is where to find it.
    stated_virtual_memory_address: usize,
    /// The size of this segment in memory.
    len: usize,
}

// unsafe because this is required to be externally synchronized
pub unsafe fn clear_symbol_cache() {
    Cache::with_global(|cache| cache.mappings.clear());
}

impl Cache {
    fn new() -> Cache {
        Cache {
            mappings: Vec::with_capacity(MAPPINGS_CACHE_SIZE),
            libraries: native_libraries(),
        }
    }

    // unsafe because this is required to be externally synchronized
    unsafe fn with_global(f: impl FnOnce(&mut Self)) {
        // A very small, very simple LRU cache for debug info mappings.
        //
        // The hit rate should be very high, since the typical stack doesn't cross
        // between many shared libraries.
        //
        // The `addr2line::Context` structures are pretty expensive to create. Its
        // cost is expected to be amortized by subsequent `locate` queries, which
        // leverage the structures built when constructing `addr2line::Context`s to
        // get nice speedups. If we didn't have this cache, that amortization would
        // never happen, and symbolicating backtraces would be ssssllllooooowwww.
        static mut MAPPINGS_CACHE: Option<Cache> = None;

        f(MAPPINGS_CACHE.get_or_insert_with(|| Cache::new()))
    }

    fn avma_to_svma(&self, addr: *const u8) -> Option<(usize, *const u8)> {
        self.libraries
            .iter()
            .enumerate()
            .filter_map(|(i, lib)| {
                // First up, test if this `lib` has any segment containing the
                // `addr` (handling relocation). If this check passes then we
                // can continue below and actually translate the address.
                //
                // Note that we're using `wrapping_add` here to avoid overflow
                // checks. It's been seen in the wild that the SVMA + bias
                // computation overflows. It seems a bit odd that would happen
                // but there's not a huge amount we can do about it other than
                // probably just ignore those segments since they're likely
                // pointing off into space. This originally came up in
                // rust-lang/backtrace-rs#329.
                if !lib.segments.iter().any(|s| {
                    let svma = s.stated_virtual_memory_address;
                    let start = svma.wrapping_add(lib.bias);
                    let end = start.wrapping_add(s.len);
                    let address = addr as usize;
                    start <= address && address < end
                }) {
                    return None;
                }

                // Now that we know `lib` contains `addr`, we can offset with
                // the bias to find the stated virtual memory address.
                let svma = (addr as usize).wrapping_sub(lib.bias);
                Some((i, svma as *const u8))
            })
            .next()
    }

    fn mapping_for_lib<'a>(&'a mut self, lib: usize) -> Option<&'a mut Context<'a>> {
        let idx = self.mappings.iter().position(|(idx, _)| *idx == lib);

        // Invariant: after this conditional completes without early returning
        // from an error, the cache entry for this path is at index 0.

        if let Some(idx) = idx {
            // When the mapping is already in the cache, move it to the front.
            if idx != 0 {
                let entry = self.mappings.remove(idx);
                self.mappings.insert(0, entry);
            }
        } else {
            // When the mapping is not in the cache, create a new mapping,
            // insert it into the front of the cache, and evict the oldest cache
            // entry if necessary.
            let name = &self.libraries[lib].name;
            let mapping = Mapping::new(name.as_ref())?;

            if self.mappings.len() == MAPPINGS_CACHE_SIZE {
                self.mappings.pop();
            }

            self.mappings.insert(0, (lib, mapping));
        }

        let cx: &'a mut Context<'static> = &mut self.mappings[0].1.cx;
        // don't leak the `'static` lifetime, make sure it's scoped to just
        // ourselves
        Some(unsafe { mem::transmute::<&'a mut Context<'static>, &'a mut Context<'a>>(cx) })
    }
}

pub unsafe fn resolve(what: ResolveWhat<'_>, cb: &mut dyn FnMut(&super::Symbol)) {
    let addr = what.address_or_ip();
    let mut call = |sym: Symbol<'_>| {
        // Extend the lifetime of `sym` to `'static` since we are unfortunately
        // required to here, but it's only ever going out as a reference so no
        // reference to it should be persisted beyond this frame anyway.
        let sym = mem::transmute::<Symbol<'_>, Symbol<'static>>(sym);
        (cb)(&super::Symbol { inner: sym });
    };

    Cache::with_global(|cache| {
        let (lib, addr) = match cache.avma_to_svma(addr as *const u8) {
            Some(pair) => pair,
            None => return,
        };

        // Finally, get a cached mapping or create a new mapping for this file, and
        // evaluate the DWARF info to find the file/line/name for this address.
        let cx = match cache.mapping_for_lib(lib) {
            Some(cx) => cx,
            None => return,
        };
        let mut any_frames = false;
        if let Ok(mut frames) = cx.dwarf.find_frames(addr as u64) {
            while let Ok(Some(frame)) = frames.next() {
                any_frames = true;
                let name = match frame.function {
                    Some(f) => Some(f.name.slice()),
                    None => cx.object.search_symtab(addr as u64),
                };
                call(Symbol::Frame {
                    addr: addr as *mut c_void,
                    location: frame.location,
                    name,
                });
            }
        }
        if !any_frames {
            if let Some((object_cx, object_addr)) = cx.object.search_object_map(addr as u64) {
                if let Ok(mut frames) = object_cx.dwarf.find_frames(object_addr) {
                    while let Ok(Some(frame)) = frames.next() {
                        any_frames = true;
                        call(Symbol::Frame {
                            addr: addr as *mut c_void,
                            location: frame.location,
                            name: frame.function.map(|f| f.name.slice()),
                        });
                    }
                }
            }
        }
        if !any_frames {
            if let Some(name) = cx.object.search_symtab(addr as u64) {
                call(Symbol::Symtab {
                    addr: addr as *mut c_void,
                    name,
                });
            }
        }
    });
}

pub enum Symbol<'a> {
    /// We were able to locate frame information for this symbol, and
    /// `addr2line`'s frame internally has all the nitty gritty details.
    Frame {
        addr: *mut c_void,
        location: Option<addr2line::Location<'a>>,
        name: Option<&'a [u8]>,
    },
    /// Couldn't find debug information, but we found it in the symbol table of
    /// the elf executable.
    Symtab { addr: *mut c_void, name: &'a [u8] },
}

impl Symbol<'_> {
    pub fn name(&self) -> Option<SymbolName<'_>> {
        match self {
            Symbol::Frame { name, .. } => {
                let name = name.as_ref()?;
                Some(SymbolName::new(name))
            }
            Symbol::Symtab { name, .. } => Some(SymbolName::new(name)),
        }
    }

    pub fn addr(&self) -> Option<*mut c_void> {
        match self {
            Symbol::Frame { addr, .. } => Some(*addr),
            Symbol::Symtab { .. } => None,
        }
    }

    pub fn filename_raw(&self) -> Option<BytesOrWideString<'_>> {
        match self {
            Symbol::Frame { location, .. } => {
                let file = location.as_ref()?.file?;
                Some(BytesOrWideString::Bytes(file.as_bytes()))
            }
            Symbol::Symtab { .. } => None,
        }
    }

    pub fn filename(&self) -> Option<&Path> {
        match self {
            Symbol::Frame { location, .. } => {
                let file = location.as_ref()?.file?;
                Some(Path::new(file))
            }
            Symbol::Symtab { .. } => None,
        }
    }

    pub fn lineno(&self) -> Option<u32> {
        match self {
            Symbol::Frame { location, .. } => location.as_ref()?.line,
            Symbol::Symtab { .. } => None,
        }
    }

    pub fn colno(&self) -> Option<u32> {
        match self {
            Symbol::Frame { location, .. } => location.as_ref()?.column,
            Symbol::Symtab { .. } => None,
        }
    }
}
