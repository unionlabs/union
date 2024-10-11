// Copyright 2013 The Gorilla WebSocket Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package websocket

import (
	"bufio"
	"bytes"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"net"
	"reflect"
	"sync"
	"testing"
	"testing/iotest"
	"time"
)

var _ net.Error = errWriteTimeout

type fakeNetConn struct {
	io.Reader
	io.Writer
}

func (c fakeNetConn) Close() error                       { return nil }
func (c fakeNetConn) LocalAddr() net.Addr                { return localAddr }
func (c fakeNetConn) RemoteAddr() net.Addr               { return remoteAddr }
func (c fakeNetConn) SetDeadline(t time.Time) error      { return nil }
func (c fakeNetConn) SetReadDeadline(t time.Time) error  { return nil }
func (c fakeNetConn) SetWriteDeadline(t time.Time) error { return nil }

type fakeAddr int

var (
	localAddr  = fakeAddr(1)
	remoteAddr = fakeAddr(2)
)

func (a fakeAddr) Network() string {
	return "net"
}

func (a fakeAddr) String() string {
	return "str"
}

// newTestConn creates a connnection backed by a fake network connection using
// default values for buffering.
func newTestConn(r io.Reader, w io.Writer, isServer bool) *Conn {
	return newConn(fakeNetConn{Reader: r, Writer: w}, isServer, 1024, 1024, nil, nil, nil)
}

func TestFraming(t *testing.T) {
	frameSizes := []int{
		0, 1, 2, 124, 125, 126, 127, 128, 129, 65534, 65535,
		// 65536, 65537
	}
	var readChunkers = []struct {
		name string
		f    func(io.Reader) io.Reader
	}{
		{"half", iotest.HalfReader},
		{"one", iotest.OneByteReader},
		{"asis", func(r io.Reader) io.Reader { return r }},
	}
	writeBuf := make([]byte, 65537)
	for i := range writeBuf {
		writeBuf[i] = byte(i)
	}
	var writers = []struct {
		name string
		f    func(w io.Writer, n int) (int, error)
	}{
		{"iocopy", func(w io.Writer, n int) (int, error) {
			nn, err := io.Copy(w, bytes.NewReader(writeBuf[:n]))
			return int(nn), err
		}},
		{"write", func(w io.Writer, n int) (int, error) {
			return w.Write(writeBuf[:n])
		}},
		{"string", func(w io.Writer, n int) (int, error) {
			return io.WriteString(w, string(writeBuf[:n]))
		}},
	}

	for _, compress := range []bool{false, true} {
		for _, isServer := range []bool{true, false} {
			for _, chunker := range readChunkers {

				var connBuf bytes.Buffer
				wc := newTestConn(nil, &connBuf, isServer)
				rc := newTestConn(chunker.f(&connBuf), nil, !isServer)
				if compress {
					wc.newCompressionWriter = compressNoContextTakeover
					rc.newDecompressionReader = decompressNoContextTakeover
				}
				for _, n := range frameSizes {
					for _, writer := range writers {
						name := fmt.Sprintf("z:%v, s:%v, r:%s, n:%d w:%s", compress, isServer, chunker.name, n, writer.name)

						w, err := wc.NextWriter(TextMessage)
						if err != nil {
							t.Errorf("%s: wc.NextWriter() returned %v", name, err)
							continue
						}
						nn, err := writer.f(w, n)
						if err != nil || nn != n {
							t.Errorf("%s: w.Write(writeBuf[:n]) returned %d, %v", name, nn, err)
							continue
						}
						err = w.Close()
						if err != nil {
							t.Errorf("%s: w.Close() returned %v", name, err)
							continue
						}

						opCode, r, err := rc.NextReader()
						if err != nil || opCode != TextMessage {
							t.Errorf("%s: NextReader() returned %d, r, %v", name, opCode, err)
							continue
						}

						t.Logf("frame size: %d", n)
						rbuf, err := ioutil.ReadAll(r)
						if err != nil {
							t.Errorf("%s: ReadFull() returned rbuf, %v", name, err)
							continue
						}

						if len(rbuf) != n {
							t.Errorf("%s: len(rbuf) is %d, want %d", name, len(rbuf), n)
							continue
						}

						for i, b := range rbuf {
							if byte(i) != b {
								t.Errorf("%s: bad byte at offset %d", name, i)
								break
							}
						}
					}
				}
			}
		}
	}
}

func TestControl(t *testing.T) {
	const message = "this is a ping/pong messsage"
	for _, isServer := range []bool{true, false} {
		for _, isWriteControl := range []bool{true, false} {
			name := fmt.Sprintf("s:%v, wc:%v", isServer, isWriteControl)
			var connBuf bytes.Buffer
			wc := newTestConn(nil, &connBuf, isServer)
			rc := newTestConn(&connBuf, nil, !isServer)
			if isWriteControl {
				wc.WriteControl(PongMessage, []byte(message), time.Now().Add(time.Second))
			} else {
				w, err := wc.NextWriter(PongMessage)
				if err != nil {
					t.Errorf("%s: wc.NextWriter() returned %v", name, err)
					continue
				}
				if _, err := w.Write([]byte(message)); err != nil {
					t.Errorf("%s: w.Write() returned %v", name, err)
					continue
				}
				if err := w.Close(); err != nil {
					t.Errorf("%s: w.Close() returned %v", name, err)
					continue
				}
				var actualMessage string
				rc.SetPongHandler(func(s string) error { actualMessage = s; return nil })
				rc.NextReader()
				if actualMessage != message {
					t.Errorf("%s: pong=%q, want %q", name, actualMessage, message)
					continue
				}
			}
		}
	}
}

// simpleBufferPool is an implementation of BufferPool for TestWriteBufferPool.
type simpleBufferPool struct {
	v interface{}
}

func (p *simpleBufferPool) Get() interface{} {
	v := p.v
	p.v = nil
	return v
}

func (p *simpleBufferPool) Put(v interface{}) {
	p.v = v
}

func TestWriteBufferPool(t *testing.T) {
	const message = "Now is the time for all good people to come to the aid of the party."

	var buf bytes.Buffer
	var pool simpleBufferPool
	rc := newTestConn(&buf, nil, false)

	// Specify writeBufferSize smaller than message size to ensure that pooling
	// works with fragmented messages.
	wc := newConn(fakeNetConn{Writer: &buf}, true, 1024, len(message)-1, &pool, nil, nil)

	if wc.writeBuf != nil {
		t.Fatal("writeBuf not nil after create")
	}

	// Part 1: test NextWriter/Write/Close

	w, err := wc.NextWriter(TextMessage)
	if err != nil {
		t.Fatalf("wc.NextWriter() returned %v", err)
	}

	if wc.writeBuf == nil {
		t.Fatal("writeBuf is nil after NextWriter")
	}

	writeBufAddr := &wc.writeBuf[0]

	if _, err := io.WriteString(w, message); err != nil {
		t.Fatalf("io.WriteString(w, message) returned %v", err)
	}

	if err := w.Close(); err != nil {
		t.Fatalf("w.Close() returned %v", err)
	}

	if wc.writeBuf != nil {
		t.Fatal("writeBuf not nil after w.Close()")
	}

	if wpd, ok := pool.v.(writePoolData); !ok || len(wpd.buf) == 0 || &wpd.buf[0] != writeBufAddr {
		t.Fatal("writeBuf not returned to pool")
	}

	opCode, p, err := rc.ReadMessage()
	if opCode != TextMessage || err != nil {
		t.Fatalf("ReadMessage() returned %d, p, %v", opCode, err)
	}

	if s := string(p); s != message {
		t.Fatalf("message is %s, want %s", s, message)
	}

	// Part 2: Test WriteMessage.

	if err := wc.WriteMessage(TextMessage, []byte(message)); err != nil {
		t.Fatalf("wc.WriteMessage() returned %v", err)
	}

	if wc.writeBuf != nil {
		t.Fatal("writeBuf not nil after wc.WriteMessage()")
	}

	if wpd, ok := pool.v.(writePoolData); !ok || len(wpd.buf) == 0 || &wpd.buf[0] != writeBufAddr {
		t.Fatal("writeBuf not returned to pool after WriteMessage")
	}

	opCode, p, err = rc.ReadMessage()
	if opCode != TextMessage || err != nil {
		t.Fatalf("ReadMessage() returned %d, p, %v", opCode, err)
	}

	if s := string(p); s != message {
		t.Fatalf("message is %s, want %s", s, message)
	}
}

// TestWriteBufferPoolSync ensures that *sync.Pool works as a buffer pool.
func TestWriteBufferPoolSync(t *testing.T) {
	var buf bytes.Buffer
	var pool sync.Pool
	wc := newConn(fakeNetConn{Writer: &buf}, true, 1024, 1024, &pool, nil, nil)
	rc := newTestConn(&buf, nil, false)

	const message = "Hello World!"
	for i := 0; i < 3; i++ {
		if err := wc.WriteMessage(TextMessage, []byte(message)); err != nil {
			t.Fatalf("wc.WriteMessage() returned %v", err)
		}
		opCode, p, err := rc.ReadMessage()
		if opCode != TextMessage || err != nil {
			t.Fatalf("ReadMessage() returned %d, p, %v", opCode, err)
		}
		if s := string(p); s != message {
			t.Fatalf("message is %s, want %s", s, message)
		}
	}
}

// errorWriter is an io.Writer than returns an error on all writes.
type errorWriter struct{}

func (ew errorWriter) Write(p []byte) (int, error) { return 0, errors.New("error") }

// TestWriteBufferPoolError ensures that buffer is returned to pool after error
// on write.
func TestWriteBufferPoolError(t *testing.T) {

	// Part 1: Test NextWriter/Write/Close

	var pool simpleBufferPool
	wc := newConn(fakeNetConn{Writer: errorWriter{}}, true, 1024, 1024, &pool, nil, nil)

	w, err := wc.NextWriter(TextMessage)
	if err != nil {
		t.Fatalf("wc.NextWriter() returned %v", err)
	}

	if wc.writeBuf == nil {
		t.Fatal("writeBuf is nil after NextWriter")
	}

	writeBufAddr := &wc.writeBuf[0]

	if _, err := io.WriteString(w, "Hello"); err != nil {
		t.Fatalf("io.WriteString(w, message) returned %v", err)
	}

	if err := w.Close(); err == nil {
		t.Fatalf("w.Close() did not return error")
	}

	if wpd, ok := pool.v.(writePoolData); !ok || len(wpd.buf) == 0 || &wpd.buf[0] != writeBufAddr {
		t.Fatal("writeBuf not returned to pool")
	}

	// Part 2: Test WriteMessage

	wc = newConn(fakeNetConn{Writer: errorWriter{}}, true, 1024, 1024, &pool, nil, nil)

	if err := wc.WriteMessage(TextMessage, []byte("Hello")); err == nil {
		t.Fatalf("wc.WriteMessage did not return error")
	}

	if wpd, ok := pool.v.(writePoolData); !ok || len(wpd.buf) == 0 || &wpd.buf[0] != writeBufAddr {
		t.Fatal("writeBuf not returned to pool")
	}
}

func TestCloseFrameBeforeFinalMessageFrame(t *testing.T) {
	const bufSize = 512

	expectedErr := &CloseError{Code: CloseNormalClosure, Text: "hello"}

	var b1, b2 bytes.Buffer
	wc := newConn(&fakeNetConn{Reader: nil, Writer: &b1}, false, 1024, bufSize, nil, nil, nil)
	rc := newTestConn(&b1, &b2, true)

	w, _ := wc.NextWriter(BinaryMessage)
	w.Write(make([]byte, bufSize+bufSize/2))
	wc.WriteControl(CloseMessage, FormatCloseMessage(expectedErr.Code, expectedErr.Text), time.Now().Add(10*time.Second))
	w.Close()

	op, r, err := rc.NextReader()
	if op != BinaryMessage || err != nil {
		t.Fatalf("NextReader() returned %d, %v", op, err)
	}
	_, err = io.Copy(ioutil.Discard, r)
	if !reflect.DeepEqual(err, expectedErr) {
		t.Fatalf("io.Copy() returned %v, want %v", err, expectedErr)
	}
	_, _, err = rc.NextReader()
	if !reflect.DeepEqual(err, expectedErr) {
		t.Fatalf("NextReader() returned %v, want %v", err, expectedErr)
	}
}

func TestEOFWithinFrame(t *testing.T) {
	const bufSize = 64

	for n := 0; ; n++ {
		var b bytes.Buffer
		wc := newTestConn(nil, &b, false)
		rc := newTestConn(&b, nil, true)

		w, _ := wc.NextWriter(BinaryMessage)
		w.Write(make([]byte, bufSize))
		w.Close()

		if n >= b.Len() {
			break
		}
		b.Truncate(n)

		op, r, err := rc.NextReader()
		if err == errUnexpectedEOF {
			continue
		}
		if op != BinaryMessage || err != nil {
			t.Fatalf("%d: NextReader() returned %d, %v", n, op, err)
		}
		_, err = io.Copy(ioutil.Discard, r)
		if err != errUnexpectedEOF {
			t.Fatalf("%d: io.Copy() returned %v, want %v", n, err, errUnexpectedEOF)
		}
		_, _, err = rc.NextReader()
		if err != errUnexpectedEOF {
			t.Fatalf("%d: NextReader() returned %v, want %v", n, err, errUnexpectedEOF)
		}
	}
}

func TestEOFBeforeFinalFrame(t *testing.T) {
	const bufSize = 512

	var b1, b2 bytes.Buffer
	wc := newConn(&fakeNetConn{Writer: &b1}, false, 1024, bufSize, nil, nil, nil)
	rc := newTestConn(&b1, &b2, true)

	w, _ := wc.NextWriter(BinaryMessage)
	w.Write(make([]byte, bufSize+bufSize/2))

	op, r, err := rc.NextReader()
	if op != BinaryMessage || err != nil {
		t.Fatalf("NextReader() returned %d, %v", op, err)
	}
	_, err = io.Copy(ioutil.Discard, r)
	if err != errUnexpectedEOF {
		t.Fatalf("io.Copy() returned %v, want %v", err, errUnexpectedEOF)
	}
	_, _, err = rc.NextReader()
	if err != errUnexpectedEOF {
		t.Fatalf("NextReader() returned %v, want %v", err, errUnexpectedEOF)
	}
}

func TestWriteAfterMessageWriterClose(t *testing.T) {
	wc := newTestConn(nil, &bytes.Buffer{}, false)
	w, _ := wc.NextWriter(BinaryMessage)
	io.WriteString(w, "hello")
	if err := w.Close(); err != nil {
		t.Fatalf("unxpected error closing message writer, %v", err)
	}

	if _, err := io.WriteString(w, "world"); err == nil {
		t.Fatalf("no error writing after close")
	}

	w, _ = wc.NextWriter(BinaryMessage)
	io.WriteString(w, "hello")

	// close w by getting next writer
	_, err := wc.NextWriter(BinaryMessage)
	if err != nil {
		t.Fatalf("unexpected error getting next writer, %v", err)
	}

	if _, err := io.WriteString(w, "world"); err == nil {
		t.Fatalf("no error writing after close")
	}
}

func TestReadLimit(t *testing.T) {
	t.Run("Test ReadLimit is enforced", func(t *testing.T) {
		const readLimit = 512
		message := make([]byte, readLimit+1)

		var b1, b2 bytes.Buffer
		wc := newConn(&fakeNetConn{Writer: &b1}, false, 1024, readLimit-2, nil, nil, nil)
		rc := newTestConn(&b1, &b2, true)
		rc.SetReadLimit(readLimit)

		// Send message at the limit with interleaved pong.
		w, _ := wc.NextWriter(BinaryMessage)
		w.Write(message[:readLimit-1])
		wc.WriteControl(PongMessage, []byte("this is a pong"), time.Now().Add(10*time.Second))
		w.Write(message[:1])
		w.Close()

		// Send message larger than the limit.
		wc.WriteMessage(BinaryMessage, message[:readLimit+1])

		op, _, err := rc.NextReader()
		if op != BinaryMessage || err != nil {
			t.Fatalf("1: NextReader() returned %d, %v", op, err)
		}
		op, r, err := rc.NextReader()
		if op != BinaryMessage || err != nil {
			t.Fatalf("2: NextReader() returned %d, %v", op, err)
		}
		_, err = io.Copy(ioutil.Discard, r)
		if err != ErrReadLimit {
			t.Fatalf("io.Copy() returned %v", err)
		}
	})

	t.Run("Test that ReadLimit cannot be overflowed", func(t *testing.T) {
		const readLimit = 1

		var b1, b2 bytes.Buffer
		rc := newTestConn(&b1, &b2, true)
		rc.SetReadLimit(readLimit)

		// First, send a non-final binary message
		b1.Write([]byte("\x02\x81"))

		// Mask key
		b1.Write([]byte("\x00\x00\x00\x00"))

		// First payload
		b1.Write([]byte("A"))

		// Next, send a negative-length, non-final continuation frame
		b1.Write([]byte("\x00\xFF\x80\x00\x00\x00\x00\x00\x00\x00"))

		// Mask key
		b1.Write([]byte("\x00\x00\x00\x00"))

		// Next, send a too long, final continuation frame
		b1.Write([]byte("\x80\xFF\x00\x00\x00\x00\x00\x00\x00\x05"))

		// Mask key
		b1.Write([]byte("\x00\x00\x00\x00"))

		// Too-long payload
		b1.Write([]byte("BCDEF"))

		op, r, err := rc.NextReader()
		if op != BinaryMessage || err != nil {
			t.Fatalf("1: NextReader() returned %d, %v", op, err)
		}

		var buf [10]byte
		var read int
		n, err := r.Read(buf[:])
		if err != nil && err != ErrReadLimit {
			t.Fatalf("unexpected error testing read limit: %v", err)
		}
		read += n

		n, err = r.Read(buf[:])
		if err != nil && err != ErrReadLimit {
			t.Fatalf("unexpected error testing read limit: %v", err)
		}
		read += n

		if err == nil && read > readLimit {
			t.Fatalf("read limit exceeded: limit %d, read %d", readLimit, read)
		}
	})
}

func TestAddrs(t *testing.T) {
	c := newTestConn(nil, nil, true)
	if c.LocalAddr() != localAddr {
		t.Errorf("LocalAddr = %v, want %v", c.LocalAddr(), localAddr)
	}
	if c.RemoteAddr() != remoteAddr {
		t.Errorf("RemoteAddr = %v, want %v", c.RemoteAddr(), remoteAddr)
	}
}

func TestDeprecatedUnderlyingConn(t *testing.T) {
	var b1, b2 bytes.Buffer
	fc := fakeNetConn{Reader: &b1, Writer: &b2}
	c := newConn(fc, true, 1024, 1024, nil, nil, nil)
	ul := c.UnderlyingConn()
	if ul != fc {
		t.Fatalf("Underlying conn is not what it should be.")
	}
}

func TestNetConn(t *testing.T) {
	var b1, b2 bytes.Buffer
	fc := fakeNetConn{Reader: &b1, Writer: &b2}
	c := newConn(fc, true, 1024, 1024, nil, nil, nil)
	ul := c.NetConn()
	if ul != fc {
		t.Fatalf("Underlying conn is not what it should be.")
	}
}

func TestBufioReadBytes(t *testing.T) {
	// Test calling bufio.ReadBytes for value longer than read buffer size.

	m := make([]byte, 512)
	m[len(m)-1] = '\n'

	var b1, b2 bytes.Buffer
	wc := newConn(fakeNetConn{Writer: &b1}, false, len(m)+64, len(m)+64, nil, nil, nil)
	rc := newConn(fakeNetConn{Reader: &b1, Writer: &b2}, true, len(m)-64, len(m)-64, nil, nil, nil)

	w, _ := wc.NextWriter(BinaryMessage)
	w.Write(m)
	w.Close()

	op, r, err := rc.NextReader()
	if op != BinaryMessage || err != nil {
		t.Fatalf("NextReader() returned %d, %v", op, err)
	}

	br := bufio.NewReader(r)
	p, err := br.ReadBytes('\n')
	if err != nil {
		t.Fatalf("ReadBytes() returned %v", err)
	}
	if len(p) != len(m) {
		t.Fatalf("read returned %d bytes, want %d bytes", len(p), len(m))
	}
}

var closeErrorTests = []struct {
	err   error
	codes []int
	ok    bool
}{
	{&CloseError{Code: CloseNormalClosure}, []int{CloseNormalClosure}, true},
	{&CloseError{Code: CloseNormalClosure}, []int{CloseNoStatusReceived}, false},
	{&CloseError{Code: CloseNormalClosure}, []int{CloseNoStatusReceived, CloseNormalClosure}, true},
	{errors.New("hello"), []int{CloseNormalClosure}, false},
}

func TestCloseError(t *testing.T) {
	for _, tt := range closeErrorTests {
		ok := IsCloseError(tt.err, tt.codes...)
		if ok != tt.ok {
			t.Errorf("IsCloseError(%#v, %#v) returned %v, want %v", tt.err, tt.codes, ok, tt.ok)
		}
	}
}

var unexpectedCloseErrorTests = []struct {
	err   error
	codes []int
	ok    bool
}{
	{&CloseError{Code: CloseNormalClosure}, []int{CloseNormalClosure}, false},
	{&CloseError{Code: CloseNormalClosure}, []int{CloseNoStatusReceived}, true},
	{&CloseError{Code: CloseNormalClosure}, []int{CloseNoStatusReceived, CloseNormalClosure}, false},
	{errors.New("hello"), []int{CloseNormalClosure}, false},
}

func TestUnexpectedCloseErrors(t *testing.T) {
	for _, tt := range unexpectedCloseErrorTests {
		ok := IsUnexpectedCloseError(tt.err, tt.codes...)
		if ok != tt.ok {
			t.Errorf("IsUnexpectedCloseError(%#v, %#v) returned %v, want %v", tt.err, tt.codes, ok, tt.ok)
		}
	}
}

type blockingWriter struct {
	c1, c2 chan struct{}
}

func (w blockingWriter) Write(p []byte) (int, error) {
	// Allow main to continue
	close(w.c1)
	// Wait for panic in main
	<-w.c2
	return len(p), nil
}

func TestConcurrentWritePanic(t *testing.T) {
	w := blockingWriter{make(chan struct{}), make(chan struct{})}
	c := newTestConn(nil, w, false)
	go func() {
		c.WriteMessage(TextMessage, []byte{})
	}()

	// wait for goroutine to block in write.
	<-w.c1

	defer func() {
		close(w.c2)
		if v := recover(); v != nil {
			return
		}
	}()

	c.WriteMessage(TextMessage, []byte{})
	t.Fatal("should not get here")
}

type failingReader struct{}

func (r failingReader) Read(p []byte) (int, error) {
	return 0, io.EOF
}

func TestFailedConnectionReadPanic(t *testing.T) {
	c := newTestConn(failingReader{}, nil, false)

	defer func() {
		if v := recover(); v != nil {
			return
		}
	}()

	for i := 0; i < 20000; i++ {
		c.ReadMessage()
	}
	t.Fatal("should not get here")
}
