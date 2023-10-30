pub type IWaaSAssessor = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct OSUpdateAssessment {
    pub isEndOfSupport: super::super::Foundation::BOOL,
    pub assessmentForCurrent: UpdateAssessment,
    pub assessmentForUpToDate: UpdateAssessment,
    pub securityStatus: UpdateAssessmentStatus,
    pub assessmentTime: super::super::Foundation::FILETIME,
    pub releaseInfoTime: super::super::Foundation::FILETIME,
    pub currentOSBuild: ::windows_sys::core::PWSTR,
    pub currentOSReleaseTime: super::super::Foundation::FILETIME,
    pub upToDateOSBuild: ::windows_sys::core::PWSTR,
    pub upToDateOSReleaseTime: super::super::Foundation::FILETIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for OSUpdateAssessment {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for OSUpdateAssessment {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub struct UpdateAssessment {
    pub status: UpdateAssessmentStatus,
    pub impact: UpdateImpactLevel,
    pub daysOutOfDate: u32,
}
impl ::core::marker::Copy for UpdateAssessment {}
impl ::core::clone::Clone for UpdateAssessment {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub type UpdateAssessmentStatus = i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_Latest: UpdateAssessmentStatus = 0i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestSoftRestriction: UpdateAssessmentStatus = 1i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestHardRestriction: UpdateAssessmentStatus = 2i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestEndOfSupport: UpdateAssessmentStatus = 3i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestServicingTrain: UpdateAssessmentStatus = 4i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestDeferredFeature: UpdateAssessmentStatus = 5i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestDeferredQuality: UpdateAssessmentStatus = 6i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestPausedFeature: UpdateAssessmentStatus = 7i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestPausedQuality: UpdateAssessmentStatus = 8i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestManaged: UpdateAssessmentStatus = 9i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestUnknown: UpdateAssessmentStatus = 10i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateAssessmentStatus_NotLatestTargetedVersion: UpdateAssessmentStatus = 11i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub type UpdateImpactLevel = i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateImpactLevel_None: UpdateImpactLevel = 0i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateImpactLevel_Low: UpdateImpactLevel = 1i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateImpactLevel_Medium: UpdateImpactLevel = 2i32;
#[doc = "*Required features: `\"Win32_System_UpdateAssessment\"`*"]
pub const UpdateImpactLevel_High: UpdateImpactLevel = 3i32;
pub const WaaSAssessor: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 160364657, data2: 64159, data3: 18095, data4: [137, 88, 192, 131, 81, 93, 124, 156] };
