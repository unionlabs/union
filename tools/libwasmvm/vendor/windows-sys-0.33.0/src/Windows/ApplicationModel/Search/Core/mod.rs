#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
pub type RequestingFocusOnKeyboardInputEventArgs = *mut ::core::ffi::c_void;
pub type SearchSuggestion = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'ApplicationModel_Search_Core'*"]
#[repr(transparent)]
pub struct SearchSuggestionKind(pub i32);
impl SearchSuggestionKind {
    pub const Query: Self = Self(0i32);
    pub const Result: Self = Self(1i32);
    pub const Separator: Self = Self(2i32);
}
impl ::core::marker::Copy for SearchSuggestionKind {}
impl ::core::clone::Clone for SearchSuggestionKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SearchSuggestionManager = *mut ::core::ffi::c_void;
pub type SearchSuggestionsRequestedEventArgs = *mut ::core::ffi::c_void;
