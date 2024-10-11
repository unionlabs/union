// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"golang.org/x/text/message/pipeline"
)

// TODO:
// - merge information into existing files
// - handle different file formats (PO, XLIFF)
// - handle features (gender, plural)
// - message rewriting

var cmdUpdate = &Command{
	Init:      initUpdate,
	Run:       runUpdate,
	UsageLine: "update <package>* [-out <gofile>]",
	Short:     "merge translations and generate catalog",
}

func initUpdate(cmd *Command) {
	lang = cmd.Flag.String("lang", "en-US", "comma-separated list of languages to process")
	out = cmd.Flag.String("out", "", "output file to write to")
}

func runUpdate(cmd *Command, config *pipeline.Config, args []string) error {
	config.Packages = args
	state, err := pipeline.Extract(config)
	if err != nil {
		return wrap(err, "extract failed")
	}
	if err := state.Import(); err != nil {
		return wrap(err, "import failed")
	}
	if err := state.Merge(); err != nil {
		return wrap(err, "merge failed")
	}
	if err := state.Export(); err != nil {
		return wrap(err, "export failed")
	}
	if *out != "" {
		return wrap(state.Generate(), "generation failed")
	}
	return nil
}
