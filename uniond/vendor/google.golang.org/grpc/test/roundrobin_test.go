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
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/channelz"
	imetadata "google.golang.org/grpc/internal/metadata"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	rrutil "google.golang.org/grpc/internal/testutils/roundrobin"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const rrServiceConfig = `{"loadBalancingConfig": [{"round_robin":{}}]}`

func testRoundRobinBasic(ctx context.Context, t *testing.T, opts ...grpc.DialOption) (*grpc.ClientConn, *manual.Resolver, []*stubserver.StubServer) {
	t.Helper()

	r := manual.NewBuilderWithScheme("whatever")

	const backendCount = 5
	backends := make([]*stubserver.StubServer, backendCount)
	addrs := make([]resolver.Address, backendCount)
	for i := 0; i < backendCount; i++ {
		backend := &stubserver.StubServer{
			EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
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
		grpc.WithDefaultServiceConfig(rrServiceConfig),
	}
	dopts = append(dopts, opts...)
	cc, err := grpc.NewClient(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("grpc.NewClient() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })
	client := testgrpc.NewTestServiceClient(cc)

	// At this point, the resolver has not returned any addresses to the channel.
	// This RPC must block until the context expires.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("EmptyCall() = %s, want %s", status.Code(err), codes.DeadlineExceeded)
	}

	r.UpdateState(resolver.State{Addresses: addrs})
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs); err != nil {
		t.Fatal(err)
	}
	return cc, r, backends
}

// TestRoundRobin_Basic tests the most basic scenario for round_robin. It brings
// up a bunch of backends and verifies that RPCs are getting round robin-ed
// across these backends.
func (s) TestRoundRobin_Basic(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testRoundRobinBasic(ctx, t)
}

// TestRoundRobin_AddressesRemoved tests the scenario where a bunch of backends
// are brought up, and round_robin is configured as the LB policy and RPCs are
// being correctly round robin-ed across these backends. We then send a resolver
// update with no addresses and verify that the channel enters TransientFailure
// and RPCs fail with an expected error message.
func (s) TestRoundRobin_AddressesRemoved(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, r, _ := testRoundRobinBasic(ctx, t)

	// Send a resolver update with no addresses. This should push the channel into
	// TransientFailure.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{}})
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	const msgWant = "produced zero addresses"
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); !strings.Contains(status.Convert(err).Message(), msgWant) {
		t.Fatalf("EmptyCall() = %v, want Contains(Message(), %q)", err, msgWant)
	}
}

// TestRoundRobin_NewAddressWhileBlocking tests the case where round_robin is
// configured on a channel, things are working as expected and then a resolver
// updates removes all addresses. An RPC attempted at this point in time will be
// blocked because there are no valid ¡ds. This test verifies that when new
// backends are added, the RPC is able to complete.
func (s) TestRoundRobin_NewAddressWhileBlocking(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, r, backends := testRoundRobinBasic(ctx, t)

	// Send a resolver update with no addresses. This should push the channel into
	// TransientFailure.
	r.UpdateState(resolver.State{Addresses: []resolver.Address{}})
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	client := testgrpc.NewTestServiceClient(cc)
	doneCh := make(chan struct{})
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

// TestRoundRobin_OneServerDown tests the scenario where a channel is configured
// to round robin across a set of backends, and things are working correctly.
// One backend goes down. The test verifies that going forward, RPCs are round
// robin-ed across the remaining set of backends.
func (s) TestRoundRobin_OneServerDown(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, _, backends := testRoundRobinBasic(ctx, t)

	// Stop one backend. RPCs should round robin across the remaining backends.
	backends[len(backends)-1].Stop()

	addrs := make([]resolver.Address, len(backends)-1)
	for i := 0; i < len(backends)-1; i++ {
		addrs[i] = resolver.Address{Addr: backends[i].Address}
	}
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, addrs); err != nil {
		t.Fatalf("RPCs are not being round robined across remaining servers: %v", err)
	}
}

// TestRoundRobin_AllServersDown tests the scenario where a channel is
// configured to round robin across a set of backends, and things are working
// correctly. Then, all backends go down. The test verifies that the channel
// moves to TransientFailure and failfast RPCs fail with Unavailable.
func (s) TestRoundRobin_AllServersDown(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, _, backends := testRoundRobinBasic(ctx, t)

	// Stop all backends.
	for _, b := range backends {
		b.Stop()
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Failfast RPCs should fail with Unavailable.
	client := testgrpc.NewTestServiceClient(cc)

	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("EmptyCall got err: %v; want Unavailable", err)
	}
}

// TestRoundRobin_UpdateAddressAttributes tests the scenario where the addresses
// returned by the resolver contain attributes. The test verifies that the
// attributes contained in the addresses show up as RPC metadata in the backend.
func (s) TestRoundRobin_UpdateAddressAttributes(t *testing.T) {
	const (
		testMDKey   = "test-md"
		testMDValue = "test-md-value"
	)
	r := manual.NewBuilderWithScheme("whatever")

	// Spin up a StubServer to serve as a backend. The implementation verifies
	// that the expected metadata is received.
	testMDChan := make(chan []string, 1)
	backend := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			md, ok := metadata.FromIncomingContext(ctx)
			if ok {
				select {
				case testMDChan <- md[testMDKey]:
				case <-ctx.Done():
					return nil, ctx.Err()
				}
			}
			return &testpb.Empty{}, nil
		},
	}
	if err := backend.StartServer(); err != nil {
		t.Fatalf("Failed to start backend: %v", err)
	}
	t.Logf("Started TestService backend at: %q", backend.Address)
	t.Cleanup(func() { backend.Stop() })

	// Dial the backend with round_robin as the LB policy.
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
		grpc.WithDefaultServiceConfig(rrServiceConfig),
	}
	cc, err := grpc.Dial(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	// Send a resolver update with no address attributes.
	addr := resolver.Address{Addr: backend.Address}
	r.UpdateState(resolver.State{Addresses: []resolver.Address{addr}})

	// Make an RPC and ensure it does not contain the metadata we are looking for.
	client := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("EmptyCall() = %v, want <nil>", err)
	}
	select {
	case <-ctx.Done():
		t.Fatalf("Timeout when waiting for metadata received in RPC")
	case md := <-testMDChan:
		if len(md) != 0 {
			t.Fatalf("received metadata %v, want nil", md)
		}
	}

	// Send a resolver update with address attributes.
	addrWithAttributes := imetadata.Set(addr, metadata.Pairs(testMDKey, testMDValue))
	r.UpdateState(resolver.State{Addresses: []resolver.Address{addrWithAttributes}})

	// Make an RPC and ensure it contains the metadata we are looking for. The
	// resolver update isn't processed synchronously, so we wait some time before
	// failing if some RPCs do not contain it.
Done:
	for {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			t.Fatalf("EmptyCall() = %v, want <nil>", err)
		}
		select {
		case <-ctx.Done():
			t.Fatalf("Timeout when waiting for metadata received in RPC")
		case md := <-testMDChan:
			if len(md) == 1 && md[0] == testMDValue {
				break Done
			}
		}
		time.Sleep(defaultTestShortTimeout)
	}
}
