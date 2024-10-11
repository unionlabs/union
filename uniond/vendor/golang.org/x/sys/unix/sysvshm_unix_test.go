// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build (darwin && amd64) || linux || zos

package unix_test

import (
	"runtime"
	"testing"

	"golang.org/x/sys/unix"
)

func TestSysvSharedMemory(t *testing.T) {
	// create ipc
	id, err := unix.SysvShmGet(unix.IPC_PRIVATE, 1024, unix.IPC_CREAT|unix.IPC_EXCL|0o600)

	// ipc isn't implemented on android, should fail
	if runtime.GOOS == "android" {
		if err != unix.ENOSYS {
			t.Fatalf("expected android to fail, but it didn't")
		}
		return
	}

	// The kernel may have been built without System V IPC support.
	if err == unix.ENOSYS {
		t.Skip("shmget not supported")
	}

	if err != nil {
		t.Fatalf("SysvShmGet: %v", err)
	}
	defer func() {
		_, err := unix.SysvShmCtl(id, unix.IPC_RMID, nil)
		if err != nil {
			t.Errorf("Remove failed: %v", err)
		}
	}()

	// attach
	b1, err := unix.SysvShmAttach(id, 0, 0)
	if err != nil {
		t.Fatalf("Attach: %v", err)
	}

	if runtime.GOOS == "zos" {
		// The minimum shared memory size is no less than units of 1M bytes in z/OS
		if len(b1) < 1024 {
			t.Fatalf("b1 len = %v, less than 1024", len(b1))
		}
	} else {
		if len(b1) != 1024 {
			t.Fatalf("b1 len = %v, want 1024", len(b1))
		}
	}

	b1[42] = 'x'

	// attach again
	b2, err := unix.SysvShmAttach(id, 0, 0)
	if err != nil {
		t.Fatalf("Attach: %v", err)
	}

	if runtime.GOOS == "zos" {
		// The returned shared memory alligns with the pagesize.
		// If pagesize is not 1024 bytes, the shared memory could be larger
		if len(b2) < 1024 {
			t.Fatalf("b1 len = %v, less than 1024", len(b2))
		}
	} else {
		if len(b2) != 1024 {
			t.Fatalf("b1 len = %v, want 1024", len(b2))
		}
	}

	b2[43] = 'y'
	if b2[42] != 'x' || b1[43] != 'y' {
		t.Fatalf("shared memory isn't shared")
	}

	// detach
	if err = unix.SysvShmDetach(b2); err != nil {
		t.Fatalf("Detach: %v", err)
	}

	if b1[42] != 'x' || b1[43] != 'y' {
		t.Fatalf("shared memory was invalidated")
	}
}
