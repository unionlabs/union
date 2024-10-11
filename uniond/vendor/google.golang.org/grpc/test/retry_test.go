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

package test

import (
	"context"
	"fmt"
	"io"
	"net"
	"reflect"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/stats"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestRetryUnary(t *testing.T) {
	i := -1
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (r *testpb.Empty, err error) {
			defer func() { t.Logf("server call %v returning err %v", i, err) }()
			i++
			switch i {
			case 0, 2, 5:
				return &testpb.Empty{}, nil
			case 6, 8, 11:
				return nil, status.New(codes.Internal, "non-retryable error").Err()
			}
			return nil, status.New(codes.AlreadyExists, "retryable error").Err()
		},
	}
	if err := ss.Start([]grpc.ServerOption{},
		grpc.WithDefaultServiceConfig(`{
    "methodConfig": [{
      "name": [{"service": "grpc.testing.TestService"}],
      "waitForReady": true,
      "retryPolicy": {
        "MaxAttempts": 4,
        "InitialBackoff": ".01s",
        "MaxBackoff": ".01s",
        "BackoffMultiplier": 1.0,
        "RetryableStatusCodes": [ "ALREADY_EXISTS" ]
      }
    }]}`)); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	testCases := []struct {
		code  codes.Code
		count int
	}{
		{codes.OK, 0},
		{codes.OK, 2},
		{codes.OK, 5},
		{codes.Internal, 6},
		{codes.Internal, 8},
		{codes.Internal, 11},
		{codes.AlreadyExists, 15},
	}
	for num, tc := range testCases {
		t.Log("Case", num)
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		_, err := ss.Client.EmptyCall(ctx, &testpb.Empty{})
		cancel()
		if status.Code(err) != tc.code {
			t.Fatalf("EmptyCall(_, _) = _, %v; want _, <Code() = %v>", err, tc.code)
		}
		if i != tc.count {
			t.Fatalf("i = %v; want %v", i, tc.count)
		}
	}
}

func (s) TestRetryThrottling(t *testing.T) {
	i := -1
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			i++
			switch i {
			case 0, 3, 6, 10, 11, 12, 13, 14, 16, 18:
				return &testpb.Empty{}, nil
			}
			return nil, status.New(codes.Unavailable, "retryable error").Err()
		},
	}
	if err := ss.Start([]grpc.ServerOption{},
		grpc.WithDefaultServiceConfig(`{
    "methodConfig": [{
      "name": [{"service": "grpc.testing.TestService"}],
      "waitForReady": true,
      "retryPolicy": {
        "MaxAttempts": 4,
        "InitialBackoff": ".01s",
        "MaxBackoff": ".01s",
        "BackoffMultiplier": 1.0,
        "RetryableStatusCodes": [ "UNAVAILABLE" ]
      }
    }],
    "retryThrottling": {
      "maxTokens": 10,
      "tokenRatio": 0.5
    }
    }`)); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	testCases := []struct {
		code  codes.Code
		count int
	}{
		{codes.OK, 0},           // tokens = 10
		{codes.OK, 3},           // tokens = 8.5 (10 - 2 failures + 0.5 success)
		{codes.OK, 6},           // tokens = 6
		{codes.Unavailable, 8},  // tokens = 5 -- first attempt is retried; second aborted.
		{codes.Unavailable, 9},  // tokens = 4
		{codes.OK, 10},          // tokens = 4.5
		{codes.OK, 11},          // tokens = 5
		{codes.OK, 12},          // tokens = 5.5
		{codes.OK, 13},          // tokens = 6
		{codes.OK, 14},          // tokens = 6.5
		{codes.OK, 16},          // tokens = 5.5
		{codes.Unavailable, 17}, // tokens = 4.5
	}
	for _, tc := range testCases {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		_, err := ss.Client.EmptyCall(ctx, &testpb.Empty{})
		cancel()
		if status.Code(err) != tc.code {
			t.Errorf("EmptyCall(_, _) = _, %v; want _, <Code() = %v>", err, tc.code)
		}
		if i != tc.count {
			t.Errorf("i = %v; want %v", i, tc.count)
		}
	}
}

func (s) TestRetryStreaming(t *testing.T) {
	req := func(b byte) *testpb.StreamingOutputCallRequest {
		return &testpb.StreamingOutputCallRequest{Payload: &testpb.Payload{Body: []byte{b}}}
	}
	res := func(b byte) *testpb.StreamingOutputCallResponse {
		return &testpb.StreamingOutputCallResponse{Payload: &testpb.Payload{Body: []byte{b}}}
	}

	largePayload, _ := newPayload(testpb.PayloadType_COMPRESSABLE, 500)

	type serverOp func(stream testgrpc.TestService_FullDuplexCallServer) error
	type clientOp func(stream testgrpc.TestService_FullDuplexCallClient) error

	// Server Operations
	sAttempts := func(n int) serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			const key = "grpc-previous-rpc-attempts"
			md, ok := metadata.FromIncomingContext(stream.Context())
			if !ok {
				return status.Errorf(codes.Internal, "server: no header metadata received")
			}
			if got := md[key]; len(got) != 1 || got[0] != strconv.Itoa(n) {
				return status.Errorf(codes.Internal, "server: metadata = %v; want <contains %q: %q>", md, key, n)
			}
			return nil
		}
	}
	sReq := func(b byte) serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			want := req(b)
			if got, err := stream.Recv(); err != nil || !proto.Equal(got, want) {
				return status.Errorf(codes.Internal, "server: Recv() = %v, %v; want %v, <nil>", got, err, want)
			}
			return nil
		}
	}
	sReqPayload := func(p *testpb.Payload) serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			want := &testpb.StreamingOutputCallRequest{Payload: p}
			if got, err := stream.Recv(); err != nil || !proto.Equal(got, want) {
				return status.Errorf(codes.Internal, "server: Recv() = %v, %v; want %v, <nil>", got, err, want)
			}
			return nil
		}
	}
	sHdr := func() serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			return stream.SendHeader(metadata.Pairs("test_header", "test_value"))
		}
	}
	sRes := func(b byte) serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			msg := res(b)
			if err := stream.Send(msg); err != nil {
				return status.Errorf(codes.Internal, "server: Send(%v) = %v; want <nil>", msg, err)
			}
			return nil
		}
	}
	sErr := func(c codes.Code) serverOp {
		return func(testgrpc.TestService_FullDuplexCallServer) error {
			return status.New(c, "this is a test error").Err()
		}
	}
	sCloseSend := func() serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			if msg, err := stream.Recv(); msg != nil || err != io.EOF {
				return status.Errorf(codes.Internal, "server: Recv() = %v, %v; want <nil>, io.EOF", msg, err)
			}
			return nil
		}
	}
	sPushback := func(s string) serverOp {
		return func(stream testgrpc.TestService_FullDuplexCallServer) error {
			stream.SetTrailer(metadata.MD{"grpc-retry-pushback-ms": []string{s}})
			return nil
		}
	}

	// Client Operations
	cReq := func(b byte) clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			msg := req(b)
			if err := stream.Send(msg); err != nil {
				return fmt.Errorf("client: Send(%v) = %v; want <nil>", msg, err)
			}
			return nil
		}
	}
	cReqPayload := func(p *testpb.Payload) clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			msg := &testpb.StreamingOutputCallRequest{Payload: p}
			if err := stream.Send(msg); err != nil {
				return fmt.Errorf("client: Send(%v) = %v; want <nil>", msg, err)
			}
			return nil
		}
	}
	cRes := func(b byte) clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			want := res(b)
			if got, err := stream.Recv(); err != nil || !proto.Equal(got, want) {
				return fmt.Errorf("client: Recv() = %v, %v; want %v, <nil>", got, err, want)
			}
			return nil
		}
	}
	cErr := func(c codes.Code) clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			want := status.New(c, "this is a test error").Err()
			if c == codes.OK {
				want = io.EOF
			}
			res, err := stream.Recv()
			if res != nil ||
				((err == nil) != (want == nil)) ||
				(want != nil && err.Error() != want.Error()) {
				return fmt.Errorf("client: Recv() = %v, %v; want <nil>, %v", res, err, want)
			}
			return nil
		}
	}
	cCloseSend := func() clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			if err := stream.CloseSend(); err != nil {
				return fmt.Errorf("client: CloseSend() = %v; want <nil>", err)
			}
			return nil
		}
	}
	var curTime time.Time
	cGetTime := func() clientOp {
		return func(_ testgrpc.TestService_FullDuplexCallClient) error {
			curTime = time.Now()
			return nil
		}
	}
	cCheckElapsed := func(d time.Duration) clientOp {
		return func(_ testgrpc.TestService_FullDuplexCallClient) error {
			if elapsed := time.Since(curTime); elapsed < d {
				return fmt.Errorf("elapsed time: %v; want >= %v", elapsed, d)
			}
			return nil
		}
	}
	cHdr := func() clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			_, err := stream.Header()
			if err == io.EOF {
				// The stream ended successfully; convert to nil to avoid
				// erroring the test case.
				err = nil
			}
			return err
		}
	}
	cCtx := func() clientOp {
		return func(stream testgrpc.TestService_FullDuplexCallClient) error {
			stream.Context()
			return nil
		}
	}

	testCases := []struct {
		desc      string
		serverOps []serverOp
		clientOps []clientOp
	}{{
		desc:      "Non-retryable error code",
		serverOps: []serverOp{sReq(1), sErr(codes.Internal)},
		clientOps: []clientOp{cReq(1), cErr(codes.Internal)},
	}, {
		desc:      "One retry necessary",
		serverOps: []serverOp{sReq(1), sErr(codes.Unavailable), sReq(1), sAttempts(1), sRes(1)},
		clientOps: []clientOp{cReq(1), cRes(1), cErr(codes.OK)},
	}, {
		desc: "Exceed max attempts (4); check attempts header on server",
		serverOps: []serverOp{
			sReq(1), sErr(codes.Unavailable),
			sReq(1), sAttempts(1), sErr(codes.Unavailable),
			sAttempts(2), sReq(1), sErr(codes.Unavailable),
			sAttempts(3), sReq(1), sErr(codes.Unavailable),
		},
		clientOps: []clientOp{cReq(1), cErr(codes.Unavailable)},
	}, {
		desc: "Multiple requests",
		serverOps: []serverOp{
			sReq(1), sReq(2), sErr(codes.Unavailable),
			sReq(1), sReq(2), sRes(5),
		},
		clientOps: []clientOp{cReq(1), cReq(2), cRes(5), cErr(codes.OK)},
	}, {
		desc: "Multiple successive requests",
		serverOps: []serverOp{
			sReq(1), sErr(codes.Unavailable),
			sReq(1), sReq(2), sErr(codes.Unavailable),
			sReq(1), sReq(2), sReq(3), sRes(5),
		},
		clientOps: []clientOp{cReq(1), cReq(2), cReq(3), cRes(5), cErr(codes.OK)},
	}, {
		desc: "No retry after receiving",
		serverOps: []serverOp{
			sReq(1), sErr(codes.Unavailable),
			sReq(1), sRes(3), sErr(codes.Unavailable),
		},
		clientOps: []clientOp{cReq(1), cRes(3), cErr(codes.Unavailable)},
	}, {
		desc:      "Retry via ClientStream.Header()",
		serverOps: []serverOp{sReq(1), sErr(codes.Unavailable), sReq(1), sAttempts(1)},
		clientOps: []clientOp{cReq(1), cHdr() /* this should cause a retry */, cErr(codes.OK)},
	}, {
		desc:      "No retry after header",
		serverOps: []serverOp{sReq(1), sHdr(), sErr(codes.Unavailable)},
		clientOps: []clientOp{cReq(1), cHdr(), cErr(codes.Unavailable)},
	}, {
		desc:      "No retry after context",
		serverOps: []serverOp{sReq(1), sErr(codes.Unavailable)},
		clientOps: []clientOp{cReq(1), cCtx(), cErr(codes.Unavailable)},
	}, {
		desc: "Replaying close send",
		serverOps: []serverOp{
			sReq(1), sReq(2), sCloseSend(), sErr(codes.Unavailable),
			sReq(1), sReq(2), sCloseSend(), sRes(1), sRes(3), sRes(5),
		},
		clientOps: []clientOp{cReq(1), cReq(2), cCloseSend(), cRes(1), cRes(3), cRes(5), cErr(codes.OK)},
	}, {
		desc:      "Negative server pushback - no retry",
		serverOps: []serverOp{sReq(1), sPushback("-1"), sErr(codes.Unavailable)},
		clientOps: []clientOp{cReq(1), cErr(codes.Unavailable)},
	}, {
		desc:      "Non-numeric server pushback - no retry",
		serverOps: []serverOp{sReq(1), sPushback("xxx"), sErr(codes.Unavailable)},
		clientOps: []clientOp{cReq(1), cErr(codes.Unavailable)},
	}, {
		desc:      "Multiple server pushback values - no retry",
		serverOps: []serverOp{sReq(1), sPushback("100"), sPushback("10"), sErr(codes.Unavailable)},
		clientOps: []clientOp{cReq(1), cErr(codes.Unavailable)},
	}, {
		desc:      "1s server pushback - delayed retry",
		serverOps: []serverOp{sReq(1), sPushback("1000"), sErr(codes.Unavailable), sReq(1), sRes(2)},
		clientOps: []clientOp{cGetTime(), cReq(1), cRes(2), cCheckElapsed(time.Second), cErr(codes.OK)},
	}, {
		desc:      "Overflowing buffer - no retry",
		serverOps: []serverOp{sReqPayload(largePayload), sErr(codes.Unavailable)},
		clientOps: []clientOp{cReqPayload(largePayload), cErr(codes.Unavailable)},
	}}

	var serverOpIter int
	var serverOps []serverOp
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			for serverOpIter < len(serverOps) {
				op := serverOps[serverOpIter]
				serverOpIter++
				if err := op(stream); err != nil {
					return err
				}
			}
			return nil
		},
	}
	if err := ss.Start([]grpc.ServerOption{}, grpc.WithDefaultCallOptions(grpc.MaxRetryRPCBufferSize(200)),
		grpc.WithDefaultServiceConfig(`{
    "methodConfig": [{
      "name": [{"service": "grpc.testing.TestService"}],
      "waitForReady": true,
      "retryPolicy": {
          "MaxAttempts": 4,
          "InitialBackoff": ".01s",
          "MaxBackoff": ".01s",
          "BackoffMultiplier": 1.0,
          "RetryableStatusCodes": [ "UNAVAILABLE" ]
      }
    }]}`)); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for {
		if ctx.Err() != nil {
			t.Fatalf("Timed out waiting for service config update")
		}
		if ss.CC.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").WaitForReady != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}

	for i, tc := range testCases {
		func() {
			serverOpIter = 0
			serverOps = tc.serverOps

			stream, err := ss.Client.FullDuplexCall(ctx)
			if err != nil {
				t.Fatalf("%v: Error while creating stream: %v", tc.desc, err)
			}
			for j, op := range tc.clientOps {
				if err := op(stream); err != nil {
					t.Errorf("%d %d %v: %v", i, j, tc.desc, err)
					break
				}
			}
			if serverOpIter != len(serverOps) {
				t.Errorf("%v: serverOpIter = %v; want %v", tc.desc, serverOpIter, len(serverOps))
			}
		}()
	}
}

func (s) TestMaxCallAttempts(t *testing.T) {
	testCases := []struct {
		serviceMaxAttempts int
		clientMaxAttempts  int
		expectedAttempts   int
	}{
		{serviceMaxAttempts: 9, clientMaxAttempts: 4, expectedAttempts: 4},
		{serviceMaxAttempts: 9, clientMaxAttempts: 7, expectedAttempts: 7},
		{serviceMaxAttempts: 3, clientMaxAttempts: 10, expectedAttempts: 3},
		{serviceMaxAttempts: 8, clientMaxAttempts: -1, expectedAttempts: 5}, // 5 is default max
		{serviceMaxAttempts: 3, clientMaxAttempts: 0, expectedAttempts: 3},
	}

	for _, tc := range testCases {
		clientOpts := []grpc.DialOption{
			grpc.WithMaxCallAttempts(tc.clientMaxAttempts),
			grpc.WithDefaultServiceConfig(fmt.Sprintf(`{
				"methodConfig": [{
					"name": [{"service": "grpc.testing.TestService"}],
					"waitForReady": true,
					"retryPolicy": {
						"MaxAttempts": %d,
						"InitialBackoff": ".01s",
						"MaxBackoff": ".01s",
						"BackoffMultiplier": 1.0,
						"RetryableStatusCodes": [ "UNAVAILABLE" ]
					}
					}]}`, tc.serviceMaxAttempts),
			),
		}

		streamCallCount := 0
		unaryCallCount := 0

		ss := &stubserver.StubServer{
			FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
				streamCallCount++
				return status.New(codes.Unavailable, "this is a test error").Err()
			},
			EmptyCallF: func(context.Context, *testpb.Empty) (r *testpb.Empty, err error) {
				unaryCallCount++
				return nil, status.New(codes.Unavailable, "this is a test error").Err()
			},
		}

		func() {

			if err := ss.Start([]grpc.ServerOption{}, clientOpts...); err != nil {
				t.Fatalf("Error starting endpoint server: %v", err)
			}
			defer ss.Stop()
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()

			for {
				if ctx.Err() != nil {
					t.Fatalf("Timed out waiting for service config update")
				}
				if ss.CC.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").WaitForReady != nil {
					break
				}
				time.Sleep(time.Millisecond)
			}

			// Test streaming RPC
			stream, err := ss.Client.FullDuplexCall(ctx)
			if err != nil {
				t.Fatalf("Error while creating stream: %v", err)
			}
			if got, err := stream.Recv(); err == nil {
				t.Fatalf("client: Recv() = %s, %v; want <nil>, error", got, err)
			} else if status.Code(err) != codes.Unavailable {
				t.Fatalf("client: Recv() = _, %v; want _, Unavailable", err)
			}
			if streamCallCount != tc.expectedAttempts {
				t.Fatalf("stream expectedAttempts = %v; want %v", streamCallCount, tc.expectedAttempts)
			}

			// Test unary RPC
			if ugot, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); err == nil {
				t.Fatalf("client: EmptyCall() = %s, %v; want <nil>, error", ugot, err)
			} else if status.Code(err) != codes.Unavailable {
				t.Fatalf("client: EmptyCall() = _, %v; want _, Unavailable", err)
			}
			if unaryCallCount != tc.expectedAttempts {
				t.Fatalf("unary expectedAttempts = %v; want %v", unaryCallCount, tc.expectedAttempts)
			}
		}()
	}
}

type retryStatsHandler struct {
	mu sync.Mutex
	s  []stats.RPCStats
}

func (*retryStatsHandler) TagRPC(ctx context.Context, _ *stats.RPCTagInfo) context.Context {
	return ctx
}
func (h *retryStatsHandler) HandleRPC(_ context.Context, s stats.RPCStats) {
	// these calls come in nondeterministically - so can just ignore
	if _, ok := s.(*stats.PickerUpdated); ok {
		return
	}
	h.mu.Lock()
	h.s = append(h.s, s)
	h.mu.Unlock()
}
func (*retryStatsHandler) TagConn(ctx context.Context, _ *stats.ConnTagInfo) context.Context {
	return ctx
}
func (*retryStatsHandler) HandleConn(context.Context, stats.ConnStats) {}

func (s) TestRetryStats(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen. Err: %v", err)
	}
	defer lis.Close()
	server := &httpServer{
		waitForEndStream: true,
		responses: []httpServerResponse{{
			trailers: [][]string{{
				":status", "200",
				"content-type", "application/grpc",
				"grpc-status", "14", // UNAVAILABLE
				"grpc-message", "unavailable retry",
				"grpc-retry-pushback-ms", "10",
			}},
		}, {
			headers: [][]string{{
				":status", "200",
				"content-type", "application/grpc",
			}},
			payload: []byte{0, 0, 0, 0, 0}, // header for 0-byte response message.
			trailers: [][]string{{
				"grpc-status", "0", // OK
			}},
		}},
		refuseStream: func(i uint32) bool {
			return i == 1
		},
	}
	server.start(t, lis)
	handler := &retryStatsHandler{}
	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithStatsHandler(handler),
		grpc.WithDefaultServiceConfig((`{
    "methodConfig": [{
      "name": [{"service": "grpc.testing.TestService"}],
      "retryPolicy": {
          "MaxAttempts": 4,
          "InitialBackoff": ".01s",
          "MaxBackoff": ".01s",
          "BackoffMultiplier": 1.0,
          "RetryableStatusCodes": [ "UNAVAILABLE" ]
      }
    }]}`)))
	if err != nil {
		t.Fatalf("failed to dial due to err: %v", err)
	}
	defer cc.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	client := testgrpc.NewTestServiceClient(cc)

	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("unexpected EmptyCall error: %v", err)
	}
	handler.mu.Lock()
	want := []stats.RPCStats{
		&stats.Begin{},
		&stats.OutHeader{FullMethod: "/grpc.testing.TestService/EmptyCall"},
		&stats.OutPayload{WireLength: 5},
		&stats.End{},

		&stats.Begin{IsTransparentRetryAttempt: true},
		&stats.OutHeader{FullMethod: "/grpc.testing.TestService/EmptyCall"},
		&stats.OutPayload{WireLength: 5},
		&stats.InTrailer{Trailer: metadata.Pairs("content-type", "application/grpc", "grpc-retry-pushback-ms", "10")},
		&stats.End{},

		&stats.Begin{},
		&stats.OutHeader{FullMethod: "/grpc.testing.TestService/EmptyCall"},
		&stats.OutPayload{WireLength: 5},
		&stats.InHeader{},
		&stats.InPayload{WireLength: 5},
		&stats.InTrailer{},
		&stats.End{},
	}

	toString := func(ss []stats.RPCStats) (ret []string) {
		for _, s := range ss {
			ret = append(ret, fmt.Sprintf("%T - %v", s, s))
		}
		return ret
	}
	t.Logf("Handler received frames:\n%v\n---\nwant:\n%v\n",
		strings.Join(toString(handler.s), "\n"),
		strings.Join(toString(want), "\n"))

	if len(handler.s) != len(want) {
		t.Fatalf("received unexpected number of RPCStats: got %v; want %v", len(handler.s), len(want))
	}

	// There is a race between receiving the payload (triggered by the
	// application / gRPC library) and receiving the trailer (triggered at the
	// transport layer).  Adjust the received stats accordingly if necessary.
	const tIdx, pIdx = 13, 14
	_, okT := handler.s[tIdx].(*stats.InTrailer)
	_, okP := handler.s[pIdx].(*stats.InPayload)
	if okT && okP {
		handler.s[pIdx], handler.s[tIdx] = handler.s[tIdx], handler.s[pIdx]
	}

	for i := range handler.s {
		w, s := want[i], handler.s[i]

		// Validate the event type
		if reflect.TypeOf(w) != reflect.TypeOf(s) {
			t.Fatalf("at position %v: got %T; want %T", i, s, w)
		}
		wv, sv := reflect.ValueOf(w).Elem(), reflect.ValueOf(s).Elem()

		// Validate that Client is always true
		if sv.FieldByName("Client").Interface().(bool) != true {
			t.Fatalf("at position %v: got Client=false; want true", i)
		}

		// Validate any set fields in want
		for i := 0; i < wv.NumField(); i++ {
			if !wv.Field(i).IsZero() {
				if got, want := sv.Field(i).Interface(), wv.Field(i).Interface(); !reflect.DeepEqual(got, want) {
					name := reflect.TypeOf(w).Elem().Field(i).Name
					t.Fatalf("at position %v, field %v: got %v; want %v", i, name, got, want)
				}
			}
		}

		// Since the above only tests non-zero-value fields, test
		// IsTransparentRetryAttempt=false explicitly when needed.
		if wb, ok := w.(*stats.Begin); ok && !wb.IsTransparentRetryAttempt {
			if s.(*stats.Begin).IsTransparentRetryAttempt {
				t.Fatalf("at position %v: got IsTransparentRetryAttempt=true; want false", i)
			}
		}
	}

	// Validate timings between last Begin and preceding End.
	end := handler.s[8].(*stats.End)
	begin := handler.s[9].(*stats.Begin)
	diff := begin.BeginTime.Sub(end.EndTime)
	if diff < 10*time.Millisecond || diff > 50*time.Millisecond {
		t.Fatalf("pushback time before final attempt = %v; want ~10ms", diff)
	}
}

func (s) TestRetryTransparentWhenCommitted(t *testing.T) {
	// With MaxConcurrentStreams=1:
	//
	// 1. Create stream 1 that is retriable.
	// 2. Stream 1 is created and fails with a retriable code.
	// 3. Create dummy stream 2, blocking indefinitely.
	// 4. Stream 1 retries (and blocks until stream 2 finishes)
	// 5. Stream 1 is canceled manually.
	//
	// If there is no bug, the stream is done and errors with CANCELED.  With a bug:
	//
	// 6. Stream 1 has a nil stream (attempt.s).  Operations like CloseSend will panic.

	first := grpcsync.NewEvent()
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			// signal?
			if !first.HasFired() {
				first.Fire()
				t.Log("returned first error")
				return status.Error(codes.AlreadyExists, "first attempt fails and is retriable")
			}
			t.Log("blocking")
			<-stream.Context().Done()
			return stream.Context().Err()
		},
	}

	if err := ss.Start([]grpc.ServerOption{grpc.MaxConcurrentStreams(1)},
		grpc.WithDefaultServiceConfig(`{
    "methodConfig": [{
      "name": [{"service": "grpc.testing.TestService"}],
      "waitForReady": true,
      "retryPolicy": {
        "MaxAttempts": 2,
        "InitialBackoff": ".1s",
        "MaxBackoff": ".1s",
        "BackoffMultiplier": 1.0,
        "RetryableStatusCodes": [ "ALREADY_EXISTS" ]
      }
    }]}`)); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx1, cancel1 := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel1()
	ctx2, cancel2 := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel2()

	stream1, err := ss.Client.FullDuplexCall(ctx1)
	if err != nil {
		t.Fatalf("Error creating stream 1: %v", err)
	}

	// Create dummy stream to block indefinitely.
	_, err = ss.Client.FullDuplexCall(ctx2)
	if err != nil {
		t.Errorf("Error creating stream 2: %v", err)
	}

	stream1Closed := grpcsync.NewEvent()
	go func() {
		_, err := stream1.Recv()
		// Will trigger a retry when it sees the ALREADY_EXISTS error
		if status.Code(err) != codes.Canceled {
			t.Errorf("Expected stream1 to be canceled; got error: %v", err)
		}
		stream1Closed.Fire()
	}()

	// Wait longer than the retry backoff timer.
	time.Sleep(200 * time.Millisecond)
	cancel1()

	// Operations on the stream should not panic.
	<-stream1Closed.Done()
	stream1.CloseSend()
	stream1.Recv()
	stream1.Send(&testpb.StreamingOutputCallRequest{})
}
