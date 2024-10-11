/*
 *
 * Copyright 2014 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package transport

import (
	"bytes"
	"context"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math"
	"net"
	"os"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/net/http2"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/leakcheck"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/mem"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/status"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

var (
	expectedRequest            = []byte("ping")
	expectedResponse           = []byte("pong")
	expectedRequestLarge       = make([]byte, initialWindowSize*2)
	expectedResponseLarge      = make([]byte, initialWindowSize*2)
	expectedInvalidHeaderField = "invalid/content-type"
)

func init() {
	expectedRequestLarge[0] = 'g'
	expectedRequestLarge[len(expectedRequestLarge)-1] = 'r'
	expectedResponseLarge[0] = 'p'
	expectedResponseLarge[len(expectedResponseLarge)-1] = 'c'
}

func newBufferSlice(b []byte) mem.BufferSlice {
	return mem.BufferSlice{mem.NewBuffer(&b, nil)}
}

func (s *Stream) readTo(p []byte) (int, error) {
	data, err := s.Read(len(p))
	defer data.Free()

	if err != nil {
		return 0, err
	}

	if data.Len() != len(p) {
		if err == nil {
			err = io.ErrUnexpectedEOF
		}
		return 0, err
	}

	data.CopyTo(p)
	return len(p), nil
}

type testStreamHandler struct {
	t           *http2Server
	notify      chan struct{}
	getNotified chan struct{}
}

type hType int

const (
	normal hType = iota
	suspended
	notifyCall
	misbehaved
	encodingRequiredStatus
	invalidHeaderField
	delayRead
	pingpong
)

func (h *testStreamHandler) handleStreamAndNotify(*Stream) {
	if h.notify == nil {
		return
	}
	go func() {
		select {
		case <-h.notify:
		default:
			close(h.notify)
		}
	}()
}

func (h *testStreamHandler) handleStream(t *testing.T, s *Stream) {
	req := expectedRequest
	resp := expectedResponse
	if s.Method() == "foo.Large" {
		req = expectedRequestLarge
		resp = expectedResponseLarge
	}
	p := make([]byte, len(req))
	_, err := s.readTo(p)
	if err != nil {
		return
	}
	if !bytes.Equal(p, req) {
		t.Errorf("handleStream got %v, want %v", p, req)
		h.t.WriteStatus(s, status.New(codes.Internal, "panic"))
		return
	}
	// send a response back to the client.
	h.t.Write(s, nil, newBufferSlice(resp), &Options{})
	// send the trailer to end the stream.
	h.t.WriteStatus(s, status.New(codes.OK, ""))
}

func (h *testStreamHandler) handleStreamPingPong(t *testing.T, s *Stream) {
	header := make([]byte, 5)
	for {
		if _, err := s.readTo(header); err != nil {
			if err == io.EOF {
				h.t.WriteStatus(s, status.New(codes.OK, ""))
				return
			}
			t.Errorf("Error on server while reading data header: %v", err)
			h.t.WriteStatus(s, status.New(codes.Internal, "panic"))
			return
		}
		sz := binary.BigEndian.Uint32(header[1:])
		msg := make([]byte, int(sz))
		if _, err := s.readTo(msg); err != nil {
			t.Errorf("Error on server while reading message: %v", err)
			h.t.WriteStatus(s, status.New(codes.Internal, "panic"))
			return
		}
		buf := make([]byte, sz+5)
		buf[0] = byte(0)
		binary.BigEndian.PutUint32(buf[1:], uint32(sz))
		copy(buf[5:], msg)
		h.t.Write(s, nil, newBufferSlice(buf), &Options{})
	}
}

func (h *testStreamHandler) handleStreamMisbehave(t *testing.T, s *Stream) {
	conn, ok := s.st.(*http2Server)
	if !ok {
		t.Errorf("Failed to convert %v to *http2Server", s.st)
		h.t.WriteStatus(s, status.New(codes.Internal, ""))
		return
	}
	var sent int
	p := make([]byte, http2MaxFrameLen)
	for sent < initialWindowSize {
		n := initialWindowSize - sent
		// The last message may be smaller than http2MaxFrameLen
		if n <= http2MaxFrameLen {
			if s.Method() == "foo.Connection" {
				// Violate connection level flow control window of client but do not
				// violate any stream level windows.
				p = make([]byte, n)
			} else {
				// Violate stream level flow control window of client.
				p = make([]byte, n+1)
			}
		}
		data := newBufferSlice(p)
		conn.controlBuf.put(&dataFrame{
			streamID:    s.id,
			h:           nil,
			reader:      data.Reader(),
			onEachWrite: func() {},
		})
		sent += len(p)
	}
}

func (h *testStreamHandler) handleStreamEncodingRequiredStatus(s *Stream) {
	// raw newline is not accepted by http2 framer so it must be encoded.
	h.t.WriteStatus(s, encodingTestStatus)
	// Drain any remaining buffers from the stream since it was closed early.
	s.Read(math.MaxInt)
}

func (h *testStreamHandler) handleStreamInvalidHeaderField(s *Stream) {
	headerFields := []hpack.HeaderField{}
	headerFields = append(headerFields, hpack.HeaderField{Name: "content-type", Value: expectedInvalidHeaderField})
	h.t.controlBuf.put(&headerFrame{
		streamID:  s.id,
		hf:        headerFields,
		endStream: false,
	})
}

// handleStreamDelayRead delays reads so that the other side has to halt on
// stream-level flow control.
// This handler assumes dynamic flow control is turned off and assumes window
// sizes to be set to defaultWindowSize.
func (h *testStreamHandler) handleStreamDelayRead(t *testing.T, s *Stream) {
	req := expectedRequest
	resp := expectedResponse
	if s.Method() == "foo.Large" {
		req = expectedRequestLarge
		resp = expectedResponseLarge
	}
	var (
		mu    sync.Mutex
		total int
	)
	s.wq.replenish = func(n int) {
		mu.Lock()
		total += n
		mu.Unlock()
		s.wq.realReplenish(n)
	}
	getTotal := func() int {
		mu.Lock()
		defer mu.Unlock()
		return total
	}
	done := make(chan struct{})
	defer close(done)
	go func() {
		for {
			select {
			// Prevent goroutine from leaking.
			case <-done:
				return
			default:
			}
			if getTotal() == defaultWindowSize {
				// Signal the client to start reading and
				// thereby send window update.
				close(h.notify)
				return
			}
			runtime.Gosched()
		}
	}()
	p := make([]byte, len(req))

	// Let the other side run out of stream-level window before
	// starting to read and thereby sending a window update.
	timer := time.NewTimer(time.Second * 10)
	select {
	case <-h.getNotified:
		timer.Stop()
	case <-timer.C:
		t.Errorf("Server timed-out.")
		return
	}
	_, err := s.readTo(p)
	if err != nil {
		t.Errorf("s.Read(_) = _, %v, want _, <nil>", err)
		return
	}

	if !bytes.Equal(p, req) {
		t.Errorf("handleStream got %v, want %v", p, req)
		return
	}
	// This write will cause server to run out of stream level,
	// flow control and the other side won't send a window update
	// until that happens.
	if err := h.t.Write(s, nil, newBufferSlice(resp), &Options{}); err != nil {
		t.Errorf("server Write got %v, want <nil>", err)
		return
	}
	// Read one more time to ensure that everything remains fine and
	// that the goroutine, that we launched earlier to signal client
	// to read, gets enough time to process.
	_, err = s.readTo(p)
	if err != nil {
		t.Errorf("s.Read(_) = _, %v, want _, nil", err)
		return
	}
	// send the trailer to end the stream.
	if err := h.t.WriteStatus(s, status.New(codes.OK, "")); err != nil {
		t.Errorf("server WriteStatus got %v, want <nil>", err)
		return
	}
}

type server struct {
	lis        net.Listener
	port       string
	startedErr chan error // error (or nil) with server start value
	mu         sync.Mutex
	conns      map[ServerTransport]net.Conn
	h          *testStreamHandler
	ready      chan struct{}
	channelz   *channelz.Server
}

func newTestServer() *server {
	return &server{
		startedErr: make(chan error, 1),
		ready:      make(chan struct{}),
		channelz:   channelz.RegisterServer("test server"),
	}
}

// start starts server. Other goroutines should block on s.readyChan for further operations.
func (s *server) start(t *testing.T, port int, serverConfig *ServerConfig, ht hType) {
	var err error
	if port == 0 {
		s.lis, err = net.Listen("tcp", "localhost:0")
	} else {
		s.lis, err = net.Listen("tcp", "localhost:"+strconv.Itoa(port))
	}
	if err != nil {
		s.startedErr <- fmt.Errorf("failed to listen: %v", err)
		return
	}
	_, p, err := net.SplitHostPort(s.lis.Addr().String())
	if err != nil {
		s.startedErr <- fmt.Errorf("failed to parse listener address: %v", err)
		return
	}
	s.port = p
	s.conns = make(map[ServerTransport]net.Conn)
	s.startedErr <- nil
	for {
		conn, err := s.lis.Accept()
		if err != nil {
			return
		}
		rawConn := conn
		if serverConfig.MaxStreams == 0 {
			serverConfig.MaxStreams = math.MaxUint32
		}
		transport, err := NewServerTransport(conn, serverConfig)
		if err != nil {
			return
		}
		s.mu.Lock()
		if s.conns == nil {
			s.mu.Unlock()
			transport.Close(errors.New("s.conns is nil"))
			return
		}
		s.conns[transport] = rawConn
		h := &testStreamHandler{t: transport.(*http2Server)}
		s.h = h
		s.mu.Unlock()
		switch ht {
		case notifyCall:
			go transport.HandleStreams(context.Background(), h.handleStreamAndNotify)
		case suspended:
			go transport.HandleStreams(context.Background(), func(*Stream) {})
		case misbehaved:
			go transport.HandleStreams(context.Background(), func(s *Stream) {
				go h.handleStreamMisbehave(t, s)
			})
		case encodingRequiredStatus:
			go transport.HandleStreams(context.Background(), func(s *Stream) {
				go h.handleStreamEncodingRequiredStatus(s)
			})
		case invalidHeaderField:
			go transport.HandleStreams(context.Background(), func(s *Stream) {
				go h.handleStreamInvalidHeaderField(s)
			})
		case delayRead:
			h.notify = make(chan struct{})
			h.getNotified = make(chan struct{})
			s.mu.Lock()
			close(s.ready)
			s.mu.Unlock()
			go transport.HandleStreams(context.Background(), func(s *Stream) {
				go h.handleStreamDelayRead(t, s)
			})
		case pingpong:
			go transport.HandleStreams(context.Background(), func(s *Stream) {
				go h.handleStreamPingPong(t, s)
			})
		default:
			go transport.HandleStreams(context.Background(), func(s *Stream) {
				go h.handleStream(t, s)
			})
		}
	}
}

func (s *server) wait(t *testing.T, timeout time.Duration) {
	select {
	case err := <-s.startedErr:
		if err != nil {
			t.Fatal(err)
		}
	case <-time.After(timeout):
		t.Fatalf("Timed out after %v waiting for server to be ready", timeout)
	}
}

func (s *server) stop() {
	s.lis.Close()
	s.mu.Lock()
	for c := range s.conns {
		c.Close(errors.New("server Stop called"))
	}
	s.conns = nil
	s.mu.Unlock()
}

func (s *server) addr() string {
	if s.lis == nil {
		return ""
	}
	return s.lis.Addr().String()
}

func setUpServerOnly(t *testing.T, port int, sc *ServerConfig, ht hType) *server {
	server := newTestServer()
	sc.ChannelzParent = server.channelz
	go server.start(t, port, sc, ht)
	server.wait(t, 2*time.Second)
	return server
}

func setUp(t *testing.T, port int, ht hType) (*server, *http2Client, func()) {
	return setUpWithOptions(t, port, &ServerConfig{}, ht, ConnectOptions{})
}

func setUpWithOptions(t *testing.T, port int, sc *ServerConfig, ht hType, copts ConnectOptions) (*server, *http2Client, func()) {
	server := setUpServerOnly(t, port, sc, ht)
	addr := resolver.Address{Addr: "localhost:" + server.port}
	copts.ChannelzParent = channelzSubChannel(t)

	connectCtx, cancel := context.WithDeadline(context.Background(), time.Now().Add(2*time.Second))
	ct, connErr := NewClientTransport(connectCtx, context.Background(), addr, copts, func(GoAwayReason) {})
	if connErr != nil {
		cancel() // Do not cancel in success path.
		t.Fatalf("failed to create transport: %v", connErr)
	}
	return server, ct.(*http2Client), cancel
}

func setUpWithNoPingServer(t *testing.T, copts ConnectOptions, connCh chan net.Conn) (*http2Client, func()) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	// Launch a non responsive server.
	go func() {
		defer lis.Close()
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error at server-side while accepting: %v", err)
			close(connCh)
			return
		}
		framer := http2.NewFramer(conn, conn)
		if err := framer.WriteSettings(); err != nil {
			t.Errorf("Error at server-side while writing settings: %v", err)
			close(connCh)
			return
		}
		connCh <- conn
	}()
	connectCtx, cancel := context.WithDeadline(context.Background(), time.Now().Add(2*time.Second))
	tr, err := NewClientTransport(connectCtx, context.Background(), resolver.Address{Addr: lis.Addr().String()}, copts, func(GoAwayReason) {})
	if err != nil {
		cancel() // Do not cancel in success path.
		// Server clean-up.
		lis.Close()
		if conn, ok := <-connCh; ok {
			conn.Close()
		}
		t.Fatalf("Failed to dial: %v", err)
	}
	return tr.(*http2Client), cancel
}

// TestInflightStreamClosing ensures that closing in-flight stream
// sends status error to concurrent stream reader.
func (s) TestInflightStreamClosing(t *testing.T) {
	serverConfig := &ServerConfig{}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer cancel()
	defer server.stop()
	defer client.Close(fmt.Errorf("closed manually by test"))

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Client failed to create RPC request: %v", err)
	}

	donec := make(chan struct{})
	serr := status.Error(codes.Internal, "client connection is closing")
	go func() {
		defer close(donec)
		if _, err := stream.readTo(make([]byte, defaultWindowSize)); err != serr {
			t.Errorf("unexpected Stream error %v, expected %v", err, serr)
		}
	}()

	// should unblock concurrent stream.Read
	client.CloseStream(stream, serr)

	// wait for stream.Read error
	timeout := time.NewTimer(5 * time.Second)
	select {
	case <-donec:
		if !timeout.Stop() {
			<-timeout.C
		}
	case <-timeout.C:
		t.Fatalf("Test timed out, expected a status error.")
	}
}

// Tests that when streamID > MaxStreamId, the current client transport drains.
func (s) TestClientTransportDrainsAfterStreamIDExhausted(t *testing.T) {
	server, ct, cancel := setUp(t, 0, normal)
	defer cancel()
	defer server.stop()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Small",
	}

	originalMaxStreamID := MaxStreamID
	MaxStreamID = 3
	defer func() {
		MaxStreamID = originalMaxStreamID
	}()

	ctx, ctxCancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer ctxCancel()

	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		t.Fatalf("ct.NewStream() = %v", err)
	}
	if s.id != 1 {
		t.Fatalf("Stream id: %d, want: 1", s.id)
	}

	if got, want := ct.stateForTesting(), reachable; got != want {
		t.Fatalf("Client transport state %v, want %v", got, want)
	}

	// The expected stream ID here is 3 since stream IDs are incremented by 2.
	s, err = ct.NewStream(ctx, callHdr)
	if err != nil {
		t.Fatalf("ct.NewStream() = %v", err)
	}
	if s.id != 3 {
		t.Fatalf("Stream id: %d, want: 3", s.id)
	}

	// Verifying that ct.state is draining when next stream ID > MaxStreamId.
	if got, want := ct.stateForTesting(), draining; got != want {
		t.Fatalf("Client transport state %v, want %v", got, want)
	}
}

func (s) TestClientSendAndReceive(t *testing.T) {
	server, ct, cancel := setUp(t, 0, normal)
	defer cancel()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Small",
	}
	ctx, ctxCancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer ctxCancel()
	s1, err1 := ct.NewStream(ctx, callHdr)
	if err1 != nil {
		t.Fatalf("failed to open stream: %v", err1)
	}
	if s1.id != 1 {
		t.Fatalf("wrong stream id: %d", s1.id)
	}
	s2, err2 := ct.NewStream(ctx, callHdr)
	if err2 != nil {
		t.Fatalf("failed to open stream: %v", err2)
	}
	if s2.id != 3 {
		t.Fatalf("wrong stream id: %d", s2.id)
	}
	opts := Options{Last: true}
	if err := ct.Write(s1, nil, newBufferSlice(expectedRequest), &opts); err != nil && err != io.EOF {
		t.Fatalf("failed to send data: %v", err)
	}
	p := make([]byte, len(expectedResponse))
	_, recvErr := s1.readTo(p)
	if recvErr != nil || !bytes.Equal(p, expectedResponse) {
		t.Fatalf("Error: %v, want <nil>; Result: %v, want %v", recvErr, p, expectedResponse)
	}
	_, recvErr = s1.readTo(p)
	if recvErr != io.EOF {
		t.Fatalf("Error: %v; want <EOF>", recvErr)
	}
	ct.Close(fmt.Errorf("closed manually by test"))
	server.stop()
}

func (s) TestClientErrorNotify(t *testing.T) {
	server, ct, cancel := setUp(t, 0, normal)
	defer cancel()
	go server.stop()
	// ct.reader should detect the error and activate ct.Error().
	<-ct.Error()
	ct.Close(fmt.Errorf("closed manually by test"))
}

func performOneRPC(ct ClientTransport) {
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Small",
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		return
	}
	opts := Options{Last: true}
	if err := ct.Write(s, []byte{}, newBufferSlice(expectedRequest), &opts); err == nil || err == io.EOF {
		time.Sleep(5 * time.Millisecond)
		// The following s.Recv()'s could error out because the
		// underlying transport is gone.
		//
		// Read response
		p := make([]byte, len(expectedResponse))
		s.readTo(p)
		// Read io.EOF
		s.readTo(p)
	}
}

func (s) TestClientMix(t *testing.T) {
	s, ct, cancel := setUp(t, 0, normal)
	defer cancel()
	time.AfterFunc(time.Second, s.stop)
	go func(ct ClientTransport) {
		<-ct.Error()
		ct.Close(fmt.Errorf("closed manually by test"))
	}(ct)
	for i := 0; i < 750; i++ {
		time.Sleep(2 * time.Millisecond)
		go performOneRPC(ct)
	}
}

func (s) TestLargeMessage(t *testing.T) {
	server, ct, cancel := setUp(t, 0, normal)
	defer cancel()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Large",
	}
	ctx, ctxCancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer ctxCancel()
	var wg sync.WaitGroup
	for i := 0; i < 2; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			s, err := ct.NewStream(ctx, callHdr)
			if err != nil {
				t.Errorf("%v.NewStream(_, _) = _, %v, want _, <nil>", ct, err)
			}
			if err := ct.Write(s, []byte{}, newBufferSlice(expectedRequestLarge), &Options{Last: true}); err != nil && err != io.EOF {
				t.Errorf("%v.Write(_, _, _) = %v, want  <nil>", ct, err)
			}
			p := make([]byte, len(expectedResponseLarge))
			if _, err := s.readTo(p); err != nil || !bytes.Equal(p, expectedResponseLarge) {
				t.Errorf("s.Read(%v) = _, %v, want %v, <nil>", err, p, expectedResponse)
			}
			if _, err = s.readTo(p); err != io.EOF {
				t.Errorf("Failed to complete the stream %v; want <EOF>", err)
			}
		}()
	}
	wg.Wait()
	ct.Close(fmt.Errorf("closed manually by test"))
	server.stop()
}

func (s) TestLargeMessageWithDelayRead(t *testing.T) {
	// Disable dynamic flow control.
	sc := &ServerConfig{
		InitialWindowSize:     defaultWindowSize,
		InitialConnWindowSize: defaultWindowSize,
	}
	co := ConnectOptions{
		InitialWindowSize:     defaultWindowSize,
		InitialConnWindowSize: defaultWindowSize,
	}
	server, ct, cancel := setUpWithOptions(t, 0, sc, delayRead, co)
	defer cancel()
	defer server.stop()
	defer ct.Close(fmt.Errorf("closed manually by test"))
	server.mu.Lock()
	ready := server.ready
	server.mu.Unlock()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Large",
	}
	ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(time.Second*10))
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		t.Fatalf("%v.NewStream(_, _) = _, %v, want _, <nil>", ct, err)
		return
	}
	// Wait for server's handler to be initialized
	select {
	case <-ready:
	case <-ctx.Done():
		t.Fatalf("Client timed out waiting for server handler to be initialized.")
	}
	server.mu.Lock()
	serviceHandler := server.h
	server.mu.Unlock()
	var (
		mu    sync.Mutex
		total int
	)
	s.wq.replenish = func(n int) {
		mu.Lock()
		total += n
		mu.Unlock()
		s.wq.realReplenish(n)
	}
	getTotal := func() int {
		mu.Lock()
		defer mu.Unlock()
		return total
	}
	done := make(chan struct{})
	defer close(done)
	go func() {
		for {
			select {
			// Prevent goroutine from leaking in case of error.
			case <-done:
				return
			default:
			}
			if getTotal() == defaultWindowSize {
				// unblock server to be able to read and
				// thereby send stream level window update.
				close(serviceHandler.getNotified)
				return
			}
			runtime.Gosched()
		}
	}()
	// This write will cause client to run out of stream level,
	// flow control and the other side won't send a window update
	// until that happens.
	if err := ct.Write(s, []byte{}, newBufferSlice(expectedRequestLarge), &Options{}); err != nil {
		t.Fatalf("write(_, _, _) = %v, want  <nil>", err)
	}
	p := make([]byte, len(expectedResponseLarge))

	// Wait for the other side to run out of stream level flow control before
	// reading and thereby sending a window update.
	select {
	case <-serviceHandler.notify:
	case <-ctx.Done():
		t.Fatalf("Client timed out")
	}
	if _, err := s.readTo(p); err != nil || !bytes.Equal(p, expectedResponseLarge) {
		t.Fatalf("s.Read(_) = _, %v, want _, <nil>", err)
	}
	if err := ct.Write(s, []byte{}, newBufferSlice(expectedRequestLarge), &Options{Last: true}); err != nil {
		t.Fatalf("Write(_, _, _) = %v, want <nil>", err)
	}
	if _, err = s.readTo(p); err != io.EOF {
		t.Fatalf("Failed to complete the stream %v; want <EOF>", err)
	}
}

// TestGracefulClose ensures that GracefulClose allows in-flight streams to
// proceed until they complete naturally, while not allowing creation of new
// streams during this window.
func (s) TestGracefulClose(t *testing.T) {
	leakcheck.SetTrackingBufferPool(t)
	server, ct, cancel := setUp(t, 0, pingpong)
	defer cancel()
	defer func() {
		// Stop the server's listener to make the server's goroutines terminate
		// (after the last active stream is done).
		server.lis.Close()
		// Check for goroutine leaks (i.e. GracefulClose with an active stream
		// doesn't eventually close the connection when that stream completes).
		leakcheck.CheckGoroutines(t, 10*time.Second)
		leakcheck.CheckTrackingBufferPool()
		// Correctly clean up the server
		server.stop()
	}()
	ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(time.Second*10))
	defer cancel()

	// Create a stream that will exist for this whole test and confirm basic
	// functionality.
	s, err := ct.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("NewStream(_, _) = _, %v, want _, <nil>", err)
	}
	msg := make([]byte, 1024)
	outgoingHeader := make([]byte, 5)
	outgoingHeader[0] = byte(0)
	binary.BigEndian.PutUint32(outgoingHeader[1:], uint32(len(msg)))
	incomingHeader := make([]byte, 5)
	if err := ct.Write(s, outgoingHeader, newBufferSlice(msg), &Options{}); err != nil {
		t.Fatalf("Error while writing: %v", err)
	}
	if _, err := s.readTo(incomingHeader); err != nil {
		t.Fatalf("Error while reading: %v", err)
	}
	sz := binary.BigEndian.Uint32(incomingHeader[1:])
	recvMsg := make([]byte, int(sz))
	if _, err := s.readTo(recvMsg); err != nil {
		t.Fatalf("Error while reading: %v", err)
	}

	// Gracefully close the transport, which should not affect the existing
	// stream.
	ct.GracefulClose()

	var wg sync.WaitGroup
	// Expect errors creating new streams because the client transport has been
	// gracefully closed.
	for i := 0; i < 200; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := ct.NewStream(ctx, &CallHdr{})
			if err != nil && err.(*NewStreamError).Err == ErrConnClosing && err.(*NewStreamError).AllowTransparentRetry {
				return
			}
			t.Errorf("_.NewStream(_, _) = _, %v, want _, %v", err, ErrConnClosing)
		}()
	}

	// Confirm the existing stream still functions as expected.
	ct.Write(s, nil, nil, &Options{Last: true})
	if _, err := s.readTo(incomingHeader); err != io.EOF {
		t.Fatalf("Client expected EOF from the server. Got: %v", err)
	}
	wg.Wait()
}

func (s) TestLargeMessageSuspension(t *testing.T) {
	server, ct, cancel := setUp(t, 0, suspended)
	defer cancel()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Large",
	}
	// Set a long enough timeout for writing a large message out.
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		t.Fatalf("failed to open stream: %v", err)
	}
	// Launch a goroutine similar to the stream monitoring goroutine in
	// stream.go to keep track of context timeout and call CloseStream.
	go func() {
		<-ctx.Done()
		ct.CloseStream(s, ContextErr(ctx.Err()))
	}()
	// Write should not be done successfully due to flow control.
	msg := make([]byte, initialWindowSize*8)
	ct.Write(s, nil, newBufferSlice(msg), &Options{})
	err = ct.Write(s, nil, newBufferSlice(msg), &Options{Last: true})
	if err != errStreamDone {
		t.Fatalf("Write got %v, want io.EOF", err)
	}
	expectedErr := status.Error(codes.DeadlineExceeded, context.DeadlineExceeded.Error())
	if _, err := s.readTo(make([]byte, 8)); err.Error() != expectedErr.Error() {
		t.Fatalf("Read got %v of type %T, want %v", err, err, expectedErr)
	}
	ct.Close(fmt.Errorf("closed manually by test"))
	server.stop()
}

func (s) TestMaxStreams(t *testing.T) {
	serverConfig := &ServerConfig{
		MaxStreams: 1,
	}
	server, ct, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer cancel()
	defer ct.Close(fmt.Errorf("closed manually by test"))
	defer server.stop()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo.Large",
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		t.Fatalf("Failed to open stream: %v", err)
	}
	// Keep creating streams until one fails with deadline exceeded, marking the application
	// of server settings on client.
	slist := []*Stream{}
	pctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	timer := time.NewTimer(time.Second * 10)
	expectedErr := status.Error(codes.DeadlineExceeded, context.DeadlineExceeded.Error())
	for {
		select {
		case <-timer.C:
			t.Fatalf("Test timeout: client didn't receive server settings.")
		default:
		}
		ctx, cancel := context.WithDeadline(pctx, time.Now().Add(time.Second))
		// This is only to get rid of govet. All these context are based on a base
		// context which is canceled at the end of the test.
		defer cancel()
		if str, err := ct.NewStream(ctx, callHdr); err == nil {
			slist = append(slist, str)
			continue
		} else if err.Error() != expectedErr.Error() {
			t.Fatalf("ct.NewStream(_,_) = _, %v, want _, %v", err, expectedErr)
		}
		timer.Stop()
		break
	}
	done := make(chan struct{})
	// Try and create a new stream.
	go func() {
		defer close(done)
		ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(time.Second*10))
		defer cancel()
		if _, err := ct.NewStream(ctx, callHdr); err != nil {
			t.Errorf("Failed to open stream: %v", err)
		}
	}()
	// Close all the extra streams created and make sure the new stream is not created.
	for _, str := range slist {
		ct.CloseStream(str, nil)
	}
	select {
	case <-done:
		t.Fatalf("Test failed: didn't expect new stream to be created just yet.")
	default:
	}
	// Close the first stream created so that the new stream can finally be created.
	ct.CloseStream(s, nil)
	<-done
	ct.Close(fmt.Errorf("closed manually by test"))
	<-ct.writerDone
	if ct.maxConcurrentStreams != 1 {
		t.Fatalf("ct.maxConcurrentStreams: %d, want 1", ct.maxConcurrentStreams)
	}
}

func (s) TestServerContextCanceledOnClosedConnection(t *testing.T) {
	server, ct, cancel := setUp(t, 0, suspended)
	defer cancel()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo",
	}
	var sc *http2Server
	// Wait until the server transport is setup.
	for {
		server.mu.Lock()
		if len(server.conns) == 0 {
			server.mu.Unlock()
			time.Sleep(time.Millisecond)
			continue
		}
		for k := range server.conns {
			var ok bool
			sc, ok = k.(*http2Server)
			if !ok {
				t.Fatalf("Failed to convert %v to *http2Server", k)
			}
		}
		server.mu.Unlock()
		break
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		t.Fatalf("Failed to open stream: %v", err)
	}
	d := newBufferSlice(make([]byte, http2MaxFrameLen))
	ct.controlBuf.put(&dataFrame{
		streamID:    s.id,
		endStream:   false,
		h:           nil,
		reader:      d.Reader(),
		onEachWrite: func() {},
	})
	// Loop until the server side stream is created.
	var ss *Stream
	for {
		time.Sleep(time.Second)
		sc.mu.Lock()
		if len(sc.activeStreams) == 0 {
			sc.mu.Unlock()
			continue
		}
		ss = sc.activeStreams[s.id]
		sc.mu.Unlock()
		break
	}
	ct.Close(fmt.Errorf("closed manually by test"))
	select {
	case <-ss.Context().Done():
		if ss.Context().Err() != context.Canceled {
			t.Fatalf("ss.Context().Err() got %v, want %v", ss.Context().Err(), context.Canceled)
		}
	case <-time.After(5 * time.Second):
		t.Fatalf("Failed to cancel the context of the sever side stream.")
	}
	server.stop()
}

func (s) TestClientConnDecoupledFromApplicationRead(t *testing.T) {
	connectOptions := ConnectOptions{
		InitialWindowSize:     defaultWindowSize,
		InitialConnWindowSize: defaultWindowSize,
	}
	server, client, cancel := setUpWithOptions(t, 0, &ServerConfig{}, notifyCall, connectOptions)
	defer cancel()
	defer server.stop()
	defer client.Close(fmt.Errorf("closed manually by test"))

	waitWhileTrue(t, func() (bool, error) {
		server.mu.Lock()
		defer server.mu.Unlock()

		if len(server.conns) == 0 {
			return true, fmt.Errorf("timed-out while waiting for connection to be created on the server")
		}
		return false, nil
	})

	var st *http2Server
	server.mu.Lock()
	for k := range server.conns {
		st = k.(*http2Server)
	}
	notifyChan := make(chan struct{})
	server.h.notify = notifyChan
	server.mu.Unlock()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cstream1, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Client failed to create first stream. Err: %v", err)
	}

	<-notifyChan
	var sstream1 *Stream
	// Access stream on the server.
	st.mu.Lock()
	for _, v := range st.activeStreams {
		if v.id == cstream1.id {
			sstream1 = v
		}
	}
	st.mu.Unlock()
	if sstream1 == nil {
		t.Fatalf("Didn't find stream corresponding to client cstream.id: %v on the server", cstream1.id)
	}
	// Exhaust client's connection window.
	if err := st.Write(sstream1, []byte{}, newBufferSlice(make([]byte, defaultWindowSize)), &Options{}); err != nil {
		t.Fatalf("Server failed to write data. Err: %v", err)
	}
	notifyChan = make(chan struct{})
	server.mu.Lock()
	server.h.notify = notifyChan
	server.mu.Unlock()
	// Create another stream on client.
	cstream2, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Client failed to create second stream. Err: %v", err)
	}
	<-notifyChan
	var sstream2 *Stream
	st.mu.Lock()
	for _, v := range st.activeStreams {
		if v.id == cstream2.id {
			sstream2 = v
		}
	}
	st.mu.Unlock()
	if sstream2 == nil {
		t.Fatalf("Didn't find stream corresponding to client cstream.id: %v on the server", cstream2.id)
	}
	// Server should be able to send data on the new stream, even though the client hasn't read anything on the first stream.
	if err := st.Write(sstream2, []byte{}, newBufferSlice(make([]byte, defaultWindowSize)), &Options{}); err != nil {
		t.Fatalf("Server failed to write data. Err: %v", err)
	}

	// Client should be able to read data on second stream.
	if _, err := cstream2.readTo(make([]byte, defaultWindowSize)); err != nil {
		t.Fatalf("_.Read(_) = _, %v, want _, <nil>", err)
	}

	// Client should be able to read data on first stream.
	if _, err := cstream1.readTo(make([]byte, defaultWindowSize)); err != nil {
		t.Fatalf("_.Read(_) = _, %v, want _, <nil>", err)
	}
}

func (s) TestServerConnDecoupledFromApplicationRead(t *testing.T) {
	serverConfig := &ServerConfig{
		InitialWindowSize:     defaultWindowSize,
		InitialConnWindowSize: defaultWindowSize,
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer cancel()
	defer server.stop()
	defer client.Close(fmt.Errorf("closed manually by test"))
	waitWhileTrue(t, func() (bool, error) {
		server.mu.Lock()
		defer server.mu.Unlock()

		if len(server.conns) == 0 {
			return true, fmt.Errorf("timed-out while waiting for connection to be created on the server")
		}
		return false, nil
	})
	var st *http2Server
	server.mu.Lock()
	for k := range server.conns {
		st = k.(*http2Server)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	server.mu.Unlock()
	cstream1, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Failed to create 1st stream. Err: %v", err)
	}
	// Exhaust server's connection window.
	if err := client.Write(cstream1, nil, newBufferSlice(make([]byte, defaultWindowSize)), &Options{Last: true}); err != nil {
		t.Fatalf("Client failed to write data. Err: %v", err)
	}
	//Client should be able to create another stream and send data on it.
	cstream2, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Failed to create 2nd stream. Err: %v", err)
	}
	if err := client.Write(cstream2, nil, newBufferSlice(make([]byte, defaultWindowSize)), &Options{}); err != nil {
		t.Fatalf("Client failed to write data. Err: %v", err)
	}
	// Get the streams on server.
	waitWhileTrue(t, func() (bool, error) {
		st.mu.Lock()
		defer st.mu.Unlock()

		if len(st.activeStreams) != 2 {
			return true, fmt.Errorf("timed-out while waiting for server to have created the streams")
		}
		return false, nil
	})
	var sstream1 *Stream
	st.mu.Lock()
	for _, v := range st.activeStreams {
		if v.id == 1 {
			sstream1 = v
		}
	}
	st.mu.Unlock()
	// Reading from the stream on server should succeed.
	if _, err := sstream1.readTo(make([]byte, defaultWindowSize)); err != nil {
		t.Fatalf("_.Read(_) = %v, want <nil>", err)
	}

	if _, err := sstream1.readTo(make([]byte, 1)); err != io.EOF {
		t.Fatalf("_.Read(_) = %v, want io.EOF", err)
	}

}

func (s) TestServerWithMisbehavedClient(t *testing.T) {
	server := setUpServerOnly(t, 0, &ServerConfig{}, suspended)
	defer server.stop()
	// Create a client that can override server stream quota.
	mconn, err := net.Dial("tcp", server.lis.Addr().String())
	if err != nil {
		t.Fatalf("Clent failed to dial:%v", err)
	}
	defer mconn.Close()
	if err := mconn.SetWriteDeadline(time.Now().Add(time.Second * 10)); err != nil {
		t.Fatalf("Failed to set write deadline: %v", err)
	}
	if n, err := mconn.Write(clientPreface); err != nil || n != len(clientPreface) {
		t.Fatalf("mconn.Write(clientPreface) = %d, %v, want %d, <nil>", n, err, len(clientPreface))
	}
	// success chan indicates that reader received a RSTStream from server.
	success := make(chan struct{})
	var mu sync.Mutex
	framer := http2.NewFramer(mconn, mconn)
	if err := framer.WriteSettings(); err != nil {
		t.Fatalf("Error while writing settings: %v", err)
	}
	go func() { // Launch a reader for this misbehaving client.
		for {
			frame, err := framer.ReadFrame()
			if err != nil {
				return
			}
			switch frame := frame.(type) {
			case *http2.PingFrame:
				// Write ping ack back so that server's BDP estimation works right.
				mu.Lock()
				framer.WritePing(true, frame.Data)
				mu.Unlock()
			case *http2.RSTStreamFrame:
				if frame.Header().StreamID != 1 || http2.ErrCode(frame.ErrCode) != http2.ErrCodeFlowControl {
					t.Errorf("RST stream received with streamID: %d and code: %v, want streamID: 1 and code: http2.ErrCodeFlowControl", frame.Header().StreamID, http2.ErrCode(frame.ErrCode))
				}
				close(success)
				return
			default:
				// Do nothing.
			}

		}
	}()
	// Create a stream.
	var buf bytes.Buffer
	henc := hpack.NewEncoder(&buf)
	// TODO(mmukhi): Remove unnecessary fields.
	if err := henc.WriteField(hpack.HeaderField{Name: ":method", Value: "POST"}); err != nil {
		t.Fatalf("Error while encoding header: %v", err)
	}
	if err := henc.WriteField(hpack.HeaderField{Name: ":path", Value: "foo"}); err != nil {
		t.Fatalf("Error while encoding header: %v", err)
	}
	if err := henc.WriteField(hpack.HeaderField{Name: ":authority", Value: "localhost"}); err != nil {
		t.Fatalf("Error while encoding header: %v", err)
	}
	if err := henc.WriteField(hpack.HeaderField{Name: "content-type", Value: "application/grpc"}); err != nil {
		t.Fatalf("Error while encoding header: %v", err)
	}
	mu.Lock()
	if err := framer.WriteHeaders(http2.HeadersFrameParam{StreamID: 1, BlockFragment: buf.Bytes(), EndHeaders: true}); err != nil {
		mu.Unlock()
		t.Fatalf("Error while writing headers: %v", err)
	}
	mu.Unlock()

	// Test server behavior for violation of stream flow control window size restriction.
	timer := time.NewTimer(time.Second * 5)
	dbuf := make([]byte, http2MaxFrameLen)
	for {
		select {
		case <-timer.C:
			t.Fatalf("Test timed out.")
		case <-success:
			return
		default:
		}
		mu.Lock()
		if err := framer.WriteData(1, false, dbuf); err != nil {
			mu.Unlock()
			// Error here means the server could have closed the connection due to flow control
			// violation. Make sure that is the case by waiting for success chan to be closed.
			select {
			case <-timer.C:
				t.Fatalf("Error while writing data: %v", err)
			case <-success:
				return
			}
		}
		mu.Unlock()
		// This for loop is capable of hogging the CPU and cause starvation
		// in Go versions prior to 1.9,
		// in single CPU environment. Explicitly relinquish processor.
		runtime.Gosched()
	}
}

func (s) TestClientHonorsConnectContext(t *testing.T) {
	// Create a server that will not send a preface.
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening: %v", err)
	}
	defer lis.Close()
	go func() { // Launch the misbehaving server.
		sconn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error while accepting: %v", err)
			return
		}
		defer sconn.Close()
		if _, err := io.ReadFull(sconn, make([]byte, len(clientPreface))); err != nil {
			t.Errorf("Error while reading client preface: %v", err)
			return
		}
		sfr := http2.NewFramer(sconn, sconn)
		// Do not write a settings frame, but read from the conn forever.
		for {
			if _, err := sfr.ReadFrame(); err != nil {
				return
			}
		}
	}()

	// Test context cancellation.
	timeBefore := time.Now()
	connectCtx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	time.AfterFunc(100*time.Millisecond, cancel)

	parent := channelzSubChannel(t)
	copts := ConnectOptions{ChannelzParent: parent}
	_, err = NewClientTransport(connectCtx, context.Background(), resolver.Address{Addr: lis.Addr().String()}, copts, func(GoAwayReason) {})
	if err == nil {
		t.Fatalf("NewClientTransport() returned successfully; wanted error")
	}
	t.Logf("NewClientTransport() = _, %v", err)
	if time.Since(timeBefore) > 3*time.Second {
		t.Fatalf("NewClientTransport returned > 2.9s after context cancellation")
	}

	// Test context deadline.
	connectCtx, cancel = context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	_, err = NewClientTransport(connectCtx, context.Background(), resolver.Address{Addr: lis.Addr().String()}, copts, func(GoAwayReason) {})
	if err == nil {
		t.Fatalf("NewClientTransport() returned successfully; wanted error")
	}
	t.Logf("NewClientTransport() = _, %v", err)
}

func (s) TestClientWithMisbehavedServer(t *testing.T) {
	// Create a misbehaving server.
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening: %v", err)
	}
	defer lis.Close()
	// success chan indicates that the server received
	// RSTStream from the client.
	success := make(chan struct{})
	go func() { // Launch the misbehaving server.
		sconn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error while accepting: %v", err)
			return
		}
		defer sconn.Close()
		if _, err := io.ReadFull(sconn, make([]byte, len(clientPreface))); err != nil {
			t.Errorf("Error while reading client preface: %v", err)
			return
		}
		sfr := http2.NewFramer(sconn, sconn)
		if err := sfr.WriteSettings(); err != nil {
			t.Errorf("Error while writing settings: %v", err)
			return
		}
		if err := sfr.WriteSettingsAck(); err != nil {
			t.Errorf("Error while writing settings: %v", err)
			return
		}
		var mu sync.Mutex
		for {
			frame, err := sfr.ReadFrame()
			if err != nil {
				return
			}
			switch frame := frame.(type) {
			case *http2.HeadersFrame:
				// When the client creates a stream, violate the stream flow control.
				go func() {
					buf := make([]byte, http2MaxFrameLen)
					for {
						mu.Lock()
						if err := sfr.WriteData(1, false, buf); err != nil {
							mu.Unlock()
							return
						}
						mu.Unlock()
						// This for loop is capable of hogging the CPU and cause starvation
						// in Go versions prior to 1.9,
						// in single CPU environment. Explicitly relinquish processor.
						runtime.Gosched()
					}
				}()
			case *http2.RSTStreamFrame:
				if frame.Header().StreamID != 1 || http2.ErrCode(frame.ErrCode) != http2.ErrCodeFlowControl {
					t.Errorf("RST stream received with streamID: %d and code: %v, want streamID: 1 and code: http2.ErrCodeFlowControl", frame.Header().StreamID, http2.ErrCode(frame.ErrCode))
				}
				close(success)
				return
			case *http2.PingFrame:
				mu.Lock()
				sfr.WritePing(true, frame.Data)
				mu.Unlock()
			default:
			}
		}
	}()
	connectCtx, cancel := context.WithDeadline(context.Background(), time.Now().Add(2*time.Second))
	defer cancel()

	parent := channelzSubChannel(t)
	copts := ConnectOptions{ChannelzParent: parent}
	ct, err := NewClientTransport(connectCtx, context.Background(), resolver.Address{Addr: lis.Addr().String()}, copts, func(GoAwayReason) {})
	if err != nil {
		t.Fatalf("Error while creating client transport: %v", err)
	}
	defer ct.Close(fmt.Errorf("closed manually by test"))

	str, err := ct.NewStream(connectCtx, &CallHdr{})
	if err != nil {
		t.Fatalf("Error while creating stream: %v", err)
	}
	timer := time.NewTimer(time.Second * 5)
	go func() { // This go routine mimics the one in stream.go to call CloseStream.
		<-str.Done()
		ct.CloseStream(str, nil)
	}()
	select {
	case <-timer.C:
		t.Fatalf("Test timed-out.")
	case <-success:
	}
	// Drain the remaining buffers in the stream by reading until an error is
	// encountered.
	str.Read(math.MaxInt)
}

var encodingTestStatus = status.New(codes.Internal, "\n")

func (s) TestEncodingRequiredStatus(t *testing.T) {
	server, ct, cancel := setUp(t, 0, encodingRequiredStatus)
	defer cancel()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo",
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		return
	}
	opts := Options{Last: true}
	if err := ct.Write(s, nil, newBufferSlice(expectedRequest), &opts); err != nil && err != errStreamDone {
		t.Fatalf("Failed to write the request: %v", err)
	}
	p := make([]byte, http2MaxFrameLen)
	if _, err := s.readTo(p); err != io.EOF {
		t.Fatalf("Read got error %v, want %v", err, io.EOF)
	}
	if !testutils.StatusErrEqual(s.Status().Err(), encodingTestStatus.Err()) {
		t.Fatalf("stream with status %v, want %v", s.Status(), encodingTestStatus)
	}
	ct.Close(fmt.Errorf("closed manually by test"))
	server.stop()
	// Drain any remaining buffers from the stream since it was closed early.
	s.Read(math.MaxInt)
}

func (s) TestInvalidHeaderField(t *testing.T) {
	server, ct, cancel := setUp(t, 0, invalidHeaderField)
	defer cancel()
	callHdr := &CallHdr{
		Host:   "localhost",
		Method: "foo",
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := ct.NewStream(ctx, callHdr)
	if err != nil {
		return
	}
	p := make([]byte, http2MaxFrameLen)
	_, err = s.readTo(p)
	if se, ok := status.FromError(err); !ok || se.Code() != codes.Internal || !strings.Contains(err.Error(), expectedInvalidHeaderField) {
		t.Fatalf("Read got error %v, want error with code %s and contains %q", err, codes.Internal, expectedInvalidHeaderField)
	}
	ct.Close(fmt.Errorf("closed manually by test"))
	server.stop()
}

func (s) TestHeaderChanClosedAfterReceivingAnInvalidHeader(t *testing.T) {
	server, ct, cancel := setUp(t, 0, invalidHeaderField)
	defer cancel()
	defer server.stop()
	defer ct.Close(fmt.Errorf("closed manually by test"))
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := ct.NewStream(ctx, &CallHdr{Host: "localhost", Method: "foo"})
	if err != nil {
		t.Fatalf("failed to create the stream")
	}
	timer := time.NewTimer(time.Second)
	defer timer.Stop()
	select {
	case <-s.headerChan:
	case <-timer.C:
		t.Errorf("s.headerChan: got open, want closed")
	}
}

func (s) TestIsReservedHeader(t *testing.T) {
	tests := []struct {
		h    string
		want bool
	}{
		{"", false}, // but should be rejected earlier
		{"foo", false},
		{"content-type", true},
		{"user-agent", true},
		{":anything", true},
		{"grpc-message-type", true},
		{"grpc-encoding", true},
		{"grpc-message", true},
		{"grpc-status", true},
		{"grpc-timeout", true},
		{"te", true},
	}
	for _, tt := range tests {
		got := isReservedHeader(tt.h)
		if got != tt.want {
			t.Errorf("isReservedHeader(%q) = %v; want %v", tt.h, got, tt.want)
		}
	}
}

func (s) TestContextErr(t *testing.T) {
	for _, test := range []struct {
		// input
		errIn error
		// outputs
		errOut error
	}{
		{context.DeadlineExceeded, status.Error(codes.DeadlineExceeded, context.DeadlineExceeded.Error())},
		{context.Canceled, status.Error(codes.Canceled, context.Canceled.Error())},
	} {
		err := ContextErr(test.errIn)
		if err.Error() != test.errOut.Error() {
			t.Fatalf("ContextErr{%v} = %v \nwant %v", test.errIn, err, test.errOut)
		}
	}
}

type windowSizeConfig struct {
	serverStream int32
	serverConn   int32
	clientStream int32
	clientConn   int32
}

func (s) TestAccountCheckWindowSizeWithLargeWindow(t *testing.T) {
	wc := windowSizeConfig{
		serverStream: 10 * 1024 * 1024,
		serverConn:   12 * 1024 * 1024,
		clientStream: 6 * 1024 * 1024,
		clientConn:   8 * 1024 * 1024,
	}
	testFlowControlAccountCheck(t, 1024*1024, wc)
}

func (s) TestAccountCheckWindowSizeWithSmallWindow(t *testing.T) {
	// These settings disable dynamic window sizes based on BDP estimation;
	// must be at least defaultWindowSize or the setting is ignored.
	wc := windowSizeConfig{
		serverStream: defaultWindowSize,
		serverConn:   defaultWindowSize,
		clientStream: defaultWindowSize,
		clientConn:   defaultWindowSize,
	}
	testFlowControlAccountCheck(t, 1024*1024, wc)
}

func (s) TestAccountCheckDynamicWindowSmallMessage(t *testing.T) {
	testFlowControlAccountCheck(t, 1024, windowSizeConfig{})
}

func (s) TestAccountCheckDynamicWindowLargeMessage(t *testing.T) {
	testFlowControlAccountCheck(t, 1024*1024, windowSizeConfig{})
}

func testFlowControlAccountCheck(t *testing.T, msgSize int, wc windowSizeConfig) {
	sc := &ServerConfig{
		InitialWindowSize:     wc.serverStream,
		InitialConnWindowSize: wc.serverConn,
	}
	co := ConnectOptions{
		InitialWindowSize:     wc.clientStream,
		InitialConnWindowSize: wc.clientConn,
	}
	server, client, cancel := setUpWithOptions(t, 0, sc, pingpong, co)
	defer cancel()
	defer server.stop()
	defer client.Close(fmt.Errorf("closed manually by test"))
	waitWhileTrue(t, func() (bool, error) {
		server.mu.Lock()
		defer server.mu.Unlock()
		if len(server.conns) == 0 {
			return true, fmt.Errorf("timed out while waiting for server transport to be created")
		}
		return false, nil
	})
	var st *http2Server
	server.mu.Lock()
	for k := range server.conns {
		st = k.(*http2Server)
	}
	server.mu.Unlock()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	const numStreams = 5
	clientStreams := make([]*Stream, numStreams)
	for i := 0; i < numStreams; i++ {
		var err error
		clientStreams[i], err = client.NewStream(ctx, &CallHdr{})
		if err != nil {
			t.Fatalf("Failed to create stream. Err: %v", err)
		}
	}
	var wg sync.WaitGroup
	// For each stream send pingpong messages to the server.
	for _, stream := range clientStreams {
		wg.Add(1)
		go func(stream *Stream) {
			defer wg.Done()
			buf := make([]byte, msgSize+5)
			buf[0] = byte(0)
			binary.BigEndian.PutUint32(buf[1:], uint32(msgSize))
			opts := Options{}
			header := make([]byte, 5)
			for i := 1; i <= 5; i++ {
				if err := client.Write(stream, nil, newBufferSlice(buf), &opts); err != nil {
					t.Errorf("Error on client while writing message %v on stream %v: %v", i, stream.id, err)
					return
				}
				if _, err := stream.readTo(header); err != nil {
					t.Errorf("Error on client while reading data frame header %v on stream %v: %v", i, stream.id, err)
					return
				}
				sz := binary.BigEndian.Uint32(header[1:])
				recvMsg := make([]byte, int(sz))
				if _, err := stream.readTo(recvMsg); err != nil {
					t.Errorf("Error on client while reading data %v on stream %v: %v", i, stream.id, err)
					return
				}
				if len(recvMsg) != msgSize {
					t.Errorf("Length of message %v received by client on stream %v: %v, want: %v", i, stream.id, len(recvMsg), msgSize)
					return
				}
			}
			t.Logf("stream %v done with pingpongs", stream.id)
		}(stream)
	}
	wg.Wait()
	serverStreams := map[uint32]*Stream{}
	loopyClientStreams := map[uint32]*outStream{}
	loopyServerStreams := map[uint32]*outStream{}
	// Get all the streams from server reader and writer and client writer.
	st.mu.Lock()
	client.mu.Lock()
	for _, stream := range clientStreams {
		id := stream.id
		serverStreams[id] = st.activeStreams[id]
		loopyServerStreams[id] = st.loopy.estdStreams[id]
		loopyClientStreams[id] = client.loopy.estdStreams[id]

	}
	client.mu.Unlock()
	st.mu.Unlock()
	// Close all streams
	for _, stream := range clientStreams {
		client.Write(stream, nil, nil, &Options{Last: true})
		if _, err := stream.readTo(make([]byte, 5)); err != io.EOF {
			t.Fatalf("Client expected an EOF from the server. Got: %v", err)
		}
	}
	// Close down both server and client so that their internals can be read without data
	// races.
	client.Close(errors.New("closed manually by test"))
	st.Close(errors.New("closed manually by test"))
	<-st.readerDone
	<-st.loopyWriterDone
	<-client.readerDone
	<-client.writerDone
	for _, cstream := range clientStreams {
		id := cstream.id
		sstream := serverStreams[id]
		loopyServerStream := loopyServerStreams[id]
		loopyClientStream := loopyClientStreams[id]
		if loopyServerStream == nil {
			t.Fatalf("Unexpected nil loopyServerStream")
		}
		// Check stream flow control.
		if int(cstream.fc.limit+cstream.fc.delta-cstream.fc.pendingData-cstream.fc.pendingUpdate) != int(st.loopy.oiws)-loopyServerStream.bytesOutStanding {
			t.Fatalf("Account mismatch: client stream inflow limit(%d) + delta(%d) - pendingData(%d) - pendingUpdate(%d) != server outgoing InitialWindowSize(%d) - outgoingStream.bytesOutStanding(%d)", cstream.fc.limit, cstream.fc.delta, cstream.fc.pendingData, cstream.fc.pendingUpdate, st.loopy.oiws, loopyServerStream.bytesOutStanding)
		}
		if int(sstream.fc.limit+sstream.fc.delta-sstream.fc.pendingData-sstream.fc.pendingUpdate) != int(client.loopy.oiws)-loopyClientStream.bytesOutStanding {
			t.Fatalf("Account mismatch: server stream inflow limit(%d) + delta(%d) - pendingData(%d) - pendingUpdate(%d) != client outgoing InitialWindowSize(%d) - outgoingStream.bytesOutStanding(%d)", sstream.fc.limit, sstream.fc.delta, sstream.fc.pendingData, sstream.fc.pendingUpdate, client.loopy.oiws, loopyClientStream.bytesOutStanding)
		}
	}
	// Check transport flow control.
	if client.fc.limit != client.fc.unacked+st.loopy.sendQuota {
		t.Fatalf("Account mismatch: client transport inflow(%d) != client unacked(%d) + server sendQuota(%d)", client.fc.limit, client.fc.unacked, st.loopy.sendQuota)
	}
	if st.fc.limit != st.fc.unacked+client.loopy.sendQuota {
		t.Fatalf("Account mismatch: server transport inflow(%d) != server unacked(%d) + client sendQuota(%d)", st.fc.limit, st.fc.unacked, client.loopy.sendQuota)
	}
}

func waitWhileTrue(t *testing.T, condition func() (bool, error)) {
	var (
		wait bool
		err  error
	)
	timer := time.NewTimer(time.Second * 5)
	for {
		wait, err = condition()
		if wait {
			select {
			case <-timer.C:
				t.Fatal(err)
			default:
				time.Sleep(50 * time.Millisecond)
				continue
			}
		}
		if !timer.Stop() {
			<-timer.C
		}
		break
	}
}

// If any error occurs on a call to Stream.Read, future calls
// should continue to return that same error.
func (s) TestReadGivesSameErrorAfterAnyErrorOccurs(t *testing.T) {
	testRecvBuffer := newRecvBuffer()
	s := &Stream{
		ctx:         context.Background(),
		buf:         testRecvBuffer,
		requestRead: func(int) {},
	}
	s.trReader = &transportReader{
		reader: &recvBufferReader{
			ctx:     s.ctx,
			ctxDone: s.ctx.Done(),
			recv:    s.buf,
		},
		windowHandler: func(int) {},
	}
	testData := make([]byte, 1)
	testData[0] = 5
	testErr := errors.New("test error")
	s.write(recvMsg{buffer: mem.NewBuffer(&testData, nil), err: testErr})

	inBuf := make([]byte, 1)
	actualCount, actualErr := s.readTo(inBuf)
	if actualCount != 0 {
		t.Errorf("actualCount, _ := s.Read(_) differs; want 0; got %v", actualCount)
	}
	if actualErr.Error() != testErr.Error() {
		t.Errorf("_ , actualErr := s.Read(_) differs; want actualErr.Error() to be %v; got %v", testErr.Error(), actualErr.Error())
	}

	s.write(recvMsg{buffer: mem.NewBuffer(&testData, nil), err: nil})
	s.write(recvMsg{buffer: mem.NewBuffer(&testData, nil), err: errors.New("different error from first")})

	for i := 0; i < 2; i++ {
		inBuf := make([]byte, 1)
		actualCount, actualErr := s.readTo(inBuf)
		if actualCount != 0 {
			t.Errorf("actualCount, _ := s.Read(_) differs; want %v; got %v", 0, actualCount)
		}
		if actualErr.Error() != testErr.Error() {
			t.Errorf("_ , actualErr := s.Read(_) differs; want actualErr.Error() to be %v; got %v", testErr.Error(), actualErr.Error())
		}
	}
}

// TestHeadersCausingStreamError tests headers that should cause a stream protocol
// error, which would end up with a RST_STREAM being sent to the client and also
// the server closing the stream.
func (s) TestHeadersCausingStreamError(t *testing.T) {
	tests := []struct {
		name    string
		headers []struct {
			name   string
			values []string
		}
	}{
		// "Transports must consider requests containing the Connection header
		// as malformed" - A41 Malformed requests map to a stream error of type
		// PROTOCOL_ERROR.
		{
			name: "Connection header present",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "content-type", values: []string{"application/grpc"}},
				{name: "connection", values: []string{"not-supported"}},
			},
		},
		// multiple :authority or multiple Host headers would make the eventual
		// :authority ambiguous as per A41. Since these headers won't have a
		// content-type that corresponds to a grpc-client, the server should
		// simply write a RST_STREAM to the wire.
		{
			// Note: multiple authority headers are handled by the framer
			// itself, which will cause a stream error. Thus, it will never get
			// to operateHeaders with the check in operateHeaders for stream
			// error, but the server transport will still send a stream error.
			name: "Multiple authority headers",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost", "localhost2"}},
				{name: "host", values: []string{"localhost"}},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			server := setUpServerOnly(t, 0, &ServerConfig{}, suspended)
			defer server.stop()
			// Create a client directly to not tie what you can send to API of
			// http2_client.go (i.e. control headers being sent).
			mconn, err := net.Dial("tcp", server.lis.Addr().String())
			if err != nil {
				t.Fatalf("Client failed to dial: %v", err)
			}
			defer mconn.Close()

			if n, err := mconn.Write(clientPreface); err != nil || n != len(clientPreface) {
				t.Fatalf("mconn.Write(clientPreface) = %d, %v, want %d, <nil>", n, err, len(clientPreface))
			}

			framer := http2.NewFramer(mconn, mconn)
			if err := framer.WriteSettings(); err != nil {
				t.Fatalf("Error while writing settings: %v", err)
			}

			// result chan indicates that reader received a RSTStream from server.
			// An error will be passed on it if any other frame is received.
			result := testutils.NewChannel()

			// Launch a reader goroutine.
			go func() {
				for {
					frame, err := framer.ReadFrame()
					if err != nil {
						return
					}
					switch frame := frame.(type) {
					case *http2.SettingsFrame:
						// Do nothing. A settings frame is expected from server preface.
					case *http2.RSTStreamFrame:
						if frame.Header().StreamID != 1 || http2.ErrCode(frame.ErrCode) != http2.ErrCodeProtocol {
							// Client only created a single stream, so RST Stream should be for that single stream.
							result.Send(fmt.Errorf("RST stream received with streamID: %d and code %v, want streamID: 1 and code: http.ErrCodeFlowControl", frame.Header().StreamID, http2.ErrCode(frame.ErrCode)))
						}
						// Records that client successfully received RST Stream frame.
						result.Send(nil)
						return
					default:
						// The server should send nothing but a single RST Stream frame.
						result.Send(errors.New("the client received a frame other than RST Stream"))
					}
				}
			}()

			var buf bytes.Buffer
			henc := hpack.NewEncoder(&buf)

			// Needs to build headers deterministically to conform to gRPC over
			// HTTP/2 spec.
			for _, header := range test.headers {
				for _, value := range header.values {
					if err := henc.WriteField(hpack.HeaderField{Name: header.name, Value: value}); err != nil {
						t.Fatalf("Error while encoding header: %v", err)
					}
				}
			}

			if err := framer.WriteHeaders(http2.HeadersFrameParam{StreamID: 1, BlockFragment: buf.Bytes(), EndHeaders: true}); err != nil {
				t.Fatalf("Error while writing headers: %v", err)
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			r, err := result.Receive(ctx)
			if err != nil {
				t.Fatalf("Error receiving from channel: %v", err)
			}
			if r != nil {
				t.Fatalf("want nil, got %v", r)
			}
		})
	}
}

// TestHeadersHTTPStatusGRPCStatus tests requests with certain headers get a
// certain HTTP and gRPC status back.
func (s) TestHeadersHTTPStatusGRPCStatus(t *testing.T) {
	tests := []struct {
		name    string
		headers []struct {
			name   string
			values []string
		}
		httpStatusWant  string
		grpcStatusWant  string
		grpcMessageWant string
	}{
		// Note: multiple authority headers are handled by the framer itself,
		// which will cause a stream error. Thus, it will never get to
		// operateHeaders with the check in operateHeaders for possible
		// grpc-status sent back.

		// multiple :authority or multiple Host headers would make the eventual
		// :authority ambiguous as per A41. This takes precedence even over the
		// fact a request is non grpc. All of these requests should be rejected
		// with grpc-status Internal. Thus, requests with multiple hosts should
		// get rejected with HTTP Status 400 and gRPC status Internal,
		// regardless of whether the client is speaking gRPC or not.
		{
			name: "Multiple host headers non grpc",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "host", values: []string{"localhost", "localhost2"}},
			},
			httpStatusWant:  "400",
			grpcStatusWant:  "13",
			grpcMessageWant: "both must only have 1 value as per HTTP/2 spec",
		},
		{
			name: "Multiple host headers grpc",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "content-type", values: []string{"application/grpc"}},
				{name: "host", values: []string{"localhost", "localhost2"}},
			},
			httpStatusWant:  "400",
			grpcStatusWant:  "13",
			grpcMessageWant: "both must only have 1 value as per HTTP/2 spec",
		},
		// If the client sends an HTTP/2 request with a :method header with a
		// value other than POST, as specified in the gRPC over HTTP/2
		// specification, the server should fail the RPC.
		{
			name: "Client Sending Wrong Method",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"PUT"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "content-type", values: []string{"application/grpc"}},
			},
			httpStatusWant:  "405",
			grpcStatusWant:  "13",
			grpcMessageWant: "which should be POST",
		},
		{
			name: "Client Sending Wrong Content-Type",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "content-type", values: []string{"application/json"}},
			},
			httpStatusWant:  "415",
			grpcStatusWant:  "3",
			grpcMessageWant: `invalid gRPC request content-type "application/json"`,
		},
		{
			name: "Client Sending Bad Timeout",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "content-type", values: []string{"application/grpc"}},
				{name: "grpc-timeout", values: []string{"18f6n"}},
			},
			httpStatusWant:  "400",
			grpcStatusWant:  "13",
			grpcMessageWant: "malformed grpc-timeout",
		},
		{
			name: "Client Sending Bad Binary Header",
			headers: []struct {
				name   string
				values []string
			}{
				{name: ":method", values: []string{"POST"}},
				{name: ":path", values: []string{"foo"}},
				{name: ":authority", values: []string{"localhost"}},
				{name: "content-type", values: []string{"application/grpc"}},
				{name: "foobar-bin", values: []string{"X()3e@#$-"}},
			},
			httpStatusWant:  "400",
			grpcStatusWant:  "13",
			grpcMessageWant: `header "foobar-bin": illegal base64 data`,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			server := setUpServerOnly(t, 0, &ServerConfig{}, suspended)
			defer server.stop()
			// Create a client directly to not tie what you can send to API of
			// http2_client.go (i.e. control headers being sent).
			mconn, err := net.Dial("tcp", server.lis.Addr().String())
			if err != nil {
				t.Fatalf("Client failed to dial: %v", err)
			}
			defer mconn.Close()

			if n, err := mconn.Write(clientPreface); err != nil || n != len(clientPreface) {
				t.Fatalf("mconn.Write(clientPreface) = %d, %v, want %d, <nil>", n, err, len(clientPreface))
			}

			framer := http2.NewFramer(mconn, mconn)
			framer.ReadMetaHeaders = hpack.NewDecoder(4096, nil)
			if err := framer.WriteSettings(); err != nil {
				t.Fatalf("Error while writing settings: %v", err)
			}

			// result chan indicates that reader received a Headers Frame with
			// desired grpc status and message from server. An error will be passed
			// on it if any other frame is received.
			result := testutils.NewChannel()

			// Launch a reader goroutine.
			go func() {
				for {
					frame, err := framer.ReadFrame()
					if err != nil {
						return
					}
					switch frame := frame.(type) {
					case *http2.SettingsFrame:
						// Do nothing. A settings frame is expected from server preface.
					case *http2.MetaHeadersFrame:
						var httpStatus, grpcStatus, grpcMessage string
						for _, header := range frame.Fields {
							if header.Name == ":status" {
								httpStatus = header.Value
							}
							if header.Name == "grpc-status" {
								grpcStatus = header.Value
							}
							if header.Name == "grpc-message" {
								grpcMessage = header.Value
							}
						}
						if httpStatus != test.httpStatusWant {
							result.Send(fmt.Errorf("incorrect HTTP Status got %v, want %v", httpStatus, test.httpStatusWant))
							return
						}
						if grpcStatus != test.grpcStatusWant { // grpc status code internal
							result.Send(fmt.Errorf("incorrect gRPC Status got %v, want %v", grpcStatus, test.grpcStatusWant))
							return
						}
						if !strings.Contains(grpcMessage, test.grpcMessageWant) {
							result.Send(fmt.Errorf("incorrect gRPC message, want %q got %q", test.grpcMessageWant, grpcMessage))
							return
						}

						// Records that client successfully received a HeadersFrame
						// with expected Trailers-Only response.
						result.Send(nil)
						return
					default:
						// The server should send nothing but a single Settings and Headers frame.
						result.Send(errors.New("the client received a frame other than Settings or Headers"))
					}
				}
			}()

			var buf bytes.Buffer
			henc := hpack.NewEncoder(&buf)

			// Needs to build headers deterministically to conform to gRPC over
			// HTTP/2 spec.
			for _, header := range test.headers {
				for _, value := range header.values {
					if err := henc.WriteField(hpack.HeaderField{Name: header.name, Value: value}); err != nil {
						t.Fatalf("Error while encoding header: %v", err)
					}
				}
			}

			if err := framer.WriteHeaders(http2.HeadersFrameParam{StreamID: 1, BlockFragment: buf.Bytes(), EndHeaders: true}); err != nil {
				t.Fatalf("Error while writing headers: %v", err)
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			r, err := result.Receive(ctx)
			if err != nil {
				t.Fatalf("Error receiving from channel: %v", err)
			}
			if r != nil {
				t.Fatalf("want nil, got %v", r)
			}
		})
	}
}

func (s) TestWriteHeaderConnectionError(t *testing.T) {
	server, client, cancel := setUp(t, 0, notifyCall)
	defer cancel()
	defer server.stop()

	waitWhileTrue(t, func() (bool, error) {
		server.mu.Lock()
		defer server.mu.Unlock()

		if len(server.conns) == 0 {
			return true, fmt.Errorf("timed-out while waiting for connection to be created on the server")
		}
		return false, nil
	})

	server.mu.Lock()

	if len(server.conns) != 1 {
		t.Fatalf("Server has %d connections from the client, want 1", len(server.conns))
	}

	// Get the server transport for the connection to the client.
	var serverTransport *http2Server
	for k := range server.conns {
		serverTransport = k.(*http2Server)
	}
	notifyChan := make(chan struct{})
	server.h.notify = notifyChan
	server.mu.Unlock()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cstream, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Client failed to create first stream. Err: %v", err)
	}

	<-notifyChan // Wait for server stream to be established.
	var sstream *Stream
	// Access stream on the server.
	serverTransport.mu.Lock()
	for _, v := range serverTransport.activeStreams {
		if v.id == cstream.id {
			sstream = v
		}
	}
	serverTransport.mu.Unlock()
	if sstream == nil {
		t.Fatalf("Didn't find stream corresponding to client cstream.id: %v on the server", cstream.id)
	}

	client.Close(fmt.Errorf("closed manually by test"))

	// Wait for server transport to be closed.
	<-serverTransport.done

	// Write header on a closed server transport.
	err = serverTransport.WriteHeader(sstream, metadata.MD{})
	st := status.Convert(err)
	if st.Code() != codes.Unavailable {
		t.Fatalf("WriteHeader() failed with status code %s, want %s", st.Code(), codes.Unavailable)
	}
}

func (s) TestPingPong1B(t *testing.T) {
	runPingPongTest(t, 1)
}

func TestPingPong1KB(t *testing.T) {
	runPingPongTest(t, 1024)
}

func TestPingPong64KB(t *testing.T) {
	runPingPongTest(t, 65536)
}

func (s) TestPingPong1MB(t *testing.T) {
	runPingPongTest(t, 1048576)
}

// This is a stress-test of flow control logic.
func runPingPongTest(t *testing.T, msgSize int) {
	server, client, cancel := setUp(t, 0, pingpong)
	defer cancel()
	defer server.stop()
	defer client.Close(fmt.Errorf("closed manually by test"))
	waitWhileTrue(t, func() (bool, error) {
		server.mu.Lock()
		defer server.mu.Unlock()
		if len(server.conns) == 0 {
			return true, fmt.Errorf("timed out while waiting for server transport to be created")
		}
		return false, nil
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("Failed to create stream. Err: %v", err)
	}
	msg := make([]byte, msgSize)
	outgoingHeader := make([]byte, 5)
	outgoingHeader[0] = byte(0)
	binary.BigEndian.PutUint32(outgoingHeader[1:], uint32(msgSize))
	opts := &Options{}
	incomingHeader := make([]byte, 5)

	ctx, cancel = context.WithTimeout(ctx, 10*time.Millisecond)
	defer cancel()
	for ctx.Err() == nil {
		if err := client.Write(stream, outgoingHeader, newBufferSlice(msg), opts); err != nil {
			t.Fatalf("Error on client while writing message. Err: %v", err)
		}
		if _, err := stream.readTo(incomingHeader); err != nil {
			t.Fatalf("Error on client while reading data header. Err: %v", err)
		}
		sz := binary.BigEndian.Uint32(incomingHeader[1:])
		recvMsg := make([]byte, int(sz))
		if _, err := stream.readTo(recvMsg); err != nil {
			t.Fatalf("Error on client while reading data. Err: %v", err)
		}
	}

	client.Write(stream, nil, nil, &Options{Last: true})
	if _, err := stream.readTo(incomingHeader); err != io.EOF {
		t.Fatalf("Client expected EOF from the server. Got: %v", err)
	}
}

type tableSizeLimit struct {
	mu     sync.Mutex
	limits []uint32
}

func (t *tableSizeLimit) add(limit uint32) {
	t.mu.Lock()
	t.limits = append(t.limits, limit)
	t.mu.Unlock()
}

func (t *tableSizeLimit) getLen() int {
	t.mu.Lock()
	defer t.mu.Unlock()
	return len(t.limits)
}

func (t *tableSizeLimit) getIndex(i int) uint32 {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.limits[i]
}

func (s) TestHeaderTblSize(t *testing.T) {
	limits := &tableSizeLimit{}
	updateHeaderTblSize = func(e *hpack.Encoder, v uint32) {
		e.SetMaxDynamicTableSizeLimit(v)
		limits.add(v)
	}
	defer func() {
		updateHeaderTblSize = func(e *hpack.Encoder, v uint32) {
			e.SetMaxDynamicTableSizeLimit(v)
		}
	}()

	server, ct, cancel := setUp(t, 0, normal)
	defer cancel()
	defer ct.Close(fmt.Errorf("closed manually by test"))
	defer server.stop()
	ctx, ctxCancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer ctxCancel()
	_, err := ct.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("failed to open stream: %v", err)
	}

	var svrTransport ServerTransport
	var i int
	for i = 0; i < 1000; i++ {
		server.mu.Lock()
		if len(server.conns) != 0 {
			server.mu.Unlock()
			break
		}
		server.mu.Unlock()
		time.Sleep(10 * time.Millisecond)
		continue
	}
	if i == 1000 {
		t.Fatalf("unable to create any server transport after 10s")
	}

	for st := range server.conns {
		svrTransport = st
		break
	}
	svrTransport.(*http2Server).controlBuf.put(&outgoingSettings{
		ss: []http2.Setting{
			{
				ID:  http2.SettingHeaderTableSize,
				Val: uint32(100),
			},
		},
	})

	for i = 0; i < 1000; i++ {
		if limits.getLen() != 1 {
			time.Sleep(10 * time.Millisecond)
			continue
		}
		if val := limits.getIndex(0); val != uint32(100) {
			t.Fatalf("expected limits[0] = 100, got %d", val)
		}
		break
	}
	if i == 1000 {
		t.Fatalf("expected len(limits) = 1 within 10s, got != 1")
	}

	ct.controlBuf.put(&outgoingSettings{
		ss: []http2.Setting{
			{
				ID:  http2.SettingHeaderTableSize,
				Val: uint32(200),
			},
		},
	})

	for i := 0; i < 1000; i++ {
		if limits.getLen() != 2 {
			time.Sleep(10 * time.Millisecond)
			continue
		}
		if val := limits.getIndex(1); val != uint32(200) {
			t.Fatalf("expected limits[1] = 200, got %d", val)
		}
		break
	}
	if i == 1000 {
		t.Fatalf("expected len(limits) = 2 within 10s, got != 2")
	}
}

// attrTransportCreds is a transport credential implementation which stores
// Attributes from the ClientHandshakeInfo struct passed in the context locally
// for the test to inspect.
type attrTransportCreds struct {
	credentials.TransportCredentials
	attr *attributes.Attributes
}

func (ac *attrTransportCreds) ClientHandshake(ctx context.Context, _ string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	ai := credentials.ClientHandshakeInfoFromContext(ctx)
	ac.attr = ai.Attributes
	return rawConn, nil, nil
}
func (ac *attrTransportCreds) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (ac *attrTransportCreds) Clone() credentials.TransportCredentials {
	return nil
}

// TestClientHandshakeInfo adds attributes to the resolver.Address passes to
// NewClientTransport and verifies that these attributes are received by the
// transport credential handshaker.
func (s) TestClientHandshakeInfo(t *testing.T) {
	server := setUpServerOnly(t, 0, &ServerConfig{}, pingpong)
	defer server.stop()

	const (
		testAttrKey = "foo"
		testAttrVal = "bar"
	)
	addr := resolver.Address{
		Addr:       "localhost:" + server.port,
		Attributes: attributes.New(testAttrKey, testAttrVal),
	}
	ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(2*time.Second))
	defer cancel()
	creds := &attrTransportCreds{}

	copts := ConnectOptions{
		TransportCredentials: creds,
		ChannelzParent:       channelzSubChannel(t),
	}
	tr, err := NewClientTransport(ctx, ctx, addr, copts, func(GoAwayReason) {})
	if err != nil {
		t.Fatalf("NewClientTransport(): %v", err)
	}
	defer tr.Close(fmt.Errorf("closed manually by test"))

	wantAttr := attributes.New(testAttrKey, testAttrVal)
	if gotAttr := creds.attr; !cmp.Equal(gotAttr, wantAttr, cmp.AllowUnexported(attributes.Attributes{})) {
		t.Fatalf("received attributes %v in creds, want %v", gotAttr, wantAttr)
	}
}

// TestClientHandshakeInfoDialer adds attributes to the resolver.Address passes to
// NewClientTransport and verifies that these attributes are received by a custom
// dialer.
func (s) TestClientHandshakeInfoDialer(t *testing.T) {
	server := setUpServerOnly(t, 0, &ServerConfig{}, pingpong)
	defer server.stop()

	const (
		testAttrKey = "foo"
		testAttrVal = "bar"
	)
	addr := resolver.Address{
		Addr:       "localhost:" + server.port,
		Attributes: attributes.New(testAttrKey, testAttrVal),
	}
	ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(2*time.Second))
	defer cancel()

	var attr *attributes.Attributes
	dialer := func(ctx context.Context, addr string) (net.Conn, error) {
		ai := credentials.ClientHandshakeInfoFromContext(ctx)
		attr = ai.Attributes
		return (&net.Dialer{}).DialContext(ctx, "tcp", addr)
	}

	copts := ConnectOptions{
		Dialer:         dialer,
		ChannelzParent: channelzSubChannel(t),
	}
	tr, err := NewClientTransport(ctx, ctx, addr, copts, func(GoAwayReason) {})
	if err != nil {
		t.Fatalf("NewClientTransport(): %v", err)
	}
	defer tr.Close(fmt.Errorf("closed manually by test"))

	wantAttr := attributes.New(testAttrKey, testAttrVal)
	if gotAttr := attr; !cmp.Equal(gotAttr, wantAttr, cmp.AllowUnexported(attributes.Attributes{})) {
		t.Errorf("Received attributes %v in custom dialer, want %v", gotAttr, wantAttr)
	}
}

func (s) TestClientDecodeHeaderStatusErr(t *testing.T) {
	testStream := func() *Stream {
		return &Stream{
			done:       make(chan struct{}),
			headerChan: make(chan struct{}),
			buf: &recvBuffer{
				c:  make(chan recvMsg),
				mu: sync.Mutex{},
			},
		}
	}

	testClient := func(ts *Stream) *http2Client {
		return &http2Client{
			mu: sync.Mutex{},
			activeStreams: map[uint32]*Stream{
				0: ts,
			},
			controlBuf: newControlBuffer(make(<-chan struct{})),
		}
	}

	for _, test := range []struct {
		name string
		// input
		metaHeaderFrame *http2.MetaHeadersFrame
		// output
		wantStatus *status.Status
	}{
		{
			name: "valid header",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					{Name: "content-type", Value: "application/grpc"},
					{Name: "grpc-status", Value: "0"},
					{Name: ":status", Value: "200"},
				},
			},
			// no error
			wantStatus: status.New(codes.OK, ""),
		},
		{
			name: "missing content-type header",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					{Name: "grpc-status", Value: "0"},
					{Name: ":status", Value: "200"},
				},
			},
			wantStatus: status.New(
				codes.Unknown,
				"malformed header: missing HTTP content-type",
			),
		},
		{
			name: "invalid grpc status header field",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					{Name: "content-type", Value: "application/grpc"},
					{Name: "grpc-status", Value: "xxxx"},
					{Name: ":status", Value: "200"},
				},
			},
			wantStatus: status.New(
				codes.Internal,
				"transport: malformed grpc-status: strconv.ParseInt: parsing \"xxxx\": invalid syntax",
			),
		},
		{
			name: "invalid http content type",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					{Name: "content-type", Value: "application/json"},
				},
			},
			wantStatus: status.New(
				codes.Internal,
				"malformed header: missing HTTP status; transport: received unexpected content-type \"application/json\"",
			),
		},
		{
			name: "http fallback and invalid http status",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					// No content type provided then fallback into handling http error.
					{Name: ":status", Value: "xxxx"},
				},
			},
			wantStatus: status.New(
				codes.Internal,
				"transport: malformed http-status: strconv.ParseInt: parsing \"xxxx\": invalid syntax",
			),
		},
		{
			name: "http2 frame size exceeds",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields:    nil,
				Truncated: true,
			},
			wantStatus: status.New(
				codes.Internal,
				"peer header list size exceeded limit",
			),
		},
		{
			name: "bad status in grpc mode",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					{Name: "content-type", Value: "application/grpc"},
					{Name: "grpc-status", Value: "0"},
					{Name: ":status", Value: "504"},
				},
			},
			wantStatus: status.New(
				codes.Unavailable,
				"unexpected HTTP status code received from server: 504 (Gateway Timeout)",
			),
		},
		{
			name: "missing http status",
			metaHeaderFrame: &http2.MetaHeadersFrame{
				Fields: []hpack.HeaderField{
					{Name: "content-type", Value: "application/grpc"},
				},
			},
			wantStatus: status.New(
				codes.Internal,
				"malformed header: missing HTTP status",
			),
		},
	} {

		t.Run(test.name, func(t *testing.T) {
			ts := testStream()
			s := testClient(ts)

			test.metaHeaderFrame.HeadersFrame = &http2.HeadersFrame{
				FrameHeader: http2.FrameHeader{
					StreamID: 0,
				},
			}

			s.operateHeaders(test.metaHeaderFrame)

			got := ts.status
			want := test.wantStatus
			if got.Code() != want.Code() || got.Message() != want.Message() {
				t.Fatalf("operateHeaders(%v); status = \ngot: %s\nwant: %s", test.metaHeaderFrame, got, want)
			}
		})
		t.Run(fmt.Sprintf("%s-end_stream", test.name), func(t *testing.T) {
			ts := testStream()
			s := testClient(ts)

			test.metaHeaderFrame.HeadersFrame = &http2.HeadersFrame{
				FrameHeader: http2.FrameHeader{
					StreamID: 0,
					Flags:    http2.FlagHeadersEndStream,
				},
			}

			s.operateHeaders(test.metaHeaderFrame)

			got := ts.status
			want := test.wantStatus
			if got.Code() != want.Code() || got.Message() != want.Message() {
				t.Fatalf("operateHeaders(%v); status = \ngot: %s\nwant: %s", test.metaHeaderFrame, got, want)
			}
		})
	}
}

func TestConnectionError_Unwrap(t *testing.T) {
	err := connectionErrorf(false, os.ErrNotExist, "unwrap me")
	if !errors.Is(err, os.ErrNotExist) {
		t.Error("ConnectionError does not unwrap")
	}
}

// Test that in the event of a graceful client transport shutdown, i.e.,
// clientTransport.Close(), client sends a goaway to the server with the correct
// error code and debug data.
func (s) TestClientSendsAGoAwayFrame(t *testing.T) {
	// Create a server.
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening: %v", err)
	}
	defer lis.Close()
	// greetDone is used to notify when server is done greeting the client.
	greetDone := make(chan struct{})
	// errorCh verifies that desired GOAWAY not received by server
	errorCh := make(chan error)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Launch the server.
	go func() {
		sconn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error while accepting: %v", err)
		}
		defer sconn.Close()
		if _, err := io.ReadFull(sconn, make([]byte, len(clientPreface))); err != nil {
			t.Errorf("Error while writing settings ack: %v", err)
			return
		}
		sfr := http2.NewFramer(sconn, sconn)
		if err := sfr.WriteSettings(); err != nil {
			t.Errorf("Error while writing settings %v", err)
			return
		}
		fr, _ := sfr.ReadFrame()
		if _, ok := fr.(*http2.SettingsFrame); !ok {
			t.Errorf("Expected settings frame, got %v", fr)
		}
		fr, _ = sfr.ReadFrame()
		if fr, ok := fr.(*http2.SettingsFrame); !ok || !fr.IsAck() {
			t.Errorf("Expected settings ACK frame, got %v", fr)
		}
		fr, _ = sfr.ReadFrame()
		if fr, ok := fr.(*http2.HeadersFrame); !ok || !fr.Flags.Has(http2.FlagHeadersEndHeaders) {
			t.Errorf("Expected Headers frame with END_HEADERS frame, got %v", fr)
		}
		close(greetDone)

		frame, err := sfr.ReadFrame()
		if err != nil {
			return
		}
		switch fr := frame.(type) {
		case *http2.GoAwayFrame:
			// Records that the server successfully received a GOAWAY frame.
			goAwayFrame := fr
			if goAwayFrame.ErrCode == http2.ErrCodeNo {
				t.Logf("Received goAway frame from client")
				close(errorCh)
			} else {
				errorCh <- fmt.Errorf("received unexpected goAway frame: %v", err)
				close(errorCh)
			}
			return
		default:
			errorCh <- fmt.Errorf("server received a frame other than GOAWAY: %v", err)
			close(errorCh)
			return
		}
	}()

	ct, err := NewClientTransport(ctx, ctx, resolver.Address{Addr: lis.Addr().String()}, ConnectOptions{}, func(GoAwayReason) {})
	if err != nil {
		t.Fatalf("Error while creating client transport: %v", err)
	}
	_, err = ct.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("failed to open stream: %v", err)
	}
	// Wait until server receives the headers and settings frame as part of greet.
	<-greetDone
	ct.Close(errors.New("manually closed by client"))
	t.Logf("Closed the client connection")
	select {
	case err := <-errorCh:
		if err != nil {
			t.Errorf("Error receiving the GOAWAY frame: %v", err)
		}
	case <-ctx.Done():
		t.Errorf("Context timed out")
	}
}

// hangingConn is a net.Conn wrapper for testing, simulating hanging connections
// after a GOAWAY frame is sent, of which Write operations pause until explicitly
// signaled or a timeout occurs.
type hangingConn struct {
	net.Conn
	hangConn     chan struct{}
	startHanging *atomic.Bool
}

func (hc *hangingConn) Write(b []byte) (n int, err error) {
	n, err = hc.Conn.Write(b)
	if hc.startHanging.Load() {
		<-hc.hangConn
	}
	return n, err
}

// Tests the scenario where a client transport is closed and writing of the
// GOAWAY frame as part of the close does not complete because of a network
// hang. The test verifies that the client transport is closed without waiting
// for too long.
func (s) TestClientCloseReturnsEarlyWhenGoAwayWriteHangs(t *testing.T) {
	// Override timer for writing GOAWAY to 0 so that the connection write
	// always times out. It is equivalent of real network hang when conn
	// write for goaway doesn't finish in specified deadline
	origGoAwayLoopyTimeout := goAwayLoopyWriterTimeout
	goAwayLoopyWriterTimeout = time.Millisecond
	defer func() {
		goAwayLoopyWriterTimeout = origGoAwayLoopyTimeout
	}()

	// Create the server set up.
	connectCtx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	server := setUpServerOnly(t, 0, &ServerConfig{}, normal)
	defer server.stop()
	addr := resolver.Address{Addr: "localhost:" + server.port}
	isGreetingDone := &atomic.Bool{}
	hangConn := make(chan struct{})
	defer close(hangConn)
	dialer := func(_ context.Context, addr string) (net.Conn, error) {
		conn, err := net.Dial("tcp", addr)
		if err != nil {
			return nil, err
		}
		return &hangingConn{Conn: conn, hangConn: hangConn, startHanging: isGreetingDone}, nil
	}
	copts := ConnectOptions{Dialer: dialer}
	copts.ChannelzParent = channelzSubChannel(t)
	// Create client transport with custom dialer
	ct, connErr := NewClientTransport(connectCtx, context.Background(), addr, copts, func(GoAwayReason) {})
	if connErr != nil {
		t.Fatalf("failed to create transport: %v", connErr)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ct.NewStream(ctx, &CallHdr{}); err != nil {
		t.Fatalf("Failed to open stream: %v", err)
	}

	isGreetingDone.Store(true)
	ct.Close(errors.New("manually closed by client"))
}

// TestReadHeaderMultipleBuffers tests the stream when the gRPC headers are
// split across multiple buffers. It verifies that the reporting of the
// number of bytes read for flow control is correct.
func (s) TestReadHeaderMultipleBuffers(t *testing.T) {
	headerLen := 5
	recvBuffer := newRecvBuffer()
	recvBuffer.put(recvMsg{buffer: make(mem.SliceBuffer, 3)})
	recvBuffer.put(recvMsg{buffer: make(mem.SliceBuffer, headerLen-3)})
	bytesRead := 0
	s := Stream{
		requestRead: func(int) {},
		trReader: &transportReader{
			reader: &recvBufferReader{
				recv: recvBuffer,
			},
			windowHandler: func(i int) {
				bytesRead += i
			},
		},
	}

	header := make([]byte, headerLen)
	err := s.ReadHeader(header)
	if err != nil {
		t.Fatalf("ReadHeader(%v) = %v", header, err)
	}
	if bytesRead != headerLen {
		t.Errorf("bytesRead = %d, want = %d", bytesRead, headerLen)
	}
}
