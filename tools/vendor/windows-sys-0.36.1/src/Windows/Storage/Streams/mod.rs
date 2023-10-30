pub type Buffer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Streams\"`*"]
#[repr(transparent)]
pub struct ByteOrder(pub i32);
impl ByteOrder {
    pub const LittleEndian: Self = Self(0i32);
    pub const BigEndian: Self = Self(1i32);
}
impl ::core::marker::Copy for ByteOrder {}
impl ::core::clone::Clone for ByteOrder {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DataReader = *mut ::core::ffi::c_void;
pub type DataReaderLoadOperation = *mut ::core::ffi::c_void;
pub type DataWriter = *mut ::core::ffi::c_void;
pub type DataWriterStoreOperation = *mut ::core::ffi::c_void;
pub type FileInputStream = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Streams\"`*"]
#[repr(transparent)]
pub struct FileOpenDisposition(pub i32);
impl FileOpenDisposition {
    pub const OpenExisting: Self = Self(0i32);
    pub const OpenAlways: Self = Self(1i32);
    pub const CreateNew: Self = Self(2i32);
    pub const CreateAlways: Self = Self(3i32);
    pub const TruncateExisting: Self = Self(4i32);
}
impl ::core::marker::Copy for FileOpenDisposition {}
impl ::core::clone::Clone for FileOpenDisposition {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FileOutputStream = *mut ::core::ffi::c_void;
pub type FileRandomAccessStream = *mut ::core::ffi::c_void;
pub type IBuffer = *mut ::core::ffi::c_void;
pub type IContentTypeProvider = *mut ::core::ffi::c_void;
pub type IDataReader = *mut ::core::ffi::c_void;
pub type IDataWriter = *mut ::core::ffi::c_void;
pub type IInputStream = *mut ::core::ffi::c_void;
pub type IInputStreamReference = *mut ::core::ffi::c_void;
pub type IOutputStream = *mut ::core::ffi::c_void;
pub type IPropertySetSerializer = *mut ::core::ffi::c_void;
pub type IRandomAccessStream = *mut ::core::ffi::c_void;
pub type IRandomAccessStreamReference = *mut ::core::ffi::c_void;
pub type IRandomAccessStreamWithContentType = *mut ::core::ffi::c_void;
pub type InMemoryRandomAccessStream = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Streams\"`*"]
#[repr(transparent)]
pub struct InputStreamOptions(pub u32);
impl InputStreamOptions {
    pub const None: Self = Self(0u32);
    pub const Partial: Self = Self(1u32);
    pub const ReadAhead: Self = Self(2u32);
}
impl ::core::marker::Copy for InputStreamOptions {}
impl ::core::clone::Clone for InputStreamOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InputStreamOverStream = *mut ::core::ffi::c_void;
pub type OutputStreamOverStream = *mut ::core::ffi::c_void;
pub type RandomAccessStreamOverStream = *mut ::core::ffi::c_void;
pub type RandomAccessStreamReference = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Streams\"`*"]
#[repr(transparent)]
pub struct UnicodeEncoding(pub i32);
impl UnicodeEncoding {
    pub const Utf8: Self = Self(0i32);
    pub const Utf16LE: Self = Self(1i32);
    pub const Utf16BE: Self = Self(2i32);
}
impl ::core::marker::Copy for UnicodeEncoding {}
impl ::core::clone::Clone for UnicodeEncoding {
    fn clone(&self) -> Self {
        *self
    }
}
