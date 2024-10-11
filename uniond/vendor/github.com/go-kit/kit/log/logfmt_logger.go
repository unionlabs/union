package log

import (
	"io"

	"github.com/go-kit/log"
)

// NewLogfmtLogger returns a logger that encodes keyvals to the Writer in
// logfmt format. Each log event produces no more than one call to w.Write.
// The passed Writer must be safe for concurrent use by multiple goroutines if
// the returned Logger will be used concurrently.
func NewLogfmtLogger(w io.Writer) Logger {
	return log.NewLogfmtLogger(w)
}
