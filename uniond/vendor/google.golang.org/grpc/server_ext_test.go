/*
 *
 * Copyright 2023 gRPC authors.
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

package grpc_test

import (
	"context"
	"io"
	"runtime"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// TestServer_MaxHandlers ensures that no more than MaxConcurrentStreams server
// handlers are active at one time.
func (s) TestServer_MaxHandlers(t *testing.T) {
	started := make(chan struct{})
	blockCalls := grpcsync.NewEvent()

	// This stub server does not properly respect the stream context, so it will
	// not exit when the context is canceled.
	ss := stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			started <- struct{}{}
			<-blockCalls.Done()
			return nil
		},
	}
	if err := ss.Start([]grpc.ServerOption{grpc.MaxConcurrentStreams(1)}); err != nil {
		t.Fatal("Error starting server:", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Start one RPC to the server.
	ctx1, cancel1 := context.WithCancel(ctx)
	_, err := ss.Client.FullDuplexCall(ctx1)
	if err != nil {
		t.Fatal("Error staring call:", err)
	}

	// Wait for the handler to be invoked.
	select {
	case <-started:
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for RPC to start on server.")
	}

	// Cancel it on the client.  The server handler will still be running.
	cancel1()

	ctx2, cancel2 := context.WithCancel(ctx)
	defer cancel2()
	s, err := ss.Client.FullDuplexCall(ctx2)
	if err != nil {
		t.Fatal("Error staring call:", err)
	}

	// After 100ms, allow the first call to unblock.  That should allow the
	// second RPC to run and finish.
	select {
	case <-started:
		blockCalls.Fire()
		t.Fatalf("RPC started unexpectedly.")
	case <-time.After(100 * time.Millisecond):
		blockCalls.Fire()
	}

	select {
	case <-started:
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for second RPC to start on server.")
	}
	if _, err := s.Recv(); err != io.EOF {
		t.Fatal("Received unexpected RPC error:", err)
	}
}

// Tests the case where the stream worker goroutine option is enabled, and a
// number of RPCs are initiated around the same time that Stop() is called. This
// used to result in a write to a closed channel. This test verifies that there
// is no panic.
func (s) TestStreamWorkers_RPCsAndStop(t *testing.T) {
	ss := stubserver.StartTestService(t, nil, grpc.NumStreamWorkers(uint32(runtime.NumCPU())))
	// This deferred stop takes care of stopping the server when one of the
	// below grpc.Dials fail, and the test exits early.
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	const numChannels = 20
	const numRPCLoops = 20

	// Create a bunch of clientconns and ensure that they are READY by making an
	// RPC on them.
	ccs := make([]*grpc.ClientConn, numChannels)
	for i := 0; i < numChannels; i++ {
		var err error
		ccs[i], err = grpc.NewClient(ss.Address, grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			t.Fatalf("[iteration: %d] grpc.NewClient(%s) failed: %v", i, ss.Address, err)
		}
		defer ccs[i].Close()
		client := testgrpc.NewTestServiceClient(ccs[i])
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
	}

	// Make a bunch of concurrent RPCs on the above clientconns. These will
	// eventually race with Stop(), and will start to fail.
	var wg sync.WaitGroup
	for i := 0; i < numChannels; i++ {
		client := testgrpc.NewTestServiceClient(ccs[i])
		for j := 0; j < numRPCLoops; j++ {
			wg.Add(1)
			go func(client testgrpc.TestServiceClient) {
				defer wg.Done()
				for {
					_, err := client.EmptyCall(ctx, &testpb.Empty{})
					if err == nil {
						continue
					}
					if code := status.Code(err); code == codes.Unavailable {
						// Once Stop() has been called on the server, we expect
						// subsequent calls to fail with Unavailable.
						return
					}
					t.Errorf("EmptyCall() failed: %v", err)
					return
				}
			}(client)
		}
	}

	// Call Stop() concurrently with the above RPC attempts.
	ss.Stop()
	wg.Wait()
}

// Tests the case where the stream worker goroutine option is enabled, and both
// Stop() and GracefulStop() care called. This used to result in a close of a
// closed channel. This test verifies that there is no panic.
func (s) TestStreamWorkers_GracefulStopAndStop(t *testing.T) {
	ss := stubserver.StartTestService(t, nil, grpc.NumStreamWorkers(uint32(runtime.NumCPU())))
	defer ss.Stop()

	if err := ss.StartClient(grpc.WithTransportCredentials(insecure.NewCredentials())); err != nil {
		t.Fatalf("Failed to create client to stub server: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(ss.CC)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	ss.S.GracefulStop()
}

// Tests the WaitForHandlers ServerOption by leaving an RPC running while Stop
// is called, and ensures Stop doesn't return until the handler returns.
func (s) TestServer_WaitForHandlers(t *testing.T) {
	started := grpcsync.NewEvent()
	blockCalls := grpcsync.NewEvent()

	// This stub server does not properly respect the stream context, so it will
	// not exit when the context is canceled.
	ss := stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			started.Fire()
			<-blockCalls.Done()
			return nil
		},
	}
	if err := ss.Start([]grpc.ServerOption{grpc.WaitForHandlers(true)}); err != nil {
		t.Fatal("Error starting server:", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Start one RPC to the server.
	ctx1, cancel1 := context.WithCancel(ctx)
	_, err := ss.Client.FullDuplexCall(ctx1)
	if err != nil {
		t.Fatal("Error staring call:", err)
	}

	// Wait for the handler to be invoked.
	select {
	case <-started.Done():
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for RPC to start on server.")
	}

	// Cancel it on the client.  The server handler will still be running.
	cancel1()

	// Close the connection.  This might be sufficient to allow the server to
	// return if it doesn't properly wait for outstanding method handlers to
	// return.
	ss.CC.Close()

	// Try to Stop() the server, which should block indefinitely (until
	// blockCalls is fired).
	stopped := grpcsync.NewEvent()
	go func() {
		ss.S.Stop()
		stopped.Fire()
	}()

	// Wait 100ms and ensure stopped does not fire.
	select {
	case <-stopped.Done():
		trace := make([]byte, 4096)
		trace = trace[0:runtime.Stack(trace, true)]
		blockCalls.Fire()
		t.Fatalf("Server returned from Stop() illegally.  Stack trace:\n%v", string(trace))
	case <-time.After(100 * time.Millisecond):
		// Success; unblock the call and wait for stopped.
		blockCalls.Fire()
	}

	select {
	case <-stopped.Done():
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for second RPC to start on server.")
	}
}

// Tests that GracefulStop will wait for all method handlers to return by
// blocking a handler and ensuring GracefulStop doesn't return until after it is
// unblocked.
func (s) TestServer_GracefulStopWaits(t *testing.T) {
	started := grpcsync.NewEvent()
	blockCalls := grpcsync.NewEvent()

	// This stub server does not properly respect the stream context, so it will
	// not exit when the context is canceled.
	ss := stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			started.Fire()
			<-blockCalls.Done()
			return nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatal("Error starting server:", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Start one RPC to the server.
	ctx1, cancel1 := context.WithCancel(ctx)
	_, err := ss.Client.FullDuplexCall(ctx1)
	if err != nil {
		t.Fatal("Error staring call:", err)
	}

	// Wait for the handler to be invoked.
	select {
	case <-started.Done():
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for RPC to start on server.")
	}

	// Cancel it on the client.  The server handler will still be running.
	cancel1()

	// Close the connection.  This might be sufficient to allow the server to
	// return if it doesn't properly wait for outstanding method handlers to
	// return.
	ss.CC.Close()

	// Try to Stop() the server, which should block indefinitely (until
	// blockCalls is fired).
	stopped := grpcsync.NewEvent()
	go func() {
		ss.S.GracefulStop()
		stopped.Fire()
	}()

	// Wait 100ms and ensure stopped does not fire.
	select {
	case <-stopped.Done():
		trace := make([]byte, 4096)
		trace = trace[0:runtime.Stack(trace, true)]
		blockCalls.Fire()
		t.Fatalf("Server returned from Stop() illegally.  Stack trace:\n%v", string(trace))
	case <-time.After(100 * time.Millisecond):
		// Success; unblock the call and wait for stopped.
		blockCalls.Fire()
	}

	select {
	case <-stopped.Done():
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for second RPC to start on server.")
	}
}
