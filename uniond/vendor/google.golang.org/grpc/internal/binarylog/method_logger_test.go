/*
 *
 * Copyright 2018 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package binarylog

import (
	"bytes"
	"context"
	"fmt"
	"net"
	"testing"
	"time"

	binlogpb "google.golang.org/grpc/binarylog/grpc_binarylog_v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"
)

const defaultTestTimeout = 10 * time.Second

func (s) TestLog(t *testing.T) {
	idGen.reset()
	ml := NewTruncatingMethodLogger(10, 10)
	// Set sink to testing buffer.
	buf := bytes.NewBuffer(nil)
	ml.sink = newWriterSink(buf)

	addr := "1.2.3.4"
	port := 790
	tcpAddr, _ := net.ResolveTCPAddr("tcp", fmt.Sprintf("%v:%d", addr, port))
	addr6 := "2001:1db8:85a3::8a2e:1370:7334"
	port6 := 796
	tcpAddr6, _ := net.ResolveTCPAddr("tcp", fmt.Sprintf("[%v]:%d", addr6, port6))

	testProtoMsg := &binlogpb.Message{
		Length: 1,
		Data:   []byte{'a'},
	}
	testProtoBytes, _ := proto.Marshal(testProtoMsg)

	testCases := []struct {
		config LogEntryConfig
		want   *binlogpb.GrpcLogEntry
	}{
		{
			config: &ClientHeader{
				OnClientSide: false,
				Header: map[string][]string{
					"a": {"b", "bb"},
				},
				MethodName: "testservice/testmethod",
				Authority:  "test.service.io",
				Timeout:    2*time.Second + 3*time.Nanosecond,
				PeerAddr:   tcpAddr,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_HEADER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_SERVER,
				Payload: &binlogpb.GrpcLogEntry_ClientHeader{
					ClientHeader: &binlogpb.ClientHeader{
						Metadata: &binlogpb.Metadata{
							Entry: []*binlogpb.MetadataEntry{
								{Key: "a", Value: []byte{'b'}},
								{Key: "a", Value: []byte{'b', 'b'}},
							},
						},
						MethodName: "testservice/testmethod",
						Authority:  "test.service.io",
						Timeout: &durationpb.Duration{
							Seconds: 2,
							Nanos:   3,
						},
					},
				},
				PayloadTruncated: false,
				Peer: &binlogpb.Address{
					Type:    binlogpb.Address_TYPE_IPV4,
					Address: addr,
					IpPort:  uint32(port),
				},
			},
		},
		{
			config: &ClientHeader{
				OnClientSide: false,
				MethodName:   "testservice/testmethod",
				Authority:    "test.service.io",
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_HEADER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_SERVER,
				Payload: &binlogpb.GrpcLogEntry_ClientHeader{
					ClientHeader: &binlogpb.ClientHeader{
						Metadata:   &binlogpb.Metadata{},
						MethodName: "testservice/testmethod",
						Authority:  "test.service.io",
					},
				},
				PayloadTruncated: false,
			},
		},
		{
			config: &ServerHeader{
				OnClientSide: true,
				Header: map[string][]string{
					"a": {"b", "bb"},
				},
				PeerAddr: tcpAddr6,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_HEADER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
				Payload: &binlogpb.GrpcLogEntry_ServerHeader{
					ServerHeader: &binlogpb.ServerHeader{
						Metadata: &binlogpb.Metadata{
							Entry: []*binlogpb.MetadataEntry{
								{Key: "a", Value: []byte{'b'}},
								{Key: "a", Value: []byte{'b', 'b'}},
							},
						},
					},
				},
				PayloadTruncated: false,
				Peer: &binlogpb.Address{
					Type:    binlogpb.Address_TYPE_IPV6,
					Address: addr6,
					IpPort:  uint32(port6),
				},
			},
		},
		{
			config: &ClientMessage{
				OnClientSide: true,
				Message:      testProtoMsg,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_MESSAGE,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
				Payload: &binlogpb.GrpcLogEntry_Message{
					Message: &binlogpb.Message{
						Length: uint32(len(testProtoBytes)),
						Data:   testProtoBytes,
					},
				},
				PayloadTruncated: false,
				Peer:             nil,
			},
		},
		{
			config: &ServerMessage{
				OnClientSide: false,
				Message:      testProtoMsg,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_MESSAGE,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_SERVER,
				Payload: &binlogpb.GrpcLogEntry_Message{
					Message: &binlogpb.Message{
						Length: uint32(len(testProtoBytes)),
						Data:   testProtoBytes,
					},
				},
				PayloadTruncated: false,
				Peer:             nil,
			},
		},
		{
			config: &ClientHalfClose{
				OnClientSide: false,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_HALF_CLOSE,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_SERVER,
				Payload:              nil,
				PayloadTruncated:     false,
				Peer:                 nil,
			},
		},
		{
			config: &ServerTrailer{
				OnClientSide: true,
				Err:          status.Errorf(codes.Unavailable, "test"),
				PeerAddr:     tcpAddr,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_TRAILER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
				Payload: &binlogpb.GrpcLogEntry_Trailer{
					Trailer: &binlogpb.Trailer{
						Metadata:      &binlogpb.Metadata{},
						StatusCode:    uint32(codes.Unavailable),
						StatusMessage: "test",
						StatusDetails: nil,
					},
				},
				PayloadTruncated: false,
				Peer: &binlogpb.Address{
					Type:    binlogpb.Address_TYPE_IPV4,
					Address: addr,
					IpPort:  uint32(port),
				},
			},
		},
		{ // Err is nil, Log OK status.
			config: &ServerTrailer{
				OnClientSide: true,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_TRAILER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
				Payload: &binlogpb.GrpcLogEntry_Trailer{
					Trailer: &binlogpb.Trailer{
						Metadata:      &binlogpb.Metadata{},
						StatusCode:    uint32(codes.OK),
						StatusMessage: "",
						StatusDetails: nil,
					},
				},
				PayloadTruncated: false,
				Peer:             nil,
			},
		},
		{
			config: &Cancel{
				OnClientSide: true,
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CANCEL,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
				Payload:              nil,
				PayloadTruncated:     false,
				Peer:                 nil,
			},
		},

		// gRPC headers should be omitted.
		{
			config: &ClientHeader{
				OnClientSide: false,
				Header: map[string][]string{
					"grpc-reserved": {"to be omitted"},
					":authority":    {"to be omitted"},
					"a":             {"b", "bb"},
				},
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_HEADER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_SERVER,
				Payload: &binlogpb.GrpcLogEntry_ClientHeader{
					ClientHeader: &binlogpb.ClientHeader{
						Metadata: &binlogpb.Metadata{
							Entry: []*binlogpb.MetadataEntry{
								{Key: "a", Value: []byte{'b'}},
								{Key: "a", Value: []byte{'b', 'b'}},
							},
						},
					},
				},
				PayloadTruncated: false,
			},
		},
		{
			config: &ServerHeader{
				OnClientSide: true,
				Header: map[string][]string{
					"grpc-reserved": {"to be omitted"},
					":authority":    {"to be omitted"},
					"a":             {"b", "bb"},
				},
			},
			want: &binlogpb.GrpcLogEntry{
				Timestamp:            nil,
				CallId:               1,
				SequenceIdWithinCall: 0,
				Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_HEADER,
				Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
				Payload: &binlogpb.GrpcLogEntry_ServerHeader{
					ServerHeader: &binlogpb.ServerHeader{
						Metadata: &binlogpb.Metadata{
							Entry: []*binlogpb.MetadataEntry{
								{Key: "a", Value: []byte{'b'}},
								{Key: "a", Value: []byte{'b', 'b'}},
							},
						},
					},
				},
				PayloadTruncated: false,
			},
		},
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i, tc := range testCases {
		buf.Reset()
		tc.want.SequenceIdWithinCall = uint64(i + 1)
		ml.Log(ctx, tc.config)
		inSink := new(binlogpb.GrpcLogEntry)
		if err := proto.Unmarshal(buf.Bytes()[4:], inSink); err != nil {
			t.Errorf("failed to unmarshal bytes in sink to proto: %v", err)
			continue
		}
		inSink.Timestamp = nil // Strip timestamp before comparing.
		if !proto.Equal(inSink, tc.want) {
			t.Errorf("Log(%+v), in sink: %+v, want %+v", tc.config, inSink, tc.want)
		}
	}
}

func (s) TestTruncateMetadataNotTruncated(t *testing.T) {
	testCases := []struct {
		ml   *TruncatingMethodLogger
		mpPb *binlogpb.Metadata
	}{
		{
			ml: NewTruncatingMethodLogger(maxUInt, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1}},
				},
			},
		},
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1}},
				},
			},
		},
		{
			ml: NewTruncatingMethodLogger(1, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: nil},
				},
			},
		},
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1, 1}},
				},
			},
		},
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1}},
					{Key: "", Value: []byte{1}},
				},
			},
		},
		// "grpc-trace-bin" is kept in log but not counted towards the size
		// limit.
		{
			ml: NewTruncatingMethodLogger(1, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1}},
					{Key: "grpc-trace-bin", Value: []byte("some.trace.key")},
				},
			},
		},
	}

	for i, tc := range testCases {
		truncated := tc.ml.truncateMetadata(tc.mpPb)
		if truncated {
			t.Errorf("test case %v, returned truncated, want not truncated", i)
		}
	}
}

func (s) TestTruncateMetadataTruncated(t *testing.T) {
	testCases := []struct {
		ml   *TruncatingMethodLogger
		mpPb *binlogpb.Metadata

		entryLen int
	}{
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1, 1, 1}},
				},
			},
			entryLen: 0,
		},
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1}},
					{Key: "", Value: []byte{1}},
					{Key: "", Value: []byte{1}},
				},
			},
			entryLen: 2,
		},
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1, 1}},
					{Key: "", Value: []byte{1}},
				},
			},
			entryLen: 1,
		},
		{
			ml: NewTruncatingMethodLogger(2, maxUInt),
			mpPb: &binlogpb.Metadata{
				Entry: []*binlogpb.MetadataEntry{
					{Key: "", Value: []byte{1}},
					{Key: "", Value: []byte{1, 1}},
				},
			},
			entryLen: 1,
		},
	}

	for i, tc := range testCases {
		truncated := tc.ml.truncateMetadata(tc.mpPb)
		if !truncated {
			t.Errorf("test case %v, returned not truncated, want truncated", i)
			continue
		}
		if len(tc.mpPb.Entry) != tc.entryLen {
			t.Errorf("test case %v, entry length: %v, want: %v", i, len(tc.mpPb.Entry), tc.entryLen)
		}
	}
}

func (s) TestTruncateMessageNotTruncated(t *testing.T) {
	testCases := []struct {
		ml    *TruncatingMethodLogger
		msgPb *binlogpb.Message
	}{
		{
			ml: NewTruncatingMethodLogger(maxUInt, maxUInt),
			msgPb: &binlogpb.Message{
				Data: []byte{1},
			},
		},
		{
			ml: NewTruncatingMethodLogger(maxUInt, 3),
			msgPb: &binlogpb.Message{
				Data: []byte{1, 1},
			},
		},
		{
			ml: NewTruncatingMethodLogger(maxUInt, 2),
			msgPb: &binlogpb.Message{
				Data: []byte{1, 1},
			},
		},
	}

	for i, tc := range testCases {
		truncated := tc.ml.truncateMessage(tc.msgPb)
		if truncated {
			t.Errorf("test case %v, returned truncated, want not truncated", i)
		}
	}
}

func (s) TestTruncateMessageTruncated(t *testing.T) {
	testCases := []struct {
		ml    *TruncatingMethodLogger
		msgPb *binlogpb.Message

		oldLength uint32
	}{
		{
			ml: NewTruncatingMethodLogger(maxUInt, 2),
			msgPb: &binlogpb.Message{
				Length: 3,
				Data:   []byte{1, 1, 1},
			},
			oldLength: 3,
		},
	}

	for i, tc := range testCases {
		truncated := tc.ml.truncateMessage(tc.msgPb)
		if !truncated {
			t.Errorf("test case %v, returned not truncated, want truncated", i)
			continue
		}
		if len(tc.msgPb.Data) != int(tc.ml.messageMaxLen) {
			t.Errorf("test case %v, message length: %v, want: %v", i, len(tc.msgPb.Data), tc.ml.messageMaxLen)
		}
		if tc.msgPb.Length != tc.oldLength {
			t.Errorf("test case %v, message.Length field: %v, want: %v", i, tc.msgPb.Length, tc.oldLength)
		}
	}
}
