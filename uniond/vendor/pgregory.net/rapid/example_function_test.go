// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"fmt"
	"strconv"
	"testing"

	"pgregory.net/rapid"
)

// ParseDate parses dates in the YYYY-MM-DD format.
func ParseDate(s string) (int, int, int, error) {
	if len(s) != 10 {
		return 0, 0, 0, fmt.Errorf("%q has wrong length: %v instead of 10", s, len(s))
	}

	if s[4] != '-' || s[7] != '-' {
		return 0, 0, 0, fmt.Errorf("'-' separators expected in %q", s)
	}

	y, err := strconv.Atoi(s[0:4])
	if err != nil {
		return 0, 0, 0, fmt.Errorf("failed to parse year: %v", err)
	}

	m, err := strconv.Atoi(s[6:7])
	if err != nil {
		return 0, 0, 0, fmt.Errorf("failed to parse month: %v", err)
	}

	d, err := strconv.Atoi(s[8:10])
	if err != nil {
		return 0, 0, 0, fmt.Errorf("failed to parse day: %v", err)
	}

	return y, m, d, nil
}

func testParseDate(t *rapid.T) {
	y := rapid.IntRange(0, 9999).Draw(t, "y")
	m := rapid.IntRange(1, 12).Draw(t, "m")
	d := rapid.IntRange(1, 31).Draw(t, "d")

	s := fmt.Sprintf("%04d-%02d-%02d", y, m, d)

	y_, m_, d_, err := ParseDate(s)
	if err != nil {
		t.Fatalf("failed to parse date %q: %v", s, err)
	}

	if y_ != y || m_ != m || d_ != d {
		t.Fatalf("got back wrong date: (%d, %d, %d)", y_, m_, d_)
	}
}

// Rename to TestParseDate(t *testing.T) to make an actual (failing) test.
func ExampleCheck_parseDate() {
	var t *testing.T
	rapid.Check(t, testParseDate)
}
