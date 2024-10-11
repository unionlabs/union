// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package cpu

import "testing"

type parseReleaseTest struct {
	in                  string
	major, minor, patch int
}

var parseReleaseTests = []parseReleaseTest{
	{"", -1, -1, -1},
	{"x", -1, -1, -1},
	{"5", 5, 0, 0},
	{"5.12", 5, 12, 0},
	{"5.12-x", 5, 12, 0},
	{"5.12.1", 5, 12, 1},
	{"5.12.1-x", 5, 12, 1},
	{"5.12.1.0", 5, 12, 1},
	{"5.20496382327982653440", -1, -1, -1},
}

func TestParseRelease(t *testing.T) {
	for _, test := range parseReleaseTests {
		major, minor, patch, ok := parseRelease(test.in)
		if !ok {
			major, minor, patch = -1, -1, -1
		}
		if test.major != major || test.minor != minor || test.patch != patch {
			t.Errorf("parseRelease(%q) = (%v, %v, %v) want (%v, %v, %v)",
				test.in, major, minor, patch,
				test.major, test.minor, test.patch)
		}
	}
}
