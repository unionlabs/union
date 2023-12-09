// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.19 && !go1.20

package slog

import (
	"reflect"
	"unsafe"
)

type (
	stringptr unsafe.Pointer // used in Value.any when the Value is a string
	groupptr  unsafe.Pointer // used in Value.any when the Value is a []Attr
)

// StringValue returns a new Value for a string.
func StringValue(value string) Value {
	hdr := (*reflect.StringHeader)(unsafe.Pointer(&value))
	return Value{num: uint64(hdr.Len), any: stringptr(hdr.Data)}
}

func (v Value) str() string {
	var s string
	hdr := (*reflect.StringHeader)(unsafe.Pointer(&s))
	hdr.Data = uintptr(v.any.(stringptr))
	hdr.Len = int(v.num)
	return s
}

// String returns Value's value as a string, formatted like fmt.Sprint. Unlike
// the methods Int64, Float64, and so on, which panic if v is of the
// wrong kind, String never panics.
func (v Value) String() string {
	if sp, ok := v.any.(stringptr); ok {
		// Inlining this code makes a huge difference.
		var s string
		hdr := (*reflect.StringHeader)(unsafe.Pointer(&s))
		hdr.Data = uintptr(sp)
		hdr.Len = int(v.num)
		return s
	}
	return string(v.append(nil))
}

// GroupValue returns a new Value for a list of Attrs.
// The caller must not subsequently mutate the argument slice.
func GroupValue(as ...Attr) Value {
	hdr := (*reflect.SliceHeader)(unsafe.Pointer(&as))
	return Value{num: uint64(hdr.Len), any: groupptr(hdr.Data)}
}
