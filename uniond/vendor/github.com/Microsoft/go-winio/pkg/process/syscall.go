//go:build windows
// +build windows

package process

import (
	"golang.org/x/sys/windows"
)

//go:generate go run github.com/Microsoft/go-winio/tools/mkwinsyscall -output zsyscall_windows.go syscall.go

//sys enumProcesses(pids *uint32, bufferSize uint32, retBufferSize *uint32) (err error) = kernel32.K32EnumProcesses
//sys getProcessMemoryInfo(process handle, memCounters *ProcessMemoryCountersEx, size uint32) (err error) = kernel32.K32GetProcessMemoryInfo
//sys queryFullProcessImageName(process handle, flags uint32, buffer *uint16, bufferSize *uint32) (err error) = kernel32.QueryFullProcessImageNameW

type handle = windows.Handle
