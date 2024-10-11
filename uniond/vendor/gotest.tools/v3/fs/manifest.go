package fs

import (
	"fmt"
	"io"
	"os"
	"path/filepath"

	"gotest.tools/v3/assert"
)

// Manifest stores the expected structure and properties of files and directories
// in a filesystem.
type Manifest struct {
	root *directory
}

type resource struct {
	mode os.FileMode
	uid  uint32
	gid  uint32
}

type file struct {
	resource
	content             io.ReadCloser
	ignoreCariageReturn bool
	compareContentFunc  func(b []byte) CompareResult
}

func (f *file) Type() string {
	return "file"
}

type symlink struct {
	resource
	target string
}

func (f *symlink) Type() string {
	return "symlink"
}

type directory struct {
	resource
	items         map[string]dirEntry
	filepathGlobs map[string]*filePath
}

func (f *directory) Type() string {
	return "directory"
}

type dirEntry interface {
	Type() string
}

// ManifestFromDir creates a [Manifest] by reading the directory at path. The
// manifest stores the structure and properties of files in the directory.
// ManifestFromDir can be used with [Equal] to compare two directories.
func ManifestFromDir(t assert.TestingT, path string) Manifest {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}

	manifest, err := manifestFromDir(path)
	assert.NilError(t, err)
	return manifest
}

func manifestFromDir(path string) (Manifest, error) {
	info, err := os.Stat(path)
	switch {
	case err != nil:
		return Manifest{}, err
	case !info.IsDir():
		return Manifest{}, fmt.Errorf("path %s must be a directory", path)
	}

	directory, err := newDirectory(path, info)
	return Manifest{root: directory}, err
}

func newDirectory(path string, info os.FileInfo) (*directory, error) {
	items := make(map[string]dirEntry)
	children, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}
	for _, child := range children {
		fullPath := filepath.Join(path, child.Name())
		items[child.Name()], err = getTypedResource(fullPath, child)
		if err != nil {
			return nil, err
		}
	}

	return &directory{
		resource:      newResourceFromInfo(info),
		items:         items,
		filepathGlobs: make(map[string]*filePath),
	}, nil
}

func getTypedResource(path string, entry os.DirEntry) (dirEntry, error) {
	info, err := entry.Info()
	if err != nil {
		return nil, err
	}
	switch {
	case info.IsDir():
		return newDirectory(path, info)
	case info.Mode()&os.ModeSymlink != 0:
		return newSymlink(path, info)
	// TODO: devices, pipes?
	default:
		return newFile(path, info)
	}
}

func newSymlink(path string, info os.FileInfo) (*symlink, error) {
	target, err := os.Readlink(path)
	if err != nil {
		return nil, err
	}
	return &symlink{
		resource: newResourceFromInfo(info),
		target:   target,
	}, err
}

func newFile(path string, info os.FileInfo) (*file, error) {
	// TODO: defer file opening to reduce number of open FDs?
	readCloser, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	return &file{
		resource: newResourceFromInfo(info),
		content:  readCloser,
	}, err
}
