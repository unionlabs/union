// Package format provides utilities for formatting diffs and messages.
package format

import (
	"bytes"
	"fmt"
	"strings"
	"unicode"

	"gotest.tools/v3/internal/difflib"
)

const (
	contextLines = 2
)

// DiffConfig for a unified diff
type DiffConfig struct {
	A    string
	B    string
	From string
	To   string
}

// UnifiedDiff is a modified version of difflib.WriteUnifiedDiff with better
// support for showing the whitespace differences.
func UnifiedDiff(conf DiffConfig) string {
	a := strings.SplitAfter(conf.A, "\n")
	b := strings.SplitAfter(conf.B, "\n")
	groups := difflib.NewMatcher(a, b).GetGroupedOpCodes(contextLines)
	if len(groups) == 0 {
		return ""
	}

	buf := new(bytes.Buffer)
	writeFormat := func(format string, args ...interface{}) {
		buf.WriteString(fmt.Sprintf(format, args...))
	}
	writeLine := func(prefix string, s string) {
		buf.WriteString(prefix + s)
	}
	if hasWhitespaceDiffLines(groups, a, b) {
		writeLine = visibleWhitespaceLine(writeLine)
	}
	formatHeader(writeFormat, conf)
	for _, group := range groups {
		formatRangeLine(writeFormat, group)
		for _, opCode := range group {
			in, out := a[opCode.I1:opCode.I2], b[opCode.J1:opCode.J2]
			switch opCode.Tag {
			case 'e':
				formatLines(writeLine, " ", in)
			case 'r':
				formatLines(writeLine, "-", in)
				formatLines(writeLine, "+", out)
			case 'd':
				formatLines(writeLine, "-", in)
			case 'i':
				formatLines(writeLine, "+", out)
			}
		}
	}
	return buf.String()
}

// hasWhitespaceDiffLines returns true if any diff groups is only different
// because of whitespace characters.
func hasWhitespaceDiffLines(groups [][]difflib.OpCode, a, b []string) bool {
	for _, group := range groups {
		in, out := new(bytes.Buffer), new(bytes.Buffer)
		for _, opCode := range group {
			if opCode.Tag == 'e' {
				continue
			}
			for _, line := range a[opCode.I1:opCode.I2] {
				in.WriteString(line)
			}
			for _, line := range b[opCode.J1:opCode.J2] {
				out.WriteString(line)
			}
		}
		if removeWhitespace(in.String()) == removeWhitespace(out.String()) {
			return true
		}
	}
	return false
}

func removeWhitespace(s string) string {
	var result []rune
	for _, r := range s {
		if !unicode.IsSpace(r) {
			result = append(result, r)
		}
	}
	return string(result)
}

func visibleWhitespaceLine(ws func(string, string)) func(string, string) {
	mapToVisibleSpace := func(r rune) rune {
		switch r {
		case '\n':
		case ' ':
			return '·'
		case '\t':
			return '▷'
		case '\v':
			return '▽'
		case '\r':
			return '↵'
		case '\f':
			return '↓'
		default:
			if unicode.IsSpace(r) {
				return '�'
			}
		}
		return r
	}
	return func(prefix, s string) {
		ws(prefix, strings.Map(mapToVisibleSpace, s))
	}
}

func formatHeader(wf func(string, ...interface{}), conf DiffConfig) {
	if conf.From != "" || conf.To != "" {
		wf("--- %s\n", conf.From)
		wf("+++ %s\n", conf.To)
	}
}

func formatRangeLine(wf func(string, ...interface{}), group []difflib.OpCode) {
	first, last := group[0], group[len(group)-1]
	range1 := formatRangeUnified(first.I1, last.I2)
	range2 := formatRangeUnified(first.J1, last.J2)
	wf("@@ -%s +%s @@\n", range1, range2)
}

// Convert range to the "ed" format
func formatRangeUnified(start, stop int) string {
	// Per the diff spec at http://www.unix.org/single_unix_specification/
	beginning := start + 1 // lines start numbering with one
	length := stop - start
	if length == 1 {
		return fmt.Sprintf("%d", beginning)
	}
	if length == 0 {
		beginning-- // empty ranges begin at line just before the range
	}
	return fmt.Sprintf("%d,%d", beginning, length)
}

func formatLines(writeLine func(string, string), prefix string, lines []string) {
	for _, line := range lines {
		writeLine(prefix, line)
	}
	// Add a newline if the last line is missing one so that the diff displays
	// properly.
	if !strings.HasSuffix(lines[len(lines)-1], "\n") {
		writeLine("", "\n")
	}
}
