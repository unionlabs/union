// Copyright 2013 The Go Authors. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://developers.google.com/open-source/licenses/bsd.

package httputil

import (
	"net/http"
	"testing"
)

var negotiateContentEncodingTests = []struct {
	s      string
	offers []string
	expect string
}{
	{"", []string{"identity", "gzip"}, "identity"},
	{"*;q=0", []string{"identity", "gzip"}, ""},
	{"gzip", []string{"identity", "gzip"}, "gzip"},
	{"gzip,zstd", []string{"identity", "zstd"}, "zstd"},
	{"zstd,gzip", []string{"gzip", "zstd"}, "gzip"},
	{"gzip,zstd", []string{"gzip", "zstd"}, "gzip"},
	{"gzip,zstd", []string{"zstd", "gzip"}, "zstd"},
	{"gzip;q=0.1,zstd;q=0.5", []string{"gzip", "zstd"}, "zstd"},
	{"gzip;q=1.0, identity; q=0.5, *;q=0", []string{"identity", "gzip"}, "gzip"},
	{"gzip;q=1.0, identity; q=0.5, *;q=0", []string{"identity", "zstd"}, "identity"},
	{"zstd", []string{"identity", "gzip"}, "identity"},
}

func TestNegotiateContentEncoding(t *testing.T) {
	for _, tt := range negotiateContentEncodingTests {
		r := &http.Request{Header: http.Header{"Accept-Encoding": {tt.s}}}
		actual := NegotiateContentEncoding(r, tt.offers)
		if actual != tt.expect {
			t.Errorf("NegotiateContentEncoding(%q, %#v)=%q, want %q", tt.s, tt.offers, actual, tt.expect)
		}
	}
}
