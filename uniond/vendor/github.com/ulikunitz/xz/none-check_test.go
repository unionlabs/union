// Copyright 2014-2022 Ulrich Kunitz. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package xz

import (
	"bytes"
	"testing"
)

func TestNoneHash(t *testing.T) {
	h := newNoneHash()

	p := []byte("foo")
	q := h.Sum(p)

	if !bytes.Equal(q, p) {
		t.Fatalf("h.Sum: got %q; want %q", q, p)
	}

}
