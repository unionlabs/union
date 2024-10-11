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
	"time"
)

func TestHandshakeTimeoutExpiresServer(t *testing.T) {
	const timeout = 5 * time.Second
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.HandshakeTimeout = timeout
	})
	tc.ignoreFrame(frameTypeAck)
	tc.ignoreFrame(frameTypeNewConnectionID)
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	// Server starts its end of the handshake.
	// Client acks these packets to avoid starting the PTO timer.
	tc.wantFrameType("server sends Initial CRYPTO flight",
		packetTypeInitial, debugFrameCrypto{})
	tc.writeAckForAll()
	tc.wantFrameType("server sends Handshake CRYPTO flight",
		packetTypeHandshake, debugFrameCrypto{})
	tc.writeAckForAll()

	if got, want := tc.timerDelay(), timeout; got != want {
		t.Errorf("connection timer = %v, want %v (handshake timeout)", got, want)
	}

	// Client sends a packet, but this does not extend the handshake timer.
	tc.advance(1 * time.Second)
	tc.writeFrames(packetTypeHandshake, debugFrameCrypto{
		data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake][:1], // partial data
	})
	tc.wantIdle("handshake is not complete")

	tc.advance(timeout - 1*time.Second)
	tc.wantFrame("server closes connection after handshake timeout",
		packetTypeHandshake, debugFrameConnectionCloseTransport{
			code: errConnectionRefused,
		})
}

func TestHandshakeTimeoutExpiresClient(t *testing.T) {
	const timeout = 5 * time.Second
	tc := newTestConn(t, clientSide, func(c *Config) {
		c.HandshakeTimeout = timeout
	})
	tc.ignoreFrame(frameTypeAck)
	tc.ignoreFrame(frameTypeNewConnectionID)
	// Start the handshake.
	// The client always sets a PTO timer until it gets an ack for a handshake packet
	// or confirms the handshake, so proceed far enough through the handshake to
	// let us not worry about PTO.
	tc.wantFrameType("client sends Initial CRYPTO flight",
		packetTypeInitial, debugFrameCrypto{})
	tc.writeAckForAll()
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
		})
	tc.wantFrameType("client sends Handshake CRYPTO flight",
		packetTypeHandshake, debugFrameCrypto{})
	tc.writeAckForAll()
	tc.wantIdle("client is waiting for end of handshake")

	if got, want := tc.timerDelay(), timeout; got != want {
		t.Errorf("connection timer = %v, want %v (handshake timeout)", got, want)
	}
	tc.advance(timeout)
	tc.wantFrame("client closes connection after handshake timeout",
		packetTypeHandshake, debugFrameConnectionCloseTransport{
			code: errConnectionRefused,
		})
}

func TestIdleTimeoutExpires(t *testing.T) {
	for _, test := range []struct {
		localMaxIdleTimeout time.Duration
		peerMaxIdleTimeout  time.Duration
		wantTimeout         time.Duration
	}{{
		localMaxIdleTimeout: 10 * time.Second,
		peerMaxIdleTimeout:  20 * time.Second,
		wantTimeout:         10 * time.Second,
	}, {
		localMaxIdleTimeout: 20 * time.Second,
		peerMaxIdleTimeout:  10 * time.Second,
		wantTimeout:         10 * time.Second,
	}, {
		localMaxIdleTimeout: 0,
		peerMaxIdleTimeout:  10 * time.Second,
		wantTimeout:         10 * time.Second,
	}, {
		localMaxIdleTimeout: 10 * time.Second,
		peerMaxIdleTimeout:  0,
		wantTimeout:         10 * time.Second,
	}} {
		name := fmt.Sprintf("local=%v/peer=%v", test.localMaxIdleTimeout, test.peerMaxIdleTimeout)
		t.Run(name, func(t *testing.T) {
			tc := newTestConn(t, serverSide, func(p *transportParameters) {
				p.maxIdleTimeout = test.peerMaxIdleTimeout
			}, func(c *Config) {
				c.MaxIdleTimeout = test.localMaxIdleTimeout
			})
			tc.handshake()
			if got, want := tc.timeUntilEvent(), test.wantTimeout; got != want {
				t.Errorf("new conn timeout=%v, want %v (idle timeout)", got, want)
			}
			tc.advance(test.wantTimeout - 1)
			tc.wantIdle("connection is idle and alive prior to timeout")
			ctx := canceledContext()
			if err := tc.conn.Wait(ctx); err != context.Canceled {
				t.Fatalf("conn.Wait() = %v, want Canceled", err)
			}
			tc.advance(1)
			tc.wantIdle("connection exits after timeout")
			if err := tc.conn.Wait(ctx); err != errIdleTimeout {
				t.Fatalf("conn.Wait() = %v, want errIdleTimeout", err)
			}
		})
	}
}

func TestIdleTimeoutKeepAlive(t *testing.T) {
	for _, test := range []struct {
		idleTimeout time.Duration
		keepAlive   time.Duration
		wantTimeout time.Duration
	}{{
		idleTimeout: 30 * time.Second,
		keepAlive:   10 * time.Second,
		wantTimeout: 10 * time.Second,
	}, {
		idleTimeout: 10 * time.Second,
		keepAlive:   30 * time.Second,
		wantTimeout: 5 * time.Second,
	}, {
		idleTimeout: -1, // disabled
		keepAlive:   30 * time.Second,
		wantTimeout: 30 * time.Second,
	}} {
		name := fmt.Sprintf("idle_timeout=%v/keepalive=%v", test.idleTimeout, test.keepAlive)
		t.Run(name, func(t *testing.T) {
			tc := newTestConn(t, serverSide, func(c *Config) {
				c.MaxIdleTimeout = test.idleTimeout
				c.KeepAlivePeriod = test.keepAlive
			})
			tc.handshake()
			if got, want := tc.timeUntilEvent(), test.wantTimeout; got != want {
				t.Errorf("new conn timeout=%v, want %v (keepalive timeout)", got, want)
			}
			tc.advance(test.wantTimeout - 1)
			tc.wantIdle("connection is idle prior to timeout")
			tc.advance(1)
			tc.wantFrameType("keep-alive ping is sent", packetType1RTT,
				debugFramePing{})
		})
	}
}

func TestIdleLongTermKeepAliveSent(t *testing.T) {
	// This test examines a connection sitting idle and sending periodic keep-alive pings.
	const keepAlivePeriod = 30 * time.Second
	tc := newTestConn(t, clientSide, func(c *Config) {
		c.KeepAlivePeriod = keepAlivePeriod
		c.MaxIdleTimeout = -1
	})
	tc.handshake()
	// The handshake will have completed a little bit after the point at which the
	// keepalive timer was set. Send two PING frames to the conn, triggering an immediate ack
	// and resetting the timer.
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.wantFrameType("conn acks received pings", packetType1RTT, debugFrameAck{})
	for i := 0; i < 10; i++ {
		tc.wantIdle("conn has nothing more to send")
		if got, want := tc.timeUntilEvent(), keepAlivePeriod; got != want {
			t.Errorf("i=%v conn timeout=%v, want %v (keepalive timeout)", i, got, want)
		}
		tc.advance(keepAlivePeriod)
		tc.wantFrameType("keep-alive ping is sent", packetType1RTT,
			debugFramePing{})
		tc.writeAckForAll()
	}
}

func TestIdleLongTermKeepAliveReceived(t *testing.T) {
	// This test examines a connection sitting idle, but receiving periodic peer
	// traffic to keep the connection alive.
	const idleTimeout = 30 * time.Second
	tc := newTestConn(t, serverSide, func(c *Config) {
		c.MaxIdleTimeout = idleTimeout
	})
	tc.handshake()
	for i := 0; i < 10; i++ {
		tc.advance(idleTimeout - 1*time.Second)
		tc.writeFrames(packetType1RTT, debugFramePing{})
		if got, want := tc.timeUntilEvent(), maxAckDelay-timerGranularity; got != want {
			t.Errorf("i=%v conn timeout=%v, want %v (max_ack_delay)", i, got, want)
		}
		tc.advanceToTimer()
		tc.wantFrameType("conn acks received ping", packetType1RTT, debugFrameAck{})
	}
	// Connection is still alive.
	ctx := canceledContext()
	if err := tc.conn.Wait(ctx); err != context.Canceled {
		t.Fatalf("conn.Wait() = %v, want Canceled", err)
	}
}
