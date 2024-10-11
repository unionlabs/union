//go:build windows

package etw

//go:generate go run github.com/Microsoft/go-winio/tools/mkwinsyscall -output zsyscall_windows.go syscall.go

//sys eventRegister(providerId *windows.GUID, callback uintptr, callbackContext uintptr, providerHandle *providerHandle) (win32err error) = advapi32.EventRegister

//sys eventUnregister_64(providerHandle providerHandle) (win32err error) = advapi32.EventUnregister
//sys eventWriteTransfer_64(providerHandle providerHandle, descriptor *eventDescriptor, activityID *windows.GUID, relatedActivityID *windows.GUID, dataDescriptorCount uint32, dataDescriptors *eventDataDescriptor) (win32err error) = advapi32.EventWriteTransfer
//sys eventSetInformation_64(providerHandle providerHandle, class eventInfoClass, information uintptr, length uint32) (win32err error) = advapi32.EventSetInformation

//sys eventUnregister_32(providerHandle_low uint32, providerHandle_high uint32) (win32err error) = advapi32.EventUnregister
//sys eventWriteTransfer_32(providerHandle_low uint32, providerHandle_high uint32, descriptor *eventDescriptor, activityID *windows.GUID, relatedActivityID *windows.GUID, dataDescriptorCount uint32, dataDescriptors *eventDataDescriptor) (win32err error) = advapi32.EventWriteTransfer
//sys eventSetInformation_32(providerHandle_low uint32, providerHandle_high uint32, class eventInfoClass, information uintptr, length uint32) (win32err error) = advapi32.EventSetInformation
