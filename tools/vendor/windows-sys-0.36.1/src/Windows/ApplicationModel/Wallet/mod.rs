#[cfg(feature = "ApplicationModel_Wallet_System")]
pub mod System;
#[doc = "*Required features: `\"ApplicationModel_Wallet\"`*"]
#[repr(transparent)]
pub struct WalletActionKind(pub i32);
impl WalletActionKind {
    pub const OpenItem: Self = Self(0i32);
    pub const Transaction: Self = Self(1i32);
    pub const MoreTransactions: Self = Self(2i32);
    pub const Message: Self = Self(3i32);
    pub const Verb: Self = Self(4i32);
}
impl ::core::marker::Copy for WalletActionKind {}
impl ::core::clone::Clone for WalletActionKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WalletBarcode = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Wallet\"`*"]
#[repr(transparent)]
pub struct WalletBarcodeSymbology(pub i32);
impl WalletBarcodeSymbology {
    pub const Invalid: Self = Self(0i32);
    pub const Upca: Self = Self(1i32);
    pub const Upce: Self = Self(2i32);
    pub const Ean13: Self = Self(3i32);
    pub const Ean8: Self = Self(4i32);
    pub const Itf: Self = Self(5i32);
    pub const Code39: Self = Self(6i32);
    pub const Code128: Self = Self(7i32);
    pub const Qr: Self = Self(8i32);
    pub const Pdf417: Self = Self(9i32);
    pub const Aztec: Self = Self(10i32);
    pub const Custom: Self = Self(100000i32);
}
impl ::core::marker::Copy for WalletBarcodeSymbology {}
impl ::core::clone::Clone for WalletBarcodeSymbology {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_Wallet\"`*"]
#[repr(transparent)]
pub struct WalletDetailViewPosition(pub i32);
impl WalletDetailViewPosition {
    pub const Hidden: Self = Self(0i32);
    pub const HeaderField1: Self = Self(1i32);
    pub const HeaderField2: Self = Self(2i32);
    pub const PrimaryField1: Self = Self(3i32);
    pub const PrimaryField2: Self = Self(4i32);
    pub const SecondaryField1: Self = Self(5i32);
    pub const SecondaryField2: Self = Self(6i32);
    pub const SecondaryField3: Self = Self(7i32);
    pub const SecondaryField4: Self = Self(8i32);
    pub const SecondaryField5: Self = Self(9i32);
    pub const CenterField1: Self = Self(10i32);
    pub const FooterField1: Self = Self(11i32);
    pub const FooterField2: Self = Self(12i32);
    pub const FooterField3: Self = Self(13i32);
    pub const FooterField4: Self = Self(14i32);
}
impl ::core::marker::Copy for WalletDetailViewPosition {}
impl ::core::clone::Clone for WalletDetailViewPosition {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WalletItem = *mut ::core::ffi::c_void;
pub type WalletItemCustomProperty = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Wallet\"`*"]
#[repr(transparent)]
pub struct WalletItemKind(pub i32);
impl WalletItemKind {
    pub const Invalid: Self = Self(0i32);
    pub const Deal: Self = Self(1i32);
    pub const General: Self = Self(2i32);
    pub const PaymentInstrument: Self = Self(3i32);
    pub const Ticket: Self = Self(4i32);
    pub const BoardingPass: Self = Self(5i32);
    pub const MembershipCard: Self = Self(6i32);
}
impl ::core::marker::Copy for WalletItemKind {}
impl ::core::clone::Clone for WalletItemKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WalletItemStore = *mut ::core::ffi::c_void;
pub type WalletRelevantLocation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Wallet\"`*"]
#[repr(transparent)]
pub struct WalletSummaryViewPosition(pub i32);
impl WalletSummaryViewPosition {
    pub const Hidden: Self = Self(0i32);
    pub const Field1: Self = Self(1i32);
    pub const Field2: Self = Self(2i32);
}
impl ::core::marker::Copy for WalletSummaryViewPosition {}
impl ::core::clone::Clone for WalletSummaryViewPosition {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WalletTransaction = *mut ::core::ffi::c_void;
pub type WalletVerb = *mut ::core::ffi::c_void;
