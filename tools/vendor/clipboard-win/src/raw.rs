//!Raw bindings to Windows clipboard.
//!
//!## General information
//!
//!All pre & post conditions are stated in description of functions.
//!
//!### Open clipboard
//! To access any information inside clipboard it is necessary to open it by means of
//! [open()](fn.open.html).
//!
//! After that Clipboard cannot be opened any more until [close()](fn.close.html) is called.

use winapi::um::winuser::{OpenClipboard, CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, GetClipboardData, IsClipboardFormatAvailable, CountClipboardFormats, EnumClipboardFormats, GetClipboardFormatNameW, RegisterClipboardFormatW, SetClipboardData, GetDC, ReleaseDC, GetClipboardOwner};
use winapi::um::winbase::{GlobalSize, GlobalLock, GlobalUnlock};
use winapi::ctypes::{c_int, c_uint, c_void};
use winapi::um::stringapiset::{MultiByteToWideChar, WideCharToMultiByte};
use winapi::um::winnls::CP_UTF8;
use winapi::um::shellapi::{DragQueryFileW};
use winapi::um::wingdi::{GetObjectW, GetDIBits, CreateDIBitmap, BITMAP, BITMAPINFO, BITMAPINFOHEADER, RGBQUAD, BI_RGB, DIB_RGB_COLORS, BITMAPFILEHEADER, CBM_INIT};
use winapi::shared::windef::{HDC};
use winapi::shared::winerror::ERROR_INCORRECT_SIZE;

use str_buf::StrBuf;
use error_code::SystemError;

use core::{slice, mem, ptr, cmp};
use core::num::{NonZeroUsize, NonZeroU32};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::format;

use crate::{SysResult, formats};
use crate::utils::{unlikely_empty_size_result, RawMem};

#[inline(always)]
fn free_dc(data: HDC) {
    unsafe {
        ReleaseDC(ptr::null_mut(), data);
    }
}

#[inline(always)]
///Opens clipboard.
///
///Wrapper around ```OpenClipboard```.
///
///# Pre-conditions:
///
///* Clipboard is not opened yet.
///
///# Post-conditions (if successful):
///
///* Clipboard can be accessed for read and write operations.
pub fn open() -> SysResult<()> {
    open_for(ptr::null_mut())
}

#[inline]
///Opens clipboard associating it with specified window handle.
///
///Unless [empty](fn.empty.html) is called, `owner` would be `None`.
///
///Wrapper around ```OpenClipboard```.
///
///# Pre-conditions:
///
///* Clipboard is not opened yet.
///
///# Post-conditions (if successful):
///
///* Clipboard can be accessed for read and write operations.
pub fn open_for(owner: winapi::shared::windef::HWND) -> SysResult<()> {
    match unsafe { OpenClipboard(owner) } {
        0 => Err(SystemError::last()),
        _ => Ok(()),
    }
}

#[inline]
///Closes clipboard.
///
///Wrapper around ```CloseClipboard```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn close() -> SysResult<()> {
    match unsafe { CloseClipboard() } {
        0 => Err(SystemError::last()),
        _ => Ok(()),
    }
}

#[inline]
///Empties clipboard.
///
///Wrapper around ```EmptyClipboard```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn empty() -> SysResult<()> {
    match unsafe { EmptyClipboard() } {
        0 => Err(SystemError::last()),
        _ => Ok(()),
    }
}

#[inline]
///Retrieves clipboard sequence number.
///
///Wrapper around ```GetClipboardSequenceNumber```.
///
///# Returns:
///
///* ```Some``` Contains return value of ```GetClipboardSequenceNumber```.
///* ```None``` In case if you do not have access. It means that zero is returned by system.
pub fn seq_num() -> Option<NonZeroU32> {
    unsafe { NonZeroU32::new(GetClipboardSequenceNumber()) }
}

#[inline]
///Retrieves size of clipboard data for specified format.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Returns:
///
///Size in bytes if format presents on clipboard.
///
///# Unsafety:
///
///In some cases, clipboard content might be so invalid that it crashes on `GlobalSize` (e.g.
///Bitmap)
///
///Due to that function is marked as unsafe
pub unsafe fn size_unsafe(format: u32) -> Option<NonZeroUsize> {
    let clipboard_data = GetClipboardData(format);

    match clipboard_data.is_null() {
        false => NonZeroUsize::new(GlobalSize(clipboard_data) as usize),
        true => None,
    }
}

#[inline]
///Retrieves size of clipboard data for specified format.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Returns:
///
///Size in bytes if format is presents on clipboard.
pub fn size(format: u32) -> Option<NonZeroUsize> {
    let clipboard_data = unsafe {GetClipboardData(format)};

    if clipboard_data.is_null() {
        return None
    }

    unsafe {
        if GlobalLock(clipboard_data).is_null() {
            return None;
        }

        let result = NonZeroUsize::new(GlobalSize(clipboard_data) as usize);

        GlobalUnlock(clipboard_data);

        result
    }
}

#[inline(always)]
///Retrieves raw pointer to clipboard data.
///
///Wrapper around ```GetClipboardData```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn get_clipboard_data(format: c_uint) -> SysResult<ptr::NonNull<c_void>> {
    let ptr = unsafe { GetClipboardData(format) as *mut c_void };
    match ptr::NonNull::new(ptr) {
        Some(ptr) => Ok(ptr),
        None => Err(SystemError::last()),
    }
}

#[inline(always)]
///Determines whenever provided clipboard format is available on clipboard or not.
pub fn is_format_avail(format: c_uint) -> bool {
    unsafe { IsClipboardFormatAvailable(format) != 0 }
}

#[inline]
///Retrieves number of currently available formats on clipboard.
///
///Returns `None` if `CountClipboardFormats` failed.
pub fn count_formats() -> Option<usize> {
    let result = unsafe { CountClipboardFormats() };

    if result == 0 {
        if !SystemError::last().is_zero() {
            return None
        }
    }

    Some(result as usize)
}

///Copies raw bytes from clipboard with specified `format`
///
///Returns number of copied bytes on success, otherwise 0.
///
///It is safe to pass uninit memory
pub fn get(format: u32, out: &mut [u8]) -> SysResult<usize> {
    let size = out.len();
    if size == 0 {
        return Ok(unlikely_empty_size_result());
    }
    let out_ptr = out.as_mut_ptr();

    let ptr = RawMem::from_borrowed(get_clipboard_data(format)?);

    let result = unsafe {
        let (data_ptr, _lock) = ptr.lock()?;
        let data_size = cmp::min(GlobalSize(ptr.get()) as usize, size);
        ptr::copy_nonoverlapping(data_ptr.as_ptr() as *const u8, out_ptr, data_size);
        data_size
    };

    Ok(result)
}

///Copies raw bytes from clipboard with specified `format`, appending to `out` buffer.
///
///Returns number of copied bytes on success, otherwise 0.
pub fn get_vec(format: u32, out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
    let ptr = RawMem::from_borrowed(get_clipboard_data(format)?);

    let result = unsafe {
        let (data_ptr, _lock) = ptr.lock()?;
        let data_size = GlobalSize(ptr.get()) as usize;

        out.reserve(data_size as usize);
        let storage_cursor = out.len();
        let storage_ptr = out.as_mut_ptr().add(out.len()) as *mut _;

        ptr::copy_nonoverlapping(data_ptr.as_ptr() as *const u8, storage_ptr, data_size);
        out.set_len(storage_cursor + data_size as usize);

        data_size
    };

    Ok(result)
}

/// Copies raw bytes onto clipboard with specified `format`, returning whether it was successful.
///
/// This function empties the clipboard before setting the data.
pub fn set(format: u32, data: &[u8]) -> SysResult<()> {
    let _ = empty();
    set_without_clear(format, data)
}

/// Copies raw bytes onto the clipboard with the specified `format`, returning whether it was successful.
///
/// This function does not empty the clipboard before setting the data.
pub fn set_without_clear(format: u32, data: &[u8]) -> SysResult<()> {
    let size = data.len();
    if size == 0 {
        return Ok(unlikely_empty_size_result());
    }

    let mem = RawMem::new_global_mem(size)?;

    {
        let (ptr, _lock) = mem.lock()?;
        unsafe { ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr() as _, size) };
    }

    if unsafe { !SetClipboardData(format, mem.get()).is_null() } {
        //SetClipboardData takes ownership
        mem.release();
        return Ok(());
    }

    Err(error_code::SystemError::last())
}

///Copies raw bytes from clipboard with specified `format`, appending to `out` buffer.
///
///Returns number of copied bytes on success, otherwise 0.
pub fn get_string(out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
    let ptr = RawMem::from_borrowed(get_clipboard_data(formats::CF_UNICODETEXT)?);

    let result = unsafe {
        let (data_ptr, _lock) = ptr.lock()?;
        let data_size = GlobalSize(ptr.get()) as usize / mem::size_of::<u16>();
        let storage_req_size = WideCharToMultiByte(CP_UTF8, 0, data_ptr.as_ptr() as _, data_size as _, ptr::null_mut(), 0, ptr::null(), ptr::null_mut());

        if storage_req_size == 0 {
            return Err(SystemError::last());
        }

        let storage_cursor = out.len();
        out.reserve(storage_req_size as usize);
        let storage_ptr = out.as_mut_ptr().add(storage_cursor) as *mut _;
        WideCharToMultiByte(CP_UTF8, 0, data_ptr.as_ptr() as _, data_size as _, storage_ptr, storage_req_size, ptr::null(), ptr::null_mut());
        out.set_len(storage_cursor + storage_req_size as usize);

        //It seems WinAPI always supposed to have at the end null char.
        //But just to be safe let's check for it and only then remove.
        if let Some(null_idx) = out.iter().skip(storage_cursor).position(|b| *b == b'\0') {
            out.set_len(storage_cursor + null_idx);
        }

        out.len() - storage_cursor
    };

    Ok(result)
}

///Copies unicode string onto clipboard, performing necessary conversions, returning true on
///success.
pub fn set_string(data: &str) -> SysResult<()> {
    if data.is_empty() {
        return Ok(unlikely_empty_size_result());
    }

    let size = unsafe {
        MultiByteToWideChar(CP_UTF8, 0, data.as_ptr() as *const _, data.len() as _, ptr::null_mut(), 0)
    };

    if size != 0 {
        let mem = RawMem::new_global_mem((mem::size_of::<u16>() * (size as usize + 1)) as _)?;
        {
            let (ptr, _lock) = mem.lock()?;
            let ptr = ptr.as_ptr() as *mut u16;
            unsafe {
                MultiByteToWideChar(CP_UTF8, 0, data.as_ptr() as *const _, data.len() as _, ptr, size);
                ptr::write(ptr.offset(size as isize), 0);
            }
        }

        let _ = empty();

        if unsafe { !SetClipboardData(formats::CF_UNICODETEXT, mem.get()).is_null() } {
            //SetClipboardData takes ownership
            mem.release();
            return Ok(());
        }
    }

    Err(error_code::SystemError::last())
}

#[cfg(feature = "std")]
///Retrieves file list from clipboard, appending each element to the provided storage.
///
///Returns number of appended file names.
pub fn get_file_list_path(out: &mut alloc::vec::Vec<std::path::PathBuf>) -> SysResult<usize> {
    use std::os::windows::ffi::OsStringExt;

    let clipboard_data = RawMem::from_borrowed(get_clipboard_data(formats::CF_HDROP)?);

    let (_data_ptr, _lock) = clipboard_data.lock()?;

    let num_files = unsafe { DragQueryFileW(clipboard_data.get() as _, u32::MAX, ptr::null_mut(), 0) };
    out.reserve(num_files as usize);

    let mut buffer = alloc::vec::Vec::new();

    for idx in 0..num_files {
        let required_size_no_null = unsafe { DragQueryFileW(clipboard_data.get() as _, idx, ptr::null_mut(), 0) };
        if required_size_no_null == 0 {
            return Err(SystemError::last());
        }

        let required_size = required_size_no_null + 1;
        buffer.reserve(required_size as usize);

        if unsafe { DragQueryFileW(clipboard_data.get() as _, idx, buffer.as_mut_ptr(), required_size) == 0 } {
            return Err(SystemError::last());
        }

        unsafe {
            buffer.set_len(required_size_no_null as usize);
        }
        //This fucking abomination of API requires double allocation,
        //just because no one had brain for to provide API for creation OsString out of owned
        //Vec<16>
        out.push(std::ffi::OsString::from_wide(&buffer).into())
    }

    Ok(num_files as usize)
}

///Retrieves file list from clipboard, appending each element to the provided storage.
///
///Returns number of appended file names.
pub fn get_file_list(out: &mut alloc::vec::Vec<alloc::string::String>) -> SysResult<usize> {
    let clipboard_data = RawMem::from_borrowed(get_clipboard_data(formats::CF_HDROP)?);

    let (_data_ptr, _lock) = clipboard_data.lock()?;

    let num_files = unsafe { DragQueryFileW(clipboard_data.get() as _, u32::MAX, ptr::null_mut(), 0) };
    out.reserve(num_files as usize);

    let mut buffer = alloc::vec::Vec::new();

    for idx in 0..num_files {
        let required_size_no_null = unsafe { DragQueryFileW(clipboard_data.get() as _, idx, ptr::null_mut(), 0) };
        if required_size_no_null == 0 {
            return Err(SystemError::last());
        }

        let required_size = required_size_no_null + 1;
        buffer.reserve(required_size as usize);

        if unsafe { DragQueryFileW(clipboard_data.get() as _, idx, buffer.as_mut_ptr(), required_size) == 0 } {
            return Err(SystemError::last());
        }

        unsafe {
            buffer.set_len(required_size_no_null as usize);
        }
        out.push(alloc::string::String::from_utf16_lossy(&buffer));
    }

    Ok(num_files as usize)
}

///Reads bitmap image, appending image to the `out` vector and returning number of bytes read on
///success.
///
///Output will contain header following by RGB
pub fn get_bitmap(out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
    let clipboard_data = get_clipboard_data(formats::CF_BITMAP)?;

    //Thanks @matheuslessarodrigues
    let mut bitmap = BITMAP {
        bmType: 0,
        bmWidth: 0,
        bmHeight: 0,
        bmWidthBytes: 0,
        bmPlanes: 0,
        bmBitsPixel: 0,
        bmBits: ptr::null_mut(),
    };

    if unsafe { GetObjectW(clipboard_data.as_ptr(), mem::size_of::<BITMAP>() as _, &mut bitmap as *mut BITMAP as _) } == 0 {
        return Err(SystemError::last());
    }

    let clr_bits = bitmap.bmPlanes * bitmap.bmBitsPixel;
    let clr_bits = if clr_bits == 1 {
        1
    } else if clr_bits <= 4 {
        4
    } else if clr_bits <= 8 {
        8
    } else if clr_bits <= 16 {
        16
    } else if clr_bits <= 24 {
        24
    } else {
        32
    };

    let header_storage = RawMem::new_rust_mem(if clr_bits < 24 {
        mem::size_of::<BITMAPINFOHEADER>() + mem::size_of::<RGBQUAD>() * (1 << clr_bits)
    } else {
        mem::size_of::<BITMAPINFOHEADER>()
    })?;

    let header = unsafe {
        &mut *(header_storage.get() as *mut BITMAPINFO)
    };

    header.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as _;
    header.bmiHeader.biWidth = bitmap.bmWidth;
    header.bmiHeader.biHeight = bitmap.bmHeight;
    header.bmiHeader.biPlanes = bitmap.bmPlanes;
    header.bmiHeader.biBitCount = bitmap.bmBitsPixel;
    header.bmiHeader.biCompression = BI_RGB;
    if clr_bits < 24 {
        header.bmiHeader.biClrUsed = 1 << clr_bits;
    }

    header.bmiHeader.biSizeImage = ((((header.bmiHeader.biWidth * clr_bits + 31) & !31) / 8) * header.bmiHeader.biHeight) as _;
    header.bmiHeader.biClrImportant = 0;

    let img_size = header.bmiHeader.biSizeImage as usize;
    let out_before = out.len();

    let dc = crate::utils::Scope(unsafe { GetDC(ptr::null_mut()) }, free_dc);
    let mut buffer = alloc::vec::Vec::new();
    buffer.resize(img_size, 0u8);

    if unsafe { GetDIBits(dc.0, clipboard_data.as_ptr() as _, 0, bitmap.bmHeight as _, buffer.as_mut_ptr() as _, header_storage.get() as _, DIB_RGB_COLORS) } == 0 {
        return Err(SystemError::last());
    }

    //Write header
    out.extend_from_slice(&u16::to_le_bytes(0x4d42));
    out.extend_from_slice(&u32::to_le_bytes(mem::size_of::<BITMAPFILEHEADER>() as u32 + header.bmiHeader.biSize + header.bmiHeader.biClrUsed * mem::size_of::<RGBQUAD>() as u32 + header.bmiHeader.biSizeImage));
    out.extend_from_slice(&u32::to_le_bytes(0)); //2 * u16 of 0
    out.extend_from_slice(&u32::to_le_bytes(mem::size_of::<BITMAPFILEHEADER>() as u32 + header.bmiHeader.biSize + header.bmiHeader.biClrUsed * mem::size_of::<RGBQUAD>() as u32));

    out.extend_from_slice(&header.bmiHeader.biSize.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biWidth.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biHeight.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biPlanes.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biBitCount.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biCompression.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biSizeImage.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biXPelsPerMeter.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biYPelsPerMeter.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biClrUsed.to_le_bytes());
    out.extend_from_slice(&header.bmiHeader.biClrImportant.to_le_bytes());

    for color in unsafe { slice::from_raw_parts(header.bmiColors.as_ptr(), header.bmiHeader.biClrUsed as _) } {
        out.push(color.rgbBlue);
        out.push(color.rgbGreen);
        out.push(color.rgbRed);
        out.push(color.rgbReserved);
    }

    out.extend_from_slice(&buffer);

    Ok(out.len() - out_before)
}

#[inline(always)]
#[doc(hidden)]
pub fn set_bitamp(data: &[u8]) -> SysResult<()> {
    set_bitmap(data)
}

///Sets bitmap (header + RGB) onto clipboard, from raw bytes.
///
///Returns `ERROR_INCORRECT_SIZE` if size of data is not valid
pub fn set_bitmap(data: &[u8]) -> SysResult<()> {
    const FILE_HEADER_LEN: usize = mem::size_of::<BITMAPFILEHEADER>();
    const INFO_HEADER_LEN: usize = mem::size_of::<BITMAPINFOHEADER>();

    if data.len() <= (FILE_HEADER_LEN + INFO_HEADER_LEN) {
        return Err(SystemError::new(ERROR_INCORRECT_SIZE as _));
    }

    let mut file_header = mem::MaybeUninit::<BITMAPFILEHEADER>::uninit();
    let mut info_header = mem::MaybeUninit::<BITMAPINFOHEADER>::uninit();

    let (file_header, info_header) = unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), file_header.as_mut_ptr() as _, FILE_HEADER_LEN);
        ptr::copy_nonoverlapping(data.as_ptr().add(FILE_HEADER_LEN), info_header.as_mut_ptr() as _, INFO_HEADER_LEN);
        (file_header.assume_init(), info_header.assume_init())
    };

    if data.len() <= file_header.bfOffBits as usize {
        return Err(SystemError::new(ERROR_INCORRECT_SIZE as _));
    }

    let bitmap = &data[file_header.bfOffBits as _..];

    if bitmap.len() < info_header.biSizeImage as usize {
        return Err(SystemError::new(ERROR_INCORRECT_SIZE as _));
    }

    let dc = crate::utils::Scope(unsafe { GetDC(ptr::null_mut()) }, free_dc);

    let handle = unsafe {
        CreateDIBitmap(dc.0, &info_header as _, CBM_INIT, bitmap.as_ptr() as _, &info_header as *const _ as *const BITMAPINFO, DIB_RGB_COLORS)
    };

    if handle.is_null() {
        return Err(SystemError::last());
    }

    let _ = empty();
    if unsafe { SetClipboardData(formats::CF_BITMAP, handle as _).is_null() } {
        return Err(SystemError::last());
    }

    Ok(())
}

///Enumerator over available clipboard formats.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub struct EnumFormats {
    idx: u32
}

impl EnumFormats {
    /// Constructs enumerator over all available formats.
    pub fn new() -> EnumFormats {
        EnumFormats { idx: 0 }
    }

    /// Constructs enumerator that starts from format.
    pub fn from(format: u32) -> EnumFormats {
        EnumFormats { idx: format }
    }

    /// Resets enumerator to list all available formats.
    pub fn reset(&mut self) -> &EnumFormats {
        self.idx = 0;
        self
    }
}

impl Iterator for EnumFormats {
    type Item = u32;

    /// Returns next format on clipboard.
    ///
    /// In case of failure (e.g. clipboard is closed) returns `None`.
    fn next(&mut self) -> Option<u32> {
        self.idx = unsafe { EnumClipboardFormats(self.idx) };

        if self.idx == 0 {
            None
        } else {
            Some(self.idx)
        }
    }

    /// Relies on `count_formats` so it is only reliable
    /// when hinting size for enumeration of all formats.
    ///
    /// Doesn't require opened clipboard.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, count_formats())
    }
}

macro_rules! match_format_name_big {
    ( $name:expr, $( $f:ident ),* ) => {
        match $name {
            $( formats::$f => Some(stringify!($f).to_owned()),)*
            formats::CF_GDIOBJFIRST ..= formats::CF_GDIOBJLAST => Some(format!("CF_GDIOBJ{}", $name - formats::CF_GDIOBJFIRST)),
            formats::CF_PRIVATEFIRST ..= formats::CF_PRIVATELAST => Some(format!("CF_PRIVATE{}", $name - formats::CF_PRIVATEFIRST)),
            _ => {
                let format_buff = [0u16; 256];
                unsafe {
                    let buff_p = format_buff.as_ptr() as *mut u16;

                    match GetClipboardFormatNameW($name, buff_p, format_buff.len() as c_int) {
                        0 => None,
                        size => Some(String::from_utf16_lossy(&format_buff[..size as usize])),
                    }
                }
            }
        }
    }
}

macro_rules! match_format_name {
    ( $name:expr, $( $f:ident ),* ) => {
        use core::fmt::Write;
        let mut result = StrBuf::<[u8; 52]>::new();

        match $name {
            $( formats::$f => {
                let _ = result.push_str(stringify!($f));
            },)*
            formats::CF_GDIOBJFIRST ..= formats::CF_GDIOBJLAST => {
                let _ = write!(result, "CF_GDIOBJ{}", $name - formats::CF_GDIOBJFIRST);
            },
            formats::CF_PRIVATEFIRST ..= formats::CF_PRIVATELAST => {
                let _ = write!(result, "CF_PRIVATE{}", $name - formats::CF_PRIVATEFIRST);
            },
            _ => {
                let mut format_buff = [0u16; 52];
                unsafe {
                    let buff_p = format_buff.as_mut_ptr() as *mut u16;
                    match GetClipboardFormatNameW($name, buff_p, format_buff.len() as c_int) {
                        0 => return None,
                        len => match WideCharToMultiByte(winapi::um::winnls::CP_UTF8, 0, format_buff.as_ptr(), len, result.as_ptr() as *mut i8, result.remaining() as i32, ptr::null(), ptr::null_mut()) {
                            0 => return None,
                            len => result.set_len(len as u8),
                        }
                    }
                }
            }
        }

        return Some(result)
    }
}

///Returns format name based on it's code.
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Some``` Name of valid format.
///* ```None``` Format is invalid or doesn't exist.
pub fn format_name(format: u32) -> Option<StrBuf<[u8; 52]>> {
    match_format_name!(format,
                       CF_BITMAP,
                       CF_DIB,
                       CF_DIBV5,
                       CF_DIF,
                       CF_DSPBITMAP,
                       CF_DSPENHMETAFILE,
                       CF_DSPMETAFILEPICT,
                       CF_DSPTEXT,
                       CF_ENHMETAFILE,
                       CF_HDROP,
                       CF_LOCALE,
                       CF_METAFILEPICT,
                       CF_OEMTEXT,
                       CF_OWNERDISPLAY,
                       CF_PALETTE,
                       CF_PENDATA,
                       CF_RIFF,
                       CF_SYLK,
                       CF_TEXT,
                       CF_WAVE,
                       CF_TIFF,
                       CF_UNICODETEXT);
}

///Returns format name based on it's code (allocating variant suitable for big names)
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Some``` Name of valid format.
///* ```None``` Format is invalid or doesn't exist.
pub fn format_name_big(format: u32) -> Option<String> {
    match_format_name_big!(format,
                           CF_BITMAP,
                           CF_DIB,
                           CF_DIBV5,
                           CF_DIF,
                           CF_DSPBITMAP,
                           CF_DSPENHMETAFILE,
                           CF_DSPMETAFILEPICT,
                           CF_DSPTEXT,
                           CF_ENHMETAFILE,
                           CF_HDROP,
                           CF_LOCALE,
                           CF_METAFILEPICT,
                           CF_OEMTEXT,
                           CF_OWNERDISPLAY,
                           CF_PALETTE,
                           CF_PENDATA,
                           CF_RIFF,
                           CF_SYLK,
                           CF_TEXT,
                           CF_WAVE,
                           CF_TIFF,
                           CF_UNICODETEXT)
}

#[inline]
///Registers a new clipboard format with specified name as C wide string (meaning it must have null
///char at the end).
///
///# Returns:
///
///Newly registered format identifier, if successful.
///
///# Note:
///
///- Custom format identifier is in range `0xC000...0xFFFF`.
///- Function fails if input is not null terminated string.
pub unsafe fn register_raw_format(name: &[u16]) -> Option<NonZeroU32> {
    if name.is_empty() || name[name.len()-1] != b'\0' as u16 {
        return unlikely_empty_size_result()
    }
    NonZeroU32::new(RegisterClipboardFormatW(name.as_ptr()) )
}

///Registers a new clipboard format with specified name.
///
///# Returns:
///
///Newly registered format identifier, if successful.
///
///# Note:
///
///Custom format identifier is in range `0xC000...0xFFFF`.
pub fn register_format(name: &str) -> Option<NonZeroU32> {
    let size = unsafe {
        MultiByteToWideChar(CP_UTF8, 0, name.as_ptr() as *const _, name.len() as c_int, ptr::null_mut(), 0)
    };

    if size == 0 {
        return unlikely_empty_size_result()
    }

    if size > 52 {
        let mut buffer = alloc::vec::Vec::with_capacity(size as usize);
        let size = unsafe {
            MultiByteToWideChar(CP_UTF8, 0, name.as_ptr() as *const _, name.len() as c_int, buffer.as_mut_ptr(), size)
        };
        unsafe {
            buffer.set_len(size as usize);
            buffer.push(0);
            register_raw_format(&buffer)
        }
    } else {
        let mut buffer = mem::MaybeUninit::<[u16; 52]>::uninit();
        let size = unsafe {
            MultiByteToWideChar(CP_UTF8, 0, name.as_ptr() as *const _, name.len() as c_int, buffer.as_mut_ptr() as *mut u16, 51)
        };
        unsafe {
            ptr::write((buffer.as_mut_ptr() as *mut u16).offset(size as isize), 0);
            register_raw_format(slice::from_raw_parts(buffer.as_ptr() as *const u16, size as usize + 1))
        }
    }
}

#[inline(always)]
///Retrieves the window handle of the current owner of the clipboard.
///
///Returns `None` if clipboard is not owned.
pub fn get_owner() -> Option<ptr::NonNull::<winapi::shared::windef::HWND__>> {
    ptr::NonNull::new(unsafe {
        GetClipboardOwner()
    })
}
