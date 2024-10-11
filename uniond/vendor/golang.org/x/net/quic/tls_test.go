// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"crypto/tls"
	"crypto/x509"
	"errors"
	"testing"
	"time"
)

// handshake executes the handshake.
func (tc *testConn) handshake() {
	tc.t.Helper()
	if *testVV {
		*testVV = false
		defer func() {
			tc.t.Helper()
			*testVV = true
			tc.t.Logf("performed connection handshake")
		}()
	}
	defer func(saved map[byte]bool) {
		tc.ignoreFrames = saved
	}(tc.ignoreFrames)
	tc.ignoreFrames = nil
	t := tc.t
	dgrams := handshakeDatagrams(tc)
	i := 0
	for {
		if i == len(dgrams)-1 {
			if tc.conn.side == clientSide {
				want := tc.endpoint.now.Add(maxAckDelay - timerGranularity)
				if !tc.timer.Equal(want) {
					t.Fatalf("want timer = %v (max_ack_delay), got %v", want, tc.timer)
				}
				if got := tc.readDatagram(); got != nil {
					t.Fatalf("client unexpectedly sent: %v", got)
				}
			}
			tc.advance(maxAckDelay)
		}

		// Check that we're sending exactly the data we expect.
		// Any variation from the norm here should be intentional.
		got := tc.readDatagram()
		var want *testDatagram
		if !(tc.conn.side == serverSide && i == 0) && i < len(dgrams) {
			want = dgrams[i]
			fillCryptoFrames(want, tc.cryptoDataOut)
			i++
		}
		if !datagramEqual(got, want) {
			t.Fatalf("dgram %v:\ngot %v\n\nwant %v", i, got, want)
		}
		if i >= len(dgrams) {
			break
		}

		fillCryptoFrames(dgrams[i], tc.cryptoDataIn)
		tc.write(dgrams[i])
		i++
	}
}

func handshakeDatagrams(tc *testConn) (dgrams []*testDatagram) {
	var (
		clientConnIDs    [][]byte
		serverConnIDs    [][]byte
		clientResetToken statelessResetToken
		serverResetToken statelessResetToken
		transientConnID  []byte
	)
	localConnIDs := [][]byte{
		testLocalConnID(0),
		testLocalConnID(1),
	}
	peerConnIDs := [][]byte{
		testPeerConnID(0),
		testPeerConnID(1),
	}
	localResetToken := tc.endpoint.e.resetGen.tokenForConnID(localConnIDs[1])
	peerResetToken := testPeerStatelessResetToken(1)
	if tc.conn.side == clientSide {
		clientConnIDs = localConnIDs
		serverConnIDs = peerConnIDs
		clientResetToken = localResetToken
		serverResetToken = peerResetToken
		transientConnID = testLocalConnID(-1)
	} else {
		clientConnIDs = peerConnIDs
		serverConnIDs = localConnIDs
		clientResetToken = peerResetToken
		serverResetToken = localResetToken
		transientConnID = testPeerConnID(-1)
	}
	return []*testDatagram{{
		// Client Initial
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       0,
			version:   quicVersion1,
			srcConnID: clientConnIDs[0],
			dstConnID: transientConnID,
			frames: []debugFrame{
				debugFrameCrypto{},
			},
		}},
		paddedSize: 1200,
	}, {
		// Server Initial + Handshake + 1-RTT
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       0,
			version:   quicVersion1,
			srcConnID: serverConnIDs[0],
			dstConnID: clientConnIDs[0],
			frames: []debugFrame{
				debugFrameAck{
					ranges: []i64range[packetNumber]{{0, 1}},
				},
				debugFrameCrypto{},
			},
		}, {
			ptype:     packetTypeHandshake,
			num:       0,
			version:   quicVersion1,
			srcConnID: serverConnIDs[0],
			dstConnID: clientConnIDs[0],
			frames: []debugFrame{
				debugFrameCrypto{},
			},
		}, {
			ptype:     packetType1RTT,
			num:       0,
			dstConnID: clientConnIDs[0],
			frames: []debugFrame{
				debugFrameNewConnectionID{
					seq:    1,
					connID: serverConnIDs[1],
					token:  serverResetToken,
				},
			},
		}},
		paddedSize: 1200,
	}, {
		// Client Initial + Handshake + 1-RTT
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       1,
			version:   quicVersion1,
			srcConnID: clientConnIDs[0],
			dstConnID: serverConnIDs[0],
			frames: []debugFrame{
				debugFrameAck{
					ranges: []i64range[packetNumber]{{0, 1}},
				},
			},
		}, {
			ptype:     packetTypeHandshake,
			num:       0,
			version:   quicVersion1,
			srcConnID: clientConnIDs[0],
			dstConnID: serverConnIDs[0],
			frames: []debugFrame{
				debugFrameAck{
					ranges: []i64range[packetNumber]{{0, 1}},
				},
				debugFrameCrypto{},
			},
		}, {
			ptype:     packetType1RTT,
			num:       0,
			dstConnID: serverConnIDs[0],
			frames: []debugFrame{
				debugFrameAck{
					ranges: []i64range[packetNumber]{{0, 1}},
				},
				debugFrameNewConnectionID{
					seq:    1,
					connID: clientConnIDs[1],
					token:  clientResetToken,
				},
			},
		}},
		paddedSize: 1200,
	}, {
		// Server HANDSHAKE_DONE
		packets: []*testPacket{{
			ptype:     packetType1RTT,
			num:       1,
			dstConnID: clientConnIDs[0],
			frames: []debugFrame{
				debugFrameAck{
					ranges: []i64range[packetNumber]{{0, 1}},
				},
				debugFrameHandshakeDone{},
			},
		}},
	}, {
		// Client ack (after max_ack_delay)
		packets: []*testPacket{{
			ptype:     packetType1RTT,
			num:       1,
			dstConnID: serverConnIDs[0],
			frames: []debugFrame{
				debugFrameAck{
					ackDelay: unscaledAckDelayFromDuration(
						maxAckDelay, ackDelayExponent),
					ranges: []i64range[packetNumber]{{0, 2}},
				},
			},
		}},
	}}
}

func fillCryptoFrames(d *testDatagram, data map[tls.QUICEncryptionLevel][]byte) {
	for _, p := range d.packets {
		var level tls.QUICEncryptionLevel
		switch p.ptype {
		case packetTypeInitial:
			level = tls.QUICEncryptionLevelInitial
		case packetTypeHandshake:
			level = tls.QUICEncryptionLevelHandshake
		case packetType1RTT:
			level = tls.QUICEncryptionLevelApplication
		default:
			continue
		}
		for i := range p.frames {
			c, ok := p.frames[i].(debugFrameCrypto)
			if !ok {
				continue
			}
			c.data = data[level]
			data[level] = nil
			p.frames[i] = c
		}
	}
}

// uncheckedHandshake executes the handshake.
//
// Unlike testConn.handshake, it sends nothing unnecessary
// (in particular, no NEW_CONNECTION_ID frames),
// and does not validate the conn's responses.
//
// Useful for testing scenarios where configuration has
// changed the handshake responses in some way.
func (tc *testConn) uncheckedHandshake() {
	tc.t.Helper()
	defer func(saved map[byte]bool) {
		tc.ignoreFrames = saved
	}(tc.ignoreFrames)
	tc.ignoreFrames = map[byte]bool{
		frameTypeAck:             true,
		frameTypeCrypto:          true,
		frameTypeNewConnectionID: true,
	}
	if tc.conn.side == serverSide {
		tc.writeFrames(packetTypeInitial,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
			})
		tc.writeFrames(packetTypeHandshake,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
			})
		tc.wantFrame("send HANDSHAKE_DONE after handshake completes",
			packetType1RTT, debugFrameHandshakeDone{})
		tc.writeFrames(packetType1RTT,
			debugFrameAck{
				ackDelay: unscaledAckDelayFromDuration(
					maxAckDelay, ackDelayExponent),
				ranges: []i64range[packetNumber]{{0, tc.lastPacket.num + 1}},
			})
	} else {
		tc.wantIdle("initial frames are ignored")
		tc.writeFrames(packetTypeInitial,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
			})
		tc.writeFrames(packetTypeHandshake,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
			})
		tc.wantIdle("don't expect any frames we aren't ignoring")
		// Send the next two frames in separate packets, so the client sends an
		// ack immediately without delay. We want to consume that ack here, rather
		// than returning with a delayed ack waiting to be sent.
		tc.ignoreFrames = nil
		tc.writeFrames(packetType1RTT,
			debugFrameHandshakeDone{})
		tc.writeFrames(packetType1RTT,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelApplication],
			})
		tc.wantFrame("client ACKs server's first 1-RTT packet",
			packetType1RTT, debugFrameAck{
				ranges: []i64range[packetNumber]{{0, 2}},
			})

	}
	tc.wantIdle("handshake is done")
}

func TestConnClientHandshake(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.advance(1 * time.Second)
	tc.wantIdle("no packets should be sent by an idle conn after the handshake")
}

func TestConnServerHandshake(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.advance(1 * time.Second)
	tc.wantIdle("no packets should be sent by an idle conn after the handshake")
}

func TestConnKeysDiscardedClient(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)

	tc.wantFrame("client sends Initial CRYPTO frame",
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
	tc.wantFrame("client provides an additional connection ID",
		packetType1RTT, debugFrameNewConnectionID{
			seq:    1,
			connID: testLocalConnID(1),
			token:  testLocalStatelessResetToken(1),
		})

	// The client discards Initial keys after sending a Handshake packet.
	tc.writeFrames(packetTypeInitial,
		debugFrameConnectionCloseTransport{code: errInternal})
	tc.wantIdle("client has discarded Initial keys, cannot read CONNECTION_CLOSE")

	// The client discards Handshake keys after receiving a HANDSHAKE_DONE frame.
	tc.writeFrames(packetType1RTT,
		debugFrameHandshakeDone{})
	tc.writeFrames(packetTypeHandshake,
		debugFrameConnectionCloseTransport{code: errInternal})
	tc.wantIdle("client has discarded Handshake keys, cannot read CONNECTION_CLOSE")

	tc.writeFrames(packetType1RTT,
		debugFrameConnectionCloseTransport{code: errInternal})
	tc.conn.Abort(nil)
	tc.wantFrame("client closes connection after 1-RTT CONNECTION_CLOSE",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})
}

func TestConnKeysDiscardedServer(t *testing.T) {
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

	// The server discards Initial keys after receiving a Handshake packet.
	// The Handshake packet contains only the start of the client's CRYPTO flight here,
	// to avoids completing the handshake yet.
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][:1],
		})
	tc.writeFrames(packetTypeInitial,
		debugFrameConnectionCloseTransport{code: errInternal})
	tc.wantFrame("server provides an additional connection ID",
		packetType1RTT, debugFrameNewConnectionID{
			seq:    1,
			connID: testLocalConnID(1),
			token:  testLocalStatelessResetToken(1),
		})
	tc.wantIdle("server has discarded Initial keys, cannot read CONNECTION_CLOSE")

	// The server discards Handshake keys after sending a HANDSHAKE_DONE frame.
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			off:  1,
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][1:],
		})
	tc.wantFrame("server sends HANDSHAKE_DONE after handshake completes",
		packetType1RTT, debugFrameHandshakeDone{})
	tc.writeFrames(packetTypeHandshake,
		debugFrameConnectionCloseTransport{code: errInternal})
	tc.wantIdle("server has discarded Handshake keys, cannot read CONNECTION_CLOSE")

	tc.writeFrames(packetType1RTT,
		debugFrameConnectionCloseTransport{code: errInternal})
	tc.conn.Abort(nil)
	tc.wantFrame("server closes connection after 1-RTT CONNECTION_CLOSE",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})
}

func TestConnInvalidCryptoData(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)

	tc.wantFrame("client sends Initial CRYPTO frame",
		packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})

	// Render the server's response invalid.
	//
	// The client closes the connection with CRYPTO_ERROR.
	//
	// Changing the first byte will change the TLS message type,
	// so we can reasonably assume that this is an unexpected_message alert (10).
	tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][0] ^= 0x1
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
		})
	tc.wantFrame("client closes connection due to TLS handshake error",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errTLSBase + 10,
		})
}

func TestConnInvalidPeerCertificate(t *testing.T) {
	tc := newTestConn(t, clientSide, func(c *tls.Config) {
		c.VerifyPeerCertificate = func([][]byte, [][]*x509.Certificate) error {
			return errors.New("I will not buy this certificate. It is scratched.")
		}
	})
	tc.ignoreFrame(frameTypeAck)

	tc.wantFrame("client sends Initial CRYPTO frame",
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
	tc.wantFrame("client closes connection due to rejecting server certificate",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errTLSBase + 42, // 42: bad_certificate
		})
}

func TestConnHandshakeDoneSentToServer(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()

	tc.writeFrames(packetType1RTT,
		debugFrameHandshakeDone{})
	tc.wantFrame("server closes connection when client sends a HANDSHAKE_DONE frame",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errProtocolViolation,
		})
}

func TestConnCryptoDataOutOfOrder(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)

	tc.wantFrame("client sends Initial CRYPTO frame",
		packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.wantIdle("client is idle, server Handshake flight has not arrived")

	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			off:  15,
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][15:],
		})
	tc.wantIdle("client is idle, server Handshake flight is not complete")

	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			off:  1,
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][1:20],
		})
	tc.wantIdle("client is idle, server Handshake flight is still not complete")

	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][0:1],
		})
	tc.wantFrame("client sends Handshake CRYPTO frame",
		packetTypeHandshake, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelHandshake],
		})
}

func TestConnCryptoBufferSizeExceeded(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)

	tc.wantFrame("client sends Initial CRYPTO frame",
		packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			off:  cryptoBufferSize,
			data: []byte{0},
		})
	tc.wantFrame("client closes connection after server exceeds CRYPTO buffer",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errCryptoBufferExceeded,
		})
}

func TestConnAEADLimitReached(t *testing.T) {
	// "[...] endpoints MUST count the number of received packets that
	// fail authentication during the lifetime of a connection.
	// If the total number of received packets that fail authentication [...]
	// exceeds the integrity limit for the selected AEAD,
	// the endpoint MUST immediately close the connection [...]"
	// https://www.rfc-editor.org/rfc/rfc9001#section-6.6-6
	tc := newTestConn(t, clientSide, func(c *Config) {
		clear(c.StatelessResetKey[:])
	})
	tc.handshake()

	var limit int64
	switch suite := tc.conn.keysAppData.r.suite; suite {
	case tls.TLS_AES_128_GCM_SHA256, tls.TLS_AES_256_GCM_SHA384:
		limit = 1 << 52
	case tls.TLS_CHACHA20_POLY1305_SHA256:
		limit = 1 << 36
	default:
		t.Fatalf("conn.keysAppData.r.suite = %v, unknown suite", suite)
	}

	dstConnID := tc.conn.connIDState.local[0].cid
	if tc.conn.connIDState.local[0].seq == -1 {
		// Only use the transient connection ID in Initial packets.
		dstConnID = tc.conn.connIDState.local[1].cid
	}
	invalid := encodeTestPacket(t, tc, &testPacket{
		ptype:     packetType1RTT,
		num:       1000,
		frames:    []debugFrame{debugFramePing{}},
		version:   quicVersion1,
		dstConnID: dstConnID,
		srcConnID: tc.peerConnID,
	}, 0)
	invalid[len(invalid)-1] ^= 1
	sendInvalid := func() {
		t.Logf("<- conn under test receives invalid datagram")
		tc.conn.sendMsg(&datagram{
			b: invalid,
		})
		tc.wait()
	}

	// Set the conn's auth failure count to just before the AEAD integrity limit.
	tc.conn.keysAppData.authFailures = limit - 1

	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.advanceToTimer()
	tc.wantFrameType("auth failures less than limit: conn ACKs packet",
		packetType1RTT, debugFrameAck{})

	sendInvalid()
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.advanceToTimer()
	tc.wantFrameType("auth failures at limit: conn closes",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errAEADLimitReached,
		})

	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.advance(1 * time.Second)
	tc.wantIdle("auth failures at limit: conn does not process additional packets")
}
