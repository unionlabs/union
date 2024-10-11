package log

import (
	"github.com/go-kit/log"
)

// Logger is the fundamental interface for all log operations. Log creates a
// log event from keyvals, a variadic sequence of alternating keys and values.
// Implementations must be safe for concurrent use by multiple goroutines. In
// particular, any implementation of Logger that appends to keyvals or
// modifies or retains any of its elements must make a copy first.
type Logger = log.Logger

// ErrMissingValue is appended to keyvals slices with odd length to substitute
// the missing value.
var ErrMissingValue = log.ErrMissingValue

// With returns a new contextual logger with keyvals prepended to those passed
// to calls to Log. If logger is also a contextual logger created by With,
// WithPrefix, or WithSuffix, keyvals is appended to the existing context.
//
// The returned Logger replaces all value elements (odd indexes) containing a
// Valuer with their generated value for each call to its Log method.
func With(logger Logger, keyvals ...interface{}) Logger {
	return log.With(logger, keyvals...)
}

// WithPrefix returns a new contextual logger with keyvals prepended to those
// passed to calls to Log. If logger is also a contextual logger created by
// With, WithPrefix, or WithSuffix, keyvals is prepended to the existing context.
//
// The returned Logger replaces all value elements (odd indexes) containing a
// Valuer with their generated value for each call to its Log method.
func WithPrefix(logger Logger, keyvals ...interface{}) Logger {
	return log.WithPrefix(logger, keyvals...)
}

// WithSuffix returns a new contextual logger with keyvals appended to those
// passed to calls to Log. If logger is also a contextual logger created by
// With, WithPrefix, or WithSuffix, keyvals is appended to the existing context.
//
// The returned Logger replaces all value elements (odd indexes) containing a
// Valuer with their generated value for each call to its Log method.
func WithSuffix(logger Logger, keyvals ...interface{}) Logger {
	return log.WithSuffix(logger, keyvals...)
}

// LoggerFunc is an adapter to allow use of ordinary functions as Loggers. If
// f is a function with the appropriate signature, LoggerFunc(f) is a Logger
// object that calls f.
type LoggerFunc = log.LoggerFunc
