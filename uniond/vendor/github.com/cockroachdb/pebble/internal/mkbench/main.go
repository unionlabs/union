// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// mkbench is a utility for processing the raw nightly benchmark data in JSON
// data that can be visualized by docs/js/app.js. The raw data is expected to
// be stored in dated directories underneath the "data/" directory:
//
//	data/YYYYMMDD/.../<file>
//
// The files are expected to be bzip2 compressed. Within each file mkbench
// looks for Go-bench-style lines of the form:
//
//	Benchmark<name> %d %f ops/sec %d read %d write %f r-amp %f w-amp
//
// The output is written to "data.js". In order to avoid reading all of the raw
// data to regenerate "data.js" on every run, mkbench first reads "data.js",
// noting which days have already been processed and exluding files in those
// directories from being read. This has the additional effect of merging the
// existing "data.js" with new raw data, which avoids needing to have all of
// the raw data present to construct a new "data.js" (only the new raw data is
// necessary).
//
// The nightly Pebble benchmarks are orchestrated from the CockroachDB
// repo:
//
//	https://github.com/cockroachdb/cockroach/blob/master/build/teamcity-nightly-pebble.sh
package main

import (
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "mkbench",
	Short: "pebble benchmark data tools",
}

func init() {
	y := getYCSBCommand()
	rootCmd.AddCommand(getYCSBCommand())
	rootCmd.AddCommand(getWriteCommand())
	rootCmd.SilenceUsage = true

	// For backwards compatability, the YCSB command is run, with the same
	// flags, if a subcommand is not specified.
	// TODO(travers): Remove this after updating the call site in the
	// nightly-pebble script in cockroach.
	*rootCmd.Flags() = *y.Flags()
	rootCmd.RunE = y.RunE
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		// Cobra has already printed the error message.
		os.Exit(1)
	}
}
