// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"crypto/tls"
	"fmt"
	"net/netip"
	"strings"
	"testing"
)

func TestConnIDClientHandshake(t *testing.T) {
	tc := newTestConn(t, clientSide)
	// On initialization, the client chooses local and remote IDs.
	//
	// The order in which we allocate the two isn't actually important,
	// but test is a lot simpler if we assume.
	if got, want := tc.conn.connIDState.srcConnID(), testLocalConnID(0); !bytes.Equal(got, want) {
		t.Errorf("after initialization: srcConnID = %x, want %x", got, want)
	}
	dstConnID, _ := tc.conn.connIDState.dstConnID()
	if got, want := dstConnID, testLocalConnID(-1); !bytes.Equal(got, want) {
		t.Errorf("after initialization: dstConnID = %x, want %x", got, want)
	}

	// The server's first Initial packet provides the client with a
	// non-transient remote connection ID.
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	dstConnID, _ = tc.conn.connIDState.dstConnID()
	if got, want := dstConnID, testPeerConnID(0); !bytes.Equal(got, want) {
		t.Errorf("after receiving Initial: dstConnID = %x, want %x", got, want)
	}

	wantLocal := []connID{{
		cid: testLocalConnID(0),
		seq: 0,
	}}
	if got := tc.conn.connIDState.local; !connIDListEqual(got, wantLocal) {
		t.Errorf("local ids: %v, want %v", fmtConnIDList(got), fmtConnIDList(wantLocal))
	}
	wantRemote := []remoteConnID{{
		connID: connID{
			cid: testPeerConnID(0),
			seq: 0,
		},
	}}
	if got := tc.conn.connIDState.remote; !remoteConnIDListEqual(got, wantRemote) {
		t.Errorf("remote ids: %v, want %v", fmtRemoteConnIDList(got), fmtRemoteConnIDList(wantRemote))
	}
}

func TestConnIDServerHandshake(t *testing.T) {
	tc := newTestConn(t, serverSide)
	// On initialization, the server is provided with the client-chosen
	// transient connection ID, and allocates an ID of its own.
	// The Initial packet sets the remote connection ID.
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial][:1],
		})
	if got, want := tc.conn.connIDState.srcConnID(), testLocalConnID(0); !bytes.Equal(got, want) {
		t.Errorf("after initClient: srcConnID = %q, want %q", got, want)
	}
	dstConnID, _ := tc.conn.connIDState.dstConnID()
	if got, want := dstConnID, testPeerConnID(0); !bytes.Equal(got, want) {
		t.Errorf("after initClient: dstConnID = %q, want %q", got, want)
	}

	// The Initial flight of CRYPTO data includes transport parameters,
	// which cause us to allocate another local connection ID.
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			off:  1,
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial][1:],
		})
	wantLocal := []connID{{
		cid: testPeerConnID(-1),
		seq: -1,
	}, {
		cid: testLocalConnID(0),
		seq: 0,
	}, {
		cid: testLocalConnID(1),
		seq: 1,
	}}
	if got := tc.conn.connIDState.local; !connIDListEqual(got, wantLocal) {
		t.Errorf("local ids: %v, want %v", fmtConnIDList(got), fmtConnIDList(wantLocal))
	}
	wantRemote := []remoteConnID{{
		connID: connID{
			cid: testPeerConnID(0),
			seq: 0,
		},
	}}
	if got := tc.conn.connIDState.remote; !remoteConnIDListEqual(got, wantRemote) {
		t.Errorf("remote ids: %v, want %v", fmtRemoteConnIDList(got), fmtRemoteConnIDList(wantRemote))
	}

	// The client's first Handshake packet permits the server to discard the
	// transient connection ID.
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
		})
	wantLocal = []connID{{
		cid: testLocalConnID(0),
		seq: 0,
	}, {
		cid: testLocalConnID(1),
		seq: 1,
	}}
	if got := tc.conn.connIDState.local; !connIDListEqual(got, wantLocal) {
		t.Errorf("local ids: %v, want %v", fmtConnIDList(got), fmtConnIDList(wantLocal))
	}
}

func connIDListEqual(a, b []connID) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i].seq != b[i].seq {
			return false
		}
		if !bytes.Equal(a[i].cid, b[i].cid) {
			return false
		}
	}
	return true
}

func remoteConnIDListEqual(a, b []remoteConnID) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i].seq != b[i].seq {
			return false
		}
		if !bytes.Equal(a[i].cid, b[i].cid) {
			return false
		}
		if a[i].resetToken != b[i].resetToken {
			return false
		}
	}
	return true
}

func fmtConnIDList(s []connID) string {
	var strs []string
	for _, cid := range s {
		strs = append(strs, fmt.Sprintf("[seq:%v cid:{%x}]", cid.seq, cid.cid))
	}
	return "{" + strings.Join(strs, " ") + "}"
}

func fmtRemoteConnIDList(s []remoteConnID) string {
	var strs []string
	for _, cid := range s {
		strs = append(strs, fmt.Sprintf("[seq:%v cid:{%x} token:{%x}]", cid.seq, cid.cid, cid.resetToken))
	}
	return "{" + strings.Join(strs, " ") + "}"
}

func TestNewRandomConnID(t *testing.T) {
	cid, err := newRandomConnID(0)
	if len(cid) != connIDLen || err != nil {
		t.Fatalf("newConnID() = %x, %v; want %v bytes", cid, connIDLen, err)
	}
}

func TestConnIDPeerRequestsManyIDs(t *testing.T) {
	// "An endpoint SHOULD ensure that its peer has a sufficient number
	// of available and unused connection IDs."
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.1-4
	//
	// "An endpoint MAY limit the total number of connection IDs
	// issued for each connection [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.1-6
	//
	// Peer requests 100 connection IDs.
	// We give them 4 in total.
	tc := newTestConn(t, serverSide, func(p *transportParameters) {
		p.activeConnIDLimit = 100
	})
	tc.ignoreFrame(frameTypeAck)
	tc.ignoreFrame(frameTypeCrypto)

	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.wantFrame("provide additional connection ID 1",
		packetType1RTT, debugFrameNewConnectionID{
			seq:    1,
			connID: testLocalConnID(1),
			token:  testLocalStatelessResetToken(1),
		})
	tc.wantFrame("provide additional connection ID 2",
		packetType1RTT, debugFrameNewConnectionID{
			seq:    2,
			connID: testLocalConnID(2),
			token:  testLocalStatelessResetToken(2),
		})
	tc.wantFrame("provide additional connection ID 3",
		packetType1RTT, debugFrameNewConnectionID{
			seq:    3,
			connID: testLocalConnID(3),
			token:  testLocalStatelessResetToken(3),
		})
	tc.wantIdle("connection ID limit reached, no more to provide")
}

func TestConnIDPeerProvidesTooManyIDs(t *testing.T) {
	// "An endpoint MUST NOT provide more connection IDs than the peer's limit."
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.1-4
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:    2,
			connID: testLocalConnID(2),
		})
	tc.wantFrame("peer provided 3 connection IDs, our limit is 2",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errConnectionIDLimit,
		})
}

func TestConnIDPeerTemporarilyExceedsActiveConnIDLimit(t *testing.T) {
	// "An endpoint MAY send connection IDs that temporarily exceed a peer's limit
	// if the NEW_CONNECTION_ID frame also requires the retirement of any excess [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.1-4
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			retirePriorTo: 2,
			seq:           2,
			connID:        testPeerConnID(2),
		}, debugFrameNewConnectionID{
			retirePriorTo: 2,
			seq:           3,
			connID:        testPeerConnID(3),
		})
	tc.wantFrame("peer requested we retire conn id 0",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 0,
		})
	tc.wantFrame("peer requested we retire conn id 1",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 1,
		})
}

func TestConnIDPeerRetiresConnID(t *testing.T) {
	// "An endpoint SHOULD supply a new connection ID when the peer retires a connection ID."
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.1-6
	for _, side := range []connSide{
		clientSide,
		serverSide,
	} {
		t.Run(side.String(), func(t *testing.T) {
			tc := newTestConn(t, side)
			tc.handshake()
			tc.ignoreFrame(frameTypeAck)

			tc.writeFrames(packetType1RTT,
				debugFrameRetireConnectionID{
					seq: 0,
				})
			tc.wantFrame("provide replacement connection ID",
				packetType1RTT, debugFrameNewConnectionID{
					seq:           2,
					retirePriorTo: 1,
					connID:        testLocalConnID(2),
					token:         testLocalStatelessResetToken(2),
				})
		})
	}
}

func TestConnIDPeerWithZeroLengthConnIDSendsNewConnectionID(t *testing.T) {
	// "An endpoint that selects a zero-length connection ID during the handshake
	// cannot issue a new connection ID."
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.1-8
	tc := newTestConn(t, clientSide, func(p *transportParameters) {
		p.initialSrcConnID = []byte{}
	})
	tc.peerConnID = []byte{}
	tc.ignoreFrame(frameTypeAck)
	tc.uncheckedHandshake()

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:    1,
			connID: testPeerConnID(1),
		})
	tc.wantFrame("invalid NEW_CONNECTION_ID: previous conn id is zero-length",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errProtocolViolation,
		})
}

func TestConnIDPeerRequestsRetirement(t *testing.T) {
	// "Upon receipt of an increased Retire Prior To field, the peer MUST
	// stop using the corresponding connection IDs and retire them with
	// RETIRE_CONNECTION_ID frames [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.2-5
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

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
	if got, want := tc.lastPacket.dstConnID, testPeerConnID(1); !bytes.Equal(got, want) {
		t.Fatalf("used destination conn id {%x}, want {%x}", got, want)
	}
}

func TestConnIDPeerDoesNotAcknowledgeRetirement(t *testing.T) {
	// "An endpoint SHOULD limit the number of connection IDs it has retired locally
	// for which RETIRE_CONNECTION_ID frames have not yet been acknowledged."
	// https://www.rfc-editor.org/rfc/rfc9000#section-5.1.2-6
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	tc.ignoreFrame(frameTypeRetireConnectionID)

	// Send a number of NEW_CONNECTION_ID frames, each retiring an old one.
	for seq := int64(0); seq < 7; seq++ {
		tc.writeFrames(packetType1RTT,
			debugFrameNewConnectionID{
				seq:           seq + 2,
				retirePriorTo: seq + 1,
				connID:        testPeerConnID(seq + 2),
			})
		// We're ignoring the RETIRE_CONNECTION_ID frames.
	}
	tc.wantFrame("number of retired, unacked conn ids is too large",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errConnectionIDLimit,
		})
}

func TestConnIDRepeatedNewConnectionIDFrame(t *testing.T) {
	// "Receipt of the same [NEW_CONNECTION_ID] frame multiple times
	// MUST NOT be treated as a connection error.
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.15-7
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	for i := 0; i < 4; i++ {
		tc.writeFrames(packetType1RTT,
			debugFrameNewConnectionID{
				seq:           2,
				retirePriorTo: 1,
				connID:        testPeerConnID(2),
			})
	}
	tc.wantFrame("peer asked for conn id to be retired",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 0,
		})
	tc.wantIdle("repeated NEW_CONNECTION_ID frames are not an error")
}

func TestConnIDForSequenceNumberChanges(t *testing.T) {
	// "[...] if a sequence number is used for different connection IDs,
	// the endpoint MAY treat that receipt as a connection error
	// of type PROTOCOL_VIOLATION."
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.15-8
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	tc.ignoreFrame(frameTypeRetireConnectionID)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           2,
			retirePriorTo: 1,
			connID:        testPeerConnID(2),
		})
	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           2,
			retirePriorTo: 1,
			connID:        testPeerConnID(3),
		})
	tc.wantFrame("connection ID for sequence 0 has changed",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errProtocolViolation,
		})
}

func TestConnIDRetirePriorToAfterNewConnID(t *testing.T) {
	// "Receiving a value in the Retire Prior To field that is greater than
	// that in the Sequence Number field MUST be treated as a connection error
	// of type FRAME_ENCODING_ERROR.
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.15-9
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			retirePriorTo: 3,
			seq:           2,
			connID:        testPeerConnID(2),
		})
	tc.wantFrame("invalid NEW_CONNECTION_ID: retired the new conn id",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errFrameEncoding,
		})
}

func TestConnIDAlreadyRetired(t *testing.T) {
	// "An endpoint that receives a NEW_CONNECTION_ID frame with a
	// sequence number smaller than the Retire Prior To field of a
	// previously received NEW_CONNECTION_ID frame MUST send a
	// corresponding RETIRE_CONNECTION_ID frame [...]"
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.15-11
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           4,
			retirePriorTo: 3,
			connID:        testPeerConnID(4),
		})
	tc.wantFrame("peer asked for conn id to be retired",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 0,
		})
	tc.wantFrame("peer asked for conn id to be retired",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 1,
		})
	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           2,
			retirePriorTo: 0,
			connID:        testPeerConnID(2),
		})
	tc.wantFrame("NEW_CONNECTION_ID was for an already-retired ID",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 2,
		})
}

func TestConnIDRepeatedRetireConnectionIDFrame(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	for i := 0; i < 4; i++ {
		tc.writeFrames(packetType1RTT,
			debugFrameRetireConnectionID{
				seq: 0,
			})
	}
	tc.wantFrame("issue new conn id after peer retires one",
		packetType1RTT, debugFrameNewConnectionID{
			retirePriorTo: 1,
			seq:           2,
			connID:        testLocalConnID(2),
			token:         testLocalStatelessResetToken(2),
		})
	tc.wantIdle("repeated RETIRE_CONNECTION_ID frames are not an error")
}

func TestConnIDRetiredUnsent(t *testing.T) {
	// "Receipt of a RETIRE_CONNECTION_ID frame containing a sequence number
	// greater than any previously sent to the peer MUST be treated as a
	// connection error of type PROTOCOL_VIOLATION."
	// https://www.rfc-editor.org/rfc/rfc9000#section-19.16-7
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameRetireConnectionID{
			seq: 2,
		})
	tc.wantFrame("invalid NEW_CONNECTION_ID: previous conn id is zero-length",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errProtocolViolation,
		})
}

func TestConnIDUsePreferredAddressConnID(t *testing.T) {
	// Peer gives us a connection ID in the preferred address transport parameter.
	// We don't use the preferred address at this time, but we should use the
	// connection ID. (It isn't tied to any specific address.)
	//
	// This test will probably need updating if/when we start using the preferred address.
	cid := testPeerConnID(10)
	tc := newTestConn(t, serverSide, func(p *transportParameters) {
		p.preferredAddrV4 = netip.MustParseAddrPort("0.0.0.0:0")
		p.preferredAddrV6 = netip.MustParseAddrPort("[::0]:0")
		p.preferredAddrConnID = cid
		p.preferredAddrResetToken = make([]byte, 16)
	})
	tc.uncheckedHandshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           2,
			retirePriorTo: 1,
			connID:        []byte{0xff},
		})
	tc.wantFrame("peer asked for conn id 0 to be retired",
		packetType1RTT, debugFrameRetireConnectionID{
			seq: 0,
		})
	if got, want := tc.lastPacket.dstConnID, cid; !bytes.Equal(got, want) {
		t.Fatalf("used destination conn id {%x}, want {%x} from preferred address transport parameter", got, want)
	}
}

func TestConnIDPeerProvidesPreferredAddrAndTooManyConnIDs(t *testing.T) {
	// Peer gives us more conn ids than our advertised limit,
	// including a conn id in the preferred address transport parameter.
	cid := testPeerConnID(10)
	tc := newTestConn(t, serverSide, func(p *transportParameters) {
		p.preferredAddrV4 = netip.MustParseAddrPort("0.0.0.0:0")
		p.preferredAddrV6 = netip.MustParseAddrPort("[::0]:0")
		p.preferredAddrConnID = cid
		p.preferredAddrResetToken = make([]byte, 16)
	})
	tc.uncheckedHandshake()
	tc.ignoreFrame(frameTypeAck)

	tc.writeFrames(packetType1RTT,
		debugFrameNewConnectionID{
			seq:           2,
			retirePriorTo: 0,
			connID:        testPeerConnID(2),
		})
	tc.wantFrame("peer provided 3 connection IDs, our limit is 2",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errConnectionIDLimit,
		})
}

func TestConnIDPeerWithZeroLengthIDProvidesPreferredAddr(t *testing.T) {
	// Peer gives us more conn ids than our advertised limit,
	// including a conn id in the preferred address transport parameter.
	tc := newTestConn(t, serverSide, func(p *transportParameters) {
		p.initialSrcConnID = []byte{}
		p.preferredAddrV4 = netip.MustParseAddrPort("0.0.0.0:0")
		p.preferredAddrV6 = netip.MustParseAddrPort("[::0]:0")
		p.preferredAddrConnID = testPeerConnID(1)
		p.preferredAddrResetToken = make([]byte, 16)
	}, func(cids *newServerConnIDs) {
		cids.srcConnID = []byte{}
	}, func(tc *testConn) {
		tc.peerConnID = []byte{}
	})

	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.wantFrame("peer with zero-length connection ID tried to provide another in transport parameters",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errProtocolViolation,
		})
}

func TestConnIDInitialSrcConnIDMismatch(t *testing.T) {
	// "Endpoints MUST validate that received [initial_source_connection_id]
	// parameters match received connection ID values."
	// https://www.rfc-editor.org/rfc/rfc9000#section-7.3-3
	testSides(t, "", func(t *testing.T, side connSide) {
		tc := newTestConn(t, side, func(p *transportParameters) {
			p.initialSrcConnID = []byte("invalid")
		})
		tc.ignoreFrame(frameTypeAck)
		tc.ignoreFrame(frameTypeCrypto)
		tc.writeFrames(packetTypeInitial,
			debugFrameCrypto{
				data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
			})
		if side == clientSide {
			// Server transport parameters are carried in the Handshake packet.
			tc.writeFrames(packetTypeHandshake,
				debugFrameCrypto{
					data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
				})
		}
		tc.wantFrame("initial_source_connection_id transport parameter mismatch",
			packetTypeInitial, debugFrameConnectionCloseTransport{
				code: errTransportParameter,
			})
	})
}

func TestConnIDsCleanedUpAfterClose(t *testing.T) {
	testSides(t, "", func(t *testing.T, side connSide) {
		tc := newTestConn(t, side, func(p *transportParameters) {
			if side == clientSide {
				token := testPeerStatelessResetToken(0)
				p.statelessResetToken = token[:]
			}
		})
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		tc.writeFrames(packetType1RTT,
			debugFrameNewConnectionID{
				seq:           2,
				retirePriorTo: 1,
				connID:        testPeerConnID(2),
				token:         testPeerStatelessResetToken(0),
			})
		tc.wantFrame("peer asked for conn id 0 to be retired",
			packetType1RTT, debugFrameRetireConnectionID{
				seq: 0,
			})
		tc.writeFrames(packetType1RTT, debugFrameConnectionCloseTransport{})
		tc.conn.Abort(nil)
		tc.wantFrame("CONN_CLOSE sent after user closes connection",
			packetType1RTT, debugFrameConnectionCloseTransport{})

		// Wait for the conn to drain.
		// Then wait for the conn loop to exit,
		// and force an immediate sync of the connsMap updates
		// (normally only done by the endpoint read loop).
		tc.advanceToTimer()
		<-tc.conn.donec
		tc.endpoint.e.connsMap.applyUpdates()

		if got := len(tc.endpoint.e.connsMap.byConnID); got != 0 {
			t.Errorf("%v conn ids in endpoint map after closing, want 0", got)
		}
		if got := len(tc.endpoint.e.connsMap.byResetToken); got != 0 {
			t.Errorf("%v reset tokens in endpoint map after closing, want 0", got)
		}
	})
}
