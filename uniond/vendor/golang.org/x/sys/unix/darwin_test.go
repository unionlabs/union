// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build darwin

package unix

import (
	"os"
	"os/exec"
	"strings"
	"testing"
)

type darwinTest struct {
	name string
	f    uintptr
}

// TODO(khr): decide whether to keep this test enabled permanently or
// only temporarily.
func TestDarwinLoader(t *testing.T) {
	// Make sure the Darwin dynamic loader can actually resolve
	// all the system calls into libSystem.dylib. Unfortunately
	// there is no easy way to test this at compile time. So we
	// implement a crazy hack here, calling into the syscall
	// function with all its arguments set to junk, and see what
	// error we get. We are happy with any error (or none) except
	// an error from the dynamic loader.
	//
	// We have to run each test in a separate subprocess for fault isolation.
	//
	// Hopefully the junk args won't accidentally ask the system to do "rm -fr /".
	//
	// In an ideal world each syscall would have its own test, so this test
	// would be unnecessary. Unfortunately, we do not live in that world.
	exe, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	for _, test := range darwinTests {
		// Call the test binary recursively, giving it a magic argument
		// (see init below) and the name of the test to run.
		cmd := exec.Command(exe, "testDarwinLoader", test.name)

		// Run subprocess, collect results. Note that we expect the subprocess
		// to fail somehow, so the error is irrelevant.
		out, _ := cmd.CombinedOutput()

		if strings.Contains(string(out), "dyld: Symbol not found:") {
			t.Errorf("can't resolve %s in libSystem.dylib", test.name)
		}
		if !strings.Contains(string(out), "success") {
			// Not really an error. Might be a syscall that never returns,
			// like exit, or one that segfaults, like gettimeofday.
			t.Logf("test never finished: %s: %s", test.name, string(out))
		}
	}
}

func init() {
	// The test binary execs itself with the "testDarwinLoader" argument.
	// Run the test specified by os.Args[2], then panic.
	if len(os.Args) >= 3 && os.Args[1] == "testDarwinLoader" {
		for _, test := range darwinTests {
			if test.name == os.Args[2] {
				syscall_syscall(test.f, ^uintptr(0), ^uintptr(0), ^uintptr(0))
			}
		}
		// Panic with a "success" label, so the parent process can check it.
		panic("success")
	}
}
