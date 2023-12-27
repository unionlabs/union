// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"unsafe"

	"github.com/cockroachdb/pebble/internal/manual"
)

func getBytes(ptr unsafe.Pointer, length int) []byte {
	return (*[manual.MaxArrayLen]byte)(ptr)[:length:length]
}

func decodeVarint(ptr unsafe.Pointer) (uint32, unsafe.Pointer) {
	if a := *((*uint8)(ptr)); a < 128 {
		return uint32(a),
			unsafe.Pointer(uintptr(ptr) + 1)
	} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
		return uint32(b)<<7 | uint32(a),
			unsafe.Pointer(uintptr(ptr) + 2)
	} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
		return uint32(c)<<14 | uint32(b)<<7 | uint32(a),
			unsafe.Pointer(uintptr(ptr) + 3)
	} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
		return uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a),
			unsafe.Pointer(uintptr(ptr) + 4)
	} else {
		d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
		return uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a),
			unsafe.Pointer(uintptr(ptr) + 5)
	}
}
