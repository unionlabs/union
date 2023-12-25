// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build (invariants && !race) || (tracing && !race)
// +build invariants,!race tracing,!race

package cache

import (
	"fmt"
	"os"

	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/manual"
)

// newValue creates a Value with a manually managed buffer of size n.
//
// This definition of newValue is used when either the "invariants" or
// "tracing" build tags are specified. It hooks up a finalizer to the returned
// Value that checks for memory leaks when the GC determines the Value is no
// longer reachable.
func newValue(n int) *Value {
	if n == 0 {
		return nil
	}
	b := manual.New(n)
	v := &Value{buf: b}
	v.ref.init(1)
	// Note: this is a no-op if invariants and tracing are disabled or race is
	// enabled.
	invariants.SetFinalizer(v, func(obj interface{}) {
		v := obj.(*Value)
		if v.buf != nil {
			fmt.Fprintf(os.Stderr, "%p: cache value was not freed: refs=%d\n%s",
				v, v.refs(), v.ref.traces())
			os.Exit(1)
		}
	})
	return v
}

func (v *Value) free() {
	// When "invariants" are enabled set the value contents to 0xff in order to
	// cache use-after-free bugs.
	for i := range v.buf {
		v.buf[i] = 0xff
	}
	manual.Free(v.buf)
	// Setting Value.buf to nil is needed for correctness of the leak checking
	// that is performed when the "invariants" or "tracing" build tags are
	// enabled.
	v.buf = nil
}
