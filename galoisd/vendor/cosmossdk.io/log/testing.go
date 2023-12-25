package log

import "github.com/rs/zerolog"

// TestingT is the interface required for logging in tests.
// It is a subset of testing.T to avoid a direct dependency on the testing package.
type TestingT zerolog.TestingLog

// NewTestLogger returns a logger that calls t.Log to write entries.
//
// The returned logger emits messages at any level.
// For active debugging of a test with verbose logs,
// the [NewTestLoggerInfo] and [NewTestLoggerError] functions
// only emit messages at or above the corresponding log levels.
//
// If the logs may help debug a test failure,
// you may want to use NewTestLogger(t) in your test.
// Otherwise, use NewNopLogger().
func NewTestLogger(t TestingT) Logger {
	return newTestLogger(t, zerolog.DebugLevel)
}

// NewTestLoggerInfo returns a test logger that filters out messages
// below info level.
//
// This is primarily helpful during active debugging of a test
// with verbose logs.
func NewTestLoggerInfo(t TestingT) Logger {
	return newTestLogger(t, zerolog.InfoLevel)
}

// NewTestLoggerError returns a test logger that filters out messages
// below Error level.
//
// This is primarily helpful during active debugging of a test
// with verbose logs.
func NewTestLoggerError(t TestingT) Logger {
	return newTestLogger(t, zerolog.ErrorLevel)
}

func newTestLogger(t TestingT, lvl zerolog.Level) Logger {
	cw := zerolog.NewConsoleWriter()
	cw.Out = zerolog.TestWriter{
		T: t,
		// Normally one would use zerolog.ConsoleTestWriter
		// to set the option on NewConsoleWriter,
		// but the zerolog source for that is hardcoded to Frame=6.
		// With Frame=6, all source locations are printed as "logger.go",
		// but Frame=7 prints correct source locations.
		Frame: 7,
	}
	return NewCustomLogger(zerolog.New(cw).Level(lvl))
}
