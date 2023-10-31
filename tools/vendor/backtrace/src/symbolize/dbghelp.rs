//! Symbolication strategy using `dbghelp.dll` on Windows, only used for MSVC
//!
//! This symbolication strategy, like with backtraces, uses dynamically loaded
//! information from `dbghelp.dll`. (see `src/dbghelp.rs` for info about why
//! it's dynamically loaded).
//!
//! This API selects its resolution strategy based on the frame provided or the
//! information we have at hand. If a frame from `StackWalkEx` is given to us
//! then we use similar APIs to generate correct information about inlined
//! functions. Otherwise if all we have is an address or an older stack frame
//! from `StackWalk64` we use the older APIs for symbolication.
//!
//! There's a good deal of support in this module, but a good chunk of it is
//! converting back and forth between Windows types and Rust types. For example
//! symbols come to us as wide strings which we then convert to utf-8 strings if
//! we can.

#![allow(bad_style)]

use super::super::{backtrace::StackFrame, dbghelp, windows::*};
use super::{BytesOrWideString, ResolveWhat, SymbolName};
use core::char;
use core::ffi::c_void;
use core::marker;
use core::mem;
use core::slice;

// Store an OsString on std so we can provide the symbol name and filename.
pub struct Symbol<'a> {
    name: *const [u8],
    addr: *mut c_void,
    line: Option<u32>,
    filename: Option<*const [u16]>,
    #[cfg(feature = "std")]
    _filename_cache: Option<::std::ffi::OsString>,
    #[cfg(not(feature = "std"))]
    _filename_cache: (),
    _marker: marker::PhantomData<&'a i32>,
}

impl Symbol<'_> {
    pub fn name(&self) -> Option<SymbolName<'_>> {
        Some(SymbolName::new(unsafe { &*self.name }))
    }

    pub fn addr(&self) -> Option<*mut c_void> {
        Some(self.addr as *mut _)
    }

    pub fn filename_raw(&self) -> Option<BytesOrWideString<'_>> {
        self.filename
            .map(|slice| unsafe { BytesOrWideString::Wide(&*slice) })
    }

    pub fn colno(&self) -> Option<u32> {
        None
    }

    pub fn lineno(&self) -> Option<u32> {
        self.line
    }

    #[cfg(feature = "std")]
    pub fn filename(&self) -> Option<&::std::path::Path> {
        use std::path::Path;

        self._filename_cache.as_ref().map(Path::new)
    }
}

#[repr(C, align(8))]
struct Aligned8<T>(T);

pub unsafe fn resolve(what: ResolveWhat<'_>, cb: &mut dyn FnMut(&super::Symbol)) {
    // Ensure this process's symbols are initialized
    let dbghelp = match dbghelp::init() {
        Ok(dbghelp) => dbghelp,
        Err(()) => return, // oh well...
    };

    match what {
        ResolveWhat::Address(_) => resolve_without_inline(&dbghelp, what.address_or_ip(), cb),
        ResolveWhat::Frame(frame) => match &frame.inner.stack_frame {
            StackFrame::New(frame) => resolve_with_inline(&dbghelp, frame, cb),
            StackFrame::Old(_) => resolve_without_inline(&dbghelp, frame.ip(), cb),
        },
    }
}

unsafe fn resolve_with_inline(
    dbghelp: &dbghelp::Init,
    frame: &STACKFRAME_EX,
    cb: &mut dyn FnMut(&super::Symbol),
) {
    do_resolve(
        |info| {
            dbghelp.SymFromInlineContextW()(
                GetCurrentProcess(),
                super::adjust_ip(frame.AddrPC.Offset as *mut _) as u64,
                frame.InlineFrameContext,
                &mut 0,
                info,
            )
        },
        |line| {
            dbghelp.SymGetLineFromInlineContextW()(
                GetCurrentProcess(),
                super::adjust_ip(frame.AddrPC.Offset as *mut _) as u64,
                frame.InlineFrameContext,
                0,
                &mut 0,
                line,
            )
        },
        cb,
    )
}

unsafe fn resolve_without_inline(
    dbghelp: &dbghelp::Init,
    addr: *mut c_void,
    cb: &mut dyn FnMut(&super::Symbol),
) {
    do_resolve(
        |info| dbghelp.SymFromAddrW()(GetCurrentProcess(), addr as DWORD64, &mut 0, info),
        |line| dbghelp.SymGetLineFromAddrW64()(GetCurrentProcess(), addr as DWORD64, &mut 0, line),
        cb,
    )
}

unsafe fn do_resolve(
    sym_from_addr: impl FnOnce(*mut SYMBOL_INFOW) -> BOOL,
    get_line_from_addr: impl FnOnce(&mut IMAGEHLP_LINEW64) -> BOOL,
    cb: &mut dyn FnMut(&super::Symbol),
) {
    const SIZE: usize = 2 * MAX_SYM_NAME + mem::size_of::<SYMBOL_INFOW>();
    let mut data = Aligned8([0u8; SIZE]);
    let data = &mut data.0;
    let info = &mut *(data.as_mut_ptr() as *mut SYMBOL_INFOW);
    info.MaxNameLen = MAX_SYM_NAME as ULONG;
    // the struct size in C.  the value is different to
    // `size_of::<SYMBOL_INFOW>() - MAX_SYM_NAME + 1` (== 81)
    // due to struct alignment.
    info.SizeOfStruct = 88;

    if sym_from_addr(info) != TRUE {
        return;
    }

    // If the symbol name is greater than MaxNameLen, SymFromAddrW will
    // give a buffer of (MaxNameLen - 1) characters and set NameLen to
    // the real value.
    let name_len = ::core::cmp::min(info.NameLen as usize, info.MaxNameLen as usize - 1);
    let name_ptr = info.Name.as_ptr() as *const u16;
    let name = slice::from_raw_parts(name_ptr, name_len);

    // Reencode the utf-16 symbol to utf-8 so we can use `SymbolName::new` like
    // all other platforms
    let mut name_len = 0;
    let mut name_buffer = [0; 256];
    {
        let mut remaining = &mut name_buffer[..];
        for c in char::decode_utf16(name.iter().cloned()) {
            let c = c.unwrap_or(char::REPLACEMENT_CHARACTER);
            let len = c.len_utf8();
            if len < remaining.len() {
                c.encode_utf8(remaining);
                let tmp = remaining;
                remaining = &mut tmp[len..];
                name_len += len;
            } else {
                break;
            }
        }
    }
    let name = &name_buffer[..name_len] as *const [u8];

    let mut line = mem::zeroed::<IMAGEHLP_LINEW64>();
    line.SizeOfStruct = mem::size_of::<IMAGEHLP_LINEW64>() as DWORD;

    let mut filename = None;
    let mut lineno = None;
    if get_line_from_addr(&mut line) == TRUE {
        lineno = Some(line.LineNumber as u32);

        let base = line.FileName;
        let mut len = 0;
        while *base.offset(len) != 0 {
            len += 1;
        }

        let len = len as usize;

        filename = Some(slice::from_raw_parts(base, len) as *const [u16]);
    }

    cb(&super::Symbol {
        inner: Symbol {
            name,
            addr: info.Address as *mut _,
            line: lineno,
            filename,
            _filename_cache: cache(filename),
            _marker: marker::PhantomData,
        },
    })
}

#[cfg(feature = "std")]
unsafe fn cache(filename: Option<*const [u16]>) -> Option<::std::ffi::OsString> {
    use std::os::windows::ffi::OsStringExt;
    filename.map(|f| ::std::ffi::OsString::from_wide(&*f))
}

#[cfg(not(feature = "std"))]
unsafe fn cache(_filename: Option<*const [u16]>) {}

pub unsafe fn clear_symbol_cache() {}
