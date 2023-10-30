#[cfg(feature = "UI_Composition_Core")]
pub mod Core;
#[cfg(feature = "UI_Composition_Desktop")]
pub mod Desktop;
#[cfg(feature = "UI_Composition_Diagnostics")]
pub mod Diagnostics;
#[cfg(feature = "UI_Composition_Effects")]
pub mod Effects;
#[cfg(feature = "UI_Composition_Interactions")]
pub mod Interactions;
#[cfg(feature = "UI_Composition_Scenes")]
pub mod Scenes;
pub type AmbientLight = *mut ::core::ffi::c_void;
pub type AnimationController = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct AnimationControllerProgressBehavior(pub i32);
impl AnimationControllerProgressBehavior {
    pub const Default: Self = Self(0i32);
    pub const IncludesDelayTime: Self = Self(1i32);
}
impl ::core::marker::Copy for AnimationControllerProgressBehavior {}
impl ::core::clone::Clone for AnimationControllerProgressBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct AnimationDelayBehavior(pub i32);
impl AnimationDelayBehavior {
    pub const SetInitialValueAfterDelay: Self = Self(0i32);
    pub const SetInitialValueBeforeDelay: Self = Self(1i32);
}
impl ::core::marker::Copy for AnimationDelayBehavior {}
impl ::core::clone::Clone for AnimationDelayBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct AnimationDirection(pub i32);
impl AnimationDirection {
    pub const Normal: Self = Self(0i32);
    pub const Reverse: Self = Self(1i32);
    pub const Alternate: Self = Self(2i32);
    pub const AlternateReverse: Self = Self(3i32);
}
impl ::core::marker::Copy for AnimationDirection {}
impl ::core::clone::Clone for AnimationDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct AnimationIterationBehavior(pub i32);
impl AnimationIterationBehavior {
    pub const Count: Self = Self(0i32);
    pub const Forever: Self = Self(1i32);
}
impl ::core::marker::Copy for AnimationIterationBehavior {}
impl ::core::clone::Clone for AnimationIterationBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct AnimationPropertyAccessMode(pub i32);
impl AnimationPropertyAccessMode {
    pub const None: Self = Self(0i32);
    pub const ReadOnly: Self = Self(1i32);
    pub const WriteOnly: Self = Self(2i32);
    pub const ReadWrite: Self = Self(3i32);
}
impl ::core::marker::Copy for AnimationPropertyAccessMode {}
impl ::core::clone::Clone for AnimationPropertyAccessMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AnimationPropertyInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct AnimationStopBehavior(pub i32);
impl AnimationStopBehavior {
    pub const LeaveCurrentValue: Self = Self(0i32);
    pub const SetToInitialValue: Self = Self(1i32);
    pub const SetToFinalValue: Self = Self(2i32);
}
impl ::core::marker::Copy for AnimationStopBehavior {}
impl ::core::clone::Clone for AnimationStopBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BackEasingFunction = *mut ::core::ffi::c_void;
pub type BooleanKeyFrameAnimation = *mut ::core::ffi::c_void;
pub type BounceEasingFunction = *mut ::core::ffi::c_void;
pub type BounceScalarNaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type BounceVector2NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type BounceVector3NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type CircleEasingFunction = *mut ::core::ffi::c_void;
pub type ColorKeyFrameAnimation = *mut ::core::ffi::c_void;
pub type CompositionAnimation = *mut ::core::ffi::c_void;
pub type CompositionAnimationGroup = *mut ::core::ffi::c_void;
pub type CompositionBackdropBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionBackfaceVisibility(pub i32);
impl CompositionBackfaceVisibility {
    pub const Inherit: Self = Self(0i32);
    pub const Visible: Self = Self(1i32);
    pub const Hidden: Self = Self(2i32);
}
impl ::core::marker::Copy for CompositionBackfaceVisibility {}
impl ::core::clone::Clone for CompositionBackfaceVisibility {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionBatchCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionBatchTypes(pub u32);
impl CompositionBatchTypes {
    pub const None: Self = Self(0u32);
    pub const Animation: Self = Self(1u32);
    pub const Effect: Self = Self(2u32);
    pub const InfiniteAnimation: Self = Self(4u32);
    pub const AllAnimations: Self = Self(5u32);
}
impl ::core::marker::Copy for CompositionBatchTypes {}
impl ::core::clone::Clone for CompositionBatchTypes {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionBitmapInterpolationMode(pub i32);
impl CompositionBitmapInterpolationMode {
    pub const NearestNeighbor: Self = Self(0i32);
    pub const Linear: Self = Self(1i32);
    pub const MagLinearMinLinearMipLinear: Self = Self(2i32);
    pub const MagLinearMinLinearMipNearest: Self = Self(3i32);
    pub const MagLinearMinNearestMipLinear: Self = Self(4i32);
    pub const MagLinearMinNearestMipNearest: Self = Self(5i32);
    pub const MagNearestMinLinearMipLinear: Self = Self(6i32);
    pub const MagNearestMinLinearMipNearest: Self = Self(7i32);
    pub const MagNearestMinNearestMipLinear: Self = Self(8i32);
    pub const MagNearestMinNearestMipNearest: Self = Self(9i32);
}
impl ::core::marker::Copy for CompositionBitmapInterpolationMode {}
impl ::core::clone::Clone for CompositionBitmapInterpolationMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionBorderMode(pub i32);
impl CompositionBorderMode {
    pub const Inherit: Self = Self(0i32);
    pub const Soft: Self = Self(1i32);
    pub const Hard: Self = Self(2i32);
}
impl ::core::marker::Copy for CompositionBorderMode {}
impl ::core::clone::Clone for CompositionBorderMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionBrush = *mut ::core::ffi::c_void;
pub type CompositionCapabilities = *mut ::core::ffi::c_void;
pub type CompositionClip = *mut ::core::ffi::c_void;
pub type CompositionColorBrush = *mut ::core::ffi::c_void;
pub type CompositionColorGradientStop = *mut ::core::ffi::c_void;
pub type CompositionColorGradientStopCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionColorSpace(pub i32);
impl CompositionColorSpace {
    pub const Auto: Self = Self(0i32);
    pub const Hsl: Self = Self(1i32);
    pub const Rgb: Self = Self(2i32);
    pub const HslLinear: Self = Self(3i32);
    pub const RgbLinear: Self = Self(4i32);
}
impl ::core::marker::Copy for CompositionColorSpace {}
impl ::core::clone::Clone for CompositionColorSpace {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionCommitBatch = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionCompositeMode(pub i32);
impl CompositionCompositeMode {
    pub const Inherit: Self = Self(0i32);
    pub const SourceOver: Self = Self(1i32);
    pub const DestinationInvert: Self = Self(2i32);
    pub const MinBlend: Self = Self(3i32);
}
impl ::core::marker::Copy for CompositionCompositeMode {}
impl ::core::clone::Clone for CompositionCompositeMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionContainerShape = *mut ::core::ffi::c_void;
pub type CompositionDrawingSurface = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionDropShadowSourcePolicy(pub i32);
impl CompositionDropShadowSourcePolicy {
    pub const Default: Self = Self(0i32);
    pub const InheritFromVisualContent: Self = Self(1i32);
}
impl ::core::marker::Copy for CompositionDropShadowSourcePolicy {}
impl ::core::clone::Clone for CompositionDropShadowSourcePolicy {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionEasingFunction = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionEasingFunctionMode(pub i32);
impl CompositionEasingFunctionMode {
    pub const In: Self = Self(0i32);
    pub const Out: Self = Self(1i32);
    pub const InOut: Self = Self(2i32);
}
impl ::core::marker::Copy for CompositionEasingFunctionMode {}
impl ::core::clone::Clone for CompositionEasingFunctionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionEffectBrush = *mut ::core::ffi::c_void;
pub type CompositionEffectFactory = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionEffectFactoryLoadStatus(pub i32);
impl CompositionEffectFactoryLoadStatus {
    pub const Success: Self = Self(0i32);
    pub const EffectTooComplex: Self = Self(1i32);
    pub const Pending: Self = Self(2i32);
    pub const Other: Self = Self(-1i32);
}
impl ::core::marker::Copy for CompositionEffectFactoryLoadStatus {}
impl ::core::clone::Clone for CompositionEffectFactoryLoadStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionEffectSourceParameter = *mut ::core::ffi::c_void;
pub type CompositionEllipseGeometry = *mut ::core::ffi::c_void;
pub type CompositionGeometricClip = *mut ::core::ffi::c_void;
pub type CompositionGeometry = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionGetValueStatus(pub i32);
impl CompositionGetValueStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const TypeMismatch: Self = Self(1i32);
    pub const NotFound: Self = Self(2i32);
}
impl ::core::marker::Copy for CompositionGetValueStatus {}
impl ::core::clone::Clone for CompositionGetValueStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionGradientBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionGradientExtendMode(pub i32);
impl CompositionGradientExtendMode {
    pub const Clamp: Self = Self(0i32);
    pub const Wrap: Self = Self(1i32);
    pub const Mirror: Self = Self(2i32);
}
impl ::core::marker::Copy for CompositionGradientExtendMode {}
impl ::core::clone::Clone for CompositionGradientExtendMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionGraphicsDevice = *mut ::core::ffi::c_void;
pub type CompositionLight = *mut ::core::ffi::c_void;
pub type CompositionLineGeometry = *mut ::core::ffi::c_void;
pub type CompositionLinearGradientBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionMappingMode(pub i32);
impl CompositionMappingMode {
    pub const Absolute: Self = Self(0i32);
    pub const Relative: Self = Self(1i32);
}
impl ::core::marker::Copy for CompositionMappingMode {}
impl ::core::clone::Clone for CompositionMappingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionMaskBrush = *mut ::core::ffi::c_void;
pub type CompositionMipmapSurface = *mut ::core::ffi::c_void;
pub type CompositionNineGridBrush = *mut ::core::ffi::c_void;
pub type CompositionObject = *mut ::core::ffi::c_void;
pub type CompositionPath = *mut ::core::ffi::c_void;
pub type CompositionPathGeometry = *mut ::core::ffi::c_void;
pub type CompositionProjectedShadow = *mut ::core::ffi::c_void;
pub type CompositionProjectedShadowCaster = *mut ::core::ffi::c_void;
pub type CompositionProjectedShadowCasterCollection = *mut ::core::ffi::c_void;
pub type CompositionProjectedShadowReceiver = *mut ::core::ffi::c_void;
pub type CompositionProjectedShadowReceiverUnorderedCollection = *mut ::core::ffi::c_void;
pub type CompositionPropertySet = *mut ::core::ffi::c_void;
pub type CompositionRadialGradientBrush = *mut ::core::ffi::c_void;
pub type CompositionRectangleGeometry = *mut ::core::ffi::c_void;
pub type CompositionRoundedRectangleGeometry = *mut ::core::ffi::c_void;
pub type CompositionScopedBatch = *mut ::core::ffi::c_void;
pub type CompositionShadow = *mut ::core::ffi::c_void;
pub type CompositionShape = *mut ::core::ffi::c_void;
pub type CompositionShapeCollection = *mut ::core::ffi::c_void;
pub type CompositionSpriteShape = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionStretch(pub i32);
impl CompositionStretch {
    pub const None: Self = Self(0i32);
    pub const Fill: Self = Self(1i32);
    pub const Uniform: Self = Self(2i32);
    pub const UniformToFill: Self = Self(3i32);
}
impl ::core::marker::Copy for CompositionStretch {}
impl ::core::clone::Clone for CompositionStretch {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionStrokeCap(pub i32);
impl CompositionStrokeCap {
    pub const Flat: Self = Self(0i32);
    pub const Square: Self = Self(1i32);
    pub const Round: Self = Self(2i32);
    pub const Triangle: Self = Self(3i32);
}
impl ::core::marker::Copy for CompositionStrokeCap {}
impl ::core::clone::Clone for CompositionStrokeCap {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionStrokeDashArray = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition\"`*"]
#[repr(transparent)]
pub struct CompositionStrokeLineJoin(pub i32);
impl CompositionStrokeLineJoin {
    pub const Miter: Self = Self(0i32);
    pub const Bevel: Self = Self(1i32);
    pub const Round: Self = Self(2i32);
    pub const MiterOrBevel: Self = Self(3i32);
}
impl ::core::marker::Copy for CompositionStrokeLineJoin {}
impl ::core::clone::Clone for CompositionStrokeLineJoin {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositionSurfaceBrush = *mut ::core::ffi::c_void;
pub type CompositionTarget = *mut ::core::ffi::c_void;
pub type CompositionTransform = *mut ::core::ffi::c_void;
pub type CompositionViewBox = *mut ::core::ffi::c_void;
pub type CompositionVirtualDrawingSurface = *mut ::core::ffi::c_void;
pub type CompositionVisualSurface = *mut ::core::ffi::c_void;
pub type Compositor = *mut ::core::ffi::c_void;
pub type ContainerVisual = *mut ::core::ffi::c_void;
pub type CubicBezierEasingFunction = *mut ::core::ffi::c_void;
pub type DelegatedInkTrailVisual = *mut ::core::ffi::c_void;
pub type DistantLight = *mut ::core::ffi::c_void;
pub type DropShadow = *mut ::core::ffi::c_void;
pub type ElasticEasingFunction = *mut ::core::ffi::c_void;
pub type ExponentialEasingFunction = *mut ::core::ffi::c_void;
pub type ExpressionAnimation = *mut ::core::ffi::c_void;
pub type IAnimationObject = *mut ::core::ffi::c_void;
pub type ICompositionAnimationBase = *mut ::core::ffi::c_void;
pub type ICompositionSupportsSystemBackdrop = *mut ::core::ffi::c_void;
pub type ICompositionSurface = *mut ::core::ffi::c_void;
pub type ICompositionSurfaceFacade = *mut ::core::ffi::c_void;
pub type IVisualElement = *mut ::core::ffi::c_void;
pub type IVisualElement2 = *mut ::core::ffi::c_void;
pub type ImplicitAnimationCollection = *mut ::core::ffi::c_void;
pub type InitialValueExpressionCollection = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Composition\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct InkTrailPoint {
    pub Point: super::super::Foundation::Point,
    pub Radius: f32,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for InkTrailPoint {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for InkTrailPoint {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InsetClip = *mut ::core::ffi::c_void;
pub type KeyFrameAnimation = *mut ::core::ffi::c_void;
pub type LayerVisual = *mut ::core::ffi::c_void;
pub type LinearEasingFunction = *mut ::core::ffi::c_void;
pub type NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type PathKeyFrameAnimation = *mut ::core::ffi::c_void;
pub type PointLight = *mut ::core::ffi::c_void;
pub type PowerEasingFunction = *mut ::core::ffi::c_void;
pub type QuaternionKeyFrameAnimation = *mut ::core::ffi::c_void;
pub type RectangleClip = *mut ::core::ffi::c_void;
pub type RedirectVisual = *mut ::core::ffi::c_void;
pub type RenderingDeviceReplacedEventArgs = *mut ::core::ffi::c_void;
pub type ScalarKeyFrameAnimation = *mut ::core::ffi::c_void;
pub type ScalarNaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type ShapeVisual = *mut ::core::ffi::c_void;
pub type SineEasingFunction = *mut ::core::ffi::c_void;
pub type SpotLight = *mut ::core::ffi::c_void;
pub type SpringScalarNaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type SpringVector2NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type SpringVector3NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type SpriteVisual = *mut ::core::ffi::c_void;
pub type StepEasingFunction = *mut ::core::ffi::c_void;
pub type Vector2KeyFrameAnimation = *mut ::core::ffi::c_void;
pub type Vector2NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type Vector3KeyFrameAnimation = *mut ::core::ffi::c_void;
pub type Vector3NaturalMotionAnimation = *mut ::core::ffi::c_void;
pub type Vector4KeyFrameAnimation = *mut ::core::ffi::c_void;
pub type Visual = *mut ::core::ffi::c_void;
pub type VisualCollection = *mut ::core::ffi::c_void;
pub type VisualUnorderedCollection = *mut ::core::ffi::c_void;
