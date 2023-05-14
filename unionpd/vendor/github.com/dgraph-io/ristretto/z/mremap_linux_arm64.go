/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
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

package z

import (
	"reflect"
	"unsafe"

	"golang.org/x/sys/unix"
)

// mremap is a Linux-specific system call to remap pages in memory. This can be used in place of munmap + mmap.
func mremap(data []byte, size int) ([]byte, error) {
	//nolint:lll
	// taken from <https://github.com/torvalds/linux/blob/f8394f232b1eab649ce2df5c5f15b0e528c92091/include/uapi/linux/mman.h#L8>
	const MREMAP_MAYMOVE = 0x1

	header := (*reflect.SliceHeader)(unsafe.Pointer(&data))
	// For ARM64, the second return argument for SYS_MREMAP is inconsistent (prior allocated size) with
	// other architectures, which return the size allocated
	mmapAddr, _, errno := unix.Syscall6(
		unix.SYS_MREMAP,
		header.Data,
		uintptr(header.Len),
		uintptr(size),
		uintptr(MREMAP_MAYMOVE),
		0,
		0,
	)
	if errno != 0 {
		return nil, errno
	}

	header.Data = mmapAddr
	header.Cap = size
	header.Len = size
	return data, nil
}
