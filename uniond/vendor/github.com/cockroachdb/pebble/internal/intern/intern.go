// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package intern

import "sync"

var pool = sync.Pool{
	New: func() interface{} {
		return make(map[string]string)
	},
}

// Bytes returns b converted to a string, interned.
func Bytes(b []byte) string {
	m := pool.Get().(map[string]string)
	c, ok := m[string(b)]
	if ok {
		pool.Put(m)
		return c
	}
	s := string(b)
	m[s] = s
	pool.Put(m)
	return s
}
