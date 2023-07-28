package pool

import (
	"bufio"
	"io"
	"sync"
)

const WriterBufferSize = 4096

var bufioWriterPool = sync.Pool{
	New: func() interface{} {
		return bufio.NewWriterSize(nil, WriterBufferSize)
	},
}

// Writer is a buffered writer that returns its internal buffer in a pool when
// not in use.
type Writer struct {
	W    io.Writer
	bufw *bufio.Writer
}

func (w *Writer) ensureBuffer() {
	if w.bufw == nil {
		w.bufw = bufioWriterPool.Get().(*bufio.Writer)
		w.bufw.Reset(w.W)
	}
}

// Write writes the given byte slice to the underlying connection.
//
// Note: Write won't return the write buffer to the pool even if it ends up
// being empty after the write. You must call Flush() to do that.
func (w *Writer) Write(b []byte) (int, error) {
	if w.bufw == nil {
		if len(b) >= WriterBufferSize {
			return w.W.Write(b)
		}
		w.bufw = bufioWriterPool.Get().(*bufio.Writer)
		w.bufw.Reset(w.W)
	}
	return w.bufw.Write(b)
}

// Size returns the size of the underlying buffer.
func (w *Writer) Size() int {
	return WriterBufferSize
}

// Available returns the amount buffer space available.
func (w *Writer) Available() int {
	if w.bufw != nil {
		return w.bufw.Available()
	}
	return WriterBufferSize
}

// Buffered returns the amount of data buffered.
func (w *Writer) Buffered() int {
	if w.bufw != nil {
		return w.bufw.Buffered()
	}
	return 0
}

// WriteByte writes a single byte.
func (w *Writer) WriteByte(b byte) error {
	w.ensureBuffer()
	return w.bufw.WriteByte(b)
}

// WriteRune writes a single rune, returning the number of bytes written.
func (w *Writer) WriteRune(r rune) (int, error) {
	w.ensureBuffer()
	return w.bufw.WriteRune(r)
}

// WriteString writes a string, returning the number of bytes written.
func (w *Writer) WriteString(s string) (int, error) {
	w.ensureBuffer()
	return w.bufw.WriteString(s)
}

// Flush flushes the write buffer, if any, and returns it to the pool.
func (w *Writer) Flush() error {
	if w.bufw == nil {
		return nil
	}
	if err := w.bufw.Flush(); err != nil {
		return err
	}
	w.bufw.Reset(nil)
	bufioWriterPool.Put(w.bufw)
	w.bufw = nil
	return nil
}

// Close flushes the underlying writer and closes it if it implements the
// io.Closer interface.
//
// Note: Close() closes the writer even if Flush() fails to avoid leaking system
// resources. If you want to make sure Flush() succeeds, call it first.
func (w *Writer) Close() error {
	var (
		ferr, cerr error
	)
	ferr = w.Flush()

	// always close even if flush fails.
	if closer, ok := w.W.(io.Closer); ok {
		cerr = closer.Close()
	}

	if ferr != nil {
		return ferr
	}
	return cerr
}
