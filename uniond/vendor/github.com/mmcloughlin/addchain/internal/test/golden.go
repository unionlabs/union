package test

import (
	"flag"
	"path/filepath"
)

// Flags controlling whether to write testdata files.
var golden = flag.Bool("golden", false, "write golden testdata files")

// Golden reports whether to write golden testdata files.
func Golden() bool {
	return *golden
}

// GoldenName returns a path to the named golden file.
func GoldenName(name string) string {
	return filepath.Join("testdata", name+".golden")
}
