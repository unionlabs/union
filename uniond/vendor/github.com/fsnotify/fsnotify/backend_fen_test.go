//go:build solaris
// +build solaris

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

	check := func(wantDirs, wantFiles int) {
		t.Helper()
		if len(w.watches) != wantFiles {
			var d []string
			for k, v := range w.watches {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.watches (have %d, want %d):\n%v",
				len(w.watches), wantFiles, strings.Join(d, "\n"))
		}
		if len(w.dirs) != wantDirs {
			var d []string
			for k, v := range w.dirs {
				d = append(d, fmt.Sprintf("%#v = %#v", k, v))
			}
			t.Errorf("unexpected number of entries in w.dirs (have %d, want %d):\n%v",
				len(w.dirs), wantDirs, strings.Join(d, "\n"))
		}
	}

	check(1, 1)

	if err := w.Remove(file); err != nil {
		t.Fatal(err)
	}
	check(1, 0)

	if err := w.Remove(tmp); err != nil {
		t.Fatal(err)
	}
	check(0, 0)
}
