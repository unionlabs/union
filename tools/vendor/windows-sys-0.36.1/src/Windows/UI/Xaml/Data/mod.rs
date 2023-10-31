pub type Binding = *mut ::core::ffi::c_void;
pub type BindingBase = *mut ::core::ffi::c_void;
pub type BindingExpression = *mut ::core::ffi::c_void;
pub type BindingExpressionBase = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Data\"`*"]
#[repr(transparent)]
pub struct BindingMode(pub i32);
impl BindingMode {
    pub const OneWay: Self = Self(1i32);
    pub const OneTime: Self = Self(2i32);
    pub const TwoWay: Self = Self(3i32);
}
impl ::core::marker::Copy for BindingMode {}
impl ::core::clone::Clone for BindingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BindingOperations = *mut ::core::ffi::c_void;
pub type CollectionViewSource = *mut ::core::ffi::c_void;
pub type CurrentChangingEventArgs = *mut ::core::ffi::c_void;
pub type CurrentChangingEventHandler = *mut ::core::ffi::c_void;
pub type ICollectionView = *mut ::core::ffi::c_void;
pub type ICollectionViewFactory = *mut ::core::ffi::c_void;
pub type ICollectionViewGroup = *mut ::core::ffi::c_void;
pub type ICustomProperty = *mut ::core::ffi::c_void;
pub type ICustomPropertyProvider = *mut ::core::ffi::c_void;
pub type IItemsRangeInfo = *mut ::core::ffi::c_void;
pub type INotifyPropertyChanged = *mut ::core::ffi::c_void;
pub type ISelectionInfo = *mut ::core::ffi::c_void;
pub type ISupportIncrementalLoading = *mut ::core::ffi::c_void;
pub type IValueConverter = *mut ::core::ffi::c_void;
pub type ItemIndexRange = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Data\"`*"]
pub struct LoadMoreItemsResult {
    pub Count: u32,
}
impl ::core::marker::Copy for LoadMoreItemsResult {}
impl ::core::clone::Clone for LoadMoreItemsResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PropertyChangedEventArgs = *mut ::core::ffi::c_void;
pub type PropertyChangedEventHandler = *mut ::core::ffi::c_void;
pub type RelativeSource = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Data\"`*"]
#[repr(transparent)]
pub struct RelativeSourceMode(pub i32);
impl RelativeSourceMode {
    pub const None: Self = Self(0i32);
    pub const TemplatedParent: Self = Self(1i32);
    pub const Self_: Self = Self(2i32);
}
impl ::core::marker::Copy for RelativeSourceMode {}
impl ::core::clone::Clone for RelativeSourceMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Data\"`*"]
#[repr(transparent)]
pub struct UpdateSourceTrigger(pub i32);
impl UpdateSourceTrigger {
    pub const Default: Self = Self(0i32);
    pub const PropertyChanged: Self = Self(1i32);
    pub const Explicit: Self = Self(2i32);
    pub const LostFocus: Self = Self(3i32);
}
impl ::core::marker::Copy for UpdateSourceTrigger {}
impl ::core::clone::Clone for UpdateSourceTrigger {
    fn clone(&self) -> Self {
        *self
    }
}
