// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package testscript

import (
	"io"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

// TestingM is implemented by *testing.M. It's defined as an interface
// to allow testscript to co-exist with other testing frameworks
// that might also wish to call M.Run.
type TestingM interface {
	Run() int
}

// Deprecated: this option is no longer used.
func IgnoreMissedCoverage() {}

// RunMain should be called within a TestMain function to allow
// subcommands to be run in the testscript context.
//
// The commands map holds the set of command names, each
// with an associated run function which should return the
// code to pass to os.Exit. It's OK for a command function to
// exit itself, but this may result in loss of coverage information.
//
// When Run is called, these commands are installed as regular commands in the shell
// path, so can be invoked with "exec" or via any other command (for example a shell script).
//
// For backwards compatibility, the commands declared in the map can be run
// without "exec" - that is, "foo" will behave like "exec foo".
// This can be disabled with Params.RequireExplicitExec to keep consistency
// across test scripts, and to keep separate process executions explicit.
//
// This function returns an exit code to pass to os.Exit, after calling m.Run.
func RunMain(m TestingM, commands map[string]func() int) (exitCode int) {
	// Depending on os.Args[0], this is either the top-level execution of
	// the test binary by "go test", or the execution of one of the provided
	// commands via "foo" or "exec foo".

	cmdName := filepath.Base(os.Args[0])
	if runtime.GOOS == "windows" {
		cmdName = strings.TrimSuffix(cmdName, ".exe")
	}
	mainf := commands[cmdName]
	if mainf == nil {
		// Unknown command; this is just the top-level execution of the
		// test binary by "go test".

		// Set up all commands in a directory, added in $PATH.
		tmpdir, err := os.MkdirTemp("", "testscript-main")
		if err != nil {
			log.Printf("could not set up temporary directory: %v", err)
			return 2
		}
		defer func() {
			if err := os.RemoveAll(tmpdir); err != nil {
				log.Printf("cannot delete temporary directory: %v", err)
				exitCode = 2
			}
		}()
		bindir := filepath.Join(tmpdir, "bin")
		if err := os.MkdirAll(bindir, 0o777); err != nil {
			log.Printf("could not set up PATH binary directory: %v", err)
			return 2
		}
		os.Setenv("PATH", bindir+string(filepath.ListSeparator)+os.Getenv("PATH"))

		// We're not in a subcommand.
		for name := range commands {
			// Set up this command in the directory we added to $PATH.
			binfile := filepath.Join(bindir, name)
			if runtime.GOOS == "windows" {
				binfile += ".exe"
			}
			binpath, err := os.Executable()
			if err == nil {
				err = copyBinary(binpath, binfile)
			}
			if err != nil {
				log.Printf("could not set up %s in $PATH: %v", name, err)
				return 2
			}
			scriptCmds[name] = func(ts *TestScript, neg bool, args []string) {
				if ts.params.RequireExplicitExec {
					ts.Fatalf("use 'exec %s' rather than '%s' (because RequireExplicitExec is enabled)", name, name)
				}
				ts.cmdExec(neg, append([]string{name}, args...))
			}
		}
		return m.Run()
	}
	// The command being registered is being invoked, so run it, then exit.
	os.Args[0] = cmdName
	return mainf()
}

// copyBinary makes a copy of a binary to a new location. It is used as part of
// setting up top-level commands in $PATH.
//
// It does not attempt to use symlinks for two reasons:
//
// First, some tools like cmd/go's -toolexec will be clever enough to realise
// when they're given a symlink, and they will use the symlink target for
// executing the program. This breaks testscript, as we depend on os.Args[0] to
// know what command to run.
//
// Second, symlinks might not be available on some environments, so we have to
// implement a "full copy" fallback anyway.
//
// However, we do try to use cloneFile, since that will probably work on most
// unix-like setups. Note that "go test" also places test binaries in the
// system's temporary directory, like we do.
func copyBinary(from, to string) error {
	if err := cloneFile(from, to); err == nil {
		return nil
	}
	writer, err := os.OpenFile(to, os.O_WRONLY|os.O_CREATE, 0o777)
	if err != nil {
		return err
	}
	defer writer.Close()

	reader, err := os.Open(from)
	if err != nil {
		return err
	}
	defer reader.Close()

	_, err = io.Copy(writer, reader)
	return err
}
