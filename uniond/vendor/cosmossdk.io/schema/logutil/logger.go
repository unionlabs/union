// Package logutil defines the Logger interface expected by indexer implementations.
// It is implemented by cosmossdk.io/log which is not imported to minimize dependencies.
package logutil

// Logger is the logger interface expected by indexer implementations.
type Logger interface {
	// Info takes a message and a set of key/value pairs and logs with level INFO.
	// The key of the tuple must be a string.
	Info(msg string, keyVals ...interface{})

	// Warn takes a message and a set of key/value pairs and logs with level WARN.
	// The key of the tuple must be a string.
	Warn(msg string, keyVals ...interface{})

	// Error takes a message and a set of key/value pairs and logs with level ERR.
	// The key of the tuple must be a string.
	Error(msg string, keyVals ...interface{})

	// Debug takes a message and a set of key/value pairs and logs with level DEBUG.
	// The key of the tuple must be a string.
	Debug(msg string, keyVals ...interface{})
}

// ScopeableLogger is a logger that can be scoped with key/value pairs.
// It is implemented by all the loggers in cosmossdk.io/log.
type ScopeableLogger interface {
	// WithContext returns a new logger with the provided key/value pairs set.
	WithContext(keyVals ...interface{}) interface{}
}

// NoopLogger is a logger that doesn't do anything.
type NoopLogger struct{}

func (n NoopLogger) Info(string, ...interface{}) {}

func (n NoopLogger) Warn(string, ...interface{}) {}

func (n NoopLogger) Error(string, ...interface{}) {}

func (n NoopLogger) Debug(string, ...interface{}) {}

var _ Logger = NoopLogger{}
