// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"runtime"
	"testing"
)

func TestHasPathPrefix(t *testing.T) {
	for _, test := range []struct {
		desc, path, prefix string
		want               bool
	}{
		{
			desc:   "empty_prefix",
			path:   "a/b",
			prefix: "",
			want:   true,
		}, {
			desc:   "partial_prefix",
			path:   "a/b",
			prefix: "a",
			want:   true,
		}, {
			desc:   "full_prefix",
			path:   "a/b",
			prefix: "a/b",
			want:   true,
		}, {
			desc:   "partial_component",
			path:   "aa/b",
			prefix: "a",
			want:   false,
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			if got := hasPathPrefix(test.path, test.prefix); got != test.want {
				t.Errorf("hasPathPrefix(%q, %q): got %v, want %v", test.path, test.prefix, got, test.want)
			}
		})
	}
}

func TestHasFilePathPrefix(t *testing.T) {
	type test struct {
		desc, path, prefix string
		want               bool
	}
	var tests []test
	if runtime.GOOS == "windows" {
		tests = []test{
			{
				desc:   "empty_prefix",
				path:   `c:\a\b`,
				prefix: "",
				want:   true,
			}, {
				desc:   "drive_prefix",
				path:   `c:\a\b`,
				prefix: `c:\`,
				want:   true,
			}, {
				desc:   "partial_prefix",
				path:   `c:\a\b`,
				prefix: `c:\a`,
				want:   true,
			}, {
				desc:   "full_prefix",
				path:   `c:\a\b`,
				prefix: `c:\a\b`,
				want:   true,
			}, {
				desc:   "partial_component",
				path:   `c:\aa\b`,
				prefix: `c:\a`,
				want:   false,
			},
		}
	} else {
		tests = []test{
			{
				desc:   "empty_prefix",
				path:   "/a/b",
				prefix: "",
				want:   true,
			}, {
				desc:   "partial_prefix",
				path:   "/a/b",
				prefix: "/a",
				want:   true,
			}, {
				desc:   "full_prefix",
				path:   "/a/b",
				prefix: "/a/b",
				want:   true,
			}, {
				desc:   "partial_component",
				path:   "/aa/b",
				prefix: "/a",
				want:   false,
			},
		}
	}
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if got := hasFilePathPrefix(test.path, test.prefix); got != test.want {
				t.Errorf("hasFilePathPrefix(%q, %q): got %v, want %v", test.path, test.prefix, got, test.want)
			}
		})
	}
}

func TestTrimFilePathPrefix(t *testing.T) {
	type test struct {
		desc, path, prefix, want string
	}
	var tests []test
	if runtime.GOOS == "windows" {
		tests = []test{
			// Note: these two cases in which the result preserves the leading \
			// don't come up in reality in gorelease. That's because prefix is
			// always far to the right of the path parts (ex github.com/foo/bar
			// in C:\Users\foo\AppData\Local\Temp\...\github.com\foo\bar).
			{
				desc:   "empty_prefix",
				path:   `c:\a\b`,
				prefix: "",
				want:   `\a\b`,
			}, {
				desc:   "partial_component",
				path:   `c:\aa\b`,
				prefix: `c:\a`,
				want:   `\aa\b`,
			},

			{
				desc:   "drive_prefix",
				path:   `c:\a\b`,
				prefix: `c:\`,
				want:   `a\b`,
			}, {
				desc:   "partial_prefix",
				path:   `c:\a\b`,
				prefix: `c:\a`,
				want:   `b`,
			}, {
				desc:   "full_prefix",
				path:   `c:\a\b`,
				prefix: `c:\a\b`,
				want:   "",
			},
		}
	} else {
		tests = []test{
			{
				desc:   "empty_prefix",
				path:   "/a/b",
				prefix: "",
				want:   "/a/b",
			}, {
				desc:   "partial_prefix",
				path:   "/a/b",
				prefix: "/a",
				want:   "b",
			}, {
				desc:   "full_prefix",
				path:   "/a/b",
				prefix: "/a/b",
				want:   "",
			}, {
				desc:   "partial_component",
				path:   "/aa/b",
				prefix: "/a",
				want:   "/aa/b",
			},
		}
	}
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if got := trimFilePathPrefix(test.path, test.prefix); got != test.want {
				t.Errorf("hasFilePathPrefix(%q, %q): got %v, want %v", test.path, test.prefix, got, test.want)
			}
		})
	}
}

func TestTrimPathPrefix(t *testing.T) {
	for _, test := range []struct {
		desc, path, prefix, want string
	}{
		{
			desc:   "empty_prefix",
			path:   "a/b",
			prefix: "",
			want:   "a/b",
		}, {
			desc:   "abs_empty_prefix",
			path:   "/a/b",
			prefix: "",
			want:   "/a/b",
		}, {
			desc:   "partial_prefix",
			path:   "a/b",
			prefix: "a",
			want:   "b",
		}, {
			desc:   "full_prefix",
			path:   "a/b",
			prefix: "a/b",
			want:   "",
		}, {
			desc:   "partial_component",
			path:   "aa/b",
			prefix: "a",
			want:   "aa/b",
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			if got := trimPathPrefix(test.path, test.prefix); got != test.want {
				t.Errorf("trimPathPrefix(%q, %q): got %q, want %q", test.path, test.prefix, got, test.want)
			}
		})
	}
}
