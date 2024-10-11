//go:build windows
// +build windows

package process

import (
	"unsafe"

	"golang.org/x/sys/windows"
)

// EnumProcesses returns a slice containing the process IDs of all processes
// currently running on the system.
func EnumProcesses() ([]uint32, error) {
	count := 256
	uint32Size := unsafe.Sizeof(uint32(0))
	for {
		buf := make([]uint32, count)
		bufferSize := uint32(len(buf) * int(uint32Size))
		retBufferSize := uint32(0)
		if err := enumProcesses(&buf[0], bufferSize, &retBufferSize); err != nil {
			return nil, err
		}
		if retBufferSize == bufferSize {
			count = count * 2
			continue
		}
		actualCount := retBufferSize / uint32(uint32Size)
		return buf[:actualCount], nil
	}
}

// ProcessMemoryCountersEx is the PROCESS_MEMORY_COUNTERS_EX struct from
// Windows:
// https://docs.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex
//
//nolint:revive // process.ProcessMemoryCountersEx stutters, too late to change it
type ProcessMemoryCountersEx struct {
	Cb                         uint32
	PageFaultCount             uint32
	PeakWorkingSetSize         uint
	WorkingSetSize             uint
	QuotaPeakPagedPoolUsage    uint
	QuotaPagedPoolUsage        uint
	QuotaPeakNonPagedPoolUsage uint
	QuotaNonPagedPoolUsage     uint
	PagefileUsage              uint
	PeakPagefileUsage          uint
	PrivateUsage               uint
}

// GetProcessMemoryInfo returns the memory usage information for the given
// process. The process handle must have the PROCESS_QUERY_INFORMATION or
// PROCESS_QUERY_LIMITED_INFORMATION, and the PROCESS_VM_READ access rights.
func GetProcessMemoryInfo(process windows.Handle) (*ProcessMemoryCountersEx, error) {
	memCounters := &ProcessMemoryCountersEx{}
	size := unsafe.Sizeof(*memCounters)
	if err := getProcessMemoryInfo(process, memCounters, uint32(size)); err != nil {
		return nil, err
	}
	return memCounters, nil
}

// These constants are used with QueryFullProcessImageName's flags.
const (
	// ImageNameFormatWin32Path indicates to format the name as a Win32 path.
	ImageNameFormatWin32Path = iota
	// ImageNameFormatNTPath indicates to format the name as a NT path.
	ImageNameFormatNTPath
)

// QueryFullProcessImageName returns the full process image name for the given
// process. The process handle must have the PROCESS_QUERY_INFORMATION or
// PROCESS_QUERY_LIMITED_INFORMATION access right. The flags can be either
// `ImageNameFormatWin32Path` or `ImageNameFormatNTPath`.
func QueryFullProcessImageName(process windows.Handle, flags uint32) (string, error) {
	bufferSize := uint32(256)
	for {
		b := make([]uint16, bufferSize)
		err := queryFullProcessImageName(process, flags, &b[0], &bufferSize)
		if err == windows.ERROR_INSUFFICIENT_BUFFER { //nolint:errorlint // err is Errno
			bufferSize = bufferSize * 2
			continue
		}
		if err != nil {
			return "", err
		}
		return windows.UTF16ToString(b[:bufferSize]), nil
	}
}
