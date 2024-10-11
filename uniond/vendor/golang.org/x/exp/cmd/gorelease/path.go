// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"path/filepath"
	"strings"
)

// hasPathPrefix reports whether the slash-separated path s
// begins with the elements in prefix.
// Copied from cmd/go/internal/str.HasPathPrefix.
func hasPathPrefix(s, prefix string) bool {
	if len(s) == len(prefix) {
		return s == prefix
	}
	if prefix == "" {
		return true
	}
	if len(s) > len(prefix) {
		if prefix[len(prefix)-1] == '/' || s[len(prefix)] == '/' {
			return s[:len(prefix)] == prefix
		}
	}
	return false
}

// hasFilePathPrefix reports whether the filesystem path s
// begins with the elements in prefix.
// Copied from cmd/go/internal/str.HasFilePathPrefix.
func hasFilePathPrefix(s, prefix string) bool {
	sv := strings.ToUpper(filepath.VolumeName(s))
	pv := strings.ToUpper(filepath.VolumeName(prefix))
	s = s[len(sv):]
	prefix = prefix[len(pv):]
	switch {
	default:
		return false
	case pv != "" && sv != pv:
		return false
	case len(s) == len(prefix):
		return s == prefix
	case prefix == "":
		return true
	case len(s) > len(prefix):
		if prefix[len(prefix)-1] == filepath.Separator {
			return strings.HasPrefix(s, prefix)
		}
		return s[len(prefix)] == filepath.Separator && s[:len(prefix)] == prefix
	}
}

// trimFilePathPrefix returns the given filesystem path s without the leading
// prefix.
func trimFilePathPrefix(s, prefix string) string {
	sv := strings.ToUpper(filepath.VolumeName(s))
	pv := strings.ToUpper(filepath.VolumeName(prefix))
	s = s[len(sv):]
	prefix = prefix[len(pv):]

	if !hasFilePathPrefix(s, prefix) || len(prefix) == 0 {
		return s
	}
	if len(s) == len(prefix) {
		return ""
	}
	if prefix[len(prefix)-1] == filepath.Separator {
		return strings.TrimPrefix(s, prefix)
	}
	return s[len(prefix)+1:]
}

// trimPathPrefix returns p without the leading prefix. Unlike
// strings.TrimPrefix, the prefix will only match on slash-separted component
// boundaries, so trimPathPrefix("aa/b", "aa") returns "b", but
// trimPathPrefix("aa/b", "a") returns "aa/b".
func trimPathPrefix(p, prefix string) string {
	if prefix == "" {
		return p
	}
	if prefix == p {
		return ""
	}
	return strings.TrimPrefix(p, prefix+"/")
}
