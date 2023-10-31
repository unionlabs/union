#![allow(dead_code)]
//! Standard clipboard formats.
//!
//! Header: Winuser.h
//!
//! Description is taken from [Standard Clipboard Formats](https://msdn.microsoft.com/en-us/library/windows/desktop/ff729168%28v=vs.85%29.aspx)

use crate::{SysResult, Getter, Setter};

use winapi::um::winuser;

///A handle to a bitmap (HBITMAP).
pub const CF_BITMAP: u32 = winuser::CF_BITMAP;
///A memory object containing a <b>BITMAPINFO</b> structure followed by the bitmap bits.
pub const CF_DIB: u32 = winuser::CF_DIB;
///A memory object containing a <b>BITMAPV5HEADER</b> structure followed by the bitmap color space
///information and the bitmap bits.
pub const CF_DIBV5: u32 = winuser::CF_DIBV5;
///Software Arts' Data Interchange Format.
pub const CF_DIF: u32 = winuser::CF_DIF;
///Bitmap display format associated with a private format. The hMem parameter must be a handle to
///data that can be displayed in bitmap format in lieu of the privately formatted data.
pub const CF_DSPBITMAP: u32 = winuser::CF_DSPBITMAP;
///Enhanced metafile display format associated with a private format. The *hMem* parameter must be a
///handle to data that can be displayed in enhanced metafile format in lieu of the privately
///formatted data.
pub const CF_DSPENHMETAFILE: u32 = winuser::CF_DSPENHMETAFILE;
///Metafile-picture display format associated with a private format. The hMem parameter must be a
///handle to data that can be displayed in metafile-picture format in lieu of the privately
///formatted data.
pub const CF_DSPMETAFILEPICT: u32 = winuser::CF_DSPMETAFILEPICT;
///Text display format associated with a private format. The *hMem* parameter must be a handle to
///data that can be displayed in text format in lieu of the privately formatted data.
pub const CF_DSPTEXT: u32 = winuser::CF_DSPTEXT;
///A handle to an enhanced metafile (<b>HENHMETAFILE</b>).
pub const CF_ENHMETAFILE: u32 = winuser::CF_ENHMETAFILE;
///Start of a range of integer values for application-defined GDI object clipboard formats.
pub const CF_GDIOBJFIRST: u32 = winuser::CF_GDIOBJFIRST;
///End of a range of integer values for application-defined GDI object clipboard formats.
pub const CF_GDIOBJLAST: u32 = winuser::CF_GDIOBJLAST;
///A handle to type <b>HDROP</b> that identifies a list of files.
pub const CF_HDROP: u32 = winuser::CF_HDROP;
///The data is a handle to the locale identifier associated with text in the clipboard.
///
///For details see [Standart Clipboard Formats](https://msdn.microsoft.com/en-us/library/windows/desktop/ff729168%28v=vs.85%29.aspx)
pub const CF_LOCALE: u32 = winuser::CF_LOCALE;
///Handle to a metafile picture format as defined by the <b>METAFILEPICT</b> structure.
pub const CF_METAFILEPICT: u32 = winuser::CF_METAFILEPICT;
///Text format containing characters in the OEM character set.
pub const CF_OEMTEXT: u32 = winuser::CF_OEMTEXT;
///Owner-display format.
///
///For details see [Standart Clipboard Formats](https://msdn.microsoft.com/en-us/library/windows/desktop/ff729168%28v=vs.85%29.aspx)
pub const CF_OWNERDISPLAY: u32 = winuser::CF_OWNERDISPLAY;
///Handle to a color palette.
///
///For details see [Standart Clipboard Formats](https://msdn.microsoft.com/en-us/library/windows/desktop/ff729168%28v=vs.85%29.aspx)
pub const CF_PALETTE: u32 = winuser::CF_PALETTE;
///Data for the pen extensions to the Microsoft Windows for Pen Computing.
pub const CF_PENDATA: u32 = winuser::CF_PENDATA;
///Start of a range of integer values for private clipboard formats.
pub const CF_PRIVATEFIRST: u32 = winuser::CF_PRIVATEFIRST;
///End of a range of integer values for private clipboard formats.
pub const CF_PRIVATELAST: u32 = winuser::CF_PRIVATELAST;
///Represents audio data more complex than can be represented in a ```CF_WAVE``` standard wave format.
pub const CF_RIFF: u32 = winuser::CF_RIFF;
///Microsoft Symbolic Link (SYLK) format.
pub const CF_SYLK: u32 = winuser::CF_SYLK;
///ANSI text format.
pub const CF_TEXT: u32 = winuser::CF_TEXT;
///Tagged-image file format.
pub const CF_TIFF: u32 = winuser::CF_TIFF;
///UTF16 text format.
pub const CF_UNICODETEXT: u32 = winuser::CF_UNICODETEXT;
///Represents audio data in one of the standard wave formats.
pub const CF_WAVE: u32 = winuser::CF_WAVE;

///Format to write/read from clipboard as raw bytes
///
///Has to be initialized with format `id`
pub struct RawData(pub u32);

impl<T: AsRef<[u8]>> Setter<T> for RawData {
    #[inline(always)]
    fn write_clipboard(&self, data: &T) -> SysResult<()> {
        crate::raw::set(self.0, data.as_ref())
    }
}

impl Getter<alloc::vec::Vec<u8>> for RawData {
    #[inline(always)]
    fn read_clipboard(&self, out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
        crate::raw::get_vec(self.0, out)
    }
}

///Format to read/write unicode string.
///
///Refer to `Getter` and `Setter`
pub struct Unicode;

impl Getter<alloc::vec::Vec<u8>> for Unicode {
    #[inline(always)]
    fn read_clipboard(&self, out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
        crate::raw::get_string(out)
    }
}

impl Getter<alloc::string::String> for Unicode {
    #[inline(always)]
    fn read_clipboard(&self, out: &mut alloc::string::String) -> SysResult<usize> {
        self.read_clipboard(unsafe { out.as_mut_vec() })
    }
}

impl<T: AsRef<str>> Setter<T> for Unicode {
    #[inline(always)]
    fn write_clipboard(&self, data: &T) -> SysResult<()> {
        crate::raw::set_string(data.as_ref())
    }
}

///Format for file lists (generated by drag & drop).
///
///Corresponds to `CF_HDROP`
///
///`read_clipboard` returns number of file names
pub struct FileList;

impl Getter<alloc::vec::Vec<alloc::string::String>> for FileList {
    #[inline(always)]
    fn read_clipboard(&self, out: &mut alloc::vec::Vec<alloc::string::String>) -> SysResult<usize> {
        crate::raw::get_file_list(out)
    }
}

#[cfg(feature = "std")]
impl Getter<alloc::vec::Vec<std::path::PathBuf>> for FileList {
    #[inline(always)]
    fn read_clipboard(&self, out: &mut alloc::vec::Vec<std::path::PathBuf>) -> SysResult<usize> {
        crate::raw::get_file_list_path(out)
    }
}

///Format for bitmap images i.e. `CF_BITMAP`.
///
///Both `Getter` and `Setter` expects image as header and rgb payload
pub struct Bitmap;

impl Getter<alloc::vec::Vec<u8>> for Bitmap {
    #[inline(always)]
    fn read_clipboard(&self, out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
        crate::raw::get_bitmap(out)
    }
}

impl<T: AsRef<[u8]>> Setter<T> for Bitmap {
    #[inline(always)]
    fn write_clipboard(&self, data: &T) -> SysResult<()> {
        crate::raw::set_bitmap(data.as_ref())
    }
}
