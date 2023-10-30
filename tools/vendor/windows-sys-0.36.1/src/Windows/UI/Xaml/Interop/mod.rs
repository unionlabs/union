pub type BindableVectorChangedEventHandler = *mut ::core::ffi::c_void;
pub type IBindableIterable = *mut ::core::ffi::c_void;
pub type IBindableIterator = *mut ::core::ffi::c_void;
pub type IBindableObservableVector = *mut ::core::ffi::c_void;
pub type IBindableVector = *mut ::core::ffi::c_void;
pub type IBindableVectorView = *mut ::core::ffi::c_void;
pub type INotifyCollectionChanged = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Interop\"`*"]
#[repr(transparent)]
pub struct NotifyCollectionChangedAction(pub i32);
impl NotifyCollectionChangedAction {
    pub const Add: Self = Self(0i32);
    pub const Remove: Self = Self(1i32);
    pub const Replace: Self = Self(2i32);
    pub const Move: Self = Self(3i32);
    pub const Reset: Self = Self(4i32);
}
impl ::core::marker::Copy for NotifyCollectionChangedAction {}
impl ::core::clone::Clone for NotifyCollectionChangedAction {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NotifyCollectionChangedEventArgs = *mut ::core::ffi::c_void;
pub type NotifyCollectionChangedEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Interop\"`*"]
#[repr(transparent)]
pub struct TypeKind(pub i32);
impl TypeKind {
    pub const Primitive: Self = Self(0i32);
    pub const Metadata: Self = Self(1i32);
    pub const Custom: Self = Self(2i32);
}
impl ::core::marker::Copy for TypeKind {}
impl ::core::clone::Clone for TypeKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Interop\"`*"]
pub struct TypeName {
    pub Name: ::windows_sys::core::HSTRING,
    pub Kind: TypeKind,
}
impl ::core::marker::Copy for TypeName {}
impl ::core::clone::Clone for TypeName {
    fn clone(&self) -> Self {
        *self
    }
}
