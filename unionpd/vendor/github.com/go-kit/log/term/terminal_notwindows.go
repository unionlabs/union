// Based on ssh/terminal:
// Copyright 2011 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build (linux && !appengine) || darwin || freebsd || openbsd
// +build linux,!appengine darwin freebsd openbsd

package term

import (
	"io"
	"syscall"
	"unsafe"
)

// IsTerminal returns true if w writes to a terminal.
func IsTerminal(w io.Writer) bool {
	fw, ok := w.(fder)
	if !ok {
		return false
	}
	var termios syscall.Termios
	_, _, err := syscall.Syscall6(syscall.SYS_IOCTL, fw.Fd(), ioctlReadTermios, uintptr(unsafe.Pointer(&termios)), 0, 0, 0)
	return err == 0
}
