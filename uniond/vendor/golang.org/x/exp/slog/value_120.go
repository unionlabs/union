// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.20

package slog

import "unsafe"

type (
	stringptr *byte // used in Value.any when the Value is a string
	groupptr  *Attr // used in Value.any when the Value is a []Attr
)

// StringValue returns a new Value for a string.
func StringValue(value string) Value {
	return Value{num: uint64(len(value)), any: stringptr(unsafe.StringData(value))}
}

// GroupValue returns a new Value for a list of Attrs.
// The caller must not subsequently mutate the argument slice.
func GroupValue(as ...Attr) Value {
	return Value{num: uint64(len(as)), any: groupptr(unsafe.SliceData(as))}
}

// String returns Value's value as a string, formatted like fmt.Sprint. Unlike
// the methods Int64, Float64, and so on, which panic if v is of the
// wrong kind, String never panics.
func (v Value) String() string {
	if sp, ok := v.any.(stringptr); ok {
		return unsafe.String(sp, v.num)
	}
	return string(v.append(nil))
}

func (v Value) str() string {
	return unsafe.String(v.any.(stringptr), v.num)
}
