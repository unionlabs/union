package fs_test

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/fs"
)

func TestFromDir(t *testing.T) {
	dir := fs.NewDir(t, "test-from-dir", fs.FromDir("testdata/copy-test"))
	defer dir.Remove()

	expected := fs.Expected(t,
		fs.WithFile("1", "1\n"),
		fs.WithDir("a",
			fs.WithFile("1", "1\n"),
			fs.WithFile("2", "2\n"),
			fs.WithDir("b",
				fs.WithFile("1", "1\n"))))

	assert.Assert(t, fs.Equal(dir.Path(), expected))
}

func TestFromDirSymlink(t *testing.T) {
	dir := fs.NewDir(t, "test-from-dir", fs.FromDir("testdata/copy-test-with-symlink"))
	defer dir.Remove()

	currentdir, err := os.Getwd()
	assert.NilError(t, err)

	link2 := filepath.FromSlash("../2")
	link3 := "/some/inexistent/link"
	if runtime.GOOS == "windows" {
		link3 = filepath.Join(filepath.VolumeName(currentdir), link3)
	}

	expected := fs.Expected(t,
		fs.WithFile("1", "1\n"),
		fs.WithDir("a",
			fs.WithFile("1", "1\n"),
			fs.WithFile("2", "2\n"),
			fs.WithDir("b",
				fs.WithFile("1", "1\n"),
				fs.WithSymlink("2", link2),
				fs.WithSymlink("3", link3),
				fs.WithSymlink("4", "5"),
			)))

	assert.Assert(t, fs.Equal(dir.Path(), expected))
}

func TestWithTimestamps(t *testing.T) {
	stamp := time.Date(2011, 11, 11, 5, 55, 55, 0, time.UTC)
	tmpFile := fs.NewFile(t, t.Name(), fs.WithTimestamps(stamp, stamp))
	defer tmpFile.Remove()

	stat, err := os.Stat(tmpFile.Path())
	assert.NilError(t, err)
	assert.DeepEqual(t, stat.ModTime(), stamp)
}

func TestApply(t *testing.T) {
	t.Run("with file", func(t *testing.T) {
		tmpFile := fs.NewFile(t, "test-update-file", fs.WithContent("contenta"))
		defer tmpFile.Remove()
		fs.Apply(t, tmpFile, fs.WithContent("contentb"))
		content, err := os.ReadFile(tmpFile.Path())
		assert.NilError(t, err)
		assert.Equal(t, string(content), "contentb")
	})

	t.Run("with dir", func(t *testing.T) {
		tmpDir := fs.NewDir(t, "test-update-dir")
		defer tmpDir.Remove()
		fs.Apply(t, tmpDir, fs.WithFile("file1", "contenta"))
		fs.Apply(t, tmpDir, fs.WithFile("file2", "contentb"))
		expected := fs.Expected(t,
			fs.WithFile("file1", "contenta"),
			fs.WithFile("file2", "contentb"))
		assert.Assert(t, fs.Equal(tmpDir.Path(), expected))
	})
}

func TestWithReaderContent(t *testing.T) {
	content := "this is a test"
	dir := fs.NewDir(t, t.Name(),
		fs.WithFile("1", "",
			fs.WithReaderContent(strings.NewReader(content))),
	)
	defer dir.Remove()
	expected := fs.Expected(t, fs.WithFile("1", content))
	assert.Assert(t, fs.Equal(dir.Path(), expected))
}
