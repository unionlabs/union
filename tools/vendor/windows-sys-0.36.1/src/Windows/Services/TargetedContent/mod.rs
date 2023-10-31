pub type TargetedContentAction = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_TargetedContent\"`*"]
#[repr(transparent)]
pub struct TargetedContentAppInstallationState(pub i32);
impl TargetedContentAppInstallationState {
    pub const NotApplicable: Self = Self(0i32);
    pub const NotInstalled: Self = Self(1i32);
    pub const Installed: Self = Self(2i32);
}
impl ::core::marker::Copy for TargetedContentAppInstallationState {}
impl ::core::clone::Clone for TargetedContentAppInstallationState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Services_TargetedContent\"`*"]
#[repr(transparent)]
pub struct TargetedContentAvailability(pub i32);
impl TargetedContentAvailability {
    pub const None: Self = Self(0i32);
    pub const Partial: Self = Self(1i32);
    pub const All: Self = Self(2i32);
}
impl ::core::marker::Copy for TargetedContentAvailability {}
impl ::core::clone::Clone for TargetedContentAvailability {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TargetedContentAvailabilityChangedEventArgs = *mut ::core::ffi::c_void;
pub type TargetedContentChangedEventArgs = *mut ::core::ffi::c_void;
pub type TargetedContentCollection = *mut ::core::ffi::c_void;
pub type TargetedContentContainer = *mut ::core::ffi::c_void;
pub type TargetedContentFile = *mut ::core::ffi::c_void;
pub type TargetedContentImage = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_TargetedContent\"`*"]
#[repr(transparent)]
pub struct TargetedContentInteraction(pub i32);
impl TargetedContentInteraction {
    pub const Impression: Self = Self(0i32);
    pub const ClickThrough: Self = Self(1i32);
    pub const Hover: Self = Self(2i32);
    pub const Like: Self = Self(3i32);
    pub const Dislike: Self = Self(4i32);
    pub const Dismiss: Self = Self(5i32);
    pub const Ineligible: Self = Self(6i32);
    pub const Accept: Self = Self(7i32);
    pub const Decline: Self = Self(8i32);
    pub const Defer: Self = Self(9i32);
    pub const Canceled: Self = Self(10i32);
    pub const Conversion: Self = Self(11i32);
    pub const Opportunity: Self = Self(12i32);
}
impl ::core::marker::Copy for TargetedContentInteraction {}
impl ::core::clone::Clone for TargetedContentInteraction {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TargetedContentItem = *mut ::core::ffi::c_void;
pub type TargetedContentItemState = *mut ::core::ffi::c_void;
pub type TargetedContentObject = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_TargetedContent\"`*"]
#[repr(transparent)]
pub struct TargetedContentObjectKind(pub i32);
impl TargetedContentObjectKind {
    pub const Collection: Self = Self(0i32);
    pub const Item: Self = Self(1i32);
    pub const Value: Self = Self(2i32);
}
impl ::core::marker::Copy for TargetedContentObjectKind {}
impl ::core::clone::Clone for TargetedContentObjectKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TargetedContentStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type TargetedContentSubscription = *mut ::core::ffi::c_void;
pub type TargetedContentSubscriptionOptions = *mut ::core::ffi::c_void;
pub type TargetedContentValue = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_TargetedContent\"`*"]
#[repr(transparent)]
pub struct TargetedContentValueKind(pub i32);
impl TargetedContentValueKind {
    pub const String: Self = Self(0i32);
    pub const Uri: Self = Self(1i32);
    pub const Number: Self = Self(2i32);
    pub const Boolean: Self = Self(3i32);
    pub const File: Self = Self(4i32);
    pub const ImageFile: Self = Self(5i32);
    pub const Action: Self = Self(6i32);
    pub const Strings: Self = Self(7i32);
    pub const Uris: Self = Self(8i32);
    pub const Numbers: Self = Self(9i32);
    pub const Booleans: Self = Self(10i32);
    pub const Files: Self = Self(11i32);
    pub const ImageFiles: Self = Self(12i32);
    pub const Actions: Self = Self(13i32);
}
impl ::core::marker::Copy for TargetedContentValueKind {}
impl ::core::clone::Clone for TargetedContentValueKind {
    fn clone(&self) -> Self {
        *self
    }
}
