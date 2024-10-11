// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import "testing"

func TestRemotecat(t *testing.T) {
	runTests(t, "testdata/remotecat")
}
