#[cfg(feature = "UI_Xaml_Media_Animation")]
pub mod Animation;
#[cfg(feature = "UI_Xaml_Media_Imaging")]
pub mod Imaging;
#[cfg(feature = "UI_Xaml_Media_Media3D")]
pub mod Media3D;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct AcrylicBackgroundSource(pub i32);
impl AcrylicBackgroundSource {
    pub const HostBackdrop: Self = Self(0i32);
    pub const Backdrop: Self = Self(1i32);
}
impl ::core::marker::Copy for AcrylicBackgroundSource {}
impl ::core::clone::Clone for AcrylicBackgroundSource {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AcrylicBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct AlignmentX(pub i32);
impl AlignmentX {
    pub const Left: Self = Self(0i32);
    pub const Center: Self = Self(1i32);
    pub const Right: Self = Self(2i32);
}
impl ::core::marker::Copy for AlignmentX {}
impl ::core::clone::Clone for AlignmentX {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct AlignmentY(pub i32);
impl AlignmentY {
    pub const Top: Self = Self(0i32);
    pub const Center: Self = Self(1i32);
    pub const Bottom: Self = Self(2i32);
}
impl ::core::marker::Copy for AlignmentY {}
impl ::core::clone::Clone for AlignmentY {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ArcSegment = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct AudioCategory(pub i32);
impl AudioCategory {
    pub const Other: Self = Self(0i32);
    pub const ForegroundOnlyMedia: Self = Self(1i32);
    pub const BackgroundCapableMedia: Self = Self(2i32);
    pub const Communications: Self = Self(3i32);
    pub const Alerts: Self = Self(4i32);
    pub const SoundEffects: Self = Self(5i32);
    pub const GameEffects: Self = Self(6i32);
    pub const GameMedia: Self = Self(7i32);
    pub const GameChat: Self = Self(8i32);
    pub const Speech: Self = Self(9i32);
    pub const Movie: Self = Self(10i32);
    pub const Media: Self = Self(11i32);
}
impl ::core::marker::Copy for AudioCategory {}
impl ::core::clone::Clone for AudioCategory {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct AudioDeviceType(pub i32);
impl AudioDeviceType {
    pub const Console: Self = Self(0i32);
    pub const Multimedia: Self = Self(1i32);
    pub const Communications: Self = Self(2i32);
}
impl ::core::marker::Copy for AudioDeviceType {}
impl ::core::clone::Clone for AudioDeviceType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BezierSegment = *mut ::core::ffi::c_void;
pub type BitmapCache = *mut ::core::ffi::c_void;
pub type Brush = *mut ::core::ffi::c_void;
pub type BrushCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct BrushMappingMode(pub i32);
impl BrushMappingMode {
    pub const Absolute: Self = Self(0i32);
    pub const RelativeToBoundingBox: Self = Self(1i32);
}
impl ::core::marker::Copy for BrushMappingMode {}
impl ::core::clone::Clone for BrushMappingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CacheMode = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct ColorInterpolationMode(pub i32);
impl ColorInterpolationMode {
    pub const ScRgbLinearInterpolation: Self = Self(0i32);
    pub const SRgbLinearInterpolation: Self = Self(1i32);
}
impl ::core::marker::Copy for ColorInterpolationMode {}
impl ::core::clone::Clone for ColorInterpolationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompositeTransform = *mut ::core::ffi::c_void;
pub type CompositionTarget = *mut ::core::ffi::c_void;
pub type DoubleCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct ElementCompositeMode(pub i32);
impl ElementCompositeMode {
    pub const Inherit: Self = Self(0i32);
    pub const SourceOver: Self = Self(1i32);
    pub const MinBlend: Self = Self(2i32);
}
impl ::core::marker::Copy for ElementCompositeMode {}
impl ::core::clone::Clone for ElementCompositeMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type EllipseGeometry = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct FastPlayFallbackBehaviour(pub i32);
impl FastPlayFallbackBehaviour {
    pub const Skip: Self = Self(0i32);
    pub const Hide: Self = Self(1i32);
    pub const Disable: Self = Self(2i32);
}
impl ::core::marker::Copy for FastPlayFallbackBehaviour {}
impl ::core::clone::Clone for FastPlayFallbackBehaviour {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct FillRule(pub i32);
impl FillRule {
    pub const EvenOdd: Self = Self(0i32);
    pub const Nonzero: Self = Self(1i32);
}
impl ::core::marker::Copy for FillRule {}
impl ::core::clone::Clone for FillRule {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FontFamily = *mut ::core::ffi::c_void;
pub type GeneralTransform = *mut ::core::ffi::c_void;
pub type Geometry = *mut ::core::ffi::c_void;
pub type GeometryCollection = *mut ::core::ffi::c_void;
pub type GeometryGroup = *mut ::core::ffi::c_void;
pub type GradientBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct GradientSpreadMethod(pub i32);
impl GradientSpreadMethod {
    pub const Pad: Self = Self(0i32);
    pub const Reflect: Self = Self(1i32);
    pub const Repeat: Self = Self(2i32);
}
impl ::core::marker::Copy for GradientSpreadMethod {}
impl ::core::clone::Clone for GradientSpreadMethod {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GradientStop = *mut ::core::ffi::c_void;
pub type GradientStopCollection = *mut ::core::ffi::c_void;
pub type ImageBrush = *mut ::core::ffi::c_void;
pub type ImageSource = *mut ::core::ffi::c_void;
pub type LineGeometry = *mut ::core::ffi::c_void;
pub type LineSegment = *mut ::core::ffi::c_void;
pub type LinearGradientBrush = *mut ::core::ffi::c_void;
pub type LoadedImageSourceLoadCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct LoadedImageSourceLoadStatus(pub i32);
impl LoadedImageSourceLoadStatus {
    pub const Success: Self = Self(0i32);
    pub const NetworkError: Self = Self(1i32);
    pub const InvalidFormat: Self = Self(2i32);
    pub const Other: Self = Self(3i32);
}
impl ::core::marker::Copy for LoadedImageSourceLoadStatus {}
impl ::core::clone::Clone for LoadedImageSourceLoadStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LoadedImageSurface = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
pub struct Matrix {
    pub M11: f64,
    pub M12: f64,
    pub M21: f64,
    pub M22: f64,
    pub OffsetX: f64,
    pub OffsetY: f64,
}
impl ::core::marker::Copy for Matrix {}
impl ::core::clone::Clone for Matrix {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Matrix3DProjection = *mut ::core::ffi::c_void;
pub type MatrixHelper = *mut ::core::ffi::c_void;
pub type MatrixTransform = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct MediaCanPlayResponse(pub i32);
impl MediaCanPlayResponse {
    pub const NotSupported: Self = Self(0i32);
    pub const Maybe: Self = Self(1i32);
    pub const Probably: Self = Self(2i32);
}
impl ::core::marker::Copy for MediaCanPlayResponse {}
impl ::core::clone::Clone for MediaCanPlayResponse {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct MediaElementState(pub i32);
impl MediaElementState {
    pub const Closed: Self = Self(0i32);
    pub const Opening: Self = Self(1i32);
    pub const Buffering: Self = Self(2i32);
    pub const Playing: Self = Self(3i32);
    pub const Paused: Self = Self(4i32);
    pub const Stopped: Self = Self(5i32);
}
impl ::core::marker::Copy for MediaElementState {}
impl ::core::clone::Clone for MediaElementState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaTransportControlsThumbnailRequestedEventArgs = *mut ::core::ffi::c_void;
pub type PartialMediaFailureDetectedEventArgs = *mut ::core::ffi::c_void;
pub type PathFigure = *mut ::core::ffi::c_void;
pub type PathFigureCollection = *mut ::core::ffi::c_void;
pub type PathGeometry = *mut ::core::ffi::c_void;
pub type PathSegment = *mut ::core::ffi::c_void;
pub type PathSegmentCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct PenLineCap(pub i32);
impl PenLineCap {
    pub const Flat: Self = Self(0i32);
    pub const Square: Self = Self(1i32);
    pub const Round: Self = Self(2i32);
    pub const Triangle: Self = Self(3i32);
}
impl ::core::marker::Copy for PenLineCap {}
impl ::core::clone::Clone for PenLineCap {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct PenLineJoin(pub i32);
impl PenLineJoin {
    pub const Miter: Self = Self(0i32);
    pub const Bevel: Self = Self(1i32);
    pub const Round: Self = Self(2i32);
}
impl ::core::marker::Copy for PenLineJoin {}
impl ::core::clone::Clone for PenLineJoin {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PlaneProjection = *mut ::core::ffi::c_void;
pub type PointCollection = *mut ::core::ffi::c_void;
pub type PolyBezierSegment = *mut ::core::ffi::c_void;
pub type PolyLineSegment = *mut ::core::ffi::c_void;
pub type PolyQuadraticBezierSegment = *mut ::core::ffi::c_void;
pub type Projection = *mut ::core::ffi::c_void;
pub type QuadraticBezierSegment = *mut ::core::ffi::c_void;
pub type RateChangedRoutedEventArgs = *mut ::core::ffi::c_void;
pub type RateChangedRoutedEventHandler = *mut ::core::ffi::c_void;
pub type RectangleGeometry = *mut ::core::ffi::c_void;
pub type RenderedEventArgs = *mut ::core::ffi::c_void;
pub type RenderingEventArgs = *mut ::core::ffi::c_void;
pub type RevealBackgroundBrush = *mut ::core::ffi::c_void;
pub type RevealBorderBrush = *mut ::core::ffi::c_void;
pub type RevealBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct RevealBrushState(pub i32);
impl RevealBrushState {
    pub const Normal: Self = Self(0i32);
    pub const PointerOver: Self = Self(1i32);
    pub const Pressed: Self = Self(2i32);
}
impl ::core::marker::Copy for RevealBrushState {}
impl ::core::clone::Clone for RevealBrushState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RotateTransform = *mut ::core::ffi::c_void;
pub type ScaleTransform = *mut ::core::ffi::c_void;
pub type Shadow = *mut ::core::ffi::c_void;
pub type SkewTransform = *mut ::core::ffi::c_void;
pub type SolidColorBrush = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct Stereo3DVideoPackingMode(pub i32);
impl Stereo3DVideoPackingMode {
    pub const None: Self = Self(0i32);
    pub const SideBySide: Self = Self(1i32);
    pub const TopBottom: Self = Self(2i32);
}
impl ::core::marker::Copy for Stereo3DVideoPackingMode {}
impl ::core::clone::Clone for Stereo3DVideoPackingMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct Stereo3DVideoRenderMode(pub i32);
impl Stereo3DVideoRenderMode {
    pub const Mono: Self = Self(0i32);
    pub const Stereo: Self = Self(1i32);
}
impl ::core::marker::Copy for Stereo3DVideoRenderMode {}
impl ::core::clone::Clone for Stereo3DVideoRenderMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct Stretch(pub i32);
impl Stretch {
    pub const None: Self = Self(0i32);
    pub const Fill: Self = Self(1i32);
    pub const Uniform: Self = Self(2i32);
    pub const UniformToFill: Self = Self(3i32);
}
impl ::core::marker::Copy for Stretch {}
impl ::core::clone::Clone for Stretch {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct StyleSimulations(pub i32);
impl StyleSimulations {
    pub const None: Self = Self(0i32);
    pub const BoldSimulation: Self = Self(1i32);
    pub const ItalicSimulation: Self = Self(2i32);
    pub const BoldItalicSimulation: Self = Self(3i32);
}
impl ::core::marker::Copy for StyleSimulations {}
impl ::core::clone::Clone for StyleSimulations {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Media\"`*"]
#[repr(transparent)]
pub struct SweepDirection(pub i32);
impl SweepDirection {
    pub const Counterclockwise: Self = Self(0i32);
    pub const Clockwise: Self = Self(1i32);
}
impl ::core::marker::Copy for SweepDirection {}
impl ::core::clone::Clone for SweepDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ThemeShadow = *mut ::core::ffi::c_void;
pub type TileBrush = *mut ::core::ffi::c_void;
pub type TimelineMarker = *mut ::core::ffi::c_void;
pub type TimelineMarkerCollection = *mut ::core::ffi::c_void;
pub type TimelineMarkerRoutedEventArgs = *mut ::core::ffi::c_void;
pub type TimelineMarkerRoutedEventHandler = *mut ::core::ffi::c_void;
pub type Transform = *mut ::core::ffi::c_void;
pub type TransformCollection = *mut ::core::ffi::c_void;
pub type TransformGroup = *mut ::core::ffi::c_void;
pub type TranslateTransform = *mut ::core::ffi::c_void;
pub type VisualTreeHelper = *mut ::core::ffi::c_void;
pub type XamlCompositionBrushBase = *mut ::core::ffi::c_void;
pub type XamlLight = *mut ::core::ffi::c_void;
