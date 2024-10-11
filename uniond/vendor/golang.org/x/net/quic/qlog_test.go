// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"reflect"
	"testing"
	"time"

	"golang.org/x/net/quic/qlog"
)

func TestQLogHandshake(t *testing.T) {
	testSides(t, "", func(t *testing.T, side connSide) {
		qr := &qlogRecord{}
		tc := newTestConn(t, side, qr.config)
		tc.handshake()
		tc.conn.Abort(nil)
		tc.wantFrame("aborting connection generates CONN_CLOSE",
			packetType1RTT, debugFrameConnectionCloseTransport{
				code: errNo,
			})
		tc.writeFrames(packetType1RTT, debugFrameConnectionCloseTransport{})
		tc.advanceToTimer() // let the conn finish draining

		var src, dst []byte
		if side == clientSide {
			src = testLocalConnID(0)
			dst = testLocalConnID(-1)
		} else {
			src = testPeerConnID(-1)
			dst = testPeerConnID(0)
		}
		qr.wantEvents(t, jsonEvent{
			"name": "connectivity:connection_started",
			"data": map[string]any{
				"src_cid": hex.EncodeToString(src),
				"dst_cid": hex.EncodeToString(dst),
			},
		}, jsonEvent{
			"name": "connectivity:connection_closed",
			"data": map[string]any{
				"trigger": "clean",
			},
		})
	})
}

func TestQLogPacketFrames(t *testing.T) {
	qr := &qlogRecord{}
	tc := newTestConn(t, clientSide, qr.config)
	tc.handshake()
	tc.conn.Abort(nil)
	tc.writeFrames(packetType1RTT, debugFrameConnectionCloseTransport{})
	tc.advanceToTimer() // let the conn finish draining

	qr.wantEvents(t, jsonEvent{
		"name": "transport:packet_sent",
		"data": map[string]any{
			"header": map[string]any{
				"packet_type":   "initial",
				"packet_number": 0,
				"dcid":          hex.EncodeToString(testLocalConnID(-1)),
				"scid":          hex.EncodeToString(testLocalConnID(0)),
			},
			"frames": []any{
				map[string]any{"frame_type": "crypto"},
			},
		},
	}, jsonEvent{
		"name": "transport:packet_received",
		"data": map[string]any{
			"header": map[string]any{
				"packet_type":   "initial",
				"packet_number": 0,
				"dcid":          hex.EncodeToString(testLocalConnID(0)),
				"scid":          hex.EncodeToString(testPeerConnID(0)),
			},
			"frames": []any{map[string]any{"frame_type": "crypto"}},
		},
	})
}

func TestQLogConnectionClosedTrigger(t *testing.T) {
	for _, test := range []struct {
		trigger  string
		connOpts []any
		f        func(*testConn)
	}{{
		trigger: "clean",
		f: func(tc *testConn) {
			tc.handshake()
			tc.conn.Abort(nil)
		},
	}, {
		trigger: "handshake_timeout",
		connOpts: []any{
			func(c *Config) {
				c.HandshakeTimeout = 5 * time.Second
			},
		},
		f: func(tc *testConn) {
			tc.ignoreFrame(frameTypeCrypto)
			tc.ignoreFrame(frameTypeAck)
			tc.ignoreFrame(frameTypePing)
			tc.advance(5 * time.Second)
		},
	}, {
		trigger: "idle_timeout",
		connOpts: []any{
			func(c *Config) {
				c.MaxIdleTimeout = 5 * time.Second
			},
		},
		f: func(tc *testConn) {
			tc.handshake()
			tc.advance(5 * time.Second)
		},
	}, {
		trigger: "error",
		f: func(tc *testConn) {
			tc.handshake()
			tc.writeFrames(packetType1RTT, debugFrameConnectionCloseTransport{
				code: errProtocolViolation,
			})
			tc.conn.Abort(nil)
		},
	}} {
		t.Run(test.trigger, func(t *testing.T) {
			qr := &qlogRecord{}
			tc := newTestConn(t, clientSide, append(test.connOpts, qr.config)...)
			test.f(tc)
			fr, ptype := tc.readFrame()
			switch fr := fr.(type) {
			case debugFrameConnectionCloseTransport:
				tc.writeFrames(ptype, fr)
			case nil:
			default:
				t.Fatalf("unexpected frame: %v", fr)
			}
			tc.wantIdle("connection should be idle while closing")
			tc.advance(5 * time.Second) // long enough for the drain timer to expire
			qr.wantEvents(t, jsonEvent{
				"name": "connectivity:connection_closed",
				"data": map[string]any{
					"trigger": test.trigger,
				},
			})
		})
	}
}

func TestQLogRecovery(t *testing.T) {
	qr := &qlogRecord{}
	tc, s := newTestConnAndLocalStream(t, clientSide, uniStream,
		permissiveTransportParameters, qr.config)

	// Ignore events from the handshake.
	qr.ev = nil

	data := make([]byte, 16)
	s.Write(data)
	s.CloseWrite()
	tc.wantFrame("created stream 0",
		packetType1RTT, debugFrameStream{
			id:   newStreamID(clientSide, uniStream, 0),
			fin:  true,
			data: data,
		})
	tc.writeAckForAll()
	tc.wantIdle("connection should be idle now")

	// Don't check the contents of fields, but verify that recovery metrics are logged.
	qr.wantEvents(t, jsonEvent{
		"name": "recovery:metrics_updated",
		"data": map[string]any{
			"bytes_in_flight": nil,
		},
	}, jsonEvent{
		"name": "recovery:metrics_updated",
		"data": map[string]any{
			"bytes_in_flight":   0,
			"congestion_window": nil,
			"latest_rtt":        nil,
			"min_rtt":           nil,
			"rtt_variance":      nil,
			"smoothed_rtt":      nil,
		},
	})
}

func TestQLogLoss(t *testing.T) {
	qr := &qlogRecord{}
	tc, s := newTestConnAndLocalStream(t, clientSide, uniStream,
		permissiveTransportParameters, qr.config)

	// Ignore events from the handshake.
	qr.ev = nil

	data := make([]byte, 16)
	s.Write(data)
	s.CloseWrite()
	tc.wantFrame("created stream 0",
		packetType1RTT, debugFrameStream{
			id:   newStreamID(clientSide, uniStream, 0),
			fin:  true,
			data: data,
		})

	const pto = false
	tc.triggerLossOrPTO(packetType1RTT, pto)

	qr.wantEvents(t, jsonEvent{
		"name": "recovery:packet_lost",
		"data": map[string]any{
			"header": map[string]any{
				"packet_number": nil,
				"packet_type":   "1RTT",
			},
		},
	})
}

func TestQLogPacketDropped(t *testing.T) {
	qr := &qlogRecord{}
	tc := newTestConn(t, clientSide, permissiveTransportParameters, qr.config)
	tc.handshake()

	// A garbage-filled datagram with a DCID matching this connection.
	dgram := bytes.Join([][]byte{
		{headerFormShort | fixedBit},
		testLocalConnID(0),
		make([]byte, 100),
		[]byte{1, 2, 3, 4}, // random data, to avoid this looking like a stateless reset
	}, nil)
	tc.endpoint.write(&datagram{
		b: dgram,
	})

	qr.wantEvents(t, jsonEvent{
		"name": "connectivity:packet_dropped",
	})
}

type nopCloseWriter struct {
	io.Writer
}

func (nopCloseWriter) Close() error { return nil }

type jsonEvent map[string]any

func (j jsonEvent) String() string {
	b, _ := json.MarshalIndent(j, "", "  ")
	return string(b)
}

// jsonPartialEqual compares two JSON structures.
// It ignores fields not set in want (see below for specifics).
func jsonPartialEqual(got, want any) (equal bool) {
	cmpval := func(v any) any {
		// Map certain types to a common representation.
		switch v := v.(type) {
		case int:
			// JSON uses float64s rather than ints for numbers.
			// Map int->float64 so we can use integers in expectations.
			return float64(v)
		case jsonEvent:
			return (map[string]any)(v)
		case []jsonEvent:
			s := []any{}
			for _, e := range v {
				s = append(s, e)
			}
			return s
		}
		return v
	}
	if want == nil {
		return true // match anything
	}
	got = cmpval(got)
	want = cmpval(want)
	if reflect.TypeOf(got) != reflect.TypeOf(want) {
		return false
	}
	switch w := want.(type) {
	case map[string]any:
		// JSON object: Every field in want must match a field in got.
		g := got.(map[string]any)
		for k := range w {
			if !jsonPartialEqual(g[k], w[k]) {
				return false
			}
		}
	case []any:
		// JSON slice: Every field in want must match a field in got, in order.
		// So want=[2,4] matches got=[1,2,3,4] but not [4,2].
		g := got.([]any)
		for _, ge := range g {
			if jsonPartialEqual(ge, w[0]) {
				w = w[1:]
				if len(w) == 0 {
					return true
				}
			}
		}
		return false
	default:
		if !reflect.DeepEqual(got, want) {
			return false
		}
	}
	return true
}

// A qlogRecord records events.
type qlogRecord struct {
	ev []jsonEvent
}

func (q *qlogRecord) Write(b []byte) (int, error) {
	// This relies on the property that the Handler always makes one Write call per event.
	if len(b) < 1 || b[0] != 0x1e {
		panic(fmt.Errorf("trace Write should start with record separator, got %q", string(b)))
	}
	var val map[string]any
	if err := json.Unmarshal(b[1:], &val); err != nil {
		panic(fmt.Errorf("log unmarshal failure: %v\n%v", err, string(b)))
	}
	q.ev = append(q.ev, val)
	return len(b), nil
}

func (q *qlogRecord) Close() error { return nil }

// config may be passed to newTestConn to configure the conn to use this logger.
func (q *qlogRecord) config(c *Config) {
	c.QLogLogger = slog.New(qlog.NewJSONHandler(qlog.HandlerOptions{
		Level: QLogLevelFrame,
		NewTrace: func(info qlog.TraceInfo) (io.WriteCloser, error) {
			return q, nil
		},
	}))
}

// wantEvents checks that every event in want occurs in the order specified.
func (q *qlogRecord) wantEvents(t *testing.T, want ...jsonEvent) {
	t.Helper()
	got := q.ev
	if !jsonPartialEqual(got, want) {
		t.Fatalf("got events:\n%v\n\nwant events:\n%v", got, want)
	}
}
