// This is a derivitive work of Go's bytes.Buffer implementation.
//
// Originally copyright 2009 The Go Authors. All rights reserved.
//
// Modifications copyright 2018 Steven Allen. All rights reserved.
//
// Use of this source code is governed by both a BSD-style and an MIT-style
// license that can be found in the LICENSE_BSD and LICENSE files.

package pool

import (
	"io"
)

// Buffer is a buffer like bytes.Buffer that:
//
// 1. Uses a buffer pool.
// 2. Frees memory on read.
//
// If you only have a few buffers and read/write at a steady rate, *don't* use
// this package, it'll be slower.
//
// However:
//
// 1. If you frequently create/destroy buffers, this implementation will be
//    significantly nicer to the allocator.
// 2. If you have many buffers with bursty traffic, this implementation will use
//    significantly less memory.
type Buffer struct {
	// Pool is the buffer pool to use. If nil, this Buffer will use the
	// global buffer pool.
	Pool *BufferPool

	buf  []byte
	rOff int

	// Preallocated slice for samll reads/writes.
	// This is *really* important for performance and only costs 8 words.
	bootstrap [64]byte
}

// NewBuffer constructs a new buffer initialized to `buf`.
// Unlike `bytes.Buffer`, we *copy* the buffer but don't reuse it (to ensure
// that we *only* use buffers from the pool).
func NewBuffer(buf []byte) *Buffer {
	b := new(Buffer)
	if len(buf) > 0 {
		b.buf = b.getBuf(len(buf))
		copy(b.buf, buf)
	}
	return b
}

// NewBufferString is identical to NewBuffer *except* that it allows one to
// initialize the buffer from a string (without having to allocate an
// intermediate bytes slice).
func NewBufferString(buf string) *Buffer {
	b := new(Buffer)
	if len(buf) > 0 {
		b.buf = b.getBuf(len(buf))
		copy(b.buf, buf)
	}
	return b
}

func (b *Buffer) grow(n int) int {
	wOff := len(b.buf)
	bCap := cap(b.buf)

	if bCap >= wOff+n {
		b.buf = b.buf[:wOff+n]
		return wOff
	}

	bSize := b.Len()

	minCap := 2*bSize + n

	// Slide if cap >= minCap.
	// Reallocate otherwise.
	if bCap >= minCap {
		copy(b.buf, b.buf[b.rOff:])
	} else {
		// Needs new buffer.
		newBuf := b.getBuf(minCap)
		copy(newBuf, b.buf[b.rOff:])
		b.returnBuf()
		b.buf = newBuf
	}

	b.rOff = 0
	b.buf = b.buf[:bSize+n]
	return bSize
}

func (b *Buffer) getPool() *BufferPool {
	if b.Pool == nil {
		return GlobalPool
	}
	return b.Pool
}

func (b *Buffer) returnBuf() {
	if cap(b.buf) > len(b.bootstrap) {
		b.getPool().Put(b.buf)
	}
	b.buf = nil
}

func (b *Buffer) getBuf(n int) []byte {
	if n <= len(b.bootstrap) {
		return b.bootstrap[:n]
	}
	return b.getPool().Get(n)
}

// Len returns the number of bytes that can be read from this buffer.
func (b *Buffer) Len() int {
	return len(b.buf) - b.rOff
}

// Cap returns the current capacity of the buffer.
//
// Note: Buffer *may* re-allocate when writing (or growing by) `n` bytes even if
// `Cap() < Len() + n` to avoid excessive copying.
func (b *Buffer) Cap() int {
	return cap(b.buf)
}

// Bytes returns the slice of bytes currently buffered in the Buffer.
//
// The buffer returned by Bytes is valid until the next call grow, truncate,
// read, or write. Really, just don't touch the Buffer until you're done with
// the return value of this function.
func (b *Buffer) Bytes() []byte {
	return b.buf[b.rOff:]
}

// String returns the string representation of the buffer.
//
// It returns `<nil>` the buffer is a nil pointer.
func (b *Buffer) String() string {
	if b == nil {
		return "<nil>"
	}
	return string(b.buf[b.rOff:])
}

// WriteString writes a string to the buffer.
//
// This function is identical to Write except that it allows one to write a
// string directly without allocating an intermediate byte slice.
func (b *Buffer) WriteString(buf string) (int, error) {
	wOff := b.grow(len(buf))
	return copy(b.buf[wOff:], buf), nil
}

// Truncate truncates the Buffer.
//
// Panics if `n > b.Len()`.
//
// This function may free memory by shrinking the internal buffer.
func (b *Buffer) Truncate(n int) {
	if n < 0 || n > b.Len() {
		panic("truncation out of range")
	}
	b.buf = b.buf[:b.rOff+n]
	b.shrink()
}

// Reset is equivalent to Truncate(0).
func (b *Buffer) Reset() {
	b.returnBuf()
	b.rOff = 0
}

// ReadByte reads a single byte from the Buffer.
func (b *Buffer) ReadByte() (byte, error) {
	if b.rOff >= len(b.buf) {
		return 0, io.EOF
	}
	c := b.buf[b.rOff]
	b.rOff++
	return c, nil
}

// WriteByte writes a single byte to the Buffer.
func (b *Buffer) WriteByte(c byte) error {
	wOff := b.grow(1)
	b.buf[wOff] = c
	return nil
}

// Grow grows the internal buffer such that `n` bytes can be written without
// reallocating.
func (b *Buffer) Grow(n int) {
	wOff := b.grow(n)
	b.buf = b.buf[:wOff]
}

// Next is an alternative to `Read` that returns a byte slice instead of taking
// one.
//
// The returned byte slice is valid until the next read, write, grow, or
// truncate.
func (b *Buffer) Next(n int) []byte {
	m := b.Len()
	if m < n {
		n = m
	}
	data := b.buf[b.rOff : b.rOff+n]
	b.rOff += n
	return data
}

// Write writes the byte slice to the buffer.
func (b *Buffer) Write(buf []byte) (int, error) {
	wOff := b.grow(len(buf))
	return copy(b.buf[wOff:], buf), nil
}

// WriteTo copies from the buffer into the given writer until the buffer is
// empty.
func (b *Buffer) WriteTo(w io.Writer) (int64, error) {
	if b.rOff < len(b.buf) {
		n, err := w.Write(b.buf[b.rOff:])
		b.rOff += n
		if b.rOff > len(b.buf) {
			panic("invalid write count")
		}
		b.shrink()
		return int64(n), err
	}
	return 0, nil
}

// MinRead is the minimum slice size passed to a Read call by
// Buffer.ReadFrom. As long as the Buffer has at least MinRead bytes beyond
// what is required to hold the contents of r, ReadFrom will not grow the
// underlying buffer.
const MinRead = 512

// ReadFrom reads from the given reader into the buffer.
func (b *Buffer) ReadFrom(r io.Reader) (int64, error) {
	n := int64(0)
	for {
		wOff := b.grow(MinRead)
		// Use *entire* buffer.
		b.buf = b.buf[:cap(b.buf)]

		read, err := r.Read(b.buf[wOff:])
		b.buf = b.buf[:wOff+read]
		n += int64(read)
		switch err {
		case nil:
		case io.EOF:
			err = nil
			fallthrough
		default:
			b.shrink()
			return n, err
		}
	}
}

// Read reads at most `len(buf)` bytes from the internal buffer into the given
// buffer.
func (b *Buffer) Read(buf []byte) (int, error) {
	if len(buf) == 0 {
		return 0, nil
	}
	if b.rOff >= len(b.buf) {
		return 0, io.EOF
	}
	n := copy(buf, b.buf[b.rOff:])
	b.rOff += n
	b.shrink()
	return n, nil
}

func (b *Buffer) shrink() {
	c := b.Cap()
	// Either nil or bootstrap.
	if c <= len(b.bootstrap) {
		return
	}

	l := b.Len()
	if l == 0 {
		// Shortcut if empty.
		b.returnBuf()
		b.rOff = 0
	} else if l*8 < c {
		// Only shrink when capacity > 8x length. Avoids shrinking too aggressively.
		newBuf := b.getBuf(l)
		copy(newBuf, b.buf[b.rOff:])
		b.returnBuf()
		b.rOff = 0
		b.buf = newBuf[:l]
	}
}
