package term

import (
	"io"

	"github.com/go-kit/log/term"
)

// NewColorWriter returns an io.Writer that writes to w and provides cross
// platform support for ANSI color codes. If w is not a terminal it is
// returned unmodified.
func NewColorWriter(w io.Writer) io.Writer {
	return term.NewColorWriter(w)
}
