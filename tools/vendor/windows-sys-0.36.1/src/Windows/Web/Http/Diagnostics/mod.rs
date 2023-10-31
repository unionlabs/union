pub type HttpDiagnosticProvider = *mut ::core::ffi::c_void;
pub type HttpDiagnosticProviderRequestResponseCompletedEventArgs = *mut ::core::ffi::c_void;
pub type HttpDiagnosticProviderRequestResponseTimestamps = *mut ::core::ffi::c_void;
pub type HttpDiagnosticProviderRequestSentEventArgs = *mut ::core::ffi::c_void;
pub type HttpDiagnosticProviderResponseReceivedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Web_Http_Diagnostics\"`*"]
#[repr(transparent)]
pub struct HttpDiagnosticRequestInitiator(pub i32);
impl HttpDiagnosticRequestInitiator {
    pub const ParsedElement: Self = Self(0i32);
    pub const Script: Self = Self(1i32);
    pub const Image: Self = Self(2i32);
    pub const Link: Self = Self(3i32);
    pub const Style: Self = Self(4i32);
    pub const XmlHttpRequest: Self = Self(5i32);
    pub const Media: Self = Self(6i32);
    pub const HtmlDownload: Self = Self(7i32);
    pub const Prefetch: Self = Self(8i32);
    pub const Other: Self = Self(9i32);
    pub const CrossOriginPreFlight: Self = Self(10i32);
    pub const Fetch: Self = Self(11i32);
    pub const Beacon: Self = Self(12i32);
}
impl ::core::marker::Copy for HttpDiagnosticRequestInitiator {}
impl ::core::clone::Clone for HttpDiagnosticRequestInitiator {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HttpDiagnosticSourceLocation = *mut ::core::ffi::c_void;
