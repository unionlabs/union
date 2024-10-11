// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"fmt"
	"io"
	"math"
	"sync"
	"testing"
)

func TestStreamsCreate(t *testing.T) {
	ctx := canceledContext()
	tc := newTestConn(t, clientSide, permissiveTransportParameters)
	tc.handshake()

	s, err := tc.conn.NewStream(ctx)
	if err != nil {
		t.Fatalf("NewStream: %v", err)
	}
	s.Flush() // open the stream
	tc.wantFrame("created bidirectional stream 0",
		packetType1RTT, debugFrameStream{
			id:   0, // client-initiated, bidi, number 0
			data: []byte{},
		})

	s, err = tc.conn.NewSendOnlyStream(ctx)
	if err != nil {
		t.Fatalf("NewStream: %v", err)
	}
	s.Flush() // open the stream
	tc.wantFrame("created unidirectional stream 0",
		packetType1RTT, debugFrameStream{
			id:   2, // client-initiated, uni, number 0
			data: []byte{},
		})

	s, err = tc.conn.NewStream(ctx)
	if err != nil {
		t.Fatalf("NewStream: %v", err)
	}
	s.Flush() // open the stream
	tc.wantFrame("created bidirectional stream 1",
		packetType1RTT, debugFrameStream{
			id:   4, // client-initiated, uni, number 4
			data: []byte{},
		})
}

func TestStreamsAccept(t *testing.T) {
	ctx := canceledContext()
	tc := newTestConn(t, serverSide)
	tc.handshake()

	tc.writeFrames(packetType1RTT,
		debugFrameStream{
			id: 0, // client-initiated, bidi, number 0
		},
		debugFrameStream{
			id: 2, // client-initiated, uni, number 0
		},
		debugFrameStream{
			id: 4, // client-initiated, bidi, number 1
		})

	for _, accept := range []struct {
		id       streamID
		readOnly bool
	}{
		{0, false},
		{2, true},
		{4, false},
	} {
		s, err := tc.conn.AcceptStream(ctx)
		if err != nil {
			t.Fatalf("conn.AcceptStream() = %v, want stream %v", err, accept.id)
		}
		if got, want := s.id, accept.id; got != want {
			t.Fatalf("conn.AcceptStream() = stream %v, want %v", got, want)
		}
		if got, want := s.IsReadOnly(), accept.readOnly; got != want {
			t.Fatalf("stream %v: s.IsReadOnly() = %v, want %v", accept.id, got, want)
		}
	}

	_, err := tc.conn.AcceptStream(ctx)
	if err != context.Canceled {
		t.Fatalf("conn.AcceptStream() = %v, want context.Canceled", err)
	}
}

func TestStreamsBlockingAccept(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()

	a := runAsync(tc, func(ctx context.Context) (*Stream, error) {
		return tc.conn.AcceptStream(ctx)
	})
	if _, err := a.result(); err != errNotDone {
		tc.t.Fatalf("AcceptStream() = _, %v; want errNotDone", err)
	}

	sid := newStreamID(clientSide, bidiStream, 0)
	tc.writeFrames(packetType1RTT,
		debugFrameStream{
			id: sid,
		})

	s, err := a.result()
	if err != nil {
		t.Fatalf("conn.AcceptStream() = _, %v, want stream", err)
	}
	if got, want := s.id, sid; got != want {
		t.Fatalf("conn.AcceptStream() = stream %v, want %v", got, want)
	}
	if got, want := s.IsReadOnly(), false; got != want {
		t.Fatalf("s.IsReadOnly() = %v, want %v", got, want)
	}
}

func TestStreamsLocalStreamNotCreated(t *testing.T) {
	// "An endpoint MUST terminate the connection with error STREAM_STATE_ERROR
	// if it receives a STREAM frame for a locally initiated stream that has
	// not yet been created [...]"
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.8-3
	tc := newTestConn(t, serverSide)
	tc.handshake()

	tc.writeFrames(packetType1RTT,
		debugFrameStream{
			id: 1, // server-initiated, bidi, number 0
		})
	tc.wantFrame("peer sent STREAM frame for an uncreated local stream",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errStreamState,
		})
}

func TestStreamsLocalStreamClosed(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, clientSide, uniStream, permissiveTransportParameters)
	s.CloseWrite()
	tc.wantFrame("FIN for closed stream",
		packetType1RTT, debugFrameStream{
			id:   newStreamID(clientSide, uniStream, 0),
			fin:  true,
			data: []byte{},
		})
	tc.writeAckForAll()

	tc.writeFrames(packetType1RTT, debugFrameStopSending{
		id: newStreamID(clientSide, uniStream, 0),
	})
	tc.wantIdle("frame for finalized stream is ignored")

	// ACKing the last stream packet should have cleaned up the stream.
	// Check that we don't have any state left.
	if got := len(tc.conn.streams.streams); got != 0 {
		t.Fatalf("after close, len(tc.conn.streams.streams) = %v, want 0", got)
	}
	if tc.conn.streams.queueMeta.head != nil {
		t.Fatalf("after close, stream send queue is not empty; should be")
	}
}

func TestStreamsStreamSendOnly(t *testing.T) {
	// "An endpoint MUST terminate the connection with error STREAM_STATE_ERROR
	// if it receives a STREAM frame for a locally initiated stream that has
	// not yet been created [...]"
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.8-3
	ctx := canceledContext()
	tc := newTestConn(t, serverSide, permissiveTransportParameters)
	tc.handshake()

	s, err := tc.conn.NewSendOnlyStream(ctx)
	if err != nil {
		t.Fatalf("NewStream: %v", err)
	}
	s.Flush() // open the stream
	tc.wantFrame("created unidirectional stream 0",
		packetType1RTT, debugFrameStream{
			id:   3, // server-initiated, uni, number 0
			data: []byte{},
		})

	tc.writeFrames(packetType1RTT,
		debugFrameStream{
			id: 3, // server-initiated, bidi, number 0
		})
	tc.wantFrame("peer sent STREAM frame for a send-only stream",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errStreamState,
		})
}

func TestStreamsWriteQueueFairness(t *testing.T) {
	ctx := canceledContext()
	const dataLen = 1 << 20
	const numStreams = 3
	tc := newTestConn(t, clientSide, func(p *transportParameters) {
		p.initialMaxStreamsBidi = numStreams
		p.initialMaxData = 1<<62 - 1
		p.initialMaxStreamDataBidiRemote = dataLen
	}, func(c *Config) {
		c.MaxStreamWriteBufferSize = dataLen
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	// Create a number of streams, and write a bunch of data to them.
	// The streams are not limited by flow control.
	//
	// The first stream we create is going to immediately consume all
	// available congestion window.
	//
	// Once we've created all the remaining streams,
	// we start sending acks back to open up the congestion window.
	// We verify that all streams can make progress.
	data := make([]byte, dataLen)
	var streams []*Stream
	for i := 0; i < numStreams; i++ {
		s, err := tc.conn.NewStream(ctx)
		if err != nil {
			t.Fatal(err)
		}
		streams = append(streams, s)
		if n, err := s.Write(data); n != len(data) || err != nil {
			t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(data))
		}
		// Wait for the stream to finish writing whatever frames it can before
		// congestion control blocks it.
		tc.wait()
	}

	sent := make([]int64, len(streams))
	for {
		p := tc.readPacket()
		if p == nil {
			break
		}
		tc.writeFrames(packetType1RTT, debugFrameAck{
			ranges: []i64range[packetNumber]{{0, p.num}},
		})
		for _, f := range p.frames {
			sf, ok := f.(debugFrameStream)
			if !ok {
				t.Fatalf("got unexpected frame (want STREAM): %v", sf)
			}
			if got, want := sf.off, sent[sf.id.num()]; got != want {
				t.Fatalf("got frame: %v\nwant offset: %v", sf, want)
			}
			sent[sf.id.num()] = sf.off + int64(len(sf.data))
			// Look at the amount of data sent by all streams, excluding the first one.
			// (The first stream got a head start when it consumed the initial window.)
			//
			// We expect that difference between the streams making the most and least progress
			// so far will be less than the maximum datagram size.
			minSent := sent[1]
			maxSent := sent[1]
			for _, s := range sent[2:] {
				minSent = min(minSent, s)
				maxSent = max(maxSent, s)
			}
			const maxDelta = maxUDPPayloadSize
			if d := maxSent - minSent; d > maxDelta {
				t.Fatalf("stream data sent: %v; delta=%v, want delta <= %v", sent, d, maxDelta)
			}
		}
	}
	// Final check that every stream sent the full amount of data expected.
	for num, s := range sent {
		if s != dataLen {
			t.Errorf("stream %v sent %v bytes, want %v", num, s, dataLen)
		}
	}
}

func TestStreamsShutdown(t *testing.T) {
	// These tests verify that a stream is removed from the Conn's map of live streams
	// after it is fully shut down.
	//
	// Each case consists of a setup step, after which one stream should exist,
	// and a shutdown step, after which no streams should remain in the Conn.
	for _, test := range []struct {
		name     string
		side     streamSide
		styp     streamType
		setup    func(*testing.T, *testConn, *Stream)
		shutdown func(*testing.T, *testConn, *Stream)
	}{{
		name: "closed",
		side: localStream,
		styp: uniStream,
		setup: func(t *testing.T, tc *testConn, s *Stream) {
			s.Close()
		},
		shutdown: func(t *testing.T, tc *testConn, s *Stream) {
			tc.writeAckForAll()
		},
	}, {
		name: "local close",
		side: localStream,
		styp: bidiStream,
		setup: func(t *testing.T, tc *testConn, s *Stream) {
			tc.writeFrames(packetType1RTT, debugFrameResetStream{
				id: s.id,
			})
			s.Close()
		},
		shutdown: func(t *testing.T, tc *testConn, s *Stream) {
			tc.writeAckForAll()
		},
	}, {
		name: "remote reset",
		side: localStream,
		styp: bidiStream,
		setup: func(t *testing.T, tc *testConn, s *Stream) {
			s.Close()
			tc.wantIdle("all frames after Close are ignored")
			tc.writeAckForAll()
		},
		shutdown: func(t *testing.T, tc *testConn, s *Stream) {
			tc.writeFrames(packetType1RTT, debugFrameResetStream{
				id: s.id,
			})
		},
	}, {
		name: "local close",
		side: remoteStream,
		styp: uniStream,
		setup: func(t *testing.T, tc *testConn, s *Stream) {
			tc.writeFrames(packetType1RTT, debugFrameStream{
				id:  s.id,
				fin: true,
			})
			if n, err := s.Read(make([]byte, 16)); n != 0 || err != io.EOF {
				t.Errorf("Read() = %v, %v; want 0, io.EOF", n, err)
			}
		},
		shutdown: func(t *testing.T, tc *testConn, s *Stream) {
			s.CloseRead()
		},
	}} {
		name := fmt.Sprintf("%v/%v/%v", test.side, test.styp, test.name)
		t.Run(name, func(t *testing.T) {
			tc, s := newTestConnAndStream(t, serverSide, test.side, test.styp,
				permissiveTransportParameters)
			tc.ignoreFrame(frameTypeStreamBase)
			tc.ignoreFrame(frameTypeStopSending)
			test.setup(t, tc, s)
			tc.wantIdle("conn should be idle after setup")
			if got, want := len(tc.conn.streams.streams), 1; got != want {
				t.Fatalf("after setup: %v streams in Conn's map; want %v", got, want)
			}
			test.shutdown(t, tc, s)
			tc.wantIdle("conn should be idle after shutdown")
			if got, want := len(tc.conn.streams.streams), 0; got != want {
				t.Fatalf("after shutdown: %v streams in Conn's map; want %v", got, want)
			}
		})
	}
}

func TestStreamsCreateAndCloseRemote(t *testing.T) {
	// This test exercises creating new streams in response to frames
	// from the peer, and cleaning up after streams are fully closed.
	//
	// It's overfitted to the current implementation, but works through
	// a number of corner cases in that implementation.
	//
	// Disable verbose logging in this test: It sends a lot of packets,
	// and they're not especially interesting on their own.
	defer func(vv bool) {
		*testVV = vv
	}(*testVV)
	*testVV = false
	ctx := canceledContext()
	tc := newTestConn(t, serverSide, permissiveTransportParameters)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	type op struct {
		id streamID
	}
	type streamOp op
	type resetOp op
	type acceptOp op
	const noStream = math.MaxInt64
	stringID := func(id streamID) string {
		return fmt.Sprintf("%v/%v", id.streamType(), id.num())
	}
	for _, op := range []any{
		"opening bidi/5 implicitly opens bidi/0-4",
		streamOp{newStreamID(clientSide, bidiStream, 5)},
		acceptOp{newStreamID(clientSide, bidiStream, 5)},
		"bidi/3 was implicitly opened",
		streamOp{newStreamID(clientSide, bidiStream, 3)},
		acceptOp{newStreamID(clientSide, bidiStream, 3)},
		resetOp{newStreamID(clientSide, bidiStream, 3)},
		"bidi/3 is done, frames for it are discarded",
		streamOp{newStreamID(clientSide, bidiStream, 3)},
		"open and close some uni streams as well",
		streamOp{newStreamID(clientSide, uniStream, 0)},
		acceptOp{newStreamID(clientSide, uniStream, 0)},
		streamOp{newStreamID(clientSide, uniStream, 1)},
		acceptOp{newStreamID(clientSide, uniStream, 1)},
		streamOp{newStreamID(clientSide, uniStream, 2)},
		acceptOp{newStreamID(clientSide, uniStream, 2)},
		resetOp{newStreamID(clientSide, uniStream, 1)},
		resetOp{newStreamID(clientSide, uniStream, 0)},
		resetOp{newStreamID(clientSide, uniStream, 2)},
		"closing an implicitly opened stream causes us to accept it",
		resetOp{newStreamID(clientSide, bidiStream, 0)},
		acceptOp{newStreamID(clientSide, bidiStream, 0)},
		resetOp{newStreamID(clientSide, bidiStream, 1)},
		acceptOp{newStreamID(clientSide, bidiStream, 1)},
		resetOp{newStreamID(clientSide, bidiStream, 2)},
		acceptOp{newStreamID(clientSide, bidiStream, 2)},
		"stream bidi/3 was reset previously",
		resetOp{newStreamID(clientSide, bidiStream, 3)},
		resetOp{newStreamID(clientSide, bidiStream, 4)},
		acceptOp{newStreamID(clientSide, bidiStream, 4)},
		"stream bidi/5 was reset previously",
		resetOp{newStreamID(clientSide, bidiStream, 5)},
		"stream bidi/6 was not implicitly opened",
		resetOp{newStreamID(clientSide, bidiStream, 6)},
		acceptOp{newStreamID(clientSide, bidiStream, 6)},
	} {
		if _, ok := op.(acceptOp); !ok {
			if s, err := tc.conn.AcceptStream(ctx); err == nil {
				t.Fatalf("accepted stream %v, want none", stringID(s.id))
			}
		}
		switch op := op.(type) {
		case string:
			t.Log("# " + op)
		case streamOp:
			t.Logf("open stream %v", stringID(op.id))
			tc.writeFrames(packetType1RTT, debugFrameStream{
				id: streamID(op.id),
			})
		case resetOp:
			t.Logf("reset stream %v", stringID(op.id))
			tc.writeFrames(packetType1RTT, debugFrameResetStream{
				id: op.id,
			})
		case acceptOp:
			s := tc.acceptStream()
			if s.id != op.id {
				t.Fatalf("accepted stream %v; want stream %v", stringID(s.id), stringID(op.id))
			}
			t.Logf("accepted stream %v", stringID(op.id))
			// Immediately close the stream, so the stream becomes done when the
			// peer closes its end.
			s.Close()
		}
		p := tc.readPacket()
		if p != nil {
			tc.writeFrames(p.ptype, debugFrameAck{
				ranges: []i64range[packetNumber]{{0, p.num + 1}},
			})
		}
	}
	// Every stream should be fully closed now.
	// Check that we don't have any state left.
	if got := len(tc.conn.streams.streams); got != 0 {
		t.Fatalf("after test, len(tc.conn.streams.streams) = %v, want 0", got)
	}
	if tc.conn.streams.queueMeta.head != nil {
		t.Fatalf("after test, stream send queue is not empty; should be")
	}
}

func TestStreamsCreateConcurrency(t *testing.T) {
	cli, srv := newLocalConnPair(t, &Config{}, &Config{})

	srvdone := make(chan int)
	go func() {
		defer close(srvdone)
		for streams := 0; ; streams++ {
			s, err := srv.AcceptStream(context.Background())
			if err != nil {
				srvdone <- streams
				return
			}
			s.Close()
		}
	}()

	var wg sync.WaitGroup
	const concurrency = 10
	const streams = 10
	for i := 0; i < concurrency; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < streams; j++ {
				s, err := cli.NewStream(context.Background())
				if err != nil {
					t.Errorf("NewStream: %v", err)
					return
				}
				s.Flush()
				_, err = io.ReadAll(s)
				if err != nil {
					t.Errorf("ReadFull: %v", err)
				}
				s.Close()
			}
		}()
	}
	wg.Wait()

	cli.Abort(nil)
	srv.Abort(nil)
	if got, want := <-srvdone, concurrency*streams; got != want {
		t.Errorf("accepted %v streams, want %v", got, want)
	}
}

func TestStreamsPTOWithImplicitStream(t *testing.T) {
	ctx := canceledContext()
	tc := newTestConn(t, serverSide, permissiveTransportParameters)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	// Peer creates stream 1, and implicitly creates stream 0.
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id: newStreamID(clientSide, bidiStream, 1),
	})

	// We accept stream 1 and write data to it.
	data := []byte("data")
	s, err := tc.conn.AcceptStream(ctx)
	if err != nil {
		t.Fatalf("conn.AcceptStream() = %v, want stream", err)
	}
	s.Write(data)
	s.Flush()
	tc.wantFrame("data written to stream",
		packetType1RTT, debugFrameStream{
			id:   newStreamID(clientSide, bidiStream, 1),
			data: data,
		})

	// PTO expires, and the data is resent.
	const pto = true
	tc.triggerLossOrPTO(packetType1RTT, true)
	tc.wantFrame("data resent after PTO expires",
		packetType1RTT, debugFrameStream{
			id:   newStreamID(clientSide, bidiStream, 1),
			data: data,
		})
}
