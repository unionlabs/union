package log

const ModuleKey = "module"

// Logger defines basic logger functionality that all previous versions of the Logger interface should
// support. Library users should prefer to use this interface when possible, then type case to Logger
// to see if WithContext is supported.
type Logger interface {
	// Info takes a message and a set of key/value pairs and logs with level INFO.
	// The key of the tuple must be a string.
	Info(msg string, keyVals ...any)

	// Warn takes a message and a set of key/value pairs and logs with level WARN.
	// The key of the tuple must be a string.
	Warn(msg string, keyVals ...any)

	// Error takes a message and a set of key/value pairs and logs with level ERR.
	// The key of the tuple must be a string.
	Error(msg string, keyVals ...any)

	// Debug takes a message and a set of key/value pairs and logs with level DEBUG.
	// The key of the tuple must be a string.
	Debug(msg string, keyVals ...any)

	// Impl returns the underlying logger implementation.
	// It is used to access the full functionalities of the underlying logger.
	// Advanced users can type cast the returned value to the actual logger.
	Impl() any
}
