// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"fmt"
	"io"
	"log"
	"os"
	"regexp"
	"strconv"
	"strings"
	"sync/atomic"
	"unicode"

	"github.com/cockroachdb/errors"
	"github.com/pmezard/go-difflib/difflib"
	"github.com/stretchr/testify/require"
)

// history records the results of running a series of operations.
//
// history also implements the pebble.Logger interface, outputting to a stdlib
// logger, prefixing the log messages with "//"-style comments.
type history struct {
	err    atomic.Value
	failRE *regexp.Regexp
	log    *log.Logger
}

func newHistory(failRE *regexp.Regexp, writers ...io.Writer) *history {
	h := &history{failRE: failRE}
	h.log = log.New(io.MultiWriter(writers...), "", 0)
	return h
}

// Recordf records the results of a single operation.
func (h *history) Recordf(op int, format string, args ...interface{}) {
	if strings.Contains(format, "\n") {
		// We could remove this restriction but suffixing every line with "#<seq>".
		panic(fmt.Sprintf("format string must not contain \\n: %q", format))
	}

	// We suffix every line with #<op> in order to provide a marker to locate
	// the line using the diff output. This is necessary because the diff of two
	// histories is done after stripping comment lines (`// ...`) from the
	// history output, which ruins the line number information in the diff
	// output.
	m := fmt.Sprintf(format, args...) + fmt.Sprintf(" #%d", op)
	h.log.Print(m)

	if h.failRE != nil && h.failRE.MatchString(m) {
		err := errors.Errorf("failure regexp %q matched output: %s", h.failRE, m)
		h.err.Store(err)
	}
}

// Error returns an error if the test has failed from log output, either a
// failure regexp match or a call to Fatalf.
func (h *history) Error() error {
	if v := h.err.Load(); v != nil {
		return v.(error)
	}
	return nil
}

func (h *history) format(prefix, format string, args ...interface{}) string {
	var buf strings.Builder
	orig := fmt.Sprintf(format, args...)
	for _, line := range strings.Split(strings.TrimSpace(orig), "\n") {
		buf.WriteString(prefix)
		buf.WriteString(line)
		buf.WriteString("\n")
	}
	return buf.String()
}

// Infof implements the pebble.Logger interface. Note that the output is
// commented.
func (h *history) Infof(format string, args ...interface{}) {
	_ = h.log.Output(2, h.format("// INFO: ", format, args...))
}

// Fatalf implements the pebble.Logger interface. Note that the output is
// commented.
func (h *history) Fatalf(format string, args ...interface{}) {
	_ = h.log.Output(2, h.format("// FATAL: ", format, args...))
	h.err.Store(errors.Errorf(format, args...))
}

func (h *history) recorder(thread int, op int) historyRecorder {
	return historyRecorder{
		history: h,
		op:      op,
	}
}

// historyRecorder pairs a history with an operation, annotating all lines
// recorded through it with the operation number.
type historyRecorder struct {
	history *history
	op      int
}

// Recordf records the results of a single operation.
func (h historyRecorder) Recordf(format string, args ...interface{}) {
	h.history.Recordf(h.op, format, args...)
}

// Error returns an error if the test has failed from log output, either a
// failure regexp match or a call to Fatalf.
func (h historyRecorder) Error() error {
	return h.history.Error()
}

// CompareHistories takes a slice of file paths containing history files. It
// performs a diff comparing the first path to all other paths. CompareHistories
// returns the index and diff for the first history that differs. If all the
// histories are identical, CompareHistories returns a zero index and an empty
// string.
func CompareHistories(t TestingT, paths []string) (i int, diff string) {
	base := readHistory(t, paths[0])
	base = reorderHistory(base)

	for i := 1; i < len(paths); i++ {
		lines := readHistory(t, paths[i])
		lines = reorderHistory(lines)
		diff := difflib.UnifiedDiff{
			A:       base,
			B:       lines,
			Context: 5,
		}
		text, err := difflib.GetUnifiedDiffString(diff)
		require.NoError(t, err)
		if text != "" {
			return i, text
		}
	}
	return 0, ""
}

// reorderHistory takes lines from a history file and reorders the operation
// results to be in the order of the operation index numbers. Runs with more
// than 1 thread may produce out-of-order histories. Comment lines must've
// already been filtered out.
func reorderHistory(lines []string) []string {
	reordered := make([]string, len(lines))
	for _, l := range lines {
		if cleaned := strings.TrimSpace(l); cleaned == "" {
			continue
		}
		reordered[extractOp(l)] = l
	}
	return reordered
}

// extractOp parses out an operation's index from the trailing comment. Every
// line of history output is suffixed with a comment containing `#<op>`
func extractOp(line string) int {
	i := strings.LastIndexByte(line, '#')
	j := strings.IndexFunc(line[i+1:], unicode.IsSpace)
	if j == -1 {
		j = len(line[i+1:])
	}
	v, err := strconv.Atoi(line[i+1 : i+1+j])
	if err != nil {
		panic(fmt.Sprintf("unable to parse line %q: %s", line, err))
	}
	return v
}

// Read a history file, stripping out lines that begin with a comment.
func readHistory(t TestingT, historyPath string) []string {
	data, err := os.ReadFile(historyPath)
	require.NoError(t, err)
	lines := difflib.SplitLines(string(data))
	newLines := make([]string, 0, len(lines))
	for _, line := range lines {
		if strings.HasPrefix(line, "// ") {
			continue
		}
		newLines = append(newLines, line)
	}
	return newLines
}
