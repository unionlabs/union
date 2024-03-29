package zerolog

import (
	"bytes"
	"io"
	"path"
	"runtime"
	"strconv"
	"strings"
	"sync"
)

// LevelWriter defines as interface a writer may implement in order
// to receive level information with payload.
type LevelWriter interface {
	io.Writer
	WriteLevel(level Level, p []byte) (n int, err error)
}

// LevelWriterAdapter adapts an io.Writer to support the LevelWriter interface.
type LevelWriterAdapter struct {
	io.Writer
}

// WriteLevel simply writes everything to the adapted writer, ignoring the level.
func (lw LevelWriterAdapter) WriteLevel(l Level, p []byte) (n int, err error) {
	return lw.Write(p)
}

// Call the underlying writer's Close method if it is an io.Closer. Otherwise
// does nothing.
func (lw LevelWriterAdapter) Close() error {
	if closer, ok := lw.Writer.(io.Closer); ok {
		return closer.Close()
	}
	return nil
}

type syncWriter struct {
	mu sync.Mutex
	lw LevelWriter
}

// SyncWriter wraps w so that each call to Write is synchronized with a mutex.
// This syncer can be used to wrap the call to writer's Write method if it is
// not thread safe. Note that you do not need this wrapper for os.File Write
// operations on POSIX and Windows systems as they are already thread-safe.
func SyncWriter(w io.Writer) io.Writer {
	if lw, ok := w.(LevelWriter); ok {
		return &syncWriter{lw: lw}
	}
	return &syncWriter{lw: LevelWriterAdapter{w}}
}

// Write implements the io.Writer interface.
func (s *syncWriter) Write(p []byte) (n int, err error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.lw.Write(p)
}

// WriteLevel implements the LevelWriter interface.
func (s *syncWriter) WriteLevel(l Level, p []byte) (n int, err error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.lw.WriteLevel(l, p)
}

func (s *syncWriter) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if closer, ok := s.lw.(io.Closer); ok {
		return closer.Close()
	}
	return nil
}

type multiLevelWriter struct {
	writers []LevelWriter
}

func (t multiLevelWriter) Write(p []byte) (n int, err error) {
	for _, w := range t.writers {
		if _n, _err := w.Write(p); err == nil {
			n = _n
			if _err != nil {
				err = _err
			} else if _n != len(p) {
				err = io.ErrShortWrite
			}
		}
	}
	return n, err
}

func (t multiLevelWriter) WriteLevel(l Level, p []byte) (n int, err error) {
	for _, w := range t.writers {
		if _n, _err := w.WriteLevel(l, p); err == nil {
			n = _n
			if _err != nil {
				err = _err
			} else if _n != len(p) {
				err = io.ErrShortWrite
			}
		}
	}
	return n, err
}

// Calls close on all the underlying writers that are io.Closers. If any of the
// Close methods return an error, the remainder of the closers are not closed
// and the error is returned.
func (t multiLevelWriter) Close() error {
	for _, w := range t.writers {
		if closer, ok := w.(io.Closer); ok {
			if err := closer.Close(); err != nil {
				return err
			}
		}
	}
	return nil
}

// MultiLevelWriter creates a writer that duplicates its writes to all the
// provided writers, similar to the Unix tee(1) command. If some writers
// implement LevelWriter, their WriteLevel method will be used instead of Write.
func MultiLevelWriter(writers ...io.Writer) LevelWriter {
	lwriters := make([]LevelWriter, 0, len(writers))
	for _, w := range writers {
		if lw, ok := w.(LevelWriter); ok {
			lwriters = append(lwriters, lw)
		} else {
			lwriters = append(lwriters, LevelWriterAdapter{w})
		}
	}
	return multiLevelWriter{lwriters}
}

// TestingLog is the logging interface of testing.TB.
type TestingLog interface {
	Log(args ...interface{})
	Logf(format string, args ...interface{})
	Helper()
}

// TestWriter is a writer that writes to testing.TB.
type TestWriter struct {
	T TestingLog

	// Frame skips caller frames to capture the original file and line numbers.
	Frame int
}

// NewTestWriter creates a writer that logs to the testing.TB.
func NewTestWriter(t TestingLog) TestWriter {
	return TestWriter{T: t}
}

// Write to testing.TB.
func (t TestWriter) Write(p []byte) (n int, err error) {
	t.T.Helper()

	n = len(p)

	// Strip trailing newline because t.Log always adds one.
	p = bytes.TrimRight(p, "\n")

	// Try to correct the log file and line number to the caller.
	if t.Frame > 0 {
		_, origFile, origLine, _ := runtime.Caller(1)
		_, frameFile, frameLine, ok := runtime.Caller(1 + t.Frame)
		if ok {
			erase := strings.Repeat("\b", len(path.Base(origFile))+len(strconv.Itoa(origLine))+3)
			t.T.Logf("%s%s:%d: %s", erase, path.Base(frameFile), frameLine, p)
			return n, err
		}
	}
	t.T.Log(string(p))

	return n, err
}

// ConsoleTestWriter creates an option that correctly sets the file frame depth for testing.TB log.
func ConsoleTestWriter(t TestingLog) func(w *ConsoleWriter) {
	return func(w *ConsoleWriter) {
		w.Out = TestWriter{T: t, Frame: 6}
	}
}

// FilteredLevelWriter writes only logs at Level or above to Writer.
//
// It should be used only in combination with MultiLevelWriter when you
// want to write to multiple destinations at different levels. Otherwise
// you should just set the level on the logger and filter events early.
// When using MultiLevelWriter then you set the level on the logger to
// the lowest of the levels you use for writers.
type FilteredLevelWriter struct {
	Writer LevelWriter
	Level  Level
}

// Write writes to the underlying Writer.
func (w *FilteredLevelWriter) Write(p []byte) (int, error) {
	return w.Writer.Write(p)
}

// WriteLevel calls WriteLevel of the underlying Writer only if the level is equal
// or above the Level.
func (w *FilteredLevelWriter) WriteLevel(level Level, p []byte) (int, error) {
	if level >= w.Level {
		return w.Writer.WriteLevel(level, p)
	}
	return len(p), nil
}

var triggerWriterPool = &sync.Pool{
	New: func() interface{} {
		return bytes.NewBuffer(make([]byte, 0, 1024))
	},
}

// TriggerLevelWriter buffers log lines at the ConditionalLevel or below
// until a trigger level (or higher) line is emitted. Log lines with level
// higher than ConditionalLevel are always written out to the destination
// writer. If trigger never happens, buffered log lines are never written out.
//
// It can be used to configure "log level per request".
type TriggerLevelWriter struct {
	// Destination writer. If LevelWriter is provided (usually), its WriteLevel is used
	// instead of Write.
	io.Writer

	// ConditionalLevel is the level (and below) at which lines are buffered until
	// a trigger level (or higher) line is emitted. Usually this is set to DebugLevel.
	ConditionalLevel Level

	// TriggerLevel is the lowest level that triggers the sending of the conditional
	// level lines. Usually this is set to ErrorLevel.
	TriggerLevel Level

	buf       *bytes.Buffer
	triggered bool
	mu        sync.Mutex
}

func (w *TriggerLevelWriter) WriteLevel(l Level, p []byte) (n int, err error) {
	w.mu.Lock()
	defer w.mu.Unlock()

	// At first trigger level or above log line, we flush the buffer and change the
	// trigger state to triggered.
	if !w.triggered && l >= w.TriggerLevel {
		err := w.trigger()
		if err != nil {
			return 0, err
		}
	}

	// Unless triggered, we buffer everything at and below ConditionalLevel.
	if !w.triggered && l <= w.ConditionalLevel {
		if w.buf == nil {
			w.buf = triggerWriterPool.Get().(*bytes.Buffer)
		}

		// We prefix each log line with a byte with the level.
		// Hopefully we will never have a level value which equals a newline
		// (which could interfere with reconstruction of log lines in the trigger method).
		w.buf.WriteByte(byte(l))
		w.buf.Write(p)
		return len(p), nil
	}

	// Anything above ConditionalLevel is always passed through.
	// Once triggered, everything is passed through.
	if lw, ok := w.Writer.(LevelWriter); ok {
		return lw.WriteLevel(l, p)
	}
	return w.Write(p)
}

// trigger expects lock to be held.
func (w *TriggerLevelWriter) trigger() error {
	if w.triggered {
		return nil
	}
	w.triggered = true

	if w.buf == nil {
		return nil
	}

	p := w.buf.Bytes()
	for len(p) > 0 {
		// We do not use bufio.Scanner here because we already have full buffer
		// in the memory and we do not want extra copying from the buffer to
		// scanner's token slice, nor we want to hit scanner's token size limit,
		// and we also want to preserve newlines.
		i := bytes.IndexByte(p, '\n')
		line := p[0 : i+1]
		p = p[i+1:]
		// We prefixed each log line with a byte with the level.
		level := Level(line[0])
		line = line[1:]
		var err error
		if lw, ok := w.Writer.(LevelWriter); ok {
			_, err = lw.WriteLevel(level, line)
		} else {
			_, err = w.Write(line)
		}
		if err != nil {
			return err
		}
	}

	return nil
}

// Trigger forces flushing the buffer and change the trigger state to
// triggered, if the writer has not already been triggered before.
func (w *TriggerLevelWriter) Trigger() error {
	w.mu.Lock()
	defer w.mu.Unlock()

	return w.trigger()
}

// Close closes the writer and returns the buffer to the pool.
func (w *TriggerLevelWriter) Close() error {
	w.mu.Lock()
	defer w.mu.Unlock()

	if w.buf == nil {
		return nil
	}

	// We return the buffer only if it has not grown above the limit.
	// This prevents accumulation of large buffers in the pool just
	// because occasionally a large buffer might be needed.
	if w.buf.Cap() <= TriggerLevelWriterBufferReuseLimit {
		w.buf.Reset()
		triggerWriterPool.Put(w.buf)
	}
	w.buf = nil

	return nil
}
