// Copyright 2013 The Go Authors. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://developers.google.com/open-source/licenses/bsd.

package header

import (
	"net/http"
	"testing"

	"github.com/google/go-cmp/cmp"
)

var parseAcceptTests = []struct {
	s        string
	expected []AcceptSpec
}{
	{"text/html", []AcceptSpec{{"text/html", 1}}},
	{"text/html; q=0", []AcceptSpec{{"text/html", 0}}},
	{"text/html; q=0.0", []AcceptSpec{{"text/html", 0}}},
	{"text/html; q=1", []AcceptSpec{{"text/html", 1}}},
	{"text/html; q=1.0", []AcceptSpec{{"text/html", 1}}},
	{"text/html; q=0.1", []AcceptSpec{{"text/html", 0.1}}},
	{"text/html;q=0.1", []AcceptSpec{{"text/html", 0.1}}},
	{"text/html, text/plain", []AcceptSpec{{"text/html", 1}, {"text/plain", 1}}},
	{"text/html; q=0.1, text/plain", []AcceptSpec{{"text/html", 0.1}, {"text/plain", 1}}},
	{"iso-8859-5, unicode-1-1;q=0.8,iso-8859-1", []AcceptSpec{{"iso-8859-5", 1}, {"unicode-1-1", 0.8}, {"iso-8859-1", 1}}},
	{"iso-8859-1", []AcceptSpec{{"iso-8859-1", 1}}},
	{"*", []AcceptSpec{{"*", 1}}},
	{"da, en-gb;q=0.8, en;q=0.7", []AcceptSpec{{"da", 1}, {"en-gb", 0.8}, {"en", 0.7}}},
	{"da, q, en-gb;q=0.8", []AcceptSpec{{"da", 1}, {"q", 1}, {"en-gb", 0.8}}},
	{"image/png, image/*;q=0.5", []AcceptSpec{{"image/png", 1}, {"image/*", 0.5}}},

	// bad cases
	{"value1; q=0.1.2", []AcceptSpec{{"value1", 0.1}}},
	{"da, en-gb;q=foo", []AcceptSpec{{"da", 1}}},
}

func TestParseAccept(t *testing.T) {
	for _, tt := range parseAcceptTests {
		header := http.Header{"Accept": {tt.s}}
		actual := ParseAccept(header, "Accept")
		if !cmp.Equal(actual, tt.expected) {
			t.Errorf("ParseAccept(h, %q)=%v, want %v", tt.s, actual, tt.expected)
		}
	}
}
