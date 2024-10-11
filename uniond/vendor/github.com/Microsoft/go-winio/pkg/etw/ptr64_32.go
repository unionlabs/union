//go:build windows && (386 || arm)
// +build windows
// +build 386 arm

package etw

import (
	"unsafe"
)

// byteptr64 defines a struct containing a pointer. The struct is guaranteed to
// be 64 bits, regardless of the actual size of a pointer on the platform. This
// is intended for use with certain Windows APIs that expect a pointer as a
// ULONGLONG.
type ptr64 struct {
	ptr unsafe.Pointer
	_   uint32
}
