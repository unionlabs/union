#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmCalloutAdd0(enginehandle: super::super::Foundation::HANDLE, callout: *const FWPM_CALLOUT0, sd: super::super::Security::PSECURITY_DESCRIPTOR, id: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_CALLOUT_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutDeleteById0(enginehandle: super::super::Foundation::HANDLE, id: u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutDeleteByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_CALLOUT0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutGetById0(enginehandle: super::super::Foundation::HANDLE, id: u32, callout: *mut *mut FWPM_CALLOUT0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutGetByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, callout: *mut *mut FWPM_CALLOUT0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmCalloutGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmCalloutSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutSubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_CALLOUT_SUBSCRIPTION0, callback: FWPM_CALLOUT_CHANGE_CALLBACK0, context: *const ::core::ffi::c_void, changehandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut FWPM_CALLOUT_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmCalloutUnsubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, changehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmConnectionCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_CONNECTION_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmConnectionDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmConnectionEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_CONNECTION0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmConnectionGetById0(enginehandle: super::super::Foundation::HANDLE, id: u64, connection: *mut *mut FWPM_CONNECTION0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmConnectionGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmConnectionSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmConnectionSubscribe0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_CONNECTION_SUBSCRIPTION0, callback: FWPM_CONNECTION_CALLBACK0, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmConnectionUnsubscribe0(enginehandle: super::super::Foundation::HANDLE, eventshandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmDynamicKeywordSubscribe0(flags: u32, callback: FWPM_DYNAMIC_KEYWORD_CALLBACK0, context: *const ::core::ffi::c_void, subscriptionhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmDynamicKeywordUnsubscribe0(subscriptionhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmEngineClose0(enginehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmEngineGetOption0(enginehandle: super::super::Foundation::HANDLE, option: FWPM_ENGINE_OPTION, value: *mut *mut FWP_VALUE0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmEngineGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`, `\"Win32_System_Rpc\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Rpc"))]
    pub fn FwpmEngineOpen0(servername: ::windows_sys::core::PCWSTR, authnservice: u32, authidentity: *const super::super::System::Rpc::SEC_WINNT_AUTH_IDENTITY_W, session: *const FWPM_SESSION0, enginehandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmEngineSetOption0(enginehandle: super::super::Foundation::HANDLE, option: FWPM_ENGINE_OPTION, newvalue: *const FWP_VALUE0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmEngineSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterAdd0(enginehandle: super::super::Foundation::HANDLE, filter: *const FWPM_FILTER0, sd: super::super::Security::PSECURITY_DESCRIPTOR, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_FILTER_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmFilterDeleteById0(enginehandle: super::super::Foundation::HANDLE, id: u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmFilterDeleteByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmFilterDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_FILTER0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterGetById0(enginehandle: super::super::Foundation::HANDLE, id: u64, filter: *mut *mut FWPM_FILTER0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterGetByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, filter: *mut *mut FWPM_FILTER0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterSubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_FILTER_SUBSCRIPTION0, callback: FWPM_FILTER_CHANGE_CALLBACK0, context: *const ::core::ffi::c_void, changehandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmFilterSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut FWPM_FILTER_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmFilterUnsubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, changehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
    pub fn FwpmFreeMemory0(p: *mut *mut ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
    pub fn FwpmGetAppIdFromFileName0(filename: ::windows_sys::core::PCWSTR, appid: *mut *mut FWP_BYTE_BLOB) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmIPsecTunnelAdd0(enginehandle: super::super::Foundation::HANDLE, flags: u32, mainmodepolicy: *const FWPM_PROVIDER_CONTEXT0, tunnelpolicy: *const FWPM_PROVIDER_CONTEXT0, numfilterconditions: u32, filterconditions: *const FWPM_FILTER_CONDITION0, sd: super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmIPsecTunnelAdd1(enginehandle: super::super::Foundation::HANDLE, flags: u32, mainmodepolicy: *const FWPM_PROVIDER_CONTEXT1, tunnelpolicy: *const FWPM_PROVIDER_CONTEXT1, numfilterconditions: u32, filterconditions: *const FWPM_FILTER_CONDITION0, keymodkey: *const ::windows_sys::core::GUID, sd: super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmIPsecTunnelAdd2(enginehandle: super::super::Foundation::HANDLE, flags: u32, mainmodepolicy: *const FWPM_PROVIDER_CONTEXT2, tunnelpolicy: *const FWPM_PROVIDER_CONTEXT2, numfilterconditions: u32, filterconditions: *const FWPM_FILTER_CONDITION0, keymodkey: *const ::windows_sys::core::GUID, sd: super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmIPsecTunnelAdd3(enginehandle: super::super::Foundation::HANDLE, flags: u32, mainmodepolicy: *const FWPM_PROVIDER_CONTEXT3_, tunnelpolicy: *const FWPM_PROVIDER_CONTEXT3_, numfilterconditions: u32, filterconditions: *const FWPM_FILTER_CONDITION0, keymodkey: *const ::windows_sys::core::GUID, sd: super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmIPsecTunnelDeleteByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmLayerCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_LAYER_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmLayerDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmLayerEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_LAYER0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmLayerGetById0(enginehandle: super::super::Foundation::HANDLE, id: u16, layer: *mut *mut FWPM_LAYER0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmLayerGetByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, layer: *mut *mut FWPM_LAYER0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmLayerGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmLayerSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_NET_EVENT_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmNetEventDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_NET_EVENT0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventEnum1(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_NET_EVENT1, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventEnum2(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_NET_EVENT2, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventEnum3(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_NET_EVENT3, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventEnum4(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_NET_EVENT4_, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventEnum5(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_NET_EVENT5_, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventSubscribe0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_NET_EVENT_SUBSCRIPTION0, callback: FWPM_NET_EVENT_CALLBACK0, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventSubscribe1(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_NET_EVENT_SUBSCRIPTION0, callback: FWPM_NET_EVENT_CALLBACK1, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventSubscribe2(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_NET_EVENT_SUBSCRIPTION0, callback: FWPM_NET_EVENT_CALLBACK2, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventSubscribe3(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_NET_EVENT_SUBSCRIPTION0, callback: FWPM_NET_EVENT_CALLBACK3, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventSubscribe4(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_NET_EVENT_SUBSCRIPTION0, callback: FWPM_NET_EVENT_CALLBACK4, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut FWPM_NET_EVENT_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmNetEventUnsubscribe0(enginehandle: super::super::Foundation::HANDLE, eventshandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventsGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmNetEventsSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderAdd0(enginehandle: super::super::Foundation::HANDLE, provider: *const FWPM_PROVIDER0, sd: super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextAdd0(enginehandle: super::super::Foundation::HANDLE, providercontext: *const FWPM_PROVIDER_CONTEXT0, sd: super::super::Security::PSECURITY_DESCRIPTOR, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextAdd1(enginehandle: super::super::Foundation::HANDLE, providercontext: *const FWPM_PROVIDER_CONTEXT1, sd: super::super::Security::PSECURITY_DESCRIPTOR, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextAdd2(enginehandle: super::super::Foundation::HANDLE, providercontext: *const FWPM_PROVIDER_CONTEXT2, sd: super::super::Security::PSECURITY_DESCRIPTOR, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextAdd3(enginehandle: super::super::Foundation::HANDLE, providercontext: *const FWPM_PROVIDER_CONTEXT3_, sd: super::super::Security::PSECURITY_DESCRIPTOR, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_PROVIDER_CONTEXT_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextDeleteById0(enginehandle: super::super::Foundation::HANDLE, id: u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextDeleteByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_PROVIDER_CONTEXT0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextEnum1(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_PROVIDER_CONTEXT1, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextEnum2(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_PROVIDER_CONTEXT2, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextEnum3(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_PROVIDER_CONTEXT3_, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetById0(enginehandle: super::super::Foundation::HANDLE, id: u64, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetById1(enginehandle: super::super::Foundation::HANDLE, id: u64, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetById2(enginehandle: super::super::Foundation::HANDLE, id: u64, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT2) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetById3(enginehandle: super::super::Foundation::HANDLE, id: u64, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT3_) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetByKey1(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetByKey2(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT2) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetByKey3(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, providercontext: *mut *mut FWPM_PROVIDER_CONTEXT3_) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderContextSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextSubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_PROVIDER_CONTEXT_SUBSCRIPTION0, callback: FWPM_PROVIDER_CONTEXT_CHANGE_CALLBACK0, context: *const ::core::ffi::c_void, changehandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut FWPM_PROVIDER_CONTEXT_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderContextUnsubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, changehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_PROVIDER_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderDeleteByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_PROVIDER0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderGetByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, provider: *mut *mut FWPM_PROVIDER0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmProviderSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderSubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_PROVIDER_SUBSCRIPTION0, callback: FWPM_PROVIDER_CHANGE_CALLBACK0, context: *const ::core::ffi::c_void, changehandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut FWPM_PROVIDER_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmProviderUnsubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, changehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSessionCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_SESSION_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSessionDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmSessionEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_SESSION0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmSubLayerAdd0(enginehandle: super::super::Foundation::HANDLE, sublayer: *const FWPM_SUBLAYER0, sd: super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const FWPM_SUBLAYER_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerDeleteByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut FWPM_SUBLAYER0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerGetByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, sublayer: *mut *mut FWPM_SUBLAYER0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmSubLayerGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmSubLayerSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, key: *const ::windows_sys::core::GUID, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerSubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_SUBLAYER_SUBSCRIPTION0, callback: FWPM_SUBLAYER_CHANGE_CALLBACK0, context: *const ::core::ffi::c_void, changehandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut FWPM_SUBLAYER_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSubLayerUnsubscribeChanges0(enginehandle: super::super::Foundation::HANDLE, changehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSystemPortsGet0(enginehandle: super::super::Foundation::HANDLE, sysports: *mut *mut FWPM_SYSTEM_PORTS0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSystemPortsSubscribe0(enginehandle: super::super::Foundation::HANDLE, reserved: *mut ::core::ffi::c_void, callback: FWPM_SYSTEM_PORTS_CALLBACK0, context: *const ::core::ffi::c_void, sysportshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmSystemPortsUnsubscribe0(enginehandle: super::super::Foundation::HANDLE, sysportshandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmTransactionAbort0(enginehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmTransactionBegin0(enginehandle: super::super::Foundation::HANDLE, flags: u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmTransactionCommit0(enginehandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmvSwitchEventSubscribe0(enginehandle: super::super::Foundation::HANDLE, subscription: *const FWPM_VSWITCH_EVENT_SUBSCRIPTION0, callback: FWPM_VSWITCH_EVENT_CALLBACK0, context: *const ::core::ffi::c_void, subscriptionhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FwpmvSwitchEventUnsubscribe0(enginehandle: super::super::Foundation::HANDLE, subscriptionhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmvSwitchEventsGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn FwpmvSwitchEventsSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecDospGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecDospGetStatistics0(enginehandle: super::super::Foundation::HANDLE, idpstatistics: *mut IPSEC_DOSP_STATISTICS0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecDospSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecDospStateCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const IPSEC_DOSP_STATE_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecDospStateDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecDospStateEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IPSEC_DOSP_STATE0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecGetStatistics0(enginehandle: super::super::Foundation::HANDLE, ipsecstatistics: *mut IPSEC_STATISTICS0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecGetStatistics1(enginehandle: super::super::Foundation::HANDLE, ipsecstatistics: *mut IPSEC_STATISTICS1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecKeyManagerAddAndRegister0(enginehandle: super::super::Foundation::HANDLE, keymanager: *const IPSEC_KEY_MANAGER0, keymanagercallbacks: *const IPSEC_KEY_MANAGER_CALLBACKS0, keymgmthandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecKeyManagerGetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, reserved: *const ::core::ffi::c_void, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecKeyManagerSetSecurityInfoByKey0(enginehandle: super::super::Foundation::HANDLE, reserved: *const ::core::ffi::c_void, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecKeyManagerUnregisterAndDelete0(enginehandle: super::super::Foundation::HANDLE, keymgmthandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecKeyManagersGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut IPSEC_KEY_MANAGER0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextAddInbound0(enginehandle: super::super::Foundation::HANDLE, id: u64, inboundbundle: *const IPSEC_SA_BUNDLE0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextAddInbound1(enginehandle: super::super::Foundation::HANDLE, id: u64, inboundbundle: *const IPSEC_SA_BUNDLE1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextAddOutbound0(enginehandle: super::super::Foundation::HANDLE, id: u64, outboundbundle: *const IPSEC_SA_BUNDLE0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextAddOutbound1(enginehandle: super::super::Foundation::HANDLE, id: u64, outboundbundle: *const IPSEC_SA_BUNDLE1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextCreate0(enginehandle: super::super::Foundation::HANDLE, outboundtraffic: *const IPSEC_TRAFFIC0, inboundfilterid: *mut u64, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextCreate1(enginehandle: super::super::Foundation::HANDLE, outboundtraffic: *const IPSEC_TRAFFIC1, virtualiftunnelinfo: *const IPSEC_VIRTUAL_IF_TUNNEL_INFO0, inboundfilterid: *mut u64, id: *mut u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const IPSEC_SA_CONTEXT_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextDeleteById0(enginehandle: super::super::Foundation::HANDLE, id: u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IPSEC_SA_CONTEXT0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextEnum1(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IPSEC_SA_CONTEXT1, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextExpire0(enginehandle: super::super::Foundation::HANDLE, id: u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextGetById0(enginehandle: super::super::Foundation::HANDLE, id: u64, sacontext: *mut *mut IPSEC_SA_CONTEXT0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextGetById1(enginehandle: super::super::Foundation::HANDLE, id: u64, sacontext: *mut *mut IPSEC_SA_CONTEXT1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextGetSpi0(enginehandle: super::super::Foundation::HANDLE, id: u64, getspi: *const IPSEC_GETSPI0, inboundspi: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextGetSpi1(enginehandle: super::super::Foundation::HANDLE, id: u64, getspi: *const IPSEC_GETSPI1, inboundspi: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextSetSpi0(enginehandle: super::super::Foundation::HANDLE, id: u64, getspi: *const IPSEC_GETSPI1, inboundspi: u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextSubscribe0(enginehandle: super::super::Foundation::HANDLE, subscription: *const IPSEC_SA_CONTEXT_SUBSCRIPTION0, callback: IPSEC_SA_CONTEXT_CALLBACK0, context: *const ::core::ffi::c_void, eventshandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextSubscriptionsGet0(enginehandle: super::super::Foundation::HANDLE, entries: *mut *mut *mut IPSEC_SA_CONTEXT_SUBSCRIPTION0, numentries: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaContextUnsubscribe0(enginehandle: super::super::Foundation::HANDLE, eventshandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaContextUpdate0(enginehandle: super::super::Foundation::HANDLE, flags: u64, newvalues: *const IPSEC_SA_CONTEXT1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const IPSEC_SA_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaDbGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaDbSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IPsecSaDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IPSEC_SA_DETAILS0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IPsecSaEnum1(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IPSEC_SA_DETAILS1, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextGetStatistics0(enginehandle: super::super::Foundation::HANDLE, ikeextstatistics: *mut IKEEXT_STATISTICS0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextGetStatistics1(enginehandle: super::super::Foundation::HANDLE, ikeextstatistics: *mut IKEEXT_STATISTICS1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IkeextSaCreateEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumtemplate: *const IKEEXT_SA_ENUM_TEMPLATE0, enumhandle: *mut super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IkeextSaDbGetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *mut super::super::Foundation::PSID, sidgroup: *mut super::super::Foundation::PSID, dacl: *mut *mut super::super::Security::ACL, sacl: *mut *mut super::super::Security::ACL, securitydescriptor: *mut super::super::Security::PSECURITY_DESCRIPTOR) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn IkeextSaDbSetSecurityInfo0(enginehandle: super::super::Foundation::HANDLE, securityinfo: u32, sidowner: *const super::super::Security::SID, sidgroup: *const super::super::Security::SID, dacl: *const super::super::Security::ACL, sacl: *const super::super::Security::ACL) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaDeleteById0(enginehandle: super::super::Foundation::HANDLE, id: u64) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaDestroyEnumHandle0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaEnum0(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IKEEXT_SA_DETAILS0, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaEnum1(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IKEEXT_SA_DETAILS1, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaEnum2(enginehandle: super::super::Foundation::HANDLE, enumhandle: super::super::Foundation::HANDLE, numentriesrequested: u32, entries: *mut *mut *mut IKEEXT_SA_DETAILS2, numentriesreturned: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaGetById0(enginehandle: super::super::Foundation::HANDLE, id: u64, sa: *mut *mut IKEEXT_SA_DETAILS0) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaGetById1(enginehandle: super::super::Foundation::HANDLE, id: u64, salookupcontext: *const ::windows_sys::core::GUID, sa: *mut *mut IKEEXT_SA_DETAILS1) -> u32;
    #[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IkeextSaGetById2(enginehandle: super::super::Foundation::HANDLE, id: u64, salookupcontext: *const ::windows_sys::core::GUID, sa: *mut *mut IKEEXT_SA_DETAILS2) -> u32;
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type ARP_HARDWARE_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ARP_HW_ENET: ARP_HARDWARE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ARP_HW_802: ARP_HARDWARE_TYPE = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ARP_HEADER {
    pub HardwareAddressSpace: u16,
    pub ProtocolAddressSpace: u16,
    pub HardwareAddressLength: u8,
    pub ProtocolAddressLength: u8,
    pub Opcode: u16,
    pub SenderHardwareAddress: [u8; 1],
}
impl ::core::marker::Copy for ARP_HEADER {}
impl ::core::clone::Clone for ARP_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type ARP_OPCODE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ARP_REQUEST: ARP_OPCODE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ARP_RESPONSE: ARP_OPCODE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const BYTE_ORDER: u32 = 1234u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const DL_ADDRESS_LENGTH_MAXIMUM: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type DL_ADDRESS_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const DlUnicast: DL_ADDRESS_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const DlMulticast: DL_ADDRESS_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const DlBroadcast: DL_ADDRESS_TYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union DL_EI48 {
    pub Byte: [u8; 3],
}
impl ::core::marker::Copy for DL_EI48 {}
impl ::core::clone::Clone for DL_EI48 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union DL_EI64 {
    pub Byte: [u8; 5],
}
impl ::core::marker::Copy for DL_EI64 {}
impl ::core::clone::Clone for DL_EI64 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union DL_EUI48 {
    pub Byte: [u8; 6],
    pub Anonymous: DL_EUI48_0,
}
impl ::core::marker::Copy for DL_EUI48 {}
impl ::core::clone::Clone for DL_EUI48 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct DL_EUI48_0 {
    pub Oui: DL_OUI,
    pub Ei48: DL_EI48,
}
impl ::core::marker::Copy for DL_EUI48_0 {}
impl ::core::clone::Clone for DL_EUI48_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union DL_EUI64 {
    pub Byte: [u8; 8],
    pub Value: u64,
    pub Anonymous: DL_EUI64_0,
}
impl ::core::marker::Copy for DL_EUI64 {}
impl ::core::clone::Clone for DL_EUI64 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct DL_EUI64_0 {
    pub Oui: DL_OUI,
    pub Anonymous: DL_EUI64_0_0,
}
impl ::core::marker::Copy for DL_EUI64_0 {}
impl ::core::clone::Clone for DL_EUI64_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union DL_EUI64_0_0 {
    pub Ei64: DL_EI64,
    pub Anonymous: DL_EUI64_0_0_0,
}
impl ::core::marker::Copy for DL_EUI64_0_0 {}
impl ::core::clone::Clone for DL_EUI64_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct DL_EUI64_0_0_0 {
    pub Type: u8,
    pub Tse: u8,
    pub Ei48: DL_EI48,
}
impl ::core::marker::Copy for DL_EUI64_0_0_0 {}
impl ::core::clone::Clone for DL_EUI64_0_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const DL_HEADER_LENGTH_MAXIMUM: u32 = 64u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union DL_OUI {
    pub Byte: [u8; 3],
    pub Anonymous: DL_OUI_0,
}
impl ::core::marker::Copy for DL_OUI {}
impl ::core::clone::Clone for DL_OUI {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct DL_OUI_0 {
    pub _bitfield: u8,
}
impl ::core::marker::Copy for DL_OUI_0 {}
impl ::core::clone::Clone for DL_OUI_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct DL_TEREDO_ADDRESS {
    pub Reserved: [u8; 6],
    pub Anonymous: DL_TEREDO_ADDRESS_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for DL_TEREDO_ADDRESS {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for DL_TEREDO_ADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union DL_TEREDO_ADDRESS_0 {
    pub Eui64: DL_EUI64,
    pub Anonymous: DL_TEREDO_ADDRESS_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for DL_TEREDO_ADDRESS_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for DL_TEREDO_ADDRESS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct DL_TEREDO_ADDRESS_0_0 {
    pub Flags: u16,
    pub MappedPort: u16,
    pub MappedAddress: super::super::Networking::WinSock::IN_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for DL_TEREDO_ADDRESS_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for DL_TEREDO_ADDRESS_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct DL_TEREDO_ADDRESS_PRV {
    pub Reserved: [u8; 6],
    pub Anonymous: DL_TEREDO_ADDRESS_PRV_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for DL_TEREDO_ADDRESS_PRV {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for DL_TEREDO_ADDRESS_PRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union DL_TEREDO_ADDRESS_PRV_0 {
    pub Eui64: DL_EUI64,
    pub Anonymous: DL_TEREDO_ADDRESS_PRV_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for DL_TEREDO_ADDRESS_PRV_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for DL_TEREDO_ADDRESS_PRV_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct DL_TEREDO_ADDRESS_PRV_0_0 {
    pub Flags: u16,
    pub MappedPort: u16,
    pub MappedAddress: super::super::Networking::WinSock::IN_ADDR,
    pub LocalAddress: super::super::Networking::WinSock::IN_ADDR,
    pub InterfaceIndex: u32,
    pub LocalPort: u16,
    pub DlDestination: DL_EUI48,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for DL_TEREDO_ADDRESS_PRV_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for DL_TEREDO_ADDRESS_PRV_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`, `\"Win32_System_Kernel\"`*"]
#[cfg(all(feature = "Win32_Networking_WinSock", feature = "Win32_System_Kernel"))]
pub struct DL_TUNNEL_ADDRESS {
    pub CompartmentId: super::super::System::Kernel::COMPARTMENT_ID,
    pub ScopeId: super::super::Networking::WinSock::SCOPE_ID,
    pub IpAddress: [u8; 1],
}
#[cfg(all(feature = "Win32_Networking_WinSock", feature = "Win32_System_Kernel"))]
impl ::core::marker::Copy for DL_TUNNEL_ADDRESS {}
#[cfg(all(feature = "Win32_Networking_WinSock", feature = "Win32_System_Kernel"))]
impl ::core::clone::Clone for DL_TUNNEL_ADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ETHERNET_HEADER {
    pub Destination: DL_EUI48,
    pub Source: DL_EUI48,
    pub Anonymous: ETHERNET_HEADER_0,
}
impl ::core::marker::Copy for ETHERNET_HEADER {}
impl ::core::clone::Clone for ETHERNET_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union ETHERNET_HEADER_0 {
    pub Type: u16,
    pub Length: u16,
}
impl ::core::marker::Copy for ETHERNET_HEADER_0 {}
impl ::core::clone::Clone for ETHERNET_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETHERNET_TYPE_802_1AD: u32 = 34984u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETHERNET_TYPE_802_1Q: u32 = 33024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETHERNET_TYPE_ARP: u32 = 2054u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETHERNET_TYPE_IPV4: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETHERNET_TYPE_IPV6: u32 = 34525u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETHERNET_TYPE_MINIMUM: u32 = 1536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETH_LENGTH_OF_HEADER: u32 = 14u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETH_LENGTH_OF_SNAP_HEADER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ETH_LENGTH_OF_VLAN_HEADER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const EXT_LEN_UNIT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FALLBACK_INDEX = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FallbackIndexTcpFastopen: FALLBACK_INDEX = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FallbackIndexMax: FALLBACK_INDEX = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_ACTION0 {
    pub r#type: u32,
    pub Anonymous: FWPM_ACTION0_0,
}
impl ::core::marker::Copy for FWPM_ACTION0 {}
impl ::core::clone::Clone for FWPM_ACTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union FWPM_ACTION0_0 {
    pub filterType: ::windows_sys::core::GUID,
    pub calloutKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_ACTION0_0 {}
impl ::core::clone::Clone for FWPM_ACTION0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_ADD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_ADD_LINK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_BEGIN_READ_TXN: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_BEGIN_WRITE_TXN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_CLASSIFY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_ENUM: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_OPEN: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_READ: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_READ_STATS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_SUBSCRIBE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ACTRL_WRITE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_APPC_NETWORK_CAPABILITY_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_APPC_NETWORK_CAPABILITY_INTERNET_CLIENT: FWPM_APPC_NETWORK_CAPABILITY_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_APPC_NETWORK_CAPABILITY_INTERNET_CLIENT_SERVER: FWPM_APPC_NETWORK_CAPABILITY_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_APPC_NETWORK_CAPABILITY_INTERNET_PRIVATE_NETWORK: FWPM_APPC_NETWORK_CAPABILITY_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_AUTO_WEIGHT_BITS: u32 = 60u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_CALLOUT0 {
    pub calloutKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub applicableLayer: ::windows_sys::core::GUID,
    pub calloutId: u32,
}
impl ::core::marker::Copy for FWPM_CALLOUT0 {}
impl ::core::clone::Clone for FWPM_CALLOUT0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_CALLOUT_BUILT_IN_RESERVED_1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2006391204, data2: 59029, data3: 18358, data4: [161, 153, 121, 153, 254, 201, 22, 59] };
pub const FWPM_CALLOUT_BUILT_IN_RESERVED_2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4019610038, data2: 31838, data3: 18685, data4: [161, 48, 150, 103, 140, 234, 204, 65] };
pub const FWPM_CALLOUT_BUILT_IN_RESERVED_3: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 410164346, data2: 12130, data3: 19424, data4: [150, 111, 151, 75, 33, 184, 109, 241] };
pub const FWPM_CALLOUT_BUILT_IN_RESERVED_4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1816115201, data2: 56063, data3: 16617, data4: [145, 230, 247, 255, 126, 82, 247, 217] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_CALLOUT_CHANGE0 {
    pub changeType: FWPM_CHANGE_TYPE,
    pub calloutKey: ::windows_sys::core::GUID,
    pub calloutId: u32,
}
impl ::core::marker::Copy for FWPM_CALLOUT_CHANGE0 {}
impl ::core::clone::Clone for FWPM_CALLOUT_CHANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_CALLOUT_CHANGE_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, change: *const FWPM_CALLOUT_CHANGE0)>;
pub const FWPM_CALLOUT_EDGE_TRAVERSAL_ALE_LISTEN_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 860383925, data2: 27998, data3: 20069, data4: [160, 11, 167, 175, 237, 11, 169, 161] };
pub const FWPM_CALLOUT_EDGE_TRAVERSAL_ALE_RESOURCE_ASSIGNMENT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 127602704, data2: 61893, data3: 20429, data4: [174, 5, 218, 65, 16, 122, 189, 11] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_CALLOUT_ENUM_TEMPLATE0 {
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub layerKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_CALLOUT_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_CALLOUT_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CALLOUT_FLAG_PERSISTENT: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CALLOUT_FLAG_REGISTERED: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CALLOUT_FLAG_USES_PROVIDER_CONTEXT: u32 = 131072u32;
pub const FWPM_CALLOUT_HTTP_TEMPLATE_SSL_HANDSHAKE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3007459913, data2: 36105, data3: 18520, data4: [146, 16, 149, 199, 253, 168, 227, 15] };
pub const FWPM_CALLOUT_IPSEC_ALE_CONNECT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1791050236, data2: 63325, data3: 16899, data4: [185, 200, 72, 230, 20, 156, 39, 18] };
pub const FWPM_CALLOUT_IPSEC_ALE_CONNECT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1275976197, data2: 58143, data3: 18022, data4: [144, 176, 179, 223, 173, 52, 18, 154] };
pub const FWPM_CALLOUT_IPSEC_DOSP_FORWARD_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 801855212, data2: 52535, data3: 19279, data4: [177, 8, 98, 194, 177, 133, 10, 12] };
pub const FWPM_CALLOUT_IPSEC_DOSP_FORWARD_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1829282626, data2: 56222, data3: 20414, data4: [158, 210, 87, 55, 76, 232, 159, 121] };
pub const FWPM_CALLOUT_IPSEC_FORWARD_INBOUND_TUNNEL_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 679646771, data2: 50416, data3: 20070, data4: [135, 63, 132, 77, 178, 168, 153, 199] };
pub const FWPM_CALLOUT_IPSEC_FORWARD_INBOUND_TUNNEL_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2941304514, data2: 50822, data3: 17050, data4: [136, 77, 183, 68, 67, 231, 176, 180] };
pub const FWPM_CALLOUT_IPSEC_FORWARD_OUTBOUND_TUNNEL_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4216529206, data2: 5579, data3: 17419, data4: [147, 124, 23, 23, 202, 50, 12, 64] };
pub const FWPM_CALLOUT_IPSEC_FORWARD_OUTBOUND_TUNNEL_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3672522956, data2: 57377, data3: 19438, data4: [158, 182, 164, 139, 39, 92, 140, 29] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_INITIATE_SECURE_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2113876123, data2: 47741, data3: 19130, data4: [145, 170, 174, 92, 102, 64, 201, 68] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_INITIATE_SECURE_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2845890265, data2: 50572, data3: 18254, data4: [138, 235, 60, 254, 153, 214, 213, 61] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_TRANSPORT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1362268173, data2: 24196, data3: 19295, data4: [128, 228, 1, 116, 30, 129, 255, 16] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_TRANSPORT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1238609042, data2: 10860, data3: 19919, data4: [149, 95, 28, 59, 224, 9, 221, 153] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_TUNNEL_ALE_ACCEPT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1039591390, data2: 64800, data3: 18674, data4: [159, 38, 248, 84, 68, 76, 186, 121] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_TUNNEL_ALE_ACCEPT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2716046035, data2: 29356, data3: 18363, data4: [135, 167, 1, 34, 198, 148, 52, 171] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_TUNNEL_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 421169734, data2: 3064, data3: 18127, data4: [176, 69, 75, 69, 223, 166, 163, 36] };
pub const FWPM_CALLOUT_IPSEC_INBOUND_TUNNEL_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2160280291, data2: 7763, data3: 19823, data4: [155, 68, 3, 223, 90, 238, 225, 84] };
pub const FWPM_CALLOUT_IPSEC_OUTBOUND_TRANSPORT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1262927626, data2: 17699, data3: 20055, data4: [170, 56, 168, 121, 135, 201, 16, 217] };
pub const FWPM_CALLOUT_IPSEC_OUTBOUND_TRANSPORT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 953710370, data2: 44419, data3: 20241, data4: [169, 31, 223, 15, 176, 119, 34, 91] };
pub const FWPM_CALLOUT_IPSEC_OUTBOUND_TUNNEL_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1889802604, data2: 33627, data3: 20400, data4: [152, 232, 7, 95, 77, 151, 125, 70] };
pub const FWPM_CALLOUT_IPSEC_OUTBOUND_TUNNEL_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4051915619, data2: 42661, data3: 20066, data4: [177, 128, 35, 219, 120, 157, 141, 166] };
pub const FWPM_CALLOUT_POLICY_SILENT_MODE_AUTH_CONNECT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1606402845, data2: 42268, data3: 17628, data4: [172, 182, 6, 36, 160, 48, 167, 0] };
pub const FWPM_CALLOUT_POLICY_SILENT_MODE_AUTH_CONNECT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1606402845, data2: 42268, data3: 17628, data4: [172, 182, 6, 36, 160, 48, 167, 1] };
pub const FWPM_CALLOUT_POLICY_SILENT_MODE_AUTH_RECV_ACCEPT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1606402845, data2: 42268, data3: 17628, data4: [172, 182, 6, 36, 160, 48, 167, 2] };
pub const FWPM_CALLOUT_POLICY_SILENT_MODE_AUTH_RECV_ACCEPT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1606402845, data2: 42268, data3: 17628, data4: [172, 182, 6, 36, 160, 48, 167, 3] };
pub const FWPM_CALLOUT_RESERVED_AUTH_CONNECT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 680219213, data2: 1382, data3: 19993, data4: [182, 18, 143, 68, 26, 46, 89, 73] };
pub const FWPM_CALLOUT_RESERVED_AUTH_CONNECT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 12077970, data2: 11102, data3: 19313, data4: [171, 14, 170, 202, 67, 227, 135, 230] };
pub const FWPM_CALLOUT_SET_OPTIONS_AUTH_CONNECT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3159892608, data2: 5751, data3: 16873, data4: [148, 171, 194, 252, 177, 92, 46, 235] };
pub const FWPM_CALLOUT_SET_OPTIONS_AUTH_CONNECT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2565158716, data2: 47236, data3: 18703, data4: [182, 95, 47, 106, 74, 87, 81, 149] };
pub const FWPM_CALLOUT_SET_OPTIONS_AUTH_RECV_ACCEPT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 760606728, data2: 3073, data3: 20370, data4: [178, 110, 160, 138, 148, 86, 155, 141] };
pub const FWPM_CALLOUT_SET_OPTIONS_AUTH_RECV_ACCEPT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1661044023, data2: 62081, data3: 19908, data4: [131, 211, 141, 236, 24, 183, 173, 226] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_CALLOUT_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_CALLOUT_ENUM_TEMPLATE0,
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_CALLOUT_SUBSCRIPTION0 {}
impl ::core::clone::Clone for FWPM_CALLOUT_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_CALLOUT_TCP_CHIMNEY_ACCEPT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3783519410, data2: 14975, data3: 19284, data4: [138, 217, 118, 5, 14, 216, 128, 202] };
pub const FWPM_CALLOUT_TCP_CHIMNEY_ACCEPT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 58249025, data2: 49048, data3: 17923, data4: [129, 242, 127, 18, 88, 96, 121, 246] };
pub const FWPM_CALLOUT_TCP_CHIMNEY_CONNECT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4091611827, data2: 11301, data3: 17017, data4: [172, 54, 195, 15, 193, 129, 190, 196] };
pub const FWPM_CALLOUT_TCP_CHIMNEY_CONNECT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 971120773, data2: 41793, data3: 17148, data4: [162, 121, 174, 201, 78, 104, 156, 86] };
pub const FWPM_CALLOUT_TCP_TEMPLATES_ACCEPT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 790885840, data2: 16580, data3: 19521, data4: [162, 84, 70, 216, 219, 168, 149, 124] };
pub const FWPM_CALLOUT_TCP_TEMPLATES_ACCEPT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2991674096, data2: 39196, data3: 20307, data4: [187, 231, 210, 75, 69, 254, 99, 44] };
pub const FWPM_CALLOUT_TCP_TEMPLATES_CONNECT_LAYER_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 559549241, data2: 19326, data3: 20186, data4: [140, 228, 23, 150, 121, 223, 98, 36] };
pub const FWPM_CALLOUT_TCP_TEMPLATES_CONNECT_LAYER_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2206939041, data2: 23570, data3: 19764, data4: [139, 56, 7, 135, 40, 178, 210, 92] };
pub const FWPM_CALLOUT_TEREDO_ALE_LISTEN_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2175022311, data2: 62988, data3: 17272, data4: [186, 184, 198, 37, 163, 15, 1, 151] };
pub const FWPM_CALLOUT_TEREDO_ALE_RESOURCE_ASSIGNMENT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 834229138, data2: 1646, data3: 17058, data4: [183, 219, 146, 248, 172, 221, 86, 249] };
pub const FWPM_CALLOUT_WFP_TRANSPORT_LAYER_V4_SILENT_DROP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3986720262, data2: 9364, data3: 19832, data4: [137, 188, 103, 131, 124, 3, 185, 105] };
pub const FWPM_CALLOUT_WFP_TRANSPORT_LAYER_V6_SILENT_DROP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2257833076, data2: 41077, data3: 16726, data4: [180, 118, 146, 134, 238, 206, 129, 78] };
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_CHANGE_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CHANGE_ADD: FWPM_CHANGE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CHANGE_DELETE: FWPM_CHANGE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CHANGE_TYPE_MAX: FWPM_CHANGE_TYPE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_CLASSIFY_OPTION0 {
    pub r#type: FWP_CLASSIFY_OPTION_TYPE,
    pub value: FWP_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_CLASSIFY_OPTION0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_CLASSIFY_OPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_CLASSIFY_OPTIONS0 {
    pub numOptions: u32,
    pub options: *mut FWPM_CLASSIFY_OPTION0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_CLASSIFY_OPTIONS0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_CLASSIFY_OPTIONS0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_CONDITION_ALE_APP_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3616415367, data2: 34372, data3: 20133, data4: [148, 55, 216, 9, 236, 239, 201, 113] };
pub const FWPM_CONDITION_ALE_EFFECTIVE_NAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2972154778, data2: 46977, data3: 16636, data4: [150, 113, 229, 241, 185, 137, 243, 78] };
pub const FWPM_CONDITION_ALE_NAP_CONTEXT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1176984221, data2: 49215, data3: 19831, data4: [183, 132, 28, 87, 244, 208, 39, 83] };
pub const FWPM_CONDITION_ALE_ORIGINAL_APP_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 242012294, data2: 57851, data3: 16914, data4: [132, 47, 138, 159, 153, 63, 179, 246] };
pub const FWPM_CONDITION_ALE_PACKAGE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1908177146, data2: 61820, data3: 18839, data4: [166, 2, 106, 187, 38, 31, 53, 28] };
pub const FWPM_CONDITION_ALE_PROMISCUOUS_MODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 479676278, data2: 29058, data3: 18153, data4: [175, 211, 176, 41, 16, 227, 3, 52] };
pub const FWPM_CONDITION_ALE_REAUTH_REASON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3028472359, data2: 6521, data3: 19096, data4: [128, 68, 24, 187, 230, 35, 117, 66] };
pub const FWPM_CONDITION_ALE_REMOTE_MACHINE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 446988113, data2: 32659, data3: 17672, data4: [162, 113, 129, 171, 176, 12, 156, 171] };
pub const FWPM_CONDITION_ALE_REMOTE_USER_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4130370487, data2: 393, data3: 19120, data4: [149, 164, 97, 35, 203, 250, 184, 98] };
pub const FWPM_CONDITION_ALE_SECURITY_ATTRIBUTE_FQBN_VALUE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 933590681, data2: 22659, data3: 18787, data4: [146, 184, 62, 112, 70, 136, 176, 173] };
pub const FWPM_CONDITION_ALE_SIO_FIREWALL_SYSTEM_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3119833224, data2: 52120, data3: 20219, data4: [162, 199, 173, 7, 51, 38, 67, 219] };
pub const FWPM_CONDITION_ALE_USER_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2936289802, data2: 45901, data3: 20358, data4: [151, 156, 201, 3, 113, 175, 110, 102] };
pub const FWPM_CONDITION_ARRIVAL_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3423112627, data2: 6034, data3: 19057, data4: [176, 249, 3, 125, 33, 205, 130, 139] };
pub const FWPM_CONDITION_ARRIVAL_INTERFACE_PROFILE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3456002731, data2: 49283, data3: 16706, data4: [134, 121, 192, 143, 149, 50, 156, 97] };
pub const FWPM_CONDITION_ARRIVAL_INTERFACE_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2314834142, data2: 59288, data3: 20077, data4: [171, 118, 124, 149, 88, 41, 46, 111] };
pub const FWPM_CONDITION_ARRIVAL_TUNNEL_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1360094940, data2: 31372, data3: 19111, data4: [181, 51, 149, 171, 89, 251, 3, 64] };
pub const FWPM_CONDITION_AUTHENTICATION_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3947203797, data2: 55931, data3: 20217, data4: [141, 67, 123, 10, 132, 3, 50, 242] };
pub const FWPM_CONDITION_CLIENT_CERT_KEY_LENGTH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2750152903, data2: 1524, data3: 19959, data4: [145, 242, 95, 96, 217, 31, 244, 67] };
pub const FWPM_CONDITION_CLIENT_CERT_OID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3297881438, data2: 63618, data3: 17027, data4: [185, 22, 67, 107, 16, 63, 244, 173] };
pub const FWPM_CONDITION_CLIENT_TOKEN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3257465886, data2: 16442, data3: 17528, data4: [190, 5, 201, 186, 164, 192, 90, 206] };
pub const FWPM_CONDITION_COMPARTMENT_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 900174251, data2: 1196, data3: 20466, data4: [166, 187, 218, 108, 250, 199, 24, 6] };
pub const FWPM_CONDITION_CURRENT_PROFILE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2872062921, data2: 49379, data3: 18265, data4: [147, 125, 87, 88, 198, 93, 74, 227] };
pub const FWPM_CONDITION_DCOM_APP_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4281236301, data2: 12562, data3: 18288, data4: [182, 54, 77, 36, 174, 58, 106, 242] };
pub const FWPM_CONDITION_DESTINATION_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 902784290, data2: 16697, data3: 17902, data4: [160, 213, 103, 184, 9, 73, 216, 121] };
pub const FWPM_CONDITION_DESTINATION_SUB_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 729629593, data2: 54471, data3: 18232, data4: [162, 245, 233, 148, 180, 61, 163, 136] };
pub const FWPM_CONDITION_DIRECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2273624390, data2: 51863, data3: 17622, data4: [159, 209, 25, 251, 24, 64, 203, 247] };
pub const FWPM_CONDITION_EMBEDDED_LOCAL_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1181918312, data2: 35338, data3: 16898, data4: [171, 180, 132, 158, 146, 230, 104, 9] };
pub const FWPM_CONDITION_EMBEDDED_LOCAL_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3217701197, data2: 44251, data3: 18510, data4: [184, 230, 42, 255, 121, 117, 115, 69] };
pub const FWPM_CONDITION_EMBEDDED_PROTOCOL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 125321479, data2: 41630, data3: 19579, data4: [158, 199, 41, 196, 74, 250, 253, 188] };
pub const FWPM_CONDITION_EMBEDDED_REMOTE_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2012105529, data2: 12915, data3: 18033, data4: [182, 59, 171, 111, 235, 102, 238, 182] };
pub const FWPM_CONDITION_EMBEDDED_REMOTE_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3403994785, data2: 10600, data3: 16621, data4: [164, 206, 84, 113, 96, 221, 168, 141] };
pub const FWPM_CONDITION_ETHER_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4245197965, data2: 41497, data3: 19794, data4: [187, 152, 26, 85, 64, 238, 123, 78] };
pub const FWPM_CONDITION_FLAGS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1663885883, data2: 20839, data3: 17244, data4: [134, 215, 233, 3, 104, 74, 168, 12] };
pub const FWPM_CONDITION_IMAGE_NAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3492077133, data2: 57002, data3: 17175, data4: [156, 133, 228, 14, 246, 225, 64, 195] };
pub const FWPM_CONDITION_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1719654229, data2: 54933, data3: 17226, data4: [138, 245, 211, 131, 90, 18, 89, 188] };
pub const FWPM_CONDITION_INTERFACE_MAC_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4142284238, data2: 8011, data3: 19563, data4: [182, 239, 17, 101, 231, 31, 142, 231] };
pub const FWPM_CONDITION_INTERFACE_QUARANTINE_EPOCH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3437661534, data2: 1339, data3: 17320, data4: [154, 111, 51, 56, 76, 40, 228, 246] };
pub const FWPM_CONDITION_INTERFACE_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3673738516, data2: 57502, data3: 19603, data4: [165, 174, 197, 193, 59, 115, 255, 202] };
pub const FWPM_CONDITION_IPSEC_POLICY_KEY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2906119907, data2: 29231, data3: 17868, data4: [164, 227, 6, 128, 72, 18, 68, 82] };
pub const FWPM_CONDITION_IPSEC_SECURITY_REALM_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 933590784, data2: 22660, data3: 18788, data4: [146, 184, 62, 112, 70, 136, 176, 173] };
pub const FWPM_CONDITION_IP_ARRIVAL_INTERFACE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1636473709, data2: 14443, data3: 16694, data4: [173, 110, 181, 21, 135, 207, 177, 205] };
pub const FWPM_CONDITION_IP_DESTINATION_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 762909499, data2: 45968, data3: 17862, data4: [134, 153, 172, 172, 234, 175, 237, 51] };
pub const FWPM_CONDITION_IP_DESTINATION_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 516011977, data2: 20202, data3: 20318, data4: [185, 239, 118, 190, 170, 175, 23, 238] };
pub const FWPM_CONDITION_IP_DESTINATION_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3463311173, data2: 24827, data3: 19067, data4: [163, 4, 175, 48, 161, 23, 0, 14] };
pub const FWPM_CONDITION_IP_FORWARD_INTERFACE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 276215973, data2: 25379, data3: 19550, data4: [152, 16, 232, 211, 252, 158, 97, 54] };
pub const FWPM_CONDITION_IP_LOCAL_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3656253662, data2: 49647, data3: 17943, data4: [191, 227, 255, 216, 245, 160, 137, 87] };
pub const FWPM_CONDITION_IP_LOCAL_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1858598596, data2: 14187, data3: 17879, data4: [158, 156, 211, 55, 206, 220, 210, 55] };
pub const FWPM_CONDITION_IP_LOCAL_ADDRESS_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 61221323, data2: 28242, data3: 18936, data4: [156, 65, 87, 9, 99, 60, 9, 207] };
pub const FWPM_CONDITION_IP_LOCAL_ADDRESS_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 595705476, data2: 29988, data3: 17843, data4: [160, 91, 30, 99, 125, 156, 122, 106] };
pub const FWPM_CONDITION_IP_LOCAL_INTERFACE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1289103945, data2: 22979, data3: 18793, data4: [183, 243, 189, 165, 211, 40, 144, 164] };
pub const FWPM_CONDITION_IP_LOCAL_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 203137455, data2: 22373, data3: 17727, data4: [175, 34, 168, 247, 145, 172, 119, 91] };
pub const FWPM_CONDITION_IP_NEXTHOP_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3938337930, data2: 42769, data3: 19812, data4: [133, 183, 63, 118, 182, 82, 153, 199] };
pub const FWPM_CONDITION_IP_NEXTHOP_INTERFACE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2477690715, data2: 32623, data3: 18201, data4: [152, 200, 20, 233, 116, 41, 239, 4] };
pub const FWPM_CONDITION_IP_PHYSICAL_ARRIVAL_INTERFACE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3662730696, data2: 64013, data3: 19593, data4: [176, 50, 110, 98, 19, 109, 30, 150] };
pub const FWPM_CONDITION_IP_PHYSICAL_NEXTHOP_INTERFACE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4036744654, data2: 20816, data3: 18622, data4: [176, 152, 194, 81, 82, 251, 31, 146] };
pub const FWPM_CONDITION_IP_PROTOCOL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 963768107, data2: 25150, data3: 20378, data4: [140, 177, 110, 121, 184, 6, 185, 167] };
pub const FWPM_CONDITION_IP_REMOTE_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2989862554, data2: 7524, data3: 18872, data4: [164, 76, 95, 243, 217, 9, 80, 69] };
pub const FWPM_CONDITION_IP_REMOTE_ADDRESS_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 535541264, data2: 15308, data3: 17889, data4: [188, 54, 46, 6, 126, 44, 177, 134] };
pub const FWPM_CONDITION_IP_REMOTE_ADDRESS_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 611196300, data2: 35822, data3: 16408, data4: [155, 152, 49, 212, 88, 47, 51, 97] };
pub const FWPM_CONDITION_IP_REMOTE_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3277480013, data2: 53803, data3: 19994, data4: [145, 180, 104, 246, 116, 238, 103, 75] };
pub const FWPM_CONDITION_IP_SOURCE_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2929101182, data2: 11924, data3: 19401, data4: [179, 19, 178, 126, 232, 14, 87, 77] };
pub const FWPM_CONDITION_IP_SOURCE_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2796547985, data2: 15860, data3: 18224, data4: [162, 20, 245, 66, 106, 235, 248, 33] };
pub const FWPM_CONDITION_KM_AUTH_NAP_CONTEXT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 902883854, data2: 5578, data3: 18731, data4: [144, 14, 151, 253, 70, 53, 44, 206] };
pub const FWPM_CONDITION_KM_MODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4277093762, data2: 61327, data3: 20347, data4: [133, 139, 144, 119, 209, 34, 222, 71] };
pub const FWPM_CONDITION_KM_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4279197513, data2: 3307, data3: 18459, data4: [134, 56, 20, 121, 121, 31, 63, 44] };
pub const FWPM_CONDITION_L2_FLAGS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2076458175, data2: 14266, data3: 17905, data4: [183, 74, 130, 255, 81, 142, 235, 16] };
pub const FWPM_CONDITION_LOCAL_INTERFACE_PROFILE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1321170274, data2: 40728, data3: 19718, data4: [153, 65, 167, 166, 37, 116, 77, 113] };
pub const FWPM_CONDITION_MAC_DESTINATION_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 82455187, data2: 34188, data3: 16423, data4: [182, 19, 180, 49, 128, 199, 133, 158] };
pub const FWPM_CONDITION_MAC_DESTINATION_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2919573810, data2: 61250, data3: 20121, data4: [177, 41, 243, 179, 19, 158, 52, 247] };
pub const FWPM_CONDITION_MAC_LOCAL_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3650742657, data2: 31048, data3: 19587, data4: [183, 66, 200, 78, 59, 103, 143, 143] };
pub const FWPM_CONDITION_MAC_LOCAL_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3425776988, data2: 12403, data3: 20475, data4: [161, 79, 121, 65, 92, 177, 234, 209] };
pub const FWPM_CONDITION_MAC_REMOTE_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1083125460, data2: 14960, data3: 19277, data4: [146, 166, 65, 90, 194, 14, 47, 18] };
pub const FWPM_CONDITION_MAC_REMOTE_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 41938356, data2: 61889, data3: 16432, data4: [181, 100, 238, 119, 127, 216, 103, 234] };
pub const FWPM_CONDITION_MAC_SOURCE_ADDRESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2071549009, data2: 61942, data3: 19717, data4: [183, 203, 33, 119, 157, 128, 35, 54] };
pub const FWPM_CONDITION_MAC_SOURCE_ADDRESS_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1545302756, data2: 10654, data3: 17463, data4: [162, 152, 188, 63, 1, 75, 61, 194] };
pub const FWPM_CONDITION_NDIS_MEDIA_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3409039089, data2: 31005, data3: 18235, data4: [137, 209, 97, 197, 152, 67, 4, 160] };
pub const FWPM_CONDITION_NDIS_PHYSICAL_MEDIA_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 885495843, data2: 49705, data3: 17650, data4: [184, 60, 116, 2, 8, 130, 174, 119] };
pub const FWPM_CONDITION_NDIS_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3682317355, data2: 11692, data3: 19668, data4: [165, 154, 224, 189, 206, 30, 104, 52] };
pub const FWPM_CONDITION_NET_EVENT_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 544119190, data2: 18702, data3: 16591, data4: [184, 49, 179, 134, 65, 235, 111, 203] };
pub const FWPM_CONDITION_NEXTHOP_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 328099976, data2: 31416, data3: 19813, data4: [158, 232, 5, 145, 188, 246, 164, 148] };
pub const FWPM_CONDITION_NEXTHOP_INTERFACE_PROFILE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3623852630, data2: 52650, data3: 18219, data4: [132, 219, 210, 57, 99, 193, 209, 191] };
pub const FWPM_CONDITION_NEXTHOP_INTERFACE_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2538830956, data2: 55715, data3: 18279, data4: [163, 129, 233, 66, 103, 92, 217, 32] };
pub const FWPM_CONDITION_NEXTHOP_SUB_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4018823458, data2: 1399, data3: 17831, data4: [154, 175, 130, 95, 190, 180, 251, 149] };
pub const FWPM_CONDITION_NEXTHOP_TUNNEL_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1924243729, data2: 39035, data3: 18208, data4: [153, 221, 199, 197, 118, 250, 45, 76] };
pub const FWPM_CONDITION_ORIGINAL_ICMP_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 124648894, data2: 50540, data3: 20338, data4: [174, 138, 44, 254, 126, 92, 130, 134] };
pub const FWPM_CONDITION_ORIGINAL_PROFILE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1189746001, data2: 8789, data3: 18731, data4: [128, 25, 170, 190, 238, 52, 159, 64] };
pub const FWPM_CONDITION_PEER_NAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2605944962, data2: 60304, data3: 16774, data4: [166, 204, 222, 91, 99, 35, 80, 22] };
pub const FWPM_CONDITION_PIPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 466646045, data2: 58335, data3: 20004, data4: [134, 52, 118, 32, 70, 238, 246, 235] };
pub const FWPM_CONDITION_PROCESS_WITH_RPC_IF_UUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3809575080, data2: 48061, data3: 19732, data4: [166, 94, 113, 87, 176, 98, 51, 187] };
pub const FWPM_CONDITION_QM_MODE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4132423377, data2: 63947, data3: 17362, data4: [138, 95, 225, 59, 200, 148, 242, 101] };
pub const FWPM_CONDITION_REAUTHORIZE_REASON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 287334028, data2: 4526, data3: 17786, data4: [138, 68, 71, 112, 38, 221, 118, 74] };
pub const FWPM_CONDITION_REMOTE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4135675645, data2: 1666, data3: 19593, data4: [184, 245, 134, 67, 108, 126, 249, 183] };
pub const FWPM_CONDITION_REMOTE_USER_TOKEN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2616258150, data2: 1737, data3: 16825, data4: [132, 218, 40, 140, 180, 58, 245, 31] };
pub const FWPM_CONDITION_RESERVED0: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1737444843, data2: 17839, data3: 18562, data4: [147, 254, 25, 212, 114, 157, 152, 52] };
pub const FWPM_CONDITION_RESERVED1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3625515047, data2: 23657, data3: 18667, data4: [191, 128, 216, 107, 23, 117, 95, 151] };
pub const FWPM_CONDITION_RESERVED10: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3111772802, data2: 54817, data3: 19596, data4: [177, 132, 177, 5, 166, 28, 54, 206] };
pub const FWPM_CONDITION_RESERVED11: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 761458253, data2: 573, data3: 16671, data4: [149, 130, 67, 172, 187, 121, 89, 117] };
pub const FWPM_CONDITION_RESERVED12: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2741468210, data2: 32309, data3: 19932, data4: [147, 218, 232, 195, 63, 201, 35, 199] };
pub const FWPM_CONDITION_RESERVED13: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 861552272, data2: 33962, data3: 17141, data4: [158, 111, 89, 48, 149, 54, 164, 76] };
pub const FWPM_CONDITION_RESERVED14: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 820268450, data2: 12058, data3: 16662, data4: [165, 89, 249, 7, 222, 131, 96, 74] };
pub const FWPM_CONDITION_RESERVED15: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3132634127, data2: 45024, data3: 17361, data4: [128, 216, 92, 164, 86, 150, 45, 227] };
pub const FWPM_CONDITION_RESERVED2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1406407229, data2: 57691, data3: 20100, data4: [183, 168, 220, 225, 111, 123, 98, 217] };
pub const FWPM_CONDITION_RESERVED3: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2137951395, data2: 26118, data3: 18738, data4: [151, 199, 225, 242, 7, 16, 175, 59] };
pub const FWPM_CONDITION_RESERVED4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1599661634, data2: 47415, data3: 18782, data4: [169, 75, 246, 176, 81, 164, 146, 80] };
pub const FWPM_CONDITION_RESERVED5: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2611541709, data2: 63356, data3: 17382, data4: [136, 71, 17, 147, 157, 197, 219, 90] };
pub const FWPM_CONDITION_RESERVED6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4047340733, data2: 22997, data3: 17604, data4: [136, 23, 94, 205, 174, 24, 5, 189] };
pub const FWPM_CONDITION_RESERVED7: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1705048368, data2: 17885, data3: 18819, data4: [170, 51, 239, 199, 182, 17, 175, 8] };
pub const FWPM_CONDITION_RESERVED8: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1329744244, data2: 3090, data3: 18454, data4: [155, 71, 154, 84, 125, 179, 154, 50] };
pub const FWPM_CONDITION_RESERVED9: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3464028431, data2: 5119, data3: 19568, data4: [134, 67, 54, 173, 24, 121, 175, 163] };
pub const FWPM_CONDITION_RPC_AUTH_LEVEL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3852512981, data2: 22956, data3: 18154, data4: [190, 5, 165, 240, 94, 207, 68, 110] };
pub const FWPM_CONDITION_RPC_AUTH_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3669652651, data2: 3431, data3: 17383, data4: [152, 110, 117, 184, 79, 130, 245, 148] };
pub const FWPM_CONDITION_RPC_EP_FLAGS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 562790730, data2: 2617, data3: 18872, data4: [142, 113, 194, 12, 57, 199, 221, 46] };
pub const FWPM_CONDITION_RPC_EP_VALUE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3704529081, data2: 2182, data3: 17248, data4: [156, 106, 171, 4, 58, 36, 251, 169] };
pub const FWPM_CONDITION_RPC_IF_FLAG: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 596281906, data2: 12697, data3: 18045, data4: [135, 28, 39, 38, 33, 171, 56, 150] };
pub const FWPM_CONDITION_RPC_IF_UUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2090630559, data2: 117, data3: 19765, data4: [160, 209, 131, 17, 196, 207, 106, 241] };
pub const FWPM_CONDITION_RPC_IF_VERSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3938441655, data2: 4706, data3: 18990, data4: [173, 170, 95, 150, 246, 254, 50, 109] };
pub const FWPM_CONDITION_RPC_PROTOCOL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 655866996, data2: 14901, data3: 19687, data4: [183, 239, 200, 56, 250, 189, 236, 69] };
pub const FWPM_CONDITION_RPC_PROXY_AUTH_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1083523042, data2: 34149, data3: 18265, data4: [132, 136, 23, 113, 180, 180, 181, 219] };
pub const FWPM_CONDITION_RPC_SERVER_NAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3053822501, data2: 50099, data3: 18631, data4: [152, 51, 122, 239, 169, 82, 117, 70] };
pub const FWPM_CONDITION_RPC_SERVER_PORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2156983877, data2: 39637, data3: 20027, data4: [159, 159, 128, 35, 202, 9, 121, 9] };
pub const FWPM_CONDITION_SEC_ENCRYPT_ALGORITHM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 221277936, data2: 59764, data3: 20340, data4: [181, 199, 89, 27, 13, 167, 213, 98] };
pub const FWPM_CONDITION_SEC_KEY_SIZE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1198659643, data2: 52472, data3: 19179, data4: [188, 225, 198, 198, 22, 28, 143, 228] };
pub const FWPM_CONDITION_SOURCE_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 588329805, data2: 51501, data3: 17855, data4: [148, 150, 237, 244, 71, 130, 14, 45] };
pub const FWPM_CONDITION_SOURCE_SUB_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 90103197, data2: 44242, data3: 17249, data4: [141, 171, 249, 82, 93, 151, 102, 47] };
pub const FWPM_CONDITION_SUB_INTERFACE_INDEX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 215229555, data2: 54817, data3: 19427, data4: [174, 140, 114, 163, 72, 210, 131, 225] };
pub const FWPM_CONDITION_TUNNEL_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2007237687, data2: 34681, data3: 18536, data4: [162, 97, 245, 169, 2, 241, 192, 205] };
pub const FWPM_CONDITION_VLAN_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2475600673, data2: 13848, data3: 20068, data4: [156, 165, 33, 65, 235, 218, 28, 162] };
pub const FWPM_CONDITION_VSWITCH_DESTINATION_INTERFACE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2396294116, data2: 51494, data3: 18934, data4: [164, 246, 239, 48, 48, 227, 252, 22] };
pub const FWPM_CONDITION_VSWITCH_DESTINATION_INTERFACE_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4204478214, data2: 12058, data3: 19543, data4: [158, 104, 167, 9, 139, 40, 219, 254] };
pub const FWPM_CONDITION_VSWITCH_DESTINATION_VM_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1627826894, data2: 19937, data3: 19588, data4: [150, 113, 54, 55, 248, 188, 247, 49] };
pub const FWPM_CONDITION_VSWITCH_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3299087546, data2: 17275, data3: 19942, data4: [153, 70, 217, 156, 27, 149, 179, 18] };
pub const FWPM_CONDITION_VSWITCH_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 299141963, data2: 59258, data3: 16564, data4: [145, 85, 57, 44, 144, 108, 38, 8] };
pub const FWPM_CONDITION_VSWITCH_SOURCE_INTERFACE_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2135880267, data2: 45761, data3: 18744, data4: [186, 51, 161, 236, 190, 213, 18, 186] };
pub const FWPM_CONDITION_VSWITCH_SOURCE_INTERFACE_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3870310562, data2: 60847, data3: 19510, data4: [144, 139, 242, 245, 138, 228, 56, 7] };
pub const FWPM_CONDITION_VSWITCH_SOURCE_VM_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2620038850, data2: 40902, data3: 17084, data4: [189, 216, 64, 109, 77, 160, 190, 100] };
pub const FWPM_CONDITION_VSWITCH_TENANT_NETWORK_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3691283516, data2: 31206, data3: 20036, data4: [160, 37, 101, 185, 187, 15, 159, 148] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_CONNECTION0 {
    pub connectionId: u64,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: FWPM_CONNECTION0_0,
    pub Anonymous2: FWPM_CONNECTION0_1,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub ipsecTrafficModeType: IPSEC_TRAFFIC_TYPE,
    pub keyModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub mmCrypto: IKEEXT_PROPOSAL0,
    pub mmPeer: IKEEXT_CREDENTIAL2,
    pub emPeer: IKEEXT_CREDENTIAL2,
    pub bytesTransferredIn: u64,
    pub bytesTransferredOut: u64,
    pub bytesTransferredTotal: u64,
    pub startSysTime: super::super::Foundation::FILETIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_CONNECTION0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_CONNECTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union FWPM_CONNECTION0_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_CONNECTION0_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_CONNECTION0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union FWPM_CONNECTION0_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_CONNECTION0_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_CONNECTION0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type FWPM_CONNECTION_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, eventtype: FWPM_CONNECTION_EVENT_TYPE, connection: *const FWPM_CONNECTION0)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CONNECTION_ENUM_FLAG_QUERY_BYTES_TRANSFERRED: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_CONNECTION_ENUM_TEMPLATE0 {
    pub connectionId: u64,
    pub flags: u32,
}
impl ::core::marker::Copy for FWPM_CONNECTION_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_CONNECTION_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_CONNECTION_EVENT_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CONNECTION_EVENT_ADD: FWPM_CONNECTION_EVENT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CONNECTION_EVENT_DELETE: FWPM_CONNECTION_EVENT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CONNECTION_EVENT_MAX: FWPM_CONNECTION_EVENT_TYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_CONNECTION_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_CONNECTION_ENUM_TEMPLATE0,
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_CONNECTION_SUBSCRIPTION0 {}
impl ::core::clone::Clone for FWPM_CONNECTION_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_DISPLAY_DATA0 {
    pub name: ::windows_sys::core::PWSTR,
    pub description: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for FWPM_DISPLAY_DATA0 {}
impl ::core::clone::Clone for FWPM_DISPLAY_DATA0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_DYNAMIC_KEYWORD_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(notification: *mut ::core::ffi::c_void, context: *mut ::core::ffi::c_void)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_ENGINE_OPTION = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_COLLECT_NET_EVENTS: FWPM_ENGINE_OPTION = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_NET_EVENT_MATCH_ANY_KEYWORDS: FWPM_ENGINE_OPTION = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_NAME_CACHE: FWPM_ENGINE_OPTION = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_MONITOR_IPSEC_CONNECTIONS: FWPM_ENGINE_OPTION = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_PACKET_QUEUING: FWPM_ENGINE_OPTION = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_TXN_WATCHDOG_TIMEOUT_IN_MSEC: FWPM_ENGINE_OPTION = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_OPTION_MAX: FWPM_ENGINE_OPTION = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_OPTION_PACKET_BATCH_INBOUND: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_OPTION_PACKET_QUEUE_FORWARD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_OPTION_PACKET_QUEUE_INBOUND: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_ENGINE_OPTION_PACKET_QUEUE_NONE: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_FIELD0 {
    pub fieldKey: *mut ::windows_sys::core::GUID,
    pub r#type: FWPM_FIELD_TYPE,
    pub dataType: FWP_DATA_TYPE,
}
impl ::core::marker::Copy for FWPM_FIELD0 {}
impl ::core::clone::Clone for FWPM_FIELD0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_FIELD_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FIELD_RAW_DATA: FWPM_FIELD_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FIELD_IP_ADDRESS: FWPM_FIELD_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FIELD_FLAGS: FWPM_FIELD_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FIELD_TYPE_MAX: FWPM_FIELD_TYPE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_FILTER0 {
    pub filterKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: FWPM_FILTER_FLAGS,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub layerKey: ::windows_sys::core::GUID,
    pub subLayerKey: ::windows_sys::core::GUID,
    pub weight: FWP_VALUE0,
    pub numFilterConditions: u32,
    pub filterCondition: *mut FWPM_FILTER_CONDITION0,
    pub action: FWPM_ACTION0,
    pub Anonymous: FWPM_FILTER0_0,
    pub reserved: *mut ::windows_sys::core::GUID,
    pub filterId: u64,
    pub effectiveWeight: FWP_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_FILTER0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_FILTER0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_FILTER0_0 {
    pub rawContext: u64,
    pub providerContextKey: ::windows_sys::core::GUID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_FILTER0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_FILTER0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_FILTER_CHANGE0 {
    pub changeType: FWPM_CHANGE_TYPE,
    pub filterKey: ::windows_sys::core::GUID,
    pub filterId: u64,
}
impl ::core::marker::Copy for FWPM_FILTER_CHANGE0 {}
impl ::core::clone::Clone for FWPM_FILTER_CHANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_FILTER_CHANGE_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, change: *const FWPM_FILTER_CHANGE0)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_FILTER_CONDITION0 {
    pub fieldKey: ::windows_sys::core::GUID,
    pub matchType: FWP_MATCH_TYPE,
    pub conditionValue: FWP_CONDITION_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_FILTER_CONDITION0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_FILTER_CONDITION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_FILTER_ENUM_TEMPLATE0 {
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub layerKey: ::windows_sys::core::GUID,
    pub enumType: FWP_FILTER_ENUM_TYPE,
    pub flags: u32,
    pub providerContextTemplate: *mut FWPM_PROVIDER_CONTEXT_ENUM_TEMPLATE0,
    pub numFilterConditions: u32,
    pub filterCondition: *mut FWPM_FILTER_CONDITION0,
    pub actionMask: u32,
    pub calloutKey: *mut ::windows_sys::core::GUID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_FILTER_ENUM_TEMPLATE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_FILTER_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_FILTER_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_NONE: FWPM_FILTER_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_PERSISTENT: FWPM_FILTER_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_BOOTTIME: FWPM_FILTER_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT: FWPM_FILTER_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT: FWPM_FILTER_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_PERMIT_IF_CALLOUT_UNREGISTERED: FWPM_FILTER_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_DISABLED: FWPM_FILTER_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_INDEXED: FWPM_FILTER_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_GAMEOS_ONLY: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_HAS_SECURITY_REALM_PROVIDER_CONTEXT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_IPSEC_NO_ACQUIRE_INITIATE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_RESERVED0: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_RESERVED1: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_SILENT_MODE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_FILTER_FLAG_SYSTEMOS_ONLY: u32 = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_FILTER_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_FILTER_ENUM_TEMPLATE0,
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_FILTER_SUBSCRIPTION0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_FILTER_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_KEYING_MODULE_AUTHIP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 300145376, data2: 56614, data3: 17808, data4: [133, 125, 171, 75, 40, 209, 160, 149] };
pub const FWPM_KEYING_MODULE_IKE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2847668103, data2: 33448, data3: 17851, data4: [164, 0, 93, 126, 89, 82, 199, 169] };
pub const FWPM_KEYING_MODULE_IKEV2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 68653772, data2: 36615, data3: 16797, data4: [163, 148, 113, 105, 104, 203, 22, 71] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_LAYER0 {
    pub layerKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub numFields: u32,
    pub field: *mut FWPM_FIELD0,
    pub defaultSubLayerKey: ::windows_sys::core::GUID,
    pub layerId: u16,
}
impl ::core::marker::Copy for FWPM_LAYER0 {}
impl ::core::clone::Clone for FWPM_LAYER0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_LAYER_ALE_AUTH_CONNECT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3280820177, data2: 1447, data3: 19507, data4: [144, 79, 127, 188, 238, 230, 14, 130] };
pub const FWPM_LAYER_ALE_AUTH_CONNECT_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3593644033, data2: 62906, data3: 19158, data4: [150, 227, 96, 112, 23, 217, 131, 106] };
pub const FWPM_LAYER_ALE_AUTH_CONNECT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1248999739, data2: 12703, data3: 17596, data4: [132, 195, 186, 84, 220, 179, 182, 180] };
pub const FWPM_LAYER_ALE_AUTH_CONNECT_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3380331448, data2: 51619, data3: 20019, data4: [134, 149, 142, 23, 170, 212, 222, 9] };
pub const FWPM_LAYER_ALE_AUTH_LISTEN_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2293980589, data2: 30423, data3: 16935, data4: [156, 113, 223, 10, 62, 215, 190, 126] };
pub const FWPM_LAYER_ALE_AUTH_LISTEN_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 924711642, data2: 40742, data3: 17917, data4: [180, 235, 194, 158, 178, 18, 137, 63] };
pub const FWPM_LAYER_ALE_AUTH_LISTEN_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2060049956, data2: 6109, data3: 18452, data4: [180, 189, 169, 251, 201, 90, 50, 27] };
pub const FWPM_LAYER_ALE_AUTH_LISTEN_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1617967879, data2: 25544, data3: 18665, data4: [173, 163, 18, 177, 175, 64, 166, 23] };
pub const FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3788349415, data2: 62645, data3: 17011, data4: [150, 192, 89, 46, 72, 123, 134, 80] };
pub const FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2666178971, data2: 48418, data3: 16935, data4: [145, 159, 0, 115, 198, 51, 87, 177] };
pub const FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2746494103, data2: 40708, data3: 18034, data4: [184, 126, 206, 233, 196, 131, 37, 127] };
pub const FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2303024023, data2: 56289, data3: 17727, data4: [162, 36, 19, 218, 137, 90, 243, 150] };
pub const FWPM_LAYER_ALE_BIND_REDIRECT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1721207981, data2: 50948, data3: 17068, data4: [134, 172, 124, 26, 35, 27, 210, 83] };
pub const FWPM_LAYER_ALE_BIND_REDIRECT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3203411100, data2: 24683, data3: 17718, data4: [140, 38, 28, 47, 199, 182, 49, 212] };
pub const FWPM_LAYER_ALE_CONNECT_REDIRECT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3336977548, data2: 46980, data3: 17762, data4: [170, 125, 10, 103, 207, 202, 249, 163] };
pub const FWPM_LAYER_ALE_CONNECT_REDIRECT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1484674215, data2: 32838, data3: 17082, data4: [160, 170, 183, 22, 37, 15, 199, 253] };
pub const FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3027657767, data2: 58018, data3: 18042, data4: [189, 126, 219, 205, 27, 216, 90, 9] };
pub const FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3142806733, data2: 18261, data3: 19369, data4: [159, 247, 249, 237, 248, 105, 156, 123] };
pub const FWPM_LAYER_ALE_FLOW_ESTABLISHED_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2944419594, data2: 21910, data3: 19475, data4: [153, 146, 83, 158, 111, 229, 121, 103] };
pub const FWPM_LAYER_ALE_FLOW_ESTABLISHED_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 342549673, data2: 41426, data3: 19779, data4: [163, 26, 76, 66, 104, 43, 142, 79] };
pub const FWPM_LAYER_ALE_FLOW_ESTABLISHED_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1881264819, data2: 57252, data3: 16494, data4: [175, 235, 106, 250, 247, 231, 14, 253] };
pub const FWPM_LAYER_ALE_FLOW_ESTABLISHED_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1184007734, data2: 48074, data3: 19318, data4: [148, 29, 15, 167, 245, 215, 211, 114] };
pub const FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 306697837, data2: 2912, data3: 18965, data4: [141, 68, 113, 85, 208, 245, 58, 12] };
pub const FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 190321314, data2: 50175, data3: 20170, data4: [184, 141, 199, 158, 32, 172, 99, 34] };
pub const FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1436963041, data2: 24330, data3: 20170, data4: [166, 83, 136, 245, 59, 38, 170, 140] };
pub const FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3418986683, data2: 50463, data3: 19482, data4: [187, 79, 151, 117, 252, 172, 171, 47] };
pub const FWPM_LAYER_ALE_RESOURCE_RELEASE_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1949719758, data2: 52400, data3: 16410, data4: [191, 193, 184, 153, 52, 173, 126, 21] };
pub const FWPM_LAYER_ALE_RESOURCE_RELEASE_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4108701312, data2: 60876, data3: 19987, data4: [138, 47, 185, 20, 84, 187, 5, 123] };
pub const FWPM_LAYER_DATAGRAM_DATA_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1023983438, data2: 17910, data3: 18736, data4: [169, 34, 65, 112, 152, 226, 0, 39] };
pub const FWPM_LAYER_DATAGRAM_DATA_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 417542342, data2: 29256, data3: 20050, data4: [170, 171, 71, 46, 214, 119, 4, 253] };
pub const FWPM_LAYER_DATAGRAM_DATA_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4198891055, data2: 15546, data3: 17447, data4: [135, 252, 87, 185, 164, 177, 13, 0] };
pub const FWPM_LAYER_DATAGRAM_DATA_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 164749281, data2: 39814, data3: 19010, data4: [190, 157, 140, 49, 91, 146, 165, 208] };
pub const FWPM_LAYER_EGRESS_VSWITCH_ETHERNET: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2261283504, data2: 30458, data3: 19321, data4: [147, 164, 7, 80, 83, 10, 226, 146] };
pub const FWPM_LAYER_EGRESS_VSWITCH_TRANSPORT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3106099382, data2: 37360, data3: 18102, data4: [189, 196, 135, 29, 253, 74, 124, 152] };
pub const FWPM_LAYER_EGRESS_VSWITCH_TRANSPORT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 455995171, data2: 6273, data3: 16573, data4: [130, 244, 66, 84, 230, 49, 65, 203] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_LAYER_ENUM_TEMPLATE0 {
    pub reserved: u64,
}
impl ::core::marker::Copy for FWPM_LAYER_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_LAYER_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_LAYER_FLAG_BUFFERED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_LAYER_FLAG_BUILTIN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_LAYER_FLAG_CLASSIFY_MOSTLY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_LAYER_FLAG_KERNEL: u32 = 1u32;
pub const FWPM_LAYER_IKEEXT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2974514139, data2: 56253, data3: 18238, data4: [190, 212, 139, 71, 8, 212, 242, 112] };
pub const FWPM_LAYER_IKEEXT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3058140851, data2: 63111, data3: 20153, data4: [137, 210, 142, 243, 42, 205, 171, 226] };
pub const FWPM_LAYER_INBOUND_ICMP_ERROR_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1632213392, data2: 15542, data3: 20100, data4: [185, 80, 83, 185, 75, 105, 100, 243] };
pub const FWPM_LAYER_INBOUND_ICMP_ERROR_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2796646517, data2: 60335, data3: 16467, data4: [164, 231, 33, 60, 129, 33, 237, 229] };
pub const FWPM_LAYER_INBOUND_ICMP_ERROR_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1710865919, data2: 15149, data3: 20061, data4: [184, 198, 199, 32, 101, 31, 232, 152] };
pub const FWPM_LAYER_INBOUND_ICMP_ERROR_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2800209088, data2: 2299, data3: 18061, data4: [164, 114, 151, 113, 213, 89, 94, 9] };
pub const FWPM_LAYER_INBOUND_IPPACKET_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3362771391, data2: 8653, data3: 18814, data4: [160, 187, 23, 66, 92, 136, 92, 88] };
pub const FWPM_LAYER_INBOUND_IPPACKET_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3047305424, data2: 43200, data3: 17650, data4: [145, 110, 153, 27, 83, 222, 209, 247] };
pub const FWPM_LAYER_INBOUND_IPPACKET_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4112528075, data2: 39196, data3: 18151, data4: [151, 29, 38, 1, 69, 154, 145, 202] };
pub const FWPM_LAYER_INBOUND_IPPACKET_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3139748473, data2: 37812, data3: 18338, data4: [131, 173, 174, 22, 152, 181, 8, 133] };
pub const FWPM_LAYER_INBOUND_MAC_FRAME_ETHERNET: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4026236635, data2: 85, data3: 20378, data4: [162, 49, 79, 248, 19, 26, 209, 145] };
pub const FWPM_LAYER_INBOUND_MAC_FRAME_NATIVE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3559001043, data2: 25294, data3: 20232, data4: [174, 136, 181, 110, 133, 38, 223, 80] };
pub const FWPM_LAYER_INBOUND_MAC_FRAME_NATIVE_FAST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2235214478, data2: 11128, data3: 19748, data4: [168, 4, 54, 219, 8, 178, 151, 17] };
pub const FWPM_LAYER_INBOUND_RESERVED2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4110126421, data2: 49270, data3: 18136, data4: [162, 199, 106, 76, 114, 44, 164, 237] };
pub const FWPM_LAYER_INBOUND_TRANSPORT_FAST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3827115801, data2: 1479, data3: 16624, data4: [137, 131, 234, 141, 23, 187, 194, 246] };
pub const FWPM_LAYER_INBOUND_TRANSPORT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1495719880, data2: 58319, data3: 17446, data4: [162, 131, 220, 57, 63, 93, 15, 157] };
pub const FWPM_LAYER_INBOUND_TRANSPORT_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2890569779, data2: 63133, data3: 17992, data4: [178, 97, 109, 200, 72, 53, 239, 57] };
pub const FWPM_LAYER_INBOUND_TRANSPORT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1665828511, data2: 64547, data3: 19344, data4: [176, 193, 191, 98, 10, 54, 174, 111] };
pub const FWPM_LAYER_INBOUND_TRANSPORT_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 711981397, data2: 15147, data3: 18898, data4: [152, 72, 173, 157, 114, 220, 170, 183] };
pub const FWPM_LAYER_INGRESS_VSWITCH_ETHERNET: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2107135866, data2: 39559, data3: 16876, data4: [151, 24, 124, 245, 137, 201, 243, 45] };
pub const FWPM_LAYER_INGRESS_VSWITCH_TRANSPORT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2993254390, data2: 30543, data3: 17748, data4: [159, 125, 61, 163, 148, 95, 142, 133] };
pub const FWPM_LAYER_INGRESS_VSWITCH_TRANSPORT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1591940348, data2: 32138, data3: 18420, data4: [183, 227, 41, 26, 54, 218, 78, 18] };
pub const FWPM_LAYER_IPFORWARD_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2821377060, data2: 20193, data3: 20193, data4: [180, 101, 253, 29, 37, 203, 16, 164] };
pub const FWPM_LAYER_IPFORWARD_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2661197683, data2: 12206, data3: 16912, data4: [143, 23, 52, 18, 158, 243, 105, 235] };
pub const FWPM_LAYER_IPFORWARD_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2073446424, data2: 6599, data3: 18746, data4: [183, 31, 131, 44, 54, 132, 210, 140] };
pub const FWPM_LAYER_IPFORWARD_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 827476573, data2: 7678, data3: 18223, data4: [187, 147, 81, 142, 233, 69, 216, 162] };
pub const FWPM_LAYER_IPSEC_KM_DEMUX_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4029355302, data2: 42073, data3: 19025, data4: [185, 227, 117, 157, 229, 43, 157, 44] };
pub const FWPM_LAYER_IPSEC_KM_DEMUX_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 796220662, data2: 12244, data3: 20104, data4: [179, 228, 169, 27, 202, 73, 82, 53] };
pub const FWPM_LAYER_IPSEC_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3987102836, data2: 24845, data3: 19397, data4: [148, 143, 60, 79, 137, 85, 104, 103] };
pub const FWPM_LAYER_IPSEC_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 331646018, data2: 36231, data3: 16993, data4: [154, 41, 89, 210, 171, 195, 72, 180] };
pub const FWPM_LAYER_KM_AUTHORIZATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1252140777, data2: 36896, data3: 17915, data4: [149, 106, 192, 36, 157, 132, 17, 149] };
pub const FWPM_LAYER_NAME_RESOLUTION_CACHE_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 204121729, data2: 36955, data3: 19661, data4: [164, 103, 77, 216, 17, 208, 123, 123] };
pub const FWPM_LAYER_NAME_RESOLUTION_CACHE_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2463470330, data2: 27393, data3: 17226, data4: [157, 234, 209, 233, 110, 169, 125, 169] };
pub const FWPM_LAYER_OUTBOUND_ICMP_ERROR_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1094254848, data2: 22092, data3: 19250, data4: [188, 29, 113, 128, 72, 53, 77, 124] };
pub const FWPM_LAYER_OUTBOUND_ICMP_ERROR_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3008990518, data2: 1377, data3: 17800, data4: [166, 191, 233, 85, 227, 246, 38, 75] };
pub const FWPM_LAYER_OUTBOUND_ICMP_ERROR_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2142255968, data2: 31629, data3: 19962, data4: [186, 221, 152, 1, 118, 252, 78, 18] };
pub const FWPM_LAYER_OUTBOUND_ICMP_ERROR_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1710417479, data2: 36108, data3: 20295, data4: [177, 155, 51, 164, 211, 241, 53, 124] };
pub const FWPM_LAYER_OUTBOUND_IPPACKET_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 509386670, data2: 35460, data3: 16693, data4: [163, 49, 149, 11, 84, 34, 158, 205] };
pub const FWPM_LAYER_OUTBOUND_IPPACKET_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 149208245, data2: 46663, data3: 18675, data4: [149, 60, 229, 221, 189, 3, 147, 126] };
pub const FWPM_LAYER_OUTBOUND_IPPACKET_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2746461035, data2: 13668, data3: 18572, data4: [145, 23, 243, 78, 130, 20, 39, 99] };
pub const FWPM_LAYER_OUTBOUND_IPPACKET_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2501105604, data2: 43316, data3: 18908, data4: [145, 167, 108, 203, 128, 204, 2, 227] };
pub const FWPM_LAYER_OUTBOUND_MAC_FRAME_ETHERNET: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1766224828, data2: 55003, data3: 18544, data4: [173, 238, 10, 205, 189, 183, 244, 178] };
pub const FWPM_LAYER_OUTBOUND_MAC_FRAME_NATIVE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2495891730, data2: 40303, data3: 20159, data4: [185, 149, 5, 171, 138, 8, 141, 27] };
pub const FWPM_LAYER_OUTBOUND_MAC_FRAME_NATIVE_FAST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1192098118, data2: 51554, data3: 18543, data4: [148, 70, 130, 147, 203, 199, 94, 184] };
pub const FWPM_LAYER_OUTBOUND_TRANSPORT_FAST: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 334316424, data2: 41072, data3: 18453, data4: [153, 53, 122, 155, 230, 64, 139, 120] };
pub const FWPM_LAYER_OUTBOUND_TRANSPORT_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 166075114, data2: 53780, data3: 18146, data4: [155, 33, 178, 107, 11, 47, 40, 200] };
pub const FWPM_LAYER_OUTBOUND_TRANSPORT_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3320907089, data2: 48560, data3: 17367, data4: [163, 19, 80, 226, 17, 244, 214, 138] };
pub const FWPM_LAYER_OUTBOUND_TRANSPORT_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3782433758, data2: 319, data3: 18005, data4: [179, 81, 164, 158, 21, 118, 45, 240] };
pub const FWPM_LAYER_OUTBOUND_TRANSPORT_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4097040233, data2: 52413, data3: 18478, data4: [185, 178, 87, 22, 86, 88, 195, 179] };
pub const FWPM_LAYER_RPC_EPMAP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2454174817, data2: 60167, data3: 18414, data4: [135, 44, 191, 215, 139, 253, 22, 22] };
pub const FWPM_LAYER_RPC_EP_ADD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1636696007, data2: 50256, data3: 18755, data4: [149, 219, 153, 180, 193, 106, 85, 212] };
pub const FWPM_LAYER_RPC_PROXY_CONN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2493822219, data2: 47708, data3: 20263, data4: [144, 122, 34, 159, 172, 12, 42, 122] };
pub const FWPM_LAYER_RPC_PROXY_IF: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4171466261, data2: 57644, data3: 16812, data4: [152, 223, 18, 26, 217, 129, 170, 222] };
pub const FWPM_LAYER_RPC_UM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1973984730, data2: 38372, data3: 16627, data4: [173, 199, 118, 136, 169, 200, 71, 225] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_LAYER_STATISTICS0 {
    pub layerId: ::windows_sys::core::GUID,
    pub classifyPermitCount: u32,
    pub classifyBlockCount: u32,
    pub classifyVetoCount: u32,
    pub numCacheEntries: u32,
}
impl ::core::marker::Copy for FWPM_LAYER_STATISTICS0 {}
impl ::core::clone::Clone for FWPM_LAYER_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_LAYER_STREAM_PACKET_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2941442284, data2: 52013, data3: 17637, data4: [173, 146, 248, 220, 56, 210, 235, 41] };
pub const FWPM_LAYER_STREAM_PACKET_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2006617251, data2: 61593, data3: 18063, data4: [181, 212, 131, 83, 92, 70, 28, 2] };
pub const FWPM_LAYER_STREAM_V4: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 998860092, data2: 49520, data3: 18916, data4: [177, 205, 224, 238, 238, 225, 154, 62] };
pub const FWPM_LAYER_STREAM_V4_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 633651906, data2: 9727, data3: 17234, data4: [130, 249, 197, 74, 74, 71, 38, 220] };
pub const FWPM_LAYER_STREAM_V6: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1204360058, data2: 32452, data3: 18099, data4: [182, 228, 72, 233, 38, 177, 237, 164] };
pub const FWPM_LAYER_STREAM_V6_DISCARD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 279289799, data2: 46632, data3: 19521, data4: [158, 184, 207, 55, 213, 81, 3, 207] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT0 {
    pub header: FWPM_NET_EVENT_HEADER0,
    pub r#type: FWPM_NET_EVENT_TYPE,
    pub Anonymous: FWPM_NET_EVENT0_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT0_0 {
    pub ikeMmFailure: *mut FWPM_NET_EVENT_IKEEXT_MM_FAILURE0,
    pub ikeQmFailure: *mut FWPM_NET_EVENT_IKEEXT_QM_FAILURE0,
    pub ikeEmFailure: *mut FWPM_NET_EVENT_IKEEXT_EM_FAILURE0,
    pub classifyDrop: *mut FWPM_NET_EVENT_CLASSIFY_DROP0,
    pub ipsecDrop: *mut FWPM_NET_EVENT_IPSEC_KERNEL_DROP0,
    pub idpDrop: *mut FWPM_NET_EVENT_IPSEC_DOSP_DROP0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT1 {
    pub header: FWPM_NET_EVENT_HEADER1,
    pub r#type: FWPM_NET_EVENT_TYPE,
    pub Anonymous: FWPM_NET_EVENT1_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT1_0 {
    pub ikeMmFailure: *mut FWPM_NET_EVENT_IKEEXT_MM_FAILURE1,
    pub ikeQmFailure: *mut FWPM_NET_EVENT_IKEEXT_QM_FAILURE0,
    pub ikeEmFailure: *mut FWPM_NET_EVENT_IKEEXT_EM_FAILURE1,
    pub classifyDrop: *mut FWPM_NET_EVENT_CLASSIFY_DROP1,
    pub ipsecDrop: *mut FWPM_NET_EVENT_IPSEC_KERNEL_DROP0,
    pub idpDrop: *mut FWPM_NET_EVENT_IPSEC_DOSP_DROP0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT2 {
    pub header: FWPM_NET_EVENT_HEADER2,
    pub r#type: FWPM_NET_EVENT_TYPE,
    pub Anonymous: FWPM_NET_EVENT2_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT2_0 {
    pub ikeMmFailure: *mut FWPM_NET_EVENT_IKEEXT_MM_FAILURE1,
    pub ikeQmFailure: *mut FWPM_NET_EVENT_IKEEXT_QM_FAILURE0,
    pub ikeEmFailure: *mut FWPM_NET_EVENT_IKEEXT_EM_FAILURE1,
    pub classifyDrop: *mut FWPM_NET_EVENT_CLASSIFY_DROP2,
    pub ipsecDrop: *mut FWPM_NET_EVENT_IPSEC_KERNEL_DROP0,
    pub idpDrop: *mut FWPM_NET_EVENT_IPSEC_DOSP_DROP0,
    pub classifyAllow: *mut FWPM_NET_EVENT_CLASSIFY_ALLOW0,
    pub capabilityDrop: *mut FWPM_NET_EVENT_CAPABILITY_DROP0,
    pub capabilityAllow: *mut FWPM_NET_EVENT_CAPABILITY_ALLOW0,
    pub classifyDropMac: *mut FWPM_NET_EVENT_CLASSIFY_DROP_MAC0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT3 {
    pub header: FWPM_NET_EVENT_HEADER3,
    pub r#type: FWPM_NET_EVENT_TYPE,
    pub Anonymous: FWPM_NET_EVENT3_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT3_0 {
    pub ikeMmFailure: *mut FWPM_NET_EVENT_IKEEXT_MM_FAILURE1,
    pub ikeQmFailure: *mut FWPM_NET_EVENT_IKEEXT_QM_FAILURE0,
    pub ikeEmFailure: *mut FWPM_NET_EVENT_IKEEXT_EM_FAILURE1,
    pub classifyDrop: *mut FWPM_NET_EVENT_CLASSIFY_DROP2,
    pub ipsecDrop: *mut FWPM_NET_EVENT_IPSEC_KERNEL_DROP0,
    pub idpDrop: *mut FWPM_NET_EVENT_IPSEC_DOSP_DROP0,
    pub classifyAllow: *mut FWPM_NET_EVENT_CLASSIFY_ALLOW0,
    pub capabilityDrop: *mut FWPM_NET_EVENT_CAPABILITY_DROP0,
    pub capabilityAllow: *mut FWPM_NET_EVENT_CAPABILITY_ALLOW0,
    pub classifyDropMac: *mut FWPM_NET_EVENT_CLASSIFY_DROP_MAC0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT3_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT3_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT4_ {
    pub header: FWPM_NET_EVENT_HEADER3,
    pub r#type: FWPM_NET_EVENT_TYPE,
    pub Anonymous: FWPM_NET_EVENT4__0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT4_ {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT4_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT4__0 {
    pub ikeMmFailure: *mut FWPM_NET_EVENT_IKEEXT_MM_FAILURE2_,
    pub ikeQmFailure: *mut FWPM_NET_EVENT_IKEEXT_QM_FAILURE1_,
    pub ikeEmFailure: *mut FWPM_NET_EVENT_IKEEXT_EM_FAILURE1,
    pub classifyDrop: *mut FWPM_NET_EVENT_CLASSIFY_DROP2,
    pub ipsecDrop: *mut FWPM_NET_EVENT_IPSEC_KERNEL_DROP0,
    pub idpDrop: *mut FWPM_NET_EVENT_IPSEC_DOSP_DROP0,
    pub classifyAllow: *mut FWPM_NET_EVENT_CLASSIFY_ALLOW0,
    pub capabilityDrop: *mut FWPM_NET_EVENT_CAPABILITY_DROP0,
    pub capabilityAllow: *mut FWPM_NET_EVENT_CAPABILITY_ALLOW0,
    pub classifyDropMac: *mut FWPM_NET_EVENT_CLASSIFY_DROP_MAC0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT4__0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT4__0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT5_ {
    pub header: FWPM_NET_EVENT_HEADER3,
    pub r#type: FWPM_NET_EVENT_TYPE,
    pub Anonymous: FWPM_NET_EVENT5__0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT5_ {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT5_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT5__0 {
    pub ikeMmFailure: *mut FWPM_NET_EVENT_IKEEXT_MM_FAILURE2_,
    pub ikeQmFailure: *mut FWPM_NET_EVENT_IKEEXT_QM_FAILURE1_,
    pub ikeEmFailure: *mut FWPM_NET_EVENT_IKEEXT_EM_FAILURE1,
    pub classifyDrop: *mut FWPM_NET_EVENT_CLASSIFY_DROP2,
    pub ipsecDrop: *mut FWPM_NET_EVENT_IPSEC_KERNEL_DROP0,
    pub idpDrop: *mut FWPM_NET_EVENT_IPSEC_DOSP_DROP0,
    pub classifyAllow: *mut FWPM_NET_EVENT_CLASSIFY_ALLOW0,
    pub capabilityDrop: *mut FWPM_NET_EVENT_CAPABILITY_DROP0,
    pub capabilityAllow: *mut FWPM_NET_EVENT_CAPABILITY_ALLOW0,
    pub classifyDropMac: *mut FWPM_NET_EVENT_CLASSIFY_DROP_MAC0,
    pub lpmPacketArrival: *mut FWPM_NET_EVENT_LPM_PACKET_ARRIVAL0_,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT5__0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT5__0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type FWPM_NET_EVENT_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, event: *const FWPM_NET_EVENT1)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type FWPM_NET_EVENT_CALLBACK1 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, event: *const FWPM_NET_EVENT2)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type FWPM_NET_EVENT_CALLBACK2 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, event: *const FWPM_NET_EVENT3)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type FWPM_NET_EVENT_CALLBACK3 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, event: *const FWPM_NET_EVENT4_)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type FWPM_NET_EVENT_CALLBACK4 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, event: *const FWPM_NET_EVENT5_)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_NET_EVENT_CAPABILITY_ALLOW0 {
    pub networkCapabilityId: FWPM_APPC_NETWORK_CAPABILITY_TYPE,
    pub filterId: u64,
    pub isLoopback: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_NET_EVENT_CAPABILITY_ALLOW0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_NET_EVENT_CAPABILITY_ALLOW0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_NET_EVENT_CAPABILITY_DROP0 {
    pub networkCapabilityId: FWPM_APPC_NETWORK_CAPABILITY_TYPE,
    pub filterId: u64,
    pub isLoopback: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_NET_EVENT_CAPABILITY_DROP0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_NET_EVENT_CAPABILITY_DROP0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_NET_EVENT_CLASSIFY_ALLOW0 {
    pub filterId: u64,
    pub layerId: u16,
    pub reauthReason: u32,
    pub originalProfile: u32,
    pub currentProfile: u32,
    pub msFwpDirection: u32,
    pub isLoopback: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_NET_EVENT_CLASSIFY_ALLOW0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_NET_EVENT_CLASSIFY_ALLOW0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_CLASSIFY_DROP0 {
    pub filterId: u64,
    pub layerId: u16,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_CLASSIFY_DROP0 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_CLASSIFY_DROP0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_NET_EVENT_CLASSIFY_DROP1 {
    pub filterId: u64,
    pub layerId: u16,
    pub reauthReason: u32,
    pub originalProfile: u32,
    pub currentProfile: u32,
    pub msFwpDirection: u32,
    pub isLoopback: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_NET_EVENT_CLASSIFY_DROP1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_NET_EVENT_CLASSIFY_DROP1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_NET_EVENT_CLASSIFY_DROP2 {
    pub filterId: u64,
    pub layerId: u16,
    pub reauthReason: u32,
    pub originalProfile: u32,
    pub currentProfile: u32,
    pub msFwpDirection: u32,
    pub isLoopback: super::super::Foundation::BOOL,
    pub vSwitchId: FWP_BYTE_BLOB,
    pub vSwitchSourcePort: u32,
    pub vSwitchDestinationPort: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_NET_EVENT_CLASSIFY_DROP2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_NET_EVENT_CLASSIFY_DROP2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_NET_EVENT_CLASSIFY_DROP_MAC0 {
    pub localMacAddr: FWP_BYTE_ARRAY6,
    pub remoteMacAddr: FWP_BYTE_ARRAY6,
    pub mediaType: u32,
    pub ifType: u32,
    pub etherType: u16,
    pub ndisPortNumber: u32,
    pub reserved: u32,
    pub vlanTag: u16,
    pub ifLuid: u64,
    pub filterId: u64,
    pub layerId: u16,
    pub reauthReason: u32,
    pub originalProfile: u32,
    pub currentProfile: u32,
    pub msFwpDirection: u32,
    pub isLoopback: super::super::Foundation::BOOL,
    pub vSwitchId: FWP_BYTE_BLOB,
    pub vSwitchSourcePort: u32,
    pub vSwitchDestinationPort: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_NET_EVENT_CLASSIFY_DROP_MAC0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_NET_EVENT_CLASSIFY_DROP_MAC0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_ENUM_TEMPLATE0 {
    pub startTime: super::super::Foundation::FILETIME,
    pub endTime: super::super::Foundation::FILETIME,
    pub numFilterConditions: u32,
    pub filterCondition: *mut FWPM_FILTER_CONDITION0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_ENUM_TEMPLATE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_APP_ID_SET: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_EFFECTIVE_NAME_SET: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_ENTERPRISE_ID_SET: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_IP_PROTOCOL_SET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_IP_VERSION_SET: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_LOCAL_ADDR_SET: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_LOCAL_PORT_SET: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_PACKAGE_ID_SET: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_POLICY_FLAGS_SET: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_REAUTH_REASON_SET: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_REMOTE_ADDR_SET: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_REMOTE_PORT_SET: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_SCOPE_ID_SET: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_FLAG_USER_ID_SET: u32 = 64u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_HEADER0 {
    pub timeStamp: super::super::Foundation::FILETIME,
    pub flags: u32,
    pub ipVersion: FWP_IP_VERSION,
    pub ipProtocol: u8,
    pub Anonymous1: FWPM_NET_EVENT_HEADER0_0,
    pub Anonymous2: FWPM_NET_EVENT_HEADER0_1,
    pub localPort: u16,
    pub remotePort: u16,
    pub scopeId: u32,
    pub appId: FWP_BYTE_BLOB,
    pub userId: *mut super::super::Security::SID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER0_0 {
    pub localAddrV4: u32,
    pub localAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER0_1 {
    pub remoteAddrV4: u32,
    pub remoteAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER0_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_HEADER1 {
    pub timeStamp: super::super::Foundation::FILETIME,
    pub flags: u32,
    pub ipVersion: FWP_IP_VERSION,
    pub ipProtocol: u8,
    pub Anonymous1: FWPM_NET_EVENT_HEADER1_0,
    pub Anonymous2: FWPM_NET_EVENT_HEADER1_1,
    pub localPort: u16,
    pub remotePort: u16,
    pub scopeId: u32,
    pub appId: FWP_BYTE_BLOB,
    pub userId: *mut super::super::Security::SID,
    pub Anonymous3: FWPM_NET_EVENT_HEADER1_2,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER1_0 {
    pub localAddrV4: u32,
    pub localAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER1_1 {
    pub remoteAddrV4: u32,
    pub remoteAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER1_2 {
    pub Anonymous: FWPM_NET_EVENT_HEADER1_2_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_HEADER1_2_0 {
    pub reserved1: FWP_AF,
    pub Anonymous: FWPM_NET_EVENT_HEADER1_2_0_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1_2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1_2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER1_2_0_0 {
    pub Anonymous: FWPM_NET_EVENT_HEADER1_2_0_0_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1_2_0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1_2_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_HEADER1_2_0_0_0 {
    pub reserved2: FWP_BYTE_ARRAY6,
    pub reserved3: FWP_BYTE_ARRAY6,
    pub reserved4: u32,
    pub reserved5: u32,
    pub reserved6: u16,
    pub reserved7: u32,
    pub reserved8: u32,
    pub reserved9: u16,
    pub reserved10: u64,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER1_2_0_0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER1_2_0_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_HEADER2 {
    pub timeStamp: super::super::Foundation::FILETIME,
    pub flags: u32,
    pub ipVersion: FWP_IP_VERSION,
    pub ipProtocol: u8,
    pub Anonymous1: FWPM_NET_EVENT_HEADER2_0,
    pub Anonymous2: FWPM_NET_EVENT_HEADER2_1,
    pub localPort: u16,
    pub remotePort: u16,
    pub scopeId: u32,
    pub appId: FWP_BYTE_BLOB,
    pub userId: *mut super::super::Security::SID,
    pub addressFamily: FWP_AF,
    pub packageSid: *mut super::super::Security::SID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER2_0 {
    pub localAddrV4: u32,
    pub localAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER2_1 {
    pub remoteAddrV4: u32,
    pub remoteAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER2_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_HEADER3 {
    pub timeStamp: super::super::Foundation::FILETIME,
    pub flags: u32,
    pub ipVersion: FWP_IP_VERSION,
    pub ipProtocol: u8,
    pub Anonymous1: FWPM_NET_EVENT_HEADER3_0,
    pub Anonymous2: FWPM_NET_EVENT_HEADER3_1,
    pub localPort: u16,
    pub remotePort: u16,
    pub scopeId: u32,
    pub appId: FWP_BYTE_BLOB,
    pub userId: *mut super::super::Security::SID,
    pub addressFamily: FWP_AF,
    pub packageSid: *mut super::super::Security::SID,
    pub enterpriseId: ::windows_sys::core::PWSTR,
    pub policyFlags: u64,
    pub effectiveName: FWP_BYTE_BLOB,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER3_0 {
    pub localAddrV4: u32,
    pub localAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER3_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER3_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_HEADER3_1 {
    pub remoteAddrV4: u32,
    pub remoteAddrV6: FWP_BYTE_ARRAY16,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_HEADER3_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_HEADER3_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IKEEXT_EM_FAILURE0 {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub flags: u32,
    pub emState: IKEEXT_EM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub emAuthMethod: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub endCertHash: [u8; 20],
    pub mmId: u64,
    pub qmFilterId: u64,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_EM_FAILURE0 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_EM_FAILURE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IKEEXT_EM_FAILURE1 {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub flags: u32,
    pub emState: IKEEXT_EM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub emAuthMethod: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub endCertHash: [u8; 20],
    pub mmId: u64,
    pub qmFilterId: u64,
    pub localPrincipalNameForAuth: ::windows_sys::core::PWSTR,
    pub remotePrincipalNameForAuth: ::windows_sys::core::PWSTR,
    pub numLocalPrincipalGroupSids: u32,
    pub localPrincipalGroupSids: *mut ::windows_sys::core::PWSTR,
    pub numRemotePrincipalGroupSids: u32,
    pub remotePrincipalGroupSids: *mut ::windows_sys::core::PWSTR,
    pub saTrafficType: IPSEC_TRAFFIC_TYPE,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_EM_FAILURE1 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_EM_FAILURE1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_IKEEXT_EM_FAILURE_FLAG_BENIGN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_IKEEXT_EM_FAILURE_FLAG_MULTIPLE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IKEEXT_MM_FAILURE0 {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub flags: u32,
    pub keyingModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub mmState: IKEEXT_MM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub mmAuthMethod: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub endCertHash: [u8; 20],
    pub mmId: u64,
    pub mmFilterId: u64,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_MM_FAILURE0 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_MM_FAILURE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IKEEXT_MM_FAILURE1 {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub flags: u32,
    pub keyingModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub mmState: IKEEXT_MM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub mmAuthMethod: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub endCertHash: [u8; 20],
    pub mmId: u64,
    pub mmFilterId: u64,
    pub localPrincipalNameForAuth: ::windows_sys::core::PWSTR,
    pub remotePrincipalNameForAuth: ::windows_sys::core::PWSTR,
    pub numLocalPrincipalGroupSids: u32,
    pub localPrincipalGroupSids: *mut ::windows_sys::core::PWSTR,
    pub numRemotePrincipalGroupSids: u32,
    pub remotePrincipalGroupSids: *mut ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_MM_FAILURE1 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_MM_FAILURE1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IKEEXT_MM_FAILURE2_ {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub flags: u32,
    pub keyingModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub mmState: IKEEXT_MM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub mmAuthMethod: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub endCertHash: [u8; 20],
    pub mmId: u64,
    pub mmFilterId: u64,
    pub localPrincipalNameForAuth: ::windows_sys::core::PWSTR,
    pub remotePrincipalNameForAuth: ::windows_sys::core::PWSTR,
    pub numLocalPrincipalGroupSids: u32,
    pub localPrincipalGroupSids: *mut ::windows_sys::core::PWSTR,
    pub numRemotePrincipalGroupSids: u32,
    pub remotePrincipalGroupSids: *mut ::windows_sys::core::PWSTR,
    pub providerContextKey: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_MM_FAILURE2_ {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_MM_FAILURE2_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_IKEEXT_MM_FAILURE_FLAG_BENIGN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_IKEEXT_MM_FAILURE_FLAG_MULTIPLE: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_IKEEXT_QM_FAILURE0 {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub keyingModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub qmState: IKEEXT_QM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub saTrafficType: IPSEC_TRAFFIC_TYPE,
    pub Anonymous1: FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_0,
    pub Anonymous2: FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_1,
    pub qmFilterId: u64,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_QM_FAILURE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_QM_FAILURE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_0 {
    pub localSubNet: FWP_CONDITION_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_1 {
    pub remoteSubNet: FWP_CONDITION_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_QM_FAILURE0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_IKEEXT_QM_FAILURE1_ {
    pub failureErrorCode: u32,
    pub failurePoint: IPSEC_FAILURE_POINT,
    pub keyingModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub qmState: IKEEXT_QM_SA_STATE,
    pub saRole: IKEEXT_SA_ROLE,
    pub saTrafficType: IPSEC_TRAFFIC_TYPE,
    pub Anonymous1: FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__0,
    pub Anonymous2: FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__1,
    pub qmFilterId: u64,
    pub mmSaLuid: u64,
    pub mmProviderContextKey: ::windows_sys::core::GUID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_QM_FAILURE1_ {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_QM_FAILURE1_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__0 {
    pub localSubNet: FWP_CONDITION_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__1 {
    pub remoteSubNet: FWP_CONDITION_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_IKEEXT_QM_FAILURE1__1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IPSEC_DOSP_DROP0 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: FWPM_NET_EVENT_IPSEC_DOSP_DROP0_0,
    pub Anonymous2: FWPM_NET_EVENT_IPSEC_DOSP_DROP0_1,
    pub failureStatus: i32,
    pub direction: FWP_DIRECTION,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IPSEC_DOSP_DROP0 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IPSEC_DOSP_DROP0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union FWPM_NET_EVENT_IPSEC_DOSP_DROP0_0 {
    pub publicHostV4Addr: u32,
    pub publicHostV6Addr: [u8; 16],
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IPSEC_DOSP_DROP0_0 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IPSEC_DOSP_DROP0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union FWPM_NET_EVENT_IPSEC_DOSP_DROP0_1 {
    pub internalHostV4Addr: u32,
    pub internalHostV6Addr: [u8; 16],
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IPSEC_DOSP_DROP0_1 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IPSEC_DOSP_DROP0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_IPSEC_KERNEL_DROP0 {
    pub failureStatus: i32,
    pub direction: FWP_DIRECTION,
    pub spi: u32,
    pub filterId: u64,
    pub layerId: u16,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_IPSEC_KERNEL_DROP0 {}
impl ::core::clone::Clone for FWPM_NET_EVENT_IPSEC_KERNEL_DROP0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_KEYWORD_CAPABILITY_ALLOW: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_KEYWORD_CAPABILITY_DROP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_KEYWORD_CLASSIFY_ALLOW: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_KEYWORD_INBOUND_BCAST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_KEYWORD_INBOUND_MCAST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_KEYWORD_PORT_SCANNING_DROP: u32 = 32u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_NET_EVENT_LPM_PACKET_ARRIVAL0_ {
    pub spi: u32,
}
impl ::core::marker::Copy for FWPM_NET_EVENT_LPM_PACKET_ARRIVAL0_ {}
impl ::core::clone::Clone for FWPM_NET_EVENT_LPM_PACKET_ARRIVAL0_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_NET_EVENT_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_NET_EVENT_ENUM_TEMPLATE0,
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_NET_EVENT_SUBSCRIPTION0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_NET_EVENT_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_NET_EVENT_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_IKEEXT_MM_FAILURE: FWPM_NET_EVENT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_IKEEXT_QM_FAILURE: FWPM_NET_EVENT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_IKEEXT_EM_FAILURE: FWPM_NET_EVENT_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_CLASSIFY_DROP: FWPM_NET_EVENT_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_IPSEC_KERNEL_DROP: FWPM_NET_EVENT_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_IPSEC_DOSP_DROP: FWPM_NET_EVENT_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_CLASSIFY_ALLOW: FWPM_NET_EVENT_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_CAPABILITY_DROP: FWPM_NET_EVENT_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_CAPABILITY_ALLOW: FWPM_NET_EVENT_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_CLASSIFY_DROP_MAC: FWPM_NET_EVENT_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_LPM_PACKET_ARRIVAL: FWPM_NET_EVENT_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_NET_EVENT_TYPE_MAX: FWPM_NET_EVENT_TYPE = 11i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER0 {
    pub providerKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerData: FWP_BYTE_BLOB,
    pub serviceName: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for FWPM_PROVIDER0 {}
impl ::core::clone::Clone for FWPM_PROVIDER0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER_CHANGE0 {
    pub changeType: FWPM_CHANGE_TYPE,
    pub providerKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_PROVIDER_CHANGE0 {}
impl ::core::clone::Clone for FWPM_PROVIDER_CHANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_PROVIDER_CHANGE_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, change: *const FWPM_PROVIDER_CHANGE0)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_PROVIDER_CONTEXT0 {
    pub providerContextKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub r#type: FWPM_PROVIDER_CONTEXT_TYPE,
    pub Anonymous: FWPM_PROVIDER_CONTEXT0_0,
    pub providerContextId: u64,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_PROVIDER_CONTEXT0_0 {
    pub keyingPolicy: *mut IPSEC_KEYING_POLICY0,
    pub ikeQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY0,
    pub ikeQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY0,
    pub authipQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY0,
    pub authipQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY0,
    pub ikeMmPolicy: *mut IKEEXT_POLICY0,
    pub authIpMmPolicy: *mut IKEEXT_POLICY0,
    pub dataBuffer: *mut FWP_BYTE_BLOB,
    pub classifyOptions: *mut FWPM_CLASSIFY_OPTIONS0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_PROVIDER_CONTEXT1 {
    pub providerContextKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub r#type: FWPM_PROVIDER_CONTEXT_TYPE,
    pub Anonymous: FWPM_PROVIDER_CONTEXT1_0,
    pub providerContextId: u64,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_PROVIDER_CONTEXT1_0 {
    pub keyingPolicy: *mut IPSEC_KEYING_POLICY0,
    pub ikeQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY1,
    pub ikeQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY1,
    pub authipQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY1,
    pub authipQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY1,
    pub ikeMmPolicy: *mut IKEEXT_POLICY1,
    pub authIpMmPolicy: *mut IKEEXT_POLICY1,
    pub dataBuffer: *mut FWP_BYTE_BLOB,
    pub classifyOptions: *mut FWPM_CLASSIFY_OPTIONS0,
    pub ikeV2QmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY1,
    pub ikeV2MmPolicy: *mut IKEEXT_POLICY1,
    pub idpOptions: *mut IPSEC_DOSP_OPTIONS0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_PROVIDER_CONTEXT2 {
    pub providerContextKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub r#type: FWPM_PROVIDER_CONTEXT_TYPE,
    pub Anonymous: FWPM_PROVIDER_CONTEXT2_0,
    pub providerContextId: u64,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_PROVIDER_CONTEXT2_0 {
    pub keyingPolicy: *mut IPSEC_KEYING_POLICY1,
    pub ikeQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY2,
    pub ikeQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY2,
    pub authipQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY2,
    pub authipQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY2,
    pub ikeMmPolicy: *mut IKEEXT_POLICY2,
    pub authIpMmPolicy: *mut IKEEXT_POLICY2,
    pub dataBuffer: *mut FWP_BYTE_BLOB,
    pub classifyOptions: *mut FWPM_CLASSIFY_OPTIONS0,
    pub ikeV2QmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY2,
    pub ikeV2QmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY2,
    pub ikeV2MmPolicy: *mut IKEEXT_POLICY2,
    pub idpOptions: *mut IPSEC_DOSP_OPTIONS0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_PROVIDER_CONTEXT3_ {
    pub providerContextKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub r#type: FWPM_PROVIDER_CONTEXT_TYPE,
    pub Anonymous: FWPM_PROVIDER_CONTEXT3__0,
    pub providerContextId: u64,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT3_ {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT3_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWPM_PROVIDER_CONTEXT3__0 {
    pub keyingPolicy: *mut IPSEC_KEYING_POLICY1,
    pub ikeQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY2,
    pub ikeQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY3_,
    pub authipQmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY2,
    pub authipQmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY3_,
    pub ikeMmPolicy: *mut IKEEXT_POLICY2,
    pub authIpMmPolicy: *mut IKEEXT_POLICY2,
    pub dataBuffer: *mut FWP_BYTE_BLOB,
    pub classifyOptions: *mut FWPM_CLASSIFY_OPTIONS0,
    pub ikeV2QmTunnelPolicy: *mut IPSEC_TUNNEL_POLICY3_,
    pub ikeV2QmTransportPolicy: *mut IPSEC_TRANSPORT_POLICY2,
    pub ikeV2MmPolicy: *mut IKEEXT_POLICY2,
    pub idpOptions: *mut IPSEC_DOSP_OPTIONS0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT3__0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT3__0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER_CONTEXT_CHANGE0 {
    pub changeType: FWPM_CHANGE_TYPE,
    pub providerContextKey: ::windows_sys::core::GUID,
    pub providerContextId: u64,
}
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT_CHANGE0 {}
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT_CHANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_PROVIDER_CONTEXT_CHANGE_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, change: *const FWPM_PROVIDER_CONTEXT_CHANGE0)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER_CONTEXT_ENUM_TEMPLATE0 {
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerContextType: FWPM_PROVIDER_CONTEXT_TYPE,
}
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_PROVIDER_CONTEXT_FLAG_DOWNLEVEL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_PROVIDER_CONTEXT_FLAG_PERSISTENT: u32 = 1u32;
pub const FWPM_PROVIDER_CONTEXT_SECURE_SOCKET_AUTHIP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2992547840, data2: 3330, data3: 18157, data4: [146, 189, 127, 168, 75, 183, 62, 157] };
pub const FWPM_PROVIDER_CONTEXT_SECURE_SOCKET_IPSEC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2351776068, data2: 63712, data3: 17088, data4: [148, 206, 124, 207, 198, 59, 47, 155] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER_CONTEXT_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_PROVIDER_CONTEXT_ENUM_TEMPLATE0,
    pub flags: FWPM_SUBSCRIPTION_FLAGS,
    pub sessionKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_PROVIDER_CONTEXT_SUBSCRIPTION0 {}
impl ::core::clone::Clone for FWPM_PROVIDER_CONTEXT_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_PROVIDER_CONTEXT_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_KEYING_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_IKE_QM_TRANSPORT_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_IKE_QM_TUNNEL_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_AUTHIP_QM_TRANSPORT_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_AUTHIP_QM_TUNNEL_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_IKE_MM_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_AUTHIP_MM_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_CLASSIFY_OPTIONS_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_GENERAL_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_IKEV2_QM_TUNNEL_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_IKEV2_MM_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_DOSP_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_IPSEC_IKEV2_QM_TRANSPORT_CONTEXT: FWPM_PROVIDER_CONTEXT_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_PROVIDER_CONTEXT_TYPE_MAX: FWPM_PROVIDER_CONTEXT_TYPE = 13i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER_ENUM_TEMPLATE0 {
    pub reserved: u64,
}
impl ::core::marker::Copy for FWPM_PROVIDER_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_PROVIDER_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_PROVIDER_FLAG_DISABLED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_PROVIDER_FLAG_PERSISTENT: u32 = 1u32;
pub const FWPM_PROVIDER_IKEEXT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 279810582, data2: 52446, data3: 17772, data4: [139, 22, 233, 240, 78, 96, 169, 11] };
pub const FWPM_PROVIDER_IPSEC_DOSP_CONFIG: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1013712297, data2: 49244, data3: 19385, data4: [131, 56, 35, 39, 129, 76, 232, 191] };
pub const FWPM_PROVIDER_MPSSVC_EDP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2835519223, data2: 18104, data3: 17495, data4: [143, 132, 176, 94, 5, 211, 198, 34] };
pub const FWPM_PROVIDER_MPSSVC_TENANT_RESTRICTIONS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3497103353, data2: 17626, data3: 20304, data4: [157, 194, 201, 99, 164, 36, 118, 19] };
pub const FWPM_PROVIDER_MPSSVC_WF: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3737917130, data2: 16179, data3: 17222, data4: [190, 30, 143, 180, 174, 15, 61, 98] };
pub const FWPM_PROVIDER_MPSSVC_WSH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1259681589, data2: 4169, data3: 17536, data4: [170, 180, 209, 185, 189, 192, 55, 16] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_PROVIDER_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_PROVIDER_ENUM_TEMPLATE0,
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_PROVIDER_SUBSCRIPTION0 {}
impl ::core::clone::Clone for FWPM_PROVIDER_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_PROVIDER_TCP_CHIMNEY_OFFLOAD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2305466782, data2: 39476, data3: 19403, data4: [174, 121, 190, 185, 18, 124, 132, 185] };
pub const FWPM_PROVIDER_TCP_TEMPLATES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1993329968, data2: 13204, data3: 17197, data4: [190, 211, 68, 26, 229, 14, 99, 195] };
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_SERVICE_STATE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SERVICE_STOPPED: FWPM_SERVICE_STATE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SERVICE_START_PENDING: FWPM_SERVICE_STATE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SERVICE_STOP_PENDING: FWPM_SERVICE_STATE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SERVICE_RUNNING: FWPM_SERVICE_STATE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SERVICE_STATE_MAX: FWPM_SERVICE_STATE = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWPM_SESSION0 {
    pub sessionKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub txnWaitTimeoutInMSec: u32,
    pub processId: u32,
    pub sid: *mut super::super::Security::SID,
    pub username: ::windows_sys::core::PWSTR,
    pub kernelMode: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWPM_SESSION0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWPM_SESSION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SESSION_ENUM_TEMPLATE0 {
    pub reserved: u64,
}
impl ::core::marker::Copy for FWPM_SESSION_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_SESSION_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SESSION_FLAG_DYNAMIC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SESSION_FLAG_RESERVED: u32 = 268435456u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_STATISTICS0 {
    pub numLayerStatistics: u32,
    pub layerStatistics: *mut FWPM_LAYER_STATISTICS0,
    pub inboundAllowedConnectionsV4: u32,
    pub inboundBlockedConnectionsV4: u32,
    pub outboundAllowedConnectionsV4: u32,
    pub outboundBlockedConnectionsV4: u32,
    pub inboundAllowedConnectionsV6: u32,
    pub inboundBlockedConnectionsV6: u32,
    pub outboundAllowedConnectionsV6: u32,
    pub outboundBlockedConnectionsV6: u32,
    pub inboundActiveConnectionsV4: u32,
    pub outboundActiveConnectionsV4: u32,
    pub inboundActiveConnectionsV6: u32,
    pub outboundActiveConnectionsV6: u32,
    pub reauthDirInbound: u64,
    pub reauthDirOutbound: u64,
    pub reauthFamilyV4: u64,
    pub reauthFamilyV6: u64,
    pub reauthProtoOther: u64,
    pub reauthProtoIPv4: u64,
    pub reauthProtoIPv6: u64,
    pub reauthProtoICMP: u64,
    pub reauthProtoICMP6: u64,
    pub reauthProtoUDP: u64,
    pub reauthProtoTCP: u64,
    pub reauthReasonPolicyChange: u64,
    pub reauthReasonNewArrivalInterface: u64,
    pub reauthReasonNewNextHopInterface: u64,
    pub reauthReasonProfileCrossing: u64,
    pub reauthReasonClassifyCompletion: u64,
    pub reauthReasonIPSecPropertiesChanged: u64,
    pub reauthReasonMidStreamInspection: u64,
    pub reauthReasonSocketPropertyChanged: u64,
    pub reauthReasonNewInboundMCastBCastPacket: u64,
    pub reauthReasonEDPPolicyChanged: u64,
    pub reauthReasonProxyHandleChanged: u64,
}
impl ::core::marker::Copy for FWPM_STATISTICS0 {}
impl ::core::clone::Clone for FWPM_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SUBLAYER0 {
    pub subLayerKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub providerKey: *mut ::windows_sys::core::GUID,
    pub providerData: FWP_BYTE_BLOB,
    pub weight: u16,
}
impl ::core::marker::Copy for FWPM_SUBLAYER0 {}
impl ::core::clone::Clone for FWPM_SUBLAYER0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SUBLAYER_CHANGE0 {
    pub changeType: FWPM_CHANGE_TYPE,
    pub subLayerKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_SUBLAYER_CHANGE0 {}
impl ::core::clone::Clone for FWPM_SUBLAYER_CHANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_SUBLAYER_CHANGE_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, change: *const FWPM_SUBLAYER_CHANGE0)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SUBLAYER_ENUM_TEMPLATE0 {
    pub providerKey: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_SUBLAYER_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for FWPM_SUBLAYER_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SUBLAYER_FLAG_PERSISTENT: u32 = 1u32;
pub const FWPM_SUBLAYER_INSPECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2272598497, data2: 59049, data3: 16805, data4: [129, 180, 140, 79, 17, 142, 74, 96] };
pub const FWPM_SUBLAYER_IPSEC_DOSP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3765884274, data2: 23869, data3: 18671, data4: [128, 43, 144, 158, 221, 176, 152, 189] };
pub const FWPM_SUBLAYER_IPSEC_FORWARD_OUTBOUND_TUNNEL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2768776819, data2: 36721, data3: 17753, data4: [138, 154, 16, 28, 234, 4, 239, 135] };
pub const FWPM_SUBLAYER_IPSEC_SECURITY_REALM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 933590785, data2: 22660, data3: 18788, data4: [146, 184, 62, 112, 70, 136, 176, 173] };
pub const FWPM_SUBLAYER_IPSEC_TUNNEL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2213714413, data2: 40948, data3: 18791, data4: [175, 244, 195, 9, 244, 218, 184, 39] };
pub const FWPM_SUBLAYER_LIPS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 460701902, data2: 65376, data3: 18193, data4: [167, 15, 180, 149, 140, 195, 178, 208] };
pub const FWPM_SUBLAYER_MPSSVC_EDP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 161775160, data2: 64151, data3: 18203, data4: [177, 35, 24, 188, 215, 230, 80, 113] };
pub const FWPM_SUBLAYER_MPSSVC_QUARANTINE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3016610881, data2: 44944, data3: 16826, data4: [167, 69, 124, 96, 8, 255, 35, 2] };
pub const FWPM_SUBLAYER_MPSSVC_TENANT_RESTRICTIONS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 516343777, data2: 64985, data3: 18314, data4: [181, 95, 255, 139, 161, 210, 193, 125] };
pub const FWPM_SUBLAYER_MPSSVC_WF: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3016610881, data2: 44944, data3: 16826, data4: [167, 69, 124, 96, 8, 255, 35, 1] };
pub const FWPM_SUBLAYER_MPSSVC_WSH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3016610881, data2: 44944, data3: 16826, data4: [167, 69, 124, 96, 8, 255, 35, 0] };
pub const FWPM_SUBLAYER_RPC_AUDIT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1972143348, data2: 64328, data3: 19945, data4: [154, 235, 62, 217, 85, 26, 177, 253] };
pub const FWPM_SUBLAYER_SECURE_SOCKET: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 363228695, data2: 16188, data3: 20347, data4: [170, 108, 129, 42, 166, 19, 221, 130] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SUBLAYER_SUBSCRIPTION0 {
    pub enumTemplate: *mut FWPM_SUBLAYER_ENUM_TEMPLATE0,
    pub flags: FWPM_SUBSCRIPTION_FLAGS,
    pub sessionKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_SUBLAYER_SUBSCRIPTION0 {}
impl ::core::clone::Clone for FWPM_SUBLAYER_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FWPM_SUBLAYER_TCP_CHIMNEY_OFFLOAD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 863373497, data2: 47061, data3: 19807, data4: [130, 249, 54, 24, 97, 139, 192, 88] };
pub const FWPM_SUBLAYER_TCP_TEMPLATES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 608312783, data2: 2757, data3: 19626, data4: [158, 20, 80, 246, 227, 99, 106, 240] };
pub const FWPM_SUBLAYER_TEREDO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3127499878, data2: 20854, data3: 18809, data4: [156, 137, 38, 167, 180, 106, 131, 39] };
pub const FWPM_SUBLAYER_UNIVERSAL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4005481475, data2: 52948, data3: 17280, data4: [129, 154, 39, 52, 57, 123, 43, 116] };
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_SUBSCRIPTION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SUBSCRIPTION_FLAG_NOTIFY_ON_ADD: FWPM_SUBSCRIPTION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SUBSCRIPTION_FLAG_NOTIFY_ON_DELETE: FWPM_SUBSCRIPTION_FLAGS = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SYSTEM_PORTS0 {
    pub numTypes: u32,
    pub types: *mut FWPM_SYSTEM_PORTS_BY_TYPE0,
}
impl ::core::marker::Copy for FWPM_SYSTEM_PORTS0 {}
impl ::core::clone::Clone for FWPM_SYSTEM_PORTS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_SYSTEM_PORTS_BY_TYPE0 {
    pub r#type: FWPM_SYSTEM_PORT_TYPE,
    pub numPorts: u32,
    pub ports: *mut u16,
}
impl ::core::marker::Copy for FWPM_SYSTEM_PORTS_BY_TYPE0 {}
impl ::core::clone::Clone for FWPM_SYSTEM_PORTS_BY_TYPE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_SYSTEM_PORTS_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, sysports: *const FWPM_SYSTEM_PORTS0)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_SYSTEM_PORT_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SYSTEM_PORT_RPC_EPMAP: FWPM_SYSTEM_PORT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SYSTEM_PORT_TEREDO: FWPM_SYSTEM_PORT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SYSTEM_PORT_IPHTTPS_IN: FWPM_SYSTEM_PORT_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SYSTEM_PORT_IPHTTPS_OUT: FWPM_SYSTEM_PORT_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_SYSTEM_PORT_TYPE_MAX: FWPM_SYSTEM_PORT_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_TUNNEL_FLAG_ENABLE_VIRTUAL_IF_TUNNELING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_TUNNEL_FLAG_POINT_TO_POINT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_TUNNEL_FLAG_RESERVED0: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_TXN_READ_ONLY: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_VSWITCH_EVENT0 {
    pub eventType: FWPM_VSWITCH_EVENT_TYPE,
    pub vSwitchId: ::windows_sys::core::PWSTR,
    pub Anonymous: FWPM_VSWITCH_EVENT0_0,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_VSWITCH_EVENT0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_VSWITCH_EVENT0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union FWPM_VSWITCH_EVENT0_0 {
    pub positionInfo: FWPM_VSWITCH_EVENT0_0_0,
    pub reorderInfo: FWPM_VSWITCH_EVENT0_0_1,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_VSWITCH_EVENT0_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_VSWITCH_EVENT0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_VSWITCH_EVENT0_0_0 {
    pub numvSwitchFilterExtensions: u32,
    pub vSwitchFilterExtensions: *mut ::windows_sys::core::PWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_VSWITCH_EVENT0_0_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_VSWITCH_EVENT0_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FWPM_VSWITCH_EVENT0_0_1 {
    pub inRequiredPosition: super::super::Foundation::BOOL,
    pub numvSwitchFilterExtensions: u32,
    pub vSwitchFilterExtensions: *mut ::windows_sys::core::PWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FWPM_VSWITCH_EVENT0_0_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FWPM_VSWITCH_EVENT0_0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type FWPM_VSWITCH_EVENT_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, vswitchevent: *const FWPM_VSWITCH_EVENT0) -> u32>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWPM_VSWITCH_EVENT_SUBSCRIPTION0 {
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for FWPM_VSWITCH_EVENT_SUBSCRIPTION0 {}
impl ::core::clone::Clone for FWPM_VSWITCH_EVENT_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWPM_VSWITCH_EVENT_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_VSWITCH_EVENT_FILTER_ADD_TO_INCOMPLETE_LAYER: FWPM_VSWITCH_EVENT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_VSWITCH_EVENT_FILTER_ENGINE_NOT_IN_REQUIRED_POSITION: FWPM_VSWITCH_EVENT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_VSWITCH_EVENT_ENABLED_FOR_INSPECTION: FWPM_VSWITCH_EVENT_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_VSWITCH_EVENT_DISABLED_FOR_INSPECTION: FWPM_VSWITCH_EVENT_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_VSWITCH_EVENT_FILTER_ENGINE_REORDER: FWPM_VSWITCH_EVENT_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_VSWITCH_EVENT_MAX: FWPM_VSWITCH_EVENT_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_WEIGHT_RANGE_IKE_EXEMPTIONS: u32 = 12u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPM_WEIGHT_RANGE_IPSEC: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_ALE_ENDPOINT_FLAG_IPSEC_SECURED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_CLASSIFY_OUT_FLAG_ABSORB: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_CLASSIFY_OUT_FLAG_ALE_FAST_CACHE_CHECK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_CLASSIFY_OUT_FLAG_ALE_FAST_CACHE_POSSIBLE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_CLASSIFY_OUT_FLAG_BUFFER_LIMIT_REACHED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_CLASSIFY_OUT_FLAG_NO_MORE_DATA: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_CLEAR_ACTION_RIGHT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_HAS_SECURITY_REALM_PROVIDER_CONTEXT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_IPSEC_NO_ACQUIRE_INITIATE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_OR_CONDITIONS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_PERMIT_IF_CALLOUT_UNREGISTERED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_RESERVED0: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_RESERVED1: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_FILTER_FLAG_SILENT_MODE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_ABSORB: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_CACHE_SAFE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_CONNECTION_FAILING_INDICATION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_ENFORCE_QUERY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_IS_LOCAL_ONLY_FLOW: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_IS_LOOSE_SOURCE_FLOW: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_MID_STREAM_INSPECTION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_RECLASSIFY: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_INCOMING_FLAG_RESERVED0: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_INCOMING_FLAG_IS_RAW_IPV4_FRAMING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_INCOMING_FLAG_IS_RAW_IPV6_FRAMING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_INCOMING_FLAG_RECLASSIFY_MULTI_DESTINATION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_ETHERNET_MAC_HEADER_SIZE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_RESERVED: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_VSWITCH_DESTINATION_PORT_ID: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_VSWITCH_PACKET_CONTEXT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_VSWITCH_SOURCE_NIC_INDEX: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_VSWITCH_SOURCE_PORT_ID: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_L2_METADATA_FIELD_WIFI_OPERATION_MODE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_ALE_CLASSIFY_REQUIRED: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_COMPARTMENT_ID: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_COMPLETION_HANDLE: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_DESTINATION_INTERFACE_INDEX: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_DESTINATION_PREFIX: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_DISCARD_REASON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_ETHER_FRAME_LENGTH: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_FLOW_HANDLE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_FORWARD_LAYER_INBOUND_PASS_THRU: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_FORWARD_LAYER_OUTBOUND_PASS_THRU: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_FRAGMENT_DATA: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_ICMP_ID_AND_SEQUENCE: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_IP_HEADER_SIZE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_LOCAL_REDIRECT_TARGET_PID: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_ORIGINAL_DESTINATION: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_PACKET_DIRECTION: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_PACKET_SYSTEM_CRITICAL: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_PARENT_ENDPOINT_HANDLE: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_PATH_MTU: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_PROCESS_ID: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_PROCESS_PATH: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_REDIRECT_RECORD_HANDLE: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_REMOTE_SCOPE_ID: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_RESERVED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_SOURCE_INTERFACE_INDEX: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_SUB_PROCESS_TAG: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_SYSTEM_FLAGS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_TOKEN: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_TRANSPORT_CONTROL_DATA: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_TRANSPORT_ENDPOINT_HANDLE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_TRANSPORT_HEADER_INCLUDE_HEADER: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_METADATA_FIELD_TRANSPORT_HEADER_SIZE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWPS_RIGHT_ACTION_WRITE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ACTION_FLAG_CALLOUT: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ACTION_FLAG_NON_TERMINATING: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ACTION_FLAG_TERMINATING: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ACTION_NONE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ACTION_NONE_NO_MATCH: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ACTRL_MATCH_FILTER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_AF = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_AF_INET: FWP_AF = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_AF_INET6: FWP_AF = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_AF_ETHER: FWP_AF = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_AF_NONE: FWP_AF = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_BYTEMAP_ARRAY64_SIZE: u32 = 8u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWP_BYTE_ARRAY16 {
    pub byteArray16: [u8; 16],
}
impl ::core::marker::Copy for FWP_BYTE_ARRAY16 {}
impl ::core::clone::Clone for FWP_BYTE_ARRAY16 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWP_BYTE_ARRAY6 {
    pub byteArray6: [u8; 6],
}
impl ::core::marker::Copy for FWP_BYTE_ARRAY6 {}
impl ::core::clone::Clone for FWP_BYTE_ARRAY6 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_BYTE_ARRAY6_SIZE: u32 = 6u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWP_BYTE_BLOB {
    pub size: u32,
    pub data: *mut u8,
}
impl ::core::marker::Copy for FWP_BYTE_BLOB {}
impl ::core::clone::Clone for FWP_BYTE_BLOB {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_L2_BATCH_CLASSIFY: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_MID_STREAM_INSPECTION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_OFFLOAD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_RECLASSIFY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_RSC: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_URO: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ALLOW_USO: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_CONDITIONAL_ON_FLOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_ENABLE_COMMIT_ADD_NOTIFY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_RESERVED1: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CALLOUT_FLAG_RESERVED2: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_CLASSIFY_OPTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_MULTICAST_STATE: FWP_CLASSIFY_OPTION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_LOOSE_SOURCE_MAPPING: FWP_CLASSIFY_OPTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_UNICAST_LIFETIME: FWP_CLASSIFY_OPTION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_MCAST_BCAST_LIFETIME: FWP_CLASSIFY_OPTION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_SECURE_SOCKET_SECURITY_FLAGS: FWP_CLASSIFY_OPTION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_SECURE_SOCKET_AUTHIP_MM_POLICY_KEY: FWP_CLASSIFY_OPTION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_SECURE_SOCKET_AUTHIP_QM_POLICY_KEY: FWP_CLASSIFY_OPTION_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_LOCAL_ONLY_MAPPING: FWP_CLASSIFY_OPTION_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CLASSIFY_OPTION_MAX: FWP_CLASSIFY_OPTION_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_APPCONTAINER_LOOPBACK: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_AUTH_FW: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_CONNECTION_REDIRECTED: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_FRAGMENT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_FRAGMENT_GROUP: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_HONORING_POLICY_AUTHORIZE: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_IMPLICIT_BIND: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_INBOUND_PASS_THRU: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_IPSEC_NATT_RECLASSIFY: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_IPSEC_SECURED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_LOOPBACK: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_NAME_APP_SPECIFIED: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_NON_APPCONTAINER_LOOPBACK: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_OUTBOUND_PASS_THRU: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_PROMISCUOUS: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_PROXY_CONNECTION: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_RAW_ENDPOINT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_REASSEMBLED: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_REAUTHORIZE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_RECLASSIFY: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_RESERVED: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_IS_WILDCARD_BIND: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_FLAG_REQUIRES_ALE_CLASSIFY: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IF_CONNECTOR_PRESENT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_IP_FRAGMENT_GROUP: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_MALFORMED_PACKET: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_MOBILE_BROADBAND: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_NATIVE_ETHERNET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_VM2VM: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_WIFI: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_L2_IS_WIFI_DIRECT_DATA: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_CHECK_OFFLOAD: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_CLASSIFY_COMPLETION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_EDP_POLICY_CHANGED: u32 = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_IPSEC_PROPERTIES_CHANGED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_MID_STREAM_INSPECTION: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_NEW_ARRIVAL_INTERFACE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_NEW_INBOUND_MCAST_BCAST_PACKET: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_NEW_NEXTHOP_INTERFACE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_POLICY_CHANGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_PROFILE_CROSSING: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_PROXY_HANDLE_CHANGED: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_REAUTHORIZE_REASON_SOCKET_PROPERTY_CHANGED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_SOCKET_PROPERTY_FLAG_ALLOW_EDGE_TRAFFIC: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_SOCKET_PROPERTY_FLAG_DENY_EDGE_TRAFFIC: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_CONDITION_SOCKET_PROPERTY_FLAG_IS_SYSTEM_PORT_RPC: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWP_CONDITION_VALUE0 {
    pub r#type: FWP_DATA_TYPE,
    pub Anonymous: FWP_CONDITION_VALUE0_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWP_CONDITION_VALUE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWP_CONDITION_VALUE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWP_CONDITION_VALUE0_0 {
    pub uint8: u8,
    pub uint16: u16,
    pub uint32: u32,
    pub uint64: *mut u64,
    pub int8: i8,
    pub int16: i16,
    pub int32: i32,
    pub int64: *mut i64,
    pub float32: f32,
    pub double64: *mut f64,
    pub byteArray16: *mut FWP_BYTE_ARRAY16,
    pub byteBlob: *mut FWP_BYTE_BLOB,
    pub sid: *mut super::super::Security::SID,
    pub sd: *mut FWP_BYTE_BLOB,
    pub tokenInformation: *mut FWP_TOKEN_INFORMATION,
    pub tokenAccessInformation: *mut FWP_BYTE_BLOB,
    pub unicodeString: ::windows_sys::core::PWSTR,
    pub byteArray6: *mut FWP_BYTE_ARRAY6,
    pub v4AddrMask: *mut FWP_V4_ADDR_AND_MASK,
    pub v6AddrMask: *mut FWP_V6_ADDR_AND_MASK,
    pub rangeValue: *mut FWP_RANGE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWP_CONDITION_VALUE0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWP_CONDITION_VALUE0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_DATA_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_EMPTY: FWP_DATA_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_UINT8: FWP_DATA_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_UINT16: FWP_DATA_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_UINT32: FWP_DATA_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_UINT64: FWP_DATA_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_INT8: FWP_DATA_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_INT16: FWP_DATA_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_INT32: FWP_DATA_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_INT64: FWP_DATA_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FLOAT: FWP_DATA_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_DOUBLE: FWP_DATA_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_BYTE_ARRAY16_TYPE: FWP_DATA_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_BYTE_BLOB_TYPE: FWP_DATA_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_SID: FWP_DATA_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_SECURITY_DESCRIPTOR_TYPE: FWP_DATA_TYPE = 14i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_TOKEN_INFORMATION_TYPE: FWP_DATA_TYPE = 15i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_TOKEN_ACCESS_INFORMATION_TYPE: FWP_DATA_TYPE = 16i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_UNICODE_STRING_TYPE: FWP_DATA_TYPE = 17i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_BYTE_ARRAY6_TYPE: FWP_DATA_TYPE = 18i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_SINGLE_DATA_TYPE_MAX: FWP_DATA_TYPE = 255i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_V4_ADDR_MASK: FWP_DATA_TYPE = 256i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_V6_ADDR_MASK: FWP_DATA_TYPE = 257i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_RANGE_TYPE: FWP_DATA_TYPE = 258i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_DATA_TYPE_MAX: FWP_DATA_TYPE = 259i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_DIRECTION_OUTBOUND: FWP_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_DIRECTION_INBOUND: FWP_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_DIRECTION_MAX: FWP_DIRECTION = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_ETHER_ENCAP_METHOD = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ETHER_ENCAP_METHOD_ETHER_V2: FWP_ETHER_ENCAP_METHOD = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ETHER_ENCAP_METHOD_SNAP: FWP_ETHER_ENCAP_METHOD = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_ETHER_ENCAP_METHOD_SNAP_W_OUI_ZERO: FWP_ETHER_ENCAP_METHOD = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FLAG_BEST_TERMINATING_MATCH: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FLAG_BOOTTIME_ONLY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FLAG_INCLUDE_BOOTTIME: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FLAG_INCLUDE_DISABLED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FLAG_RESERVED1: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FLAG_SORTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_FILTER_ENUM_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_FULLY_CONTAINED: FWP_FILTER_ENUM_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_OVERLAPPING: FWP_FILTER_ENUM_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_FILTER_ENUM_TYPE_MAX: FWP_FILTER_ENUM_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_IP_VERSION = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_IP_VERSION_V4: FWP_IP_VERSION = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_IP_VERSION_V6: FWP_IP_VERSION = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_IP_VERSION_NONE: FWP_IP_VERSION = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_IP_VERSION_MAX: FWP_IP_VERSION = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_MATCH_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_EQUAL: FWP_MATCH_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_GREATER: FWP_MATCH_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_LESS: FWP_MATCH_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_GREATER_OR_EQUAL: FWP_MATCH_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_LESS_OR_EQUAL: FWP_MATCH_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_RANGE: FWP_MATCH_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_FLAGS_ALL_SET: FWP_MATCH_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_FLAGS_ANY_SET: FWP_MATCH_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_FLAGS_NONE_SET: FWP_MATCH_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_EQUAL_CASE_INSENSITIVE: FWP_MATCH_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_NOT_EQUAL: FWP_MATCH_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_PREFIX: FWP_MATCH_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_NOT_PREFIX: FWP_MATCH_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_MATCH_TYPE_MAX: FWP_MATCH_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_ALLOW_GLOBAL_MULTICAST_STATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_ALLOW_MULTICAST_STATE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_DENY_MULTICAST_STATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_DISABLE_LOCAL_ONLY_MAPPING: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_DISABLE_LOOSE_SOURCE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_ENABLE_LOCAL_ONLY_MAPPING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_OPTION_VALUE_ENABLE_LOOSE_SOURCE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWP_RANGE0 {
    pub valueLow: FWP_VALUE0,
    pub valueHigh: FWP_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWP_RANGE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWP_RANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWP_TOKEN_INFORMATION {
    pub sidCount: u32,
    pub sids: *mut super::super::Security::SID_AND_ATTRIBUTES,
    pub restrictedSidCount: u32,
    pub restrictedSids: *mut super::super::Security::SID_AND_ATTRIBUTES,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWP_TOKEN_INFORMATION {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWP_TOKEN_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWP_V4_ADDR_AND_MASK {
    pub addr: u32,
    pub mask: u32,
}
impl ::core::marker::Copy for FWP_V4_ADDR_AND_MASK {}
impl ::core::clone::Clone for FWP_V4_ADDR_AND_MASK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct FWP_V6_ADDR_AND_MASK {
    pub addr: [u8; 16],
    pub prefixLength: u8,
}
impl ::core::marker::Copy for FWP_V6_ADDR_AND_MASK {}
impl ::core::clone::Clone for FWP_V6_ADDR_AND_MASK {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_V6_ADDR_SIZE: u32 = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct FWP_VALUE0 {
    pub r#type: FWP_DATA_TYPE,
    pub Anonymous: FWP_VALUE0_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWP_VALUE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWP_VALUE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union FWP_VALUE0_0 {
    pub uint8: u8,
    pub uint16: u16,
    pub uint32: u32,
    pub uint64: *mut u64,
    pub int8: i8,
    pub int16: i16,
    pub int32: i32,
    pub int64: *mut i64,
    pub float32: f32,
    pub double64: *mut f64,
    pub byteArray16: *mut FWP_BYTE_ARRAY16,
    pub byteBlob: *mut FWP_BYTE_BLOB,
    pub sid: *mut super::super::Security::SID,
    pub sd: *mut FWP_BYTE_BLOB,
    pub tokenInformation: *mut FWP_TOKEN_INFORMATION,
    pub tokenAccessInformation: *mut FWP_BYTE_BLOB,
    pub unicodeString: ::windows_sys::core::PWSTR,
    pub byteArray6: *mut FWP_BYTE_ARRAY6,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for FWP_VALUE0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for FWP_VALUE0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type FWP_VSWITCH_NETWORK_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_VSWITCH_NETWORK_TYPE_UNKNOWN: FWP_VSWITCH_NETWORK_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_VSWITCH_NETWORK_TYPE_PRIVATE: FWP_VSWITCH_NETWORK_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_VSWITCH_NETWORK_TYPE_INTERNAL: FWP_VSWITCH_NETWORK_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const FWP_VSWITCH_NETWORK_TYPE_EXTERNAL: FWP_VSWITCH_NETWORK_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type ICMP4_TIME_EXCEED_CODE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_TIME_EXCEED_TRANSIT: ICMP4_TIME_EXCEED_CODE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_TIME_EXCEED_REASSEMBLY: ICMP4_TIME_EXCEED_CODE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type ICMP4_UNREACH_CODE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_NET: ICMP4_UNREACH_CODE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_HOST: ICMP4_UNREACH_CODE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_PROTOCOL: ICMP4_UNREACH_CODE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_PORT: ICMP4_UNREACH_CODE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_FRAG_NEEDED: ICMP4_UNREACH_CODE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_SOURCEROUTE_FAILED: ICMP4_UNREACH_CODE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_NET_UNKNOWN: ICMP4_UNREACH_CODE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_HOST_UNKNOWN: ICMP4_UNREACH_CODE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_ISOLATED: ICMP4_UNREACH_CODE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_NET_ADMIN: ICMP4_UNREACH_CODE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_HOST_ADMIN: ICMP4_UNREACH_CODE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_NET_TOS: ICMP4_UNREACH_CODE = 11i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_HOST_TOS: ICMP4_UNREACH_CODE = 12i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP4_UNREACH_ADMIN: ICMP4_UNREACH_CODE = 13i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_DST_UNREACH_ADDR: u32 = 3u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_DST_UNREACH_ADMIN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_DST_UNREACH_BEYONDSCOPE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_DST_UNREACH_NOPORT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_DST_UNREACH_NOROUTE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_PARAMPROB_HEADER: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_PARAMPROB_NEXTHEADER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_PARAMPROB_OPTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_TIME_EXCEED_REASSEMBLY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMP6_TIME_EXCEED_TRANSIT: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ICMPV4_ADDRESS_MASK_MESSAGE {
    pub Header: ICMP_MESSAGE,
    pub AddressMask: u32,
}
impl ::core::marker::Copy for ICMPV4_ADDRESS_MASK_MESSAGE {}
impl ::core::clone::Clone for ICMPV4_ADDRESS_MASK_MESSAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMPV4_INVALID_PREFERENCE_LEVEL: u32 = 2147483648u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct ICMPV4_ROUTER_ADVERT_ENTRY {
    pub RouterAdvertAddr: super::super::Networking::WinSock::IN_ADDR,
    pub PreferenceLevel: i32,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for ICMPV4_ROUTER_ADVERT_ENTRY {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for ICMPV4_ROUTER_ADVERT_ENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ICMPV4_ROUTER_ADVERT_HEADER {
    pub RaHeader: ICMP_MESSAGE,
}
impl ::core::marker::Copy for ICMPV4_ROUTER_ADVERT_HEADER {}
impl ::core::clone::Clone for ICMPV4_ROUTER_ADVERT_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ICMPV4_ROUTER_SOLICIT {
    pub RsHeader: ICMP_MESSAGE,
}
impl ::core::marker::Copy for ICMPV4_ROUTER_SOLICIT {}
impl ::core::clone::Clone for ICMPV4_ROUTER_SOLICIT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ICMPV4_TIMESTAMP_MESSAGE {
    pub Header: ICMP_MESSAGE,
    pub OriginateTimestamp: u32,
    pub ReceiveTimestamp: u32,
    pub TransmitTimestamp: u32,
}
impl ::core::marker::Copy for ICMPV4_TIMESTAMP_MESSAGE {}
impl ::core::clone::Clone for ICMPV4_TIMESTAMP_MESSAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ICMPV6_ECHO_REQUEST_FLAG_REVERSE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ICMP_HEADER {
    pub Type: u8,
    pub Code: u8,
    pub Checksum: u16,
}
impl ::core::marker::Copy for ICMP_HEADER {}
impl ::core::clone::Clone for ICMP_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct ICMP_MESSAGE {
    pub Header: ICMP_HEADER,
    pub Data: ICMP_MESSAGE_0,
}
impl ::core::marker::Copy for ICMP_MESSAGE {}
impl ::core::clone::Clone for ICMP_MESSAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union ICMP_MESSAGE_0 {
    pub Data32: [u32; 1],
    pub Data16: [u16; 2],
    pub Data8: [u8; 4],
}
impl ::core::marker::Copy for ICMP_MESSAGE_0 {}
impl ::core::clone::Clone for ICMP_MESSAGE_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IGMPV3_QUERY_HEADER {
    pub Type: u8,
    pub Anonymous1: IGMPV3_QUERY_HEADER_0,
    pub Checksum: u16,
    pub MulticastAddress: super::super::Networking::WinSock::IN_ADDR,
    pub _bitfield: u8,
    pub Anonymous2: IGMPV3_QUERY_HEADER_1,
    pub SourceCount: u16,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMPV3_QUERY_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMPV3_QUERY_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IGMPV3_QUERY_HEADER_0 {
    pub MaxRespCode: u8,
    pub Anonymous: IGMPV3_QUERY_HEADER_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMPV3_QUERY_HEADER_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMPV3_QUERY_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IGMPV3_QUERY_HEADER_0_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMPV3_QUERY_HEADER_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMPV3_QUERY_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IGMPV3_QUERY_HEADER_1 {
    pub QueriersQueryInterfaceCode: u8,
    pub Anonymous: IGMPV3_QUERY_HEADER_1_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMPV3_QUERY_HEADER_1 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMPV3_QUERY_HEADER_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IGMPV3_QUERY_HEADER_1_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMPV3_QUERY_HEADER_1_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMPV3_QUERY_HEADER_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IGMPV3_REPORT_HEADER {
    pub Type: u8,
    pub Reserved: u8,
    pub Checksum: u16,
    pub Reserved2: u16,
    pub RecordCount: u16,
}
impl ::core::marker::Copy for IGMPV3_REPORT_HEADER {}
impl ::core::clone::Clone for IGMPV3_REPORT_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IGMPV3_REPORT_RECORD_HEADER {
    pub Type: u8,
    pub AuxillaryDataLength: u8,
    pub SourceCount: u16,
    pub MulticastAddress: super::super::Networking::WinSock::IN_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMPV3_REPORT_RECORD_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMPV3_REPORT_RECORD_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IGMP_HEADER {
    pub Anonymous1: IGMP_HEADER_0,
    pub Anonymous2: IGMP_HEADER_1,
    pub Checksum: u16,
    pub MulticastAddress: super::super::Networking::WinSock::IN_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMP_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMP_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IGMP_HEADER_0 {
    pub Anonymous: IGMP_HEADER_0_0,
    pub VersionType: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMP_HEADER_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMP_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IGMP_HEADER_0_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMP_HEADER_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMP_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IGMP_HEADER_1 {
    pub Reserved: u8,
    pub MaxRespTime: u8,
    pub Code: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IGMP_HEADER_1 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IGMP_HEADER_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_LEAVE_GROUP_TYPE: u32 = 23u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IGMP_MAX_RESP_CODE_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_MAX_RESP_CODE_TYPE_NORMAL: IGMP_MAX_RESP_CODE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_MAX_RESP_CODE_TYPE_FLOAT: IGMP_MAX_RESP_CODE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_QUERY_TYPE: u32 = 17u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_VERSION1_REPORT_TYPE: u32 = 18u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_VERSION2_REPORT_TYPE: u32 = 22u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IGMP_VERSION3_REPORT_TYPE: u32 = 34u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_IMPERSONATION_NONE: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_IMPERSONATION_SOCKET_PRINCIPAL: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_IMPERSONATION_MAX: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_AUTHENTICATION_METHOD0 {
    pub authenticationMethodType: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub Anonymous: IKEEXT_AUTHENTICATION_METHOD0_0,
}
impl ::core::marker::Copy for IKEEXT_AUTHENTICATION_METHOD0 {}
impl ::core::clone::Clone for IKEEXT_AUTHENTICATION_METHOD0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_AUTHENTICATION_METHOD0_0 {
    pub presharedKeyAuthentication: IKEEXT_PRESHARED_KEY_AUTHENTICATION0,
    pub certificateAuthentication: IKEEXT_CERTIFICATE_AUTHENTICATION0,
    pub kerberosAuthentication: IKEEXT_KERBEROS_AUTHENTICATION0,
    pub ntlmV2Authentication: IKEEXT_NTLM_V2_AUTHENTICATION0,
    pub sslAuthentication: IKEEXT_CERTIFICATE_AUTHENTICATION0,
    pub cgaAuthentication: IKEEXT_IPV6_CGA_AUTHENTICATION0,
}
impl ::core::marker::Copy for IKEEXT_AUTHENTICATION_METHOD0_0 {}
impl ::core::clone::Clone for IKEEXT_AUTHENTICATION_METHOD0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_AUTHENTICATION_METHOD1 {
    pub authenticationMethodType: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub Anonymous: IKEEXT_AUTHENTICATION_METHOD1_0,
}
impl ::core::marker::Copy for IKEEXT_AUTHENTICATION_METHOD1 {}
impl ::core::clone::Clone for IKEEXT_AUTHENTICATION_METHOD1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_AUTHENTICATION_METHOD1_0 {
    pub presharedKeyAuthentication: IKEEXT_PRESHARED_KEY_AUTHENTICATION1,
    pub certificateAuthentication: IKEEXT_CERTIFICATE_AUTHENTICATION1,
    pub kerberosAuthentication: IKEEXT_KERBEROS_AUTHENTICATION0,
    pub ntlmV2Authentication: IKEEXT_NTLM_V2_AUTHENTICATION0,
    pub sslAuthentication: IKEEXT_CERTIFICATE_AUTHENTICATION1,
    pub cgaAuthentication: IKEEXT_IPV6_CGA_AUTHENTICATION0,
    pub eapAuthentication: IKEEXT_EAP_AUTHENTICATION0,
}
impl ::core::marker::Copy for IKEEXT_AUTHENTICATION_METHOD1_0 {}
impl ::core::clone::Clone for IKEEXT_AUTHENTICATION_METHOD1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_AUTHENTICATION_METHOD2 {
    pub authenticationMethodType: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub Anonymous: IKEEXT_AUTHENTICATION_METHOD2_0,
}
impl ::core::marker::Copy for IKEEXT_AUTHENTICATION_METHOD2 {}
impl ::core::clone::Clone for IKEEXT_AUTHENTICATION_METHOD2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_AUTHENTICATION_METHOD2_0 {
    pub presharedKeyAuthentication: IKEEXT_PRESHARED_KEY_AUTHENTICATION1,
    pub certificateAuthentication: IKEEXT_CERTIFICATE_AUTHENTICATION2,
    pub kerberosAuthentication: IKEEXT_KERBEROS_AUTHENTICATION1,
    pub reservedAuthentication: IKEEXT_RESERVED_AUTHENTICATION0,
    pub ntlmV2Authentication: IKEEXT_NTLM_V2_AUTHENTICATION0,
    pub sslAuthentication: IKEEXT_CERTIFICATE_AUTHENTICATION2,
    pub cgaAuthentication: IKEEXT_IPV6_CGA_AUTHENTICATION0,
    pub eapAuthentication: IKEEXT_EAP_AUTHENTICATION0,
}
impl ::core::marker::Copy for IKEEXT_AUTHENTICATION_METHOD2_0 {}
impl ::core::clone::Clone for IKEEXT_AUTHENTICATION_METHOD2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_AUTHENTICATION_METHOD_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_PRESHARED_KEY: IKEEXT_AUTHENTICATION_METHOD_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERTIFICATE: IKEEXT_AUTHENTICATION_METHOD_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KERBEROS: IKEEXT_AUTHENTICATION_METHOD_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_ANONYMOUS: IKEEXT_AUTHENTICATION_METHOD_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_SSL: IKEEXT_AUTHENTICATION_METHOD_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_NTLM_V2: IKEEXT_AUTHENTICATION_METHOD_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_IPV6_CGA: IKEEXT_AUTHENTICATION_METHOD_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERTIFICATE_ECDSA_P256: IKEEXT_AUTHENTICATION_METHOD_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERTIFICATE_ECDSA_P384: IKEEXT_AUTHENTICATION_METHOD_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_SSL_ECDSA_P256: IKEEXT_AUTHENTICATION_METHOD_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_SSL_ECDSA_P384: IKEEXT_AUTHENTICATION_METHOD_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EAP: IKEEXT_AUTHENTICATION_METHOD_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_RESERVED: IKEEXT_AUTHENTICATION_METHOD_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_AUTHENTICATION_METHOD_TYPE_MAX: IKEEXT_AUTHENTICATION_METHOD_TYPE = 13i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION0 {
    pub inboundConfigType: IKEEXT_CERT_CONFIG_TYPE,
    pub Anonymous1: IKEEXT_CERTIFICATE_AUTHENTICATION0_0,
    pub outboundConfigType: IKEEXT_CERT_CONFIG_TYPE,
    pub Anonymous2: IKEEXT_CERTIFICATE_AUTHENTICATION0_1,
    pub flags: IKEEXT_CERT_AUTH,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CERTIFICATE_AUTHENTICATION0_0 {
    pub Anonymous: IKEEXT_CERTIFICATE_AUTHENTICATION0_0_0,
    pub inboundEnterpriseStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
    pub inboundTrustedRootStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION0_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION0_0_0 {
    pub inboundRootArraySize: u32,
    pub inboundRootArray: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION0_0_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION0_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CERTIFICATE_AUTHENTICATION0_1 {
    pub Anonymous: IKEEXT_CERTIFICATE_AUTHENTICATION0_1_0,
    pub outboundEnterpriseStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
    pub outboundTrustedRootStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION0_1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION0_1_0 {
    pub outboundRootArraySize: u32,
    pub outboundRootArray: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION0_1_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION0_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION1 {
    pub inboundConfigType: IKEEXT_CERT_CONFIG_TYPE,
    pub Anonymous1: IKEEXT_CERTIFICATE_AUTHENTICATION1_0,
    pub outboundConfigType: IKEEXT_CERT_CONFIG_TYPE,
    pub Anonymous2: IKEEXT_CERTIFICATE_AUTHENTICATION1_1,
    pub flags: IKEEXT_CERT_AUTH,
    pub localCertLocationUrl: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CERTIFICATE_AUTHENTICATION1_0 {
    pub Anonymous: IKEEXT_CERTIFICATE_AUTHENTICATION1_0_0,
    pub inboundEnterpriseStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
    pub inboundTrustedRootStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION1_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION1_0_0 {
    pub inboundRootArraySize: u32,
    pub inboundRootArray: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION1_0_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION1_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CERTIFICATE_AUTHENTICATION1_1 {
    pub Anonymous: IKEEXT_CERTIFICATE_AUTHENTICATION1_1_0,
    pub outboundEnterpriseStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
    pub outboundTrustedRootStoreConfig: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION1_1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION1_1_0 {
    pub outboundRootArraySize: u32,
    pub outboundRootArray: *mut IKEEXT_CERT_ROOT_CONFIG0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION1_1_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION1_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2 {
    pub inboundConfigType: IKEEXT_CERT_CONFIG_TYPE,
    pub Anonymous1: IKEEXT_CERTIFICATE_AUTHENTICATION2_0,
    pub outboundConfigType: IKEEXT_CERT_CONFIG_TYPE,
    pub Anonymous2: IKEEXT_CERTIFICATE_AUTHENTICATION2_1,
    pub flags: IKEEXT_CERT_AUTH,
    pub localCertLocationUrl: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CERTIFICATE_AUTHENTICATION2_0 {
    pub Anonymous1: IKEEXT_CERTIFICATE_AUTHENTICATION2_0_0,
    pub Anonymous2: IKEEXT_CERTIFICATE_AUTHENTICATION2_0_1,
    pub Anonymous3: IKEEXT_CERTIFICATE_AUTHENTICATION2_0_2,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2_0_0 {
    pub inboundRootArraySize: u32,
    pub inboundRootCriteria: *mut IKEEXT_CERTIFICATE_CRITERIA0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_0_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2_0_1 {
    pub inboundEnterpriseStoreArraySize: u32,
    pub inboundEnterpriseStoreCriteria: *mut IKEEXT_CERTIFICATE_CRITERIA0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_0_1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2_0_2 {
    pub inboundRootStoreArraySize: u32,
    pub inboundTrustedRootStoreCriteria: *mut IKEEXT_CERTIFICATE_CRITERIA0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_0_2 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_0_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CERTIFICATE_AUTHENTICATION2_1 {
    pub Anonymous1: IKEEXT_CERTIFICATE_AUTHENTICATION2_1_0,
    pub Anonymous2: IKEEXT_CERTIFICATE_AUTHENTICATION2_1_1,
    pub Anonymous3: IKEEXT_CERTIFICATE_AUTHENTICATION2_1_2,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2_1_0 {
    pub outboundRootArraySize: u32,
    pub outboundRootCriteria: *mut IKEEXT_CERTIFICATE_CRITERIA0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_1_0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2_1_1 {
    pub outboundEnterpriseStoreArraySize: u32,
    pub outboundEnterpriseStoreCriteria: *mut IKEEXT_CERTIFICATE_CRITERIA0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_1_1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_AUTHENTICATION2_1_2 {
    pub outboundRootStoreArraySize: u32,
    pub outboundTrustedRootStoreCriteria: *mut IKEEXT_CERTIFICATE_CRITERIA0,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_AUTHENTICATION2_1_2 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_AUTHENTICATION2_1_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_CREDENTIAL0 {
    pub subjectName: FWP_BYTE_BLOB,
    pub certHash: FWP_BYTE_BLOB,
    pub flags: u32,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_CREDENTIAL0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_CREDENTIAL0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_CREDENTIAL1 {
    pub subjectName: FWP_BYTE_BLOB,
    pub certHash: FWP_BYTE_BLOB,
    pub flags: u32,
    pub certificate: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_CREDENTIAL1 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_CREDENTIAL1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERTIFICATE_CRITERIA0 {
    pub certData: FWP_BYTE_BLOB,
    pub certHash: FWP_BYTE_BLOB,
    pub eku: *mut IKEEXT_CERT_EKUS0,
    pub name: *mut IKEEXT_CERT_NAME0,
    pub flags: u32,
}
impl ::core::marker::Copy for IKEEXT_CERTIFICATE_CRITERIA0 {}
impl ::core::clone::Clone for IKEEXT_CERTIFICATE_CRITERIA0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_CERT_AUTH = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_FLAG_SSL_ONE_WAY: IKEEXT_CERT_AUTH = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_ENABLE_CRL_CHECK_STRONG: IKEEXT_CERT_AUTH = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_DISABLE_SSL_CERT_VALIDATION: IKEEXT_CERT_AUTH = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_ALLOW_HTTP_CERT_LOOKUP: IKEEXT_CERT_AUTH = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_URL_CONTAINS_BUNDLE: IKEEXT_CERT_AUTH = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_FLAG_DISABLE_CRL_CHECK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_AUTH_FLAG_DISABLE_REQUEST_PAYLOAD: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_CERT_CONFIG_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CONFIG_EXPLICIT_TRUST_LIST: IKEEXT_CERT_CONFIG_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CONFIG_ENTERPRISE_STORE: IKEEXT_CERT_CONFIG_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CONFIG_TRUSTED_ROOT_STORE: IKEEXT_CERT_CONFIG_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CONFIG_UNSPECIFIED: IKEEXT_CERT_CONFIG_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CONFIG_TYPE_MAX: IKEEXT_CERT_CONFIG_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CREDENTIAL_FLAG_NAP_CERT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_CERT_CRITERIA_NAME_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_DNS: IKEEXT_CERT_CRITERIA_NAME_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_UPN: IKEEXT_CERT_CRITERIA_NAME_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_RFC822: IKEEXT_CERT_CRITERIA_NAME_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_CN: IKEEXT_CERT_CRITERIA_NAME_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_OU: IKEEXT_CERT_CRITERIA_NAME_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_O: IKEEXT_CERT_CRITERIA_NAME_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_DC: IKEEXT_CERT_CRITERIA_NAME_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_CRITERIA_NAME_TYPE_MAX: IKEEXT_CERT_CRITERIA_NAME_TYPE = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERT_EKUS0 {
    pub numEku: u32,
    pub eku: *mut ::windows_sys::core::PSTR,
}
impl ::core::marker::Copy for IKEEXT_CERT_EKUS0 {}
impl ::core::clone::Clone for IKEEXT_CERT_EKUS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_CERT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_ENABLE_ACCOUNT_MAPPING: IKEEXT_CERT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_DISABLE_REQUEST_PAYLOAD: IKEEXT_CERT_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_USE_NAP_CERTIFICATE: IKEEXT_CERT_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_INTERMEDIATE_CA: IKEEXT_CERT_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_IGNORE_INIT_CERT_MAP_FAILURE: IKEEXT_CERT_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_PREFER_NAP_CERTIFICATE_OUTBOUND: IKEEXT_CERT_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_SELECT_NAP_CERTIFICATE: IKEEXT_CERT_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_VERIFY_NAP_CERTIFICATE: IKEEXT_CERT_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_FLAG_FOLLOW_RENEWAL_CERTIFICATE: IKEEXT_CERT_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CERT_HASH_LEN: u32 = 20u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERT_NAME0 {
    pub nameType: IKEEXT_CERT_CRITERIA_NAME_TYPE,
    pub certName: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for IKEEXT_CERT_NAME0 {}
impl ::core::clone::Clone for IKEEXT_CERT_NAME0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CERT_ROOT_CONFIG0 {
    pub certData: FWP_BYTE_BLOB,
    pub flags: IKEEXT_CERT_FLAGS,
}
impl ::core::marker::Copy for IKEEXT_CERT_ROOT_CONFIG0 {}
impl ::core::clone::Clone for IKEEXT_CERT_ROOT_CONFIG0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CIPHER_ALGORITHM0 {
    pub algoIdentifier: IKEEXT_CIPHER_TYPE,
    pub keyLen: u32,
    pub rounds: u32,
}
impl ::core::marker::Copy for IKEEXT_CIPHER_ALGORITHM0 {}
impl ::core::clone::Clone for IKEEXT_CIPHER_ALGORITHM0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_CIPHER_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_DES: IKEEXT_CIPHER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_3DES: IKEEXT_CIPHER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_AES_128: IKEEXT_CIPHER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_AES_192: IKEEXT_CIPHER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_AES_256: IKEEXT_CIPHER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_AES_GCM_128_16ICV: IKEEXT_CIPHER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_AES_GCM_256_16ICV: IKEEXT_CIPHER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_CIPHER_TYPE_MAX: IKEEXT_CIPHER_TYPE = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_COMMON_STATISTICS0 {
    pub v4Statistics: IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS0,
    pub v6Statistics: IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS0,
    pub totalPacketsReceived: u32,
    pub totalInvalidPacketsReceived: u32,
    pub currentQueuedWorkitems: u32,
}
impl ::core::marker::Copy for IKEEXT_COMMON_STATISTICS0 {}
impl ::core::clone::Clone for IKEEXT_COMMON_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_COMMON_STATISTICS1 {
    pub v4Statistics: IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS1,
    pub v6Statistics: IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS1,
    pub totalPacketsReceived: u32,
    pub totalInvalidPacketsReceived: u32,
    pub currentQueuedWorkitems: u32,
}
impl ::core::marker::Copy for IKEEXT_COMMON_STATISTICS1 {}
impl ::core::clone::Clone for IKEEXT_COMMON_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_COOKIE_PAIR0 {
    pub initiator: u64,
    pub responder: u64,
}
impl ::core::marker::Copy for IKEEXT_COOKIE_PAIR0 {}
impl ::core::clone::Clone for IKEEXT_COOKIE_PAIR0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIAL0 {
    pub authenticationMethodType: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub impersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
    pub Anonymous: IKEEXT_CREDENTIAL0_0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL0 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CREDENTIAL0_0 {
    pub presharedKey: *mut IKEEXT_PRESHARED_KEY_AUTHENTICATION0,
    pub certificate: *mut IKEEXT_CERTIFICATE_CREDENTIAL0,
    pub name: *mut IKEEXT_NAME_CREDENTIAL0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL0_0 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIAL1 {
    pub authenticationMethodType: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub impersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
    pub Anonymous: IKEEXT_CREDENTIAL1_0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL1 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CREDENTIAL1_0 {
    pub presharedKey: *mut IKEEXT_PRESHARED_KEY_AUTHENTICATION1,
    pub certificate: *mut IKEEXT_CERTIFICATE_CREDENTIAL1,
    pub name: *mut IKEEXT_NAME_CREDENTIAL0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL1_0 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIAL2 {
    pub authenticationMethodType: IKEEXT_AUTHENTICATION_METHOD_TYPE,
    pub impersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
    pub Anonymous: IKEEXT_CREDENTIAL2_0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL2 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_CREDENTIAL2_0 {
    pub presharedKey: *mut IKEEXT_PRESHARED_KEY_AUTHENTICATION1,
    pub certificate: *mut IKEEXT_CERTIFICATE_CREDENTIAL1,
    pub name: *mut IKEEXT_NAME_CREDENTIAL0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL2_0 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIALS0 {
    pub numCredentials: u32,
    pub credentials: *mut IKEEXT_CREDENTIAL_PAIR0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIALS0 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIALS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIALS1 {
    pub numCredentials: u32,
    pub credentials: *mut IKEEXT_CREDENTIAL_PAIR1,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIALS1 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIALS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIALS2 {
    pub numCredentials: u32,
    pub credentials: *mut IKEEXT_CREDENTIAL_PAIR2,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIALS2 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIALS2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIAL_PAIR0 {
    pub localCredentials: IKEEXT_CREDENTIAL0,
    pub peerCredentials: IKEEXT_CREDENTIAL0,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL_PAIR0 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL_PAIR0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIAL_PAIR1 {
    pub localCredentials: IKEEXT_CREDENTIAL1,
    pub peerCredentials: IKEEXT_CREDENTIAL1,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL_PAIR1 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL_PAIR1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_CREDENTIAL_PAIR2 {
    pub localCredentials: IKEEXT_CREDENTIAL2,
    pub peerCredentials: IKEEXT_CREDENTIAL2,
}
impl ::core::marker::Copy for IKEEXT_CREDENTIAL_PAIR2 {}
impl ::core::clone::Clone for IKEEXT_CREDENTIAL_PAIR2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_DH_GROUP = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_NONE: IKEEXT_DH_GROUP = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_1: IKEEXT_DH_GROUP = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_2: IKEEXT_DH_GROUP = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_14: IKEEXT_DH_GROUP = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_2048: IKEEXT_DH_GROUP = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_ECP_256: IKEEXT_DH_GROUP = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_ECP_384: IKEEXT_DH_GROUP = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_24: IKEEXT_DH_GROUP = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_DH_GROUP_MAX: IKEEXT_DH_GROUP = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_EAP_AUTHENTICATION0 {
    pub flags: IKEEXT_EAP_AUTHENTICATION_FLAGS,
}
impl ::core::marker::Copy for IKEEXT_EAP_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_EAP_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_EAP_AUTHENTICATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EAP_FLAG_LOCAL_AUTH_ONLY: IKEEXT_EAP_AUTHENTICATION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EAP_FLAG_REMOTE_AUTH_ONLY: IKEEXT_EAP_AUTHENTICATION_FLAGS = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_EM_POLICY0 {
    pub numAuthenticationMethods: u32,
    pub authenticationMethods: *mut IKEEXT_AUTHENTICATION_METHOD0,
    pub initiatorImpersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
}
impl ::core::marker::Copy for IKEEXT_EM_POLICY0 {}
impl ::core::clone::Clone for IKEEXT_EM_POLICY0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_EM_POLICY1 {
    pub numAuthenticationMethods: u32,
    pub authenticationMethods: *mut IKEEXT_AUTHENTICATION_METHOD1,
    pub initiatorImpersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
}
impl ::core::marker::Copy for IKEEXT_EM_POLICY1 {}
impl ::core::clone::Clone for IKEEXT_EM_POLICY1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_EM_POLICY2 {
    pub numAuthenticationMethods: u32,
    pub authenticationMethods: *mut IKEEXT_AUTHENTICATION_METHOD2,
    pub initiatorImpersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
}
impl ::core::marker::Copy for IKEEXT_EM_POLICY2 {}
impl ::core::clone::Clone for IKEEXT_EM_POLICY2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_EM_SA_STATE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_NONE: IKEEXT_EM_SA_STATE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_SENT_ATTS: IKEEXT_EM_SA_STATE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_SSPI_SENT: IKEEXT_EM_SA_STATE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_AUTH_COMPLETE: IKEEXT_EM_SA_STATE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_FINAL: IKEEXT_EM_SA_STATE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_COMPLETE: IKEEXT_EM_SA_STATE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_EM_SA_STATE_MAX: IKEEXT_EM_SA_STATE = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_INTEGRITY_ALGORITHM0 {
    pub algoIdentifier: IKEEXT_INTEGRITY_TYPE,
}
impl ::core::marker::Copy for IKEEXT_INTEGRITY_ALGORITHM0 {}
impl ::core::clone::Clone for IKEEXT_INTEGRITY_ALGORITHM0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_INTEGRITY_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_INTEGRITY_MD5: IKEEXT_INTEGRITY_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_INTEGRITY_SHA1: IKEEXT_INTEGRITY_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_INTEGRITY_SHA_256: IKEEXT_INTEGRITY_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_INTEGRITY_SHA_384: IKEEXT_INTEGRITY_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_INTEGRITY_TYPE_MAX: IKEEXT_INTEGRITY_TYPE = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_IPV6_CGA_AUTHENTICATION0 {
    pub keyContainerName: ::windows_sys::core::PWSTR,
    pub cspName: ::windows_sys::core::PWSTR,
    pub cspType: u32,
    pub cgaModifier: FWP_BYTE_ARRAY16,
    pub cgaCollisionCount: u8,
}
impl ::core::marker::Copy for IKEEXT_IPV6_CGA_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_IPV6_CGA_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS0 {
    pub totalSocketReceiveFailures: u32,
    pub totalSocketSendFailures: u32,
}
impl ::core::marker::Copy for IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS0 {}
impl ::core::clone::Clone for IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS1 {
    pub totalSocketReceiveFailures: u32,
    pub totalSocketSendFailures: u32,
}
impl ::core::marker::Copy for IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS1 {}
impl ::core::clone::Clone for IKEEXT_IP_VERSION_SPECIFIC_COMMON_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS0 {
    pub currentActiveMainModes: u32,
    pub totalMainModesStarted: u32,
    pub totalSuccessfulMainModes: u32,
    pub totalFailedMainModes: u32,
    pub totalResponderMainModes: u32,
    pub currentNewResponderMainModes: u32,
    pub currentActiveQuickModes: u32,
    pub totalQuickModesStarted: u32,
    pub totalSuccessfulQuickModes: u32,
    pub totalFailedQuickModes: u32,
    pub totalAcquires: u32,
    pub totalReinitAcquires: u32,
    pub currentActiveExtendedModes: u32,
    pub totalExtendedModesStarted: u32,
    pub totalSuccessfulExtendedModes: u32,
    pub totalFailedExtendedModes: u32,
    pub totalImpersonationExtendedModes: u32,
    pub totalImpersonationMainModes: u32,
}
impl ::core::marker::Copy for IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS0 {}
impl ::core::clone::Clone for IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS1 {
    pub currentActiveMainModes: u32,
    pub totalMainModesStarted: u32,
    pub totalSuccessfulMainModes: u32,
    pub totalFailedMainModes: u32,
    pub totalResponderMainModes: u32,
    pub currentNewResponderMainModes: u32,
    pub currentActiveQuickModes: u32,
    pub totalQuickModesStarted: u32,
    pub totalSuccessfulQuickModes: u32,
    pub totalFailedQuickModes: u32,
    pub totalAcquires: u32,
    pub totalReinitAcquires: u32,
    pub currentActiveExtendedModes: u32,
    pub totalExtendedModesStarted: u32,
    pub totalSuccessfulExtendedModes: u32,
    pub totalFailedExtendedModes: u32,
    pub totalImpersonationExtendedModes: u32,
    pub totalImpersonationMainModes: u32,
}
impl ::core::marker::Copy for IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS1 {}
impl ::core::clone::Clone for IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_KERBEROS_AUTHENTICATION0 {
    pub flags: IKEEXT_KERBEROS_AUTHENTICATION_FLAGS,
}
impl ::core::marker::Copy for IKEEXT_KERBEROS_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_KERBEROS_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_KERBEROS_AUTHENTICATION1 {
    pub flags: IKEEXT_KERBEROS_AUTHENTICATION_FLAGS,
    pub proxyServer: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for IKEEXT_KERBEROS_AUTHENTICATION1 {}
impl ::core::clone::Clone for IKEEXT_KERBEROS_AUTHENTICATION1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_KERBEROS_AUTHENTICATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KERB_AUTH_DISABLE_INITIATOR_TOKEN_GENERATION: IKEEXT_KERBEROS_AUTHENTICATION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KERB_AUTH_DONT_ACCEPT_EXPLICIT_CREDENTIALS: IKEEXT_KERBEROS_AUTHENTICATION_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KERB_AUTH_FORCE_PROXY_ON_INITIATOR: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_KEYMODULE_STATISTICS0 {
    pub v4Statistics: IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS0,
    pub v6Statistics: IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS0,
    pub errorFrequencyTable: [u32; 97],
    pub mainModeNegotiationTime: u32,
    pub quickModeNegotiationTime: u32,
    pub extendedModeNegotiationTime: u32,
}
impl ::core::marker::Copy for IKEEXT_KEYMODULE_STATISTICS0 {}
impl ::core::clone::Clone for IKEEXT_KEYMODULE_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_KEYMODULE_STATISTICS1 {
    pub v4Statistics: IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS1,
    pub v6Statistics: IKEEXT_IP_VERSION_SPECIFIC_KEYMODULE_STATISTICS1,
    pub errorFrequencyTable: [u32; 97],
    pub mainModeNegotiationTime: u32,
    pub quickModeNegotiationTime: u32,
    pub extendedModeNegotiationTime: u32,
}
impl ::core::marker::Copy for IKEEXT_KEYMODULE_STATISTICS1 {}
impl ::core::clone::Clone for IKEEXT_KEYMODULE_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_KEY_MODULE_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KEY_MODULE_IKE: IKEEXT_KEY_MODULE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KEY_MODULE_AUTHIP: IKEEXT_KEY_MODULE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KEY_MODULE_IKEV2: IKEEXT_KEY_MODULE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_KEY_MODULE_MAX: IKEEXT_KEY_MODULE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_MM_SA_STATE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_NONE: IKEEXT_MM_SA_STATE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_SA_SENT: IKEEXT_MM_SA_STATE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_SSPI_SENT: IKEEXT_MM_SA_STATE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_FINAL: IKEEXT_MM_SA_STATE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_FINAL_SENT: IKEEXT_MM_SA_STATE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_COMPLETE: IKEEXT_MM_SA_STATE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_MM_SA_STATE_MAX: IKEEXT_MM_SA_STATE = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_NAME_CREDENTIAL0 {
    pub principalName: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for IKEEXT_NAME_CREDENTIAL0 {}
impl ::core::clone::Clone for IKEEXT_NAME_CREDENTIAL0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_NTLM_V2_AUTHENTICATION0 {
    pub flags: u32,
}
impl ::core::marker::Copy for IKEEXT_NTLM_V2_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_NTLM_V2_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_NTLM_V2_AUTH_DONT_ACCEPT_EXPLICIT_CREDENTIALS: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_POLICY0 {
    pub softExpirationTime: u32,
    pub numAuthenticationMethods: u32,
    pub authenticationMethods: *mut IKEEXT_AUTHENTICATION_METHOD0,
    pub initiatorImpersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
    pub numIkeProposals: u32,
    pub ikeProposals: *mut IKEEXT_PROPOSAL0,
    pub flags: IKEEXT_POLICY_FLAG,
    pub maxDynamicFilters: u32,
}
impl ::core::marker::Copy for IKEEXT_POLICY0 {}
impl ::core::clone::Clone for IKEEXT_POLICY0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_POLICY1 {
    pub softExpirationTime: u32,
    pub numAuthenticationMethods: u32,
    pub authenticationMethods: *mut IKEEXT_AUTHENTICATION_METHOD1,
    pub initiatorImpersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
    pub numIkeProposals: u32,
    pub ikeProposals: *mut IKEEXT_PROPOSAL0,
    pub flags: IKEEXT_POLICY_FLAG,
    pub maxDynamicFilters: u32,
    pub retransmitDurationSecs: u32,
}
impl ::core::marker::Copy for IKEEXT_POLICY1 {}
impl ::core::clone::Clone for IKEEXT_POLICY1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_POLICY2 {
    pub softExpirationTime: u32,
    pub numAuthenticationMethods: u32,
    pub authenticationMethods: *mut IKEEXT_AUTHENTICATION_METHOD2,
    pub initiatorImpersonationType: IKEEXT_AUTHENTICATION_IMPERSONATION_TYPE,
    pub numIkeProposals: u32,
    pub ikeProposals: *mut IKEEXT_PROPOSAL0,
    pub flags: IKEEXT_POLICY_FLAG,
    pub maxDynamicFilters: u32,
    pub retransmitDurationSecs: u32,
}
impl ::core::marker::Copy for IKEEXT_POLICY2 {}
impl ::core::clone::Clone for IKEEXT_POLICY2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_ENABLE_IKEV2_FRAGMENTATION: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_POLICY_FLAG = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_DISABLE_DIAGNOSTICS: IKEEXT_POLICY_FLAG = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_NO_MACHINE_LUID_VERIFY: IKEEXT_POLICY_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_NO_IMPERSONATION_LUID_VERIFY: IKEEXT_POLICY_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_ENABLE_OPTIONAL_DH: IKEEXT_POLICY_FLAG = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_IMS_VPN: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_MOBIKE_NOT_SUPPORTED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_FLAG_SITE_TO_SITE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_POLICY_SUPPORT_LOW_POWER_MODE: u32 = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_PRESHARED_KEY_AUTHENTICATION0 {
    pub presharedKey: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IKEEXT_PRESHARED_KEY_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_PRESHARED_KEY_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_PRESHARED_KEY_AUTHENTICATION1 {
    pub presharedKey: FWP_BYTE_BLOB,
    pub flags: IKEEXT_PRESHARED_KEY_AUTHENTICATION_FLAGS,
}
impl ::core::marker::Copy for IKEEXT_PRESHARED_KEY_AUTHENTICATION1 {}
impl ::core::clone::Clone for IKEEXT_PRESHARED_KEY_AUTHENTICATION1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_PRESHARED_KEY_AUTHENTICATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_PSK_FLAG_LOCAL_AUTH_ONLY: IKEEXT_PRESHARED_KEY_AUTHENTICATION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_PSK_FLAG_REMOTE_AUTH_ONLY: IKEEXT_PRESHARED_KEY_AUTHENTICATION_FLAGS = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_PROPOSAL0 {
    pub cipherAlgorithm: IKEEXT_CIPHER_ALGORITHM0,
    pub integrityAlgorithm: IKEEXT_INTEGRITY_ALGORITHM0,
    pub maxLifetimeSeconds: u32,
    pub dhGroup: IKEEXT_DH_GROUP,
    pub quickModeLimit: u32,
}
impl ::core::marker::Copy for IKEEXT_PROPOSAL0 {}
impl ::core::clone::Clone for IKEEXT_PROPOSAL0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_QM_SA_STATE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_QM_SA_STATE_NONE: IKEEXT_QM_SA_STATE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_QM_SA_STATE_INITIAL: IKEEXT_QM_SA_STATE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_QM_SA_STATE_FINAL: IKEEXT_QM_SA_STATE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_QM_SA_STATE_COMPLETE: IKEEXT_QM_SA_STATE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_QM_SA_STATE_MAX: IKEEXT_QM_SA_STATE = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_RESERVED_AUTHENTICATION0 {
    pub flags: IKEEXT_RESERVED_AUTHENTICATION_FLAGS,
}
impl ::core::marker::Copy for IKEEXT_RESERVED_AUTHENTICATION0 {}
impl ::core::clone::Clone for IKEEXT_RESERVED_AUTHENTICATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_RESERVED_AUTHENTICATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_RESERVED_AUTH_DISABLE_INITIATOR_TOKEN_GENERATION: IKEEXT_RESERVED_AUTHENTICATION_FLAGS = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_SA_DETAILS0 {
    pub saId: u64,
    pub keyModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IKEEXT_SA_DETAILS0_0,
    pub ikeTraffic: IKEEXT_TRAFFIC0,
    pub ikeProposal: IKEEXT_PROPOSAL0,
    pub cookiePair: IKEEXT_COOKIE_PAIR0,
    pub ikeCredentials: IKEEXT_CREDENTIALS0,
    pub ikePolicyKey: ::windows_sys::core::GUID,
    pub virtualIfTunnelId: u64,
}
impl ::core::marker::Copy for IKEEXT_SA_DETAILS0 {}
impl ::core::clone::Clone for IKEEXT_SA_DETAILS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_SA_DETAILS0_0 {
    pub v4UdpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
impl ::core::marker::Copy for IKEEXT_SA_DETAILS0_0 {}
impl ::core::clone::Clone for IKEEXT_SA_DETAILS0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_SA_DETAILS1 {
    pub saId: u64,
    pub keyModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IKEEXT_SA_DETAILS1_0,
    pub ikeTraffic: IKEEXT_TRAFFIC0,
    pub ikeProposal: IKEEXT_PROPOSAL0,
    pub cookiePair: IKEEXT_COOKIE_PAIR0,
    pub ikeCredentials: IKEEXT_CREDENTIALS1,
    pub ikePolicyKey: ::windows_sys::core::GUID,
    pub virtualIfTunnelId: u64,
    pub correlationKey: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IKEEXT_SA_DETAILS1 {}
impl ::core::clone::Clone for IKEEXT_SA_DETAILS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_SA_DETAILS1_0 {
    pub v4UdpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
impl ::core::marker::Copy for IKEEXT_SA_DETAILS1_0 {}
impl ::core::clone::Clone for IKEEXT_SA_DETAILS1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_SA_DETAILS2 {
    pub saId: u64,
    pub keyModuleType: IKEEXT_KEY_MODULE_TYPE,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IKEEXT_SA_DETAILS2_0,
    pub ikeTraffic: IKEEXT_TRAFFIC0,
    pub ikeProposal: IKEEXT_PROPOSAL0,
    pub cookiePair: IKEEXT_COOKIE_PAIR0,
    pub ikeCredentials: IKEEXT_CREDENTIALS2,
    pub ikePolicyKey: ::windows_sys::core::GUID,
    pub virtualIfTunnelId: u64,
    pub correlationKey: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IKEEXT_SA_DETAILS2 {}
impl ::core::clone::Clone for IKEEXT_SA_DETAILS2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_SA_DETAILS2_0 {
    pub v4UdpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
impl ::core::marker::Copy for IKEEXT_SA_DETAILS2_0 {}
impl ::core::clone::Clone for IKEEXT_SA_DETAILS2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IKEEXT_SA_ENUM_TEMPLATE0 {
    pub localSubNet: FWP_CONDITION_VALUE0,
    pub remoteSubNet: FWP_CONDITION_VALUE0,
    pub localMainModeCertHash: FWP_BYTE_BLOB,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IKEEXT_SA_ENUM_TEMPLATE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IKEEXT_SA_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IKEEXT_SA_ROLE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_SA_ROLE_INITIATOR: IKEEXT_SA_ROLE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_SA_ROLE_RESPONDER: IKEEXT_SA_ROLE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IKEEXT_SA_ROLE_MAX: IKEEXT_SA_ROLE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_STATISTICS0 {
    pub ikeStatistics: IKEEXT_KEYMODULE_STATISTICS0,
    pub authipStatistics: IKEEXT_KEYMODULE_STATISTICS0,
    pub commonStatistics: IKEEXT_COMMON_STATISTICS0,
}
impl ::core::marker::Copy for IKEEXT_STATISTICS0 {}
impl ::core::clone::Clone for IKEEXT_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_STATISTICS1 {
    pub ikeStatistics: IKEEXT_KEYMODULE_STATISTICS1,
    pub authipStatistics: IKEEXT_KEYMODULE_STATISTICS1,
    pub ikeV2Statistics: IKEEXT_KEYMODULE_STATISTICS1,
    pub commonStatistics: IKEEXT_COMMON_STATISTICS1,
}
impl ::core::marker::Copy for IKEEXT_STATISTICS1 {}
impl ::core::clone::Clone for IKEEXT_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IKEEXT_TRAFFIC0 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IKEEXT_TRAFFIC0_0,
    pub Anonymous2: IKEEXT_TRAFFIC0_1,
    pub authIpFilterId: u64,
}
impl ::core::marker::Copy for IKEEXT_TRAFFIC0 {}
impl ::core::clone::Clone for IKEEXT_TRAFFIC0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_TRAFFIC0_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
impl ::core::marker::Copy for IKEEXT_TRAFFIC0_0 {}
impl ::core::clone::Clone for IKEEXT_TRAFFIC0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IKEEXT_TRAFFIC0_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
impl ::core::marker::Copy for IKEEXT_TRAFFIC0_1 {}
impl ::core::clone::Clone for IKEEXT_TRAFFIC0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IN6_EMBEDDEDV4_BITS_IN_BYTE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IN6_EMBEDDEDV4_UOCTET_POSITION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP4_OFF_MASK: u32 = 65311u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6F_MORE_FRAG: u32 = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6F_OFF_MASK: u32 = 63743u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6F_RESERVED_MASK: u32 = 1536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_MUTABLE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_TYPE_DISCARD: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_TYPE_FORCEICMP: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_TYPE_ICMP: u32 = 192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_TYPE_SKIP: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_ADDRESS_INFO0 {
    pub numV4Addresses: u32,
    pub v4Addresses: *mut u32,
    pub numV6Addresses: u32,
    pub v6Addresses: *mut FWP_BYTE_ARRAY16,
}
impl ::core::marker::Copy for IPSEC_ADDRESS_INFO0 {}
impl ::core::clone::Clone for IPSEC_ADDRESS_INFO0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AGGREGATE_DROP_PACKET_STATISTICS0 {
    pub invalidSpisOnInbound: u32,
    pub decryptionFailuresOnInbound: u32,
    pub authenticationFailuresOnInbound: u32,
    pub udpEspValidationFailuresOnInbound: u32,
    pub replayCheckFailuresOnInbound: u32,
    pub invalidClearTextInbound: u32,
    pub saNotInitializedOnInbound: u32,
    pub receiveOverIncorrectSaInbound: u32,
    pub secureReceivesNotMatchingFilters: u32,
}
impl ::core::marker::Copy for IPSEC_AGGREGATE_DROP_PACKET_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_AGGREGATE_DROP_PACKET_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AGGREGATE_DROP_PACKET_STATISTICS1 {
    pub invalidSpisOnInbound: u32,
    pub decryptionFailuresOnInbound: u32,
    pub authenticationFailuresOnInbound: u32,
    pub udpEspValidationFailuresOnInbound: u32,
    pub replayCheckFailuresOnInbound: u32,
    pub invalidClearTextInbound: u32,
    pub saNotInitializedOnInbound: u32,
    pub receiveOverIncorrectSaInbound: u32,
    pub secureReceivesNotMatchingFilters: u32,
    pub totalDropPacketsInbound: u32,
}
impl ::core::marker::Copy for IPSEC_AGGREGATE_DROP_PACKET_STATISTICS1 {}
impl ::core::clone::Clone for IPSEC_AGGREGATE_DROP_PACKET_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AGGREGATE_SA_STATISTICS0 {
    pub activeSas: u32,
    pub pendingSaNegotiations: u32,
    pub totalSasAdded: u32,
    pub totalSasDeleted: u32,
    pub successfulRekeys: u32,
    pub activeTunnels: u32,
    pub offloadedSas: u32,
}
impl ::core::marker::Copy for IPSEC_AGGREGATE_SA_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_AGGREGATE_SA_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AH_DROP_PACKET_STATISTICS0 {
    pub invalidSpisOnInbound: u32,
    pub authenticationFailuresOnInbound: u32,
    pub replayCheckFailuresOnInbound: u32,
    pub saNotInitializedOnInbound: u32,
}
impl ::core::marker::Copy for IPSEC_AH_DROP_PACKET_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_AH_DROP_PACKET_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AUTH_AND_CIPHER_TRANSFORM0 {
    pub authTransform: IPSEC_AUTH_TRANSFORM0,
    pub cipherTransform: IPSEC_CIPHER_TRANSFORM0,
}
impl ::core::marker::Copy for IPSEC_AUTH_AND_CIPHER_TRANSFORM0 {}
impl ::core::clone::Clone for IPSEC_AUTH_AND_CIPHER_TRANSFORM0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_GCM_AES_128: u32 = 3u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_GCM_AES_192: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_GCM_AES_256: u32 = 5u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_HMAC_MD5_96: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_HMAC_SHA_1_96: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_HMAC_SHA_256_128: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_CONFIG_MAX: u32 = 6u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AUTH_TRANSFORM0 {
    pub authTransformId: IPSEC_AUTH_TRANSFORM_ID0,
    pub cryptoModuleId: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for IPSEC_AUTH_TRANSFORM0 {}
impl ::core::clone::Clone for IPSEC_AUTH_TRANSFORM0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_AUTH_TRANSFORM_ID0 {
    pub authType: IPSEC_AUTH_TYPE,
    pub authConfig: u8,
}
impl ::core::marker::Copy for IPSEC_AUTH_TRANSFORM_ID0 {}
impl ::core::clone::Clone for IPSEC_AUTH_TRANSFORM_ID0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_AUTH_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_MD5: IPSEC_AUTH_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_SHA_1: IPSEC_AUTH_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_SHA_256: IPSEC_AUTH_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_AES_128: IPSEC_AUTH_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_AES_192: IPSEC_AUTH_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_AES_256: IPSEC_AUTH_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_AUTH_MAX: IPSEC_AUTH_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_CBC_3DES: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_CBC_AES_128: u32 = 3u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_CBC_AES_192: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_CBC_AES_256: u32 = 5u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_CBC_DES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_GCM_AES_128: u32 = 6u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_GCM_AES_192: u32 = 7u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_GCM_AES_256: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_CONFIG_MAX: u32 = 9u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_CIPHER_TRANSFORM0 {
    pub cipherTransformId: IPSEC_CIPHER_TRANSFORM_ID0,
    pub cryptoModuleId: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for IPSEC_CIPHER_TRANSFORM0 {}
impl ::core::clone::Clone for IPSEC_CIPHER_TRANSFORM0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_CIPHER_TRANSFORM_ID0 {
    pub cipherType: IPSEC_CIPHER_TYPE,
    pub cipherConfig: u8,
}
impl ::core::marker::Copy for IPSEC_CIPHER_TRANSFORM_ID0 {}
impl ::core::clone::Clone for IPSEC_CIPHER_TRANSFORM_ID0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_CIPHER_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_TYPE_DES: IPSEC_CIPHER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_TYPE_3DES: IPSEC_CIPHER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_TYPE_AES_128: IPSEC_CIPHER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_TYPE_AES_192: IPSEC_CIPHER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_TYPE_AES_256: IPSEC_CIPHER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_CIPHER_TYPE_MAX: IPSEC_CIPHER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_DSCP_DISABLE_VALUE: u32 = 255u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_DOSP_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_FLAG_ENABLE_IKEV1: IPSEC_DOSP_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_FLAG_ENABLE_IKEV2: IPSEC_DOSP_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_FLAG_DISABLE_AUTHIP: IPSEC_DOSP_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_FLAG_DISABLE_DEFAULT_BLOCK: IPSEC_DOSP_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_FLAG_FILTER_BLOCK: IPSEC_DOSP_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_FLAG_FILTER_EXEMPT: IPSEC_DOSP_FLAGS = 32u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_DOSP_OPTIONS0 {
    pub stateIdleTimeoutSeconds: u32,
    pub perIPRateLimitQueueIdleTimeoutSeconds: u32,
    pub ipV6IPsecUnauthDscp: u8,
    pub ipV6IPsecUnauthRateLimitBytesPerSec: u32,
    pub ipV6IPsecUnauthPerIPRateLimitBytesPerSec: u32,
    pub ipV6IPsecAuthDscp: u8,
    pub ipV6IPsecAuthRateLimitBytesPerSec: u32,
    pub icmpV6Dscp: u8,
    pub icmpV6RateLimitBytesPerSec: u32,
    pub ipV6FilterExemptDscp: u8,
    pub ipV6FilterExemptRateLimitBytesPerSec: u32,
    pub defBlockExemptDscp: u8,
    pub defBlockExemptRateLimitBytesPerSec: u32,
    pub maxStateEntries: u32,
    pub maxPerIPRateLimitQueues: u32,
    pub flags: IPSEC_DOSP_FLAGS,
    pub numPublicIFLuids: u32,
    pub publicIFLuids: *mut u64,
    pub numInternalIFLuids: u32,
    pub internalIFLuids: *mut u64,
    pub publicV6AddrMask: FWP_V6_ADDR_AND_MASK,
    pub internalV6AddrMask: FWP_V6_ADDR_AND_MASK,
}
impl ::core::marker::Copy for IPSEC_DOSP_OPTIONS0 {}
impl ::core::clone::Clone for IPSEC_DOSP_OPTIONS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_DOSP_RATE_LIMIT_DISABLE_VALUE: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_DOSP_STATE0 {
    pub publicHostV6Addr: [u8; 16],
    pub internalHostV6Addr: [u8; 16],
    pub totalInboundIPv6IPsecAuthPackets: u64,
    pub totalOutboundIPv6IPsecAuthPackets: u64,
    pub durationSecs: u32,
}
impl ::core::marker::Copy for IPSEC_DOSP_STATE0 {}
impl ::core::clone::Clone for IPSEC_DOSP_STATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_DOSP_STATE_ENUM_TEMPLATE0 {
    pub publicV6AddrMask: FWP_V6_ADDR_AND_MASK,
    pub internalV6AddrMask: FWP_V6_ADDR_AND_MASK,
}
impl ::core::marker::Copy for IPSEC_DOSP_STATE_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for IPSEC_DOSP_STATE_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_DOSP_STATISTICS0 {
    pub totalStateEntriesCreated: u64,
    pub currentStateEntries: u64,
    pub totalInboundAllowedIPv6IPsecUnauthPkts: u64,
    pub totalInboundRatelimitDiscardedIPv6IPsecUnauthPkts: u64,
    pub totalInboundPerIPRatelimitDiscardedIPv6IPsecUnauthPkts: u64,
    pub totalInboundOtherDiscardedIPv6IPsecUnauthPkts: u64,
    pub totalInboundAllowedIPv6IPsecAuthPkts: u64,
    pub totalInboundRatelimitDiscardedIPv6IPsecAuthPkts: u64,
    pub totalInboundOtherDiscardedIPv6IPsecAuthPkts: u64,
    pub totalInboundAllowedICMPv6Pkts: u64,
    pub totalInboundRatelimitDiscardedICMPv6Pkts: u64,
    pub totalInboundAllowedIPv6FilterExemptPkts: u64,
    pub totalInboundRatelimitDiscardedIPv6FilterExemptPkts: u64,
    pub totalInboundDiscardedIPv6FilterBlockPkts: u64,
    pub totalInboundAllowedDefBlockExemptPkts: u64,
    pub totalInboundRatelimitDiscardedDefBlockExemptPkts: u64,
    pub totalInboundDiscardedDefBlockPkts: u64,
    pub currentInboundIPv6IPsecUnauthPerIPRateLimitQueues: u64,
}
impl ::core::marker::Copy for IPSEC_DOSP_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_DOSP_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_ESP_DROP_PACKET_STATISTICS0 {
    pub invalidSpisOnInbound: u32,
    pub decryptionFailuresOnInbound: u32,
    pub authenticationFailuresOnInbound: u32,
    pub replayCheckFailuresOnInbound: u32,
    pub saNotInitializedOnInbound: u32,
}
impl ::core::marker::Copy for IPSEC_ESP_DROP_PACKET_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_ESP_DROP_PACKET_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_FAILURE_POINT = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_FAILURE_NONE: IPSEC_FAILURE_POINT = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_FAILURE_ME: IPSEC_FAILURE_POINT = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_FAILURE_PEER: IPSEC_FAILURE_POINT = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_FAILURE_POINT_MAX: IPSEC_FAILURE_POINT = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_GETSPI0 {
    pub inboundIpsecTraffic: IPSEC_TRAFFIC0,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IPSEC_GETSPI0_0,
    pub rngCryptoModuleID: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for IPSEC_GETSPI0 {}
impl ::core::clone::Clone for IPSEC_GETSPI0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_GETSPI0_0 {
    pub inboundUdpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
impl ::core::marker::Copy for IPSEC_GETSPI0_0 {}
impl ::core::clone::Clone for IPSEC_GETSPI0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_GETSPI1 {
    pub inboundIpsecTraffic: IPSEC_TRAFFIC1,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IPSEC_GETSPI1_0,
    pub rngCryptoModuleID: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for IPSEC_GETSPI1 {}
impl ::core::clone::Clone for IPSEC_GETSPI1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_GETSPI1_0 {
    pub inboundUdpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
impl ::core::marker::Copy for IPSEC_GETSPI1_0 {}
impl ::core::clone::Clone for IPSEC_GETSPI1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_ID0 {
    pub mmTargetName: ::windows_sys::core::PWSTR,
    pub emTargetName: ::windows_sys::core::PWSTR,
    pub numTokens: u32,
    pub tokens: *mut IPSEC_TOKEN0,
    pub explicitCredentials: u64,
    pub logonId: u64,
}
impl ::core::marker::Copy for IPSEC_ID0 {}
impl ::core::clone::Clone for IPSEC_ID0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_KEYING_POLICY0 {
    pub numKeyMods: u32,
    pub keyModKeys: *mut ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for IPSEC_KEYING_POLICY0 {}
impl ::core::clone::Clone for IPSEC_KEYING_POLICY0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_KEYING_POLICY1 {
    pub numKeyMods: u32,
    pub keyModKeys: *mut ::windows_sys::core::GUID,
    pub flags: u32,
}
impl ::core::marker::Copy for IPSEC_KEYING_POLICY1 {}
impl ::core::clone::Clone for IPSEC_KEYING_POLICY1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_KEYING_POLICY_FLAG_TERMINATING_MATCH: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_KEYMODULE_STATE0 {
    pub keyModuleKey: ::windows_sys::core::GUID,
    pub stateBlob: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IPSEC_KEYMODULE_STATE0 {}
impl ::core::clone::Clone for IPSEC_KEYMODULE_STATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_KEY_MANAGER0 {
    pub keyManagerKey: ::windows_sys::core::GUID,
    pub displayData: FWPM_DISPLAY_DATA0,
    pub flags: u32,
    pub keyDictationTimeoutHint: u8,
}
impl ::core::marker::Copy for IPSEC_KEY_MANAGER0 {}
impl ::core::clone::Clone for IPSEC_KEY_MANAGER0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_KEY_MANAGER_CALLBACKS0 {
    pub reserved: ::windows_sys::core::GUID,
    pub flags: u32,
    pub keyDictationCheck: IPSEC_KEY_MANAGER_KEY_DICTATION_CHECK0,
    pub keyDictation: IPSEC_KEY_MANAGER_DICTATE_KEY0,
    pub keyNotify: IPSEC_KEY_MANAGER_NOTIFY_KEY0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_KEY_MANAGER_CALLBACKS0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_KEY_MANAGER_CALLBACKS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type IPSEC_KEY_MANAGER_DICTATE_KEY0 = ::core::option::Option<unsafe extern "system" fn(inboundsadetails: *mut IPSEC_SA_DETAILS1, outboundsadetails: *mut IPSEC_SA_DETAILS1, keyingmodulegenkey: *mut super::super::Foundation::BOOL) -> u32>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_KEY_MANAGER_FLAG_DICTATE_KEY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type IPSEC_KEY_MANAGER_KEY_DICTATION_CHECK0 = ::core::option::Option<unsafe extern "system" fn(iketraffic: *const IKEEXT_TRAFFIC0, willdictatekey: *mut super::super::Foundation::BOOL, weight: *mut u32)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub type IPSEC_KEY_MANAGER_NOTIFY_KEY0 = ::core::option::Option<unsafe extern "system" fn(inboundsa: *const IPSEC_SA_DETAILS1, outboundsa: *const IPSEC_SA_DETAILS1)>;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_PFS_GROUP = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_NONE: IPSEC_PFS_GROUP = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_1: IPSEC_PFS_GROUP = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_2: IPSEC_PFS_GROUP = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_2048: IPSEC_PFS_GROUP = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_14: IPSEC_PFS_GROUP = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_ECP_256: IPSEC_PFS_GROUP = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_ECP_384: IPSEC_PFS_GROUP = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_MM: IPSEC_PFS_GROUP = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_24: IPSEC_PFS_GROUP = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_PFS_MAX: IPSEC_PFS_GROUP = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_POLICY_FLAG = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_ND_SECURE: IPSEC_POLICY_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_ND_BOUNDARY: IPSEC_POLICY_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_NAT_ENCAP_ALLOW_PEER_BEHIND_NAT: IPSEC_POLICY_FLAG = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_NAT_ENCAP_ALLOW_GENERAL_NAT_TRAVERSAL: IPSEC_POLICY_FLAG = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_DONT_NEGOTIATE_SECOND_LIFETIME: IPSEC_POLICY_FLAG = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_DONT_NEGOTIATE_BYTE_LIFETIME: IPSEC_POLICY_FLAG = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_CLEAR_DF_ON_TUNNEL: IPSEC_POLICY_FLAG = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_ENABLE_V6_IN_V4_TUNNELING: IPSEC_POLICY_FLAG = 256u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_ENABLE_SERVER_ADDR_ASSIGNMENT: IPSEC_POLICY_FLAG = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_TUNNEL_ALLOW_OUTBOUND_CLEAR_CONNECTION: IPSEC_POLICY_FLAG = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_TUNNEL_BYPASS_ALREADY_SECURE_CONNECTION: IPSEC_POLICY_FLAG = 2048u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_TUNNEL_BYPASS_ICMPV6: IPSEC_POLICY_FLAG = 4096u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_KEY_MANAGER_ALLOW_DICTATE_KEY: IPSEC_POLICY_FLAG = 8192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_KEY_MANAGER_ALLOW_NOTIFY_KEY: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_RESERVED1: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_POLICY_FLAG_SITE_TO_SITE_TUNNEL: u32 = 65536u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_PROPOSAL0 {
    pub lifetime: IPSEC_SA_LIFETIME0,
    pub numSaTransforms: u32,
    pub saTransforms: *mut IPSEC_SA_TRANSFORM0,
    pub pfsGroup: IPSEC_PFS_GROUP,
}
impl ::core::marker::Copy for IPSEC_PROPOSAL0 {}
impl ::core::clone::Clone for IPSEC_PROPOSAL0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA0 {
    pub spi: u32,
    pub saTransformType: IPSEC_TRANSFORM_TYPE,
    pub Anonymous: IPSEC_SA0_0,
}
impl ::core::marker::Copy for IPSEC_SA0 {}
impl ::core::clone::Clone for IPSEC_SA0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_SA0_0 {
    pub ahInformation: *mut IPSEC_SA_AUTH_INFORMATION0,
    pub espAuthInformation: *mut IPSEC_SA_AUTH_INFORMATION0,
    pub espCipherInformation: *mut IPSEC_SA_CIPHER_INFORMATION0,
    pub espAuthAndCipherInformation: *mut IPSEC_SA_AUTH_AND_CIPHER_INFORMATION0,
    pub espAuthFwInformation: *mut IPSEC_SA_AUTH_INFORMATION0,
}
impl ::core::marker::Copy for IPSEC_SA0_0 {}
impl ::core::clone::Clone for IPSEC_SA0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_AUTH_AND_CIPHER_INFORMATION0 {
    pub saCipherInformation: IPSEC_SA_CIPHER_INFORMATION0,
    pub saAuthInformation: IPSEC_SA_AUTH_INFORMATION0,
}
impl ::core::marker::Copy for IPSEC_SA_AUTH_AND_CIPHER_INFORMATION0 {}
impl ::core::clone::Clone for IPSEC_SA_AUTH_AND_CIPHER_INFORMATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_AUTH_INFORMATION0 {
    pub authTransform: IPSEC_AUTH_TRANSFORM0,
    pub authKey: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IPSEC_SA_AUTH_INFORMATION0 {}
impl ::core::clone::Clone for IPSEC_SA_AUTH_INFORMATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_BUNDLE0 {
    pub flags: IPSEC_SA_BUNDLE_FLAGS,
    pub lifetime: IPSEC_SA_LIFETIME0,
    pub idleTimeoutSeconds: u32,
    pub ndAllowClearTimeoutSeconds: u32,
    pub ipsecId: *mut IPSEC_ID0,
    pub napContext: u32,
    pub qmSaId: u32,
    pub numSAs: u32,
    pub saList: *mut IPSEC_SA0,
    pub keyModuleState: *mut IPSEC_KEYMODULE_STATE0,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IPSEC_SA_BUNDLE0_0,
    pub mmSaId: u64,
    pub pfsGroup: IPSEC_PFS_GROUP,
}
impl ::core::marker::Copy for IPSEC_SA_BUNDLE0 {}
impl ::core::clone::Clone for IPSEC_SA_BUNDLE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_SA_BUNDLE0_0 {
    pub peerV4PrivateAddress: u32,
}
impl ::core::marker::Copy for IPSEC_SA_BUNDLE0_0 {}
impl ::core::clone::Clone for IPSEC_SA_BUNDLE0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_BUNDLE1 {
    pub flags: IPSEC_SA_BUNDLE_FLAGS,
    pub lifetime: IPSEC_SA_LIFETIME0,
    pub idleTimeoutSeconds: u32,
    pub ndAllowClearTimeoutSeconds: u32,
    pub ipsecId: *mut IPSEC_ID0,
    pub napContext: u32,
    pub qmSaId: u32,
    pub numSAs: u32,
    pub saList: *mut IPSEC_SA0,
    pub keyModuleState: *mut IPSEC_KEYMODULE_STATE0,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IPSEC_SA_BUNDLE1_0,
    pub mmSaId: u64,
    pub pfsGroup: IPSEC_PFS_GROUP,
    pub saLookupContext: ::windows_sys::core::GUID,
    pub qmFilterId: u64,
}
impl ::core::marker::Copy for IPSEC_SA_BUNDLE1 {}
impl ::core::clone::Clone for IPSEC_SA_BUNDLE1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_SA_BUNDLE1_0 {
    pub peerV4PrivateAddress: u32,
}
impl ::core::marker::Copy for IPSEC_SA_BUNDLE1_0 {}
impl ::core::clone::Clone for IPSEC_SA_BUNDLE1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_SA_BUNDLE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ND_SECURE: IPSEC_SA_BUNDLE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ND_BOUNDARY: IPSEC_SA_BUNDLE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ND_PEER_NAT_BOUNDARY: IPSEC_SA_BUNDLE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_GUARANTEE_ENCRYPTION: IPSEC_SA_BUNDLE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ALLOW_NULL_TARGET_NAME_MATCH: IPSEC_SA_BUNDLE_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_CLEAR_DF_ON_TUNNEL: IPSEC_SA_BUNDLE_FLAGS = 1024u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ASSUME_UDP_CONTEXT_OUTBOUND: IPSEC_SA_BUNDLE_FLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ND_PEER_BOUNDARY: IPSEC_SA_BUNDLE_FLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_SUPPRESS_DUPLICATE_DELETION: IPSEC_SA_BUNDLE_FLAGS = 8192u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_PEER_SUPPORTS_GUARANTEE_ENCRYPTION: IPSEC_SA_BUNDLE_FLAGS = 16384u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_ENABLE_OPTIONAL_ASYMMETRIC_IDLE: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_FORCE_INBOUND_CONNECTIONS: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_FORCE_OUTBOUND_CONNECTIONS: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_FORWARD_PATH_INITIATOR: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_IP_IN_IP_PKT: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_LOCALLY_DICTATED_KEYS: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_LOW_POWER_MODE_SUPPORT: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_NLB: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_NO_EXPLICIT_CRED_MATCH: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_NO_IMPERSONATION_LUID_VERIFY: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_NO_MACHINE_LUID_VERIFY: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_SA_OFFLOADED: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_BUNDLE_FLAG_USING_DICTATED_KEYS: u32 = 524288u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_CIPHER_INFORMATION0 {
    pub cipherTransform: IPSEC_CIPHER_TRANSFORM0,
    pub cipherKey: FWP_BYTE_BLOB,
}
impl ::core::marker::Copy for IPSEC_SA_CIPHER_INFORMATION0 {}
impl ::core::clone::Clone for IPSEC_SA_CIPHER_INFORMATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_SA_CONTEXT0 {
    pub saContextId: u64,
    pub inboundSa: *mut IPSEC_SA_DETAILS0,
    pub outboundSa: *mut IPSEC_SA_DETAILS0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_CONTEXT0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_CONTEXT0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_SA_CONTEXT1 {
    pub saContextId: u64,
    pub inboundSa: *mut IPSEC_SA_DETAILS1,
    pub outboundSa: *mut IPSEC_SA_DETAILS1,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_CONTEXT1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_CONTEXT1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_SA_CONTEXT_CALLBACK0 = ::core::option::Option<unsafe extern "system" fn(context: *mut ::core::ffi::c_void, change: *const IPSEC_SA_CONTEXT_CHANGE0)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_CONTEXT_CHANGE0 {
    pub changeType: IPSEC_SA_CONTEXT_EVENT_TYPE0,
    pub saContextId: u64,
}
impl ::core::marker::Copy for IPSEC_SA_CONTEXT_CHANGE0 {}
impl ::core::clone::Clone for IPSEC_SA_CONTEXT_CHANGE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_SA_CONTEXT_ENUM_TEMPLATE0 {
    pub localSubNet: FWP_CONDITION_VALUE0,
    pub remoteSubNet: FWP_CONDITION_VALUE0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_CONTEXT_ENUM_TEMPLATE0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_CONTEXT_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_SA_CONTEXT_EVENT_TYPE0 = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_CONTEXT_EVENT_ADD: IPSEC_SA_CONTEXT_EVENT_TYPE0 = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_CONTEXT_EVENT_DELETE: IPSEC_SA_CONTEXT_EVENT_TYPE0 = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_SA_CONTEXT_EVENT_MAX: IPSEC_SA_CONTEXT_EVENT_TYPE0 = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_SA_CONTEXT_SUBSCRIPTION0 {
    pub enumTemplate: *mut IPSEC_SA_CONTEXT_ENUM_TEMPLATE0,
    pub flags: u32,
    pub sessionKey: ::windows_sys::core::GUID,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_CONTEXT_SUBSCRIPTION0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_CONTEXT_SUBSCRIPTION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_SA_DETAILS0 {
    pub ipVersion: FWP_IP_VERSION,
    pub saDirection: FWP_DIRECTION,
    pub traffic: IPSEC_TRAFFIC0,
    pub saBundle: IPSEC_SA_BUNDLE0,
    pub Anonymous: IPSEC_SA_DETAILS0_0,
    pub transportFilter: *mut FWPM_FILTER0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_DETAILS0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_DETAILS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union IPSEC_SA_DETAILS0_0 {
    pub udpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_DETAILS0_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_DETAILS0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct IPSEC_SA_DETAILS1 {
    pub ipVersion: FWP_IP_VERSION,
    pub saDirection: FWP_DIRECTION,
    pub traffic: IPSEC_TRAFFIC1,
    pub saBundle: IPSEC_SA_BUNDLE1,
    pub Anonymous: IPSEC_SA_DETAILS1_0,
    pub transportFilter: *mut FWPM_FILTER0,
    pub virtualIfTunnelInfo: IPSEC_VIRTUAL_IF_TUNNEL_INFO0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_DETAILS1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_DETAILS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub union IPSEC_SA_DETAILS1_0 {
    pub udpEncapsulation: *mut IPSEC_V4_UDP_ENCAPSULATION0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for IPSEC_SA_DETAILS1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for IPSEC_SA_DETAILS1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_ENUM_TEMPLATE0 {
    pub saDirection: FWP_DIRECTION,
}
impl ::core::marker::Copy for IPSEC_SA_ENUM_TEMPLATE0 {}
impl ::core::clone::Clone for IPSEC_SA_ENUM_TEMPLATE0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_IDLE_TIMEOUT0 {
    pub idleTimeoutSeconds: u32,
    pub idleTimeoutSecondsFailOver: u32,
}
impl ::core::marker::Copy for IPSEC_SA_IDLE_TIMEOUT0 {}
impl ::core::clone::Clone for IPSEC_SA_IDLE_TIMEOUT0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_LIFETIME0 {
    pub lifetimeSeconds: u32,
    pub lifetimeKilobytes: u32,
    pub lifetimePackets: u32,
}
impl ::core::marker::Copy for IPSEC_SA_LIFETIME0 {}
impl ::core::clone::Clone for IPSEC_SA_LIFETIME0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_SA_TRANSFORM0 {
    pub ipsecTransformType: IPSEC_TRANSFORM_TYPE,
    pub Anonymous: IPSEC_SA_TRANSFORM0_0,
}
impl ::core::marker::Copy for IPSEC_SA_TRANSFORM0 {}
impl ::core::clone::Clone for IPSEC_SA_TRANSFORM0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_SA_TRANSFORM0_0 {
    pub ahTransform: *mut IPSEC_AUTH_TRANSFORM0,
    pub espAuthTransform: *mut IPSEC_AUTH_TRANSFORM0,
    pub espCipherTransform: *mut IPSEC_CIPHER_TRANSFORM0,
    pub espAuthAndCipherTransform: *mut IPSEC_AUTH_AND_CIPHER_TRANSFORM0,
    pub espAuthFwTransform: *mut IPSEC_AUTH_TRANSFORM0,
}
impl ::core::marker::Copy for IPSEC_SA_TRANSFORM0_0 {}
impl ::core::clone::Clone for IPSEC_SA_TRANSFORM0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_STATISTICS0 {
    pub aggregateSaStatistics: IPSEC_AGGREGATE_SA_STATISTICS0,
    pub espDropPacketStatistics: IPSEC_ESP_DROP_PACKET_STATISTICS0,
    pub ahDropPacketStatistics: IPSEC_AH_DROP_PACKET_STATISTICS0,
    pub aggregateDropPacketStatistics: IPSEC_AGGREGATE_DROP_PACKET_STATISTICS0,
    pub inboundTrafficStatistics: IPSEC_TRAFFIC_STATISTICS0,
    pub outboundTrafficStatistics: IPSEC_TRAFFIC_STATISTICS0,
}
impl ::core::marker::Copy for IPSEC_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_STATISTICS1 {
    pub aggregateSaStatistics: IPSEC_AGGREGATE_SA_STATISTICS0,
    pub espDropPacketStatistics: IPSEC_ESP_DROP_PACKET_STATISTICS0,
    pub ahDropPacketStatistics: IPSEC_AH_DROP_PACKET_STATISTICS0,
    pub aggregateDropPacketStatistics: IPSEC_AGGREGATE_DROP_PACKET_STATISTICS1,
    pub inboundTrafficStatistics: IPSEC_TRAFFIC_STATISTICS1,
    pub outboundTrafficStatistics: IPSEC_TRAFFIC_STATISTICS1,
}
impl ::core::marker::Copy for IPSEC_STATISTICS1 {}
impl ::core::clone::Clone for IPSEC_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TOKEN0 {
    pub r#type: IPSEC_TOKEN_TYPE,
    pub principal: IPSEC_TOKEN_PRINCIPAL,
    pub mode: IPSEC_TOKEN_MODE,
    pub token: u64,
}
impl ::core::marker::Copy for IPSEC_TOKEN0 {}
impl ::core::clone::Clone for IPSEC_TOKEN0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_TOKEN_MODE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_MODE_MAIN: IPSEC_TOKEN_MODE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_MODE_EXTENDED: IPSEC_TOKEN_MODE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_MODE_MAX: IPSEC_TOKEN_MODE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_TOKEN_PRINCIPAL = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_PRINCIPAL_LOCAL: IPSEC_TOKEN_PRINCIPAL = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_PRINCIPAL_PEER: IPSEC_TOKEN_PRINCIPAL = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_PRINCIPAL_MAX: IPSEC_TOKEN_PRINCIPAL = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_TOKEN_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_TYPE_MACHINE: IPSEC_TOKEN_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_TYPE_IMPERSONATION: IPSEC_TOKEN_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TOKEN_TYPE_MAX: IPSEC_TOKEN_TYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRAFFIC0 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IPSEC_TRAFFIC0_0,
    pub Anonymous2: IPSEC_TRAFFIC0_1,
    pub trafficType: IPSEC_TRAFFIC_TYPE,
    pub Anonymous3: IPSEC_TRAFFIC0_2,
    pub remotePort: u16,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC0 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC0_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TRAFFIC0_0 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC0_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TRAFFIC0_1 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC0_2 {
    pub ipsecFilterId: u64,
    pub tunnelPolicyId: u64,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC0_2 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC0_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRAFFIC1 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IPSEC_TRAFFIC1_0,
    pub Anonymous2: IPSEC_TRAFFIC1_1,
    pub trafficType: IPSEC_TRAFFIC_TYPE,
    pub Anonymous3: IPSEC_TRAFFIC1_2,
    pub remotePort: u16,
    pub localPort: u16,
    pub ipProtocol: u8,
    pub localIfLuid: u64,
    pub realIfProfileId: u32,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC1 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC1_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TRAFFIC1_0 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC1_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TRAFFIC1_1 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC1_2 {
    pub ipsecFilterId: u64,
    pub tunnelPolicyId: u64,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC1_2 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC1_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRAFFIC_SELECTOR0_ {
    pub protocolId: u8,
    pub portStart: u16,
    pub portEnd: u16,
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IPSEC_TRAFFIC_SELECTOR0__0,
    pub Anonymous2: IPSEC_TRAFFIC_SELECTOR0__1,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC_SELECTOR0_ {}
impl ::core::clone::Clone for IPSEC_TRAFFIC_SELECTOR0_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC_SELECTOR0__0 {
    pub startV4Address: u32,
    pub startV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TRAFFIC_SELECTOR0__0 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC_SELECTOR0__0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TRAFFIC_SELECTOR0__1 {
    pub endV4Address: u32,
    pub endV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TRAFFIC_SELECTOR0__1 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC_SELECTOR0__1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRAFFIC_SELECTOR_POLICY0_ {
    pub flags: u32,
    pub numLocalTrafficSelectors: u32,
    pub localTrafficSelectors: *mut IPSEC_TRAFFIC_SELECTOR0_,
    pub numRemoteTrafficSelectors: u32,
    pub remoteTrafficSelectors: *mut IPSEC_TRAFFIC_SELECTOR0_,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC_SELECTOR_POLICY0_ {}
impl ::core::clone::Clone for IPSEC_TRAFFIC_SELECTOR_POLICY0_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRAFFIC_STATISTICS0 {
    pub encryptedByteCount: u64,
    pub authenticatedAHByteCount: u64,
    pub authenticatedESPByteCount: u64,
    pub transportByteCount: u64,
    pub tunnelByteCount: u64,
    pub offloadByteCount: u64,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC_STATISTICS0 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC_STATISTICS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRAFFIC_STATISTICS1 {
    pub encryptedByteCount: u64,
    pub authenticatedAHByteCount: u64,
    pub authenticatedESPByteCount: u64,
    pub transportByteCount: u64,
    pub tunnelByteCount: u64,
    pub offloadByteCount: u64,
    pub totalSuccessfulPackets: u64,
}
impl ::core::marker::Copy for IPSEC_TRAFFIC_STATISTICS1 {}
impl ::core::clone::Clone for IPSEC_TRAFFIC_STATISTICS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_TRAFFIC_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRAFFIC_TYPE_TRANSPORT: IPSEC_TRAFFIC_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRAFFIC_TYPE_TUNNEL: IPSEC_TRAFFIC_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRAFFIC_TYPE_MAX: IPSEC_TRAFFIC_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPSEC_TRANSFORM_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRANSFORM_AH: IPSEC_TRANSFORM_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRANSFORM_ESP_AUTH: IPSEC_TRANSFORM_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRANSFORM_ESP_CIPHER: IPSEC_TRANSFORM_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRANSFORM_ESP_AUTH_AND_CIPHER: IPSEC_TRANSFORM_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRANSFORM_ESP_AUTH_FW: IPSEC_TRANSFORM_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPSEC_TRANSFORM_TYPE_MAX: IPSEC_TRANSFORM_TYPE = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRANSPORT_POLICY0 {
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub flags: IPSEC_POLICY_FLAG,
    pub ndAllowClearTimeoutSeconds: u32,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY0,
}
impl ::core::marker::Copy for IPSEC_TRANSPORT_POLICY0 {}
impl ::core::clone::Clone for IPSEC_TRANSPORT_POLICY0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRANSPORT_POLICY1 {
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub flags: IPSEC_POLICY_FLAG,
    pub ndAllowClearTimeoutSeconds: u32,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY1,
}
impl ::core::marker::Copy for IPSEC_TRANSPORT_POLICY1 {}
impl ::core::clone::Clone for IPSEC_TRANSPORT_POLICY1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TRANSPORT_POLICY2 {
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub flags: IPSEC_POLICY_FLAG,
    pub ndAllowClearTimeoutSeconds: u32,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY2,
}
impl ::core::marker::Copy for IPSEC_TRANSPORT_POLICY2 {}
impl ::core::clone::Clone for IPSEC_TRANSPORT_POLICY2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_ENDPOINT0 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous: IPSEC_TUNNEL_ENDPOINT0_0,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINT0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINT0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINT0_0 {
    pub v4Address: u32,
    pub v6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINT0_0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINT0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_ENDPOINTS0 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IPSEC_TUNNEL_ENDPOINTS0_0,
    pub Anonymous2: IPSEC_TUNNEL_ENDPOINTS0_1,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINTS0_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS0_0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINTS0_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS0_1 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_ENDPOINTS1 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IPSEC_TUNNEL_ENDPOINTS1_0,
    pub Anonymous2: IPSEC_TUNNEL_ENDPOINTS1_1,
    pub localIfLuid: u64,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS1 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINTS1_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS1_0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINTS1_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS1_1 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_ENDPOINTS2 {
    pub ipVersion: FWP_IP_VERSION,
    pub Anonymous1: IPSEC_TUNNEL_ENDPOINTS2_0,
    pub Anonymous2: IPSEC_TUNNEL_ENDPOINTS2_1,
    pub localIfLuid: u64,
    pub remoteFqdn: ::windows_sys::core::PWSTR,
    pub numAddresses: u32,
    pub remoteAddresses: *mut IPSEC_TUNNEL_ENDPOINT0,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS2 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINTS2_0 {
    pub localV4Address: u32,
    pub localV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS2_0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPSEC_TUNNEL_ENDPOINTS2_1 {
    pub remoteV4Address: u32,
    pub remoteV6Address: [u8; 16],
}
impl ::core::marker::Copy for IPSEC_TUNNEL_ENDPOINTS2_1 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_ENDPOINTS2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_POLICY0 {
    pub flags: IPSEC_POLICY_FLAG,
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub tunnelEndpoints: IPSEC_TUNNEL_ENDPOINTS0,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY0,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_POLICY0 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_POLICY0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_POLICY1 {
    pub flags: IPSEC_POLICY_FLAG,
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub tunnelEndpoints: IPSEC_TUNNEL_ENDPOINTS1,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY1,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_POLICY1 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_POLICY1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_POLICY2 {
    pub flags: IPSEC_POLICY_FLAG,
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub tunnelEndpoints: IPSEC_TUNNEL_ENDPOINTS2,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY2,
    pub fwdPathSaLifetime: u32,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_POLICY2 {}
impl ::core::clone::Clone for IPSEC_TUNNEL_POLICY2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_TUNNEL_POLICY3_ {
    pub flags: u32,
    pub numIpsecProposals: u32,
    pub ipsecProposals: *mut IPSEC_PROPOSAL0,
    pub tunnelEndpoints: IPSEC_TUNNEL_ENDPOINTS2,
    pub saIdleTimeout: IPSEC_SA_IDLE_TIMEOUT0,
    pub emPolicy: *mut IKEEXT_EM_POLICY2,
    pub fwdPathSaLifetime: u32,
    pub compartmentId: u32,
    pub numTrafficSelectorPolicy: u32,
    pub trafficSelectorPolicies: *mut IPSEC_TRAFFIC_SELECTOR_POLICY0_,
}
impl ::core::marker::Copy for IPSEC_TUNNEL_POLICY3_ {}
impl ::core::clone::Clone for IPSEC_TUNNEL_POLICY3_ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_V4_UDP_ENCAPSULATION0 {
    pub localUdpEncapPort: u16,
    pub remoteUdpEncapPort: u16,
}
impl ::core::marker::Copy for IPSEC_V4_UDP_ENCAPSULATION0 {}
impl ::core::clone::Clone for IPSEC_V4_UDP_ENCAPSULATION0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPSEC_VIRTUAL_IF_TUNNEL_INFO0 {
    pub virtualIfTunnelId: u64,
    pub trafficSelectorId: u64,
}
impl ::core::marker::Copy for IPSEC_VIRTUAL_IF_TUNNEL_INFO0 {}
impl ::core::clone::Clone for IPSEC_VIRTUAL_IF_TUNNEL_INFO0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPTLS_METADATA {
    pub SequenceNumber: u64,
}
impl ::core::marker::Copy for IPTLS_METADATA {}
impl ::core::clone::Clone for IPTLS_METADATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IPV4_HEADER {
    pub Anonymous1: IPV4_HEADER_0,
    pub Anonymous2: IPV4_HEADER_1,
    pub TotalLength: u16,
    pub Identification: u16,
    pub Anonymous3: IPV4_HEADER_2,
    pub TimeToLive: u8,
    pub Protocol: u8,
    pub HeaderChecksum: u16,
    pub SourceAddress: super::super::Networking::WinSock::IN_ADDR,
    pub DestinationAddress: super::super::Networking::WinSock::IN_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IPV4_HEADER_0 {
    pub VersionAndHeaderLength: u8,
    pub Anonymous: IPV4_HEADER_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IPV4_HEADER_0_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IPV4_HEADER_1 {
    pub TypeOfServiceAndEcnField: u8,
    pub Anonymous: IPV4_HEADER_1_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER_1 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IPV4_HEADER_1_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER_1_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IPV4_HEADER_2 {
    pub FlagsAndOffset: u16,
    pub Anonymous: IPV4_HEADER_2_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER_2 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IPV4_HEADER_2_0 {
    pub _bitfield: u16,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV4_HEADER_2_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV4_HEADER_2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV4_MAX_MINIMUM_MTU: u32 = 576u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV4_MINIMUM_MTU: u32 = 576u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV4_MIN_MINIMUM_MTU: u32 = 352u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV4_OPTION_HEADER {
    pub Anonymous: IPV4_OPTION_HEADER_0,
    pub OptionLength: u8,
}
impl ::core::marker::Copy for IPV4_OPTION_HEADER {}
impl ::core::clone::Clone for IPV4_OPTION_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPV4_OPTION_HEADER_0 {
    pub OptionType: u8,
    pub Anonymous: IPV4_OPTION_HEADER_0_0,
}
impl ::core::marker::Copy for IPV4_OPTION_HEADER_0 {}
impl ::core::clone::Clone for IPV4_OPTION_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV4_OPTION_HEADER_0_0 {
    pub _bitfield: u8,
}
impl ::core::marker::Copy for IPV4_OPTION_HEADER_0_0 {}
impl ::core::clone::Clone for IPV4_OPTION_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPV4_OPTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_EOL: IPV4_OPTION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_NOP: IPV4_OPTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_SECURITY: IPV4_OPTION_TYPE = 130i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_LSRR: IPV4_OPTION_TYPE = 131i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_TS: IPV4_OPTION_TYPE = 68i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_RR: IPV4_OPTION_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_SSRR: IPV4_OPTION_TYPE = 137i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_SID: IPV4_OPTION_TYPE = 136i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_ROUTER_ALERT: IPV4_OPTION_TYPE = 148i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPT_MULTIDEST: IPV4_OPTION_TYPE = 149i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV4_ROUTING_HEADER {
    pub OptionHeader: IPV4_OPTION_HEADER,
    pub Pointer: u8,
}
impl ::core::marker::Copy for IPV4_ROUTING_HEADER {}
impl ::core::clone::Clone for IPV4_ROUTING_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV4_TIMESTAMP_OPTION {
    pub OptionHeader: IPV4_OPTION_HEADER,
    pub Pointer: u8,
    pub Anonymous: IPV4_TIMESTAMP_OPTION_0,
}
impl ::core::marker::Copy for IPV4_TIMESTAMP_OPTION {}
impl ::core::clone::Clone for IPV4_TIMESTAMP_OPTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPV4_TIMESTAMP_OPTION_0 {
    pub FlagsOverflow: u8,
    pub Anonymous: IPV4_TIMESTAMP_OPTION_0_0,
}
impl ::core::marker::Copy for IPV4_TIMESTAMP_OPTION_0 {}
impl ::core::clone::Clone for IPV4_TIMESTAMP_OPTION_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV4_TIMESTAMP_OPTION_0_0 {
    pub _bitfield: u8,
}
impl ::core::marker::Copy for IPV4_TIMESTAMP_OPTION_0_0 {}
impl ::core::clone::Clone for IPV4_TIMESTAMP_OPTION_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV4_VERSION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_ECN_MASK: u32 = 12288u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_ECN_SHIFT: u32 = 12u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_EXTENSION_HEADER {
    pub NextHeader: u8,
    pub Length: u8,
}
impl ::core::marker::Copy for IPV6_EXTENSION_HEADER {}
impl ::core::clone::Clone for IPV6_EXTENSION_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_FLOW_LABEL_MASK: u32 = 4294905600u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_FRAGMENT_HEADER {
    pub NextHeader: u8,
    pub Reserved: u8,
    pub Anonymous: IPV6_FRAGMENT_HEADER_0,
    pub Id: u32,
}
impl ::core::marker::Copy for IPV6_FRAGMENT_HEADER {}
impl ::core::clone::Clone for IPV6_FRAGMENT_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPV6_FRAGMENT_HEADER_0 {
    pub Anonymous: IPV6_FRAGMENT_HEADER_0_0,
    pub OffsetAndFlags: u16,
}
impl ::core::marker::Copy for IPV6_FRAGMENT_HEADER_0 {}
impl ::core::clone::Clone for IPV6_FRAGMENT_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_FRAGMENT_HEADER_0_0 {
    pub _bitfield: u16,
}
impl ::core::marker::Copy for IPV6_FRAGMENT_HEADER_0_0 {}
impl ::core::clone::Clone for IPV6_FRAGMENT_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_FULL_TRAFFIC_CLASS_MASK: u32 = 61455u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IPV6_HEADER {
    pub Anonymous: IPV6_HEADER_0,
    pub PayloadLength: u16,
    pub NextHeader: u8,
    pub HopLimit: u8,
    pub SourceAddress: super::super::Networking::WinSock::IN6_ADDR,
    pub DestinationAddress: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV6_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV6_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union IPV6_HEADER_0 {
    pub VersionClassFlow: u32,
    pub Anonymous: IPV6_HEADER_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV6_HEADER_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV6_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct IPV6_HEADER_0_0 {
    pub _bitfield: u32,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for IPV6_HEADER_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for IPV6_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_MINIMUM_MTU: u32 = 1280u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS {
    pub Anonymous: IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS_0,
    pub Value: u32,
}
impl ::core::marker::Copy for IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS {}
impl ::core::clone::Clone for IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS_0 {
    pub _bitfield: u8,
    pub Reserved2: [u8; 3],
}
impl ::core::marker::Copy for IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS_0 {}
impl ::core::clone::Clone for IPV6_NEIGHBOR_ADVERTISEMENT_FLAGS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_OPTION_HEADER {
    pub Type: u8,
    pub DataLength: u8,
}
impl ::core::marker::Copy for IPV6_OPTION_HEADER {}
impl ::core::clone::Clone for IPV6_OPTION_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_OPTION_JUMBOGRAM {
    pub Header: IPV6_OPTION_HEADER,
    pub JumbogramLength: [u8; 4],
}
impl ::core::marker::Copy for IPV6_OPTION_JUMBOGRAM {}
impl ::core::clone::Clone for IPV6_OPTION_JUMBOGRAM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_OPTION_ROUTER_ALERT {
    pub Header: IPV6_OPTION_HEADER,
    pub Value: [u8; 2],
}
impl ::core::marker::Copy for IPV6_OPTION_ROUTER_ALERT {}
impl ::core::clone::Clone for IPV6_OPTION_ROUTER_ALERT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IPV6_OPTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_PAD1: IPV6_OPTION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_PADN: IPV6_OPTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_TUNNEL_LIMIT: IPV6_OPTION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_ROUTER_ALERT: IPV6_OPTION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_JUMBO: IPV6_OPTION_TYPE = 194i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP6OPT_NSAP_ADDR: IPV6_OPTION_TYPE = 195i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union IPV6_ROUTER_ADVERTISEMENT_FLAGS {
    pub Anonymous: IPV6_ROUTER_ADVERTISEMENT_FLAGS_0,
    pub Value: u8,
}
impl ::core::marker::Copy for IPV6_ROUTER_ADVERTISEMENT_FLAGS {}
impl ::core::clone::Clone for IPV6_ROUTER_ADVERTISEMENT_FLAGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_ROUTER_ADVERTISEMENT_FLAGS_0 {
    pub _bitfield: u8,
}
impl ::core::marker::Copy for IPV6_ROUTER_ADVERTISEMENT_FLAGS_0 {}
impl ::core::clone::Clone for IPV6_ROUTER_ADVERTISEMENT_FLAGS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct IPV6_ROUTING_HEADER {
    pub NextHeader: u8,
    pub Length: u8,
    pub RoutingType: u8,
    pub SegmentsLeft: u8,
    pub Reserved: [u8; 4],
}
impl ::core::marker::Copy for IPV6_ROUTING_HEADER {}
impl ::core::clone::Clone for IPV6_ROUTING_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_TRAFFIC_CLASS_MASK: u32 = 49167u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IPV6_VERSION: u32 = 96u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type IP_OPTION_TIMESTAMP_FLAGS = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPTION_TIMESTAMP_ONLY: IP_OPTION_TIMESTAMP_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPTION_TIMESTAMP_ADDRESS: IP_OPTION_TIMESTAMP_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_OPTION_TIMESTAMP_SPECIFIC_ADDRESS: IP_OPTION_TIMESTAMP_FLAGS = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const IP_VER_MASK: u32 = 240u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MAX_IPV4_HLEN: u32 = 60u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MAX_IPV4_PACKET: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MAX_IPV6_PAYLOAD: u32 = 65535u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct MLDV2_QUERY_HEADER {
    pub IcmpHeader: ICMP_HEADER,
    pub Anonymous1: MLDV2_QUERY_HEADER_0,
    pub Reserved: u16,
    pub MulticastAddress: super::super::Networking::WinSock::IN6_ADDR,
    pub _bitfield: u8,
    pub Anonymous2: MLDV2_QUERY_HEADER_1,
    pub SourceCount: u16,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLDV2_QUERY_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLDV2_QUERY_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union MLDV2_QUERY_HEADER_0 {
    pub MaxRespCode: u16,
    pub Anonymous: MLDV2_QUERY_HEADER_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLDV2_QUERY_HEADER_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLDV2_QUERY_HEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct MLDV2_QUERY_HEADER_0_0 {
    pub _bitfield: u16,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLDV2_QUERY_HEADER_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLDV2_QUERY_HEADER_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union MLDV2_QUERY_HEADER_1 {
    pub QueriersQueryInterfaceCode: u8,
    pub Anonymous: MLDV2_QUERY_HEADER_1_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLDV2_QUERY_HEADER_1 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLDV2_QUERY_HEADER_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct MLDV2_QUERY_HEADER_1_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLDV2_QUERY_HEADER_1_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLDV2_QUERY_HEADER_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct MLDV2_REPORT_HEADER {
    pub IcmpHeader: ICMP_HEADER,
    pub Reserved: u16,
    pub RecordCount: u16,
}
impl ::core::marker::Copy for MLDV2_REPORT_HEADER {}
impl ::core::clone::Clone for MLDV2_REPORT_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct MLDV2_REPORT_RECORD_HEADER {
    pub Type: u8,
    pub AuxillaryDataLength: u8,
    pub SourceCount: u16,
    pub MulticastAddress: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLDV2_REPORT_RECORD_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLDV2_REPORT_RECORD_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct MLD_HEADER {
    pub IcmpHeader: ICMP_HEADER,
    pub MaxRespTime: u16,
    pub Reserved: u16,
    pub MulticastAddress: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for MLD_HEADER {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for MLD_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type MLD_MAX_RESP_CODE_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MLD_MAX_RESP_CODE_TYPE_NORMAL: MLD_MAX_RESP_CODE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MLD_MAX_RESP_CODE_TYPE_FLOAT: MLD_MAX_RESP_CODE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_NA_FLAG_OVERRIDE: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_NA_FLAG_ROUTER: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_NA_FLAG_SOLICITED: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type ND_OPTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_SOURCE_LINKADDR: ND_OPTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_TARGET_LINKADDR: ND_OPTION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_PREFIX_INFORMATION: ND_OPTION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_REDIRECTED_HEADER: ND_OPTION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_MTU: ND_OPTION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_NBMA_SHORTCUT_LIMIT: ND_OPTION_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_ADVERTISEMENT_INTERVAL: ND_OPTION_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_HOME_AGENT_INFORMATION: ND_OPTION_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_SOURCE_ADDR_LIST: ND_OPTION_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_TARGET_ADDR_LIST: ND_OPTION_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_ROUTE_INFO: ND_OPTION_TYPE = 24i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_RDNSS: ND_OPTION_TYPE = 25i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_DNSSL: ND_OPTION_TYPE = 31i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_DNSSL_MIN_LEN: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_PI_FLAG_AUTO: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_PI_FLAG_ONLINK: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_PI_FLAG_ROUTE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_PI_FLAG_ROUTER_ADDR: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_PI_FLAG_SITE_PREFIX: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_RDNSS_MIN_LEN: u32 = 24u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_OPT_RI_FLAG_PREFERENCE: u32 = 24u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_RA_FLAG_HOME_AGENT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_RA_FLAG_MANAGED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_RA_FLAG_OTHER: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const ND_RA_FLAG_PREFERENCE: u32 = 24u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NPI_MODULEID {
    pub Length: u16,
    pub Type: NPI_MODULEID_TYPE,
    pub Anonymous: NPI_MODULEID_0,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NPI_MODULEID {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NPI_MODULEID {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union NPI_MODULEID_0 {
    pub Guid: ::windows_sys::core::GUID,
    pub IfLuid: super::super::Foundation::LUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NPI_MODULEID_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NPI_MODULEID_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type NPI_MODULEID_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MIT_GUID: NPI_MODULEID_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const MIT_IF_LUID: NPI_MODULEID_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SIZEOF_IP_OPT_ROUTERALERT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SIZEOF_IP_OPT_ROUTING_HEADER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SIZEOF_IP_OPT_SECURITY: u32 = 11u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SIZEOF_IP_OPT_STREAMIDENTIFIER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SIZEOF_IP_OPT_TIMESTAMP_HEADER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SNAP_CONTROL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SNAP_DSAP: u32 = 170u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct SNAP_HEADER {
    pub Dsap: u8,
    pub Ssap: u8,
    pub Control: u8,
    pub Oui: [u8; 3],
    pub Type: u16,
}
impl ::core::marker::Copy for SNAP_HEADER {}
impl ::core::clone::Clone for SNAP_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SNAP_OUI: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const SNAP_SSAP: u32 = 170u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_ACK: u32 = 16u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_CWR: u32 = 128u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_ECE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_FIN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_EOL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_FASTOPEN: u32 = 34u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_MSS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_NOP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_SACK: u32 = 5u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_SACK_PERMITTED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_TS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_OPT_WS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_PSH: u32 = 8u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_RST: u32 = 4u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_SYN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TH_URG: u32 = 32u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub type TUNNEL_SUB_TYPE = i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TUNNEL_SUB_TYPE_NONE: TUNNEL_SUB_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TUNNEL_SUB_TYPE_CP: TUNNEL_SUB_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TUNNEL_SUB_TYPE_IPTLS: TUNNEL_SUB_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const TUNNEL_SUB_TYPE_HA: TUNNEL_SUB_TYPE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct VLAN_TAG {
    pub Anonymous: VLAN_TAG_0,
    pub Type: u16,
}
impl ::core::marker::Copy for VLAN_TAG {}
impl ::core::clone::Clone for VLAN_TAG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub union VLAN_TAG_0 {
    pub Tag: u16,
    pub Anonymous: VLAN_TAG_0_0,
}
impl ::core::marker::Copy for VLAN_TAG_0 {}
impl ::core::clone::Clone for VLAN_TAG_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct VLAN_TAG_0_0 {
    pub _bitfield: u16,
}
impl ::core::marker::Copy for VLAN_TAG_0_0 {}
impl ::core::clone::Clone for VLAN_TAG_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const _BIG_ENDIAN: u32 = 4321u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const _LITTLE_ENDIAN: u32 = 1234u32;
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub const _PDP_ENDIAN: u32 = 3412u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_neighbor_advert {
    pub nd_na_hdr: ICMP_MESSAGE,
    pub nd_na_target: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_neighbor_advert {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_neighbor_advert {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_neighbor_solicit {
    pub nd_ns_hdr: ICMP_MESSAGE,
    pub nd_ns_target: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_neighbor_solicit {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_neighbor_solicit {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_opt_dnssl {
    pub nd_opt_dnssl_type: u8,
    pub nd_opt_dnssl_len: u8,
    pub nd_opt_dnssl_reserved: u16,
    pub nd_opt_dnssl_lifetime: u32,
}
impl ::core::marker::Copy for nd_opt_dnssl {}
impl ::core::clone::Clone for nd_opt_dnssl {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_opt_hdr {
    pub nd_opt_type: u8,
    pub nd_opt_len: u8,
}
impl ::core::marker::Copy for nd_opt_hdr {}
impl ::core::clone::Clone for nd_opt_hdr {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_opt_mtu {
    pub nd_opt_mtu_type: u8,
    pub nd_opt_mtu_len: u8,
    pub nd_opt_mtu_reserved: u16,
    pub nd_opt_mtu_mtu: u32,
}
impl ::core::marker::Copy for nd_opt_mtu {}
impl ::core::clone::Clone for nd_opt_mtu {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_opt_prefix_info {
    pub nd_opt_pi_type: u8,
    pub nd_opt_pi_len: u8,
    pub nd_opt_pi_prefix_len: u8,
    pub Anonymous1: nd_opt_prefix_info_0,
    pub nd_opt_pi_valid_time: u32,
    pub nd_opt_pi_preferred_time: u32,
    pub Anonymous2: nd_opt_prefix_info_1,
    pub nd_opt_pi_prefix: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_prefix_info {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_prefix_info {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union nd_opt_prefix_info_0 {
    pub nd_opt_pi_flags_reserved: u8,
    pub Flags: nd_opt_prefix_info_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_prefix_info_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_prefix_info_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_opt_prefix_info_0_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_prefix_info_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_prefix_info_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union nd_opt_prefix_info_1 {
    pub nd_opt_pi_reserved2: u32,
    pub Anonymous: nd_opt_prefix_info_1_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_prefix_info_1 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_prefix_info_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_opt_prefix_info_1_0 {
    pub nd_opt_pi_reserved3: [u8; 3],
    pub nd_opt_pi_site_prefix_len: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_prefix_info_1_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_prefix_info_1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_opt_rd_hdr {
    pub nd_opt_rh_type: u8,
    pub nd_opt_rh_len: u8,
    pub nd_opt_rh_reserved1: u16,
    pub nd_opt_rh_reserved2: u32,
}
impl ::core::marker::Copy for nd_opt_rd_hdr {}
impl ::core::clone::Clone for nd_opt_rd_hdr {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_opt_rdnss {
    pub nd_opt_rdnss_type: u8,
    pub nd_opt_rdnss_len: u8,
    pub nd_opt_rdnss_reserved: u16,
    pub nd_opt_rdnss_lifetime: u32,
}
impl ::core::marker::Copy for nd_opt_rdnss {}
impl ::core::clone::Clone for nd_opt_rdnss {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_opt_route_info {
    pub nd_opt_ri_type: u8,
    pub nd_opt_ri_len: u8,
    pub nd_opt_ri_prefix_len: u8,
    pub Anonymous: nd_opt_route_info_0,
    pub nd_opt_ri_route_lifetime: u32,
    pub nd_opt_ri_prefix: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_route_info {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_route_info {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub union nd_opt_route_info_0 {
    pub nd_opt_ri_flags_reserved: u8,
    pub Flags: nd_opt_route_info_0_0,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_route_info_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_route_info_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_opt_route_info_0_0 {
    pub _bitfield: u8,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_opt_route_info_0_0 {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_opt_route_info_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`, `\"Win32_Networking_WinSock\"`*"]
#[cfg(feature = "Win32_Networking_WinSock")]
pub struct nd_redirect {
    pub nd_rd_hdr: ICMP_MESSAGE,
    pub nd_rd_target: super::super::Networking::WinSock::IN6_ADDR,
    pub nd_rd_dst: super::super::Networking::WinSock::IN6_ADDR,
}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::marker::Copy for nd_redirect {}
#[cfg(feature = "Win32_Networking_WinSock")]
impl ::core::clone::Clone for nd_redirect {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_router_advert {
    pub nd_ra_hdr: ICMP_MESSAGE,
    pub nd_ra_reachable: u32,
    pub nd_ra_retransmit: u32,
}
impl ::core::marker::Copy for nd_router_advert {}
impl ::core::clone::Clone for nd_router_advert {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct nd_router_solicit {
    pub nd_rs_hdr: ICMP_MESSAGE,
}
impl ::core::marker::Copy for nd_router_solicit {}
impl ::core::clone::Clone for nd_router_solicit {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_hdr {
    pub th_sport: u16,
    pub th_dport: u16,
    pub th_seq: u32,
    pub th_ack: u32,
    pub _bitfield: u8,
    pub th_flags: u8,
    pub th_win: u16,
    pub th_sum: u16,
    pub th_urp: u16,
}
impl ::core::marker::Copy for tcp_hdr {}
impl ::core::clone::Clone for tcp_hdr {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_fastopen {
    pub Kind: u8,
    pub Length: u8,
    pub Cookie: [u8; 1],
}
impl ::core::marker::Copy for tcp_opt_fastopen {}
impl ::core::clone::Clone for tcp_opt_fastopen {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_mss {
    pub Kind: u8,
    pub Length: u8,
    pub Mss: u16,
}
impl ::core::marker::Copy for tcp_opt_mss {}
impl ::core::clone::Clone for tcp_opt_mss {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_sack {
    pub Kind: u8,
    pub Length: u8,
    pub Block: [tcp_opt_sack_0; 1],
}
impl ::core::marker::Copy for tcp_opt_sack {}
impl ::core::clone::Clone for tcp_opt_sack {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_sack_0 {
    pub Left: u32,
    pub Right: u32,
}
impl ::core::marker::Copy for tcp_opt_sack_0 {}
impl ::core::clone::Clone for tcp_opt_sack_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_sack_permitted {
    pub Kind: u8,
    pub Length: u8,
}
impl ::core::marker::Copy for tcp_opt_sack_permitted {}
impl ::core::clone::Clone for tcp_opt_sack_permitted {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_ts {
    pub Kind: u8,
    pub Length: u8,
    pub Val: u32,
    pub EcR: u32,
}
impl ::core::marker::Copy for tcp_opt_ts {}
impl ::core::clone::Clone for tcp_opt_ts {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_unknown {
    pub Kind: u8,
    pub Length: u8,
}
impl ::core::marker::Copy for tcp_opt_unknown {}
impl ::core::clone::Clone for tcp_opt_unknown {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_NetworkManagement_WindowsFilteringPlatform\"`*"]
pub struct tcp_opt_ws {
    pub Kind: u8,
    pub Length: u8,
    pub ShiftCnt: u8,
}
impl ::core::marker::Copy for tcp_opt_ws {}
impl ::core::clone::Clone for tcp_opt_ws {
    fn clone(&self) -> Self {
        *self
    }
}
