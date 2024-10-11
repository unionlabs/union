package fs

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"

	"gotest.tools/v3/assert"
)

const defaultFileMode = 0644

// PathOp is a function which accepts a [Path] and performs an operation on that
// path. When called with real filesystem objects ([File] or [Dir]) a PathOp modifies
// the filesystem at the path. When used with a [Manifest] object a PathOp updates
// the manifest to expect a value.
type PathOp func(path Path) error

type manifestResource interface {
	SetMode(mode os.FileMode)
	SetUID(uid uint32)
	SetGID(gid uint32)
}

type manifestFile interface {
	manifestResource
	SetContent(content io.ReadCloser)
}

type manifestDirectory interface {
	manifestResource
	AddSymlink(path, target string) error
	AddFile(path string, ops ...PathOp) error
	AddDirectory(path string, ops ...PathOp) error
}

// WithContent writes content to a file at [Path]
func WithContent(content string) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestFile); ok {
			m.SetContent(io.NopCloser(strings.NewReader(content)))
			return nil
		}
		return os.WriteFile(path.Path(), []byte(content), defaultFileMode)
	}
}

// WithBytes write bytes to a file at [Path]
func WithBytes(raw []byte) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestFile); ok {
			m.SetContent(io.NopCloser(bytes.NewReader(raw)))
			return nil
		}
		return os.WriteFile(path.Path(), raw, defaultFileMode)
	}
}

// WithReaderContent copies the reader contents to the file at [Path]
func WithReaderContent(r io.Reader) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestFile); ok {
			m.SetContent(io.NopCloser(r))
			return nil
		}
		f, err := os.OpenFile(path.Path(), os.O_WRONLY, defaultFileMode)
		if err != nil {
			return err
		}
		defer f.Close()
		_, err = io.Copy(f, r)
		return err
	}
}

// AsUser changes ownership of the file system object at [Path]
func AsUser(uid, gid int) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestResource); ok {
			m.SetUID(uint32(uid))
			m.SetGID(uint32(gid))
			return nil
		}
		return os.Chown(path.Path(), uid, gid)
	}
}

// WithFile creates a file in the directory at path with content
func WithFile(filename, content string, ops ...PathOp) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestDirectory); ok {
			ops = append([]PathOp{WithContent(content), WithMode(defaultFileMode)}, ops...)
			return m.AddFile(filename, ops...)
		}

		fullpath := filepath.Join(path.Path(), filepath.FromSlash(filename))
		if err := createFile(fullpath, content); err != nil {
			return err
		}
		return applyPathOps(&File{path: fullpath}, ops)
	}
}

func createFile(fullpath string, content string) error {
	return os.WriteFile(fullpath, []byte(content), defaultFileMode)
}

// WithFiles creates all the files in the directory at path with their content
func WithFiles(files map[string]string) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestDirectory); ok {
			for filename, content := range files {
				// TODO: remove duplication with WithFile
				if err := m.AddFile(filename, WithContent(content), WithMode(defaultFileMode)); err != nil {
					return err
				}
			}
			return nil
		}

		for filename, content := range files {
			fullpath := filepath.Join(path.Path(), filepath.FromSlash(filename))
			if err := createFile(fullpath, content); err != nil {
				return err
			}
		}
		return nil
	}
}

// FromDir copies the directory tree from the source path into the new [Dir]
func FromDir(source string) PathOp {
	return func(path Path) error {
		if _, ok := path.(manifestDirectory); ok {
			return fmt.Errorf("use manifest.FromDir")
		}
		return copyDirectory(source, path.Path())
	}
}

// WithDir creates a subdirectory in the directory at path. Additional [PathOp]
// can be used to modify the subdirectory
func WithDir(name string, ops ...PathOp) PathOp {
	const defaultMode = 0755
	return func(path Path) error {
		if m, ok := path.(manifestDirectory); ok {
			ops = append([]PathOp{WithMode(defaultMode)}, ops...)
			return m.AddDirectory(name, ops...)
		}

		fullpath := filepath.Join(path.Path(), filepath.FromSlash(name))
		err := os.MkdirAll(fullpath, defaultMode)
		if err != nil {
			return err
		}
		return applyPathOps(&Dir{path: fullpath}, ops)
	}
}

// Apply the PathOps to the [File]
func Apply(t assert.TestingT, path Path, ops ...PathOp) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	assert.NilError(t, applyPathOps(path, ops))
}

func applyPathOps(path Path, ops []PathOp) error {
	for _, op := range ops {
		if err := op(path); err != nil {
			return err
		}
	}
	return nil
}

// WithMode sets the file mode on the directory or file at [Path]
func WithMode(mode os.FileMode) PathOp {
	return func(path Path) error {
		if m, ok := path.(manifestResource); ok {
			m.SetMode(mode)
			return nil
		}
		return os.Chmod(path.Path(), mode)
	}
}

func copyDirectory(source, dest string) error {
	entries, err := os.ReadDir(source)
	if err != nil {
		return err
	}
	for _, entry := range entries {
		sourcePath := filepath.Join(source, entry.Name())
		destPath := filepath.Join(dest, entry.Name())
		err = copyEntry(entry, destPath, sourcePath)
		if err != nil {
			return err
		}
	}
	return nil
}

func copyEntry(entry os.DirEntry, destPath string, sourcePath string) error {
	if entry.IsDir() {
		if err := os.Mkdir(destPath, 0755); err != nil {
			return err
		}
		return copyDirectory(sourcePath, destPath)
	}
	info, err := entry.Info()
	if err != nil {
		return err
	}
	if info.Mode()&os.ModeSymlink != 0 {
		return copySymLink(sourcePath, destPath)
	}
	return copyFile(sourcePath, destPath)
}

func copySymLink(source, dest string) error {
	link, err := os.Readlink(source)
	if err != nil {
		return err
	}
	return os.Symlink(link, dest)
}

func copyFile(source, dest string) error {
	content, err := os.ReadFile(source)
	if err != nil {
		return err
	}
	return os.WriteFile(dest, content, 0644)
}

// WithSymlink creates a symlink in the directory which links to target.
// Target must be a path relative to the directory.
//
// Note: the argument order is the inverse of [os.Symlink] to be consistent with
// the other functions in this package.
func WithSymlink(path, target string) PathOp {
	return func(root Path) error {
		if v, ok := root.(manifestDirectory); ok {
			return v.AddSymlink(path, target)
		}
		return os.Symlink(filepath.Join(root.Path(), target), filepath.Join(root.Path(), path))
	}
}

// WithHardlink creates a link in the directory which links to target.
// Target must be a path relative to the directory.
//
// Note: the argument order is the inverse of [os.Link] to be consistent with
// the other functions in this package.
func WithHardlink(path, target string) PathOp {
	return func(root Path) error {
		if _, ok := root.(manifestDirectory); ok {
			return fmt.Errorf("WithHardlink not implemented for manifests")
		}
		return os.Link(filepath.Join(root.Path(), target), filepath.Join(root.Path(), path))
	}
}

// WithTimestamps sets the access and modification times of the file system object
// at path.
func WithTimestamps(atime, mtime time.Time) PathOp {
	return func(root Path) error {
		if _, ok := root.(manifestDirectory); ok {
			return fmt.Errorf("WithTimestamp not implemented for manifests")
		}
		return os.Chtimes(root.Path(), atime, mtime)
	}
}
