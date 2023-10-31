pub type PushNotificationChannel = *mut ::core::ffi::c_void;
pub type PushNotificationChannelManagerForUser = *mut ::core::ffi::c_void;
pub type PushNotificationChannelsRevokedEventArgs = *mut ::core::ffi::c_void;
pub type PushNotificationReceivedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_PushNotifications\"`*"]
#[repr(transparent)]
pub struct PushNotificationType(pub i32);
impl PushNotificationType {
    pub const Toast: Self = Self(0i32);
    pub const Tile: Self = Self(1i32);
    pub const Badge: Self = Self(2i32);
    pub const Raw: Self = Self(3i32);
    pub const TileFlyout: Self = Self(4i32);
}
impl ::core::marker::Copy for PushNotificationType {}
impl ::core::clone::Clone for PushNotificationType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RawNotification = *mut ::core::ffi::c_void;
