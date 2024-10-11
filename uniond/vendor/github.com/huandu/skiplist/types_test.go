// Copyright 2011 Huan Du. All rights reserved.
// Licensed under the MIT license that can be found in the LICENSE file.

package skiplist

import (
	"testing"

	"github.com/huandu/go-assert"
)

func TestCompareTypes(t *testing.T) {
	a := assert.New(t)
	cases := []struct {
		kt       keyType
		lhs, rhs interface{}
		result   int
	}{
		{Int, 0, 0, 0},
		{Int, 2, 0, 1},
		{Int, -1, 1, -1},
		{Byte, 9, 2, 1},
		{Float32, 1.2, 1.20001, -1},
		{String, "foo", "bar", 1},
		{String, "001", "101", -1},
		{String, "equals", "equals", 0},
		{Bytes, []byte("abcdefghijk"), []byte("abcdefghij"), 1},
	}

	for i, c := range cases {
		a.Use(&i, &c)
		a.Equal(c.result, c.kt.Compare(c.lhs, c.rhs))
	}
}
