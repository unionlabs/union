// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build race
// +build race

package invariants

// RaceEnabled is true if we were built with the "race" build tag.
const RaceEnabled = true
