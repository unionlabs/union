// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

//go:build !windows
// +build !windows

package plugin

import (
	"fmt"
	"os"
	"os/user"
	"runtime"
	"syscall"
	"testing"
)

func TestUnixSocketGroupPermissions(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("go-plugin doesn't support unix sockets on Windows")
	}

	group, err := user.LookupGroupId(fmt.Sprintf("%d", os.Getgid()))
	if err != nil {
		t.Fatal(err)
	}
	for name, tc := range map[string]struct {
		group string
	}{
		"as integer": {fmt.Sprintf("%d", os.Getgid())},
		"as name":    {group.Name},
	} {
		t.Run(name, func(t *testing.T) {
			ln, err := serverListener_unix(UnixSocketConfig{Group: tc.group})
			if err != nil {
				t.Fatal(err)
			}
			defer ln.Close()

			info, err := os.Lstat(ln.Addr().String())
			if err != nil {
				t.Fatal(err)
			}
			if info.Mode()&os.ModePerm != 0o660 {
				t.Fatal(info.Mode())
			}
			stat, ok := info.Sys().(*syscall.Stat_t)
			if !ok {
				t.Fatal()
			}
			if stat.Gid != uint32(os.Getgid()) {
				t.Fatalf("Expected %d, but got %d", os.Getgid(), stat.Gid)
			}
		})
	}
}
