// Package module is a thin forwarding layer on top of
// [golang.org/x/mod/module]. Note that the Encode* and
// Decode* functions map to Escape* and Unescape*
// in that package.
//
// See that package for documentation on everything else.
//
// Deprecated: use [golang.org/x/mod/module] instead.
package module

import "golang.org/x/mod/module"

type Version = module.Version

func Check(path, version string) error {
	return module.Check(path, version)
}

func CheckPath(path string) error {
	return module.CheckPath(path)
}

func CheckImportPath(path string) error {
	return module.CheckImportPath(path)
}

func CheckFilePath(path string) error {
	return module.CheckFilePath(path)
}

func SplitPathVersion(path string) (prefix, pathMajor string, ok bool) {
	return module.SplitPathVersion(path)
}

func MatchPathMajor(v, pathMajor string) bool {
	return module.MatchPathMajor(v, pathMajor)
}

func CanonicalVersion(v string) string {
	return module.CanonicalVersion(v)
}

func Sort(list []Version) {
	module.Sort(list)
}

func EncodePath(path string) (encoding string, err error) {
	return module.EscapePath(path)
}

func EncodeVersion(v string) (encoding string, err error) {
	return module.EscapeVersion(v)
}

func DecodePath(encoding string) (path string, err error) {
	return module.UnescapePath(encoding)
}

func DecodeVersion(encoding string) (v string, err error) {
	return module.UnescapeVersion(encoding)
}
