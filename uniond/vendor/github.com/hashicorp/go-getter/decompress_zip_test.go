package getter

import (
	"archive/zip"
	"bytes"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

func TestZipDecompressor(t *testing.T) {
	cases := []TestDecompressCase{
		{
			"empty.zip",
			false,
			true,
			nil,
			"",
			nil,
		},

		{
			"single.zip",
			false,
			false,
			nil,
			"d3b07384d113edec49eaa6238ad5ff00",
			nil,
		},

		{
			"single.zip",
			true,
			false,
			[]string{"file"},
			"",
			nil,
		},

		{
			"multiple.zip",
			true,
			false,
			[]string{"file1", "file2"},
			"",
			nil,
		},

		{
			"multiple.zip",
			false,
			true,
			nil,
			"",
			nil,
		},

		{
			"subdir.zip",
			true,
			false,
			[]string{"file1", "subdir/", "subdir/child"},
			"",
			nil,
		},

		{
			"subdir_empty.zip",
			true,
			false,
			[]string{"file1", "subdir/"},
			"",
			nil,
		},

		{
			"subdir_missing_dir.zip",
			true,
			false,
			[]string{"file1", "subdir/", "subdir/child"},
			"",
			nil,
		},

		// Tests that a zip can't contain references with "..".
		{
			"outside_parent.zip",
			true,
			true,
			nil,
			"",
			nil,
		},
	}

	for i, tc := range cases {
		cases[i].Input = filepath.Join("./testdata", "decompress-zip", tc.Input)
	}

	TestDecompressor(t, new(ZipDecompressor), cases)
}

func TestDecompressZipPermissions(t *testing.T) {
	d := new(ZipDecompressor)
	input := "./test-fixtures/decompress-zip/permissions.zip"

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

func TestDecompressZipBomb(t *testing.T) {
	buf := new(bytes.Buffer)

	// Create a zip file inline, written to the buffer.
	{
		w := zip.NewWriter(buf)

		var files = []struct {
			Name, Body string
		}{
			{"readme.txt", "This archive contains some text files."},
			{"gopher.txt", "Gopher names:\nGeorge\nGeoffrey\nGonzo"},
			{"todo.txt", "Get animal handling licence.\nWrite more examples."},
		}
		for _, file := range files {
			f, err := w.Create(file.Name)
			if err != nil {
				t.Fatal(err)
			}
			_, err = f.Write([]byte(file.Body))
			if err != nil {
				t.Fatal(err)
			}
		}

		err := w.Close()
		if err != nil {
			log.Fatal(err)
		}
	}

	td, err := ioutil.TempDir("", "go-getter-zip")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	zipFilePath := filepath.Join(td, "input.zip")

	err = os.WriteFile(zipFilePath, buf.Bytes(), 0666)
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	t.Run("error with limit", func(t *testing.T) {
		d := new(ZipDecompressor)
		d.FileSizeLimit = 7 // bytes

		err = d.Decompress(t.TempDir(), zipFilePath, true, 0644)
		if err == nil {
			t.FailNow()
		}
		if !strings.Contains(err.Error(), "zip archive larger than limit: 7") {
			t.Fatalf("unexpected error: %q", err.Error())
		}
	})

	t.Run("no error without limit", func(t *testing.T) {
		d := new(ZipDecompressor)

		err = d.Decompress(t.TempDir(), zipFilePath, true, 0644)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
	})
}
