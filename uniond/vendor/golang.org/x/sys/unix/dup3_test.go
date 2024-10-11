// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build freebsd || linux || netbsd || openbsd

package unix_test

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"

	"golang.org/x/sys/unix"
)

func TestDup3(t *testing.T) {
	tempFile, err := os.Create(filepath.Join(t.TempDir(), "TestDup"))
	if err != nil {
		t.Fatal(err)
	}
	defer tempFile.Close()
	oldFd := int(tempFile.Fd())

	// On NetBSD, it is not an error if oldFd == newFd
	if runtime.GOOS != "netbsd" {
		if got, want := unix.Dup3(oldFd, oldFd, 0), unix.EINVAL; got != want {
			t.Fatalf("Dup3: expected err %v, got %v", want, got)
		}
	}

	// Create and reserve a file descriptor.
	// Dup3 automatically closes it before reusing it.
	nullFile, err := os.Open("/dev/null")
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer nullFile.Close()
	newFd := int(nullFile.Fd())

	err = unix.Dup3(oldFd, newFd, 0)
	if err != nil {
		t.Fatalf("Dup3: %v", err)
	}

	b1 := []byte("Test123")
	b2 := make([]byte, 7)
	_, err = unix.Write(newFd, b1)
	if err != nil {
		t.Fatalf("Write to Dup3 fd failed: %v", err)
	}
	_, err = unix.Seek(oldFd, 0, 0)
	if err != nil {
		t.Fatalf("Seek failed: %v", err)
	}
	_, err = unix.Read(oldFd, b2)
	if err != nil {
		t.Fatalf("Read back failed: %v", err)
	}
	if string(b1) != string(b2) {
		t.Errorf("Dup3: read %q from file, want %q", string(b2), string(b1))
	}
}
