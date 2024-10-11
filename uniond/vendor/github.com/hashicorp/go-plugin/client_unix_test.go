// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

//go:build !windows
// +build !windows

package plugin

import (
	"fmt"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"
	"runtime"
	"syscall"
	"testing"

	"github.com/hashicorp/go-hclog"
	"github.com/hashicorp/go-plugin/internal/cmdrunner"
	"github.com/hashicorp/go-plugin/runner"
)

func TestSetGroup(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("go-plugin doesn't support unix sockets on Windows")
	}

	group, err := user.LookupGroupId(fmt.Sprintf("%d", os.Getgid()))
	if err != nil {
		t.Fatal(err)
	}
	baseTempDir := t.TempDir()
	baseTempDir, err = filepath.EvalSymlinks(baseTempDir)
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
			process := helperProcess("mock")
			c := NewClient(&ClientConfig{
				HandshakeConfig: testHandshake,
				Plugins:         testPluginMap,
				UnixSocketConfig: &UnixSocketConfig{
					Group:   tc.group,
					TempDir: baseTempDir,
				},
				RunnerFunc: func(l hclog.Logger, cmd *exec.Cmd, tmpDir string) (runner.Runner, error) {
					// Run tests inside the RunnerFunc to ensure we don't race
					// with the code that deletes tmpDir when the client fails
					// to start properly.

					// Test that it creates a directory with the proper owners and permissions.
					if filepath.Dir(tmpDir) != baseTempDir {
						t.Errorf("Expected base TempDir to be %s, but tmpDir was %s", baseTempDir, tmpDir)
					}
					info, err := os.Lstat(tmpDir)
					if err != nil {
						t.Fatal(err)
					}
					if info.Mode()&os.ModePerm != 0o770 {
						t.Fatal(info.Mode())
					}
					stat, ok := info.Sys().(*syscall.Stat_t)
					if !ok {
						t.Fatal()
					}
					if stat.Gid != uint32(os.Getgid()) {
						t.Fatalf("Expected %d, but got %d", os.Getgid(), stat.Gid)
					}

					// Check the correct environment variables were set to forward
					// Unix socket config onto the plugin.
					var foundUnixSocketDir, foundUnixSocketGroup bool
					for _, env := range cmd.Env {
						if env == fmt.Sprintf("%s=%s", EnvUnixSocketDir, tmpDir) {
							foundUnixSocketDir = true
						}
						if env == fmt.Sprintf("%s=%s", EnvUnixSocketGroup, tc.group) {
							foundUnixSocketGroup = true
						}
					}
					if !foundUnixSocketDir {
						t.Errorf("Did not find correct %s env in %v", EnvUnixSocketDir, cmd.Env)
					}
					if !foundUnixSocketGroup {
						t.Errorf("Did not find correct %s env in %v", EnvUnixSocketGroup, cmd.Env)
					}

					process.Env = append(process.Env, cmd.Env...)
					return cmdrunner.NewCmdRunner(l, process)
				},
			})
			defer c.Kill()

			_, err := c.Start()
			if err != nil {
				t.Fatalf("err should be nil, got %s", err)
			}
		})
	}
}
