// +build !js

package websocket

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"runtime"
	"strconv"
	"sync"
	"sync/atomic"
)

// Conn represents a WebSocket connection.
// All methods may be called concurrently except for Reader and Read.
//
// You must always read from the connection. Otherwise control
// frames will not be handled. See Reader and CloseRead.
//
// Be sure to call Close on the connection when you
// are finished with it to release associated resources.
//
// On any error from any method, the connection is closed
// with an appropriate reason.
type Conn struct {
	subprotocol    string
	rwc            io.ReadWriteCloser
	client         bool
	copts          *compressionOptions
	flateThreshold int
	br             *bufio.Reader
	bw             *bufio.Writer

	readTimeout  chan context.Context
	writeTimeout chan context.Context

	// Read state.
	readMu            *mu
	readHeaderBuf     [8]byte
	readControlBuf    [maxControlPayload]byte
	msgReader         *msgReader
	readCloseFrameErr error

	// Write state.
	msgWriterState *msgWriterState
	writeFrameMu   *mu
	writeBuf       []byte
	writeHeaderBuf [8]byte
	writeHeader    header

	closed     chan struct{}
	closeMu    sync.Mutex
	closeErr   error
	wroteClose bool

	pingCounter   int32
	activePingsMu sync.Mutex
	activePings   map[string]chan<- struct{}
}

type connConfig struct {
	subprotocol    string
	rwc            io.ReadWriteCloser
	client         bool
	copts          *compressionOptions
	flateThreshold int

	br *bufio.Reader
	bw *bufio.Writer
}

func newConn(cfg connConfig) *Conn {
	c := &Conn{
		subprotocol:    cfg.subprotocol,
		rwc:            cfg.rwc,
		client:         cfg.client,
		copts:          cfg.copts,
		flateThreshold: cfg.flateThreshold,

		br: cfg.br,
		bw: cfg.bw,

		readTimeout:  make(chan context.Context),
		writeTimeout: make(chan context.Context),

		closed:      make(chan struct{}),
		activePings: make(map[string]chan<- struct{}),
	}

	c.readMu = newMu(c)
	c.writeFrameMu = newMu(c)

	c.msgReader = newMsgReader(c)

	c.msgWriterState = newMsgWriterState(c)
	if c.client {
		c.writeBuf = extractBufioWriterBuf(c.bw, c.rwc)
	}

	if c.flate() && c.flateThreshold == 0 {
		c.flateThreshold = 128
		if !c.msgWriterState.flateContextTakeover() {
			c.flateThreshold = 512
		}
	}

	runtime.SetFinalizer(c, func(c *Conn) {
		c.close(errors.New("connection garbage collected"))
	})

	go c.timeoutLoop()

	return c
}

// Subprotocol returns the negotiated subprotocol.
// An empty string means the default protocol.
func (c *Conn) Subprotocol() string {
	return c.subprotocol
}

func (c *Conn) close(err error) {
	c.closeMu.Lock()
	defer c.closeMu.Unlock()

	if c.isClosed() {
		return
	}
	c.setCloseErrLocked(err)
	close(c.closed)
	runtime.SetFinalizer(c, nil)

	// Have to close after c.closed is closed to ensure any goroutine that wakes up
	// from the connection being closed also sees that c.closed is closed and returns
	// closeErr.
	c.rwc.Close()

	go func() {
		c.msgWriterState.close()

		c.msgReader.close()
	}()
}

func (c *Conn) timeoutLoop() {
	readCtx := context.Background()
	writeCtx := context.Background()

	for {
		select {
		case <-c.closed:
			return

		case writeCtx = <-c.writeTimeout:
		case readCtx = <-c.readTimeout:

		case <-readCtx.Done():
			c.setCloseErr(fmt.Errorf("read timed out: %w", readCtx.Err()))
			go c.writeError(StatusPolicyViolation, errors.New("timed out"))
		case <-writeCtx.Done():
			c.close(fmt.Errorf("write timed out: %w", writeCtx.Err()))
			return
		}
	}
}

func (c *Conn) flate() bool {
	return c.copts != nil
}

// Ping sends a ping to the peer and waits for a pong.
// Use this to measure latency or ensure the peer is responsive.
// Ping must be called concurrently with Reader as it does
// not read from the connection but instead waits for a Reader call
// to read the pong.
//
// TCP Keepalives should suffice for most use cases.
func (c *Conn) Ping(ctx context.Context) error {
	p := atomic.AddInt32(&c.pingCounter, 1)

	err := c.ping(ctx, strconv.Itoa(int(p)))
	if err != nil {
		return fmt.Errorf("failed to ping: %w", err)
	}
	return nil
}

func (c *Conn) ping(ctx context.Context, p string) error {
	pong := make(chan struct{}, 1)

	c.activePingsMu.Lock()
	c.activePings[p] = pong
	c.activePingsMu.Unlock()

	defer func() {
		c.activePingsMu.Lock()
		delete(c.activePings, p)
		c.activePingsMu.Unlock()
	}()

	err := c.writeControl(ctx, opPing, []byte(p))
	if err != nil {
		return err
	}

	select {
	case <-c.closed:
		return c.closeErr
	case <-ctx.Done():
		err := fmt.Errorf("failed to wait for pong: %w", ctx.Err())
		c.close(err)
		return err
	case <-pong:
		return nil
	}
}

type mu struct {
	c  *Conn
	ch chan struct{}
}

func newMu(c *Conn) *mu {
	return &mu{
		c:  c,
		ch: make(chan struct{}, 1),
	}
}

func (m *mu) forceLock() {
	m.ch <- struct{}{}
}

func (m *mu) lock(ctx context.Context) error {
	select {
	case <-m.c.closed:
		return m.c.closeErr
	case <-ctx.Done():
		err := fmt.Errorf("failed to acquire lock: %w", ctx.Err())
		m.c.close(err)
		return err
	case m.ch <- struct{}{}:
		// To make sure the connection is certainly alive.
		// As it's possible the send on m.ch was selected
		// over the receive on closed.
		select {
		case <-m.c.closed:
			// Make sure to release.
			m.unlock()
			return m.c.closeErr
		default:
		}
		return nil
	}
}

func (m *mu) unlock() {
	select {
	case <-m.ch:
	default:
	}
}
