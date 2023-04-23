package xsync

import (
	"sync/atomic"
)

// Int64 represents an atomic int64.
type Int64 struct {
	// We do not use atomic.Load/StoreInt64 since it does not
	// work on 32 bit computers but we need 64 bit integers.
	i atomic.Value
}

// Load loads the int64.
func (v *Int64) Load() int64 {
	i, _ := v.i.Load().(int64)
	return i
}

// Store stores the int64.
func (v *Int64) Store(i int64) {
	v.i.Store(i)
}
