// +build !js

package websocket

import (
	"io"
	"net/http"
	"sync"

	"github.com/klauspost/compress/flate"
)

func (m CompressionMode) opts() *compressionOptions {
	return &compressionOptions{
		clientNoContextTakeover: m == CompressionNoContextTakeover,
		serverNoContextTakeover: m == CompressionNoContextTakeover,
	}
}

type compressionOptions struct {
	clientNoContextTakeover bool
	serverNoContextTakeover bool
}

func (copts *compressionOptions) setHeader(h http.Header) {
	s := "permessage-deflate"
	if copts.clientNoContextTakeover {
		s += "; client_no_context_takeover"
	}
	if copts.serverNoContextTakeover {
		s += "; server_no_context_takeover"
	}
	h.Set("Sec-WebSocket-Extensions", s)
}

// These bytes are required to get flate.Reader to return.
// They are removed when sending to avoid the overhead as
// WebSocket framing tell's when the message has ended but then
// we need to add them back otherwise flate.Reader keeps
// trying to return more bytes.
const deflateMessageTail = "\x00\x00\xff\xff"

type trimLastFourBytesWriter struct {
	w    io.Writer
	tail []byte
}

func (tw *trimLastFourBytesWriter) reset() {
	if tw != nil && tw.tail != nil {
		tw.tail = tw.tail[:0]
	}
}

func (tw *trimLastFourBytesWriter) Write(p []byte) (int, error) {
	if tw.tail == nil {
		tw.tail = make([]byte, 0, 4)
	}

	extra := len(tw.tail) + len(p) - 4

	if extra <= 0 {
		tw.tail = append(tw.tail, p...)
		return len(p), nil
	}

	// Now we need to write as many extra bytes as we can from the previous tail.
	if extra > len(tw.tail) {
		extra = len(tw.tail)
	}
	if extra > 0 {
		_, err := tw.w.Write(tw.tail[:extra])
		if err != nil {
			return 0, err
		}

		// Shift remaining bytes in tail over.
		n := copy(tw.tail, tw.tail[extra:])
		tw.tail = tw.tail[:n]
	}

	// If p is less than or equal to 4 bytes,
	// all of it is is part of the tail.
	if len(p) <= 4 {
		tw.tail = append(tw.tail, p...)
		return len(p), nil
	}

	// Otherwise, only the last 4 bytes are.
	tw.tail = append(tw.tail, p[len(p)-4:]...)

	p = p[:len(p)-4]
	n, err := tw.w.Write(p)
	return n + 4, err
}

var flateReaderPool sync.Pool

func getFlateReader(r io.Reader, dict []byte) io.Reader {
	fr, ok := flateReaderPool.Get().(io.Reader)
	if !ok {
		return flate.NewReaderDict(r, dict)
	}
	fr.(flate.Resetter).Reset(r, dict)
	return fr
}

func putFlateReader(fr io.Reader) {
	flateReaderPool.Put(fr)
}

type slidingWindow struct {
	buf []byte
}

var swPoolMu sync.RWMutex
var swPool = map[int]*sync.Pool{}

func slidingWindowPool(n int) *sync.Pool {
	swPoolMu.RLock()
	p, ok := swPool[n]
	swPoolMu.RUnlock()
	if ok {
		return p
	}

	p = &sync.Pool{}

	swPoolMu.Lock()
	swPool[n] = p
	swPoolMu.Unlock()

	return p
}

func (sw *slidingWindow) init(n int) {
	if sw.buf != nil {
		return
	}

	if n == 0 {
		n = 32768
	}

	p := slidingWindowPool(n)
	buf, ok := p.Get().([]byte)
	if ok {
		sw.buf = buf[:0]
	} else {
		sw.buf = make([]byte, 0, n)
	}
}

func (sw *slidingWindow) close() {
	if sw.buf == nil {
		return
	}

	swPoolMu.Lock()
	swPool[cap(sw.buf)].Put(sw.buf)
	swPoolMu.Unlock()
	sw.buf = nil
}

func (sw *slidingWindow) write(p []byte) {
	if len(p) >= cap(sw.buf) {
		sw.buf = sw.buf[:cap(sw.buf)]
		p = p[len(p)-cap(sw.buf):]
		copy(sw.buf, p)
		return
	}

	left := cap(sw.buf) - len(sw.buf)
	if left < len(p) {
		// We need to shift spaceNeeded bytes from the end to make room for p at the end.
		spaceNeeded := len(p) - left
		copy(sw.buf, sw.buf[spaceNeeded:])
		sw.buf = sw.buf[:len(sw.buf)-spaceNeeded]
	}

	sw.buf = append(sw.buf, p...)
}
