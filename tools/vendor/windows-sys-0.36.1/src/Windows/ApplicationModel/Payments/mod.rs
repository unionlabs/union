#[cfg(feature = "ApplicationModel_Payments_Provider")]
pub mod Provider;
pub type PaymentAddress = *mut ::core::ffi::c_void;
pub type PaymentCanMakePaymentResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Payments\"`*"]
#[repr(transparent)]
pub struct PaymentCanMakePaymentResultStatus(pub i32);
impl PaymentCanMakePaymentResultStatus {
    pub const Unknown: Self = Self(0i32);
    pub const Yes: Self = Self(1i32);
    pub const No: Self = Self(2i32);
    pub const NotAllowed: Self = Self(3i32);
    pub const UserNotSignedIn: Self = Self(4i32);
    pub const SpecifiedPaymentMethodIdsNotSupported: Self = Self(5i32);
    pub const NoQualifyingCardOnFile: Self = Self(6i32);
}
impl ::core::marker::Copy for PaymentCanMakePaymentResultStatus {}
impl ::core::clone::Clone for PaymentCanMakePaymentResultStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PaymentCurrencyAmount = *mut ::core::ffi::c_void;
pub type PaymentDetails = *mut ::core::ffi::c_void;
pub type PaymentDetailsModifier = *mut ::core::ffi::c_void;
pub type PaymentItem = *mut ::core::ffi::c_void;
pub type PaymentMediator = *mut ::core::ffi::c_void;
pub type PaymentMerchantInfo = *mut ::core::ffi::c_void;
pub type PaymentMethodData = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Payments\"`*"]
#[repr(transparent)]
pub struct PaymentOptionPresence(pub i32);
impl PaymentOptionPresence {
    pub const None: Self = Self(0i32);
    pub const Optional: Self = Self(1i32);
    pub const Required: Self = Self(2i32);
}
impl ::core::marker::Copy for PaymentOptionPresence {}
impl ::core::clone::Clone for PaymentOptionPresence {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PaymentOptions = *mut ::core::ffi::c_void;
pub type PaymentRequest = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Payments\"`*"]
#[repr(transparent)]
pub struct PaymentRequestChangeKind(pub i32);
impl PaymentRequestChangeKind {
    pub const ShippingOption: Self = Self(0i32);
    pub const ShippingAddress: Self = Self(1i32);
}
impl ::core::marker::Copy for PaymentRequestChangeKind {}
impl ::core::clone::Clone for PaymentRequestChangeKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PaymentRequestChangedArgs = *mut ::core::ffi::c_void;
pub type PaymentRequestChangedHandler = *mut ::core::ffi::c_void;
pub type PaymentRequestChangedResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Payments\"`*"]
#[repr(transparent)]
pub struct PaymentRequestCompletionStatus(pub i32);
impl PaymentRequestCompletionStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const Failed: Self = Self(1i32);
    pub const Unknown: Self = Self(2i32);
}
impl ::core::marker::Copy for PaymentRequestCompletionStatus {}
impl ::core::clone::Clone for PaymentRequestCompletionStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_Payments\"`*"]
#[repr(transparent)]
pub struct PaymentRequestStatus(pub i32);
impl PaymentRequestStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const Failed: Self = Self(1i32);
    pub const Canceled: Self = Self(2i32);
}
impl ::core::marker::Copy for PaymentRequestStatus {}
impl ::core::clone::Clone for PaymentRequestStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PaymentRequestSubmitResult = *mut ::core::ffi::c_void;
pub type PaymentResponse = *mut ::core::ffi::c_void;
pub type PaymentShippingOption = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Payments\"`*"]
#[repr(transparent)]
pub struct PaymentShippingType(pub i32);
impl PaymentShippingType {
    pub const Shipping: Self = Self(0i32);
    pub const Delivery: Self = Self(1i32);
    pub const Pickup: Self = Self(2i32);
}
impl ::core::marker::Copy for PaymentShippingType {}
impl ::core::clone::Clone for PaymentShippingType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PaymentToken = *mut ::core::ffi::c_void;
