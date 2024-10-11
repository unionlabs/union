// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"path/filepath"
	"strings"
	"testing"

	"github.com/cockroachdb/pebble/internal/metamorphic/metaflags"
	"github.com/cockroachdb/pebble/metamorphic"
)

// TODO(peter):
//
// Miscellaneous:
// - Add support for different comparers. In particular, allow reverse
//   comparers and a comparer which supports Comparer.Split (by splitting off
//   a variable length suffix).
// - Add support for Writer.LogData

var runOnceFlags, runFlags = metaflags.InitAllFlags()

// TestMeta generates a random set of operations to run, then runs multiple
// instances of the test with varying options. See standardOptions() for the set
// of options that are always run, and randomOptions() for the randomly
// generated options. The number of operations to generate is determined by the
// `--ops` flag. If a failure occurs, the output is kept in `_meta/<test>`,
// though note that a subsequent invocation will overwrite that output. A test
// can be re-run by using the `--run-dir` flag. For example:
//
//	go test -v -run TestMeta --run-dir _meta/standard-017
//
// This will reuse the existing operations present in _meta/ops, rather than
// generating a new set.
//
// The generated operations and options are generated deterministically from a
// pseudorandom number generator seed. If a failure occurs, the seed is
// printed, and the full suite of tests may be re-run using the `--seed` flag:
//
//	go test -v -run TestMeta --seed 1594395154492165000
//
// This will generate a new `_meta/<test>` directory, with the same operations
// and options. This must be run on the same commit SHA as the original
// failure, otherwise changes to the metamorphic tests may cause the generated
// operations and options to differ.
//
// Each instance of the test is run in a different process, by executing the
// same binary (i.e. os.Args[0]) and passing `--run_dir`; the "inner" binary can
// be customized via the --inner-binary flag (used for code coverage
// instrumentation).
func TestMeta(t *testing.T) {
	switch {
	case runOnceFlags.Compare != "":
		runDirs := strings.Split(runOnceFlags.Compare, ",")
		onceOpts := runOnceFlags.MakeRunOnceOptions()
		metamorphic.Compare(t, runOnceFlags.Dir, runOnceFlags.Seed, runDirs, onceOpts...)

	case runOnceFlags.RunDir != "":
		// The --run-dir flag is specified either in the child process (see
		// runOptions() below) or the user specified it manually in order to re-run
		// a test.
		onceOpts := runOnceFlags.MakeRunOnceOptions()
		metamorphic.RunOnce(t, runOnceFlags.RunDir, runOnceFlags.Seed, filepath.Join(runOnceFlags.RunDir, "history"), onceOpts...)

	default:
		opts := runFlags.MakeRunOptions()
		metamorphic.RunAndCompare(t, runFlags.Dir, opts...)
	}
}
