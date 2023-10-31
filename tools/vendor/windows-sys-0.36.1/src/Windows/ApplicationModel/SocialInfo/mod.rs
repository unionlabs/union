#[cfg(feature = "ApplicationModel_SocialInfo_Provider")]
pub mod Provider;
pub type SocialFeedChildItem = *mut ::core::ffi::c_void;
pub type SocialFeedContent = *mut ::core::ffi::c_void;
pub type SocialFeedItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_SocialInfo\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct SocialFeedItemStyle(pub i32);
#[cfg(feature = "deprecated")]
impl SocialFeedItemStyle {
    pub const Default: Self = Self(0i32);
    pub const Photo: Self = Self(1i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for SocialFeedItemStyle {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for SocialFeedItemStyle {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_SocialInfo\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct SocialFeedKind(pub i32);
#[cfg(feature = "deprecated")]
impl SocialFeedKind {
    pub const HomeFeed: Self = Self(0i32);
    pub const ContactFeed: Self = Self(1i32);
    pub const Dashboard: Self = Self(2i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for SocialFeedKind {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for SocialFeedKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SocialFeedSharedItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_SocialInfo\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct SocialFeedUpdateMode(pub i32);
#[cfg(feature = "deprecated")]
impl SocialFeedUpdateMode {
    pub const Append: Self = Self(0i32);
    pub const Replace: Self = Self(1i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for SocialFeedUpdateMode {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for SocialFeedUpdateMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_SocialInfo\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct SocialItemBadgeStyle(pub i32);
#[cfg(feature = "deprecated")]
impl SocialItemBadgeStyle {
    pub const Hidden: Self = Self(0i32);
    pub const Visible: Self = Self(1i32);
    pub const VisibleWithCount: Self = Self(2i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for SocialItemBadgeStyle {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for SocialItemBadgeStyle {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SocialItemThumbnail = *mut ::core::ffi::c_void;
pub type SocialUserInfo = *mut ::core::ffi::c_void;
