#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn AddStroke(hrc: HRECOCONTEXT, ppacketdesc: *const PACKET_DESCRIPTION, cbpacket: u32, ppacket: *const u8, pxform: *const super::super::Graphics::Gdi::XFORM) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn AddWordsToWordList(hwl: HRECOWORDLIST, pwcwords: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn AdviseInkChange(hrc: HRECOCONTEXT, bnewstroke: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn CreateContext(hrec: HRECOGNIZER, phrc: *mut HRECOCONTEXT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn CreateRecognizer(pclsid: *mut ::windows_sys::core::GUID, phrec: *mut HRECOGNIZER) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn DestroyContext(hrc: HRECOCONTEXT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn DestroyRecognizer(hrec: HRECOGNIZER) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn DestroyWordList(hwl: HRECOWORDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn EndInkInput(hrc: HRECOCONTEXT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetAllRecognizers(recognizerclsids: *mut *mut ::windows_sys::core::GUID, count: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetBestResultString(hrc: HRECOCONTEXT, pcsize: *mut u32, pwcbestresult: ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetLatticePtr(hrc: HRECOCONTEXT, pplattice: *mut *mut RECO_LATTICE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetLeftSeparator(hrc: HRECOCONTEXT, pcsize: *mut u32, pwcleftseparator: ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetRecoAttributes(hrec: HRECOGNIZER, precoattrs: *mut RECO_ATTRS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetResultPropertyList(hrec: HRECOGNIZER, ppropertycount: *mut u32, ppropertyguid: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetRightSeparator(hrc: HRECOCONTEXT, pcsize: *mut u32, pwcrightseparator: ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn GetUnicodeRanges(hrec: HRECOGNIZER, pcranges: *mut u32, pcr: *mut CHARACTER_RANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn IsStringSupported(hrc: HRECOCONTEXT, wcstring: u32, pwcstring: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn LoadCachedAttributes(clsid: ::windows_sys::core::GUID, precoattributes: *mut RECO_ATTRS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn MakeWordList(hrec: HRECOGNIZER, pbuffer: ::windows_sys::core::PCWSTR, phwl: *mut HRECOWORDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn Process(hrc: HRECOCONTEXT, pbpartialprocessing: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn SetEnabledUnicodeRanges(hrc: HRECOCONTEXT, cranges: u32, pcr: *mut CHARACTER_RANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn SetFactoid(hrc: HRECOCONTEXT, cwcfactoid: u32, pwcfactoid: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn SetFlags(hrc: HRECOCONTEXT, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn SetGuide(hrc: HRECOCONTEXT, pguide: *const RECO_GUIDE, iindex: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn SetTextContext(hrc: HRECOCONTEXT, cwcbefore: u32, pwcbefore: ::windows_sys::core::PCWSTR, cwcafter: u32, pwcafter: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
    pub fn SetWordList(hrc: HRECOCONTEXT, hwl: HRECOWORDLIST) -> ::windows_sys::core::HRESULT;
}
pub type IDynamicRenderer = *mut ::core::ffi::c_void;
pub type IGestureRecognizer = *mut ::core::ffi::c_void;
pub type IHandwrittenTextInsertion = *mut ::core::ffi::c_void;
pub type IInk = *mut ::core::ffi::c_void;
pub type IInkCollector = *mut ::core::ffi::c_void;
pub type IInkCursor = *mut ::core::ffi::c_void;
pub type IInkCursorButton = *mut ::core::ffi::c_void;
pub type IInkCursorButtons = *mut ::core::ffi::c_void;
pub type IInkCursors = *mut ::core::ffi::c_void;
pub type IInkCustomStrokes = *mut ::core::ffi::c_void;
pub type IInkDisp = *mut ::core::ffi::c_void;
pub type IInkDivider = *mut ::core::ffi::c_void;
pub type IInkDivisionResult = *mut ::core::ffi::c_void;
pub type IInkDivisionUnit = *mut ::core::ffi::c_void;
pub type IInkDivisionUnits = *mut ::core::ffi::c_void;
pub type IInkDrawingAttributes = *mut ::core::ffi::c_void;
pub type IInkEdit = *mut ::core::ffi::c_void;
pub type IInkExtendedProperties = *mut ::core::ffi::c_void;
pub type IInkExtendedProperty = *mut ::core::ffi::c_void;
pub type IInkGesture = *mut ::core::ffi::c_void;
pub type IInkLineInfo = *mut ::core::ffi::c_void;
pub type IInkOverlay = *mut ::core::ffi::c_void;
pub type IInkPicture = *mut ::core::ffi::c_void;
pub type IInkRecognitionAlternate = *mut ::core::ffi::c_void;
pub type IInkRecognitionAlternates = *mut ::core::ffi::c_void;
pub type IInkRecognitionResult = *mut ::core::ffi::c_void;
pub type IInkRecognizer = *mut ::core::ffi::c_void;
pub type IInkRecognizer2 = *mut ::core::ffi::c_void;
pub type IInkRecognizerContext = *mut ::core::ffi::c_void;
pub type IInkRecognizerContext2 = *mut ::core::ffi::c_void;
pub type IInkRecognizerGuide = *mut ::core::ffi::c_void;
pub type IInkRecognizers = *mut ::core::ffi::c_void;
pub type IInkRectangle = *mut ::core::ffi::c_void;
pub type IInkRenderer = *mut ::core::ffi::c_void;
pub type IInkStrokeDisp = *mut ::core::ffi::c_void;
pub type IInkStrokes = *mut ::core::ffi::c_void;
pub type IInkTablet = *mut ::core::ffi::c_void;
pub type IInkTablet2 = *mut ::core::ffi::c_void;
pub type IInkTablet3 = *mut ::core::ffi::c_void;
pub type IInkTablets = *mut ::core::ffi::c_void;
pub type IInkTransform = *mut ::core::ffi::c_void;
pub type IInkWordList = *mut ::core::ffi::c_void;
pub type IInkWordList2 = *mut ::core::ffi::c_void;
pub type IInputPanelWindowHandle = *mut ::core::ffi::c_void;
pub type IMathInputControl = *mut ::core::ffi::c_void;
pub type IPenInputPanel = *mut ::core::ffi::c_void;
pub type IRealTimeStylus = *mut ::core::ffi::c_void;
pub type IRealTimeStylus2 = *mut ::core::ffi::c_void;
pub type IRealTimeStylus3 = *mut ::core::ffi::c_void;
pub type IRealTimeStylusSynchronization = *mut ::core::ffi::c_void;
pub type ISketchInk = *mut ::core::ffi::c_void;
pub type IStrokeBuilder = *mut ::core::ffi::c_void;
pub type IStylusAsyncPlugin = *mut ::core::ffi::c_void;
pub type IStylusPlugin = *mut ::core::ffi::c_void;
pub type IStylusSyncPlugin = *mut ::core::ffi::c_void;
pub type ITextInputPanel = *mut ::core::ffi::c_void;
pub type ITextInputPanelEventSink = *mut ::core::ffi::c_void;
pub type ITextInputPanelRunInfo = *mut ::core::ffi::c_void;
pub type ITipAutoCompleteClient = *mut ::core::ffi::c_void;
pub type ITipAutoCompleteProvider = *mut ::core::ffi::c_void;
pub type _IInkCollectorEvents = *mut ::core::ffi::c_void;
pub type _IInkEditEvents = *mut ::core::ffi::c_void;
pub type _IInkEvents = *mut ::core::ffi::c_void;
pub type _IInkOverlayEvents = *mut ::core::ffi::c_void;
pub type _IInkPictureEvents = *mut ::core::ffi::c_void;
pub type _IInkRecognitionEvents = *mut ::core::ffi::c_void;
pub type _IInkStrokesEvents = *mut ::core::ffi::c_void;
pub type _IMathInputControlEvents = *mut ::core::ffi::c_void;
pub type _IPenInputPanelEvents = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_ADDSTROKE_FAILED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_INTERRUPTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_PROCESS_FAILED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_RESETCONTEXT_FAILED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_SETCACMODE_FAILED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_SETFACTOID_FAILED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_SETFLAGS_FAILED: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_SETGUIDE_FAILED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_SETTEXTCONTEXT_FAILED: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ASYNC_RECO_SETWORDLIST_FAILED: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const BEST_COMPLETE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CAC_FULL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CAC_PREFIX: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CAC_RANDOM: u32 = 2u32;
pub const DynamicRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3973262058, data2: 29807, data3: 19915, data4: [191, 104, 8, 39, 87, 250, 255, 24] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETDRAWATTR: u32 = 1541u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETFACTOID: u32 = 1549u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETGESTURESTATUS: u32 = 1545u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETINKINSERTMODE: u32 = 1539u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETINKMODE: u32 = 1537u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETMOUSEICON: u32 = 1553u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETMOUSEPOINTER: u32 = 1555u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETRECOGNIZER: u32 = 1547u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETRECOTIMEOUT: u32 = 1543u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETSELINK: u32 = 1551u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETSELINKDISPLAYMODE: u32 = 1562u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETSTATUS: u32 = 1557u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_GETUSEMOUSEFORINPUT: u32 = 1559u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_RECOGNIZE: u32 = 1558u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETDRAWATTR: u32 = 1542u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETFACTOID: u32 = 1550u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETGESTURESTATUS: u32 = 1546u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETINKINSERTMODE: u32 = 1540u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETINKMODE: u32 = 1538u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETMOUSEICON: u32 = 1554u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETMOUSEPOINTER: u32 = 1556u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETRECOGNIZER: u32 = 1548u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETRECOTIMEOUT: u32 = 1544u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETSELINK: u32 = 1552u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETSELINKDISPLAYMODE: u32 = 1561u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EM_SETUSEMOUSEFORINPUT: u32 = 1560u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACILITY_INK: u32 = 40u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_BOPOMOFO: &str = "BOPOMOFO";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_CHINESESIMPLECOMMON: &str = "CHS_COMMON";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_CHINESETRADITIONALCOMMON: &str = "CHT_COMMON";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_CURRENCY: &str = "CURRENCY";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_DATE: &str = "DATE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_DEFAULT: &str = "DEFAULT";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_DIGIT: &str = "DIGIT";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_EMAIL: &str = "EMAIL";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_FILENAME: &str = "FILENAME";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_HANGULCOMMON: &str = "HANGUL_COMMON";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_HANGULRARE: &str = "HANGUL_RARE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_HIRAGANA: &str = "HIRAGANA";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_JAMO: &str = "JAMO";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_JAPANESECOMMON: &str = "JPN_COMMON";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_KANJICOMMON: &str = "KANJI_COMMON";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_KANJIRARE: &str = "KANJI_RARE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_KATAKANA: &str = "KATAKANA";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_KOREANCOMMON: &str = "KOR_COMMON";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_LOWERCHAR: &str = "LOWERCHAR";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_NONE: &str = "NONE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_NUMBER: &str = "NUMBER";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_NUMBERSIMPLE: &str = "NUMSIMPLE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_ONECHAR: &str = "ONECHAR";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_PERCENT: &str = "PERCENT";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_POSTALCODE: &str = "POSTALCODE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_PUNCCHAR: &str = "PUNCCHAR";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_SYSTEMDICTIONARY: &str = "SYSDICT";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_TELEPHONE: &str = "TELEPHONE";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_TIME: &str = "TIME";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_UPPERCHAR: &str = "UPPERCHAR";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_WEB: &str = "WEB";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FACTOID_WORDLIST: &str = "WORDLIST";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICK_WM_HANDLED_MASK: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_ARROW_DOWN: u32 = 61497u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_ARROW_LEFT: u32 = 61498u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_ARROW_RIGHT: u32 = 61499u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_ARROW_UP: u32 = 61496u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_ASTERISK: u32 = 61608u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACE_LEFT: u32 = 61674u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACE_OVER: u32 = 61672u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACE_RIGHT: u32 = 61675u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACE_UNDER: u32 = 61673u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACKET_LEFT: u32 = 61670u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACKET_OVER: u32 = 61668u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACKET_RIGHT: u32 = 61671u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BRACKET_UNDER: u32 = 61669u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BULLET: u32 = 61450u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_BULLET_CROSS: u32 = 61451u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CHECK: u32 = 61445u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CHEVRON_DOWN: u32 = 61489u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CHEVRON_LEFT: u32 = 61490u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CHEVRON_RIGHT: u32 = 61491u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CHEVRON_UP: u32 = 61488u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CIRCLE: u32 = 61472u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CIRCLE_CIRCLE: u32 = 61475u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CIRCLE_CROSS: u32 = 61477u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CIRCLE_LINE_HORZ: u32 = 61479u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CIRCLE_LINE_VERT: u32 = 61478u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CIRCLE_TAP: u32 = 61474u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CLOSEUP: u32 = 61455u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CROSS: u32 = 61447u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_CURLICUE: u32 = 61456u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIAGONAL_LEFTDOWN: u32 = 61534u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIAGONAL_LEFTUP: u32 = 61532u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIAGONAL_RIGHTDOWN: u32 = 61535u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIAGONAL_RIGHTUP: u32 = 61533u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_0: u32 = 61594u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_1: u32 = 61595u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_2: u32 = 61596u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_3: u32 = 61597u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_4: u32 = 61598u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_5: u32 = 61599u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_6: u32 = 61600u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_7: u32 = 61601u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_8: u32 = 61602u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DIGIT_9: u32 = 61603u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOLLAR: u32 = 61607u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_ARROW_DOWN: u32 = 61501u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_ARROW_LEFT: u32 = 61502u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_ARROW_RIGHT: u32 = 61503u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_ARROW_UP: u32 = 61500u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_CIRCLE: u32 = 61473u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_CURLICUE: u32 = 61457u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_DOWN: u32 = 61625u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_LEFT: u32 = 61626u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_RIGHT: u32 = 61627u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_TAP: u32 = 61681u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOUBLE_UP: u32 = 61624u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN: u32 = 61529u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_ARROW_LEFT: u32 = 61506u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_ARROW_RIGHT: u32 = 61507u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_LEFT: u32 = 61546u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_LEFT_LONG: u32 = 61542u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_RIGHT: u32 = 61547u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_RIGHT_LONG: u32 = 61543u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_DOWN_UP: u32 = 61537u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_EXCLAMATION: u32 = 61604u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_INFINITY: u32 = 61446u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LEFT: u32 = 61530u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LEFT_ARROW_DOWN: u32 = 61509u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LEFT_ARROW_UP: u32 = 61508u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LEFT_DOWN: u32 = 61549u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LEFT_RIGHT: u32 = 61538u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LEFT_UP: u32 = 61548u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_A: u32 = 61568u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_B: u32 = 61569u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_C: u32 = 61570u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_D: u32 = 61571u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_E: u32 = 61572u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_F: u32 = 61573u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_G: u32 = 61574u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_H: u32 = 61575u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_I: u32 = 61576u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_J: u32 = 61577u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_K: u32 = 61578u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_L: u32 = 61579u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_M: u32 = 61580u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_N: u32 = 61581u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_O: u32 = 61582u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_P: u32 = 61583u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_Q: u32 = 61584u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_R: u32 = 61585u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_S: u32 = 61586u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_T: u32 = 61587u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_U: u32 = 61588u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_V: u32 = 61589u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_W: u32 = 61590u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_X: u32 = 61591u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_Y: u32 = 61592u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_LETTER_Z: u32 = 61593u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_NULL: u32 = 61440u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_OPENUP: u32 = 61454u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_PARAGRAPH: u32 = 61448u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_PLUS: u32 = 61609u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_QUAD_TAP: u32 = 61683u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_QUESTION: u32 = 61605u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RECTANGLE: u32 = 61458u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RIGHT: u32 = 61531u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RIGHT_ARROW_DOWN: u32 = 61511u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RIGHT_ARROW_UP: u32 = 61510u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RIGHT_DOWN: u32 = 61551u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RIGHT_LEFT: u32 = 61539u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_RIGHT_UP: u32 = 61550u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SCRATCHOUT: u32 = 61441u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SECTION: u32 = 61449u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SEMICIRCLE_LEFT: u32 = 61480u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SEMICIRCLE_RIGHT: u32 = 61481u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SHARP: u32 = 61606u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SQUARE: u32 = 61443u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SQUIGGLE: u32 = 61452u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_STAR: u32 = 61444u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_SWAP: u32 = 61453u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TAP: u32 = 61680u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TRIANGLE: u32 = 61442u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TRIPLE_DOWN: u32 = 61629u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TRIPLE_LEFT: u32 = 61630u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TRIPLE_RIGHT: u32 = 61631u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TRIPLE_TAP: u32 = 61682u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_TRIPLE_UP: u32 = 61628u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP: u32 = 61528u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_ARROW_LEFT: u32 = 61504u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_ARROW_RIGHT: u32 = 61505u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_DOWN: u32 = 61536u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_LEFT: u32 = 61544u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_LEFT_LONG: u32 = 61540u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_RIGHT: u32 = 61545u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const GESTURE_UP_RIGHT_LONG: u32 = 61541u32;
pub const GUID_DYNAMIC_RENDERER_CACHED_DATA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3209894802, data2: 9663, data3: 19093, data4: [137, 173, 14, 71, 107, 52, 180, 245] };
pub const GUID_GESTURE_DATA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1105521679, data2: 9898, data3: 17754, data4: [154, 165, 44, 211, 108, 246, 63, 185] };
pub const GUID_PACKETPROPERTY_GUID_ALTITUDE_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2195637703, data2: 63162, data3: 18694, data4: [137, 79, 102, 214, 141, 252, 69, 108] };
pub const GUID_PACKETPROPERTY_GUID_AZIMUTH_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 43066292, data2: 34856, data3: 16651, data4: [178, 80, 160, 83, 101, 149, 229, 220] };
pub const GUID_PACKETPROPERTY_GUID_BUTTON_PRESSURE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2340417476, data2: 38570, data3: 19454, data4: [172, 38, 138, 95, 11, 224, 123, 245] };
pub const GUID_PACKETPROPERTY_GUID_DEVICE_CONTACT_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 39345041, data2: 1179, data3: 18256, data4: [150, 21, 223, 137, 72, 171, 60, 156] };
pub const GUID_PACKETPROPERTY_GUID_FINGERCONTACTCONFIDENCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3875981316, data2: 22512, data3: 20224, data4: [138, 12, 133, 61, 87, 120, 155, 233] };
pub const GUID_PACKETPROPERTY_GUID_HEIGHT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3860355282, data2: 58439, data3: 16920, data4: [157, 63, 24, 134, 92, 32, 61, 244] };
pub const GUID_PACKETPROPERTY_GUID_NORMAL_PRESSURE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1929859117, data2: 63988, data3: 19992, data4: [179, 242, 44, 225, 177, 163, 97, 12] };
pub const GUID_PACKETPROPERTY_GUID_PACKET_STATUS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1846413247, data2: 45031, data3: 19703, data4: [135, 209, 175, 100, 70, 32, 132, 24] };
pub const GUID_PACKETPROPERTY_GUID_PITCH_ROTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2138986423, data2: 48695, data3: 19425, data4: [163, 86, 122, 132, 22, 14, 24, 147] };
pub const GUID_PACKETPROPERTY_GUID_ROLL_ROTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1566400086, data2: 27561, data3: 19547, data4: [159, 176, 133, 28, 145, 113, 78, 86] };
pub const GUID_PACKETPROPERTY_GUID_SERIAL_NUMBER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2024282966, data2: 2357, data3: 17555, data4: [186, 174, 0, 84, 26, 138, 22, 196] };
pub const GUID_PACKETPROPERTY_GUID_TANGENT_PRESSURE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1839483019, data2: 21060, data3: 16876, data4: [144, 91, 50, 216, 154, 184, 8, 9] };
pub const GUID_PACKETPROPERTY_GUID_TIMER_TICK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1130696901, data2: 65235, data3: 17873, data4: [139, 118, 113, 211, 234, 122, 130, 157] };
pub const GUID_PACKETPROPERTY_GUID_TWIST_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 221399392, data2: 5042, data3: 16868, data4: [172, 230, 122, 233, 212, 61, 45, 59] };
pub const GUID_PACKETPROPERTY_GUID_WIDTH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3131828557, data2: 10002, data3: 18677, data4: [190, 157, 143, 139, 94, 160, 113, 26] };
pub const GUID_PACKETPROPERTY_GUID_X: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1502243471, data2: 21184, data3: 19360, data4: [147, 175, 175, 53, 116, 17, 165, 97] };
pub const GUID_PACKETPROPERTY_GUID_X_TILT_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2832235322, data2: 35824, data3: 16560, data4: [149, 169, 184, 10, 107, 183, 135, 191] };
pub const GUID_PACKETPROPERTY_GUID_Y: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3040845685, data2: 1248, data3: 17560, data4: [167, 238, 195, 13, 187, 90, 144, 17] };
pub const GUID_PACKETPROPERTY_GUID_YAW_ROTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1787074944, data2: 31802, data3: 17847, data4: [170, 130, 144, 162, 98, 149, 14, 137] };
pub const GUID_PACKETPROPERTY_GUID_Y_TILT_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 244523913, data2: 7543, data3: 17327, data4: [172, 0, 91, 149, 13, 109, 75, 45] };
pub const GUID_PACKETPROPERTY_GUID_Z: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1935334192, data2: 3771, data3: 18312, data4: [160, 228, 15, 49, 100, 144, 5, 93] };
pub const GestureRecognizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3929065044, data2: 50732, data3: 17439, data4: [172, 0, 149, 249, 161, 150, 120, 44] };
pub const HandwrittenTextInsertion: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2668056290, data2: 59113, data3: 19850, data4: [160, 71, 235, 91, 92, 60, 85, 218] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IECN_GESTURE: u32 = 2050u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IECN_RECOGNITIONRESULT: u32 = 2051u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IECN_STROKE: u32 = 2049u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IECN__BASE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEC__BASE: u32 = 1536u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKEDIT_CLASS: &str = "INKEDIT";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKEDIT_CLASSW: &str = "INKEDIT";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_BOXNUMBER: &str = "{2C243E3A-F733-4EB6-B1F8-B5DC5C2C4CDA}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_CONFIDENCELEVEL: &str = "{7DFE11A7-FB5D-4958-8765-154ADF0D833F}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_HOTPOINT: &str = "{CA6F40DC-5292-452a-91FB-2181C0BEC0DE}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_LINEMETRICS: &str = "{8CC24B27-30A9-4b96-9056-2D3A90DA0727}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_LINENUMBER: &str = "{DBF29F2C-5289-4BE8-B3D8-6EF63246253E}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_MAXIMUMSTROKECOUNT: &str = "{BF0EEC4E-4B7D-47a9-8CFA-234DD24BD22A}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_POINTSPERINCH: &str = "{7ED16B76-889C-468e-8276-0021B770187E}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INKRECOGNITIONPROPERTY_SEGMENTATION: &str = "{B3C0FE6C-FB51-4164-BA2F-844AF8F983DA}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const INK_SERIALIZED_FORMAT: &str = "Ink Serialized Format";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IP_CURSOR_DOWN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IP_INVERTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IP_MARGIN: u32 = 4u32;
pub const Ink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 333335106, data2: 36129, data3: 19598, data4: [191, 156, 143, 105, 203, 6, 143, 202] };
pub const InkCollector: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1140528467, data2: 44404, data3: 20200, data4: [136, 228, 62, 109, 170, 201, 21, 219] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InkCollectorClipInkToMargin: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InkCollectorDefaultMargin: i32 = -2147483648i32;
pub const InkDisp: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2474383924, data2: 5405, data3: 17936, data4: [156, 166, 168, 204, 155, 219, 93, 131] };
pub const InkDivider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2287269536, data2: 18051, data3: 19175, data4: [145, 145, 117, 47, 230, 70, 18, 195] };
pub const InkDrawingAttributes: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3636408994, data2: 1445, data3: 17603, data4: [179, 170, 94, 128, 172, 125, 37, 118] };
pub const InkEdit: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3855243765, data2: 22468, data3: 19928, data4: [155, 214, 29, 238, 237, 210, 122, 244] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InkMaxTransparencyValue: i32 = 255i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InkMinTransparencyValue: i32 = 0i32;
pub const InkOverlay: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1708131910, data2: 52707, data3: 19080, data4: [145, 99, 103, 105, 240, 241, 169, 125] };
pub const InkPicture: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 77718867, data2: 65078, data3: 20446, data4: [134, 94, 52, 65, 148, 230, 148, 36] };
pub const InkRecognizerContext: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2864998967, data2: 37417, data3: 20416, data4: [140, 206, 68, 151, 86, 155, 244, 209] };
pub const InkRecognizerGuide: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2272319809, data2: 42554, data3: 18033, data4: [163, 117, 40, 85, 161, 142, 186, 115] };
pub const InkRecognizers: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2681530376, data2: 63206, data3: 20069, data4: [152, 211, 170, 57, 5, 76, 18, 85] };
pub const InkRectangle: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1135637286, data2: 43744, data3: 19298, data4: [168, 61, 95, 215, 104, 183, 53, 60] };
pub const InkRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2619131620, data2: 55275, data3: 20203, data4: [144, 145, 21, 167, 200, 121, 30, 217] };
pub const InkStrokes: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1223987644, data2: 9230, data3: 18528, data4: [176, 121, 161, 233, 77, 61, 44, 134] };
pub const InkTablets: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1850723090, data2: 20746, data3: 19776, data4: [147, 4, 29, 161, 10, 233, 20, 124] };
pub const InkTransform: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3822442812, data2: 5731, data3: 19064, data4: [161, 167, 34, 55, 93, 254, 186, 238] };
pub const InkWordList: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2649247892, data2: 63263, data3: 17649, data4: [132, 113, 21, 162, 250, 118, 252, 243] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MAX_FRIENDLYNAME: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MAX_LANGUAGES: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MAX_PACKET_BUTTON_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MAX_PACKET_PROPERTY_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MAX_VENDORNAME: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICROSOFT_PENINPUT_PANEL_PROPERTY_T: &str = "Microsoft PenInputPanel 1.5";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICROSOFT_TIP_COMBOBOXLIST_PROPERTY: &str = "Microsoft TIP ComboBox List Window Identifier";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICROSOFT_TIP_NO_INSERT_BUTTON_PROPERTY: &str = "Microsoft TIP No Insert Option";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICROSOFT_TIP_OPENING_MSG: &str = "TabletInputPanelOpening";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICROSOFT_URL_EXPERIENCE_PROPERTY: &str = "Microsoft TIP URL Experience";
pub const MathInputControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3311501676, data2: 5336, data3: 16528, data4: [131, 12, 152, 217, 148, 178, 28, 123] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const NUM_FLICK_DIRECTIONS: u32 = 8u32;
pub const PenInputPanel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4148487318, data2: 7002, data3: 18590, data4: [129, 220, 251, 215, 172, 98, 152, 168] };
pub const PenInputPanel_Internal: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2150309817, data2: 1387, data3: 18208, data4: [176, 204, 128, 210, 59, 113, 23, 30] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOCONF_HIGHCONFIDENCE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOCONF_LOWCONFIDENCE: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOCONF_MEDIUMCONFIDENCE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOCONF_NOTSET: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_AUTOSPACE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_COERCE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_DISABLEPERSONALIZATION: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_LINEMODE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_PREFIXOK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_SINGLESEG: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECOFLAG_WORDMODE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_ADVISEINKCHANGE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_ARBITRARY_ANGLE: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_BOXED_INPUT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_CAC_INPUT: i32 = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_DONTCARE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_DOWN_AND_LEFT: i32 = 256i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_DOWN_AND_RIGHT: i32 = 512i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_FREE_INPUT: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_LATTICE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_LEFT_AND_DOWN: i32 = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_LINED_INPUT: i32 = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_OBJECT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_PERFORMSLINEBREAKING: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_PERSONALIZABLE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_REQUIRESSEGMENTATIONBREAKING: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_RIGHT_AND_DOWN: i32 = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RF_STROKEREORDER: i32 = 8192i32;
pub const RealTimeStylus: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3798677101, data2: 63896, data3: 17358, data4: [131, 111, 203, 109, 144, 68, 50, 176] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SAFE_PARTIAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_ALTITUDEORIENTATION: &str = "{82DEC5C7-F6BA-4906-894F-66D68DFC456C}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_AZIMUTHORIENTATION: &str = "{029123B4-8828-410B-B250-A0536595E5DC}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_BUTTONPRESSURE: &str = "{8B7FEFC4-96AA-4BFE-AC26-8A5F0BE07BF5}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_DEVICE_CONTACT_ID: &str = "{02585B91-049B-4750-9615-DF8948AB3C9C}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_FINGERCONTACTCONFIDENCE: &str = "{E706C804-57F0-4F00-8A0C-853D57789BE9}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_HEIGHT: &str = "{E61858D2-E447-4218-9D3F-18865C203DF4}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_NORMALPRESSURE: &str = "{7307502D-F9F4-4E18-B3F2-2CE1B1A3610C}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_PAKETSTATUS: &str = "{6E0E07BF-AFE7-4CF7-87D1-AF6446208418}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_PITCHROTATION: &str = "{7F7E57B7-BE37-4BE1-A356-7A84160E1893}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_ROLLROTATION: &str = "{5D5D5E56-6BA9-4C5B-9FB0-851C91714E56}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_SERIALNUMBER: &str = "{78A81B56-0935-4493-BAAE-00541A8A16C4}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_TANGENTPRESSURE: &str = "{6DA4488B-5244-41EC-905B-32D89AB80809}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_TIMERTICK: &str = "{436510C5-FED3-45D1-8B76-71D3EA7A829D}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_TWISTORIENTATION: &str = "{0D324960-13B2-41E4-ACE6-7AE9D43D2D3B}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_WIDTH: &str = "{BAABE94D-2712-48F5-BE9D-8F8B5EA0711A}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_X: &str = "{598A6A8F-52C0-4BA0-93AF-AF357411A561}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_XTILTORIENTATION: &str = "{A8D07B3A-8BF0-40B0-95A9-B80A6BB787BF}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_Y: &str = "{B53F9F75-04E0-4498-A7EE-C30DBB5A9011}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_YAWROTATION: &str = "{6A849980-7C3A-45B7-AA82-90A262950E89}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_YTILTORIENTATION: &str = "{0E932389-1D77-43AF-AC00-5B950D6D4B2D}";
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const STR_GUID_Z: &str = "{735ADB30-0EBB-4788-A0E4-0F316490055D}";
pub const SketchInk: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4029223041, data2: 59516, data3: 19975, data4: [151, 218, 160, 160, 55, 97, 229, 134] };
pub const StrokeBuilder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3893415655, data2: 28241, data3: 19632, data4: [170, 58, 11, 152, 91, 112, 218, 247] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_FLICKFALLBACKKEYS: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_FLICKS: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_PENBARRELFEEDBACK: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_PENTAPFEEDBACK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_PRESSANDHOLD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_SMOOTHSCROLLING: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_TOUCHSWITCH: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_TOUCHUIFORCEOFF: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_DISABLE_TOUCHUIFORCEON: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_ENABLE_FLICKLEARNINGMODE: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_ENABLE_FLICKSONCONTEXT: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TABLET_ENABLE_MULTITOUCHDATA: u32 = 16777216u32;
pub const TextInputPanel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4189161943, data2: 8843, data3: 20267, data4: [134, 80, 185, 127, 89, 224, 44, 140] };
pub const TipAutoCompleteClient: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2155617900, data2: 7424, data3: 17727, data4: [185, 32, 182, 27, 183, 205, 217, 151] };
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const WM_TABLET_ADDED: u32 = 712u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const WM_TABLET_DEFBASE: u32 = 704u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const WM_TABLET_DELETED: u32 = 713u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const WM_TABLET_FLICK: u32 = 715u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const WM_TABLET_MAXOFFSET: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const WM_TABLET_QUERYSYSTEMGESTURESTATUS: u32 = 716u32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type ALT_BREAKS = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ALT_BREAKS_SAME: ALT_BREAKS = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ALT_BREAKS_UNIQUE: ALT_BREAKS = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ALT_BREAKS_FULL: ALT_BREAKS = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type AppearanceConstants = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfFlat: AppearanceConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfThreeD: AppearanceConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type BorderStyleConstants = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfNoBorder: BorderStyleConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfFixedSingle: BorderStyleConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type CONFIDENCE_LEVEL = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CFL_STRONG: CONFIDENCE_LEVEL = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CFL_INTERMEDIATE: CONFIDENCE_LEVEL = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CFL_POOR: CONFIDENCE_LEVEL = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type CorrectionMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionMode_NotVisible: CorrectionMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionMode_PreInsertion: CorrectionMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionMode_PostInsertionCollapsed: CorrectionMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionMode_PostInsertionExpanded: CorrectionMode = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type CorrectionPosition = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionPosition_Auto: CorrectionPosition = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionPosition_Bottom: CorrectionPosition = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const CorrectionPosition_Top: CorrectionPosition = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_Ink = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IStrokes: DISPID_Ink = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IExtendedProperties: DISPID_Ink = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IGetBoundingBox: DISPID_Ink = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IDeleteStrokes: DISPID_Ink = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IDeleteStroke: DISPID_Ink = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IExtractStrokes: DISPID_Ink = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IExtractWithRectangle: DISPID_Ink = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IDirty: DISPID_Ink = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICustomStrokes: DISPID_Ink = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IClone: DISPID_Ink = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IHitTestCircle: DISPID_Ink = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IHitTestWithRectangle: DISPID_Ink = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IHitTestWithLasso: DISPID_Ink = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_INearestPoint: DISPID_Ink = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICreateStrokes: DISPID_Ink = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICreateStroke: DISPID_Ink = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IAddStrokesAtRectangle: DISPID_Ink = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IClip: DISPID_Ink = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISave: DISPID_Ink = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ILoad: DISPID_Ink = 20i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICreateStrokeFromPoints: DISPID_Ink = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IClipboardCopyWithRectangle: DISPID_Ink = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IClipboardCopy: DISPID_Ink = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICanPaste: DISPID_Ink = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IClipboardPaste: DISPID_Ink = 25i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCollector = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICEnabled: DISPID_InkCollector = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICHwnd: DISPID_InkCollector = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICPaint: DISPID_InkCollector = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICText: DISPID_InkCollector = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICDefaultDrawingAttributes: DISPID_InkCollector = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICRenderer: DISPID_InkCollector = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICInk: DISPID_InkCollector = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICAutoRedraw: DISPID_InkCollector = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICCollectingInk: DISPID_InkCollector = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSetEventInterest: DISPID_InkCollector = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICGetEventInterest: DISPID_InkCollector = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEditingMode: DISPID_InkCollector = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOSelection: DISPID_InkCollector = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOAttachMode: DISPID_InkCollector = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOHitTestSelection: DISPID_InkCollector = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IODraw: DISPID_InkCollector = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPPicture: DISPID_InkCollector = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPSizeMode: DISPID_InkCollector = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPBackColor: DISPID_InkCollector = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICCursors: DISPID_InkCollector = 20i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICMarginX: DISPID_InkCollector = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICMarginY: DISPID_InkCollector = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSetWindowInputRectangle: DISPID_InkCollector = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICGetWindowInputRectangle: DISPID_InkCollector = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICTablet: DISPID_InkCollector = 25i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSetAllTabletsMode: DISPID_InkCollector = 26i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSetSingleTabletIntegratedMode: DISPID_InkCollector = 27i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICCollectionMode: DISPID_InkCollector = 28i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSetGestureStatus: DISPID_InkCollector = 29i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICGetGestureStatus: DISPID_InkCollector = 30i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICDynamicRendering: DISPID_InkCollector = 31i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICDesiredPacketDescription: DISPID_InkCollector = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEraserMode: DISPID_InkCollector = 33i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEraserWidth: DISPID_InkCollector = 34i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICMouseIcon: DISPID_InkCollector = 35i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICMousePointer: DISPID_InkCollector = 36i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPInkEnabled: DISPID_InkCollector = 37i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSupportHighContrastInk: DISPID_InkCollector = 38i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOSupportHighContrastSelectionUI: DISPID_InkCollector = 39i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCollectorEvent = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICEStroke: DISPID_InkCollectorEvent = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICECursorDown: DISPID_InkCollectorEvent = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICENewPackets: DISPID_InkCollectorEvent = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICENewInAirPackets: DISPID_InkCollectorEvent = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICECursorButtonDown: DISPID_InkCollectorEvent = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICECursorButtonUp: DISPID_InkCollectorEvent = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICECursorInRange: DISPID_InkCollectorEvent = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICECursorOutOfRange: DISPID_InkCollectorEvent = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICESystemGesture: DISPID_InkCollectorEvent = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICEGesture: DISPID_InkCollectorEvent = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICETabletAdded: DISPID_InkCollectorEvent = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICETabletRemoved: DISPID_InkCollectorEvent = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEPainting: DISPID_InkCollectorEvent = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEPainted: DISPID_InkCollectorEvent = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOESelectionChanging: DISPID_InkCollectorEvent = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOESelectionChanged: DISPID_InkCollectorEvent = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOESelectionMoving: DISPID_InkCollectorEvent = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOESelectionMoved: DISPID_InkCollectorEvent = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOESelectionResizing: DISPID_InkCollectorEvent = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOESelectionResized: DISPID_InkCollectorEvent = 20i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEStrokesDeleting: DISPID_InkCollectorEvent = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IOEStrokesDeleted: DISPID_InkCollectorEvent = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEChangeUICues: DISPID_InkCollectorEvent = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEClick: DISPID_InkCollectorEvent = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEDblClick: DISPID_InkCollectorEvent = 25i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEInvalidated: DISPID_InkCollectorEvent = 26i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseDown: DISPID_InkCollectorEvent = 27i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseEnter: DISPID_InkCollectorEvent = 28i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseHover: DISPID_InkCollectorEvent = 29i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseLeave: DISPID_InkCollectorEvent = 30i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseMove: DISPID_InkCollectorEvent = 31i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseUp: DISPID_InkCollectorEvent = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEMouseWheel: DISPID_InkCollectorEvent = 33i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPESizeModeChanged: DISPID_InkCollectorEvent = 34i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEStyleChanged: DISPID_InkCollectorEvent = 35i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPESystemColorsChanged: DISPID_InkCollectorEvent = 36i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEKeyDown: DISPID_InkCollectorEvent = 37i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEKeyPress: DISPID_InkCollectorEvent = 38i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEKeyUp: DISPID_InkCollectorEvent = 39i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPEResize: DISPID_InkCollectorEvent = 40i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IPESizeChanged: DISPID_InkCollectorEvent = 41i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCursor = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsrName: DISPID_InkCursor = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsrId: DISPID_InkCursor = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsrDrawingAttributes: DISPID_InkCursor = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsrButtons: DISPID_InkCursor = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsrInverted: DISPID_InkCursor = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsrTablet: DISPID_InkCursor = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCursorButton = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICBName: DISPID_InkCursorButton = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICBId: DISPID_InkCursorButton = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICBState: DISPID_InkCursorButton = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCursorButtons = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICBs_NewEnum: DISPID_InkCursorButtons = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICBsItem: DISPID_InkCursorButtons = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICBsCount: DISPID_InkCursorButtons = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCursors = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICs_NewEnum: DISPID_InkCursors = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsItem: DISPID_InkCursors = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICsCount: DISPID_InkCursors = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkCustomStrokes = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSs_NewEnum: DISPID_InkCustomStrokes = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSsItem: DISPID_InkCustomStrokes = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSsCount: DISPID_InkCustomStrokes = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSsAdd: DISPID_InkCustomStrokes = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSsRemove: DISPID_InkCustomStrokes = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ICSsClear: DISPID_InkCustomStrokes = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkDivider = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivider_Strokes: DISPID_InkDivider = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivider_RecognizerContext: DISPID_InkDivider = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivider_LineHeight: DISPID_InkDivider = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivider_Divide: DISPID_InkDivider = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkDivisionResult = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionResult_Strokes: DISPID_InkDivisionResult = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionResult_ResultByType: DISPID_InkDivisionResult = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkDivisionUnit = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnit_Strokes: DISPID_InkDivisionUnit = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnit_DivisionType: DISPID_InkDivisionUnit = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnit_RecognizedString: DISPID_InkDivisionUnit = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnit_RotationTransform: DISPID_InkDivisionUnit = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkDivisionUnits = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnits_NewEnum: DISPID_InkDivisionUnits = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnits_Item: DISPID_InkDivisionUnits = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IInkDivisionUnits_Count: DISPID_InkDivisionUnits = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkDrawingAttributes = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAHeight: DISPID_InkDrawingAttributes = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAColor: DISPID_InkDrawingAttributes = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAWidth: DISPID_InkDrawingAttributes = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAFitToCurve: DISPID_InkDrawingAttributes = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAIgnorePressure: DISPID_InkDrawingAttributes = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAAntiAliased: DISPID_InkDrawingAttributes = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DATransparency: DISPID_InkDrawingAttributes = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DARasterOperation: DISPID_InkDrawingAttributes = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAPenTip: DISPID_InkDrawingAttributes = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAClone: DISPID_InkDrawingAttributes = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DAExtendedProperties: DISPID_InkDrawingAttributes = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkEdit = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Text: DISPID_InkEdit = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_TextRTF: DISPID_InkEdit = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Hwnd: DISPID_InkEdit = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DisableNoScroll: DISPID_InkEdit = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Locked: DISPID_InkEdit = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Enabled: DISPID_InkEdit = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_MaxLength: DISPID_InkEdit = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_MultiLine: DISPID_InkEdit = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ScrollBars: DISPID_InkEdit = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RTSelStart: DISPID_InkEdit = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RTSelLength: DISPID_InkEdit = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RTSelText: DISPID_InkEdit = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelAlignment: DISPID_InkEdit = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelBold: DISPID_InkEdit = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelCharOffset: DISPID_InkEdit = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelColor: DISPID_InkEdit = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelFontName: DISPID_InkEdit = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelFontSize: DISPID_InkEdit = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelItalic: DISPID_InkEdit = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelRTF: DISPID_InkEdit = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelUnderline: DISPID_InkEdit = 20i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DragIcon: DISPID_InkEdit = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Status: DISPID_InkEdit = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_UseMouseForInput: DISPID_InkEdit = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkMode: DISPID_InkEdit = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkInsertMode: DISPID_InkEdit = 25i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoTimeout: DISPID_InkEdit = 26i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_DrawAttr: DISPID_InkEdit = 27i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Recognizer: DISPID_InkEdit = 28i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Factoid: DISPID_InkEdit = 29i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelInk: DISPID_InkEdit = 30i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SelInksDisplayMode: DISPID_InkEdit = 31i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Recognize: DISPID_InkEdit = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_GetGestStatus: DISPID_InkEdit = 33i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SetGestStatus: DISPID_InkEdit = 34i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_Refresh: DISPID_InkEdit = 35i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkEditEvents = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeChange: DISPID_InkEditEvents = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeSelChange: DISPID_InkEditEvents = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeKeyDown: DISPID_InkEditEvents = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeKeyUp: DISPID_InkEditEvents = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeMouseUp: DISPID_InkEditEvents = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeMouseDown: DISPID_InkEditEvents = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeKeyPress: DISPID_InkEditEvents = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeDblClick: DISPID_InkEditEvents = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeClick: DISPID_InkEditEvents = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeMouseMove: DISPID_InkEditEvents = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeCursorDown: DISPID_InkEditEvents = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeStroke: DISPID_InkEditEvents = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeGesture: DISPID_InkEditEvents = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IeeRecognitionResult: DISPID_InkEditEvents = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkEvent = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEInkAdded: DISPID_InkEvent = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEInkDeleted: DISPID_InkEvent = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkExtendedProperties = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPs_NewEnum: DISPID_InkExtendedProperties = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPsItem: DISPID_InkExtendedProperties = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPsCount: DISPID_InkExtendedProperties = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPsAdd: DISPID_InkExtendedProperties = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPsRemove: DISPID_InkExtendedProperties = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPsClear: DISPID_InkExtendedProperties = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPsDoesPropertyExist: DISPID_InkExtendedProperties = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkExtendedProperty = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPGuid: DISPID_InkExtendedProperty = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IEPData: DISPID_InkExtendedProperty = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkGesture = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IGId: DISPID_InkGesture = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IGGetHotPoint: DISPID_InkGesture = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IGConfidence: DISPID_InkGesture = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecoAlternate = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_String: DISPID_InkRecoAlternate = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_LineNumber: DISPID_InkRecoAlternate = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_Baseline: DISPID_InkRecoAlternate = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_Midline: DISPID_InkRecoAlternate = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_Ascender: DISPID_InkRecoAlternate = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_Descender: DISPID_InkRecoAlternate = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_Confidence: DISPID_InkRecoAlternate = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_Strokes: DISPID_InkRecoAlternate = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_GetStrokesFromStrokeRanges: DISPID_InkRecoAlternate = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_GetStrokesFromTextRange: DISPID_InkRecoAlternate = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_GetTextRangeFromStrokes: DISPID_InkRecoAlternate = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_GetPropertyValue: DISPID_InkRecoAlternate = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_LineAlternates: DISPID_InkRecoAlternate = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_ConfidenceAlternates: DISPID_InkRecoAlternate = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecoAlternate_AlternatesWithConstantPropertyValues: DISPID_InkRecoAlternate = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecoContext = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Strokes: DISPID_InkRecoContext = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_CharacterAutoCompletionMode: DISPID_InkRecoContext = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Factoid: DISPID_InkRecoContext = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_WordList: DISPID_InkRecoContext = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Recognizer: DISPID_InkRecoContext = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Guide: DISPID_InkRecoContext = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Flags: DISPID_InkRecoContext = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_PrefixText: DISPID_InkRecoContext = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_SuffixText: DISPID_InkRecoContext = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_StopRecognition: DISPID_InkRecoContext = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Clone: DISPID_InkRecoContext = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_Recognize: DISPID_InkRecoContext = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_StopBackgroundRecognition: DISPID_InkRecoContext = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_EndInkInput: DISPID_InkRecoContext = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_BackgroundRecognize: DISPID_InkRecoContext = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_BackgroundRecognizeWithAlternates: DISPID_InkRecoContext = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx_IsStringSupported: DISPID_InkRecoContext = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecoContext2 = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecoCtx2_EnabledUnicodeRanges: DISPID_InkRecoContext2 = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognitionAlternates = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionAlternates_NewEnum: DISPID_InkRecognitionAlternates = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionAlternates_Item: DISPID_InkRecognitionAlternates = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionAlternates_Count: DISPID_InkRecognitionAlternates = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionAlternates_Strokes: DISPID_InkRecognitionAlternates = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognitionEvent = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRERecognitionWithAlternates: DISPID_InkRecognitionEvent = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRERecognition: DISPID_InkRecognitionEvent = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognitionResult = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_TopString: DISPID_InkRecognitionResult = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_TopAlternate: DISPID_InkRecognitionResult = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_Strokes: DISPID_InkRecognitionResult = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_TopConfidence: DISPID_InkRecognitionResult = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_AlternatesFromSelection: DISPID_InkRecognitionResult = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_ModifyTopAlternate: DISPID_InkRecognitionResult = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkRecognitionResult_SetResultOnStrokes: DISPID_InkRecognitionResult = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognizer = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoClsid: DISPID_InkRecognizer = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoName: DISPID_InkRecognizer = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoVendor: DISPID_InkRecognizer = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoCapabilities: DISPID_InkRecognizer = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoLanguageID: DISPID_InkRecognizer = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoPreferredPacketDescription: DISPID_InkRecognizer = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoCreateRecognizerContext: DISPID_InkRecognizer = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoSupportedProperties: DISPID_InkRecognizer = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognizer2 = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoId: DISPID_InkRecognizer2 = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_RecoUnicodeRanges: DISPID_InkRecognizer2 = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognizerGuide = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGWritingBox: DISPID_InkRecognizerGuide = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGDrawnBox: DISPID_InkRecognizerGuide = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGRows: DISPID_InkRecognizerGuide = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGColumns: DISPID_InkRecognizerGuide = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGMidline: DISPID_InkRecognizerGuide = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGGuideData: DISPID_InkRecognizerGuide = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRecognizers = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecos_NewEnum: DISPID_InkRecognizers = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecosItem: DISPID_InkRecognizers = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecosCount: DISPID_InkRecognizers = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRecosGetDefaultRecognizer: DISPID_InkRecognizers = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRectangle = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRTop: DISPID_InkRectangle = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRLeft: DISPID_InkRectangle = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRBottom: DISPID_InkRectangle = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRRight: DISPID_InkRectangle = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGetRectangle: DISPID_InkRectangle = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRSetRectangle: DISPID_InkRectangle = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRData: DISPID_InkRectangle = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkRenderer = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGetViewTransform: DISPID_InkRenderer = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRSetViewTransform: DISPID_InkRenderer = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRGetObjectTransform: DISPID_InkRenderer = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRSetObjectTransform: DISPID_InkRenderer = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRDraw: DISPID_InkRenderer = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRDrawStroke: DISPID_InkRenderer = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRPixelToInkSpace: DISPID_InkRenderer = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRInkSpaceToPixel: DISPID_InkRenderer = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRPixelToInkSpaceFromPoints: DISPID_InkRenderer = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRInkSpaceToPixelFromPoints: DISPID_InkRenderer = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRMeasure: DISPID_InkRenderer = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRMeasureStroke: DISPID_InkRenderer = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRMove: DISPID_InkRenderer = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRRotate: DISPID_InkRenderer = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IRScale: DISPID_InkRenderer = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkStrokeDisp = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDInkIndex: DISPID_InkStrokeDisp = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDID: DISPID_InkStrokeDisp = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetBoundingBox: DISPID_InkStrokeDisp = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDDrawingAttributes: DISPID_InkStrokeDisp = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDFindIntersections: DISPID_InkStrokeDisp = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetRectangleIntersections: DISPID_InkStrokeDisp = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDClip: DISPID_InkStrokeDisp = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDHitTestCircle: DISPID_InkStrokeDisp = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDNearestPoint: DISPID_InkStrokeDisp = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDSplit: DISPID_InkStrokeDisp = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDExtendedProperties: DISPID_InkStrokeDisp = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDInk: DISPID_InkStrokeDisp = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDBezierPoints: DISPID_InkStrokeDisp = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDPolylineCusps: DISPID_InkStrokeDisp = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDBezierCusps: DISPID_InkStrokeDisp = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDSelfIntersections: DISPID_InkStrokeDisp = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDPacketCount: DISPID_InkStrokeDisp = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDPacketSize: DISPID_InkStrokeDisp = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDPacketDescription: DISPID_InkStrokeDisp = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDDeleted: DISPID_InkStrokeDisp = 20i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetPacketDescriptionPropertyMetrics: DISPID_InkStrokeDisp = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetPoints: DISPID_InkStrokeDisp = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDSetPoints: DISPID_InkStrokeDisp = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetPacketData: DISPID_InkStrokeDisp = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetPacketValuesByProperty: DISPID_InkStrokeDisp = 25i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDSetPacketValuesByProperty: DISPID_InkStrokeDisp = 26i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDGetFlattenedBezierPoints: DISPID_InkStrokeDisp = 27i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDScaleToRectangle: DISPID_InkStrokeDisp = 28i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDTransform: DISPID_InkStrokeDisp = 29i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDMove: DISPID_InkStrokeDisp = 30i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDRotate: DISPID_InkStrokeDisp = 31i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDShear: DISPID_InkStrokeDisp = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISDScale: DISPID_InkStrokeDisp = 33i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkStrokes = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISs_NewEnum: DISPID_InkStrokes = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsItem: DISPID_InkStrokes = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsCount: DISPID_InkStrokes = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsValid: DISPID_InkStrokes = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsInk: DISPID_InkStrokes = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsAdd: DISPID_InkStrokes = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsAddStrokes: DISPID_InkStrokes = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsRemove: DISPID_InkStrokes = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsRemoveStrokes: DISPID_InkStrokes = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsToString: DISPID_InkStrokes = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsModifyDrawingAttributes: DISPID_InkStrokes = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsGetBoundingBox: DISPID_InkStrokes = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsScaleToRectangle: DISPID_InkStrokes = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsTransform: DISPID_InkStrokes = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsMove: DISPID_InkStrokes = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsRotate: DISPID_InkStrokes = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsShear: DISPID_InkStrokes = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsScale: DISPID_InkStrokes = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsClip: DISPID_InkStrokes = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsRecognitionResult: DISPID_InkStrokes = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ISsRemoveRecognitionResult: DISPID_InkStrokes = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkTablet = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITName: DISPID_InkTablet = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITPlugAndPlayId: DISPID_InkTablet = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITPropertyMetrics: DISPID_InkTablet = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITIsPacketPropertySupported: DISPID_InkTablet = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITMaximumInputRectangle: DISPID_InkTablet = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITHardwareCapabilities: DISPID_InkTablet = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkTablet2 = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IT2DeviceKind: DISPID_InkTablet2 = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkTablet3 = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IT3IsMultiTouch: DISPID_InkTablet3 = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_IT3MaximumCursors: DISPID_InkTablet3 = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkTablets = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITs_NewEnum: DISPID_InkTablets = -4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITsItem: DISPID_InkTablets = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITsDefaultTablet: DISPID_InkTablets = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITsCount: DISPID_InkTablets = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITsIsPacketPropertySupported: DISPID_InkTablets = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkTransform = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITReset: DISPID_InkTransform = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITTranslate: DISPID_InkTransform = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITRotate: DISPID_InkTransform = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITReflect: DISPID_InkTransform = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITShear: DISPID_InkTransform = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITScale: DISPID_InkTransform = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITeM11: DISPID_InkTransform = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITeM12: DISPID_InkTransform = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITeM21: DISPID_InkTransform = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITeM22: DISPID_InkTransform = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITeDx: DISPID_InkTransform = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITeDy: DISPID_InkTransform = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITGetTransform: DISPID_InkTransform = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITSetTransform: DISPID_InkTransform = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_ITData: DISPID_InkTransform = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkWordList = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkWordList_AddWord: DISPID_InkWordList = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkWordList_RemoveWord: DISPID_InkWordList = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkWordList_Merge: DISPID_InkWordList = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_InkWordList2 = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_InkWordList2_AddWords: DISPID_InkWordList2 = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_MathInputControlEvents = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_MICInsert: DISPID_MathInputControlEvents = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_MICClose: DISPID_MathInputControlEvents = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_MICPaint: DISPID_MathInputControlEvents = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_MICClear: DISPID_MathInputControlEvents = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_PenInputPanel = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPAttachedEditWindow: DISPID_PenInputPanel = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPFactoid: DISPID_PenInputPanel = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPCurrentPanel: DISPID_PenInputPanel = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPDefaultPanel: DISPID_PenInputPanel = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPVisible: DISPID_PenInputPanel = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPTop: DISPID_PenInputPanel = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPLeft: DISPID_PenInputPanel = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPWidth: DISPID_PenInputPanel = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPHeight: DISPID_PenInputPanel = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPMoveTo: DISPID_PenInputPanel = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPCommitPendingInput: DISPID_PenInputPanel = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPRefresh: DISPID_PenInputPanel = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPBusy: DISPID_PenInputPanel = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPVerticalOffset: DISPID_PenInputPanel = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPHorizontalOffset: DISPID_PenInputPanel = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPEnableTsf: DISPID_PenInputPanel = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPAutoShow: DISPID_PenInputPanel = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_PenInputPanelEvents = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPEVisibleChanged: DISPID_PenInputPanelEvents = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPEPanelChanged: DISPID_PenInputPanelEvents = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPEInputFailed: DISPID_PenInputPanelEvents = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_PIPEPanelMoving: DISPID_PenInputPanelEvents = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type DISPID_StrokeEvent = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SEStrokesAdded: DISPID_StrokeEvent = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DISPID_SEStrokesRemoved: DISPID_StrokeEvent = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type EventMask = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InPlaceStateChanging: EventMask = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InPlaceStateChanged: EventMask = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InPlaceSizeChanging: EventMask = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InPlaceSizeChanged: EventMask = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InputAreaChanging: EventMask = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InputAreaChanged: EventMask = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_CorrectionModeChanging: EventMask = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_CorrectionModeChanged: EventMask = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InPlaceVisibilityChanging: EventMask = 256i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_InPlaceVisibilityChanged: EventMask = 512i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_TextInserting: EventMask = 1024i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_TextInserted: EventMask = 2048i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const EventMask_All: EventMask = 4095i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type FLICKACTION_COMMANDCODE = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKACTION_COMMANDCODE_NULL: FLICKACTION_COMMANDCODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKACTION_COMMANDCODE_SCROLL: FLICKACTION_COMMANDCODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKACTION_COMMANDCODE_APPCOMMAND: FLICKACTION_COMMANDCODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKACTION_COMMANDCODE_CUSTOMKEY: FLICKACTION_COMMANDCODE = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKACTION_COMMANDCODE_KEYMODIFIER: FLICKACTION_COMMANDCODE = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type FLICKDIRECTION = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_MIN: FLICKDIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_RIGHT: FLICKDIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_UPRIGHT: FLICKDIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_UP: FLICKDIRECTION = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_UPLEFT: FLICKDIRECTION = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_LEFT: FLICKDIRECTION = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_DOWNLEFT: FLICKDIRECTION = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_DOWN: FLICKDIRECTION = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_DOWNRIGHT: FLICKDIRECTION = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKDIRECTION_INVALID: FLICKDIRECTION = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type FLICKMODE = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKMODE_MIN: FLICKMODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKMODE_OFF: FLICKMODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKMODE_ON: FLICKMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKMODE_LEARNING: FLICKMODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKMODE_MAX: FLICKMODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const FLICKMODE_DEFAULT: FLICKMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type GET_DANDIDATE_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TCF_ALLOW_RECOGNITION: GET_DANDIDATE_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TCF_FORCE_RECOGNITION: GET_DANDIDATE_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type INK_METRIC_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMF_FONT_SELECTED_IN_HDC: INK_METRIC_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMF_ITALIC: INK_METRIC_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMF_BOLD: INK_METRIC_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InPlaceDirection = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlaceDirection_Auto: InPlaceDirection = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlaceDirection_Bottom: InPlaceDirection = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlaceDirection_Top: InPlaceDirection = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InPlaceState = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlaceState_Auto: InPlaceState = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlaceState_HoverTarget: InPlaceState = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlaceState_Expanded: InPlaceState = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkApplicationGesture = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_AllGestures: InkApplicationGesture = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_NoGesture: InkApplicationGesture = 61440i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Scratchout: InkApplicationGesture = 61441i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Triangle: InkApplicationGesture = 61442i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Square: InkApplicationGesture = 61443i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Star: InkApplicationGesture = 61444i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Check: InkApplicationGesture = 61445i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Curlicue: InkApplicationGesture = 61456i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DoubleCurlicue: InkApplicationGesture = 61457i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Circle: InkApplicationGesture = 61472i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DoubleCircle: InkApplicationGesture = 61473i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_SemiCircleLeft: InkApplicationGesture = 61480i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_SemiCircleRight: InkApplicationGesture = 61481i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ChevronUp: InkApplicationGesture = 61488i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ChevronDown: InkApplicationGesture = 61489i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ChevronLeft: InkApplicationGesture = 61490i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ChevronRight: InkApplicationGesture = 61491i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ArrowUp: InkApplicationGesture = 61496i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ArrowDown: InkApplicationGesture = 61497i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ArrowLeft: InkApplicationGesture = 61498i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_ArrowRight: InkApplicationGesture = 61499i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Up: InkApplicationGesture = 61528i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Down: InkApplicationGesture = 61529i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Left: InkApplicationGesture = 61530i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Right: InkApplicationGesture = 61531i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_UpDown: InkApplicationGesture = 61536i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DownUp: InkApplicationGesture = 61537i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_LeftRight: InkApplicationGesture = 61538i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_RightLeft: InkApplicationGesture = 61539i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_UpLeftLong: InkApplicationGesture = 61540i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_UpRightLong: InkApplicationGesture = 61541i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DownLeftLong: InkApplicationGesture = 61542i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DownRightLong: InkApplicationGesture = 61543i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_UpLeft: InkApplicationGesture = 61544i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_UpRight: InkApplicationGesture = 61545i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DownLeft: InkApplicationGesture = 61546i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DownRight: InkApplicationGesture = 61547i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_LeftUp: InkApplicationGesture = 61548i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_LeftDown: InkApplicationGesture = 61549i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_RightUp: InkApplicationGesture = 61550i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_RightDown: InkApplicationGesture = 61551i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Exclamation: InkApplicationGesture = 61604i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_Tap: InkApplicationGesture = 61680i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IAG_DoubleTap: InkApplicationGesture = 61681i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkBoundingBoxMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IBBM_Default: InkBoundingBoxMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IBBM_NoCurveFit: InkBoundingBoxMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IBBM_CurveFit: InkBoundingBoxMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IBBM_PointsOnly: InkBoundingBoxMode = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IBBM_Union: InkBoundingBoxMode = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkClipboardFormats = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_None: InkClipboardFormats = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_InkSerializedFormat: InkClipboardFormats = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_SketchInk: InkClipboardFormats = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_TextInk: InkClipboardFormats = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_EnhancedMetafile: InkClipboardFormats = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_Metafile: InkClipboardFormats = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_Bitmap: InkClipboardFormats = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_PasteMask: InkClipboardFormats = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_CopyMask: InkClipboardFormats = 127i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICF_Default: InkClipboardFormats = 127i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkClipboardModes = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICB_Copy: InkClipboardModes = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICB_Cut: InkClipboardModes = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICB_ExtractOnly: InkClipboardModes = 48i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICB_DelayedCopy: InkClipboardModes = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICB_Default: InkClipboardModes = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkCollectionMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICM_InkOnly: InkCollectionMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICM_GestureOnly: InkCollectionMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICM_InkAndGesture: InkCollectionMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkCollectorEventInterest = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_DefaultEvents: InkCollectorEventInterest = -1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_CursorDown: InkCollectorEventInterest = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_Stroke: InkCollectorEventInterest = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_NewPackets: InkCollectorEventInterest = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_NewInAirPackets: InkCollectorEventInterest = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_CursorButtonDown: InkCollectorEventInterest = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_CursorButtonUp: InkCollectorEventInterest = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_CursorInRange: InkCollectorEventInterest = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_CursorOutOfRange: InkCollectorEventInterest = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_SystemGesture: InkCollectorEventInterest = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_TabletAdded: InkCollectorEventInterest = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_TabletRemoved: InkCollectorEventInterest = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_MouseDown: InkCollectorEventInterest = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_MouseMove: InkCollectorEventInterest = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_MouseUp: InkCollectorEventInterest = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_MouseWheel: InkCollectorEventInterest = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_DblClick: InkCollectorEventInterest = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICEI_AllEvents: InkCollectorEventInterest = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkCursorButtonState = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICBS_Unavailable: InkCursorButtonState = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICBS_Up: InkCursorButtonState = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ICBS_Down: InkCursorButtonState = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkDisplayMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IDM_Ink: InkDisplayMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IDM_Text: InkDisplayMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkDivisionType = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IDT_Segment: InkDivisionType = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IDT_Line: InkDivisionType = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IDT_Paragraph: InkDivisionType = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IDT_Drawing: InkDivisionType = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkEditStatus = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IES_Idle: InkEditStatus = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IES_Collecting: InkEditStatus = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IES_Recognizing: InkEditStatus = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkExtractFlags = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEF_CopyFromOriginal: InkExtractFlags = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEF_RemoveFromOriginal: InkExtractFlags = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEF_Default: InkExtractFlags = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkInsertMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEM_InsertText: InkInsertMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEM_InsertInk: InkInsertMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEM_Disabled: InkMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEM_Ink: InkMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IEM_InkAndGesture: InkMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkMouseButton = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMF_Left: InkMouseButton = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMF_Right: InkMouseButton = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMF_Middle: InkMouseButton = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkMousePointer = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Default: InkMousePointer = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Arrow: InkMousePointer = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Crosshair: InkMousePointer = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Ibeam: InkMousePointer = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_SizeNESW: InkMousePointer = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_SizeNS: InkMousePointer = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_SizeNWSE: InkMousePointer = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_SizeWE: InkMousePointer = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_UpArrow: InkMousePointer = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Hourglass: InkMousePointer = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_NoDrop: InkMousePointer = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_ArrowHourglass: InkMousePointer = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_ArrowQuestion: InkMousePointer = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_SizeAll: InkMousePointer = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Hand: InkMousePointer = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IMP_Custom: InkMousePointer = 99i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkOverlayAttachMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOAM_Behind: InkOverlayAttachMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOAM_InFront: InkOverlayAttachMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkOverlayEditingMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOEM_Ink: InkOverlayEditingMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOEM_Delete: InkOverlayEditingMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOEM_Select: InkOverlayEditingMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkOverlayEraserMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOERM_StrokeErase: InkOverlayEraserMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IOERM_PointErase: InkOverlayEraserMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkPenTip = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPT_Ball: InkPenTip = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPT_Rectangle: InkPenTip = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkPersistenceCompressionMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPCM_Default: InkPersistenceCompressionMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPCM_MaximumCompression: InkPersistenceCompressionMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPCM_NoCompression: InkPersistenceCompressionMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkPersistenceFormat = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPF_InkSerializedFormat: InkPersistenceFormat = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPF_Base64InkSerializedFormat: InkPersistenceFormat = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPF_GIF: InkPersistenceFormat = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPF_Base64GIF: InkPersistenceFormat = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkPictureSizeMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPSM_AutoSize: InkPictureSizeMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPSM_CenterImage: InkPictureSizeMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPSM_Normal: InkPictureSizeMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IPSM_StretchImage: InkPictureSizeMode = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRasterOperation = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_Black: InkRasterOperation = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_NotMergePen: InkRasterOperation = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_MaskNotPen: InkRasterOperation = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_NotCopyPen: InkRasterOperation = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_MaskPenNot: InkRasterOperation = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_Not: InkRasterOperation = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_XOrPen: InkRasterOperation = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_NotMaskPen: InkRasterOperation = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_MaskPen: InkRasterOperation = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_NotXOrPen: InkRasterOperation = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_NoOperation: InkRasterOperation = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_MergeNotPen: InkRasterOperation = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_CopyPen: InkRasterOperation = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_MergePenNot: InkRasterOperation = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_MergePen: InkRasterOperation = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRO_White: InkRasterOperation = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRecognitionAlternatesSelection = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRAS_Start: InkRecognitionAlternatesSelection = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRAS_DefaultCount: InkRecognitionAlternatesSelection = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRAS_All: InkRecognitionAlternatesSelection = -1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRecognitionConfidence = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Strong: InkRecognitionConfidence = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Intermediate: InkRecognitionConfidence = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Poor: InkRecognitionConfidence = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRecognitionModes = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_None: InkRecognitionModes = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_WordModeOnly: InkRecognitionModes = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_Coerce: InkRecognitionModes = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_TopInkBreaksOnly: InkRecognitionModes = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_PrefixOk: InkRecognitionModes = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_LineMode: InkRecognitionModes = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_DisablePersonalization: InkRecognitionModes = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_AutoSpace: InkRecognitionModes = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRM_Max: InkRecognitionModes = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRecognitionStatus = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_NoError: InkRecognitionStatus = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_Interrupted: InkRecognitionStatus = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_ProcessFailed: InkRecognitionStatus = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_InkAddedFailed: InkRecognitionStatus = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetAutoCompletionModeFailed: InkRecognitionStatus = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetStrokesFailed: InkRecognitionStatus = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetGuideFailed: InkRecognitionStatus = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetFlagsFailed: InkRecognitionStatus = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetFactoidFailed: InkRecognitionStatus = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetPrefixSuffixFailed: InkRecognitionStatus = 256i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRS_SetWordListFailed: InkRecognitionStatus = 512i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRecognizerCapabilities = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_DontCare: InkRecognizerCapabilities = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Object: InkRecognizerCapabilities = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_FreeInput: InkRecognizerCapabilities = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_LinedInput: InkRecognizerCapabilities = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_BoxedInput: InkRecognizerCapabilities = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_CharacterAutoCompletionInput: InkRecognizerCapabilities = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_RightAndDown: InkRecognizerCapabilities = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_LeftAndDown: InkRecognizerCapabilities = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_DownAndLeft: InkRecognizerCapabilities = 256i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_DownAndRight: InkRecognizerCapabilities = 512i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_ArbitraryAngle: InkRecognizerCapabilities = 1024i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Lattice: InkRecognizerCapabilities = 2048i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_AdviseInkChange: InkRecognizerCapabilities = 4096i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_StrokeReorder: InkRecognizerCapabilities = 8192i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Personalizable: InkRecognizerCapabilities = 16384i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_PrefersArbitraryAngle: InkRecognizerCapabilities = 32768i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_PrefersParagraphBreaking: InkRecognizerCapabilities = 65536i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_PrefersSegmentation: InkRecognizerCapabilities = 131072i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Cursive: InkRecognizerCapabilities = 262144i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_TextPrediction: InkRecognizerCapabilities = 524288i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Alpha: InkRecognizerCapabilities = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRC_Beta: InkRecognizerCapabilities = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkRecognizerCharacterAutoCompletionMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRCACM_Full: InkRecognizerCharacterAutoCompletionMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRCACM_Prefix: InkRecognizerCharacterAutoCompletionMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IRCACM_Random: InkRecognizerCharacterAutoCompletionMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkSelectionConstants = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISC_FirstElement: InkSelectionConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISC_AllElements: InkSelectionConstants = -1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkShiftKeyModifierFlags = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IKM_Shift: InkShiftKeyModifierFlags = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IKM_Control: InkShiftKeyModifierFlags = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const IKM_Alt: InkShiftKeyModifierFlags = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InkSystemGesture = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_Tap: InkSystemGesture = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_DoubleTap: InkSystemGesture = 17i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_RightTap: InkSystemGesture = 18i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_Drag: InkSystemGesture = 19i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_RightDrag: InkSystemGesture = 20i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_HoldEnter: InkSystemGesture = 21i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_HoldLeave: InkSystemGesture = 22i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_HoverEnter: InkSystemGesture = 23i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_HoverLeave: InkSystemGesture = 24i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const ISG_Flick: InkSystemGesture = 31i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type InteractionMode = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InteractionMode_InPlace: InteractionMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InteractionMode_Floating: InteractionMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InteractionMode_DockedTop: InteractionMode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InteractionMode_DockedBottom: InteractionMode = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type KEYMODIFIER = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const KEYMODIFIER_CONTROL: KEYMODIFIER = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const KEYMODIFIER_MENU: KEYMODIFIER = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const KEYMODIFIER_SHIFT: KEYMODIFIER = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const KEYMODIFIER_WIN: KEYMODIFIER = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const KEYMODIFIER_ALTGR: KEYMODIFIER = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const KEYMODIFIER_EXT: KEYMODIFIER = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type LINE_METRICS = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const LM_BASELINE: LINE_METRICS = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const LM_MIDLINE: LINE_METRICS = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const LM_ASCENDER: LINE_METRICS = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const LM_DESCENDER: LINE_METRICS = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type MICUIELEMENT = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_WRITE: MICUIELEMENT = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_ERASE: MICUIELEMENT = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_CORRECT: MICUIELEMENT = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_CLEAR: MICUIELEMENT = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_UNDO: MICUIELEMENT = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_REDO: MICUIELEMENT = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_INSERT: MICUIELEMENT = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_BUTTON_CANCEL: MICUIELEMENT = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_INKPANEL_BACKGROUND: MICUIELEMENT = 256i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENT_RESULTPANEL_BACKGROUND: MICUIELEMENT = 512i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type MICUIELEMENTSTATE = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENTSTATE_NORMAL: MICUIELEMENTSTATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENTSTATE_HOT: MICUIELEMENTSTATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENTSTATE_PRESSED: MICUIELEMENTSTATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MICUIELEMENTSTATE_DISABLED: MICUIELEMENTSTATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type MouseButton = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const NO_BUTTON: MouseButton = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const LEFT_BUTTON: MouseButton = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RIGHT_BUTTON: MouseButton = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const MIDDLE_BUTTON: MouseButton = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type PROPERTY_UNITS = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_DEFAULT: PROPERTY_UNITS = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_INCHES: PROPERTY_UNITS = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_CENTIMETERS: PROPERTY_UNITS = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_DEGREES: PROPERTY_UNITS = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_RADIANS: PROPERTY_UNITS = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_SECONDS: PROPERTY_UNITS = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_POUNDS: PROPERTY_UNITS = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_GRAMS: PROPERTY_UNITS = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_SILINEAR: PROPERTY_UNITS = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_SIROTATION: PROPERTY_UNITS = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_ENGLINEAR: PROPERTY_UNITS = 10i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_ENGROTATION: PROPERTY_UNITS = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_SLUGS: PROPERTY_UNITS = 12i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_KELVIN: PROPERTY_UNITS = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_FAHRENHEIT: PROPERTY_UNITS = 14i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_AMPERE: PROPERTY_UNITS = 15i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PROPERTY_UNITS_CANDELA: PROPERTY_UNITS = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type PanelInputArea = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PanelInputArea_Auto: PanelInputArea = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PanelInputArea_Keyboard: PanelInputArea = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PanelInputArea_WritingPad: PanelInputArea = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PanelInputArea_CharacterPad: PanelInputArea = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type PanelType = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PT_Default: PanelType = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PT_Inactive: PanelType = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PT_Handwriting: PanelType = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const PT_Keyboard: PanelType = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type RECO_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECO_TYPE_WSTRING: RECO_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RECO_TYPE_WCHAR: RECO_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type RealTimeStylusDataInterest = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_AllData: RealTimeStylusDataInterest = -1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_None: RealTimeStylusDataInterest = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_Error: RealTimeStylusDataInterest = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_RealTimeStylusEnabled: RealTimeStylusDataInterest = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_RealTimeStylusDisabled: RealTimeStylusDataInterest = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusNew: RealTimeStylusDataInterest = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusInRange: RealTimeStylusDataInterest = 16i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_InAirPackets: RealTimeStylusDataInterest = 32i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusOutOfRange: RealTimeStylusDataInterest = 64i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusDown: RealTimeStylusDataInterest = 128i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_Packets: RealTimeStylusDataInterest = 256i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusUp: RealTimeStylusDataInterest = 512i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusButtonUp: RealTimeStylusDataInterest = 1024i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_StylusButtonDown: RealTimeStylusDataInterest = 2048i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_SystemEvents: RealTimeStylusDataInterest = 4096i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_TabletAdded: RealTimeStylusDataInterest = 8192i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_TabletRemoved: RealTimeStylusDataInterest = 16384i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_CustomStylusDataAdded: RealTimeStylusDataInterest = 32768i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_UpdateMapping: RealTimeStylusDataInterest = 65536i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSDI_DefaultEvents: RealTimeStylusDataInterest = 37766i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type RealTimeStylusLockType = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSLT_ObjLock: RealTimeStylusLockType = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSLT_SyncEventLock: RealTimeStylusLockType = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSLT_AsyncEventLock: RealTimeStylusLockType = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSLT_ExcludeCallback: RealTimeStylusLockType = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSLT_SyncObjLock: RealTimeStylusLockType = 11i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const RTSLT_AsyncObjLock: RealTimeStylusLockType = 13i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type SCROLLDIRECTION = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SCROLLDIRECTION_UP: SCROLLDIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SCROLLDIRECTION_DOWN: SCROLLDIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type ScrollBarsConstants = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfNone: ScrollBarsConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfHorizontal: ScrollBarsConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfVertical: ScrollBarsConstants = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfBoth: ScrollBarsConstants = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type SelAlignmentConstants = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfLeft: SelAlignmentConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfRight: SelAlignmentConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const rtfCenter: SelAlignmentConstants = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type SelectionHitResult = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_None: SelectionHitResult = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_NW: SelectionHitResult = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_SE: SelectionHitResult = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_NE: SelectionHitResult = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_SW: SelectionHitResult = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_E: SelectionHitResult = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_W: SelectionHitResult = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_N: SelectionHitResult = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_S: SelectionHitResult = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SHR_Selection: SelectionHitResult = 9i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type StylusQueue = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const SyncStylusQueue: StylusQueue = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const AsyncStylusQueueImmediate: StylusQueue = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const AsyncStylusQueue: StylusQueue = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type TabletDeviceKind = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TDK_Mouse: TabletDeviceKind = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TDK_Pen: TabletDeviceKind = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TDK_Touch: TabletDeviceKind = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type TabletHardwareCapabilities = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const THWC_Integrated: TabletHardwareCapabilities = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const THWC_CursorMustTouch: TabletHardwareCapabilities = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const THWC_HardProximity: TabletHardwareCapabilities = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const THWC_CursorsHavePhysicalIds: TabletHardwareCapabilities = 8i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type TabletPropertyMetricUnit = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Default: TabletPropertyMetricUnit = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Inches: TabletPropertyMetricUnit = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Centimeters: TabletPropertyMetricUnit = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Degrees: TabletPropertyMetricUnit = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Radians: TabletPropertyMetricUnit = 4i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Seconds: TabletPropertyMetricUnit = 5i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Pounds: TabletPropertyMetricUnit = 6i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const TPMU_Grams: TabletPropertyMetricUnit = 7i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type VisualState = i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const InPlace: VisualState = 0i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const Floating: VisualState = 1i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DockedTop: VisualState = 2i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const DockedBottom: VisualState = 3i32;
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub const Closed: VisualState = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct CHARACTER_RANGE {
    pub wcLow: u16,
    pub cChars: u16,
}
impl ::core::marker::Copy for CHARACTER_RANGE {}
impl ::core::clone::Clone for CHARACTER_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct DYNAMIC_RENDERER_CACHED_DATA {
    pub strokeId: i32,
    pub dynamicRenderer: IDynamicRenderer,
}
impl ::core::marker::Copy for DYNAMIC_RENDERER_CACHED_DATA {}
impl ::core::clone::Clone for DYNAMIC_RENDERER_CACHED_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct FLICK_DATA {
    pub _bitfield: i32,
}
impl ::core::marker::Copy for FLICK_DATA {}
impl ::core::clone::Clone for FLICK_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct FLICK_POINT {
    pub _bitfield: i32,
}
impl ::core::marker::Copy for FLICK_POINT {}
impl ::core::clone::Clone for FLICK_POINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct GESTURE_DATA {
    pub gestureId: i32,
    pub recoConfidence: i32,
    pub strokeCount: i32,
}
impl ::core::marker::Copy for GESTURE_DATA {}
impl ::core::clone::Clone for GESTURE_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HRECOALT = isize;
pub type HRECOCONTEXT = isize;
pub type HRECOGNIZER = isize;
pub type HRECOLATTICE = isize;
pub type HRECOWORDLIST = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Controls\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Controls"))]
pub struct IEC_GESTUREINFO {
    pub nmhdr: super::Controls::NMHDR,
    pub Cursor: IInkCursor,
    pub Strokes: IInkStrokes,
    pub Gestures: super::super::System::Com::VARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Controls"))]
impl ::core::marker::Copy for IEC_GESTUREINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Controls"))]
impl ::core::clone::Clone for IEC_GESTUREINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Controls\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_UI_Controls"))]
pub struct IEC_RECOGNITIONRESULTINFO {
    pub nmhdr: super::Controls::NMHDR,
    pub RecognitionResult: IInkRecognitionResult,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_UI_Controls"))]
impl ::core::marker::Copy for IEC_RECOGNITIONRESULTINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_UI_Controls"))]
impl ::core::clone::Clone for IEC_RECOGNITIONRESULTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Controls\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_UI_Controls"))]
pub struct IEC_STROKEINFO {
    pub nmhdr: super::Controls::NMHDR,
    pub Cursor: IInkCursor,
    pub Stroke: IInkStrokeDisp,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_UI_Controls"))]
impl ::core::marker::Copy for IEC_STROKEINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_UI_Controls"))]
impl ::core::clone::Clone for IEC_STROKEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct INKMETRIC {
    pub iHeight: i32,
    pub iFontAscent: i32,
    pub iFontDescent: i32,
    pub dwFlags: u32,
    pub color: super::super::Foundation::COLORREF,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for INKMETRIC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for INKMETRIC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct InkRecoGuide {
    pub rectWritingBox: super::super::Foundation::RECT,
    pub rectDrawnBox: super::super::Foundation::RECT,
    pub cRows: i32,
    pub cColumns: i32,
    pub midline: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for InkRecoGuide {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for InkRecoGuide {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LATTICE_METRICS {
    pub lsBaseline: LINE_SEGMENT,
    pub iMidlineOffset: i16,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LATTICE_METRICS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LATTICE_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LINE_SEGMENT {
    pub PtA: super::super::Foundation::POINT,
    pub PtB: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LINE_SEGMENT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LINE_SEGMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct PACKET_DESCRIPTION {
    pub cbPacketSize: u32,
    pub cPacketProperties: u32,
    pub pPacketProperties: *mut PACKET_PROPERTY,
    pub cButtons: u32,
    pub pguidButtons: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for PACKET_DESCRIPTION {}
impl ::core::clone::Clone for PACKET_DESCRIPTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct PACKET_PROPERTY {
    pub guid: ::windows_sys::core::GUID,
    pub PropertyMetrics: PROPERTY_METRICS,
}
impl ::core::marker::Copy for PACKET_PROPERTY {}
impl ::core::clone::Clone for PACKET_PROPERTY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct PROPERTY_METRICS {
    pub nLogicalMin: i32,
    pub nLogicalMax: i32,
    pub Units: PROPERTY_UNITS,
    pub fResolution: f32,
}
impl ::core::marker::Copy for PROPERTY_METRICS {}
impl ::core::clone::Clone for PROPERTY_METRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_ATTRS {
    pub dwRecoCapabilityFlags: u32,
    pub awcVendorName: [u16; 32],
    pub awcFriendlyName: [u16; 64],
    pub awLanguageId: [u16; 64],
}
impl ::core::marker::Copy for RECO_ATTRS {}
impl ::core::clone::Clone for RECO_ATTRS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_GUIDE {
    pub xOrigin: i32,
    pub yOrigin: i32,
    pub cxBox: i32,
    pub cyBox: i32,
    pub cxBase: i32,
    pub cyBase: i32,
    pub cHorzBox: i32,
    pub cVertBox: i32,
    pub cyMid: i32,
}
impl ::core::marker::Copy for RECO_GUIDE {}
impl ::core::clone::Clone for RECO_GUIDE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_LATTICE {
    pub ulColumnCount: u32,
    pub pLatticeColumns: *mut RECO_LATTICE_COLUMN,
    pub ulPropertyCount: u32,
    pub pGuidProperties: *mut ::windows_sys::core::GUID,
    pub ulBestResultColumnCount: u32,
    pub pulBestResultColumns: *mut u32,
    pub pulBestResultIndexes: *mut u32,
}
impl ::core::marker::Copy for RECO_LATTICE {}
impl ::core::clone::Clone for RECO_LATTICE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_LATTICE_COLUMN {
    pub key: u32,
    pub cpProp: RECO_LATTICE_PROPERTIES,
    pub cStrokes: u32,
    pub pStrokes: *mut u32,
    pub cLatticeElements: u32,
    pub pLatticeElements: *mut RECO_LATTICE_ELEMENT,
}
impl ::core::marker::Copy for RECO_LATTICE_COLUMN {}
impl ::core::clone::Clone for RECO_LATTICE_COLUMN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_LATTICE_ELEMENT {
    pub score: i32,
    pub r#type: u16,
    pub pData: *mut u8,
    pub ulNextColumn: u32,
    pub ulStrokeNumber: u32,
    pub epProp: RECO_LATTICE_PROPERTIES,
}
impl ::core::marker::Copy for RECO_LATTICE_ELEMENT {}
impl ::core::clone::Clone for RECO_LATTICE_ELEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_LATTICE_PROPERTIES {
    pub cProperties: u32,
    pub apProps: *mut *mut RECO_LATTICE_PROPERTY,
}
impl ::core::marker::Copy for RECO_LATTICE_PROPERTIES {}
impl ::core::clone::Clone for RECO_LATTICE_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_LATTICE_PROPERTY {
    pub guidProperty: ::windows_sys::core::GUID,
    pub cbPropertyValue: u16,
    pub pPropertyValue: *mut u8,
}
impl ::core::marker::Copy for RECO_LATTICE_PROPERTY {}
impl ::core::clone::Clone for RECO_LATTICE_PROPERTY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct RECO_RANGE {
    pub iwcBegin: u32,
    pub cCount: u32,
}
impl ::core::marker::Copy for RECO_RANGE {}
impl ::core::clone::Clone for RECO_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct STROKE_RANGE {
    pub iStrokeBegin: u32,
    pub iStrokeEnd: u32,
}
impl ::core::marker::Copy for STROKE_RANGE {}
impl ::core::clone::Clone for STROKE_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub struct SYSTEM_EVENT_DATA {
    pub bModifier: u8,
    pub wKey: u16,
    pub xPos: i32,
    pub yPos: i32,
    pub bCursorMode: u8,
    pub dwButtonState: u32,
}
impl ::core::marker::Copy for SYSTEM_EVENT_DATA {}
impl ::core::clone::Clone for SYSTEM_EVENT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct StylusInfo {
    pub tcid: u32,
    pub cid: u32,
    pub bIsInvertedCursor: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for StylusInfo {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for StylusInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_TabletPC\"`*"]
pub type PfnRecoCallback = ::core::option::Option<unsafe extern "system" fn(param0: u32, param1: *mut u8, param2: HRECOCONTEXT) -> ::windows_sys::core::HRESULT>;
