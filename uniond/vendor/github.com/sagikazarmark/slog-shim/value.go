// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package slog

import (
	"log/slog"
	"time"
)

// A Value can represent any Go value, but unlike type any,
// it can represent most small values without an allocation.
// The zero Value corresponds to nil.
type Value = slog.Value

// Kind is the kind of a Value.
type Kind = slog.Kind

// The following list is sorted alphabetically, but it's also important that
// KindAny is 0 so that a zero Value represents nil.
const (
	KindAny       = slog.KindAny
	KindBool      = slog.KindBool
	KindDuration  = slog.KindDuration
	KindFloat64   = slog.KindFloat64
	KindInt64     = slog.KindInt64
	KindString    = slog.KindString
	KindTime      = slog.KindTime
	KindUint64    = slog.KindUint64
	KindGroup     = slog.KindGroup
	KindLogValuer = slog.KindLogValuer
)

//////////////// Constructors

// StringValue returns a new Value for a string.
func StringValue(value string) Value {
	return slog.StringValue(value)
}

// IntValue returns a Value for an int.
func IntValue(v int) Value {
	return slog.IntValue(v)
}

// Int64Value returns a Value for an int64.
func Int64Value(v int64) Value {
	return slog.Int64Value(v)
}

// Uint64Value returns a Value for a uint64.
func Uint64Value(v uint64) Value {
	return slog.Uint64Value(v)
}

// Float64Value returns a Value for a floating-point number.
func Float64Value(v float64) Value {
	return slog.Float64Value(v)
}

// BoolValue returns a Value for a bool.
func BoolValue(v bool) Value {
	return slog.BoolValue(v)
}

// TimeValue returns a Value for a time.Time.
// It discards the monotonic portion.
func TimeValue(v time.Time) Value {
	return slog.TimeValue(v)
}

// DurationValue returns a Value for a time.Duration.
func DurationValue(v time.Duration) Value {
	return slog.DurationValue(v)
}

// GroupValue returns a new Value for a list of Attrs.
// The caller must not subsequently mutate the argument slice.
func GroupValue(as ...Attr) Value {
	return slog.GroupValue(as...)
}

// AnyValue returns a Value for the supplied value.
//
// If the supplied value is of type Value, it is returned
// unmodified.
//
// Given a value of one of Go's predeclared string, bool, or
// (non-complex) numeric types, AnyValue returns a Value of kind
// String, Bool, Uint64, Int64, or Float64. The width of the
// original numeric type is not preserved.
//
// Given a time.Time or time.Duration value, AnyValue returns a Value of kind
// KindTime or KindDuration. The monotonic time is not preserved.
//
// For nil, or values of all other types, including named types whose
// underlying type is numeric, AnyValue returns a value of kind KindAny.
func AnyValue(v any) Value {
	return slog.AnyValue(v)
}

// A LogValuer is any Go value that can convert itself into a Value for logging.
//
// This mechanism may be used to defer expensive operations until they are
// needed, or to expand a single value into a sequence of components.
type LogValuer = slog.LogValuer
