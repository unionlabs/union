package term

import (
	"io"

	"github.com/go-kit/log"
	"github.com/go-kit/log/term"
)

// Color represents an ANSI color. The zero value is Default.
type Color = term.Color

// ANSI colors.
const (
	Default = term.Default

	Black       = term.Black
	DarkRed     = term.DarkRed
	DarkGreen   = term.DarkGreen
	Brown       = term.Brown
	DarkBlue    = term.DarkBlue
	DarkMagenta = term.DarkMagenta
	DarkCyan    = term.DarkCyan
	Gray        = term.Gray

	DarkGray = term.DarkGray
	Red      = term.Red
	Green    = term.Green
	Yellow   = term.Yellow
	Blue     = term.Blue
	Magenta  = term.Magenta
	Cyan     = term.Cyan
	White    = term.White
)

// FgBgColor represents a foreground and background color.
type FgBgColor = term.FgBgColor

// NewColorLogger returns a Logger which writes colored logs to w. ANSI color
// codes for the colors returned by color are added to the formatted output
// from the Logger returned by newLogger and the combined result written to w.
func NewColorLogger(w io.Writer, newLogger func(io.Writer) log.Logger, color func(keyvals ...interface{}) FgBgColor) log.Logger {
	return term.NewColorLogger(w, newLogger, color)
}
