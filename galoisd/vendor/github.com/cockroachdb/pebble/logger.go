// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import "github.com/cockroachdb/pebble/internal/base"

// Logger defines an interface for writing log messages.
type Logger = base.Logger

// DefaultLogger logs to the Go stdlib logs.
var DefaultLogger = base.DefaultLogger

// LoggerAndTracer defines an interface for logging and tracing.
type LoggerAndTracer = base.LoggerAndTracer
