//go:build js || plan9 || aix
// +build js plan9 aix

package termenv

import "io"

// ColorProfile returns the supported color profile:
// ANSI256
func (o Output) ColorProfile() Profile {
	return ANSI256
}

func (o Output) foregroundColor() Color {
	// default gray
	return ANSIColor(7)
}

func (o Output) backgroundColor() Color {
	// default black
	return ANSIColor(0)
}

// EnableVirtualTerminalProcessing enables virtual terminal processing on
// Windows for w and returns a function that restores w to its previous state.
// On non-Windows platforms, or if w does not refer to a terminal, then it
// returns a non-nil no-op function and no error.
func EnableVirtualTerminalProcessing(w io.Writer) (func() error, error) {
	return func() error { return nil }, nil
}
