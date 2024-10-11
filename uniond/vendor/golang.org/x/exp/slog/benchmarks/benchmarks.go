// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package benchmarks contains benchmarks for slog.
//
// These benchmarks are loosely based on github.com/uber-go/zap/benchmarks.
// They have the following desirable properties:
//
//   - They test a complete log event, from the user's call to its return.
//
//   - The benchmarked code is run concurrently in multiple goroutines, to
//     better simulate a real server (the most common environment for structured
//     logs).
//
//   - Some handlers are optimistic versions of real handlers, doing real-world
//     tasks as fast as possible (and sometimes faster, in that an
//     implementation may not be concurrency-safe). This gives us a lower bound
//     on handler performance, so we can evaluate the (handler-independent) core
//     activity of the package in an end-to-end context without concern that a
//     slow handler implementation is skewing the results.
//
//   - We also test the built-in handlers, for comparison.
//
// As of Go 1.20, fetching the pc for a single nearby frame is slow. We hope to
// improve its speed before this package is released. Run the benchmarks with
//
//	-tags nopc
//
// to remove this cost.
package benchmarks

import (
	"errors"
	"time"

	"golang.org/x/exp/slog"
)

// The symbols in this file are exported so that the Zap benchmarks can use them.

const TestMessage = "Test logging, but use a somewhat realistic message length."

var (
	TestTime     = time.Date(2022, time.May, 1, 0, 0, 0, 0, time.UTC)
	TestString   = "7e3b3b2aaeff56a7108fe11e154200dd/7819479873059528190"
	TestInt      = 32768
	TestDuration = 23 * time.Second
	TestError    = errors.New("fail")
)

var TestAttrs = []slog.Attr{
	slog.String("string", TestString),
	slog.Int("status", TestInt),
	slog.Duration("duration", TestDuration),
	slog.Time("time", TestTime),
	slog.Any("error", TestError),
}

const WantText = "time=1651363200 level=0 msg=Test logging, but use a somewhat realistic message length. string=7e3b3b2aaeff56a7108fe11e154200dd/7819479873059528190 status=32768 duration=23000000000 time=1651363200 error=fail\n"
