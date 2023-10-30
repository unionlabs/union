#[doc = "*Required features: `\"Gaming_Preview_GamesEnumeration\"`*"]
#[repr(transparent)]
pub struct GameListCategory(pub i32);
impl GameListCategory {
    pub const Candidate: Self = Self(0i32);
    pub const ConfirmedBySystem: Self = Self(1i32);
    pub const ConfirmedByUser: Self = Self(2i32);
}
impl ::core::marker::Copy for GameListCategory {}
impl ::core::clone::Clone for GameListCategory {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GameListChangedEventHandler = *mut ::core::ffi::c_void;
pub type GameListEntry = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Gaming_Preview_GamesEnumeration\"`*"]
#[repr(transparent)]
pub struct GameListEntryLaunchableState(pub i32);
impl GameListEntryLaunchableState {
    pub const NotLaunchable: Self = Self(0i32);
    pub const ByLastRunningFullPath: Self = Self(1i32);
    pub const ByUserProvidedPath: Self = Self(2i32);
    pub const ByTile: Self = Self(3i32);
}
impl ::core::marker::Copy for GameListEntryLaunchableState {}
impl ::core::clone::Clone for GameListEntryLaunchableState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GameListRemovedEventHandler = *mut ::core::ffi::c_void;
pub type GameModeConfiguration = *mut ::core::ffi::c_void;
pub type GameModeUserConfiguration = *mut ::core::ffi::c_void;
pub type IGameListEntry = *mut ::core::ffi::c_void;
