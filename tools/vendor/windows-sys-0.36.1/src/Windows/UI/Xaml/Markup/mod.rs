pub type IComponentConnector = *mut ::core::ffi::c_void;
pub type IComponentConnector2 = *mut ::core::ffi::c_void;
pub type IDataTemplateComponent = *mut ::core::ffi::c_void;
pub type IXamlBindScopeDiagnostics = *mut ::core::ffi::c_void;
pub type IXamlMember = *mut ::core::ffi::c_void;
pub type IXamlMetadataProvider = *mut ::core::ffi::c_void;
pub type IXamlType = *mut ::core::ffi::c_void;
pub type IXamlType2 = *mut ::core::ffi::c_void;
pub type MarkupExtension = *mut ::core::ffi::c_void;
pub type XamlBinaryWriter = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Markup\"`*"]
pub struct XamlBinaryWriterErrorInformation {
    pub InputStreamIndex: u32,
    pub LineNumber: u32,
    pub LinePosition: u32,
}
impl ::core::marker::Copy for XamlBinaryWriterErrorInformation {}
impl ::core::clone::Clone for XamlBinaryWriterErrorInformation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XamlBindingHelper = *mut ::core::ffi::c_void;
pub type XamlMarkupHelper = *mut ::core::ffi::c_void;
pub type XamlReader = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Markup\"`*"]
pub struct XmlnsDefinition {
    pub XmlNamespace: ::windows_sys::core::HSTRING,
    pub Namespace: ::windows_sys::core::HSTRING,
}
impl ::core::marker::Copy for XmlnsDefinition {}
impl ::core::clone::Clone for XmlnsDefinition {
    fn clone(&self) -> Self {
        *self
    }
}
