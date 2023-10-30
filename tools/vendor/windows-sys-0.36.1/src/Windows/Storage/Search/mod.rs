#[doc = "*Required features: `\"Storage_Search\"`*"]
#[repr(transparent)]
pub struct CommonFileQuery(pub i32);
impl CommonFileQuery {
    pub const DefaultQuery: Self = Self(0i32);
    pub const OrderByName: Self = Self(1i32);
    pub const OrderByTitle: Self = Self(2i32);
    pub const OrderByMusicProperties: Self = Self(3i32);
    pub const OrderBySearchRank: Self = Self(4i32);
    pub const OrderByDate: Self = Self(5i32);
}
impl ::core::marker::Copy for CommonFileQuery {}
impl ::core::clone::Clone for CommonFileQuery {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Storage_Search\"`*"]
#[repr(transparent)]
pub struct CommonFolderQuery(pub i32);
impl CommonFolderQuery {
    pub const DefaultQuery: Self = Self(0i32);
    pub const GroupByYear: Self = Self(100i32);
    pub const GroupByMonth: Self = Self(101i32);
    pub const GroupByArtist: Self = Self(102i32);
    pub const GroupByAlbum: Self = Self(103i32);
    pub const GroupByAlbumArtist: Self = Self(104i32);
    pub const GroupByComposer: Self = Self(105i32);
    pub const GroupByGenre: Self = Self(106i32);
    pub const GroupByPublishedYear: Self = Self(107i32);
    pub const GroupByRating: Self = Self(108i32);
    pub const GroupByTag: Self = Self(109i32);
    pub const GroupByAuthor: Self = Self(110i32);
    pub const GroupByType: Self = Self(111i32);
}
impl ::core::marker::Copy for CommonFolderQuery {}
impl ::core::clone::Clone for CommonFolderQuery {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ContentIndexer = *mut ::core::ffi::c_void;
pub type ContentIndexerQuery = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Search\"`*"]
#[repr(transparent)]
pub struct DateStackOption(pub i32);
impl DateStackOption {
    pub const None: Self = Self(0i32);
    pub const Year: Self = Self(1i32);
    pub const Month: Self = Self(2i32);
}
impl ::core::marker::Copy for DateStackOption {}
impl ::core::clone::Clone for DateStackOption {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Storage_Search\"`*"]
#[repr(transparent)]
pub struct FolderDepth(pub i32);
impl FolderDepth {
    pub const Shallow: Self = Self(0i32);
    pub const Deep: Self = Self(1i32);
}
impl ::core::marker::Copy for FolderDepth {}
impl ::core::clone::Clone for FolderDepth {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IIndexableContent = *mut ::core::ffi::c_void;
pub type IStorageFolderQueryOperations = *mut ::core::ffi::c_void;
pub type IStorageQueryResultBase = *mut ::core::ffi::c_void;
pub type IndexableContent = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Search\"`*"]
#[repr(transparent)]
pub struct IndexedState(pub i32);
impl IndexedState {
    pub const Unknown: Self = Self(0i32);
    pub const NotIndexed: Self = Self(1i32);
    pub const PartiallyIndexed: Self = Self(2i32);
    pub const FullyIndexed: Self = Self(3i32);
}
impl ::core::marker::Copy for IndexedState {}
impl ::core::clone::Clone for IndexedState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Storage_Search\"`*"]
#[repr(transparent)]
pub struct IndexerOption(pub i32);
impl IndexerOption {
    pub const UseIndexerWhenAvailable: Self = Self(0i32);
    pub const OnlyUseIndexer: Self = Self(1i32);
    pub const DoNotUseIndexer: Self = Self(2i32);
    pub const OnlyUseIndexerAndOptimizeForIndexedProperties: Self = Self(3i32);
}
impl ::core::marker::Copy for IndexerOption {}
impl ::core::clone::Clone for IndexerOption {
    fn clone(&self) -> Self {
        *self
    }
}
pub type QueryOptions = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Storage_Search\"`*"]
pub struct SortEntry {
    pub PropertyName: ::windows_sys::core::HSTRING,
    pub AscendingOrder: bool,
}
impl ::core::marker::Copy for SortEntry {}
impl ::core::clone::Clone for SortEntry {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SortEntryVector = *mut ::core::ffi::c_void;
pub type StorageFileQueryResult = *mut ::core::ffi::c_void;
pub type StorageFolderQueryResult = *mut ::core::ffi::c_void;
pub type StorageItemQueryResult = *mut ::core::ffi::c_void;
pub type StorageLibraryChangeTrackerTriggerDetails = *mut ::core::ffi::c_void;
pub type StorageLibraryContentChangedTriggerDetails = *mut ::core::ffi::c_void;
pub type ValueAndLanguage = *mut ::core::ffi::c_void;
