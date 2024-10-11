//go:build windows && (386 || arm)
// +build windows
// +build 386 arm

package etw

import (
	"github.com/Microsoft/go-winio/pkg/guid"
	"golang.org/x/sys/windows"
)

func low(v providerHandle) uint32 {
	return uint32(v & 0xffffffff)
}

func high(v providerHandle) uint32 {
	return low(v >> 32)
}

func eventUnregister(providerHandle providerHandle) (win32err error) {
	return eventUnregister_32(low(providerHandle), high(providerHandle))
}

func eventWriteTransfer(
	providerHandle providerHandle,
	descriptor *eventDescriptor,
	activityID *windows.GUID,
	relatedActivityID *windows.GUID,
	dataDescriptorCount uint32,
	dataDescriptors *eventDataDescriptor) (win32err error) {

	return eventWriteTransfer_32(
		low(providerHandle),
		high(providerHandle),
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

	return eventSetInformation_32(
		low(providerHandle),
		high(providerHandle),
		class,
		information,
		length)
}

// providerCallbackAdapter acts as the first-level callback from the C/ETW side
// for provider notifications. Because Go has trouble with callback arguments of
// different size, it has only pointer-sized arguments, which are then cast to
// the appropriate types when calling providerCallback.
// For x86, the matchAny and matchAll keywords need to be assembled from two
// 32-bit integers, because the max size of an argument is uintptr, but those
// two arguments are actually 64-bit integers.
func providerCallbackAdapter(sourceID *guid.GUID, state uint32, level uint32, matchAnyKeyword_low uint32, matchAnyKeyword_high uint32, matchAllKeyword_low uint32, matchAllKeyword_high uint32, filterData uintptr, i uintptr) uintptr {
	matchAnyKeyword := uint64(matchAnyKeyword_high)<<32 | uint64(matchAnyKeyword_low)
	matchAllKeyword := uint64(matchAllKeyword_high)<<32 | uint64(matchAllKeyword_low)
	providerCallback(*sourceID, ProviderState(state), Level(level), uint64(matchAnyKeyword), uint64(matchAllKeyword), filterData, i)
	return 0
}
