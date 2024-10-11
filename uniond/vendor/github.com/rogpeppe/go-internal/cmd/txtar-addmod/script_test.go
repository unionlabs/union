// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"fmt"
	"os"
	"testing"

	"github.com/rogpeppe/go-internal/goproxytest"
	"github.com/rogpeppe/go-internal/gotooltest"
	"github.com/rogpeppe/go-internal/testscript"
)

var proxyURL string

func TestMain(m *testing.M) {
	os.Exit(testscript.RunMain(gobinMain{m}, map[string]func() int{
		"txtar-addmod": main1,
	}))
}

type gobinMain struct {
	m *testing.M
}

func (m gobinMain) Run() int {
	// Start the Go proxy server running for all tests.
	srv, err := goproxytest.NewServer("testdata/mod", "")
	if err != nil {
		fmt.Fprintf(os.Stderr, "cannot start proxy: %v", err)
		return 1
	}
	proxyURL = srv.URL

	return m.m.Run()
}

func TestScripts(t *testing.T) {
	p := testscript.Params{
		Dir: "testdata",
		Setup: func(e *testscript.Env) error {
			e.Vars = append(e.Vars,
				"GOPROXY="+proxyURL,
				"GONOSUMDB=*",
			)
			return nil
		},
	}
	if err := gotooltest.Setup(&p); err != nil {
		t.Fatal(err)
	}
	testscript.Run(t, p)
}
