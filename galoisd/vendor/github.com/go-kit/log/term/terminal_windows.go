// Based on ssh/terminal:
// Copyright 2011 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build windows
// +build windows

package term

import (
	"encoding/binary"
	"io"
	"regexp"
	"syscall"
	"unsafe"
)

var kernel32 = syscall.NewLazyDLL("kernel32.dll")

var (
	procGetFileInformationByHandleEx = kernel32.NewProc("GetFileInformationByHandleEx")
	msysPipeNameRegex                = regexp.MustCompile(`\\(cygwin|msys)-\w+-pty\d?-(to|from)-master`)
)

const (
	fileNameInfo = 0x02
)

// IsTerminal returns true if w writes to a terminal.
func IsTerminal(w io.Writer) bool {
	return IsConsole(w) || IsMSYSTerminal(w)
}

// IsConsole returns true if w writes to a Windows console.
func IsConsole(w io.Writer) bool {
	var handle syscall.Handle

	if fw, ok := w.(fder); ok {
		handle = syscall.Handle(fw.Fd())
	} else {
		// The writer has no file-descriptor and so can't be a terminal.
		return false
	}

	var st uint32
	err := syscall.GetConsoleMode(handle, &st)

	// If the handle is attached to a terminal, GetConsoleMode returns a
	// non-zero value containing the console mode flags. We don't care about
	// the specifics of flags, just that it is not zero.
	return (err == nil && st != 0)
}

// IsMSYSTerminal returns true if w writes to a MSYS/MSYS2 terminal.
func IsMSYSTerminal(w io.Writer) bool {
	var handle syscall.Handle

	if fw, ok := w.(fder); ok {
		handle = syscall.Handle(fw.Fd())
	} else {
		// The writer has no file-descriptor and so can't be a terminal.
		return false
	}

	// MSYS(2) terminal reports as a pipe for STDIN/STDOUT/STDERR. If it isn't
	// a pipe, it can't be a MSYS(2) terminal.
	filetype, err := syscall.GetFileType(handle)

	if filetype != syscall.FILE_TYPE_PIPE || err != nil {
		return false
	}

	// MSYS2/Cygwin terminal's name looks like: \msys-dd50a72ab4668b33-pty2-to-master
	data := make([]byte, 256)

	r, _, e := syscall.Syscall6(
		procGetFileInformationByHandleEx.Addr(),
		4,
		uintptr(handle),
		uintptr(fileNameInfo),
		uintptr(unsafe.Pointer(&data[0])),
		uintptr(len(data)),
		0,
		0,
	)

	if r != 0 && e == 0 {
		// The first 4 bytes of the buffer are the size of the UTF16 name, in bytes.
		unameLen := binary.LittleEndian.Uint32(data[:4]) / 2
		uname := make([]uint16, unameLen)

		for i := uint32(0); i < unameLen; i++ {
			uname[i] = binary.LittleEndian.Uint16(data[i*2+4 : i*2+2+4])
		}

		name := syscall.UTF16ToString(uname)

		return msysPipeNameRegex.MatchString(name)
	}

	return false
}
