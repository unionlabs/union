/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package record

import (
	"bytes"
	"errors"
	"net"
	"reflect"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/testing/protocmp"

	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	"github.com/google/s2a-go/internal/record/internal/aeadcrypter/testutil"
)

var errFakeConnEOF = errors.New("fakeConn is out of bounds")

// fakeConn is a fake implementation of the net.Conn interface used for testing.
type fakeConn struct {
	net.Conn
	// bufCount tracks the current index of the buf.
	bufCount int
	buf      [][]byte
	// additionalBuf is used to store records sent through the fakeConn. It is
	// used in record write tests and TestConnReadKeyUpdates.
	additionalBuf [][]byte
	closed        bool
}

// Read returns part of the `in` buffer in sequential order each time it is
// called.
func (c *fakeConn) Read(b []byte) (n int, err error) {
	if c.bufCount >= len(c.buf) {
		return 0, errFakeConnEOF
	}
	n = copy(b, c.buf[c.bufCount])
	if n < len(c.buf[c.bufCount]) {
		c.buf[c.bufCount] = c.buf[c.bufCount][n:]
	} else {
		c.bufCount++
	}
	return n, nil
}

// Write copies the given buffer b, stores it in the `out` buffer, and returns
// the number of bytes copied.
func (c *fakeConn) Write(b []byte) (n int, err error) {
	buf := make([]byte, len(b))
	n = copy(buf, b)
	c.additionalBuf = append(c.additionalBuf, buf)
	return n, nil
}

func (c *fakeConn) Close() error {
	c.closed = true
	return nil
}

type fakeTicketSender struct {
	sessionTickets [][]byte
}

func (f *fakeTicketSender) sendTicketsToS2A(sessionTickets [][]byte, callComplete chan bool) {
	f.sessionTickets = sessionTickets
	go func() {
		callComplete <- true
		close(callComplete)
	}()
}

func TestNewS2ARecordConn(t *testing.T) {
	for _, tc := range []struct {
		desc                     string
		options                  *ConnParameters
		outUnusedBytesBuf        []byte
		outOverheadSize          int
		outHandshakerServiceAddr string
		outConnectionID          uint64
		outLocalIdentity         *commonpb.Identity
		outErr                   bool
	}{
		{
			desc:   "nil conn options",
			outErr: true,
		},
		{
			desc: "invalid input traffic secret size",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				HSAddr:           "test handshaker address",
			},
			outErr: true,
		},
		{
			desc: "invalid output traffic secret size",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				HSAddr:           "test handshaker address",
			},
			outErr: true,
		},
		{
			desc: "invalid tls version",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
				TLSVersion:       commonpb.TLSVersion_TLS1_2,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				HSAddr:           "test handshaker address",
			},
			outErr: true,
		},
		{
			desc: "basic with AES-128-GCM-SHA256",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				HSAddr:           "test handshaker address",
				ConnectionID:     1,
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "test_spiffe_id",
					},
				},
			},
			// outOverheadSize = header size (5) + record type byte (1) +
			// tag size (16).
			outOverheadSize:          22,
			outHandshakerServiceAddr: "test handshaker address",
			outConnectionID:          1,
			outLocalIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "test_spiffe_id",
				},
			},
		},
		{
			desc: "basic with AES-256-GCM-SHA384",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				HSAddr:           "test handshaker address",
				ConnectionID:     1,
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "test_spiffe_id",
					},
				},
			},
			// outOverheadSize = header size (5) + record type byte (1) +
			// tag size (16).
			outOverheadSize:          22,
			outHandshakerServiceAddr: "test handshaker address",
			outConnectionID:          1,
			outLocalIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "test_spiffe_id",
				},
			},
		},
		{
			desc: "basic with CHACHA20-POLY1305-SHA256",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				HSAddr:           "test handshaker address",
				ConnectionID:     1,
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "test_spiffe_id",
					},
				},
			},
			// outOverheadSize = header size (5) + record type byte (1) +
			// tag size (16).
			outOverheadSize:          22,
			outHandshakerServiceAddr: "test handshaker address",
			outConnectionID:          1,
			outLocalIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "test_spiffe_id",
				},
			},
		},
		{
			desc: "basic with unusedBytes",
			options: &ConnParameters{
				NetConn:          &fakeConn{},
				Ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				OutTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
				UnusedBuf:        testutil.Dehex("ffffffff"),
				HSAddr:           "test handshaker address",
			},
			outUnusedBytesBuf: testutil.Dehex("ffffffff"),
			// outOverheadSize = header size (5) + record type byte (1) +
			// tag size (16).
			outOverheadSize:          22,
			outHandshakerServiceAddr: "test handshaker address",
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			NetConn, err := NewConn(tc.options)
			if got, want := err == nil, !tc.outErr; got != want {
				t.Errorf("NewConn(%v) = (err=nil) = %v, want %v", *tc.options, got, want)
			}
			if err != nil {
				return
			}
			conn := NetConn.(*conn)
			if got, want := conn.unusedBuf, tc.outUnusedBytesBuf; !bytes.Equal(got, want) {
				t.Errorf("conn.unusedBytes = %v, want %v", got, want)
			}
			if got, want := conn.overheadSize, tc.outOverheadSize; got != want {
				t.Errorf("conn.overheadSize = %v, want %v", got, want)
			}
			ticketSender := conn.ticketSender.(*ticketSender)
			if got, want := ticketSender.hsAddr, tc.outHandshakerServiceAddr; got != want {
				t.Errorf("ticketSender.hsAddr = %v, want %v", got, want)
			}
			if got, want := ticketSender.connectionID, tc.outConnectionID; got != want {
				t.Errorf("ticketSender.connectionID = %v, want %v", got, want)
			}
			if got, want := ticketSender.localIdentity, tc.outLocalIdentity; !cmp.Equal(got, want, protocmp.Transform()) {
				t.Errorf("ticketSender.localIdentity = %v, want %v", got, want)
			}
		})
	}
}

func TestStripPaddingAndType(t *testing.T) {
	for _, tc := range []struct {
		desc                                              string
		pendingApplicationData, outPendingApplicationData []byte
		outContentType                                    recordType
	}{
		{
			desc:                   "no padding",
			pendingApplicationData: []byte{byte(alert)},
			outContentType:         alert,
		},
		{
			desc:                   "single padding",
			pendingApplicationData: []byte{byte(applicationData), 0x00},
			outContentType:         applicationData,
		},
		{
			desc:                   "multi padding",
			pendingApplicationData: []byte{byte(handshake), 0x00, 0x00},
			outContentType:         handshake,
		},
		{
			desc:                      "app data with no padding",
			pendingApplicationData:    []byte{0xff, byte(handshake)},
			outPendingApplicationData: []byte{0xff},
			outContentType:            handshake,
		},
		{
			desc:                      "app data with padding",
			pendingApplicationData:    []byte{0xff, byte(handshake), 0x00},
			outPendingApplicationData: []byte{0xff},
			outContentType:            handshake,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c := conn{pendingApplicationData: tc.pendingApplicationData}
			ct, err := c.stripPaddingAndType()
			if err != nil {
				t.Errorf("c.stripPaddingAndType() failed: %v", err)
			}
			if got, want := c.pendingApplicationData, tc.outPendingApplicationData; !bytes.Equal(got, want) {
				t.Errorf("c.pendingApplicationData = %v, want %v", got, want)
			}
			if got, want := ct, tc.outContentType; got != want {
				t.Errorf("ct = %v, want %v", got, want)
			}
		})
	}
}

func TestParseRecord(t *testing.T) {
	for _, tc := range []struct {
		desc                             string
		b                                []byte
		maxLen                           uint16
		outCompletedRecord, outRemaining []byte
		outErr                           bool
	}{
		{
			desc:         "buffer smaller than header size",
			b:            make([]byte, 1),
			outRemaining: make([]byte, 1),
		},
		{
			desc:         "header payload size larger than maxLen",
			b:            testutil.Dehex("000000ffff"),
			maxLen:       1,
			outRemaining: testutil.Dehex("000000ffff"),
			outErr:       true,
		},
		{
			desc:               "header payload size same as maxLen",
			b:                  testutil.Dehex("0000000003ffffff"),
			maxLen:             3,
			outCompletedRecord: testutil.Dehex("0000000003ffffff"),
		},
		{
			desc:         "incomplete record",
			b:            testutil.Dehex("0000000001"),
			maxLen:       10,
			outRemaining: testutil.Dehex("0000000001"),
		},
		{
			desc:               "complete record",
			b:                  testutil.Dehex("0000000001ff"),
			maxLen:             10,
			outCompletedRecord: testutil.Dehex("0000000001ff"),
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			completedRecord, remaining, err := parseReadBuffer(tc.b, tc.maxLen)
			if got, want := err == nil, !tc.outErr; got != want {
				t.Errorf("parseReadBuffer(%v, %v) = (err=nil) = %v, want %v", tc.b, tc.maxLen, got, want)
			}
			if err != nil {
				return
			}
			if got, want := completedRecord, tc.outCompletedRecord; !bytes.Equal(got, want) {
				t.Errorf("completedRecord = %v, want %v", got, want)
			}
			if got, want := remaining, tc.outRemaining; !bytes.Equal(got, want) {
				t.Errorf("remaining = %v, want %v", got, want)
			}
		})
	}
}

func TestReadCompletedRecord(t *testing.T) {
	for _, tc := range []struct {
		desc                  string
		connBufs              [][]byte
		nextRecord, unusedBuf []byte
		outCompletedRecords   [][]byte
		outErr                bool
	}{
		{
			desc:       "invalid record header size",
			nextRecord: testutil.Dehex("170303ffff"),
			outErr:     true,
		},
		{
			desc: "complete record in single read",
			connBufs: [][]byte{
				testutil.Dehex("1703030001ff"),
			},
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030001ff"),
			},
		},
		{
			desc:       "complete record in single read from leftover buffer",
			nextRecord: testutil.Dehex("1703030001ff"),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030001ff"),
			},
		},
		{
			desc: "complete record split in header",
			connBufs: [][]byte{
				testutil.Dehex("170303"),
				testutil.Dehex("0001ff"),
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030001ff"),
			},
		},
		{
			desc: "complete record split in ciphertext",
			connBufs: [][]byte{
				testutil.Dehex("1703030002ff"),
				testutil.Dehex("ff"),
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030002ffff"),
			},
		},
		{
			desc: "two complete records split in header",
			connBufs: [][]byte{
				testutil.Dehex("170303"),
				testutil.Dehex("0002ffff1703030001ff"),
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030002ffff"),
				testutil.Dehex("1703030001ff"),
			},
		},
		{
			desc: "two complete records split in second header",
			connBufs: [][]byte{
				testutil.Dehex("1703030002ffff1703"),
				testutil.Dehex("030001ff"),
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030002ffff"),
				testutil.Dehex("1703030001ff"),
			},
		},
		{
			desc: "two complete records split in ciphertext",
			connBufs: [][]byte{
				testutil.Dehex("1703030002ff"),
				testutil.Dehex("ff1703030001ff"),
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030002ffff"),
				testutil.Dehex("1703030001ff"),
			},
		},
		{
			desc: "two complete records split in second ciphertext",
			connBufs: [][]byte{
				testutil.Dehex("1703030002ffff1703030002ff"),
				testutil.Dehex("ff"),
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030002ffff"),
				testutil.Dehex("1703030002ffff"),
			},
		},
		{
			desc: "complete record split by each byte",
			connBufs: [][]byte{
				{0x17}, {0x03}, {0x03}, {0x00}, {0x01}, {0xff},
			},
			unusedBuf: make([]byte, tlsRecordMaxPlaintextSize),
			outCompletedRecords: [][]byte{
				testutil.Dehex("1703030001ff"),
			},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fConn := &fakeConn{buf: tc.connBufs}
			c := &conn{Conn: fConn, nextRecord: tc.nextRecord, unusedBuf: tc.unusedBuf}
			for _, outCompletedRecord := range tc.outCompletedRecords {
				completedRecord, err := c.readFullRecord()
				if got, want := err == nil, !tc.outErr; got != want {
					t.Errorf("c.readCompletecRecord() = (err=nil) = %v, want %v", got, want)
				}
				if err != nil {
					return
				}
				if got, want := completedRecord, outCompletedRecord; !bytes.Equal(got, want) {
					t.Errorf("c.readFullRecord() = %v, want %v", got, want)
				}
			}
		})
	}
}

func TestSplitAndValidateHeader(t *testing.T) {
	for _, tc := range []struct {
		desc                     string
		completedRecord          []byte
		outHeader, outCiphertext []byte
		outErr                   bool
	}{
		{
			desc:            "invalid header type",
			completedRecord: make([]byte, tlsRecordHeaderSize),
			outErr:          true,
		},
		{
			desc:            "invalid legacy record version",
			completedRecord: []byte{byte(tlsApplicationData), 0x00, 0x00, 0x00, 0x00},
			outErr:          true,
		},
		{
			desc:            "basic with no ciphertext",
			completedRecord: []byte{byte(tlsApplicationData), 0x03, 0x03, 0x00, 0x00},
			outHeader:       []byte{byte(tlsApplicationData), 0x03, 0x03, 0x00, 0x00},
		},
		{
			desc:            "basic with ciphertext",
			completedRecord: []byte{byte(tlsApplicationData), 0x03, 0x03, 0x00, 0x01, 0xff},
			outHeader:       []byte{byte(tlsApplicationData), 0x03, 0x03, 0x00, 0x01},
			outCiphertext:   []byte{0xff},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			header, ciphertext, err := splitAndValidateHeader(tc.completedRecord)
			if got, want := err == nil, !tc.outErr; got != want {
				t.Errorf("splitAndValidateHeader(%v) = (err=nil) = %v, want %v", tc.completedRecord, got, want)
			}
			if err != nil {
				return
			}
			if got, want := header, tc.outHeader; !bytes.Equal(got, want) {
				t.Errorf("header = %v, want %v", got, want)
			}
			if got, want := ciphertext, tc.outCiphertext; !bytes.Equal(got, want) {
				t.Errorf("ciphertext = %v, want %v", got, want)
			}
		})
	}
}

func TestConnReadApplicationData(t *testing.T) {
	for _, tc := range []struct {
		desc             string
		ciphersuite      commonpb.Ciphersuite
		trafficSecret    []byte
		completedRecords [][]byte
		outPlaintexts    [][]byte
		outErr           bool
	}{
		// The traffic secrets were chosen randomly and are equivalent to the
		// ones used in C++ and Java. The ciphertext was constructed using an
		// existing TLS library.
		{
			desc:          "AES-128-GCM-SHA256 with no padding",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760e4e3f074a36574c45ee4c1906103db0d"),
				testutil.Dehex("170303001ad7853afd6d7ceaabab950a0b6707905d2b908894871c7c62021f"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 with padding",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030021f2e4e411ac6760e84726e4886d7432e39b34f0fccfc1f4558303c68a19535c0ff5"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 empty",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030011d47cb2ec040f26cc8989330339c669dd4e"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 with no padding",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001724efee5af1a62170ad5a95f899d038b965386a1a7daed9"),
				testutil.Dehex("170303001a832a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 with padding",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303002124efee5af1a621e8a4d1f269930e7835cfdd05e2d0bec5b01a67decfa6372c2af7"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 empty",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001102a04134d38c1118f36b01d177c5d2dcf7"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 with no padding",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017c947ffa470304370338bb07ce468e6b8a0944a338ba402"),
				testutil.Dehex("170303001a0cedeb922170c110c172262542c67916b78fa0d1c1261709cd00"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 with padding",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030021c947ffa4703043f063e7b6a0519fbd0956cf3a7c9730c13597eec17ec7e700f140"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 empty",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030011ef8f7a428ddc84ee5968cd6306bf1d2d1b"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 split in first record",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760"),
				testutil.Dehex("e4e3f074a36574c45ee4c1906103db0d"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 split in first record",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001724efee5af1a6"),
				testutil.Dehex("2170ad5a95f899d038b965386a1a7daed9"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 split in first record",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017c947ffa470"),
				testutil.Dehex("304370338bb07ce468e6b8a0944a338ba402"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 split in first record header",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("17"),
				testutil.Dehex("03030017f2e4e411ac6760e4e3f074a36574c45ee4c1906103db0d"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 split in first record header",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("17"),
				testutil.Dehex("0303001724efee5af1a62170ad5a95f899d038b965386a1a7daed9"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 split in first record header",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("17"),
				testutil.Dehex("03030017c947ffa470304370338bb07ce468e6b8a0944a338ba402"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 split in second record",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760e4e3f074a36574c45ee4c1906103db0d170303001ad7"),
				testutil.Dehex("853afd6d7ceaabab950a0b6707905d2b908894871c7c62021f"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 split in second record",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001724efee5af1a62170ad5a95f899d038b965386a1a7daed9170303001a83"),
				testutil.Dehex("2a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 split in second record",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017c947ffa470304370338bb07ce468e6b8a0944a338ba402170303001a0c"),
				testutil.Dehex("edeb922170c110c172262542c67916b78fa0d1c1261709cd00"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 split in second record header",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760e4e3f074a36574c45ee4c1906103db0d17"),
				testutil.Dehex("0303001ad7853afd6d7ceaabab950a0b6707905d2b908894871c7c62021f"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 split in second record header",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001724efee5af1a62170ad5a95f899d038b965386a1a7daed917"),
				testutil.Dehex("0303001a832a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 split in second record header",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017c947ffa470304370338bb07ce468e6b8a0944a338ba40217"),
				testutil.Dehex("0303001a0cedeb922170c110c172262542c67916b78fa0d1c1261709cd00"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 split randomly",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760e4"),
				testutil.Dehex("e3f074a36574c45ee4c1906103db0d17"),
				testutil.Dehex("0303001ad7853afd6d7ceaab"),
				testutil.Dehex("ab950a0b6707905d2b908894871c7c62021f"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 split randomly",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001724efee"),
				testutil.Dehex("5af1a62170ad5a95f899d038b965386a1a7daed917"),
				testutil.Dehex("03"),
				testutil.Dehex("03001a832a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 split randomly",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("17"),
				testutil.Dehex("03030017c947ffa470304370338bb07ce468e6b8a0944a338ba40217"),
				testutil.Dehex("0303001a0cedeb922170"),
				testutil.Dehex("c110c172262542c67916b78fa0d1c1261709cd00"),
			},
			outPlaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c, err := NewConn(&ConnParameters{
				NetConn:          &fakeConn{buf: tc.completedRecords},
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			for _, outPlaintext := range tc.outPlaintexts {
				plaintext := make([]byte, tlsRecordMaxPlaintextSize)
				n, err := c.Read(plaintext)
				if got, want := err == nil, !tc.outErr; got != want {
					t.Errorf("c.Read(plaintext) = (err=nil) = %v, want %v", got, want)
				}
				if err != nil {
					return
				}
				plaintext = plaintext[:n]
				if got, want := plaintext, outPlaintext; !bytes.Equal(got, want) {
					t.Errorf("c.Read(plaintext) = %v, want %v", got, want)
				}
			}
		})
	}
}

func TestConnReadAlert(t *testing.T) {
	for _, tc := range []struct {
		desc            string
		ciphersuite     commonpb.Ciphersuite
		trafficSecret   []byte
		completedRecord []byte
	}{
		// The records below are TLS 1.3 records that hold the ciphertext
		// obtained by encrypting (with or without padding) the close notify
		// alert {0x01, 0x00} using the keys derived from the given traffic
		// secrets and the sequence number zero.
		{
			desc:            "AES-128-GCM-SHA256 with no padding",
			ciphersuite:     commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("1703030013c2d6c245fb80969de1dd9d14499261b67735b0"),
		},
		{
			desc:            "AES-128-GCM-SHA256 with padding",
			ciphersuite:     commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("170303001dc2d6c225995177e84726e4886d5ea79383e5d529cd8339fbbfcafe2418"),
		},
		{
			desc:            "AES-256-GCM-SHA384 with no padding",
			ciphersuite:     commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("170303001314ddc8f3b3856660bb5ac81533c157582f8b4c"),
		},
		{
			desc:            "AES-256-GCM-SHA384 with padding",
			ciphersuite:     commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("170303001d14ddc86ec49036e8a4d1f269933545f03b0fe9ffd8b02acd1e41f7139e"),
		},
		{
			desc:            "CHACHA20-POLY1305-SHA256 with no padding",
			ciphersuite:     commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("1703030013f975d9cb2f116d85d4e3859f5288a9b013d778"),
		},
		{
			desc:            "CHACHA20-POLY1305-SHA256 with padding",
			ciphersuite:     commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("170303001df975d990450654f063e7b6a0514c2714c9827e796071389802f451585a"),
		},
		// The records below are TLS 1.3 records that hold the ciphertext
		// obtained by encrypting the alert {0x01, 0x2c} using the keys derived
		// from the given traffic secrets and the sequence number zero.
		{
			desc:            "AES-128-GCM-SHA256 other alert",
			ciphersuite:     commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("1703030013c2fac23f995cbe79a8d1e4c8f0353afefeaac9"),
		},
		{
			desc:            "AES-256-GCM-SHA384 other alert",
			ciphersuite:     commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("170303001314f1c80add85193c9598219ae9dc26f2479ccf"),
		},
		{
			desc:            "CHACHA20-POLY1305-SHA256 other alert",
			ciphersuite:     commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("1703030013f959d96fed92bdc7e85e04e86c19eaf154b052"),
		},
		// The records below are TLS 1.3 records that hold the ciphertext
		// obtained by encrypting the message {0x01} using the keys derived
		// from the given traffic secrets and the sequence number zero. The
		// first byte of this message indicates that it should be an alert
		// message, but the length of the message is too small.
		{
			desc:            "AES-128-GCM-SHA256 invalid",
			ciphersuite:     commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("1703030012c2c351fc48d9ac84fa165adcc9a26ffbc3c7"),
		},
		{
			desc:            "AES-256-GCM-SHA384 invalid",
			ciphersuite:     commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("170303001214c8476102a460b5cf9e9ba59e1726215ca9"),
		},
		{
			desc:            "CHACHA20-POLY1305-SHA256 invalid",
			ciphersuite:     commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecord: testutil.Dehex("1703030012f9606a83ac17b165a51f3fe764da8560c706"),
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			f := &fakeConn{buf: [][]byte{tc.completedRecord}}
			c, err := NewConn(&ConnParameters{
				NetConn:          f,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			plaintext := make([]byte, tlsRecordMaxPlaintextSize)
			_, err = c.Read(plaintext)
			if got, want := err == nil, false; got != want {
				t.Errorf("c.Read(plaintext) = (err=nil) = %v, want %v", got, want)
			}
		})
	}
}

func TestConnReadKeyUpdate(t *testing.T) {
	for _, tc := range []struct {
		desc             string
		ciphersuite      commonpb.Ciphersuite
		trafficSecret    []byte
		completedRecords [][]byte
		plaintexts       [][]byte
		outPlaintexts    [][]byte
		outWriteBuf      [][]byte
	}{
		{
			desc:          "AES-128-GCM-SHA256",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030020dbd6d724994777e84726e4886d7432e311a73b42d0073f28ea60e30e8eb498fd"),
				testutil.Dehex("1703030017dd99ebef48292cd4c372a000740372d2ae9aad31cfd274"),
			},
			plaintexts: [][]byte{
				[]byte("123456"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("123456"),
			},
			outWriteBuf: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760e4e3f074a36574c45ee4c1906103db0d"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("17030300200ddddd6fc48636e8a4d1f269930e7835adc07e732ba7fd617ff9a65a51c36b6d"),
				testutil.Dehex("17030300179cd5972e76baf56af644c92235460301c0a013ad35be00"),
			},
			plaintexts: [][]byte{
				[]byte("123456"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("123456"),
			},
			outWriteBuf: [][]byte{
				testutil.Dehex("170303001724efee5af1a62170ad5a95f899d038b965386a1a7daed9"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030020e075cc91451054f063e7b6a0519fbd098e83bda4b515bea5196cccc008556ad0"),
				testutil.Dehex("1703030017c4e48ccaf036bd9bc146bbc6192404f9a2d2da5d1afe78"),
			},
			plaintexts: [][]byte{
				[]byte("123456"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("123456"),
			},
			outWriteBuf: [][]byte{
				testutil.Dehex("1703030017c947ffa470304370338bb07ce468e6b8a0944a338ba402"),
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 send key update request",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016dbd6d724984756cc7bd50502b024f94b489f1a943df5"),
				testutil.Dehex("1703030017dd99ebef48292cd4c372a000740372d2ae9aad31cfd274"),
			},
			plaintexts: [][]byte{
				[]byte("123456"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("123456"),
			},
			outWriteBuf: [][]byte{
				testutil.Dehex("1703030016dbd6d7249947cdda08655a3c2622891c1758fb0c0d0e"),
				testutil.Dehex("1703030017dd99ebef48292cd4c372a000740372d2ae9aad31cfd274"),
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 send key update request",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("17030300160ddddd6fc5862e8275a95a35a9a43502a1da78f8a416"),
				testutil.Dehex("17030300179cd5972e76baf56af644c92235460301c0a013ad35be00"),
			},
			plaintexts: [][]byte{
				[]byte("123456"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("123456"),
			},
			outWriteBuf: [][]byte{
				testutil.Dehex("17030300160ddddd6fc486a82b55d4457e3dc846f77b6d3f24de45"),
				testutil.Dehex("17030300179cd5972e76baf56af644c92235460301c0a013ad35be00"),
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 send key update request",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016e075cc914410ffeb7e4d960f8e3b1e830c137cc69692"),
				testutil.Dehex("1703030017c4e48ccaf036bd9bc146bbc6192404f9a2d2da5d1afe78"),
			},
			plaintexts: [][]byte{
				[]byte("123456"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("123456"),
			},
			outWriteBuf: [][]byte{
				testutil.Dehex("1703030016e075cc914510bc0dbc87cfc12f394b235147042f42d0"),
				testutil.Dehex("1703030017c4e48ccaf036bd9bc146bbc6192404f9a2d2da5d1afe78"),
			},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fConn := &fakeConn{buf: tc.completedRecords}
			c, err := NewConn(&ConnParameters{
				NetConn:          fConn,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			for _, outPlaintext := range tc.outPlaintexts {
				plaintext := make([]byte, tlsRecordMaxPlaintextSize)
				n, err := c.Read(plaintext)
				if err != nil {
					t.Fatalf("c.Read(plaintext) failed: %v", err)
				}
				plaintext = plaintext[:n]
				if got, want := plaintext, outPlaintext; !bytes.Equal(got, want) {
					t.Errorf("c.Read(plaintext) = %v, want %v", got, want)
				}
			}
			// Check that the outbound traffic secret was updated properly. This
			// is done by writing plaintexts after the key update and verifying
			// the output.
			for _, plaintext := range tc.plaintexts {
				n, err := c.Write(plaintext)
				if err != nil {
					t.Fatalf("c.Write(plaintext) failed: %v", err)
				}
				if got, want := n, len(plaintext); got != want {
					t.Fatalf("c.Write(plaintext) = %v, want %v", got, want)
				}
			}
			if got, want := fConn.additionalBuf, tc.outWriteBuf; !cmp.Equal(fConn.additionalBuf, tc.outWriteBuf) {
				t.Errorf("fConn.additionalBuf = %x, want %x", got, want)
			}
			if got, want := len(c.(*conn).pendingApplicationData), 0; got != want {
				t.Errorf("len(c.(*conn).pendingApplicationData) = %v, want %v", got, want)
			}
			// If a key update message is produced on request, verify that it
			// can be decrypted properly by a new Conn object. Also, verify that
			// messages written by the original Conn object after the key update
			// can be decrypted properly by the new Conn object.
			fConn2 := &fakeConn{buf: tc.outWriteBuf}
			c2, err := NewConn(&ConnParameters{
				NetConn:          fConn2,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			for range tc.outWriteBuf {
				plaintext := make([]byte, tlsRecordMaxPlaintextSize)
				_, err := c2.Read(plaintext)
				if err != nil {
					t.Fatalf("c.Read(plaintext) failed: %v", err)
				}
			}
		})
	}
}

// buildSessionTicket builds a new session ticket with the given bytes.
func buildSessionTicket(msg []byte) []byte {
	b := make([]byte, tlsHandshakePrefixSize+len(msg))
	b[0] = tlsHandshakeNewSessionTicketType
	v := len(msg)
	b[1] = byte(v >> 16)
	b[2] = byte(v >> 8)
	b[3] = byte(v)
	copy(b[4:], msg)
	return b
}

func TestConnNewSessionTicket(t *testing.T) {
	emptyTicket := []byte{4, 0, 0, 1, 0} // type(1) + length(3) + ticket(1)
	for _, tc := range []struct {
		desc              string
		ciphersuite       commonpb.Ciphersuite
		trafficSecret     []byte
		completedRecords  [][]byte
		outPlaintexts     [][]byte
		finalTicketState  sessionTicketState
		outSessionTickets [][]byte
		ticketsSent       bool
	}{
		// All the session tickets below are []byte{0}. This is not a valid
		// ticket, but is sufficient for testing since the client does not
		// care about the actual value of the ticket.
		{
			desc:          "AES-128-GCM-SHA256 new session ticket",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016c7d6d72499478b3d80281cae5b7c1a3e5cd553aae716"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 new session ticket",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001611dddd6fc4869be0e1c12a5a29db1aa2e5814e5894e5"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 new session ticket",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016fc75cc914510008c6B45bb46b1f030921006c3556882"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 new session ticket followed by application data",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016c7d6d72499478b3d80281cae5b7c1a3e5cd553aae716"),
				testutil.Dehex("170303001ad7853afd6d7ceaabab950a0b6707905d2b908894871c7c62021f"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("789123456"),
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "AES-256-GCM-SHA384 new session ticket followed by application data",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001611dddd6fc4869be0e1c12a5a29db1aa2e5814e5894e5"),
				testutil.Dehex("170303001a832a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("789123456"),
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 new session ticket followed by application data",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016fc75cc914510008c6B45bb46b1f030921006c3556882"),
				testutil.Dehex("170303001a0cedeb922170c110c172262542c67916b78fa0d1c1261709cd00"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("789123456"),
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "AES-128-GCM-SHA256 ticket, application data, then another ticket",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016c7d6d72499478b3d80281cae5b7c1a3e5cd553aae716"),
				testutil.Dehex("170303001ad7853afd6d7ceaabab950a0b6707905d2b908894871c7c62021f"),
				testutil.Dehex("17030300169c4fb23ec187cec7a8443ae3cd6f45e9dca53023e952"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("789123456"),
				[]byte(""),
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "AES-256-GCM-SHA384 ticket, application data, then another ticket",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("170303001611dddd6fc4869be0e1c12a5a29db1aa2e5814e5894e5"),
				testutil.Dehex("170303001a832a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
				testutil.Dehex("1703030016a5f3bdfc4ab1cb73dca49bc86a7fde6396e83d9eb6ac"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("789123456"),
				[]byte(""),
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 ticket, application data, then another ticket",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			completedRecords: [][]byte{
				testutil.Dehex("1703030016fc75cc914510008c6B45bb46b1f030921006c3556882"),
				testutil.Dehex("170303001a0cedeb922170c110c172262542c67916b78fa0d1c1261709cd00"),
				testutil.Dehex("1703030016a1726a73d83b83e97018b7d4b9d33ec9528d7f10e8f2"),
			},
			outPlaintexts: [][]byte{
				[]byte(""),
				[]byte("789123456"),
				[]byte(""),
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				emptyTicket,
			},
			ticketsSent: true,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c, err := NewConn(&ConnParameters{
				NetConn:          &fakeConn{buf: tc.completedRecords},
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			newConn := c.(*conn)
			// Replace the ticket sender with a fake.
			fakeTicketSender := &fakeTicketSender{}
			newConn.ticketSender = fakeTicketSender
			for _, outPlaintext := range tc.outPlaintexts {
				plaintext := make([]byte, tlsRecordMaxPlaintextSize)
				n, err := c.Read(plaintext)
				if err != nil {
					t.Fatalf("c.Read(plaintext) failed: %v", err)
				}
				plaintext = plaintext[:n]
				if got, want := plaintext, outPlaintext; !bytes.Equal(got, want) {
					t.Errorf("c.Read(plaintext) = %v, want %v", got, want)
				}
				if got, want := len(c.(*conn).pendingApplicationData), 0; got != want {
					t.Errorf("len(c.(*conn).pendingApplicationData) = %v, want %v", got, want)
				}
			}
			newConn.Close()
			if got, want := newConn.ticketState, tc.finalTicketState; got != want {
				t.Errorf("newConn.ticketState = %v, want %v", got, want)
			}
			if got, want := newConn.handshakeBuf, make([]byte, 0); !bytes.Equal(got, want) {
				t.Errorf("newConn.handshakeBuf = %v, want %v", got, want)
			}
			if got, want := newConn.sessionTickets, tc.outSessionTickets; !cmp.Equal(got, want) {
				t.Errorf("newConn.sessionTickets = %v, want %v", got, want)
			}
			if tc.ticketsSent {
				if got, want := fakeTicketSender.sessionTickets, tc.outSessionTickets; !cmp.Equal(got, want) {
					t.Errorf("fakeTicketSender.sessionTickets = %v, want %v", got, want)
				}
			}
		})
	}
}

type handshakeMsg struct {
	msg         []byte
	isKeyUpdate bool
}

func TestConnHandshakeWithHandshakeBuilder(t *testing.T) {
	dummyTicket := buildSessionTicket([]byte("abc"))
	dummyTicketFragment1 := buildSessionTicket(make([]byte, tlsRecordMaxPlaintextSize/2))
	dummyTicketFragment2 := buildSessionTicket(make([]byte, tlsRecordMaxPlaintextSize))
	dummyLargeTicket := buildSessionTicket(make([]byte, tlsRecordMaxPlaintextSize*2))
	dummyTicketSplitHeader1 := buildSessionTicket([]byte("abc"))[:2]
	dummyTicketSplitHeader2 := buildSessionTicket([]byte("abc"))[2:]

	for _, tc := range []struct {
		desc              string
		ciphersuite       commonpb.Ciphersuite
		trafficSecret     []byte
		handshakeMsgs     []handshakeMsg
		finalTicketState  sessionTicketState
		outSessionTickets [][]byte
		ticketsSent       bool
		outErr            bool
	}{
		{
			desc:          "AES-128-GCM-SHA256 consecutive tickets",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 consecutive tickets",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 consecutive tickets",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 multiple tickets in one record",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(dummyTicket, dummyTicket...),
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 multiple tickets in one record",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(dummyTicket, dummyTicket...),
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 multiple tickets in one record",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(dummyTicket, dummyTicket...),
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 fragmented tickets",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(dummyTicketFragment1, dummyTicketFragment2...),
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicketFragment1,
				dummyTicketFragment2,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 fragmented tickets",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(dummyTicketFragment1, dummyTicketFragment2...),
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicketFragment1,
				dummyTicketFragment2,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 fragmented tickets",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(dummyTicketFragment1, dummyTicketFragment2...),
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicketFragment1,
				dummyTicketFragment2,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 large ticket",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyLargeTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyLargeTicket,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 large ticket",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyLargeTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyLargeTicket,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 large ticket",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyLargeTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyLargeTicket,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 split in header",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicketSplitHeader1,
				},
				{
					msg: dummyTicketSplitHeader2,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 split in header",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicketSplitHeader1,
				},
				{
					msg: dummyTicketSplitHeader2,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 split in header",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicketSplitHeader1,
				},
				{
					msg: dummyTicketSplitHeader2,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 past max limit",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
				dummyTicket,
				dummyTicket,
				dummyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "AES-256-GCM-SHA384 past max limit",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
				dummyTicket,
				dummyTicket,
				dummyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 past max limit",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: notReceivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
				dummyTicket,
				dummyTicket,
				dummyTicket,
			},
			ticketsSent: true,
		},
		{
			desc:          "AES-128-GCM-SHA256 consecutive key updates",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 consecutive key updates",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 consecutive key updates",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 consecutive key updates in single record",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg:         append(buildKeyUpdateRequest(), buildKeyUpdateRequest()...),
					isKeyUpdate: true,
				},
			},
			outErr: true,
		},
		{
			desc:          "AES-256-GCM-SHA384 consecutive key updates in single record",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg:         append(buildKeyUpdateRequest(), buildKeyUpdateRequest()...),
					isKeyUpdate: true,
				},
			},
			outErr: true,
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 consecutive key updates in single record",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg:         append(buildKeyUpdateRequest(), buildKeyUpdateRequest()...),
					isKeyUpdate: true,
				},
			},
			outErr: true,
		},
		{
			desc:          "AES-128-GCM-SHA256 fragmented key update",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: buildKeyUpdateRequest()[:2],
				},
				{
					msg:         buildKeyUpdateRequest()[2:],
					isKeyUpdate: true,
				},
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 fragmented key update",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: buildKeyUpdateRequest()[:2],
				},
				{
					msg:         buildKeyUpdateRequest()[2:],
					isKeyUpdate: true,
				},
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 fragmented key update",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: buildKeyUpdateRequest()[:2],
				},
				{
					msg:         buildKeyUpdateRequest()[2:],
					isKeyUpdate: true,
				},
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 key update between session tickets",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "AES-256-GCM-SHA384 key update between session tickets",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 key update between session tickets",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: dummyTicket,
				},
				{
					msg:         buildKeyUpdateRequest(),
					isKeyUpdate: true,
				},
				{
					msg: dummyTicket,
				},
			},
			finalTicketState: receivingTickets,
			outSessionTickets: [][]byte{
				dummyTicket,
				dummyTicket,
			},
		},
		{
			desc:          "AES-128-GCM-SHA256 key update between session tickets in a single record",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(append(dummyTicket, buildKeyUpdateRequest()...), dummyTicket...),
				},
			},
			outErr: true,
		},
		{
			desc:          "AES-256-GCM-SHA384 key update between session tickets in a single record",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(append(dummyTicket, buildKeyUpdateRequest()...), dummyTicket...),
				},
			},
			outErr: true,
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 key update between session tickets in a single record",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			handshakeMsgs: []handshakeMsg{
				{
					msg: append(append(dummyTicket, buildKeyUpdateRequest()...), dummyTicket...),
				},
			},
			outErr: true,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fc := &fakeConn{}
			netConn, err := NewConn(&ConnParameters{
				NetConn:          fc,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			c := netConn.(*conn)
			// Replace the ticket sender with a fake.
			fakeTicketSender := &fakeTicketSender{}
			c.ticketSender = fakeTicketSender
			for _, handshakeMsg := range tc.handshakeMsgs {
				_, err = c.writeTLSRecord(handshakeMsg.msg, byte(handshake))
				if err != nil {
					t.Fatalf("c.writeTLSRecord(hanshakeMsg, byte(handshake)) failed: %v", err)
				}
				if handshakeMsg.isKeyUpdate {
					if err := c.outConn.UpdateKey(); err != nil {
						t.Fatalf("c.outConn.UpdateKey() failed: %v", err)
					}
				}
			}
			fc.buf = fc.additionalBuf
			// Read until an EOF error occurs.
			for {
				plaintext := make([]byte, tlsRecordMaxPlaintextSize)
				n, err := c.Read(plaintext)
				if err != nil {
					if err == errFakeConnEOF {
						break
					}
					if !tc.outErr {
						t.Fatalf("c.Read(plaintext) failed: %v", err)
					}
					return
				}
				if got, want := n, 0; got != want {
					t.Errorf("c.Read(plaintext) = %v, want %v,", got, want)
				}
				if got, want := len(c.pendingApplicationData), 0; got != want {
					t.Errorf("len(c.(*conn).pendingApplicationData) = %v, want %v", got, want)
				}
			}
			netConn.Close()
			if got, want := c.ticketState, tc.finalTicketState; got != want {
				t.Errorf("newConn.ticketState = %v, want %v", got, want)
			}
			if got, want := c.handshakeBuf, make([]byte, 0); !bytes.Equal(got, want) {
				t.Errorf("newConn.handshakeBuf = %v, want %v", got, want)
			}
			if got, want := c.sessionTickets, tc.outSessionTickets; !cmp.Equal(got, want) {
				t.Errorf("newConn.sessionTickets = %v, want %v", got, want)
			}
			if tc.ticketsSent {
				if got, want := fakeTicketSender.sessionTickets, tc.outSessionTickets; !cmp.Equal(got, want) {
					t.Errorf("fakeTicketSender.sessionTickets = %v, want %v", got, want)
				}
			}
		})
	}
}

func TestWrite(t *testing.T) {
	for _, tc := range []struct {
		desc            string
		ciphersuite     commonpb.Ciphersuite
		trafficSecret   []byte
		plaintexts      [][]byte
		outRecords      [][]byte
		outBytesWritten []int
		outErr          bool
	}{
		// The traffic secrets were chosen randomly and are equivalent to the
		// ones used in C++ and Java. The ciphertext was constructed using an
		// existing TLS library.

		{
			desc:          "AES-128-GCM-SHA256",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			outRecords: [][]byte{
				testutil.Dehex("1703030017f2e4e411ac6760e4e3f074a36574c45ee4c1906103db0d"),
				testutil.Dehex("170303001ad7853afd6d7ceaabab950a0b6707905d2b908894871c7c62021f"),
			},
			outBytesWritten: []int{6, 9},
		},
		{
			desc:          "AES-128-GCM-SHA256 empty",
			ciphersuite:   commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte(""),
			},
			outRecords: [][]byte{
				testutil.Dehex("1703030011d47cb2ec040f26cc8989330339c669dd4e"),
			},
			outBytesWritten: []int{0},
		},
		{
			desc:          "AES-256-GCM-SHA384",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			outRecords: [][]byte{
				testutil.Dehex("170303001724efee5af1a62170ad5a95f899d038b965386a1a7daed9"),
				testutil.Dehex("170303001a832a5fd271b6442e74bc02111a8e8b52a74b14dd3eca8598b293"),
			},
			outBytesWritten: []int{6, 9},
		},
		{
			desc:          "AES-256-GCM-SHA384 empty",
			ciphersuite:   commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte(""),
			},
			outRecords: [][]byte{
				testutil.Dehex("170303001102a04134d38c1118f36b01d177c5d2dcf7"),
			},
			outBytesWritten: []int{0},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			outRecords: [][]byte{
				testutil.Dehex("1703030017c947ffa470304370338bb07ce468e6b8a0944a338ba402"),
				testutil.Dehex("170303001a0cedeb922170c110c172262542c67916b78fa0d1c1261709cd00"),
			},
			outBytesWritten: []int{6, 9},
		},
		{
			desc:          "CHACHA20-POLY1305-SHA256 empty",
			ciphersuite:   commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte(""),
			},
			outRecords: [][]byte{
				testutil.Dehex("1703030011ef8f7a428ddc84ee5968cd6306bf1d2d1b"),
			},
			outBytesWritten: []int{0},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fConn := &fakeConn{}
			newConn, err := NewConn(&ConnParameters{
				NetConn:          fConn,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			c := newConn.(*conn)
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			for i, plaintext := range tc.plaintexts {
				bytesWritten, err := c.writeTLSRecord(plaintext, tlsApplicationData)
				if got, want := err == nil, !tc.outErr; got != want {
					t.Errorf("c.Write(plaintext) = (err=nil) = %v, want %v", got, want)
				}
				if bytesWritten != tc.outBytesWritten[i] {
					t.Errorf("Incorrect number of bytes written: got: %v, want: %v", bytesWritten, tc.outBytesWritten[i])
				}
			}
			if !reflect.DeepEqual(fConn.additionalBuf, tc.outRecords) {
				t.Errorf("Incorrect Record: got: %v, want: %v", fConn.additionalBuf, tc.outRecords)
			}
		})
	}
}

func TestWriteTwoRecords(t *testing.T) {
	for _, tc := range []struct {
		desc            string
		ciphersuite     commonpb.Ciphersuite
		trafficSecret   []byte
		plaintext       []byte
		numRecordBytes  int
		outBytesWritten int
		outErr          bool
	}{
		// The plaintext of size tlsRecordMaxPlaintextSize + 1 will be written
		// to the underlying connection in 2 TLS records: one containing 2^14
		// bytes of plaintext, and the other containing 1 byte of plaintext,
		// resulting in 23+tlsRecordMaxSize total record bytes written,
		// including the overheads.
		{
			desc:            "AES-128-GCM-SHA256",
			ciphersuite:     commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintext:       make([]byte, 1+tlsRecordMaxPlaintextSize), // 2^14+1
			numRecordBytes:  23 + tlsRecordMaxSize,
			outBytesWritten: 1 + tlsRecordMaxPlaintextSize,
		},
		{
			desc:            "AES-256-GCM-SHA384",
			ciphersuite:     commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintext:       make([]byte, 1+tlsRecordMaxPlaintextSize), // 2^14+1
			numRecordBytes:  23 + tlsRecordMaxSize,
			outBytesWritten: 1 + tlsRecordMaxPlaintextSize,
		},
		{
			desc:            "CHACHA20-POLY1305-SHA256",
			ciphersuite:     commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret:   testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintext:       make([]byte, 1+tlsRecordMaxPlaintextSize), // 2^14+1
			numRecordBytes:  23 + tlsRecordMaxSize,
			outBytesWritten: 1 + tlsRecordMaxPlaintextSize,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fConn := &fakeConn{}
			newConn, err := NewConn(&ConnParameters{
				NetConn:          fConn,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			c := newConn.(*conn)
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			bytesWritten, err := c.writeTLSRecord(tc.plaintext, tlsApplicationData)
			if got, want := err == nil, !tc.outErr; got != want {
				t.Errorf("c.Write(plaintext) = (err=nil) = %v, want %v", got, want)
			}
			if bytesWritten != tc.outBytesWritten {
				t.Errorf("Incorrect number of bytes written: got: %v, want: %v", bytesWritten, tc.outBytesWritten)
			}
			if len(fConn.additionalBuf[0]) != tc.numRecordBytes {
				t.Errorf("Incorrect number of bytes prepared: got: %v, want: %v", len(fConn.additionalBuf[0]), tc.numRecordBytes)
			}
		})
	}
}

func TestExceedBufferSize(t *testing.T) {
	for _, tc := range []struct {
		desc                     string
		ciphersuite              commonpb.Ciphersuite
		trafficSecret            []byte
		plaintext                []byte
		expectedOutRecordBufSize int
		expectedNumWrites        int
		outErr                   bool
	}{
		// plaintext is set to 1+tlsRecordMaxPlaintextSize, 1 byte more than the maximum number of
		// plaintext bytes in a single record, expectedOutRecordBufSize is set
		// to 16406, as it is the maximum size of a single record.

		{
			desc:                     "AES-128-GCM-SHA256",
			ciphersuite:              commonpb.Ciphersuite_AES_128_GCM_SHA256,
			trafficSecret:            testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintext:                make([]byte, 1+tlsRecordMaxPlaintextSize*outBufMaxRecords),
			expectedOutRecordBufSize: outBufMaxSize,
			expectedNumWrites:        2,
		},
		{
			desc:                     "AES-256-GCM-SHA384",
			ciphersuite:              commonpb.Ciphersuite_AES_256_GCM_SHA384,
			trafficSecret:            testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintext:                make([]byte, 1+tlsRecordMaxPlaintextSize*outBufMaxRecords),
			expectedOutRecordBufSize: outBufMaxSize,
			expectedNumWrites:        2,
		},
		{
			desc:                     "CHACHA20-POLY1305-SHA256",
			ciphersuite:              commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			trafficSecret:            testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintext:                make([]byte, 1+tlsRecordMaxPlaintextSize*outBufMaxRecords),
			expectedOutRecordBufSize: outBufMaxSize,
			expectedNumWrites:        2,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fConn := &fakeConn{}
			newConn, err := NewConn(&ConnParameters{
				NetConn:          fConn,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.trafficSecret,
				OutTrafficSecret: tc.trafficSecret,
			})
			c := newConn.(*conn)
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			bytesWritten, err := c.writeTLSRecord(tc.plaintext, tlsApplicationData)
			if got, want := err == nil, !tc.outErr; got != want {
				t.Errorf("c.Write(plaintext) = (err=nil) = %v, want %v", err, want)
			}
			if bytesWritten != len(tc.plaintext) {
				t.Errorf("Incorrect number of bytes written: got: %v, want: %v", bytesWritten, len(tc.plaintext))
			}
			if len(c.outRecordsBuf) != tc.expectedOutRecordBufSize {
				t.Errorf("Incorrect buf size: got: %v, want: %v", len(c.outRecordsBuf), tc.expectedOutRecordBufSize)
			}
			if len(fConn.additionalBuf) != tc.expectedNumWrites {
				t.Errorf("Inforrect number of records: got: %v, want: %v,", len(fConn.additionalBuf), tc.expectedNumWrites)
			}
		})
	}
}

func TestRoundtrip(t *testing.T) {
	for _, tc := range []struct {
		desc                  string
		ciphersuite           commonpb.Ciphersuite
		inTrafficSecret       []byte
		outTrafficSecret      []byte
		plaintexts            [][]byte
		plaintextBytesWritten []int
		numRecordBytes        []int
	}{
		// numRecordBytes is calculated as
		// len(plaintext)+header(5)+tag(16)+record_type(1)
		{
			desc:             "AES-128-GCM-SHA256",
			ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			plaintextBytesWritten: []int{6, 9},
			numRecordBytes:        []int{28, 31},
		},
		{
			desc:             "AES-128-GCM-SHA256 different traffic secrets",
			ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			plaintextBytesWritten: []int{6, 9},
			numRecordBytes:        []int{28, 31},
		},
		{
			desc:             "AES-128-GCM-SHA256 empty",
			ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte(""),
			},
			plaintextBytesWritten: []int{0},
			numRecordBytes:        []int{22},
		},
		{
			desc:             "AES-128-GCM-SHA256 max buffer size",
			ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				make([]byte, tlsRecordMaxPlaintextSize*outBufMaxRecords),
			},
			plaintextBytesWritten: []int{tlsRecordMaxPlaintextSize * outBufMaxRecords},
			numRecordBytes:        []int{outBufMaxSize},
		},
		{
			desc:             "AES-128-GCM-SHA256 exceed buffer size",
			ciphersuite:      commonpb.Ciphersuite_AES_128_GCM_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				make([]byte, 1+tlsRecordMaxPlaintextSize*outBufMaxRecords),
			},
			plaintextBytesWritten: []int{1 + tlsRecordMaxPlaintextSize*outBufMaxRecords},
			numRecordBytes:        []int{outBufMaxSize, 23},
		},
		{
			desc:             "AES-256-GCM-SHA384",
			ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			plaintextBytesWritten: []int{6, 9},
			numRecordBytes:        []int{28, 31},
		},
		{
			desc:             "AES-256-GCM-SHA384 different traffic secrets",
			ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			plaintextBytesWritten: []int{6, 9},
			numRecordBytes:        []int{28, 31},
		},
		{
			desc:             "AES-256-GCM-SHA384 empty",
			ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte(""),
			},
			plaintextBytesWritten: []int{0},
			numRecordBytes:        []int{22},
		},
		{
			desc:             "AES-256-GCM-SHA384 max buffer size",
			ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				make([]byte, tlsRecordMaxPlaintextSize*outBufMaxRecords),
			},
			plaintextBytesWritten: []int{tlsRecordMaxPlaintextSize * outBufMaxRecords},
			numRecordBytes:        []int{outBufMaxSize},
		},
		{
			desc:             "AES-256-GCM-SHA384 exceed buffer size",
			ciphersuite:      commonpb.Ciphersuite_AES_256_GCM_SHA384,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				make([]byte, 1+tlsRecordMaxPlaintextSize*outBufMaxRecords),
			},
			plaintextBytesWritten: []int{1 + tlsRecordMaxPlaintextSize*outBufMaxRecords},
			numRecordBytes:        []int{outBufMaxSize, 23},
		},
		{
			desc:             "CHACHA20-POLY1305-SHA256",
			ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			plaintextBytesWritten: []int{6, 9},
			numRecordBytes:        []int{28, 31},
		},
		{
			desc:             "CHACHA20-POLY1305-SHA256 different traffic secrets",
			ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b"),
			plaintexts: [][]byte{
				[]byte("123456"),
				[]byte("789123456"),
			},
			plaintextBytesWritten: []int{6, 9},
			numRecordBytes:        []int{28, 31},
		},
		{
			desc:             "CHACHA20-POLY1305-SHA256 empty",
			ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				[]byte(""),
			},
			plaintextBytesWritten: []int{0},
			numRecordBytes:        []int{22},
		},
		{
			desc:             "CHACHA20-POLY1305-SHA256 max buffer size",
			ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				make([]byte, tlsRecordMaxPlaintextSize*outBufMaxRecords),
			},
			plaintextBytesWritten: []int{tlsRecordMaxPlaintextSize * outBufMaxRecords},
			numRecordBytes:        []int{outBufMaxSize},
		},
		{
			desc:             "CHACHA20-POLY1305-SHA256 max buffer size",
			ciphersuite:      commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			inTrafficSecret:  testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			outTrafficSecret: testutil.Dehex("6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b"),
			plaintexts: [][]byte{
				make([]byte, 1+tlsRecordMaxPlaintextSize*outBufMaxRecords),
			},
			plaintextBytesWritten: []int{1 + tlsRecordMaxPlaintextSize*outBufMaxRecords},
			numRecordBytes:        []int{outBufMaxSize, 23},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			fConnClient := &fakeConn{}
			client, err := NewConn(&ConnParameters{
				NetConn:          fConnClient,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.inTrafficSecret,
				OutTrafficSecret: tc.outTrafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			fConnServer := &fakeConn{}
			server, err := NewConn(&ConnParameters{
				NetConn:          fConnServer,
				Ciphersuite:      tc.ciphersuite,
				TLSVersion:       commonpb.TLSVersion_TLS1_3,
				InTrafficSecret:  tc.outTrafficSecret,
				OutTrafficSecret: tc.inTrafficSecret,
			})
			if err != nil {
				t.Fatalf("NewConn() failed: %v", err)
			}
			err = sendRecordsRoundtrip(t, client, server, fConnClient, fConnServer, tc.plaintexts, tc.plaintextBytesWritten, tc.numRecordBytes)
			if err != nil {
				return
			}
			sendRecordsRoundtrip(t, server, client, fConnServer, fConnClient, tc.plaintexts, tc.plaintextBytesWritten, tc.numRecordBytes)

		})
	}
}

func sendRecordsRoundtrip(t *testing.T, src net.Conn, dst net.Conn, fConnSrc *fakeConn, fConnDst *fakeConn, plaintexts [][]byte, plaintextBytesWritten []int, recordBytes []int) error {
	for i, plaintext := range plaintexts {
		bytesWritten, err := src.Write(plaintext)
		if got, want := err == nil, true; got != want {
			t.Errorf("c.Write(plaintext) = (err=nil) = %v, want %v", err, want)
			return errors.New("Write returned unexpected output")
		}

		if bytesWritten != plaintextBytesWritten[i] {
			t.Errorf("Incorrect number of bytes written: got: %v, want: %v", bytesWritten, plaintextBytesWritten[i])
			return errors.New("Write returned unexpected output")
		}

		if len(fConnSrc.additionalBuf[i]) != recordBytes[i] {
			t.Errorf("Incorrect number of bytes prepared: got: %v, want: %v", len(fConnSrc.additionalBuf[i]), recordBytes[i])
			return errors.New("Write returned unexpected output")
		}
	}
	fConnDst.buf = fConnSrc.additionalBuf
	for _, outPlaintext := range plaintexts {
		n := 0
		for n < len(outPlaintext) {
			plaintext := make([]byte, tlsRecordMaxPlaintextSize)
			dn, err := dst.Read(plaintext)
			if got, want := err == nil, true; got != want {
				t.Errorf("c.Read(plaintext) = (err=nil) = %v, want %v", err, want)
				return errors.New("Read returned unexpected output")
			}
			if got, want := plaintext[:dn], outPlaintext[n:n+dn]; !bytes.Equal(got, want) {
				t.Errorf("c.Read(plaintext) = %v, want %v", len(got), len(want))
				return errors.New("Read returned unexpected output")
			}
			n += dn
		}
	}
	return nil
}
