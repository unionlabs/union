package log

import (
	"io"

	kitlog "github.com/go-kit/log"
)

// Logger is what any CometBFT library should take.
type Logger interface {
	Debug(msg string, keyvals ...interface{})
	Info(msg string, keyvals ...interface{})
	Error(msg string, keyvals ...interface{})

	With(keyvals ...interface{}) Logger
}

// NewSyncWriter returns a new writer that is safe for concurrent use by
// multiple goroutines. Writes to the returned writer are passed on to w. If
// another write is already in progress, the calling goroutine blocks until
// the writer is available.
//
// If w implements the following interface, so does the returned writer.
//
//	interface {
//	    Fd() uintptr
//	}
func NewSyncWriter(w io.Writer) io.Writer {
	return kitlog.NewSyncWriter(w)
}
