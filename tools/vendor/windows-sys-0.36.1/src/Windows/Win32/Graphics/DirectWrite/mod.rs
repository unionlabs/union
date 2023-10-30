#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
    pub fn DWriteCreateFactory(factorytype: DWRITE_FACTORY_TYPE, iid: *const ::windows_sys::core::GUID, factory: *mut ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_ALPHA_MAX: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_AUTOMATIC_FONT_AXES = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_AUTOMATIC_FONT_AXES_NONE: DWRITE_AUTOMATIC_FONT_AXES = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_AUTOMATIC_FONT_AXES_OPTICAL_SIZE: DWRITE_AUTOMATIC_FONT_AXES = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_BASELINE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_DEFAULT: DWRITE_BASELINE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_ROMAN: DWRITE_BASELINE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_CENTRAL: DWRITE_BASELINE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_MATH: DWRITE_BASELINE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_HANGING: DWRITE_BASELINE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_IDEOGRAPHIC_BOTTOM: DWRITE_BASELINE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_IDEOGRAPHIC_TOP: DWRITE_BASELINE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_MINIMUM: DWRITE_BASELINE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BASELINE_MAXIMUM: DWRITE_BASELINE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_BREAK_CONDITION = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BREAK_CONDITION_NEUTRAL: DWRITE_BREAK_CONDITION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BREAK_CONDITION_CAN_BREAK: DWRITE_BREAK_CONDITION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BREAK_CONDITION_MAY_NOT_BREAK: DWRITE_BREAK_CONDITION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_BREAK_CONDITION_MUST_BREAK: DWRITE_BREAK_CONDITION = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_CARET_METRICS {
    pub slopeRise: i16,
    pub slopeRun: i16,
    pub offset: i16,
}
impl ::core::marker::Copy for DWRITE_CARET_METRICS {}
impl ::core::clone::Clone for DWRITE_CARET_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_CLUSTER_METRICS {
    pub width: f32,
    pub length: u16,
    pub _bitfield: u16,
}
impl ::core::marker::Copy for DWRITE_CLUSTER_METRICS {}
impl ::core::clone::Clone for DWRITE_CLUSTER_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_COLOR_F {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl ::core::marker::Copy for DWRITE_COLOR_F {}
impl ::core::clone::Clone for DWRITE_COLOR_F {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_COLOR_GLYPH_RUN {
    pub glyphRun: DWRITE_GLYPH_RUN,
    pub glyphRunDescription: *mut DWRITE_GLYPH_RUN_DESCRIPTION,
    pub baselineOriginX: f32,
    pub baselineOriginY: f32,
    pub runColor: DWRITE_COLOR_F,
    pub paletteIndex: u16,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_COLOR_GLYPH_RUN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_COLOR_GLYPH_RUN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_COLOR_GLYPH_RUN1 {
    pub Base: DWRITE_COLOR_GLYPH_RUN,
    pub glyphImageFormat: DWRITE_GLYPH_IMAGE_FORMATS,
    pub measuringMode: DWRITE_MEASURING_MODE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_COLOR_GLYPH_RUN1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_COLOR_GLYPH_RUN1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_CONTAINER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_CONTAINER_TYPE_UNKNOWN: DWRITE_CONTAINER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_CONTAINER_TYPE_WOFF: DWRITE_CONTAINER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_CONTAINER_TYPE_WOFF2: DWRITE_CONTAINER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_ERR_BASE: u32 = 20480u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_E_DOWNLOADCANCELLED: ::windows_sys::core::HRESULT = -2003283954i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_E_DOWNLOADFAILED: ::windows_sys::core::HRESULT = -2003283953i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_E_REMOTEFONT: ::windows_sys::core::HRESULT = -2003283955i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_E_TOOMANYDOWNLOADS: ::windows_sys::core::HRESULT = -2003283952i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FACTORY_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FACTORY_TYPE_SHARED: DWRITE_FACTORY_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FACTORY_TYPE_ISOLATED: DWRITE_FACTORY_TYPE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_FILE_FRAGMENT {
    pub fileOffset: u64,
    pub fragmentSize: u64,
}
impl ::core::marker::Copy for DWRITE_FILE_FRAGMENT {}
impl ::core::clone::Clone for DWRITE_FILE_FRAGMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FLOW_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM: DWRITE_FLOW_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FLOW_DIRECTION_BOTTOM_TO_TOP: DWRITE_FLOW_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FLOW_DIRECTION_LEFT_TO_RIGHT: DWRITE_FLOW_DIRECTION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FLOW_DIRECTION_RIGHT_TO_LEFT: DWRITE_FLOW_DIRECTION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_AXIS_ATTRIBUTES = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_ATTRIBUTES_NONE: DWRITE_FONT_AXIS_ATTRIBUTES = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_ATTRIBUTES_VARIABLE: DWRITE_FONT_AXIS_ATTRIBUTES = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_ATTRIBUTES_HIDDEN: DWRITE_FONT_AXIS_ATTRIBUTES = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_FONT_AXIS_RANGE {
    pub axisTag: DWRITE_FONT_AXIS_TAG,
    pub minValue: f32,
    pub maxValue: f32,
}
impl ::core::marker::Copy for DWRITE_FONT_AXIS_RANGE {}
impl ::core::clone::Clone for DWRITE_FONT_AXIS_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_AXIS_TAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_TAG_WEIGHT: DWRITE_FONT_AXIS_TAG = 1952999287u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_TAG_WIDTH: DWRITE_FONT_AXIS_TAG = 1752458359u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_TAG_SLANT: DWRITE_FONT_AXIS_TAG = 1953393779u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_TAG_OPTICAL_SIZE: DWRITE_FONT_AXIS_TAG = 2054385775u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_AXIS_TAG_ITALIC: DWRITE_FONT_AXIS_TAG = 1818326121u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_FONT_AXIS_VALUE {
    pub axisTag: DWRITE_FONT_AXIS_TAG,
    pub value: f32,
}
impl ::core::marker::Copy for DWRITE_FONT_AXIS_VALUE {}
impl ::core::clone::Clone for DWRITE_FONT_AXIS_VALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_FACE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_CFF: DWRITE_FONT_FACE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_TRUETYPE: DWRITE_FONT_FACE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_OPENTYPE_COLLECTION: DWRITE_FONT_FACE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_TYPE1: DWRITE_FONT_FACE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_VECTOR: DWRITE_FONT_FACE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_BITMAP: DWRITE_FONT_FACE_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_UNKNOWN: DWRITE_FONT_FACE_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_RAW_CFF: DWRITE_FONT_FACE_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FACE_TYPE_TRUETYPE_COLLECTION: DWRITE_FONT_FACE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_FAMILY_MODEL = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FAMILY_MODEL_TYPOGRAPHIC: DWRITE_FONT_FAMILY_MODEL = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FAMILY_MODEL_WEIGHT_STRETCH_STYLE: DWRITE_FONT_FAMILY_MODEL = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_FONT_FEATURE {
    pub nameTag: DWRITE_FONT_FEATURE_TAG,
    pub parameter: u32,
}
impl ::core::marker::Copy for DWRITE_FONT_FEATURE {}
impl ::core::clone::Clone for DWRITE_FONT_FEATURE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_FEATURE_TAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_ALTERNATIVE_FRACTIONS: DWRITE_FONT_FEATURE_TAG = 1668441697u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_PETITE_CAPITALS_FROM_CAPITALS: DWRITE_FONT_FEATURE_TAG = 1668297315u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SMALL_CAPITALS_FROM_CAPITALS: DWRITE_FONT_FEATURE_TAG = 1668493923u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_CONTEXTUAL_ALTERNATES: DWRITE_FONT_FEATURE_TAG = 1953259875u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_CASE_SENSITIVE_FORMS: DWRITE_FONT_FEATURE_TAG = 1702060387u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_GLYPH_COMPOSITION_DECOMPOSITION: DWRITE_FONT_FEATURE_TAG = 1886217059u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_CONTEXTUAL_LIGATURES: DWRITE_FONT_FEATURE_TAG = 1734962275u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_CAPITAL_SPACING: DWRITE_FONT_FEATURE_TAG = 1886613603u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_CONTEXTUAL_SWASH: DWRITE_FONT_FEATURE_TAG = 1752658787u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_CURSIVE_POSITIONING: DWRITE_FONT_FEATURE_TAG = 1936880995u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_DEFAULT: DWRITE_FONT_FEATURE_TAG = 1953261156u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_DISCRETIONARY_LIGATURES: DWRITE_FONT_FEATURE_TAG = 1734962276u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_EXPERT_FORMS: DWRITE_FONT_FEATURE_TAG = 1953527909u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_FRACTIONS: DWRITE_FONT_FEATURE_TAG = 1667330662u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_FULL_WIDTH: DWRITE_FONT_FEATURE_TAG = 1684633446u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HALF_FORMS: DWRITE_FONT_FEATURE_TAG = 1718378856u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HALANT_FORMS: DWRITE_FONT_FEATURE_TAG = 1852596584u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_ALTERNATE_HALF_WIDTH: DWRITE_FONT_FEATURE_TAG = 1953259880u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HISTORICAL_FORMS: DWRITE_FONT_FEATURE_TAG = 1953720680u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HORIZONTAL_KANA_ALTERNATES: DWRITE_FONT_FEATURE_TAG = 1634626408u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HISTORICAL_LIGATURES: DWRITE_FONT_FEATURE_TAG = 1734962280u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HALF_WIDTH: DWRITE_FONT_FEATURE_TAG = 1684633448u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_HOJO_KANJI_FORMS: DWRITE_FONT_FEATURE_TAG = 1869246312u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_JIS04_FORMS: DWRITE_FONT_FEATURE_TAG = 875589738u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_JIS78_FORMS: DWRITE_FONT_FEATURE_TAG = 943157354u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_JIS83_FORMS: DWRITE_FONT_FEATURE_TAG = 859336810u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_JIS90_FORMS: DWRITE_FONT_FEATURE_TAG = 809070698u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_KERNING: DWRITE_FONT_FEATURE_TAG = 1852990827u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STANDARD_LIGATURES: DWRITE_FONT_FEATURE_TAG = 1634167148u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_LINING_FIGURES: DWRITE_FONT_FEATURE_TAG = 1836412524u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_LOCALIZED_FORMS: DWRITE_FONT_FEATURE_TAG = 1818455916u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_MARK_POSITIONING: DWRITE_FONT_FEATURE_TAG = 1802658157u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_MATHEMATICAL_GREEK: DWRITE_FONT_FEATURE_TAG = 1802659693u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_MARK_TO_MARK_POSITIONING: DWRITE_FONT_FEATURE_TAG = 1802333037u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_ALTERNATE_ANNOTATION_FORMS: DWRITE_FONT_FEATURE_TAG = 1953259886u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_NLC_KANJI_FORMS: DWRITE_FONT_FEATURE_TAG = 1801677934u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_OLD_STYLE_FIGURES: DWRITE_FONT_FEATURE_TAG = 1836412527u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_ORDINALS: DWRITE_FONT_FEATURE_TAG = 1852076655u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_PROPORTIONAL_ALTERNATE_WIDTH: DWRITE_FONT_FEATURE_TAG = 1953259888u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_PETITE_CAPITALS: DWRITE_FONT_FEATURE_TAG = 1885430640u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_PROPORTIONAL_FIGURES: DWRITE_FONT_FEATURE_TAG = 1836412528u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_PROPORTIONAL_WIDTHS: DWRITE_FONT_FEATURE_TAG = 1684633456u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_QUARTER_WIDTHS: DWRITE_FONT_FEATURE_TAG = 1684633457u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_REQUIRED_LIGATURES: DWRITE_FONT_FEATURE_TAG = 1734962290u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_RUBY_NOTATION_FORMS: DWRITE_FONT_FEATURE_TAG = 2036495730u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_ALTERNATES: DWRITE_FONT_FEATURE_TAG = 1953259891u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SCIENTIFIC_INFERIORS: DWRITE_FONT_FEATURE_TAG = 1718511987u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SMALL_CAPITALS: DWRITE_FONT_FEATURE_TAG = 1885564275u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SIMPLIFIED_FORMS: DWRITE_FONT_FEATURE_TAG = 1819307379u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_1: DWRITE_FONT_FEATURE_TAG = 825258867u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_2: DWRITE_FONT_FEATURE_TAG = 842036083u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_3: DWRITE_FONT_FEATURE_TAG = 858813299u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_4: DWRITE_FONT_FEATURE_TAG = 875590515u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_5: DWRITE_FONT_FEATURE_TAG = 892367731u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_6: DWRITE_FONT_FEATURE_TAG = 909144947u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_7: DWRITE_FONT_FEATURE_TAG = 925922163u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_8: DWRITE_FONT_FEATURE_TAG = 942699379u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_9: DWRITE_FONT_FEATURE_TAG = 959476595u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_10: DWRITE_FONT_FEATURE_TAG = 808547187u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_11: DWRITE_FONT_FEATURE_TAG = 825324403u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_12: DWRITE_FONT_FEATURE_TAG = 842101619u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_13: DWRITE_FONT_FEATURE_TAG = 858878835u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_14: DWRITE_FONT_FEATURE_TAG = 875656051u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_15: DWRITE_FONT_FEATURE_TAG = 892433267u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_16: DWRITE_FONT_FEATURE_TAG = 909210483u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_17: DWRITE_FONT_FEATURE_TAG = 925987699u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_18: DWRITE_FONT_FEATURE_TAG = 942764915u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_19: DWRITE_FONT_FEATURE_TAG = 959542131u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_STYLISTIC_SET_20: DWRITE_FONT_FEATURE_TAG = 808612723u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SUBSCRIPT: DWRITE_FONT_FEATURE_TAG = 1935832435u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SUPERSCRIPT: DWRITE_FONT_FEATURE_TAG = 1936749939u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SWASH: DWRITE_FONT_FEATURE_TAG = 1752397683u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_TITLING: DWRITE_FONT_FEATURE_TAG = 1819568500u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_TRADITIONAL_NAME_FORMS: DWRITE_FONT_FEATURE_TAG = 1835101812u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES: DWRITE_FONT_FEATURE_TAG = 1836412532u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_TRADITIONAL_FORMS: DWRITE_FONT_FEATURE_TAG = 1684107892u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_THIRD_WIDTHS: DWRITE_FONT_FEATURE_TAG = 1684633460u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_UNICASE: DWRITE_FONT_FEATURE_TAG = 1667853941u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_VERTICAL_WRITING: DWRITE_FONT_FEATURE_TAG = 1953654134u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_VERTICAL_ALTERNATES_AND_ROTATION: DWRITE_FONT_FEATURE_TAG = 846492278u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FEATURE_TAG_SLASHED_ZERO: DWRITE_FONT_FEATURE_TAG = 1869768058u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_FILE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_UNKNOWN: DWRITE_FONT_FILE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_CFF: DWRITE_FONT_FILE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_TRUETYPE: DWRITE_FONT_FILE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_OPENTYPE_COLLECTION: DWRITE_FONT_FILE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_TYPE1_PFM: DWRITE_FONT_FILE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_TYPE1_PFB: DWRITE_FONT_FILE_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_VECTOR: DWRITE_FONT_FILE_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_BITMAP: DWRITE_FONT_FILE_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_FILE_TYPE_TRUETYPE_COLLECTION: DWRITE_FONT_FILE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_LINE_GAP_USAGE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_LINE_GAP_USAGE_DEFAULT: DWRITE_FONT_LINE_GAP_USAGE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_LINE_GAP_USAGE_DISABLED: DWRITE_FONT_LINE_GAP_USAGE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_LINE_GAP_USAGE_ENABLED: DWRITE_FONT_LINE_GAP_USAGE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_FONT_METRICS {
    pub designUnitsPerEm: u16,
    pub ascent: u16,
    pub descent: u16,
    pub lineGap: i16,
    pub capHeight: u16,
    pub xHeight: u16,
    pub underlinePosition: i16,
    pub underlineThickness: u16,
    pub strikethroughPosition: i16,
    pub strikethroughThickness: u16,
}
impl ::core::marker::Copy for DWRITE_FONT_METRICS {}
impl ::core::clone::Clone for DWRITE_FONT_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_FONT_METRICS1 {
    pub __AnonymousBase_DWrite_1_L627_C38: DWRITE_FONT_METRICS,
    pub glyphBoxLeft: i16,
    pub glyphBoxTop: i16,
    pub glyphBoxRight: i16,
    pub glyphBoxBottom: i16,
    pub subscriptPositionX: i16,
    pub subscriptPositionY: i16,
    pub subscriptSizeX: i16,
    pub subscriptSizeY: i16,
    pub superscriptPositionX: i16,
    pub superscriptPositionY: i16,
    pub superscriptSizeX: i16,
    pub superscriptSizeY: i16,
    pub hasTypographicMetrics: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_FONT_METRICS1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_FONT_METRICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_FONT_PROPERTY {
    pub propertyId: DWRITE_FONT_PROPERTY_ID,
    pub propertyValue: ::windows_sys::core::PCWSTR,
    pub localeName: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for DWRITE_FONT_PROPERTY {}
impl ::core::clone::Clone for DWRITE_FONT_PROPERTY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_PROPERTY_ID = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_NONE: DWRITE_FONT_PROPERTY_ID = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_WEIGHT_STRETCH_STYLE_FAMILY_NAME: DWRITE_FONT_PROPERTY_ID = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_TYPOGRAPHIC_FAMILY_NAME: DWRITE_FONT_PROPERTY_ID = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_WEIGHT_STRETCH_STYLE_FACE_NAME: DWRITE_FONT_PROPERTY_ID = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_FULL_NAME: DWRITE_FONT_PROPERTY_ID = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_WIN32_FAMILY_NAME: DWRITE_FONT_PROPERTY_ID = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_POSTSCRIPT_NAME: DWRITE_FONT_PROPERTY_ID = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_DESIGN_SCRIPT_LANGUAGE_TAG: DWRITE_FONT_PROPERTY_ID = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_SUPPORTED_SCRIPT_LANGUAGE_TAG: DWRITE_FONT_PROPERTY_ID = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_SEMANTIC_TAG: DWRITE_FONT_PROPERTY_ID = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_WEIGHT: DWRITE_FONT_PROPERTY_ID = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_STRETCH: DWRITE_FONT_PROPERTY_ID = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_STYLE: DWRITE_FONT_PROPERTY_ID = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_TYPOGRAPHIC_FACE_NAME: DWRITE_FONT_PROPERTY_ID = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_TOTAL: DWRITE_FONT_PROPERTY_ID = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_TOTAL_RS3: DWRITE_FONT_PROPERTY_ID = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_PREFERRED_FAMILY_NAME: DWRITE_FONT_PROPERTY_ID = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_FAMILY_NAME: DWRITE_FONT_PROPERTY_ID = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_PROPERTY_ID_FACE_NAME: DWRITE_FONT_PROPERTY_ID = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_SIMULATIONS = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SIMULATIONS_NONE: DWRITE_FONT_SIMULATIONS = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SIMULATIONS_BOLD: DWRITE_FONT_SIMULATIONS = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SIMULATIONS_OBLIQUE: DWRITE_FONT_SIMULATIONS = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_SOURCE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SOURCE_TYPE_UNKNOWN: DWRITE_FONT_SOURCE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SOURCE_TYPE_PER_MACHINE: DWRITE_FONT_SOURCE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SOURCE_TYPE_PER_USER: DWRITE_FONT_SOURCE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SOURCE_TYPE_APPX_PACKAGE: DWRITE_FONT_SOURCE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_SOURCE_TYPE_REMOTE_FONT_PROVIDER: DWRITE_FONT_SOURCE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_STRETCH = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_UNDEFINED: DWRITE_FONT_STRETCH = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_ULTRA_CONDENSED: DWRITE_FONT_STRETCH = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_EXTRA_CONDENSED: DWRITE_FONT_STRETCH = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_CONDENSED: DWRITE_FONT_STRETCH = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_SEMI_CONDENSED: DWRITE_FONT_STRETCH = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_NORMAL: DWRITE_FONT_STRETCH = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_MEDIUM: DWRITE_FONT_STRETCH = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_SEMI_EXPANDED: DWRITE_FONT_STRETCH = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_EXPANDED: DWRITE_FONT_STRETCH = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_EXTRA_EXPANDED: DWRITE_FONT_STRETCH = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STRETCH_ULTRA_EXPANDED: DWRITE_FONT_STRETCH = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_STYLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STYLE_NORMAL: DWRITE_FONT_STYLE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STYLE_OBLIQUE: DWRITE_FONT_STYLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_STYLE_ITALIC: DWRITE_FONT_STYLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_FONT_WEIGHT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_THIN: DWRITE_FONT_WEIGHT = 100i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_EXTRA_LIGHT: DWRITE_FONT_WEIGHT = 200i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_ULTRA_LIGHT: DWRITE_FONT_WEIGHT = 200i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_LIGHT: DWRITE_FONT_WEIGHT = 300i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_SEMI_LIGHT: DWRITE_FONT_WEIGHT = 350i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_NORMAL: DWRITE_FONT_WEIGHT = 400i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_REGULAR: DWRITE_FONT_WEIGHT = 400i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_MEDIUM: DWRITE_FONT_WEIGHT = 500i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_DEMI_BOLD: DWRITE_FONT_WEIGHT = 600i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_SEMI_BOLD: DWRITE_FONT_WEIGHT = 600i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_BOLD: DWRITE_FONT_WEIGHT = 700i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_EXTRA_BOLD: DWRITE_FONT_WEIGHT = 800i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_ULTRA_BOLD: DWRITE_FONT_WEIGHT = 800i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_BLACK: DWRITE_FONT_WEIGHT = 900i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_HEAVY: DWRITE_FONT_WEIGHT = 900i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_EXTRA_BLACK: DWRITE_FONT_WEIGHT = 950i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_FONT_WEIGHT_ULTRA_BLACK: DWRITE_FONT_WEIGHT = 950i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct2D_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub struct DWRITE_GLYPH_IMAGE_DATA {
    pub imageData: *const ::core::ffi::c_void,
    pub imageDataSize: u32,
    pub uniqueDataId: u32,
    pub pixelsPerEm: u32,
    pub pixelSize: super::Direct2D::Common::D2D_SIZE_U,
    pub horizontalLeftOrigin: super::super::Foundation::POINT,
    pub horizontalRightOrigin: super::super::Foundation::POINT,
    pub verticalTopOrigin: super::super::Foundation::POINT,
    pub verticalBottomOrigin: super::super::Foundation::POINT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ::core::marker::Copy for DWRITE_GLYPH_IMAGE_DATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ::core::clone::Clone for DWRITE_GLYPH_IMAGE_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_GLYPH_IMAGE_FORMATS = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_NONE: DWRITE_GLYPH_IMAGE_FORMATS = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_TRUETYPE: DWRITE_GLYPH_IMAGE_FORMATS = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_CFF: DWRITE_GLYPH_IMAGE_FORMATS = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_COLR: DWRITE_GLYPH_IMAGE_FORMATS = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_SVG: DWRITE_GLYPH_IMAGE_FORMATS = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_PNG: DWRITE_GLYPH_IMAGE_FORMATS = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_JPEG: DWRITE_GLYPH_IMAGE_FORMATS = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_TIFF: DWRITE_GLYPH_IMAGE_FORMATS = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_IMAGE_FORMATS_PREMULTIPLIED_B8G8R8A8: DWRITE_GLYPH_IMAGE_FORMATS = 128u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_GLYPH_METRICS {
    pub leftSideBearing: i32,
    pub advanceWidth: u32,
    pub rightSideBearing: i32,
    pub topSideBearing: i32,
    pub advanceHeight: u32,
    pub bottomSideBearing: i32,
    pub verticalOriginY: i32,
}
impl ::core::marker::Copy for DWRITE_GLYPH_METRICS {}
impl ::core::clone::Clone for DWRITE_GLYPH_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_GLYPH_OFFSET {
    pub advanceOffset: f32,
    pub ascenderOffset: f32,
}
impl ::core::marker::Copy for DWRITE_GLYPH_OFFSET {}
impl ::core::clone::Clone for DWRITE_GLYPH_OFFSET {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_GLYPH_ORIENTATION_ANGLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_ORIENTATION_ANGLE_0_DEGREES: DWRITE_GLYPH_ORIENTATION_ANGLE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_ORIENTATION_ANGLE_90_DEGREES: DWRITE_GLYPH_ORIENTATION_ANGLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_ORIENTATION_ANGLE_180_DEGREES: DWRITE_GLYPH_ORIENTATION_ANGLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GLYPH_ORIENTATION_ANGLE_270_DEGREES: DWRITE_GLYPH_ORIENTATION_ANGLE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_GLYPH_RUN {
    pub fontFace: IDWriteFontFace,
    pub fontEmSize: f32,
    pub glyphCount: u32,
    pub glyphIndices: *const u16,
    pub glyphAdvances: *const f32,
    pub glyphOffsets: *const DWRITE_GLYPH_OFFSET,
    pub isSideways: super::super::Foundation::BOOL,
    pub bidiLevel: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_GLYPH_RUN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_GLYPH_RUN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_GLYPH_RUN_DESCRIPTION {
    pub localeName: ::windows_sys::core::PCWSTR,
    pub string: ::windows_sys::core::PCWSTR,
    pub stringLength: u32,
    pub clusterMap: *const u16,
    pub textPosition: u32,
}
impl ::core::marker::Copy for DWRITE_GLYPH_RUN_DESCRIPTION {}
impl ::core::clone::Clone for DWRITE_GLYPH_RUN_DESCRIPTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_GRID_FIT_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GRID_FIT_MODE_DEFAULT: DWRITE_GRID_FIT_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GRID_FIT_MODE_DISABLED: DWRITE_GRID_FIT_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_GRID_FIT_MODE_ENABLED: DWRITE_GRID_FIT_MODE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_HIT_TEST_METRICS {
    pub textPosition: u32,
    pub length: u32,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub bidiLevel: u32,
    pub isText: super::super::Foundation::BOOL,
    pub isTrimmed: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_HIT_TEST_METRICS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_HIT_TEST_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_INFORMATIONAL_STRING_ID = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_NONE: DWRITE_INFORMATIONAL_STRING_ID = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_COPYRIGHT_NOTICE: DWRITE_INFORMATIONAL_STRING_ID = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_VERSION_STRINGS: DWRITE_INFORMATIONAL_STRING_ID = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_TRADEMARK: DWRITE_INFORMATIONAL_STRING_ID = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_MANUFACTURER: DWRITE_INFORMATIONAL_STRING_ID = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_DESIGNER: DWRITE_INFORMATIONAL_STRING_ID = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_DESIGNER_URL: DWRITE_INFORMATIONAL_STRING_ID = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_DESCRIPTION: DWRITE_INFORMATIONAL_STRING_ID = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_FONT_VENDOR_URL: DWRITE_INFORMATIONAL_STRING_ID = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_LICENSE_DESCRIPTION: DWRITE_INFORMATIONAL_STRING_ID = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_LICENSE_INFO_URL: DWRITE_INFORMATIONAL_STRING_ID = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_WIN32_FAMILY_NAMES: DWRITE_INFORMATIONAL_STRING_ID = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_WIN32_SUBFAMILY_NAMES: DWRITE_INFORMATIONAL_STRING_ID = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_TYPOGRAPHIC_FAMILY_NAMES: DWRITE_INFORMATIONAL_STRING_ID = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_TYPOGRAPHIC_SUBFAMILY_NAMES: DWRITE_INFORMATIONAL_STRING_ID = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_SAMPLE_TEXT: DWRITE_INFORMATIONAL_STRING_ID = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_FULL_NAME: DWRITE_INFORMATIONAL_STRING_ID = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME: DWRITE_INFORMATIONAL_STRING_ID = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_CID_NAME: DWRITE_INFORMATIONAL_STRING_ID = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_WEIGHT_STRETCH_STYLE_FAMILY_NAME: DWRITE_INFORMATIONAL_STRING_ID = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_DESIGN_SCRIPT_LANGUAGE_TAG: DWRITE_INFORMATIONAL_STRING_ID = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_SUPPORTED_SCRIPT_LANGUAGE_TAG: DWRITE_INFORMATIONAL_STRING_ID = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_PREFERRED_FAMILY_NAMES: DWRITE_INFORMATIONAL_STRING_ID = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_PREFERRED_SUBFAMILY_NAMES: DWRITE_INFORMATIONAL_STRING_ID = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_INFORMATIONAL_STRING_WWS_FAMILY_NAME: DWRITE_INFORMATIONAL_STRING_ID = 19i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_INLINE_OBJECT_METRICS {
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub supportsSideways: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_INLINE_OBJECT_METRICS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_INLINE_OBJECT_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_JUSTIFICATION_OPPORTUNITY {
    pub expansionMinimum: f32,
    pub expansionMaximum: f32,
    pub compressionMaximum: f32,
    pub _bitfield: u32,
}
impl ::core::marker::Copy for DWRITE_JUSTIFICATION_OPPORTUNITY {}
impl ::core::clone::Clone for DWRITE_JUSTIFICATION_OPPORTUNITY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_LINE_BREAKPOINT {
    pub _bitfield: u8,
}
impl ::core::marker::Copy for DWRITE_LINE_BREAKPOINT {}
impl ::core::clone::Clone for DWRITE_LINE_BREAKPOINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_LINE_METRICS {
    pub length: u32,
    pub trailingWhitespaceLength: u32,
    pub newlineLength: u32,
    pub height: f32,
    pub baseline: f32,
    pub isTrimmed: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_LINE_METRICS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_LINE_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DWRITE_LINE_METRICS1 {
    pub Base: DWRITE_LINE_METRICS,
    pub leadingBefore: f32,
    pub leadingAfter: f32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DWRITE_LINE_METRICS1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DWRITE_LINE_METRICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_LINE_SPACING {
    pub method: DWRITE_LINE_SPACING_METHOD,
    pub height: f32,
    pub baseline: f32,
    pub leadingBefore: f32,
    pub fontLineGapUsage: DWRITE_FONT_LINE_GAP_USAGE,
}
impl ::core::marker::Copy for DWRITE_LINE_SPACING {}
impl ::core::clone::Clone for DWRITE_LINE_SPACING {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_LINE_SPACING_METHOD = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_LINE_SPACING_METHOD_DEFAULT: DWRITE_LINE_SPACING_METHOD = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_LINE_SPACING_METHOD_UNIFORM: DWRITE_LINE_SPACING_METHOD = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_LINE_SPACING_METHOD_PROPORTIONAL: DWRITE_LINE_SPACING_METHOD = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_LOCALITY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_LOCALITY_REMOTE: DWRITE_LOCALITY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_LOCALITY_PARTIAL: DWRITE_LOCALITY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_LOCALITY_LOCAL: DWRITE_LOCALITY = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_MATRIX {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub dx: f32,
    pub dy: f32,
}
impl ::core::marker::Copy for DWRITE_MATRIX {}
impl ::core::clone::Clone for DWRITE_MATRIX {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_MEASURING_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_MEASURING_MODE_NATURAL: DWRITE_MEASURING_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_MEASURING_MODE_GDI_CLASSIC: DWRITE_MEASURING_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_MEASURING_MODE_GDI_NATURAL: DWRITE_MEASURING_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_NUMBER_SUBSTITUTION_METHOD = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_NUMBER_SUBSTITUTION_METHOD_FROM_CULTURE: DWRITE_NUMBER_SUBSTITUTION_METHOD = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_NUMBER_SUBSTITUTION_METHOD_CONTEXTUAL: DWRITE_NUMBER_SUBSTITUTION_METHOD = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_NUMBER_SUBSTITUTION_METHOD_NONE: DWRITE_NUMBER_SUBSTITUTION_METHOD = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_NUMBER_SUBSTITUTION_METHOD_NATIONAL: DWRITE_NUMBER_SUBSTITUTION_METHOD = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_NUMBER_SUBSTITUTION_METHOD_TRADITIONAL: DWRITE_NUMBER_SUBSTITUTION_METHOD = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_OPTICAL_ALIGNMENT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_OPTICAL_ALIGNMENT_NONE: DWRITE_OPTICAL_ALIGNMENT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_OPTICAL_ALIGNMENT_NO_SIDE_BEARINGS: DWRITE_OPTICAL_ALIGNMENT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_OUTLINE_THRESHOLD = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_OUTLINE_THRESHOLD_ANTIALIASED: DWRITE_OUTLINE_THRESHOLD = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_OUTLINE_THRESHOLD_ALIASED: DWRITE_OUTLINE_THRESHOLD = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_OVERHANG_METRICS {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl ::core::marker::Copy for DWRITE_OVERHANG_METRICS {}
impl ::core::clone::Clone for DWRITE_OVERHANG_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub union DWRITE_PANOSE {
    pub values: [u8; 10],
    pub familyKind: u8,
    pub text: DWRITE_PANOSE_3,
    pub script: DWRITE_PANOSE_1,
    pub decorative: DWRITE_PANOSE_0,
    pub symbol: DWRITE_PANOSE_2,
}
impl ::core::marker::Copy for DWRITE_PANOSE {}
impl ::core::clone::Clone for DWRITE_PANOSE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_PANOSE_0 {
    pub familyKind: u8,
    pub decorativeClass: u8,
    pub weight: u8,
    pub aspect: u8,
    pub contrast: u8,
    pub serifVariant: u8,
    pub fill: u8,
    pub lining: u8,
    pub decorativeTopology: u8,
    pub characterRange: u8,
}
impl ::core::marker::Copy for DWRITE_PANOSE_0 {}
impl ::core::clone::Clone for DWRITE_PANOSE_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_PANOSE_1 {
    pub familyKind: u8,
    pub toolKind: u8,
    pub weight: u8,
    pub spacing: u8,
    pub aspectRatio: u8,
    pub contrast: u8,
    pub scriptTopology: u8,
    pub scriptForm: u8,
    pub finials: u8,
    pub xAscent: u8,
}
impl ::core::marker::Copy for DWRITE_PANOSE_1 {}
impl ::core::clone::Clone for DWRITE_PANOSE_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_PANOSE_2 {
    pub familyKind: u8,
    pub symbolKind: u8,
    pub weight: u8,
    pub spacing: u8,
    pub aspectRatioAndContrast: u8,
    pub aspectRatio94: u8,
    pub aspectRatio119: u8,
    pub aspectRatio157: u8,
    pub aspectRatio163: u8,
    pub aspectRatio211: u8,
}
impl ::core::marker::Copy for DWRITE_PANOSE_2 {}
impl ::core::clone::Clone for DWRITE_PANOSE_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_PANOSE_3 {
    pub familyKind: u8,
    pub serifStyle: u8,
    pub weight: u8,
    pub proportion: u8,
    pub contrast: u8,
    pub strokeVariation: u8,
    pub armStyle: u8,
    pub letterform: u8,
    pub midline: u8,
    pub xHeight: u8,
}
impl ::core::marker::Copy for DWRITE_PANOSE_3 {}
impl ::core::clone::Clone for DWRITE_PANOSE_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_ARM_STYLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_ANY: DWRITE_PANOSE_ARM_STYLE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_NO_FIT: DWRITE_PANOSE_ARM_STYLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_HORIZONTAL: DWRITE_PANOSE_ARM_STYLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_WEDGE: DWRITE_PANOSE_ARM_STYLE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_VERTICAL: DWRITE_PANOSE_ARM_STYLE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_SINGLE_SERIF: DWRITE_PANOSE_ARM_STYLE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_DOUBLE_SERIF: DWRITE_PANOSE_ARM_STYLE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_NONSTRAIGHT_ARMS_HORIZONTAL: DWRITE_PANOSE_ARM_STYLE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_NONSTRAIGHT_ARMS_WEDGE: DWRITE_PANOSE_ARM_STYLE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_NONSTRAIGHT_ARMS_VERTICAL: DWRITE_PANOSE_ARM_STYLE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_NONSTRAIGHT_ARMS_SINGLE_SERIF: DWRITE_PANOSE_ARM_STYLE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_NONSTRAIGHT_ARMS_DOUBLE_SERIF: DWRITE_PANOSE_ARM_STYLE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_HORZ: DWRITE_PANOSE_ARM_STYLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_STRAIGHT_ARMS_VERT: DWRITE_PANOSE_ARM_STYLE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_BENT_ARMS_HORZ: DWRITE_PANOSE_ARM_STYLE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_BENT_ARMS_WEDGE: DWRITE_PANOSE_ARM_STYLE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_BENT_ARMS_VERT: DWRITE_PANOSE_ARM_STYLE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_BENT_ARMS_SINGLE_SERIF: DWRITE_PANOSE_ARM_STYLE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ARM_STYLE_BENT_ARMS_DOUBLE_SERIF: DWRITE_PANOSE_ARM_STYLE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_ASPECT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_ANY: DWRITE_PANOSE_ASPECT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_NO_FIT: DWRITE_PANOSE_ASPECT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_SUPER_CONDENSED: DWRITE_PANOSE_ASPECT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_VERY_CONDENSED: DWRITE_PANOSE_ASPECT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_CONDENSED: DWRITE_PANOSE_ASPECT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_NORMAL: DWRITE_PANOSE_ASPECT = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_EXTENDED: DWRITE_PANOSE_ASPECT = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_VERY_EXTENDED: DWRITE_PANOSE_ASPECT = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_SUPER_EXTENDED: DWRITE_PANOSE_ASPECT = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_MONOSPACED: DWRITE_PANOSE_ASPECT = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_ASPECT_RATIO = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_ANY: DWRITE_PANOSE_ASPECT_RATIO = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_NO_FIT: DWRITE_PANOSE_ASPECT_RATIO = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_VERY_CONDENSED: DWRITE_PANOSE_ASPECT_RATIO = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_CONDENSED: DWRITE_PANOSE_ASPECT_RATIO = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_NORMAL: DWRITE_PANOSE_ASPECT_RATIO = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_EXPANDED: DWRITE_PANOSE_ASPECT_RATIO = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_ASPECT_RATIO_VERY_EXPANDED: DWRITE_PANOSE_ASPECT_RATIO = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_CHARACTER_RANGES = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CHARACTER_RANGES_ANY: DWRITE_PANOSE_CHARACTER_RANGES = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CHARACTER_RANGES_NO_FIT: DWRITE_PANOSE_CHARACTER_RANGES = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CHARACTER_RANGES_EXTENDED_COLLECTION: DWRITE_PANOSE_CHARACTER_RANGES = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CHARACTER_RANGES_LITERALS: DWRITE_PANOSE_CHARACTER_RANGES = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CHARACTER_RANGES_NO_LOWER_CASE: DWRITE_PANOSE_CHARACTER_RANGES = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CHARACTER_RANGES_SMALL_CAPS: DWRITE_PANOSE_CHARACTER_RANGES = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_CONTRAST = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_ANY: DWRITE_PANOSE_CONTRAST = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_NO_FIT: DWRITE_PANOSE_CONTRAST = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_NONE: DWRITE_PANOSE_CONTRAST = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_VERY_LOW: DWRITE_PANOSE_CONTRAST = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_LOW: DWRITE_PANOSE_CONTRAST = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_MEDIUM_LOW: DWRITE_PANOSE_CONTRAST = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_MEDIUM: DWRITE_PANOSE_CONTRAST = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_MEDIUM_HIGH: DWRITE_PANOSE_CONTRAST = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_HIGH: DWRITE_PANOSE_CONTRAST = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_VERY_HIGH: DWRITE_PANOSE_CONTRAST = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_HORIZONTAL_LOW: DWRITE_PANOSE_CONTRAST = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_HORIZONTAL_MEDIUM: DWRITE_PANOSE_CONTRAST = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_HORIZONTAL_HIGH: DWRITE_PANOSE_CONTRAST = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_CONTRAST_BROKEN: DWRITE_PANOSE_CONTRAST = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_DECORATIVE_CLASS = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_ANY: DWRITE_PANOSE_DECORATIVE_CLASS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_NO_FIT: DWRITE_PANOSE_DECORATIVE_CLASS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_DERIVATIVE: DWRITE_PANOSE_DECORATIVE_CLASS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_NONSTANDARD_TOPOLOGY: DWRITE_PANOSE_DECORATIVE_CLASS = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_NONSTANDARD_ELEMENTS: DWRITE_PANOSE_DECORATIVE_CLASS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_NONSTANDARD_ASPECT: DWRITE_PANOSE_DECORATIVE_CLASS = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_INITIALS: DWRITE_PANOSE_DECORATIVE_CLASS = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_CARTOON: DWRITE_PANOSE_DECORATIVE_CLASS = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_PICTURE_STEMS: DWRITE_PANOSE_DECORATIVE_CLASS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_ORNAMENTED: DWRITE_PANOSE_DECORATIVE_CLASS = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_TEXT_AND_BACKGROUND: DWRITE_PANOSE_DECORATIVE_CLASS = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_COLLAGE: DWRITE_PANOSE_DECORATIVE_CLASS = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_CLASS_MONTAGE: DWRITE_PANOSE_DECORATIVE_CLASS = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_DECORATIVE_TOPOLOGY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_ANY: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_NO_FIT: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_STANDARD: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_SQUARE: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_MULTIPLE_SEGMENT: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_ART_DECO: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_UNEVEN_WEIGHTING: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_DIVERSE_ARMS: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_DIVERSE_FORMS: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_LOMBARDIC_FORMS: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_UPPER_CASE_IN_LOWER_CASE: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_IMPLIED_TOPOLOGY: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_HORSESHOE_E_AND_A: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_CURSIVE: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_BLACKLETTER: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_DECORATIVE_TOPOLOGY_SWASH_VARIANCE: DWRITE_PANOSE_DECORATIVE_TOPOLOGY = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_FAMILY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_ANY: DWRITE_PANOSE_FAMILY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_NO_FIT: DWRITE_PANOSE_FAMILY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_TEXT_DISPLAY: DWRITE_PANOSE_FAMILY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_SCRIPT: DWRITE_PANOSE_FAMILY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_DECORATIVE: DWRITE_PANOSE_FAMILY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_SYMBOL: DWRITE_PANOSE_FAMILY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FAMILY_PICTORIAL: DWRITE_PANOSE_FAMILY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_FILL = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_ANY: DWRITE_PANOSE_FILL = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_NO_FIT: DWRITE_PANOSE_FILL = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_STANDARD_SOLID_FILL: DWRITE_PANOSE_FILL = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_NO_FILL: DWRITE_PANOSE_FILL = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_PATTERNED_FILL: DWRITE_PANOSE_FILL = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_COMPLEX_FILL: DWRITE_PANOSE_FILL = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_SHAPED_FILL: DWRITE_PANOSE_FILL = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FILL_DRAWN_DISTRESSED: DWRITE_PANOSE_FILL = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_FINIALS = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_ANY: DWRITE_PANOSE_FINIALS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_NO_FIT: DWRITE_PANOSE_FINIALS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_NONE_NO_LOOPS: DWRITE_PANOSE_FINIALS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_NONE_CLOSED_LOOPS: DWRITE_PANOSE_FINIALS = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_NONE_OPEN_LOOPS: DWRITE_PANOSE_FINIALS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_SHARP_NO_LOOPS: DWRITE_PANOSE_FINIALS = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_SHARP_CLOSED_LOOPS: DWRITE_PANOSE_FINIALS = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_SHARP_OPEN_LOOPS: DWRITE_PANOSE_FINIALS = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_TAPERED_NO_LOOPS: DWRITE_PANOSE_FINIALS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_TAPERED_CLOSED_LOOPS: DWRITE_PANOSE_FINIALS = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_TAPERED_OPEN_LOOPS: DWRITE_PANOSE_FINIALS = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_ROUND_NO_LOOPS: DWRITE_PANOSE_FINIALS = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_ROUND_CLOSED_LOOPS: DWRITE_PANOSE_FINIALS = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_FINIALS_ROUND_OPEN_LOOPS: DWRITE_PANOSE_FINIALS = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_LETTERFORM = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_ANY: DWRITE_PANOSE_LETTERFORM = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NO_FIT: DWRITE_PANOSE_LETTERFORM = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_CONTACT: DWRITE_PANOSE_LETTERFORM = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_WEIGHTED: DWRITE_PANOSE_LETTERFORM = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_BOXED: DWRITE_PANOSE_LETTERFORM = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_FLATTENED: DWRITE_PANOSE_LETTERFORM = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_ROUNDED: DWRITE_PANOSE_LETTERFORM = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_OFF_CENTER: DWRITE_PANOSE_LETTERFORM = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_NORMAL_SQUARE: DWRITE_PANOSE_LETTERFORM = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_CONTACT: DWRITE_PANOSE_LETTERFORM = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_WEIGHTED: DWRITE_PANOSE_LETTERFORM = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_BOXED: DWRITE_PANOSE_LETTERFORM = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_FLATTENED: DWRITE_PANOSE_LETTERFORM = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_ROUNDED: DWRITE_PANOSE_LETTERFORM = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_OFF_CENTER: DWRITE_PANOSE_LETTERFORM = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LETTERFORM_OBLIQUE_SQUARE: DWRITE_PANOSE_LETTERFORM = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_LINING = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_ANY: DWRITE_PANOSE_LINING = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_NO_FIT: DWRITE_PANOSE_LINING = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_NONE: DWRITE_PANOSE_LINING = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_INLINE: DWRITE_PANOSE_LINING = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_OUTLINE: DWRITE_PANOSE_LINING = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_ENGRAVED: DWRITE_PANOSE_LINING = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_SHADOW: DWRITE_PANOSE_LINING = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_RELIEF: DWRITE_PANOSE_LINING = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_LINING_BACKDROP: DWRITE_PANOSE_LINING = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_MIDLINE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_ANY: DWRITE_PANOSE_MIDLINE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_NO_FIT: DWRITE_PANOSE_MIDLINE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_STANDARD_TRIMMED: DWRITE_PANOSE_MIDLINE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_STANDARD_POINTED: DWRITE_PANOSE_MIDLINE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_STANDARD_SERIFED: DWRITE_PANOSE_MIDLINE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_HIGH_TRIMMED: DWRITE_PANOSE_MIDLINE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_HIGH_POINTED: DWRITE_PANOSE_MIDLINE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_HIGH_SERIFED: DWRITE_PANOSE_MIDLINE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_CONSTANT_TRIMMED: DWRITE_PANOSE_MIDLINE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_CONSTANT_POINTED: DWRITE_PANOSE_MIDLINE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_CONSTANT_SERIFED: DWRITE_PANOSE_MIDLINE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_LOW_TRIMMED: DWRITE_PANOSE_MIDLINE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_LOW_POINTED: DWRITE_PANOSE_MIDLINE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_MIDLINE_LOW_SERIFED: DWRITE_PANOSE_MIDLINE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_PROPORTION = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_ANY: DWRITE_PANOSE_PROPORTION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_NO_FIT: DWRITE_PANOSE_PROPORTION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_OLD_STYLE: DWRITE_PANOSE_PROPORTION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_MODERN: DWRITE_PANOSE_PROPORTION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_EVEN_WIDTH: DWRITE_PANOSE_PROPORTION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_EXPANDED: DWRITE_PANOSE_PROPORTION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_CONDENSED: DWRITE_PANOSE_PROPORTION = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_VERY_EXPANDED: DWRITE_PANOSE_PROPORTION = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_VERY_CONDENSED: DWRITE_PANOSE_PROPORTION = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_PROPORTION_MONOSPACED: DWRITE_PANOSE_PROPORTION = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_SCRIPT_FORM = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_ANY: DWRITE_PANOSE_SCRIPT_FORM = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_NO_FIT: DWRITE_PANOSE_SCRIPT_FORM = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_UPRIGHT_NO_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_UPRIGHT_SOME_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_UPRIGHT_MORE_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_UPRIGHT_EXTREME_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_OBLIQUE_NO_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_OBLIQUE_SOME_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_OBLIQUE_MORE_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_OBLIQUE_EXTREME_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_EXAGGERATED_NO_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_EXAGGERATED_SOME_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_EXAGGERATED_MORE_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_FORM_EXAGGERATED_EXTREME_WRAPPING: DWRITE_PANOSE_SCRIPT_FORM = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_SCRIPT_TOPOLOGY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_ANY: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_NO_FIT: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_ROMAN_DISCONNECTED: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_ROMAN_TRAILING: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_ROMAN_CONNECTED: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_CURSIVE_DISCONNECTED: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_CURSIVE_TRAILING: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_CURSIVE_CONNECTED: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_BLACKLETTER_DISCONNECTED: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_BLACKLETTER_TRAILING: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SCRIPT_TOPOLOGY_BLACKLETTER_CONNECTED: DWRITE_PANOSE_SCRIPT_TOPOLOGY = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_SERIF_STYLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_ANY: DWRITE_PANOSE_SERIF_STYLE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_NO_FIT: DWRITE_PANOSE_SERIF_STYLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_COVE: DWRITE_PANOSE_SERIF_STYLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_OBTUSE_COVE: DWRITE_PANOSE_SERIF_STYLE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_SQUARE_COVE: DWRITE_PANOSE_SERIF_STYLE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_OBTUSE_SQUARE_COVE: DWRITE_PANOSE_SERIF_STYLE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_SQUARE: DWRITE_PANOSE_SERIF_STYLE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_THIN: DWRITE_PANOSE_SERIF_STYLE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_OVAL: DWRITE_PANOSE_SERIF_STYLE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_EXAGGERATED: DWRITE_PANOSE_SERIF_STYLE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_TRIANGLE: DWRITE_PANOSE_SERIF_STYLE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_NORMAL_SANS: DWRITE_PANOSE_SERIF_STYLE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_OBTUSE_SANS: DWRITE_PANOSE_SERIF_STYLE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_PERPENDICULAR_SANS: DWRITE_PANOSE_SERIF_STYLE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_FLARED: DWRITE_PANOSE_SERIF_STYLE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_ROUNDED: DWRITE_PANOSE_SERIF_STYLE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_SCRIPT: DWRITE_PANOSE_SERIF_STYLE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_PERP_SANS: DWRITE_PANOSE_SERIF_STYLE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SERIF_STYLE_BONE: DWRITE_PANOSE_SERIF_STYLE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_SPACING = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SPACING_ANY: DWRITE_PANOSE_SPACING = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SPACING_NO_FIT: DWRITE_PANOSE_SPACING = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SPACING_PROPORTIONAL_SPACED: DWRITE_PANOSE_SPACING = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SPACING_MONOSPACED: DWRITE_PANOSE_SPACING = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_STROKE_VARIATION = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_ANY: DWRITE_PANOSE_STROKE_VARIATION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_NO_FIT: DWRITE_PANOSE_STROKE_VARIATION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_NO_VARIATION: DWRITE_PANOSE_STROKE_VARIATION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_GRADUAL_DIAGONAL: DWRITE_PANOSE_STROKE_VARIATION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_GRADUAL_TRANSITIONAL: DWRITE_PANOSE_STROKE_VARIATION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_GRADUAL_VERTICAL: DWRITE_PANOSE_STROKE_VARIATION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_GRADUAL_HORIZONTAL: DWRITE_PANOSE_STROKE_VARIATION = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_RAPID_VERTICAL: DWRITE_PANOSE_STROKE_VARIATION = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_RAPID_HORIZONTAL: DWRITE_PANOSE_STROKE_VARIATION = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_INSTANT_VERTICAL: DWRITE_PANOSE_STROKE_VARIATION = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_STROKE_VARIATION_INSTANT_HORIZONTAL: DWRITE_PANOSE_STROKE_VARIATION = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_ANY: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_NO_FIT: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_NO_WIDTH: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_EXCEPTIONALLY_WIDE: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_SUPER_WIDE: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_VERY_WIDE: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_WIDE: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_NORMAL: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_NARROW: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_ASPECT_RATIO_VERY_NARROW: DWRITE_PANOSE_SYMBOL_ASPECT_RATIO = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_SYMBOL_KIND = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_ANY: DWRITE_PANOSE_SYMBOL_KIND = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_NO_FIT: DWRITE_PANOSE_SYMBOL_KIND = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_MONTAGES: DWRITE_PANOSE_SYMBOL_KIND = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_PICTURES: DWRITE_PANOSE_SYMBOL_KIND = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_SHAPES: DWRITE_PANOSE_SYMBOL_KIND = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_SCIENTIFIC: DWRITE_PANOSE_SYMBOL_KIND = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_MUSIC: DWRITE_PANOSE_SYMBOL_KIND = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_EXPERT: DWRITE_PANOSE_SYMBOL_KIND = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_PATTERNS: DWRITE_PANOSE_SYMBOL_KIND = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_BOARDERS: DWRITE_PANOSE_SYMBOL_KIND = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_ICONS: DWRITE_PANOSE_SYMBOL_KIND = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_LOGOS: DWRITE_PANOSE_SYMBOL_KIND = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_SYMBOL_KIND_INDUSTRY_SPECIFIC: DWRITE_PANOSE_SYMBOL_KIND = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_TOOL_KIND = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_ANY: DWRITE_PANOSE_TOOL_KIND = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_NO_FIT: DWRITE_PANOSE_TOOL_KIND = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_FLAT_NIB: DWRITE_PANOSE_TOOL_KIND = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_PRESSURE_POINT: DWRITE_PANOSE_TOOL_KIND = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_ENGRAVED: DWRITE_PANOSE_TOOL_KIND = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_BALL: DWRITE_PANOSE_TOOL_KIND = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_BRUSH: DWRITE_PANOSE_TOOL_KIND = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_ROUGH: DWRITE_PANOSE_TOOL_KIND = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_FELT_PEN_BRUSH_TIP: DWRITE_PANOSE_TOOL_KIND = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_TOOL_KIND_WILD_BRUSH: DWRITE_PANOSE_TOOL_KIND = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_WEIGHT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_ANY: DWRITE_PANOSE_WEIGHT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_NO_FIT: DWRITE_PANOSE_WEIGHT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_VERY_LIGHT: DWRITE_PANOSE_WEIGHT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_LIGHT: DWRITE_PANOSE_WEIGHT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_THIN: DWRITE_PANOSE_WEIGHT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_BOOK: DWRITE_PANOSE_WEIGHT = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_MEDIUM: DWRITE_PANOSE_WEIGHT = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_DEMI: DWRITE_PANOSE_WEIGHT = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_BOLD: DWRITE_PANOSE_WEIGHT = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_HEAVY: DWRITE_PANOSE_WEIGHT = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_BLACK: DWRITE_PANOSE_WEIGHT = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_EXTRA_BLACK: DWRITE_PANOSE_WEIGHT = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_WEIGHT_NORD: DWRITE_PANOSE_WEIGHT = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_XASCENT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_ANY: DWRITE_PANOSE_XASCENT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_NO_FIT: DWRITE_PANOSE_XASCENT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_VERY_LOW: DWRITE_PANOSE_XASCENT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_LOW: DWRITE_PANOSE_XASCENT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_MEDIUM: DWRITE_PANOSE_XASCENT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_HIGH: DWRITE_PANOSE_XASCENT = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XASCENT_VERY_HIGH: DWRITE_PANOSE_XASCENT = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PANOSE_XHEIGHT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_ANY: DWRITE_PANOSE_XHEIGHT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_NO_FIT: DWRITE_PANOSE_XHEIGHT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_CONSTANT_SMALL: DWRITE_PANOSE_XHEIGHT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_CONSTANT_STANDARD: DWRITE_PANOSE_XHEIGHT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_CONSTANT_LARGE: DWRITE_PANOSE_XHEIGHT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_DUCKING_SMALL: DWRITE_PANOSE_XHEIGHT = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_DUCKING_STANDARD: DWRITE_PANOSE_XHEIGHT = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_DUCKING_LARGE: DWRITE_PANOSE_XHEIGHT = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_CONSTANT_STD: DWRITE_PANOSE_XHEIGHT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PANOSE_XHEIGHT_DUCKING_STD: DWRITE_PANOSE_XHEIGHT = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PARAGRAPH_ALIGNMENT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PARAGRAPH_ALIGNMENT_NEAR: DWRITE_PARAGRAPH_ALIGNMENT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PARAGRAPH_ALIGNMENT_FAR: DWRITE_PARAGRAPH_ALIGNMENT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PARAGRAPH_ALIGNMENT_CENTER: DWRITE_PARAGRAPH_ALIGNMENT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_PIXEL_GEOMETRY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PIXEL_GEOMETRY_FLAT: DWRITE_PIXEL_GEOMETRY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PIXEL_GEOMETRY_RGB: DWRITE_PIXEL_GEOMETRY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_PIXEL_GEOMETRY_BGR: DWRITE_PIXEL_GEOMETRY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_READING_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_READING_DIRECTION_LEFT_TO_RIGHT: DWRITE_READING_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_READING_DIRECTION_RIGHT_TO_LEFT: DWRITE_READING_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_READING_DIRECTION_TOP_TO_BOTTOM: DWRITE_READING_DIRECTION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_READING_DIRECTION_BOTTOM_TO_TOP: DWRITE_READING_DIRECTION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_RENDERING_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_DEFAULT: DWRITE_RENDERING_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_ALIASED: DWRITE_RENDERING_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_GDI_CLASSIC: DWRITE_RENDERING_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_GDI_NATURAL: DWRITE_RENDERING_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_NATURAL: DWRITE_RENDERING_MODE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC: DWRITE_RENDERING_MODE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_OUTLINE: DWRITE_RENDERING_MODE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_CLEARTYPE_GDI_CLASSIC: DWRITE_RENDERING_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_CLEARTYPE_GDI_NATURAL: DWRITE_RENDERING_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL: DWRITE_RENDERING_MODE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL_SYMMETRIC: DWRITE_RENDERING_MODE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_RENDERING_MODE1 = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_DEFAULT: DWRITE_RENDERING_MODE1 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_ALIASED: DWRITE_RENDERING_MODE1 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_GDI_CLASSIC: DWRITE_RENDERING_MODE1 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_GDI_NATURAL: DWRITE_RENDERING_MODE1 = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_NATURAL: DWRITE_RENDERING_MODE1 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_NATURAL_SYMMETRIC: DWRITE_RENDERING_MODE1 = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_OUTLINE: DWRITE_RENDERING_MODE1 = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_RENDERING_MODE1_NATURAL_SYMMETRIC_DOWNSAMPLED: DWRITE_RENDERING_MODE1 = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_SCRIPT_ANALYSIS {
    pub script: u16,
    pub shapes: DWRITE_SCRIPT_SHAPES,
}
impl ::core::marker::Copy for DWRITE_SCRIPT_ANALYSIS {}
impl ::core::clone::Clone for DWRITE_SCRIPT_ANALYSIS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_SCRIPT_PROPERTIES {
    pub isoScriptCode: u32,
    pub isoScriptNumber: u32,
    pub clusterLookahead: u32,
    pub justificationCharacter: u32,
    pub _bitfield: u32,
}
impl ::core::marker::Copy for DWRITE_SCRIPT_PROPERTIES {}
impl ::core::clone::Clone for DWRITE_SCRIPT_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_SCRIPT_SHAPES = u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_SCRIPT_SHAPES_DEFAULT: DWRITE_SCRIPT_SHAPES = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_SCRIPT_SHAPES_NO_VISUAL: DWRITE_SCRIPT_SHAPES = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_SHAPING_GLYPH_PROPERTIES {
    pub _bitfield: u16,
}
impl ::core::marker::Copy for DWRITE_SHAPING_GLYPH_PROPERTIES {}
impl ::core::clone::Clone for DWRITE_SHAPING_GLYPH_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_SHAPING_TEXT_PROPERTIES {
    pub _bitfield: u16,
}
impl ::core::marker::Copy for DWRITE_SHAPING_TEXT_PROPERTIES {}
impl ::core::clone::Clone for DWRITE_SHAPING_TEXT_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_STRIKETHROUGH {
    pub width: f32,
    pub thickness: f32,
    pub offset: f32,
    pub readingDirection: DWRITE_READING_DIRECTION,
    pub flowDirection: DWRITE_FLOW_DIRECTION,
    pub localeName: ::windows_sys::core::PCWSTR,
    pub measuringMode: DWRITE_MEASURING_MODE,
}
impl ::core::marker::Copy for DWRITE_STRIKETHROUGH {}
impl ::core::clone::Clone for DWRITE_STRIKETHROUGH {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_TEXTURE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXTURE_ALIASED_1x1: DWRITE_TEXTURE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXTURE_CLEARTYPE_3x1: DWRITE_TEXTURE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_TEXT_ALIGNMENT = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXT_ALIGNMENT_LEADING: DWRITE_TEXT_ALIGNMENT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXT_ALIGNMENT_TRAILING: DWRITE_TEXT_ALIGNMENT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXT_ALIGNMENT_CENTER: DWRITE_TEXT_ALIGNMENT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXT_ALIGNMENT_JUSTIFIED: DWRITE_TEXT_ALIGNMENT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_TEXT_ANTIALIAS_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE: DWRITE_TEXT_ANTIALIAS_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE: DWRITE_TEXT_ANTIALIAS_MODE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_TEXT_METRICS {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub widthIncludingTrailingWhitespace: f32,
    pub height: f32,
    pub layoutWidth: f32,
    pub layoutHeight: f32,
    pub maxBidiReorderingDepth: u32,
    pub lineCount: u32,
}
impl ::core::marker::Copy for DWRITE_TEXT_METRICS {}
impl ::core::clone::Clone for DWRITE_TEXT_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_TEXT_METRICS1 {
    pub Base: DWRITE_TEXT_METRICS,
    pub heightIncludingTrailingWhitespace: f32,
}
impl ::core::marker::Copy for DWRITE_TEXT_METRICS1 {}
impl ::core::clone::Clone for DWRITE_TEXT_METRICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_TEXT_RANGE {
    pub startPosition: u32,
    pub length: u32,
}
impl ::core::marker::Copy for DWRITE_TEXT_RANGE {}
impl ::core::clone::Clone for DWRITE_TEXT_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_TRIMMING {
    pub granularity: DWRITE_TRIMMING_GRANULARITY,
    pub delimiter: u32,
    pub delimiterCount: u32,
}
impl ::core::marker::Copy for DWRITE_TRIMMING {}
impl ::core::clone::Clone for DWRITE_TRIMMING {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_TRIMMING_GRANULARITY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TRIMMING_GRANULARITY_NONE: DWRITE_TRIMMING_GRANULARITY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TRIMMING_GRANULARITY_CHARACTER: DWRITE_TRIMMING_GRANULARITY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_TRIMMING_GRANULARITY_WORD: DWRITE_TRIMMING_GRANULARITY = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_TYPOGRAPHIC_FEATURES {
    pub features: *mut DWRITE_FONT_FEATURE,
    pub featureCount: u32,
}
impl ::core::marker::Copy for DWRITE_TYPOGRAPHIC_FEATURES {}
impl ::core::clone::Clone for DWRITE_TYPOGRAPHIC_FEATURES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_UNDERLINE {
    pub width: f32,
    pub thickness: f32,
    pub offset: f32,
    pub runHeight: f32,
    pub readingDirection: DWRITE_READING_DIRECTION,
    pub flowDirection: DWRITE_FLOW_DIRECTION,
    pub localeName: ::windows_sys::core::PCWSTR,
    pub measuringMode: DWRITE_MEASURING_MODE,
}
impl ::core::marker::Copy for DWRITE_UNDERLINE {}
impl ::core::clone::Clone for DWRITE_UNDERLINE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub struct DWRITE_UNICODE_RANGE {
    pub first: u32,
    pub last: u32,
}
impl ::core::marker::Copy for DWRITE_UNICODE_RANGE {}
impl ::core::clone::Clone for DWRITE_UNICODE_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_VERTICAL_GLYPH_ORIENTATION = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_VERTICAL_GLYPH_ORIENTATION_DEFAULT: DWRITE_VERTICAL_GLYPH_ORIENTATION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_VERTICAL_GLYPH_ORIENTATION_STACKED: DWRITE_VERTICAL_GLYPH_ORIENTATION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub type DWRITE_WORD_WRAPPING = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_WORD_WRAPPING_WRAP: DWRITE_WORD_WRAPPING = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_WORD_WRAPPING_NO_WRAP: DWRITE_WORD_WRAPPING = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_WORD_WRAPPING_EMERGENCY_BREAK: DWRITE_WORD_WRAPPING = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_WORD_WRAPPING_WHOLE_WORD: DWRITE_WORD_WRAPPING = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const DWRITE_WORD_WRAPPING_CHARACTER: DWRITE_WORD_WRAPPING = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectWrite\"`*"]
pub const FACILITY_DWRITE: u32 = 2200u32;
pub type IDWriteAsyncResult = *mut ::core::ffi::c_void;
pub type IDWriteBitmapRenderTarget = *mut ::core::ffi::c_void;
pub type IDWriteBitmapRenderTarget1 = *mut ::core::ffi::c_void;
pub type IDWriteColorGlyphRunEnumerator = *mut ::core::ffi::c_void;
pub type IDWriteColorGlyphRunEnumerator1 = *mut ::core::ffi::c_void;
pub type IDWriteFactory = *mut ::core::ffi::c_void;
pub type IDWriteFactory1 = *mut ::core::ffi::c_void;
pub type IDWriteFactory2 = *mut ::core::ffi::c_void;
pub type IDWriteFactory3 = *mut ::core::ffi::c_void;
pub type IDWriteFactory4 = *mut ::core::ffi::c_void;
pub type IDWriteFactory5 = *mut ::core::ffi::c_void;
pub type IDWriteFactory6 = *mut ::core::ffi::c_void;
pub type IDWriteFactory7 = *mut ::core::ffi::c_void;
pub type IDWriteFont = *mut ::core::ffi::c_void;
pub type IDWriteFont1 = *mut ::core::ffi::c_void;
pub type IDWriteFont2 = *mut ::core::ffi::c_void;
pub type IDWriteFont3 = *mut ::core::ffi::c_void;
pub type IDWriteFontCollection = *mut ::core::ffi::c_void;
pub type IDWriteFontCollection1 = *mut ::core::ffi::c_void;
pub type IDWriteFontCollection2 = *mut ::core::ffi::c_void;
pub type IDWriteFontCollection3 = *mut ::core::ffi::c_void;
pub type IDWriteFontCollectionLoader = *mut ::core::ffi::c_void;
pub type IDWriteFontDownloadListener = *mut ::core::ffi::c_void;
pub type IDWriteFontDownloadQueue = *mut ::core::ffi::c_void;
pub type IDWriteFontFace = *mut ::core::ffi::c_void;
pub type IDWriteFontFace1 = *mut ::core::ffi::c_void;
pub type IDWriteFontFace2 = *mut ::core::ffi::c_void;
pub type IDWriteFontFace3 = *mut ::core::ffi::c_void;
pub type IDWriteFontFace4 = *mut ::core::ffi::c_void;
pub type IDWriteFontFace5 = *mut ::core::ffi::c_void;
pub type IDWriteFontFace6 = *mut ::core::ffi::c_void;
pub type IDWriteFontFaceReference = *mut ::core::ffi::c_void;
pub type IDWriteFontFaceReference1 = *mut ::core::ffi::c_void;
pub type IDWriteFontFallback = *mut ::core::ffi::c_void;
pub type IDWriteFontFallback1 = *mut ::core::ffi::c_void;
pub type IDWriteFontFallbackBuilder = *mut ::core::ffi::c_void;
pub type IDWriteFontFamily = *mut ::core::ffi::c_void;
pub type IDWriteFontFamily1 = *mut ::core::ffi::c_void;
pub type IDWriteFontFamily2 = *mut ::core::ffi::c_void;
pub type IDWriteFontFile = *mut ::core::ffi::c_void;
pub type IDWriteFontFileEnumerator = *mut ::core::ffi::c_void;
pub type IDWriteFontFileLoader = *mut ::core::ffi::c_void;
pub type IDWriteFontFileStream = *mut ::core::ffi::c_void;
pub type IDWriteFontList = *mut ::core::ffi::c_void;
pub type IDWriteFontList1 = *mut ::core::ffi::c_void;
pub type IDWriteFontList2 = *mut ::core::ffi::c_void;
pub type IDWriteFontResource = *mut ::core::ffi::c_void;
pub type IDWriteFontSet = *mut ::core::ffi::c_void;
pub type IDWriteFontSet1 = *mut ::core::ffi::c_void;
pub type IDWriteFontSet2 = *mut ::core::ffi::c_void;
pub type IDWriteFontSet3 = *mut ::core::ffi::c_void;
pub type IDWriteFontSetBuilder = *mut ::core::ffi::c_void;
pub type IDWriteFontSetBuilder1 = *mut ::core::ffi::c_void;
pub type IDWriteFontSetBuilder2 = *mut ::core::ffi::c_void;
pub type IDWriteGdiInterop = *mut ::core::ffi::c_void;
pub type IDWriteGdiInterop1 = *mut ::core::ffi::c_void;
pub type IDWriteGlyphRunAnalysis = *mut ::core::ffi::c_void;
pub type IDWriteInMemoryFontFileLoader = *mut ::core::ffi::c_void;
pub type IDWriteInlineObject = *mut ::core::ffi::c_void;
pub type IDWriteLocalFontFileLoader = *mut ::core::ffi::c_void;
pub type IDWriteLocalizedStrings = *mut ::core::ffi::c_void;
pub type IDWriteNumberSubstitution = *mut ::core::ffi::c_void;
pub type IDWritePixelSnapping = *mut ::core::ffi::c_void;
pub type IDWriteRemoteFontFileLoader = *mut ::core::ffi::c_void;
pub type IDWriteRemoteFontFileStream = *mut ::core::ffi::c_void;
pub type IDWriteRenderingParams = *mut ::core::ffi::c_void;
pub type IDWriteRenderingParams1 = *mut ::core::ffi::c_void;
pub type IDWriteRenderingParams2 = *mut ::core::ffi::c_void;
pub type IDWriteRenderingParams3 = *mut ::core::ffi::c_void;
pub type IDWriteStringList = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalysisSink = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalysisSink1 = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalysisSource = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalysisSource1 = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalyzer = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalyzer1 = *mut ::core::ffi::c_void;
pub type IDWriteTextAnalyzer2 = *mut ::core::ffi::c_void;
pub type IDWriteTextFormat = *mut ::core::ffi::c_void;
pub type IDWriteTextFormat1 = *mut ::core::ffi::c_void;
pub type IDWriteTextFormat2 = *mut ::core::ffi::c_void;
pub type IDWriteTextFormat3 = *mut ::core::ffi::c_void;
pub type IDWriteTextLayout = *mut ::core::ffi::c_void;
pub type IDWriteTextLayout1 = *mut ::core::ffi::c_void;
pub type IDWriteTextLayout2 = *mut ::core::ffi::c_void;
pub type IDWriteTextLayout3 = *mut ::core::ffi::c_void;
pub type IDWriteTextLayout4 = *mut ::core::ffi::c_void;
pub type IDWriteTextRenderer = *mut ::core::ffi::c_void;
pub type IDWriteTextRenderer1 = *mut ::core::ffi::c_void;
pub type IDWriteTypography = *mut ::core::ffi::c_void;
