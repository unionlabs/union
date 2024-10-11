// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestAlignment_String(t *testing.T) {
	var zero bitarray.Alignment
	tcs := []struct {
		a bitarray.Alignment
		s string
	}{
		{zero, "align-left"},
		{bitarray.AlignLeft, "align-left"},
		{bitarray.AlignRight, "align-right"},
	}
	for _, tc := range tcs {
		s := tc.a.String()
		if s != tc.s {
			t.Errorf("unexpected result: got %q, want %q", s, tc.s)
		}
	}
}
