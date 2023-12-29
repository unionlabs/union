// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build tracing
// +build tracing

package cache

import (
	"fmt"
	"runtime/debug"
)

func (c *Cache) trace(msg string, refs int64) {
	s := fmt.Sprintf("%s: refs=%d\n%s", msg, refs, debug.Stack())
	c.tr.Lock()
	c.tr.msgs = append(c.tr.msgs, s)
	c.tr.Unlock()
}
