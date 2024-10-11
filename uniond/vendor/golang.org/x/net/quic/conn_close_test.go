// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"crypto/tls"
	"errors"
	"testing"
	"time"
)

func TestConnCloseResponseBackoff(t *testing.T) {
	tc := newTestConn(t, clientSide, func(c *Config) {
		clear(c.StatelessResetKey[:])
	})
	tc.handshake()

	tc.conn.Abort(nil)
	tc.wantFrame("aborting connection generates CONN_CLOSE",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})

	waiting := runAsync(tc, func(ctx context.Context) (struct{}, error) {
		return struct{}{}, tc.conn.Wait(ctx)
	})
	if _, err := waiting.result(); err != errNotDone {
		t.Errorf("conn.Wait() = %v, want still waiting", err)
	}

	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantIdle("packets received immediately after CONN_CLOSE receive no response")

	tc.advance(1100 * time.Microsecond)
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantFrame("receiving packet 1.1ms after CONN_CLOSE generates another CONN_CLOSE",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})

	tc.advance(1100 * time.Microsecond)
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantIdle("no response to packet, because CONN_CLOSE backoff is now 2ms")

	tc.advance(1000 * time.Microsecond)
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantFrame("2ms since last CONN_CLOSE, receiving a packet generates another CONN_CLOSE",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})
	if _, err := waiting.result(); err != errNotDone {
		t.Errorf("conn.Wait() = %v, want still waiting", err)
	}

	tc.advance(100000 * time.Microsecond)
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantIdle("drain timer expired, no more responses")

	if _, err := waiting.result(); !errors.Is(err, errNoPeerResponse) {
		t.Errorf("blocked conn.Wait() = %v, want errNoPeerResponse", err)
	}
	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, errNoPeerResponse) {
		t.Errorf("non-blocking conn.Wait() = %v, want errNoPeerResponse", err)
	}
}

func TestConnCloseWithPeerResponse(t *testing.T) {
	qr := &qlogRecord{}
	tc := newTestConn(t, clientSide, qr.config)
	tc.handshake()

	tc.conn.Abort(nil)
	tc.wantFrame("aborting connection generates CONN_CLOSE",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})

	waiting := runAsync(tc, func(ctx context.Context) (struct{}, error) {
		return struct{}{}, tc.conn.Wait(ctx)
	})
	if _, err := waiting.result(); err != errNotDone {
		t.Errorf("conn.Wait() = %v, want still waiting", err)
	}

	tc.writeFrames(packetType1RTT, debugFrameConnectionCloseApplication{
		code: 20,
	})

	wantErr := &ApplicationError{
		Code: 20,
	}
	if _, err := waiting.result(); !errors.Is(err, wantErr) {
		t.Errorf("blocked conn.Wait() = %v, want %v", err, wantErr)
	}
	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, wantErr) {
		t.Errorf("non-blocking conn.Wait() = %v, want %v", err, wantErr)
	}

	tc.advance(1 * time.Second) // long enough to exit the draining state
	qr.wantEvents(t, jsonEvent{
		"name": "connectivity:connection_closed",
		"data": map[string]any{
			"trigger": "application",
		},
	})
}

func TestConnClosePeerCloses(t *testing.T) {
	qr := &qlogRecord{}
	tc := newTestConn(t, clientSide, qr.config)
	tc.handshake()

	wantErr := &ApplicationError{
		Code:   42,
		Reason: "why?",
	}
	tc.writeFrames(packetType1RTT, debugFrameConnectionCloseApplication{
		code:   wantErr.Code,
		reason: wantErr.Reason,
	})
	tc.wantIdle("CONN_CLOSE response not sent until user closes this side")

	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, wantErr) {
		t.Errorf("conn.Wait() = %v, want %v", err, wantErr)
	}

	tc.conn.Abort(&ApplicationError{
		Code:   9,
		Reason: "because",
	})
	tc.wantFrame("CONN_CLOSE sent after user closes connection",
		packetType1RTT, debugFrameConnectionCloseApplication{
			code:   9,
			reason: "because",
		})

	tc.advance(1 * time.Second) // long enough to exit the draining state
	qr.wantEvents(t, jsonEvent{
		"name": "connectivity:connection_closed",
		"data": map[string]any{
			"trigger": "application",
		},
	})
}

func TestConnCloseReceiveInInitial(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.wantFrame("client sends Initial CRYPTO frame",
		packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeInitial, debugFrameConnectionCloseTransport{
		code: errConnectionRefused,
	})
	tc.wantIdle("CONN_CLOSE response not sent until user closes this side")

	wantErr := peerTransportError{code: errConnectionRefused}
	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, wantErr) {
		t.Errorf("conn.Wait() = %v, want %v", err, wantErr)
	}

	tc.conn.Abort(&ApplicationError{Code: 1})
	tc.wantFrame("CONN_CLOSE in Initial frame is APPLICATION_ERROR",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errApplicationError,
		})
	tc.wantIdle("no more frames to send")
}

func TestConnCloseReceiveInHandshake(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)
	tc.wantFrame("client sends Initial CRYPTO frame",
		packetTypeInitial, debugFrameCrypto{
			data: tc.cryptoDataOut[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeInitial, debugFrameCrypto{
		data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
	})
	tc.writeFrames(packetTypeHandshake, debugFrameConnectionCloseTransport{
		code: errConnectionRefused,
	})
	tc.wantIdle("CONN_CLOSE response not sent until user closes this side")

	wantErr := peerTransportError{code: errConnectionRefused}
	if err := tc.conn.Wait(canceledContext()); !errors.Is(err, wantErr) {
		t.Errorf("conn.Wait() = %v, want %v", err, wantErr)
	}

	// The conn has Initial and Handshake keys, so it will send CONN_CLOSE in both spaces.
	tc.conn.Abort(&ApplicationError{Code: 1})
	tc.wantFrame("CONN_CLOSE in Initial frame is APPLICATION_ERROR",
		packetTypeInitial, debugFrameConnectionCloseTransport{
			code: errApplicationError,
		})
	tc.wantFrame("CONN_CLOSE in Handshake frame is APPLICATION_ERROR",
		packetTypeHandshake, debugFrameConnectionCloseTransport{
			code: errApplicationError,
		})
	tc.wantIdle("no more frames to send")
}

func TestConnCloseClosedByEndpoint(t *testing.T) {
	ctx := canceledContext()
	tc := newTestConn(t, clientSide)
	tc.handshake()

	tc.endpoint.e.Close(ctx)
	tc.wantFrame("endpoint closes connection before exiting",
		packetType1RTT, debugFrameConnectionCloseTransport{
			code: errNo,
		})
}

func testConnCloseUnblocks(t *testing.T, f func(context.Context, *testConn) error, opts ...any) {
	tc := newTestConn(t, clientSide, opts...)
	tc.handshake()
	op := runAsync(tc, func(ctx context.Context) (struct{}, error) {
		return struct{}{}, f(ctx, tc)
	})
	if _, err := op.result(); err != errNotDone {
		t.Fatalf("before abort, op = %v, want errNotDone", err)
	}
	tc.conn.Abort(nil)
	if _, err := op.result(); err == nil || err == errNotDone {
		t.Fatalf("after abort, op = %v, want error", err)
	}
}

func TestConnCloseUnblocksAcceptStream(t *testing.T) {
	testConnCloseUnblocks(t, func(ctx context.Context, tc *testConn) error {
		_, err := tc.conn.AcceptStream(ctx)
		return err
	}, permissiveTransportParameters)
}

func TestConnCloseUnblocksNewStream(t *testing.T) {
	testConnCloseUnblocks(t, func(ctx context.Context, tc *testConn) error {
		_, err := tc.conn.NewStream(ctx)
		return err
	})
}

func TestConnCloseUnblocksStreamRead(t *testing.T) {
	testConnCloseUnblocks(t, func(ctx context.Context, tc *testConn) error {
		s := newLocalStream(t, tc, bidiStream)
		s.SetReadContext(ctx)
		buf := make([]byte, 16)
		_, err := s.Read(buf)
		return err
	}, permissiveTransportParameters)
}

func TestConnCloseUnblocksStreamWrite(t *testing.T) {
	testConnCloseUnblocks(t, func(ctx context.Context, tc *testConn) error {
		s := newLocalStream(t, tc, bidiStream)
		s.SetWriteContext(ctx)
		buf := make([]byte, 32)
		_, err := s.Write(buf)
		return err
	}, permissiveTransportParameters, func(c *Config) {
		c.MaxStreamWriteBufferSize = 16
	})
}

func TestConnCloseUnblocksStreamClose(t *testing.T) {
	testConnCloseUnblocks(t, func(ctx context.Context, tc *testConn) error {
		s := newLocalStream(t, tc, bidiStream)
		s.SetWriteContext(ctx)
		buf := make([]byte, 16)
		_, err := s.Write(buf)
		if err != nil {
			return err
		}
		return s.Close()
	}, permissiveTransportParameters)
}
