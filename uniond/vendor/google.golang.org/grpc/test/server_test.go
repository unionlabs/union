/*
 *
 * Copyright 2020 gRPC authors.
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
	"io"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

type ctxKey string

// TestServerReturningContextError verifies that if a context error is returned
// by the service handler, the status will have the correct status code, not
// Unknown.
func (s) TestServerReturningContextError(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return nil, context.DeadlineExceeded
		},
		FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
			return context.DeadlineExceeded
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	_, err := ss.Client.EmptyCall(ctx, &testpb.Empty{})
	if s, ok := status.FromError(err); !ok || s.Code() != codes.DeadlineExceeded {
		t.Fatalf("ss.Client.EmptyCall() got error %v; want <status with Code()=DeadlineExceeded>", err)
	}

	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("unexpected error starting the stream: %v", err)
	}
	_, err = stream.Recv()
	if s, ok := status.FromError(err); !ok || s.Code() != codes.DeadlineExceeded {
		t.Fatalf("ss.Client.FullDuplexCall().Recv() got error %v; want <status with Code()=DeadlineExceeded>", err)
	}

}

func (s) TestChainUnaryServerInterceptor(t *testing.T) {
	var (
		firstIntKey  = ctxKey("firstIntKey")
		secondIntKey = ctxKey("secondIntKey")
	)

	firstInt := func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		if ctx.Value(firstIntKey) != nil {
			return nil, status.Errorf(codes.Internal, "first interceptor should not have %v in context", firstIntKey)
		}
		if ctx.Value(secondIntKey) != nil {
			return nil, status.Errorf(codes.Internal, "first interceptor should not have %v in context", secondIntKey)
		}

		firstCtx := context.WithValue(ctx, firstIntKey, 0)
		resp, err := handler(firstCtx, req)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to handle request at firstInt")
		}

		simpleResp, ok := resp.(*testpb.SimpleResponse)
		if !ok {
			return nil, status.Errorf(codes.Internal, "failed to get *testpb.SimpleResponse at firstInt")
		}
		return &testpb.SimpleResponse{
			Payload: &testpb.Payload{
				Type: simpleResp.GetPayload().GetType(),
				Body: append(simpleResp.GetPayload().GetBody(), '1'),
			},
		}, nil
	}

	secondInt := func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		if ctx.Value(firstIntKey) == nil {
			return nil, status.Errorf(codes.Internal, "second interceptor should have %v in context", firstIntKey)
		}
		if ctx.Value(secondIntKey) != nil {
			return nil, status.Errorf(codes.Internal, "second interceptor should not have %v in context", secondIntKey)
		}

		secondCtx := context.WithValue(ctx, secondIntKey, 1)
		resp, err := handler(secondCtx, req)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to handle request at secondInt")
		}

		simpleResp, ok := resp.(*testpb.SimpleResponse)
		if !ok {
			return nil, status.Errorf(codes.Internal, "failed to get *testpb.SimpleResponse at secondInt")
		}
		return &testpb.SimpleResponse{
			Payload: &testpb.Payload{
				Type: simpleResp.GetPayload().GetType(),
				Body: append(simpleResp.GetPayload().GetBody(), '2'),
			},
		}, nil
	}

	lastInt := func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		if ctx.Value(firstIntKey) == nil {
			return nil, status.Errorf(codes.Internal, "last interceptor should have %v in context", firstIntKey)
		}
		if ctx.Value(secondIntKey) == nil {
			return nil, status.Errorf(codes.Internal, "last interceptor should not have %v in context", secondIntKey)
		}

		resp, err := handler(ctx, req)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "failed to handle request at lastInt at lastInt")
		}

		simpleResp, ok := resp.(*testpb.SimpleResponse)
		if !ok {
			return nil, status.Errorf(codes.Internal, "failed to get *testpb.SimpleResponse at lastInt")
		}
		return &testpb.SimpleResponse{
			Payload: &testpb.Payload{
				Type: simpleResp.GetPayload().GetType(),
				Body: append(simpleResp.GetPayload().GetBody(), '3'),
			},
		}, nil
	}

	sopts := []grpc.ServerOption{
		grpc.ChainUnaryInterceptor(firstInt, secondInt, lastInt),
	}

	ss := &stubserver.StubServer{
		UnaryCallF: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 0)
			if err != nil {
				return nil, status.Errorf(codes.Aborted, "failed to make payload: %v", err)
			}

			return &testpb.SimpleResponse{
				Payload: payload,
			}, nil
		},
	}
	if err := ss.Start(sopts); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{})
	if s, ok := status.FromError(err); !ok || s.Code() != codes.OK {
		t.Fatalf("ss.Client.UnaryCall(ctx, _) = %v, %v; want nil, <status with Code()=OK>", resp, err)
	}

	respBytes := resp.Payload.GetBody()
	if string(respBytes) != "321" {
		t.Fatalf("invalid response: want=%s, but got=%s", "321", resp)
	}
}

func (s) TestChainOnBaseUnaryServerInterceptor(t *testing.T) {
	baseIntKey := ctxKey("baseIntKey")

	baseInt := func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		if ctx.Value(baseIntKey) != nil {
			return nil, status.Errorf(codes.Internal, "base interceptor should not have %v in context", baseIntKey)
		}

		baseCtx := context.WithValue(ctx, baseIntKey, 1)
		return handler(baseCtx, req)
	}

	chainInt := func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		if ctx.Value(baseIntKey) == nil {
			return nil, status.Errorf(codes.Internal, "chain interceptor should have %v in context", baseIntKey)
		}

		return handler(ctx, req)
	}

	sopts := []grpc.ServerOption{
		grpc.UnaryInterceptor(baseInt),
		grpc.ChainUnaryInterceptor(chainInt),
	}

	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}
	if err := ss.Start(sopts); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, err := ss.Client.EmptyCall(ctx, &testpb.Empty{})
	if s, ok := status.FromError(err); !ok || s.Code() != codes.OK {
		t.Fatalf("ss.Client.EmptyCall(ctx, _) = %v, %v; want nil, <status with Code()=OK>", resp, err)
	}
}

func (s) TestChainStreamServerInterceptor(t *testing.T) {
	callCounts := make([]int, 4)

	firstInt := func(srv any, stream grpc.ServerStream, _ *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		if callCounts[0] != 0 {
			return status.Errorf(codes.Internal, "callCounts[0] should be 0, but got=%d", callCounts[0])
		}
		if callCounts[1] != 0 {
			return status.Errorf(codes.Internal, "callCounts[1] should be 0, but got=%d", callCounts[1])
		}
		if callCounts[2] != 0 {
			return status.Errorf(codes.Internal, "callCounts[2] should be 0, but got=%d", callCounts[2])
		}
		if callCounts[3] != 0 {
			return status.Errorf(codes.Internal, "callCounts[3] should be 0, but got=%d", callCounts[3])
		}
		callCounts[0]++
		return handler(srv, stream)
	}

	secondInt := func(srv any, stream grpc.ServerStream, _ *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		if callCounts[0] != 1 {
			return status.Errorf(codes.Internal, "callCounts[0] should be 1, but got=%d", callCounts[0])
		}
		if callCounts[1] != 0 {
			return status.Errorf(codes.Internal, "callCounts[1] should be 0, but got=%d", callCounts[1])
		}
		if callCounts[2] != 0 {
			return status.Errorf(codes.Internal, "callCounts[2] should be 0, but got=%d", callCounts[2])
		}
		if callCounts[3] != 0 {
			return status.Errorf(codes.Internal, "callCounts[3] should be 0, but got=%d", callCounts[3])
		}
		callCounts[1]++
		return handler(srv, stream)
	}

	lastInt := func(srv any, stream grpc.ServerStream, _ *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		if callCounts[0] != 1 {
			return status.Errorf(codes.Internal, "callCounts[0] should be 1, but got=%d", callCounts[0])
		}
		if callCounts[1] != 1 {
			return status.Errorf(codes.Internal, "callCounts[1] should be 1, but got=%d", callCounts[1])
		}
		if callCounts[2] != 0 {
			return status.Errorf(codes.Internal, "callCounts[2] should be 0, but got=%d", callCounts[2])
		}
		if callCounts[3] != 0 {
			return status.Errorf(codes.Internal, "callCounts[3] should be 0, but got=%d", callCounts[3])
		}
		callCounts[2]++
		return handler(srv, stream)
	}

	sopts := []grpc.ServerOption{
		grpc.ChainStreamInterceptor(firstInt, secondInt, lastInt),
	}

	ss := &stubserver.StubServer{
		FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
			if callCounts[0] != 1 {
				return status.Errorf(codes.Internal, "callCounts[0] should be 1, but got=%d", callCounts[0])
			}
			if callCounts[1] != 1 {
				return status.Errorf(codes.Internal, "callCounts[1] should be 1, but got=%d", callCounts[1])
			}
			if callCounts[2] != 1 {
				return status.Errorf(codes.Internal, "callCounts[2] should be 0, but got=%d", callCounts[2])
			}
			if callCounts[3] != 0 {
				return status.Errorf(codes.Internal, "callCounts[3] should be 0, but got=%d", callCounts[3])
			}
			callCounts[3]++
			return nil
		},
	}
	if err := ss.Start(sopts); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("failed to FullDuplexCall: %v", err)
	}

	_, err = stream.Recv()
	if err != io.EOF {
		t.Fatalf("failed to recv from stream: %v", err)
	}

	if callCounts[3] != 1 {
		t.Fatalf("callCounts[3] should be 1, but got=%d", callCounts[3])
	}
}
