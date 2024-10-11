// Copyright (c) 2015 Klaus Post, released under MIT License. See LICENSE file.

// Package readahead will do asynchronous read-ahead from an input io.Reader
// and make the data available as an io.Reader.
//
// This should be fully transparent, except that once an error
// has been returned from the Reader, it will not recover.
//
// The readahead object also fulfills the io.WriterTo interface, which
// is likely to speed up copies.
//
// Package home: https://github.com/klauspost/readahead
package readahead

import (
	"errors"
	"fmt"
	"io"
)

const (
	// DefaultBuffers is the default number of buffers used.
	DefaultBuffers = 4

	// DefaultBufferSize is the default buffer size, 1 MB.
	DefaultBufferSize = 1 << 20
)

type seekable struct {
	*reader
}

type ReadSeekCloser interface {
	io.ReadCloser
	io.Seeker
}

type reader struct {
	in      io.Reader     // Input reader
	closer  io.Closer     // Optional closer
	ready   chan *buffer  // Buffers ready to be handed to the reader
	reuse   chan *buffer  // Buffers to reuse for input reading
	exit    chan struct{} // Closes when finished
	buffers int           // Number of buffers
	size    int           // Size of each buffer
	err     error         // If an error has occurred it is here
	cur     *buffer       // Current buffer being served
	exited  chan struct{} // Channel is closed been the async reader shuts down
	bufs    [][]byte
}

// NewReader returns a reader that will asynchronously read from
// the supplied reader into 4 buffers of 1MB each.
//
// It will start reading from the input at once, maybe even before this
// function has returned.
//
// The input can be read from the returned reader.
// When done use Close() to release the buffers.
// If a reader supporting the io.Seeker is given,
// the returned reader will also support it.
func NewReader(rd io.Reader) io.ReadCloser {
	if rd == nil {
		return nil
	}

	ret, err := NewReaderSize(rd, DefaultBuffers, DefaultBufferSize)

	// Should not be possible to trigger from other packages.
	if err != nil {
		panic("unexpected error:" + err.Error())
	}
	return ret
}

// NewReadCloser returns a reader that will asynchronously read from
// the supplied reader into 4 buffers of 1MB each.
//
// It will start reading from the input at once, maybe even before this
// function has returned.
//
// The input can be read from the returned reader.
// When done use Close() to release the buffers,
// which will also close the supplied closer.
// If a reader supporting the io.Seeker is given,
// the returned reader will also support it.
func NewReadCloser(rd io.ReadCloser) io.ReadCloser {
	if rd == nil {
		return nil
	}

	ret, err := NewReadCloserSize(rd, DefaultBuffers, DefaultBufferSize)

	// Should not be possible to trigger from other packages.
	if err != nil {
		panic("unexpected error:" + err.Error())
	}
	return ret
}

// NewReadSeeker returns a reader that will asynchronously read from
// the supplied reader into 4 buffers of 1MB each.
//
// It will start reading from the input at once, maybe even before this
// function has returned.
//
// The input can be read and seeked from the returned reader.
// When done use Close() to release the buffers.
func NewReadSeeker(rd io.ReadSeeker) ReadSeekCloser {
	//Not checking for result as the input interface guarantees it's seekable
	res, _ := NewReader(rd).(ReadSeekCloser)
	return res
}

// NewReadSeekCloser returns a reader that will asynchronously read from
// the supplied reader into 4 buffers of 1MB each.
//
// It will start reading from the input at once, maybe even before this
// function has returned.
//
// The input can be read and seeked from the returned reader.
// When done use Close() to release the buffers,
// which will also close the supplied closer.
func NewReadSeekCloser(rd ReadSeekCloser) ReadSeekCloser {
	// Not checking for result as the input interface guarantees it's seekable
	res, _ := NewReadCloser(rd).(ReadSeekCloser)
	return res
}

// NewReaderSize returns a reader with a custom number of buffers and size.
// buffers is the number of queued buffers and size is the size of each
// buffer in bytes.
func NewReaderSize(rd io.Reader, buffers, size int) (res io.ReadCloser, err error) {
	if size <= 0 {
		return nil, fmt.Errorf("buffer size too small")
	}
	if buffers <= 0 {
		return nil, fmt.Errorf("number of buffers too small")
	}
	if rd == nil {
		return nil, fmt.Errorf("nil input reader supplied")
	}
	a := &reader{}
	if _, ok := rd.(io.Seeker); ok {
		res = &seekable{a}
	} else {
		res = a
	}
	a.init(rd, buffers, size)
	return
}

// NewReaderBuffer returns a reader with a custom number of buffers and size.
// All buffers must be the same size.
// Buffers can be reused after Close has been called.
func NewReaderBuffer(rd io.Reader, buffers [][]byte) (res io.ReadCloser, err error) {
	if len(buffers) == 0 {
		return nil, fmt.Errorf("number of buffers too small")
	}
	sz := 0
	for _, buf := range buffers {
		if len(buf) == 0 {
			return nil, fmt.Errorf("zero size buffer sent")
		}
		if sz == 0 {
			sz = len(buf)
		}
		if sz != len(buf) {
			return nil, fmt.Errorf("buffers should have similar size")
		}
	}
	if rd == nil {
		return nil, fmt.Errorf("nil input reader supplied")
	}
	a := &reader{}
	if _, ok := rd.(io.Seeker); ok {
		res = &seekable{a}
	} else {
		res = a
	}
	a.initBuffers(rd, buffers, sz)

	return
}

// NewReadCloserSize returns a reader with a custom number of buffers and size.
// buffers is the number of queued buffers and size is the size of each
// buffer in bytes.
func NewReadCloserSize(rc io.ReadCloser, buffers, size int) (res io.ReadCloser, err error) {
	if size <= 0 {
		return nil, fmt.Errorf("buffer size too small")
	}
	if buffers <= 0 {
		return nil, fmt.Errorf("number of buffers too small")
	}
	if rc == nil {
		return nil, fmt.Errorf("nil input reader supplied")
	}
	a := &reader{closer: rc}
	if _, ok := rc.(io.Seeker); ok {
		res = &seekable{a}
	} else {
		res = a
	}
	a.init(rc, buffers, size)
	return
}

// NewReadCloserBuffer returns a reader with a custom number of buffers and size.
// All buffers must be the same size.
// Buffers can be reused after Close has been called.
func NewReadCloserBuffer(rc io.ReadCloser, buffers [][]byte) (res io.ReadCloser, err error) {
	if len(buffers) == 0 {
		return nil, fmt.Errorf("number of buffers too small")
	}
	sz := 0
	for _, buf := range buffers {
		if len(buf) == 0 {
			return nil, fmt.Errorf("zero size buffer sent")
		}
		if sz == 0 {
			sz = len(buf)
		}
		if sz != len(buf) {
			return nil, fmt.Errorf("buffers should have similar size")
		}
	}

	if rc == nil {
		return nil, fmt.Errorf("nil input reader supplied")
	}
	a := &reader{closer: rc}
	if _, ok := rc.(io.Seeker); ok {
		res = &seekable{a}
	} else {
		res = a
	}
	a.initBuffers(rc, buffers, sz)
	return
}

// NewReadSeekerSize returns a reader with a custom number of buffers and size.
// buffers is the number of queued buffers and size is the size of each
// buffer in bytes.
func NewReadSeekerSize(rd io.ReadSeeker, buffers, size int) (res ReadSeekCloser, err error) {
	reader, err := NewReaderSize(rd, buffers, size)
	if err != nil {
		return nil, err
	}
	//Not checking for result as the input interface guarantees it's seekable
	res, _ = reader.(ReadSeekCloser)
	return
}

// NewReadSeekCloserSize returns a reader with a custom number of buffers and size.
// buffers is the number of queued buffers and size is the size of each
// buffer in bytes.
func NewReadSeekCloserSize(rd ReadSeekCloser, buffers, size int) (res ReadSeekCloser, err error) {
	reader, err := NewReadCloserSize(rd, buffers, size)
	if err != nil {
		return nil, err
	}
	//Not checking for result as the input interface guarantees it's seekable
	res, _ = reader.(ReadSeekCloser)
	return
}

// NewReadSeekCloserBuffer returns a reader with a custom number of buffers and size.
// All buffers must be the same size.
// Buffers can be reused after Close has been called.
func NewReadSeekCloserBuffer(rd ReadSeekCloser, buffers [][]byte) (res ReadSeekCloser, err error) {
	reader, err := NewReadCloserBuffer(rd, buffers)
	if err != nil {
		return nil, err
	}
	//Not checking for result as the input interface guarantees it's seekable
	res, _ = reader.(ReadSeekCloser)
	return
}

// initialize the reader
func (a *reader) init(rd io.Reader, buffers, size int) {
	x := make([]byte, buffers*size)
	bufs := make([][]byte, buffers)
	for i := range bufs {
		bufs[i] = x[i*size : (i+1)*size : (i+1)*size]
	}
	a.initBuffers(rd, bufs, size)
}

// initialize the reader
func (a *reader) initBuffers(rd io.Reader, buffers [][]byte, size int) {
	a.in = rd
	a.ready = make(chan *buffer, len(buffers))
	a.reuse = make(chan *buffer, len(buffers))
	a.exit = make(chan struct{})
	a.exited = make(chan struct{})
	a.buffers = len(buffers)
	a.size = size
	a.cur = nil
	a.err = nil
	a.bufs = buffers

	// Create buffers
	for _, buf := range buffers {
		a.reuse <- newBuffer(buf)
	}

	// Start async reader
	go func() {
		// Ensure that when we exit this is signalled.
		defer close(a.exited)
		defer close(a.ready)
		var atEOF bool
		for {
			select {
			case b := <-a.reuse:
				if atEOF {
					// Return delay
					b.err = io.EOF
					b.buf = b.buf[:0]
					b.offset = 0
					a.ready <- b
					return
				}
				err := b.read(a.in)
				// Delay EOF if we have content.
				if err == io.EOF && len(b.buf) > 0 {
					atEOF = true
					err = nil
					b.err = nil
				}
				a.ready <- b
				if err != nil {
					return
				}
			case <-a.exit:
				return
			}
		}
	}()
}

// fill will check if the current buffer is empty and fill it if it is.
// If an error was returned at the end of the current buffer it is returned.
func (a *reader) fill() (err error) {
	if a.cur.isEmpty() {
		if a.cur != nil {
			a.reuse <- a.cur
			a.cur = nil
		}
		b, ok := <-a.ready
		if !ok {
			if a.err == nil {
				a.err = errors.New("readahead: read after Close")
			}
			return a.err
		}
		a.cur = b
	}
	return nil
}

// Read will return the next available data.
func (a *reader) Read(p []byte) (n int, err error) {
	if a.err != nil {
		return 0, a.err
	}
	// Swap buffer and maybe return error
	err = a.fill()
	if err != nil {
		return 0, err
	}

	// Copy what we can
	n = copy(p, a.cur.buffer())
	a.cur.inc(n)

	if a.cur.isEmpty() {
		// Return current, so a fetch can start.
		if a.cur != nil {
			// If at end of buffer, return any error, if present
			a.err = a.cur.err
			a.reuse <- a.cur
			a.cur = nil
		}
		return n, a.err
	}
	return n, nil
}

func (a *seekable) Seek(offset int64, whence int) (res int64, err error) {
	//Not checking the result as seekable receiver guarantees it to be assertable
	seeker, _ := a.in.(io.Seeker)
	//Make sure the async routine is closed
	select {
	case <-a.exited:
	case a.exit <- struct{}{}:
		<-a.exited
	}
	if whence == io.SeekCurrent {
		//If need to seek based on current position, take into consideration the bytes we read but the consumer
		//doesn't know about
		err = nil
		for a.cur != nil {
			if err = a.fill(); err == nil && a.cur != nil {
				offset -= int64(len(a.cur.buffer()))
				a.cur.offset = len(a.cur.buf)
			}
		}
	}
	//Seek the actual Seeker
	if res, err = seeker.Seek(offset, whence); err == nil {
		//If the seek was successful, reinitialize ourselves (with the new position).
		a.initBuffers(a.in, a.bufs, a.size)
	}
	return
}

// WriteTo writes data to w until there's no more data to write or when an error occurs.
// The return value n is the number of bytes written.
// Any error encountered during the write is also returned.
func (a *reader) WriteTo(w io.Writer) (n int64, err error) {
	if a.err != nil {
		return 0, a.err
	}
	n = 0
	for {
		err = a.fill()
		if err != nil {
			return n, err
		}
		n2, err := w.Write(a.cur.buffer())
		a.cur.inc(n2)
		n += int64(n2)
		if err != nil {
			return n, err
		}
		if a.cur.err != nil {
			// io.Writer should return nil if we are at EOF.
			if a.cur.err == io.EOF {
				a.err = a.cur.err
				return n, nil
			}
			a.err = a.cur.err
			return n, a.cur.err
		}
	}
}

// Close will ensure that the underlying async reader is shut down.
// It will also close the input supplied on newAsyncReader.
func (a *reader) Close() (err error) {
	select {
	case <-a.exited:
	case a.exit <- struct{}{}:
		<-a.exited
	}
	if a.closer != nil {
		// Only call once
		c := a.closer
		a.closer = nil
		return c.Close()
	}
	a.err = errors.New("readahead: read after Close")
	return nil
}

// Internal buffer representing a single read.
// If an error is present, it must be returned
// once all buffer content has been served.
type buffer struct {
	buf    []byte
	err    error
	offset int
	size   int
}

func newBuffer(buf []byte) *buffer {
	return &buffer{buf: buf, err: nil, size: len(buf)}
}

// isEmpty returns true is offset is at end of
// buffer, or if the buffer is nil
func (b *buffer) isEmpty() bool {
	if b == nil {
		return true
	}
	if len(b.buf)-b.offset <= 0 {
		return true
	}
	return false
}

// read into start of the buffer from the supplied reader,
// resets the offset and updates the size of the buffer.
// Any error encountered during the read is returned.
func (b *buffer) read(rd io.Reader) (err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("panic reading: %v", r)
			b.err = err
		}
	}()

	var n int
	buf := b.buf[0:b.size]
	for n < b.size {
		n2, err := rd.Read(buf)
		n += n2
		if err != nil {
			b.err = err
			break
		}
		buf = buf[n2:]
	}
	b.buf = b.buf[0:n]
	b.offset = 0
	return b.err
}

// Return the buffer at current offset
func (b *buffer) buffer() []byte {
	return b.buf[b.offset:]
}

// inc will increment the read offset
func (b *buffer) inc(n int) {
	b.offset += n
}
