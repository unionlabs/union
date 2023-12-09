// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package cache

// Value holds a reference counted immutable value.
type Value struct {
	buf []byte
	// Reference count for the value. The value is freed when the reference count
	// drops to zero.
	ref refcnt
}

// Buf returns the buffer associated with the value. The contents of the buffer
// should not be changed once the value has been added to the cache. Instead, a
// new Value should be created and added to the cache to replace the existing
// value.
func (v *Value) Buf() []byte {
	if v == nil {
		return nil
	}
	return v.buf
}

// Truncate the buffer to the specified length. The buffer length should not be
// changed once the value has been added to the cache as there may be
// concurrent readers of the Value. Instead, a new Value should be created and
// added to the cache to replace the existing value.
func (v *Value) Truncate(n int) {
	v.buf = v.buf[:n]
}

func (v *Value) refs() int32 {
	return v.ref.refs()
}

func (v *Value) acquire() {
	v.ref.acquire()
}

func (v *Value) release() {
	if v != nil && v.ref.release() {
		v.free()
	}
}
