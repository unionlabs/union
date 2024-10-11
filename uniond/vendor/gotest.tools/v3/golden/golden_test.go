package golden

import (
	"os"
	"path/filepath"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/fs"
	"gotest.tools/v3/internal/source"
)

type fakeT struct {
	Failed bool
}

func (t *fakeT) Log(...interface{}) {
}

func (t *fakeT) FailNow() {
	t.Failed = true
}

func (t *fakeT) Fail() {
	t.Failed = true
}

func (t *fakeT) Helper() {}

func TestGoldenOpenInvalidFile(t *testing.T) {
	fakeT := new(fakeT)

	Open(fakeT, "/invalid/path")
	assert.Assert(t, fakeT.Failed)
}

func TestGoldenOpenAbsolutePath(t *testing.T) {
	file := fs.NewFile(t, "abs-test", fs.WithContent("content\n"))
	defer file.Remove()
	fakeT := new(fakeT)

	f := Open(fakeT, file.Path())
	assert.Assert(t, !fakeT.Failed)
	f.Close()
}

func TestGoldenOpen(t *testing.T) {
	filename, clean := setupGoldenFile(t, "")
	defer clean()

	fakeT := new(fakeT)

	f := Open(fakeT, filename)
	assert.Assert(t, !fakeT.Failed)
	f.Close()
}

func TestGoldenGetInvalidFile(t *testing.T) {
	fakeT := new(fakeT)

	Get(fakeT, "/invalid/path")
	assert.Assert(t, fakeT.Failed)
}

func TestGoldenGetAbsolutePath(t *testing.T) {
	file := fs.NewFile(t, "abs-test", fs.WithContent("content\n"))
	defer file.Remove()
	fakeT := new(fakeT)

	Get(fakeT, file.Path())
	assert.Assert(t, !fakeT.Failed)
}

func TestGoldenGet(t *testing.T) {
	expected := "content\nline1\nline2"

	filename, clean := setupGoldenFile(t, expected)
	defer clean()

	fakeT := new(fakeT)

	actual := Get(fakeT, filename)
	assert.Assert(t, !fakeT.Failed)
	assert.Assert(t, cmp.DeepEqual(actual, []byte(expected)))
}

func TestGoldenAssertInvalidContent(t *testing.T) {
	filename, clean := setupGoldenFile(t, "content")
	defer clean()

	fakeT := new(fakeT)

	Assert(fakeT, "foo", filename)
	assert.Assert(t, fakeT.Failed)
}

func TestGoldenAssertInvalidContentUpdate(t *testing.T) {
	setUpdateFlag(t)
	filename, clean := setupGoldenFile(t, "content")
	defer clean()

	fakeT := new(fakeT)

	Assert(fakeT, "foo", filename)
	assert.Assert(t, !fakeT.Failed)
}

func TestGoldenAssertAbsolutePath(t *testing.T) {
	file := fs.NewFile(t, "abs-test", fs.WithContent("foo"))
	defer file.Remove()
	fakeT := new(fakeT)

	Assert(fakeT, "foo", file.Path())
	assert.Assert(t, !fakeT.Failed)
}

func TestGoldenAssertInDir(t *testing.T) {
	filename, clean := setupGoldenFileWithDir(t, "testdatasubdir", "foo")
	defer clean()

	fakeT := new(fakeT)

	Assert(fakeT, "foo", filepath.Join("testdatasubdir", filename))
	assert.Assert(t, !fakeT.Failed)

	_, err := os.Stat("testdatasubdir")
	assert.Assert(t, os.IsNotExist(err), "testdatasubdir should not exist outside of testdata")
}

func TestGoldenAssertInDir_UpdateGolden(t *testing.T) {
	filename, clean := setupGoldenFileWithDir(t, "testdatasubdir", "foo")
	defer clean()
	setUpdateFlag(t)

	fakeT := new(fakeT)

	Assert(fakeT, "foo", filepath.Join("testdatasubdir", filename))
	assert.Assert(t, !fakeT.Failed)

	_, err := os.Stat("testdatasubdir")
	assert.Assert(t, os.IsNotExist(err), "testdatasubdir should not exist outside of testdata")
}

func TestGoldenAssert(t *testing.T) {
	filename, clean := setupGoldenFile(t, "foo")
	defer clean()

	fakeT := new(fakeT)

	Assert(fakeT, "foo", filename)
	assert.Assert(t, !fakeT.Failed)
}

func TestAssert_WithCarriageReturnInActual(t *testing.T) {
	filename, clean := setupGoldenFile(t, "a\rfoo\nbar\n")
	defer clean()

	fakeT := new(fakeT)

	Assert(fakeT, "a\rfoo\r\nbar\r\n", filename)
	assert.Assert(t, !fakeT.Failed)
}

func TestAssert_WithCarriageReturnInActual_UpdateGolden(t *testing.T) {
	filename, clean := setupGoldenFile(t, "")
	defer clean()
	unsetUpdateFlag := setUpdateFlag(t)

	fakeT := new(fakeT)
	Assert(fakeT, "a\rfoo\r\nbar\r\n", filename)
	assert.Assert(t, !fakeT.Failed)

	unsetUpdateFlag()
	actual := Get(fakeT, filename)
	assert.Equal(t, string(actual), "a\rfoo\nbar\n")

	Assert(t, "a\rfoo\r\nbar\r\n", filename, "matches with carriage returns")
	Assert(t, "a\rfoo\nbar\n", filename, "matches without carriage returns")
}

func TestGoldenAssertBytes(t *testing.T) {
	filename, clean := setupGoldenFile(t, "foo")
	defer clean()

	fakeT := new(fakeT)

	AssertBytes(fakeT, []byte("foo"), filename)
	assert.Assert(t, !fakeT.Failed)
}

func setUpdateFlag(t *testing.T) func() {
	orig := source.Update
	source.Update = true
	undo := func() {
		source.Update = orig
	}
	t.Cleanup(undo)
	return undo
}

func setupGoldenFileWithDir(t *testing.T, dirname, content string) (string, func()) {
	dirpath := filepath.Join("testdata", dirname)
	_ = os.MkdirAll(filepath.Join("testdata", dirname), 0755)
	f, err := os.CreateTemp(dirpath, t.Name()+"-")
	assert.NilError(t, err, "fail to create test golden file")
	defer f.Close()

	_, err = f.Write([]byte(content))
	assert.NilError(t, err)

	return filepath.Base(f.Name()), func() {
		assert.NilError(t, os.Remove(f.Name()))
		assert.NilError(t, os.Remove(dirpath))
	}
}

func setupGoldenFile(t *testing.T, content string) (string, func()) {
	_ = os.Mkdir("testdata", 0755)
	f, err := os.CreateTemp("testdata", t.Name()+"-")
	assert.NilError(t, err, "fail to create test golden file")
	defer f.Close()

	_, err = f.Write([]byte(content))
	assert.NilError(t, err)

	return filepath.Base(f.Name()), func() {
		assert.NilError(t, os.Remove(f.Name()))
	}
}

func TestStringFailure(t *testing.T) {
	filename, clean := setupGoldenFile(t, "this is\nthe text")
	defer clean()

	result := String("this is\nnot the text", filename)()
	assert.Assert(t, !result.Success())
	assert.Equal(t, result.(failure).FailureMessage(), `
--- expected
+++ actual
@@ -1,2 +1,2 @@
 this is
-the text
+not the text
`+failurePostamble(filename))
}

type failure interface {
	FailureMessage() string
}

func TestBytesFailure(t *testing.T) {
	filename, clean := setupGoldenFile(t, "5556")
	defer clean()

	result := Bytes([]byte("5555"), filename)()
	assert.Assert(t, !result.Success())
	assert.Equal(t, result.(failure).FailureMessage(),
		`[53 53 53 53] (actual) != [53 53 53 54] (expected)`+failurePostamble(filename))
}

func TestFlagUpdate(t *testing.T) {
	assert.Assert(t, !FlagUpdate())
	setUpdateFlag(t)
	assert.Assert(t, FlagUpdate())
}

func TestUpdate_CreatesPathsAndFile(t *testing.T) {
	setUpdateFlag(t)

	dir := fs.NewDir(t, t.Name())

	t.Run("creates the file", func(t *testing.T) {
		filename := dir.Join("filename")
		err := update(filename, nil)
		assert.NilError(t, err)

		_, err = os.Stat(filename)
		assert.NilError(t, err)
	})

	t.Run("creates directories", func(t *testing.T) {
		filename := dir.Join("one/two/filename")
		err := update(filename, nil)
		assert.NilError(t, err)

		_, err = os.Stat(filename)
		assert.NilError(t, err)

		t.Run("no error when directory exists", func(t *testing.T) {
			err = update(filename, nil)
			assert.NilError(t, err)
		})
	})
}
