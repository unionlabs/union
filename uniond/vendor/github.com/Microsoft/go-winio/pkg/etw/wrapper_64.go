//go:build windows && (amd64 || arm64)
// +build windows
// +build amd64 arm64

package etw

import (
	"github.com/Microsoft/go-winio/pkg/guid"
	"golang.org/x/sys/windows"
)

func eventUnregister(providerHandle providerHandle) (win32err error) {
	return eventUnregister_64(providerHandle)
}

func eventWriteTransfer(
	providerHandle providerHandle,
	descriptor *eventDescriptor,
	activityID *windows.GUID,
	relatedActivityID *windows.GUID,
	dataDescriptorCount uint32,
	dataDescriptors *eventDataDescriptor) (win32err error) {
	return eventWriteTransfer_64(
		providerHandle,
		descriptor,
		activityID,
		relatedActivityID,
		dataDescriptorCount,
		dataDescriptors)
}

func eventSetInformation(
	providerHandle providerHandle,
	class eventInfoClass,
	information uintptr,
	length uint32) (win32err error) {
	return eventSetInformation_64(
		providerHandle,
		class,
		information,
		length)
}

// providerCallbackAdapter acts as the first-level callback from the C/ETW side
// for provider notifications. Because Go has trouble with callback arguments of
// different size, it has only pointer-sized arguments, which are then cast to
// the appropriate types when calling providerCallback.
func providerCallbackAdapter(
	sourceID *guid.GUID,
	state uintptr,
	level uintptr,
	matchAnyKeyword uintptr,
	matchAllKeyword uintptr,
	filterData uintptr,
	i uintptr,
) uintptr {
	providerCallback(*sourceID,
		ProviderState(state),
		Level(level),
		uint64(matchAnyKeyword),
		uint64(matchAllKeyword),
		filterData,
		i)
	return 0
}
