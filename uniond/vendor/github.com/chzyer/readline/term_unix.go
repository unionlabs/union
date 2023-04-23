// Copyright 2011 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// +build darwin dragonfly freebsd linux,!appengine netbsd openbsd

package readline

import (
	"syscall"
	"unsafe"
)

type Termios syscall.Termios

// GetSize returns the dimensions of the given terminal.
func GetSize(fd int) (int, int, error) {
	var dimensions [4]uint16
	_, _, err := syscall.Syscall6(syscall.SYS_IOCTL, uintptr(fd), uintptr(syscall.TIOCGWINSZ), uintptr(unsafe.Pointer(&dimensions)), 0, 0, 0)
	if err != 0 {
		return 0, 0, err
	}
	return int(dimensions[1]), int(dimensions[0]), nil
}
