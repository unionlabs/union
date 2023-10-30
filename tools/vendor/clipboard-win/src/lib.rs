#![cfg(windows)]
//! This crate provide simple means to operate with Windows clipboard.
//!
//!# Note keeping Clipboard around:
//!
//! In Windows [Clipboard](struct.Clipboard.html) opens globally and only one application can set data onto format at the time.
//!
//! Therefore as soon as operations are finished, user is advised to close [Clipboard](struct.Clipboard.html).
//!
//!# Features
//!
//! - `std` - Enables usage of `std`, including `std::error::Error` trait.
//!
//!# Clipboard
//!
//! All read and write access to Windows clipboard requires user to open it.
//!
//!# Usage
//!
//!## Getter
//!
//! Library provides various extractors from clipboard to particular format using [Getter](trait.Getter.html):
//!
//! - [RawData](formats/struct.RawData.html) - Reads raw bytes from specified format.
//! - [Unicode](formats/struct.Unicode.html) - Reads unicode string from clipboard.
//! - [Bitmap](formats/struct.Bitmap.html) - Reads RGB data of image on clipboard.
//! - [FileList](formats/struct.FileList.html) - Reads list of files from clipboard.
//!
//! Depending on format, getter can extract data into various data types.
//!
//!## Setter
//!
//! Library provides various setters onto clipboard by using [Setter](trait.Setter.html):
//!
//! - [RawData](formats/struct.RawData.html) - Writes raw bytes onto specified format.
//! - [Unicode](formats/struct.Unicode.html) - Writes unicode string onto clipboard.
//! - [Bitmap](formats/struct.Bitmap.html) - Writes RGB data of image on clipboard.
//!
//! Default setters are generic over type allowing anything that can be referenced as byte slice or
//! `str`
//!
//!## Manually lock clipboard
//!
//!```
//!use clipboard_win::{Clipboard, formats, Getter, Setter};
//!
//!const SAMPLE: &str = "MY loli sample ^^";
//!
//!let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
//!formats::Unicode.write_clipboard(&SAMPLE).expect("Write sample");
//!
//!let mut output = String::new();
//!
//!assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
//!assert_eq!(output, SAMPLE);
//!
//!//Efficiently re-use buffer ;)
//!output.clear();
//!assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
//!assert_eq!(output, SAMPLE);
//!
//!//Or take the same string twice?
//!assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
//!assert_eq!(format!("{0}{0}", SAMPLE), output);
//!
//!```
//!
//!## Simplified API
//!
//!```
//!use clipboard_win::{formats, get_clipboard, set_clipboard};
//!
//!let text = "my sample ><";
//!
//!set_clipboard(formats::Unicode, text).expect("To set clipboard");
//!//Type is necessary as string can be stored in various storages
//!let result: String = get_clipboard(formats::Unicode).expect("To set clipboard");
//!assert_eq!(result, text)
//!```

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub mod formats;
pub mod raw;
pub(crate) mod utils;

pub use raw::{get_owner, empty, seq_num, size, is_format_avail, register_format, count_formats, EnumFormats};
pub use formats::Unicode;

pub use error_code::SystemError;
///Alias to result used by this crate
pub type SysResult<T> = Result<T, error_code::SystemError>;

///Clipboard instance, which allows to perform clipboard ops.
///
///# Note:
///
///You can have only one such instance across your program.
///
///# Warning:
///
///In Windows Clipboard opens globally and only one application can set data
///onto format at the time.
///
///Therefore as soon as operations are finished, user is advised to close Clipboard.
pub struct Clipboard {
    _dummy: ()
}

impl Clipboard {
    #[inline(always)]
    ///Attempts to open clipboard, returning clipboard instance on success.
    pub fn new() -> SysResult<Self> {
        raw::open().map(|_| Self { _dummy: () })
    }

    #[inline(always)]
    ///Attempts to open clipboard, associating it with specified `owner` and returning clipboard instance on success.
    pub fn new_for(owner: winapi::shared::windef::HWND) -> SysResult<Self> {
        raw::open_for(owner).map(|_| Self { _dummy: () })
    }

    #[inline(always)]
    ///Attempts to open clipboard, giving it `num` retries in case of failure.
    pub fn new_attempts(num: usize) -> SysResult<Self> {
        Self::new_attempts_for(core::ptr::null_mut(), num)
    }

    #[inline]
    ///Attempts to open clipboard, giving it `num` retries in case of failure.
    pub fn new_attempts_for(owner: winapi::shared::windef::HWND, mut num: usize) -> SysResult<Self> {
        loop {
            match Self::new_for(owner) {
                Ok(this) => break Ok(this),
                Err(err) => match num {
                    0 => break Err(err),
                    _ => num -= 1,
                }
            }

            //0 causes to yield remaining time in scheduler, but remain to be scheduled once again.
            unsafe { winapi::um::synchapi::Sleep(0) };
        }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = raw::close();
    }
}

///Describes format getter, specifying data type as type param
///
///Default implementations only perform write, without opening/closing clipboard
pub trait Getter<Type> {
    ///Reads content of clipboard into `out`, returning number of bytes read on success, or otherwise 0.
    fn read_clipboard(&self, out: &mut Type) -> SysResult<usize>;
}

///Describes format setter, specifying data type as type param
///
///Default implementations only perform write, without opening/closing clipboard
pub trait Setter<Type> {
    ///Writes content of `data` onto clipboard, returning whether it was successful or not
    fn write_clipboard(&self, data: &Type) -> SysResult<()>;
}

#[inline(always)]
///Runs provided callable with open clipboard, returning whether clipboard was open successfully.
///
///If clipboard fails to open, callable is not invoked.
pub fn with_clipboard<F: FnMut()>(mut cb: F) -> SysResult<()> {
    let _clip = Clipboard::new()?;
    cb();
    Ok(())
}

#[inline(always)]
///Runs provided callable with open clipboard, returning whether clipboard was open successfully.
///
///If clipboard fails to open, attempts `num` number of retries before giving up.
///In which case closure is not called
pub fn with_clipboard_attempts<F: FnMut()>(num: usize, mut cb: F) -> SysResult<()> {
    let _clip = Clipboard::new_attempts(num)?;
    cb();
    Ok(())
}

#[inline(always)]
///Retrieve data from clipboard.
pub fn get<R: Default, T: Getter<R>>(format: T) -> SysResult<R> {
    let mut result = R::default();
    format.read_clipboard(&mut result).map(|_| result)
}

#[inline(always)]
///Shortcut to retrieve data from clipboard.
///
///It opens clipboard and gets output, if possible.
pub fn get_clipboard<R: Default, T: Getter<R>>(format: T) -> SysResult<R> {
    let _clip = Clipboard::new_attempts(10)?;
    get(format)
}

#[inline(always)]
///Set data onto clipboard.
pub fn set<R, T: Setter<R>>(format: T, data: R) -> SysResult<()> {
    format.write_clipboard(&data)
}

#[inline(always)]
///Shortcut to set data onto clipboard.
///
///It opens clipboard and attempts to set data.
pub fn set_clipboard<R, T: Setter<R>>(format: T, data: R) -> SysResult<()> {
    let _clip = Clipboard::new_attempts(10)?;
    set(format, data)
}

///Shortcut to retrieve string from clipboard.
///
///It opens clipboard and gets string, if possible.
#[inline(always)]
pub fn get_clipboard_string() -> SysResult<alloc::string::String> {
    get_clipboard(Unicode)
}

///Shortcut to set string onto clipboard.
///
///It opens clipboard and attempts to set string.
#[inline(always)]
pub fn set_clipboard_string(data: &str) -> SysResult<()> {
    set_clipboard(Unicode, data)
}
