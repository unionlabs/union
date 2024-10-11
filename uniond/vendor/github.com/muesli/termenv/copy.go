package termenv

import (
	"strings"

	"github.com/aymanbagabas/go-osc52/v2"
)

// Copy copies text to clipboard using OSC 52 escape sequence.
func (o Output) Copy(str string) {
	s := osc52.New(str)
	if strings.HasPrefix(o.environ.Getenv("TERM"), "screen") {
		s = s.Screen()
	}
	_, _ = s.WriteTo(o)
}

// CopyPrimary copies text to primary clipboard (X11) using OSC 52 escape
// sequence.
func (o Output) CopyPrimary(str string) {
	s := osc52.New(str).Primary()
	if strings.HasPrefix(o.environ.Getenv("TERM"), "screen") {
		s = s.Screen()
	}
	_, _ = s.WriteTo(o)
}

// Copy copies text to clipboard using OSC 52 escape sequence.
func Copy(str string) {
	output.Copy(str)
}

// CopyPrimary copies text to primary clipboard (X11) using OSC 52 escape
// sequence.
func CopyPrimary(str string) {
	output.CopyPrimary(str)
}
