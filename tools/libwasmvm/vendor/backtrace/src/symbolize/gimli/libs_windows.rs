use super::super::super::windows::*;
use super::mystd::os::windows::prelude::*;
use super::{coff, mmap, Library, LibrarySegment, OsString};
use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use core::mem::MaybeUninit;

// For loading native libraries on Windows, see some discussion on
// rust-lang/rust#71060 for the various strategies here.
pub(super) fn native_libraries() -> Vec<Library> {
    let mut ret = Vec::new();
    unsafe {
        add_loaded_images(&mut ret);
    }
    return ret;
}

unsafe fn add_loaded_images(ret: &mut Vec<Library>) {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, 0);
    if snap == INVALID_HANDLE_VALUE {
        return;
    }

    let mut me = MaybeUninit::<MODULEENTRY32W>::zeroed().assume_init();
    me.dwSize = mem::size_of_val(&me) as DWORD;
    if Module32FirstW(snap, &mut me) == TRUE {
        loop {
            if let Some(lib) = load_library(&me) {
                ret.push(lib);
            }

            if Module32NextW(snap, &mut me) != TRUE {
                break;
            }
        }
    }

    CloseHandle(snap);
}

unsafe fn load_library(me: &MODULEENTRY32W) -> Option<Library> {
    let pos = me
        .szExePath
        .iter()
        .position(|i| *i == 0)
        .unwrap_or(me.szExePath.len());
    let name = OsString::from_wide(&me.szExePath[..pos]);

    // MinGW libraries currently don't support ASLR
    // (rust-lang/rust#16514), but DLLs can still be relocated around in
    // the address space. It appears that addresses in debug info are
    // all as-if this library was loaded at its "image base", which is a
    // field in its COFF file headers. Since this is what debuginfo
    // seems to list we parse the symbol table and store addresses as if
    // the library was loaded at "image base" as well.
    //
    // The library may not be loaded at "image base", however.
    // (presumably something else may be loaded there?) This is where
    // the `bias` field comes into play, and we need to figure out the
    // value of `bias` here. Unfortunately though it's not clear how to
    // acquire this from a loaded module. What we do have, however, is
    // the actual load address (`modBaseAddr`).
    //
    // As a bit of a cop-out for now we mmap the file, read the file
    // header information, then drop the mmap. This is wasteful because
    // we'll probably reopen the mmap later, but this should work well
    // enough for now.
    //
    // Once we have the `image_base` (desired load location) and the
    // `base_addr` (actual load location) we can fill in the `bias`
    // (difference between the actual and desired) and then the stated
    // address of each segment is the `image_base` since that's what the
    // file says.
    //
    // For now it appears that unlike ELF/MachO we can make do with one
    // segment per library, using `modBaseSize` as the whole size.
    let mmap = mmap(name.as_ref())?;
    let image_base = coff::get_image_base(&mmap)?;
    let base_addr = me.modBaseAddr as usize;
    Some(Library {
        name,
        bias: base_addr.wrapping_sub(image_base),
        segments: vec![LibrarySegment {
            stated_virtual_memory_address: image_base,
            len: me.modBaseSize as usize,
        }],
    })
}
