#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListAllocateBufferAndSerialize(sourcecollection: *const SENSOR_COLLECTION_LIST, ptargetbuffersizeinbytes: *mut u32, ptargetbuffer: *mut *mut u8) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListCopyAndMarshall(target: *mut SENSOR_COLLECTION_LIST, source: *const SENSOR_COLLECTION_LIST) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListDeserializeFromBuffer(sourcebuffersizeinbytes: u32, sourcebuffer: *const u8, targetcollection: *mut SENSOR_COLLECTION_LIST) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
    pub fn CollectionsListGetFillableCount(buffersizebytes: u32) -> u32;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListGetMarshalledSize(collection: *const SENSOR_COLLECTION_LIST) -> u32;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListGetMarshalledSizeWithoutSerialization(collection: *const SENSOR_COLLECTION_LIST) -> u32;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListGetSerializedSize(collection: *const SENSOR_COLLECTION_LIST) -> u32;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListMarshall(target: *mut SENSOR_COLLECTION_LIST) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListSerializeToBuffer(sourcecollection: *const SENSOR_COLLECTION_LIST, targetbuffersizeinbytes: u32, targetbuffer: *mut u8) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListSortSubscribedActivitiesByConfidence(thresholds: *const SENSOR_COLLECTION_LIST, pcollection: *mut SENSOR_COLLECTION_LIST) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn CollectionsListUpdateMarshalledPointer(collection: *mut SENSOR_COLLECTION_LIST) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn EvaluateActivityThresholds(newsample: *const SENSOR_COLLECTION_LIST, oldsample: *const SENSOR_COLLECTION_LIST, thresholds: *const SENSOR_COLLECTION_LIST) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetPerformanceTime(timems: *mut u32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromCLSIDArray(members: *const ::windows_sys::core::GUID, size: u32, ppropvar: *mut super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromFloat(fltval: f32, ppropvar: *mut super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn IsCollectionListSame(lista: *const SENSOR_COLLECTION_LIST, listb: *const SENSOR_COLLECTION_LIST) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsGUIDPresentInList(guidarray: *const ::windows_sys::core::GUID, arraylength: u32, guidelem: *const ::windows_sys::core::GUID) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn IsKeyPresentInCollectionList(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn IsKeyPresentInPropertyList(plist: *const SENSOR_PROPERTY_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn IsSensorSubscribed(subscriptionlist: *const SENSOR_COLLECTION_LIST, currenttype: ::windows_sys::core::GUID) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetBool(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut super::super::Foundation::BOOL) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetDouble(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut f64) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetFileTime(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut super::super::Foundation::FILETIME) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetFloat(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut f32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetGuid(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut ::windows_sys::core::GUID) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetInt32(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut i32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetInt64(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut i64) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetNthInt64(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, occurrence: u32, pretvalue: *mut i64) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetNthUlong(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, occurrence: u32, pretvalue: *mut u32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetNthUshort(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, occurrence: u32, pretvalue: *mut u16) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetPropVariant(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, typecheck: super::super::Foundation::BOOLEAN, pvalue: *mut super::super::System::Com::StructuredStorage::PROPVARIANT) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetUlong(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut u32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeyGetUshort(plist: *const SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pretvalue: *mut u16) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropKeyFindKeySetPropVariant(plist: *mut SENSOR_COLLECTION_LIST, pkey: *const super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, typecheck: super::super::Foundation::BOOLEAN, pvalue: *const super::super::System::Com::StructuredStorage::PROPVARIANT) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetInformation(propvariantvalue: *const super::super::System::Com::StructuredStorage::PROPVARIANT, propvariantoffset: *mut u32, propvariantsize: *mut u32, propvariantpointer: *mut *mut ::core::ffi::c_void, remappedtype: *mut u32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn PropertiesListCopy(target: *mut SENSOR_PROPERTY_LIST, source: *const SENSOR_PROPERTY_LIST) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
    pub fn PropertiesListGetFillableCount(buffersizebytes: u32) -> u32;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn SensorCollectionGetAt(index: u32, psensorslist: *const SENSOR_COLLECTION_LIST, pkey: *mut super::super::UI::Shell::PropertiesSystem::PROPERTYKEY, pvalue: *mut super::super::System::Com::StructuredStorage::PROPVARIANT) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SerializationBufferAllocate(sizeinbytes: u32, pbuffer: *mut *mut u8) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
    pub fn SerializationBufferFree(buffer: *const u8);
}
pub type ILocationPermissions = *mut ::core::ffi::c_void;
pub type ISensor = *mut ::core::ffi::c_void;
pub type ISensorCollection = *mut ::core::ffi::c_void;
pub type ISensorDataReport = *mut ::core::ffi::c_void;
pub type ISensorEvents = *mut ::core::ffi::c_void;
pub type ISensorManager = *mut ::core::ffi::c_void;
pub type ISensorManagerEvents = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const GNSS_CLEAR_ALL_ASSISTANCE_DATA: u32 = 1u32;
pub const GUID_DEVINTERFACE_SENSOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3122378386, data2: 39802, data3: 18483, data4: [154, 30, 82, 94, 209, 52, 231, 226] };
pub const GUID_SensorCategory_All: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3273114246, data2: 50280, data3: 17032, data4: [153, 117, 212, 196, 88, 124, 68, 44] };
pub const GUID_SensorCategory_Biometric: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3390662927, data2: 41671, data3: 18301, data4: [169, 158, 153, 236, 110, 43, 86, 72] };
pub const GUID_SensorCategory_Electrical: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4218682584, data2: 64586, data3: 18492, data4: [172, 88, 39, 182, 145, 198, 190, 255] };
pub const GUID_SensorCategory_Environmental: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 842283434, data2: 32614, data3: 18731, data4: [186, 12, 115, 233, 170, 10, 101, 213] };
pub const GUID_SensorCategory_Light: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 396780992, data2: 36963, data3: 16918, data4: [178, 2, 92, 122, 37, 94, 24, 206] };
pub const GUID_SensorCategory_Location: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3215430884, data2: 63844, data3: 20443, data4: [144, 246, 81, 5, 107, 254, 75, 68] };
pub const GUID_SensorCategory_Mechanical: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2366840168, data2: 36599, data3: 18006, data4: [128, 181, 204, 203, 217, 55, 145, 197] };
pub const GUID_SensorCategory_Motion: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3439975153, data2: 15150, data3: 19517, data4: [181, 152, 181, 229, 255, 147, 253, 70] };
pub const GUID_SensorCategory_Orientation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2657879222, data2: 38654, data3: 18772, data4: [183, 38, 104, 104, 42, 71, 63, 105] };
pub const GUID_SensorCategory_Other: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 747693993, data2: 62665, data3: 20386, data4: [175, 55, 86, 212, 113, 254, 90, 61] };
pub const GUID_SensorCategory_PersonalActivity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4049637505, data2: 7698, data3: 16683, data4: [161, 77, 203, 176, 233, 91, 210, 229] };
pub const GUID_SensorCategory_Scanner: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2952849278, data2: 62901, data3: 16911, data4: [129, 93, 2, 112, 167, 38, 242, 112] };
pub const GUID_SensorCategory_Unsupported: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 736815098, data2: 6576, data3: 18629, data4: [161, 246, 181, 72, 13, 194, 6, 176] };
pub const GUID_SensorType_Accelerometer3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3271233375, data2: 58066, data3: 19576, data4: [188, 208, 53, 42, 149, 130, 129, 157] };
pub const GUID_SensorType_ActivityDetection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2644377880, data2: 6151, data3: 20270, data4: [150, 228, 44, 229, 113, 66, 225, 150] };
pub const GUID_SensorType_AmbientLight: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2549159368, data2: 22938, data3: 16723, data4: [136, 148, 210, 209, 40, 153, 145, 138] };
pub const GUID_SensorType_Barometer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 244332585, data2: 65418, data3: 19091, data4: [151, 223, 61, 203, 222, 64, 34, 136] };
pub const GUID_SensorType_Custom: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3896177193, data2: 34368, data3: 19736, data4: [162, 19, 226, 38, 117, 235, 178, 195] };
pub const GUID_SensorType_FloorElevation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2917439615, data2: 31428, data3: 19962, data4: [151, 34, 10, 2, 113, 129, 199, 71] };
pub const GUID_SensorType_GeomagneticOrientation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3882980856, data2: 11551, data3: 18467, data4: [151, 27, 28, 68, 103, 85, 108, 157] };
pub const GUID_SensorType_GravityVector: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 62205043, data2: 47990, data3: 17983, data4: [149, 36, 56, 222, 118, 235, 112, 11] };
pub const GUID_SensorType_Gyrometer3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 155737946, data2: 30110, data3: 17090, data4: [189, 75, 163, 73, 183, 92, 134, 67] };
pub const GUID_SensorType_HingeAngle: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2184544357, data2: 62660, data3: 19873, data4: [178, 114, 19, 194, 51, 50, 162, 7] };
pub const GUID_SensorType_Humidity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1551023975, data2: 48510, data3: 16983, data4: [153, 11, 152, 163, 186, 59, 64, 10] };
pub const GUID_SensorType_LinearAccelerometer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 59441795, data2: 38836, data3: 16840, data4: [188, 36, 95, 241, 170, 72, 254, 199] };
pub const GUID_SensorType_Magnetometer3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1441132539, data2: 5575, data3: 16607, data4: [134, 152, 168, 75, 124, 134, 60, 83] };
pub const GUID_SensorType_Orientation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3451246839, data2: 15613, data3: 16840, data4: [133, 66, 204, 230, 34, 207, 93, 110] };
pub const GUID_SensorType_Pedometer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2980022703, data2: 58347, data3: 17483, data4: [141, 234, 32, 37, 117, 167, 21, 153] };
pub const GUID_SensorType_Proximity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1377884905, data2: 12665, data3: 17456, data4: [159, 144, 6, 38, 109, 42, 52, 222] };
pub const GUID_SensorType_RelativeOrientation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1083784017, data2: 18182, data3: 17628, data4: [152, 213, 201, 32, 192, 55, 255, 171] };
pub const GUID_SensorType_SimpleDeviceOrientation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2258735761, data2: 1154, data3: 16428, data4: [191, 76, 173, 218, 197, 43, 28, 57] };
pub const GUID_SensorType_Temperature: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 83693252, data2: 54746, data3: 17914, data4: [149, 169, 93, 179, 142, 225, 147, 6] };
pub const SENSOR_CATEGORY_ALL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3273114246, data2: 50280, data3: 17032, data4: [153, 117, 212, 196, 88, 124, 68, 44] };
pub const SENSOR_CATEGORY_BIOMETRIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3390662927, data2: 41671, data3: 18301, data4: [169, 158, 153, 236, 110, 43, 86, 72] };
pub const SENSOR_CATEGORY_ELECTRICAL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4218682584, data2: 64586, data3: 18492, data4: [172, 88, 39, 182, 145, 198, 190, 255] };
pub const SENSOR_CATEGORY_ENVIRONMENTAL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 842283434, data2: 32614, data3: 18731, data4: [186, 12, 115, 233, 170, 10, 101, 213] };
pub const SENSOR_CATEGORY_LIGHT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 396780992, data2: 36963, data3: 16918, data4: [178, 2, 92, 122, 37, 94, 24, 206] };
pub const SENSOR_CATEGORY_LOCATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3215430884, data2: 63844, data3: 20443, data4: [144, 246, 81, 5, 107, 254, 75, 68] };
pub const SENSOR_CATEGORY_MECHANICAL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2366840168, data2: 36599, data3: 18006, data4: [128, 181, 204, 203, 217, 55, 145, 197] };
pub const SENSOR_CATEGORY_MOTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3439975153, data2: 15150, data3: 19517, data4: [181, 152, 181, 229, 255, 147, 253, 70] };
pub const SENSOR_CATEGORY_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2657879222, data2: 38654, data3: 18772, data4: [183, 38, 104, 104, 42, 71, 63, 105] };
pub const SENSOR_CATEGORY_OTHER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 747693993, data2: 62665, data3: 20386, data4: [175, 55, 86, 212, 113, 254, 90, 61] };
pub const SENSOR_CATEGORY_SCANNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2952849278, data2: 62901, data3: 16911, data4: [129, 93, 2, 112, 167, 38, 242, 112] };
pub const SENSOR_CATEGORY_UNSUPPORTED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 736815098, data2: 6576, data3: 18629, data4: [161, 246, 181, 72, 13, 194, 6, 176] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ABSOLUTE_PRESSURE_PASCAL: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ACCELERATION_X_G: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ACCELERATION_Y_G: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ACCELERATION_Z_G: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ADDRESS1: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 23u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ADDRESS2: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 24u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ALTITUDE_ANTENNA_SEALEVEL_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 36u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ALTITUDE_ELLIPSOID_ERROR_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 29u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ALTITUDE_ELLIPSOID_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ALTITUDE_SEALEVEL_ERROR_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 30u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ALTITUDE_SEALEVEL_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ANGULAR_ACCELERATION_X_DEGREES_PER_SECOND_SQUARED: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ANGULAR_ACCELERATION_Y_DEGREES_PER_SECOND_SQUARED: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ANGULAR_ACCELERATION_Z_DEGREES_PER_SECOND_SQUARED: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ANGULAR_VELOCITY_X_DEGREES_PER_SECOND: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ANGULAR_VELOCITY_Y_DEGREES_PER_SECOND: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 11u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ANGULAR_VELOCITY_Z_DEGREES_PER_SECOND: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 12u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ATMOSPHERIC_PRESSURE_BAR: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2332730097, data2: 11607, data3: 17134, data4: [140, 192, 77, 39, 98, 43, 70, 196] }, pid: 4u32 };
pub const SENSOR_DATA_TYPE_BIOMETRIC_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 580462730, data2: 28062, data3: 19211, data4: [183, 236, 53, 40, 248, 158, 64, 175] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_BOOLEAN_SWITCH_ARRAY_STATES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_BOOLEAN_SWITCH_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CAPACITANCE_FARAD: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CITY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 25u32 };
pub const SENSOR_DATA_TYPE_COMMON_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3680374002, data2: 53023, data3: 19480, data4: [180, 108, 216, 96, 17, 214, 33, 80] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_COUNTRY_REGION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 28u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CURRENT_AMPS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_BOOLEAN_ARRAY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 6u32 };
pub const SENSOR_DATA_TYPE_CUSTOM_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_USAGE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE1: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE10: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 16u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE11: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 17u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE12: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 18u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE13: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 19u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE14: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 20u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE15: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 21u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE16: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 22u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE17: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 23u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE18: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 24u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE19: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 25u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE2: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE20: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 26u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE21: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 27u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE22: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 28u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE23: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 29u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE24: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 30u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE25: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 31u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE26: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 32u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE27: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 33u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE28: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 34u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE3: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 9u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE4: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE5: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 11u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE6: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 12u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE7: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 13u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE8: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 14u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_CUSTOM_VALUE9: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2974578255, data2: 1999, data3: 16872, data4: [157, 130, 235, 227, 208, 119, 106, 111] }, pid: 15u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_DGPS_DATA_AGE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 35u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_DIFFERENTIAL_REFERENCE_STATION_ID: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 37u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_DISTANCE_X_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_DISTANCE_Y_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 9u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_DISTANCE_Z_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ELECTRICAL_FREQUENCY_HERTZ: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 9u32 };
pub const SENSOR_DATA_TYPE_ELECTRICAL_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ELECTRICAL_PERCENT_OF_RANGE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ELECTRICAL_POWER_WATTS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 7u32 };
pub const SENSOR_DATA_TYPE_ENVIRONMENTAL_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2332730097, data2: 11607, data3: 17134, data4: [140, 192, 77, 39, 98, 43, 70, 196] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ERROR_RADIUS_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 22u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_FIX_QUALITY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_FIX_TYPE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 11u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_FORCE_NEWTONS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_GAUGE_PRESSURE_PASCAL: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_GEOIDAL_SEPARATION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 34u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_GPS_OPERATION_MODE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 32u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_GPS_SELECTION_MODE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 31u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_GPS_STATUS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 33u32 };
pub const SENSOR_DATA_TYPE_GUID_MECHANICAL_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_HORIZONAL_DILUTION_OF_PRECISION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 13u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_HUMAN_PRESENCE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 580462730, data2: 28062, data3: 19211, data4: [183, 236, 53, 40, 248, 158, 64, 175] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_HUMAN_PROXIMITY_METERS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 580462730, data2: 28062, data3: 19211, data4: [183, 236, 53, 40, 248, 158, 64, 175] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_INDUCTANCE_HENRY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_LATITUDE_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_LIGHT_CHROMACITY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3838278882, data2: 56503, data3: 18153, data4: [132, 57, 79, 236, 84, 136, 51, 166] }, pid: 4u32 };
pub const SENSOR_DATA_TYPE_LIGHT_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3838278882, data2: 56503, data3: 18153, data4: [132, 57, 79, 236, 84, 136, 51, 166] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_LIGHT_LEVEL_LUX: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3838278882, data2: 56503, data3: 18153, data4: [132, 57, 79, 236, 84, 136, 51, 166] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_LIGHT_TEMPERATURE_KELVIN: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3838278882, data2: 56503, data3: 18153, data4: [132, 57, 79, 236, 84, 136, 51, 166] }, pid: 3u32 };
pub const SENSOR_DATA_TYPE_LOCATION_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_LOCATION_SOURCE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 40u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_LONGITUDE_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_FIELD_STRENGTH_X_MILLIGAUSS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 19u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_FIELD_STRENGTH_Y_MILLIGAUSS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 20u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_FIELD_STRENGTH_Z_MILLIGAUSS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 21u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_COMPENSATED_MAGNETIC_NORTH_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 11u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_COMPENSATED_TRUE_NORTH_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 12u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_MAGNETIC_NORTH_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 13u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_TRUE_NORTH_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 14u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_X_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_Y_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_HEADING_Z_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETIC_VARIATION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 9u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MAGNETOMETER_ACCURACY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 22u32 };
pub const SENSOR_DATA_TYPE_MOTION_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MOTION_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 9u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_MULTIVALUE_SWITCH_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_NMEA_SENTENCE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 38u32 };
pub const SENSOR_DATA_TYPE_ORIENTATION_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_POSITION_DILUTION_OF_PRECISION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 12u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_POSTALCODE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 27u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_QUADRANT_ANGLE_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 15u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_QUATERNION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 17u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_RELATIVE_HUMIDITY_PERCENT: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2332730097, data2: 11607, data3: 17134, data4: [140, 192, 77, 39, 98, 43, 70, 196] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_RESISTANCE_OHMS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_RFID_TAG_40_BIT: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3617954364, data2: 13345, data3: 17579, data4: [141, 58, 157, 232, 171, 108, 76, 174] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_ROTATION_MATRIX: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 16u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_IN_VIEW: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 17u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_IN_VIEW_AZIMUTH: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 20u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_IN_VIEW_ELEVATION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 19u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_IN_VIEW_ID: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 39u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_IN_VIEW_PRNS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 18u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_IN_VIEW_STN_RATIO: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 21u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_USED_COUNT: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 15u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_USED_PRNS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 16u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SATELLITES_USED_PRNS_AND_CONSTELLATIONS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 41u32 };
pub const SENSOR_DATA_TYPE_SCANNER_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3617954364, data2: 13345, data3: 17579, data4: [141, 58, 157, 232, 171, 108, 76, 174] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SIMPLE_DEVICE_ORIENTATION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 18u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SPEED_KNOTS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_SPEED_METERS_PER_SECOND: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1066035618, data2: 1989, data3: 20040, data4: [169, 101, 205, 121, 122, 171, 86, 213] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_STATE_PROVINCE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 26u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_STRAIN: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TEMPERATURE_CELSIUS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2332730097, data2: 11607, data3: 17134, data4: [140, 192, 77, 39, 98, 43, 70, 196] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TILT_X_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TILT_Y_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TILT_Z_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 372758690, data2: 16968, data3: 17013, data4: [134, 93, 85, 141, 232, 74, 237, 253] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TIMESTAMP: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3680374002, data2: 53023, data3: 19480, data4: [180, 108, 216, 96, 17, 214, 33, 80] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TOUCH_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 580462730, data2: 28062, data3: 19211, data4: [183, 236, 53, 40, 248, 158, 64, 175] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_TRUE_HEADING_DEGREES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_VERTICAL_DILUTION_OF_PRECISION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 89945304, data2: 51823, data3: 18390, data4: [149, 198, 30, 211, 99, 122, 15, 244] }, pid: 14u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_VOLTAGE_VOLTS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3149022929, data2: 57922, data3: 18304, data4: [162, 211, 205, 237, 132, 243, 88, 66] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_WEIGHT_KILOGRAMS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 945179260, data2: 62194, data3: 18875, data4: [155, 43, 186, 96, 246, 106, 88, 223] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_WIND_DIRECTION_DEGREES_ANTICLOCKWISE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2332730097, data2: 11607, data3: 17134, data4: [140, 192, 77, 39, 98, 43, 70, 196] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_DATA_TYPE_WIND_SPEED_METERS_PER_SECOND: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2332730097, data2: 11607, data3: 17134, data4: [140, 192, 77, 39, 98, 43, 70, 196] }, pid: 6u32 };
pub const SENSOR_ERROR_PARAMETER_COMMON_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1997614029, data2: 64737, data3: 20291, data4: [184, 184, 168, 130, 86, 173, 180, 179] };
pub const SENSOR_EVENT_ACCELEROMETER_SHAKE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2187287188, data2: 3912, data3: 17302, data4: [156, 160, 110, 203, 92, 153, 217, 21] };
pub const SENSOR_EVENT_DATA_UPDATED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 785445540, data2: 135, data3: 16851, data4: [135, 219, 103, 115, 55, 11, 60, 136] };
pub const SENSOR_EVENT_PARAMETER_COMMON_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1681157680, data2: 34600, data3: 19252, data4: [189, 246, 79, 82, 68, 44, 92, 40] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_EVENT_PARAMETER_EVENT_ID: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1681157680, data2: 34600, data3: 19252, data4: [189, 246, 79, 82, 68, 44, 92, 40] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_EVENT_PARAMETER_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 1681157680, data2: 34600, data3: 19252, data4: [189, 246, 79, 82, 68, 44, 92, 40] }, pid: 3u32 };
pub const SENSOR_EVENT_PROPERTY_CHANGED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 593031321, data2: 33993, data3: 19773, data4: [144, 223, 194, 66, 30, 43, 32, 69] };
pub const SENSOR_EVENT_STATE_CHANGED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3218694166, data2: 27607, data3: 17760, data4: [173, 52, 242, 246, 96, 126, 143, 129] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_ACCURACY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 17u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_CHANGE_SENSITIVITY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 14u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_CLEAR_ASSISTANCE_DATA: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3790168820, data2: 28261, data3: 17911, data4: [156, 54, 212, 135, 183, 177, 189, 52] }, pid: 2u32 };
pub const SENSOR_PROPERTY_COMMON_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_CONNECTION_TYPE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 11u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_CURRENT_REPORT_INTERVAL: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 13u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_DESCRIPTION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_DEVICE_PATH: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 15u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_FRIENDLY_NAME: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 9u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_HID_USAGE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 22u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_LIGHT_RESPONSE_CURVE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 16u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_PROPERTY_LIST_HEADER_SIZE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_LOCATION_DESIRED_ACCURACY: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 19u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_MANUFACTURER: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_MIN_REPORT_INTERVAL: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 12u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_MODEL: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_PERSISTENT_UNIQUE_ID: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_RADIO_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 23u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_RADIO_STATE_PREVIOUS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 24u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_RANGE_MAXIMUM: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 21u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_RANGE_MINIMUM: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 20u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_RESOLUTION: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 18u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_SERIAL_NUMBER: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_STATE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 3u32 };
pub const SENSOR_PROPERTY_TEST_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3790168820, data2: 28261, data3: 17911, data4: [156, 54, 212, 135, 183, 177, 189, 52] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_TURN_ON_OFF_NMEA: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 3790168820, data2: 28261, data3: 17911, data4: [156, 54, 212, 135, 183, 177, 189, 52] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SENSOR_PROPERTY_TYPE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2139325420, data2: 54252, data3: 18780, data4: [168, 207, 184, 187, 232, 92, 41, 32] }, pid: 2u32 };
pub const SENSOR_TYPE_ACCELEROMETER_1D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3226280839, data2: 29504, data3: 19650, data4: [153, 30, 59, 24, 203, 142, 242, 244] };
pub const SENSOR_TYPE_ACCELEROMETER_2D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2999261096, data2: 63157, data3: 19366, data4: [164, 35, 93, 245, 96, 180, 204, 7] };
pub const SENSOR_TYPE_ACCELEROMETER_3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3271233375, data2: 58066, data3: 19576, data4: [188, 208, 53, 42, 149, 130, 129, 157] };
pub const SENSOR_TYPE_AGGREGATED_DEVICE_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3451246839, data2: 15613, data3: 16840, data4: [133, 66, 204, 230, 34, 207, 93, 110] };
pub const SENSOR_TYPE_AGGREGATED_QUADRANT_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2676093359, data2: 50347, data3: 17159, data4: [153, 4, 200, 40, 191, 185, 8, 41] };
pub const SENSOR_TYPE_AGGREGATED_SIMPLE_DEVICE_ORIENTATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2258735761, data2: 1154, data3: 16428, data4: [191, 76, 173, 218, 197, 43, 28, 57] };
pub const SENSOR_TYPE_AMBIENT_LIGHT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2549159368, data2: 22938, data3: 16723, data4: [136, 148, 210, 209, 40, 153, 145, 138] };
pub const SENSOR_TYPE_BARCODE_SCANNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2567650703, data2: 34235, data3: 17919, data4: [145, 77, 153, 140, 4, 243, 114, 223] };
pub const SENSOR_TYPE_BOOLEAN_SWITCH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2625517343, data2: 4161, data3: 17931, data4: [141, 92, 113, 228, 117, 46, 53, 12] };
pub const SENSOR_TYPE_BOOLEAN_SWITCH_ARRAY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1415351205, data2: 45379, data3: 17733, data4: [134, 143, 202, 127, 217, 134, 180, 246] };
pub const SENSOR_TYPE_CAPACITANCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3392142108, data2: 8983, data3: 18880, data4: [160, 180, 182, 60, 230, 52, 97, 160] };
pub const SENSOR_TYPE_COMPASS_1D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2752902853, data2: 52048, data3: 18896, data4: [142, 98, 168, 39, 11, 215, 162, 108] };
pub const SENSOR_TYPE_COMPASS_2D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 358964416, data2: 39290, data3: 19760, data4: [132, 219, 87, 202, 186, 54, 72, 187] };
pub const SENSOR_TYPE_COMPASS_3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1991626253, data2: 6109, data3: 16717, data4: [147, 161, 225, 39, 244, 11, 223, 110] };
pub const SENSOR_TYPE_CURRENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1524408270, data2: 5536, data3: 19390, data4: [161, 173, 45, 56, 169, 174, 131, 28] };
pub const SENSOR_TYPE_CUSTOM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3896177193, data2: 34368, data3: 19736, data4: [162, 19, 226, 38, 117, 235, 178, 195] };
pub const SENSOR_TYPE_DISTANCE_1D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1595190063, data2: 5127, data3: 17158, data4: [169, 63, 177, 219, 171, 228, 249, 192] };
pub const SENSOR_TYPE_DISTANCE_2D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1559864428, data2: 43426, data3: 20053, data4: [182, 161, 160, 74, 175, 169, 90, 146] };
pub const SENSOR_TYPE_DISTANCE_3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2718740017, data2: 3621, data3: 18290, data4: [159, 229, 150, 96, 138, 19, 84, 178] };
pub const SENSOR_TYPE_ELECTRICAL_POWER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 556732661, data2: 5291, data3: 17270, data4: [154, 67, 167, 121, 64, 152, 194, 254] };
pub const SENSOR_TYPE_ENVIRONMENTAL_ATMOSPHERIC_PRESSURE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 244332585, data2: 65418, data3: 19091, data4: [151, 223, 61, 203, 222, 64, 34, 136] };
pub const SENSOR_TYPE_ENVIRONMENTAL_HUMIDITY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1551023975, data2: 48510, data3: 16983, data4: [153, 11, 152, 163, 186, 59, 64, 10] };
pub const SENSOR_TYPE_ENVIRONMENTAL_TEMPERATURE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 83693252, data2: 54746, data3: 17914, data4: [149, 169, 93, 179, 142, 225, 147, 6] };
pub const SENSOR_TYPE_ENVIRONMENTAL_WIND_DIRECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2666887733, data2: 37638, data3: 17229, data4: [175, 9, 55, 250, 90, 156, 0, 189] };
pub const SENSOR_TYPE_ENVIRONMENTAL_WIND_SPEED: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3713032315, data2: 42079, data3: 17101, data4: [142, 253, 236, 97, 118, 28, 66, 38] };
pub const SENSOR_TYPE_FORCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3265997570, data2: 6684, data3: 18296, data4: [168, 27, 149, 74, 23, 136, 204, 117] };
pub const SENSOR_TYPE_FREQUENCY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2362624950, data2: 29670, data3: 17984, data4: [167, 9, 114, 174, 143, 182, 13, 127] };
pub const SENSOR_TYPE_GYROMETER_1D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4194862900, data2: 62802, data3: 17796, data4: [131, 36, 237, 250, 246, 73, 101, 44] };
pub const SENSOR_TYPE_GYROMETER_2D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 837767043, data2: 37275, data3: 18623, data4: [141, 224, 93, 122, 157, 36, 5, 86] };
pub const SENSOR_TYPE_GYROMETER_3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 155737946, data2: 30110, data3: 17090, data4: [189, 75, 163, 73, 183, 92, 134, 67] };
pub const SENSOR_TYPE_HUMAN_PRESENCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3241722155, data2: 44370, data3: 17692, data4: [147, 117, 135, 245, 24, 255, 16, 198] };
pub const SENSOR_TYPE_HUMAN_PROXIMITY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1377884905, data2: 12665, data3: 17456, data4: [159, 144, 6, 38, 109, 42, 52, 222] };
pub const SENSOR_TYPE_INCLINOMETER_1D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3111098565, data2: 31349, data3: 19367, data4: [148, 233, 172, 134, 140, 150, 109, 216] };
pub const SENSOR_TYPE_INCLINOMETER_2D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2870218605, data2: 33771, data3: 16996, data4: [183, 11, 177, 106, 91, 37, 106, 1] };
pub const SENSOR_TYPE_INCLINOMETER_3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3091798523, data2: 60037, data3: 18806, data4: [132, 68, 111, 111, 92, 109, 49, 219] };
pub const SENSOR_TYPE_INDUCTANCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3692925759, data2: 50229, data3: 19581, data4: [162, 254, 96, 113, 146, 165, 36, 211] };
pub const SENSOR_TYPE_LOCATION_BROADCAST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3530131663, data2: 20834, data3: 16441, data4: [187, 23, 76, 88, 182, 152, 228, 74] };
pub const SENSOR_TYPE_LOCATION_DEAD_RECKONING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 439866680, data2: 62091, data3: 17114, data4: [159, 206, 169, 208, 162, 166, 216, 41] };
pub const SENSOR_TYPE_LOCATION_GPS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3981223305, data2: 12922, data3: 20473, data4: [165, 96, 145, 218, 75, 72, 39, 94] };
pub const SENSOR_TYPE_LOCATION_LOOKUP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 992915018, data2: 29390, data3: 17261, data4: [150, 210, 60, 91, 133, 112, 233, 135] };
pub const SENSOR_TYPE_LOCATION_OTHER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2603418982, data2: 872, data3: 20337, data4: [184, 141, 83, 63, 19, 32, 49, 222] };
pub const SENSOR_TYPE_LOCATION_STATIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 157254020, data2: 4009, data3: 17477, data4: [142, 110, 183, 15, 50, 11, 107, 76] };
pub const SENSOR_TYPE_LOCATION_TRIANGULATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1763456026, data2: 21510, data3: 20449, data4: [148, 47, 34, 70, 203, 235, 57, 224] };
pub const SENSOR_TYPE_MOTION_DETECTOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1551637010, data2: 12453, data3: 17337, data4: [164, 178, 207, 9, 236, 91, 123, 232] };
pub const SENSOR_TYPE_MULTIVALUE_SWITCH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3018739062, data2: 14244, data3: 17410, data4: [178, 94, 153, 198, 10, 119, 95, 161] };
pub const SENSOR_TYPE_POTENTIOMETER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 724992425, data2: 51932, data3: 17834, data4: [166, 255, 84, 149, 124, 139, 180, 64] };
pub const SENSOR_TYPE_PRESSURE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 651370292, data2: 25426, data3: 16847, data4: [183, 147, 234, 7, 19, 213, 61, 119] };
pub const SENSOR_TYPE_RESISTANCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2576601800, data2: 49495, data3: 19026, data4: [167, 181, 25, 92, 118, 3, 114, 49] };
pub const SENSOR_TYPE_RFID_SCANNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1144164085, data2: 733, data3: 20109, data4: [173, 93, 146, 73, 131, 43, 46, 202] };
pub const SENSOR_TYPE_SCALE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3228424492, data2: 32747, data3: 17294, data4: [155, 246, 130, 32, 127, 255, 91, 184] };
pub const SENSOR_TYPE_SPEEDOMETER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1809267743, data2: 2996, data3: 17168, data4: [129, 178, 223, 193, 138, 82, 191, 148] };
pub const SENSOR_TYPE_STRAIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3335646222, data2: 26627, data3: 17249, data4: [173, 61, 133, 188, 197, 140, 109, 41] };
pub const SENSOR_TYPE_TOUCH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 400240664, data2: 1732, data3: 20349, data4: [129, 175, 146, 116, 183, 89, 156, 39] };
pub const SENSOR_TYPE_UNKNOWN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 280658915, data2: 61263, data3: 16877, data4: [152, 133, 168, 125, 100, 53, 168, 225] };
pub const SENSOR_TYPE_VOLTAGE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3309848119, data2: 20407, data3: 18771, data4: [152, 184, 165, 109, 138, 161, 251, 30] };
pub const Sensor: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3917278464, data2: 21050, data3: 16691, data4: [191, 111, 211, 162, 218, 231, 246, 186] };
pub const SensorCollection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2042903259, data2: 42025, data3: 18079, data4: [170, 57, 47, 43, 116, 183, 89, 55] };
pub const SensorDataReport: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1319753455, data2: 26955, data3: 16920, data4: [136, 22, 204, 218, 141, 167, 75, 186] };
pub const SensorManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2007091239, data2: 64722, data3: 18057, data4: [137, 21, 157, 97, 60, 197, 250, 62] };
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type ACTIVITY_STATE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Unknown: ACTIVITY_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Stationary: ACTIVITY_STATE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Fidgeting: ACTIVITY_STATE = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Walking: ACTIVITY_STATE = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Running: ACTIVITY_STATE = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_InVehicle: ACTIVITY_STATE = 32i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Biking: ACTIVITY_STATE = 64i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Idle: ACTIVITY_STATE = 128i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Max: ACTIVITY_STATE = 256i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityState_Force_Dword: ACTIVITY_STATE = -1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type ACTIVITY_STATE_COUNT = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ActivityStateCount: ACTIVITY_STATE_COUNT = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type AXIS = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const AXIS_X: AXIS = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const AXIS_Y: AXIS = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const AXIS_Z: AXIS = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const AXIS_MAX: AXIS = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type ELEVATION_CHANGE_MODE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ElevationChangeMode_Unknown: ELEVATION_CHANGE_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ElevationChangeMode_Elevator: ELEVATION_CHANGE_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ElevationChangeMode_Stepping: ELEVATION_CHANGE_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ElevationChangeMode_Max: ELEVATION_CHANGE_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ElevationChangeMode_Force_Dword: ELEVATION_CHANGE_MODE = -1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type HUMAN_PRESENCE_DETECTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const HumanPresenceDetectionType_VendorDefinedNonBiometric: HUMAN_PRESENCE_DETECTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const HumanPresenceDetectionType_VendorDefinedBiometric: HUMAN_PRESENCE_DETECTION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const HumanPresenceDetectionType_FacialBiometric: HUMAN_PRESENCE_DETECTION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const HumanPresenceDetectionType_AudioBiometric: HUMAN_PRESENCE_DETECTION_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const HumanPresenceDetectionType_Force_Dword: HUMAN_PRESENCE_DETECTION_TYPE = -1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type HUMAN_PRESENCE_DETECTION_TYPE_COUNT = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const HumanPresenceDetectionTypeCount: HUMAN_PRESENCE_DETECTION_TYPE_COUNT = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type LOCATION_DESIRED_ACCURACY = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_DESIRED_ACCURACY_DEFAULT: LOCATION_DESIRED_ACCURACY = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_DESIRED_ACCURACY_HIGH: LOCATION_DESIRED_ACCURACY = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type LOCATION_POSITION_SOURCE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_POSITION_SOURCE_CELLULAR: LOCATION_POSITION_SOURCE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_POSITION_SOURCE_SATELLITE: LOCATION_POSITION_SOURCE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_POSITION_SOURCE_WIFI: LOCATION_POSITION_SOURCE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_POSITION_SOURCE_IPADDRESS: LOCATION_POSITION_SOURCE = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const LOCATION_POSITION_SOURCE_UNKNOWN: LOCATION_POSITION_SOURCE = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type MAGNETOMETER_ACCURACY = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MagnetometerAccuracy_Unknown: MAGNETOMETER_ACCURACY = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MagnetometerAccuracy_Unreliable: MAGNETOMETER_ACCURACY = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MagnetometerAccuracy_Approximate: MAGNETOMETER_ACCURACY = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MagnetometerAccuracy_High: MAGNETOMETER_ACCURACY = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type MagnetometerAccuracy = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MAGNETOMETER_ACCURACY_UNKNOWN: MagnetometerAccuracy = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MAGNETOMETER_ACCURACY_UNRELIABLE: MagnetometerAccuracy = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MAGNETOMETER_ACCURACY_APPROXIMATE: MagnetometerAccuracy = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const MAGNETOMETER_ACCURACY_HIGH: MagnetometerAccuracy = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type PEDOMETER_STEP_TYPE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const PedometerStepType_Unknown: PEDOMETER_STEP_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const PedometerStepType_Walking: PEDOMETER_STEP_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const PedometerStepType_Running: PEDOMETER_STEP_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const PedometerStepType_Max: PEDOMETER_STEP_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const PedometerStepType_Force_Dword: PEDOMETER_STEP_TYPE = -1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type PEDOMETER_STEP_TYPE_COUNT = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const PedometerStepTypeCount: PEDOMETER_STEP_TYPE_COUNT = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type PROXIMITY_TYPE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ProximityType_ObjectProximity: PROXIMITY_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ProximityType_HumanProximity: PROXIMITY_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const ProximityType_Force_Dword: PROXIMITY_TYPE = -1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type SENSOR_CONNECTION_TYPES = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorConnectionType_Integrated: SENSOR_CONNECTION_TYPES = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorConnectionType_Attached: SENSOR_CONNECTION_TYPES = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorConnectionType_External: SENSOR_CONNECTION_TYPES = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type SENSOR_STATE = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorState_Initializing: SENSOR_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorState_Idle: SENSOR_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorState_Active: SENSOR_STATE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SensorState_Error: SENSOR_STATE = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type SIMPLE_DEVICE_ORIENTATION = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SimpleDeviceOrientation_NotRotated: SIMPLE_DEVICE_ORIENTATION = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SimpleDeviceOrientation_Rotated90DegreesCounterclockwise: SIMPLE_DEVICE_ORIENTATION = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SimpleDeviceOrientation_Rotated180DegreesCounterclockwise: SIMPLE_DEVICE_ORIENTATION = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SimpleDeviceOrientation_Rotated270DegreesCounterclockwise: SIMPLE_DEVICE_ORIENTATION = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SimpleDeviceOrientation_Faceup: SIMPLE_DEVICE_ORIENTATION = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SimpleDeviceOrientation_Facedown: SIMPLE_DEVICE_ORIENTATION = 5i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type SensorConnectionType = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_CONNECTION_TYPE_PC_INTEGRATED: SensorConnectionType = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_CONNECTION_TYPE_PC_ATTACHED: SensorConnectionType = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_CONNECTION_TYPE_PC_EXTERNAL: SensorConnectionType = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type SensorState = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_MIN: SensorState = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_READY: SensorState = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_NOT_AVAILABLE: SensorState = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_NO_DATA: SensorState = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_INITIALIZING: SensorState = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_ACCESS_DENIED: SensorState = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_ERROR: SensorState = 5i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SENSOR_STATE_MAX: SensorState = 5i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub type SimpleDeviceOrientation = i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SIMPLE_DEVICE_ORIENTATION_NOT_ROTATED: SimpleDeviceOrientation = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SIMPLE_DEVICE_ORIENTATION_ROTATED_90: SimpleDeviceOrientation = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SIMPLE_DEVICE_ORIENTATION_ROTATED_180: SimpleDeviceOrientation = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SIMPLE_DEVICE_ORIENTATION_ROTATED_270: SimpleDeviceOrientation = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SIMPLE_DEVICE_ORIENTATION_ROTATED_FACE_UP: SimpleDeviceOrientation = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub const SIMPLE_DEVICE_ORIENTATION_ROTATED_FACE_DOWN: SimpleDeviceOrientation = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub struct MATRIX3X3 {
    pub Anonymous: MATRIX3X3_0,
}
impl ::core::marker::Copy for MATRIX3X3 {}
impl ::core::clone::Clone for MATRIX3X3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub union MATRIX3X3_0 {
    pub Anonymous1: MATRIX3X3_0_0,
    pub Anonymous2: MATRIX3X3_0_1,
    pub M: [f32; 9],
}
impl ::core::marker::Copy for MATRIX3X3_0 {}
impl ::core::clone::Clone for MATRIX3X3_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub struct MATRIX3X3_0_0 {
    pub A11: f32,
    pub A12: f32,
    pub A13: f32,
    pub A21: f32,
    pub A22: f32,
    pub A23: f32,
    pub A31: f32,
    pub A32: f32,
    pub A33: f32,
}
impl ::core::marker::Copy for MATRIX3X3_0_0 {}
impl ::core::clone::Clone for MATRIX3X3_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub struct MATRIX3X3_0_1 {
    pub V1: VEC3D,
    pub V2: VEC3D,
    pub V3: VEC3D,
}
impl ::core::marker::Copy for MATRIX3X3_0_1 {}
impl ::core::clone::Clone for MATRIX3X3_0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub struct QUATERNION {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}
impl ::core::marker::Copy for QUATERNION {}
impl ::core::clone::Clone for QUATERNION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
pub struct SENSOR_COLLECTION_LIST {
    pub AllocatedSizeInBytes: u32,
    pub Count: u32,
    pub List: [SENSOR_VALUE_PAIR; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
impl ::core::marker::Copy for SENSOR_COLLECTION_LIST {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
impl ::core::clone::Clone for SENSOR_COLLECTION_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub struct SENSOR_PROPERTY_LIST {
    pub AllocatedSizeInBytes: u32,
    pub Count: u32,
    pub List: [super::super::UI::Shell::PropertiesSystem::PROPERTYKEY; 1],
}
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
impl ::core::marker::Copy for SENSOR_PROPERTY_LIST {}
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
impl ::core::clone::Clone for SENSOR_PROPERTY_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
pub struct SENSOR_VALUE_PAIR {
    pub Key: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY,
    pub Value: super::super::System::Com::StructuredStorage::PROPVARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
impl ::core::marker::Copy for SENSOR_VALUE_PAIR {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
impl ::core::clone::Clone for SENSOR_VALUE_PAIR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Sensors\"`*"]
pub struct VEC3D {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}
impl ::core::marker::Copy for VEC3D {}
impl ::core::clone::Clone for VEC3D {
    fn clone(&self) -> Self {
        *self
    }
}
