package equality

import (
	"reflect"
	"unsafe"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"google.golang.org/protobuf/proto"
)

// During deepValueEqual, must keep track of checks that are
// in progress. The comparison algorithm assumes that all
// checks in progress are true when it reencounters them.
// Visited comparisons are stored in a map indexed by visit.
type visit struct {
	a1  unsafe.Pointer // #nosec G103 -- Test use only
	a2  unsafe.Pointer // #nosec G103 -- Test use only
	typ reflect.Type
}

// Copyright 2009 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
//
// This file extends Go's reflect.DeepEqual function into a ssz.DeepEqual
// function that is compliant with the supported types of ssz and its
// intricacies when determining equality of empty values.
//
// Tests for deep equality using reflected types. The map argument tracks
// comparisons that have already been seen, which allows short circuiting on
// recursive types.
func deepValueEqual(v1, v2 reflect.Value, visited map[visit]bool, depth int) bool {
	if !v1.IsValid() || !v2.IsValid() {
		return v1.IsValid() == v2.IsValid()
	}
	if v1.Type() != v2.Type() {
		return false
	}
	// We want to avoid putting more in the visited map than we need to.
	// For any possible reference cycle that might be encountered,
	// hard(t) needs to return true for at least one of the types in the cycle.
	hard := func(k reflect.Kind) bool {
		switch k {
		case reflect.Slice, reflect.Ptr, reflect.Interface:
			return true
		}
		return false
	}

	if v1.CanAddr() && v2.CanAddr() && hard(v1.Kind()) {
		addr1 := unsafe.Pointer(v1.UnsafeAddr()) // #nosec G103 -- Test compare only
		addr2 := unsafe.Pointer(v2.UnsafeAddr()) // #nosec G103 -- Test compare only

		if uintptr(addr1) > uintptr(addr2) {
			// Canonicalize order to reduce number of entries in visited.
			// Assumes non-moving garbage collector.
			addr1, addr2 = addr2, addr1
		}

		// Short circuit if references are already seen.
		typ := v1.Type()
		v := visit{addr1, addr2, typ}
		if visited[v] {
			return true
		}

		// Remember for later.
		visited[v] = true
	}

	switch v1.Kind() {
	case reflect.Array:
		for i := 0; i < v1.Len(); i++ {
			if !deepValueEqual(v1.Index(i), v2.Index(i), visited, depth+1) {
				return false
			}
		}
		return true
	case reflect.Slice:
		if v1.IsNil() && v2.Len() == 0 {
			return true
		}
		if v1.Len() == 0 && v2.IsNil() {
			return true
		}
		if v1.IsNil() && v2.IsNil() {
			return true
		}
		if v1.Len() != v2.Len() {
			return false
		}
		if v1.Pointer() == v2.Pointer() {
			return true
		}
		for i := 0; i < v1.Len(); i++ {
			if !deepValueEqual(v1.Index(i), v2.Index(i), visited, depth+1) {
				return false
			}
		}
		return true
	case reflect.Interface:
		if v1.IsNil() || v2.IsNil() {
			return v1.IsNil() == v2.IsNil()
		}
		return deepValueEqual(v1.Elem(), v2.Elem(), visited, depth+1)
	case reflect.Ptr:
		if v1.Pointer() == v2.Pointer() {
			return true
		}
		return deepValueEqual(v1.Elem(), v2.Elem(), visited, depth+1)
	case reflect.Struct:
		for i, n := 0, v1.NumField(); i < n; i++ {
			if !deepValueEqual(v1.Field(i), v2.Field(i), visited, depth+1) {
				return false
			}
		}
		return true
	default:
		return deepValueBaseTypeEqual(v1, v2)
	}
}

func deepValueEqualExportedOnly(v1, v2 reflect.Value, visited map[visit]bool, depth int) bool {
	if !v1.IsValid() || !v2.IsValid() {
		return v1.IsValid() == v2.IsValid()
	}
	if v1.Type() != v2.Type() {
		return false
	}
	// We want to avoid putting more in the visited map than we need to.
	// For any possible reference cycle that might be encountered,
	// hard(t) needs to return true for at least one of the types in the cycle.
	hard := func(k reflect.Kind) bool {
		switch k {
		case reflect.Slice, reflect.Ptr, reflect.Interface:
			return true
		}
		return false
	}

	if v1.CanAddr() && v2.CanAddr() && hard(v1.Kind()) {
		addr1 := unsafe.Pointer(v1.UnsafeAddr()) // #nosec G103 -- Test compare only
		addr2 := unsafe.Pointer(v2.UnsafeAddr()) // #nosec G103 -- Test compare only
		if uintptr(addr1) > uintptr(addr2) {
			// Canonicalize order to reduce number of entries in visited.
			// Assumes non-moving garbage collector.
			addr1, addr2 = addr2, addr1
		}

		// Short circuit if references are already seen.
		typ := v1.Type()
		v := visit{addr1, addr2, typ}
		if visited[v] {
			return true
		}

		// Remember for later.
		visited[v] = true
	}

	switch v1.Kind() {
	case reflect.Array:
		for i := 0; i < v1.Len(); i++ {
			if !deepValueEqualExportedOnly(v1.Index(i), v2.Index(i), visited, depth+1) {
				return false
			}
		}
		return true
	case reflect.Slice:
		if v1.IsNil() && v2.Len() == 0 {
			return true
		}
		if v1.Len() == 0 && v2.IsNil() {
			return true
		}
		if v1.IsNil() && v2.IsNil() {
			return true
		}
		if v1.Len() != v2.Len() {
			return false
		}
		if v1.Pointer() == v2.Pointer() {
			return true
		}
		for i := 0; i < v1.Len(); i++ {
			if !deepValueEqualExportedOnly(v1.Index(i), v2.Index(i), visited, depth+1) {
				return false
			}
		}
		return true
	case reflect.Interface:
		if v1.IsNil() || v2.IsNil() {
			return v1.IsNil() == v2.IsNil()
		}
		return deepValueEqualExportedOnly(v1.Elem(), v2.Elem(), visited, depth+1)
	case reflect.Ptr:
		if v1.Pointer() == v2.Pointer() {
			return true
		}
		return deepValueEqualExportedOnly(v1.Elem(), v2.Elem(), visited, depth+1)
	case reflect.Struct:
		for i, n := 0, v1.NumField(); i < n; i++ {
			v1Field := v1.Field(i)
			v2Field := v2.Field(i)
			if !v1Field.CanInterface() || !v2Field.CanInterface() {
				// Continue for unexported fields, since they cannot be read anyways.
				continue
			}
			if !deepValueEqualExportedOnly(v1Field, v2Field, visited, depth+1) {
				return false
			}
		}
		return true
	default:
		return deepValueBaseTypeEqual(v1, v2)
	}
}

func deepValueBaseTypeEqual(v1, v2 reflect.Value) bool {
	switch v1.Kind() {
	case reflect.String:
		return v1.String() == v2.String()
	case reflect.Uint64:
		switch v1.Type().Name() {
		case "Epoch":
			return v1.Interface().(primitives.Epoch) == v2.Interface().(primitives.Epoch)
		case "Slot":
			return v1.Interface().(primitives.Slot) == v2.Interface().(primitives.Slot)
		case "ValidatorIndex":
			return v1.Interface().(primitives.ValidatorIndex) == v2.Interface().(primitives.ValidatorIndex)
		case "CommitteeIndex":
			return v1.Interface().(primitives.CommitteeIndex) == v2.Interface().(primitives.CommitteeIndex)
		}
		return v1.Interface().(uint64) == v2.Interface().(uint64)
	case reflect.Uint32:
		return v1.Interface().(uint32) == v2.Interface().(uint32)
	case reflect.Int32:
		return v1.Interface().(int32) == v2.Interface().(int32)
	case reflect.Uint16:
		return v1.Interface().(uint16) == v2.Interface().(uint16)
	case reflect.Uint8:
		return v1.Interface().(uint8) == v2.Interface().(uint8)
	case reflect.Bool:
		return v1.Interface().(bool) == v2.Interface().(bool)
	default:
		return false
	}
}

// DeepEqual reports whether two SSZ-able values x and y are “deeply equal,” defined as follows:
// Two values of identical type are deeply equal if one of the following cases applies:
//
// Values of distinct types are never deeply equal.
//
// Array values are deeply equal when their corresponding elements are deeply equal.
//
// Struct values are deeply equal if their corresponding fields,
// both exported and unexported, are deeply equal.
//
// Interface values are deeply equal if they hold deeply equal concrete values.
//
// Pointer values are deeply equal if they are equal using Go's == operator
// or if they point to deeply equal values.
//
// Slice values are deeply equal when all of the following are true:
// they are both nil, one is nil and the other is empty or vice-versa,
// they have the same length, and either they point to the same initial entry of the same array
// (that is, &x[0] == &y[0]) or their corresponding elements (up to length) are deeply equal.
//
// Other values - numbers, bools, strings, and channels - are deeply equal
// if they are equal using Go's == operator.
//
// In general DeepEqual is a recursive relaxation of Go's == operator.
// However, this idea is impossible to implement without some inconsistency.
// Specifically, it is possible for a value to be unequal to itself,
// either because it is of func type (uncomparable in general)
// or because it is a floating-point NaN value (not equal to itself in floating-point comparison),
// or because it is an array, struct, or interface containing
// such a value.
//
// On the other hand, pointer values are always equal to themselves,
// even if they point at or contain such problematic values,
// because they compare equal using Go's == operator, and that
// is a sufficient condition to be deeply equal, regardless of content.
// DeepEqual has been defined so that the same short-cut applies
// to slices and maps: if x and y are the same slice or the same map,
// they are deeply equal regardless of content.
//
// As DeepEqual traverses the data values it may find a cycle. The
// second and subsequent times that DeepEqual compares two pointer
// values that have been compared before, it treats the values as
// equal rather than examining the values to which they point.
// This ensures that DeepEqual terminates.
//
// Credits go to the Go team as this is an extension of the official Go source code's
// reflect.DeepEqual function to handle special SSZ edge cases.
func DeepEqual(x, y interface{}) bool {
	if x == nil || y == nil {
		return x == y
	}
	v1 := reflect.ValueOf(x)
	v2 := reflect.ValueOf(y)
	if v1.Type() != v2.Type() {
		return false
	}
	if IsProto(x) && IsProto(y) {
		// Exclude unexported fields for protos.
		return deepValueEqualExportedOnly(v1, v2, make(map[visit]bool), 0)
	}
	return deepValueEqual(v1, v2, make(map[visit]bool), 0)
}

func IsProto(item interface{}) bool {
	typ := reflect.TypeOf(item)
	kind := typ.Kind()
	if kind != reflect.Slice && kind != reflect.Array && kind != reflect.Map {
		_, ok := item.(proto.Message)
		return ok
	}
	elemTyp := typ.Elem()
	modelType := reflect.TypeOf((*proto.Message)(nil)).Elem()
	return elemTyp.Implements(modelType)
}
