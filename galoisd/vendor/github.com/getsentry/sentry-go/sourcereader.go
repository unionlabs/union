package sentry

import (
	"bytes"
	"os"
	"sync"
)

type sourceReader struct {
	mu    sync.Mutex
	cache map[string][][]byte
}

func newSourceReader() sourceReader {
	return sourceReader{
		cache: make(map[string][][]byte),
	}
}

func (sr *sourceReader) readContextLines(filename string, line, context int) ([][]byte, int) {
	sr.mu.Lock()
	defer sr.mu.Unlock()

	lines, ok := sr.cache[filename]

	if !ok {
		data, err := os.ReadFile(filename)
		if err != nil {
			sr.cache[filename] = nil
			return nil, 0
		}
		lines = bytes.Split(data, []byte{'\n'})
		sr.cache[filename] = lines
	}

	return sr.calculateContextLines(lines, line, context)
}

func (sr *sourceReader) calculateContextLines(lines [][]byte, line, context int) ([][]byte, int) {
	// Stacktrace lines are 1-indexed, slices are 0-indexed
	line--

	// contextLine points to a line that caused an issue itself, in relation to
	// returned slice.
	contextLine := context

	if lines == nil || line >= len(lines) || line < 0 {
		return nil, 0
	}

	if context < 0 {
		context = 0
		contextLine = 0
	}

	start := line - context

	if start < 0 {
		contextLine += start
		start = 0
	}

	end := line + context + 1

	if end > len(lines) {
		end = len(lines)
	}

	return lines[start:end], contextLine
}
