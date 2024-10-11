// Most of the tests are stolen from the zipfs implementation
package tarfs

import (
	"archive/tar"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"syscall"
	"testing"

	"github.com/spf13/afero"
)

var files = []struct {
	name        string
	exists      bool
	isdir       bool
	size        int64
	content     string
	contentAt4k string
}{
	{"/", true, true, 0, "", ""},
	{"/sub", true, true, 0, "", ""},
	{"/sub/testDir2", true, true, 0, "", ""},
	{"/sub/testDir2/testFile", true, false, 8192, "cccccccc", "ccccdddd"},
	{"/testFile", true, false, 8192, "aaaaaaaa", "aaaabbbb"},
	{"/testDir1/testFile", true, false, 8192, "bbbbbbbb", "bbbbcccc"},

	{"/nonExisting", false, false, 0, "", ""},
}

var dirs = []struct {
	name     string
	children []string
}{
	{"/", []string{"sub", "testDir1", "testFile"}},
	{"/sub", []string{"testDir2"}},
	{"/sub/testDir2", []string{"testFile"}},
	{"/testDir1", []string{"testFile"}},
}

var afs *afero.Afero

func TestMain(m *testing.M) {
	tf, err := os.Open("testdata/t.tar")
	if err != nil {
		fmt.Print(err)
		os.Exit(1)
	}

	tfs := New(tar.NewReader(tf))
	afs = &afero.Afero{Fs: tfs}

	// Check that an empty reader does not panic.
	_ = New(tar.NewReader(strings.NewReader("")))
	os.Exit(m.Run())
}

func TestFsOpen(t *testing.T) {
	for _, f := range files {
		file, err := afs.Open(f.name)
		if (err == nil) != f.exists {
			t.Errorf("%v exists = %v, but got err = %v", f.name, f.exists, err)
		}

		if !f.exists {
			continue
		}
		if err != nil {
			t.Fatalf("%v: %v", f.name, err)
		}

		if file.Name() != filepath.FromSlash(f.name) {
			t.Errorf("Name(), got %v, expected %v", file.Name(), filepath.FromSlash(f.name))
		}

		s, err := file.Stat()
		if err != nil {
			t.Fatalf("stat %v: got error '%v'", file.Name(), err)
		}

		if isdir := s.IsDir(); isdir != f.isdir {
			t.Errorf("%v directory, got: %v, expected: %v", file.Name(), isdir, f.isdir)
		}

		if size := s.Size(); size != f.size {
			t.Errorf("%v size, got: %v, expected: %v", file.Name(), size, f.size)
		}
	}
}

func TestRead(t *testing.T) {
	for _, f := range files {
		if !f.exists {
			continue
		}

		file, err := afs.Open(f.name)
		if err != nil {
			t.Fatalf("opening %v: %v", f.name, err)
		}

		buf := make([]byte, 8)
		n, err := file.Read(buf)
		if err != nil {
			if f.isdir && (err != syscall.EISDIR) {
				t.Errorf("%v got error %v, expected EISDIR", f.name, err)
			} else if !f.isdir {
				t.Errorf("%v: %v", f.name, err)
			}
		} else if n != 8 {
			t.Errorf("%v: got %d read bytes, expected 8", f.name, n)
		} else if string(buf) != f.content {
			t.Errorf("%v: got <%s>, expected <%s>", f.name, f.content, string(buf))
		}

	}
}

func TestReadAt(t *testing.T) {
	for _, f := range files {
		if !f.exists {
			continue
		}

		file, err := afs.Open(f.name)
		if err != nil {
			t.Fatalf("opening %v: %v", f.name, err)
		}

		buf := make([]byte, 8)
		n, err := file.ReadAt(buf, 4092)
		if err != nil {
			if f.isdir && (err != syscall.EISDIR) {
				t.Errorf("%v got error %v, expected EISDIR", f.name, err)
			} else if !f.isdir {
				t.Errorf("%v: %v", f.name, err)
			}
		} else if n != 8 {
			t.Errorf("%v: got %d read bytes, expected 8", f.name, n)
		} else if string(buf) != f.contentAt4k {
			t.Errorf("%v: got <%s>, expected <%s>", f.name, f.contentAt4k, string(buf))
		}

	}
}

func TestSeek(t *testing.T) {
	for _, f := range files {
		if !f.exists {
			continue
		}

		file, err := afs.Open(f.name)
		if err != nil {
			t.Fatalf("opening %v: %v", f.name, err)
		}

		tests := []struct {
			offin  int64
			whence int
			offout int64
		}{
			{0, io.SeekStart, 0},
			{10, io.SeekStart, 10},
			{1, io.SeekCurrent, 11},
			{10, io.SeekCurrent, 21},
			{0, io.SeekEnd, f.size},
			{-1, io.SeekEnd, f.size - 1},
		}

		for _, s := range tests {
			n, err := file.Seek(s.offin, s.whence)
			if err != nil {
				if f.isdir && err == syscall.EISDIR {
					continue
				}

				t.Errorf("%v: %v", f.name, err)
			}

			if n != s.offout {
				t.Errorf("%v: (off: %v, whence: %v): got %v, expected %v", f.name, s.offin, s.whence, n, s.offout)
			}
		}

	}
}

func TestName(t *testing.T) {
	for _, f := range files {
		if !f.exists {
			continue
		}

		file, err := afs.Open(f.name)
		if err != nil {
			t.Fatalf("opening %v: %v", f.name, err)
		}

		n := file.Name()
		if n != filepath.FromSlash(f.name) {
			t.Errorf("got: %v, expected: %v", n, filepath.FromSlash(f.name))
		}

	}
}

func TestClose(t *testing.T) {
	for _, f := range files {
		if !f.exists {
			continue
		}

		file, err := afs.Open(f.name)
		if err != nil {
			t.Fatalf("opening %v: %v", f.name, err)
		}

		err = file.Close()
		if err != nil {
			t.Errorf("%v: %v", f.name, err)
		}

		err = file.Close()
		if err == nil {
			t.Errorf("%v: closing twice should return an error", f.name)
		}

		buf := make([]byte, 8)
		n, err := file.Read(buf)
		if n != 0 || err == nil {
			t.Errorf("%v: could read from a closed file", f.name)
		}

		n, err = file.ReadAt(buf, 256)
		if n != 0 || err == nil {
			t.Errorf("%v: could readAt from a closed file", f.name)
		}

		off, err := file.Seek(0, io.SeekStart)
		if off != 0 || err == nil {
			t.Errorf("%v: could seek from a closed file", f.name)
		}
	}
}

func TestOpenFile(t *testing.T) {
	for _, f := range files {
		file, err := afs.OpenFile(f.name, os.O_RDONLY, 0o400)
		if !f.exists {
			if !errors.Is(err, syscall.ENOENT) {
				t.Errorf("%v: got %v, expected%v", f.name, err, syscall.ENOENT)
			}

			continue
		}

		if err != nil {
			t.Fatalf("%v: %v", f.name, err)
		}
		file.Close()

		_, err = afs.OpenFile(f.name, os.O_CREATE, 0o600)
		if !errors.Is(err, syscall.EPERM) {
			t.Errorf("%v: open for write: got %v, expected %v", f.name, err, syscall.EPERM)
		}

	}
}

func TestFsStat(t *testing.T) {
	for _, f := range files {
		fi, err := afs.Stat(f.name)
		if !f.exists {
			if !errors.Is(err, syscall.ENOENT) {
				t.Errorf("%v: got %v, expected%v", f.name, err, syscall.ENOENT)
			}

			continue
		}

		if err != nil {
			t.Fatalf("stat %v: got error '%v'", f.name, err)
		}

		if isdir := fi.IsDir(); isdir != f.isdir {
			t.Errorf("%v directory, got: %v, expected: %v", f.name, isdir, f.isdir)
		}

		if size := fi.Size(); size != f.size {
			t.Errorf("%v size, got: %v, expected: %v", f.name, size, f.size)
		}
	}
}

func TestReaddir(t *testing.T) {
	for _, d := range dirs {
		dir, err := afs.Open(d.name)
		if err != nil {
			t.Fatal(err)
		}

		fi, err := dir.Readdir(0)
		if err != nil {
			t.Fatal(err)
		}
		var names []string
		for _, f := range fi {
			names = append(names, f.Name())
		}

		if !reflect.DeepEqual(names, d.children) {
			t.Errorf("%v: children, got '%v', expected '%v'", d.name, names, d.children)
		}

		fi, err = dir.Readdir(1)
		if err != nil {
			t.Fatal(err)
		}

		names = []string{}
		for _, f := range fi {
			names = append(names, f.Name())
		}

		if !reflect.DeepEqual(names, d.children[0:1]) {
			t.Errorf("%v: children, got '%v', expected '%v'", d.name, names, d.children[0:1])
		}
	}

	dir, err := afs.Open("/testFile")
	if err != nil {
		t.Fatal(err)
	}

	_, err = dir.Readdir(-1)
	if err != syscall.ENOTDIR {
		t.Fatal("Expected error")
	}
}

func TestReaddirnames(t *testing.T) {
	for _, d := range dirs {
		dir, err := afs.Open(d.name)
		if err != nil {
			t.Fatal(err)
		}

		names, err := dir.Readdirnames(0)
		if err != nil {
			t.Fatal(err)
		}

		if !reflect.DeepEqual(names, d.children) {
			t.Errorf("%v: children, got '%v', expected '%v'", d.name, names, d.children)
		}

		names, err = dir.Readdirnames(1)
		if err != nil {
			t.Fatal(err)
		}

		if !reflect.DeepEqual(names, d.children[0:1]) {
			t.Errorf("%v: children, got '%v', expected '%v'", d.name, names, d.children[0:1])
		}
	}

	dir, err := afs.Open("/testFile")
	if err != nil {
		t.Fatal(err)
	}

	_, err = dir.Readdir(-1)
	if err != syscall.ENOTDIR {
		t.Fatal("Expected error")
	}
}

func TestGlob(t *testing.T) {
	for _, s := range []struct {
		glob    string
		entries []string
	}{
		{filepath.FromSlash("/*"), []string{filepath.FromSlash("/sub"), filepath.FromSlash("/testDir1"), filepath.FromSlash("/testFile")}},
		{filepath.FromSlash("*"), []string{filepath.FromSlash("sub"), filepath.FromSlash("testDir1"), filepath.FromSlash("testFile")}},
		{filepath.FromSlash("sub/*"), []string{filepath.FromSlash("sub/testDir2")}},
		{filepath.FromSlash("sub/testDir2/*"), []string{filepath.FromSlash("sub/testDir2/testFile")}},
		{filepath.FromSlash("testDir1/*"), []string{filepath.FromSlash("testDir1/testFile")}},
	} {
		entries, err := afero.Glob(afs.Fs, s.glob)
		if err != nil {
			t.Error(err)
		}
		if reflect.DeepEqual(entries, s.entries) {
			t.Logf("glob: %s: glob ok", s.glob)
		} else {
			t.Errorf("glob: %s: got %#v, expected %#v", s.glob, entries, s.entries)
		}
	}
}
