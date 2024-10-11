package atomicfile_test

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/creachadair/atomicfile"
	"github.com/creachadair/mds/mtest"
)

var (
	_ io.ReaderFrom = (*atomicfile.File)(nil)
)

func checkFile(t *testing.T, path string, perm os.FileMode, want string) {
	t.Helper()

	got, err := os.ReadFile(path)
	if err != nil {
		t.Errorf("Reading target: %v", err)
	}
	if s := string(got); s != want {
		t.Errorf("Target contents: got %q, want %q", s, want)
	}

	if fi, err := os.Stat(path); err != nil {
		t.Errorf("Stat failed: %v", err)
	} else if m := fi.Mode().Perm(); m != perm {
		t.Errorf("Target mode: got %v, want %v", m, perm)
	}
}

func TestFile(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "a", "b", "c", "target.txt")
	if err := os.MkdirAll(filepath.Dir(path), 0700); err != nil {
		t.Fatalf("Create output directory: %v", err)
	}

	f, err := atomicfile.New(path, 0623)
	if err != nil {
		t.Fatalf("New %q failed: %v", path, err)
	}

	const message = "Hello, world\n"
	fmt.Fprint(f, message)
	if err := f.Close(); err != nil {
		t.Errorf("Close failed: %v", err)
	}
	checkFile(t, path, 0623, message)
}

func TestExists(t *testing.T) {
	target := filepath.Join(t.TempDir(), "target")
	if err := os.Mkdir(target, 0755); err != nil {
		t.Fatalf("Create target directory: %v", err)
	}
	f, err := atomicfile.New(target, 0600)
	if err == nil {
		f.Cancel()
		t.Fatalf("New: got %v, want error", f)
	}
}

func TestCancel(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "target.txt")

	f, err := atomicfile.New(path, 0600)
	if err != nil {
		t.Fatalf("New %q failed: %v", path, err)
	}

	fmt.Fprintln(f, "Some of what a fool thinks often remains")
	f.Cancel()

	// After cancellation, a close should report an error.
	if err := f.Close(); err == nil {
		t.Error("Closing f should have reported an error")
	}

	// The target file should not exist, since it did not already.
	if fi, err := os.Stat(path); err == nil {
		t.Errorf("Stat %q should have failed, but found %d bytes", path, fi.Size())
	}

	// No temporary files should be left around.
	dc, err := os.ReadDir(tmp)
	if err != nil {
		t.Errorf("ReadDir %q: unexpected error: %v", tmp, err)
	}
	for _, fi := range dc {
		t.Errorf("Unexpected file %q in output directory", fi.Name())
	}
}

func TestNoClobber(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "target.txt")

	const oldMessage = "If I keep my eyes closed he looks just like you"
	if err := os.WriteFile(path, []byte(oldMessage), 0400); err != nil {
		t.Fatalf("Writing target file: %v", err)
	}

	f, err := atomicfile.New(path, 0644)
	if err != nil {
		t.Fatalf("New %q failed: %v", path, err)
	}

	fmt.Fprintln(f, "You should never see this")
	f.Cancel()

	if err := f.Close(); err == nil {
		t.Error("Closing f should have reported an error")
	}

	// After cancellation, the existing target should be unchanged.
	checkFile(t, path, 0400, oldMessage)
}

func TestDeferredCancel(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "target.txt")

	f, err := atomicfile.New(path, 0640)
	if err != nil {
		t.Fatalf("New %q failed: %v", path, err)
	}

	// A cancel that happens after a successful close does not interfere with
	// the output.
	const message = "There's a place way down in Bed-Stuy"
	func() {
		defer f.Cancel()
		fmt.Fprint(f, message)

		if err := f.Close(); err != nil {
			t.Errorf("Close failed: %v", err)
		}
	}()

	checkFile(t, path, 0640, message)
}

func TestSameDirectory(t *testing.T) {
	tmp := t.TempDir()

	f, err := atomicfile.New("xyzzy", 0644)
	if err != nil {
		t.Fatalf("New failed: %v", err)
	}
	defer f.Cancel()

	m, err := filepath.Glob("xyzzy-*.aftmp")
	if err != nil {
		t.Fatalf("Invalid glob: %v", err)
	}
	if len(m) == 0 {
		t.Errorf("No matches for temp file in %q", tmp)
	} else {
		t.Logf("Found matching temp: %q", m[0])
	}
}

func TestTx(t *testing.T) {
	testErr := errors.New("plumbing error")

	t.Run("Error", func(t *testing.T) {
		path := filepath.Join(t.TempDir(), "nonesuch.txt")
		err := atomicfile.Tx(path, 0600, func(*atomicfile.File) error {
			return testErr
		})
		if err != testErr {
			t.Errorf("Got error %v, want %v", err, testErr)
		}
		if _, err := os.Stat(path); !os.IsNotExist(err) {
			t.Errorf("Target path should not exist, err=%v", err)
		}
	})

	t.Run("OK", func(t *testing.T) {
		const text = "hello world\n"
		path := filepath.Join(t.TempDir(), "goodies.txt")
		err := atomicfile.Tx(path, 0604, func(f *atomicfile.File) error {
			io.WriteString(f, text)
			return nil
		})
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		checkFile(t, path, 0604, text)
	})

	t.Run("Panic", func(t *testing.T) {
		path := filepath.Join(t.TempDir(), "knucklebones.txt")
		v := mtest.MustPanic(t, func() {
			atomicfile.Tx(path, 0600, func(*atomicfile.File) error {
				panic("ouchies")
			})
		})

		// Make sure we got the panic from the callback.
		if s, ok := v.(string); !ok || s != "ouchies" {
			t.Errorf("Unexpected panic: %v", v)
		}

		// Make sure nothing was left in the output directory.
		elts, err := os.ReadDir(filepath.Dir(path))
		if err != nil {
			t.Fatalf("Reading output directory: %v", err)
		} else if len(elts) != 0 {
			t.Errorf("Unexpected output: %v", elts)
		}
	})
}

func TestWrite(t *testing.T) {
	const input = "some of what a fool thinks often remains"

	t.Run("WriteAll", func(t *testing.T) {
		path := filepath.Join(t.TempDir(), "target.txt")
		data := strings.NewReader(input)

		nw, err := atomicfile.WriteAll(path, data, 0600)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if int(nw) != len(input) {
			t.Errorf("Length: got %d, want %d", nw, len(input))
		}
		checkFile(t, path, 0600, input)
	})

	t.Run("WriteData", func(t *testing.T) {
		path := filepath.Join(t.TempDir(), "target.txt")
		if err := atomicfile.WriteData(path, []byte(input), 0664); err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		checkFile(t, path, 0664, input)
	})
}
