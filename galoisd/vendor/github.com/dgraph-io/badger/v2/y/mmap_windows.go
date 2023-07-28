// +build windows

/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package y

import (
	"fmt"
	"os"
	"syscall"
	"unsafe"
)

func mmap(fd *os.File, write bool, size int64) ([]byte, error) {
	protect := syscall.PAGE_READONLY
	access := syscall.FILE_MAP_READ

	if write {
		protect = syscall.PAGE_READWRITE
		access = syscall.FILE_MAP_WRITE
	}
	fi, err := fd.Stat()
	if err != nil {
		return nil, err
	}

	// In windows, we cannot mmap a file more than it's actual size.
	// So truncate the file to the size of the mmap.
	if fi.Size() < size {
		if err := fd.Truncate(size); err != nil {
			return nil, fmt.Errorf("truncate: %s", err)
		}
	}

	// Open a file mapping handle.
	sizelo := uint32(size >> 32)
	sizehi := uint32(size) & 0xffffffff

	handler, err := syscall.CreateFileMapping(syscall.Handle(fd.Fd()), nil,
		uint32(protect), sizelo, sizehi, nil)
	if err != nil {
		return nil, os.NewSyscallError("CreateFileMapping", err)
	}

	// Create the memory map.
	addr, err := syscall.MapViewOfFile(handler, uint32(access), 0, 0, uintptr(size))
	if addr == 0 {
		return nil, os.NewSyscallError("MapViewOfFile", err)
	}

	// Close mapping handle.
	if err := syscall.CloseHandle(syscall.Handle(handler)); err != nil {
		return nil, os.NewSyscallError("CloseHandle", err)
	}

	// Slice memory layout
	// Copied this snippet from golang/sys package
	var sl = struct {
		addr uintptr
		len  int
		cap  int
	}{addr, int(size), int(size)}

	// Use unsafe to turn sl into a []byte.
	data := *(*[]byte)(unsafe.Pointer(&sl))

	return data, nil
}

func munmap(b []byte) error {
	return syscall.UnmapViewOfFile(uintptr(unsafe.Pointer(&b[0])))
}

func madvise(b []byte, readahead bool) error {
	// Do Nothing. We don’t care about this setting on Windows
	return nil
}
