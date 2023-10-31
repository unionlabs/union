#[cfg(feature = "ApplicationModel_UserDataTasks_DataProvider")]
pub mod DataProvider;
pub type UserDataTask = *mut ::core::ffi::c_void;
pub type UserDataTaskBatch = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskDaysOfWeek(pub u32);
impl UserDataTaskDaysOfWeek {
    pub const None: Self = Self(0u32);
    pub const Sunday: Self = Self(1u32);
    pub const Monday: Self = Self(2u32);
    pub const Tuesday: Self = Self(4u32);
    pub const Wednesday: Self = Self(8u32);
    pub const Thursday: Self = Self(16u32);
    pub const Friday: Self = Self(32u32);
    pub const Saturday: Self = Self(64u32);
}
impl ::core::marker::Copy for UserDataTaskDaysOfWeek {}
impl ::core::clone::Clone for UserDataTaskDaysOfWeek {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskDetailsKind(pub i32);
impl UserDataTaskDetailsKind {
    pub const PlainText: Self = Self(0i32);
    pub const Html: Self = Self(1i32);
}
impl ::core::marker::Copy for UserDataTaskDetailsKind {}
impl ::core::clone::Clone for UserDataTaskDetailsKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskKind(pub i32);
impl UserDataTaskKind {
    pub const Single: Self = Self(0i32);
    pub const Recurring: Self = Self(1i32);
    pub const Regenerating: Self = Self(2i32);
}
impl ::core::marker::Copy for UserDataTaskKind {}
impl ::core::clone::Clone for UserDataTaskKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskList = *mut ::core::ffi::c_void;
pub type UserDataTaskListLimitedWriteOperations = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskListOtherAppReadAccess(pub i32);
impl UserDataTaskListOtherAppReadAccess {
    pub const Full: Self = Self(0i32);
    pub const SystemOnly: Self = Self(1i32);
    pub const None: Self = Self(2i32);
}
impl ::core::marker::Copy for UserDataTaskListOtherAppReadAccess {}
impl ::core::clone::Clone for UserDataTaskListOtherAppReadAccess {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskListOtherAppWriteAccess(pub i32);
impl UserDataTaskListOtherAppWriteAccess {
    pub const Limited: Self = Self(0i32);
    pub const None: Self = Self(1i32);
}
impl ::core::marker::Copy for UserDataTaskListOtherAppWriteAccess {}
impl ::core::clone::Clone for UserDataTaskListOtherAppWriteAccess {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskListSyncManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskListSyncStatus(pub i32);
impl UserDataTaskListSyncStatus {
    pub const Idle: Self = Self(0i32);
    pub const Syncing: Self = Self(1i32);
    pub const UpToDate: Self = Self(2i32);
    pub const AuthenticationError: Self = Self(3i32);
    pub const PolicyError: Self = Self(4i32);
    pub const UnknownError: Self = Self(5i32);
}
impl ::core::marker::Copy for UserDataTaskListSyncStatus {}
impl ::core::clone::Clone for UserDataTaskListSyncStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskPriority(pub i32);
impl UserDataTaskPriority {
    pub const Normal: Self = Self(0i32);
    pub const Low: Self = Self(-1i32);
    pub const High: Self = Self(1i32);
}
impl ::core::marker::Copy for UserDataTaskPriority {}
impl ::core::clone::Clone for UserDataTaskPriority {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskQueryKind(pub i32);
impl UserDataTaskQueryKind {
    pub const All: Self = Self(0i32);
    pub const Incomplete: Self = Self(1i32);
    pub const Complete: Self = Self(2i32);
}
impl ::core::marker::Copy for UserDataTaskQueryKind {}
impl ::core::clone::Clone for UserDataTaskQueryKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskQueryOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskQuerySortProperty(pub i32);
impl UserDataTaskQuerySortProperty {
    pub const DueDate: Self = Self(0i32);
}
impl ::core::marker::Copy for UserDataTaskQuerySortProperty {}
impl ::core::clone::Clone for UserDataTaskQuerySortProperty {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskReader = *mut ::core::ffi::c_void;
pub type UserDataTaskRecurrenceProperties = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskRecurrenceUnit(pub i32);
impl UserDataTaskRecurrenceUnit {
    pub const Daily: Self = Self(0i32);
    pub const Weekly: Self = Self(1i32);
    pub const Monthly: Self = Self(2i32);
    pub const MonthlyOnDay: Self = Self(3i32);
    pub const Yearly: Self = Self(4i32);
    pub const YearlyOnDay: Self = Self(5i32);
}
impl ::core::marker::Copy for UserDataTaskRecurrenceUnit {}
impl ::core::clone::Clone for UserDataTaskRecurrenceUnit {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskRegenerationProperties = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskRegenerationUnit(pub i32);
impl UserDataTaskRegenerationUnit {
    pub const Daily: Self = Self(0i32);
    pub const Weekly: Self = Self(1i32);
    pub const Monthly: Self = Self(2i32);
    pub const Yearly: Self = Self(4i32);
}
impl ::core::marker::Copy for UserDataTaskRegenerationUnit {}
impl ::core::clone::Clone for UserDataTaskRegenerationUnit {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskSensitivity(pub i32);
impl UserDataTaskSensitivity {
    pub const Public: Self = Self(0i32);
    pub const Private: Self = Self(1i32);
}
impl ::core::marker::Copy for UserDataTaskSensitivity {}
impl ::core::clone::Clone for UserDataTaskSensitivity {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataTaskStore = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskStoreAccessType(pub i32);
impl UserDataTaskStoreAccessType {
    pub const AppTasksReadWrite: Self = Self(0i32);
    pub const AllTasksLimitedReadWrite: Self = Self(1i32);
}
impl ::core::marker::Copy for UserDataTaskStoreAccessType {}
impl ::core::clone::Clone for UserDataTaskStoreAccessType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataTasks\"`*"]
#[repr(transparent)]
pub struct UserDataTaskWeekOfMonth(pub i32);
impl UserDataTaskWeekOfMonth {
    pub const First: Self = Self(0i32);
    pub const Second: Self = Self(1i32);
    pub const Third: Self = Self(2i32);
    pub const Fourth: Self = Self(3i32);
    pub const Last: Self = Self(4i32);
}
impl ::core::marker::Copy for UserDataTaskWeekOfMonth {}
impl ::core::clone::Clone for UserDataTaskWeekOfMonth {
    fn clone(&self) -> Self {
        *self
    }
}
