#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
pub type Playlist = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'Media_Playlists'*"]
#[repr(transparent)]
pub struct PlaylistFormat(pub i32);
impl PlaylistFormat {
    pub const WindowsMedia: Self = Self(0i32);
    pub const Zune: Self = Self(1i32);
    pub const M3u: Self = Self(2i32);
}
impl ::core::marker::Copy for PlaylistFormat {}
impl ::core::clone::Clone for PlaylistFormat {
    fn clone(&self) -> Self {
        *self
    }
}
