package log

import "github.com/go-kit/log"

// NewNopLogger returns a logger that doesn't do anything.
func NewNopLogger() Logger {
	return log.NewNopLogger()
}
