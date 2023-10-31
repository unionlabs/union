pub type CoreIncrementalInkStroke = *mut ::core::ffi::c_void;
pub type CoreInkIndependentInputSource = *mut ::core::ffi::c_void;
pub type CoreInkPresenterHost = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking_Core\"`*"]
#[repr(transparent)]
pub struct CoreWetStrokeDisposition(pub i32);
impl CoreWetStrokeDisposition {
    pub const Inking: Self = Self(0i32);
    pub const Completed: Self = Self(1i32);
    pub const Canceled: Self = Self(2i32);
}
impl ::core::marker::Copy for CoreWetStrokeDisposition {}
impl ::core::clone::Clone for CoreWetStrokeDisposition {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CoreWetStrokeUpdateEventArgs = *mut ::core::ffi::c_void;
pub type CoreWetStrokeUpdateSource = *mut ::core::ffi::c_void;
