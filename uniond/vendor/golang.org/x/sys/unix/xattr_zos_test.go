// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build zos

package unix_test

import (
	"os"
	"testing"

	"golang.org/x/sys/unix"
)

func TestXattr(t *testing.T) {
	defer chtmpdir(t)()

	f := "xattr1"
	touch(t, f)

	xattrName := "system.filetag"
	xattrDataSet := make([]byte, 4)
	xattrDataSet[0] = 0xFF
	xattrDataSet[1] = 0xFF
	xattrDataSet[2] = 0
	xattrDataSet[3] = 0

	err := unix.Setxattr(f, xattrName, xattrDataSet, 0)
	if err != nil {
		t.Fatalf("Setxattr: %v", err)
	}

	// find size
	size, err := unix.Listxattr(f, nil)
	if err != nil {
		t.Fatalf("Listxattr: %v", err)
	}

	if size <= 0 {
		t.Fatalf("Listxattr returned an empty list of attributes")
	}

	buf := make([]byte, size)
	read, err := unix.Listxattr(f, buf)
	if err != nil {
		t.Fatalf("Listxattr: %v", err)
	}

	xattrs := stringsFromByteSlice(buf[:read])

	xattrWant := xattrName
	found := false
	for _, name := range xattrs {
		if name == xattrWant {
			found = true
			break
		}
	}

	if !found {
		t.Errorf("Listxattr did not return previously set attribute '%s'", xattrName)
	}

	// find size
	size, err = unix.Getxattr(f, xattrName, nil)
	if err != nil {
		t.Fatalf("Getxattr: %v", err)
	}

	if size <= 0 {
		t.Fatalf("Getxattr returned an empty attribute")
	}

	xattrDataGet := make([]byte, size)
	_, err = unix.Getxattr(f, xattrName, xattrDataGet)
	if err != nil {
		t.Fatalf("Getxattr: %v", err)
	}
	got := string([]byte(xattrDataGet))
	if got != string(xattrDataSet) {
		t.Errorf("Getxattr: expected attribute value %s, got %s", xattrDataSet, got)
	}

	err = unix.Removexattr(f, xattrName)
	if err != nil {
		t.Fatalf("Removexattr: %v", err)
	}

	//confirm xattr removed
	// find size
	size, err = unix.Listxattr(f, nil)
	if err != nil {
		t.Fatalf("Listxattr: %v", err)
	}
	if size != 0 {
		buf := make([]byte, size)
		read, err = unix.Listxattr(f, buf)
		if err != nil {
			t.Fatalf("Listxattr: %v", err)
		}

		xattrs = stringsFromByteSlice(buf[:read])

		found = false
		for _, name := range xattrs {
			if name == xattrWant {
				found = true
				break
			}
			if found {
				t.Errorf("Removexattr failed to remove attribute '%s'", xattrName)
			}
		}
	}
	n := "nonexistent"
	err = unix.Lsetxattr(n, xattrName, xattrDataSet, 0)
	if err != unix.ENOENT {
		t.Errorf("Lsetxattr: expected %v on non-existent file, got %v", unix.ENODATA, err)
	}

	_, err = unix.Lgetxattr(f, n, nil)
	if err != unix.ENODATA {
		t.Errorf("Lgetxattr: %v", err)
	}

	_, err = unix.Lgetxattr(n, xattrName, nil)
	if err != unix.ENOENT {
		t.Errorf("Lgetxattr: %v", err)
	}
}

func TestFdXattr(t *testing.T) {
	file, err := os.CreateTemp("", "TestFdXattr")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(file.Name())
	defer file.Close()

	fd := int(file.Fd())
	xattrName := "system.filetag"
	xattrDataSet := make([]byte, 4)
	xattrDataSet[0] = 0xFF
	xattrDataSet[1] = 0xFF
	xattrDataSet[2] = 0
	xattrDataSet[3] = 0

	err = unix.Fsetxattr(fd, xattrName, xattrDataSet, 0)
	if err == unix.ENOTSUP || err == unix.EOPNOTSUPP {
		t.Skip("filesystem does not support extended attributes, skipping test")
	} else if err != nil {
		t.Fatalf("Fsetxattr: %v", err)
	}

	// find size
	size, err := unix.Flistxattr(fd, nil)
	if err != nil {
		t.Fatalf("Flistxattr: %v", err)
	}
	if size <= 0 {
		t.Fatalf("Flistxattr returned an empty list of attributes")
	}

	buf := make([]byte, size)
	read, err := unix.Flistxattr(fd, buf)
	if err != nil {
		t.Fatalf("Flistxattr: %v", err)
	}

	xattrs := stringsFromByteSlice(buf[:read])

	xattrWant := xattrName
	found := false
	for _, name := range xattrs {
		if name == xattrWant {
			found = true
			break
		}
	}

	if !found {
		t.Errorf("Flistxattr did not return previously set attribute '%s'", xattrName)
	}

	// find size
	size, err = unix.Fgetxattr(fd, xattrName, nil)
	if err != nil {
		t.Fatalf("Fgetxattr: %v", err)
	}

	if size <= 0 {
		t.Fatalf("Fgetxattr returned an empty attribute")
	}

	xattrDataGet := make([]byte, size)
	_, err = unix.Fgetxattr(fd, xattrName, xattrDataGet)
	if err != nil {
		t.Fatalf("Fgetxattr: %v", err)
	}

	got := string([]byte(xattrDataGet))
	if got != string(xattrDataSet) {
		t.Errorf("Getxattr: expected attribute value %s, got %s", xattrDataSet, got)
	}

	err = unix.Fremovexattr(fd, xattrName)
	if err != nil {
		t.Fatalf("Fremovexattr: %v", err)
	}

	//confirm xattr removed
	// find size
	size, err = unix.Flistxattr(fd, nil)
	if err != nil {
		t.Fatalf("Flistxattr: %v", err)
	}
	if size != 0 {
		buf := make([]byte, size)
		read, err = unix.Flistxattr(fd, buf)
		if err != nil {
			t.Fatalf("Flistxattr: %v", err)
		}

		xattrs = stringsFromByteSlice(buf[:read])

		found = false
		for _, name := range xattrs {
			if name == xattrWant {
				found = true
				break
			}
			if found {
				t.Errorf("Fremovexattr failed to remove attribute '%s'", xattrName)
			}
		}
	}
}
