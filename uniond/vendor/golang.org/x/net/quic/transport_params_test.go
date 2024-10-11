// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"math"
	"net/netip"
	"reflect"
	"testing"
	"time"
)

func TestTransportParametersMarshalUnmarshal(t *testing.T) {
	for _, test := range []struct {
		params func(p *transportParameters)
		enc    []byte
	}{{
		params: func(p *transportParameters) {
			p.originalDstConnID = []byte("connid")
		},
		enc: []byte{
			0x00, // original_destination_connection_id
			byte(len("connid")),
			'c', 'o', 'n', 'n', 'i', 'd',
		},
	}, {
		params: func(p *transportParameters) {
			p.maxIdleTimeout = 10 * time.Millisecond
		},
		enc: []byte{
			0x01, // max_idle_timeout
			1,    // length
			10,   // varint msecs
		},
	}, {
		params: func(p *transportParameters) {
			p.statelessResetToken = []byte("0123456789abcdef")
		},
		enc: []byte{
			0x02, // stateless_reset_token
			16,   // length
			'0', '1', '2', '3', '4', '5', '6', '7',
			'8', '9', 'a', 'b', 'c', 'd', 'e', 'f', // reset token
		},
	}, {
		params: func(p *transportParameters) {
			p.maxUDPPayloadSize = 1200
		},
		enc: []byte{
			0x03,       // max_udp_payload_size
			2,          // length
			0x44, 0xb0, // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialMaxData = 10
		},
		enc: []byte{
			0x04, // initial_max_data
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialMaxStreamDataBidiLocal = 10
		},
		enc: []byte{
			0x05, // initial_max_stream_data_bidi_local
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialMaxStreamDataBidiRemote = 10
		},
		enc: []byte{
			0x06, // initial_max_stream_data_bidi_remote
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialMaxStreamDataUni = 10
		},
		enc: []byte{
			0x07, // initial_max_stream_data_uni
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialMaxStreamsBidi = 10
		},
		enc: []byte{
			0x08, // initial_max_streams_bidi
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialMaxStreamsUni = 10
		},
		enc: []byte{
			0x09, // initial_max_streams_uni
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.ackDelayExponent = 4
		},
		enc: []byte{
			0x0a, // ack_delay_exponent
			1,    // length
			4,    // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.maxAckDelay = 10 * time.Millisecond
		},
		enc: []byte{
			0x0b, // max_ack_delay
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.disableActiveMigration = true
		},
		enc: []byte{
			0x0c, // disable_active_migration
			0,    // length
		},
	}, {
		params: func(p *transportParameters) {
			p.preferredAddrV4 = netip.MustParseAddrPort("127.0.0.1:80")
			p.preferredAddrV6 = netip.MustParseAddrPort("[fe80::1]:1024")
			p.preferredAddrConnID = []byte("connid")
			p.preferredAddrResetToken = []byte("0123456789abcdef")
		},
		enc: []byte{
			0x0d, // preferred_address
			byte(4 + 2 + 16 + 2 + 1 + len("connid") + 16), // length
			127, 0, 0, 1, // v4 address
			0, 80, // v4 port
			0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // v6 address
			0x04, 0x00, // v6 port,
			6,                            // connection id length
			'c', 'o', 'n', 'n', 'i', 'd', // connection id
			'0', '1', '2', '3', '4', '5', '6', '7',
			'8', '9', 'a', 'b', 'c', 'd', 'e', 'f', // reset token
		},
	}, {
		params: func(p *transportParameters) {
			p.activeConnIDLimit = 10
		},
		enc: []byte{
			0x0e, // active_connection_id_limit
			1,    // length
			10,   // varint value
		},
	}, {
		params: func(p *transportParameters) {
			p.initialSrcConnID = []byte("connid")
		},
		enc: []byte{
			0x0f, // initial_source_connection_id
			byte(len("connid")),
			'c', 'o', 'n', 'n', 'i', 'd',
		},
	}, {
		params: func(p *transportParameters) {
			p.retrySrcConnID = []byte("connid")
		},
		enc: []byte{
			0x10, // retry_source_connection_id
			byte(len("connid")),
			'c', 'o', 'n', 'n', 'i', 'd',
		},
	}} {
		wantParams := defaultTransportParameters()
		test.params(&wantParams)
		gotBytes := marshalTransportParameters(wantParams)
		if !bytes.Equal(gotBytes, test.enc) {
			t.Errorf("marshalTransportParameters(%#v):\n got: %x\nwant: %x", wantParams, gotBytes, test.enc)
		}
		gotParams, err := unmarshalTransportParams(test.enc)
		if err != nil {
			t.Errorf("unmarshalTransportParams(%x): unexpected error: %v", test.enc, err)
		} else if !reflect.DeepEqual(gotParams, wantParams) {
			t.Errorf("unmarshalTransportParams(%x):\n got: %#v\nwant: %#v", test.enc, gotParams, wantParams)
		}
	}
}

func TestTransportParametersErrors(t *testing.T) {
	for _, test := range []struct {
		desc string
		enc  []byte
	}{{
		desc: "invalid id",
		enc: []byte{
			0x40, // too short
		},
	}, {
		desc: "parameter too short",
		enc: []byte{
			0x00,    // original_destination_connection_id
			0x04,    // length
			1, 2, 3, // not enough data
		},
	}, {
		desc: "extra data in parameter",
		enc: []byte{
			0x01, // max_idle_timeout
			2,    // length
			10,   // varint msecs
			0,    // extra junk
		},
	}, {
		desc: "invalid varint in parameter",
		enc: []byte{
			0x01, // max_idle_timeout
			1,    // length
			0x40, // incomplete varint
		},
	}, {
		desc: "stateless_reset_token not 16 bytes",
		enc: []byte{
			0x02, // stateless_reset_token,
			15,   // length
			0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
		},
	}, {
		desc: "initial_max_streams_bidi is too large",
		enc: []byte{
			0x08, // initial_max_streams_bidi,
			8,    // length,
			0xd0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
		},
	}, {
		desc: "initial_max_streams_uni is too large",
		enc: []byte{
			0x08, // initial_max_streams_uni,
			9,    // length,
			0xd0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
		},
	}, {
		desc: "preferred_address is too short",
		enc: []byte{
			0x0d, // preferred_address
			byte(3),
			127, 0, 0,
		},
	}, {
		desc: "preferred_address reset token too short",
		enc: []byte{
			0x0d, // preferred_address
			byte(4 + 2 + 16 + 2 + 1 + len("connid") + 15), // length
			127, 0, 0, 1, // v4 address
			0, 80, // v4 port
			0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // v6 address
			0x04, 0x00, // v6 port,
			6,                            // connection id length
			'c', 'o', 'n', 'n', 'i', 'd', // connection id
			'0', '1', '2', '3', '4', '5', '6', '7',
			'8', '9', 'a', 'b', 'c', 'd', 'e', // reset token, one byte too short

		},
	}, {
		desc: "preferred_address conn id too long",
		enc: []byte{
			0x0d, // preferred_address
			byte(4 + 2 + 16 + 2 + 1 + len("connid") + 16), // length
			127, 0, 0, 1, // v4 address
			0, 80, // v4 port
			0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // v6 address
			0x04, 0x00, // v6 port,
			byte(len("connid")) + 16 + 1, // connection id length, too long
			'c', 'o', 'n', 'n', 'i', 'd', // connection id
			'0', '1', '2', '3', '4', '5', '6', '7',
			'8', '9', 'a', 'b', 'c', 'd', 'e', 'f', // reset token

		},
	}} {
		_, err := unmarshalTransportParams(test.enc)
		if err == nil {
			t.Errorf("%v:\nunmarshalTransportParams(%x): unexpectedly succeeded", test.desc, test.enc)
		}
	}
}

func TestTransportParametersRangeErrors(t *testing.T) {
	for _, test := range []struct {
		desc   string
		params func(p *transportParameters)
	}{{
		desc: "max_udp_payload_size < 1200",
		params: func(p *transportParameters) {
			p.maxUDPPayloadSize = 1199
		},
	}, {
		desc: "ack_delay_exponent > 20",
		params: func(p *transportParameters) {
			p.ackDelayExponent = 21
		},
	}, {
		desc: "max_ack_delay > 1^14 ms",
		params: func(p *transportParameters) {
			p.maxAckDelay = (1 << 14) * time.Millisecond
		},
	}, {
		desc: "active_connection_id_limit < 2",
		params: func(p *transportParameters) {
			p.activeConnIDLimit = 1
		},
	}} {
		p := defaultTransportParameters()
		test.params(&p)
		enc := marshalTransportParameters(p)
		_, err := unmarshalTransportParams(enc)
		if err == nil {
			t.Errorf("%v: unmarshalTransportParams unexpectedly succeeded", test.desc)
		}
	}
}

func TestTransportParameterMaxIdleTimeoutOverflowsDuration(t *testing.T) {
	tooManyMS := 1 + (math.MaxInt64 / uint64(time.Millisecond))

	var enc []byte
	enc = appendVarint(enc, paramMaxIdleTimeout)
	enc = appendVarint(enc, uint64(sizeVarint(tooManyMS)))
	enc = appendVarint(enc, uint64(tooManyMS))

	dec, err := unmarshalTransportParams(enc)
	if err != nil {
		t.Fatalf("unmarshalTransportParameters(enc) = %v", err)
	}
	if got, want := dec.maxIdleTimeout, time.Duration(0); got != want {
		t.Errorf("max_idle_timeout=%v, got maxIdleTimeout=%v; want %v", tooManyMS, got, want)
	}
}

func TestTransportParametersSkipUnknownParameters(t *testing.T) {
	enc := []byte{
		0x20, // unknown transport parameter
		1,    // length
		0,    // varint value

		0x04, // initial_max_data
		1,    // length
		10,   // varint value

		0x21, // unknown transport parameter
		1,    // length
		0,    // varint value
	}
	dec, err := unmarshalTransportParams(enc)
	if err != nil {
		t.Fatalf("unmarshalTransportParameters(enc) = %v", err)
	}
	if got, want := dec.initialMaxData, int64(10); got != want {
		t.Errorf("got initial_max_data=%v; want %v", got, want)
	}
}

func FuzzTransportParametersMarshalUnmarshal(f *testing.F) {
	f.Fuzz(func(t *testing.T, in []byte) {
		p1, err := unmarshalTransportParams(in)
		if err != nil {
			return
		}
		out := marshalTransportParameters(p1)
		p2, err := unmarshalTransportParams(out)
		if err != nil {
			t.Fatalf("round trip unmarshal/remarshal: unmarshal error: %v\n%x", err, in)
		}
		if !reflect.DeepEqual(p1, p2) {
			t.Fatalf("round trip unmarshal/remarshal: parameters differ:\n%x\n%#v\n%#v", in, p1, p2)
		}
	})
}
