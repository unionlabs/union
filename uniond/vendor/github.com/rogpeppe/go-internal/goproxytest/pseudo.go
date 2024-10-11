// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package goproxytest

import (
	"regexp"
	"strings"

	"golang.org/x/mod/semver"
)

// This code was taken from cmd/go/internal/modfetch/pseudo.go

var pseudoVersionRE = regexp.MustCompile(`^v[0-9]+\.(0\.0-|\d+\.\d+-([^+]*\.)?0\.)\d{14}-[A-Za-z0-9]+(\+incompatible)?$`)

// isPseudoVersion reports whether v is a pseudo-version.
func isPseudoVersion(v string) bool {
	return strings.Count(v, "-") >= 2 && semver.IsValid(v) && pseudoVersionRE.MatchString(v)
}
