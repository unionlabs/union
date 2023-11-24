// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build !tracing
// +build !tracing

package cache

import (
	"fmt"
	"sync/atomic"

	"github.com/cockroachdb/redact"
)

// refcnt provides an atomic reference count. This version is used when the
// "tracing" build tag is not enabled. See refcnt_tracing.go for the "tracing"
// enabled version.
type refcnt struct {
	val atomic.Int32
}

// initialize the reference count to the specified value.
func (v *refcnt) init(val int32) {
	v.val.Store(val)
}

func (v *refcnt) refs() int32 {
	return v.val.Load()
}

func (v *refcnt) acquire() {
	switch v := v.val.Add(1); {
	case v <= 1:
		panic(redact.Safe(fmt.Sprintf("pebble: inconsistent reference count: %d", v)))
	}
}

func (v *refcnt) release() bool {
	switch v := v.val.Add(-1); {
	case v < 0:
		panic(redact.Safe(fmt.Sprintf("pebble: inconsistent reference count: %d", v)))
	case v == 0:
		return true
	default:
		return false
	}
}

func (v *refcnt) trace(msg string) {
}

func (v *refcnt) traces() string {
	return ""
}

// Silence unused warning.
var _ = (*refcnt)(nil).traces
