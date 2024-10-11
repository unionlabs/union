// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"testing"
)

func TestBitArray_SHA512_cavp(t *testing.T) {
	test := func(name string) {
		tcs, err := cavpTestCases(name)
		if err != nil {
			t.Fatalf("failed to load test cases: %s: %s", name, err)
		}
		for i, tc := range tcs {
			got := tc.ba.SHA512()
			if !bytes.Equal(got[:], tc.md) {
				t.Errorf("unexpected hash: %s: #%d", name, i)
				t.Logf("mlen: %d", tc.ba.Len())
				t.Logf(" msg: %#b", tc.ba)
				t.Logf(" got: %X", got)
				t.Logf("want: %X", tc.md)
			}
		}
	}
	test("SHA512ShortMsg")
	test("SHA512LongMsg")
}

func TestBitArray_SHA384_cavp(t *testing.T) {
	test := func(name string) {
		tcs, err := cavpTestCases(name)
		if err != nil {
			t.Fatalf("failed to load test cases: %s: %s", name, err)
		}
		for i, tc := range tcs {
			got := tc.ba.SHA384()
			if !bytes.Equal(got[:], tc.md) {
				t.Errorf("unexpected hash: %s: #%d", name, i)
				t.Logf("mlen: %d", tc.ba.Len())
				t.Logf(" msg: %#b", tc.ba)
				t.Logf(" got: %X", got)
				t.Logf("want: %X", tc.md)
			}
		}
	}
	test("SHA384ShortMsg")
	test("SHA384LongMsg")
}

func TestBitArray_SHA512_256_cavp(t *testing.T) {
	test := func(name string) {
		tcs, err := cavpTestCases(name)
		if err != nil {
			t.Fatalf("failed to load test cases: %s: %s", name, err)
		}
		for i, tc := range tcs {
			got := tc.ba.SHA512_256()
			if !bytes.Equal(got[:], tc.md) {
				t.Errorf("unexpected hash: %s: #%d", name, i)
				t.Logf("mlen: %d", tc.ba.Len())
				t.Logf(" msg: %#b", tc.ba)
				t.Logf(" got: %X", got)
				t.Logf("want: %X", tc.md)
			}
		}
	}
	test("SHA512_256ShortMsg")
	test("SHA512_256LongMsg")
}

func TestBitArray_SHA512_224_cavp(t *testing.T) {
	test := func(name string) {
		tcs, err := cavpTestCases(name)
		if err != nil {
			t.Fatalf("failed to load test cases: %s: %s", name, err)
		}
		for i, tc := range tcs {
			got := tc.ba.SHA512_224()
			if !bytes.Equal(got[:], tc.md) {
				t.Errorf("unexpected hash: %s: #%d", name, i)
				t.Logf("mlen: %d", tc.ba.Len())
				t.Logf(" msg: %#b", tc.ba)
				t.Logf(" got: %X", got)
				t.Logf("want: %X", tc.md)
			}
		}
	}
	test("SHA512_224ShortMsg")
	test("SHA512_224LongMsg")
}
