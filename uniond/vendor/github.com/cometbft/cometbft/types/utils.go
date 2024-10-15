package types

import "reflect"

// Go lacks a simple and safe way to see if something is a typed nil.
// See:
//   - https://dave.cheney.net/2017/08/09/typed-nils-in-go-2
//   - https://groups.google.com/forum/#!topic/golang-nuts/wnH302gBa4I/discussion
//   - https://github.com/golang/go/issues/21538
func isTypedNil(o any) bool {
	rv := reflect.ValueOf(o)
	switch rv.Kind() {
	case reflect.Chan, reflect.Func, reflect.Map, reflect.Ptr, reflect.Slice:
		return rv.IsNil()
	default:
		return false
	}
}

// isEmpty returns true if it has zero length.
func isEmpty(o any) bool {
	rv := reflect.ValueOf(o)
	switch rv.Kind() {
	case reflect.Array, reflect.Chan, reflect.Map, reflect.Slice, reflect.String:
		return rv.Len() == 0
	default:
		return false
	}
}
