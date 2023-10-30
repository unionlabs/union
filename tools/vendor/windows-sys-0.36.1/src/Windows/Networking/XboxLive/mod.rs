pub type XboxLiveDeviceAddress = *mut ::core::ffi::c_void;
pub type XboxLiveEndpointPair = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveEndpointPairCreationBehaviors(pub u32);
impl XboxLiveEndpointPairCreationBehaviors {
    pub const None: Self = Self(0u32);
    pub const ReevaluatePath: Self = Self(1u32);
}
impl ::core::marker::Copy for XboxLiveEndpointPairCreationBehaviors {}
impl ::core::clone::Clone for XboxLiveEndpointPairCreationBehaviors {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XboxLiveEndpointPairCreationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveEndpointPairCreationStatus(pub i32);
impl XboxLiveEndpointPairCreationStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const NoLocalNetworks: Self = Self(1i32);
    pub const NoCompatibleNetworkPaths: Self = Self(2i32);
    pub const LocalSystemNotAuthorized: Self = Self(3i32);
    pub const Canceled: Self = Self(4i32);
    pub const TimedOut: Self = Self(5i32);
    pub const RemoteSystemNotAuthorized: Self = Self(6i32);
    pub const RefusedDueToConfiguration: Self = Self(7i32);
    pub const UnexpectedInternalError: Self = Self(8i32);
}
impl ::core::marker::Copy for XboxLiveEndpointPairCreationStatus {}
impl ::core::clone::Clone for XboxLiveEndpointPairCreationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveEndpointPairState(pub i32);
impl XboxLiveEndpointPairState {
    pub const Invalid: Self = Self(0i32);
    pub const CreatingOutbound: Self = Self(1i32);
    pub const CreatingInbound: Self = Self(2i32);
    pub const Ready: Self = Self(3i32);
    pub const DeletingLocally: Self = Self(4i32);
    pub const RemoteEndpointTerminating: Self = Self(5i32);
    pub const Deleted: Self = Self(6i32);
}
impl ::core::marker::Copy for XboxLiveEndpointPairState {}
impl ::core::clone::Clone for XboxLiveEndpointPairState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XboxLiveEndpointPairStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type XboxLiveEndpointPairTemplate = *mut ::core::ffi::c_void;
pub type XboxLiveInboundEndpointPairCreatedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveNetworkAccessKind(pub i32);
impl XboxLiveNetworkAccessKind {
    pub const Open: Self = Self(0i32);
    pub const Moderate: Self = Self(1i32);
    pub const Strict: Self = Self(2i32);
}
impl ::core::marker::Copy for XboxLiveNetworkAccessKind {}
impl ::core::clone::Clone for XboxLiveNetworkAccessKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XboxLiveQualityOfServiceMeasurement = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveQualityOfServiceMeasurementStatus(pub i32);
impl XboxLiveQualityOfServiceMeasurementStatus {
    pub const NotStarted: Self = Self(0i32);
    pub const InProgress: Self = Self(1i32);
    pub const InProgressWithProvisionalResults: Self = Self(2i32);
    pub const Succeeded: Self = Self(3i32);
    pub const NoLocalNetworks: Self = Self(4i32);
    pub const NoCompatibleNetworkPaths: Self = Self(5i32);
    pub const LocalSystemNotAuthorized: Self = Self(6i32);
    pub const Canceled: Self = Self(7i32);
    pub const TimedOut: Self = Self(8i32);
    pub const RemoteSystemNotAuthorized: Self = Self(9i32);
    pub const RefusedDueToConfiguration: Self = Self(10i32);
    pub const UnexpectedInternalError: Self = Self(11i32);
}
impl ::core::marker::Copy for XboxLiveQualityOfServiceMeasurementStatus {}
impl ::core::clone::Clone for XboxLiveQualityOfServiceMeasurementStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveQualityOfServiceMetric(pub i32);
impl XboxLiveQualityOfServiceMetric {
    pub const AverageLatencyInMilliseconds: Self = Self(0i32);
    pub const MinLatencyInMilliseconds: Self = Self(1i32);
    pub const MaxLatencyInMilliseconds: Self = Self(2i32);
    pub const AverageOutboundBitsPerSecond: Self = Self(3i32);
    pub const MinOutboundBitsPerSecond: Self = Self(4i32);
    pub const MaxOutboundBitsPerSecond: Self = Self(5i32);
    pub const AverageInboundBitsPerSecond: Self = Self(6i32);
    pub const MinInboundBitsPerSecond: Self = Self(7i32);
    pub const MaxInboundBitsPerSecond: Self = Self(8i32);
}
impl ::core::marker::Copy for XboxLiveQualityOfServiceMetric {}
impl ::core::clone::Clone for XboxLiveQualityOfServiceMetric {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XboxLiveQualityOfServiceMetricResult = *mut ::core::ffi::c_void;
pub type XboxLiveQualityOfServicePrivatePayloadResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_XboxLive\"`*"]
#[repr(transparent)]
pub struct XboxLiveSocketKind(pub i32);
impl XboxLiveSocketKind {
    pub const None: Self = Self(0i32);
    pub const Datagram: Self = Self(1i32);
    pub const Stream: Self = Self(2i32);
}
impl ::core::marker::Copy for XboxLiveSocketKind {}
impl ::core::clone::Clone for XboxLiveSocketKind {
    fn clone(&self) -> Self {
        *self
    }
}
