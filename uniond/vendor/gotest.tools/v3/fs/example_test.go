package fs_test

import (
	"os"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/fs"
	"gotest.tools/v3/golden"
)

var t = &testing.T{}

// Create a temporary directory which contains a single file
func ExampleNewDir() {
	dir := fs.NewDir(t, "test-name", fs.WithFile("file1", "content\n"))
	defer dir.Remove()

	files, err := os.ReadDir(dir.Path())
	assert.NilError(t, err)
	assert.Assert(t, cmp.Len(files, 0))
}

// Create a new file with some content
func ExampleNewFile() {
	file := fs.NewFile(t, "test-name", fs.WithContent("content\n"), fs.AsUser(0, 0))
	defer file.Remove()

	content, err := os.ReadFile(file.Path())
	assert.NilError(t, err)
	assert.Equal(t, "content\n", content)
}

// Create a directory and subdirectory with files
func ExampleWithDir() {
	dir := fs.NewDir(t, "test-name",
		fs.WithDir("subdir",
			fs.WithMode(os.FileMode(0700)),
			fs.WithFile("file1", "content\n")),
	)
	defer dir.Remove()
}

// Test that a directory contains the expected files, and all the files have the
// expected properties.
func ExampleEqual() {
	path := operationWhichCreatesFiles()
	expected := fs.Expected(t,
		fs.WithFile("one", "",
			fs.WithBytes(golden.Get(t, "one.golden")),
			fs.WithMode(0600)),
		fs.WithDir("data",
			fs.WithFile("config", "", fs.MatchAnyFileContent)))

	assert.Assert(t, fs.Equal(path, expected))
}

func operationWhichCreatesFiles() string {
	return "example-path"
}

// Add a file to an existing directory
func ExampleApply() {
	dir := fs.NewDir(t, "test-name")
	defer dir.Remove()

	fs.Apply(t, dir, fs.WithFile("file1", "content\n"))
}
