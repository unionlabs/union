// Package semver is a thin forwarding layer on top of
// [golang.org/x/mod/semver]. See that package for documentation.
//
// Deprecated: use [golang.org/x/mod/semver] instead.
package semver

import "golang.org/x/mod/semver"

func IsValid(v string) bool {
	return semver.IsValid(v)
}

func Canonical(v string) string {
	return semver.Canonical(v)
}

func Major(v string) string {
	return semver.Major(v)
}

func MajorMinor(v string) string {
	return semver.MajorMinor(v)
}

func Prerelease(v string) string {
	return semver.Prerelease(v)
}

func Build(v string) string {
	return semver.Build(v)
}

func Compare(v, w string) int {
	return semver.Compare(v, w)
}

func Max(v, w string) string {
	return semver.Max(v, w)
}
