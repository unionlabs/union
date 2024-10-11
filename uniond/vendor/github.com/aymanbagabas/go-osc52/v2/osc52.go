// OSC52 is a terminal escape sequence that allows copying text to the clipboard.
//
// The sequence consists of the following:
//
//	OSC 52 ; Pc ; Pd BEL
//
// Pc is the clipboard choice:
//
//	c: clipboard
//	p: primary
//	q: secondary (not supported)
//	s: select (not supported)
//	0-7: cut-buffers (not supported)
//
// Pd is the data to copy to the clipboard. This string should be encoded in
// base64 (RFC-4648).
//
// If Pd is "?", the terminal replies to the host with the current contents of
// the clipboard.
//
// If Pd is neither a base64 string nor "?", the terminal clears the clipboard.
//
// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands
// where Ps = 52 => Manipulate Selection Data.
//
// Examples:
//
//	// copy "hello world" to the system clipboard
//	fmt.Fprint(os.Stderr, osc52.New("hello world"))
//
//	// copy "hello world" to the primary Clipboard
//	fmt.Fprint(os.Stderr, osc52.New("hello world").Primary())
//
//	// limit the size of the string to copy 10 bytes
//	fmt.Fprint(os.Stderr, osc52.New("0123456789").Limit(10))
//
//	// escape the OSC52 sequence for screen using DCS sequences
//	fmt.Fprint(os.Stderr, osc52.New("hello world").Screen())
//
//	// escape the OSC52 sequence for Tmux
//	fmt.Fprint(os.Stderr, osc52.New("hello world").Tmux())
//
//	// query the system Clipboard
//	fmt.Fprint(os.Stderr, osc52.Query())
//
//	// query the primary clipboard
//	fmt.Fprint(os.Stderr, osc52.Query().Primary())
//
//	// clear the system Clipboard
//	fmt.Fprint(os.Stderr, osc52.Clear())
//
//	// clear the primary Clipboard
//	fmt.Fprint(os.Stderr, osc52.Clear().Primary())
package osc52

import (
	"encoding/base64"
	"fmt"
	"io"
	"strings"
)

// Clipboard is the clipboard buffer to use.
type Clipboard rune

const (
	// SystemClipboard is the system clipboard buffer.
	SystemClipboard Clipboard = 'c'
	// PrimaryClipboard is the primary clipboard buffer (X11).
	PrimaryClipboard = 'p'
)

// Mode is the mode to use for the OSC52 sequence.
type Mode uint

const (
	// DefaultMode is the default OSC52 sequence mode.
	DefaultMode Mode = iota
	// ScreenMode escapes the OSC52 sequence for screen using DCS sequences.
	ScreenMode
	// TmuxMode escapes the OSC52 sequence for tmux. Not needed if tmux
	// clipboard is set to `set-clipboard on`
	TmuxMode
)

// Operation is the OSC52 operation.
type Operation uint

const (
	// SetOperation is the copy operation.
	SetOperation Operation = iota
	// QueryOperation is the query operation.
	QueryOperation
	// ClearOperation is the clear operation.
	ClearOperation
)

// Sequence is the OSC52 sequence.
type Sequence struct {
	str       string
	limit     int
	op        Operation
	mode      Mode
	clipboard Clipboard
}

var _ fmt.Stringer = Sequence{}

var _ io.WriterTo = Sequence{}

// String returns the OSC52 sequence.
func (s Sequence) String() string {
	var seq strings.Builder
	// mode escape sequences start
	seq.WriteString(s.seqStart())
	// actual OSC52 sequence start
	seq.WriteString(fmt.Sprintf("\x1b]52;%c;", s.clipboard))
	switch s.op {
	case SetOperation:
		str := s.str
		if s.limit > 0 && len(str) > s.limit {
			return ""
		}
		b64 := base64.StdEncoding.EncodeToString([]byte(str))
		switch s.mode {
		case ScreenMode:
			// Screen doesn't support OSC52 but will pass the contents of a DCS
			// sequence to the outer terminal unchanged.
			//
			// Here, we split the encoded string into 76 bytes chunks and then
			// join the chunks with <end-dsc><start-dsc> sequences. Finally,
			// wrap the whole thing in
			// <start-dsc><start-osc52><joined-chunks><end-osc52><end-dsc>.
			// s := strings.SplitN(b64, "", 76)
			s := make([]string, 0, len(b64)/76+1)
			for i := 0; i < len(b64); i += 76 {
				end := i + 76
				if end > len(b64) {
					end = len(b64)
				}
				s = append(s, b64[i:end])
			}
			seq.WriteString(strings.Join(s, "\x1b\\\x1bP"))
		default:
			seq.WriteString(b64)
		}
	case QueryOperation:
		// OSC52 queries the clipboard using "?"
		seq.WriteString("?")
	case ClearOperation:
		// OSC52 clears the clipboard if the data is neither a base64 string nor "?"
		// we're using "!" as a default
		seq.WriteString("!")
	}
	// actual OSC52 sequence end
	seq.WriteString("\x07")
	// mode escape end
	seq.WriteString(s.seqEnd())
	return seq.String()
}

// WriteTo writes the OSC52 sequence to the writer.
func (s Sequence) WriteTo(out io.Writer) (int64, error) {
	n, err := out.Write([]byte(s.String()))
	return int64(n), err
}

// Mode sets the mode for the OSC52 sequence.
func (s Sequence) Mode(m Mode) Sequence {
	s.mode = m
	return s
}

// Tmux sets the mode to TmuxMode.
// Used to escape the OSC52 sequence for `tmux`.
//
// Note: this is not needed if tmux clipboard is set to `set-clipboard on`. If
// TmuxMode is used, tmux must have `allow-passthrough on` set.
//
// This is a syntactic sugar for s.Mode(TmuxMode).
func (s Sequence) Tmux() Sequence {
	return s.Mode(TmuxMode)
}

// Screen sets the mode to ScreenMode.
// Used to escape the OSC52 sequence for `screen`.
//
// This is a syntactic sugar for s.Mode(ScreenMode).
func (s Sequence) Screen() Sequence {
	return s.Mode(ScreenMode)
}

// Clipboard sets the clipboard buffer for the OSC52 sequence.
func (s Sequence) Clipboard(c Clipboard) Sequence {
	s.clipboard = c
	return s
}

// Primary sets the clipboard buffer to PrimaryClipboard.
// This is the X11 primary clipboard.
//
// This is a syntactic sugar for s.Clipboard(PrimaryClipboard).
func (s Sequence) Primary() Sequence {
	return s.Clipboard(PrimaryClipboard)
}

// Limit sets the limit for the OSC52 sequence.
// The default limit is 0 (no limit).
//
// Strings longer than the limit get ignored. Settting the limit to 0 or a
// negative value disables the limit. Each terminal defines its own escapse
// sequence limit.
func (s Sequence) Limit(l int) Sequence {
	if l < 0 {
		s.limit = 0
	} else {
		s.limit = l
	}
	return s
}

// Operation sets the operation for the OSC52 sequence.
// The default operation is SetOperation.
func (s Sequence) Operation(o Operation) Sequence {
	s.op = o
	return s
}

// Clear sets the operation to ClearOperation.
// This clears the clipboard.
//
// This is a syntactic sugar for s.Operation(ClearOperation).
func (s Sequence) Clear() Sequence {
	return s.Operation(ClearOperation)
}

// Query sets the operation to QueryOperation.
// This queries the clipboard contents.
//
// This is a syntactic sugar for s.Operation(QueryOperation).
func (s Sequence) Query() Sequence {
	return s.Operation(QueryOperation)
}

// SetString sets the string for the OSC52 sequence. Strings are joined with a
// space character.
func (s Sequence) SetString(strs ...string) Sequence {
	s.str = strings.Join(strs, " ")
	return s
}

// New creates a new OSC52 sequence with the given string(s). Strings are
// joined with a space character.
func New(strs ...string) Sequence {
	s := Sequence{
		str:       strings.Join(strs, " "),
		limit:     0,
		mode:      DefaultMode,
		clipboard: SystemClipboard,
		op:        SetOperation,
	}
	return s
}

// Query creates a new OSC52 sequence with the QueryOperation.
// This returns a new OSC52 sequence to query the clipboard contents.
//
// This is a syntactic sugar for New().Query().
func Query() Sequence {
	return New().Query()
}

// Clear creates a new OSC52 sequence with the ClearOperation.
// This returns a new OSC52 sequence to clear the clipboard.
//
// This is a syntactic sugar for New().Clear().
func Clear() Sequence {
	return New().Clear()
}

func (s Sequence) seqStart() string {
	switch s.mode {
	case TmuxMode:
		// Write the start of a tmux escape sequence.
		return "\x1bPtmux;\x1b"
	case ScreenMode:
		// Write the start of a DCS sequence.
		return "\x1bP"
	default:
		return ""
	}
}

func (s Sequence) seqEnd() string {
	switch s.mode {
	case TmuxMode:
		// Terminate the tmux escape sequence.
		return "\x1b\\"
	case ScreenMode:
		// Write the end of a DCS sequence.
		return "\x1b\x5c"
	default:
		return ""
	}
}
