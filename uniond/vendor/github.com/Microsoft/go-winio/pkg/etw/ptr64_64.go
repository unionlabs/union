//go:build windows && (amd64 || arm64)
// +build windows
// +build amd64 arm64

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
}
