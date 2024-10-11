// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"fmt"
	"os"
	"runtime"
	"testing"
	"time"
)

func TestMain(m *testing.M) {
	// Add all goroutines running at the start of the test to the set
	// of not-leaked goroutines. This includes TestMain, and anything else
	// that might have been started by test infrastructure.
	skip := [][]byte{
		[]byte("created by os/signal.Notify"),
		[]byte("gotraceback_test.go"),
	}
	buf := make([]byte, 2<<20)
	buf = buf[:runtime.Stack(buf, true)]
	for _, g := range bytes.Split(buf, []byte("\n\n")) {
		id, _, _ := bytes.Cut(g, []byte("["))
		skip = append(skip, id)
	}

	defer os.Exit(m.Run())

	// Look for leaked goroutines.
	//
	// Checking after every test makes it easier to tell which test is the culprit,
	// but checking once at the end is faster and less likely to miss something.
	if runtime.GOOS == "js" {
		// The js-wasm runtime creates an additional background goroutine.
		// Just skip the leak check there.
		return
	}
	start := time.Now()
	warned := false
	for {
		buf := make([]byte, 2<<20)
		buf = buf[:runtime.Stack(buf, true)]
		leaked := false
		for _, g := range bytes.Split(buf, []byte("\n\n")) {
			leaked = true
			for _, s := range skip {
				if bytes.Contains(g, s) {
					leaked = false
					break
				}
			}
		}
		if !leaked {
			break
		}
		if !warned && time.Since(start) > 1*time.Second {
			// Print a warning quickly, in case this is an interactive session.
			// Keep waiting until the test times out, in case this is a slow trybot.
			fmt.Printf("Tests seem to have leaked some goroutines, still waiting.\n\n")
			fmt.Print(string(buf))
			warned = true
		}
		// Goroutines might still be shutting down.
		time.Sleep(1 * time.Millisecond)
	}
}
