// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"fmt"
	"io"
	"os"
	"sort"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/stretchr/testify/require"
)

func runTestCases(t *testing.T, testCases []string, fs *MemFS) {
	var f File
	for _, tc := range testCases {
		s := strings.Split(tc, " ")[1:]

		saveF := s[0] == "f" && s[1] == "="
		if saveF {
			s = s[2:]
		}

		fails := s[len(s)-1] == "fails"
		if fails {
			s = s[:len(s)-1]
		}

		var (
			fi  os.FileInfo
			g   File
			err error
		)
		switch s[0] {
		case "create":
			g, err = fs.Create(s[1])
		case "link":
			err = fs.Link(s[1], s[2])
		case "open":
			g, err = fs.Open(s[1])
		case "openDir":
			g, err = fs.OpenDir(s[1])
		case "mkdirall":
			err = fs.MkdirAll(s[1], 0755)
		case "remove":
			err = fs.Remove(s[1])
		case "rename":
			err = fs.Rename(s[1], s[2])
		case "reuseForWrite":
			g, err = fs.ReuseForWrite(s[1], s[2])
		case "resetToSynced":
			fs.ResetToSyncedState()
		case "ignoreSyncs":
			fs.SetIgnoreSyncs(true)
		case "stopIgnoringSyncs":
			fs.SetIgnoreSyncs(false)
		case "f.write":
			_, err = f.Write([]byte(s[1]))
		case "f.sync":
			err = f.Sync()
		case "f.read":
			n, _ := strconv.Atoi(s[1])
			buf := make([]byte, n)
			_, err = io.ReadFull(f, buf)
			if err != nil {
				break
			}
			if got, want := string(buf), s[3]; got != want {
				t.Fatalf("%q: got %q, want %q", tc, got, want)
			}
		case "f.readat":
			n, _ := strconv.Atoi(s[1])
			off, _ := strconv.Atoi(s[2])
			buf := make([]byte, n)
			_, err = f.ReadAt(buf, int64(off))
			if err != nil {
				break
			}
			if got, want := string(buf), s[4]; got != want {
				t.Fatalf("%q: got %q, want %q", tc, got, want)
			}
		case "f.close":
			f, err = nil, f.Close()
		case "f.stat.name":
			fi, err = f.Stat()
			if err != nil {
				break
			}
			if got, want := fi.Name(), s[2]; got != want {
				t.Fatalf("%q: got %q, want %q", tc, got, want)
			}
		default:
			t.Fatalf("bad test case: %q", tc)
		}

		if saveF {
			f, g = g, nil
		} else if g != nil {
			g.Close()
		}

		if fails {
			if err == nil {
				t.Fatalf("%q: got nil error, want non-nil", tc)
			}
		} else {
			if err != nil {
				t.Fatalf("%q: %v", tc, err)
			}
		}
	}

	// Both "" and "/" are allowed to be used to refer to the root of the FS
	// for the purposes of cloning.
	checkClonedIsEquivalent(t, fs, "")
	checkClonedIsEquivalent(t, fs, "/")
}

// Test that the FS can be cloned and that the clone serializes identically.
func checkClonedIsEquivalent(t *testing.T, fs *MemFS, path string) {
	t.Helper()
	clone := NewMem()
	cloned, err := Clone(fs, clone, path, path)
	require.NoError(t, err)
	require.True(t, cloned)
	require.Equal(t, fs.String(), clone.String())
}

func TestBasics(t *testing.T) {
	fs := NewMem()
	testCases := []string{
		// Create a top-level file.
		"1a: create /foo",
		// Create a child of that file. It should fail, since /foo is not a directory.
		"2a: create /foo/x fails",
		// Create a third-level file. It should fail, since /bar has not been created.
		// Similarly, opening that file should fail.
		"3a: create /bar/baz/y fails",
		"3b: open /bar/baz/y fails",
		// Make the /bar/baz directory; create a third-level file. Creation should now succeed.
		"4a: mkdirall /bar/baz",
		"4b: f = create /bar/baz/y",
		"4c: f.stat.name == y",
		// Write some data; read it back.
		"5a: f.write abcde",
		"5b: f.close",
		"5c: f = open /bar/baz/y",
		"5d: f.read 5 == abcde",
		"5e: f.readat 2 1 == bc",
		"5f: f.close",
		// Link /bar/baz/y to /bar/baz/z. We should be able to read from both files
		// and remove them independently.
		"6a: link /bar/baz/y /bar/baz/z",
		"6b: f = open /bar/baz/z",
		"6c: f.read 5 == abcde",
		"6d: f.close",
		"6e: remove /bar/baz/z",
		"6f: f = open /bar/baz/y",
		"6g: f.read 5 == abcde",
		"6h: f.close",
		// Remove the file twice. The first should succeed, the second should fail.
		"7a: remove /bar/baz/y",
		"7b: remove /bar/baz/y fails",
		"7c: open /bar/baz/y fails",
		// Rename /foo to /goo. Trying to open /foo should succeed before the rename and
		// fail afterwards, and vice versa for /goo.
		"8a: open /foo",
		"8b: open /goo fails",
		"8c: rename /foo /goo",
		"8d: open /foo fails",
		"8e: open /goo",
		// Create /bar/baz/z and rename /bar/baz to /bar/caz.
		"9a: create /bar/baz/z",
		"9b: open /bar/baz/z",
		"9c: open /bar/caz/z fails",
		"9d: rename /bar/baz /bar/caz",
		"9e: open /bar/baz/z fails",
		"9f: open /bar/caz/z",
		// ReuseForWrite
		"10a: reuseForWrite /bar/caz/z /bar/z",
		"10b: open /bar/caz/z fails",
		"10c: open /bar/z",
		// Opening the root directory works.
		"11a: f = open /",
		"11b: f.stat.name == /",
	}
	runTestCases(t, testCases, fs)
}

func TestList(t *testing.T) {
	fs := NewMem()

	dirnames := []string{
		"/bar",
		"/foo/2",
	}
	for _, dirname := range dirnames {
		err := fs.MkdirAll(dirname, 0755)
		if err != nil {
			t.Fatalf("MkdirAll %q: %v", dirname, err)
		}
	}

	filenames := []string{
		"/a",
		"/bar/baz",
		"/foo/0",
		"/foo/1",
		"/foo/2/a",
		"/foo/2/b",
		"/foo/3",
		"/foot",
	}
	for _, filename := range filenames {
		f, err := fs.Create(filename)
		if err != nil {
			t.Fatalf("Create %q: %v", filename, err)
		}
		if err := f.Close(); err != nil {
			t.Fatalf("Close %q: %v", filename, err)
		}
	}

	{
		got := fs.String()
		const want = `          /
       0    a
            bar/
       0      baz
            foo/
       0      0
       0      1
              2/
       0        a
       0        b
       0      3
       0    foot
`
		if got != want {
			t.Fatalf("String:\n----got----\n%s----want----\n%s", got, want)
		}
	}

	testCases := []string{
		"/:a bar foo foot",
		"/bar:baz",
		"/bar/:baz",
		"/baz:",
		"/baz/:",
		"/foo:0 1 2 3",
		"/foo/:0 1 2 3",
		"/foo/1:",
		"/foo/1/:",
		"/foo/2:a b",
		"/foo/2/:a b",
		"/foot:",
		"/foot/:",
	}
	for _, tc := range testCases {
		s := strings.Split(tc, ":")
		list, _ := fs.List(s[0])
		sort.Strings(list)
		got := strings.Join(list, " ")
		want := s[1]
		if got != want {
			t.Errorf("List %q: got %q, want %q", s[0], got, want)
		}
	}
}

func TestMemFile(t *testing.T) {
	want := "foo"
	f := NewMemFile([]byte(want))
	buf, err := io.ReadAll(f)
	if err != nil {
		t.Fatalf("%v", err)
	}
	if got := string(buf); got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestStrictFS(t *testing.T) {
	fs := NewStrictMem()
	testCases := []string{
		// Created file disappears if directory is not synced.
		"1a: create /foo",
		"1b: open /foo",
		"1c: resetToSynced",
		"1d: open /foo fails",

		// Create directory and a file in it and write and read from it.
		"2a: mkdirall /bar",
		"2b: f = create /bar/y",
		"2c: f.stat.name == y",
		// Write some data; read it back.
		"2d: f.write abcde",
		"2e: f.close",
		"2f: f = open /bar/y",
		"2g: f.read 5 == abcde",
		"2h: f.close",
		"2i: open /bar",

		// Resetting causes both the directory and file to disappear.
		"3a: resetToSynced",
		"3b: openDir /bar fails",
		"3c: open /bar/y fails",

		// Create the directory and file again. Link the file to another file in the same dir,
		// and to a file in the root dir. Sync the root dir. After reset, the created dir and the
		// file in the root dir are the only ones visible.
		"4a: mkdirall /bar",
		"4b: create /bar/y",
		"4c: f = openDir /",
		"4d: f.sync",
		"4e: f.close",
		"4f: link /bar/y /bar/z",
		"4g: link /bar/y /z",
		"4h: f = openDir /",
		"4i: f.sync",
		"4j: f.close",
		"4k: resetToSynced",
		"4l: openDir /bar",
		"4m: open /bar/y fails",
		"4n: open /bar/z fails",
		"4o: open /z",

		// Create the file in the directory again and this time sync /bar directory. The file is
		// preserved after reset.
		"5a: create /bar/y",
		"5b: f = openDir /bar",
		"5c: f.sync",
		"5d: f.close",
		"5e: resetToSynced",
		"5f: openDir /bar",
		"5g: open /bar/y",

		// Unsynced data in the file is lost on reset.
		"5a: f = create /bar/y",
		"5b: f.write a",
		"5c: f.sync",
		"5d: f.write b",
		"5e: f.close",
		"5f: f = openDir /bar",
		"5g: f.sync",
		"5h: f.close",
		"5i: resetToSynced",
		"5j: f = open /bar/y",
		"5k: f.read 1 = a",
		"5l: f.read 1 fails",
		"5m: f.close",

		// reuseForWrite works correctly in strict mode in that unsynced data does not overwrite
		// previous contents when a reset happens.
		"6a: f = create /z",
		"6b: f.write abcdefgh",
		"6c: f.sync",
		"6d: f.close",
		"6e: f = reuseForWrite /z /y",
		"6f: f.write x",
		"6g: f.sync",
		"6h: f.write y", // will be lost
		"6i: f.close",
		"6j: f = openDir /",
		"6k: f.sync",
		"6l: f.close",
		"6m: resetToSynced",
		"6n: f = open /y",
		"6o: f.read 8 = xbcdefgh",
		"6p: f.close",

		// Ignore syncs.
		"7a: f = create /z",
		"7b: f.write a",
		"7c: f.sync",
		"7d: ignoreSyncs",
		"7e: f.write b",
		"7f: f.sync",
		"7g: f.close",
		"7h: stopIgnoringSyncs",
		"7e: f = openDir /",
		"7f: f.sync",
		"7g: f.close",
		"7h: resetToSynced",
		"7i: f = open /z",
		"7j: f.read 1 = a",
		"7k: f.read 1 fails",
		"7l: f.close",
	}
	runTestCases(t, testCases, fs)
}

func TestMemFSLock(t *testing.T) {
	filesystems := map[string]FS{}
	fileLocks := map[string]io.Closer{}

	datadriven.RunTest(t, "testdata/memfs_lock", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "mkfs":
			for _, arg := range td.CmdArgs {
				filesystems[arg.String()] = NewMem()
			}
			return "OK"

		// lock fs=<filesystem-name> handle=<handle> path=<path>
		case "lock":
			var filesystemName string
			var path string
			var handle string
			td.ScanArgs(t, "fs", &filesystemName)
			td.ScanArgs(t, "path", &path)
			td.ScanArgs(t, "handle", &handle)
			fs := filesystems[filesystemName]
			if fs == nil {
				return fmt.Sprintf("filesystem %q doesn't exist", filesystemName)
			}
			l, err := fs.Lock(path)
			if err != nil {
				return err.Error()
			}
			fileLocks[handle] = l
			return "OK"

		// mkdirall fs=<filesystem-name> path=<path>
		case "mkdirall":
			var filesystemName string
			var path string
			td.ScanArgs(t, "fs", &filesystemName)
			td.ScanArgs(t, "path", &path)
			fs := filesystems[filesystemName]
			if fs == nil {
				return fmt.Sprintf("filesystem %q doesn't exist", filesystemName)
			}
			err := fs.MkdirAll(path, 0755)
			if err != nil {
				return err.Error()
			}
			return "OK"

		// close handle=<handle>
		case "close":
			var handle string
			td.ScanArgs(t, "handle", &handle)
			err := fileLocks[handle].Close()
			delete(fileLocks, handle)
			if err != nil {
				return err.Error()
			}
			return "OK"
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}
