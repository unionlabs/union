// Package term provides tools for logging to a terminal.
//
// Deprecated: Use github.com/go-kit/log/term instead.
package term

import (
	"io"

	"github.com/go-kit/log"
	"github.com/go-kit/log/term"
)

// NewLogger returns a Logger that takes advantage of terminal features if
// possible. Log events are formatted by the Logger returned by newLogger. If
// w is a terminal each log event is colored according to the color function.
func NewLogger(w io.Writer, newLogger func(io.Writer) log.Logger, color func(keyvals ...interface{}) FgBgColor) log.Logger {
	return term.NewLogger(w, newLogger, color)
}

// IsTerminal returns true if w writes to a terminal.
func IsTerminal(w io.Writer) bool {
	return term.IsTerminal(w)
}
