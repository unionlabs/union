// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build tracing
// +build tracing

package cache

import (
	"fmt"
	"runtime/debug"
	"strings"
	"sync"
	"sync/atomic"
)

// refcnt provides an atomic reference count, along with a tracing facility for
// debugging logic errors in manipulating the reference count. This version is
// used when the "tracing" build tag is enabled.
type refcnt struct {
	val atomic.Int32
	sync.Mutex
	msgs []string
}

func (v *refcnt) init(val int32) {
	v.val.Store(val)
	v.trace("init")
}

func (v *refcnt) refs() int32 {
	return v.val.Load()
}

func (v *refcnt) acquire() {
	switch n := v.val.Add(1); {
	case n <= 1:
		panic(fmt.Sprintf("pebble: inconsistent reference count: %d", n))
	}
	v.trace("acquire")
}

func (v *refcnt) release() bool {
	n := v.val.Add(-1)
	switch {
	case n < 0:
		panic(fmt.Sprintf("pebble: inconsistent reference count: %d", n))
	}
	v.trace("release")
	return n == 0
}

func (v *refcnt) trace(msg string) {
	s := fmt.Sprintf("%s: refs=%d\n%s", msg, v.refs(), debug.Stack())
	v.Lock()
	v.msgs = append(v.msgs, s)
	v.Unlock()
}

func (v *refcnt) traces() string {
	v.Lock()
	s := strings.Join(v.msgs, "\n")
	v.Unlock()
	return s
}
