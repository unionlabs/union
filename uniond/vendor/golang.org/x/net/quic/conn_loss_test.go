// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"crypto/tls"
	"fmt"
	"testing"
)

// Frames may be retransmitted either when the packet containing the frame is lost, or on PTO.
// lostFrameTest runs a test in both configurations.
func lostFrameTest(t *testing.T, f func(t *testing.T, pto bool)) {
	t.Run("lost", func(t *testing.T) {
		f(t, false)
	})
	t.Run("pto", func(t *testing.T) {
		f(t, true)
	})
}

// triggerLossOrPTO causes the conn to declare the last sent packet lost,
// or advances to the PTO timer.
func (tc *testConn) triggerLossOrPTO(ptype packetType, pto bool) {
	tc.t.Helper()
	if pto {
		if !tc.conn.loss.ptoTimerArmed {
			tc.t.Fatalf("PTO timer not armed, expected it to be")
		}
		if *testVV {
			tc.t.Logf("advancing to PTO timer")
		}
		tc.advanceTo(tc.conn.loss.timer)
		return
	}
	if *testVV {
		*testVV = false
		defer func() {
			tc.t.Logf("cause conn to declare last packet lost")
			*testVV = true
		}()
	}
	defer func(ignoreFrames map[byte]bool) {
		tc.ignoreFrames = ignoreFrames
	}(tc.ignoreFrames)
	tc.ignoreFrames = map[byte]bool{
		frameTypeAck:     true,
		frameTypePadding: true,
	}
	// Send three packets containing PINGs, and then respond with an ACK for the
	// last one. This puts the last packet before the PINGs outside the packet
	// reordering threshold, and it will be declared lost.
	const lossThreshold = 3
	var num packetNumber
	for i := 0; i < lossThreshold; i++ {
		tc.conn.ping(spaceForPacketType(ptype))
		d := tc.readDatagram()
		if d == nil {
			tc.t.Fatalf("conn is idle; want PING frame")
		}
		if d.packets[0].ptype != ptype {
			tc.t.Fatalf("conn sent %v packet; want %v", d.packets[0].ptype, ptype)
		}
		num = d.packets[0].num
	}
	tc.writeFrames(ptype, debugFrameAck{
		ranges: []i64range[packetNumber]{
			{num, num + 1},
		},
	})
}

func TestLostResetStreamFrame(t *testing.T) {
	// "Cancellation of stream transmission, as carried in a RESET_STREAM frame,
	// is sent until acknowledged or until all stream data is acknowledged by the peer [...]"
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-13.3-3.4
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, permissiveTransportParameters)
		tc.ignoreFrame(frameTypeAck)

		s.Reset(1)
		tc.wantFrame("reset stream",
			packetType1RTT, debugFrameResetStream{
				id:   s.id,
				code: 1,
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resent RESET_STREAM frame",
			packetType1RTT, debugFrameResetStream{
				id:   s.id,
				code: 1,
			})
	})
}

func TestLostStopSendingFrame(t *testing.T) {
	// "[...] a request to cancel stream transmission, as encoded in a STOP_SENDING frame,
	// is sent until the receiving part of the stream enters either a "Data Recvd" or
	// "Reset Recvd" state [...]"
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-13.3-3.5
	//
	// Technically, we can stop sending a STOP_SENDING frame if the peer sends
	// us all the data for the stream or resets it. We don't bother tracking this,
	// however, so we'll keep sending the frame until it is acked. This is harmless.
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, permissiveTransportParameters)
		tc.ignoreFrame(frameTypeAck)

		s.CloseRead()
		tc.wantFrame("stream is read-closed",
			packetType1RTT, debugFrameStopSending{
				id: s.id,
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resent STOP_SENDING frame",
			packetType1RTT, debugFrameStopSending{
				id: s.id,
			})
	})
}

func TestLostCryptoFrame(t *testing.T) {
	// "Data sent in CRYPTO frames is retransmitted [...] until all data has been acknowledged."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-13.3-3.1
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc := newTestConn(t, clientSide)
		tc.ignoreFrame(frameTypeAck)

		tc.wantFrame("client sends Initial CRYPTO frame",
			packetTypeInitial, debugFrameCrypto{
				data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
			})
		tc.triggerLossOrPTO(packetTypeInitial, pto)
		tc.wantFrame("client resends Initial CRYPTO frame",
			packetTypeInitial, debugFrameCrypto{
				data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
			})

		tc.writeFrames(packetTypeInitial,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
			})
		tc.writeFrames(packetTypeHandshake,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
			})

		tc.wantFrame("client sends Handshake CRYPTO frame",
			packetTypeHandshake, debugFrameCrypto{
				data: tc.cryptoDataOut[tls.QUICEncryptionLevelHandshake],
			})
		tc.wantFrame("client provides server with an additional connection ID",
			packetType1RTT, debugFrameNewConnectionID{
				seq:    1,
				connID: testLocalConnID(1),
				token:  testLocalStatelessResetToken(1),
			})
		tc.triggerLossOrPTO(packetTypeHandshake, pto)
		tc.wantFrame("client resends Handshake CRYPTO frame",
			packetTypeHandshake, debugFrameCrypto{
				data: tc.cryptoDataOut[tls.QUICEncryptionLevelHandshake],
			})
	})
}

func TestLostStreamFrameEmpty(t *testing.T) {
	// A STREAM frame opening a stream, but containing no stream data, should
	// be retransmitted if lost.
	lostFrameTest(t, func(t *testing.T, pto bool) {
		ctx := canceledContext()
		tc := newTestConn(t, clientSide, permissiveTransportParameters)
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)

		c, err := tc.conn.NewStream(ctx)
		if err != nil {
			t.Fatalf("NewStream: %v", err)
		}
		c.Flush() // open the stream
		tc.wantFrame("created bidirectional stream 0",
			packetType1RTT, debugFrameStream{
				id:   newStreamID(clientSide, bidiStream, 0),
				data: []byte{},
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resent stream frame",
			packetType1RTT, debugFrameStream{
				id:   newStreamID(clientSide, bidiStream, 0),
				data: []byte{},
			})
	})
}

func TestLostStreamWithData(t *testing.T) {
	// "Application data sent in STREAM frames is retransmitted in new STREAM
	// frames unless the endpoint has sent a RESET_STREAM for that stream."
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.2
	//
	// TODO: Lost stream frame after RESET_STREAM
	lostFrameTest(t, func(t *testing.T, pto bool) {
		data := []byte{0, 1, 2, 3, 4, 5, 6, 7}
		tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, func(p *transportParameters) {
			p.initialMaxStreamsUni = 1
			p.initialMaxData = 1 << 20
			p.initialMaxStreamDataUni = 1 << 20
		})
		s.Write(data[:4])
		s.Flush()
		tc.wantFrame("send [0,4)",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: data[:4],
			})
		s.Write(data[4:8])
		s.Flush()
		tc.wantFrame("send [4,8)",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  4,
				data: data[4:8],
			})
		s.CloseWrite()
		tc.wantFrame("send FIN",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  8,
				fin:  true,
				data: []byte{},
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resend data",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				fin:  true,
				data: data[:8],
			})
	})
}

func TestLostStreamPartialLoss(t *testing.T) {
	// Conn sends four STREAM packets.
	// ACKs are received for the packets containing bytes 0 and 2.
	// The remaining packets are declared lost.
	// The Conn resends only the lost data.
	//
	// This test doesn't have a PTO mode, because the ACK for the packet containing byte 2
	// starts the loss timer for the packet containing byte 1, and the PTO timer is not
	// armed when the loss timer is.
	data := []byte{0, 1, 2, 3}
	tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, func(p *transportParameters) {
		p.initialMaxStreamsUni = 1
		p.initialMaxData = 1 << 20
		p.initialMaxStreamDataUni = 1 << 20
	})
	for i := range data {
		s.Write(data[i : i+1])
		s.Flush()
		tc.wantFrame(fmt.Sprintf("send STREAM frame with byte %v", i),
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  int64(i),
				data: data[i : i+1],
			})
		if i%2 == 0 {
			tc.writeAckForLatest()
		}
	}
	const pto = false
	tc.triggerLossOrPTO(packetType1RTT, pto)
	tc.wantFrame("resend byte 1",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  1,
			data: data[1:2],
		})
	tc.wantFrame("resend byte 3",
		packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  3,
			data: data[3:4],
		})
	tc.wantIdle("no more frames sent after packet loss")
}

func TestLostMaxDataFrame(t *testing.T) {
	// "An updated value is sent in a MAX_DATA frame if the packet
	// containing the most recently sent MAX_DATA frame is declared lost [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.7
	lostFrameTest(t, func(t *testing.T, pto bool) {
		const maxWindowSize = 32
		buf := make([]byte, maxWindowSize)
		tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, func(c *Config) {
			c.MaxConnReadBufferSize = 32
		})

		// We send MAX_DATA = 63.
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  0,
			data: make([]byte, maxWindowSize-1),
		})
		if n, err := s.Read(buf[:maxWindowSize]); err != nil || n != maxWindowSize-1 {
			t.Fatalf("Read() = %v, %v; want %v, nil", n, err, maxWindowSize-1)
		}
		tc.wantFrame("conn window is extended after reading data",
			packetType1RTT, debugFrameMaxData{
				max: (maxWindowSize * 2) - 1,
			})

		// MAX_DATA = 64, which is only one more byte, so we don't send the frame.
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  maxWindowSize - 1,
			data: make([]byte, 1),
		})
		if n, err := s.Read(buf[:1]); err != nil || n != 1 {
			t.Fatalf("Read() = %v, %v; want %v, nil", n, err, 1)
		}
		tc.wantIdle("read doesn't extend window enough to send another MAX_DATA")

		// The MAX_DATA = 63 packet was lost, so we send 64.
		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resent MAX_DATA includes most current value",
			packetType1RTT, debugFrameMaxData{
				max: maxWindowSize * 2,
			})
	})
}

func TestLostMaxStreamDataFrame(t *testing.T) {
	// "[...] an updated value is sent when the packet containing
	// the most recent MAX_STREAM_DATA frame for a stream is lost"
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.8
	lostFrameTest(t, func(t *testing.T, pto bool) {
		const maxWindowSize = 32
		buf := make([]byte, maxWindowSize)
		tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, func(c *Config) {
			c.MaxStreamReadBufferSize = maxWindowSize
		})

		// We send MAX_STREAM_DATA = 63.
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  0,
			data: make([]byte, maxWindowSize-1),
		})
		if n, err := s.Read(buf[:maxWindowSize]); err != nil || n != maxWindowSize-1 {
			t.Fatalf("Read() = %v, %v; want %v, nil", n, err, maxWindowSize-1)
		}
		tc.wantFrame("stream window is extended after reading data",
			packetType1RTT, debugFrameMaxStreamData{
				id:  s.id,
				max: (maxWindowSize * 2) - 1,
			})

		// MAX_STREAM_DATA = 64, which is only one more byte, so we don't send the frame.
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  maxWindowSize - 1,
			data: make([]byte, 1),
		})
		if n, err := s.Read(buf); err != nil || n != 1 {
			t.Fatalf("Read() = %v, %v; want %v, nil", n, err, 1)
		}
		tc.wantIdle("read doesn't extend window enough to send another MAX_STREAM_DATA")

		// The MAX_STREAM_DATA = 63 packet was lost, so we send 64.
		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resent MAX_STREAM_DATA includes most current value",
			packetType1RTT, debugFrameMaxStreamData{
				id:  s.id,
				max: maxWindowSize * 2,
			})
	})
}

func TestLostMaxStreamDataFrameAfterStreamFinReceived(t *testing.T) {
	// "An endpoint SHOULD stop sending MAX_STREAM_DATA frames when
	// the receiving part of the stream enters a "Size Known" or "Reset Recvd" state."
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.8
	lostFrameTest(t, func(t *testing.T, pto bool) {
		const maxWindowSize = 10
		buf := make([]byte, maxWindowSize)
		tc, s := newTestConnAndRemoteStream(t, serverSide, uniStream, func(c *Config) {
			c.MaxStreamReadBufferSize = maxWindowSize
		})

		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:   s.id,
			off:  0,
			data: make([]byte, maxWindowSize),
		})
		if n, err := s.Read(buf); err != nil || n != maxWindowSize {
			t.Fatalf("Read() = %v, %v; want %v, nil", n, err, maxWindowSize)
		}
		tc.wantFrame("stream window is extended after reading data",
			packetType1RTT, debugFrameMaxStreamData{
				id:  s.id,
				max: 2 * maxWindowSize,
			})

		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:  s.id,
			off: maxWindowSize,
			fin: true,
		})

		tc.ignoreFrame(frameTypePing)
		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantIdle("lost MAX_STREAM_DATA not resent for stream in 'size known'")
	})
}

func TestLostMaxStreamsFrameMostRecent(t *testing.T) {
	// "[...] an updated value is sent when a packet containing the
	// most recent MAX_STREAMS for a stream type frame is declared lost [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.9
	testStreamTypes(t, "", func(t *testing.T, styp streamType) {
		lostFrameTest(t, func(t *testing.T, pto bool) {
			ctx := canceledContext()
			tc := newTestConn(t, serverSide, func(c *Config) {
				c.MaxUniRemoteStreams = 1
				c.MaxBidiRemoteStreams = 1
			})
			tc.handshake()
			tc.ignoreFrame(frameTypeAck)
			tc.writeFrames(packetType1RTT, debugFrameStream{
				id:  newStreamID(clientSide, styp, 0),
				fin: true,
			})
			s, err := tc.conn.AcceptStream(ctx)
			if err != nil {
				t.Fatalf("AcceptStream() = %v", err)
			}
			s.SetWriteContext(ctx)
			s.Close()
			if styp == bidiStream {
				tc.wantFrame("stream is closed",
					packetType1RTT, debugFrameStream{
						id:   s.id,
						data: []byte{},
						fin:  true,
					})
				tc.writeAckForAll()
			}
			tc.wantFrame("closing stream updates peer's MAX_STREAMS",
				packetType1RTT, debugFrameMaxStreams{
					streamType: styp,
					max:        2,
				})

			tc.triggerLossOrPTO(packetType1RTT, pto)
			tc.wantFrame("lost MAX_STREAMS is resent",
				packetType1RTT, debugFrameMaxStreams{
					streamType: styp,
					max:        2,
				})
		})
	})
}

func TestLostMaxStreamsFrameNotMostRecent(t *testing.T) {
	// Send two MAX_STREAMS frames, lose the first one.
	//
	// No PTO mode for this test: The ack that causes the first frame
	// to be lost arms the loss timer for the second, so the PTO timer is not armed.
	const pto = false
	ctx := canceledContext()
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.MaxUniRemoteStreams = 2
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	for i := int64(0); i < 2; i++ {
		tc.writeFrames(packetType1RTT, debugFrameStream{
			id:  newStreamID(clientSide, uniStream, i),
			fin: true,
		})
		s, err := tc.conn.AcceptStream(ctx)
		if err != nil {
			t.Fatalf("AcceptStream() = %v", err)
		}
		if err := s.Close(); err != nil {
			t.Fatalf("stream.Close() = %v", err)
		}
		tc.wantFrame("closing stream updates peer's MAX_STREAMS",
			packetType1RTT, debugFrameMaxStreams{
				streamType: uniStream,
				max:        3 + i,
			})
	}

	// The second MAX_STREAMS frame is acked.
	tc.writeAckForLatest()

	// The first MAX_STREAMS frame is lost.
	tc.conn.ping(appDataSpace)
	tc.wantFrame("connection should send a PING frame",
		packetType1RTT, debugFramePing{})
	tc.triggerLossOrPTO(packetType1RTT, pto)
	tc.wantIdle("superseded MAX_DATA is not resent on loss")
}

func TestLostStreamDataBlockedFrame(t *testing.T) {
	// "A new [STREAM_DATA_BLOCKED] frame is sent if a packet containing
	// the most recent frame for a scope is lost [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.10
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, func(p *transportParameters) {
			p.initialMaxStreamsUni = 1
			p.initialMaxData = 1 << 20
		})

		w := runAsync(tc, func(ctx context.Context) (int, error) {
			return s.Write([]byte{0, 1, 2, 3})
		})
		defer w.cancel()
		tc.wantFrame("write is blocked by flow control",
			packetType1RTT, debugFrameStreamDataBlocked{
				id:  s.id,
				max: 0,
			})

		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: 1,
		})
		tc.wantFrame("write makes some progress, but is still blocked by flow control",
			packetType1RTT, debugFrameStreamDataBlocked{
				id:  s.id,
				max: 1,
			})
		tc.wantFrame("write consuming available window",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: []byte{0},
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("STREAM_DATA_BLOCKED is resent",
			packetType1RTT, debugFrameStreamDataBlocked{
				id:  s.id,
				max: 1,
			})
		tc.wantFrame("STREAM is resent as well",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: []byte{0},
			})
	})
}

func TestLostStreamDataBlockedFrameAfterStreamUnblocked(t *testing.T) {
	// "A new [STREAM_DATA_BLOCKED] frame is sent [...] only while
	// the endpoint is blocked on the corresponding limit."
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.10
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc, s := newTestConnAndLocalStream(t, serverSide, uniStream, func(p *transportParameters) {
			p.initialMaxStreamsUni = 1
			p.initialMaxData = 1 << 20
		})

		data := []byte{0, 1, 2, 3}
		w := runAsync(tc, func(ctx context.Context) (int, error) {
			return s.Write(data)
		})
		defer w.cancel()
		tc.wantFrame("write is blocked by flow control",
			packetType1RTT, debugFrameStreamDataBlocked{
				id:  s.id,
				max: 0,
			})

		tc.writeFrames(packetType1RTT, debugFrameMaxStreamData{
			id:  s.id,
			max: 10,
		})
		tc.wantFrame("write completes after flow control available",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: data,
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("STREAM data is resent",
			packetType1RTT, debugFrameStream{
				id:   s.id,
				off:  0,
				data: data,
			})
		tc.wantIdle("STREAM_DATA_BLOCKED is not resent, since the stream is not blocked")
	})
}

func TestLostNewConnectionIDFrame(t *testing.T) {
	// "New connection IDs are [...] retransmitted if the packet containing them is lost."
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.13
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc := newTestConn(t, serverSide)
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)

		tc.writeFrames(packetType1RTT,
			debugFrameRetireConnectionID{
				seq: 1,
			})
		tc.wantFrame("provide a new connection ID after peer retires old one",
			packetType1RTT, debugFrameNewConnectionID{
				seq:    2,
				connID: testLocalConnID(2),
				token:  testLocalStatelessResetToken(2),
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resend new connection ID",
			packetType1RTT, debugFrameNewConnectionID{
				seq:    2,
				connID: testLocalConnID(2),
				token:  testLocalStatelessResetToken(2),
			})
	})
}

func TestLostRetireConnectionIDFrame(t *testing.T) {
	// "[...] retired connection IDs are [...] retransmitted
	// if the packet containing them is lost."
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.3-3.13
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc := newTestConn(t, clientSide)
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)

		tc.writeFrames(packetType1RTT,
			debugFrameNewConnectionID{
				seq:           2,
				retirePriorTo: 1,
				connID:        testPeerConnID(2),
			})
		tc.wantFrame("peer requested connection id be retired",
			packetType1RTT, debugFrameRetireConnectionID{
				seq: 0,
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("resend RETIRE_CONNECTION_ID",
			packetType1RTT, debugFrameRetireConnectionID{
				seq: 0,
			})
	})
}

func TestLostPathResponseFrame(t *testing.T) {
	// "Responses to path validation using PATH_RESPONSE frames are sent just once."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-13.3-3.12
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc := newTestConn(t, clientSide)
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		tc.ignoreFrame(frameTypePing)

		data := pathChallengeData{0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef}
		tc.writeFrames(packetType1RTT, debugFramePathChallenge{
			data: data,
		})
		tc.wantFrame("response to PATH_CHALLENGE",
			packetType1RTT, debugFramePathResponse{
				data: data,
			})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantIdle("lost PATH_RESPONSE frame is not retransmitted")
	})
}

func TestLostHandshakeDoneFrame(t *testing.T) {
	// "The HANDSHAKE_DONE frame MUST be retransmitted until it is acknowledged."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-13.3-3.16
	lostFrameTest(t, func(t *testing.T, pto bool) {
		tc := newTestConn(t, serverSide)
		tc.ignoreFrame(frameTypeAck)

		tc.writeFrames(packetTypeInitial,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
			})
		tc.wantFrame("server sends Initial CRYPTO frame",
			packetTypeInitial, debugFrameCrypto{
				data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
			})
		tc.wantFrame("server sends Handshake CRYPTO frame",
			packetTypeHandshake, debugFrameCrypto{
				data: tc.cryptoDataOut[tls.QUICEncryptionLevelHandshake],
			})
		tc.wantFrame("server provides an additional connection ID",
			packetType1RTT, debugFrameNewConnectionID{
				seq:    1,
				connID: testLocalConnID(1),
				token:  testLocalStatelessResetToken(1),
			})
		tc.writeFrames(packetTypeHandshake,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
			})

		tc.wantFrame("server sends HANDSHAKE_DONE after handshake completes",
			packetType1RTT, debugFrameHandshakeDone{})

		tc.triggerLossOrPTO(packetType1RTT, pto)
		tc.wantFrame("server resends HANDSHAKE_DONE",
			packetType1RTT, debugFrameHandshakeDone{})
	})
}
