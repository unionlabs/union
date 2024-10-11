/*
 *
 * Copyright 2022 gRPC authors.
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

package test

import (
	"context"
	"fmt"
	"io"
	"reflect"
	"strings"
	"testing"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestInvalidMetadata(t *testing.T) {
	grpctest.TLogger.ExpectErrorN("stream: failed to validate md when setting trailer", 5)

	tests := []struct {
		name     string
		md       metadata.MD
		appendMD []string
		want     error
		recv     error
	}{
		{
			name: "invalid key",
			md:   map[string][]string{string(rune(0x19)): {"testVal"}},
			want: status.Error(codes.Internal, "header key \"\\x19\" contains illegal characters not in [0-9a-z-_.]"),
			recv: status.Error(codes.Internal, "invalid header field"),
		},
		{
			name: "invalid value",
			md:   map[string][]string{"test": {string(rune(0x19))}},
			want: status.Error(codes.Internal, "header key \"test\" contains value with non-printable ASCII characters"),
			recv: status.Error(codes.Internal, "invalid header field"),
		},
		{
			name:     "invalid appended value",
			md:       map[string][]string{"test": {"test"}},
			appendMD: []string{"/", "value"},
			want:     status.Error(codes.Internal, "header key \"/\" contains illegal characters not in [0-9a-z-_.]"),
			recv:     status.Error(codes.Internal, "invalid header field"),
		},
		{
			name:     "empty appended key",
			md:       map[string][]string{"test": {"test"}},
			appendMD: []string{"", "value"},
			want:     status.Error(codes.Internal, "there is an empty key in the header"),
			recv:     status.Error(codes.Internal, "invalid header field"),
		},
		{
			name: "empty key",
			md:   map[string][]string{"": {"test"}},
			want: status.Error(codes.Internal, "there is an empty key in the header"),
			recv: status.Error(codes.Internal, "invalid header field"),
		},
		{
			name: "-bin key with arbitrary value",
			md:   map[string][]string{"test-bin": {string(rune(0x19))}},
			want: nil,
			recv: io.EOF,
		},
		{
			name: "valid key and value",
			md:   map[string][]string{"test": {"value"}},
			want: nil,
			recv: io.EOF,
		},
	}

	testNum := 0
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			_, err := stream.Recv()
			if err != nil {
				return err
			}
			test := tests[testNum]
			testNum++
			// merge original md and added md.
			md := metadata.Join(test.md, metadata.Pairs(test.appendMD...))

			if err := stream.SetHeader(md); !reflect.DeepEqual(test.want, err) {
				return fmt.Errorf("call stream.SendHeader(md) validate metadata which is %v got err :%v, want err :%v", md, err, test.want)
			}
			if err := stream.SendHeader(md); !reflect.DeepEqual(test.want, err) {
				return fmt.Errorf("call stream.SendHeader(md) validate metadata which is %v got err :%v, want err :%v", md, err, test.want)
			}
			stream.SetTrailer(md)
			return nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting ss endpoint server: %v", err)
	}
	defer ss.Stop()

	for _, test := range tests {
		t.Run("unary "+test.name, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			ctx = metadata.NewOutgoingContext(ctx, test.md)
			ctx = metadata.AppendToOutgoingContext(ctx, test.appendMD...)
			if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); !reflect.DeepEqual(test.want, err) {
				t.Errorf("call ss.Client.EmptyCall() validate metadata which is %v got err :%v, want err :%v", test.md, err, test.want)
			}
		})
	}

	// call the stream server's api to drive the server-side unit testing
	for _, test := range tests {
		t.Run("streaming "+test.name, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			stream, err := ss.Client.FullDuplexCall(ctx)
			if err != nil {
				t.Errorf("call ss.Client.FullDuplexCall got err :%v", err)
				return
			}
			if err := stream.Send(&testpb.StreamingOutputCallRequest{}); err != nil {
				t.Errorf("call ss.Client stream Send(nil) will success but got err :%v", err)
			}
			if _, err := stream.Recv(); status.Code(err) != status.Code(test.recv) || !strings.Contains(err.Error(), test.recv.Error()) {
				t.Errorf("stream.Recv() = _, get err :%v, want err :%v", err, test.recv)
			}
		})
	}
}
