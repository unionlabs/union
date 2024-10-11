package spectests

import (
	"fmt"
	"reflect"
)

func deepEqualImpl(v1, v2 reflect.Value, depth int) bool {
	if !v1.IsValid() || !v2.IsValid() {
		return v1.IsValid() == v2.IsValid()
	}
	if v1.Type() != v2.Type() {
		return false
	}

	switch v1.Kind() {
	case reflect.Array:
		for i := 0; i < v1.Len(); i++ {
			if !deepEqualImpl(v1.Index(i), v2.Index(i), depth+1) {
				return false
			}
		}
		return true

	case reflect.Slice:
		v1Empty := v1.IsNil() || v1.Len() == 0
		v2Empty := v2.IsNil() || v2.Len() == 0

		if v1Empty && v2Empty {
			return true
		}
		if v1.Len() != v2.Len() {
			return false
		}
		if v1.Pointer() == v2.Pointer() {
			return true
		}
		for i := 0; i < v1.Len(); i++ {
			if !deepEqualImpl(v1.Index(i), v2.Index(i), depth+1) {
				return false
			}
		}
		return true

	case reflect.Ptr:
		if v1.Pointer() == v2.Pointer() {
			return true
		}
		return deepEqualImpl(v1.Elem(), v2.Elem(), depth+1)

	case reflect.Struct:
		for i, n := 0, v1.NumField(); i < n; i++ {
			if !deepEqualImpl(v1.Field(i), v2.Field(i), depth+1) {
				return false
			}
		}
		return true

	default:
		// basic types
		if v2.Kind() != v1.Kind() {
			panic("BUG")
		}
		switch v1.Kind() {
		case reflect.Uint8, reflect.Uint64:
			return v1.Uint() == v2.Uint()

		case reflect.Bool:
			return v1.Bool() == v2.Bool()

		default:
			panic(fmt.Errorf("comparison for type %s not supported", v1.Kind().String()))
		}
	}
}

func deepEqual(x, y interface{}) bool {
	if x == nil || y == nil {
		return x == y
	}
	v1 := reflect.ValueOf(x)
	v2 := reflect.ValueOf(y)
	if v1.Type() != v2.Type() {
		return false
	}
	return deepEqualImpl(v1, v2, 0)
}
