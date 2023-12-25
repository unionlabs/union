// Copyright 2018 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package rawalloc

// New returns a new byte slice of the specified length and capacity where the
// backing memory is uninitialized. This differs from make([]byte) which
// guarantees that the backing memory for the slice is initialized to zero. Use
// carefully.
func New(len, cap int) []byte {
	ptr := mallocgc(uintptr(cap), nil, false)
	return (*[maxArrayLen]byte)(ptr)[:len:cap]
}
