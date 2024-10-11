// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
)

func TestPathChallengeReceived(t *testing.T) {
	for _, test := range []struct {
		name        string
		padTo       int
		wantPadding int
	}{{
		name:        "unexpanded",
		padTo:       0,
		wantPadding: 0,
	}, {
		name:        "expanded",
		padTo:       1200,
		wantPadding: 1200,
	}} {
		// "The recipient of [a PATH_CHALLENGE] frame MUST generate
		// a PATH_RESPONSE frame [...] containing the same Data value."
		// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.17-7
		tc := newTestConn(t, clientSide)
		tc.handshake()
		tc.ignoreFrame(frameTypeAck)
		data := pathChallengeData{0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef}
		tc.writeFrames(packetType1RTT, debugFramePathChallenge{
			data: data,
		}, debugFramePadding{
			to: test.padTo,
		})
		tc.wantFrame("response to PATH_CHALLENGE",
			packetType1RTT, debugFramePathResponse{
				data: data,
			})
		if got, want := tc.lastDatagram.paddedSize, test.wantPadding; got != want {
			t.Errorf("PATH_RESPONSE expanded to %v bytes, want %v", got, want)
		}
		tc.wantIdle("connection is idle")
	}
}

func TestPathResponseMismatchReceived(t *testing.T) {
	// "If the content of a PATH_RESPONSE frame does not match the content of
	// a PATH_CHALLENGE frame previously sent by the endpoint,
	// the endpoint MAY generate a connection error of type PROTOCOL_VIOLATION."
	// https://www.rfc-editor.org/rfc/rfc9000.html#section-19.18-4
	tc := newTestConn(t, clientSide)
	tc.handshake()
	tc.ignoreFrame(frameTypeAck)
	tc.writeFrames(packetType1RTT, debugFramePathResponse{
		data: pathChallengeData{},
	})
	tc.wantFrame("invalid PATH_RESPONSE causes the connection to close",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errProtocolViolation,
		},
	)
}
