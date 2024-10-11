/*
 *
 * Copyright 2022 gRPC authors.

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

package test

import (
	"context"
	"strings"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/stubserver"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/status"
)

// TestInvoke verifies a straightforward invocation of ClientConn.Invoke().
func (s) TestInvoke(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := ss.CC.Invoke(ctx, "/grpc.testing.TestService/EmptyCall", &testpb.Empty{}, &testpb.Empty{}); err != nil {
		t.Fatalf("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") failed: %v", err)
	}
}

// TestInvokeLargeErr verifies an invocation of ClientConn.Invoke() where the
// server returns a really large error message.
func (s) TestInvokeLargeErr(t *testing.T) {
	largeErrorStr := strings.Repeat("A", 1024*1024)
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, status.Error(codes.Internal, largeErrorStr)
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	err := ss.CC.Invoke(ctx, "/grpc.testing.TestService/EmptyCall", &testpb.Empty{}, &testpb.Empty{})
	if err == nil {
		t.Fatal("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") succeeded when expected to fail")
	}
	st, ok := status.FromError(err)
	if !ok {
		t.Fatal("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") received non-status error")
	}
	if status.Code(err) != codes.Internal || st.Message() != largeErrorStr {
		t.Fatalf("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") failed with error: %v, want an error of code %d and desc size %d", err, codes.Internal, len(largeErrorStr))
	}
}

// TestInvokeErrorSpecialChars tests an invocation of ClientConn.Invoke() and
// verifies that error messages don't get mangled.
func (s) TestInvokeErrorSpecialChars(t *testing.T) {
	const weirdError = "format verbs: %v%s"
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, status.Error(codes.Internal, weirdError)
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	err := ss.CC.Invoke(ctx, "/grpc.testing.TestService/EmptyCall", &testpb.Empty{}, &testpb.Empty{})
	if err == nil {
		t.Fatal("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") succeeded when expected to fail")
	}
	st, ok := status.FromError(err)
	if !ok {
		t.Fatal("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") received non-status error")
	}
	if status.Code(err) != codes.Internal || st.Message() != weirdError {
		t.Fatalf("grpc.Invoke(\"/grpc.testing.TestService/EmptyCall\") failed with error: %v, want %v", err, weirdError)
	}
}

// TestInvokeCancel tests an invocation of ClientConn.Invoke() with a cancelled
// context and verifies that the request is not actually sent to the server.
func (s) TestInvokeCancel(t *testing.T) {
	cancelled := 0
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			cancelled++
			return &testpb.Empty{}, nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	for i := 0; i < 100; i++ {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		cancel()
		ss.CC.Invoke(ctx, "/grpc.testing.TestService/EmptyCall", &testpb.Empty{}, &testpb.Empty{})
	}
	if cancelled != 0 {
		t.Fatalf("server received %d of 100 cancelled requests", cancelled)
	}
}

// TestInvokeCancelClosedNonFail tests an invocation of ClientConn.Invoke() with
// a cancelled non-failfast RPC on a closed ClientConn and verifies that the
// call terminates with an error.
func (s) TestInvokeCancelClosedNonFailFast(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ss.CC.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	cancel()
	if err := ss.CC.Invoke(ctx, "/grpc.testing.TestService/EmptyCall", &testpb.Empty{}, &testpb.Empty{}, grpc.WaitForReady(true)); err == nil {
		t.Fatal("ClientConn.Invoke() on closed connection succeeded when expected to fail")
	}
}
