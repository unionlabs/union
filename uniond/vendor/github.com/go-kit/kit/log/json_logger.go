package log

import (
	"io"

	"github.com/go-kit/log"
)

// NewJSONLogger returns a Logger that encodes keyvals to the Writer as a
// single JSON object. Each log event produces no more than one call to
// w.Write. The passed Writer must be safe for concurrent use by multiple
// goroutines if the returned Logger will be used concurrently.
func NewJSONLogger(w io.Writer) Logger {
	return log.NewJSONLogger(w)
}
