// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync/atomic"

	"github.com/rogpeppe/go-internal/goproxytest"
	"github.com/rogpeppe/go-internal/gotooltest"
	"github.com/rogpeppe/go-internal/testscript"
)

const (
	// goModProxyDir is the special subdirectory in a txtar script's supporting files
	// within which we expect to find github.com/rogpeppe/go-internal/goproxytest
	// directories.
	goModProxyDir = ".gomodproxy"
)

type envVarsFlag struct {
	vals []string
}

func (e *envVarsFlag) String() string {
	return fmt.Sprintf("%v", e.vals)
}

func (e *envVarsFlag) Set(v string) error {
	e.vals = append(e.vals, v)
	return nil
}

func main() {
	os.Exit(main1())
}

func main1() int {
	switch err := mainerr(); err {
	case nil:
		return 0
	case flag.ErrHelp:
		return 2
	default:
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
}

func mainerr() (retErr error) {
	fs := flag.NewFlagSet(os.Args[0], flag.ContinueOnError)
	fs.Usage = func() {
		mainUsage(os.Stderr)
	}
	var envVars envVarsFlag
	fUpdate := fs.Bool("u", false, "update archive file if a cmp fails")
	fWork := fs.Bool("work", false, "print temporary work directory and do not remove when done")
	fContinue := fs.Bool("continue", false, "continue running the script if an error occurs")
	fVerbose := fs.Bool("v", false, "run tests verbosely")
	fs.Var(&envVars, "e", "pass through environment variable to script (can appear multiple times)")
	if err := fs.Parse(os.Args[1:]); err != nil {
		return err
	}

	files := fs.Args()
	if len(files) == 0 {
		files = []string{"-"}
	}

	// If we are only reading from stdin, -u cannot be specified. It seems a bit
	// bizarre to invoke testscript with '-' and a regular file, but hey. In
	// that case the -u flag will only apply to the regular file and we assume
	// the user knows it.
	onlyReadFromStdin := true
	for _, f := range files {
		if f != "-" {
			onlyReadFromStdin = false
		}
	}
	if onlyReadFromStdin && *fUpdate {
		return fmt.Errorf("cannot use -u when reading from stdin")
	}
	var stdinTempFile string
	for i, f := range files {
		if f != "-" {
			continue
		}
		if stdinTempFile != "" {
			return fmt.Errorf("cannot read stdin twice")
		}
		data, err := io.ReadAll(os.Stdin)
		if err != nil {
			return fmt.Errorf("error reading stdin: %v", err)
		}
		f, err := os.CreateTemp("", "stdin*.txtar")
		if err != nil {
			return err
		}
		if _, err := f.Write(data); err != nil {
			return err
		}
		if err := f.Close(); err != nil {
			return err
		}
		stdinTempFile = f.Name()
		files[i] = stdinTempFile
		defer os.Remove(stdinTempFile)
	}

	p := testscript.Params{
		Setup:           func(*testscript.Env) error { return nil },
		Files:           files,
		UpdateScripts:   *fUpdate,
		ContinueOnError: *fContinue,
		TestWork:        *fWork,
	}

	if _, err := exec.LookPath("go"); err == nil {
		if err := gotooltest.Setup(&p); err != nil {
			return fmt.Errorf("failed to setup go tool: %v", err)
		}
	}
	origSetup := p.Setup
	p.Setup = func(env *testscript.Env) error {
		if err := origSetup(env); err != nil {
			return err
		}
		if *fWork {
			env.T().Log("temporary work directory: ", env.WorkDir)
		}
		proxyDir := filepath.Join(env.WorkDir, goModProxyDir)
		if info, err := os.Stat(proxyDir); err == nil && info.IsDir() {
			srv, err := goproxytest.NewServer(proxyDir, "")
			if err != nil {
				return fmt.Errorf("cannot start Go proxy: %v", err)
			}
			env.Defer(srv.Close)

			// Add GOPROXY after calling the original setup
			// so that it overrides any GOPROXY set there.
			env.Vars = append(env.Vars,
				"GOPROXY="+srv.URL,
				"GONOSUMDB=*",
			)
		}
		for _, v := range envVars.vals {
			varName, _, ok := strings.Cut(v, "=")
			if !ok {
				v += "=" + os.Getenv(v)
			}
			switch varName {
			case "":
				return fmt.Errorf("invalid variable name %q", varName)
			case "WORK":
				return fmt.Errorf("cannot override WORK variable")
			}
			env.Vars = append(env.Vars, v)
		}
		return nil
	}

	r := &runT{
		verbose:       *fVerbose,
		stdinTempFile: stdinTempFile,
	}
	r.Run("", func(t testscript.T) {
		testscript.RunT(t, p)
	})
	if r.failed.Load() {
		return failedRun
	}
	return nil
}

var (
	failedRun = errors.New("failed run")
	skipRun   = errors.New("skip")
)

// runT implements testscript.T and is used in the call to testscript.Run
type runT struct {
	verbose       bool
	stdinTempFile string
	failed        atomic.Bool
}

func (r *runT) Skip(is ...interface{}) {
	panic(skipRun)
}

func (r *runT) Fatal(is ...interface{}) {
	r.Log(is...)
	r.FailNow()
}

func (r *runT) Parallel() {
	// TODO run tests in parallel.
}

func (r *runT) Log(is ...interface{}) {
	msg := fmt.Sprint(is...)
	if r.stdinTempFile != "" {
		msg = strings.ReplaceAll(msg, r.stdinTempFile, "<stdin>")
	}
	if !strings.HasSuffix(msg, "\n") {
		msg += "\n"
	}
	fmt.Print(msg)
}

func (r *runT) FailNow() {
	panic(failedRun)
}

func (r *runT) Run(name string, f func(t testscript.T)) {
	// TODO: perhaps log the test name when there's more
	// than one test file?
	defer func() {
		switch err := recover(); err {
		case nil, skipRun:
		case failedRun:
			r.failed.Store(true)
		default:
			panic(fmt.Errorf("unexpected panic: %v [%T]", err, err))
		}
	}()
	f(r)
}

func (r *runT) Verbose() bool {
	return r.verbose
}
