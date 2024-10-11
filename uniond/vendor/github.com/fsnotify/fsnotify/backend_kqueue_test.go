//go:build freebsd || openbsd || netbsd || dragonfly || darwin
// +build freebsd openbsd netbsd dragonfly darwin

package fsnotify

import (
	"fmt"
	"strings"
	"testing"
)

func TestRemoveState(t *testing.T) {
	var (
		tmp  = t.TempDir()
		dir  = join(tmp, "dir")
		file = join(dir, "file")
	)
	mkdir(t, dir)
	touch(t, file)

	w := newWatcher(t, tmp)
	addWatch(t, w, tmp)
	addWatch(t, w, file)

	check := func(wantUser, wantTotal int) {
		t.Helper()

		if len(w.watches) != wantTotal {
			var d []string
			for k, v := range w.watches {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.watches (have %d, want %d):\n%v",
				len(w.watches), wantTotal, strings.Join(d, "\n"))
		}
		if len(w.paths) != wantTotal {
			var d []string
			for k, v := range w.paths {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.paths (have %d, want %d):\n%v",
				len(w.paths), wantTotal, strings.Join(d, "\n"))
		}
		if len(w.userWatches) != wantUser {
			var d []string
			for k, v := range w.userWatches {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.userWatches (have %d, want %d):\n%v",
				len(w.userWatches), wantUser, strings.Join(d, "\n"))
		}
	}

	check(2, 3)

	if err := w.Remove(file); err != nil {
		t.Fatal(err)
	}
	check(1, 2)

	if err := w.Remove(tmp); err != nil {
		t.Fatal(err)
	}
	check(0, 0)

	// Don't check these after ever remove since they don't map easily to number
	// of files watches. Just make sure they're 0 after everything is removed.
	{
		want := 0
		if len(w.watchesByDir) != want {
			var d []string
			for k, v := range w.watchesByDir {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.watchesByDir (have %d, want %d):\n%v",
				len(w.watchesByDir), want, strings.Join(d, "\n"))
		}
		if len(w.dirFlags) != want {
			var d []string
			for k, v := range w.dirFlags {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.dirFlags (have %d, want %d):\n%v",
				len(w.dirFlags), want, strings.Join(d, "\n"))
		}

		if len(w.fileExists) != want {
			var d []string
			for k, v := range w.fileExists {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.fileExists (have %d, want %d):\n%v",
				len(w.fileExists), want, strings.Join(d, "\n"))
		}
	}
}
