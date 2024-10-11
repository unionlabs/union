package zipfs

import (
	"archive/zip"
	"path/filepath"
	"reflect"
	"testing"

	"github.com/spf13/afero"
)

func TestZipFS(t *testing.T) {
	zrc, err := zip.OpenReader("testdata/t.zip")
	if err != nil {
		t.Fatal(err)
	}
	zfs := New(&zrc.Reader)
	a := &afero.Afero{Fs: zfs}

	buf, err := a.ReadFile("testFile")
	if err != nil {
		t.Error(err)
	}
	if len(buf) != 8192 {
		t.Errorf("short read: %d != 8192", len(buf))
	}

	buf = make([]byte, 8)
	f, err := a.Open("testFile")
	if err != nil {
		t.Error(err)
	}
	if n, err := f.ReadAt(buf, 4092); err != nil {
		t.Error(err)
	} else if n != 8 {
		t.Errorf("expected to read 8 bytes, got %d", n)
	} else if string(buf) != "aaaabbbb" {
		t.Errorf("expected to get <aaaabbbb>, got <%s>", string(buf))
	}

	d, err := a.Open("/")
	if d == nil {
		t.Error(`Open("/") returns nil`)
	}
	if err != nil {
		t.Errorf(`Open("/"): err = %v`, err)
	}
	if s, _ := d.Stat(); !s.IsDir() {
		t.Error(`expected root ("/") to be a directory`)
	}
	if n := d.Name(); n != string(filepath.Separator) {
		t.Errorf("Wrong Name() of root directory: Expected: '%c', got '%s'", filepath.Separator, n)
	}

	buf = make([]byte, 8192)
	if n, err := f.Read(buf); err != nil {
		t.Error(err)
	} else if n != 8192 {
		t.Errorf("expected to read 8192 bytes, got %d", n)
	} else if buf[4095] != 'a' || buf[4096] != 'b' {
		t.Error("got wrong contents")
	}

	for _, s := range []struct {
		path string
		dir  bool
	}{
		{"/", true},
		{"testDir1", true},
		{"testDir1/testFile", false},
		{"testFile", false},
		{"sub", true},
		{"sub/testDir2", true},
		{"sub/testDir2/testFile", false},
	} {
		if dir, _ := a.IsDir(s.path); dir == s.dir {
			t.Logf("%s: directory check ok", s.path)
		} else {
			t.Errorf("%s: directory check NOT ok: %t, expected %t", s.path, dir, s.dir)
		}
	}

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
		entries, err := afero.Glob(zfs, s.glob)
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
