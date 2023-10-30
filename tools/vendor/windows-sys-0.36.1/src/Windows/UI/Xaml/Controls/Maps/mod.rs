pub type CustomMapTileDataSource = *mut ::core::ffi::c_void;
pub type HttpMapTileDataSource = *mut ::core::ffi::c_void;
pub type LocalMapTileDataSource = *mut ::core::ffi::c_void;
pub type MapActualCameraChangedEventArgs = *mut ::core::ffi::c_void;
pub type MapActualCameraChangingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapAnimationKind(pub i32);
impl MapAnimationKind {
    pub const Default: Self = Self(0i32);
    pub const None: Self = Self(1i32);
    pub const Linear: Self = Self(2i32);
    pub const Bow: Self = Self(3i32);
}
impl ::core::marker::Copy for MapAnimationKind {}
impl ::core::clone::Clone for MapAnimationKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapBillboard = *mut ::core::ffi::c_void;
pub type MapCamera = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapCameraChangeReason(pub i32);
impl MapCameraChangeReason {
    pub const System: Self = Self(0i32);
    pub const UserInteraction: Self = Self(1i32);
    pub const Programmatic: Self = Self(2i32);
}
impl ::core::marker::Copy for MapCameraChangeReason {}
impl ::core::clone::Clone for MapCameraChangeReason {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapColorScheme(pub i32);
impl MapColorScheme {
    pub const Light: Self = Self(0i32);
    pub const Dark: Self = Self(1i32);
}
impl ::core::marker::Copy for MapColorScheme {}
impl ::core::clone::Clone for MapColorScheme {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapContextRequestedEventArgs = *mut ::core::ffi::c_void;
pub type MapControl = *mut ::core::ffi::c_void;
pub type MapControlBusinessLandmarkClickEventArgs = *mut ::core::ffi::c_void;
pub type MapControlBusinessLandmarkPointerEnteredEventArgs = *mut ::core::ffi::c_void;
pub type MapControlBusinessLandmarkPointerExitedEventArgs = *mut ::core::ffi::c_void;
pub type MapControlBusinessLandmarkRightTappedEventArgs = *mut ::core::ffi::c_void;
pub type MapControlDataHelper = *mut ::core::ffi::c_void;
pub type MapControlTransitFeatureClickEventArgs = *mut ::core::ffi::c_void;
pub type MapControlTransitFeaturePointerEnteredEventArgs = *mut ::core::ffi::c_void;
pub type MapControlTransitFeaturePointerExitedEventArgs = *mut ::core::ffi::c_void;
pub type MapControlTransitFeatureRightTappedEventArgs = *mut ::core::ffi::c_void;
pub type MapCustomExperience = *mut ::core::ffi::c_void;
pub type MapCustomExperienceChangedEventArgs = *mut ::core::ffi::c_void;
pub type MapElement = *mut ::core::ffi::c_void;
pub type MapElement3D = *mut ::core::ffi::c_void;
pub type MapElementClickEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapElementCollisionBehavior(pub i32);
impl MapElementCollisionBehavior {
    pub const Hide: Self = Self(0i32);
    pub const RemainVisible: Self = Self(1i32);
}
impl ::core::marker::Copy for MapElementCollisionBehavior {}
impl ::core::clone::Clone for MapElementCollisionBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapElementPointerEnteredEventArgs = *mut ::core::ffi::c_void;
pub type MapElementPointerExitedEventArgs = *mut ::core::ffi::c_void;
pub type MapElementsLayer = *mut ::core::ffi::c_void;
pub type MapElementsLayerClickEventArgs = *mut ::core::ffi::c_void;
pub type MapElementsLayerContextRequestedEventArgs = *mut ::core::ffi::c_void;
pub type MapElementsLayerPointerEnteredEventArgs = *mut ::core::ffi::c_void;
pub type MapElementsLayerPointerExitedEventArgs = *mut ::core::ffi::c_void;
pub type MapIcon = *mut ::core::ffi::c_void;
pub type MapInputEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapInteractionMode(pub i32);
impl MapInteractionMode {
    pub const Auto: Self = Self(0i32);
    pub const Disabled: Self = Self(1i32);
    pub const GestureOnly: Self = Self(2i32);
    pub const PointerAndKeyboard: Self = Self(2i32);
    pub const ControlOnly: Self = Self(3i32);
    pub const GestureAndControl: Self = Self(4i32);
    pub const PointerKeyboardAndControl: Self = Self(4i32);
    pub const PointerOnly: Self = Self(5i32);
}
impl ::core::marker::Copy for MapInteractionMode {}
impl ::core::clone::Clone for MapInteractionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapItemsControl = *mut ::core::ffi::c_void;
pub type MapLayer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapLoadingStatus(pub i32);
impl MapLoadingStatus {
    pub const Loading: Self = Self(0i32);
    pub const Loaded: Self = Self(1i32);
    pub const DataUnavailable: Self = Self(2i32);
    pub const DownloadedMapsManagerUnavailable: Self = Self(3i32);
}
impl ::core::marker::Copy for MapLoadingStatus {}
impl ::core::clone::Clone for MapLoadingStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapModel3D = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapModel3DShadingOption(pub i32);
impl MapModel3DShadingOption {
    pub const Default: Self = Self(0i32);
    pub const Flat: Self = Self(1i32);
    pub const Smooth: Self = Self(2i32);
}
impl ::core::marker::Copy for MapModel3DShadingOption {}
impl ::core::clone::Clone for MapModel3DShadingOption {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapPanInteractionMode(pub i32);
impl MapPanInteractionMode {
    pub const Auto: Self = Self(0i32);
    pub const Disabled: Self = Self(1i32);
}
impl ::core::marker::Copy for MapPanInteractionMode {}
impl ::core::clone::Clone for MapPanInteractionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapPolygon = *mut ::core::ffi::c_void;
pub type MapPolyline = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapProjection(pub i32);
impl MapProjection {
    pub const WebMercator: Self = Self(0i32);
    pub const Globe: Self = Self(1i32);
}
impl ::core::marker::Copy for MapProjection {}
impl ::core::clone::Clone for MapProjection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapRightTappedEventArgs = *mut ::core::ffi::c_void;
pub type MapRouteView = *mut ::core::ffi::c_void;
pub type MapScene = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapStyle(pub i32);
impl MapStyle {
    pub const None: Self = Self(0i32);
    pub const Road: Self = Self(1i32);
    pub const Aerial: Self = Self(2i32);
    pub const AerialWithRoads: Self = Self(3i32);
    pub const Terrain: Self = Self(4i32);
    pub const Aerial3D: Self = Self(5i32);
    pub const Aerial3DWithRoads: Self = Self(6i32);
    pub const Custom: Self = Self(7i32);
}
impl ::core::marker::Copy for MapStyle {}
impl ::core::clone::Clone for MapStyle {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapStyleSheet = *mut ::core::ffi::c_void;
pub type MapTargetCameraChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapTileAnimationState(pub i32);
impl MapTileAnimationState {
    pub const Stopped: Self = Self(0i32);
    pub const Paused: Self = Self(1i32);
    pub const Playing: Self = Self(2i32);
}
impl ::core::marker::Copy for MapTileAnimationState {}
impl ::core::clone::Clone for MapTileAnimationState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapTileBitmapRequest = *mut ::core::ffi::c_void;
pub type MapTileBitmapRequestDeferral = *mut ::core::ffi::c_void;
pub type MapTileBitmapRequestedEventArgs = *mut ::core::ffi::c_void;
pub type MapTileDataSource = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapTileLayer(pub i32);
impl MapTileLayer {
    pub const LabelOverlay: Self = Self(0i32);
    pub const RoadOverlay: Self = Self(1i32);
    pub const AreaOverlay: Self = Self(2i32);
    pub const BackgroundOverlay: Self = Self(3i32);
    pub const BackgroundReplacement: Self = Self(4i32);
}
impl ::core::marker::Copy for MapTileLayer {}
impl ::core::clone::Clone for MapTileLayer {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MapTileSource = *mut ::core::ffi::c_void;
pub type MapTileUriRequest = *mut ::core::ffi::c_void;
pub type MapTileUriRequestDeferral = *mut ::core::ffi::c_void;
pub type MapTileUriRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapVisibleRegionKind(pub i32);
impl MapVisibleRegionKind {
    pub const Near: Self = Self(0i32);
    pub const Full: Self = Self(1i32);
}
impl ::core::marker::Copy for MapVisibleRegionKind {}
impl ::core::clone::Clone for MapVisibleRegionKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
#[repr(transparent)]
pub struct MapWatermarkMode(pub i32);
impl MapWatermarkMode {
    pub const Automatic: Self = Self(0i32);
    pub const On: Self = Self(1i32);
}
impl ::core::marker::Copy for MapWatermarkMode {}
impl ::core::clone::Clone for MapWatermarkMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Controls_Maps\"`*"]
pub struct MapZoomLevelRange {
    pub Min: f64,
    pub Max: f64,
}
impl ::core::marker::Copy for MapZoomLevelRange {}
impl ::core::clone::Clone for MapZoomLevelRange {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StreetsideExperience = *mut ::core::ffi::c_void;
pub type StreetsidePanorama = *mut ::core::ffi::c_void;
