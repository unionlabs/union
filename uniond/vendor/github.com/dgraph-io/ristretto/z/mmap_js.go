//go:build js

/*
 * Copyright 2023 Dgraph Labs, Inc. and Contributors
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
	"os"
	"syscall"
)

func mmap(fd *os.File, writeable bool, size int64) ([]byte, error) {
	return nil, syscall.ENOSYS
}

func munmap(b []byte) error {
	return syscall.ENOSYS
}

func madvise(b []byte, readahead bool) error {
	return syscall.ENOSYS
}

func msync(b []byte) error {
	return syscall.ENOSYS
}
