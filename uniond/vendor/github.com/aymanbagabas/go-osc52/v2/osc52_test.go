package osc52

import (
	"bytes"
	"testing"
)

func TestCopy(t *testing.T) {
	cases := []struct {
		name      string
		str       string
		clipboard Clipboard
		mode      Mode
		limit     int
		expected  string
	}{
		{
			name:      "hello world",
			str:       "hello world",
			clipboard: SystemClipboard,
			mode:      DefaultMode,
			limit:     0,
			expected:  "\x1b]52;c;aGVsbG8gd29ybGQ=\x07",
		},
		{
			name:      "empty string",
			str:       "",
			clipboard: SystemClipboard,
			mode:      DefaultMode,
			limit:     0,
			expected:  "\x1b]52;c;\x07",
		},
		{
			name:      "hello world primary",
			str:       "hello world",
			clipboard: PrimaryClipboard,
			mode:      DefaultMode,
			limit:     0,
			expected:  "\x1b]52;p;aGVsbG8gd29ybGQ=\x07",
		},
		{
			name:      "hello world tmux mode",
			str:       "hello world",
			clipboard: SystemClipboard,
			mode:      TmuxMode,
			limit:     0,
			expected:  "\x1bPtmux;\x1b\x1b]52;c;aGVsbG8gd29ybGQ=\x07\x1b\\",
		},
		{
			name:      "hello world screen mode",
			str:       "hello world",
			clipboard: SystemClipboard,
			mode:      ScreenMode,
			limit:     0,
			expected:  "\x1bP\x1b]52;c;aGVsbG8gd29ybGQ=\x07\x1b\\",
		},
		{
			name:      "hello world screen mode longer than 76 bytes string",
			str:       "hello world hello world hello world hello world hello world hello world hello world hello world",
			clipboard: SystemClipboard,
			mode:      ScreenMode,
			limit:     0,
			expected:  "\x1bP\x1b]52;c;aGVsbG8gd29ybGQgaGVsbG8gd29ybGQgaGVsbG8gd29ybGQgaGVsbG8gd29ybGQgaGVsbG8gd29y\x1b\\\x1bPbGQgaGVsbG8gd29ybGQgaGVsbG8gd29ybGQgaGVsbG8gd29ybGQ=\a\x1b\\",
		},
		{
			name:      "hello world with limit 11",
			str:       "hello world",
			clipboard: SystemClipboard,
			mode:      DefaultMode,
			limit:     11,
			expected:  "\x1b]52;c;aGVsbG8gd29ybGQ=\x07",
		},
		{
			name:      "hello world with limit 10",
			str:       "hello world",
			clipboard: SystemClipboard,
			mode:      DefaultMode,
			limit:     10,
			expected:  "",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			s := New(c.str)
			s = s.Clipboard(c.clipboard)
			s = s.Mode(c.mode)
			s = s.Limit(c.limit)
			if s.String() != c.expected {
				t.Errorf("expected %q, got %q", c.expected, s.String())
			}
		})
	}
}

func TestQuery(t *testing.T) {
	cases := []struct {
		name      string
		mode      Mode
		clipboard Clipboard
		expected  string
	}{
		{
			name:      "query system clipboard",
			mode:      DefaultMode,
			clipboard: SystemClipboard,
			expected:  "\x1b]52;c;?\x07",
		},
		{
			name:      "query primary clipboard",
			mode:      DefaultMode,
			clipboard: PrimaryClipboard,
			expected:  "\x1b]52;p;?\x07",
		},
		{
			name:      "query system clipboard tmux mode",
			mode:      TmuxMode,
			clipboard: SystemClipboard,
			expected:  "\x1bPtmux;\x1b\x1b]52;c;?\x07\x1b\\",
		},
		{
			name:      "query system clipboard screen mode",
			mode:      ScreenMode,
			clipboard: SystemClipboard,
			expected:  "\x1bP\x1b]52;c;?\x07\x1b\\",
		},
		{
			name:      "query primary clipboard tmux mode",
			mode:      TmuxMode,
			clipboard: PrimaryClipboard,
			expected:  "\x1bPtmux;\x1b\x1b]52;p;?\x07\x1b\\",
		},
		{
			name:      "query primary clipboard screen mode",
			mode:      ScreenMode,
			clipboard: PrimaryClipboard,
			expected:  "\x1bP\x1b]52;p;?\x07\x1b\\",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			s := New().Query().Clipboard(c.clipboard).Mode(c.mode)
			if s.String() != c.expected {
				t.Errorf("expected %q, got %q", c.expected, s.Query())
			}
		})
	}
}

func TestClear(t *testing.T) {
	cases := []struct {
		name      string
		mode      Mode
		clipboard Clipboard
		expected  string
	}{
		{
			name:      "clear system clipboard",
			mode:      DefaultMode,
			clipboard: SystemClipboard,
			expected:  "\x1b]52;c;!\x07",
		},
		{
			name:      "clear system clipboard tmux mode",
			mode:      TmuxMode,
			clipboard: SystemClipboard,
			expected:  "\x1bPtmux;\x1b\x1b]52;c;!\x07\x1b\\",
		},
		{
			name:      "clear system clipboard screen mode",
			mode:      ScreenMode,
			clipboard: SystemClipboard,
			expected:  "\x1bP\x1b]52;c;!\x07\x1b\\",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			s := New().Clear().Clipboard(c.clipboard).Mode(c.mode)
			if s.String() != c.expected {
				t.Errorf("expected %q, got %q", c.expected, s.Clear())
			}
		})
	}
}

func TestWriteTo(t *testing.T) {
	var buf bytes.Buffer
	cases := []struct {
		name      string
		str       string
		clipboard Clipboard
		mode      Mode
		limit     int
		expected  string
	}{
		{
			name:      "hello world",
			str:       "hello world",
			clipboard: SystemClipboard,
			mode:      DefaultMode,
			limit:     0,
			expected:  "\x1b]52;c;aGVsbG8gd29ybGQ=\x07",
		},
		{
			name:      "empty string",
			str:       "",
			clipboard: SystemClipboard,
			mode:      DefaultMode,
			limit:     0,
			expected:  "\x1b]52;c;\x07",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			buf.Reset()
			s := New(c.str)
			s.Clipboard(c.clipboard)
			s.Mode(c.mode)
			s.Limit(c.limit)
			if _, err := s.WriteTo(&buf); err != nil {
				t.Errorf("expected nil, got %v", err)
			}
			if buf.String() != c.expected {
				t.Errorf("expected %q, got %q", c.expected, buf.String())
			}
		})
	}
}
