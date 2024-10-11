// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package metaflags defines command-line flags for the metamorphic tests and
// provides functionality to construct the respective
// metamorphic.RunOptions/RunOnceOptions.
package metaflags

import (
	"flag"
	"fmt"
	"math"
	"regexp"

	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/cockroachdb/pebble/metamorphic"
)

// CommonFlags contains flags that apply to both metamorphic.Run and
// metamorphic.RunOnce/Compare.
type CommonFlags struct {
	// Dir is the directory storing test state. See "dir" flag below.
	Dir string
	// Seed for generation of random operations. See "seed" flag below.
	Seed uint64
	// ErrorRate is the rate of injected filesystem errors. See "error-rate" flag
	// below.
	ErrorRate float64
	// FailRE causes the test to fail if the output matches this regex. See "fail"
	// flag below.
	FailRE string
	// Keep determines if the DB directory is kept on successful runs. See "keep"
	// flag below.
	Keep bool
	// MaxThreads used by a single run. See "max-threads" flag below.
	MaxThreads int
}

func initCommonFlags() *CommonFlags {
	c := &CommonFlags{}
	flag.StringVar(&c.Dir, "dir", "_meta",
		"the directory storing test state")

	flag.Uint64Var(&c.Seed, "seed", 0,
		"a pseudorandom number generator seed")

	// TODO: default error rate to a non-zero value. Currently, retrying is
	// non-deterministic because of the Ierator.*WithLimit() methods since
	// they may say that the Iterator is not valid, but be positioned at a
	// certain key that can be returned in the future if the limit is changed.
	// Since that key is hidden from clients of Iterator, the retryableIter
	// using SeekGE will not necessarily position the Iterator that saw an
	// injected error at the same place as an Iterator that did not see that
	// error.
	flag.Float64Var(&c.ErrorRate, "error-rate", 0.0,
		"rate of errors injected into filesystem operations (0 ≤ r < 1)")

	flag.StringVar(&c.FailRE, "fail", "",
		"fail the test if the supplied regular expression matches the output")

	flag.BoolVar(&c.Keep, "keep", false,
		"keep the DB directory even on successful runs")

	flag.IntVar(&c.MaxThreads, "max-threads", math.MaxInt,
		"limit execution of a single run to the provided number of threads; must be ≥ 1")

	return c
}

// RunOnceFlags contains flags that apply only to metamorphic.RunOnce/Compare.
type RunOnceFlags struct {
	*CommonFlags
	// RunDir applies to metamorphic.RunOnce and contains the specific
	// configuration of the run. See "run-dir" flag below.
	RunDir string
	// Compare applies to metamorphic.Compare. See "compare" flag below.
	Compare string
}

func initRunOnceFlags(c *CommonFlags) *RunOnceFlags {
	ro := &RunOnceFlags{CommonFlags: c}
	flag.StringVar(&ro.RunDir, "run-dir", "",
		"the specific configuration to (re-)run (used for post-mortem debugging)")

	flag.StringVar(&ro.Compare, "compare", "",
		`comma separated list of options files to compare. The result of each run is compared with
the result of the run from the first options file in the list. Example, -compare
random-003,standard-000. The dir flag should have the directory containing these directories.
Example, -dir _meta/200610-203012.077`)
	return ro
}

// RunFlags contains flags that apply only to metamorphic.Run.
type RunFlags struct {
	*CommonFlags
	// FS controls the type of filesystems to use. See "fs" flag below.
	FS string
	// TraceFile for execution tracing. See "trace-file" flag below.
	TraceFile string
	// Ops describes how the total number of operations is generated. See "ops" flags below.
	Ops randvar.Flag
	// InnerBinary is the binary to invoke for a single run. See "inner-binary"
	// flag below.
	InnerBinary string
	// PreviousOps is the path to the ops file of a previous run. See the
	// "previous-ops" flag below.
	PreviousOps string
	// InitialStatePath is the path to a database data directory from a previous
	// run. See the "initial-state" flag below.
	InitialStatePath string
	// InitialStateDesc is a human-readable description of the initial database
	// state. See "initial-state-desc" flag below.
	InitialStateDesc string
}

func initRunFlags(c *CommonFlags) *RunFlags {
	r := &RunFlags{CommonFlags: c}
	flag.StringVar(&r.FS, "fs", "rand",
		`force the tests to use either memory or disk-backed filesystems (valid: "mem", "disk", "rand")`)

	flag.StringVar(&r.TraceFile, "trace-file", "",
		"write an execution trace to `<run-dir>/file`")

	if err := r.Ops.Set("uniform:5000-10000"); err != nil {
		panic(err)
	}
	flag.Var(&r.Ops, "ops", "uniform:5000-10000")

	flag.StringVar(&r.InnerBinary, "inner-binary", "",
		`binary to run for each instance of the test (this same binary by default); cannot be used
with --run-dir or --compare`)

	// The following options may be used for split-version metamorphic testing.
	// To perform split-version testing, the client runs the metamorphic tests
	// on an earlier Pebble SHA passing the `--keep` flag. The client then
	// switches to the later Pebble SHA, setting the below options to point to
	// the `ops` file and one of the previous run's data directories.

	flag.StringVar(&r.PreviousOps, "previous-ops", "",
		"path to an ops file, used to prepopulate the set of keys operations draw from")

	flag.StringVar(&r.InitialStatePath, "initial-state", "",
		"path to a database's data directory, used to prepopulate the test run's databases")

	flag.StringVar(&r.InitialStateDesc, "initial-state-desc", "",
		`a human-readable description of the initial database state.
		If set this parameter is written to the OPTIONS to aid in
		debugging. It's intended to describe the lineage of a
		database's state, including sufficient information for
		reproduction (eg, SHA, prng seed, etc).`)
	return r
}

// InitRunOnceFlags initializes the flags that are used for a single run of the
// metamorphic test.
func InitRunOnceFlags() *RunOnceFlags {
	return initRunOnceFlags(initCommonFlags())
}

// InitAllFlags initializes all metamorphic test flags: those used for a
// single run, and those used for a "top level" run.
func InitAllFlags() (*RunOnceFlags, *RunFlags) {
	c := initCommonFlags()
	return initRunOnceFlags(c), initRunFlags(c)
}

// MakeRunOnceOptions constructs RunOnceOptions based on the flags.
func (ro *RunOnceFlags) MakeRunOnceOptions() []metamorphic.RunOnceOption {
	onceOpts := []metamorphic.RunOnceOption{
		metamorphic.MaxThreads(ro.MaxThreads),
	}
	if ro.Keep {
		onceOpts = append(onceOpts, metamorphic.KeepData{})
	}
	if ro.FailRE != "" {
		onceOpts = append(onceOpts, metamorphic.FailOnMatch{Regexp: regexp.MustCompile(ro.FailRE)})
	}
	if ro.ErrorRate > 0 {
		onceOpts = append(onceOpts, metamorphic.InjectErrorsRate(ro.ErrorRate))
	}
	return onceOpts
}

// MakeRunOptions constructs RunOptions based on the flags.
func (r *RunFlags) MakeRunOptions() []metamorphic.RunOption {
	opts := []metamorphic.RunOption{
		metamorphic.Seed(r.Seed),
		metamorphic.OpCount(r.Ops.Static),
		metamorphic.MaxThreads(r.MaxThreads),
	}
	if r.Keep {
		opts = append(opts, metamorphic.KeepData{})
	}
	if r.FailRE != "" {
		opts = append(opts, metamorphic.FailOnMatch{Regexp: regexp.MustCompile(r.FailRE)})
	}
	if r.ErrorRate > 0 {
		opts = append(opts, metamorphic.InjectErrorsRate(r.ErrorRate))
	}
	if r.TraceFile != "" {
		opts = append(opts, metamorphic.RuntimeTrace(r.TraceFile))
	}
	if r.PreviousOps != "" {
		opts = append(opts, metamorphic.ExtendPreviousRun(r.PreviousOps, r.InitialStatePath, r.InitialStateDesc))
	}

	// If the filesystem type was forced, all tests will use that value.
	switch r.FS {
	case "", "rand", "default":
		// No-op. Use the generated value for the filesystem.
	case "disk":
		opts = append(opts, metamorphic.UseDisk)
	case "mem":
		opts = append(opts, metamorphic.UseInMemory)
	default:
		panic(fmt.Sprintf("unknown forced filesystem type: %q", r.FS))
	}
	if r.InnerBinary != "" {
		opts = append(opts, metamorphic.InnerBinary(r.InnerBinary))
	}
	return opts
}
