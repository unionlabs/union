// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build !tracing
// +build !tracing

package cache

func (c *Cache) trace(_ string, _ int64) {}
