// +build !js

package websocket

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"strings"
	"time"

	"nhooyr.io/websocket/internal/errd"
	"nhooyr.io/websocket/internal/xsync"
)

// Reader reads from the connection until until there is a WebSocket
// data message to be read. It will handle ping, pong and close frames as appropriate.
//
// It returns the type of the message and an io.Reader to read it.
// The passed context will also bound the reader.
// Ensure you read to EOF otherwise the connection will hang.
//
// Call CloseRead if you do not expect any data messages from the peer.
//
// Only one Reader may be open at a time.
func (c *Conn) Reader(ctx context.Context) (MessageType, io.Reader, error) {
	return c.reader(ctx)
}

// Read is a convenience method around Reader to read a single message
// from the connection.
func (c *Conn) Read(ctx context.Context) (MessageType, []byte, error) {
	typ, r, err := c.Reader(ctx)
	if err != nil {
		return 0, nil, err
	}

	b, err := ioutil.ReadAll(r)
	return typ, b, err
}

// CloseRead starts a goroutine to read from the connection until it is closed
// or a data message is received.
//
// Once CloseRead is called you cannot read any messages from the connection.
// The returned context will be cancelled when the connection is closed.
//
// If a data message is received, the connection will be closed with StatusPolicyViolation.
//
// Call CloseRead when you do not expect to read any more messages.
// Since it actively reads from the connection, it will ensure that ping, pong and close
// frames are responded to. This means c.Ping and c.Close will still work as expected.
func (c *Conn) CloseRead(ctx context.Context) context.Context {
	ctx, cancel := context.WithCancel(ctx)
	go func() {
		defer cancel()
		c.Reader(ctx)
		c.Close(StatusPolicyViolation, "unexpected data message")
	}()
	return ctx
}

// SetReadLimit sets the max number of bytes to read for a single message.
// It applies to the Reader and Read methods.
//
// By default, the connection has a message read limit of 32768 bytes.
//
// When the limit is hit, the connection will be closed with StatusMessageTooBig.
func (c *Conn) SetReadLimit(n int64) {
	// We add read one more byte than the limit in case
	// there is a fin frame that needs to be read.
	c.msgReader.limitReader.limit.Store(n + 1)
}

const defaultReadLimit = 32768

func newMsgReader(c *Conn) *msgReader {
	mr := &msgReader{
		c:   c,
		fin: true,
	}
	mr.readFunc = mr.read

	mr.limitReader = newLimitReader(c, mr.readFunc, defaultReadLimit+1)
	return mr
}

func (mr *msgReader) resetFlate() {
	if mr.flateContextTakeover() {
		mr.dict.init(32768)
	}
	if mr.flateBufio == nil {
		mr.flateBufio = getBufioReader(mr.readFunc)
	}

	mr.flateReader = getFlateReader(mr.flateBufio, mr.dict.buf)
	mr.limitReader.r = mr.flateReader
	mr.flateTail.Reset(deflateMessageTail)
}

func (mr *msgReader) putFlateReader() {
	if mr.flateReader != nil {
		putFlateReader(mr.flateReader)
		mr.flateReader = nil
	}
}

func (mr *msgReader) close() {
	mr.c.readMu.forceLock()
	mr.putFlateReader()
	mr.dict.close()
	if mr.flateBufio != nil {
		putBufioReader(mr.flateBufio)
	}

	if mr.c.client {
		putBufioReader(mr.c.br)
		mr.c.br = nil
	}
}

func (mr *msgReader) flateContextTakeover() bool {
	if mr.c.client {
		return !mr.c.copts.serverNoContextTakeover
	}
	return !mr.c.copts.clientNoContextTakeover
}

func (c *Conn) readRSV1Illegal(h header) bool {
	// If compression is disabled, rsv1 is illegal.
	if !c.flate() {
		return true
	}
	// rsv1 is only allowed on data frames beginning messages.
	if h.opcode != opText && h.opcode != opBinary {
		return true
	}
	return false
}

func (c *Conn) readLoop(ctx context.Context) (header, error) {
	for {
		h, err := c.readFrameHeader(ctx)
		if err != nil {
			return header{}, err
		}

		if h.rsv1 && c.readRSV1Illegal(h) || h.rsv2 || h.rsv3 {
			err := fmt.Errorf("received header with unexpected rsv bits set: %v:%v:%v", h.rsv1, h.rsv2, h.rsv3)
			c.writeError(StatusProtocolError, err)
			return header{}, err
		}

		if !c.client && !h.masked {
			return header{}, errors.New("received unmasked frame from client")
		}

		switch h.opcode {
		case opClose, opPing, opPong:
			err = c.handleControl(ctx, h)
			if err != nil {
				// Pass through CloseErrors when receiving a close frame.
				if h.opcode == opClose && CloseStatus(err) != -1 {
					return header{}, err
				}
				return header{}, fmt.Errorf("failed to handle control frame %v: %w", h.opcode, err)
			}
		case opContinuation, opText, opBinary:
			return h, nil
		default:
			err := fmt.Errorf("received unknown opcode %v", h.opcode)
			c.writeError(StatusProtocolError, err)
			return header{}, err
		}
	}
}

func (c *Conn) readFrameHeader(ctx context.Context) (header, error) {
	select {
	case <-c.closed:
		return header{}, c.closeErr
	case c.readTimeout <- ctx:
	}

	h, err := readFrameHeader(c.br, c.readHeaderBuf[:])
	if err != nil {
		select {
		case <-c.closed:
			return header{}, c.closeErr
		case <-ctx.Done():
			return header{}, ctx.Err()
		default:
			c.close(err)
			return header{}, err
		}
	}

	select {
	case <-c.closed:
		return header{}, c.closeErr
	case c.readTimeout <- context.Background():
	}

	return h, nil
}

func (c *Conn) readFramePayload(ctx context.Context, p []byte) (int, error) {
	select {
	case <-c.closed:
		return 0, c.closeErr
	case c.readTimeout <- ctx:
	}

	n, err := io.ReadFull(c.br, p)
	if err != nil {
		select {
		case <-c.closed:
			return n, c.closeErr
		case <-ctx.Done():
			return n, ctx.Err()
		default:
			err = fmt.Errorf("failed to read frame payload: %w", err)
			c.close(err)
			return n, err
		}
	}

	select {
	case <-c.closed:
		return n, c.closeErr
	case c.readTimeout <- context.Background():
	}

	return n, err
}

func (c *Conn) handleControl(ctx context.Context, h header) (err error) {
	if h.payloadLength < 0 || h.payloadLength > maxControlPayload {
		err := fmt.Errorf("received control frame payload with invalid length: %d", h.payloadLength)
		c.writeError(StatusProtocolError, err)
		return err
	}

	if !h.fin {
		err := errors.New("received fragmented control frame")
		c.writeError(StatusProtocolError, err)
		return err
	}

	ctx, cancel := context.WithTimeout(ctx, time.Second*5)
	defer cancel()

	b := c.readControlBuf[:h.payloadLength]
	_, err = c.readFramePayload(ctx, b)
	if err != nil {
		return err
	}

	if h.masked {
		mask(h.maskKey, b)
	}

	switch h.opcode {
	case opPing:
		return c.writeControl(ctx, opPong, b)
	case opPong:
		c.activePingsMu.Lock()
		pong, ok := c.activePings[string(b)]
		c.activePingsMu.Unlock()
		if ok {
			close(pong)
		}
		return nil
	}

	defer func() {
		c.readCloseFrameErr = err
	}()

	ce, err := parseClosePayload(b)
	if err != nil {
		err = fmt.Errorf("received invalid close payload: %w", err)
		c.writeError(StatusProtocolError, err)
		return err
	}

	err = fmt.Errorf("received close frame: %w", ce)
	c.setCloseErr(err)
	c.writeClose(ce.Code, ce.Reason)
	c.close(err)
	return err
}

func (c *Conn) reader(ctx context.Context) (_ MessageType, _ io.Reader, err error) {
	defer errd.Wrap(&err, "failed to get reader")

	err = c.readMu.lock(ctx)
	if err != nil {
		return 0, nil, err
	}
	defer c.readMu.unlock()

	if !c.msgReader.fin {
		err = errors.New("previous message not read to completion")
		c.close(fmt.Errorf("failed to get reader: %w", err))
		return 0, nil, err
	}

	h, err := c.readLoop(ctx)
	if err != nil {
		return 0, nil, err
	}

	if h.opcode == opContinuation {
		err := errors.New("received continuation frame without text or binary frame")
		c.writeError(StatusProtocolError, err)
		return 0, nil, err
	}

	c.msgReader.reset(ctx, h)

	return MessageType(h.opcode), c.msgReader, nil
}

type msgReader struct {
	c *Conn

	ctx         context.Context
	flate       bool
	flateReader io.Reader
	flateBufio  *bufio.Reader
	flateTail   strings.Reader
	limitReader *limitReader
	dict        slidingWindow

	fin           bool
	payloadLength int64
	maskKey       uint32

	// readerFunc(mr.Read) to avoid continuous allocations.
	readFunc readerFunc
}

func (mr *msgReader) reset(ctx context.Context, h header) {
	mr.ctx = ctx
	mr.flate = h.rsv1
	mr.limitReader.reset(mr.readFunc)

	if mr.flate {
		mr.resetFlate()
	}

	mr.setFrame(h)
}

func (mr *msgReader) setFrame(h header) {
	mr.fin = h.fin
	mr.payloadLength = h.payloadLength
	mr.maskKey = h.maskKey
}

func (mr *msgReader) Read(p []byte) (n int, err error) {
	err = mr.c.readMu.lock(mr.ctx)
	if err != nil {
		return 0, fmt.Errorf("failed to read: %w", err)
	}
	defer mr.c.readMu.unlock()

	n, err = mr.limitReader.Read(p)
	if mr.flate && mr.flateContextTakeover() {
		p = p[:n]
		mr.dict.write(p)
	}
	if errors.Is(err, io.EOF) || errors.Is(err, io.ErrUnexpectedEOF) && mr.fin && mr.flate {
		mr.putFlateReader()
		return n, io.EOF
	}
	if err != nil {
		err = fmt.Errorf("failed to read: %w", err)
		mr.c.close(err)
	}
	return n, err
}

func (mr *msgReader) read(p []byte) (int, error) {
	for {
		if mr.payloadLength == 0 {
			if mr.fin {
				if mr.flate {
					return mr.flateTail.Read(p)
				}
				return 0, io.EOF
			}

			h, err := mr.c.readLoop(mr.ctx)
			if err != nil {
				return 0, err
			}
			if h.opcode != opContinuation {
				err := errors.New("received new data message without finishing the previous message")
				mr.c.writeError(StatusProtocolError, err)
				return 0, err
			}
			mr.setFrame(h)

			continue
		}

		if int64(len(p)) > mr.payloadLength {
			p = p[:mr.payloadLength]
		}

		n, err := mr.c.readFramePayload(mr.ctx, p)
		if err != nil {
			return n, err
		}

		mr.payloadLength -= int64(n)

		if !mr.c.client {
			mr.maskKey = mask(mr.maskKey, p)
		}

		return n, nil
	}
}

type limitReader struct {
	c     *Conn
	r     io.Reader
	limit xsync.Int64
	n     int64
}

func newLimitReader(c *Conn, r io.Reader, limit int64) *limitReader {
	lr := &limitReader{
		c: c,
	}
	lr.limit.Store(limit)
	lr.reset(r)
	return lr
}

func (lr *limitReader) reset(r io.Reader) {
	lr.n = lr.limit.Load()
	lr.r = r
}

func (lr *limitReader) Read(p []byte) (int, error) {
	if lr.n <= 0 {
		err := fmt.Errorf("read limited at %v bytes", lr.limit.Load())
		lr.c.writeError(StatusMessageTooBig, err)
		return 0, err
	}

	if int64(len(p)) > lr.n {
		p = p[:lr.n]
	}
	n, err := lr.r.Read(p)
	lr.n -= int64(n)
	return n, err
}

type readerFunc func(p []byte) (int, error)

func (f readerFunc) Read(p []byte) (int, error) {
	return f(p)
}
