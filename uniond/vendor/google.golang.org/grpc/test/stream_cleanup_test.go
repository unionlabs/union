/*
 *
 * Copyright 2019 gRPC authors.
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
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestStreamCleanup(t *testing.T) {
	const initialWindowSize uint = 70 * 1024 // Must be higher than default 64K, ignored otherwise
	const bodySize = 2 * initialWindowSize   // Something that is not going to fit in a single window
	const callRecvMsgSize uint = 1           // The maximum message size the client can receive

	ss := &stubserver.StubServer{
		UnaryCallF: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{Payload: &testpb.Payload{
				Body: make([]byte, bodySize),
			}}, nil
		},
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}
	if err := ss.Start(nil, grpc.WithDefaultCallOptions(grpc.MaxCallRecvMsgSize(int(callRecvMsgSize))), grpc.WithInitialWindowSize(int32(initialWindowSize))); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{}); status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("should fail with ResourceExhausted, message's body size: %v, maximum message size the client can receive: %v", bodySize, callRecvMsgSize)
	}
	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("should succeed, err: %v", err)
	}
}

func (s) TestStreamCleanupAfterSendStatus(t *testing.T) {
	const initialWindowSize uint = 70 * 1024 // Must be higher than default 64K, ignored otherwise
	const bodySize = 2 * initialWindowSize   // Something that is not going to fit in a single window

	serverReturnedStatus := make(chan struct{})

	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			defer func() {
				close(serverReturnedStatus)
			}()
			return stream.Send(&testpb.StreamingOutputCallResponse{
				Payload: &testpb.Payload{
					Body: make([]byte, bodySize),
				},
			})
		},
	}
	if err := ss.Start(nil, grpc.WithInitialWindowSize(int32(initialWindowSize))); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	// This test makes sure we don't delete stream from server transport's
	// activeStreams list too aggressively.

	// 1. Make a long living stream RPC. So server's activeStream list is not
	// empty.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("FullDuplexCall= _, %v; want _, <nil>", err)
	}

	// 2. Wait for service handler to return status.
	//
	// This will trigger a stream cleanup code, which will eventually remove
	// this stream from activeStream.
	//
	// But the stream removal won't happen because it's supposed to be done
	// after the status is sent by loopyWriter, and the status send is blocked
	// by flow control.
	<-serverReturnedStatus

	// 3. GracefulStop (besides sending goaway) checks the number of
	// activeStreams.
	//
	// It will close the connection if there's no active streams. This won't
	// happen because of the pending stream. But if there's a bug in stream
	// cleanup that causes stream to be removed too aggressively, the connection
	// will be closed and the stream will be broken.
	gracefulStopDone := make(chan struct{})
	go func() {
		defer close(gracefulStopDone)
		ss.S.GracefulStop()
	}()

	// 4. Make sure the stream is not broken.
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("stream.Recv() = _, %v, want _, <nil>", err)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("stream.Recv() = _, %v, want _, io.EOF", err)
	}

	timer := time.NewTimer(time.Second)
	select {
	case <-gracefulStopDone:
		timer.Stop()
	case <-timer.C:
		t.Fatalf("s.GracefulStop() didn't finish within 1 second after the last RPC")
	}
}
