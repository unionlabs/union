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
	"fmt"
	"io"
	"net"
	"strings"
	"testing"
	"time"

	"golang.org/x/net/http2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/keepalive"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// TestGracefulClientOnGoAway attempts to ensure that when the server sends a
// GOAWAY (in this test, by configuring max connection age on the server), a
// client will never see an error.  This requires that the client is appraised
// of the GOAWAY and updates its state accordingly before the transport stops
// accepting new streams.  If a subconn is chosen by a picker and receives the
// goaway before creating the stream, an error will occur, but upon transparent
// retry, the clientconn will ensure a ready subconn is chosen.
func (s) TestGracefulClientOnGoAway(t *testing.T) {
	const maxConnAge = 100 * time.Millisecond
	const testTime = maxConnAge * 10

	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}

	s := grpc.NewServer(grpc.KeepaliveParams(keepalive.ServerParameters{MaxConnectionAge: maxConnAge}))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}
	go s.Serve(lis)

	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial server: %v", err)
	}
	defer cc.Close()
	c := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	endTime := time.Now().Add(testTime)
	for time.Now().Before(endTime) {
		if _, err := c.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			t.Fatalf("EmptyCall(_, _) = _, %v; want _, <nil>", err)
		}
	}
}

func (s) TestDetailedGoAwayErrorOnGracefulClosePropagatesToRPCError(t *testing.T) {
	rpcDoneOnClient := make(chan struct{})
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
			<-rpcDoneOnClient
			return status.Error(codes.Internal, "arbitrary status")
		},
	}
	sopts := []grpc.ServerOption{
		grpc.KeepaliveParams(keepalive.ServerParameters{
			MaxConnectionAge:      time.Millisecond * 100,
			MaxConnectionAgeGrace: time.Nanosecond, // ~instantaneously, but non-zero to avoid default
		}),
	}
	if err := ss.Start(sopts); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall = _, %v, want _, <nil>", ss.Client, err)
	}
	const expectedErrorMessageSubstring = "received prior goaway: code: NO_ERROR"
	_, err = stream.Recv()
	close(rpcDoneOnClient)
	if err == nil || !strings.Contains(err.Error(), expectedErrorMessageSubstring) {
		t.Fatalf("%v.Recv() = _, %v, want _, rpc error containing substring: %q", stream, err, expectedErrorMessageSubstring)
	}
}

func (s) TestDetailedGoAwayErrorOnAbruptClosePropagatesToRPCError(t *testing.T) {
	grpctest.TLogger.ExpectError("Client received GoAway with error code ENHANCE_YOUR_CALM and debug data equal to ASCII \"too_many_pings\"")
	// set the min keepalive time very low so that this test can take
	// a reasonable amount of time
	prev := internal.KeepaliveMinPingTime
	internal.KeepaliveMinPingTime = time.Millisecond
	defer func() { internal.KeepaliveMinPingTime = prev }()

	rpcDoneOnClient := make(chan struct{})
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
			<-rpcDoneOnClient
			return status.Error(codes.Internal, "arbitrary status")
		},
	}
	sopts := []grpc.ServerOption{
		grpc.KeepaliveEnforcementPolicy(keepalive.EnforcementPolicy{
			MinTime: time.Second * 1000, /* arbitrary, large value */
		}),
	}
	dopts := []grpc.DialOption{
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			Time:                time.Millisecond,   /* should trigger "too many pings" error quickly */
			Timeout:             time.Second * 1000, /* arbitrary, large value */
			PermitWithoutStream: false,
		}),
	}
	if err := ss.Start(sopts, dopts...); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall = _, %v, want _, <nil>", ss.Client, err)
	}
	const expectedErrorMessageSubstring = `received prior goaway: code: ENHANCE_YOUR_CALM, debug data: "too_many_pings"`
	_, err = stream.Recv()
	close(rpcDoneOnClient)
	if err == nil || !strings.Contains(err.Error(), expectedErrorMessageSubstring) {
		t.Fatalf("%v.Recv() = _, %v, want _, rpc error containing substring: |%v|", stream, err, expectedErrorMessageSubstring)
	}
}

func (s) TestClientConnCloseAfterGoAwayWithActiveStream(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testClientConnCloseAfterGoAwayWithActiveStream(t, e)
	}
}

func testClientConnCloseAfterGoAwayWithActiveStream(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.FullDuplexCall(ctx); err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want _, <nil>", tc, err)
	}
	done := make(chan struct{})
	go func() {
		te.srv.GracefulStop()
		close(done)
	}()
	time.Sleep(50 * time.Millisecond)
	cc.Close()
	timeout := time.NewTimer(time.Second)
	select {
	case <-done:
	case <-timeout.C:
		t.Fatalf("Test timed-out.")
	}
}

func (s) TestServerGoAway(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testServerGoAway(t, e)
	}
}

func testServerGoAway(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	// Finish an RPC to make sure the connection is good.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	ch := make(chan struct{})
	go func() {
		te.srv.GracefulStop()
		close(ch)
	}()
	// Loop until the server side GoAway signal is propagated to the client.
	for {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil && status.Code(err) != codes.DeadlineExceeded {
			cancel()
			break
		}
		cancel()
	}
	// A new RPC should fail.
	ctx, cancel = context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable && status.Code(err) != codes.Internal {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s or %s", err, codes.Unavailable, codes.Internal)
	}
	<-ch
	awaitNewConnLogOutput()
}

func (s) TestServerGoAwayPendingRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testServerGoAwayPendingRPC(t, e)
	}
}

func testServerGoAwayPendingRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"transport: http2Client.notifyError got notified that the client transport was broken EOF",
		"grpc: addrConn.transportMonitor exits due to: grpc: the connection is closing",
		"grpc: addrConn.resetTransport failed to create client transport: connection error",
	)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	// Finish an RPC to make sure the connection is good.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("%v.EmptyCall(_, _, _) = _, %v, want _, <nil>", tc, err)
	}
	ch := make(chan struct{})
	go func() {
		te.srv.GracefulStop()
		close(ch)
	}()
	// Loop until the server side GoAway signal is propagated to the client.
	start := time.Now()
	errored := false
	for time.Since(start) < time.Second {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		_, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true))
		cancel()
		if err != nil {
			errored = true
			break
		}
	}
	if !errored {
		t.Fatalf("GoAway never received by client")
	}
	respParam := []*testpb.ResponseParameters{{Size: 1}}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(100))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	// The existing RPC should be still good to proceed.
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(_) = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
	}
	// The RPC will run until canceled.
	cancel()
	<-ch
	awaitNewConnLogOutput()
}

func (s) TestServerMultipleGoAwayPendingRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testServerMultipleGoAwayPendingRPC(t, e)
	}
}

func testServerMultipleGoAwayPendingRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"transport: http2Client.notifyError got notified that the client transport was broken EOF",
		"grpc: addrConn.transportMonitor exits due to: grpc: the connection is closing",
		"grpc: addrConn.resetTransport failed to create client transport: connection error",
	)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	// Finish an RPC to make sure the connection is good.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("%v.EmptyCall(_, _, _) = _, %v, want _, <nil>", tc, err)
	}
	ch1 := make(chan struct{})
	go func() {
		te.srv.GracefulStop()
		close(ch1)
	}()
	ch2 := make(chan struct{})
	go func() {
		te.srv.GracefulStop()
		close(ch2)
	}()
	// Loop until the server side GoAway signal is propagated to the client.

	for {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
			cancel()
			break
		}
		cancel()
	}
	select {
	case <-ch1:
		t.Fatal("GracefulStop() terminated early")
	case <-ch2:
		t.Fatal("GracefulStop() terminated early")
	default:
	}
	respParam := []*testpb.ResponseParameters{
		{
			Size: 1,
		},
	}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(100))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	// The existing RPC should be still good to proceed.
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() = %v, want <nil>", stream, err)
	}

	<-ch1
	<-ch2
	cancel()
	awaitNewConnLogOutput()
}

func (s) TestConcurrentClientConnCloseAndServerGoAway(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testConcurrentClientConnCloseAndServerGoAway(t, e)
	}
}

func testConcurrentClientConnCloseAndServerGoAway(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"transport: http2Client.notifyError got notified that the client transport was broken EOF",
		"grpc: addrConn.transportMonitor exits due to: grpc: the connection is closing",
		"grpc: addrConn.resetTransport failed to create client transport: connection error",
	)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("%v.EmptyCall(_, _, _) = _, %v, want _, <nil>", tc, err)
	}
	ch := make(chan struct{})
	// Close ClientConn and Server concurrently.
	go func() {
		te.srv.GracefulStop()
		close(ch)
	}()
	go func() {
		cc.Close()
	}()
	<-ch
}

func (s) TestConcurrentServerStopAndGoAway(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testConcurrentServerStopAndGoAway(t, e)
	}
}

func testConcurrentServerStopAndGoAway(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"transport: http2Client.notifyError got notified that the client transport was broken EOF",
		"grpc: addrConn.transportMonitor exits due to: grpc: the connection is closing",
		"grpc: addrConn.resetTransport failed to create client transport: connection error",
	)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}

	// Finish an RPC to make sure the connection is good.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("%v.EmptyCall(_, _, _) = _, %v, want _, <nil>", tc, err)
	}

	ch := make(chan struct{})
	go func() {
		te.srv.GracefulStop()
		close(ch)
	}()
	// Loop until the server side GoAway signal is propagated to the client.
	for {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
			cancel()
			break
		}
		cancel()
	}
	// Stop the server and close all the connections.
	te.srv.Stop()
	respParam := []*testpb.ResponseParameters{
		{
			Size: 1,
		},
	}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(100))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	sendStart := time.Now()
	for {
		if err := stream.Send(req); err == io.EOF {
			// stream.Send should eventually send io.EOF
			break
		} else if err != nil {
			// Send should never return a transport-level error.
			t.Fatalf("stream.Send(%v) = %v; want <nil or io.EOF>", req, err)
		}
		if time.Since(sendStart) > 2*time.Second {
			t.Fatalf("stream.Send(_) did not return io.EOF after 2s")
		}
		time.Sleep(time.Millisecond)
	}
	if _, err := stream.Recv(); err == nil || err == io.EOF {
		t.Fatalf("%v.Recv() = _, %v, want _, <non-nil, non-EOF>", stream, err)
	}
	<-ch
	awaitNewConnLogOutput()
}

// Proxies typically send GO_AWAY followed by connection closure a minute or so later. This
// test ensures that the connection is re-created after GO_AWAY and not affected by the
// subsequent (old) connection closure.
func (s) TestGoAwayThenClose(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	lis1, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	s1 := grpc.NewServer()
	defer s1.Stop()
	ts := &funcServer{
		unaryCall: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{}, nil
		},
		fullDuplexCall: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			if err := stream.Send(&testpb.StreamingOutputCallResponse{}); err != nil {
				t.Errorf("unexpected error from send: %v", err)
				return err
			}
			// Wait forever.
			_, err := stream.Recv()
			if err == nil {
				t.Error("expected to never receive any message")
			}
			return err
		},
	}
	testgrpc.RegisterTestServiceServer(s1, ts)
	go s1.Serve(lis1)

	conn2Established := grpcsync.NewEvent()
	lis2, err := listenWithNotifyingListener("tcp", "localhost:0", conn2Established)
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	s2 := grpc.NewServer()
	defer s2.Stop()
	testgrpc.RegisterTestServiceServer(s2, ts)

	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{
		{Addr: lis1.Addr().String()},
		{Addr: lis2.Addr().String()},
	}})
	cc, err := grpc.DialContext(ctx, r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Error creating client: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)

	t.Log("Waiting for the ClientConn to enter READY state.")
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)

	// We make a streaming RPC and do an one-message-round-trip to make sure
	// it's created on connection 1.
	//
	// We use a long-lived RPC because it will cause GracefulStop to send
	// GO_AWAY, but the connection won't get closed until the server stops and
	// the client receives the error.
	t.Log("Creating first streaming RPC to server 1.")
	stream, err := client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("FullDuplexCall(_) = _, %v; want _, nil", err)
	}
	if _, err = stream.Recv(); err != nil {
		t.Fatalf("unexpected error from first recv: %v", err)
	}

	go s2.Serve(lis2)

	t.Log("Gracefully stopping server 1.")
	go s1.GracefulStop()

	t.Log("Waiting for the ClientConn to enter IDLE state.")
	testutils.AwaitState(ctx, t, cc, connectivity.Idle)

	t.Log("Performing another RPC to create a connection to server 2.")
	if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("UnaryCall(_) = _, %v; want _, nil", err)
	}

	t.Log("Waiting for a connection to server 2.")
	select {
	case <-conn2Established.Done():
	case <-ctx.Done():
		t.Fatalf("timed out waiting for connection 2 to be established")
	}

	// Close the listener for server2 to prevent it from allowing new connections.
	lis2.Close()

	t.Log("Hard closing connection 1.")
	s1.Stop()

	t.Log("Waiting for the first stream to error.")
	if _, err = stream.Recv(); err == nil {
		t.Fatal("expected the stream to die, but got a successful Recv")
	}

	t.Log("Ensuring connection 2 is stable.")
	for i := 0; i < 10; i++ {
		if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
			t.Fatalf("UnaryCall(_) = _, %v; want _, nil", err)
		}
	}
}

// TestGoAwayStreamIDSmallerThanCreatedStreams tests the scenario where a server
// sends a goaway with a stream id that is smaller than some created streams on
// the client, while the client is simultaneously creating new streams. This
// should not induce a deadlock.
func (s) TestGoAwayStreamIDSmallerThanCreatedStreams(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}

	ctCh := testutils.NewChannel()
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("error in lis.Accept(): %v", err)
		}
		ct := newClientTester(t, conn)
		ctCh.Send(ct)
	}()

	cc, err := grpc.Dial(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("error dialing: %v", err)
	}
	defer cc.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	val, err := ctCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout waiting for client transport (should be given after http2 creation)")
	}
	ct := val.(*clientTester)

	tc := testgrpc.NewTestServiceClient(cc)
	someStreamsCreated := grpcsync.NewEvent()
	goAwayWritten := grpcsync.NewEvent()
	go func() {
		for i := 0; i < 20; i++ {
			if i == 10 {
				<-goAwayWritten.Done()
			}
			tc.FullDuplexCall(ctx)
			if i == 4 {
				someStreamsCreated.Fire()
			}
		}
	}()

	<-someStreamsCreated.Done()
	ct.writeGoAway(1, http2.ErrCodeNo, []byte{})
	goAwayWritten.Fire()
}

// TestTwoGoAwayPingFrames tests the scenario where you get two go away ping
// frames from the client during graceful shutdown. This should not crash the
// server.
func (s) TestTwoGoAwayPingFrames(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()
	s := grpc.NewServer()
	defer s.Stop()
	go s.Serve(lis)

	conn, err := net.DialTimeout("tcp", lis.Addr().String(), defaultTestTimeout)
	if err != nil {
		t.Fatalf("Failed to dial: %v", err)
	}

	st := newServerTesterFromConn(t, conn)
	st.greet()
	pingReceivedClientSide := testutils.NewChannel()
	go func() {
		for {
			f, err := st.readFrame()
			if err != nil {
				return
			}
			switch f.(type) {
			case *http2.GoAwayFrame:
			case *http2.PingFrame:
				pingReceivedClientSide.Send(nil)
			default:
				t.Errorf("server tester received unexpected frame type %T", f)
			}
		}
	}()
	gsDone := testutils.NewChannel()
	go func() {
		s.GracefulStop()
		gsDone.Send(nil)
	}()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := pingReceivedClientSide.Receive(ctx); err != nil {
		t.Fatalf("Error waiting for ping frame client side from graceful shutdown: %v", err)
	}
	// Write two goaway pings here.
	st.writePing(true, [8]byte{1, 6, 1, 8, 0, 3, 3, 9})
	st.writePing(true, [8]byte{1, 6, 1, 8, 0, 3, 3, 9})
	// Close the conn to finish up the Graceful Shutdown process.
	conn.Close()
	if _, err := gsDone.Receive(ctx); err != nil {
		t.Fatalf("Error waiting for graceful shutdown of the server: %v", err)
	}
}

// TestClientSendsAGoAway tests the scenario where you get a go away ping
// frames from the client during graceful shutdown.
func (s) TestClientSendsAGoAway(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}
	defer lis.Close()
	goAwayReceived := make(chan struct{})
	errCh := make(chan error)
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("error in lis.Accept(): %v", err)
		}
		ct := newClientTester(t, conn)
		defer ct.conn.Close()
		for {
			f, err := ct.fr.ReadFrame()
			if err != nil {
				errCh <- fmt.Errorf("error reading frame: %v", err)
				return
			}
			switch fr := f.(type) {
			case *http2.GoAwayFrame:
				fr = f.(*http2.GoAwayFrame)
				if fr.ErrCode == http2.ErrCodeNo {
					t.Logf("GoAway received from client")
					close(goAwayReceived)
					return
				}
			default:
				t.Errorf("server tester received unexpected frame type %T", f)
				errCh <- fmt.Errorf("server tester received unexpected frame type %T", f)
				close(errCh)
			}
		}
	}()

	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("error dialing: %v", err)
	}
	cc.Connect()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	testutils.AwaitState(ctx, t, cc, connectivity.Ready)
	cc.Close()
	select {
	case <-goAwayReceived:
	case err := <-errCh:
		t.Errorf("Error receiving the goAway: %v", err)
	case <-ctx.Done():
		t.Errorf("Context timed out")
	}
}
