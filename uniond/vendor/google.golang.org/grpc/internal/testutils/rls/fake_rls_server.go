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

// Package rls contains utilities for RouteLookupService e2e tests.
package rls

import (
	"context"
	"net"
	"sync"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	rlsgrpc "google.golang.org/grpc/internal/proto/grpc_lookup_v1"
	rlspb "google.golang.org/grpc/internal/proto/grpc_lookup_v1"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/status"
)

// RouteLookupResponse wraps an RLS response and the associated error to be sent
// to a client when the RouteLookup RPC is invoked.
type RouteLookupResponse struct {
	Resp *rlspb.RouteLookupResponse
	Err  error
}

// SetupFakeRLSServer starts and returns a fake RouteLookupService server
// listening on the given listener or on a random local port. Also returns a
// channel for tests to get notified whenever the RouteLookup RPC is invoked on
// the fake server.
//
// This function sets up the fake server to respond with an empty response for
// the RouteLookup RPCs. Tests can override this by calling the
// SetResponseCallback() method on the returned fake server.
func SetupFakeRLSServer(t *testing.T, lis net.Listener, opts ...grpc.ServerOption) (*FakeRouteLookupServer, chan struct{}) {
	s, cancel := StartFakeRouteLookupServer(t, lis, opts...)
	t.Logf("Started fake RLS server at %q", s.Address)

	ch := make(chan struct{}, 1)
	s.SetRequestCallback(func(*rlspb.RouteLookupRequest) {
		select {
		case ch <- struct{}{}:
		default:
		}
	})
	t.Cleanup(cancel)
	return s, ch
}

// FakeRouteLookupServer is a fake implementation of the RouteLookupService.
//
// It is safe for concurrent use.
type FakeRouteLookupServer struct {
	rlsgrpc.UnimplementedRouteLookupServiceServer
	Address string

	mu     sync.Mutex
	respCb func(context.Context, *rlspb.RouteLookupRequest) *RouteLookupResponse
	reqCb  func(*rlspb.RouteLookupRequest)
}

// StartFakeRouteLookupServer starts a fake RLS server listening for requests on
// lis. If lis is nil, it creates a new listener on a random local port. The
// returned cancel function should be invoked by the caller upon completion of
// the test.
func StartFakeRouteLookupServer(t *testing.T, lis net.Listener, opts ...grpc.ServerOption) (*FakeRouteLookupServer, func()) {
	t.Helper()

	if lis == nil {
		var err error
		lis, err = testutils.LocalTCPListener()
		if err != nil {
			t.Fatalf("net.Listen() failed: %v", err)
		}
	}

	s := &FakeRouteLookupServer{Address: lis.Addr().String()}
	server := grpc.NewServer(opts...)
	rlsgrpc.RegisterRouteLookupServiceServer(server, s)
	go server.Serve(lis)
	return s, func() { server.Stop() }
}

// RouteLookup implements the RouteLookupService.
func (s *FakeRouteLookupServer) RouteLookup(ctx context.Context, req *rlspb.RouteLookupRequest) (*rlspb.RouteLookupResponse, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.reqCb != nil {
		s.reqCb(req)
	}
	if err := ctx.Err(); err != nil {
		return nil, status.Error(codes.DeadlineExceeded, err.Error())
	}
	if s.respCb == nil {
		return &rlspb.RouteLookupResponse{}, nil
	}
	resp := s.respCb(ctx, req)
	return resp.Resp, resp.Err
}

// SetResponseCallback sets a callback to be invoked on every RLS request. If
// this callback is set, the response returned by the fake server depends on the
// value returned by the callback. If this callback is not set, the fake server
// responds with an empty response.
func (s *FakeRouteLookupServer) SetResponseCallback(f func(context.Context, *rlspb.RouteLookupRequest) *RouteLookupResponse) {
	s.mu.Lock()
	s.respCb = f
	s.mu.Unlock()
}

// SetRequestCallback sets a callback to be invoked on every RLS request. The
// callback is given the incoming request, and tests can use this to verify that
// the request matches its expectations.
func (s *FakeRouteLookupServer) SetRequestCallback(f func(*rlspb.RouteLookupRequest)) {
	s.mu.Lock()
	s.reqCb = f
	s.mu.Unlock()
}
