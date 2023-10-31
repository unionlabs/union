pub type PrintTicketCapabilities = *mut ::core::ffi::c_void;
pub type PrintTicketFeature = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_PrintTicket\"`*"]
#[repr(transparent)]
pub struct PrintTicketFeatureSelectionType(pub i32);
impl PrintTicketFeatureSelectionType {
    pub const PickOne: Self = Self(0i32);
    pub const PickMany: Self = Self(1i32);
}
impl ::core::marker::Copy for PrintTicketFeatureSelectionType {}
impl ::core::clone::Clone for PrintTicketFeatureSelectionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintTicketOption = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_PrintTicket\"`*"]
#[repr(transparent)]
pub struct PrintTicketParameterDataType(pub i32);
impl PrintTicketParameterDataType {
    pub const Integer: Self = Self(0i32);
    pub const NumericString: Self = Self(1i32);
    pub const String: Self = Self(2i32);
}
impl ::core::marker::Copy for PrintTicketParameterDataType {}
impl ::core::clone::Clone for PrintTicketParameterDataType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintTicketParameterDefinition = *mut ::core::ffi::c_void;
pub type PrintTicketParameterInitializer = *mut ::core::ffi::c_void;
pub type PrintTicketValue = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_PrintTicket\"`*"]
#[repr(transparent)]
pub struct PrintTicketValueType(pub i32);
impl PrintTicketValueType {
    pub const Integer: Self = Self(0i32);
    pub const String: Self = Self(1i32);
    pub const Unknown: Self = Self(2i32);
}
impl ::core::marker::Copy for PrintTicketValueType {}
impl ::core::clone::Clone for PrintTicketValueType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WorkflowPrintTicket = *mut ::core::ffi::c_void;
pub type WorkflowPrintTicketValidationResult = *mut ::core::ffi::c_void;
