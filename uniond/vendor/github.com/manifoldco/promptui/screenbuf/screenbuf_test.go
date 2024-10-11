package screenbuf

import (
	"bytes"
	"testing"
)

func TestScreen(t *testing.T) {
	// overwrite regular movement codes for easier visualization
	clearLine = []byte("\\c")
	moveUp = []byte("\\u")
	moveDown = []byte("\\d")

	var buf bytes.Buffer
	s := New(&buf)

	tcs := []struct {
		scenario string
		lines    []string
		expect   string
		cursor   int
		height   int
		flush    bool
		reset    bool
		clear    bool
	}{
		{
			scenario: "initial write",
			lines:    []string{"Line One"},
			expect:   "\\cLine One\n",
			cursor:   1,
			height:   1,
		},
		{
			scenario: "write of with same number of lines",
			lines:    []string{"Line One"},
			expect:   "\\u\\cLine One\\d",
			cursor:   1,
			height:   1,
		},
		{
			scenario: "write of with more lines",
			lines:    []string{"Line One", "Line Two"},
			expect:   "\\u\\cLine One\\d\\cLine Two\n",
			cursor:   2,
			height:   2,
		},
		{
			scenario: "write of with fewer lines",
			lines:    []string{"line One"},
			expect:   "\\u\\u\\cline One\\d\\c\\d",
			cursor:   1,
			height:   2,
		},
		{
			scenario: "write of way more lines",
			lines:    []string{"line one", "line two", "line three", "line four", "line five"},
			expect:   "\\u\\u\\cline one\\d\\cline two\\d\\cline three\n\\cline four\n\\cline five\n",
			cursor:   5,
			height:   5,
		},
		{
			scenario: "write of way less lines",
			lines:    []string{"line one", "line two"},
			expect:   "\\u\\u\\u\\u\\u\\cline one\\d\\cline two\\d\\c\\d\\c\\d\\c\\d",
			cursor:   2,
			height:   5,
		},
		{
			scenario: "write of way more lines",
			lines:    []string{"line one", "line two", "line three", "line four", "line five"},
			expect:   "\\u\\u\\u\\u\\u\\cline one\\d\\cline two\\d\\cline three\\d\\cline four\\d\\cline five\\d",
			cursor:   5,
			height:   5,
		},
		{
			scenario: "reset and write",
			lines:    []string{"line one", "line two"},
			expect:   "\\u\\c\\u\\c\\u\\c\\u\\c\\u\\c\\cline one\n\\cline two\n",
			cursor:   2,
			height:   2,
			reset:    true,
		},
		{
			scenario: "clear all previous lines",
			lines:    []string{"line one", "line two"},
			expect:   "\\u\\u\\cline one\\d\\cline two\\d\\u\\c\\u\\c",
			cursor:   0,
			height:   0,
			clear:    true,
		},
	}

	for _, tc := range tcs {
		t.Run(tc.scenario, func(t *testing.T) {
			buf.Reset()
			if tc.reset {
				s.Reset()
			}

			for _, line := range tc.lines {
				_, err := s.WriteString(line)
				if err != nil {
					t.Fatalf("expected no error, got %v", err)
				}
			}

			if tc.clear {
				if err := s.Clear(); err != nil {
					t.Errorf("expected no error, got %d", err)
				}
			}

			if tc.cursor != s.cursor {
				t.Errorf("expected cursor %d, got %d", tc.cursor, s.cursor)
			}

			err := s.Flush()
			if err != nil {
				t.Fatalf("expected no error, got %v", err)
			}

			got := buf.String()

			if tc.expect != got {
				t.Errorf("expected %q, got %q", tc.expect, got)
			}

			if tc.height != s.height {
				t.Errorf("expected height %d, got %d", tc.height, s.height)
			}
		})
	}
}
