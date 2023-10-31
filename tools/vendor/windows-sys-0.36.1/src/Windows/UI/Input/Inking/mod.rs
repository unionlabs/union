#[cfg(feature = "UI_Input_Inking_Analysis")]
pub mod Analysis;
#[cfg(feature = "UI_Input_Inking_Core")]
pub mod Core;
#[cfg(feature = "UI_Input_Inking_Preview")]
pub mod Preview;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct HandwritingLineHeight(pub i32);
impl HandwritingLineHeight {
    pub const Small: Self = Self(0i32);
    pub const Medium: Self = Self(1i32);
    pub const Large: Self = Self(2i32);
}
impl ::core::marker::Copy for HandwritingLineHeight {}
impl ::core::clone::Clone for HandwritingLineHeight {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IInkPointFactory = *mut ::core::ffi::c_void;
pub type IInkPresenterRulerFactory = *mut ::core::ffi::c_void;
pub type IInkPresenterStencil = *mut ::core::ffi::c_void;
pub type IInkRecognizerContainer = *mut ::core::ffi::c_void;
pub type IInkStrokeContainer = *mut ::core::ffi::c_void;
pub type InkDrawingAttributes = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkDrawingAttributesKind(pub i32);
impl InkDrawingAttributesKind {
    pub const Default: Self = Self(0i32);
    pub const Pencil: Self = Self(1i32);
}
impl ::core::marker::Copy for InkDrawingAttributesKind {}
impl ::core::clone::Clone for InkDrawingAttributesKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkDrawingAttributesPencilProperties = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkHighContrastAdjustment(pub i32);
impl InkHighContrastAdjustment {
    pub const UseSystemColorsWhenNecessary: Self = Self(0i32);
    pub const UseSystemColors: Self = Self(1i32);
    pub const UseOriginalColors: Self = Self(2i32);
}
impl ::core::marker::Copy for InkHighContrastAdjustment {}
impl ::core::clone::Clone for InkHighContrastAdjustment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkInputConfiguration = *mut ::core::ffi::c_void;
pub type InkInputProcessingConfiguration = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkInputProcessingMode(pub i32);
impl InkInputProcessingMode {
    pub const None: Self = Self(0i32);
    pub const Inking: Self = Self(1i32);
    pub const Erasing: Self = Self(2i32);
}
impl ::core::marker::Copy for InkInputProcessingMode {}
impl ::core::clone::Clone for InkInputProcessingMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkInputRightDragAction(pub i32);
impl InkInputRightDragAction {
    pub const LeaveUnprocessed: Self = Self(0i32);
    pub const AllowProcessing: Self = Self(1i32);
}
impl ::core::marker::Copy for InkInputRightDragAction {}
impl ::core::clone::Clone for InkInputRightDragAction {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkManipulationMode(pub i32);
impl InkManipulationMode {
    pub const Inking: Self = Self(0i32);
    pub const Erasing: Self = Self(1i32);
    pub const Selecting: Self = Self(2i32);
}
impl ::core::marker::Copy for InkManipulationMode {}
impl ::core::clone::Clone for InkManipulationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkModelerAttributes = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkPersistenceFormat(pub i32);
impl InkPersistenceFormat {
    pub const GifWithEmbeddedIsf: Self = Self(0i32);
    pub const Isf: Self = Self(1i32);
}
impl ::core::marker::Copy for InkPersistenceFormat {}
impl ::core::clone::Clone for InkPersistenceFormat {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkPoint = *mut ::core::ffi::c_void;
pub type InkPresenter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkPresenterPredefinedConfiguration(pub i32);
impl InkPresenterPredefinedConfiguration {
    pub const SimpleSinglePointer: Self = Self(0i32);
    pub const SimpleMultiplePointer: Self = Self(1i32);
}
impl ::core::marker::Copy for InkPresenterPredefinedConfiguration {}
impl ::core::clone::Clone for InkPresenterPredefinedConfiguration {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkPresenterProtractor = *mut ::core::ffi::c_void;
pub type InkPresenterRuler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkPresenterStencilKind(pub i32);
impl InkPresenterStencilKind {
    pub const Other: Self = Self(0i32);
    pub const Ruler: Self = Self(1i32);
    pub const Protractor: Self = Self(2i32);
}
impl ::core::marker::Copy for InkPresenterStencilKind {}
impl ::core::clone::Clone for InkPresenterStencilKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkRecognitionResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct InkRecognitionTarget(pub i32);
impl InkRecognitionTarget {
    pub const All: Self = Self(0i32);
    pub const Selected: Self = Self(1i32);
    pub const Recent: Self = Self(2i32);
}
impl ::core::marker::Copy for InkRecognitionTarget {}
impl ::core::clone::Clone for InkRecognitionTarget {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkRecognizer = *mut ::core::ffi::c_void;
pub type InkRecognizerContainer = *mut ::core::ffi::c_void;
pub type InkStroke = *mut ::core::ffi::c_void;
pub type InkStrokeBuilder = *mut ::core::ffi::c_void;
pub type InkStrokeContainer = *mut ::core::ffi::c_void;
pub type InkStrokeInput = *mut ::core::ffi::c_void;
pub type InkStrokeRenderingSegment = *mut ::core::ffi::c_void;
pub type InkStrokesCollectedEventArgs = *mut ::core::ffi::c_void;
pub type InkStrokesErasedEventArgs = *mut ::core::ffi::c_void;
pub type InkSynchronizer = *mut ::core::ffi::c_void;
pub type InkUnprocessedInput = *mut ::core::ffi::c_void;
pub type PenAndInkSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct PenHandedness(pub i32);
impl PenHandedness {
    pub const Right: Self = Self(0i32);
    pub const Left: Self = Self(1i32);
}
impl ::core::marker::Copy for PenHandedness {}
impl ::core::clone::Clone for PenHandedness {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Inking\"`*"]
#[repr(transparent)]
pub struct PenTipShape(pub i32);
impl PenTipShape {
    pub const Circle: Self = Self(0i32);
    pub const Rectangle: Self = Self(1i32);
}
impl ::core::marker::Copy for PenTipShape {}
impl ::core::clone::Clone for PenTipShape {
    fn clone(&self) -> Self {
        *self
    }
}
