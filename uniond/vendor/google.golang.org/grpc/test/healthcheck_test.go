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
	"errors"
	"fmt"
	"net"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/health"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"

	healthgrpc "google.golang.org/grpc/health/grpc_health_v1"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

var testHealthCheckFunc = internal.HealthCheckFunc

func newTestHealthServer() *testHealthServer {
	return newTestHealthServerWithWatchFunc(defaultWatchFunc)
}

func newTestHealthServerWithWatchFunc(f healthWatchFunc) *testHealthServer {
	return &testHealthServer{
		watchFunc: f,
		update:    make(chan struct{}, 1),
		status:    make(map[string]healthpb.HealthCheckResponse_ServingStatus),
	}
}

// defaultWatchFunc will send a HealthCheckResponse to the client whenever SetServingStatus is called.
func defaultWatchFunc(s *testHealthServer, in *healthpb.HealthCheckRequest, stream healthgrpc.Health_WatchServer) error {
	if in.Service != "foo" {
		return status.Error(codes.FailedPrecondition,
			"the defaultWatchFunc only handles request with service name to be \"foo\"")
	}
	var done bool
	for {
		select {
		case <-stream.Context().Done():
			done = true
		case <-s.update:
		}
		if done {
			break
		}
		s.mu.Lock()
		resp := &healthpb.HealthCheckResponse{
			Status: s.status[in.Service],
		}
		s.mu.Unlock()
		stream.SendMsg(resp)
	}
	return nil
}

type healthWatchFunc func(*testHealthServer, *healthpb.HealthCheckRequest, healthgrpc.Health_WatchServer) error

type testHealthServer struct {
	healthgrpc.UnimplementedHealthServer
	watchFunc healthWatchFunc
	mu        sync.Mutex
	status    map[string]healthpb.HealthCheckResponse_ServingStatus
	update    chan struct{}
}

func (s *testHealthServer) Check(context.Context, *healthpb.HealthCheckRequest) (*healthpb.HealthCheckResponse, error) {
	return &healthpb.HealthCheckResponse{
		Status: healthpb.HealthCheckResponse_SERVING,
	}, nil
}

func (s *testHealthServer) Watch(in *healthpb.HealthCheckRequest, stream healthgrpc.Health_WatchServer) error {
	return s.watchFunc(s, in, stream)
}

// SetServingStatus is called when need to reset the serving status of a service
// or insert a new service entry into the statusMap.
func (s *testHealthServer) SetServingStatus(service string, status healthpb.HealthCheckResponse_ServingStatus) {
	s.mu.Lock()
	s.status[service] = status
	select {
	case <-s.update:
	default:
	}
	s.update <- struct{}{}
	s.mu.Unlock()
}

func setupHealthCheckWrapper() (hcEnterChan chan struct{}, hcExitChan chan struct{}, wrapper internal.HealthChecker) {
	hcEnterChan = make(chan struct{})
	hcExitChan = make(chan struct{})
	wrapper = func(ctx context.Context, newStream func(string) (any, error), update func(connectivity.State, error), service string) error {
		close(hcEnterChan)
		defer close(hcExitChan)
		return testHealthCheckFunc(ctx, newStream, update, service)
	}
	return
}

func setupServer(t *testing.T, watchFunc healthWatchFunc) (*grpc.Server, net.Listener, *testHealthServer) {
	t.Helper()

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("net.Listen() failed: %v", err)
	}

	var ts *testHealthServer
	if watchFunc != nil {
		ts = newTestHealthServerWithWatchFunc(watchFunc)
	} else {
		ts = newTestHealthServer()
	}
	s := grpc.NewServer()
	healthgrpc.RegisterHealthServer(s, ts)
	testgrpc.RegisterTestServiceServer(s, &testServer{})
	go s.Serve(lis)
	t.Cleanup(func() { s.Stop() })
	return s, lis, ts
}

type clientConfig struct {
	balancerName               string
	testHealthCheckFuncWrapper internal.HealthChecker
	extraDialOption            []grpc.DialOption
}

func setupClient(t *testing.T, c *clientConfig) (*grpc.ClientConn, *manual.Resolver) {
	t.Helper()

	r := manual.NewBuilderWithScheme("whatever")
	opts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
	}
	if c != nil {
		if c.balancerName != "" {
			opts = append(opts, grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, c.balancerName)))
		}
		if c.testHealthCheckFuncWrapper != nil {
			opts = append(opts, internal.WithHealthCheckFunc.(func(internal.HealthChecker) grpc.DialOption)(c.testHealthCheckFuncWrapper))
		}
		opts = append(opts, c.extraDialOption...)
	}

	cc, err := grpc.Dial(r.Scheme()+":///test.server", opts...)
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })
	return cc, r
}

func (s) TestHealthCheckWatchStateChange(t *testing.T) {
	_, lis, ts := setupServer(t, nil)

	// The table below shows the expected series of addrConn connectivity transitions when server
	// updates its health status. As there's only one addrConn corresponds with the ClientConn in this
	// test, we use ClientConn's connectivity state as the addrConn connectivity state.
	//+------------------------------+-------------------------------------------+
	//| Health Check Returned Status | Expected addrConn Connectivity Transition |
	//+------------------------------+-------------------------------------------+
	//| NOT_SERVING                  | ->TRANSIENT FAILURE                       |
	//| SERVING                      | ->READY                                   |
	//| SERVICE_UNKNOWN              | ->TRANSIENT FAILURE                       |
	//| SERVING                      | ->READY                                   |
	//| UNKNOWN                      | ->TRANSIENT FAILURE                       |
	//+------------------------------+-------------------------------------------+
	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_NOT_SERVING)

	cc, r := setupClient(t, nil)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitNotState(ctx, t, cc, connectivity.Idle)
	testutils.AwaitNotState(ctx, t, cc, connectivity.Connecting)
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)
	if s := cc.GetState(); s != connectivity.TransientFailure {
		t.Fatalf("ClientConn is in %v state, want TRANSIENT FAILURE", s)
	}

	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)
	testutils.AwaitNotState(ctx, t, cc, connectivity.TransientFailure)
	if s := cc.GetState(); s != connectivity.Ready {
		t.Fatalf("ClientConn is in %v state, want READY", s)
	}

	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVICE_UNKNOWN)
	testutils.AwaitNotState(ctx, t, cc, connectivity.Ready)
	if s := cc.GetState(); s != connectivity.TransientFailure {
		t.Fatalf("ClientConn is in %v state, want TRANSIENT FAILURE", s)
	}

	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)
	testutils.AwaitNotState(ctx, t, cc, connectivity.TransientFailure)
	if s := cc.GetState(); s != connectivity.Ready {
		t.Fatalf("ClientConn is in %v state, want READY", s)
	}

	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_UNKNOWN)
	testutils.AwaitNotState(ctx, t, cc, connectivity.Ready)
	if s := cc.GetState(); s != connectivity.TransientFailure {
		t.Fatalf("ClientConn is in %v state, want TRANSIENT FAILURE", s)
	}
}

// If Watch returns Unimplemented, then the ClientConn should go into READY state.
func (s) TestHealthCheckHealthServerNotRegistered(t *testing.T) {
	grpctest.TLogger.ExpectError("Subchannel health check is unimplemented at server side, thus health check is disabled")
	s := grpc.NewServer()
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("failed to listen due to err: %v", err)
	}
	go s.Serve(lis)
	defer s.Stop()

	cc, r := setupClient(t, nil)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitNotState(ctx, t, cc, connectivity.Idle)
	testutils.AwaitNotState(ctx, t, cc, connectivity.Connecting)
	if s := cc.GetState(); s != connectivity.Ready {
		t.Fatalf("ClientConn is in %v state, want READY", s)
	}
}

// In the case of a goaway received, the health check stream should be terminated and health check
// function should exit.
func (s) TestHealthCheckWithGoAway(t *testing.T) {
	s, lis, ts := setupServer(t, nil)
	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)

	hcEnterChan, hcExitChan, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})
	tc := testgrpc.NewTestServiceClient(cc)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// make some rpcs to make sure connection is working.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	// the stream rpc will persist through goaway event.
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam := []*testpb.ResponseParameters{{Size: 1}}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(1))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseParameters: respParam,
		Payload:            payload,
	}
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(_) = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
	}

	select {
	case <-hcExitChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}

	// server sends GoAway
	go s.GracefulStop()

	select {
	case <-hcExitChan:
	case <-time.After(5 * time.Second):
		select {
		case <-hcEnterChan:
		default:
			t.Fatal("Health check function has not entered after 5s.")
		}
		t.Fatal("Health check function has not exited after 5s.")
	}

	// The existing RPC should be still good to proceed.
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(_) = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
	}
}

func (s) TestHealthCheckWithConnClose(t *testing.T) {
	s, lis, ts := setupServer(t, nil)
	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)

	hcEnterChan, hcExitChan, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})
	tc := testgrpc.NewTestServiceClient(cc)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// make some rpcs to make sure connection is working.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	select {
	case <-hcExitChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}
	// server closes the connection
	s.Stop()

	select {
	case <-hcExitChan:
	case <-time.After(5 * time.Second):
		select {
		case <-hcEnterChan:
		default:
			t.Fatal("Health check function has not entered after 5s.")
		}
		t.Fatal("Health check function has not exited after 5s.")
	}
}

// addrConn drain happens when addrConn gets torn down due to its address being no longer in the
// address list returned by the resolver.
func (s) TestHealthCheckWithAddrConnDrain(t *testing.T) {
	_, lis, ts := setupServer(t, nil)
	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)

	hcEnterChan, hcExitChan, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})
	tc := testgrpc.NewTestServiceClient(cc)
	sc := parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)
	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: sc,
	})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// make some rpcs to make sure connection is working.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	// the stream rpc will persist through goaway event.
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam := []*testpb.ResponseParameters{{Size: 1}}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(1))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseParameters: respParam,
		Payload:            payload,
	}
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(_) = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
	}

	select {
	case <-hcExitChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}
	// trigger teardown of the ac
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "fake address"}}, ServiceConfig: sc})

	select {
	case <-hcExitChan:
	case <-time.After(5 * time.Second):
		select {
		case <-hcEnterChan:
		default:
			t.Fatal("Health check function has not entered after 5s.")
		}
		t.Fatal("Health check function has not exited after 5s.")
	}

	// The existing RPC should be still good to proceed.
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(_) = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
	}
}

// ClientConn close will lead to its addrConns being torn down.
func (s) TestHealthCheckWithClientConnClose(t *testing.T) {
	_, lis, ts := setupServer(t, nil)
	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)

	hcEnterChan, hcExitChan, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})
	tc := testgrpc.NewTestServiceClient(cc)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// make some rpcs to make sure connection is working.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	select {
	case <-hcExitChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}

	// trigger addrConn teardown
	cc.Close()

	select {
	case <-hcExitChan:
	case <-time.After(5 * time.Second):
		select {
		case <-hcEnterChan:
		default:
			t.Fatal("Health check function has not entered after 5s.")
		}
		t.Fatal("Health check function has not exited after 5s.")
	}
}

// This test is to test the logic in the createTransport after the health check function returns which
// closes the skipReset channel(since it has not been closed inside health check func) to unblock
// onGoAway/onClose goroutine.
func (s) TestHealthCheckWithoutSetConnectivityStateCalledAddrConnShutDown(t *testing.T) {
	watchFunc := func(_ *testHealthServer, in *healthpb.HealthCheckRequest, stream healthgrpc.Health_WatchServer) error {
		if in.Service != "delay" {
			return status.Error(codes.FailedPrecondition,
				"this special Watch function only handles request with service name to be \"delay\"")
		}
		// Do nothing to mock a delay of health check response from server side.
		// This case is to help with the test that covers the condition that setConnectivityState is not
		// called inside HealthCheckFunc before the func returns.
		select {
		case <-stream.Context().Done():
		case <-time.After(5 * time.Second):
		}
		return nil
	}
	_, lis, ts := setupServer(t, watchFunc)
	ts.SetServingStatus("delay", healthpb.HealthCheckResponse_SERVING)

	hcEnterChan, hcExitChan, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	_, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})

	// The serviceName "delay" is specially handled at server side, where response will not be sent
	// back to client immediately upon receiving the request (client should receive no response until
	// test ends).
	sc := parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "delay"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)
	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: sc,
	})

	select {
	case <-hcExitChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}

	select {
	case <-hcEnterChan:
	case <-time.After(5 * time.Second):
		t.Fatal("Health check function has not been invoked after 5s.")
	}
	// trigger teardown of the ac, ac in SHUTDOWN state
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "fake address"}}, ServiceConfig: sc})

	// The health check func should exit without calling the setConnectivityState func, as server hasn't sent
	// any response.
	select {
	case <-hcExitChan:
	case <-time.After(5 * time.Second):
		t.Fatal("Health check function has not exited after 5s.")
	}
	// The deferred leakcheck will check whether there's leaked goroutine, which is an indication
	// whether we closes the skipReset channel to unblock onGoAway/onClose goroutine.
}

// This test is to test the logic in the createTransport after the health check function returns which
// closes the allowedToReset channel(since it has not been closed inside health check func) to unblock
// onGoAway/onClose goroutine.
func (s) TestHealthCheckWithoutSetConnectivityStateCalled(t *testing.T) {
	watchFunc := func(_ *testHealthServer, in *healthpb.HealthCheckRequest, stream healthgrpc.Health_WatchServer) error {
		if in.Service != "delay" {
			return status.Error(codes.FailedPrecondition,
				"this special Watch function only handles request with service name to be \"delay\"")
		}
		// Do nothing to mock a delay of health check response from server side.
		// This case is to help with the test that covers the condition that setConnectivityState is not
		// called inside HealthCheckFunc before the func returns.
		select {
		case <-stream.Context().Done():
		case <-time.After(5 * time.Second):
		}
		return nil
	}
	s, lis, ts := setupServer(t, watchFunc)
	ts.SetServingStatus("delay", healthpb.HealthCheckResponse_SERVING)

	hcEnterChan, hcExitChan, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	_, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})

	// The serviceName "delay" is specially handled at server side, where response will not be sent
	// back to client immediately upon receiving the request (client should receive no response until
	// test ends).
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "delay"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	select {
	case <-hcExitChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}

	select {
	case <-hcEnterChan:
	case <-time.After(5 * time.Second):
		t.Fatal("Health check function has not been invoked after 5s.")
	}
	// trigger transport being closed
	s.Stop()

	// The health check func should exit without calling the setConnectivityState func, as server hasn't sent
	// any response.
	select {
	case <-hcExitChan:
	case <-time.After(5 * time.Second):
		t.Fatal("Health check function has not exited after 5s.")
	}
	// The deferred leakcheck will check whether there's leaked goroutine, which is an indication
	// whether we closes the allowedToReset channel to unblock onGoAway/onClose goroutine.
}

func testHealthCheckDisableWithDialOption(t *testing.T, addr string) {
	hcEnterChan, _, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{
		testHealthCheckFuncWrapper: testHealthCheckFuncWrapper,
		extraDialOption:            []grpc.DialOption{grpc.WithDisableHealthCheck()},
	})
	tc := testgrpc.NewTestServiceClient(cc)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: addr}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// send some rpcs to make sure transport has been created and is ready for use.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	select {
	case <-hcEnterChan:
		t.Fatal("Health check function has exited, which is not expected.")
	default:
	}
}

func testHealthCheckDisableWithBalancer(t *testing.T, addr string) {
	hcEnterChan, _, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{
		testHealthCheckFuncWrapper: testHealthCheckFuncWrapper,
	})
	tc := testgrpc.NewTestServiceClient(cc)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: addr}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "foo"
	},
	"loadBalancingConfig": [{"pick_first":{}}]
}`)})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// send some rpcs to make sure transport has been created and is ready for use.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	select {
	case <-hcEnterChan:
		t.Fatal("Health check function has started, which is not expected.")
	default:
	}
}

func testHealthCheckDisableWithServiceConfig(t *testing.T, addr string) {
	hcEnterChan, _, testHealthCheckFuncWrapper := setupHealthCheckWrapper()
	cc, r := setupClient(t, &clientConfig{testHealthCheckFuncWrapper: testHealthCheckFuncWrapper})
	tc := testgrpc.NewTestServiceClient(cc)
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: addr}}})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// send some rpcs to make sure transport has been created and is ready for use.
	if err := verifyResultWithDelay(func() (bool, error) {
		if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			return false, fmt.Errorf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	select {
	case <-hcEnterChan:
		t.Fatal("Health check function has started, which is not expected.")
	default:
	}
}

func (s) TestHealthCheckDisable(t *testing.T) {
	_, lis, ts := setupServer(t, nil)
	ts.SetServingStatus("foo", healthpb.HealthCheckResponse_SERVING)

	// test client side disabling configuration.
	testHealthCheckDisableWithDialOption(t, lis.Addr().String())
	testHealthCheckDisableWithBalancer(t, lis.Addr().String())
	testHealthCheckDisableWithServiceConfig(t, lis.Addr().String())
}

func (s) TestHealthCheckChannelzCountingCallSuccess(t *testing.T) {
	watchFunc := func(_ *testHealthServer, in *healthpb.HealthCheckRequest, _ healthgrpc.Health_WatchServer) error {
		if in.Service != "channelzSuccess" {
			return status.Error(codes.FailedPrecondition,
				"this special Watch function only handles request with service name to be \"channelzSuccess\"")
		}
		return status.Error(codes.OK, "fake success")
	}
	_, lis, _ := setupServer(t, watchFunc)

	_, r := setupClient(t, nil)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "channelzSuccess"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	if err := verifyResultWithDelay(func() (bool, error) {
		cm, _ := channelz.GetTopChannels(0, 0)
		if len(cm) == 0 {
			return false, errors.New("channelz.GetTopChannels return 0 top channel")
		}
		subChans := cm[0].SubChans()
		if len(subChans) == 0 {
			return false, errors.New("there is 0 subchannel")
		}
		var id int64
		for k := range subChans {
			id = k
			break
		}
		scm := channelz.GetSubChannel(id)
		if scm == nil {
			return false, errors.New("nil subchannel returned")
		}
		// exponential backoff retry may result in more than one health check call.
		cstart, csucc, cfail := scm.ChannelMetrics.CallsStarted.Load(), scm.ChannelMetrics.CallsSucceeded.Load(), scm.ChannelMetrics.CallsFailed.Load()
		if cstart > 0 && csucc > 0 && cfail == 0 {
			return true, nil
		}
		return false, fmt.Errorf("got %d CallsStarted, %d CallsSucceeded %d CallsFailed, want >0 >0 =0", cstart, csucc, cfail)
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestHealthCheckChannelzCountingCallFailure(t *testing.T) {
	watchFunc := func(_ *testHealthServer, in *healthpb.HealthCheckRequest, _ healthgrpc.Health_WatchServer) error {
		if in.Service != "channelzFailure" {
			return status.Error(codes.FailedPrecondition,
				"this special Watch function only handles request with service name to be \"channelzFailure\"")
		}
		return status.Error(codes.Internal, "fake failure")
	}
	_, lis, _ := setupServer(t, watchFunc)

	_, r := setupClient(t, nil)
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: lis.Addr().String()}},
		ServiceConfig: parseServiceConfig(t, r, `{
	"healthCheckConfig": {
		"serviceName": "channelzFailure"
	},
	"loadBalancingConfig": [{"round_robin":{}}]
}`)})

	if err := verifyResultWithDelay(func() (bool, error) {
		cm, _ := channelz.GetTopChannels(0, 0)
		if len(cm) == 0 {
			return false, errors.New("channelz.GetTopChannels return 0 top channel")
		}
		subChans := cm[0].SubChans()
		if len(subChans) == 0 {
			return false, errors.New("there is 0 subchannel")
		}
		var id int64
		for k := range subChans {
			id = k
			break
		}
		scm := channelz.GetSubChannel(id)
		if scm == nil {
			return false, errors.New("nil subchannel returned")
		}
		// exponential backoff retry may result in more than one health check call.
		cstart, cfail, csucc := scm.ChannelMetrics.CallsStarted.Load(), scm.ChannelMetrics.CallsFailed.Load(), scm.ChannelMetrics.CallsSucceeded.Load()
		if cstart > 0 && cfail > 0 && csucc == 0 {
			return true, nil
		}
		return false, fmt.Errorf("got %d CallsStarted, %d CallsFailed, %d CallsSucceeded, want >0, >0", cstart, cfail, csucc)
	}); err != nil {
		t.Fatal(err)
	}
}

// healthCheck is a helper function to make a unary health check RPC and return
// the response.
func healthCheck(d time.Duration, cc *grpc.ClientConn, service string) (*healthpb.HealthCheckResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), d)
	defer cancel()
	hc := healthgrpc.NewHealthClient(cc)
	return hc.Check(ctx, &healthpb.HealthCheckRequest{Service: service})
}

// verifyHealthCheckStatus is a helper function to verify that the current
// health status of the service matches the one passed in 'wantStatus'.
func verifyHealthCheckStatus(t *testing.T, d time.Duration, cc *grpc.ClientConn, service string, wantStatus healthpb.HealthCheckResponse_ServingStatus) {
	t.Helper()
	resp, err := healthCheck(d, cc, service)
	if err != nil {
		t.Fatalf("Health/Check(_, _) = _, %v, want _, <nil>", err)
	}
	if resp.Status != wantStatus {
		t.Fatalf("Got the serving status %v, want %v", resp.Status, wantStatus)
	}
}

// verifyHealthCheckErrCode is a helper function to verify that a unary health
// check RPC returns an error with a code set to 'wantCode'.
func verifyHealthCheckErrCode(t *testing.T, d time.Duration, cc *grpc.ClientConn, service string, wantCode codes.Code) {
	t.Helper()
	if _, err := healthCheck(d, cc, service); status.Code(err) != wantCode {
		t.Fatalf("Health/Check() got errCode %v, want %v", status.Code(err), wantCode)
	}
}

// newHealthCheckStream is a helper function to start a health check streaming
// RPC, and returns the stream.
func newHealthCheckStream(t *testing.T, cc *grpc.ClientConn, service string) (healthgrpc.Health_WatchClient, context.CancelFunc) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	hc := healthgrpc.NewHealthClient(cc)
	stream, err := hc.Watch(ctx, &healthpb.HealthCheckRequest{Service: service})
	if err != nil {
		t.Fatalf("hc.Watch(_, %v) failed: %v", service, err)
	}
	return stream, cancel
}

// healthWatchChecker is a helper function to verify that the next health
// status returned on the given stream matches the one passed in 'wantStatus'.
func healthWatchChecker(t *testing.T, stream healthgrpc.Health_WatchClient, wantStatus healthpb.HealthCheckResponse_ServingStatus) {
	t.Helper()
	response, err := stream.Recv()
	if err != nil {
		t.Fatalf("stream.Recv() failed: %v", err)
	}
	if response.Status != wantStatus {
		t.Fatalf("got servingStatus %v, want %v", response.Status, wantStatus)
	}
}

// TestHealthCheckSuccess invokes the unary Check() RPC on the health server in
// a successful case.
func (s) TestHealthCheckSuccess(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthCheckSuccess(t, e)
	}
}

func testHealthCheckSuccess(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	defer te.tearDown()

	verifyHealthCheckErrCode(t, 1*time.Second, te.clientConn(), defaultHealthService, codes.OK)
}

// TestHealthCheckFailure invokes the unary Check() RPC on the health server
// with an expired context and expects the RPC to fail.
func (s) TestHealthCheckFailure(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthCheckFailure(t, e)
	}
}

func testHealthCheckFailure(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise(
		"Failed to dial ",
		"grpc: the client connection is closing; please retry",
	)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	defer te.tearDown()

	verifyHealthCheckErrCode(t, 0*time.Second, te.clientConn(), defaultHealthService, codes.DeadlineExceeded)
	awaitNewConnLogOutput()
}

// TestHealthCheckOff makes a unary Check() RPC on the health server where the
// health status of the defaultHealthService is not set, and therefore expects
// an error code 'codes.NotFound'.
func (s) TestHealthCheckOff(t *testing.T) {
	for _, e := range listTestEnv() {
		// TODO(bradfitz): Temporarily skip this env due to #619.
		if e.name == "handler-tls" {
			continue
		}
		testHealthCheckOff(t, e)
	}
}

func testHealthCheckOff(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	verifyHealthCheckErrCode(t, 1*time.Second, te.clientConn(), defaultHealthService, codes.NotFound)
}

// TestHealthWatchMultipleClients makes a streaming Watch() RPC on the health
// server with multiple clients and expects the same status on both streams.
func (s) TestHealthWatchMultipleClients(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthWatchMultipleClients(t, e)
	}
}

func testHealthWatchMultipleClients(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	stream1, cf1 := newHealthCheckStream(t, cc, defaultHealthService)
	defer cf1()
	healthWatchChecker(t, stream1, healthpb.HealthCheckResponse_SERVICE_UNKNOWN)

	stream2, cf2 := newHealthCheckStream(t, cc, defaultHealthService)
	defer cf2()
	healthWatchChecker(t, stream2, healthpb.HealthCheckResponse_SERVICE_UNKNOWN)

	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_NOT_SERVING)
	healthWatchChecker(t, stream1, healthpb.HealthCheckResponse_NOT_SERVING)
	healthWatchChecker(t, stream2, healthpb.HealthCheckResponse_NOT_SERVING)
}

// TestHealthWatchSameStatus makes a streaming Watch() RPC on the health server
// and makes sure that the health status of the server is as expected after
// multiple calls to SetServingStatus with the same status.
func (s) TestHealthWatchSameStatus(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthWatchSameStatus(t, e)
	}
}

func testHealthWatchSameStatus(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	stream, cf := newHealthCheckStream(t, te.clientConn(), defaultHealthService)
	defer cf()

	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVICE_UNKNOWN)
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVING)
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_NOT_SERVING)
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_NOT_SERVING)
}

// TestHealthWatchServiceStatusSetBeforeStartingServer starts a health server
// on which the health status for the defaultService is set before the gRPC
// server is started, and expects the correct health status to be returned.
func (s) TestHealthWatchServiceStatusSetBeforeStartingServer(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthWatchSetServiceStatusBeforeStartingServer(t, e)
	}
}

func testHealthWatchSetServiceStatusBeforeStartingServer(t *testing.T, e env) {
	hs := health.NewServer()
	te := newTest(t, e)
	te.healthServer = hs
	hs.SetServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	stream, cf := newHealthCheckStream(t, te.clientConn(), defaultHealthService)
	defer cf()
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVING)
}

// TestHealthWatchDefaultStatusChange verifies the simple case where the
// service starts off with a SERVICE_UNKNOWN status (because SetServingStatus
// hasn't been called yet) and then moves to SERVING after SetServingStatus is
// called.
func (s) TestHealthWatchDefaultStatusChange(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthWatchDefaultStatusChange(t, e)
	}
}

func testHealthWatchDefaultStatusChange(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	stream, cf := newHealthCheckStream(t, te.clientConn(), defaultHealthService)
	defer cf()
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVICE_UNKNOWN)
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVING)
}

// TestHealthWatchSetServiceStatusBeforeClientCallsWatch verifies the case
// where the health status is set to SERVING before the client calls Watch().
func (s) TestHealthWatchSetServiceStatusBeforeClientCallsWatch(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthWatchSetServiceStatusBeforeClientCallsWatch(t, e)
	}
}

func testHealthWatchSetServiceStatusBeforeClientCallsWatch(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	defer te.tearDown()

	stream, cf := newHealthCheckStream(t, te.clientConn(), defaultHealthService)
	defer cf()
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVING)
}

// TestHealthWatchOverallServerHealthChange verifies setting the overall status
// of the server by using the empty service name.
func (s) TestHealthWatchOverallServerHealthChange(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthWatchOverallServerHealthChange(t, e)
	}
}

func testHealthWatchOverallServerHealthChange(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	stream, cf := newHealthCheckStream(t, te.clientConn(), "")
	defer cf()
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_SERVING)
	te.setHealthServingStatus("", healthpb.HealthCheckResponse_NOT_SERVING)
	healthWatchChecker(t, stream, healthpb.HealthCheckResponse_NOT_SERVING)
}

// TestUnknownHandler verifies that an expected error is returned (by setting
// the unknownHandler on the server) for a service which is not exposed to the
// client.
func (s) TestUnknownHandler(t *testing.T) {
	// An example unknownHandler that returns a different code and a different
	// method, making sure that we do not expose what methods are implemented to
	// a client that is not authenticated.
	unknownHandler := func(any, grpc.ServerStream) error {
		return status.Error(codes.Unauthenticated, "user unauthenticated")
	}
	for _, e := range listTestEnv() {
		// TODO(bradfitz): Temporarily skip this env due to #619.
		if e.name == "handler-tls" {
			continue
		}
		testUnknownHandler(t, e, unknownHandler)
	}
}

func testUnknownHandler(t *testing.T, e env, unknownHandler grpc.StreamHandler) {
	te := newTest(t, e)
	te.unknownHandler = unknownHandler
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	verifyHealthCheckErrCode(t, 1*time.Second, te.clientConn(), "", codes.Unauthenticated)
}

// TestHealthCheckServingStatus makes a streaming Watch() RPC on the health
// server and verifies a bunch of health status transitions.
func (s) TestHealthCheckServingStatus(t *testing.T) {
	for _, e := range listTestEnv() {
		testHealthCheckServingStatus(t, e)
	}
}

func testHealthCheckServingStatus(t *testing.T, e env) {
	te := newTest(t, e)
	te.enableHealthServer = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	verifyHealthCheckStatus(t, 1*time.Second, cc, "", healthpb.HealthCheckResponse_SERVING)
	verifyHealthCheckErrCode(t, 1*time.Second, cc, defaultHealthService, codes.NotFound)
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	verifyHealthCheckStatus(t, 1*time.Second, cc, defaultHealthService, healthpb.HealthCheckResponse_SERVING)
	te.setHealthServingStatus(defaultHealthService, healthpb.HealthCheckResponse_NOT_SERVING)
	verifyHealthCheckStatus(t, 1*time.Second, cc, defaultHealthService, healthpb.HealthCheckResponse_NOT_SERVING)
}
