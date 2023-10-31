#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DoMsCtfMonitor(dwflags: u32, heventforservicestop: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
    pub fn InitLocalMsCtfMonitor(dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
    pub fn UninitLocalMsCtfMonitor() -> ::windows_sys::core::HRESULT;
}
pub type IAccClientDocMgr = *mut ::core::ffi::c_void;
pub type IAccDictionary = *mut ::core::ffi::c_void;
pub type IAccServerDocMgr = *mut ::core::ffi::c_void;
pub type IAccStore = *mut ::core::ffi::c_void;
pub type IAnchor = *mut ::core::ffi::c_void;
pub type IClonableWrapper = *mut ::core::ffi::c_void;
pub type ICoCreateLocally = *mut ::core::ffi::c_void;
pub type ICoCreatedLocally = *mut ::core::ffi::c_void;
pub type IDocWrap = *mut ::core::ffi::c_void;
pub type IEnumITfCompositionView = *mut ::core::ffi::c_void;
pub type IEnumSpeechCommands = *mut ::core::ffi::c_void;
pub type IEnumTfCandidates = *mut ::core::ffi::c_void;
pub type IEnumTfContextViews = *mut ::core::ffi::c_void;
pub type IEnumTfContexts = *mut ::core::ffi::c_void;
pub type IEnumTfDisplayAttributeInfo = *mut ::core::ffi::c_void;
pub type IEnumTfDocumentMgrs = *mut ::core::ffi::c_void;
pub type IEnumTfFunctionProviders = *mut ::core::ffi::c_void;
pub type IEnumTfInputProcessorProfiles = *mut ::core::ffi::c_void;
pub type IEnumTfLangBarItems = *mut ::core::ffi::c_void;
pub type IEnumTfLanguageProfiles = *mut ::core::ffi::c_void;
pub type IEnumTfLatticeElements = *mut ::core::ffi::c_void;
pub type IEnumTfProperties = *mut ::core::ffi::c_void;
pub type IEnumTfPropertyValue = *mut ::core::ffi::c_void;
pub type IEnumTfRanges = *mut ::core::ffi::c_void;
pub type IEnumTfUIElements = *mut ::core::ffi::c_void;
pub type IInternalDocWrap = *mut ::core::ffi::c_void;
pub type ISpeechCommandProvider = *mut ::core::ffi::c_void;
pub type ITextStoreACP = *mut ::core::ffi::c_void;
pub type ITextStoreACP2 = *mut ::core::ffi::c_void;
pub type ITextStoreACPEx = *mut ::core::ffi::c_void;
pub type ITextStoreACPServices = *mut ::core::ffi::c_void;
pub type ITextStoreACPSink = *mut ::core::ffi::c_void;
pub type ITextStoreACPSinkEx = *mut ::core::ffi::c_void;
pub type ITextStoreAnchor = *mut ::core::ffi::c_void;
pub type ITextStoreAnchorEx = *mut ::core::ffi::c_void;
pub type ITextStoreAnchorSink = *mut ::core::ffi::c_void;
pub type ITextStoreSinkAnchorEx = *mut ::core::ffi::c_void;
pub type ITfActiveLanguageProfileNotifySink = *mut ::core::ffi::c_void;
pub type ITfCandidateList = *mut ::core::ffi::c_void;
pub type ITfCandidateListUIElement = *mut ::core::ffi::c_void;
pub type ITfCandidateListUIElementBehavior = *mut ::core::ffi::c_void;
pub type ITfCandidateString = *mut ::core::ffi::c_void;
pub type ITfCategoryMgr = *mut ::core::ffi::c_void;
pub type ITfCleanupContextDurationSink = *mut ::core::ffi::c_void;
pub type ITfCleanupContextSink = *mut ::core::ffi::c_void;
pub type ITfClientId = *mut ::core::ffi::c_void;
pub type ITfCompartment = *mut ::core::ffi::c_void;
pub type ITfCompartmentEventSink = *mut ::core::ffi::c_void;
pub type ITfCompartmentMgr = *mut ::core::ffi::c_void;
pub type ITfComposition = *mut ::core::ffi::c_void;
pub type ITfCompositionSink = *mut ::core::ffi::c_void;
pub type ITfCompositionView = *mut ::core::ffi::c_void;
pub type ITfConfigureSystemKeystrokeFeed = *mut ::core::ffi::c_void;
pub type ITfContext = *mut ::core::ffi::c_void;
pub type ITfContextComposition = *mut ::core::ffi::c_void;
pub type ITfContextKeyEventSink = *mut ::core::ffi::c_void;
pub type ITfContextOwner = *mut ::core::ffi::c_void;
pub type ITfContextOwnerCompositionServices = *mut ::core::ffi::c_void;
pub type ITfContextOwnerCompositionSink = *mut ::core::ffi::c_void;
pub type ITfContextOwnerServices = *mut ::core::ffi::c_void;
pub type ITfContextView = *mut ::core::ffi::c_void;
pub type ITfCreatePropertyStore = *mut ::core::ffi::c_void;
pub type ITfDisplayAttributeInfo = *mut ::core::ffi::c_void;
pub type ITfDisplayAttributeMgr = *mut ::core::ffi::c_void;
pub type ITfDisplayAttributeNotifySink = *mut ::core::ffi::c_void;
pub type ITfDisplayAttributeProvider = *mut ::core::ffi::c_void;
pub type ITfDocumentMgr = *mut ::core::ffi::c_void;
pub type ITfEditRecord = *mut ::core::ffi::c_void;
pub type ITfEditSession = *mut ::core::ffi::c_void;
pub type ITfEditTransactionSink = *mut ::core::ffi::c_void;
pub type ITfFnAdviseText = *mut ::core::ffi::c_void;
pub type ITfFnBalloon = *mut ::core::ffi::c_void;
pub type ITfFnConfigure = *mut ::core::ffi::c_void;
pub type ITfFnConfigureRegisterEudc = *mut ::core::ffi::c_void;
pub type ITfFnConfigureRegisterWord = *mut ::core::ffi::c_void;
pub type ITfFnCustomSpeechCommand = *mut ::core::ffi::c_void;
pub type ITfFnGetLinguisticAlternates = *mut ::core::ffi::c_void;
pub type ITfFnGetPreferredTouchKeyboardLayout = *mut ::core::ffi::c_void;
pub type ITfFnGetSAPIObject = *mut ::core::ffi::c_void;
pub type ITfFnLMInternal = *mut ::core::ffi::c_void;
pub type ITfFnLMProcessor = *mut ::core::ffi::c_void;
pub type ITfFnLangProfileUtil = *mut ::core::ffi::c_void;
pub type ITfFnPlayBack = *mut ::core::ffi::c_void;
pub type ITfFnPropertyUIStatus = *mut ::core::ffi::c_void;
pub type ITfFnReconversion = *mut ::core::ffi::c_void;
pub type ITfFnSearchCandidateProvider = *mut ::core::ffi::c_void;
pub type ITfFnShowHelp = *mut ::core::ffi::c_void;
pub type ITfFunction = *mut ::core::ffi::c_void;
pub type ITfFunctionProvider = *mut ::core::ffi::c_void;
pub type ITfInputProcessorProfileActivationSink = *mut ::core::ffi::c_void;
pub type ITfInputProcessorProfileMgr = *mut ::core::ffi::c_void;
pub type ITfInputProcessorProfileSubstituteLayout = *mut ::core::ffi::c_void;
pub type ITfInputProcessorProfiles = *mut ::core::ffi::c_void;
pub type ITfInputProcessorProfilesEx = *mut ::core::ffi::c_void;
pub type ITfInputScope = *mut ::core::ffi::c_void;
pub type ITfInputScope2 = *mut ::core::ffi::c_void;
pub type ITfInsertAtSelection = *mut ::core::ffi::c_void;
pub type ITfIntegratableCandidateListUIElement = *mut ::core::ffi::c_void;
pub type ITfKeyEventSink = *mut ::core::ffi::c_void;
pub type ITfKeyTraceEventSink = *mut ::core::ffi::c_void;
pub type ITfKeystrokeMgr = *mut ::core::ffi::c_void;
pub type ITfLMLattice = *mut ::core::ffi::c_void;
pub type ITfLangBarEventSink = *mut ::core::ffi::c_void;
pub type ITfLangBarItem = *mut ::core::ffi::c_void;
pub type ITfLangBarItemBalloon = *mut ::core::ffi::c_void;
pub type ITfLangBarItemBitmap = *mut ::core::ffi::c_void;
pub type ITfLangBarItemBitmapButton = *mut ::core::ffi::c_void;
pub type ITfLangBarItemButton = *mut ::core::ffi::c_void;
pub type ITfLangBarItemMgr = *mut ::core::ffi::c_void;
pub type ITfLangBarItemSink = *mut ::core::ffi::c_void;
pub type ITfLangBarMgr = *mut ::core::ffi::c_void;
pub type ITfLanguageProfileNotifySink = *mut ::core::ffi::c_void;
pub type ITfMSAAControl = *mut ::core::ffi::c_void;
pub type ITfMenu = *mut ::core::ffi::c_void;
pub type ITfMessagePump = *mut ::core::ffi::c_void;
pub type ITfMouseSink = *mut ::core::ffi::c_void;
pub type ITfMouseTracker = *mut ::core::ffi::c_void;
pub type ITfMouseTrackerACP = *mut ::core::ffi::c_void;
pub type ITfPersistentPropertyLoaderACP = *mut ::core::ffi::c_void;
pub type ITfPreservedKeyNotifySink = *mut ::core::ffi::c_void;
pub type ITfProperty = *mut ::core::ffi::c_void;
pub type ITfPropertyStore = *mut ::core::ffi::c_void;
pub type ITfQueryEmbedded = *mut ::core::ffi::c_void;
pub type ITfRange = *mut ::core::ffi::c_void;
pub type ITfRangeACP = *mut ::core::ffi::c_void;
pub type ITfRangeBackup = *mut ::core::ffi::c_void;
pub type ITfReadOnlyProperty = *mut ::core::ffi::c_void;
pub type ITfReadingInformationUIElement = *mut ::core::ffi::c_void;
pub type ITfReverseConversion = *mut ::core::ffi::c_void;
pub type ITfReverseConversionList = *mut ::core::ffi::c_void;
pub type ITfReverseConversionMgr = *mut ::core::ffi::c_void;
pub type ITfSource = *mut ::core::ffi::c_void;
pub type ITfSourceSingle = *mut ::core::ffi::c_void;
pub type ITfSpeechUIServer = *mut ::core::ffi::c_void;
pub type ITfStatusSink = *mut ::core::ffi::c_void;
pub type ITfSystemDeviceTypeLangBarItem = *mut ::core::ffi::c_void;
pub type ITfSystemLangBarItem = *mut ::core::ffi::c_void;
pub type ITfSystemLangBarItemSink = *mut ::core::ffi::c_void;
pub type ITfSystemLangBarItemText = *mut ::core::ffi::c_void;
pub type ITfTextEditSink = *mut ::core::ffi::c_void;
pub type ITfTextInputProcessor = *mut ::core::ffi::c_void;
pub type ITfTextInputProcessorEx = *mut ::core::ffi::c_void;
pub type ITfTextLayoutSink = *mut ::core::ffi::c_void;
pub type ITfThreadFocusSink = *mut ::core::ffi::c_void;
pub type ITfThreadMgr = *mut ::core::ffi::c_void;
pub type ITfThreadMgr2 = *mut ::core::ffi::c_void;
pub type ITfThreadMgrEventSink = *mut ::core::ffi::c_void;
pub type ITfThreadMgrEx = *mut ::core::ffi::c_void;
pub type ITfToolTipUIElement = *mut ::core::ffi::c_void;
pub type ITfTransitoryExtensionSink = *mut ::core::ffi::c_void;
pub type ITfTransitoryExtensionUIElement = *mut ::core::ffi::c_void;
pub type ITfUIElement = *mut ::core::ffi::c_void;
pub type ITfUIElementMgr = *mut ::core::ffi::c_void;
pub type ITfUIElementSink = *mut ::core::ffi::c_void;
pub type IUIManagerEventSink = *mut ::core::ffi::c_void;
pub type IVersionInfo = *mut ::core::ffi::c_void;
pub const AccClientDocMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4232629296, data2: 20286, data3: 20385, data4: [128, 59, 173, 14, 25, 106, 131, 177] };
pub const AccDictionary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1702030870, data2: 24549, data3: 17201, data4: [187, 109, 118, 164, 156, 86, 228, 35] };
pub const AccServerDocMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1619633022, data2: 60298, data3: 18477, data4: [189, 111, 249, 244, 105, 4, 209, 109] };
pub const AccStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1413514111, data2: 19455, data3: 19173, data4: [161, 177, 119, 34, 236, 198, 51, 42] };
pub const CLSID_TF_CategoryMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2763343009, data2: 17293, data3: 19265, data4: [147, 37, 134, 149, 35, 226, 214, 199] };
pub const CLSID_TF_ClassicLangBar: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 857224716, data2: 6908, data3: 19721, data4: [168, 107, 159, 156, 182, 220, 235, 156] };
pub const CLSID_TF_DisplayAttributeMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1021791716, data2: 21459, data3: 19828, data4: [139, 131, 67, 27, 56, 40, 186, 83] };
pub const CLSID_TF_InputProcessorProfiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 868563536, data2: 62550, data3: 18564, data4: [176, 73, 133, 253, 100, 62, 207, 237] };
pub const CLSID_TF_LangBarItemMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3113424530, data2: 41651, data3: 20395, data4: [191, 51, 158, 198, 249, 251, 150, 172] };
pub const CLSID_TF_LangBarMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3954216005, data2: 27722, data3: 20444, data4: [174, 83, 78, 184, 196, 199, 219, 142] };
pub const CLSID_TF_ThreadMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1385864811, data2: 25991, data3: 20259, data4: [171, 158, 156, 125, 104, 62, 60, 80] };
pub const CLSID_TF_TransitoryExtensionUIEntry: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2926305288, data2: 2043, data3: 16397, data4: [139, 235, 51, 122, 100, 247, 5, 31] };
pub const CLSID_TsfServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 967760896, data2: 27488, data3: 18139, data4: [141, 49, 54, 66, 190, 14, 67, 115] };
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const DCM_FLAGS_CTFMON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const DCM_FLAGS_LOCALTHREADTSF: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const DCM_FLAGS_TASKENG: u32 = 1u32;
pub const DocWrap: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3208802174, data2: 31326, data3: 17622, data4: [131, 12, 163, 144, 234, 148, 98, 163] };
pub const GUID_APP_FUNCTIONPROVIDER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1286533150, data2: 4783, data3: 19214, data4: [157, 177, 166, 236, 91, 136, 18, 8] };
pub const GUID_COMPARTMENT_CONVERSIONMODEBIAS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1419244822, data2: 61073, data3: 17262, data4: [185, 70, 170, 44, 5, 241, 172, 91] };
pub const GUID_COMPARTMENT_EMPTYCONTEXT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3611852223, data2: 32846, data3: 16837, data4: [137, 77, 173, 150, 253, 78, 234, 19] };
pub const GUID_COMPARTMENT_ENABLED_PROFILES_UPDATED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2462186824, data2: 43438, data3: 19068, data4: [190, 8, 67, 41, 228, 114, 56, 23] };
pub const GUID_COMPARTMENT_HANDWRITING_OPENCLOSE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4188941419, data2: 6246, data3: 17249, data4: [175, 114, 122, 163, 9, 72, 137, 14] };
pub const GUID_COMPARTMENT_KEYBOARD_DISABLED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1906684499, data2: 6481, data3: 18027, data4: [159, 188, 156, 136, 8, 250, 132, 242] };
pub const GUID_COMPARTMENT_KEYBOARD_INPUTMODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3059295505, data2: 48366, data3: 16674, data4: [167, 196, 9, 244, 179, 250, 67, 150] };
pub const GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3438304728, data2: 19079, data3: 4567, data4: [166, 226, 0, 6, 91, 132, 67, 92] };
pub const GUID_COMPARTMENT_KEYBOARD_INPUTMODE_SENTENCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3438304729, data2: 19079, data3: 4567, data4: [166, 226, 0, 6, 91, 132, 67, 92] };
pub const GUID_COMPARTMENT_KEYBOARD_OPENCLOSE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1478965933, data2: 443, data3: 16740, data4: [149, 198, 117, 91, 160, 181, 22, 45] };
pub const GUID_COMPARTMENT_SAPI_AUDIO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1370431622, data2: 52331, data3: 17789, data4: [181, 170, 139, 25, 220, 41, 10, 180] };
pub const GUID_COMPARTMENT_SPEECH_CFGMENU: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4218182701, data2: 20099, data3: 19382, data4: [145, 162, 224, 25, 191, 246, 118, 45] };
pub const GUID_COMPARTMENT_SPEECH_DISABLED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1455801863, data2: 1795, data3: 20057, data4: [142, 82, 203, 200, 78, 139, 190, 53] };
pub const GUID_COMPARTMENT_SPEECH_GLOBALSTATE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 710213262, data2: 3336, data3: 17932, data4: [167, 93, 135, 3, 95, 244, 54, 197] };
pub const GUID_COMPARTMENT_SPEECH_OPENCLOSE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1414359651, data2: 58088, data3: 18258, data4: [187, 209, 0, 9, 96, 188, 160, 131] };
pub const GUID_COMPARTMENT_SPEECH_UI_STATUS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3642758896, data2: 37735, data3: 20455, data4: [154, 191, 188, 89, 218, 203, 224, 227] };
pub const GUID_COMPARTMENT_TIPUISTATUS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 344761324, data2: 870, data3: 16412, data4: [141, 117, 237, 151, 141, 133, 251, 201] };
pub const GUID_COMPARTMENT_TRANSITORYEXTENSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2346928117, data2: 51104, data3: 4567, data4: [180, 8, 0, 6, 91, 132, 67, 92] };
pub const GUID_COMPARTMENT_TRANSITORYEXTENSION_DOCUMENTMANAGER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2346928119, data2: 51104, data3: 4567, data4: [180, 8, 0, 6, 91, 132, 67, 92] };
pub const GUID_COMPARTMENT_TRANSITORYEXTENSION_PARENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2346928120, data2: 51104, data3: 4567, data4: [180, 8, 0, 6, 91, 132, 67, 92] };
pub const GUID_INTEGRATIONSTYLE_SEARCHBOX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3872505105, data2: 33527, data3: 18691, data4: [174, 33, 26, 99, 151, 205, 226, 235] };
pub const GUID_LBI_INPUTMODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 746039326, data2: 16844, data3: 16760, data4: [163, 167, 95, 138, 152, 117, 104, 230] };
pub const GUID_LBI_SAPILAYR_CFGMENUBUTTON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3492750497, data2: 37933, data3: 16942, data4: [141, 153, 180, 242, 173, 222, 233, 153] };
pub const GUID_MODEBIAS_CHINESE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2061313758, data2: 17192, data3: 18587, data4: [131, 174, 100, 147, 117, 12, 173, 92] };
pub const GUID_MODEBIAS_CONVERSATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 256819460, data2: 6032, data3: 17467, data4: [149, 241, 225, 15, 147, 157, 101, 70] };
pub const GUID_MODEBIAS_DATETIME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4072518514, data2: 32609, data3: 16441, data4: [146, 239, 28, 53, 89, 159, 2, 34] };
pub const GUID_MODEBIAS_FILENAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3623290878, data2: 17606, data3: 20426, data4: [142, 118, 134, 171, 80, 199, 147, 27] };
pub const GUID_MODEBIAS_FULLWIDTHALPHANUMERIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2169020344, data2: 45930, data3: 18237, data4: [129, 70, 228, 162, 37, 139, 36, 174] };
pub const GUID_MODEBIAS_FULLWIDTHHANGUL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3222988489, data2: 17845, data3: 20432, data4: [156, 177, 159, 76, 235, 195, 159, 234] };
pub const GUID_MODEBIAS_HALFWIDTHKATAKANA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 6253411, data2: 30932, data3: 16844, data4: [136, 89, 72, 92, 168, 33, 167, 149] };
pub const GUID_MODEBIAS_HANGUL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1995375937, data2: 9139, data3: 19831, data4: [160, 116, 105, 24, 1, 204, 234, 23] };
pub const GUID_MODEBIAS_HIRAGANA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3611111790, data2: 39825, data3: 18161, data4: [162, 128, 49, 89, 127, 82, 198, 148] };
pub const GUID_MODEBIAS_KATAKANA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 772730333, data2: 14874, data3: 18846, data4: [133, 67, 60, 126, 231, 148, 152, 17] };
pub const GUID_MODEBIAS_NAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4259057904, data2: 53817, data3: 18879, data4: [184, 252, 84, 16, 202, 170, 66, 126] };
pub const GUID_MODEBIAS_NONE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 0, data2: 0, data3: 0, data4: [0, 0, 0, 0, 0, 0, 0, 0] };
pub const GUID_MODEBIAS_NUMERIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1075934828, data2: 59506, data3: 18685, data4: [156, 238, 78, 197, 199, 94, 22, 195] };
pub const GUID_MODEBIAS_READING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3809887139, data2: 25702, data3: 19647, data4: [141, 139, 11, 212, 216, 84, 84, 97] };
pub const GUID_MODEBIAS_URLHISTORY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2332972249, data2: 25586, data3: 19560, data4: [132, 212, 121, 174, 231, 165, 159, 9] };
pub const GUID_PROP_ATTRIBUTE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 884233840, data2: 29990, data3: 4562, data4: [161, 71, 0, 16, 90, 39, 153, 181] };
pub const GUID_PROP_COMPOSING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3777675360, data2: 44821, data3: 4562, data4: [175, 197, 0, 16, 90, 39, 153, 181] };
pub const GUID_PROP_INPUTSCOPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 387177818, data2: 26855, data3: 19035, data4: [154, 246, 89, 42, 89, 92, 119, 141] };
pub const GUID_PROP_LANGID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 847302176, data2: 32818, data3: 4562, data4: [182, 3, 0, 16, 90, 39, 153, 181] };
pub const GUID_PROP_MODEBIAS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 925763350, data2: 38735, data3: 16556, data4: [160, 136, 8, 205, 201, 46, 191, 188] };
pub const GUID_PROP_READING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1415837632, data2: 36401, data3: 4562, data4: [191, 70, 0, 16, 90, 39, 153, 181] };
pub const GUID_PROP_TEXTOWNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4058174752, data2: 2409, data3: 4563, data4: [141, 240, 0, 16, 90, 39, 153, 181] };
pub const GUID_PROP_TKB_ALTERNATES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1890756611, data2: 38541, data3: 17966, data4: [185, 59, 33, 100, 201, 21, 23, 247] };
pub const GUID_SYSTEM_FUNCTIONPROVIDER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2590608304, data2: 3873, data3: 4563, data4: [141, 241, 0, 16, 90, 39, 153, 181] };
pub const GUID_TFCAT_CATEGORY_OF_TIP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1397508289, data2: 1543, data3: 16536, data4: [165, 33, 79, 200, 153, 199, 62, 144] };
pub const GUID_TFCAT_DISPLAYATTRIBUTEPROPERTY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3110017051, data2: 59980, data3: 19185, data4: [128, 86, 124, 50, 26, 187, 176, 145] };
pub const GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 74157184, data2: 5703, data3: 16631, data4: [155, 33, 185, 59, 129, 170, 188, 27] };
pub const GUID_TFCAT_PROPSTYLE_STATIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1449113816, data2: 27604, data3: 19617, data4: [178, 35, 15, 44, 203, 143, 79, 150] };
pub const GUID_TFCAT_PROP_AUDIODATA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2608587689, data2: 59563, data3: 19783, data4: [168, 254, 37, 79, 164, 35, 67, 109] };
pub const GUID_TFCAT_PROP_INKDATA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2087355054, data2: 45271, data3: 20244, data4: [167, 69, 20, 242, 139, 0, 157, 97] };
pub const GUID_TFCAT_TIPCAP_COMLESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 910300633, data2: 30140, data3: 4567, data4: [166, 239, 0, 6, 91, 132, 67, 92] };
pub const GUID_TFCAT_TIPCAP_DUALMODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 989009058, data2: 55199, data3: 19227, data4: [153, 146, 21, 8, 109, 51, 155, 5] };
pub const GUID_TFCAT_TIPCAP_IMMERSIVEONLY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 977426860, data2: 25613, data3: 19156, data4: [137, 247, 30, 182, 126, 124, 78, 232] };
pub const GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 329258719, data2: 22027, data3: 18125, data4: [148, 122, 76, 58, 241, 224, 227, 93] };
pub const GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3438304727, data2: 19079, data3: 4567, data4: [166, 226, 0, 6, 91, 132, 67, 92] };
pub const GUID_TFCAT_TIPCAP_LOCALSERVER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1953930985, data2: 19046, data3: 20381, data4: [144, 214, 191, 139, 124, 62, 180, 97] };
pub const GUID_TFCAT_TIPCAP_SECUREMODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1238563278, data2: 8030, data3: 4567, data4: [166, 211, 0, 6, 91, 132, 67, 92] };
pub const GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 626020276, data2: 31659, data3: 19393, data4: [156, 105, 207, 129, 137, 15, 14, 245] };
pub const GUID_TFCAT_TIPCAP_TSF3: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 131904687, data2: 39134, data3: 17736, data4: [190, 247, 37, 189, 69, 151, 154, 31] };
pub const GUID_TFCAT_TIPCAP_UIELEMENTENABLED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1238563279, data2: 8030, data3: 4567, data4: [166, 211, 0, 6, 91, 132, 67, 92] };
pub const GUID_TFCAT_TIPCAP_WOW16: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 910300634, data2: 30140, data3: 4567, data4: [166, 239, 0, 6, 91, 132, 67, 92] };
pub const GUID_TFCAT_TIP_HANDWRITING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 611240839, data2: 49906, data3: 19134, data4: [144, 91, 200, 179, 138, 221, 44, 67] };
pub const GUID_TFCAT_TIP_KEYBOARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 880041059, data2: 45808, data3: 18308, data4: [139, 103, 94, 18, 200, 112, 26, 49] };
pub const GUID_TFCAT_TIP_SPEECH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3047636177, data2: 33621, data3: 17003, data4: [161, 97, 37, 152, 8, 242, 107, 20] };
pub const GUID_TFCAT_TRANSITORYEXTENSIONUI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1661132322, data2: 42447, data3: 19202, data4: [191, 232, 77, 114, 178, 190, 211, 198] };
pub const GUID_TS_SERVICE_ACCESSIBLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4185416192, data2: 42431, data3: 18959, data4: [140, 36, 251, 22, 245, 209, 170, 187] };
pub const GUID_TS_SERVICE_ACTIVEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3935533648, data2: 51622, data3: 19325, data4: [137, 74, 73, 217, 155, 120, 72, 52] };
pub const GUID_TS_SERVICE_DATAOBJECT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1619458997, data2: 57893, data3: 18126, data4: [167, 112, 193, 187, 211, 224, 93, 123] };
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GXFPF_NEAREST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GXFPF_ROUND_NEAREST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const ILMCM_CHECKLAYOUTANDTIPENABLED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const ILMCM_LANGUAGEBAROFF: u32 = 2u32;
pub const LIBID_MSAATEXTLib: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 353250682, data2: 56001, data3: 17794, data4: [148, 125, 42, 143, 215, 139, 130, 205] };
pub const MSAAControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 147691071, data2: 31294, data3: 20316, data4: [155, 216, 214, 146, 187, 4, 60, 91] };
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CHAR_EMBEDDED: u32 = 65532u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CLUIE_COUNT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CLUIE_CURRENTPAGE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CLUIE_DOCUMENTMGR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CLUIE_PAGEINDEX: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CLUIE_SELECTION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CLUIE_STRING: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_COMMANDING_ENABLED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_COMMANDING_ON: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_ALPHANUMERIC: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_CHARCODE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_EUDC: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_FIXED: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_FULLSHAPE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_KATAKANA: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_NATIVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_NOCONVERSION: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_ROMAN: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_SOFTKEYBOARD: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CONVERSIONMODE_SYMBOL: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DICTATION_ENABLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DICTATION_ON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DISABLE_BALLOON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DISABLE_COMMANDING: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DISABLE_DICTATION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DISABLE_SPEECH: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ENABLE_PROCESS_ATOM: &str = "_CTF_ENABLE_PROCESS_ATOM_";
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_ALREADY_EXISTS: ::windows_sys::core::HRESULT = -2147220218i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_COMPOSITION_REJECTED: ::windows_sys::core::HRESULT = -2147220216i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_DISCONNECTED: ::windows_sys::core::HRESULT = -2147220220i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_EMPTYCONTEXT: ::windows_sys::core::HRESULT = -2147220215i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_FORMAT: ::windows_sys::core::HRESULT = -2147220982i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_INVALIDPOINT: ::windows_sys::core::HRESULT = -2147220985i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_INVALIDPOS: ::windows_sys::core::HRESULT = -2147220992i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_INVALIDVIEW: ::windows_sys::core::HRESULT = -2147220219i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_LOCKED: ::windows_sys::core::HRESULT = -2147220224i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOCONVERSION: ::windows_sys::core::HRESULT = -2147219968i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOINTERFACE: ::windows_sys::core::HRESULT = -2147220988i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOLAYOUT: ::windows_sys::core::HRESULT = -2147220986i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOLOCK: ::windows_sys::core::HRESULT = -2147220991i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOOBJECT: ::windows_sys::core::HRESULT = -2147220990i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOPROVIDER: ::windows_sys::core::HRESULT = -2147220221i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOSELECTION: ::windows_sys::core::HRESULT = -2147220987i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOSERVICE: ::windows_sys::core::HRESULT = -2147220989i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_NOTOWNEDRANGE: ::windows_sys::core::HRESULT = -2147220222i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_RANGE_NOT_COVERED: ::windows_sys::core::HRESULT = -2147220217i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_READONLY: ::windows_sys::core::HRESULT = -2147220983i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_STACKFULL: ::windows_sys::core::HRESULT = -2147220223i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_E_SYNCHRONOUS: ::windows_sys::core::HRESULT = -2147220984i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_FLOATINGLANGBAR_WNDTITLE: &str = "TF_FloatingLangBar_WndTitle";
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_FLOATINGLANGBAR_WNDTITLEA: &str = "TF_FloatingLangBar_WndTitle";
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_FLOATINGLANGBAR_WNDTITLEW: &str = "TF_FloatingLangBar_WndTitle";
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_HF_OBJECT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IE_CORRECTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_INVALID_COOKIE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_INVALID_EDIT_COOKIE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPPMF_DISABLEPROFILE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPPMF_ENABLEPROFILE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPPMF_FORPROCESS: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPPMF_FORSESSION: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPPMF_FORSYSTEMALL: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_COMLESSSUPPORT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_DISABLEONTRANSITORY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_IMMERSIVESUPPORT: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_SECUREMODESUPPORT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_SYSTRAYSUPPORT: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_UIELEMENTENABLED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_CAPS_WOW16SUPPORT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_FLAG_ACTIVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_FLAG_ENABLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPP_FLAG_SUBSTITUTEDBYINPUTPROCESSOR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IPSINK_FLAG_ACTIVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_BALLOON: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_BITMAP: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_BMPF_VERTICAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_CUSTOMUI: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_DESC_MAXLEN: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_ICON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STATUS: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STATUS_BTN_TOGGLED: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STATUS_DISABLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STATUS_HIDDEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_BTN_BUTTON: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_BTN_MENU: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_BTN_TOGGLE: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_HIDDENBYDEFAULT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_HIDDENSTATUSCONTROL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_HIDEONNOOTHERITEMS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_SHOWNINTRAY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_SHOWNINTRAYONLY: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_STYLE_TEXTCOLORICON: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_TEXT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_TOOLTIP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBMENUF_CHECKED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBMENUF_GRAYED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBMENUF_RADIOCHECKED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBMENUF_SEPARATOR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBMENUF_SUBMENU: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MENUREADY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_ALT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_CONTROL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_IGNORE_ALL_MODIFIER: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_LALT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_LCONTROL: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_LSHIFT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_ON_KEYUP: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_RALT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_RCONTROL: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_RSHIFT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_MOD_SHIFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_POPF_ALL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_PROCESS_ATOM: &str = "_CTF_PROCESS_ATOM_";
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_PROFILETYPE_INPUTPROCESSOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_PROFILETYPE_KEYBOARDLAYOUT: u32 = 2u32;
pub const TF_PROFILE_ARRAY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3549364069, data2: 43590, data3: 20437, data4: [145, 167, 103, 132, 95, 176, 47, 91] };
pub const TF_PROFILE_CANTONESE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 183242908, data2: 32406, data3: 4564, data4: [178, 239, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_CHANGJIE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1272946435, data2: 51155, data3: 4564, data4: [178, 171, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_DAYI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 58403877, data2: 18444, data3: 19839, data4: [176, 39, 214, 202, 107, 105, 120, 138] };
pub const TF_PROFILE_NEWCHANGJIE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4089090170, data2: 27774, data3: 4564, data4: [151, 250, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_NEWPHONETIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3002713346, data2: 5954, data3: 4564, data4: [151, 144, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_NEWQUICK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 193477536, data2: 49607, data3: 4564, data4: [135, 249, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_PHONETIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1980959198, data2: 12666, data3: 4564, data4: [155, 93, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_PINYIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4089090167, data2: 27774, data3: 4564, data4: [151, 250, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_QUICK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1613018207, data2: 23636, data3: 4564, data4: [185, 33, 0, 128, 200, 130, 104, 126] };
pub const TF_PROFILE_SIMPLEFAST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4199877380, data2: 23255, data3: 16671, data4: [165, 172, 202, 3, 142, 197, 21, 215] };
pub const TF_PROFILE_TIGRINYA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1017874615, data2: 52286, data3: 18086, data4: [151, 101, 183, 114, 173, 119, 97, 255] };
pub const TF_PROFILE_WUBI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2186873875, data2: 62685, data3: 17652, data4: [186, 29, 134, 103, 36, 111, 223, 142] };
pub const TF_PROFILE_YI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1083999094, data2: 123, data3: 17239, data4: [174, 142, 38, 49, 110, 227, 251, 13] };
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_PROPUI_STATUS_SAVETOFILE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RCM_COMLESS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RCM_HINT_COLLISION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RCM_HINT_READING_LENGTH: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RCM_VKEY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RIP_FLAG_FREEUNUSEDLIBRARIES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RIUIE_CONTEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RIUIE_ERRORINDEX: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RIUIE_MAXREADINGSTRINGLENGTH: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RIUIE_STRING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RIUIE_VERTICALORDER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RP_HIDDENINSETTINGUI: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RP_LOCALPROCESS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RP_LOCALTHREAD: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_RP_SUBITEMINSETTINGUI: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SD_LOADING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SD_READONLY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SENTENCEMODE_AUTOMATIC: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SENTENCEMODE_CONVERSATION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SENTENCEMODE_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SENTENCEMODE_PHRASEPREDICT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SENTENCEMODE_PLAURALCLAUSE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SENTENCEMODE_SINGLECONVERT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_DESKBAND: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_DOCK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_EXTRAICONSONMINIMIZED: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_HIDDEN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_HIGHTRANSPARENCY: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_LABELS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_LOWTRANSPARENCY: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_MINIMIZED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_NOEXTRAICONSONMINIMIZED: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_NOLABELS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_NOTRANSPARENCY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SFT_SHOWNORMAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SHOW_BALLOON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SPEECHUI_SHOWN: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SS_DISJOINTSEL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SS_REGIONS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SS_TKBAUTOCORRECTENABLE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SS_TKBPREDICTIONENABLE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SS_TRANSITORY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ST_CORRECTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_S_ASYNC: ::windows_sys::core::HRESULT = 262912i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TF_IGNOREEND: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TF_MOVESTART: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_COMLESS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_CONSOLE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_NOACTIVATEKEYBOARDLAYOUT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_NOACTIVATETIP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_SECUREMODE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_UIELEMENTENABLEDONLY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMAE_WOW16: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_ACTIVATED: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_COMLESS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_CONSOLE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_IMMERSIVEMODE: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_NOACTIVATETIP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_SECUREMODE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_UIELEMENTENABLEDONLY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TMF_WOW16: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TRANSITORYEXTENSION_ATSELECTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TRANSITORYEXTENSION_FLOATING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TRANSITORYEXTENSION_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_TU_CORRECTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_URP_ALLPROFILES: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_URP_LOCALPROCESS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_URP_LOCALTHREAD: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_US_HIDETIPUI: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_CLASSIC_TRADITIONAL_CHINESE_CHANGJIE: u32 = 61506u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_CLASSIC_TRADITIONAL_CHINESE_DAYI: u32 = 61507u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_CLASSIC_TRADITIONAL_CHINESE_PHONETIC: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_OPT_JAPANESE_ABC: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_OPT_KOREAN_HANGUL_2_BULSIK: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_OPT_SIMPLIFIED_CHINESE_PINYIN: u32 = 2052u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_OPT_TRADITIONAL_CHINESE_PHONETIC: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBL_UNDEFINED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKB_ALTERNATES_AUTOCORRECTION_APPLIED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKB_ALTERNATES_FOR_AUTOCORRECTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKB_ALTERNATES_FOR_PREDICTION: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKB_ALTERNATES_STANDARD: u32 = 1u32;
pub const TSATTRID_App: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2819586015, data2: 16951, data3: 16613, data4: [132, 156, 181, 250, 81, 193, 58, 199] };
pub const TSATTRID_App_IncorrectGrammar: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3176457112, data2: 44291, data3: 19316, data4: [182, 179, 94, 219, 25, 153, 99, 136] };
pub const TSATTRID_App_IncorrectSpelling: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4096648252, data2: 61202, data3: 17165, data4: [148, 76, 154, 8, 151, 10, 37, 210] };
pub const TSATTRID_Font: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1463724069, data2: 29851, data3: 20362, data4: [156, 253, 33, 195, 96, 92, 168, 40] };
pub const TSATTRID_Font_FaceName: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3040259766, data2: 1339, data3: 20152, data4: [182, 90, 80, 218, 30, 129, 231, 46] };
pub const TSATTRID_Font_SizePts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3360240386, data2: 42473, data3: 17773, data4: [175, 4, 128, 5, 228, 19, 15, 3] };
pub const TSATTRID_Font_Style: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1756538751, data2: 27406, data3: 20264, data4: [129, 119, 87, 28, 47, 58, 66, 177] };
pub const TSATTRID_Font_Style_Animation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3707190562, data2: 57385, data3: 18359, data4: [187, 54, 242, 99, 163, 208, 4, 204] };
pub const TSATTRID_Font_Style_Animation_BlinkingBackground: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2263200004, data2: 260, data3: 19216, data4: [181, 133, 0, 242, 82, 117, 34, 181] };
pub const TSATTRID_Font_Style_Animation_LasVegasLights: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4093912021, data2: 3975, data3: 20367, data4: [186, 218, 230, 214, 12, 37, 225, 82] };
pub const TSATTRID_Font_Style_Animation_MarchingBlackAnts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1984225383, data2: 61830, data3: 18690, data4: [191, 198, 236, 129, 90, 162, 14, 157] };
pub const TSATTRID_Font_Style_Animation_MarchingRedAnts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2016841133, data2: 20731, data3: 19567, data4: [132, 11, 212, 134, 187, 108, 247, 129] };
pub const TSATTRID_Font_Style_Animation_Shimmer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 753081176, data2: 21139, data3: 19510, data4: [136, 9, 191, 139, 181, 26, 39, 179] };
pub const TSATTRID_Font_Style_Animation_SparkleText: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1396354336, data2: 38444, data3: 20127, data4: [140, 9, 180, 46, 164, 116, 151, 17] };
pub const TSATTRID_Font_Style_Animation_WipeDown: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1483925620, data2: 13947, data3: 18435, data4: [177, 96, 201, 15, 246, 37, 105, 208] };
pub const TSATTRID_Font_Style_Animation_WipeRight: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3092630499, data2: 15660, data3: 17920, data4: [177, 233, 225, 201, 206, 2, 248, 66] };
pub const TSATTRID_Font_Style_BackgroundColor: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3037637198, data2: 12433, data3: 17512, data4: [129, 219, 215, 158, 161, 144, 199, 199] };
pub const TSATTRID_Font_Style_Blink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3216162870, data2: 31439, data3: 17714, data4: [183, 32, 180, 22, 221, 119, 101, 168] };
pub const TSATTRID_Font_Style_Bold: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1216428611, data2: 35360, data3: 18752, data4: [142, 88, 151, 130, 63, 123, 38, 138] };
pub const TSATTRID_Font_Style_Capitalize: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2105910202, data2: 46333, data3: 17331, data4: [190, 252, 107, 152, 92, 132, 49, 65] };
pub const TSATTRID_Font_Style_Color: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2239396407, data2: 47279, data3: 20122, data4: [129, 180, 172, 247, 0, 200, 65, 27] };
pub const TSATTRID_Font_Style_Emboss: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3180255042, data2: 13470, data3: 20023, data4: [130, 251, 67, 121, 121, 203, 83, 167] };
pub const TSATTRID_Font_Style_Engrave: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2620617182, data2: 33586, data3: 18583, data4: [190, 93, 137, 35, 50, 35, 23, 154] };
pub const TSATTRID_Font_Style_Height: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2123592823, data2: 4838, data3: 17803, data4: [146, 106, 31, 164, 78, 232, 243, 145] };
pub const TSATTRID_Font_Style_Hidden: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2984413040, data2: 34844, data3: 18271, data4: [134, 63, 136, 122, 100, 123, 16, 144] };
pub const TSATTRID_Font_Style_Italic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2269145130, data2: 42853, data3: 18657, data4: [172, 252, 210, 34, 34, 178, 248, 16] };
pub const TSATTRID_Font_Style_Kerning: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3425100212, data2: 12186, data3: 18376, data4: [139, 255, 191, 30, 183, 204, 224, 221] };
pub const TSATTRID_Font_Style_Lowercase: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1993919669, data2: 51835, data3: 17560, data4: [142, 233, 213, 196, 246, 247, 76, 96] };
pub const TSATTRID_Font_Style_Outlined: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 283564849, data2: 56077, data3: 19142, data4: [167, 245, 156, 156, 255, 111, 42, 180] };
pub const TSATTRID_Font_Style_Overline: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3818430282, data2: 39211, data3: 17153, data4: [140, 225, 165, 183, 198, 209, 243, 200] };
pub const TSATTRID_Font_Style_Overline_Double: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3695576634, data2: 57621, data3: 18147, data4: [188, 216, 202, 103, 114, 170, 149, 180] };
pub const TSATTRID_Font_Style_Overline_Single: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2218842444, data2: 20942, data3: 18354, data4: [141, 76, 21, 117, 30, 95, 114, 27] };
pub const TSATTRID_Font_Style_Position: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 365766315, data2: 62203, data3: 16482, data4: [181, 166, 154, 73, 225, 165, 204, 11] };
pub const TSATTRID_Font_Style_Protected: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 475364530, data2: 5327, data3: 17748, data4: [165, 116, 236, 178, 247, 231, 239, 212] };
pub const TSATTRID_Font_Style_Shadow: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1600679215, data2: 50893, data3: 19542, data4: [138, 26, 153, 74, 75, 151, 102, 190] };
pub const TSATTRID_Font_Style_SmallCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4207635398, data2: 37120, data3: 19654, data4: [185, 105, 17, 238, 164, 90, 134, 180] };
pub const TSATTRID_Font_Style_Spacing: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2562793485, data2: 36614, data3: 16538, data4: [142, 73, 106, 85, 75, 247, 193, 83] };
pub const TSATTRID_Font_Style_Strikethrough: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 206971283, data2: 11528, data3: 18024, data4: [150, 1, 206, 212, 19, 9, 215, 175] };
pub const TSATTRID_Font_Style_Strikethrough_Double: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1648925489, data2: 41959, data3: 20372, data4: [172, 67, 235, 175, 143, 204, 122, 159] };
pub const TSATTRID_Font_Style_Strikethrough_Single: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1977038518, data2: 15503, data3: 19351, data4: [171, 120, 24, 119, 203, 153, 13, 49] };
pub const TSATTRID_Font_Style_Subscript: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1467284356, data2: 14491, data3: 17340, data4: [167, 75, 21, 104, 52, 124, 240, 244] };
pub const TSATTRID_Font_Style_Superscript: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 782539068, data2: 22076, data3: 18858, data4: [147, 114, 11, 239, 9, 169, 37, 91] };
pub const TSATTRID_Font_Style_Underline: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3284781555, data2: 30978, data3: 17483, data4: [154, 123, 72, 231, 15, 75, 80, 247] };
pub const TSATTRID_Font_Style_Underline_Double: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1959938726, data2: 7603, data3: 19561, data4: [161, 118, 49, 18, 14, 117, 134, 213] };
pub const TSATTRID_Font_Style_Underline_Single: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 459743461, data2: 3955, data3: 18769, data4: [166, 179, 111, 25, 228, 60, 148, 97] };
pub const TSATTRID_Font_Style_Uppercase: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 866320616, data2: 58176, data3: 18743, data4: [182, 151, 143, 35, 64, 69, 205, 154] };
pub const TSATTRID_Font_Style_Weight: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 317921436, data2: 35760, data3: 17947, data4: [177, 250, 234, 249, 7, 4, 127, 224] };
pub const TSATTRID_List: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1131243323, data2: 9969, data3: 19182, data4: [158, 101, 143, 131, 164, 237, 72, 132] };
pub const TSATTRID_List_LevelIndel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2138884249, data2: 12575, data3: 18555, data4: [173, 93, 226, 164, 89, 225, 45, 66] };
pub const TSATTRID_List_Type: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2923325022, data2: 19406, data3: 18915, data4: [160, 254, 45, 180, 125, 58, 23, 174] };
pub const TSATTRID_List_Type_Arabic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 322487766, data2: 39075, data3: 20387, data4: [155, 209, 122, 96, 238, 248, 233, 224] };
pub const TSATTRID_List_Type_Bullet: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3167582149, data2: 19533, data3: 19682, data4: [177, 2, 85, 159, 59, 43, 252, 234] };
pub const TSATTRID_List_Type_LowerLetter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2520195717, data2: 62415, data3: 18718, data4: [169, 37, 56, 50, 52, 127, 210, 55] };
pub const TSATTRID_List_Type_LowerRoman: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2420531810, data2: 14720, data3: 19342, data4: [147, 104, 145, 139, 209, 33, 138, 65] };
pub const TSATTRID_List_Type_UpperLetter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2038937549, data2: 52818, data3: 17035, data4: [155, 149, 163, 87, 246, 241, 12, 69] };
pub const TSATTRID_List_Type_UpperRoman: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 258651474, data2: 19072, data3: 18047, data4: [178, 241, 18, 126, 42, 163, 186, 158] };
pub const TSATTRID_OTHERS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3015912185, data2: 22480, data3: 18089, data4: [188, 168, 218, 194, 56, 161, 48, 87] };
pub const TSATTRID_Text: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2128318056, data2: 33273, data3: 17565, data4: [161, 90, 135, 168, 56, 143, 170, 192] };
pub const TSATTRID_Text_Alignment: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 328810982, data2: 5991, data3: 17773, data4: [147, 142, 53, 186, 86, 139, 92, 212] };
pub const TSATTRID_Text_Alignment_Center: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2762562582, data2: 21439, data3: 19797, data4: [139, 135, 75, 221, 141, 66, 117, 252] };
pub const TSATTRID_Text_Alignment_Justify: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3979675456, data2: 41207, data3: 17107, data4: [142, 168, 248, 27, 100, 136, 250, 240] };
pub const TSATTRID_Text_Alignment_Left: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 380540371, data2: 25441, data3: 17314, data4: [132, 149, 208, 15, 57, 127, 22, 147] };
pub const TSATTRID_Text_Alignment_Right: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3010400152, data2: 7070, data3: 17248, data4: [134, 22, 3, 251, 8, 167, 132, 86] };
pub const TSATTRID_Text_EmbeddedObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2128318056, data2: 33273, data3: 17565, data4: [161, 90, 135, 168, 56, 143, 170, 192] };
pub const TSATTRID_Text_Hyphenation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3672065317, data2: 24974, data3: 18923, data4: [177, 168, 59, 104, 189, 118, 72, 227] };
pub const TSATTRID_Text_Language: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3636481777, data2: 22355, data3: 19493, data4: [136, 135, 133, 68, 63, 229, 248, 25] };
pub const TSATTRID_Text_Link: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1204654161, data2: 14114, data3: 19672, data4: [183, 200, 78, 23, 202, 23, 89, 245] };
pub const TSATTRID_Text_Orientation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1806397567, data2: 34693, data3: 19513, data4: [139, 82, 150, 248, 120, 48, 63, 251] };
pub const TSATTRID_Text_Para: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1591498786, data2: 39388, data3: 19926, data4: [174, 195, 182, 43, 170, 91, 46, 124] };
pub const TSATTRID_Text_Para_FirstLineIndent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 130644499, data2: 29810, data3: 19928, data4: [144, 169, 145, 227, 215, 228, 242, 156] };
pub const TSATTRID_Text_Para_LeftIndent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4213721321, data2: 29809, data3: 16841, data4: [182, 179, 138, 20, 80, 224, 24, 151] };
pub const TSATTRID_Text_Para_LineSpacing: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1771780109, data2: 32652, data3: 18134, data4: [167, 59, 223, 227, 209, 83, 141, 243] };
pub const TSATTRID_Text_Para_LineSpacing_AtLeast: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2919161649, data2: 11588, data3: 17460, data4: [165, 255, 127, 76, 73, 144, 169, 5] };
pub const TSATTRID_Text_Para_LineSpacing_Double: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2197493765, data2: 42692, data3: 16945, data4: [172, 18, 98, 96, 175, 42, 186, 40] };
pub const TSATTRID_Text_Para_LineSpacing_Exactly: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1027976512, data2: 9182, data3: 18647, data4: [166, 179, 118, 84, 32, 198, 32, 204] };
pub const TSATTRID_Text_Para_LineSpacing_Multiple: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2433687100, data2: 54992, data3: 20325, data4: [138, 60, 66, 180, 179, 24, 104, 197] };
pub const TSATTRID_Text_Para_LineSpacing_OnePtFive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 69771297, data2: 919, data3: 19287, data4: [154, 23, 7, 149, 153, 76, 211, 197] };
pub const TSATTRID_Text_Para_LineSpacing_Single: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3979675456, data2: 41207, data3: 17107, data4: [142, 168, 248, 27, 100, 136, 250, 240] };
pub const TSATTRID_Text_Para_RightIndent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 746530553, data2: 42466, data3: 18650, data4: [185, 138, 82, 12, 177, 101, 19, 191] };
pub const TSATTRID_Text_Para_SpaceAfter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2064269141, data2: 8924, data3: 16991, data4: [164, 17, 147, 218, 29, 143, 155, 170] };
pub const TSATTRID_Text_Para_SpaceBefore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2381940105, data2: 6474, data3: 17921, data4: [178, 81, 152, 101, 163, 233, 6, 221] };
pub const TSATTRID_Text_ReadOnly: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2239981079, data2: 56882, data3: 19197, data4: [165, 15, 162, 219, 17, 14, 110, 77] };
pub const TSATTRID_Text_RightToLeft: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3395710577, data2: 6920, data3: 17725, data4: [191, 221, 40, 224, 140, 138, 175, 122] };
pub const TSATTRID_Text_VerticalWriting: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1807384981, data2: 1135, data3: 20137, data4: [179, 17, 151, 253, 102, 196, 39, 75] };
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AS_ATTR_CHANGE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AS_LAYOUT_CHANGE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AS_SEL_CHANGE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AS_STATUS_CHANGE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AS_TEXT_CHANGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ATTR_FIND_BACKWARDS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ATTR_FIND_HIDDEN: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ATTR_FIND_UPDATESTART: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ATTR_FIND_WANT_END: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ATTR_FIND_WANT_OFFSET: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ATTR_FIND_WANT_VALUE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_CHAR_EMBEDDED: u32 = 65532u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_CHAR_REGION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_CHAR_REPLACEMENT: u32 = 65533u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_FORMAT: ::windows_sys::core::HRESULT = -2147220982i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_INVALIDPOINT: ::windows_sys::core::HRESULT = -2147220985i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_INVALIDPOS: ::windows_sys::core::HRESULT = -2147220992i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_NOINTERFACE: ::windows_sys::core::HRESULT = -2147220988i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_NOLAYOUT: ::windows_sys::core::HRESULT = -2147220986i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_NOLOCK: ::windows_sys::core::HRESULT = -2147220991i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_NOOBJECT: ::windows_sys::core::HRESULT = -2147220990i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_NOSELECTION: ::windows_sys::core::HRESULT = -2147220987i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_NOSERVICE: ::windows_sys::core::HRESULT = -2147220989i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_READONLY: ::windows_sys::core::HRESULT = -2147220983i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_E_SYNCHRONOUS: ::windows_sys::core::HRESULT = -2147220984i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_GEA_HIDDEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_GTA_HIDDEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_IAS_NOQUERY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_IAS_QUERYONLY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_IE_COMPOSITION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_IE_CORRECTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_LF_SYNC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_EMBEDDEDHANDWRITINGVIEW_ENABLED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_EMBEDDEDHANDWRITINGVIEW_VISIBLE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_INPUTPANEMANUALDISPLAYENABLE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_LOADING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_READONLY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_RESERVED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_TKBAUTOCORRECTENABLE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_TKBPREDICTIONENABLE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_UIINTEGRATIONENABLE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SHIFT_COUNT_HIDDEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SHIFT_COUNT_ONLY: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SHIFT_HALT_HIDDEN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SHIFT_HALT_VISIBLE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_DISJOINTSEL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_NOHIDDENTEXT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_REGIONS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_TKBAUTOCORRECTENABLE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_TKBPREDICTIONENABLE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_TRANSITORY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SS_UWPCONTROL: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_STRF_END: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_STRF_MID: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_STRF_START: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_S_ASYNC: ::windows_sys::core::HRESULT = 262912i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_VCOOKIE_NUL: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type ANCHOR_CHANGE_HISTORY_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_CH_PRECEDING_DEL: ANCHOR_CHANGE_HISTORY_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_CH_FOLLOWING_DEL: ANCHOR_CHANGE_HISTORY_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type GET_TEXT_AND_PROPERTY_UPDATES_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_GTP_NONE: GET_TEXT_AND_PROPERTY_UPDATES_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_GTP_INCL_TEXT: GET_TEXT_AND_PROPERTY_UPDATES_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type INSERT_TEXT_AT_SELECTION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IAS_NOQUERY: INSERT_TEXT_AT_SELECTION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IAS_QUERYONLY: INSERT_TEXT_AT_SELECTION_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_IAS_NO_DEFAULT_COMPOSITION: INSERT_TEXT_AT_SELECTION_FLAGS = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type InputScope = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DEFAULT: InputScope = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_URL: InputScope = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_FILE_FULLFILEPATH: InputScope = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_FILE_FILENAME: InputScope = 3i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_EMAIL_USERNAME: InputScope = 4i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_EMAIL_SMTPEMAILADDRESS: InputScope = 5i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_LOGINNAME: InputScope = 6i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PERSONALNAME_FULLNAME: InputScope = 7i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PERSONALNAME_PREFIX: InputScope = 8i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PERSONALNAME_GIVENNAME: InputScope = 9i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PERSONALNAME_MIDDLENAME: InputScope = 10i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PERSONALNAME_SURNAME: InputScope = 11i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PERSONALNAME_SUFFIX: InputScope = 12i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_FULLPOSTALADDRESS: InputScope = 13i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_POSTALCODE: InputScope = 14i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_STREET: InputScope = 15i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_STATEORPROVINCE: InputScope = 16i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_CITY: InputScope = 17i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_COUNTRYNAME: InputScope = 18i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ADDRESS_COUNTRYSHORTNAME: InputScope = 19i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CURRENCY_AMOUNTANDSYMBOL: InputScope = 20i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CURRENCY_AMOUNT: InputScope = 21i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DATE_FULLDATE: InputScope = 22i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DATE_MONTH: InputScope = 23i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DATE_DAY: InputScope = 24i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DATE_YEAR: InputScope = 25i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DATE_MONTHNAME: InputScope = 26i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DATE_DAYNAME: InputScope = 27i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_DIGITS: InputScope = 28i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_NUMBER: InputScope = 29i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ONECHAR: InputScope = 30i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PASSWORD: InputScope = 31i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TELEPHONE_FULLTELEPHONENUMBER: InputScope = 32i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TELEPHONE_COUNTRYCODE: InputScope = 33i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TELEPHONE_AREACODE: InputScope = 34i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TELEPHONE_LOCALNUMBER: InputScope = 35i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TIME_FULLTIME: InputScope = 36i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TIME_HOUR: InputScope = 37i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TIME_MINORSEC: InputScope = 38i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_NUMBER_FULLWIDTH: InputScope = 39i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ALPHANUMERIC_HALFWIDTH: InputScope = 40i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ALPHANUMERIC_FULLWIDTH: InputScope = 41i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CURRENCY_CHINESE: InputScope = 42i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_BOPOMOFO: InputScope = 43i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_HIRAGANA: InputScope = 44i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_KATAKANA_HALFWIDTH: InputScope = 45i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_KATAKANA_FULLWIDTH: InputScope = 46i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_HANJA: InputScope = 47i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_HANGUL_HALFWIDTH: InputScope = 48i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_HANGUL_FULLWIDTH: InputScope = 49i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_SEARCH: InputScope = 50i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_FORMULA: InputScope = 51i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_SEARCH_INCREMENTAL: InputScope = 52i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CHINESE_HALFWIDTH: InputScope = 53i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CHINESE_FULLWIDTH: InputScope = 54i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_NATIVE_SCRIPT: InputScope = 55i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_YOMI: InputScope = 56i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_TEXT: InputScope = 57i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CHAT: InputScope = 58i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_NAME_OR_PHONENUMBER: InputScope = 59i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_EMAILNAME_OR_ADDRESS: InputScope = 60i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PRIVATE: InputScope = 61i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_MAPS: InputScope = 62i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_NUMERIC_PASSWORD: InputScope = 63i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_NUMERIC_PIN: InputScope = 64i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ALPHANUMERIC_PIN: InputScope = 65i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ALPHANUMERIC_PIN_SET: InputScope = 66i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_FORMULA_NUMBER: InputScope = 67i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_CHAT_WITHOUT_EMOJI: InputScope = 68i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_PHRASELIST: InputScope = -1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_REGULAREXPRESSION: InputScope = -2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_SRGS: InputScope = -3i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_XML: InputScope = -4i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const IS_ENUMSTRING: InputScope = -5i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type LANG_BAR_ITEM_ICON_MODE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DTLBI_NONE: LANG_BAR_ITEM_ICON_MODE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_DTLBI_USEPROFILEICON: LANG_BAR_ITEM_ICON_MODE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TEXT_STORE_CHANGE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_TC_NONE: TEXT_STORE_CHANGE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_TC_CORRECTION: TEXT_STORE_CHANGE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TEXT_STORE_LOCK_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_LF_READ: TEXT_STORE_LOCK_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_LF_READWRITE: TEXT_STORE_LOCK_FLAGS = 6u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TEXT_STORE_TEXT_CHANGE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ST_NONE: TEXT_STORE_TEXT_CHANGE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_ST_CORRECTION: TEXT_STORE_TEXT_CHANGE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TF_CONTEXT_EDIT_CONTEXT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ES_ASYNCDONTCARE: TF_CONTEXT_EDIT_CONTEXT_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ES_SYNC: TF_CONTEXT_EDIT_CONTEXT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ES_READ: TF_CONTEXT_EDIT_CONTEXT_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ES_READWRITE: TF_CONTEXT_EDIT_CONTEXT_FLAGS = 6u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ES_ASYNC: TF_CONTEXT_EDIT_CONTEXT_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TF_DA_ATTR_INFO = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_INPUT: TF_DA_ATTR_INFO = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_TARGET_CONVERTED: TF_DA_ATTR_INFO = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_CONVERTED: TF_DA_ATTR_INFO = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_TARGET_NOTCONVERTED: TF_DA_ATTR_INFO = 3i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_INPUT_ERROR: TF_DA_ATTR_INFO = 4i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_FIXEDCONVERTED: TF_DA_ATTR_INFO = 5i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ATTR_OTHER: TF_DA_ATTR_INFO = -1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TF_DA_COLORTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CT_NONE: TF_DA_COLORTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CT_SYSCOLOR: TF_DA_COLORTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_CT_COLORREF: TF_DA_COLORTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TF_DA_LINESTYLE = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LS_NONE: TF_DA_LINESTYLE = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LS_SOLID: TF_DA_LINESTYLE = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LS_DOT: TF_DA_LINESTYLE = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LS_DASH: TF_DA_LINESTYLE = 3i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LS_SQUIGGLE: TF_DA_LINESTYLE = 4i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TKBLayoutType = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBLT_UNDEFINED: TKBLayoutType = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBLT_CLASSIC: TKBLayoutType = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TKBLT_OPTIMIZED: TKBLayoutType = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfActiveSelEnd = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_AE_NONE: TfActiveSelEnd = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_AE_START: TfActiveSelEnd = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_AE_END: TfActiveSelEnd = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfAnchor = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ANCHOR_START: TfAnchor = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_ANCHOR_END: TfAnchor = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfCandidateResult = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const CAND_FINALIZED: TfCandidateResult = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const CAND_SELECTED: TfCandidateResult = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const CAND_CANCELED: TfCandidateResult = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfGravity = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_GRAVITY_BACKWARD: TfGravity = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_GRAVITY_FORWARD: TfGravity = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfIntegratableCandidateListSelectionStyle = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const STYLE_ACTIVE_SELECTION: TfIntegratableCandidateListSelectionStyle = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const STYLE_IMPLIED_SELECTION: TfIntegratableCandidateListSelectionStyle = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfLBBalloonStyle = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LB_BALLOON_RECO: TfLBBalloonStyle = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LB_BALLOON_SHOW: TfLBBalloonStyle = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LB_BALLOON_MISS: TfLBBalloonStyle = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfLBIClick = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_CLK_RIGHT: TfLBIClick = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LBI_CLK_LEFT: TfLBIClick = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfLayoutCode = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LC_CREATE: TfLayoutCode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LC_CHANGE: TfLayoutCode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_LC_DESTROY: TfLayoutCode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfSapiObject = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GETIF_RESMGR: TfSapiObject = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GETIF_RECOCONTEXT: TfSapiObject = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GETIF_RECOGNIZER: TfSapiObject = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GETIF_VOICE: TfSapiObject = 3i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GETIF_DICTGRAM: TfSapiObject = 4i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const GETIF_RECOGNIZERNOINIT: TfSapiObject = 5i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TfShiftDir = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SD_BACKWARD: TfShiftDir = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TF_SD_FORWARD: TfShiftDir = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TsActiveSelEnd = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AE_NONE: TsActiveSelEnd = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AE_START: TsActiveSelEnd = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_AE_END: TsActiveSelEnd = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TsGravity = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_GR_BACKWARD: TsGravity = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_GR_FORWARD: TsGravity = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TsLayoutCode = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_LC_CREATE: TsLayoutCode = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_LC_CHANGE: TsLayoutCode = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_LC_DESTROY: TsLayoutCode = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TsRunType = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_RT_PLAIN: TsRunType = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_RT_HIDDEN: TsRunType = 1i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_RT_OPAQUE: TsRunType = 2i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub type TsShiftDir = i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_BACKWARD: TsShiftDir = 0i32;
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub const TS_SD_FORWARD: TsShiftDir = 1i32;
pub type HKL = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TF_DA_COLOR {
    pub r#type: TF_DA_COLORTYPE,
    pub Anonymous: TF_DA_COLOR_0,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TF_DA_COLOR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TF_DA_COLOR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union TF_DA_COLOR_0 {
    pub nIndex: i32,
    pub cr: super::super::Foundation::COLORREF,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TF_DA_COLOR_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TF_DA_COLOR_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TF_DISPLAYATTRIBUTE {
    pub crText: TF_DA_COLOR,
    pub crBk: TF_DA_COLOR,
    pub lsStyle: TF_DA_LINESTYLE,
    pub fBoldLine: super::super::Foundation::BOOL,
    pub crLine: TF_DA_COLOR,
    pub bAttr: TF_DA_ATTR_INFO,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TF_DISPLAYATTRIBUTE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TF_DISPLAYATTRIBUTE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_HALTCOND {
    pub pHaltRange: ITfRange,
    pub aHaltPos: TfAnchor,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for TF_HALTCOND {}
impl ::core::clone::Clone for TF_HALTCOND {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_INPUTPROCESSORPROFILE {
    pub dwProfileType: u32,
    pub langid: u16,
    pub clsid: ::windows_sys::core::GUID,
    pub guidProfile: ::windows_sys::core::GUID,
    pub catid: ::windows_sys::core::GUID,
    pub hklSubstitute: HKL,
    pub dwCaps: u32,
    pub hkl: HKL,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for TF_INPUTPROCESSORPROFILE {}
impl ::core::clone::Clone for TF_INPUTPROCESSORPROFILE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_LANGBARITEMINFO {
    pub clsidService: ::windows_sys::core::GUID,
    pub guidItem: ::windows_sys::core::GUID,
    pub dwStyle: u32,
    pub ulSort: u32,
    pub szDescription: [u16; 32],
}
impl ::core::marker::Copy for TF_LANGBARITEMINFO {}
impl ::core::clone::Clone for TF_LANGBARITEMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TF_LANGUAGEPROFILE {
    pub clsid: ::windows_sys::core::GUID,
    pub langid: u16,
    pub catid: ::windows_sys::core::GUID,
    pub fActive: super::super::Foundation::BOOL,
    pub guidProfile: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TF_LANGUAGEPROFILE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TF_LANGUAGEPROFILE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_LBBALLOONINFO {
    pub style: TfLBBalloonStyle,
    pub bstrText: ::windows_sys::core::BSTR,
}
impl ::core::marker::Copy for TF_LBBALLOONINFO {}
impl ::core::clone::Clone for TF_LBBALLOONINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_LMLATTELEMENT {
    pub dwFrameStart: u32,
    pub dwFrameLen: u32,
    pub dwFlags: u32,
    pub Anonymous: TF_LMLATTELEMENT_0,
    pub bstrText: ::windows_sys::core::BSTR,
}
impl ::core::marker::Copy for TF_LMLATTELEMENT {}
impl ::core::clone::Clone for TF_LMLATTELEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub union TF_LMLATTELEMENT_0 {
    pub iCost: i32,
}
impl ::core::marker::Copy for TF_LMLATTELEMENT_0 {}
impl ::core::clone::Clone for TF_LMLATTELEMENT_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_PERSISTENT_PROPERTY_HEADER_ACP {
    pub guidType: ::windows_sys::core::GUID,
    pub ichStart: i32,
    pub cch: i32,
    pub cb: u32,
    pub dwPrivate: u32,
    pub clsidTIP: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for TF_PERSISTENT_PROPERTY_HEADER_ACP {}
impl ::core::clone::Clone for TF_PERSISTENT_PROPERTY_HEADER_ACP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TF_PRESERVEDKEY {
    pub uVKey: u32,
    pub uModifiers: u32,
}
impl ::core::marker::Copy for TF_PRESERVEDKEY {}
impl ::core::clone::Clone for TF_PRESERVEDKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub struct TF_PROPERTYVAL {
    pub guidId: ::windows_sys::core::GUID,
    pub varValue: super::super::System::Com::VARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for TF_PROPERTYVAL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for TF_PROPERTYVAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TF_SELECTION {
    pub range: ITfRange,
    pub style: TF_SELECTIONSTYLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TF_SELECTION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TF_SELECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TF_SELECTIONSTYLE {
    pub ase: TfActiveSelEnd,
    pub fInterimChar: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TF_SELECTIONSTYLE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TF_SELECTIONSTYLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub struct TS_ATTRVAL {
    pub idAttr: ::windows_sys::core::GUID,
    pub dwOverlapId: u32,
    pub varValue: super::super::System::Com::VARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for TS_ATTRVAL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for TS_ATTRVAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TS_RUNINFO {
    pub uCount: u32,
    pub r#type: TsRunType,
}
impl ::core::marker::Copy for TS_RUNINFO {}
impl ::core::clone::Clone for TS_RUNINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TS_SELECTIONSTYLE {
    pub ase: TsActiveSelEnd,
    pub fInterimChar: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TS_SELECTIONSTYLE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TS_SELECTIONSTYLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TS_SELECTION_ACP {
    pub acpStart: i32,
    pub acpEnd: i32,
    pub style: TS_SELECTIONSTYLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TS_SELECTION_ACP {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TS_SELECTION_ACP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TS_SELECTION_ANCHOR {
    pub paStart: IAnchor,
    pub paEnd: IAnchor,
    pub style: TS_SELECTIONSTYLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TS_SELECTION_ANCHOR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TS_SELECTION_ANCHOR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TS_STATUS {
    pub dwDynamicFlags: u32,
    pub dwStaticFlags: u32,
}
impl ::core::marker::Copy for TS_STATUS {}
impl ::core::clone::Clone for TS_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_TextServices\"`*"]
pub struct TS_TEXTCHANGE {
    pub acpStart: i32,
    pub acpOldEnd: i32,
    pub acpNewEnd: i32,
}
impl ::core::marker::Copy for TS_TEXTCHANGE {}
impl ::core::clone::Clone for TS_TEXTCHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
