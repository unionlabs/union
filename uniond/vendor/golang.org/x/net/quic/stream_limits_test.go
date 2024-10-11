// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"crypto/tls"
	"testing"
)

func TestStreamLimitNewStreamBlocked(t *testing.T) {
	// "An endpoint that receives a frame with a stream ID exceeding the limit
	// it has sent MUST treat this as a connection error of type STREAM_LIMIT_ERROR [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.6-3
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		ctx := canceledContext()
		tc := newTestConn(t, clientSide,
			permissiveTransportParameters,
			func(p *transportParameters) {
				p.initialMaxStreamsBidi = 0
				p.initialMaxStreamsUni = 0
			})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		opening := runAsync(tc, func(ctx context.Context) (*Stream, error) {
			return tc.conn.newLocalStream(ctx, styp)
		})
		if _, err := opening.result(); err != errNotDone {
			t.Fatalf("new stream blocked by limit: %v, want errNotDone", err)
		}
		tc.writeFrames(packetType1RTT, debugFrameMaxStreams{
			streamType: styp,
			max:        1,
		})
		if _, err := opening.result(); err != nil {
			t.Fatalf("new stream not created after limit raised: %v", err)
		}
		if _, err := tc.conn.newLocalStream(ctx, styp); err == nil {
			t.Fatalf("new stream blocked by raised limit: %v, want error", err)
		}
	})
}

func TestStreamLimitMaxStreamsDecreases(t *testing.T) {
	// "MAX_STREAMS frames that do not increase the stream limit MUST be ignored."
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.6-4
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		ctx := canceledContext()
		tc := newTestConn(t, clientSide,
			permissiveTransportParameters,
			func(p *transportParameters) {
				p.initialMaxStreamsBidi = 0
				p.initialMaxStreamsUni = 0
			})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		tc.writeFrames(packetType1RTT, debugFrameMaxStreams{
			streamType: styp,
			max:        2,
		})
		tc.writeFrames(packetType1RTT, debugFrameMaxStreams{
			streamType: styp,
			max:        1,
		})
		if _, err := tc.conn.newLocalStream(ctx, styp); err != nil {
			t.Fatalf("open stream 1, limit 2, got error: %v", err)
		}
		if _, err := tc.conn.newLocalStream(ctx, styp); err != nil {
			t.Fatalf("open stream 2, limit 2, got error: %v", err)
		}
		if _, err := tc.conn.newLocalStream(ctx, styp); err == nil {
			t.Fatalf("open stream 3, limit 2, got error: %v", err)
		}
	})
}

func TestStreamLimitViolated(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide,
			func(c *Config) {
				if styp == bidiStream {
					c.MaxBidiRemoteStreams = 10
				} else {
					c.MaxUniRemoteStreams = 10
				}
			})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id: newStreamID(clientSide, styp, 9),
		})
		tc.wantIdle("stream number 9 is within the limit")
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id: newStreamID(clientSide, styp, 10),
		})
		tc.wantFrame("stream number 10 is beyond the limit",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errStreamLimit,
			},
		)
	})
}

func TestStreamLimitImplicitStreams(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide,
			func(c *Config) {
				c.MaxBidiRemoteStreams = 1 << 60
				c.MaxUniRemoteStreams = 1 << 60
			})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		if got, want := tc.sentTransportParameters.initialMaxStreamsBidi, int64(implicitStreamLimit); got != want {
			t.Errorf("sent initial_max_streams_bidi = %v, want %v", got, want)
		}
		if got, want := tc.sentTransportParameters.initialMaxStreamsUni, int64(implicitStreamLimit); got != want {
			t.Errorf("sent initial_max_streams_uni = %v, want %v", got, want)
		}

		// Create stream 0.
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id: newStreamID(clientSide, styp, 0),
		})
		tc.wantIdle("max streams not increased enough to send a new frame")

		// Create streams [0, implicitStreamLimit).
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id: newStreamID(clientSide, styp, implicitStreamLimit-1),
		})
		tc.wantFrame("max streams increases to implicit stream limit",
			packetType1RTT, debugFrameMaxStreams{
				streamType: styp,
				max:        2 * implicitStreamLimit,
			})

		// Create a stream past the limit.
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id: newStreamID(clientSide, styp, 2*implicitStreamLimit),
		})
		tc.wantFrame("stream is past the limit",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errStreamLimit,
			},
		)
	})
}

func TestStreamLimitMaxStreamsTransportParameterTooLarge(t *testing.T) {
	// "If a max_streams transport parameter [...] is received with
	// a value greater than 2^60 [...] the connection MUST be closed
	// immediately with a connection error of type TRANSPORT_PARAMETER_ERROR [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.6-2
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide,
			func(p *transportParameters) {
				if styp == bidiStream {
					p.initialMaxStreamsBidi = 1<<60 + 1
				} else {
					p.initialMaxStreamsUni = 1<<60 + 1
				}
			})
		tc.writeFrames(packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
		tc.wantFrame("max streams transport parameter is too large",
			packetTypeInitial, debugFrameConnectionCloseTransport{
				code: errTransportParameter,
			},
		)
	})
}

func TestStreamLimitMaxStreamsFrameTooLarge(t *testing.T) {
	// "If [...] a MAX_STREAMS frame is received with a value
	// greater than 2^60 [...] the connection MUST be closed immediately
	// with a connection error [...] of type FRAME_ENCODING_ERROR [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-4.6-2
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide)
		tc.handshake()
		tc.writeFrames(packetTypeInitial,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
			})
		tc.writeFrames(packetType1RTT, debugFrameMaxStreams{
			streamType: styp,
			max:        1<<60 + 1,
		})
		tc.wantFrame("MAX_STREAMS value is too large",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errFrameEncoding,
			},
		)
	})
}

func TestStreamLimitSendUpdatesMaxStreams(t *testing.T) {
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		tc := newTestConn(t, serverSide, func(c *Config) {
			if styp == uniStream {
				c.MaxUniRemoteStreams = 4
				c.MaxBidiRemoteStreams = 0
			} else {
				c.MaxUniRemoteStreams = 0
				c.MaxBidiRemoteStreams = 4
			}
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		var streams []*Stream
		for i := 0; i < 4; i++ {
			tc.writeFrames(packetType1RTT, debugFrameStream{
				id:  newStreamID(clientSide, styp, int64(i)),
				fin: true,
			})
			streams = append(streams, tc.acceptStream())
		}
		streams[3].Close()
		if styp == bidiStream {
			tc.wantFrame("stream is closed",
				packetType1RTT, debugFrameStream{
					id:   streams[3].id,
					fin:  true,
					data: []byte{},
				})
			tc.writeAckForAll()
		}
		tc.wantFrame("closing a stream when peer is at limit immediately extends the limit",
			packetType1RTT, debugFrameMaxStreams{
				streamType: styp,
				max:        5,
			})
	})
}

func TestStreamLimitStopSendingDoesNotUpdateMaxStreams(t *testing.T) {
	tc, s := newTestConnAndRemoteStream(t, serverSide, bidiStream, func(c *Config) {
		c.MaxBidiRemoteStreams = 1
	})
	tc.writeFrames(packetType1RTT, debugFrameStream{
		id:  s.id,
		fin: true,
	})
	s.CloseRead()
	tc.writeFrames(packetType1RTT, debugFrameStopSending{
		id: s.id,
	})
	tc.wantFrame("received STOP_SENDING, send RESET_STREAM",
		packetType1RTT, debugFrameResetStream{
			id: s.id,
		})
	tc.writeAckForAll()
	tc.wantIdle("MAX_STREAMS is not extended until the user fully closes the stream")
	s.CloseWrite()
	tc.wantFrame("user closing the stream triggers MAX_STREAMS update",
		packetType1RTT, debugFrameMaxStreams{
			streamType: bidiStream,
			max:        2,
		})
}
