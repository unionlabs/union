// Other Unix (e.g. Linux) platforms use ELF as an object file format
// and typically implement an API called `dl_iterate_phdr` to load
// native libraries.

use super::mystd::borrow::ToOwned;
use super::mystd::env;
use super::mystd::ffi::{CStr, OsStr};
use super::mystd::os::unix::prelude::*;
use super::{Library, LibrarySegment, OsString, Vec};
use core::slice;

pub(super) fn native_libraries() -> Vec<Library> {
    let mut ret = Vec::new();
    unsafe {
        libc::dl_iterate_phdr(Some(callback), &mut ret as *mut Vec<_> as *mut _);
    }
    return ret;
}

fn infer_current_exe(base_addr: usize) -> OsString {
    if let Ok(entries) = super::parse_running_mmaps::parse_maps() {
        let opt_path = entries
            .iter()
            .find(|e| e.ip_matches(base_addr) && e.pathname().len() > 0)
            .map(|e| e.pathname())
            .cloned();
        if let Some(path) = opt_path {
            return path;
        }
    }
    env::current_exe().map(|e| e.into()).unwrap_or_default()
}

// `info` should be a valid pointers.
// `vec` should be a valid pointer to a `std::Vec`.
unsafe extern "C" fn callback(
    info: *mut libc::dl_phdr_info,
    _size: libc::size_t,
    vec: *mut libc::c_void,
) -> libc::c_int {
    let info = &*info;
    let libs = &mut *(vec as *mut Vec<Library>);
    let is_main_prog = info.dlpi_name.is_null() || *info.dlpi_name == 0;
    let name = if is_main_prog {
        // The man page for dl_iterate_phdr says that the first object visited by
        // callback is the main program; so the first time we encounter a
        // nameless entry, we can assume its the main program and try to infer its path.
        // After that, we cannot continue that assumption, and we use an empty string.
        if libs.is_empty() {
            infer_current_exe(info.dlpi_addr as usize)
        } else {
            OsString::new()
        }
    } else {
        let bytes = CStr::from_ptr(info.dlpi_name).to_bytes();
        OsStr::from_bytes(bytes).to_owned()
    };
    let headers = slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize);
    libs.push(Library {
        name,
        segments: headers
            .iter()
            .map(|header| LibrarySegment {
                len: (*header).p_memsz as usize,
                stated_virtual_memory_address: (*header).p_vaddr as usize,
            })
            .collect(),
        bias: info.dlpi_addr as usize,
    });
    0
}
