//go:build windows

package fs

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/sys/windows"
)

func Test_GetFinalPathNameByHandle(t *testing.T) {
	d := t.TempDir()
	// open f via a relative path
	name := t.Name() + ".txt"
	fullPath := filepath.Join(d, name)

	w, err := os.Getwd()
	if err != nil {
		t.Fatalf("could not get working directory: %v", err)
	}
	if err := os.Chdir(d); err != nil {
		t.Fatalf("could not chdir to %s: %v", d, err)
	}
	defer os.Chdir(w) //nolint:errcheck

	f, err := os.Create(name)
	if err != nil {
		t.Fatalf("could not open %s: %v", fullPath, err)
	}
	defer f.Close()

	path, err := GetFinalPathNameByHandle(windows.Handle(f.Fd()), GetFinalPathDefaultFlag)
	if err != nil {
		t.Fatalf("could not get final path for %s: %v", fullPath, err)
	}
	if strings.EqualFold(fullPath, path) {
		t.Fatalf("expected %s, got %s", fullPath, path)
	}
}
