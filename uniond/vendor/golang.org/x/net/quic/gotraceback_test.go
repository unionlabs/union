// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21 && unix

package quic

import (
	"os"
	"os/signal"
	"runtime/debug"
	"syscall"
)

// When killed with SIGQUIT (C-\), print stacks with GOTRACEBACK=all rather than system,
// to reduce irrelevant noise when debugging hung tests.
func init() {
	ch := make(chan os.Signal, 1)
	signal.Notify(ch, syscall.SIGQUIT)
	go func() {
		<-ch
		debug.SetTraceback("all")
		panic("SIGQUIT")
	}()
}
