package traceparser

import (
	"bytes"
	"strconv"
)

var blockSeparator = []byte("\n\n")
var lineSeparator = []byte("\n")

// Parses multi-stacktrace text dump produced by runtime.Stack([]byte, all=true).
// The parser prioritizes performance but requires the input to be well-formed in order to return correct data.
// See https://github.com/golang/go/blob/go1.20.4/src/runtime/mprof.go#L1191
func Parse(data []byte) TraceCollection {
	var it = TraceCollection{}
	if len(data) > 0 {
		it.blocks = bytes.Split(data, blockSeparator)
	}
	return it
}

type TraceCollection struct {
	blocks [][]byte
}

func (it TraceCollection) Length() int {
	return len(it.blocks)
}

// Returns the stacktrace item at the given index.
func (it *TraceCollection) Item(i int) Trace {
	// The first item may have a leading data separator and the last one may have a trailing one.
	// Note: Trim() doesn't make a copy for single-character cutset under 0x80. It will just slice the original.
	var data []byte
	switch {
	case i == 0:
		data = bytes.TrimLeft(it.blocks[i], "\n")
	case i == len(it.blocks)-1:
		data = bytes.TrimRight(it.blocks[i], "\n")
	default:
		data = it.blocks[i]
	}

	var splitAt = bytes.IndexByte(data, '\n')
	if splitAt < 0 {
		return Trace{header: data}
	}

	return Trace{
		header: data[:splitAt],
		data:   data[splitAt+1:],
	}
}

// Trace represents a single stacktrace block, identified by a Goroutine ID and a sequence of Frames.
type Trace struct {
	header []byte
	data   []byte
}

var goroutinePrefix = []byte("goroutine ")

// GoID parses the Goroutine ID from the header.
func (t *Trace) GoID() (id uint64) {
	if bytes.HasPrefix(t.header, goroutinePrefix) {
		var line = t.header[len(goroutinePrefix):]
		var splitAt = bytes.IndexByte(line, ' ')
		if splitAt >= 0 {
			id, _ = strconv.ParseUint(string(line[:splitAt]), 10, 64)
		}
	}
	return id
}

// UniqueIdentifier can be used as a map key to identify the trace.
func (t *Trace) UniqueIdentifier() []byte {
	return t.data
}

func (t *Trace) Frames() FrameIterator {
	var lines = bytes.Split(t.data, lineSeparator)
	return FrameIterator{lines: lines, i: 0, len: len(lines)}
}

func (t *Trace) FramesReversed() ReverseFrameIterator {
	var lines = bytes.Split(t.data, lineSeparator)
	return ReverseFrameIterator{lines: lines, i: len(lines)}
}

const framesElided = "...additional frames elided..."

// FrameIterator iterates over stack frames.
type FrameIterator struct {
	lines [][]byte
	i     int
	len   int
}

// Next returns the next frame, or nil if there are none.
func (it *FrameIterator) Next() Frame {
	return Frame{it.popLine(), it.popLine()}
}

func (it *FrameIterator) popLine() []byte {
	switch {
	case it.i >= it.len:
		return nil
	case string(it.lines[it.i]) == framesElided:
		it.i++
		return it.popLine()
	default:
		it.i++
		return it.lines[it.i-1]
	}
}

// HasNext return true if there are values to be read.
func (it *FrameIterator) HasNext() bool {
	return it.i < it.len
}

// LengthUpperBound returns the maximum number of elements this stacks may contain.
// The actual number may be lower because of elided frames. As such, the returned value
// cannot be used to iterate over the frames but may be used to reserve capacity.
func (it *FrameIterator) LengthUpperBound() int {
	return it.len / 2
}

// ReverseFrameIterator iterates over stack frames in reverse order.
type ReverseFrameIterator struct {
	lines [][]byte
	i     int
}

// Next returns the next frame, or nil if there are none.
func (it *ReverseFrameIterator) Next() Frame {
	var line2 = it.popLine()
	return Frame{it.popLine(), line2}
}

func (it *ReverseFrameIterator) popLine() []byte {
	it.i--
	switch {
	case it.i < 0:
		return nil
	case string(it.lines[it.i]) == framesElided:
		return it.popLine()
	default:
		return it.lines[it.i]
	}
}

// HasNext return true if there are values to be read.
func (it *ReverseFrameIterator) HasNext() bool {
	return it.i > 1
}

// LengthUpperBound returns the maximum number of elements this stacks may contain.
// The actual number may be lower because of elided frames. As such, the returned value
// cannot be used to iterate over the frames but may be used to reserve capacity.
func (it *ReverseFrameIterator) LengthUpperBound() int {
	return len(it.lines) / 2
}

type Frame struct {
	line1 []byte
	line2 []byte
}

// UniqueIdentifier can be used as a map key to identify the frame.
func (f *Frame) UniqueIdentifier() []byte {
	// line2 contains file path, line number and program-counter offset from the beginning of a function
	// e.g. C:/Users/name/scoop/apps/go/current/src/testing/testing.go:1906 +0x63a
	return f.line2
}

var createdByPrefix = []byte("created by ")

func (f *Frame) Func() []byte {
	if bytes.HasPrefix(f.line1, createdByPrefix) {
		// Since go1.21, the line ends with " in goroutine X", saying which goroutine created this one.
		// We currently don't have use for that so just remove it.
		var line = f.line1[len(createdByPrefix):]
		var spaceAt = bytes.IndexByte(line, ' ')
		if spaceAt < 0 {
			return line
		}
		return line[:spaceAt]
	}

	var end = bytes.LastIndexByte(f.line1, '(')
	if end >= 0 {
		return f.line1[:end]
	}

	return f.line1
}

func (f *Frame) File() (path []byte, lineNumber int) {
	var line = f.line2
	if len(line) > 0 && line[0] == '\t' {
		line = line[1:]
	}

	var splitAt = bytes.IndexByte(line, ' ')
	if splitAt >= 0 {
		line = line[:splitAt]
	}

	splitAt = bytes.LastIndexByte(line, ':')
	if splitAt < 0 {
		return line, 0
	}

	lineNumber, _ = strconv.Atoi(string(line[splitAt+1:]))
	return line[:splitAt], lineNumber
}
