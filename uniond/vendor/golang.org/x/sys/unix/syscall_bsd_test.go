// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build darwin || dragonfly || freebsd || netbsd || openbsd

package unix_test

import (
	"runtime"
	"testing"
	"time"

	"golang.org/x/sys/unix"
)

func TestSysctlRaw(t *testing.T) {
	switch runtime.GOOS {
	case "netbsd", "openbsd":
		t.Skipf("kern.proc.pid does not exist on %s", runtime.GOOS)
	}

	_, err := unix.SysctlRaw("kern.proc.pid", unix.Getpid())
	if err != nil {
		t.Fatal(err)
	}
}

func TestSysctlUint32(t *testing.T) {
	maxproc, err := unix.SysctlUint32("kern.maxproc")
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("kern.maxproc: %v", maxproc)
}

func TestSysctlClockinfo(t *testing.T) {
	ci, err := unix.SysctlClockinfo("kern.clockrate")
	if err != nil {
		if runtime.GOOS == "openbsd" && (err == unix.ENOMEM || err == unix.EIO) {
			if osrev, _ := unix.SysctlUint32("kern.osrevision"); osrev <= 202010 {
				// SysctlClockinfo should fail gracefully due to a struct size
				// mismatch on OpenBSD 6.8 and earlier, see
				// https://golang.org/issue/47629
				return
			}
		}
		t.Fatal(err)
	}
	t.Logf("tick = %v, hz = %v, profhz = %v, stathz = %v",
		ci.Tick, ci.Hz, ci.Profhz, ci.Stathz)
}

func TestSysctlTimeval(t *testing.T) {
	tv, err := unix.SysctlTimeval("kern.boottime")
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("boottime = %v", time.Unix(tv.Unix()))
}
