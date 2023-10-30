#[cfg(feature = "Win32_Graphics_Imaging_D2D")]
pub mod D2D;
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
    pub fn WICConvertBitmapSource(dstformat: *const ::windows_sys::core::GUID, pisrc: IWICBitmapSource, ppidst: *mut IWICBitmapSource) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WICCreateBitmapFromSection(width: u32, height: u32, pixelformat: *const ::windows_sys::core::GUID, hsection: super::super::Foundation::HANDLE, stride: u32, offset: u32, ppibitmap: *mut IWICBitmap) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WICCreateBitmapFromSectionEx(width: u32, height: u32, pixelformat: *const ::windows_sys::core::GUID, hsection: super::super::Foundation::HANDLE, stride: u32, offset: u32, desiredaccesslevel: WICSectionAccessLevel, ppibitmap: *mut IWICBitmap) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
    pub fn WICGetMetadataContentSize(guidcontainerformat: *const ::windows_sys::core::GUID, piwriter: IWICMetadataWriter, pcbsize: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
    pub fn WICMapGuidToShortName(guid: *const ::windows_sys::core::GUID, cchname: u32, wzname: ::windows_sys::core::PWSTR, pcchactual: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
    pub fn WICMapSchemaToName(guidmetadataformat: *const ::windows_sys::core::GUID, pwzschema: ::windows_sys::core::PCWSTR, cchname: u32, wzname: ::windows_sys::core::PWSTR, pcchactual: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
    pub fn WICMapShortNameToGuid(wzname: ::windows_sys::core::PCWSTR, pguid: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn WICMatchMetadataContent(guidcontainerformat: *const ::windows_sys::core::GUID, pguidvendor: *const ::windows_sys::core::GUID, pistream: super::super::System::Com::IStream, pguidmetadataformat: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn WICSerializeMetadataContent(guidcontainerformat: *const ::windows_sys::core::GUID, piwriter: IWICMetadataWriter, dwpersistoptions: u32, pistream: super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
}
pub const CATID_WICBitmapDecoders: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2128177207, data2: 38640, data3: 18450, data4: [178, 17, 241, 60, 36, 17, 126, 211] };
pub const CATID_WICBitmapEncoders: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2893378198, data2: 13602, data3: 19985, data4: [152, 98, 193, 123, 229, 161, 118, 126] };
pub const CATID_WICFormatConverters: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2016799464, data2: 48916, data3: 18897, data4: [147, 206, 83, 58, 64, 123, 34, 72] };
pub const CATID_WICMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 95392984, data2: 29044, data3: 19666, data4: [190, 74, 65, 36, 184, 14, 228, 184] };
pub const CATID_WICMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2883828132, data2: 9597, data3: 19351, data4: [189, 26, 41, 74, 244, 150, 34, 46] };
pub const CATID_WICPixelFormats: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 726066959, data2: 52647, data3: 18238, data4: [137, 246, 220, 150, 48, 162, 57, 11] };
pub const CLSID_WIC8BIMIPTCDigestMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 41967390, data2: 54698, data3: 16731, data4: [130, 197, 97, 192, 51, 169, 136, 166] };
pub const CLSID_WIC8BIMIPTCDigestMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 766895659, data2: 3431, data3: 18783, data4: [143, 157, 194, 240, 24, 134, 71, 172] };
pub const CLSID_WIC8BIMIPTCMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1074828, data2: 2049, data3: 19878, data4: [164, 164, 130, 101, 34, 182, 210, 143] };
pub const CLSID_WIC8BIMIPTCMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1081894, data2: 60993, data3: 17570, data4: [158, 156, 75, 228, 213, 177, 210, 205] };
pub const CLSID_WIC8BIMResolutionInfoMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1476727674, data2: 58184, data3: 20348, data4: [179, 204, 109, 185, 150, 90, 5, 153] };
pub const CLSID_WIC8BIMResolutionInfoMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1341324814, data2: 59210, data3: 19313, data4: [152, 196, 171, 125, 193, 103, 7, 186] };
pub const CLSID_WICAPEMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 392673594, data2: 45089, data3: 17642, data4: [146, 15, 134, 60, 17, 244, 247, 104] };
pub const CLSID_WICAPEMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3178160074, data2: 10384, data3: 18479, data4: [178, 51, 141, 115, 57, 161, 207, 141] };
pub const CLSID_WICAdngDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2552075281, data2: 37022, data3: 17063, data4: [143, 93, 167, 71, 255, 5, 46, 219] };
pub const CLSID_WICApp0MetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1127369523, data2: 42895, data3: 18447, data4: [145, 17, 150, 56, 170, 204, 200, 50] };
pub const CLSID_WICApp0MetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4089852834, data2: 18120, data3: 18830, data4: [143, 187, 204, 111, 114, 27, 188, 222] };
pub const CLSID_WICApp13MetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2860399696, data2: 34380, data3: 17924, data4: [188, 4, 139, 11, 118, 230, 55, 246] };
pub const CLSID_WICApp13MetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2065279257, data2: 43478, data3: 18917, data4: [189, 69, 2, 195, 78, 78, 76, 213] };
pub const CLSID_WICApp1MetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3722654995, data2: 30542, data3: 19405, data4: [174, 121, 2, 244, 173, 254, 98, 252] };
pub const CLSID_WICApp1MetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3996541033, data2: 6194, data3: 16911, data4: [179, 129, 4, 121, 173, 6, 111, 25] };
pub const CLSID_WICBmpDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1799757922, data2: 31935, data3: 16397, data4: [159, 219, 129, 61, 209, 15, 39, 120] };
pub const CLSID_WICBmpEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1774095284, data2: 54893, data3: 18376, data4: [134, 90, 237, 21, 137, 67, 55, 130] };
pub const CLSID_WICDdsDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2421385631, data2: 41793, data3: 17053, data4: [158, 144, 238, 67, 124, 248, 12, 115] };
pub const CLSID_WICDdsEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2786975380, data2: 26318, data3: 19137, data4: [136, 27, 113, 104, 5, 136, 137, 94] };
pub const CLSID_WICDdsMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 661424330, data2: 30003, data3: 19078, data4: [182, 118, 102, 179, 96, 128, 212, 132] };
pub const CLSID_WICDdsMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4251487165, data2: 12781, data3: 19895, data4: [167, 35, 147, 73, 39, 211, 131, 103] };
pub const CLSID_WICDefaultFormatConverter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 440340956, data2: 46356, data3: 19223, data4: [140, 95, 33, 84, 81, 56, 82, 241] };
pub const CLSID_WICExifMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3644864608, data2: 10623, data3: 19017, data4: [191, 155, 119, 137, 129, 80, 164, 66] };
pub const CLSID_WICExifMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3382791386, data2: 49977, data3: 17931, data4: [144, 120, 212, 222, 188, 250, 190, 145] };
pub const CLSID_WICFormatConverterHighColor: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2893403220, data2: 40759, data3: 18680, data4: [185, 114, 78, 25, 188, 133, 96, 17] };
pub const CLSID_WICFormatConverterNChannel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3246173106, data2: 54435, data3: 18391, data4: [165, 87, 51, 155, 46, 251, 212, 241] };
pub const CLSID_WICFormatConverterWMPhoto: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2629113643, data2: 54784, data3: 18106, data4: [171, 119, 119, 187, 126, 58, 0, 217] };
pub const CLSID_WICGCEMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3106813021, data2: 62765, data3: 16883, data4: [181, 98, 8, 27, 199, 114, 227, 185] };
pub const CLSID_WICGCEMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2945834102, data2: 5810, data3: 18420, data4: [179, 234, 60, 49, 121, 102, 147, 231] };
pub const CLSID_WICGifCommentMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 844463419, data2: 27100, data3: 20373, data4: [131, 110, 245, 151, 43, 47, 97, 89] };
pub const CLSID_WICGifCommentMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2686949372, data2: 50350, data3: 16780, data4: [175, 149, 230, 55, 199, 234, 210, 161] };
pub const CLSID_WICGifDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 941480508, data2: 40169, data3: 18484, data4: [162, 62, 31, 152, 248, 252, 82, 190] };
pub const CLSID_WICGifEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 290411928, data2: 2850, data3: 16544, data4: [134, 161, 200, 62, 164, 149, 173, 189] };
pub const CLSID_WICGpsMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 915896587, data2: 8763, data3: 18510, data4: [153, 37, 196, 134, 146, 24, 241, 122] };
pub const CLSID_WICGpsMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3414954980, data2: 25269, data3: 19606, data4: [164, 139, 107, 166, 172, 227, 156, 118] };
pub const CLSID_WICHeifDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3919882250, data2: 17662, data3: 19940, data4: [137, 113, 113, 80, 177, 10, 81, 153] };
pub const CLSID_WICHeifEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 230608577, data2: 40627, data3: 18528, data4: [156, 111, 221, 190, 134, 99, 69, 117] };
pub const CLSID_WICHeifHDRMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 607706685, data2: 38105, data3: 19432, data4: [132, 168, 77, 233, 90, 87, 94, 117] };
pub const CLSID_WICHeifMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2900229183, data2: 34284, data3: 16828, data4: [189, 239, 27, 194, 98, 228, 219, 5] };
pub const CLSID_WICHeifMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 988044921, data2: 16572, data3: 17409, data4: [172, 229, 221, 60, 177, 110, 106, 254] };
pub const CLSID_WICIMDMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1950851687, data2: 21, data3: 17096, data4: [168, 241, 251, 59, 148, 198, 131, 97] };
pub const CLSID_WICIMDMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2357790495, data2: 17710, data3: 20117, data4: [150, 130, 157, 16, 36, 98, 113, 114] };
pub const CLSID_WICIPTCMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 50407769, data2: 62710, data3: 17623, data4: [157, 9, 218, 160, 135, 169, 219, 87] };
pub const CLSID_WICIPTCMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 306819596, data2: 24016, data3: 17662, data4: [176, 179, 143, 146, 200, 230, 208, 128] };
pub const CLSID_WICIRBMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3571241943, data2: 46274, data3: 18393, data4: [166, 191, 184, 155, 163, 150, 164, 163] };
pub const CLSID_WICIRBMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1549539637, data2: 565, data3: 17460, data4: [128, 188, 37, 27, 193, 236, 57, 198] };
pub const CLSID_WICIcoDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3323722975, data2: 11791, data3: 19117, data4: [168, 215, 224, 107, 175, 235, 205, 254] };
pub const CLSID_WICIfdMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2408662614, data2: 40202, data3: 20146, data4: [144, 25, 11, 249, 109, 138, 158, 230] };
pub const CLSID_WICIfdMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2985032744, data2: 51645, data3: 18338, data4: [141, 51, 185, 72, 118, 151, 119, 167] };
pub const CLSID_WICImagingCategories: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4209234816, data2: 65188, data3: 17955, data4: [140, 117, 198, 182, 17, 16, 182, 129] };
pub const CLSID_WICImagingFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3402297954, data2: 37744, data3: 17941, data4: [161, 59, 159, 85, 57, 218, 76, 10] };
pub const CLSID_WICImagingFactory1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3402297954, data2: 37744, data3: 17941, data4: [161, 59, 159, 85, 57, 218, 76, 10] };
pub const CLSID_WICImagingFactory2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 830277352, data2: 24356, data3: 17213, data4: [189, 247, 121, 206, 104, 216, 171, 194] };
pub const CLSID_WICInteropMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3049830552, data2: 116, data3: 17823, data4: [183, 0, 134, 13, 70, 81, 234, 20] };
pub const CLSID_WICInteropMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 305055301, data2: 52606, data3: 17624, data4: [177, 134, 44, 140, 32, 195, 181, 15] };
pub const CLSID_WICJpegChrominanceMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1353814091, data2: 62095, data3: 17780, data4: [147, 244, 11, 173, 232, 44, 105, 233] };
pub const CLSID_WICJpegChrominanceMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1073047280, data2: 28267, data3: 18900, data4: [150, 230, 183, 136, 134, 105, 44, 98] };
pub const CLSID_WICJpegCommentMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2674275452, data2: 24772, data3: 19533, data4: [171, 88, 210, 53, 134, 133, 246, 7] };
pub const CLSID_WICJpegCommentMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3849528175, data2: 21937, data3: 20186, data4: [129, 234, 159, 101, 219, 2, 144, 211] };
pub const CLSID_WICJpegDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2488706176, data2: 59531, data3: 17386, data4: [158, 115, 11, 45, 155, 113, 177, 202] };
pub const CLSID_WICJpegEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 439678401, data2: 19034, data3: 18140, data4: [182, 68, 31, 69, 103, 231, 166, 118] };
pub const CLSID_WICJpegLuminanceMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 896479112, data2: 1446, data3: 18216, data4: [185, 164, 27, 251, 206, 4, 216, 56] };
pub const CLSID_WICJpegLuminanceMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 492321468, data2: 35342, data3: 18007, data4: [153, 130, 163, 128, 202, 88, 251, 75] };
pub const CLSID_WICJpegQualcommPhoneEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1760386146, data2: 62772, data3: 18809, data4: [178, 179, 104, 106, 18, 178, 179, 76] };
pub const CLSID_WICLSDMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1090979731, data2: 23012, data3: 18330, data4: [161, 247, 149, 74, 220, 46, 245, 252] };
pub const CLSID_WICLSDMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1941977063, data2: 58841, data3: 18772, data4: [135, 106, 109, 168, 29, 110, 87, 104] };
pub const CLSID_WICPlanarFormatConverter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 406926008, data2: 13048, data3: 18308, data4: [145, 49, 221, 114, 36, 178, 52, 56] };
pub const CLSID_WICPngBkgdMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 216507558, data2: 1000, data3: 19040, data4: [157, 21, 40, 46, 243, 46, 231, 218] };
pub const CLSID_WICPngBkgdMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1759769341, data2: 12718, data3: 17473, data4: [187, 106, 253, 112, 71, 82, 95, 144] };
pub const CLSID_WICPngChrmMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4178272054, data2: 13947, data3: 16426, data4: [157, 209, 188, 15, 213, 157, 143, 98] };
pub const CLSID_WICPngChrmMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3795641323, data2: 22024, data3: 20099, data4: [188, 239, 39, 177, 152, 126, 81, 215] };
pub const CLSID_WICPngDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 949920123, data2: 20600, data3: 19678, data4: [182, 239, 37, 193, 81, 117, 199, 81] };
pub const CLSID_WICPngDecoder1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 949920123, data2: 20600, data3: 19678, data4: [182, 239, 37, 193, 81, 117, 199, 81] };
pub const CLSID_WICPngDecoder2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3759707227, data2: 43654, data3: 16392, data4: [155, 212, 103, 119, 161, 228, 12, 17] };
pub const CLSID_WICPngEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 664050025, data2: 34666, data3: 16855, data4: [148, 71, 86, 143, 106, 53, 164, 220] };
pub const CLSID_WICPngGamaMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 915589689, data2: 57474, data3: 17232, data4: [158, 31, 55, 4, 203, 8, 60, 213] };
pub const CLSID_WICPngGamaMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4278414611, data2: 23883, data3: 18141, data4: [177, 15, 16, 102, 147, 217, 254, 79] };
pub const CLSID_WICPngHistMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2272922551, data2: 41747, data3: 17553, data4: [135, 181, 46, 109, 5, 148, 245, 32] };
pub const CLSID_WICPngHistMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2315511625, data2: 26414, data3: 17518, data4: [191, 31, 44, 17, 210, 51, 182, 255] };
pub const CLSID_WICPngIccpMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4124304955, data2: 51983, data3: 17960, data4: [164, 120, 109, 130, 68, 190, 54, 177] };
pub const CLSID_WICPngIccpMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 375856735, data2: 3302, data3: 19652, data4: [151, 104, 232, 159, 229, 1, 138, 222] };
pub const CLSID_WICPngItxtMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2864689914, data2: 15902, data3: 19087, data4: [137, 119, 85, 86, 251, 148, 234, 35] };
pub const CLSID_WICPngItxtMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 830969625, data2: 59217, data3: 19960, data4: [152, 29, 104, 223, 246, 119, 4, 237] };
pub const CLSID_WICPngSrgbMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4215289356, data2: 21630, data3: 18774, data4: [163, 185, 212, 65, 136, 89, 186, 102] };
pub const CLSID_WICPngSrgbMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2800629190, data2: 34796, data3: 18399, data4: [159, 34, 29, 90, 173, 132, 12, 130] };
pub const CLSID_WICPngTextMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1264168908, data2: 47299, data3: 16522, data4: [182, 112, 137, 229, 250, 182, 253, 167] };
pub const CLSID_WICPngTextMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3052122041, data2: 9534, data3: 19058, data4: [167, 68, 7, 98, 210, 104, 86, 131] };
pub const CLSID_WICPngTimeMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3645824770, data2: 61413, data3: 20237, data4: [133, 200, 245, 166, 139, 48, 0, 177] };
pub const CLSID_WICPngTimeMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 448234496, data2: 46499, data3: 19857, data4: [138, 206, 51, 252, 209, 73, 155, 230] };
pub const CLSID_WICRAWDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1100240642, data2: 33538, data3: 17574, data4: [148, 69, 172, 152, 232, 175, 160, 134] };
pub const CLSID_WICSubIfdMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1356082953, data2: 60625, data3: 19265, data4: [182, 93, 218, 31, 218, 167, 86, 99] };
pub const CLSID_WICSubIfdMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2329826182, data2: 36507, data3: 20300, data4: [172, 242, 240, 0, 135, 6, 178, 56] };
pub const CLSID_WICThumbnailMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4211157337, data2: 62710, data3: 17623, data4: [157, 9, 218, 160, 135, 169, 219, 87] };
pub const CLSID_WICThumbnailMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3494490636, data2: 24016, data3: 17662, data4: [176, 179, 143, 146, 200, 230, 208, 128] };
pub const CLSID_WICTiffDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3041822169, data2: 65059, data3: 18847, data4: [139, 136, 106, 206, 167, 19, 117, 43] };
pub const CLSID_WICTiffEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 20037136, data2: 8193, data3: 19551, data4: [169, 176, 204, 136, 250, 182, 76, 232] };
pub const CLSID_WICUnknownMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1771521474, data2: 20582, data3: 19330, data4: [168, 227, 212, 4, 120, 219, 236, 140] };
pub const CLSID_WICUnknownMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2694630022, data2: 10170, data3: 20281, data4: [144, 83, 18, 31, 164, 220, 8, 252] };
pub const CLSID_WICWebpAnimMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 124754193, data2: 41800, data3: 18012, data4: [168, 7, 162, 82, 243, 242, 211, 222] };
pub const CLSID_WICWebpAnmfMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2241923843, data2: 51702, data3: 17311, data4: [190, 94, 192, 251, 239, 103, 128, 124] };
pub const CLSID_WICWebpDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1989404806, data2: 20937, data3: 16496, data4: [132, 25, 159, 112, 115, 142, 200, 250] };
pub const CLSID_WICWmpDecoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2725047350, data2: 9036, data3: 18768, data4: [174, 22, 227, 74, 172, 231, 29, 13] };
pub const CLSID_WICWmpEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2890720203, data2: 57793, data3: 17613, data4: [130, 21, 90, 22, 101, 80, 158, 194] };
pub const CLSID_WICXMPAltMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2861882562, data2: 47280, data3: 18584, data4: [184, 53, 0, 10, 171, 215, 67, 147] };
pub const CLSID_WICXMPAltMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 124529260, data2: 63375, data3: 19526, data4: [167, 35, 53, 131, 231, 8, 118, 234] };
pub const CLSID_WICXMPBagMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3890715184, data2: 20268, data3: 20395, data4: [141, 0, 57, 79, 45, 107, 190, 190] };
pub const CLSID_WICXMPBagMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3984731276, data2: 54974, data3: 17153, data4: [166, 49, 14, 20, 22, 186, 210, 143] };
pub const CLSID_WICXMPMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1924539615, data2: 44561, data3: 18760, data4: [166, 92, 53, 30, 176, 130, 148, 25] };
pub const CLSID_WICXMPMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 392552782, data2: 7124, data3: 17966, data4: [182, 177, 89, 11, 241, 38, 42, 198] };
pub const CLSID_WICXMPSeqMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2131945299, data2: 64625, data3: 17367, data4: [165, 29, 146, 243, 89, 119, 171, 181] };
pub const CLSID_WICXMPSeqMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1835586014, data2: 54322, data3: 19215, data4: [146, 58, 9, 17, 131, 169, 189, 167] };
pub const CLSID_WICXMPStructMetadataReader: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 28904858, data2: 33289, data3: 18423, data4: [156, 82, 225, 36, 75, 245, 12, 237] };
pub const CLSID_WICXMPStructMetadataWriter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 583147411, data2: 32219, data3: 16668, data4: [155, 23, 197, 183, 189, 6, 74, 188] };
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const FACILITY_WINCODEC_ERR: u32 = 2200u32;
pub const GUID_ContainerFormatAdng: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4093603085, data2: 14528, data3: 16836, data4: [177, 254, 31, 56, 36, 241, 123, 132] };
pub const GUID_ContainerFormatBmp: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 183621758, data2: 64766, data3: 16776, data4: [189, 235, 167, 144, 100, 113, 203, 227] };
pub const GUID_ContainerFormatDds: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2573716373, data2: 11909, data3: 19144, data4: [140, 162, 131, 215, 204, 212, 37, 201] };
pub const GUID_ContainerFormatGif: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 529159681, data2: 32077, data3: 19645, data4: [156, 130, 27, 200, 212, 238, 185, 165] };
pub const GUID_ContainerFormatHeif: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3789956385, data2: 26503, data3: 16475, data4: [163, 57, 80, 7, 21, 181, 118, 63] };
pub const GUID_ContainerFormatIco: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2745721028, data2: 13199, data3: 19479, data4: [145, 154, 251, 164, 181, 98, 143, 33] };
pub const GUID_ContainerFormatJpeg: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 434415018, data2: 22114, data3: 20421, data4: [160, 192, 23, 88, 2, 142, 16, 87] };
pub const GUID_ContainerFormatPng: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461175540, data2: 28991, data3: 18236, data4: [187, 205, 97, 55, 66, 95, 174, 175] };
pub const GUID_ContainerFormatRaw: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4271492704, data2: 61852, data3: 17212, data4: [163, 174, 0, 172, 239, 169, 202, 33] };
pub const GUID_ContainerFormatTiff: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 373017648, data2: 58089, data3: 20235, data4: [150, 29, 163, 233, 253, 183, 136, 163] };
pub const GUID_ContainerFormatWebp: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3767840994, data2: 26610, data3: 17843, data4: [176, 234, 17, 83, 55, 202, 124, 243] };
pub const GUID_ContainerFormatWmp: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1470332074, data2: 13946, data3: 17728, data4: [145, 107, 241, 131, 197, 9, 58, 75] };
pub const GUID_MetadataFormat8BIMIPTC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1070732, data2: 2130, data3: 20074, data4: [177, 145, 92, 51, 172, 91, 4, 48] };
pub const GUID_MetadataFormat8BIMIPTCDigest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 480453253, data2: 40141, data3: 18310, data4: [139, 216, 121, 83, 157, 182, 160, 6] };
pub const GUID_MetadataFormat8BIMResolutionInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1939812445, data2: 33243, data3: 17355, data4: [172, 94, 85, 1, 62, 249, 240, 3] };
pub const GUID_MetadataFormatAPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 772029890, data2: 51559, data3: 19973, data4: [135, 94, 97, 139, 246, 126, 133, 195] };
pub const GUID_MetadataFormatApp0: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2030071848, data2: 9869, data3: 17878, data4: [163, 194, 53, 78, 106, 80, 75, 201] };
pub const GUID_MetadataFormatApp1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2413027267, data2: 63825, data3: 18731, data4: [129, 127, 105, 194, 230, 217, 165, 176] };
pub const GUID_MetadataFormatApp13: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 845502114, data2: 62722, data3: 17236, data4: [156, 192, 142, 63, 72, 234, 246, 181] };
pub const GUID_MetadataFormatChunkbKGD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3779933553, data2: 27463, data3: 19946, data4: [182, 10, 135, 206, 10, 120, 223, 183] };
pub const GUID_MetadataFormatChunkcHRM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2645779803, data2: 10306, data3: 17587, data4: [128, 103, 18, 233, 179, 117, 85, 106] };
pub const GUID_MetadataFormatChunkgAMA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4027135397, data2: 7517, data3: 19665, data4: [129, 178, 147, 36, 215, 236, 167, 129] };
pub const GUID_MetadataFormatChunkhIST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3315237594, data2: 56180, data3: 18596, data4: [189, 106, 182, 156, 73, 49, 239, 149] };
pub const GUID_MetadataFormatChunkiCCP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3947055531, data2: 46725, data3: 17679, data4: [145, 181, 232, 2, 232, 146, 83, 108] };
pub const GUID_MetadataFormatChunkiTXt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3267282729, data2: 2920, data3: 19319, data4: [170, 14, 98, 149, 166, 172, 24, 20] };
pub const GUID_MetadataFormatChunksRGB: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3239443766, data2: 52335, data3: 20031, data4: [131, 99, 82, 75, 135, 198, 176, 217] };
pub const GUID_MetadataFormatChunktEXt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1452116278, data2: 49321, data3: 18723, data4: [144, 93, 223, 43, 56, 35, 143, 188] };
pub const GUID_MetadataFormatChunktIME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1795206701, data2: 57931, data3: 17930, data4: [152, 182, 135, 139, 208, 48, 114, 253] };
pub const GUID_MetadataFormatDds: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1241925123, data2: 35891, data3: 20064, data4: [156, 41, 19, 98, 49, 112, 45, 8] };
pub const GUID_MetadataFormatExif: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 473714589, data2: 47178, data3: 18045, data4: [148, 147, 54, 207, 189, 89, 234, 87] };
pub const GUID_MetadataFormatGCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 707119832, data2: 57067, data3: 19561, data4: [167, 136, 14, 194, 38, 109, 202, 253] };
pub const GUID_MetadataFormatGifComment: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3300319456, data2: 53172, data3: 19155, data4: [171, 51, 154, 173, 35, 85, 163, 74] };
pub const GUID_MetadataFormatGps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1899277194, data2: 37713, data3: 17581, data4: [175, 98, 68, 141, 182, 181, 2, 236] };
pub const GUID_MetadataFormatHeif: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2172580833, data2: 4744, data3: 17908, data4: [168, 82, 38, 13, 158, 124, 206, 131] };
pub const GUID_MetadataFormatHeifHDR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1451986314, data2: 7781, data3: 17292, data4: [137, 104, 214, 14, 16, 18, 190, 185] };
pub const GUID_MetadataFormatIMD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3173757062, data2: 19794, data3: 18653, data4: [150, 119, 219, 72, 62, 133, 174, 143] };
pub const GUID_MetadataFormatIPTC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1336609044, data2: 57641, data3: 16519, data4: [161, 209, 188, 129, 45, 69, 167, 181] };
pub const GUID_MetadataFormatIRB: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 370150758, data2: 34160, data3: 19385, data4: [185, 45, 253, 164, 178, 62, 206, 103] };
pub const GUID_MetadataFormatIfd: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1400084166, data2: 11658, data3: 19382, data4: [155, 248, 47, 10, 142, 42, 58, 223] };
pub const GUID_MetadataFormatInterop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3983044494, data2: 26655, data3: 19595, data4: [189, 65, 168, 173, 219, 246, 179, 252] };
pub const GUID_MetadataFormatJpegChrominance: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4147973583, data2: 52934, data3: 20357, data4: [155, 14, 28, 57, 86, 177, 190, 247] };
pub const GUID_MetadataFormatJpegComment: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 571367219, data2: 45011, data3: 18254, data4: [157, 49, 125, 79, 231, 48, 245, 87] };
pub const GUID_MetadataFormatJpegLuminance: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2257616903, data2: 60924, data3: 18528, data4: [141, 75, 78, 230, 232, 62, 96, 88] };
pub const GUID_MetadataFormatLSD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3797287710, data2: 25241, data3: 18729, data4: [185, 141, 90, 200, 132, 175, 186, 146] };
pub const GUID_MetadataFormatSubIfd: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1487069480, data2: 11705, data3: 20055, data4: [187, 20, 81, 119, 137, 30, 211, 49] };
pub const GUID_MetadataFormatThumbnail: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 608030441, data2: 34563, data3: 16622, data4: [142, 240, 34, 166, 0, 184, 5, 140] };
pub const GUID_MetadataFormatUnknown: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2757646639, data2: 36984, data3: 19068, data4: [173, 181, 78, 220, 79, 214, 27, 31] };
pub const GUID_MetadataFormatWebpANIM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1841626534, data2: 30950, data3: 16642, data4: [174, 53, 188, 250, 30, 220, 199, 139] };
pub const GUID_MetadataFormatWebpANMF: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1136723438, data2: 47419, data3: 19131, data4: [176, 3, 160, 140, 13, 135, 4, 113] };
pub const GUID_MetadataFormatXMP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3143289912, data2: 61974, data3: 19692, data4: [166, 197, 95, 110, 115, 151, 99, 169] };
pub const GUID_MetadataFormatXMPAlt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2064164469, data2: 37290, data3: 18459, data4: [167, 152, 77, 169, 73, 8, 97, 59] };
pub const GUID_MetadataFormatXMPBag: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2201799263, data2: 56503, data3: 17686, data4: [128, 111, 101, 150, 171, 38, 220, 228] };
pub const GUID_MetadataFormatXMPSeq: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1676205826, data2: 60268, data3: 17772, data4: [162, 36, 178, 94, 121, 79, 214, 72] };
pub const GUID_MetadataFormatXMPStruct: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 574110961, data2: 60695, data3: 20014, data4: [175, 23, 216, 91, 143, 107, 48, 208] };
pub const GUID_VendorMicrosoft: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4041689546, data2: 60911, data3: 17801, data4: [167, 58, 238, 14, 98, 106, 42, 43] };
pub const GUID_VendorMicrosoftBuiltIn: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 628764925, data2: 1718, data3: 17963, data4: [174, 164, 99, 247, 11, 134, 229, 51] };
pub const GUID_WICPixelFormat112bpp6ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 55] };
pub const GUID_WICPixelFormat112bpp7Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 42] };
pub const GUID_WICPixelFormat128bpp7ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 56] };
pub const GUID_WICPixelFormat128bpp8Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 43] };
pub const GUID_WICPixelFormat128bppPRGBAFloat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 26] };
pub const GUID_WICPixelFormat128bppRGBAFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 30] };
pub const GUID_WICPixelFormat128bppRGBAFloat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 25] };
pub const GUID_WICPixelFormat128bppRGBFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 65] };
pub const GUID_WICPixelFormat128bppRGBFloat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 27] };
pub const GUID_WICPixelFormat144bpp8ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 57] };
pub const GUID_WICPixelFormat16bppBGR555: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 9] };
pub const GUID_WICPixelFormat16bppBGR565: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 10] };
pub const GUID_WICPixelFormat16bppBGRA5551: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 99384363, data2: 61926, data3: 18785, data4: [173, 70, 225, 204, 129, 10, 135, 210] };
pub const GUID_WICPixelFormat16bppCbCr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4288002670, data2: 4576, data3: 16995, data4: [187, 69, 1, 114, 31, 52, 96, 164] };
pub const GUID_WICPixelFormat16bppCbQuantizedDctCoefficients: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3536125793, data2: 22181, data3: 18882, data4: [139, 92, 76, 25, 37, 150, 72, 55] };
pub const GUID_WICPixelFormat16bppCrQuantizedDctCoefficients: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 803427568, data2: 5760, data3: 17112, data4: [146, 49, 231, 60, 5, 101, 191, 193] };
pub const GUID_WICPixelFormat16bppGray: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 11] };
pub const GUID_WICPixelFormat16bppGrayFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 19] };
pub const GUID_WICPixelFormat16bppGrayHalf: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 62] };
pub const GUID_WICPixelFormat16bppYQuantizedDctCoefficients: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2740319283, data2: 18664, data3: 19010, data4: [132, 216, 226, 170, 38, 202, 128, 164] };
pub const GUID_WICPixelFormat1bppIndexed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 1] };
pub const GUID_WICPixelFormat24bpp3Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 32] };
pub const GUID_WICPixelFormat24bppBGR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 12] };
pub const GUID_WICPixelFormat24bppRGB: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 13] };
pub const GUID_WICPixelFormat2bppGray: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 6] };
pub const GUID_WICPixelFormat2bppIndexed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 2] };
pub const GUID_WICPixelFormat32bpp3ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 46] };
pub const GUID_WICPixelFormat32bpp4Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 33] };
pub const GUID_WICPixelFormat32bppBGR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 14] };
pub const GUID_WICPixelFormat32bppBGR101010: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 20] };
pub const GUID_WICPixelFormat32bppBGRA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 15] };
pub const GUID_WICPixelFormat32bppCMYK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 28] };
pub const GUID_WICPixelFormat32bppGrayFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 63] };
pub const GUID_WICPixelFormat32bppGrayFloat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 17] };
pub const GUID_WICPixelFormat32bppPBGRA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 16] };
pub const GUID_WICPixelFormat32bppPRGBA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1019520592, data2: 42279, data3: 19767, data4: [169, 22, 49, 66, 199, 235, 237, 186] };
pub const GUID_WICPixelFormat32bppR10G10B10A2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1615731637, data2: 35388, data3: 19301, data4: [177, 28, 188, 11, 141, 215, 91, 127] };
pub const GUID_WICPixelFormat32bppR10G10B10A2HDR10: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2619432029, data2: 6860, data3: 20238, data4: [164, 188, 112, 251, 58, 232, 253, 40] };
pub const GUID_WICPixelFormat32bppRGB: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3649858453, data2: 16126, data3: 18390, data4: [187, 37, 235, 23, 72, 171, 12, 241] };
pub const GUID_WICPixelFormat32bppRGBA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4123503917, data2: 27277, data3: 17373, data4: [167, 168, 162, 153, 53, 38, 26, 233] };
pub const GUID_WICPixelFormat32bppRGBA1010102: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 623086962, data2: 64761, data3: 17698, data4: [181, 20, 85, 120, 229, 173, 85, 224] };
pub const GUID_WICPixelFormat32bppRGBA1010102XR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 14576538, data2: 49409, data3: 17227, data4: [181, 2, 208, 22, 94, 225, 18, 44] };
pub const GUID_WICPixelFormat32bppRGBE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 61] };
pub const GUID_WICPixelFormat40bpp4ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 47] };
pub const GUID_WICPixelFormat40bpp5Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 34] };
pub const GUID_WICPixelFormat40bppCMYKAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 44] };
pub const GUID_WICPixelFormat48bpp3Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 38] };
pub const GUID_WICPixelFormat48bpp5ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 48] };
pub const GUID_WICPixelFormat48bpp6Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 35] };
pub const GUID_WICPixelFormat48bppBGR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3859129220, data2: 46184, data3: 18126, data4: [187, 46, 54, 241, 128, 230, 67, 19] };
pub const GUID_WICPixelFormat48bppBGRFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1237980174, data2: 51894, data3: 18747, data4: [157, 223, 96, 24, 124, 55, 83, 42] };
pub const GUID_WICPixelFormat48bppRGB: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 21] };
pub const GUID_WICPixelFormat48bppRGBFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 18] };
pub const GUID_WICPixelFormat48bppRGBHalf: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 59] };
pub const GUID_WICPixelFormat4bppGray: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 7] };
pub const GUID_WICPixelFormat4bppIndexed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 3] };
pub const GUID_WICPixelFormat56bpp6ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 49] };
pub const GUID_WICPixelFormat56bpp7Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 36] };
pub const GUID_WICPixelFormat64bpp3ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 52] };
pub const GUID_WICPixelFormat64bpp4Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 39] };
pub const GUID_WICPixelFormat64bpp7ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 50] };
pub const GUID_WICPixelFormat64bpp8Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 37] };
pub const GUID_WICPixelFormat64bppBGRA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 358809468, data2: 54098, data3: 18169, data4: [151, 158, 66, 151, 107, 121, 34, 70] };
pub const GUID_WICPixelFormat64bppBGRAFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 896394044, data2: 21714, data3: 18979, data4: [187, 4, 155, 123, 249, 177, 212, 45] };
pub const GUID_WICPixelFormat64bppCMYK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 31] };
pub const GUID_WICPixelFormat64bppPBGRA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2354155150, data2: 42220, data3: 18059, data4: [174, 112, 201, 163, 90, 156, 85, 48] };
pub const GUID_WICPixelFormat64bppPRGBA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 23] };
pub const GUID_WICPixelFormat64bppPRGBAHalf: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1487742658, data2: 50723, data3: 19869, data4: [179, 32, 56, 126, 73, 248, 196, 66] };
pub const GUID_WICPixelFormat64bppRGB: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2702713105, data2: 6253, data3: 19778, data4: [188, 106, 156, 131, 3, 168, 223, 249] };
pub const GUID_WICPixelFormat64bppRGBA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 22] };
pub const GUID_WICPixelFormat64bppRGBAFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 29] };
pub const GUID_WICPixelFormat64bppRGBAHalf: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 58] };
pub const GUID_WICPixelFormat64bppRGBFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 64] };
pub const GUID_WICPixelFormat64bppRGBHalf: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 66] };
pub const GUID_WICPixelFormat72bpp8ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 51] };
pub const GUID_WICPixelFormat80bpp4ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 53] };
pub const GUID_WICPixelFormat80bpp5Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 40] };
pub const GUID_WICPixelFormat80bppCMYKAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 45] };
pub const GUID_WICPixelFormat8bppAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3872194838, data2: 61114, data3: 16737, data4: [170, 133, 39, 221, 159, 179, 168, 149] };
pub const GUID_WICPixelFormat8bppCb: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 322564644, data2: 27646, data3: 19518, data4: [147, 2, 228, 243, 166, 208, 202, 42] };
pub const GUID_WICPixelFormat8bppCr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3088339027, data2: 8470, data3: 18928, data4: [136, 53, 237, 132, 75, 32, 92, 81] };
pub const GUID_WICPixelFormat8bppGray: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 8] };
pub const GUID_WICPixelFormat8bppIndexed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 4] };
pub const GUID_WICPixelFormat8bppY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2444548948, data2: 11769, data3: 17136, data4: [180, 73, 41, 9, 187, 61, 248, 142] };
pub const GUID_WICPixelFormat96bpp5ChannelsAlpha: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 54] };
pub const GUID_WICPixelFormat96bpp6Channels: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 41] };
pub const GUID_WICPixelFormat96bppRGBFixedPoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 24] };
pub const GUID_WICPixelFormat96bppRGBFloat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3825129359, data2: 59611, data3: 19151, data4: [132, 193, 233, 127, 97, 54, 179, 39] };
pub const GUID_WICPixelFormatBlackWhite: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 5] };
pub const GUID_WICPixelFormatDontCare: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1876804388, data2: 19971, data3: 19454, data4: [177, 133, 61, 119, 118, 141, 201, 0] };
pub type IWICBitmap = *mut ::core::ffi::c_void;
pub type IWICBitmapClipper = *mut ::core::ffi::c_void;
pub type IWICBitmapCodecInfo = *mut ::core::ffi::c_void;
pub type IWICBitmapCodecProgressNotification = *mut ::core::ffi::c_void;
pub type IWICBitmapDecoder = *mut ::core::ffi::c_void;
pub type IWICBitmapDecoderInfo = *mut ::core::ffi::c_void;
pub type IWICBitmapEncoder = *mut ::core::ffi::c_void;
pub type IWICBitmapEncoderInfo = *mut ::core::ffi::c_void;
pub type IWICBitmapFlipRotator = *mut ::core::ffi::c_void;
pub type IWICBitmapFrameDecode = *mut ::core::ffi::c_void;
pub type IWICBitmapFrameEncode = *mut ::core::ffi::c_void;
pub type IWICBitmapLock = *mut ::core::ffi::c_void;
pub type IWICBitmapScaler = *mut ::core::ffi::c_void;
pub type IWICBitmapSource = *mut ::core::ffi::c_void;
pub type IWICBitmapSourceTransform = *mut ::core::ffi::c_void;
pub type IWICColorContext = *mut ::core::ffi::c_void;
pub type IWICColorTransform = *mut ::core::ffi::c_void;
pub type IWICComponentFactory = *mut ::core::ffi::c_void;
pub type IWICComponentInfo = *mut ::core::ffi::c_void;
pub type IWICDdsDecoder = *mut ::core::ffi::c_void;
pub type IWICDdsEncoder = *mut ::core::ffi::c_void;
pub type IWICDdsFrameDecode = *mut ::core::ffi::c_void;
pub type IWICDevelopRaw = *mut ::core::ffi::c_void;
pub type IWICDevelopRawNotificationCallback = *mut ::core::ffi::c_void;
pub type IWICEnumMetadataItem = *mut ::core::ffi::c_void;
pub type IWICFastMetadataEncoder = *mut ::core::ffi::c_void;
pub type IWICFormatConverter = *mut ::core::ffi::c_void;
pub type IWICFormatConverterInfo = *mut ::core::ffi::c_void;
pub type IWICImagingFactory = *mut ::core::ffi::c_void;
pub type IWICJpegFrameDecode = *mut ::core::ffi::c_void;
pub type IWICJpegFrameEncode = *mut ::core::ffi::c_void;
pub type IWICMetadataBlockReader = *mut ::core::ffi::c_void;
pub type IWICMetadataBlockWriter = *mut ::core::ffi::c_void;
pub type IWICMetadataHandlerInfo = *mut ::core::ffi::c_void;
pub type IWICMetadataQueryReader = *mut ::core::ffi::c_void;
pub type IWICMetadataQueryWriter = *mut ::core::ffi::c_void;
pub type IWICMetadataReader = *mut ::core::ffi::c_void;
pub type IWICMetadataReaderInfo = *mut ::core::ffi::c_void;
pub type IWICMetadataWriter = *mut ::core::ffi::c_void;
pub type IWICMetadataWriterInfo = *mut ::core::ffi::c_void;
pub type IWICPalette = *mut ::core::ffi::c_void;
pub type IWICPersistStream = *mut ::core::ffi::c_void;
pub type IWICPixelFormatInfo = *mut ::core::ffi::c_void;
pub type IWICPixelFormatInfo2 = *mut ::core::ffi::c_void;
pub type IWICPlanarBitmapFrameEncode = *mut ::core::ffi::c_void;
pub type IWICPlanarBitmapSourceTransform = *mut ::core::ffi::c_void;
pub type IWICPlanarFormatConverter = *mut ::core::ffi::c_void;
pub type IWICProgressCallback = *mut ::core::ffi::c_void;
pub type IWICProgressiveLevelControl = *mut ::core::ffi::c_void;
pub type IWICStream = *mut ::core::ffi::c_void;
pub type IWICStreamProvider = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type PFNProgressNotification = ::core::option::Option<unsafe extern "system" fn(pvdata: *const ::core::ffi::c_void, uframenum: u32, operation: WICProgressOperation, dblprogress: f64) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WIC8BIMIptcDigestProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMIptcDigestPString: WIC8BIMIptcDigestProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMIptcDigestIptcDigest: WIC8BIMIptcDigestProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMIptcDigestProperties_FORCE_DWORD: WIC8BIMIptcDigestProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WIC8BIMIptcProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMIptcPString: WIC8BIMIptcProperties = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMIptcEmbeddedIPTC: WIC8BIMIptcProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMIptcProperties_FORCE_DWORD: WIC8BIMIptcProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WIC8BIMResolutionInfoProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoPString: WIC8BIMResolutionInfoProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoHResolution: WIC8BIMResolutionInfoProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoHResolutionUnit: WIC8BIMResolutionInfoProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoWidthUnit: WIC8BIMResolutionInfoProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoVResolution: WIC8BIMResolutionInfoProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoVResolutionUnit: WIC8BIMResolutionInfoProperties = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoHeightUnit: WIC8BIMResolutionInfoProperties = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC8BIMResolutionInfoProperties_FORCE_DWORD: WIC8BIMResolutionInfoProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapAlphaChannelOption = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapUseAlpha: WICBitmapAlphaChannelOption = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapUsePremultipliedAlpha: WICBitmapAlphaChannelOption = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapIgnoreAlpha: WICBitmapAlphaChannelOption = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPALPHACHANNELOPTIONS_FORCE_DWORD: WICBitmapAlphaChannelOption = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapCreateCacheOption = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapNoCache: WICBitmapCreateCacheOption = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapCacheOnDemand: WICBitmapCreateCacheOption = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapCacheOnLoad: WICBitmapCreateCacheOption = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPCREATECACHEOPTION_FORCE_DWORD: WICBitmapCreateCacheOption = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapDecoderCapabilities = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDecoderCapabilitySameEncoder: WICBitmapDecoderCapabilities = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDecoderCapabilityCanDecodeAllImages: WICBitmapDecoderCapabilities = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDecoderCapabilityCanDecodeSomeImages: WICBitmapDecoderCapabilities = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDecoderCapabilityCanEnumerateMetadata: WICBitmapDecoderCapabilities = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDecoderCapabilityCanDecodeThumbnail: WICBitmapDecoderCapabilities = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPDECODERCAPABILITIES_FORCE_DWORD: WICBitmapDecoderCapabilities = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapDitherType = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeNone: WICBitmapDitherType = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeSolid: WICBitmapDitherType = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeOrdered4x4: WICBitmapDitherType = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeOrdered8x8: WICBitmapDitherType = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeOrdered16x16: WICBitmapDitherType = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeSpiral4x4: WICBitmapDitherType = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeSpiral8x8: WICBitmapDitherType = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeDualSpiral4x4: WICBitmapDitherType = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeDualSpiral8x8: WICBitmapDitherType = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapDitherTypeErrorDiffusion: WICBitmapDitherType = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPDITHERTYPE_FORCE_DWORD: WICBitmapDitherType = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapEncoderCacheOption = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapEncoderCacheInMemory: WICBitmapEncoderCacheOption = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapEncoderCacheTempFile: WICBitmapEncoderCacheOption = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapEncoderNoCache: WICBitmapEncoderCacheOption = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPENCODERCACHEOPTION_FORCE_DWORD: WICBitmapEncoderCacheOption = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapInterpolationMode = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapInterpolationModeNearestNeighbor: WICBitmapInterpolationMode = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapInterpolationModeLinear: WICBitmapInterpolationMode = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapInterpolationModeCubic: WICBitmapInterpolationMode = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapInterpolationModeFant: WICBitmapInterpolationMode = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapInterpolationModeHighQualityCubic: WICBitmapInterpolationMode = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPINTERPOLATIONMODE_FORCE_DWORD: WICBitmapInterpolationMode = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapLockFlags = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapLockRead: WICBitmapLockFlags = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapLockWrite: WICBitmapLockFlags = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPLOCKFLAGS_FORCE_DWORD: WICBitmapLockFlags = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapPaletteType = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeCustom: WICBitmapPaletteType = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeMedianCut: WICBitmapPaletteType = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedBW: WICBitmapPaletteType = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone8: WICBitmapPaletteType = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone27: WICBitmapPaletteType = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone64: WICBitmapPaletteType = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone125: WICBitmapPaletteType = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone216: WICBitmapPaletteType = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedWebPalette: WICBitmapPaletteType = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone252: WICBitmapPaletteType = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedHalftone256: WICBitmapPaletteType = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedGray4: WICBitmapPaletteType = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedGray16: WICBitmapPaletteType = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapPaletteTypeFixedGray256: WICBitmapPaletteType = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPPALETTETYPE_FORCE_DWORD: WICBitmapPaletteType = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct WICBitmapPattern {
    pub Position: u64,
    pub Length: u32,
    pub Pattern: *mut u8,
    pub Mask: *mut u8,
    pub EndOfStream: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for WICBitmapPattern {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for WICBitmapPattern {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICBitmapPlane {
    pub Format: ::windows_sys::core::GUID,
    pub pbBuffer: *mut u8,
    pub cbStride: u32,
    pub cbBufferSize: u32,
}
impl ::core::marker::Copy for WICBitmapPlane {}
impl ::core::clone::Clone for WICBitmapPlane {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICBitmapPlaneDescription {
    pub Format: ::windows_sys::core::GUID,
    pub Width: u32,
    pub Height: u32,
}
impl ::core::marker::Copy for WICBitmapPlaneDescription {}
impl ::core::clone::Clone for WICBitmapPlaneDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICBitmapTransformOptions = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapTransformRotate0: WICBitmapTransformOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapTransformRotate90: WICBitmapTransformOptions = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapTransformRotate180: WICBitmapTransformOptions = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapTransformRotate270: WICBitmapTransformOptions = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapTransformFlipHorizontal: WICBitmapTransformOptions = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBitmapTransformFlipVertical: WICBitmapTransformOptions = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICBITMAPTRANSFORMOPTIONS_FORCE_DWORD: WICBitmapTransformOptions = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICColorContextType = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICColorContextUninitialized: WICColorContextType = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICColorContextProfile: WICColorContextType = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICColorContextExifColorSpace: WICColorContextType = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICComponentEnumerateOptions = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentEnumerateDefault: WICComponentEnumerateOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentEnumerateRefresh: WICComponentEnumerateOptions = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentEnumerateDisabled: WICComponentEnumerateOptions = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentEnumerateUnsigned: WICComponentEnumerateOptions = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentEnumerateBuiltInOnly: WICComponentEnumerateOptions = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICCOMPONENTENUMERATEOPTIONS_FORCE_DWORD: WICComponentEnumerateOptions = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICComponentSigning = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentSigned: WICComponentSigning = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentUnsigned: WICComponentSigning = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentSafe: WICComponentSigning = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICComponentDisabled: WICComponentSigning = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICCOMPONENTSIGNING_FORCE_DWORD: WICComponentSigning = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICComponentType = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDecoder: WICComponentType = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICEncoder: WICComponentType = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatConverter: WICComponentType = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMetadataReader: WICComponentType = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMetadataWriter: WICComponentType = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormat: WICComponentType = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICAllComponents: WICComponentType = 63i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICCOMPONENTTYPE_FORCE_DWORD: WICComponentType = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICDdsAlphaMode = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsAlphaModeUnknown: WICDdsAlphaMode = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsAlphaModeStraight: WICDdsAlphaMode = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsAlphaModePremultiplied: WICDdsAlphaMode = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsAlphaModeOpaque: WICDdsAlphaMode = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsAlphaModeCustom: WICDdsAlphaMode = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDDSALPHAMODE_FORCE_DWORD: WICDdsAlphaMode = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICDdsDimension = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsTexture1D: WICDdsDimension = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsTexture2D: WICDdsDimension = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsTexture3D: WICDdsDimension = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDdsTextureCube: WICDdsDimension = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDDSTEXTURE_FORCE_DWORD: WICDdsDimension = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct WICDdsFormatInfo {
    pub DxgiFormat: super::Dxgi::Common::DXGI_FORMAT,
    pub BytesPerBlock: u32,
    pub BlockWidth: u32,
    pub BlockHeight: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for WICDdsFormatInfo {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for WICDdsFormatInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct WICDdsParameters {
    pub Width: u32,
    pub Height: u32,
    pub Depth: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub DxgiFormat: super::Dxgi::Common::DXGI_FORMAT,
    pub Dimension: WICDdsDimension,
    pub AlphaMode: WICDdsAlphaMode,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for WICDdsParameters {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for WICDdsParameters {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICDecodeOptions = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDecodeMetadataCacheOnDemand: WICDecodeOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICDecodeMetadataCacheOnLoad: WICDecodeOptions = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMETADATACACHEOPTION_FORCE_DWORD: WICDecodeOptions = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICGifApplicationExtensionProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifApplicationExtensionApplication: WICGifApplicationExtensionProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifApplicationExtensionData: WICGifApplicationExtensionProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifApplicationExtensionProperties_FORCE_DWORD: WICGifApplicationExtensionProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICGifCommentExtensionProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifCommentExtensionText: WICGifCommentExtensionProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifCommentExtensionProperties_FORCE_DWORD: WICGifCommentExtensionProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICGifGraphicControlExtensionProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifGraphicControlExtensionDisposal: WICGifGraphicControlExtensionProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifGraphicControlExtensionUserInputFlag: WICGifGraphicControlExtensionProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifGraphicControlExtensionTransparencyFlag: WICGifGraphicControlExtensionProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifGraphicControlExtensionDelay: WICGifGraphicControlExtensionProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifGraphicControlExtensionTransparentColorIndex: WICGifGraphicControlExtensionProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifGraphicControlExtensionProperties_FORCE_DWORD: WICGifGraphicControlExtensionProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICGifImageDescriptorProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorLeft: WICGifImageDescriptorProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorTop: WICGifImageDescriptorProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorWidth: WICGifImageDescriptorProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorHeight: WICGifImageDescriptorProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorLocalColorTableFlag: WICGifImageDescriptorProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorInterlaceFlag: WICGifImageDescriptorProperties = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorSortFlag: WICGifImageDescriptorProperties = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorLocalColorTableSize: WICGifImageDescriptorProperties = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifImageDescriptorProperties_FORCE_DWORD: WICGifImageDescriptorProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICGifLogicalScreenDescriptorProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenSignature: WICGifLogicalScreenDescriptorProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorWidth: WICGifLogicalScreenDescriptorProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorHeight: WICGifLogicalScreenDescriptorProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorGlobalColorTableFlag: WICGifLogicalScreenDescriptorProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorColorResolution: WICGifLogicalScreenDescriptorProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorSortFlag: WICGifLogicalScreenDescriptorProperties = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorGlobalColorTableSize: WICGifLogicalScreenDescriptorProperties = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorBackgroundColorIndex: WICGifLogicalScreenDescriptorProperties = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorPixelAspectRatio: WICGifLogicalScreenDescriptorProperties = 9u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICGifLogicalScreenDescriptorProperties_FORCE_DWORD: WICGifLogicalScreenDescriptorProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICHeifHdrProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifHdrMaximumLuminanceLevel: WICHeifHdrProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifHdrMaximumFrameAverageLuminanceLevel: WICHeifHdrProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifHdrMinimumMasteringDisplayLuminanceLevel: WICHeifHdrProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifHdrMaximumMasteringDisplayLuminanceLevel: WICHeifHdrProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifHdrCustomVideoPrimaries: WICHeifHdrProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifHdrProperties_FORCE_DWORD: WICHeifHdrProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICHeifProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifOrientation: WICHeifProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICHeifProperties_FORCE_DWORD: WICHeifProperties = 2147483647u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`, `\"Win32_Graphics_Direct2D_Common\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct WICImageParameters {
    pub PixelFormat: super::Direct2D::Common::D2D1_PIXEL_FORMAT,
    pub DpiX: f32,
    pub DpiY: f32,
    pub Top: f32,
    pub Left: f32,
    pub PixelWidth: u32,
    pub PixelHeight: u32,
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for WICImageParameters {}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for WICImageParameters {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegChrominanceProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegChrominanceTable: WICJpegChrominanceProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegChrominanceProperties_FORCE_DWORD: WICJpegChrominanceProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegCommentProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegCommentText: WICJpegCommentProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegCommentProperties_FORCE_DWORD: WICJpegCommentProperties = 2147483647u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICJpegFrameHeader {
    pub Width: u32,
    pub Height: u32,
    pub TransferMatrix: WICJpegTransferMatrix,
    pub ScanType: WICJpegScanType,
    pub cComponents: u32,
    pub ComponentIdentifiers: u32,
    pub SampleFactors: u32,
    pub QuantizationTableIndices: u32,
}
impl ::core::marker::Copy for WICJpegFrameHeader {}
impl ::core::clone::Clone for WICJpegFrameHeader {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegIndexingOptions = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegIndexingOptionsGenerateOnDemand: WICJpegIndexingOptions = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegIndexingOptionsGenerateOnLoad: WICJpegIndexingOptions = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegIndexingOptions_FORCE_DWORD: WICJpegIndexingOptions = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegLuminanceProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegLuminanceTable: WICJpegLuminanceProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegLuminanceProperties_FORCE_DWORD: WICJpegLuminanceProperties = 2147483647u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICJpegScanHeader {
    pub cComponents: u32,
    pub RestartInterval: u32,
    pub ComponentSelectors: u32,
    pub HuffmanTableIndices: u32,
    pub StartSpectralSelection: u8,
    pub EndSpectralSelection: u8,
    pub SuccessiveApproximationHigh: u8,
    pub SuccessiveApproximationLow: u8,
}
impl ::core::marker::Copy for WICJpegScanHeader {}
impl ::core::clone::Clone for WICJpegScanHeader {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegScanType = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegScanTypeInterleaved: WICJpegScanType = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegScanTypePlanarComponents: WICJpegScanType = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegScanTypeProgressive: WICJpegScanType = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegScanType_FORCE_DWORD: WICJpegScanType = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegTransferMatrix = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegTransferMatrixIdentity: WICJpegTransferMatrix = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegTransferMatrixBT601: WICJpegTransferMatrix = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegTransferMatrix_FORCE_DWORD: WICJpegTransferMatrix = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICJpegYCrCbSubsamplingOption = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegYCrCbSubsamplingDefault: WICJpegYCrCbSubsamplingOption = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegYCrCbSubsampling420: WICJpegYCrCbSubsamplingOption = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegYCrCbSubsampling422: WICJpegYCrCbSubsamplingOption = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegYCrCbSubsampling444: WICJpegYCrCbSubsamplingOption = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJpegYCrCbSubsampling440: WICJpegYCrCbSubsamplingOption = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICJPEGYCRCBSUBSAMPLING_FORCE_DWORD: WICJpegYCrCbSubsamplingOption = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICMetadataCreationOptions = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMetadataCreationDefault: WICMetadataCreationOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMetadataCreationAllowUnknown: WICMetadataCreationOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMetadataCreationFailUnknown: WICMetadataCreationOptions = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICMetadataCreationMask: WICMetadataCreationOptions = -65536i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICMetadataHeader {
    pub Position: u64,
    pub Length: u32,
    pub Header: *mut u8,
    pub DataOffset: u64,
}
impl ::core::marker::Copy for WICMetadataHeader {}
impl ::core::clone::Clone for WICMetadataHeader {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICMetadataPattern {
    pub Position: u64,
    pub Length: u32,
    pub Pattern: *mut u8,
    pub Mask: *mut u8,
    pub DataOffset: u64,
}
impl ::core::marker::Copy for WICMetadataPattern {}
impl ::core::clone::Clone for WICMetadataPattern {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICNamedWhitePoint = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointDefault: WICNamedWhitePoint = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointDaylight: WICNamedWhitePoint = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointCloudy: WICNamedWhitePoint = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointShade: WICNamedWhitePoint = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointTungsten: WICNamedWhitePoint = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointFluorescent: WICNamedWhitePoint = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointFlash: WICNamedWhitePoint = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointUnderwater: WICNamedWhitePoint = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointCustom: WICNamedWhitePoint = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointAutoWhiteBalance: WICNamedWhitePoint = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWhitePointAsShot: WICNamedWhitePoint = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICNAMEDWHITEPOINT_FORCE_DWORD: WICNamedWhitePoint = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPersistOptions = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionDefault: WICPersistOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionLittleEndian: WICPersistOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionBigEndian: WICPersistOptions = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionStrictFormat: WICPersistOptions = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionNoCacheStream: WICPersistOptions = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionPreferUTF8: WICPersistOptions = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPersistOptionMask: WICPersistOptions = 65535i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPixelFormatNumericRepresentation = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentationUnspecified: WICPixelFormatNumericRepresentation = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentationIndexed: WICPixelFormatNumericRepresentation = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentationUnsignedInteger: WICPixelFormatNumericRepresentation = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentationSignedInteger: WICPixelFormatNumericRepresentation = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentationFixed: WICPixelFormatNumericRepresentation = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentationFloat: WICPixelFormatNumericRepresentation = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPixelFormatNumericRepresentation_FORCE_DWORD: WICPixelFormatNumericRepresentation = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPlanarOptions = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPlanarOptionsDefault: WICPlanarOptions = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPlanarOptionsPreserveSubsampling: WICPlanarOptions = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPLANAROPTIONS_FORCE_DWORD: WICPlanarOptions = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngBkgdProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngBkgdBackgroundColor: WICPngBkgdProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngBkgdProperties_FORCE_DWORD: WICPngBkgdProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngChrmProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmWhitePointX: WICPngChrmProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmWhitePointY: WICPngChrmProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmRedX: WICPngChrmProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmRedY: WICPngChrmProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmGreenX: WICPngChrmProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmGreenY: WICPngChrmProperties = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmBlueX: WICPngChrmProperties = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmBlueY: WICPngChrmProperties = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngChrmProperties_FORCE_DWORD: WICPngChrmProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngFilterOption = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterUnspecified: WICPngFilterOption = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterNone: WICPngFilterOption = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterSub: WICPngFilterOption = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterUp: WICPngFilterOption = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterAverage: WICPngFilterOption = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterPaeth: WICPngFilterOption = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngFilterAdaptive: WICPngFilterOption = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPNGFILTEROPTION_FORCE_DWORD: WICPngFilterOption = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngGamaProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngGamaGamma: WICPngGamaProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngGamaProperties_FORCE_DWORD: WICPngGamaProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngHistProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngHistFrequencies: WICPngHistProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngHistProperties_FORCE_DWORD: WICPngHistProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngIccpProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngIccpProfileName: WICPngIccpProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngIccpProfileData: WICPngIccpProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngIccpProperties_FORCE_DWORD: WICPngIccpProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngItxtProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngItxtKeyword: WICPngItxtProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngItxtCompressionFlag: WICPngItxtProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngItxtLanguageTag: WICPngItxtProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngItxtTranslatedKeyword: WICPngItxtProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngItxtText: WICPngItxtProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngItxtProperties_FORCE_DWORD: WICPngItxtProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngSrgbProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngSrgbRenderingIntent: WICPngSrgbProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngSrgbProperties_FORCE_DWORD: WICPngSrgbProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICPngTimeProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeYear: WICPngTimeProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeMonth: WICPngTimeProperties = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeDay: WICPngTimeProperties = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeHour: WICPngTimeProperties = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeMinute: WICPngTimeProperties = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeSecond: WICPngTimeProperties = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPngTimeProperties_FORCE_DWORD: WICPngTimeProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICProgressNotification = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressNotificationBegin: WICProgressNotification = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressNotificationEnd: WICProgressNotification = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressNotificationFrequent: WICProgressNotification = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressNotificationAll: WICProgressNotification = -65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPROGRESSNOTIFICATION_FORCE_DWORD: WICProgressNotification = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICProgressOperation = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressOperationCopyPixels: WICProgressOperation = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressOperationWritePixels: WICProgressOperation = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICProgressOperationAll: WICProgressOperation = 65535i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICPROGRESSOPERATION_FORCE_DWORD: WICProgressOperation = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICRawCapabilities = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawCapabilityNotSupported: WICRawCapabilities = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawCapabilityGetSupported: WICRawCapabilities = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawCapabilityFullySupported: WICRawCapabilities = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRAWCAPABILITIES_FORCE_DWORD: WICRawCapabilities = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICRawCapabilitiesInfo {
    pub cbSize: u32,
    pub CodecMajorVersion: u32,
    pub CodecMinorVersion: u32,
    pub ExposureCompensationSupport: WICRawCapabilities,
    pub ContrastSupport: WICRawCapabilities,
    pub RGBWhitePointSupport: WICRawCapabilities,
    pub NamedWhitePointSupport: WICRawCapabilities,
    pub NamedWhitePointSupportMask: u32,
    pub KelvinWhitePointSupport: WICRawCapabilities,
    pub GammaSupport: WICRawCapabilities,
    pub TintSupport: WICRawCapabilities,
    pub SaturationSupport: WICRawCapabilities,
    pub SharpnessSupport: WICRawCapabilities,
    pub NoiseReductionSupport: WICRawCapabilities,
    pub DestinationColorProfileSupport: WICRawCapabilities,
    pub ToneCurveSupport: WICRawCapabilities,
    pub RotationSupport: WICRawRotationCapabilities,
    pub RenderModeSupport: WICRawCapabilities,
}
impl ::core::marker::Copy for WICRawCapabilitiesInfo {}
impl ::core::clone::Clone for WICRawCapabilitiesInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_Contrast: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_DestinationColorContext: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_ExposureCompensation: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_Gamma: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_KelvinWhitePoint: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_NamedWhitePoint: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_NoiseReduction: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_RGBWhitePoint: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_RenderMode: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_Rotation: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_Saturation: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_Sharpness: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_Tint: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawChangeNotification_ToneCurve: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICRawParameterSet = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICAsShotParameterSet: WICRawParameterSet = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICUserAdjustedParameterSet: WICRawParameterSet = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICAutoAdjustedParameterSet: WICRawParameterSet = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRAWPARAMETERSET_FORCE_DWORD: WICRawParameterSet = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICRawRenderMode = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRenderModeDraft: WICRawRenderMode = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRenderModeNormal: WICRawRenderMode = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRenderModeBestQuality: WICRawRenderMode = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRAWRENDERMODE_FORCE_DWORD: WICRawRenderMode = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICRawRotationCapabilities = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRotationCapabilityNotSupported: WICRawRotationCapabilities = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRotationCapabilityGetSupported: WICRawRotationCapabilities = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRotationCapabilityNinetyDegreesSupported: WICRawRotationCapabilities = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRawRotationCapabilityFullySupported: WICRawRotationCapabilities = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICRAWROTATIONCAPABILITIES_FORCE_DWORD: WICRawRotationCapabilities = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICRawToneCurve {
    pub cPoints: u32,
    pub aPoints: [WICRawToneCurvePoint; 1],
}
impl ::core::marker::Copy for WICRawToneCurve {}
impl ::core::clone::Clone for WICRawToneCurve {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICRawToneCurvePoint {
    pub Input: f64,
    pub Output: f64,
}
impl ::core::marker::Copy for WICRawToneCurvePoint {}
impl ::core::clone::Clone for WICRawToneCurvePoint {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub struct WICRect {
    pub X: i32,
    pub Y: i32,
    pub Width: i32,
    pub Height: i32,
}
impl ::core::marker::Copy for WICRect {}
impl ::core::clone::Clone for WICRect {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICSectionAccessLevel = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICSectionAccessLevelRead: WICSectionAccessLevel = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICSectionAccessLevelReadWrite: WICSectionAccessLevel = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICSectionAccessLevel_FORCE_DWORD: WICSectionAccessLevel = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICTiffCompressionOption = i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionDontCare: WICTiffCompressionOption = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionNone: WICTiffCompressionOption = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionCCITT3: WICTiffCompressionOption = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionCCITT4: WICTiffCompressionOption = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionLZW: WICTiffCompressionOption = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionRLE: WICTiffCompressionOption = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionZIP: WICTiffCompressionOption = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTiffCompressionLZWHDifferencing: WICTiffCompressionOption = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICTIFFCOMPRESSIONOPTION_FORCE_DWORD: WICTiffCompressionOption = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICWebpAnimProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWebpAnimLoopCount: WICWebpAnimProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWebpAnimProperties_FORCE_DWORD: WICWebpAnimProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub type WICWebpAnmfProperties = u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWebpAnmfFrameDuration: WICWebpAnmfProperties = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WICWebpAnmfProperties_FORCE_DWORD: WICWebpAnmfProperties = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_HUFFMAN_BASELINE_ONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_HUFFMAN_BASELINE_THREE: u32 = 1118464u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_MAX_COMPONENT_COUNT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_MAX_TABLE_INDEX: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_QUANTIZATION_BASELINE_ONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_QUANTIZATION_BASELINE_THREE: u32 = 65792u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_SAMPLE_FACTORS_ONE: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_SAMPLE_FACTORS_THREE_420: u32 = 1118498u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_SAMPLE_FACTORS_THREE_422: u32 = 1118497u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_SAMPLE_FACTORS_THREE_440: u32 = 1118482u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WIC_JPEG_SAMPLE_FACTORS_THREE_444: u32 = 1118481u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_ABORTED: i32 = -2147467260i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_ACCESSDENIED: i32 = -2147024891i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_BASE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_GENERIC_ERROR: i32 = -2147467259i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_INVALIDPARAMETER: i32 = -2147024809i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_NOTIMPLEMENTED: i32 = -2147467263i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_ERR_OUTOFMEMORY: i32 = -2147024882i32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_SDK_VERSION: u32 = 567u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_SDK_VERSION1: u32 = 566u32;
#[doc = "*Required features: `\"Win32_Graphics_Imaging\"`*"]
pub const WINCODEC_SDK_VERSION2: u32 = 567u32;
