// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/tls"
	"errors"
	"net/netip"
	"testing"
	"time"
)

func TestStatelessResetClientSendsStatelessResetTokenTransportParameter(t *testing.T) {
	// "[The stateless_reset_token] transport parameter MUST NOT be sent by a client [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-18.2-4.6.1
	resetToken := testPeerStatelessResetToken(0)
	tc := newTestConn(t, serverSide, func(p *transportParameters) {
		p.statelessResetToken = resetToken[:]
	})
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.wantFrame("client provided stateless_reset_token transport parameter",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errTransportParameter,
		})
}

var testStatelessResetKey = func() (key [32]byte) {
	if _, err := rand.Read(key[:]); err != nil {
		panic(err)
	}
	return key
}()

func testStatelessResetToken(cid []byte) statelessResetToken {
	var gen statelessResetTokenGenerator
	gen.init(testStatelessResetKey)
	return gen.tokenForConnID(cid)
}

func testLocalStatelessResetToken(seq int64) statelessResetToken {
	return testStatelessResetToken(testLocalConnID(seq))
}

func newDatagramForReset(cid []byte, size int, addr netip.AddrPort) *datagram {
	dgram := append([]byte{headerFormShort | fixedBit}, cid...)
	for len(dgram) < size {
		dgram = append(dgram, byte(len(dgram))) // semi-random junk
	}
	return &datagram{
		b:        dgram,
		peerAddr: addr,
	}
}

func TestStatelessResetSentSizes(t *testing.T) {
	config := &Config{
		TLSConfig:         newTestTLSConfig(serverSide),
		StatelessResetKey: testStatelessResetKey,
	}
	addr := netip.MustParseAddr("127.0.0.1")
	te := newTestEndpoint(t, config)
	for i, test := range []struct {
		reqSize  int
		wantSize int
	}{{
		// Datagrams larger than 42 bytes result in a 42-byte stateless reset.
		// This isn't specifically mandated by RFC 9000, but is implied.
		// https://www.rfc-editor.org/rfc/rfc9000#section-10.3-11
		reqSize:  1200,
		wantSize: 42,
	}, {
		// "An endpoint that sends a Stateless Reset in response to a packet
		// that is 43 bytes or shorter SHOULD send a Stateless Reset that is
		// one byte shorter than the packet it responds to."
		// https://www.rfc-editor.org/rfc/rfc9000#section-10.3-11
		reqSize:  43,
		wantSize: 42,
	}, {
		reqSize:  42,
		wantSize: 41,
	}, {
		// We should send a stateless reset in response to the smallest possible
		// valid datagram the peer can send us.
		// The smallest packet is 1-RTT:
		// header byte, conn id, packet num, payload, AEAD.
		reqSize:  1 + connIDLen + 1 + 1 + 16,
		wantSize: 1 + connIDLen + 1 + 1 + 16 - 1,
	}, {
		// The smallest possible stateless reset datagram is 21 bytes.
		// Since our response must be smaller than the incoming datagram,
		// we must not respond to a 21 byte or smaller packet.
		reqSize:  21,
		wantSize: 0,
	}} {
		cid := testLocalConnID(int64(i))
		token := testStatelessResetToken(cid)
		addrport := netip.AddrPortFrom(addr, uint16(8000+i))
		te.write(newDatagramForReset(cid, test.reqSize, addrport))

		got := te.read()
		if len(got) != test.wantSize {
			t.Errorf("got %v-byte response to %v-byte req, want %v",
				len(got), test.reqSize, test.wantSize)
		}
		if len(got) == 0 {
			continue
		}
		// "Endpoints MUST send Stateless Resets formatted as
		// a packet with a short header."
		// https://www.rfc-editor.org/rfc/rfc9000#section-10.3-15
		if isLongHeader(got[0]) {
			t.Errorf("response to %v-byte request is not a short-header packet\ngot: %x", test.reqSize, got)
		}
		if !bytes.HasSuffix(got, token[:]) {
			t.Errorf("response to %v-byte request does not end in stateless reset token\ngot: %x\nwant suffix: %x", test.reqSize, got, token)
		}
	}
}

func TestStatelessResetSuccessfulNewConnectionID(t *testing.T) {
	// "[...] Stateless Reset Token field values from [...] NEW_CONNECTION_ID frames [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-10.3.1-1
	qr := &qlogRecord{}
	tc := newTestConn(t, clientSide, qr.config)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	// Retire connection ID 0.
	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			retirePriorTo: 1,
			seq:           2,
			connID:        testPeerConnID(2),
		})
	tc.wantFrame("peer requested we retire conn id 0",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 0,
		})

	resetToken := testPeerStatelessResetToken(1) // provided during handshake
	dgram := append(make([]byte, 100), resetToken[:]...)
	tc.endpoint.write(&datagram{
		b: dgram,
	})

	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, errStatelessReset) {
		t.Errorf("conn.Wait() = %v, want errStatelessReset", err)
	}
	tc.wantIdle("closed connection is idle in draining")
	tc.advance(1 * time.Second) // long enough to exit the draining state
	tc.wantIdle("closed connection is idle after draining")

	qr.wantEvents(t, jsonEvent{
		"name": "connectivity:connection_closed",
		"data": map[string]any{
			"trigger": "stateless_reset",
		},
	})
}

func TestStatelessResetSuccessfulTransportParameter(t *testing.T) {
	// "[...] Stateless Reset Token field values from [...]
	// the server's transport parameters [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-10.3.1-1
	resetToken := testPeerStatelessResetToken(0)
	tc := newTestConn(t, clientSide, func(p *transportParameters) {
		p.statelessResetToken = resetToken[:]
	})
	tc.handshake()

	dgram := append(make([]byte, 100), resetToken[:]...)
	tc.endpoint.write(&datagram{
		b: dgram,
	})

	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, errStatelessReset) {
		t.Errorf("conn.Wait() = %v, want errStatelessReset", err)
	}
	tc.wantIdle("closed connection is idle")
}

func TestStatelessResetSuccessfulPrefix(t *testing.T) {
	for _, test := range []struct {
		name   string
		prefix []byte
		size   int
	}{{
		name: "short header and fixed bit",
		prefix: []byte{
			headerFormShort | fixedBit,
		},
		size: 100,
	}, {
		// "[...] endpoints MUST treat [long header packets] ending in a
		// valid stateless reset token as a Stateless Reset [...]"
		// https://www.rfc-editor.org/rfc/rfc9000#section-10.3-15
		name: "long header no fixed bit",
		prefix: []byte{
			headerFormLong,
		},
		size: 100,
	}, {
		// "[...] the comparison MUST be performed when the first packet
		// in an incoming datagram [...] cannot be decrypted."
		// https://www.rfc-editor.org/rfc/rfc9000#section-10.3.1-2
		name: "short header valid DCID",
		prefix: append([]byte{
			headerFormShort | fixedBit,
		}, testLocalConnID(0)...),
		size: 100,
	}, {
		name: "handshake valid DCID",
		prefix: append([]byte{
			headerFormLong | fixedBit | longPacketTypeHandshake,
		}, testLocalConnID(0)...),
		size: 100,
	}, {
		name: "no fixed bit valid DCID",
		prefix: append([]byte{
			0,
		}, testLocalConnID(0)...),
		size: 100,
	}} {
		t.Run(test.name, func(t *testing.T) {
			resetToken := testPeerStatelessResetToken(0)
			tc := newTestConn(t, clientSide, func(p *transportParameters) {
				p.statelessResetToken = resetToken[:]
			})
			tc.handshake()

			dgram := test.prefix
			for len(dgram) < test.size-len(resetToken) {
				dgram = append(dgram, byte(len(dgram))) // semi-random junk
			}
			dgram = append(dgram, resetToken[:]...)
			tc.endpoint.write(&datagram{
				b: dgram,
			})
			if err := tc.conn.Wait(canceledContext()); !errors.Is(err, errStatelessReset) {
				t.Errorf("conn.Wait() = %v, want errStatelessReset", err)
			}
		})
	}
}

func TestStatelessResetRetiredConnID(t *testing.T) {
	// "An endpoint MUST NOT check for any stateless reset tokens [...]
	// for connection IDs that have been retired."
	// https://www.rfc-editor.org/rfc/rfc9000#section-10.3.1-3
	resetToken := testPeerStatelessResetToken(0)
	tc := newTestConn(t, clientSide, func(p *transportParameters) {
		p.statelessResetToken = resetToken[:]
	})
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	// We retire connection ID 0.
	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           2,
			retirePriorTo: 1,
			connID:        testPeerConnID(2),
		})
	tc.wantFrame("peer asked for conn id 0 to be retired",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 0,
		})

	// Receive a stateless reset for connection ID 0.
	dgram := append(make([]byte, 100), resetToken[:]...)
	tc.endpoint.write(&datagram{
		b: dgram,
	})

	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, context.Canceled) {
		t.Errorf("conn.Wait() = %v, want connection to be alive", err)
	}
}
