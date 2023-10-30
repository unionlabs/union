#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct AttributeTargets(pub u32);
impl AttributeTargets {
    pub const All: Self = Self(4294967295u32);
    pub const Delegate: Self = Self(1u32);
    pub const Enum: Self = Self(2u32);
    pub const Event: Self = Self(4u32);
    pub const Field: Self = Self(8u32);
    pub const Interface: Self = Self(16u32);
    pub const Method: Self = Self(64u32);
    pub const Parameter: Self = Self(128u32);
    pub const Property: Self = Self(256u32);
    pub const RuntimeClass: Self = Self(512u32);
    pub const Struct: Self = Self(1024u32);
    pub const InterfaceImpl: Self = Self(2048u32);
    pub const ApiContract: Self = Self(8192u32);
}
impl ::core::marker::Copy for AttributeTargets {}
impl ::core::clone::Clone for AttributeTargets {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct CompositionType(pub i32);
impl CompositionType {
    pub const Protected: Self = Self(1i32);
    pub const Public: Self = Self(2i32);
}
impl ::core::marker::Copy for CompositionType {}
impl ::core::clone::Clone for CompositionType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct DeprecationType(pub i32);
impl DeprecationType {
    pub const Deprecate: Self = Self(0i32);
    pub const Remove: Self = Self(1i32);
}
impl ::core::marker::Copy for DeprecationType {}
impl ::core::clone::Clone for DeprecationType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct FeatureStage(pub i32);
impl FeatureStage {
    pub const AlwaysDisabled: Self = Self(0i32);
    pub const DisabledByDefault: Self = Self(1i32);
    pub const EnabledByDefault: Self = Self(2i32);
    pub const AlwaysEnabled: Self = Self(3i32);
}
impl ::core::marker::Copy for FeatureStage {}
impl ::core::clone::Clone for FeatureStage {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct GCPressureAmount(pub i32);
impl GCPressureAmount {
    pub const Low: Self = Self(0i32);
    pub const Medium: Self = Self(1i32);
    pub const High: Self = Self(2i32);
}
impl ::core::marker::Copy for GCPressureAmount {}
impl ::core::clone::Clone for GCPressureAmount {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct MarshalingType(pub i32);
impl MarshalingType {
    pub const None: Self = Self(1i32);
    pub const Agile: Self = Self(2i32);
    pub const Standard: Self = Self(3i32);
    pub const InvalidMarshaling: Self = Self(0i32);
}
impl ::core::marker::Copy for MarshalingType {}
impl ::core::clone::Clone for MarshalingType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct Platform(pub i32);
impl Platform {
    pub const Windows: Self = Self(0i32);
    pub const WindowsPhone: Self = Self(1i32);
}
impl ::core::marker::Copy for Platform {}
impl ::core::clone::Clone for Platform {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation_Metadata\"`*"]
#[repr(transparent)]
pub struct ThreadingModel(pub i32);
impl ThreadingModel {
    pub const STA: Self = Self(1i32);
    pub const MTA: Self = Self(2i32);
    pub const Both: Self = Self(3i32);
    pub const InvalidThreading: Self = Self(0i32);
}
impl ::core::marker::Copy for ThreadingModel {}
impl ::core::clone::Clone for ThreadingModel {
    fn clone(&self) -> Self {
        *self
    }
}
