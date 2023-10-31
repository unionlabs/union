#[cfg(feature = "Storage_Pickers_Provider")]
pub mod Provider;
pub type FileExtensionVector = *mut ::core::ffi::c_void;
pub type FileOpenPicker = *mut ::core::ffi::c_void;
pub type FilePickerFileTypesOrderedMap = *mut ::core::ffi::c_void;
pub type FilePickerSelectedFilesArray = *mut ::core::ffi::c_void;
pub type FileSavePicker = *mut ::core::ffi::c_void;
pub type FolderPicker = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Pickers\"`*"]
#[repr(transparent)]
pub struct PickerLocationId(pub i32);
impl PickerLocationId {
    pub const DocumentsLibrary: Self = Self(0i32);
    pub const ComputerFolder: Self = Self(1i32);
    pub const Desktop: Self = Self(2i32);
    pub const Downloads: Self = Self(3i32);
    pub const HomeGroup: Self = Self(4i32);
    pub const MusicLibrary: Self = Self(5i32);
    pub const PicturesLibrary: Self = Self(6i32);
    pub const VideosLibrary: Self = Self(7i32);
    pub const Objects3D: Self = Self(8i32);
    pub const Unspecified: Self = Self(9i32);
}
impl ::core::marker::Copy for PickerLocationId {}
impl ::core::clone::Clone for PickerLocationId {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Storage_Pickers\"`*"]
#[repr(transparent)]
pub struct PickerViewMode(pub i32);
impl PickerViewMode {
    pub const List: Self = Self(0i32);
    pub const Thumbnail: Self = Self(1i32);
}
impl ::core::marker::Copy for PickerViewMode {}
impl ::core::clone::Clone for PickerViewMode {
    fn clone(&self) -> Self {
        *self
    }
}
