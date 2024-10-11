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
 */

package leastrequest

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
)

const (
	defaultTestTimeout = 5 * time.Second
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestParseConfig(t *testing.T) {
	parser := bb{}
	tests := []struct {
		name    string
		input   string
		wantCfg serviceconfig.LoadBalancingConfig
		wantErr string
	}{
		{
			name:  "happy-case-default",
			input: `{}`,
			wantCfg: &LBConfig{
				ChoiceCount: 2,
			},
		},
		{
			name:  "happy-case-choice-count-set",
			input: `{"choiceCount": 3}`,
			wantCfg: &LBConfig{
				ChoiceCount: 3,
			},
		},
		{
			name:  "happy-case-choice-count-greater-than-ten",
			input: `{"choiceCount": 11}`,
			wantCfg: &LBConfig{
				ChoiceCount: 10,
			},
		},
		{
			name:    "choice-count-less-than-2",
			input:   `{"choiceCount": 1}`,
			wantErr: "must be >= 2",
		},
		{
			name:    "invalid-json",
			input:   "{{invalidjson{{",
			wantErr: "invalid character",
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			gotCfg, gotErr := parser.ParseConfig(json.RawMessage(test.input))
			// Substring match makes this very tightly coupled to the
			// internalserviceconfig.BalancerConfig error strings. However, it
			// is important to distinguish the different types of error messages
			// possible as the parser has a few defined buckets of ways it can
			// error out.
			if (gotErr != nil) != (test.wantErr != "") {
				t.Fatalf("ParseConfig(%v) = %v, wantErr %v", test.input, gotErr, test.wantErr)
			}
			if gotErr != nil && !strings.Contains(gotErr.Error(), test.wantErr) {
				t.Fatalf("ParseConfig(%v) = %v, wantErr %v", test.input, gotErr, test.wantErr)
			}
			if test.wantErr != "" {
				return
			}
			if diff := cmp.Diff(gotCfg, test.wantCfg); diff != "" {
				t.Fatalf("ParseConfig(%v) got unexpected output, diff (-got +want): %v", test.input, diff)
			}
		})
	}
}

// setupBackends spins up three test backends, each listening on a port on
// localhost. The three backends always reply with an empty response with no
// error, and for streaming receive until hitting an EOF error.
func setupBackends(t *testing.T) []string {
	t.Helper()
	const numBackends = 3
	addresses := make([]string, numBackends)
	// Construct and start three working backends.
	for i := 0; i < numBackends; i++ {
		backend := &stubserver.StubServer{
			EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
				return &testpb.Empty{}, nil
			},
			FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
				<-stream.Context().Done()
				return nil
			},
		}
		if err := backend.StartServer(); err != nil {
			t.Fatalf("Failed to start backend: %v", err)
		}
		t.Logf("Started good TestService backend at: %q", backend.Address)
		t.Cleanup(func() { backend.Stop() })
		addresses[i] = backend.Address
	}
	return addresses
}

// checkRoundRobinRPCs verifies that EmptyCall RPCs on the given ClientConn,
// connected to a server exposing the test.grpc_testing.TestService, are
// roundrobined across the given backend addresses.
//
// Returns a non-nil error if context deadline expires before RPCs start to get
// roundrobined across the given backends.
func checkRoundRobinRPCs(ctx context.Context, client testgrpc.TestServiceClient, addrs []resolver.Address) error {
	wantAddrCount := make(map[string]int)
	for _, addr := range addrs {
		wantAddrCount[addr.Addr]++
	}
	gotAddrCount := make(map[string]int)
	for ; ctx.Err() == nil; <-time.After(time.Millisecond) {
		gotAddrCount = make(map[string]int)
		// Perform 3 iterations.
		var iterations [][]string
		for i := 0; i < 3; i++ {
			iteration := make([]string, len(addrs))
			for c := 0; c < len(addrs); c++ {
				var peer peer.Peer
				client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer))
				iteration[c] = peer.Addr.String()
			}
			iterations = append(iterations, iteration)
		}
		// Ensure the first iteration contains all addresses in addrs.
		for _, addr := range iterations[0] {
			gotAddrCount[addr]++
		}
		if !cmp.Equal(gotAddrCount, wantAddrCount) {
			continue
		}
		// Ensure all three iterations contain the same addresses.
		if !cmp.Equal(iterations[0], iterations[1]) || !cmp.Equal(iterations[0], iterations[2]) {
			continue
		}
		return nil
	}
	return fmt.Errorf("timeout when waiting for roundrobin distribution of RPCs across addresses: %v; got: %v", addrs, gotAddrCount)
}

// TestLeastRequestE2E tests the Least Request LB policy in an e2e style. The
// Least Request balancer is configured as the top level balancer of the
// channel, and is passed three addresses. Eventually, the test creates three
// streams, which should be on certain backends according to the least request
// algorithm. The randomness in the picker is injected in the test to be
// deterministic, allowing the test to make assertions on the distribution.
func (s) TestLeastRequestE2E(t *testing.T) {
	defer func(u func() uint32) {
		randuint32 = u
	}(randuint32)
	var index int
	indexes := []uint32{
		0, 0, 1, 1, 2, 2, // Triggers a round robin distribution.
	}
	randuint32 = func() uint32 {
		ret := indexes[index%len(indexes)]
		index++
		return ret
	}
	addresses := setupBackends(t)

	mr := manual.NewBuilderWithScheme("lr-e2e")
	defer mr.Close()

	// Configure least request as top level balancer of channel.
	lrscJSON := `
{
  "loadBalancingConfig": [
    {
      "least_request_experimental": {
        "choiceCount": 2
      }
    }
  ]
}`
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(lrscJSON)
	firstThreeAddresses := []resolver.Address{
		{Addr: addresses[0]},
		{Addr: addresses[1]},
		{Addr: addresses[2]},
	}
	mr.InitialState(resolver.State{
		Addresses:     firstThreeAddresses,
		ServiceConfig: sc,
	})

	cc, err := grpc.NewClient(mr.Scheme()+":///", grpc.WithResolvers(mr), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient() failed: %v", err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testServiceClient := testgrpc.NewTestServiceClient(cc)

	// Wait for all 3 backends to round robin across. The happens because a
	// SubConn transitioning into READY causes a new picker update. Once the
	// picker update with all 3 backends is present, this test can start to make
	// assertions based on those backends.
	if err := checkRoundRobinRPCs(ctx, testServiceClient, firstThreeAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}

	// Map ordering of READY SubConns is non deterministic. Thus, perform 3 RPCs
	// mocked from the random to each index to learn the addresses of SubConns
	// at each index.
	index = 0
	peerAtIndex := make([]string, 3)
	var peer0 peer.Peer
	if _, err := testServiceClient.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer0)); err != nil {
		t.Fatalf("testServiceClient.EmptyCall failed: %v", err)
	}
	peerAtIndex[0] = peer0.Addr.String()
	if _, err := testServiceClient.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer0)); err != nil {
		t.Fatalf("testServiceClient.EmptyCall failed: %v", err)
	}
	peerAtIndex[1] = peer0.Addr.String()
	if _, err := testServiceClient.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer0)); err != nil {
		t.Fatalf("testServiceClient.EmptyCall failed: %v", err)
	}
	peerAtIndex[2] = peer0.Addr.String()

	// Start streaming RPCs, but do not finish them. Each subsequent stream
	// should be started according to the least request algorithm, and chosen
	// between the indexes provided.
	index = 0
	indexes = []uint32{
		0, 0, // Causes first stream to be on first address.
		0, 1, // Compares first address (one RPC) to second (no RPCs), so choose second.
		1, 2, // Compares second address (one RPC) to third (no RPCs), so choose third.
		0, 3, // Causes another stream on first address.
		1, 0, // Compares second address (one RPC) to first (two RPCs), so choose second.
		2, 0, // Compares third address (one RPC) to first (two RPCs), so choose third.
		0, 0, // Causes another stream on first address.
		2, 2, // Causes a stream on third address.
		2, 1, // Compares third address (three RPCs) to second (two RPCs), so choose third.
	}
	wantIndex := []uint32{0, 1, 2, 0, 1, 2, 0, 2, 1}

	// Start streaming RPC's, but do not finish them. Each created stream should
	// be started based on the least request algorithm and injected randomness
	// (see indexes slice above for exact expectations).
	for _, wantIndex := range wantIndex {
		stream, err := testServiceClient.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("testServiceClient.FullDuplexCall failed: %v", err)
		}
		p, ok := peer.FromContext(stream.Context())
		if !ok {
			t.Fatalf("testServiceClient.FullDuplexCall has no Peer")
		}
		if p.Addr.String() != peerAtIndex[wantIndex] {
			t.Fatalf("testServiceClient.FullDuplexCall's Peer got: %v, want: %v", p.Addr.String(), peerAtIndex[wantIndex])
		}
	}
}

// TestLeastRequestPersistsCounts tests that the Least Request Balancer persists
// counts once it gets a new picker update. It first updates the Least Request
// Balancer with two backends, and creates a bunch of streams on them. Then, it
// updates the Least Request Balancer with three backends, including the two
// previous. Any created streams should then be started on the new backend.
func (s) TestLeastRequestPersistsCounts(t *testing.T) {
	defer func(u func() uint32) {
		randuint32 = u
	}(randuint32)
	var index int
	indexes := []uint32{
		0, 0, 1, 1,
	}
	randuint32 = func() uint32 {
		ret := indexes[index%len(indexes)]
		index++
		return ret
	}
	addresses := setupBackends(t)

	mr := manual.NewBuilderWithScheme("lr-e2e")
	defer mr.Close()

	// Configure least request as top level balancer of channel.
	lrscJSON := `
{
  "loadBalancingConfig": [
    {
      "least_request_experimental": {
        "choiceCount": 2
      }
    }
  ]
}`
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(lrscJSON)
	firstTwoAddresses := []resolver.Address{
		{Addr: addresses[0]},
		{Addr: addresses[1]},
	}
	mr.InitialState(resolver.State{
		Addresses:     firstTwoAddresses,
		ServiceConfig: sc,
	})

	cc, err := grpc.NewClient(mr.Scheme()+":///", grpc.WithResolvers(mr), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient() failed: %v", err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testServiceClient := testgrpc.NewTestServiceClient(cc)

	// Wait for the two backends to round robin across. The happens because a
	// SubConn transitioning into READY causes a new picker update. Once the
	// picker update with the two backends is present, this test can start to
	// populate those backends with streams.
	if err := checkRoundRobinRPCs(ctx, testServiceClient, firstTwoAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}

	// Start 50 streaming RPCs, and leave them unfinished for the duration of
	// the test. This will populate the first two addresses with many active
	// RPCs.
	for i := 0; i < 50; i++ {
		_, err := testServiceClient.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("testServiceClient.FullDuplexCall failed: %v", err)
		}
	}

	// Update the least request balancer to choice count 3. Also update the
	// address list adding a third address. Alongside the injected randomness,
	// this should trigger the least request balancer to search all created
	// SubConns. Thus, since address 3 is the new address and the first two
	// addresses are populated with RPCs, once the picker update of all 3 READY
	// SubConns takes effect, all new streams should be started on address 3.
	index = 0
	indexes = []uint32{
		0, 1, 2, 3, 4, 5,
	}
	lrscJSON = `
{
  "loadBalancingConfig": [
    {
      "least_request_experimental": {
        "choiceCount": 3
      }
    }
  ]
}`
	sc = internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(lrscJSON)
	fullAddresses := []resolver.Address{
		{Addr: addresses[0]},
		{Addr: addresses[1]},
		{Addr: addresses[2]},
	}
	mr.UpdateState(resolver.State{
		Addresses:     fullAddresses,
		ServiceConfig: sc,
	})
	newAddress := fullAddresses[2]
	// Poll for only address 3 to show up. This requires a polling loop because
	// picker update with all three SubConns doesn't take into effect
	// immediately, needs the third SubConn to become READY.
	if err := checkRoundRobinRPCs(ctx, testServiceClient, []resolver.Address{newAddress}); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}

	// Start 25 rpcs, but don't finish them. They should all start on address 3,
	// since the first two addresses both have 25 RPCs (and randomness
	// injection/choiceCount causes all 3 to be compared every iteration).
	for i := 0; i < 25; i++ {
		stream, err := testServiceClient.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("testServiceClient.FullDuplexCall failed: %v", err)
		}
		p, ok := peer.FromContext(stream.Context())
		if !ok {
			t.Fatalf("testServiceClient.FullDuplexCall has no Peer")
		}
		if p.Addr.String() != addresses[2] {
			t.Fatalf("testServiceClient.FullDuplexCall's Peer got: %v, want: %v", p.Addr.String(), addresses[2])
		}
	}

	// Now 25 RPC's are active on each address, the next three RPC's should
	// round robin, since choiceCount is three and the injected random indexes
	// cause it to search all three addresses for fewest outstanding requests on
	// each iteration.
	wantAddrCount := map[string]int{
		addresses[0]: 1,
		addresses[1]: 1,
		addresses[2]: 1,
	}
	gotAddrCount := make(map[string]int)
	for i := 0; i < len(addresses); i++ {
		stream, err := testServiceClient.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("testServiceClient.FullDuplexCall failed: %v", err)
		}
		p, ok := peer.FromContext(stream.Context())
		if !ok {
			t.Fatalf("testServiceClient.FullDuplexCall has no Peer")
		}
		if p.Addr != nil {
			gotAddrCount[p.Addr.String()]++
		}
	}
	if diff := cmp.Diff(gotAddrCount, wantAddrCount); diff != "" {
		t.Fatalf("addr count (-got:, +want): %v", diff)
	}
}

// TestConcurrentRPCs tests concurrent RPCs on the least request balancer. It
// configures a channel with a least request balancer as the top level balancer,
// and makes 100 RPCs asynchronously. This makes sure no race conditions happen
// in this scenario.
func (s) TestConcurrentRPCs(t *testing.T) {
	addresses := setupBackends(t)

	mr := manual.NewBuilderWithScheme("lr-e2e")
	defer mr.Close()

	// Configure least request as top level balancer of channel.
	lrscJSON := `
{
  "loadBalancingConfig": [
    {
      "least_request_experimental": {
        "choiceCount": 2
      }
    }
  ]
}`
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(lrscJSON)
	firstTwoAddresses := []resolver.Address{
		{Addr: addresses[0]},
		{Addr: addresses[1]},
	}
	mr.InitialState(resolver.State{
		Addresses:     firstTwoAddresses,
		ServiceConfig: sc,
	})

	cc, err := grpc.NewClient(mr.Scheme()+":///", grpc.WithResolvers(mr), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient() failed: %v", err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testServiceClient := testgrpc.NewTestServiceClient(cc)

	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < 5; j++ {
				testServiceClient.EmptyCall(ctx, &testpb.Empty{})
			}
		}()
	}
	wg.Wait()

}
