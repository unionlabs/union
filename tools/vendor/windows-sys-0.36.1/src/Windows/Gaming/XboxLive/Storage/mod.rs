pub type GameSaveBlobGetResult = *mut ::core::ffi::c_void;
pub type GameSaveBlobInfo = *mut ::core::ffi::c_void;
pub type GameSaveBlobInfoGetResult = *mut ::core::ffi::c_void;
pub type GameSaveBlobInfoQuery = *mut ::core::ffi::c_void;
pub type GameSaveContainer = *mut ::core::ffi::c_void;
pub type GameSaveContainerInfo = *mut ::core::ffi::c_void;
pub type GameSaveContainerInfoGetResult = *mut ::core::ffi::c_void;
pub type GameSaveContainerInfoQuery = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Gaming_XboxLive_Storage\"`*"]
#[repr(transparent)]
pub struct GameSaveErrorStatus(pub i32);
impl GameSaveErrorStatus {
    pub const Ok: Self = Self(0i32);
    pub const Abort: Self = Self(-2147467260i32);
    pub const InvalidContainerName: Self = Self(-2138898431i32);
    pub const NoAccess: Self = Self(-2138898430i32);
    pub const OutOfLocalStorage: Self = Self(-2138898429i32);
    pub const UserCanceled: Self = Self(-2138898428i32);
    pub const UpdateTooBig: Self = Self(-2138898427i32);
    pub const QuotaExceeded: Self = Self(-2138898426i32);
    pub const ProvidedBufferTooSmall: Self = Self(-2138898425i32);
    pub const BlobNotFound: Self = Self(-2138898424i32);
    pub const NoXboxLiveInfo: Self = Self(-2138898423i32);
    pub const ContainerNotInSync: Self = Self(-2138898422i32);
    pub const ContainerSyncFailed: Self = Self(-2138898421i32);
    pub const UserHasNoXboxLiveInfo: Self = Self(-2138898420i32);
    pub const ObjectExpired: Self = Self(-2138898419i32);
}
impl ::core::marker::Copy for GameSaveErrorStatus {}
impl ::core::clone::Clone for GameSaveErrorStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GameSaveOperationResult = *mut ::core::ffi::c_void;
pub type GameSaveProvider = *mut ::core::ffi::c_void;
pub type GameSaveProviderGetResult = *mut ::core::ffi::c_void;
