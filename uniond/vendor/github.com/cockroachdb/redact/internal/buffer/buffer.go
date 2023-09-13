// Copyright 2021 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package buffer

import (
	"bytes"
	origFmt "fmt"
	"unicode/utf8"
	"unsafe"

	"github.com/cockroachdb/redact/internal/escape"
	m "github.com/cockroachdb/redact/internal/markers"
)

// Buffer is a variable-sized buffer of bytes with a
// Write method. Writes are escaped in different ways
// depending on the output mode.
type Buffer struct {
	buf        []byte
	validUntil int // exclusive upper bound of data that's already validated
	mode       OutputMode
	markerOpen bool
}

// OutputMode determines how writes are processed in the Buffer.
type OutputMode int

const (
	// UnsafeEscaped says that written data is unsafe for reporting,
	// so it should be enclosed in redaction markers,
	// and may contain redaction markers itself, and so it should be escaped.
	UnsafeEscaped OutputMode = iota
	// SafeEscaped says that written data is safe for reporting
	// (should not be enclosed in redaction markers) but may itself
	// contain redaction markers, and so it should be escaped.
	SafeEscaped
	// SafeRaw says that written data is safe for reporting
	// (should not be enclosed in redaction markers) and is
	// guaranteed not to contain redaction markers, so it
	// needs not be escaped.
	SafeRaw
	// PreRedactable says that written data already contains a mixed
	// of safe and unsafe information with suitably placed redaction markers,
	// and so should be inlined.
	PreRedactable = SafeRaw
)

// RedactableBytes returns the bytes in the buffer.
func (b Buffer) RedactableBytes() m.RedactableBytes {
	// NB: we're dependent on the fact this is a copy of the original
	// buffer. The finalize() method should not be called
	// in a conceputally read-only accessor like RedactableBytes().
	b.finalize()
	return m.RedactableBytes(b.buf)
}

// RedactableString returns the bytes in the buffer.
func (b Buffer) RedactableString() m.RedactableString {
	// NB: we're dependent on the fact this is a copy of the original
	// buffer. The finalize() method should not be called
	// in a conceputally read-only accessor like RedactableString().
	b.finalize()
	return m.RedactableString(b.buf)
}

// String implemens fmt.Stringer.
func (b Buffer) String() string {
	// NB: we're dependent on the fact this is a copy of the original
	// buffer. The finalize() method should not be called
	// in a conceputally read-only accessor like String().
	b.finalize()
	return m.RedactableString(b.buf).StripMarkers()
}

// TakeRedactableBytes returns the buffer
// contents and reinitializes the buffer. This saves
// a memory allocation compared to RedactableBytes().
func (b *Buffer) TakeRedactableBytes() m.RedactableBytes {
	b.finalize()
	r := b.buf
	b.buf = nil
	b.validUntil = 0
	b.mode = UnsafeEscaped
	return m.RedactableBytes(r)
}

// TakeRedactableString returns the buffer
// contents and reinitializes the buffer. This saves
// a memory allocation compared to RedactableString().
func (b *Buffer) TakeRedactableString() m.RedactableString {
	if b == nil {
		// Special case, useful in debugging.
		return "<nil>"
	}
	b.finalize()
	r := *(*m.RedactableString)(unsafe.Pointer(&b.buf))
	b.buf = nil
	b.validUntil = 0
	b.mode = UnsafeEscaped
	return r
}

// Len returns the number of bytes in the buffer.
func (b *Buffer) Len() int {
	copy := *b
	copy.finalize()
	return len(copy.buf)
}

// Cap returns the capacity of the buffer's underlying byte slice,
// that is, the total space allocated for the buffer's data.
func (b *Buffer) Cap() int {
	return cap(b.buf)
}

// Write appends the contents of p to the buffer, growing the buffer as
// needed. The return value n is the length of p; err is always nil. If the
// buffer becomes too large, Write will panic with ErrTooLarge.
func (b *Buffer) Write(p []byte) (n int, err error) {
	b.startWrite()
	m, ok := b.tryGrowByReslice(len(p))
	if !ok {
		m = b.grow(len(p))
	}
	return copy(b.buf[m:], p), nil
}

// WriteString appends the contents of s to the buffer, growing the buffer as
// needed. The return value n is the length of s; err is always nil. If the
// buffer becomes too large, WriteString will panic with ErrTooLarge.
func (b *Buffer) WriteString(s string) (n int, err error) {
	b.startWrite()
	m, ok := b.tryGrowByReslice(len(s))
	if !ok {
		m = b.grow(len(s))
	}
	return copy(b.buf[m:], s), nil
}

// WriteByte emits a single byte.
func (b *Buffer) WriteByte(s byte) error {
	b.startWrite()
	if b.mode == UnsafeEscaped &&
		(s >= utf8.RuneSelf ||
			s == m.StartS[0] || s == m.EndS[0]) {
		// Unsafe byte. Escape it.
		_, err := b.WriteString(m.EscapeMarkS)
		return err
	}
	m, ok := b.tryGrowByReslice(1)
	if !ok {
		m = b.grow(1)
	}
	b.buf[m] = s
	return nil
}

// WriteRune emits a single rune.
func (b *Buffer) WriteRune(s rune) error {
	b.startWrite()
	l := utf8.RuneLen(s)
	m, ok := b.tryGrowByReslice(l)
	if !ok {
		m = b.grow(l)
	}
	_ = utf8.EncodeRune(b.buf[m:], s)
	return nil
}

// finalize ensures that all the buffer is properly
// marked.
func (b *Buffer) finalize() {
	if b.mode == SafeRaw {
		b.validUntil = len(b.buf)
	} else {
		b.escapeToEnd(b.mode == UnsafeEscaped /* breakNewLines */)
	}
	if b.markerOpen {
		b.endRedactable()
		b.validUntil = len(b.buf)
	}
}

// endRedactable adds the closing redaction marker.
func (b *Buffer) endRedactable() {
	if len(b.buf) == 0 {
		return
	}
	if bytes.HasSuffix(b.buf, m.StartBytes) {
		// Special case: remove a trailing open marker and call it a day.
		b.buf = b.buf[:len(b.buf)-m.StartLen]
	} else {
		p, ok := b.tryGrowByReslice(m.EndLen)
		if !ok {
			p = b.grow(m.EndLen)
		}
		copy(b.buf[p:], m.EndS)
	}
	b.markerOpen = false
}

func (b *Buffer) startWrite() {
	if b.mode == UnsafeEscaped && !b.markerOpen {
		b.startRedactable()
		b.validUntil = len(b.buf)
	}
}

// endRedactable adds the closing redaction marker.
func (b *Buffer) startRedactable() {
	if bytes.HasSuffix(b.buf, m.EndBytes) {
		// Special case: remove a trailing closing marker and call it a day.
		b.buf = b.buf[:len(b.buf)-m.EndLen]
	} else {
		p, ok := b.tryGrowByReslice(len(m.StartS))
		if !ok {
			p = b.grow(len(m.StartS))
		}
		copy(b.buf[p:], m.StartS)
	}
	b.markerOpen = true
}

// escapeToEnd escapes occurrences of redaction markers in
// b.buf[b.validUntil:] and advances b.validUntil until the end.
func (b *Buffer) escapeToEnd(breakNewLines bool) {
	b.buf = escape.InternalEscapeBytes(b.buf, b.validUntil, breakNewLines, false /* trim */)
	b.validUntil = len(b.buf)
}

// GetMode retrieves the output mode.
func (b *Buffer) GetMode() OutputMode {
	return b.mode
}

// SetMode changes the output mode.
func (b *Buffer) SetMode(newMode OutputMode) {
	if b.mode == newMode {
		// noop
		return
	}
	if b.mode == UnsafeEscaped || b.mode == SafeEscaped {
		b.escapeToEnd(b.mode == UnsafeEscaped /* breakNewLines */)
	}
	if b.markerOpen {
		b.endRedactable()
	}
	b.validUntil = len(b.buf)
	b.mode = newMode
}

// Reset resets the buffer to be empty,
// but it retains the underlying storage for use by future writes.
// It also resets the output mode to UnsafeEscaped.
func (b *Buffer) Reset() {
	b.buf = b.buf[:0]
	b.validUntil = 0
	b.mode = UnsafeEscaped
	b.markerOpen = false
}

// tryGrowByReslice is a inlineable version of grow for the fast-case where the
// internal buffer only needs to be resliced.
// It returns the index where bytes should be written and whether it succeeded.
func (b *Buffer) tryGrowByReslice(n int) (int, bool) {
	if l := len(b.buf); n <= cap(b.buf)-l {
		b.buf = b.buf[:l+n]
		return l, true
	}
	return 0, false
}

// grow grows the buffer to guarantee space for n more bytes.
// It returns the index where bytes should be written.
// If the buffer can't grow it will panic with ErrTooLarge.
func (b *Buffer) grow(n int) int {
	m := len(b.buf)
	// Try to grow by means of a reslice.
	if i, ok := b.tryGrowByReslice(n); ok {
		return i
	}
	if b.buf == nil && n <= smallBufferSize {
		b.buf = make([]byte, n, smallBufferSize)
		return 0
	}
	c := cap(b.buf)
	const maxInt = int(^uint(0) >> 1)
	if n <= c/2-m {
		// let capacity get twice as large.
	} else if c > maxInt-c-n {
		panic(ErrTooLarge)
	} else {
		// Not enough space anywhere, we need to allocate.
		buf := makeSlice(2*c + n)
		copy(buf, b.buf)
		b.buf = buf
	}
	b.buf = b.buf[:m+n]
	return m
}

// Grow grows the buffer's capacity, if necessary, to guarantee space for
// another n bytes. After Grow(n), at least n bytes can be written to the
// buffer without another allocation.
// If n is negative, Grow will panic.
// If the buffer can't grow it will panic with ErrTooLarge.
func (b *Buffer) Grow(n int) {
	if n < 0 {
		panic(origFmt.Errorf("redact.Buffer.Grow: negative count"))
	}
	m := b.grow(n)
	b.buf = b.buf[:m]
}

// clone is used in tests.
func (b *Buffer) clone() *Buffer {
	c := *b
	c.buf = append([]byte(nil), b.buf...)
	return &c
}

// makeSlice allocates a slice of size n. If the allocation fails, it panics
// with ErrTooLarge.
func makeSlice(n int) []byte {
	// If the make fails, give a known error.
	defer func() {
		if recover() != nil {
			panic(ErrTooLarge)
		}
	}()
	return make([]byte, n)
}

// ErrTooLarge is passed to panic if memory cannot be allocated to
// store data in a buffer.
var ErrTooLarge = origFmt.Errorf("redact.Buffer: too large")

// smallBufferSize is an initial allocation minimal capacity.
const smallBufferSize = 64
