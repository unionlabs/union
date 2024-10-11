// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package proto_test

import (
	"google.golang.org/protobuf/proto"
)

// Checking if [Size] returns 0 is an easy way to recognize empty messages:
func ExampleSize() {
	var m proto.Message
	if proto.Size(m) == 0 {
		// No fields set (or, in proto3, all fields matching the default);
		// skip processing this message, or return an error, or similar.
	}
}
