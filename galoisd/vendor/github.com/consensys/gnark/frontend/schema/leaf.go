package schema

import "reflect"

// LeafInfo stores the leaf visibility (always set to Secret or Public)
// and the fully qualified name of the path to reach the leaf in the circuit struct.
type LeafInfo struct {
	Visibility Visibility
	FullName   func() string // in most instances, we don't need to actually evaluate the name.
	name       string
}

// LeafCount stores the number of secret and public interface of type target(reflect.Type)
// found by the walker.
type LeafCount struct {
	Secret int
	Public int
}

// LeafHandler is the handler function that will be called when Walk reaches leafs of the struct
type LeafHandler func(field LeafInfo, tValue reflect.Value) error

// An object implementing an init hook knows how to "init" itself
// when parsed at compile time
type InitHook interface {
	GnarkInitHook() // TODO @gbotrel find a better home for this
}
