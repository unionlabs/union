// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"testing"
)

func TestConnInflowReturnOnRead(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, func(c *Config) {
		c.MaxConnReadBufferSize = 64
	})
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   s.id,
		data: make([]byte, 8),
	})
	if n, err := s.Read(make([]byte, 8)); n != 8 || err != nil {
		t.Fatalf("s.Read() = %v, %v; want %v, nil", n, err, 8)
	}
	tc.wantFrame("available window increases, send a MAX_DATA",
		packetType1RTT, debugFrameMaxData{
			max: 64 + 8,
		})
	// Peer can write up to the new limit.
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   s.id,
		off:  8,
		data: make([]byte, 64),
	})
	if n, err := s.Read(make([]byte, 64+1)); n != 64 {
		t.Fatalf("s.Read() = %v, %v; want %v, anything", n, err, 64)
	}
	tc.wantFrame("available window increases, send a MAX_DATA",
		packetType1RTT, debugFrameMaxData{
			max: 64 + 8 + 64,
		})
	tc.wantIdle("connection is idle")
}

func TestConnInflowReturnOnRacingReads(t *testing.T) {
	// Perform two reads at the same time,
	// one for half of MaxConnReadBufferSize
	// and one for one byte.
	//
	// We should observe a single MAX_DATA update.
	// Depending on the ordering of events,
	// this may include the credit from just the larger read
	// or the credit from both.
	ctx := canceledContext()
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.MaxConnReadBufferSize = 64
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, uniStream, 0),
		data: make([]byte, 16),
	})
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, uniStream, 1),
		data: make([]byte, 1),
	})
	s1, err := tc.conn.AcceptStream(ctx)
	if err != nil {
		t.Fatalf("conn.AcceptStream() = %v", err)
	}
	s2, err := tc.conn.AcceptStream(ctx)
	if err != nil {
		t.Fatalf("conn.AcceptStream() = %v", err)
	}
	read1 := runAsync(tc, func(ctx context.Context) (int, error) {
		return s1.Read(make([]byte, 16))
	})
	read2 := runAsync(tc, func(ctx context.Context) (int, error) {
		return s2.Read(make([]byte, 1))
	})
	// This MAX_DATA might extend the window by 16 or 17, depending on
	// whether the second write occurs before the update happens.
	tc.wantFrameType("MAX_DATA update is sent",
		packetType1RTT, debugFrameMaxData{})
	tc.wantIdle("redundant MAX_DATA is not sent")
	if _, err := read1.result(); err != nil {
		t.Errorf("Read #1 = %v", err)
	}
	if _, err := read2.result(); err != nil {
		t.Errorf("Read #2 = %v", err)
	}
}

func TestConnInflowReturnOnClose(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, func(c *Config) {
		c.MaxConnReadBufferSize = 64
	})
	tc.ignoreFrame(frameTypeStopSending)
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   s.id,
		data: make([]byte, 64),
	})
	s.CloseRead()
	tc.wantFrame("closing stream updates connection-level flow control",
		packetType1RTT, debugFrameMaxData{
			max: 128,
		})
}

func TestConnInflowReturnOnReset(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, func(c *Config) {
		c.MaxConnReadBufferSize = 64
	})
	tc.ignoreFrame(frameTypeStopSending)
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   s.id,
		data: make([]byte, 32),
	})
	tc.writeFrames(packetType1RTT, debugFrameResetStream{
		id:        s.id,
		finalSize: 64,
	})
	s.CloseRead()
	tc.wantFrame("receiving stream reseet updates connection-level flow control",
		packetType1RTT, debugFrameMaxData{
			max: 128,
		})
}

func TestConnInflowStreamViolation(t *testing.T) {
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.MaxConnReadBufferSize = 100
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	// Total MAX_DATA consumed: 50
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, bidiStream, 0),
		data: make([]byte, 50),
	})
	// Total MAX_DATA consumed: 80
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, uniStream, 0),
		off:  20,
		data: make([]byte, 10),
	})
	// Total MAX_DATA consumed: 100
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:  newStreamID(clientSide, bidiStream, 0),
		off: 70,
		fin: true,
	})
	// This stream has already consumed quota for these bytes.
	// Total MAX_DATA consumed: 100
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, uniStream, 0),
		data: make([]byte, 20),
	})
	tc.wantIdle("peer has consumed all MAX_DATA quota")

	// Total MAX_DATA consumed: 101
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, bidiStream, 2),
		data: make([]byte, 1),
	})
	tc.wantFrame("peer violates MAX_DATA limit",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errFlowControl,
		})
}

func TestConnInflowResetViolation(t *testing.T) {
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.MaxConnReadBufferSize = 100
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:   newStreamID(clientSide, bidiStream, 0),
		data: make([]byte, 100),
	})
	tc.wantIdle("peer has consumed all MAX_DATA quota")

	tc.writeFrames(packetType1RTT, debugFrameResetStream{
		id:        newStreamID(clientSide, uniStream, 0),
		finalSize: 0,
	})
	tc.wantIdle("stream reset does not consume MAX_DATA quota, no error")

	tc.writeFrames(packetType1RTT, debugFrameResetStream{
		id:        newStreamID(clientSide, uniStream, 1),
		finalSize: 1,
	})
	tc.wantFrame("RESET_STREAM final size violates MAX_DATA limit",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errFlowControl,
		})
}

func TestConnInflowMultipleStreams(t *testing.T) {
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.MaxConnReadBufferSize = 128
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	var streams []*Stream
	for _, id := range []streamID{
		newStreamID(clientSide, uniStream, 0),
		newStreamID(clientSide, uniStream, 1),
		newStreamID(clientSide, bidiStream, 0),
		newStreamID(clientSide, bidiStream, 1),
	} {
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   id,
			data: make([]byte, 1),
		})
		s := tc.acceptStream()
		streams = append(streams, s)
		if n, err := s.Read(make([]byte, 1)); err != nil || n != 1 {
			t.Fatalf("s.Read() = %v, %v; want 1, nil", n, err)
		}
	}
	tc.wantIdle("streams have read data, but not enough to update MAX_DATA")

	for _, s := range streams {
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  1,
			data: make([]byte, 31),
		})
	}

	if n, err := streams[0].Read(make([]byte, 32)); n != 31 {
		t.Fatalf("s.Read() = %v, %v; want 31, anything", n, err)
	}
	tc.wantFrame("read enough data to trigger a MAX_DATA update",
		packetType1RTT, debugFrameMaxData{
			max: 128 + 32 + 1 + 1 + 1,
		})

	tc.ignoreFrame(frameTypeStopSending)
	streams[2].CloseRead()
	tc.wantFrame("closed stream triggers another MAX_DATA update",
		packetType1RTT, debugFrameMaxData{
			max: 128 + 32 + 1 + 32 + 1,
		})
}

func TestConnOutflowBlocked(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, clientSide, uniStream,
		permissiveTransportParameters,
		func(p *transportParameters) {
			p.initialMaxData = 10
		})
	tc.ignoreFrame(frameTypeAck)

	data := makeTestData(32)
	n, err := s.Write(data)
	if n != len(data) || err != nil {
		t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(data))
	}
	s.Flush()

	tc.wantFrame("stream writes data up to MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data[:10],
		})
	tc.wantIdle("stream is blocked by MAX_DATA limit")

	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 20,
	})
	tc.wantFrame("stream writes data up to new MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  10,
			data: data[10:20],
		})
	tc.wantIdle("stream is blocked by new MAX_DATA limit")

	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 100,
	})
	tc.wantFrame("stream writes remaining data",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  20,
			data: data[20:],
		})
}

func TestConnOutflowMaxDataDecreases(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, clientSide, uniStream,
		permissiveTransportParameters,
		func(p *transportParameters) {
			p.initialMaxData = 10
		})
	tc.ignoreFrame(frameTypeAck)

	// Decrease in MAX_DATA is ignored.
	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 5,
	})

	data := makeTestData(32)
	n, err := s.Write(data)
	if n != len(data) || err != nil {
		t.Fatalf("s.Write() = %v, %v; want %v, nil", n, err, len(data))
	}
	s.Flush()

	tc.wantFrame("stream writes data up to MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data[:10],
		})
}

func TestConnOutflowMaxDataRoundRobin(t *testing.T) {
	ctx := canceledContext()
	tc := newTestConn(t, clientSide, permissiveTransportParameters,
		func(p *transportParameters) {
			p.initialMaxData = 0
		})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	s1, err := tc.conn.newLocalStream(ctx, uniStream)
	if err != nil {
		t.Fatalf("conn.newLocalStream(%v) = %v", uniStream, err)
	}
	s2, err := tc.conn.newLocalStream(ctx, uniStream)
	if err != nil {
		t.Fatalf("conn.newLocalStream(%v) = %v", uniStream, err)
	}

	s1.Write(make([]byte, 10))
	s1.Flush()
	s2.Write(make([]byte, 10))
	s2.Flush()

	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 1,
	})
	tc.wantFrame("stream 1 writes data up to MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s1.id,
			data: []byte{0},
		})

	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 2,
	})
	tc.wantFrame("stream 2 writes data up to MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s2.id,
			data: []byte{0},
		})

	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 3,
	})
	tc.wantFrame("stream 1 writes data up to MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s1.id,
			off:  1,
			data: []byte{0},
		})
}

func TestConnOutflowMetaAndData(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, clientSide, bidiStream,
		permissiveTransportParameters,
		func(p *transportParameters) {
			p.initialMaxData = 0
		})
	tc.ignoreFrame(frameTypeAck)

	data := makeTestData(32)
	s.Write(data)
	s.Flush()

	s.CloseRead()
	tc.wantFrame("CloseRead sends a STOP_SENDING, not flow controlled",
		packetType1RTT, debugFrameStopSending{
			id: s.id,
		})

	tc.writeFrames(packetType1RTT, debugFrameMaxData{
		max: 100,
	})
	tc.wantFrame("unblocked MAX_DATA",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data,
		})
}

func TestConnOutflowResentData(t *testing.T) {
	tc, s := newTestConnAndLocalStream(t, clientSide, bidiStream,
		permissiveTransportParameters,
		func(p *transportParameters) {
			p.initialMaxData = 10
		})
	tc.ignoreFrame(frameTypeAck)

	data := makeTestData(15)
	s.Write(data[:8])
	s.Flush()
	tc.wantFrame("data is under MAX_DATA limit, all sent",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data[:8],
		})

	// Lose the last STREAM packet.
	const pto = false
	tc.triggerLossOrPTO(packetType1RTT, false)
	tc.wantFrame("lost STREAM data is retransmitted",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			data: data[:8],
		})

	s.Write(data[8:])
	s.Flush()
	tc.wantFrame("new data is sent up to the MAX_DATA limit",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  8,
			data: data[8:10],
		})
}
