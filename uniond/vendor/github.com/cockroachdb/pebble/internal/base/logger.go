// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"bytes"
	"context"
	"fmt"
	"log"
	"os"
	"runtime"
	"sync"

	"github.com/cockroachdb/pebble/internal/invariants"
)

// Logger defines an interface for writing log messages.
type Logger interface {
	Infof(format string, args ...interface{})
	Fatalf(format string, args ...interface{})
}
type defaultLogger struct{}

// DefaultLogger logs to the Go stdlib logs.
var DefaultLogger defaultLogger

var _ Logger = DefaultLogger

// Infof implements the Logger.Infof interface.
func (defaultLogger) Infof(format string, args ...interface{}) {
	_ = log.Output(2, fmt.Sprintf(format, args...))
}

// Fatalf implements the Logger.Fatalf interface.
func (defaultLogger) Fatalf(format string, args ...interface{}) {
	_ = log.Output(2, fmt.Sprintf(format, args...))
	os.Exit(1)
}

// InMemLogger implements Logger using an in-memory buffer (used for testing).
// The buffer can be read via String() and cleared via Reset().
type InMemLogger struct {
	mu struct {
		sync.Mutex
		buf bytes.Buffer
	}
}

var _ Logger = (*InMemLogger)(nil)

// Reset clears the internal buffer.
func (b *InMemLogger) Reset() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.mu.buf.Reset()
}

// String returns the current internal buffer.
func (b *InMemLogger) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.mu.buf.String()
}

// Infof is part of the Logger interface.
func (b *InMemLogger) Infof(format string, args ...interface{}) {
	s := fmt.Sprintf(format, args...)
	b.mu.Lock()
	defer b.mu.Unlock()
	b.mu.buf.Write([]byte(s))
	if n := len(s); n == 0 || s[n-1] != '\n' {
		b.mu.buf.Write([]byte("\n"))
	}
}

// Fatalf is part of the Logger interface.
func (b *InMemLogger) Fatalf(format string, args ...interface{}) {
	b.Infof(format, args...)
	runtime.Goexit()
}

// LoggerAndTracer defines an interface for logging and tracing.
type LoggerAndTracer interface {
	Logger
	// Eventf formats and emits a tracing log, if tracing is enabled in the
	// current context.
	Eventf(ctx context.Context, format string, args ...interface{})
	// IsTracingEnabled returns true if tracing is enabled. It can be used as an
	// optimization to avoid calling Eventf (which will be a noop when tracing
	// is not enabled) to avoid the overhead of boxing the args.
	IsTracingEnabled(ctx context.Context) bool
}

// LoggerWithNoopTracer wraps a logger and does no tracing.
type LoggerWithNoopTracer struct {
	Logger
}

var _ LoggerAndTracer = &LoggerWithNoopTracer{}

// Eventf implements LoggerAndTracer.
func (*LoggerWithNoopTracer) Eventf(ctx context.Context, format string, args ...interface{}) {
	if invariants.Enabled && ctx == nil {
		panic("Eventf context is nil")
	}
}

// IsTracingEnabled implements LoggerAndTracer.
func (*LoggerWithNoopTracer) IsTracingEnabled(ctx context.Context) bool {
	if invariants.Enabled && ctx == nil {
		panic("IsTracingEnabled ctx is nil")
	}
	return false
}

// NoopLoggerAndTracer does no logging and tracing. Remember that struct{} is
// special cased in Go and does not incur an allocation when it backs the
// interface LoggerAndTracer.
type NoopLoggerAndTracer struct{}

var _ LoggerAndTracer = NoopLoggerAndTracer{}

// Infof implements LoggerAndTracer.
func (l NoopLoggerAndTracer) Infof(format string, args ...interface{}) {}

// Fatalf implements LoggerAndTracer.
func (l NoopLoggerAndTracer) Fatalf(format string, args ...interface{}) {}

// Eventf implements LoggerAndTracer.
func (l NoopLoggerAndTracer) Eventf(ctx context.Context, format string, args ...interface{}) {
	if invariants.Enabled && ctx == nil {
		panic("Eventf context is nil")
	}
}

// IsTracingEnabled implements LoggerAndTracer.
func (l NoopLoggerAndTracer) IsTracingEnabled(ctx context.Context) bool {
	if invariants.Enabled && ctx == nil {
		panic("IsTracingEnabled ctx is nil")
	}
	return false
}
