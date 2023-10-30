pub type IVpnChannelStatics = *mut ::core::ffi::c_void;
pub type IVpnCredential = *mut ::core::ffi::c_void;
pub type IVpnCustomPrompt = *mut ::core::ffi::c_void;
pub type IVpnCustomPromptElement = *mut ::core::ffi::c_void;
pub type IVpnDomainNameInfoFactory = *mut ::core::ffi::c_void;
pub type IVpnInterfaceIdFactory = *mut ::core::ffi::c_void;
pub type IVpnNamespaceInfoFactory = *mut ::core::ffi::c_void;
pub type IVpnPacketBufferFactory = *mut ::core::ffi::c_void;
pub type IVpnPlugIn = *mut ::core::ffi::c_void;
pub type IVpnProfile = *mut ::core::ffi::c_void;
pub type IVpnRouteFactory = *mut ::core::ffi::c_void;
pub type VpnAppId = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnAppIdType(pub i32);
impl VpnAppIdType {
    pub const PackageFamilyName: Self = Self(0i32);
    pub const FullyQualifiedBinaryName: Self = Self(1i32);
    pub const FilePath: Self = Self(2i32);
}
impl ::core::marker::Copy for VpnAppIdType {}
impl ::core::clone::Clone for VpnAppIdType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnAuthenticationMethod(pub i32);
impl VpnAuthenticationMethod {
    pub const Mschapv2: Self = Self(0i32);
    pub const Eap: Self = Self(1i32);
    pub const Certificate: Self = Self(2i32);
    pub const PresharedKey: Self = Self(3i32);
}
impl ::core::marker::Copy for VpnAuthenticationMethod {}
impl ::core::clone::Clone for VpnAuthenticationMethod {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnChannel = *mut ::core::ffi::c_void;
pub type VpnChannelActivityEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnChannelActivityEventType(pub i32);
impl VpnChannelActivityEventType {
    pub const Idle: Self = Self(0i32);
    pub const Active: Self = Self(1i32);
}
impl ::core::marker::Copy for VpnChannelActivityEventType {}
impl ::core::clone::Clone for VpnChannelActivityEventType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnChannelActivityStateChangedArgs = *mut ::core::ffi::c_void;
pub type VpnChannelConfiguration = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnChannelRequestCredentialsOptions(pub u32);
impl VpnChannelRequestCredentialsOptions {
    pub const None: Self = Self(0u32);
    pub const Retrying: Self = Self(1u32);
    pub const UseForSingleSignIn: Self = Self(2u32);
}
impl ::core::marker::Copy for VpnChannelRequestCredentialsOptions {}
impl ::core::clone::Clone for VpnChannelRequestCredentialsOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnCredential = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnCredentialType(pub i32);
impl VpnCredentialType {
    pub const UsernamePassword: Self = Self(0i32);
    pub const UsernameOtpPin: Self = Self(1i32);
    pub const UsernamePasswordAndPin: Self = Self(2i32);
    pub const UsernamePasswordChange: Self = Self(3i32);
    pub const SmartCard: Self = Self(4i32);
    pub const ProtectedCertificate: Self = Self(5i32);
    pub const UnProtectedCertificate: Self = Self(6i32);
}
impl ::core::marker::Copy for VpnCredentialType {}
impl ::core::clone::Clone for VpnCredentialType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnCustomCheckBox = *mut ::core::ffi::c_void;
pub type VpnCustomComboBox = *mut ::core::ffi::c_void;
pub type VpnCustomEditBox = *mut ::core::ffi::c_void;
pub type VpnCustomErrorBox = *mut ::core::ffi::c_void;
pub type VpnCustomPromptBooleanInput = *mut ::core::ffi::c_void;
pub type VpnCustomPromptOptionSelector = *mut ::core::ffi::c_void;
pub type VpnCustomPromptText = *mut ::core::ffi::c_void;
pub type VpnCustomPromptTextInput = *mut ::core::ffi::c_void;
pub type VpnCustomTextBox = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnDataPathType(pub i32);
impl VpnDataPathType {
    pub const Send: Self = Self(0i32);
    pub const Receive: Self = Self(1i32);
}
impl ::core::marker::Copy for VpnDataPathType {}
impl ::core::clone::Clone for VpnDataPathType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnDomainNameAssignment = *mut ::core::ffi::c_void;
pub type VpnDomainNameInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnDomainNameType(pub i32);
impl VpnDomainNameType {
    pub const Suffix: Self = Self(0i32);
    pub const FullyQualified: Self = Self(1i32);
    pub const Reserved: Self = Self(65535i32);
}
impl ::core::marker::Copy for VpnDomainNameType {}
impl ::core::clone::Clone for VpnDomainNameType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnForegroundActivatedEventArgs = *mut ::core::ffi::c_void;
pub type VpnForegroundActivationOperation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnIPProtocol(pub i32);
impl VpnIPProtocol {
    pub const None: Self = Self(0i32);
    pub const Tcp: Self = Self(6i32);
    pub const Udp: Self = Self(17i32);
    pub const Icmp: Self = Self(1i32);
    pub const Ipv6Icmp: Self = Self(58i32);
    pub const Igmp: Self = Self(2i32);
    pub const Pgm: Self = Self(113i32);
}
impl ::core::marker::Copy for VpnIPProtocol {}
impl ::core::clone::Clone for VpnIPProtocol {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnInterfaceId = *mut ::core::ffi::c_void;
pub type VpnManagementAgent = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnManagementConnectionStatus(pub i32);
impl VpnManagementConnectionStatus {
    pub const Disconnected: Self = Self(0i32);
    pub const Disconnecting: Self = Self(1i32);
    pub const Connected: Self = Self(2i32);
    pub const Connecting: Self = Self(3i32);
}
impl ::core::marker::Copy for VpnManagementConnectionStatus {}
impl ::core::clone::Clone for VpnManagementConnectionStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnManagementErrorStatus(pub i32);
impl VpnManagementErrorStatus {
    pub const Ok: Self = Self(0i32);
    pub const Other: Self = Self(1i32);
    pub const InvalidXmlSyntax: Self = Self(2i32);
    pub const ProfileNameTooLong: Self = Self(3i32);
    pub const ProfileInvalidAppId: Self = Self(4i32);
    pub const AccessDenied: Self = Self(5i32);
    pub const CannotFindProfile: Self = Self(6i32);
    pub const AlreadyDisconnecting: Self = Self(7i32);
    pub const AlreadyConnected: Self = Self(8i32);
    pub const GeneralAuthenticationFailure: Self = Self(9i32);
    pub const EapFailure: Self = Self(10i32);
    pub const SmartCardFailure: Self = Self(11i32);
    pub const CertificateFailure: Self = Self(12i32);
    pub const ServerConfiguration: Self = Self(13i32);
    pub const NoConnection: Self = Self(14i32);
    pub const ServerConnection: Self = Self(15i32);
    pub const UserNamePassword: Self = Self(16i32);
    pub const DnsNotResolvable: Self = Self(17i32);
    pub const InvalidIP: Self = Self(18i32);
}
impl ::core::marker::Copy for VpnManagementErrorStatus {}
impl ::core::clone::Clone for VpnManagementErrorStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnNamespaceAssignment = *mut ::core::ffi::c_void;
pub type VpnNamespaceInfo = *mut ::core::ffi::c_void;
pub type VpnNativeProfile = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnNativeProtocolType(pub i32);
impl VpnNativeProtocolType {
    pub const Pptp: Self = Self(0i32);
    pub const L2tp: Self = Self(1i32);
    pub const IpsecIkev2: Self = Self(2i32);
}
impl ::core::marker::Copy for VpnNativeProtocolType {}
impl ::core::clone::Clone for VpnNativeProtocolType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnPacketBuffer = *mut ::core::ffi::c_void;
pub type VpnPacketBufferList = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnPacketBufferStatus(pub i32);
impl VpnPacketBufferStatus {
    pub const Ok: Self = Self(0i32);
    pub const InvalidBufferSize: Self = Self(1i32);
}
impl ::core::marker::Copy for VpnPacketBufferStatus {}
impl ::core::clone::Clone for VpnPacketBufferStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnPickedCredential = *mut ::core::ffi::c_void;
pub type VpnPlugInProfile = *mut ::core::ffi::c_void;
pub type VpnRoute = *mut ::core::ffi::c_void;
pub type VpnRouteAssignment = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_Vpn\"`*"]
#[repr(transparent)]
pub struct VpnRoutingPolicyType(pub i32);
impl VpnRoutingPolicyType {
    pub const SplitRouting: Self = Self(0i32);
    pub const ForceAllTrafficOverVpn: Self = Self(1i32);
}
impl ::core::marker::Copy for VpnRoutingPolicyType {}
impl ::core::clone::Clone for VpnRoutingPolicyType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VpnSystemHealth = *mut ::core::ffi::c_void;
pub type VpnTrafficFilter = *mut ::core::ffi::c_void;
pub type VpnTrafficFilterAssignment = *mut ::core::ffi::c_void;
