// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package txtar_test

import (
	"io/fs"
	"strings"
	"testing"
	"testing/fstest"

	"golang.org/x/tools/txtar"
)

func TestFS(t *testing.T) {
	var fstestcases = []struct {
		name, input, files string
	}{
		{
			name:  "empty",
			input: ``,
			files: "",
		},
		{
			name: "one",
			input: `
-- one.txt --
one
`,
			files: "one.txt",
		},
		{
			name: "two",
			input: `
-- one.txt --
one
-- two.txt --
two
`,
			files: "one.txt two.txt",
		},
		{
			name: "subdirectories",
			input: `
-- one.txt --
one
-- 2/two.txt --
two
-- 2/3/three.txt --
three
-- 4/four.txt --
four
`,
			files: "one.txt 2/two.txt 2/3/three.txt 4/four.txt",
		},
	}

	for _, tc := range fstestcases {
		t.Run(tc.name, func(t *testing.T) {
			a := txtar.Parse([]byte(tc.input))
			fsys, err := txtar.FS(a)
			if err != nil {
				t.Fatal(err)
			}

			files := strings.Fields(tc.files)
			if err := fstest.TestFS(fsys, files...); err != nil {
				t.Fatal(err)
			}

			for _, f := range a.Files {
				b, err := fs.ReadFile(fsys, f.Name)
				if err != nil {
					t.Errorf("ReadFile(%q) failed with error: %v", f.Name, err)
				}
				if got, want := string(b), string(f.Data); got != want {
					t.Errorf("ReadFile(%q) = %q; want %q", f.Name, got, want)
				}
			}
		})
	}
}

func TestInvalid(t *testing.T) {
	invalidtestcases := []struct {
		name, want string
		input      string
	}{
		{"unclean file names", "invalid path", `
-- 1/../one.txt --
one
-- 2/sub/../two.txt --
two
`},
		{"duplicate name", `cannot create fs.FS from txtar.Archive: duplicate path "1/2/one.txt"`, `
-- 1/2/one.txt --
one
-- 1/2/one.txt --
two
`},
		{"file conflicts with directory", `duplicate path "1/2"`, `
-- 1/2 --
one
-- 1/2/one.txt --
two
`},
	}

	for _, tc := range invalidtestcases {
		t.Run(tc.name, func(t *testing.T) {
			a := txtar.Parse([]byte(tc.input))
			_, err := txtar.FS(a)
			if err == nil {
				t.Fatal("txtar.FS(...) succeeded; expected an error")
			}
			if got := err.Error(); !strings.Contains(got, tc.want) || tc.want == "" {
				t.Errorf("txtar.FS(...) got error %q; want %q", got, tc.want)
			}
		})
	}
}

func TestModified(t *testing.T) {
	const input = `
-- one.txt --
one
`
	for _, mod := range []func(a *txtar.Archive){
		func(a *txtar.Archive) { a.Files[0].Data = []byte("other") },
		func(a *txtar.Archive) { a.Files[0].Name = "other" },
		func(a *txtar.Archive) { a.Files = nil },
	} {
		a := txtar.Parse([]byte(input))
		if n := len(a.Files); n != 1 {
			t.Fatalf("txtar.Parse(%q) got %d files; expected 1", input, n)
		}

		fsys, err := txtar.FS(a)
		if err != nil {
			t.Fatal(err)
		}

		// Confirm we can open "one.txt".
		_, err = fsys.Open("one.txt")
		if err != nil {
			t.Fatal(err)
		}
		// Modify a to get ErrModified when opening "one.txt".
		mod(a)

		_, err = fsys.Open("one.txt")
		if err != txtar.ErrModified {
			t.Errorf("Open(%q) got error %s; want ErrModified", "one.txt", err)
		}
	}
}

func TestReadFile(t *testing.T) {
	const input = `
-- 1/one.txt --
one
`
	a := txtar.Parse([]byte(input))
	fsys, err := txtar.FS(a)
	if err != nil {
		t.Fatal(err)
	}
	readfs := fsys.(fs.ReadFileFS)
	_, err = readfs.ReadFile("1")
	if err == nil {
		t.Errorf("ReadFile(%q) succeeded; expected an error when reading a directory", "1")
	}

	content, err := readfs.ReadFile("1/one.txt")
	if err != nil {
		t.Fatal(err)
	}
	want := "one\n"
	if got := string(content); want != got {
		t.Errorf("ReadFile(%q) = %q; want %q", "1/one.txt", got, want)
	}
}
