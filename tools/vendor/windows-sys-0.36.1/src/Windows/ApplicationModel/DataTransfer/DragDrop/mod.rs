#[cfg(feature = "ApplicationModel_DataTransfer_DragDrop_Core")]
pub mod Core;
#[doc = "*Required features: `\"ApplicationModel_DataTransfer_DragDrop\"`*"]
#[repr(transparent)]
pub struct DragDropModifiers(pub u32);
impl DragDropModifiers {
    pub const None: Self = Self(0u32);
    pub const Shift: Self = Self(1u32);
    pub const Control: Self = Self(2u32);
    pub const Alt: Self = Self(4u32);
    pub const LeftButton: Self = Self(8u32);
    pub const MiddleButton: Self = Self(16u32);
    pub const RightButton: Self = Self(32u32);
}
impl ::core::marker::Copy for DragDropModifiers {}
impl ::core::clone::Clone for DragDropModifiers {
    fn clone(&self) -> Self {
        *self
    }
}
