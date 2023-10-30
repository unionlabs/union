pub type ActivitySensorTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct AlarmAccessStatus(pub i32);
impl AlarmAccessStatus {
    pub const Unspecified: Self = Self(0i32);
    pub const AllowedWithWakeupCapability: Self = Self(1i32);
    pub const AllowedWithoutWakeupCapability: Self = Self(2i32);
    pub const Denied: Self = Self(3i32);
}
impl ::core::marker::Copy for AlarmAccessStatus {}
impl ::core::clone::Clone for AlarmAccessStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppBroadcastTrigger = *mut ::core::ffi::c_void;
pub type AppBroadcastTriggerProviderInfo = *mut ::core::ffi::c_void;
pub type ApplicationTrigger = *mut ::core::ffi::c_void;
pub type ApplicationTriggerDetails = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct ApplicationTriggerResult(pub i32);
impl ApplicationTriggerResult {
    pub const Allowed: Self = Self(0i32);
    pub const CurrentlyRunning: Self = Self(1i32);
    pub const DisabledByPolicy: Self = Self(2i32);
    pub const UnknownError: Self = Self(3i32);
}
impl ::core::marker::Copy for ApplicationTriggerResult {}
impl ::core::clone::Clone for ApplicationTriggerResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppointmentStoreNotificationTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct BackgroundAccessRequestKind(pub i32);
impl BackgroundAccessRequestKind {
    pub const AlwaysAllowed: Self = Self(0i32);
    pub const AllowedSubjectToSystemPolicy: Self = Self(1i32);
}
impl ::core::marker::Copy for BackgroundAccessRequestKind {}
impl ::core::clone::Clone for BackgroundAccessRequestKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct BackgroundAccessStatus(pub i32);
impl BackgroundAccessStatus {
    pub const Unspecified: Self = Self(0i32);
    pub const AllowedWithAlwaysOnRealTimeConnectivity: Self = Self(1i32);
    pub const AllowedMayUseActiveRealTimeConnectivity: Self = Self(2i32);
    pub const Denied: Self = Self(3i32);
    pub const AlwaysAllowed: Self = Self(4i32);
    pub const AllowedSubjectToSystemPolicy: Self = Self(5i32);
    pub const DeniedBySystemPolicy: Self = Self(6i32);
    pub const DeniedByUser: Self = Self(7i32);
}
impl ::core::marker::Copy for BackgroundAccessStatus {}
impl ::core::clone::Clone for BackgroundAccessStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BackgroundTaskBuilder = *mut ::core::ffi::c_void;
pub type BackgroundTaskCanceledEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct BackgroundTaskCancellationReason(pub i32);
impl BackgroundTaskCancellationReason {
    pub const Abort: Self = Self(0i32);
    pub const Terminating: Self = Self(1i32);
    pub const LoggingOff: Self = Self(2i32);
    pub const ServicingUpdate: Self = Self(3i32);
    pub const IdleTask: Self = Self(4i32);
    pub const Uninstall: Self = Self(5i32);
    pub const ConditionLoss: Self = Self(6i32);
    pub const SystemPolicy: Self = Self(7i32);
    pub const QuietHoursEntered: Self = Self(8i32);
    pub const ExecutionTimeExceeded: Self = Self(9i32);
    pub const ResourceRevocation: Self = Self(10i32);
    pub const EnergySaver: Self = Self(11i32);
}
impl ::core::marker::Copy for BackgroundTaskCancellationReason {}
impl ::core::clone::Clone for BackgroundTaskCancellationReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BackgroundTaskCompletedEventArgs = *mut ::core::ffi::c_void;
pub type BackgroundTaskCompletedEventHandler = *mut ::core::ffi::c_void;
pub type BackgroundTaskDeferral = *mut ::core::ffi::c_void;
pub type BackgroundTaskProgressEventArgs = *mut ::core::ffi::c_void;
pub type BackgroundTaskProgressEventHandler = *mut ::core::ffi::c_void;
pub type BackgroundTaskRegistration = *mut ::core::ffi::c_void;
pub type BackgroundTaskRegistrationGroup = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct BackgroundTaskThrottleCounter(pub i32);
impl BackgroundTaskThrottleCounter {
    pub const All: Self = Self(0i32);
    pub const Cpu: Self = Self(1i32);
    pub const Network: Self = Self(2i32);
}
impl ::core::marker::Copy for BackgroundTaskThrottleCounter {}
impl ::core::clone::Clone for BackgroundTaskThrottleCounter {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct BackgroundWorkCostValue(pub i32);
impl BackgroundWorkCostValue {
    pub const Low: Self = Self(0i32);
    pub const Medium: Self = Self(1i32);
    pub const High: Self = Self(2i32);
}
impl ::core::marker::Copy for BackgroundWorkCostValue {}
impl ::core::clone::Clone for BackgroundWorkCostValue {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BluetoothLEAdvertisementPublisherTrigger = *mut ::core::ffi::c_void;
pub type BluetoothLEAdvertisementWatcherTrigger = *mut ::core::ffi::c_void;
pub type CachedFileUpdaterTrigger = *mut ::core::ffi::c_void;
pub type CachedFileUpdaterTriggerDetails = *mut ::core::ffi::c_void;
pub type ChatMessageNotificationTrigger = *mut ::core::ffi::c_void;
pub type ChatMessageReceivedNotificationTrigger = *mut ::core::ffi::c_void;
pub type CommunicationBlockingAppSetAsActiveTrigger = *mut ::core::ffi::c_void;
pub type ContactStoreNotificationTrigger = *mut ::core::ffi::c_void;
pub type ContentPrefetchTrigger = *mut ::core::ffi::c_void;
pub type ConversationalAgentTrigger = *mut ::core::ffi::c_void;
pub type CustomSystemEventTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct CustomSystemEventTriggerRecurrence(pub i32);
impl CustomSystemEventTriggerRecurrence {
    pub const Once: Self = Self(0i32);
    pub const Always: Self = Self(1i32);
}
impl ::core::marker::Copy for CustomSystemEventTriggerRecurrence {}
impl ::core::clone::Clone for CustomSystemEventTriggerRecurrence {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DeviceConnectionChangeTrigger = *mut ::core::ffi::c_void;
pub type DeviceManufacturerNotificationTrigger = *mut ::core::ffi::c_void;
pub type DeviceServicingTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct DeviceTriggerResult(pub i32);
impl DeviceTriggerResult {
    pub const Allowed: Self = Self(0i32);
    pub const DeniedByUser: Self = Self(1i32);
    pub const DeniedBySystem: Self = Self(2i32);
    pub const LowBattery: Self = Self(3i32);
}
impl ::core::marker::Copy for DeviceTriggerResult {}
impl ::core::clone::Clone for DeviceTriggerResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DeviceUseTrigger = *mut ::core::ffi::c_void;
pub type DeviceWatcherTrigger = *mut ::core::ffi::c_void;
pub type EmailStoreNotificationTrigger = *mut ::core::ffi::c_void;
pub type GattCharacteristicNotificationTrigger = *mut ::core::ffi::c_void;
pub type GattServiceProviderTrigger = *mut ::core::ffi::c_void;
pub type GattServiceProviderTriggerResult = *mut ::core::ffi::c_void;
pub type GeovisitTrigger = *mut ::core::ffi::c_void;
pub type IBackgroundCondition = *mut ::core::ffi::c_void;
pub type IBackgroundTask = *mut ::core::ffi::c_void;
pub type IBackgroundTaskInstance = *mut ::core::ffi::c_void;
pub type IBackgroundTaskInstance2 = *mut ::core::ffi::c_void;
pub type IBackgroundTaskInstance4 = *mut ::core::ffi::c_void;
pub type IBackgroundTaskRegistration = *mut ::core::ffi::c_void;
pub type IBackgroundTaskRegistration2 = *mut ::core::ffi::c_void;
pub type IBackgroundTaskRegistration3 = *mut ::core::ffi::c_void;
pub type IBackgroundTrigger = *mut ::core::ffi::c_void;
pub type LocationTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct LocationTriggerType(pub i32);
impl LocationTriggerType {
    pub const Geofence: Self = Self(0i32);
}
impl ::core::marker::Copy for LocationTriggerType {}
impl ::core::clone::Clone for LocationTriggerType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MaintenanceTrigger = *mut ::core::ffi::c_void;
pub type MediaProcessingTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct MediaProcessingTriggerResult(pub i32);
impl MediaProcessingTriggerResult {
    pub const Allowed: Self = Self(0i32);
    pub const CurrentlyRunning: Self = Self(1i32);
    pub const DisabledByPolicy: Self = Self(2i32);
    pub const UnknownError: Self = Self(3i32);
}
impl ::core::marker::Copy for MediaProcessingTriggerResult {}
impl ::core::clone::Clone for MediaProcessingTriggerResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MobileBroadbandDeviceServiceNotificationTrigger = *mut ::core::ffi::c_void;
pub type MobileBroadbandPcoDataChangeTrigger = *mut ::core::ffi::c_void;
pub type MobileBroadbandPinLockStateChangeTrigger = *mut ::core::ffi::c_void;
pub type MobileBroadbandRadioStateChangeTrigger = *mut ::core::ffi::c_void;
pub type MobileBroadbandRegistrationStateChangeTrigger = *mut ::core::ffi::c_void;
pub type NetworkOperatorDataUsageTrigger = *mut ::core::ffi::c_void;
pub type NetworkOperatorHotspotAuthenticationTrigger = *mut ::core::ffi::c_void;
pub type NetworkOperatorNotificationTrigger = *mut ::core::ffi::c_void;
pub type PaymentAppCanMakePaymentTrigger = *mut ::core::ffi::c_void;
pub type PhoneTrigger = *mut ::core::ffi::c_void;
pub type PushNotificationTrigger = *mut ::core::ffi::c_void;
pub type RcsEndUserMessageAvailableTrigger = *mut ::core::ffi::c_void;
pub type RfcommConnectionTrigger = *mut ::core::ffi::c_void;
pub type SecondaryAuthenticationFactorAuthenticationTrigger = *mut ::core::ffi::c_void;
pub type SensorDataThresholdTrigger = *mut ::core::ffi::c_void;
pub type SmartCardTrigger = *mut ::core::ffi::c_void;
pub type SmsMessageReceivedTrigger = *mut ::core::ffi::c_void;
pub type SocketActivityTrigger = *mut ::core::ffi::c_void;
pub type StorageLibraryChangeTrackerTrigger = *mut ::core::ffi::c_void;
pub type StorageLibraryContentChangedTrigger = *mut ::core::ffi::c_void;
pub type SystemCondition = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct SystemConditionType(pub i32);
impl SystemConditionType {
    pub const Invalid: Self = Self(0i32);
    pub const UserPresent: Self = Self(1i32);
    pub const UserNotPresent: Self = Self(2i32);
    pub const InternetAvailable: Self = Self(3i32);
    pub const InternetNotAvailable: Self = Self(4i32);
    pub const SessionConnected: Self = Self(5i32);
    pub const SessionDisconnected: Self = Self(6i32);
    pub const FreeNetworkAvailable: Self = Self(7i32);
    pub const BackgroundWorkCostNotHigh: Self = Self(8i32);
}
impl ::core::marker::Copy for SystemConditionType {}
impl ::core::clone::Clone for SystemConditionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SystemTrigger = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Background\"`*"]
#[repr(transparent)]
pub struct SystemTriggerType(pub i32);
impl SystemTriggerType {
    pub const Invalid: Self = Self(0i32);
    pub const SmsReceived: Self = Self(1i32);
    pub const UserPresent: Self = Self(2i32);
    pub const UserAway: Self = Self(3i32);
    pub const NetworkStateChange: Self = Self(4i32);
    pub const ControlChannelReset: Self = Self(5i32);
    pub const InternetAvailable: Self = Self(6i32);
    pub const SessionConnected: Self = Self(7i32);
    pub const ServicingComplete: Self = Self(8i32);
    pub const LockScreenApplicationAdded: Self = Self(9i32);
    pub const LockScreenApplicationRemoved: Self = Self(10i32);
    pub const TimeZoneChange: Self = Self(11i32);
    pub const OnlineIdConnectedStateChange: Self = Self(12i32);
    pub const BackgroundWorkCostChange: Self = Self(13i32);
    pub const PowerStateChange: Self = Self(14i32);
    pub const DefaultSignInAccountChange: Self = Self(15i32);
}
impl ::core::marker::Copy for SystemTriggerType {}
impl ::core::clone::Clone for SystemTriggerType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TetheringEntitlementCheckTrigger = *mut ::core::ffi::c_void;
pub type TimeTrigger = *mut ::core::ffi::c_void;
pub type ToastNotificationActionTrigger = *mut ::core::ffi::c_void;
pub type ToastNotificationHistoryChangedTrigger = *mut ::core::ffi::c_void;
pub type UserNotificationChangedTrigger = *mut ::core::ffi::c_void;
