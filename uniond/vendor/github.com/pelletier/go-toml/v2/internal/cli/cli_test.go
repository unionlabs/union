package cli

import (
	"bytes"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func processMain(args []string, input io.Reader, stdout, stderr io.Writer, f ConvertFn) int {
	p := Program{Fn: f}
	return p.main(args, input, stdout, stderr)
}

func TestProcessMainStdin(t *testing.T) {
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)
	input := strings.NewReader("this is the input")

	exit := processMain([]string{}, input, stdout, stderr, func(r io.Reader, w io.Writer) error {
		return nil
	})

	assert.Equal(t, 0, exit)
	assert.Empty(t, stdout.String())
	assert.Empty(t, stderr.String())
}

func TestProcessMainStdinErr(t *testing.T) {
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)
	input := strings.NewReader("this is the input")

	exit := processMain([]string{}, input, stdout, stderr, func(r io.Reader, w io.Writer) error {
		return fmt.Errorf("something bad")
	})

	assert.Equal(t, -1, exit)
	assert.Empty(t, stdout.String())
	assert.NotEmpty(t, stderr.String())
}

func TestProcessMainStdinDecodeErr(t *testing.T) {
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)
	input := strings.NewReader("this is the input")

	exit := processMain([]string{}, input, stdout, stderr, func(r io.Reader, w io.Writer) error {
		var v interface{}
		return toml.Unmarshal([]byte(`qwe = 001`), &v)
	})

	assert.Equal(t, -1, exit)
	assert.Empty(t, stdout.String())
	assert.Contains(t, stderr.String(), "error occurred at")
}

func TestProcessMainFileExists(t *testing.T) {
	tmpfile, err := ioutil.TempFile("", "example")
	require.NoError(t, err)
	defer os.Remove(tmpfile.Name())
	_, err = tmpfile.Write([]byte(`some data`))
	require.NoError(t, err)

	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)

	exit := processMain([]string{tmpfile.Name()}, nil, stdout, stderr, func(r io.Reader, w io.Writer) error {
		return nil
	})

	assert.Equal(t, 0, exit)
	assert.Empty(t, stdout.String())
	assert.Empty(t, stderr.String())
}

func TestProcessMainFileDoesNotExist(t *testing.T) {
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)

	exit := processMain([]string{"/lets/hope/this/does/not/exist"}, nil, stdout, stderr, func(r io.Reader, w io.Writer) error {
		return nil
	})

	assert.Equal(t, -1, exit)
	assert.Empty(t, stdout.String())
	assert.NotEmpty(t, stderr.String())
}

func TestProcessMainFilesInPlace(t *testing.T) {
	dir, err := ioutil.TempDir("", "")
	require.NoError(t, err)
	defer os.RemoveAll(dir)

	path1 := path.Join(dir, "file1")
	path2 := path.Join(dir, "file2")

	err = ioutil.WriteFile(path1, []byte("content 1"), 0600)
	require.NoError(t, err)
	err = ioutil.WriteFile(path2, []byte("content 2"), 0600)
	require.NoError(t, err)

	p := Program{
		Fn:      dummyFileFn,
		Inplace: true,
	}

	exit := p.main([]string{path1, path2}, os.Stdin, os.Stdout, os.Stderr)

	require.Equal(t, 0, exit)

	v1, err := ioutil.ReadFile(path1)
	require.NoError(t, err)
	require.Equal(t, "1", string(v1))

	v2, err := ioutil.ReadFile(path2)
	require.NoError(t, err)
	require.Equal(t, "2", string(v2))
}

func TestProcessMainFilesInPlaceErrRead(t *testing.T) {
	p := Program{
		Fn:      dummyFileFn,
		Inplace: true,
	}

	exit := p.main([]string{"/this/path/is/invalid"}, os.Stdin, os.Stdout, os.Stderr)

	require.Equal(t, -1, exit)
}

func TestProcessMainFilesInPlaceFailFn(t *testing.T) {
	dir, err := ioutil.TempDir("", "")
	require.NoError(t, err)
	defer os.RemoveAll(dir)

	path1 := path.Join(dir, "file1")

	err = ioutil.WriteFile(path1, []byte("content 1"), 0600)
	require.NoError(t, err)

	p := Program{
		Fn:      func(io.Reader, io.Writer) error { return fmt.Errorf("oh no") },
		Inplace: true,
	}

	exit := p.main([]string{path1}, os.Stdin, os.Stdout, os.Stderr)

	require.Equal(t, -1, exit)

	v1, err := ioutil.ReadFile(path1)
	require.NoError(t, err)
	require.Equal(t, "content 1", string(v1))
}

func dummyFileFn(r io.Reader, w io.Writer) error {
	b, err := ioutil.ReadAll(r)
	if err != nil {
		return err
	}
	v := strings.SplitN(string(b), " ", 2)[1]
	_, err = w.Write([]byte(v))
	return err
}
