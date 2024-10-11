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

// Package e2e_test contains e2e test cases for the Outlier Detection LB Policy.
package e2e_test

import (
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/xds/internal/balancer/outlierdetection" // To register helper functions which register/unregister Outlier Detection LB Policy.
)

var defaultTestTimeout = 5 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// Setup spins up three test backends, each listening on a port on localhost.
// Two of the backends are configured to always reply with an empty response and
// no error and one is configured to always return an error.
func setupBackends(t *testing.T) ([]string, func()) {
	t.Helper()

	backends := make([]*stubserver.StubServer, 3)
	addresses := make([]string, 3)
	// Construct and start 2 working backends.
	for i := 0; i < 2; i++ {
		backend := &stubserver.StubServer{
			EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
				return &testpb.Empty{}, nil
			},
		}
		if err := backend.StartServer(); err != nil {
			t.Fatalf("Failed to start backend: %v", err)
		}
		t.Logf("Started good TestService backend at: %q", backend.Address)
		backends[i] = backend
		addresses[i] = backend.Address
	}

	// Construct and start a failing backend.
	backend := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			return nil, errors.New("some error")
		},
	}
	if err := backend.StartServer(); err != nil {
		t.Fatalf("Failed to start backend: %v", err)
	}
	t.Logf("Started bad TestService backend at: %q", backend.Address)
	backends[2] = backend
	addresses[2] = backend.Address
	cancel := func() {
		for _, backend := range backends {
			backend.Stop()
		}
	}
	return addresses, cancel
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
				if peer.Addr != nil {
					iteration[c] = peer.Addr.String()
				}
			}
			iterations = append(iterations, iteration)
		}
		// Ensure the first iteration contains all addresses in addrs.
		for _, addr := range iterations[0] {
			gotAddrCount[addr]++
		}
		if diff := cmp.Diff(gotAddrCount, wantAddrCount); diff != "" {
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

// TestOutlierDetectionAlgorithmsE2E tests the Outlier Detection Success Rate
// and Failure Percentage algorithms in an e2e fashion. The Outlier Detection
// Balancer is configured as the top level LB Policy of the channel with a Round
// Robin child, and connects to three upstreams. Two of the upstreams are healthy and
// one is unhealthy. The two algorithms should at some point eject the failing
// upstream, causing RPC's to not be routed to that upstream, and only be
// Round Robined across the two healthy upstreams. Other than the intervals the
// unhealthy upstream is ejected, RPC's should regularly round robin
// across all three upstreams.
func (s) TestOutlierDetectionAlgorithmsE2E(t *testing.T) {
	tests := []struct {
		name     string
		odscJSON string
	}{
		{
			name: "Success Rate Algorithm",
			odscJSON: `
{
  "loadBalancingConfig": [
    {
      "outlier_detection_experimental": {
        "interval": "0.050s",
		"baseEjectionTime": "0.100s",
		"maxEjectionTime": "300s",
		"maxEjectionPercent": 33,
		"successRateEjection": {
			"stdevFactor": 50,
			"enforcementPercentage": 100,
			"minimumHosts": 3,
			"requestVolume": 5
		},
        "childPolicy": [{"round_robin": {}}]
      }
    }
  ]
}`,
		},
		{
			name: "Failure Percentage Algorithm",
			odscJSON: `
{
  "loadBalancingConfig": [
    {
      "outlier_detection_experimental": {
        "interval": "0.050s",
		"baseEjectionTime": "0.100s",
		"maxEjectionTime": "300s",
		"maxEjectionPercent": 33,
		"failurePercentageEjection": {
			"threshold": 50,
			"enforcementPercentage": 100,
			"minimumHosts": 3,
			"requestVolume": 5
		},
        "childPolicy": [{"round_robin": {}}
		]
      }
    }
  ]
}`,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			addresses, cancel := setupBackends(t)
			defer cancel()

			mr := manual.NewBuilderWithScheme("od-e2e")
			defer mr.Close()

			sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(test.odscJSON)
			// The full list of addresses.
			fullAddresses := []resolver.Address{
				{Addr: addresses[0]},
				{Addr: addresses[1]},
				{Addr: addresses[2]},
			}
			mr.InitialState(resolver.State{
				Addresses:     fullAddresses,
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

			// At first, due to no statistics on each of the backends, the 3
			// upstreams should all be round robined across.
			if err = checkRoundRobinRPCs(ctx, testServiceClient, fullAddresses); err != nil {
				t.Fatalf("error in expected round robin: %v", err)
			}

			// The addresses which don't return errors.
			okAddresses := []resolver.Address{
				{Addr: addresses[0]},
				{Addr: addresses[1]},
			}
			// After calling the three upstreams, one of them constantly error
			// and should eventually be ejected for a period of time. This
			// period of time should cause the RPC's to be round robined only
			// across the two that are healthy.
			if err = checkRoundRobinRPCs(ctx, testServiceClient, okAddresses); err != nil {
				t.Fatalf("error in expected round robin: %v", err)
			}

			// The failing upstream isn't ejected indefinitely, and eventually
			// should be unejected in subsequent iterations of the interval
			// algorithm as per the spec for the two specific algorithms.
			if err = checkRoundRobinRPCs(ctx, testServiceClient, fullAddresses); err != nil {
				t.Fatalf("error in expected round robin: %v", err)
			}
		})
	}
}

// TestNoopConfiguration tests the Outlier Detection Balancer configured with a
// noop configuration. The noop configuration should cause the Outlier Detection
// Balancer to not count RPC's, and thus never eject any upstreams and continue
// to route to every upstream connected to, even if they continuously error.
// Once the Outlier Detection Balancer gets reconfigured with configuration
// requiring counting RPC's, the Outlier Detection Balancer should start
// ejecting any upstreams as specified in the configuration.
func (s) TestNoopConfiguration(t *testing.T) {
	addresses, cancel := setupBackends(t)
	defer cancel()

	mr := manual.NewBuilderWithScheme("od-e2e")
	defer mr.Close()

	noopODServiceConfigJSON := `
{
  "loadBalancingConfig": [
    {
      "outlier_detection_experimental": {
        "interval": "0.050s",
		"baseEjectionTime": "0.100s",
		"maxEjectionTime": "300s",
		"maxEjectionPercent": 33,
        "childPolicy": [{"round_robin": {}}]
      }
    }
  ]
}`
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(noopODServiceConfigJSON)
	// The full list of addresses.
	fullAddresses := []resolver.Address{
		{Addr: addresses[0]},
		{Addr: addresses[1]},
		{Addr: addresses[2]},
	}
	mr.InitialState(resolver.State{
		Addresses:     fullAddresses,
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

	for i := 0; i < 2; i++ {
		// Since the Outlier Detection Balancer starts with a noop
		// configuration, it shouldn't count RPCs or eject any upstreams. Thus,
		// even though an upstream it connects to constantly errors, it should
		// continue to Round Robin across every upstream.
		if err := checkRoundRobinRPCs(ctx, testServiceClient, fullAddresses); err != nil {
			t.Fatalf("error in expected round robin: %v", err)
		}
	}

	// Reconfigure the Outlier Detection Balancer with a configuration that
	// specifies to count RPC's and eject upstreams. Due to the balancer no
	// longer being a noop, it should eject any unhealthy addresses as specified
	// by the failure percentage portion of the configuration.
	countingODServiceConfigJSON := `
{
  "loadBalancingConfig": [
    {
      "outlier_detection_experimental": {
        "interval": "0.050s",
		"baseEjectionTime": "0.100s",
		"maxEjectionTime": "300s",
		"maxEjectionPercent": 33,
		"failurePercentageEjection": {
			"threshold": 50,
			"enforcementPercentage": 100,
			"minimumHosts": 3,
			"requestVolume": 5
		},
        "childPolicy": [{"round_robin": {}}]
      }
    }
  ]
}`
	sc = internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(countingODServiceConfigJSON)

	mr.UpdateState(resolver.State{
		Addresses:     fullAddresses,
		ServiceConfig: sc,
	})

	// At first on the reconfigured balancer, the balancer has no stats
	// collected about upstreams. Thus, it should at first route across the full
	// upstream list.
	if err = checkRoundRobinRPCs(ctx, testServiceClient, fullAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}

	// The addresses which don't return errors.
	okAddresses := []resolver.Address{
		{Addr: addresses[0]},
		{Addr: addresses[1]},
	}
	// Now that the reconfigured balancer has data about the failing upstream,
	// it should eject the upstream and only route across the two healthy
	// upstreams.
	if err = checkRoundRobinRPCs(ctx, testServiceClient, okAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}
}
