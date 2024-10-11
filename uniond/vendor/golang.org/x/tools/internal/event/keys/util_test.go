// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package keys

import "testing"

func TestJoin(t *testing.T) {
	type T string
	type S []T

	tests := []struct {
		data S
		want string
	}{
		{S{"a", "b", "c"}, "a,b,c"},
		{S{"b", "a", "c"}, "a,b,c"},
		{S{"c", "a", "b"}, "a,b,c"},
		{nil, ""},
		{S{}, ""},
	}

	for _, test := range tests {
		if got := Join(test.data); got != test.want {
			t.Errorf("Join(%v) = %q, want %q", test.data, got, test.want)
		}
	}
}
