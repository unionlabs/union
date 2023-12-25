// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build (!invariants && !tracing) || race
// +build !invariants,!tracing race

package cache

import (
	"unsafe"

	"github.com/cockroachdb/pebble/internal/manual"
)

const valueSize = int(unsafe.Sizeof(Value{}))

func newValue(n int) *Value {
	if n == 0 {
		return nil
	}

	if !cgoEnabled {
		// If Cgo is disabled then all memory is allocated from the Go heap and we
		// can't play the trick below to combine the Value and buffer allocation.
		v := &Value{buf: make([]byte, n)}
		v.ref.init(1)
		return v
	}

	// When we're not performing leak detection, the lifetime of the returned
	// Value is exactly the lifetime of the backing buffer and we can manually
	// allocate both.
	//
	// TODO(peter): It may be better to separate the allocation of the value and
	// the buffer in order to reduce internal fragmentation in malloc. If the
	// buffer is right at a power of 2, adding valueSize might push the
	// allocation over into the next larger size.
	b := manual.New(valueSize + n)
	v := (*Value)(unsafe.Pointer(&b[0]))
	v.buf = b[valueSize:]
	v.ref.init(1)
	return v
}

func (v *Value) free() {
	if !cgoEnabled {
		return
	}

	// When we're not performing leak detection, the Value and buffer were
	// allocated contiguously.
	n := valueSize + cap(v.buf)
	buf := (*[manual.MaxArrayLen]byte)(unsafe.Pointer(v))[:n:n]
	v.buf = nil
	manual.Free(buf)
}
