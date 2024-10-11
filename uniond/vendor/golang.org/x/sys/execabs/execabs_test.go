// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package execabs

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

// hasExec reports whether the current system can start new processes
// using os.StartProcess or (more commonly) exec.Command.
// Copied from internal/testenv.HasExec
func hasExec() bool {
	switch runtime.GOOS {
	case "wasip1", "js", "ios":
		return false
	}
	return true
}

// mustHaveExec checks that the current system can start new processes
// using os.StartProcess or (more commonly) exec.Command.
// If not, mustHaveExec calls t.Skip with an explanation.
// Copied from internal/testenv.MustHaveExec
func mustHaveExec(t testing.TB) {
	if !hasExec() {
		t.Skipf("skipping test: cannot exec subprocess on %s/%s", runtime.GOOS, runtime.GOARCH)
	}
}

func TestFixCmd(t *testing.T) {
	cmd := &exec.Cmd{Path: "hello"}
	fixCmd("hello", cmd)
	if cmd.Path != "" {
		t.Errorf("fixCmd didn't clear cmd.Path")
	}
	expectedErr := fmt.Sprintf("hello resolves to executable in current directory (.%chello)", filepath.Separator)
	if err := cmd.Run(); err == nil {
		t.Fatal("Command.Run didn't fail")
	} else if err.Error() != expectedErr {
		t.Fatalf("Command.Run returned unexpected error: want %q, got %q", expectedErr, err.Error())
	}
}

func TestCommand(t *testing.T) {
	mustHaveExec(t)

	for _, cmd := range []func(string) *Cmd{
		func(s string) *Cmd { return Command(s) },
		func(s string) *Cmd { return CommandContext(context.Background(), s) },
	} {
		tmpDir := t.TempDir()
		executable := "execabs-test"
		if runtime.GOOS == "windows" {
			executable += ".exe"
		}
		if err := os.WriteFile(filepath.Join(tmpDir, executable), []byte{1, 2, 3}, 0111); err != nil {
			t.Fatal(err)
		}
		cwd, err := os.Getwd()
		if err != nil {
			t.Fatalf("os.Getwd failed: %s", err)
		}
		defer os.Chdir(cwd)
		if err = os.Chdir(tmpDir); err != nil {
			t.Fatalf("os.Chdir failed: %s", err)
		}
		if runtime.GOOS != "windows" {
			// add "." to PATH so that exec.LookPath looks in the current directory on
			// non-windows platforms as well
			origPath := os.Getenv("PATH")
			defer os.Setenv("PATH", origPath)
			os.Setenv("PATH", fmt.Sprintf(".:%s", origPath))
		}
		expectedErr := fmt.Sprintf("execabs-test resolves to executable in current directory (.%c%s)", filepath.Separator, executable)
		if err = cmd("execabs-test").Run(); err == nil {
			t.Fatalf("Command.Run didn't fail when exec.LookPath returned a relative path")
		} else if err.Error() != expectedErr && !isGo119ErrDot(err) {
			t.Errorf("Command.Run returned unexpected error: want %q, got %q", expectedErr, err.Error())
		}
	}
}

func TestLookPath(t *testing.T) {
	mustHaveExec(t)

	tmpDir := t.TempDir()
	executable := "execabs-test"
	if runtime.GOOS == "windows" {
		executable += ".exe"
	}
	if err := os.WriteFile(filepath.Join(tmpDir, executable), []byte{1, 2, 3}, 0111); err != nil {
		t.Fatal(err)
	}
	cwd, err := os.Getwd()
	if err != nil {
		t.Fatalf("os.Getwd failed: %s", err)
	}
	defer os.Chdir(cwd)
	if err = os.Chdir(tmpDir); err != nil {
		t.Fatalf("os.Chdir failed: %s", err)
	}
	if runtime.GOOS != "windows" {
		// add "." to PATH so that exec.LookPath looks in the current directory on
		// non-windows platforms as well
		origPath := os.Getenv("PATH")
		defer os.Setenv("PATH", origPath)
		os.Setenv("PATH", fmt.Sprintf(".:%s", origPath))
	}
	expectedErr := fmt.Sprintf("execabs-test resolves to executable in current directory (.%c%s)", filepath.Separator, executable)
	if _, err := LookPath("execabs-test"); err == nil {
		t.Fatalf("LookPath didn't fail when finding a non-relative path")
	} else if err.Error() != expectedErr {
		t.Errorf("LookPath returned unexpected error: want %q, got %q", expectedErr, err.Error())
	}
}

// Issue #58606
func TestDoesNotExist(t *testing.T) {
	err := Command("this-executable-should-not-exist").Start()
	if err == nil {
		t.Fatal("command should have failed")
	}
	if strings.Contains(err.Error(), "resolves to executable in current directory") {
		t.Errorf("error (%v) should not refer to current directory", err)
	}
}
