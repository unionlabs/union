// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"bytes"
	"os"
	"testing"

	"github.com/rogpeppe/go-internal/testscript"
)

func TestMain(m *testing.M) {
	os.Exit(testscript.RunMain(m, map[string]func() int{
		"txtar-x": main1,
	}))
}

func TestScripts(t *testing.T) {
	testscript.Run(t, testscript.Params{
		Dir: "testdata",
		Cmds: map[string]func(ts *testscript.TestScript, neg bool, args []string){
			"unquote": unquote,
		},
	})
}

func unquote(ts *testscript.TestScript, neg bool, args []string) {
	if neg {
		ts.Fatalf("unsupported: ! unquote")
	}
	for _, arg := range args {
		file := ts.MkAbs(arg)
		data, err := os.ReadFile(file)
		ts.Check(err)
		data = bytes.Replace(data, []byte("\n>"), []byte("\n"), -1)
		data = bytes.TrimPrefix(data, []byte(">"))
		err = os.WriteFile(file, data, 0o666)
		ts.Check(err)
	}
}
