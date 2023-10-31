pub type Block = *mut ::core::ffi::c_void;
pub type BlockCollection = *mut ::core::ffi::c_void;
pub type Bold = *mut ::core::ffi::c_void;
pub type ContactContentLinkProvider = *mut ::core::ffi::c_void;
pub type ContentLink = *mut ::core::ffi::c_void;
pub type ContentLinkInvokedEventArgs = *mut ::core::ffi::c_void;
pub type ContentLinkProvider = *mut ::core::ffi::c_void;
pub type ContentLinkProviderCollection = *mut ::core::ffi::c_void;
pub type Glyphs = *mut ::core::ffi::c_void;
pub type Hyperlink = *mut ::core::ffi::c_void;
pub type HyperlinkClickEventArgs = *mut ::core::ffi::c_void;
pub type Inline = *mut ::core::ffi::c_void;
pub type InlineCollection = *mut ::core::ffi::c_void;
pub type InlineUIContainer = *mut ::core::ffi::c_void;
pub type Italic = *mut ::core::ffi::c_void;
pub type LineBreak = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Documents\"`*"]
#[repr(transparent)]
pub struct LogicalDirection(pub i32);
impl LogicalDirection {
    pub const Backward: Self = Self(0i32);
    pub const Forward: Self = Self(1i32);
}
impl ::core::marker::Copy for LogicalDirection {}
impl ::core::clone::Clone for LogicalDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Paragraph = *mut ::core::ffi::c_void;
pub type PlaceContentLinkProvider = *mut ::core::ffi::c_void;
pub type Run = *mut ::core::ffi::c_void;
pub type Span = *mut ::core::ffi::c_void;
pub type TextElement = *mut ::core::ffi::c_void;
pub type TextHighlighter = *mut ::core::ffi::c_void;
pub type TextHighlighterBase = *mut ::core::ffi::c_void;
pub type TextPointer = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Documents\"`*"]
pub struct TextRange {
    pub StartIndex: i32,
    pub Length: i32,
}
impl ::core::marker::Copy for TextRange {}
impl ::core::clone::Clone for TextRange {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Typography = *mut ::core::ffi::c_void;
pub type Underline = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Documents\"`*"]
#[repr(transparent)]
pub struct UnderlineStyle(pub i32);
impl UnderlineStyle {
    pub const None: Self = Self(0i32);
    pub const Single: Self = Self(1i32);
}
impl ::core::marker::Copy for UnderlineStyle {}
impl ::core::clone::Clone for UnderlineStyle {
    fn clone(&self) -> Self {
        *self
    }
}
