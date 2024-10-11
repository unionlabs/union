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
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/backoff"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/pickfirst"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const pickFirstServiceConfig = `{"loadBalancingConfig": [{"pick_first":{}}]}`

// setupPickFirst performs steps required for pick_first tests. It starts a
// bunch of backends exporting the TestService, creates a ClientConn to them
// with service config specifying the use of the pick_first LB policy.
func setupPickFirst(t *testing.T, backendCount int, opts ...grpc.DialOption) (*grpc.ClientConn, *manual.Resolver, []*stubserver.StubServer) {
	t.Helper()

	r := manual.NewBuilderWithScheme("whatever")

	backends := make([]*stubserver.StubServer, backendCount)
	addrs := make([]resolver.Address, backendCount)
	for i := 0; i < backendCount; i++ {
		backend := &stubserver.StubServer{
			EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
				return &testpb.Empty{}, nil
			},
		}
		if err := backend.StartServer(); err != nil {
			t.Fatalf("Failed to start backend: %v", err)
		}
		t.Logf("Started TestService backend at: %q", backend.Address)
		t.Cleanup(func() { backend.Stop() })

		backends[i] = backend
		addrs[i] = resolver.Address{Addr: backend.Address}
	}

	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
		grpc.WithDefaultServiceConfig(pickFirstServiceConfig),
	}
	dopts = append(dopts, opts...)
	cc, err := grpc.NewClient(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("grpc.NewClient() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	// At this point, the resolver has not returned any addresses to the channel.
	// This RPC must block until the context expires.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("EmptyCall() = %s, want %s", status.Code(err), codes.DeadlineExceeded)
	}
	return cc, r, backends
}

// stubBackendsToResolverAddrs converts from a set of stub server backends to
// resolver addresses. Useful when pushing addresses to the manual resolver.
func stubBackendsToResolverAddrs(backends []*stubserver.StubServer) []resolver.Address {
	addrs := make([]resolver.Address, len(backends))
	for i, backend := range backends {
		addrs[i] = resolver.Address{Addr: backend.Address}
	}
	return addrs
}

// TestPickFirst_OneBackend tests the most basic scenario for pick_first. It
// brings up a single backend and verifies that all RPCs get routed to it.
func (s) TestPickFirst_OneBackend(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 1)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}
}

// TestPickFirst_MultipleBackends tests the scenario with multiple backends and
// verifies that all RPCs get routed to the first one.
func (s) TestPickFirst_MultipleBackends(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 2)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}
}

// TestPickFirst_OneServerDown tests the scenario where we have multiple
// backends and pick_first is working as expected. Verifies that RPCs get routed
// to the next backend in the list when the first one goes down.
func (s) TestPickFirst_OneServerDown(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 2)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Stop the backend which is currently being used. RPCs should get routed to
	// the next backend in the list.
	backends[0].Stop()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}
}

// TestPickFirst_AllServersDown tests the scenario where we have multiple
// backends and pick_first is working as expected. When all backends go down,
// the test verifies that RPCs fail with appropriate status code.
func (s) TestPickFirst_AllServersDown(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 2)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	for _, b := range backends {
		b.Stop()
	}

	client := testgrpc.NewTestServiceClient(cc)
	for {
		if ctx.Err() != nil {
			t.Fatalf("channel failed to move to Unavailable after all backends were stopped: %v", ctx.Err())
		}
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) == codes.Unavailable {
			return
		}
		time.Sleep(defaultTestShortTimeout)
	}
}

// TestPickFirst_AddressesRemoved tests the scenario where we have multiple
// backends and pick_first is working as expected. It then verifies that when
// addresses are removed by the name resolver, RPCs get routed appropriately.
func (s) TestPickFirst_AddressesRemoved(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 3)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Remove the first backend from the list of addresses originally pushed.
	// RPCs should get routed to the first backend in the new list.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{addrs[1], addrs[2]}})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}

	// Append the backend that we just removed to the end of the list.
	// Nothing should change.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{addrs[1], addrs[2], addrs[0]}})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}

	// Remove the first backend from the existing list of addresses.
	// RPCs should get routed to the first backend in the new list.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{addrs[2], addrs[0]}})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[2]); err != nil {
		t.Fatal(err)
	}

	// Remove the first backend from the existing list of addresses.
	// RPCs should get routed to the first backend in the new list.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{addrs[0]}})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}
}

// TestPickFirst_NewAddressWhileBlocking tests the case where pick_first is
// configured on a channel, things are working as expected and then a resolver
// updates removes all addresses. An RPC attempted at this point in time will be
// blocked because there are no valid backends. This test verifies that when new
// backends are added, the RPC is able to complete.
func (s) TestPickFirst_NewAddressWhileBlocking(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 2)
	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Send a resolver update with no addresses. This should push the channel into
	// TransientFailure.
	r.UpdateState(resolver.State{})
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	doneCh := make(chan struct{})
	client := testgrpc.NewTestServiceClient(cc)
	go func() {
		// The channel is currently in TransientFailure and this RPC will block
		// until the channel becomes Ready, which will only happen when we push a
		// resolver update with a valid backend address.
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
			t.Errorf("EmptyCall() = %v, want <nil>", err)
		}
		close(doneCh)
	}()

	// Make sure that there is one pending RPC on the ClientConn before attempting
	// to push new addresses through the name resolver. If we don't do this, the
	// resolver update can happen before the above goroutine gets to make the RPC.
	for {
		if err := ctx.Err(); err != nil {
			t.Fatal(err)
		}
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			t.Fatalf("there should only be one top channel, not %d", len(tcs))
		}
		started := tcs[0].ChannelMetrics.CallsStarted.Load()
		completed := tcs[0].ChannelMetrics.CallsSucceeded.Load() + tcs[0].ChannelMetrics.CallsFailed.Load()
		if (started - completed) == 1 {
			break
		}
		time.Sleep(defaultTestShortTimeout)
	}

	// Send a resolver update with a valid backend to push the channel to Ready
	// and unblock the above RPC.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: backends[0].Address}}})

	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for blocked RPC to complete")
	case <-doneCh:
	}
}

// TestPickFirst_StickyTransientFailure tests the case where pick_first is
// configured on a channel, and the backend is configured to close incoming
// connections as soon as they are accepted. The test verifies that the channel
// enters TransientFailure and stays there. The test also verifies that the
// pick_first LB policy is constantly trying to reconnect to the backend.
func (s) TestPickFirst_StickyTransientFailure(t *testing.T) {
	// Spin up a local server which closes the connection as soon as it receives
	// one. It also sends a signal on a channel whenever it received a connection.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}
	t.Cleanup(func() { lis.Close() })

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	connCh := make(chan struct{}, 1)
	go func() {
		for {
			conn, err := lis.Accept()
			if err != nil {
				return
			}
			select {
			case connCh <- struct{}{}:
				conn.Close()
			case <-ctx.Done():
				return
			}
		}
	}()

	// Dial the above server with a ConnectParams that does a constant backoff
	// of defaultTestShortTimeout duration.
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(pickFirstServiceConfig),
		grpc.WithConnectParams(grpc.ConnectParams{
			Backoff: backoff.Config{
				BaseDelay:  defaultTestShortTimeout,
				Multiplier: float64(0),
				Jitter:     float64(0),
				MaxDelay:   defaultTestShortTimeout,
			},
		}),
	}
	cc, err := grpc.Dial(lis.Addr().String(), dopts...)
	if err != nil {
		t.Fatalf("Failed to dial server at %q: %v", lis.Addr(), err)
	}
	t.Cleanup(func() { cc.Close() })

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Spawn a goroutine to ensure that the channel stays in TransientFailure.
	// The call to cc.WaitForStateChange will return false when the main
	// goroutine exits and the context is cancelled.
	go func() {
		if cc.WaitForStateChange(ctx, connectivity.TransientFailure) {
			if state := cc.GetState(); state != connectivity.Shutdown {
				t.Errorf("Unexpected state change from TransientFailure to %s", cc.GetState())
			}
		}
	}()

	// Ensures that the pick_first LB policy is constantly trying to reconnect.
	for i := 0; i < 10; i++ {
		select {
		case <-connCh:
		case <-time.After(2 * defaultTestShortTimeout):
			t.Error("Timeout when waiting for pick_first to reconnect")
		}
	}
}

// Tests the PF LB policy with shuffling enabled.
func (s) TestPickFirst_ShuffleAddressList(t *testing.T) {
	const serviceConfig = `{"loadBalancingConfig": [{"pick_first":{ "shuffleAddressList": true }}]}`

	// Install a shuffler that always reverses two entries.
	origShuf := internal.ShuffleAddressListForTesting
	defer func() { internal.ShuffleAddressListForTesting = origShuf }()
	internal.ShuffleAddressListForTesting = func(n int, f func(int, int)) {
		if n != 2 {
			t.Errorf("Shuffle called with n=%v; want 2", n)
			return
		}
		f(0, 1) // reverse the two addresses
	}

	// Set up our backends.
	cc, r, backends := setupPickFirst(t, 2)
	addrs := stubBackendsToResolverAddrs(backends)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Push an update with both addresses and shuffling disabled.  We should
	// connect to backend 0.
	r.UpdateState(resolver.State{Endpoints: []resolver.Endpoint{
		{Addresses: []resolver.Address{addrs[0]}},
		{Addresses: []resolver.Address{addrs[1]}},
	}})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Send a config with shuffling enabled.  This will reverse the addresses,
	// but the channel should still be connected to backend 0.
	shufState := resolver.State{
		ServiceConfig: parseServiceConfig(t, r, serviceConfig),
		Endpoints: []resolver.Endpoint{
			{Addresses: []resolver.Address{addrs[0]}},
			{Addresses: []resolver.Address{addrs[1]}},
		},
	}
	r.UpdateState(shufState)
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Send a resolver update with no addresses. This should push the channel
	// into TransientFailure.
	r.UpdateState(resolver.State{})
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Send the same config as last time with shuffling enabled.  Since we are
	// not connected to backend 0, we should connect to backend 1.
	r.UpdateState(shufState)
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}
}

// Test config parsing with the env var turned on and off for various scenarios.
func (s) TestPickFirst_ParseConfig_Success(t *testing.T) {
	// Install a shuffler that always reverses two entries.
	origShuf := internal.ShuffleAddressListForTesting
	defer func() { internal.ShuffleAddressListForTesting = origShuf }()
	internal.ShuffleAddressListForTesting = func(n int, f func(int, int)) {
		if n != 2 {
			t.Errorf("Shuffle called with n=%v; want 2", n)
			return
		}
		f(0, 1) // reverse the two addresses
	}

	tests := []struct {
		name          string
		serviceConfig string
		wantFirstAddr bool
	}{
		{
			name:          "empty pickfirst config",
			serviceConfig: `{"loadBalancingConfig": [{"pick_first":{}}]}`,
			wantFirstAddr: true,
		},
		{
			name:          "empty good pickfirst config",
			serviceConfig: `{"loadBalancingConfig": [{"pick_first":{ "shuffleAddressList": true }}]}`,
			wantFirstAddr: false,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			// Set up our backends.
			cc, r, backends := setupPickFirst(t, 2)
			addrs := stubBackendsToResolverAddrs(backends)

			r.UpdateState(resolver.State{
				ServiceConfig: parseServiceConfig(t, r, test.serviceConfig),
				Addresses:     addrs,
			})

			// Some tests expect address shuffling to happen, and indicate that
			// by setting wantFirstAddr to false (since our shuffling function
			// defined at the top of this test, simply reverses the list of
			// addresses provided to it).
			wantAddr := addrs[0]
			if !test.wantFirstAddr {
				wantAddr = addrs[1]
			}

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := pickfirst.CheckRPCsToBackend(ctx, cc, wantAddr); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// Test config parsing for a bad service config.
func (s) TestPickFirst_ParseConfig_Failure(t *testing.T) {
	// Service config should fail with the below config. Name resolvers are
	// expected to perform this parsing before they push the parsed service
	// config to the channel.
	const sc = `{"loadBalancingConfig": [{"pick_first":{ "shuffleAddressList": 666 }}]}`
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(sc)
	if scpr.Err == nil {
		t.Fatalf("ParseConfig() succeeded and returned %+v, when expected to fail", scpr)
	}
}

// setupPickFirstWithListenerWrapper is very similar to setupPickFirst, but uses
// a wrapped listener that the test can use to track accepted connections.
func setupPickFirstWithListenerWrapper(t *testing.T, backendCount int, opts ...grpc.DialOption) (*grpc.ClientConn, *manual.Resolver, []*stubserver.StubServer, []*testutils.ListenerWrapper) {
	t.Helper()

	backends := make([]*stubserver.StubServer, backendCount)
	addrs := make([]resolver.Address, backendCount)
	listeners := make([]*testutils.ListenerWrapper, backendCount)
	for i := 0; i < backendCount; i++ {
		lis := testutils.NewListenerWrapper(t, nil)
		backend := &stubserver.StubServer{
			Listener: lis,
			EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
				return &testpb.Empty{}, nil
			},
		}
		if err := backend.StartServer(); err != nil {
			t.Fatalf("Failed to start backend: %v", err)
		}
		t.Logf("Started TestService backend at: %q", backend.Address)
		t.Cleanup(func() { backend.Stop() })

		backends[i] = backend
		addrs[i] = resolver.Address{Addr: backend.Address}
		listeners[i] = lis
	}

	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
		grpc.WithDefaultServiceConfig(pickFirstServiceConfig),
	}
	dopts = append(dopts, opts...)
	cc, err := grpc.NewClient(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("grpc.NewClient() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	// At this point, the resolver has not returned any addresses to the channel.
	// This RPC must block until the context expires.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("EmptyCall() = %s, want %s", status.Code(err), codes.DeadlineExceeded)
	}
	return cc, r, backends, listeners
}

// TestPickFirst_AddressUpdateWithAttributes tests the case where an address
// update received by the pick_first LB policy differs in attributes. Addresses
// which differ in attributes are considered different from the perspective of
// subconn creation and connection establishment and the test verifies that new
// connections are created when attributes change.
func (s) TestPickFirst_AddressUpdateWithAttributes(t *testing.T) {
	cc, r, backends, listeners := setupPickFirstWithListenerWrapper(t, 2)

	// Add a set of attributes to the addresses before pushing them to the
	// pick_first LB policy through the manual resolver.
	addrs := stubBackendsToResolverAddrs(backends)
	for i := range addrs {
		addrs[i].Attributes = addrs[i].Attributes.WithValue("test-attribute-1", fmt.Sprintf("%d", i))
	}
	r.UpdateState(resolver.State{Addresses: addrs})

	// Ensure that RPCs succeed to the first backend in the list.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Grab the wrapped connection from the listener wrapper. This will be used
	// to verify the connection is closed.
	val, err := listeners[0].NewConnCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Failed to receive new connection from wrapped listener: %v", err)
	}
	conn := val.(*testutils.ConnWrapper)

	// Add another set of attributes to the addresses, and push them to the
	// pick_first LB policy through the manual resolver. Leave the order of the
	// addresses unchanged.
	for i := range addrs {
		addrs[i].Attributes = addrs[i].Attributes.WithValue("test-attribute-2", fmt.Sprintf("%d", i))
	}
	r.UpdateState(resolver.State{Addresses: addrs})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// A change in the address attributes results in the new address being
	// considered different to the current address. This will result in the old
	// connection being closed and a new connection to the same backend (since
	// address order is not modified).
	if _, err := conn.CloseCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout when expecting existing connection to be closed: %v", err)
	}
	val, err = listeners[0].NewConnCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Failed to receive new connection from wrapped listener: %v", err)
	}
	conn = val.(*testutils.ConnWrapper)

	// Add another set of attributes to the addresses, and push them to the
	// pick_first LB policy through the manual resolver.  Reverse of the order
	// of addresses.
	for i := range addrs {
		addrs[i].Attributes = addrs[i].Attributes.WithValue("test-attribute-3", fmt.Sprintf("%d", i))
	}
	addrs[0], addrs[1] = addrs[1], addrs[0]
	r.UpdateState(resolver.State{Addresses: addrs})
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Ensure that the old connection is closed and a new connection is
	// established to the first address in the new list.
	if _, err := conn.CloseCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout when expecting existing connection to be closed: %v", err)
	}
	_, err = listeners[1].NewConnCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Failed to receive new connection from wrapped listener: %v", err)
	}
}

// TestPickFirst_AddressUpdateWithBalancerAttributes tests the case where an
// address update received by the pick_first LB policy differs in balancer
// attributes, which are meant only for consumption by LB policies. In this
// case, the test verifies that new connections are not created when the address
// update only changes the balancer attributes.
func (s) TestPickFirst_AddressUpdateWithBalancerAttributes(t *testing.T) {
	cc, r, backends, listeners := setupPickFirstWithListenerWrapper(t, 2)

	// Add a set of balancer attributes to the addresses before pushing them to
	// the pick_first LB policy through the manual resolver.
	addrs := stubBackendsToResolverAddrs(backends)
	for i := range addrs {
		addrs[i].BalancerAttributes = addrs[i].BalancerAttributes.WithValue("test-attribute-1", fmt.Sprintf("%d", i))
	}
	r.UpdateState(resolver.State{Addresses: addrs})

	// Ensure that RPCs succeed to the expected backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Grab the wrapped connection from the listener wrapper. This will be used
	// to verify the connection is not closed.
	val, err := listeners[0].NewConnCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Failed to receive new connection from wrapped listener: %v", err)
	}
	conn := val.(*testutils.ConnWrapper)

	// Add a set of balancer attributes to the addresses before pushing them to
	// the pick_first LB policy through the manual resolver. Leave the order of
	// the addresses unchanged.
	for i := range addrs {
		addrs[i].BalancerAttributes = addrs[i].BalancerAttributes.WithValue("test-attribute-2", fmt.Sprintf("%d", i))
	}
	r.UpdateState(resolver.State{Addresses: addrs})

	// Ensure that no new connection is established, and ensure that the old
	// connection is not closed.
	for i := range listeners {
		sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
		defer sCancel()
		if _, err := listeners[i].NewConnCh.Receive(sCtx); err != context.DeadlineExceeded {
			t.Fatalf("Unexpected error when expecting no new connection: %v", err)
		}
	}
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := conn.CloseCh.Receive(sCtx); err != context.DeadlineExceeded {
		t.Fatalf("Unexpected error when expecting existing connection to stay active: %v", err)
	}
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Add a set of balancer attributes to the addresses before pushing them to
	// the pick_first LB policy through the manual resolver. Reverse of the
	// order of addresses.
	for i := range addrs {
		addrs[i].BalancerAttributes = addrs[i].BalancerAttributes.WithValue("test-attribute-3", fmt.Sprintf("%d", i))
	}
	addrs[0], addrs[1] = addrs[1], addrs[0]
	r.UpdateState(resolver.State{Addresses: addrs})

	// Ensure that no new connection is established, and ensure that the old
	// connection is not closed.
	for i := range listeners {
		sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
		defer sCancel()
		if _, err := listeners[i].NewConnCh.Receive(sCtx); err != context.DeadlineExceeded {
			t.Fatalf("Unexpected error when expecting no new connection: %v", err)
		}
	}
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := conn.CloseCh.Receive(sCtx); err != context.DeadlineExceeded {
		t.Fatalf("Unexpected error when expecting existing connection to stay active: %v", err)
	}
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}
}

// Tests the case where the pick_first LB policy receives an error from the name
// resolver without previously receiving a good update. Verifies that the
// channel moves to TRANSIENT_FAILURE and that error received from the name
// resolver is propagated to the caller of an RPC.
func (s) TestPickFirst_ResolverError_NoPreviousUpdate(t *testing.T) {
	cc, r, _ := setupPickFirst(t, 0)

	nrErr := errors.New("error from name resolver")
	r.ReportError(nrErr)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	client := testgrpc.NewTestServiceClient(cc)
	_, err := client.EmptyCall(ctx, &testpb.Empty{})
	if err == nil {
		t.Fatalf("EmptyCall() succeeded when expected to fail with error: %v", nrErr)
	}
	if !strings.Contains(err.Error(), nrErr.Error()) {
		t.Fatalf("EmptyCall() failed with error: %v, want error: %v", err, nrErr)
	}
}

// Tests the case where the pick_first LB policy receives an error from the name
// resolver after receiving a good update (and the channel is currently READY).
// The test verifies that the channel continues to use the previously received
// good update.
func (s) TestPickFirst_ResolverError_WithPreviousUpdate_Ready(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 1)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	nrErr := errors.New("error from name resolver")
	r.ReportError(nrErr)

	// Ensure that RPCs continue to succeed for the next second.
	client := testgrpc.NewTestServiceClient(cc)
	for end := time.Now().Add(time.Second); time.Now().Before(end); <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
	}
}

// Tests the case where the pick_first LB policy receives an error from the name
// resolver after receiving a good update (and the channel is currently in
// CONNECTING state). The test verifies that the channel continues to use the
// previously received good update, and that RPCs don't fail with the error
// received from the name resolver.
func (s) TestPickFirst_ResolverError_WithPreviousUpdate_Connecting(t *testing.T) {
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("net.Listen() failed: %v", err)
	}

	// Listen on a local port and act like a server that blocks until the
	// channel reaches CONNECTING and closes the connection without sending a
	// server preface.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	waitForConnecting := make(chan struct{})
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("Unexpected error when accepting a connection: %v", err)
		}
		defer conn.Close()

		select {
		case <-waitForConnecting:
		case <-ctx.Done():
			t.Error("Timeout when waiting for channel to move to CONNECTING state")
		}
	}()

	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
		grpc.WithDefaultServiceConfig(pickFirstServiceConfig),
	}
	cc, err := grpc.Dial(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	addrs := []resolver.Address{{Addr: lis.Addr().String()}}
	r.UpdateState(resolver.State{Addresses: addrs})
	testutils.AwaitState(ctx, t, cc, connectivity.Connecting)

	nrErr := errors.New("error from name resolver")
	r.ReportError(nrErr)

	// RPCs should fail with deadline exceed error as long as they are in
	// CONNECTING and not the error returned by the name resolver.
	client := testgrpc.NewTestServiceClient(cc)
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); !strings.Contains(err.Error(), context.DeadlineExceeded.Error()) {
		t.Fatalf("EmptyCall() failed with error: %v, want error: %v", err, context.DeadlineExceeded)
	}

	// Closing this channel leads to closing of the connection by our listener.
	// gRPC should see this as a connection error.
	close(waitForConnecting)
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)
	checkForConnectionError(ctx, t, cc)
}

// Tests the case where the pick_first LB policy receives an error from the name
// resolver after receiving a good update. The previous good update though has
// seen the channel move to TRANSIENT_FAILURE.  The test verifies that the
// channel fails RPCs with the new error from the resolver.
func (s) TestPickFirst_ResolverError_WithPreviousUpdate_TransientFailure(t *testing.T) {
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("net.Listen() failed: %v", err)
	}

	// Listen on a local port and act like a server that closes the connection
	// without sending a server preface.
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("Unexpected error when accepting a connection: %v", err)
		}
		conn.Close()
	}()

	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
		grpc.WithDefaultServiceConfig(pickFirstServiceConfig),
	}
	cc, err := grpc.Dial(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	addrs := []resolver.Address{{Addr: lis.Addr().String()}}
	r.UpdateState(resolver.State{Addresses: addrs})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)
	checkForConnectionError(ctx, t, cc)

	// An error from the name resolver should result in RPCs failing with that
	// error instead of the old error that caused the channel to move to
	// TRANSIENT_FAILURE in the first place.
	nrErr := errors.New("error from name resolver")
	r.ReportError(nrErr)
	client := testgrpc.NewTestServiceClient(cc)
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); strings.Contains(err.Error(), nrErr.Error()) {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatal("Timeout when waiting for RPCs to fail with error returned by the name resolver")
	}
}

func checkForConnectionError(ctx context.Context, t *testing.T, cc *grpc.ClientConn) {
	t.Helper()

	// RPCs may fail on the client side in two ways, once the fake server closes
	// the accepted connection:
	// - writing the client preface succeeds, but not reading the server preface
	// - writing the client preface fails
	// In either case, we should see it fail with UNAVAILABLE.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("EmptyCall() failed with error: %v, want code %v", err, codes.Unavailable)
	}
}

// Tests the case where the pick_first LB policy receives an update from the
// name resolver with no addresses after receiving a good update. The test
// verifies that the channel fails RPCs with an error indicating the fact that
// the name resolver returned no addresses.
func (s) TestPickFirst_ResolverError_ZeroAddresses_WithPreviousUpdate(t *testing.T) {
	cc, r, backends := setupPickFirst(t, 1)

	addrs := stubBackendsToResolverAddrs(backends)
	r.UpdateState(resolver.State{Addresses: addrs})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	r.UpdateState(resolver.State{})
	wantErr := "produced zero addresses"
	client := testgrpc.NewTestServiceClient(cc)
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); strings.Contains(err.Error(), wantErr) {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatal("Timeout when waiting for RPCs to fail with error returned by the name resolver")
	}
}
