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
	"fmt"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	grpclbstate "google.golang.org/grpc/balancer/grpclb/state"
	pickfirst "google.golang.org/grpc/balancer/pickfirst"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils/fakegrpclb"
	pfutil "google.golang.org/grpc/internal/testutils/pickfirst"
	rrutil "google.golang.org/grpc/internal/testutils/roundrobin"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const (
	loadBalancedServiceName = "foo.bar.service"
	loadBalancedServicePort = 443
	wantGRPCLBTraceDesc     = `Channel switches to new LB policy "grpclb"`
	wantRoundRobinTraceDesc = `Channel switches to new LB policy "round_robin"`

	// This is the number of stub backends set up at the start of each test. The
	// first backend is used for the "grpclb" policy and the rest are used for
	// other LB policies to test balancer switching.
	backendCount = 3
)

// setupBackendsAndFakeGRPCLB sets up backendCount number of stub server
// backends and a fake grpclb server for tests which exercise balancer switch
// scenarios involving grpclb.
//
// The fake grpclb server always returns the first of the configured stub
// backends as backend addresses. So, the tests are free to use the other
// backends with other LB policies to verify balancer switching scenarios.
//
// Returns a cleanup function to be invoked by the caller.
func setupBackendsAndFakeGRPCLB(t *testing.T) ([]*stubserver.StubServer, *fakegrpclb.Server, func()) {
	backends, backendsCleanup := startBackendsForBalancerSwitch(t)

	lbServer, err := fakegrpclb.NewServer(fakegrpclb.ServerParams{
		LoadBalancedServiceName: loadBalancedServiceName,
		LoadBalancedServicePort: loadBalancedServicePort,
		BackendAddresses:        []string{backends[0].Address},
	})
	if err != nil {
		t.Fatalf("failed to create fake grpclb server: %v", err)
	}
	go func() {
		if err := lbServer.Serve(); err != nil {
			t.Errorf("fake grpclb Serve() failed: %v", err)
		}
	}()

	return backends, lbServer, func() {
		backendsCleanup()
		lbServer.Stop()
	}
}

// startBackendsForBalancerSwitch spins up a bunch of stub server backends
// exposing the TestService. Returns a cleanup function to be invoked by the
// caller.
func startBackendsForBalancerSwitch(t *testing.T) ([]*stubserver.StubServer, func()) {
	t.Helper()

	backends := make([]*stubserver.StubServer, backendCount)
	for i := 0; i < backendCount; i++ {
		backend := &stubserver.StubServer{
			EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
		}
		if err := backend.StartServer(); err != nil {
			t.Fatalf("Failed to start backend: %v", err)
		}
		t.Logf("Started TestService backend at: %q", backend.Address)
		backends[i] = backend
	}
	return backends, func() {
		for _, b := range backends {
			b.Stop()
		}
	}
}

// TestBalancerSwitch_Basic tests the basic scenario of switching from one LB
// policy to another, as specified in the service config.
func (s) TestBalancerSwitch_Basic(t *testing.T) {
	backends, cleanup := startBackendsForBalancerSwitch(t)
	defer cleanup()
	addrs := stubBackendsToResolverAddrs(backends)

	r := manual.NewBuilderWithScheme("whatever")
	cc, err := grpc.Dial(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Push a resolver update without an LB policy in the service config. The
	// channel should pick the default LB policy, which is pick_first.
	r.UpdateState(resolver.State{Addresses: addrs})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update with the service config specifying "round_robin".
	r.UpdateState(resolver.State{
		Addresses:     addrs,
		ServiceConfig: parseServiceConfig(t, r, rrServiceConfig),
	})
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update with the service config specifying "pick_first".
	r.UpdateState(resolver.State{
		Addresses:     addrs,
		ServiceConfig: parseServiceConfig(t, r, pickFirstServiceConfig),
	})
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}
}

// TestBalancerSwitch_grpclbToPickFirst tests the scenario where the channel
// starts off "grpclb", switches to "pick_first" and back.
func (s) TestBalancerSwitch_grpclbToPickFirst(t *testing.T) {
	backends, lbServer, cleanup := setupBackendsAndFakeGRPCLB(t)
	defer cleanup()

	addrs := stubBackendsToResolverAddrs(backends)
	r := manual.NewBuilderWithScheme("whatever")
	target := fmt.Sprintf("%s:///%s", r.Scheme(), loadBalancedServiceName)
	cc, err := grpc.Dial(target, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Push a resolver update with a GRPCLB service config and a single address
	// pointing to the grpclb server we created above. This will cause the
	// channel to switch to the "grpclb" balancer, which returns a single
	// backend address.
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	state := resolver.State{ServiceConfig: grpclbConfig}
	r.UpdateState(grpclbstate.Set(state, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: lbServer.Address()}}}))
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[0:1]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update containing a non-existent grpclb server address.
	// This should not lead to a balancer switch.
	const nonExistentServer = "non-existent-grpclb-server-address"
	r.UpdateState(grpclbstate.Set(state, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: nonExistentServer}}}))
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[:1]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update containing no grpclb server address. This should
	// lead to the channel using the default LB policy which is pick_first. The
	// list of addresses pushed as part of this update is different from the one
	// returned by the "grpclb" balancer. So, we should see RPCs going to the
	// newly configured backends, as part of the balancer switch.
	emptyConfig := parseServiceConfig(t, r, `{}`)
	r.UpdateState(resolver.State{Addresses: addrs[1:], ServiceConfig: emptyConfig})
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}
}

// TestBalancerSwitch_pickFirstToGRPCLB tests the scenario where the channel
// starts off with "pick_first", switches to "grpclb" and back.
func (s) TestBalancerSwitch_pickFirstToGRPCLB(t *testing.T) {
	backends, lbServer, cleanup := setupBackendsAndFakeGRPCLB(t)
	defer cleanup()

	addrs := stubBackendsToResolverAddrs(backends)
	r := manual.NewBuilderWithScheme("whatever")
	target := fmt.Sprintf("%s:///%s", r.Scheme(), loadBalancedServiceName)
	cc, err := grpc.Dial(target, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Push a resolver update containing no grpclb server address. This should
	// lead to the channel using the default LB policy which is pick_first.
	r.UpdateState(resolver.State{Addresses: addrs[1:]})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update with no service config and a single address pointing
	// to the grpclb server we created above. This will cause the channel to
	// switch to the "grpclb" balancer, which returns a single backend address.
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	state := resolver.State{ServiceConfig: grpclbConfig}
	r.UpdateState(grpclbstate.Set(state, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: lbServer.Address()}}}))
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[:1]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update containing a non-existent grpclb server address.
	// This should not lead to a balancer switch.
	r.UpdateState(grpclbstate.Set(state, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: "nonExistentServer"}}}))
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[:1]); err != nil {
		t.Fatal(err)
	}

	// Switch to "pick_first" again by sending no grpclb server addresses.
	emptyConfig := parseServiceConfig(t, r, `{}`)
	r.UpdateState(resolver.State{Addresses: addrs[1:], ServiceConfig: emptyConfig})
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[1]); err != nil {
		t.Fatal(err)
	}
}

// TestBalancerSwitch_RoundRobinToGRPCLB tests the scenario where the channel
// starts off with "round_robin", switches to "grpclb" and back.
//
// Note that this test uses the deprecated `loadBalancingPolicy` field in the
// service config.
func (s) TestBalancerSwitch_RoundRobinToGRPCLB(t *testing.T) {
	backends, lbServer, cleanup := setupBackendsAndFakeGRPCLB(t)
	defer cleanup()

	addrs := stubBackendsToResolverAddrs(backends)
	r := manual.NewBuilderWithScheme("whatever")
	target := fmt.Sprintf("%s:///%s", r.Scheme(), loadBalancedServiceName)
	cc, err := grpc.Dial(target, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Note the use of the deprecated `loadBalancingPolicy` field here instead
	// of the now recommended `loadBalancingConfig` field. The logic in the
	// ClientConn which decides which balancer to switch to looks at the
	// following places in the given order of preference:
	// - `loadBalancingConfig` field
	// - addresses of type grpclb
	// - `loadBalancingPolicy` field
	// If we use the `loadBalancingPolicy` field, the switch to "grpclb" later on
	// in the test will not happen as the ClientConn will continue to use the LB
	// policy received in the first update.
	scpr := parseServiceConfig(t, r, `{"loadBalancingPolicy": "round_robin"}`)

	// Push a resolver update with the service config specifying "round_robin".
	r.UpdateState(resolver.State{Addresses: addrs[1:], ServiceConfig: scpr})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[1:]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update with grpclb and a single balancer address
	// pointing to the grpclb server we created above. This will cause the
	// channel to switch to the "grpclb" balancer, which returns a single
	// backend address.
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	state := resolver.State{ServiceConfig: grpclbConfig}
	r.UpdateState(grpclbstate.Set(state, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: lbServer.Address()}}}))
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[:1]); err != nil {
		t.Fatal(err)
	}

	// Switch back to "round_robin".
	r.UpdateState(resolver.State{Addresses: addrs[1:], ServiceConfig: scpr})
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[1:]); err != nil {
		t.Fatal(err)
	}
}

// TestBalancerSwitch_grpclbNotRegistered tests the scenario where the grpclb
// balancer is not registered. Verifies that the ClientConn falls back to the
// default LB policy or the LB policy specified in the service config, and that
// addresses of type "grpclb" are filtered out.
func (s) TestBalancerSwitch_grpclbNotRegistered(t *testing.T) {
	// Unregister the grpclb balancer builder for the duration of this test.
	grpclbBuilder := balancer.Get("grpclb")
	internal.BalancerUnregister(grpclbBuilder.Name())
	defer balancer.Register(grpclbBuilder)

	backends, cleanup := startBackendsForBalancerSwitch(t)
	defer cleanup()
	addrs := stubBackendsToResolverAddrs(backends)

	r := manual.NewBuilderWithScheme("whatever")
	cc, err := grpc.Dial(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Push a resolver update which contains a bunch of stub server backends and a
	// grpclb server address. The latter should get the ClientConn to try and
	// apply the grpclb policy. But since grpclb is not registered, it should
	// fallback to the default LB policy which is pick_first. The ClientConn is
	// also expected to filter out the grpclb address when sending the addresses
	// list for pick_first.
	grpclbAddr := []resolver.Address{{Addr: "non-existent-grpclb-server-address"}}
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	state := resolver.State{ServiceConfig: grpclbConfig, Addresses: addrs}
	r.UpdateState(grpclbstate.Set(state, &grpclbstate.State{BalancerAddresses: grpclbAddr}))
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}

	// Push a resolver update with the same addresses, but with a service config
	// specifying "round_robin". The ClientConn is expected to filter out the
	// grpclb address when sending the addresses list to round_robin.
	r.UpdateState(resolver.State{
		Addresses:     addrs,
		ServiceConfig: parseServiceConfig(t, r, rrServiceConfig),
	})
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs); err != nil {
		t.Fatal(err)
	}
}

// TestBalancerSwitch_OldBalancerCallsShutdownInClose tests the scenario where
// the balancer being switched out calls Shutdown() in its Close()
// method. Verifies that this sequence of calls doesn't lead to a deadlock.
func (s) TestBalancerSwitch_OldBalancerCallsShutdownInClose(t *testing.T) {
	// Register a stub balancer which calls Shutdown() from its Close().
	scChan := make(chan balancer.SubConn, 1)
	uccsCalled := make(chan struct{}, 1)
	stub.Register(t.Name(), stub.BalancerFuncs{
		UpdateClientConnState: func(data *stub.BalancerData, ccs balancer.ClientConnState) error {
			sc, err := data.ClientConn.NewSubConn(ccs.ResolverState.Addresses, balancer.NewSubConnOptions{})
			if err != nil {
				t.Errorf("failed to create subConn: %v", err)
			}
			scChan <- sc
			close(uccsCalled)
			return nil
		},
		Close: func(*stub.BalancerData) {
			(<-scChan).Shutdown()
		},
	})

	r := manual.NewBuilderWithScheme("whatever")
	cc, err := grpc.Dial(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Push a resolver update specifying our stub balancer as the LB policy.
	scpr := parseServiceConfig(t, r, fmt.Sprintf(`{"loadBalancingPolicy": "%v"}`, t.Name()))
	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: "dummy-address"}},
		ServiceConfig: scpr,
	})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for UpdateClientConnState to be called: %v", ctx.Err())
	case <-uccsCalled:
	}

	// The following service config update will switch balancer from our stub
	// balancer to pick_first. The former will be closed, which will call
	// sc.Shutdown() inline.
	//
	// This is to make sure the sc.Shutdown() from Close() doesn't cause a
	// deadlock (e.g. trying to grab a mutex while it's already locked).
	//
	// Do it in a goroutine so this test will fail with a helpful message
	// (though the goroutine will still leak).
	done := make(chan struct{})
	go func() {
		r.UpdateState(resolver.State{
			Addresses:     []resolver.Address{{Addr: "dummy-address"}},
			ServiceConfig: parseServiceConfig(t, r, pickFirstServiceConfig),
		})
		close(done)
	}()

	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for resolver.UpdateState to finish: %v", ctx.Err())
	case <-done:
	}
}

// TestBalancerSwitch_Graceful tests the graceful switching of LB policies. It
// starts off by configuring "round_robin" on the channel and ensures that RPCs
// are successful. Then, it switches to a stub balancer which does not report a
// picker until instructed by the test do to so. At this point, the test
// verifies that RPCs are still successful using the old balancer. Then the test
// asks the new balancer to report a healthy picker and the test verifies that
// the RPCs get routed using the picker reported by the new balancer.
func (s) TestBalancerSwitch_Graceful(t *testing.T) {
	backends, cleanup := startBackendsForBalancerSwitch(t)
	defer cleanup()
	addrs := stubBackendsToResolverAddrs(backends)

	r := manual.NewBuilderWithScheme("whatever")
	cc, err := grpc.Dial(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Push a resolver update with the service config specifying "round_robin".
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	r.UpdateState(resolver.State{
		Addresses:     addrs[1:],
		ServiceConfig: parseServiceConfig(t, r, rrServiceConfig),
	})
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[1:]); err != nil {
		t.Fatal(err)
	}

	// Register a stub balancer which uses a "pick_first" balancer underneath and
	// signals on a channel when it receives ClientConn updates. But it does not
	// forward the ccUpdate to the underlying "pick_first" balancer until the test
	// asks it to do so. This allows us to test the graceful switch functionality.
	// Until the test asks the stub balancer to forward the ccUpdate, RPCs should
	// get routed to the old balancer. And once the test gives the go ahead, RPCs
	// should get routed to the new balancer.
	ccUpdateCh := make(chan struct{})
	waitToProceed := make(chan struct{})
	stub.Register(t.Name(), stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			pf := balancer.Get(pickfirst.Name)
			bd.Data = pf.Build(bd.ClientConn, bd.BuildOptions)
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			bal := bd.Data.(balancer.Balancer)
			close(ccUpdateCh)
			go func() {
				<-waitToProceed
				bal.UpdateClientConnState(ccs)
			}()
			return nil
		},
	})

	// Push a resolver update with the service config specifying our stub
	// balancer. We should see a trace event for this balancer switch. But RPCs
	// should still be routed to the old balancer since our stub balancer does not
	// report a ready picker until we ask it to do so.
	r.UpdateState(resolver.State{
		Addresses:     addrs[:1],
		ServiceConfig: r.CC.ParseServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%v": {}}]}`, t.Name())),
	})
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for a ClientConnState update on the new balancer")
	case <-ccUpdateCh:
	}
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs[1:]); err != nil {
		t.Fatal(err)
	}

	// Ask our stub balancer to forward the earlier received ccUpdate to the
	// underlying "pick_first" balancer which will result in a healthy picker
	// being reported to the channel. RPCs should start using the new balancer.
	close(waitToProceed)
	if err := pfutil.CheckRPCsToBackend(ctx, cc, addrs[0]); err != nil {
		t.Fatal(err)
	}
}
