package types

// Logger defines basic logger that store expects.
// It is a subset of the cosmossdk.io/log.Logger interface.
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
}
