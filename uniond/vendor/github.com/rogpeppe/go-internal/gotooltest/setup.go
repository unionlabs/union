// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package gotooltest implements functionality useful for testing
// tools that use the go command.
package gotooltest

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
	"sync"

	"github.com/rogpeppe/go-internal/testscript"
)

var (
	goVersionRegex = regexp.MustCompile(`^go([1-9][0-9]*)\.([1-9][0-9]*)$`)

	goEnv struct {
		GOROOT      string
		GOCACHE     string
		GOPROXY     string
		goversion   string
		releaseTags []string
		once        sync.Once
		err         error
	}
)

// initGoEnv initialises goEnv. It should only be called using goEnv.once.Do,
// as in Setup.
//
// Run all of these probe commands in a temporary directory, so as not to make
// any assumptions about the caller's working directory.
func initGoEnv() (err error) {
	td, err := os.MkdirTemp("", "gotooltest-initGoEnv")
	if err != nil {
		return fmt.Errorf("failed to create temporary directory for go command tests: %w", err)
	}
	defer func() {
		if rerr := os.RemoveAll(td); rerr != nil && err == nil {
			err = fmt.Errorf("failed to remove temporary directory for go command tests: %w", rerr)
		}
	}()

	// Write a temporary go.mod file in td. This ensures that we create
	// a porcelain environment in which to run these probe commands.
	if err := os.WriteFile(filepath.Join(td, "go.mod"), []byte("module gotooltest"), 0600); err != nil {
		return fmt.Errorf("failed to write temporary go.mod file: %w", err)
	}

	run := func(args ...string) (*bytes.Buffer, *bytes.Buffer, error) {
		var stdout, stderr bytes.Buffer
		cmd := exec.Command(args[0], args[1:]...)
		cmd.Dir = td
		cmd.Stdout = &stdout
		cmd.Stderr = &stderr
		return &stdout, &stderr, cmd.Run()
	}

	lout, stderr, err := run("go", "list", "-f={{context.ReleaseTags}}", "runtime")
	if err != nil {
		return fmt.Errorf("failed to determine release tags from go command: %v\n%v", err, stderr.String())
	}
	tagStr := strings.TrimSpace(lout.String())
	tagStr = strings.Trim(tagStr, "[]")
	goEnv.releaseTags = strings.Split(tagStr, " ")

	eout, stderr, err := run("go", "env", "-json",
		"GOROOT",
		"GOCACHE",
		"GOPROXY",
	)
	if err != nil {
		return fmt.Errorf("failed to determine environment from go command: %v\n%v", err, stderr)
	}
	if err := json.Unmarshal(eout.Bytes(), &goEnv); err != nil {
		return fmt.Errorf("failed to unmarshal GOROOT and GOCACHE tags from go command out: %v\n%v", err, eout)
	}

	version := goEnv.releaseTags[len(goEnv.releaseTags)-1]
	if !goVersionRegex.MatchString(version) {
		return fmt.Errorf("invalid go version %q", version)
	}
	goEnv.goversion = version[2:]

	return nil
}

// Setup sets up the given test environment for tests that use the go
// command. It adds support for go tags to p.Condition and adds the go
// command to p.Cmds. It also wraps p.Setup to set up the environment
// variables for running the go command appropriately.
//
// It checks go command can run, but not that it can build or run
// binaries.
func Setup(p *testscript.Params) error {
	goEnv.once.Do(func() {
		goEnv.err = initGoEnv()
	})
	if goEnv.err != nil {
		return goEnv.err
	}

	origSetup := p.Setup
	p.Setup = func(e *testscript.Env) error {
		e.Vars = goEnviron(e.Vars)
		if origSetup != nil {
			return origSetup(e)
		}
		return nil
	}
	if p.Cmds == nil {
		p.Cmds = make(map[string]func(ts *testscript.TestScript, neg bool, args []string))
	}
	p.Cmds["go"] = cmdGo
	return nil
}

func goEnviron(env0 []string) []string {
	env := environ(env0)
	workdir := env.get("WORK")
	return append(env, []string{
		"GOPATH=" + filepath.Join(workdir, ".gopath"),
		"CCACHE_DISABLE=1", // ccache breaks with non-existent HOME
		"GOARCH=" + runtime.GOARCH,
		"GOOS=" + runtime.GOOS,
		"GOROOT=" + goEnv.GOROOT,
		"GOCACHE=" + goEnv.GOCACHE,
		"GOPROXY=" + goEnv.GOPROXY,
		"goversion=" + goEnv.goversion,
	}...)
}

func cmdGo(ts *testscript.TestScript, neg bool, args []string) {
	if len(args) < 1 {
		ts.Fatalf("usage: go subcommand ...")
	}
	err := ts.Exec("go", args...)
	if err != nil {
		ts.Logf("[%v]\n", err)
		if !neg {
			ts.Fatalf("unexpected go command failure")
		}
	} else {
		if neg {
			ts.Fatalf("unexpected go command success")
		}
	}
}

type environ []string

func (e0 *environ) get(name string) string {
	e := *e0
	for i := len(e) - 1; i >= 0; i-- {
		v := e[i]
		if len(v) <= len(name) {
			continue
		}
		if strings.HasPrefix(v, name) && v[len(name)] == '=' {
			return v[len(name)+1:]
		}
	}
	return ""
}
