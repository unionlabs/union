//go:build !windows
// +build !windows

package term

import "io"

// NewColorWriter returns an io.Writer that writes to w and provides cross
// platform support for ANSI color codes. If w is not a terminal it is
// returned unmodified.
func NewColorWriter(w io.Writer) io.Writer {
	return w
}
