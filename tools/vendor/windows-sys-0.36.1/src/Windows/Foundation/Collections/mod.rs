#[doc = "*Required features: `\"Foundation_Collections\"`*"]
#[repr(transparent)]
pub struct CollectionChange(pub i32);
impl CollectionChange {
    pub const Reset: Self = Self(0i32);
    pub const ItemInserted: Self = Self(1i32);
    pub const ItemRemoved: Self = Self(2i32);
    pub const ItemChanged: Self = Self(3i32);
}
impl ::core::marker::Copy for CollectionChange {}
impl ::core::clone::Clone for CollectionChange {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IIterable = *mut ::core::ffi::c_void;
pub type IIterator = *mut ::core::ffi::c_void;
pub type IKeyValuePair = *mut ::core::ffi::c_void;
pub type IMap = *mut ::core::ffi::c_void;
pub type IMapChangedEventArgs = *mut ::core::ffi::c_void;
pub type IMapView = *mut ::core::ffi::c_void;
pub type IObservableMap = *mut ::core::ffi::c_void;
pub type IObservableVector = *mut ::core::ffi::c_void;
pub type IPropertySet = *mut ::core::ffi::c_void;
pub type IVector = *mut ::core::ffi::c_void;
pub type IVectorChangedEventArgs = *mut ::core::ffi::c_void;
pub type IVectorView = *mut ::core::ffi::c_void;
pub type MapChangedEventHandler = *mut ::core::ffi::c_void;
pub type PropertySet = *mut ::core::ffi::c_void;
pub type StringMap = *mut ::core::ffi::c_void;
pub type ValueSet = *mut ::core::ffi::c_void;
pub type VectorChangedEventHandler = *mut ::core::ffi::c_void;
