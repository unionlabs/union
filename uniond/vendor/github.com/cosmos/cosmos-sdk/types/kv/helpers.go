package kv

import "fmt"

// AssertKeyAtLeastLength panics when store key length is less than the given length.
func AssertKeyAtLeastLength(bz []byte, length int) {
	if len(bz) < length {
		panic(fmt.Sprintf("expected key of length at least %d, got %d", length, len(bz)))
	}
}

// AssertKeyLength panics when store key length is not equal to the given length.
func AssertKeyLength(bz []byte, length int) {
	if len(bz) != length {
		panic(fmt.Sprintf("unexpected key length; got: %d, expected: %d", len(bz), length))
	}
}
