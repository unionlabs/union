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
	"fmt"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

type parentCtxkey struct{}
type firstInterceptorCtxkey struct{}
type secondInterceptorCtxkey struct{}
type baseInterceptorCtxKey struct{}

const (
	parentCtxVal            = "parent"
	firstInterceptorCtxVal  = "firstInterceptor"
	secondInterceptorCtxVal = "secondInterceptor"
	baseInterceptorCtxVal   = "baseInterceptor"
)

// TestUnaryClientInterceptor_ContextValuePropagation verifies that a unary
// interceptor receives context values specified in the context passed to the
// RPC call.
func (s) TestUnaryClientInterceptor_ContextValuePropagation(t *testing.T) {
	errCh := testutils.NewChannel()
	unaryInt := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.Send(fmt.Errorf("unaryInt got %q in context.Val, want %q", got, parentCtxVal))
		}
		errCh.Send(nil)
		return invoker(ctx, method, req, reply, cc, opts...)
	}

	// Start a stub server and use the above unary interceptor while creating a
	// ClientConn to it.
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	if err := ss.Start(nil, grpc.WithUnaryInterceptor(unaryInt)); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ss.Client.EmptyCall(context.WithValue(ctx, parentCtxkey{}, parentCtxVal), &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall() failed: %v", err)
	}
	val, err := errCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for unary interceptor to be invoked: %v", err)
	}
	if val != nil {
		t.Fatalf("unary interceptor failed: %v", val)
	}
}

// TestChainUnaryClientInterceptor_ContextValuePropagation verifies that a chain
// of unary interceptors receive context values specified in the original call
// as well as the ones specified by prior interceptors in the chain.
func (s) TestChainUnaryClientInterceptor_ContextValuePropagation(t *testing.T) {
	errCh := testutils.NewChannel()
	firstInt := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("first interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if ctx.Value(firstInterceptorCtxkey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("first interceptor should not have %T in context", firstInterceptorCtxkey{}))
		}
		if ctx.Value(secondInterceptorCtxkey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("first interceptor should not have %T in context", secondInterceptorCtxkey{}))
		}
		firstCtx := context.WithValue(ctx, firstInterceptorCtxkey{}, firstInterceptorCtxVal)
		return invoker(firstCtx, method, req, reply, cc, opts...)
	}

	secondInt := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("second interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if got, ok := ctx.Value(firstInterceptorCtxkey{}).(string); !ok || got != firstInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("second interceptor got %q in context.Val, want %q", got, firstInterceptorCtxVal))
		}
		if ctx.Value(secondInterceptorCtxkey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("second interceptor should not have %T in context", secondInterceptorCtxkey{}))
		}
		secondCtx := context.WithValue(ctx, secondInterceptorCtxkey{}, secondInterceptorCtxVal)
		return invoker(secondCtx, method, req, reply, cc, opts...)
	}

	lastInt := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("last interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if got, ok := ctx.Value(firstInterceptorCtxkey{}).(string); !ok || got != firstInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("last interceptor got %q in context.Val, want %q", got, firstInterceptorCtxVal))
		}
		if got, ok := ctx.Value(secondInterceptorCtxkey{}).(string); !ok || got != secondInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("last interceptor got %q in context.Val, want %q", got, secondInterceptorCtxVal))
		}
		errCh.SendContext(ctx, nil)
		return invoker(ctx, method, req, reply, cc, opts...)
	}

	// Start a stub server and use the above chain of interceptors while creating
	// a ClientConn to it.
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	if err := ss.Start(nil, grpc.WithChainUnaryInterceptor(firstInt, secondInt, lastInt)); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ss.Client.EmptyCall(context.WithValue(ctx, parentCtxkey{}, parentCtxVal), &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall() failed: %v", err)
	}
	val, err := errCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for unary interceptor to be invoked: %v", err)
	}
	if val != nil {
		t.Fatalf("unary interceptor failed: %v", val)
	}
}

// TestChainOnBaseUnaryClientInterceptor_ContextValuePropagation verifies that
// unary interceptors specified as a base interceptor or as a chain interceptor
// receive context values specified in the original call as well as the ones
// specified by interceptors in the chain.
func (s) TestChainOnBaseUnaryClientInterceptor_ContextValuePropagation(t *testing.T) {
	errCh := testutils.NewChannel()
	baseInt := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("base interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if ctx.Value(baseInterceptorCtxKey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("baseinterceptor should not have %T in context", baseInterceptorCtxKey{}))
		}
		baseCtx := context.WithValue(ctx, baseInterceptorCtxKey{}, baseInterceptorCtxVal)
		return invoker(baseCtx, method, req, reply, cc, opts...)
	}

	chainInt := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("chain interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if got, ok := ctx.Value(baseInterceptorCtxKey{}).(string); !ok || got != baseInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("chain interceptor got %q in context.Val, want %q", got, baseInterceptorCtxVal))
		}
		errCh.SendContext(ctx, nil)
		return invoker(ctx, method, req, reply, cc, opts...)
	}

	// Start a stub server and use the above chain of interceptors while creating
	// a ClientConn to it.
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	if err := ss.Start(nil, grpc.WithUnaryInterceptor(baseInt), grpc.WithChainUnaryInterceptor(chainInt)); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ss.Client.EmptyCall(context.WithValue(ctx, parentCtxkey{}, parentCtxVal), &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall() failed: %v", err)
	}
	val, err := errCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for unary interceptor to be invoked: %v", err)
	}
	if val != nil {
		t.Fatalf("unary interceptor failed: %v", val)
	}
}

// TestChainStreamClientInterceptor_ContextValuePropagation verifies that a
// chain of stream interceptors receive context values specified in the original
// call as well as the ones specified by the prior interceptors in the chain.
func (s) TestChainStreamClientInterceptor_ContextValuePropagation(t *testing.T) {
	errCh := testutils.NewChannel()
	firstInt := func(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("first interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if ctx.Value(firstInterceptorCtxkey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("first interceptor should not have %T in context", firstInterceptorCtxkey{}))
		}
		if ctx.Value(secondInterceptorCtxkey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("first interceptor should not have %T in context", secondInterceptorCtxkey{}))
		}
		firstCtx := context.WithValue(ctx, firstInterceptorCtxkey{}, firstInterceptorCtxVal)
		return streamer(firstCtx, desc, cc, method, opts...)
	}

	secondInt := func(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("second interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if got, ok := ctx.Value(firstInterceptorCtxkey{}).(string); !ok || got != firstInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("second interceptor got %q in context.Val, want %q", got, firstInterceptorCtxVal))
		}
		if ctx.Value(secondInterceptorCtxkey{}) != nil {
			errCh.SendContext(ctx, fmt.Errorf("second interceptor should not have %T in context", secondInterceptorCtxkey{}))
		}
		secondCtx := context.WithValue(ctx, secondInterceptorCtxkey{}, secondInterceptorCtxVal)
		return streamer(secondCtx, desc, cc, method, opts...)
	}

	lastInt := func(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
		if got, ok := ctx.Value(parentCtxkey{}).(string); !ok || got != parentCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("last interceptor got %q in context.Val, want %q", got, parentCtxVal))
		}
		if got, ok := ctx.Value(firstInterceptorCtxkey{}).(string); !ok || got != firstInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("last interceptor got %q in context.Val, want %q", got, firstInterceptorCtxVal))
		}
		if got, ok := ctx.Value(secondInterceptorCtxkey{}).(string); !ok || got != secondInterceptorCtxVal {
			errCh.SendContext(ctx, fmt.Errorf("last interceptor got %q in context.Val, want %q", got, secondInterceptorCtxVal))
		}
		errCh.SendContext(ctx, nil)
		return streamer(ctx, desc, cc, method, opts...)
	}

	// Start a stub server and use the above chain of interceptors while creating
	// a ClientConn to it.
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			if _, err := stream.Recv(); err != nil {
				return err
			}
			return stream.Send(&testpb.StreamingOutputCallResponse{})
		},
	}
	if err := ss.Start(nil, grpc.WithChainStreamInterceptor(firstInt, secondInt, lastInt)); err != nil {
		t.Fatalf("Failed to start stub server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ss.Client.FullDuplexCall(context.WithValue(ctx, parentCtxkey{}, parentCtxVal)); err != nil {
		t.Fatalf("ss.Client.FullDuplexCall() failed: %v", err)
	}
	val, err := errCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for stream interceptor to be invoked: %v", err)
	}
	if val != nil {
		t.Fatalf("stream interceptor failed: %v", val)
	}
}
