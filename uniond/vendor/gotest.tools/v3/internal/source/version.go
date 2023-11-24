package source

import (
	"runtime"
	"strconv"
	"strings"
)

// GoVersionLessThan returns true if runtime.Version() is semantically less than
// version major.minor. Returns false if a release version can not be parsed from
// runtime.Version().
func GoVersionLessThan(major, minor int64) bool {
	version := runtime.Version()
	// not a release version
	if !strings.HasPrefix(version, "go") {
		return false
	}
	version = strings.TrimPrefix(version, "go")
	parts := strings.Split(version, ".")
	if len(parts) < 2 {
		return false
	}
	rMajor, err := strconv.ParseInt(parts[0], 10, 32)
	if err != nil {
		return false
	}
	if rMajor != major {
		return rMajor < major
	}
	rMinor, err := strconv.ParseInt(parts[1], 10, 32)
	if err != nil {
		return false
	}
	return rMinor < minor
}
