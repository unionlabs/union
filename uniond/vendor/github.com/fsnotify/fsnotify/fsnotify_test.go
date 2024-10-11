package fsnotify

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"testing"
	"time"

	"github.com/fsnotify/fsnotify/internal"
)

// Set soft open file limit to the maximum; on e.g. OpenBSD it's 512/1024.
//
// Go 1.19 will always do this when the os package is imported.
//
// https://go-review.googlesource.com/c/go/+/393354/
func init() {
	internal.SetRlimit()
}

func TestWatch(t *testing.T) {
	tests := []testCase{
		{"multiple creates", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			addWatch(t, w, tmp)

			cat(t, "data", file)
			rm(t, file)

			touch(t, file)       // Recreate the file
			cat(t, "data", file) // Modify
			cat(t, "data", file) // Modify
		}, `
			create  /file
			write   /file
			remove  /file
			create  /file
			write   /file
			write   /file
		`},

		{"dir only", func(t *testing.T, w *Watcher, tmp string) {
			beforeWatch := join(tmp, "beforewatch")
			file := join(tmp, "file")

			touch(t, beforeWatch)
			addWatch(t, w, tmp)

			cat(t, "data", file)
			rm(t, file)
			rm(t, beforeWatch)
		}, `
			create /file
			write  /file
			remove /file
			remove /beforewatch
		`},

		{"subdir", func(t *testing.T, w *Watcher, tmp string) {
			addWatch(t, w, tmp)

			file := join(tmp, "file")
			dir := join(tmp, "sub")
			dirfile := join(tmp, "sub/file2")

			mkdir(t, dir)     // Create sub-directory
			touch(t, file)    // Create a file
			touch(t, dirfile) // Create a file (Should not see this! we are not watching subdir)
			time.Sleep(200 * time.Millisecond)
			rmAll(t, dir) // Make sure receive deletes for both file and sub-directory
			rm(t, file)
		}, `
			create /sub
			create /file
			remove /sub
			remove /file

			# TODO: not sure why the REMOVE /sub is dropped.
			dragonfly:
				create    /sub
				create    /file
				remove    /file
			fen:
				create /sub
				create /file
				write  /sub
				remove /sub
				remove /file
			# Windows includes a write for the /sub dir too, two of them even(?)
			windows:
				create /sub
				create /file
				write  /sub
				write  /sub
				remove /sub
				remove /file
		`},

		{"file in directory is not readable", func(t *testing.T, w *Watcher, tmp string) {
			if runtime.GOOS == "windows" {
				t.Skip("attributes don't work on Windows") // TODO: figure out how to make a file unreadable
			}

			touch(t, tmp, "file-unreadable")
			chmod(t, 0, tmp, "file-unreadable")
			touch(t, tmp, "file")
			addWatch(t, w, tmp)

			cat(t, "hello", tmp, "file")
			rm(t, tmp, "file")
			rm(t, tmp, "file-unreadable")
		}, `
			WRITE     "/file"
			REMOVE    "/file"
			REMOVE    "/file-unreadable"

			# We never set up a watcher on the unreadable file, so we don't get
			# the REMOVE.
			kqueue:
				WRITE    "/file"
				REMOVE   "/file"

			windows:
				empty
		`},

		{"watch same dir twice", func(t *testing.T, w *Watcher, tmp string) {
			addWatch(t, w, tmp)
			addWatch(t, w, tmp)

			touch(t, tmp, "file")
			cat(t, "hello", tmp, "file")
			rm(t, tmp, "file")
			mkdir(t, tmp, "dir")
		}, `
			create   /file
			write    /file
			remove   /file
			create   /dir
		`},

		{"watch same file twice", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			touch(t, file)

			addWatch(t, w, file)
			addWatch(t, w, file)

			cat(t, "hello", tmp, "file")
		}, `
			write    /file
		`},
	}

	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

func TestWatchCreate(t *testing.T) {
	tests := []testCase{
		// Files
		{"create empty file", func(t *testing.T, w *Watcher, tmp string) {
			addWatch(t, w, tmp)
			touch(t, tmp, "file")
		}, `
			create  /file
		`},
		{"create file with data", func(t *testing.T, w *Watcher, tmp string) {
			addWatch(t, w, tmp)
			cat(t, "data", tmp, "file")
		}, `
			create  /file
			write   /file
		`},

		// Directories
		{"create new directory", func(t *testing.T, w *Watcher, tmp string) {
			addWatch(t, w, tmp)
			mkdir(t, tmp, "dir")
		}, `
			create  /dir
		`},

		// Links
		{"create new symlink to file", func(t *testing.T, w *Watcher, tmp string) {
			if !internal.HasPrivilegesForSymlink() {
				t.Skip("does not have privileges for symlink on this OS")
			}
			touch(t, tmp, "file")
			addWatch(t, w, tmp)
			symlink(t, join(tmp, "file"), tmp, "link")
		}, `
			create  /link
		`},
		{"create new symlink to directory", func(t *testing.T, w *Watcher, tmp string) {
			if !internal.HasPrivilegesForSymlink() {
				t.Skip("does not have privileges for symlink on this OS")
			}
			addWatch(t, w, tmp)
			symlink(t, tmp, tmp, "link")
		}, `
			create  /link
		`},

		// FIFO
		{"create new named pipe", func(t *testing.T, w *Watcher, tmp string) {
			if runtime.GOOS == "windows" {
				t.Skip("No named pipes on Windows")
			}
			touch(t, tmp, "file")
			addWatch(t, w, tmp)
			mkfifo(t, tmp, "fifo")
		}, `
			create  /fifo
		`},
		// Device node
		{"create new device node pipe", func(t *testing.T, w *Watcher, tmp string) {
			if runtime.GOOS == "windows" {
				t.Skip("No device nodes on Windows")
			}
			if isKqueue() {
				// Don't want to use os/user to check uid, since that pulls in
				// cgo by default and stuff that uses fsnotify won't be
				// statically linked by default.
				t.Skip("needs root on BSD")
			}
			if isSolaris() {
				t.Skip(`"mknod fails with "not owner"`)
			}
			touch(t, tmp, "file")
			addWatch(t, w, tmp)

			mknod(t, 0, tmp, "dev")
		}, `
			create  /dev
		`},
	}
	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

func TestWatchWrite(t *testing.T) {
	tests := []testCase{
		// Files
		{"truncate file", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			cat(t, "data", file)
			addWatch(t, w, tmp)

			fp, err := os.Create(file)
			if err != nil {
				t.Fatal(err)
			}
			if err := fp.Sync(); err != nil {
				t.Fatal(err)
			}
			eventSeparator()
			if _, err := fp.Write([]byte("X")); err != nil {
				t.Fatal(err)
			}
			if err := fp.Close(); err != nil {
				t.Fatal(err)
			}
		}, `
			write  /file  # truncate
			write  /file  # write

			# Truncate is chmod on kqueue, except NetBSD
			netbsd:
				write  /file
			kqueue:
				chmod     /file
				write     /file
		`},

		{"multiple writes to a file", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			cat(t, "data", file)
			addWatch(t, w, tmp)

			fp, err := os.OpenFile(file, os.O_RDWR, 0)
			if err != nil {
				t.Fatal(err)
			}
			if _, err := fp.Write([]byte("X")); err != nil {
				t.Fatal(err)
			}
			if err := fp.Sync(); err != nil {
				t.Fatal(err)
			}
			eventSeparator()
			if _, err := fp.Write([]byte("Y")); err != nil {
				t.Fatal(err)
			}
			if err := fp.Close(); err != nil {
				t.Fatal(err)
			}
		}, `
			write  /file  # write X
			write  /file  # write Y
		`},
	}
	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

func TestWatchRename(t *testing.T) {
	tests := []testCase{
		{"rename file in watched dir", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			cat(t, "asd", file)

			addWatch(t, w, tmp)
			mv(t, file, tmp, "renamed")
		}, `
			rename /file
			create /renamed
		`},

		{"rename from unwatched dir", func(t *testing.T, w *Watcher, tmp string) {
			unwatched := t.TempDir()

			addWatch(t, w, tmp)
			touch(t, unwatched, "file")
			mv(t, join(unwatched, "file"), tmp, "file")
		}, `
			create /file
		`},

		{"rename to unwatched dir", func(t *testing.T, w *Watcher, tmp string) {
			if runtime.GOOS == "netbsd" && isCI() {
				t.Skip("fails in CI; see #488") // TODO
			}

			unwatched := t.TempDir()
			file := join(tmp, "file")
			renamed := join(unwatched, "renamed")

			addWatch(t, w, tmp)

			cat(t, "data", file)
			mv(t, file, renamed)
			cat(t, "data", renamed) // Modify the file outside of the watched dir
			touch(t, file)          // Recreate the file that was moved
		}, `
			create /file # cat data >file
			write  /file # ^
			rename /file # mv file ../renamed
			create /file # touch file

			# Windows has REMOVE /file, rather than CREATE /file
			windows:
				create   /file
				write    /file
				remove   /file
				create   /file
		`},

		{"rename overwriting existing file", func(t *testing.T, w *Watcher, tmp string) {
			unwatched := t.TempDir()
			file := join(unwatched, "file")

			touch(t, tmp, "renamed")
			touch(t, file)

			addWatch(t, w, tmp)
			mv(t, file, tmp, "renamed")
		}, `
			# TODO: this should really be RENAME.
			remove /renamed
			create /renamed

			# No remove event for inotify; inotify just sends MOVE_SELF.
			linux:
				create /renamed

			# TODO: this is broken.
			dragonfly:
				REMOVE               "/"
		`},

		{"rename watched directory", func(t *testing.T, w *Watcher, tmp string) {
			dir := join(tmp, "dir")
			mkdir(t, dir)
			addWatch(t, w, dir)

			mv(t, dir, tmp, "dir-renamed")
			touch(t, tmp, "dir-renamed/file")
		}, `
			rename   /dir

			# TODO(v2): Windows should behave the same by default. See #518
			windows:
				create   /dir/file
		`},

		{"rename watched file", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			rename := join(tmp, "rename-one")
			touch(t, file)

			addWatch(t, w, file)

			mv(t, file, rename)
			mv(t, rename, tmp, "rename-two")
		}, `
			rename     /file

			# TODO(v2): Windows should behave the same by default. See #518
			windows:
				rename   /file
				rename   /rename-one
		`},

		{"re-add renamed file", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			rename := join(tmp, "rename")
			touch(t, file)

			addWatch(t, w, file)

			mv(t, file, rename)
			touch(t, file)
			addWatch(t, w, file)
			cat(t, "hello", rename)
			cat(t, "hello", file)
		}, `
			rename /file    # mv file rename
			                # Watcher gets removed on rename, so no write for /rename
			write  /file    # cat hello >file

			# TODO(v2): Windows should behave the same by default. See #518
			windows:
				rename    /file
				write     /rename
				write     /file
		`},
	}

	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

func TestWatchSymlink(t *testing.T) {
	if !internal.HasPrivilegesForSymlink() {
		t.Skip("does not have privileges for symlink on this OS")
	}

	tests := []testCase{
		{"watch a symlink to a file", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			link := join(tmp, "link")
			touch(t, file)
			symlink(t, file, link)
			addWatch(t, w, link)

			cat(t, "hello", file)
		}, `
			write    /link

			# TODO: Symlinks followed on kqueue; it shouldn't do this, but I'm
			# afraid changing it will break stuff. See #227, #390
			kqueue:
				write    /file

			# TODO: see if we can fix this.
			windows:
				empty
		`},

		{"watch a symlink to a dir", func(t *testing.T, w *Watcher, tmp string) {
			dir := join(tmp, "dir")
			link := join(tmp, "link")
			mkdir(t, dir)
			symlink(t, dir, link)
			addWatch(t, w, link)

			touch(t, dir, "file")
		}, `
			create    /link/file

			# TODO: Symlinks followed on kqueue; it shouldn't do this, but I'm
			# afraid changing it will break stuff. See #227, #390
			kqueue:
				create /dir/file
		`},

		{"create unresolvable symlink", func(t *testing.T, w *Watcher, tmp string) {
			addWatch(t, w, tmp)

			symlink(t, join(tmp, "target"), tmp, "link")
		}, `
			create /link

			# No events at all on Dragonfly
			# TODO: should fix this.
			dragonfly:
				empty
		`},

		{"cyclic symlink", func(t *testing.T, w *Watcher, tmp string) {
			symlink(t, ".", tmp, "link")
			addWatch(t, w, tmp)
			rm(t, tmp, "link")
			cat(t, "foo", tmp, "link")

		}, `
			write  /link
			create /link

			linux, windows, fen:
				remove    /link
				create    /link
				write     /link
		`},

		// Bug #277
		{"277", func(t *testing.T, w *Watcher, tmp string) {
			if runtime.GOOS == "netbsd" && isCI() {
				t.Skip("fails in CI") // TODO
			}

			// TODO: there is some strange fuckery going on if I use go test
			// -count=2; the second test run has unix.Kqueue() in newKqueue()
			// return 0, which is a very odd fd number, but the first event does
			// work (create /foo). After that we get EBADF (Bad file
			// descriptor).
			//
			// This is *only* for this test, and *only* if we have the symlinks
			// below. kqueue(2) doesn't document returning fd 0.
			//
			// This happens on all the BSDs, but *not* macOS.
			touch(t, tmp, "file1")
			touch(t, tmp, "file2")
			symlink(t, join(tmp, "file1"), tmp, "link1")
			symlink(t, join(tmp, "file2"), tmp, "link2")

			addWatch(t, w, tmp)
			touch(t, tmp, "foo")
			rm(t, tmp, "foo")
			mkdir(t, tmp, "apple")
			mv(t, join(tmp, "apple"), tmp, "pear")
			rmAll(t, tmp, "pear")
		}, `
			create   /foo     # touch foo
			remove   /foo     # rm foo
			create   /apple   # mkdir apple
			rename   /apple   # mv apple pear
			create   /pear
			remove   /pear    # rm -r pear

			# TODO: the CREATE/REMOVE for /pear don't show; I'm not entirely
			# sure why; sendDirectoryChangeEvents() doesn't pick it up. It does
			# seem consistent though, both locally and in CI.
			freebsd, netbsd, dragonfly:
				create   /foo     # touch foo
				remove   /foo     # rm foo
				create   /apple   # mkdir apple
				rename   /apple   # mv apple pear
		`},
	}
	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

func TestWatchAttrib(t *testing.T) {
	tests := []testCase{
		{"chmod", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")

			cat(t, "data", file)
			addWatch(t, w, file)
			chmod(t, 0o700, file)
		}, `
			CHMOD   "/file"

			windows:
				empty
		`},

		{"write does not trigger CHMOD", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")

			cat(t, "data", file)
			addWatch(t, w, file)
			chmod(t, 0o700, file)
			cat(t, "more data", file)
		}, `
			CHMOD   "/file"
			WRITE   "/file"

			windows:
				write /file
		`},

		{"chmod after write", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")

			cat(t, "data", file)
			addWatch(t, w, file)
			chmod(t, 0o700, file)
			cat(t, "more data", file)
			chmod(t, 0o600, file)
		}, `
			CHMOD   "/file"
			WRITE   "/file"
			CHMOD   "/file"

			windows:
				write /file
		`},
	}

	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

func TestWatchRemove(t *testing.T) {
	tests := []testCase{
		{"remove watched file", func(t *testing.T, w *Watcher, tmp string) {
			file := join(tmp, "file")
			touch(t, file)

			addWatch(t, w, file)
			rm(t, file)
		}, `
			REMOVE   "/file"

			# unlink always emits a CHMOD on Linux.
			linux:
				CHMOD    "/file"
				REMOVE   "/file"
		`},

		{"remove watched file with open fd", func(t *testing.T, w *Watcher, tmp string) {
			if runtime.GOOS == "windows" {
				t.Skip("Windows hard-locks open files so this will never work")
			}

			file := join(tmp, "file")
			touch(t, file)

			// Intentionally don't close the descriptor here so it stays around.
			_, err := os.Open(file)
			if err != nil {
				t.Fatal(err)
			}

			addWatch(t, w, file)
			rm(t, file)
		}, `
			REMOVE   "/file"

			# inotify will just emit a CHMOD for the unlink, but won't actually
			# emit a REMOVE until the descriptor is closed. Bit odd, but not much
			# we can do about it. The REMOVE is tested in TestInotifyDeleteOpenFile()
			linux:
				CHMOD    "/file"
		`},

		{"remove recursive", func(t *testing.T, w *Watcher, tmp string) {
			recurseOnly(t)

			mkdirAll(t, tmp, "dir1", "subdir")
			mkdirAll(t, tmp, "dir2", "subdir")
			touch(t, tmp, "dir1", "subdir", "file")
			touch(t, tmp, "dir2", "subdir", "file")

			addWatch(t, w, tmp, "dir1", "...")
			addWatch(t, w, tmp, "dir2", "...")
			cat(t, "asd", tmp, "dir1", "subdir", "file")
			cat(t, "asd", tmp, "dir2", "subdir", "file")

			if err := w.Remove(join(tmp, "dir1")); err != nil {
				t.Fatal(err)
			}
			if err := w.Remove(join(tmp, "dir2", "...")); err != nil {
				t.Fatal(err)
			}

			if w := w.WatchList(); len(w) != 0 {
				t.Errorf("WatchList not empty: %s", w)
			}

			cat(t, "asd", tmp, "dir1", "subdir", "file")
			cat(t, "asd", tmp, "dir2", "subdir", "file")
		}, `
			write /dir1/subdir
			write /dir1/subdir/file
			write /dir2/subdir
			write /dir2/subdir/file
		`},
	}

	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}

	t.Run("remove watched directory", func(t *testing.T) {
		t.Parallel()
		tmp := t.TempDir()

		w := newCollector(t)
		w.collect(t)

		touch(t, tmp, "a")
		touch(t, tmp, "b")
		touch(t, tmp, "c")
		touch(t, tmp, "d")
		touch(t, tmp, "e")
		touch(t, tmp, "f")
		touch(t, tmp, "g")
		mkdir(t, tmp, "h")
		mkdir(t, tmp, "h", "a")
		mkdir(t, tmp, "i")
		mkdir(t, tmp, "i", "a")
		mkdir(t, tmp, "j")
		mkdir(t, tmp, "j", "a")
		addWatch(t, w.w, tmp)
		rmAll(t, tmp)

		if runtime.GOOS != "windows" {
			cmpEvents(t, tmp, w.stop(t), newEvents(t, `
				remove    /
				remove    /a
				remove    /b
				remove    /c
				remove    /d
				remove    /e
				remove    /f
				remove    /g
				remove    /h
				remove    /i
				remove    /j`))
			return
		}

		// ReadDirectoryChangesW gives undefined results: not all files are
		// always present. So test only that 1) we got the directory itself, and
		// 2) we don't get events for unspected files.
		var (
			events = w.stop(t)
			found  bool
		)
		for _, e := range events {
			if e.Name == tmp && e.Has(Remove) {
				found = true
				continue
			}
			if filepath.Dir(e.Name) != tmp {
				t.Errorf("unexpected event: %s", e)
			}
		}
		if !found {
			t.Fatalf("didn't see directory in:\n%s", events)
		}
	})
}

func TestWatchRecursive(t *testing.T) {
	recurseOnly(t)

	tests := []testCase{
		// Make a nested directory tree, then write some files there.
		{"basic", func(t *testing.T, w *Watcher, tmp string) {
			mkdirAll(t, tmp, "/one/two/three/four")
			addWatch(t, w, tmp, "/...")

			cat(t, "asd", tmp, "/file.txt")
			cat(t, "asd", tmp, "/one/two/three/file.txt")
		}, `
			create    /file.txt                  # cat asd >file.txt
			write     /file.txt

			write     /one/two/three             # cat asd >one/two/three/file.txt
			create    /one/two/three/file.txt
			write     /one/two/three/file.txt
		`},

		// Create a new directory tree and then some files under that.
		{"add directory", func(t *testing.T, w *Watcher, tmp string) {
			mkdirAll(t, tmp, "/one/two/three/four")
			addWatch(t, w, tmp, "/...")

			mkdirAll(t, tmp, "/one/two/new/dir")
			touch(t, tmp, "/one/two/new/file")
			touch(t, tmp, "/one/two/new/dir/file")
		}, `
			write     /one/two                # mkdir -p one/two/new/dir
			create    /one/two/new
			create    /one/two/new/dir

			write     /one/two/new            # touch one/two/new/file
			create    /one/two/new/file

			create    /one/two/new/dir/file   # touch one/two/new/dir/file
		`},

		// Remove nested directory
		{"remove directory", func(t *testing.T, w *Watcher, tmp string) {
			mkdirAll(t, tmp, "one/two/three/four")
			addWatch(t, w, tmp, "...")

			cat(t, "asd", tmp, "one/two/three/file.txt")
			rmAll(t, tmp, "one/two")
		}, `
			write                /one/two/three            # cat asd >one/two/three/file.txt
			create               /one/two/three/file.txt
			write                /one/two/three/file.txt

			write                /one/two                  # rm -r one/two
			write                /one/two/three
			remove               /one/two/three/file.txt
			remove               /one/two/three/four
			write                /one/two/three
			remove               /one/two/three
			write                /one/two
			remove               /one/two
		`},

		// Rename nested directory
		{"rename directory", func(t *testing.T, w *Watcher, tmp string) {
			mkdirAll(t, tmp, "/one/two/three/four")
			addWatch(t, w, tmp, "...")

			mv(t, join(tmp, "one"), tmp, "one-rename")
			touch(t, tmp, "one-rename/file")
			touch(t, tmp, "one-rename/two/three/file")
		}, `
			rename               "/one"                        # mv one one-rename
			create               "/one-rename"

			write                "/one-rename"                 # touch one-rename/file
			create               "/one-rename/file"

			write                "/one-rename/two/three"       # touch one-rename/two/three/file
			create               "/one-rename/two/three/file"
		`},

		{"remove watched directory", func(t *testing.T, w *Watcher, tmp string) {
			mk := func(r string) {
				touch(t, r, "a")
				touch(t, r, "b")
				touch(t, r, "c")
				touch(t, r, "d")
				touch(t, r, "e")
				touch(t, r, "f")
				touch(t, r, "g")

				mkdir(t, r, "h")
				mkdir(t, r, "h", "a")
				mkdir(t, r, "i")
				mkdir(t, r, "i", "a")
				mkdir(t, r, "j")
				mkdir(t, r, "j", "a")
			}
			mk(tmp)
			mkdir(t, tmp, "sub")
			mk(join(tmp, "sub"))

			addWatch(t, w, tmp, "...")
			rmAll(t, tmp)
		}, `
			remove               "/a"
			remove               "/b"
			remove               "/c"
			remove               "/d"
			remove               "/e"
			remove               "/f"
			remove               "/g"
			write                "/h"
			remove               "/h/a"
			write                "/h"
			remove               "/h"
			write                "/i"
			remove               "/i/a"
			write                "/i"
			remove               "/i"
			write                "/j"
			remove               "/j/a"
			write                "/j"
			remove               "/j"
			write                "/sub"
			remove               "/sub/a"
			remove               "/sub/b"
			remove               "/sub/c"
			remove               "/sub/d"
			remove               "/sub/e"
			remove               "/sub/f"
			remove               "/sub/g"
			write                "/sub/h"
			remove               "/sub/h/a"
			write                "/sub/h"
			remove               "/sub/h"
			write                "/sub/i"
			remove               "/sub/i/a"
			write                "/sub/i"
			remove               "/sub/i"
			write                "/sub/j"
			remove               "/sub/j/a"
			write                "/sub/j"
			remove               "/sub/j"
			write                "/sub"
			remove               "/sub"
			remove               "/"
		`},
	}

	for _, tt := range tests {
		tt := tt
		tt.run(t)
	}
}

// TODO: this fails reguarly in the CI; not sure if it's a bug with the test or
// code; need to look in to it.
func TestClose(t *testing.T) {
	chanClosed := func(t *testing.T, w *Watcher) {
		t.Helper()

		// Need a small sleep as Close() on kqueue does all sorts of things,
		// which may take a little bit.
		switch runtime.GOOS {
		case "freebsd", "openbsd", "netbsd", "dragonfly", "darwin", "solaris", "illumos":
			time.Sleep(50 * time.Millisecond)
		}

		tim := time.NewTimer(50 * time.Millisecond)
	loop:
		for {
			select {
			default:
				t.Fatal("blocking on Events")
			case <-tim.C:
				t.Fatalf("Events not closed")
			case _, ok := <-w.Events:
				if !ok {
					break loop
				}
			}
		}

		select {
		default:
			t.Fatal("blocking on Errors")
		case err, ok := <-w.Errors:
			if ok {
				t.Fatalf("Errors not closed; read:\n\t%s", err)
			}
		}
	}

	t.Run("close", func(t *testing.T) {
		t.Parallel()

		w := newWatcher(t)
		if err := w.Close(); err != nil {
			t.Fatal(err)
		}
		chanClosed(t, w)

		var done int32
		go func() {
			w.Close()
			atomic.StoreInt32(&done, 1)
		}()

		eventSeparator()
		if atomic.LoadInt32(&done) == 0 {
			t.Fatal("double Close() test failed: second Close() call didn't return")
		}

		if err := w.Add(t.TempDir()); err == nil {
			t.Fatal("expected error on Watch() after Close(), got nil")
		}
	})

	// Make sure that Close() works even when the Events channel isn't being
	// read.
	t.Run("events not read", func(t *testing.T) {
		t.Parallel()

		tmp := t.TempDir()
		w := newWatcher(t, tmp)

		touch(t, tmp, "file")
		rm(t, tmp, "file")
		eventSeparator()
		if err := w.Close(); err != nil {
			t.Fatal(err)
		}

		// TODO: windows backend doesn't work well here; can't easily fix it.
		//       Need to rewrite things a bit.
		if runtime.GOOS != "windows" {
			chanClosed(t, w)
		}
	})

	// Make sure that calling Close() while REMOVE events are emitted doesn't race.
	t.Run("close while removing files", func(t *testing.T) {
		t.Parallel()
		tmp := t.TempDir()

		files := make([]string, 0, 200)
		for i := 0; i < 200; i++ {
			f := join(tmp, fmt.Sprintf("file-%03d", i))
			touch(t, f, noWait)
			files = append(files, f)
		}

		w := newWatcher(t, tmp)

		startC, stopC, errC := make(chan struct{}), make(chan struct{}), make(chan error)
		go func() {
			for {
				select {
				case <-w.Errors:
				case <-w.Events:
				case <-stopC:
					return
				}
			}
		}()
		rmDone := make(chan struct{})
		go func() {
			<-startC
			for _, f := range files {
				rm(t, f, noWait)
			}
			rmDone <- struct{}{}
		}()
		go func() {
			<-startC
			errC <- w.Close()
		}()
		close(startC)
		defer close(stopC)
		if err := <-errC; err != nil {
			t.Fatal(err)
		}

		<-rmDone
	})

	// Make sure Close() doesn't race when called more than once; hard to write
	// a good reproducible test for this, but running it 150 times seems to
	// reproduce it in ~75% of cases and isn't too slow (~0.06s on my system).
	t.Run("double close", func(t *testing.T) {
		t.Run("default", func(t *testing.T) {
			t.Parallel()

			for i := 0; i < 150; i++ {
				w, err := NewWatcher()
				if err != nil {
					if strings.Contains(err.Error(), "too many") { // syscall.EMFILE
						time.Sleep(100 * time.Millisecond)
						continue
					}
					t.Fatal(err)
				}
				go w.Close()
				go w.Close()
				go w.Close()
			}
		})
		t.Run("buffered=4096", func(t *testing.T) {
			t.Parallel()

			for i := 0; i < 150; i++ {
				w, err := NewBufferedWatcher(4096)
				if err != nil {
					if strings.Contains(err.Error(), "too many") { // syscall.EMFILE
						time.Sleep(100 * time.Millisecond)
						continue
					}
					t.Fatal(err)
				}
				go w.Close()
				go w.Close()
				go w.Close()
			}
		})
	})

	t.Run("closes channels after read", func(t *testing.T) {
		if runtime.GOOS == "netbsd" {
			t.Skip("flaky") // TODO
		}

		t.Parallel()

		tmp := t.TempDir()

		w := newCollector(t, tmp)
		w.collect(t)
		touch(t, tmp, "qwe")
		touch(t, tmp, "asd")

		if err := w.w.Close(); err != nil {
			t.Fatal(err)
		}

		chanClosed(t, w.w)
	})

	t.Run("error after closed", func(t *testing.T) {
		t.Parallel()

		tmp := t.TempDir()
		w := newWatcher(t, tmp)
		if err := w.Close(); err != nil {
			t.Fatal(err)
		}

		file := join(tmp, "file")
		touch(t, file)
		if err := w.Add(file); !errors.Is(err, ErrClosed) {
			t.Fatalf("wrong error for Add: %#v", err)
		}
		if err := w.Remove(file); err != nil {
			t.Fatalf("wrong error for Remove: %#v", err)
		}
		if l := w.WatchList(); l != nil { // Should return an error, but meh :-/
			t.Fatalf("WatchList not nil: %#v", l)
		}
	})
}

func TestAdd(t *testing.T) {
	t.Run("doesn't exist", func(t *testing.T) {
		t.Parallel()
		tmp := t.TempDir()

		w := newWatcher(t)
		err := w.Add(join(tmp, "non-existent"))
		if err == nil {
			t.Fatal("err is nil")
		}

		// Errors for this are inconsistent; should be fixed in v2. See #144
		switch runtime.GOOS {
		case "linux":
			if _, ok := err.(syscall.Errno); !ok {
				t.Errorf("wrong error type: %[1]T: %#[1]v", err)
			}
		case "windows":
			if _, ok := err.(*os.SyscallError); !ok {
				t.Errorf("wrong error type: %[1]T: %#[1]v", err)
			}
		default:
			if _, ok := err.(*fs.PathError); !ok {
				t.Errorf("wrong error type: %[1]T: %#[1]v", err)
			}
		}
	})

	t.Run("permission denied", func(t *testing.T) {
		if runtime.GOOS == "windows" {
			t.Skip("chmod doesn't work on Windows") // TODO: see if we can make a file unreadable
		}

		t.Parallel()

		tmp := t.TempDir()
		dir := join(tmp, "dir-unreadable")
		mkdir(t, dir)
		touch(t, dir, "/file")
		chmod(t, 0, dir)

		w := newWatcher(t)
		defer func() {
			w.Close()
			chmod(t, 0o755, dir) // Make TempDir() cleanup work
		}()
		err := w.Add(dir)
		if err == nil {
			t.Fatal("error is nil")
		}
		if !errors.Is(err, internal.UnixEACCES) {
			t.Errorf("not unix.EACCESS: %T %#[1]v", err)
		}
		if !errors.Is(err, internal.SyscallEACCES) {
			t.Errorf("not syscall.EACCESS: %T %#[1]v", err)
		}
	})

	t.Run("add same path twice", func(t *testing.T) {
		tmp := t.TempDir()
		w := newCollector(t)
		if err := w.w.Add(tmp); err != nil {
			t.Fatal(err)
		}
		if err := w.w.Add(tmp); err != nil {
			t.Fatal(err)
		}

		w.collect(t)
		touch(t, tmp, "file")
		rm(t, tmp, "file")

		cmpEvents(t, tmp, w.events(t), newEvents(t, `
			create /file
			remove /file
		`))
	})
}

// TODO: should also check internal state is correct/cleaned up; e.g. no
// left-over file descriptors or whatnot.
func TestRemove(t *testing.T) {
	t.Run("works", func(t *testing.T) {
		t.Parallel()

		tmp := t.TempDir()
		touch(t, tmp, "file")

		w := newCollector(t)
		w.collect(t)
		addWatch(t, w.w, tmp)
		if err := w.w.Remove(tmp); err != nil {
			t.Fatal(err)
		}

		time.Sleep(200 * time.Millisecond)
		cat(t, "data", tmp, "file")
		chmod(t, 0o700, tmp, "file")

		have := w.stop(t)
		if len(have) > 0 {
			t.Errorf("received events; expected none:\n%s", have)
		}
	})

	t.Run("remove same dir twice", func(t *testing.T) {
		t.Parallel()

		tmp := t.TempDir()

		touch(t, tmp, "file")

		w := newWatcher(t)
		defer w.Close()

		addWatch(t, w, tmp)

		if err := w.Remove(tmp); err != nil {
			t.Fatal(err)
		}
		err := w.Remove(tmp)
		if err == nil {
			t.Fatal("no error")
		}
		if !errors.Is(err, ErrNonExistentWatch) {
			t.Fatalf("wrong error: %T", err)
		}
	})

	// Make sure that concurrent calls to Remove() don't race.
	t.Run("no race", func(t *testing.T) {
		t.Parallel()

		tmp := t.TempDir()
		touch(t, tmp, "file")

		for i := 0; i < 10; i++ {
			w := newWatcher(t)
			defer w.Close()
			addWatch(t, w, tmp)

			done := make(chan struct{})
			go func() {
				defer func() { done <- struct{}{} }()
				w.Remove(tmp)
			}()
			go func() {
				defer func() { done <- struct{}{} }()
				w.Remove(tmp)
			}()
			<-done
			<-done
			w.Close()
		}
	})

	// Make sure file handles are correctly released.
	//
	// regression test for #42 see https://gist.github.com/timshannon/603f92824c5294269797
	t.Run("", func(t *testing.T) {
		w := newWatcher(t)
		defer w.Close()

		// consume the events
		var werr error
		var wg sync.WaitGroup
		wg.Add(1)
		go func() {
			defer wg.Done()
			for {
				select {
				case werr = <-w.Errors:
					return
				case <-w.Events:
				}
			}
		}()

		tmp := t.TempDir()
		dir := join(tmp, "child")
		addWatch(t, w, tmp)
		mkdir(t, dir)
		addWatch(t, w, dir) // start watching child
		rmWatch(t, w, dir)  // stop watching child
		rmAll(t, dir)       // delete child dir

		// Child dir should no longer exist
		_, err := os.Stat(dir)
		if err == nil {
			t.Fatalf("dir %q should no longer exist!", dir)
		}
		if _, ok := err.(*os.PathError); err != nil && !ok {
			t.Errorf("Expected a PathError, got %v", err)
		}

		w.Close()
		wg.Wait()

		if werr != nil {
			t.Fatal(werr)
		}
	})

	t.Run("remove with ... when non-recursive", func(t *testing.T) {
		recurseOnly(t)
		t.Parallel()

		tmp := t.TempDir()
		w := newWatcher(t)
		addWatch(t, w, tmp)

		if err := w.Remove(join(tmp, "...")); err == nil {
			t.Fatal("err was nil")
		}
		if err := w.Remove(tmp); err != nil {
			t.Fatal(err)
		}
	})
}

func TestEventString(t *testing.T) {
	tests := []struct {
		in   Event
		want string
	}{
		{Event{}, `[no events]   ""`},
		{Event{"/file", 0}, `[no events]   "/file"`},

		{Event{"/file", Chmod | Create},
			`CREATE|CHMOD  "/file"`},
		{Event{"/file", Rename},
			`RENAME        "/file"`},
		{Event{"/file", Remove},
			`REMOVE        "/file"`},
		{Event{"/file", Write | Chmod},
			`WRITE|CHMOD   "/file"`},
	}

	for _, tt := range tests {
		t.Run("", func(t *testing.T) {
			have := tt.in.String()
			if have != tt.want {
				t.Errorf("\nhave: %q\nwant: %q", have, tt.want)
			}
		})
	}
}

// Verify the watcher can keep up with file creations/deletions when under load.
func TestWatchStress(t *testing.T) {
	if isCI() {
		t.Skip("fails too often on the CI") // TODO
	}

	// On NetBSD ioutil.ReadDir in sendDirectoryChangeEvents() returns EINVAL
	// ~80% of the time:
	//
	//    readdirent /tmp/TestWatchStress3584363325/001: invalid argument
	//
	// This ends up calling getdents(), the manpage says:
	//
	// [EINVAL]  A directory was being read on NFS, but it was modified on the
	//           server while it was being read.
	//
	// Which is, eh, odd? Maybe I read the code wrong and it's calling another
	// function too(?)
	//
	// Because this happens on the Errors channel we can't "skip" it like with
	// other kqueue platorms, so just skip the entire test for now.
	//
	// TODO: fix this.
	if runtime.GOOS == "netbsd" {
		t.Skip("broken on NetBSD")
	}

	Errorf := func(t *testing.T, msg string, args ...interface{}) {
		if !isKqueue() {
			t.Errorf(msg, args...)
			return
		}

		// On kqueue platforms it doesn't seem to sync properly; see comment for
		// the sleep below.
		//
		// TODO: fix this.
		t.Logf(msg, args...)
		t.Skip("flaky on kqueue; allowed to fail")
	}

	tmp := t.TempDir()
	w := newCollector(t, tmp)
	w.collect(t)

	fmtNum := func(n int) string {
		s := fmt.Sprintf("%09d", n)
		return s[:3] + "_" + s[3:6] + "_" + s[6:]
	}

	var (
		numFiles = 1_500_000
		runFor   = 30 * time.Second
	)
	if testing.Short() {
		runFor = time.Second
	}

	// Otherwise platforms with low limits such as as OpenBSD and NetBSD will
	// fail, since every watched file uses a file descriptor. Need superuser
	// permissions and twiddling with /etc/login.conf to adjust them, so we
	// can't "just increase it".
	if isKqueue() && uint64(numFiles) > internal.Maxfiles() {
		numFiles = int(internal.Maxfiles()) - 100
		t.Logf("limiting files to %d due to max open files limit", numFiles)
	}

	var (
		prefix = "xyz-prefix-"
		done   = make(chan struct{})
	)
	// testing.Short()
	go func() {
		numFiles = createFiles(t, tmp, prefix, numFiles, runFor)

		// TODO: this shouldn't be needed; and if this is too short some very
		//       odd events happen:
		//
		//         fsnotify_test.go:837: saw 42 unexpected events:
		//             REMOVE               ""
		//             CREATE               "."
		//             REMOVE               ""
		//             CREATE               "."
		//             REMOVE               ""
		//             ...
		//
		//         fsnotify_test.go:848: expected the following 3175 events, but didn't see them (showing first 100 only)
		//             REMOVE               "/xyz-prefix-000_015_080"
		//             REMOVE               "/xyz-prefix-000_014_536"
		//             CREATE               "/xyz-prefix-000_015_416"
		//             CREATE               "/xyz-prefix-000_015_406"
		//             ...
		//
		// Should really add a Sync() method which processes all outstanding
		// events.
		if isKqueue() {
			time.Sleep(1000 * time.Millisecond)
			if !testing.Short() {
				time.Sleep(1000 * time.Millisecond)
			}
		}

		close(done)
	}()
	<-done

	have := w.stopWait(t, 10*time.Second)

	// Do some work to get reasonably nice error reports; what cmpEvents() gives
	// us is nice if you have just a few events, but with thousands it qiuckly
	// gets unwieldy.

	want := make(map[Event]struct{})
	for i := 0; i < numFiles; i++ {
		n := "/" + prefix + fmtNum(i)
		want[Event{Name: n, Op: Remove}] = struct{}{}
		want[Event{Name: n, Op: Create}] = struct{}{}
	}

	var extra Events
	for _, h := range have {
		h.Name = filepath.ToSlash(strings.TrimPrefix(h.Name, tmp))
		_, ok := want[h]
		if ok {
			delete(want, h)
		} else {
			extra = append(extra, h)
		}
	}

	if len(extra) > 0 {
		if len(extra) > 100 {
			Errorf(t, "saw %d unexpected events (showing first 100 only):\n%s", len(extra), extra[:100])
		} else {
			Errorf(t, "saw %d unexpected events:\n%s", len(extra), extra)
		}
	}

	if len(want) != 0 {
		wantE := make(Events, 0, len(want))
		for k := range want {
			wantE = append(wantE, k)
		}

		if len(wantE) > 100 {
			Errorf(t, "expected the following %d events, but didn't see them (showing first 100 only)\n%s", len(wantE), wantE[:100])
		} else {
			Errorf(t, "expected the following %d events, but didn't see them\n%s", len(wantE), wantE)
		}
	}
}

func TestWatchList(t *testing.T) {
	if runtime.GOOS == "windows" {
		// TODO: probably should I guess...
		t.Skip("WatchList has always been broken on Windows and I don't feel like fixing it")
	}

	t.Parallel()

	tmp := t.TempDir()
	file := join(tmp, "file")
	other := join(tmp, "other")

	touch(t, file)
	touch(t, other)

	w := newWatcher(t, file, tmp)
	defer w.Close()

	have := w.WatchList()
	sort.Strings(have)
	want := []string{tmp, file}
	if !reflect.DeepEqual(have, want) {
		t.Errorf("\nhave: %s\nwant: %s", have, want)
	}
}

func TestOpHas(t *testing.T) {
	tests := []struct {
		name string
		o    Op
		h    Op
		want bool
	}{
		{
			name: "single bit match",
			o:    Remove,
			h:    Remove,
			want: true,
		},
		{
			name: "single bit no match",
			o:    Remove,
			h:    Create,
			want: false,
		},
		{
			name: "two bits match",
			o:    Remove | Create,
			h:    Create,
			want: true,
		},
		{
			name: "two bits no match",
			o:    Remove | Create,
			h:    Chmod,
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.o.Has(tt.h); got != tt.want {
				t.Errorf("Has() = %v, want %v", got, tt.want)
			}
		})
	}
}

func BenchmarkWatch(b *testing.B) {
	do := func(b *testing.B, w *Watcher) {
		tmp := b.TempDir()
		file := join(tmp, "file")
		err := w.Add(tmp)
		if err != nil {
			b.Fatal(err)
		}

		var wg sync.WaitGroup
		wg.Add(1)
		go func() {
			for {
				select {
				case err, ok := <-w.Errors:
					if !ok {
						wg.Done()
						return
					}
					b.Error(err)
				case _, ok := <-w.Events:
					if !ok {
						wg.Done()
						return
					}
				}
			}
		}()

		b.ResetTimer()
		for n := 0; n < b.N; n++ {
			fp, err := os.Create(file)
			if err != nil {
				b.Fatal(err)
			}
			err = fp.Close()
			if err != nil {
				b.Fatal(err)
			}
		}
		err = w.Close()
		if err != nil {
			b.Fatal(err)
		}
		wg.Wait()
	}

	b.Run("default", func(b *testing.B) {
		w, err := NewWatcher()
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
	b.Run("buffered=1", func(b *testing.B) {
		w, err := NewBufferedWatcher(1)
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
	b.Run("buffered=1024", func(b *testing.B) {
		w, err := NewBufferedWatcher(1024)
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
	b.Run("buffered=4096", func(b *testing.B) {
		w, err := NewBufferedWatcher(4096)
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
}

func BenchmarkAddRemove(b *testing.B) {
	do := func(b *testing.B, w *Watcher) {
		tmp := b.TempDir()
		b.ResetTimer()
		for n := 0; n < b.N; n++ {
			if err := w.Add(tmp); err != nil {
				b.Fatal(err)
			}
			if err := w.Remove(tmp); err != nil {
				b.Fatal(err)
			}
		}
	}

	b.Run("default", func(b *testing.B) {
		w, err := NewWatcher()
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
	b.Run("buffered=1", func(b *testing.B) {
		w, err := NewBufferedWatcher(1)
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
	b.Run("buffered=1024", func(b *testing.B) {
		w, err := NewBufferedWatcher(1024)
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
	b.Run("buffered=4096", func(b *testing.B) {
		w, err := NewBufferedWatcher(4096)
		if err != nil {
			b.Fatal(err)
		}
		do(b, w)
	})
}
