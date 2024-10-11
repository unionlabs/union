// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build (darwin && !ios) || (linux && !android)

package unix_test

import (
	"log"

	"golang.org/x/sys/unix"
)

func ExampleSysvShmGet() {
	// create shared memory region of 1024 bytes
	id, err := unix.SysvShmGet(unix.IPC_PRIVATE, 1024, unix.IPC_CREAT|unix.IPC_EXCL|0o600)
	if err != nil {
		log.Fatal("sysv shm create failed:", err)
	}

	// warning: sysv shared memory segments persist even after after a process
	// is destroyed, so it's very important to explicitly delete it when you
	// don't need it anymore.
	defer func() {
		_, err := unix.SysvShmCtl(id, unix.IPC_RMID, nil)
		if err != nil {
			log.Fatal(err)
		}
	}()

	// to use a shared memory region you must attach to it
	b, err := unix.SysvShmAttach(id, 0, 0)
	if err != nil {
		log.Fatal("sysv attach failed:", err)
	}

	// you should detach from the segment when finished with it. The byte
	// slice is no longer valid after detaching
	defer func() {
		if err = unix.SysvShmDetach(b); err != nil {
			log.Fatal("sysv detach failed:", err)
		}
	}()

	// Changes to the contents of the byte slice are reflected in other
	// mappings of the shared memory identifer in this and other processes
	b[42] = 'h'
	b[43] = 'i'
}
