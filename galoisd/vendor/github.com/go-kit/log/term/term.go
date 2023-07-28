// Package term provides tools for logging to a terminal.
package term

import (
	"io"

	"github.com/go-kit/log"
)

// NewLogger returns a Logger that takes advantage of terminal features if
// possible. Log events are formatted by the Logger returned by newLogger. If
// w is a terminal each log event is colored according to the color function.
func NewLogger(w io.Writer, newLogger func(io.Writer) log.Logger, color func(keyvals ...interface{}) FgBgColor) log.Logger {
	if !IsTerminal(w) {
		return newLogger(w)
	}
	return NewColorLogger(NewColorWriter(w), newLogger, color)
}

type fder interface {
	Fd() uintptr
}
