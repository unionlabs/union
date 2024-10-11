package fs_test

import (
	"errors"
	"os"
	"path/filepath"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/fs"
	"gotest.tools/v3/internal/source"
	"gotest.tools/v3/skip"
)

func TestNewDirWithOpsAndManifestEqual(t *testing.T) {
	var userOps []fs.PathOp
	if os.Geteuid() == 0 {
		userOps = append(userOps, fs.AsUser(1001, 1002))
	}

	ops := []fs.PathOp{
		fs.WithFile("file1", "contenta", fs.WithMode(0400)),
		fs.WithFile("file2", "", fs.WithBytes([]byte{0, 1, 2})),
		fs.WithFile("file5", "", userOps...),
		fs.WithSymlink("link1", "file1"),
		fs.WithDir("sub",
			fs.WithFiles(map[string]string{
				"file3": "contentb",
				"file4": "contentc",
			}),
			fs.WithMode(0705),
		),
	}

	dir := fs.NewDir(t, "test-all", ops...)
	defer dir.Remove()

	manifestOps := append(
		ops[:3],
		fs.WithSymlink("link1", dir.Join("file1")),
		ops[4],
	)
	assert.Assert(t, fs.Equal(dir.Path(), fs.Expected(t, manifestOps...)))
}

func TestNewFile(t *testing.T) {
	t.Run("with test name", func(t *testing.T) {
		tmpFile := fs.NewFile(t, t.Name())
		_, err := os.Stat(tmpFile.Path())
		assert.NilError(t, err)

		tmpFile.Remove()
		_, err = os.Stat(tmpFile.Path())
		assert.ErrorIs(t, err, os.ErrNotExist)
	})

	t.Run(`with \ in name`, func(t *testing.T) {
		tmpFile := fs.NewFile(t, `foo\thing`)
		_, err := os.Stat(tmpFile.Path())
		assert.NilError(t, err)

		tmpFile.Remove()
		_, err = os.Stat(tmpFile.Path())
		assert.ErrorIs(t, err, os.ErrNotExist)
	})
}

func TestNewFile_IntegrationWithCleanup(t *testing.T) {
	skip.If(t, source.GoVersionLessThan(1, 14))
	var tmpFile *fs.File
	t.Run("cleanup in subtest", func(t *testing.T) {
		tmpFile = fs.NewFile(t, t.Name())
		_, err := os.Stat(tmpFile.Path())
		assert.NilError(t, err)
	})

	t.Run("file has been removed", func(t *testing.T) {
		_, err := os.Stat(tmpFile.Path())
		assert.ErrorIs(t, err, os.ErrNotExist)
	})
}

func TestNewDir_IntegrationWithCleanup(t *testing.T) {
	skip.If(t, source.GoVersionLessThan(1, 14))
	var tmpFile *fs.Dir
	t.Run("cleanup in subtest", func(t *testing.T) {
		tmpFile = fs.NewDir(t, t.Name())
		_, err := os.Stat(tmpFile.Path())
		assert.NilError(t, err)
	})

	t.Run("dir has been removed", func(t *testing.T) {
		_, err := os.Stat(tmpFile.Path())
		assert.ErrorIs(t, err, os.ErrNotExist)
	})
}

func TestDirFromPath(t *testing.T) {
	tmpdir := t.TempDir()

	dir := fs.DirFromPath(t, tmpdir, fs.WithFile("newfile", ""))

	_, err := os.Stat(dir.Join("newfile"))
	assert.NilError(t, err)

	assert.Equal(t, dir.Path(), tmpdir)
	assert.Equal(t, dir.Join("newfile"), filepath.Join(tmpdir, "newfile"))

	dir.Remove()

	_, err = os.Stat(tmpdir)
	assert.Assert(t, errors.Is(err, os.ErrNotExist))
}
