// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"context"
	"crypto/tls"
	"net/netip"
	"testing"
	"time"
)

type retryServerTest struct {
	te                *testEndpoint
	originalSrcConnID []byte
	originalDstConnID []byte
	retry             retryPacket
	initialCrypto     []byte
}

// newRetryServerTest creates a test server connection,
// sends the connection an Initial packet,
// and expects a Retry in response.
func newRetryServerTest(t *testing.T) *retryServerTest {
	t.Helper()
	config := &Config{
		TLSConfig:                newTestTLSConfig(serverSide),
		RequireAddressValidation: true,
	}
	te := newTestEndpoint(t, config)
	srcID := testPeerConnID(0)
	dstID := testLocalConnID(-1)
	params := defaultTransportParameters()
	params.initialSrcConnID = srcID
	initialCrypto := initialClientCrypto(t, te, params)

	// Initial packet with no Token.
	// Server responds with a Retry containing a token.
	te.writeDatagram(&testDatagram{
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       0,
			version:   quicVersion1,
			srcConnID: srcID,
			dstConnID: dstID,
			frames: []debugFrame{
				debugFrameCrypto{
					data: initialCrypto,
				},
			},
		}},
		paddedSize: 1200,
	})
	got := te.readDatagram()
	if len(got.packets) != 1 || got.packets[0].ptype != packetTypeRetry {
		t.Fatalf("got datagram: %v\nwant Retry", got)
	}
	p := got.packets[0]
	if got, want := p.dstConnID, srcID; !bytes.Equal(got, want) {
		t.Fatalf("Retry destination = {%x}, want {%x}", got, want)
	}

	return &retryServerTest{
		te:                te,
		originalSrcConnID: srcID,
		originalDstConnID: dstID,
		retry: retryPacket{
			dstConnID: p.dstConnID,
			srcConnID: p.srcConnID,
			token:     p.token,
		},
		initialCrypto: initialCrypto,
	}
}

func TestRetryServerSucceeds(t *testing.T) {
	rt := newRetryServerTest(t)
	te := rt.te
	te.advance(retryTokenValidityPeriod)
	te.writeDatagram(&testDatagram{
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       1,
			version:   quicVersion1,
			srcConnID: rt.originalSrcConnID,
			dstConnID: rt.retry.srcConnID,
			token:     rt.retry.token,
			frames: []debugFrame{
				debugFrameCrypto{
					data: rt.initialCrypto,
				},
			},
		}},
		paddedSize: 1200,
	})
	tc := te.accept()
	initial := tc.readPacket()
	if initial == nil || initial.ptype != packetTypeInitial {
		t.Fatalf("got packet:\n%v\nwant: Initial", initial)
	}
	handshake := tc.readPacket()
	if handshake == nil || handshake.ptype != packetTypeHandshake {
		t.Fatalf("got packet:\n%v\nwant: Handshake", initial)
	}
	if got, want := tc.sentTransportParameters.retrySrcConnID, rt.retry.srcConnID; !bytes.Equal(got, want) {
		t.Errorf("retry_source_connection_id = {%x}, want {%x}", got, want)
	}
	if got, want := tc.sentTransportParameters.initialSrcConnID, initial.srcConnID; !bytes.Equal(got, want) {
		t.Errorf("initial_source_connection_id = {%x}, want {%x}", got, want)
	}
	if got, want := tc.sentTransportParameters.originalDstConnID, rt.originalDstConnID; !bytes.Equal(got, want) {
		t.Errorf("original_destination_connection_id = {%x}, want {%x}", got, want)
	}
}

func TestRetryServerTokenInvalid(t *testing.T) {
	// "If a server receives a client Initial that contains an invalid Retry token [...]
	// the server SHOULD immediately close [...] the connection with an
	// INVALID_TOKEN error."
	// https://www.rfc-editor.org/rfc/rfc9000#section-8.1.2-5
	rt := newRetryServerTest(t)
	te := rt.te
	te.writeDatagram(&testDatagram{
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       1,
			version:   quicVersion1,
			srcConnID: rt.originalSrcConnID,
			dstConnID: rt.retry.srcConnID,
			token:     append(rt.retry.token, 0),
			frames: []debugFrame{
				debugFrameCrypto{
					data: rt.initialCrypto,
				},
			},
		}},
		paddedSize: 1200,
	})
	te.wantDatagram("server closes connection after Initial with invalid Retry token",
		initialConnectionCloseDatagram(
			rt.retry.srcConnID,
			rt.originalSrcConnID,
			errInvalidToken))
}

func TestRetryServerTokenTooOld(t *testing.T) {
	// "[...] a token SHOULD have an expiration time [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-8.1.3-3
	rt := newRetryServerTest(t)
	te := rt.te
	te.advance(retryTokenValidityPeriod + time.Second)
	te.writeDatagram(&testDatagram{
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       1,
			version:   quicVersion1,
			srcConnID: rt.originalSrcConnID,
			dstConnID: rt.retry.srcConnID,
			token:     rt.retry.token,
			frames: []debugFrame{
				debugFrameCrypto{
					data: rt.initialCrypto,
				},
			},
		}},
		paddedSize: 1200,
	})
	te.wantDatagram("server closes connection after Initial with expired token",
		initialConnectionCloseDatagram(
			rt.retry.srcConnID,
			rt.originalSrcConnID,
			errInvalidToken))
}

func TestRetryServerTokenWrongIP(t *testing.T) {
	// "Tokens sent in Retry packets SHOULD include information that allows the server
	// to verify that the source IP address and port in client packets remain constant."
	// https://www.rfc-editor.org/rfc/rfc9000#section-8.1.4-3
	rt := newRetryServerTest(t)
	te := rt.te
	te.writeDatagram(&testDatagram{
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       1,
			version:   quicVersion1,
			srcConnID: rt.originalSrcConnID,
			dstConnID: rt.retry.srcConnID,
			token:     rt.retry.token,
			frames: []debugFrame{
				debugFrameCrypto{
					data: rt.initialCrypto,
				},
			},
		}},
		paddedSize: 1200,
		addr:       netip.MustParseAddrPort("10.0.0.2:8000"),
	})
	te.wantDatagram("server closes connection after Initial from wrong address",
		initialConnectionCloseDatagram(
			rt.retry.srcConnID,
			rt.originalSrcConnID,
			errInvalidToken))
}

func TestRetryServerIgnoresRetry(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.write(&testDatagram{
		packets: []*testPacket{{
			ptype:             packetTypeRetry,
			originalDstConnID: testLocalConnID(-1),
			srcConnID:         testPeerConnID(0),
			dstConnID:         testLocalConnID(0),
			token:             []byte{1, 2, 3, 4},
		}},
	})
	// Send two packets, to trigger an immediate ACK.
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantFrameType("server connection ignores spurious Retry packet",
		packetType1RTT, debugFrameAck{})
}

func TestRetryClientSuccess(t *testing.T) {
	// "This token MUST be repeated by the client in all Initial packets it sends
	// for that connection after it receives the Retry packet."
	// https://www.rfc-editor.org/rfc/rfc9000#section-8.1.2-1
	tc := newTestConn(t, clientSide)
	tc.wantFrame("client Initial CRYPTO data",
		packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
		})
	newServerConnID := []byte("new_conn_id")
	token := []byte("token")
	tc.write(&testDatagram{
		packets: []*testPacket{{
			ptype:             packetTypeRetry,
			originalDstConnID: testLocalConnID(-1),
			srcConnID:         newServerConnID,
			dstConnID:         testLocalConnID(0),
			token:             token,
		}},
	})
	tc.wantPacket("client sends a new Initial packet with a token",
		&testPacket{
			ptype:     packetTypeInitial,
			num:       1,
			version:   quicVersion1,
			srcConnID: testLocalConnID(0),
			dstConnID: newServerConnID,
			token:     token,
			frames: []debugFrame{
				debugFrameCrypto{
					data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
				},
			},
		},
	)
	tc.advanceToTimer()
	tc.wantPacket("after PTO client sends another Initial packet with a token",
		&testPacket{
			ptype:     packetTypeInitial,
			num:       2,
			version:   quicVersion1,
			srcConnID: testLocalConnID(0),
			dstConnID: newServerConnID,
			token:     token,
			frames: []debugFrame{
				debugFrameCrypto{
					data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
				},
			},
		},
	)
}

func TestRetryClientInvalidServerTransportParameters(t *testing.T) {
	// Various permutations of missing or invalid values for transport parameters
	// after a Retry.
	// https://www.rfc-editor.org/rfc/rfc9000#section-7.3
	initialSrcConnID := testPeerConnID(0)
	originalDstConnID := testLocalConnID(-1)
	retrySrcConnID := testPeerConnID(100)
	for _, test := range []struct {
		name string
		f    func(*transportParameters)
		ok   bool
	}{{
		name: "valid",
		f:    func(p *transportParameters) {},
		ok:   true,
	}, {
		name: "missing initial_source_connection_id",
		f: func(p *transportParameters) {
			p.initialSrcConnID = nil
		},
	}, {
		name: "invalid initial_source_connection_id",
		f: func(p *transportParameters) {
			p.initialSrcConnID = []byte("invalid")
		},
	}, {
		name: "missing original_destination_connection_id",
		f: func(p *transportParameters) {
			p.originalDstConnID = nil
		},
	}, {
		name: "invalid original_destination_connection_id",
		f: func(p *transportParameters) {
			p.originalDstConnID = []byte("invalid")
		},
	}, {
		name: "missing retry_source_connection_id",
		f: func(p *transportParameters) {
			p.retrySrcConnID = nil
		},
	}, {
		name: "invalid retry_source_connection_id",
		f: func(p *transportParameters) {
			p.retrySrcConnID = []byte("invalid")
		},
	}} {
		t.Run(test.name, func(t *testing.T) {
			tc := newTestConn(t, clientSide,
				func(p *transportParameters) {
					p.initialSrcConnID = initialSrcConnID
					p.originalDstConnID = originalDstConnID
					p.retrySrcConnID = retrySrcConnID
				},
				test.f)
			tc.ignoreFrame(frameTypeAck)
			tc.wantFrameType("client Initial CRYPTO data",
				packetTypeInitial, debugFrameCrypto{})
			tc.write(&testDatagram{
				packets: []*testPacket{{
					ptype:             packetTypeRetry,
					originalDstConnID: originalDstConnID,
					srcConnID:         retrySrcConnID,
					dstConnID:         testLocalConnID(0),
					token:             []byte{1, 2, 3, 4},
				}},
			})
			tc.wantFrameType("client resends Initial CRYPTO data",
				packetTypeInitial, debugFrameCrypto{})
			tc.writeFrames(packetTypeInitial,
				debugFrameCrypto{
					data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
				})
			tc.writeFrames(packetTypeHandshake,
				debugFrameCrypto{
					data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
				})
			if test.ok {
				tc.wantFrameType("valid params, client sends Handshake",
					packetTypeHandshake, debugFrameCrypto{})
			} else {
				tc.wantFrame("invalid transport parameters",
					packetTypeInitial, debugFrameConnectionCloseTransport{
						code: errTransportParameter,
					})
			}
		})
	}
}

func TestRetryClientIgnoresRetryAfterReceivingPacket(t *testing.T) {
	// "After the client has received and processed an Initial or Retry packet
	// from the server, it MUST discard any subsequent Retry packets that it receives."
	// https://www.rfc-editor.org/rfc/rfc9000#section-17.2.5.2-1
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)
	tc.ignoreFrame(frameTypeNewConnectionID)
	tc.wantFrameType("client Initial CRYPTO data",
		packetTypeInitial, debugFrameCrypto{})
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	retry := &testDatagram{
		packets: []*testPacket{{
			ptype:             packetTypeRetry,
			originalDstConnID: testLocalConnID(-1),
			srcConnID:         testPeerConnID(100),
			dstConnID:         testLocalConnID(0),
			token:             []byte{1, 2, 3, 4},
		}},
	}
	tc.write(retry)
	tc.wantIdle("client ignores Retry after receiving Initial packet")
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
		})
	tc.wantFrameType("client Handshake CRYPTO data",
		packetTypeHandshake, debugFrameCrypto{})
	tc.write(retry)
	tc.wantIdle("client ignores Retry after discarding Initial keys")
}

func TestRetryClientIgnoresRetryAfterReceivingRetry(t *testing.T) {
	// "After the client has received and processed an Initial or Retry packet
	// from the server, it MUST discard any subsequent Retry packets that it receives."
	// https://www.rfc-editor.org/rfc/rfc9000#section-17.2.5.2-1
	tc := newTestConn(t, clientSide)
	tc.wantFrameType("client Initial CRYPTO data",
		packetTypeInitial, debugFrameCrypto{})
	retry := &testDatagram{
		packets: []*testPacket{{
			ptype:             packetTypeRetry,
			originalDstConnID: testLocalConnID(-1),
			srcConnID:         testPeerConnID(100),
			dstConnID:         testLocalConnID(0),
			token:             []byte{1, 2, 3, 4},
		}},
	}
	tc.write(retry)
	tc.wantFrameType("client resends Initial CRYPTO data",
		packetTypeInitial, debugFrameCrypto{})
	tc.write(retry)
	tc.wantIdle("client ignores second Retry")
}

func TestRetryClientIgnoresRetryWithInvalidIntegrityTag(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.wantFrameType("client Initial CRYPTO data",
		packetTypeInitial, debugFrameCrypto{})
	pkt := encodeRetryPacket(testLocalConnID(-1), retryPacket{
		srcConnID: testPeerConnID(100),
		dstConnID: testLocalConnID(0),
		token:     []byte{1, 2, 3, 4},
	})
	pkt[len(pkt)-1] ^= 1 // invalidate the integrity tag
	tc.endpoint.write(&datagram{
		b:        pkt,
		peerAddr: testClientAddr,
	})
	tc.wantIdle("client ignores Retry with invalid integrity tag")
}

func TestRetryClientIgnoresRetryWithZeroLengthToken(t *testing.T) {
	// "A client MUST discard a Retry packet with a zero-length Retry Token field."
	// https://www.rfc-editor.org/rfc/rfc9000#section-17.2.5.2-2
	tc := newTestConn(t, clientSide)
	tc.wantFrameType("client Initial CRYPTO data",
		packetTypeInitial, debugFrameCrypto{})
	tc.write(&testDatagram{
		packets: []*testPacket{{
			ptype:             packetTypeRetry,
			originalDstConnID: testLocalConnID(-1),
			srcConnID:         testPeerConnID(100),
			dstConnID:         testLocalConnID(0),
			token:             []byte{},
		}},
	})
	tc.wantIdle("client ignores Retry with zero-length token")
}

func TestRetryStateValidateInvalidToken(t *testing.T) {
	// Test handling of tokens that may have a valid signature,
	// but unexpected contents.
	var rs retryState
	if err := rs.init(); err != nil {
		t.Fatal(err)
	}
	nonce := make([]byte, rs.aead.NonceSize())
	now := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
	srcConnID := []byte{1, 2, 3, 4}
	dstConnID := nonce[:20]
	addr := testClientAddr

	for _, test := range []struct {
		name  string
		token []byte
	}{{
		name:  "token too short",
		token: []byte{1, 2, 3},
	}, {
		name: "token plaintext too short",
		token: func() []byte {
			plaintext := make([]byte, 7) // not enough bytes of content
			token := append([]byte{}, nonce[20:]...)
			return rs.aead.Seal(token, nonce, plaintext, rs.additionalData(srcConnID, addr))
		}(),
	}} {
		t.Run(test.name, func(t *testing.T) {
			if _, ok := rs.validateToken(now, test.token, srcConnID, dstConnID, addr); ok {
				t.Errorf("validateToken succeeded, want failure")
			}
		})
	}
}

func TestParseInvalidRetryPackets(t *testing.T) {
	originalDstConnID := []byte{1, 2, 3, 4}
	goodPkt := encodeRetryPacket(originalDstConnID, retryPacket{
		dstConnID: []byte{1},
		srcConnID: []byte{2},
		token:     []byte{3},
	})
	for _, test := range []struct {
		name string
		pkt  []byte
	}{{
		name: "packet too short",
		pkt:  goodPkt[:len(goodPkt)-4],
	}, {
		name: "packet header invalid",
		pkt:  goodPkt[:5],
	}, {
		name: "integrity tag invalid",
		pkt: func() []byte {
			pkt := cloneBytes(goodPkt)
			pkt[len(pkt)-1] ^= 1
			return pkt
		}(),
	}} {
		t.Run(test.name, func(t *testing.T) {
			if _, ok := parseRetryPacket(test.pkt, originalDstConnID); ok {
				t.Errorf("parseRetryPacket succeeded, want failure")
			}
		})
	}
}

func initialClientCrypto(t *testing.T, e *testEndpoint, p transportParameters) []byte {
	t.Helper()
	config := &tls.QUICConfig{TLSConfig: newTestTLSConfig(clientSide)}
	tlsClient := tls.QUICClient(config)
	tlsClient.SetTransportParameters(marshalTransportParameters(p))
	tlsClient.Start(context.Background())
	t.Cleanup(func() {
		tlsClient.Close()
	})
	e.peerTLSConn = tlsClient
	var data []byte
	for {
		e := tlsClient.NextEvent()
		switch e.Kind {
		case tls.QUICNoEvent:
			return data
		case tls.QUICWriteData:
			if e.Level != tls.QUICEncryptionLevelInitial {
				t.Fatal("initial data at unexpected level")
			}
			data = append(data, e.Data...)
		}
	}
}

func initialConnectionCloseDatagram(srcConnID, dstConnID []byte, code transportError) *testDatagram {
	return &testDatagram{
		packets: []*testPacket{{
			ptype:     packetTypeInitial,
			num:       0,
			version:   quicVersion1,
			srcConnID: srcConnID,
			dstConnID: dstConnID,
			frames: []debugFrame{
				debugFrameConnectionCloseTransport{
					code: code,
				},
			},
		}},
	}
}
