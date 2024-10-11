package getter

import (
	"archive/tar"
	"bytes"
	"io/ioutil"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"
)

func TestTar(t *testing.T) {
	mtime := time.Unix(0, 0)
	cases := []TestDecompressCase{
		{
			"extended_header.tar",
			true,
			false,
			[]string{"directory/", "directory/a", "directory/b"},
			"",
			nil,
		},
		{
			"implied_dir.tar",
			true,
			false,
			[]string{"directory/", "directory/sub/", "directory/sub/a", "directory/sub/b"},
			"",
			nil,
		},
		{
			"unix_time_0.tar",
			true,
			false,
			[]string{"directory/", "directory/sub/", "directory/sub/a", "directory/sub/b"},
			"",
			&mtime,
		},
	}

	for i, tc := range cases {
		cases[i].Input = filepath.Join("./testdata", "decompress-tar", tc.Input)
	}

	TestDecompressor(t, new(TarDecompressor), cases)
}

func TestTarLimits(t *testing.T) {
	b := bytes.NewBuffer(nil)

	tw := tar.NewWriter(b)

	var files = []struct {
		Name, Body string
	}{
		{"readme.txt", "This archive contains some text files."},
		{"gopher.txt", "Gopher names:\nCharlie\nRonald\nGlenn"},
		{"todo.txt", "Get animal handling license."},
	}

	for _, file := range files {
		hdr := &tar.Header{
			Name: file.Name,
			Mode: 0600,
			Size: int64(len(file.Body)),
		}
		if err := tw.WriteHeader(hdr); err != nil {
			t.Fatal(err)
		}
		if _, err := tw.Write([]byte(file.Body)); err != nil {
			t.Fatal(err)
		}
	}

	if err := tw.Close(); err != nil {
		t.Fatal(err)
	}

	td, err := ioutil.TempDir("", "getter")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	tarFilePath := filepath.Join(td, "input.tar")

	err = os.WriteFile(tarFilePath, b.Bytes(), 0666)
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	t.Run("file size limit", func(t *testing.T) {
		d := new(TarDecompressor)

		d.FileSizeLimit = 35

		dst := filepath.Join(td, "subdir", "file-size-limit-result")

		err = d.Decompress(dst, tarFilePath, true, 0022)

		if err == nil {
			t.Fatal("expected file size limit to error")
		}

		if !strings.Contains(err.Error(), "tar archive larger than limit: 35") {
			t.Fatalf("unexpected error: %q", err.Error())
		}
	})

	t.Run("files limit", func(t *testing.T) {
		d := new(TarDecompressor)

		d.FilesLimit = 2

		dst := filepath.Join(td, "subdir", "files-limit-result")

		err = d.Decompress(dst, tarFilePath, true, 0022)

		if err == nil {
			t.Fatal("expected files limit to error")
		}

		if !strings.Contains(err.Error(), "tar archive contains too many files: 3 > 2") {
			t.Fatalf("unexpected error: %q", err.Error())
		}
	})
}

// testDecompressPermissions decompresses a directory and checks the permissions of the expanded files
func testDecompressorPermissions(t *testing.T, d Decompressor, input string, expected map[string]int, umask os.FileMode) {
	td, err := ioutil.TempDir("", "getter")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Destination is always joining result so that we have a new path
	dst := filepath.Join(td, "subdir", "result")

	err = d.Decompress(dst, input, true, umask)
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	defer os.RemoveAll(dst)

	for name, mode := range expected {
		fi, err := os.Stat(filepath.Join(dst, name))
		if err != nil {
			t.Fatalf("err: %s", err)
		}

		real := fi.Mode()
		if real != os.FileMode(mode) {
			t.Fatalf("err: %s expected mode %o got %o", name, mode, real)
		}
	}
}

func TestDecompressTarPermissions(t *testing.T) {
	d := new(TarDecompressor)
	input := "./test-fixtures/decompress-tar/permissions.tar"

	var expected map[string]int
	var masked int

	if runtime.GOOS == "windows" {
		expected = map[string]int{
			"directory/public":  0666,
			"directory/private": 0666,
			"directory/exec":    0666,
			"directory/setuid":  0666,
		}
		masked = 0666
	} else {
		expected = map[string]int{
			"directory/public":  0666,
			"directory/private": 0600,
			"directory/exec":    0755,
			"directory/setuid":  040000755,
		}
		masked = 0755
	}

	testDecompressorPermissions(t, d, input, expected, os.FileMode(0))

	expected["directory/setuid"] = masked
	testDecompressorPermissions(t, d, input, expected, os.FileMode(060000000))
}
