pub const PERCEPTIONFIELD_StateStream_TimeStamps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2861064473, data2: 62255, data3: 18879, data4: [146, 202, 249, 221, 247, 132, 210, 151] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_MixedReality\"`*"]
pub struct PERCEPTION_PAYLOAD_FIELD {
    pub FieldId: ::windows_sys::core::GUID,
    pub OffsetInBytes: u32,
    pub SizeInBytes: u32,
}
impl ::core::marker::Copy for PERCEPTION_PAYLOAD_FIELD {}
impl ::core::clone::Clone for PERCEPTION_PAYLOAD_FIELD {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_MixedReality\"`*"]
pub struct PERCEPTION_STATE_STREAM_TIMESTAMPS {
    pub InputTimestampInQpcCounts: i64,
    pub AvailableTimestampInQpcCounts: i64,
}
impl ::core::marker::Copy for PERCEPTION_STATE_STREAM_TIMESTAMPS {}
impl ::core::clone::Clone for PERCEPTION_STATE_STREAM_TIMESTAMPS {
    fn clone(&self) -> Self {
        *self
    }
}
