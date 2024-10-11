// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build race

// Package race exposes Enabled, a const indicating whether the test is running
// under the race detector.
package race

// Enabled is true if the race detector is enabled.
const Enabled = true
