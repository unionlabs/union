// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package zip

import "testing"

func TestIsVendoredPackage(t *testing.T) {
	for _, tc := range []struct {
		path          string
		want          bool
		falsePositive bool // is this case affected by https://golang.org/issue/37397?
	}{
		{path: "vendor/foo/foo.go", want: true},
		{path: "pkg/vendor/foo/foo.go", want: true},
		{path: "longpackagename/vendor/foo/foo.go", want: true},

		{path: "vendor/vendor.go", want: false},

		// We ideally want these cases to be false, but they are affected by
		// https://golang.org/issue/37397, and if we fix them we will invalidate
		// existing module checksums. We must leave them as-is-for now.
		{path: "pkg/vendor/vendor.go", falsePositive: true},
		{path: "longpackagename/vendor/vendor.go", falsePositive: true},
	} {
		got := isVendoredPackage(tc.path)
		want := tc.want
		if tc.falsePositive {
			want = true
		}
		if got != want {
			t.Errorf("isVendoredPackage(%q) = %t; want %t", tc.path, got, tc.want)
			if tc.falsePositive {
				t.Logf("(Expected a false-positive due to https://golang.org/issue/37397.)")
			}
		}
	}
}
