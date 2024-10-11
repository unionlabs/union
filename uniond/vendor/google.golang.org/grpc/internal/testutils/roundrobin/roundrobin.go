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

// Package roundrobin contains helper functions to check for roundrobin and
// weighted-roundrobin load balancing of RPCs in tests.
package roundrobin

import (
	"context"
	"fmt"
	"math"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

var logger = grpclog.Component("testutils-roundrobin")

// waitForTrafficToReachBackends repeatedly makes RPCs using the provided
// TestServiceClient until RPCs reach all backends specified in addrs, or the
// context expires, in which case a non-nil error is returned.
func waitForTrafficToReachBackends(ctx context.Context, client testgrpc.TestServiceClient, addrs []resolver.Address) error {
	// Make sure connections to all backends are up. We need to do this two
	// times (to be sure that round_robin has kicked in) because the channel
	// could have been configured with a different LB policy before the switch
	// to round_robin. And the previous LB policy could be sharing backends with
	// round_robin, and therefore in the first iteration of this loop, RPCs
	// could land on backends owned by the previous LB policy.
	for j := 0; j < 2; j++ {
		for i := 0; i < len(addrs); i++ {
			for {
				time.Sleep(time.Millisecond)
				if ctx.Err() != nil {
					return fmt.Errorf("timeout waiting for connection to %q to be up", addrs[i].Addr)
				}
				var peer peer.Peer
				if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
					// Some tests remove backends and check if round robin is
					// happening across the remaining backends. In such cases,
					// RPCs can initially fail on the connection using the
					// removed backend. Just keep retrying and eventually the
					// connection using the removed backend will shutdown and
					// will be removed.
					continue
				}
				if peer.Addr.String() == addrs[i].Addr {
					break
				}
			}
		}
	}
	return nil
}

// CheckRoundRobinRPCs verifies that EmptyCall RPCs on the given ClientConn,
// connected to a server exposing the test.grpc_testing.TestService, are
// roundrobined across the given backend addresses.
//
// Returns a non-nil error if context deadline expires before RPCs start to get
// roundrobined across the given backends.
func CheckRoundRobinRPCs(ctx context.Context, client testgrpc.TestServiceClient, addrs []resolver.Address) error {
	if err := waitForTrafficToReachBackends(ctx, client, addrs); err != nil {
		return err
	}

	// At this point, RPCs are getting successfully executed at the backends
	// that we care about. To support duplicate addresses (in addrs) and
	// backends being removed from the list of addresses passed to the
	// roundrobin LB, we do the following:
	// 1. Determine the count of RPCs that we expect each of our backends to
	//    receive per iteration.
	// 2. Wait until the same pattern repeats a few times, or the context
	//    deadline expires.
	wantAddrCount := make(map[string]int)
	for _, addr := range addrs {
		wantAddrCount[addr.Addr]++
	}
	for ; ctx.Err() == nil; <-time.After(time.Millisecond) {
		// Perform 3 more iterations.
		var iterations [][]string
		for i := 0; i < 3; i++ {
			iteration := make([]string, len(addrs))
			for c := 0; c < len(addrs); c++ {
				var peer peer.Peer
				if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
					return fmt.Errorf("EmptyCall() = %v, want <nil>", err)
				}
				iteration[c] = peer.Addr.String()
			}
			iterations = append(iterations, iteration)
		}
		// Ensure the first iteration contains all addresses in addrs.
		gotAddrCount := make(map[string]int)
		for _, addr := range iterations[0] {
			gotAddrCount[addr]++
		}
		if diff := cmp.Diff(gotAddrCount, wantAddrCount); diff != "" {
			logger.Infof("non-roundrobin, got address count in one iteration: %v, want: %v, Diff: %s", gotAddrCount, wantAddrCount, diff)
			continue
		}
		// Ensure all three iterations contain the same addresses.
		if !cmp.Equal(iterations[0], iterations[1]) || !cmp.Equal(iterations[0], iterations[2]) {
			logger.Infof("non-roundrobin, first iter: %v, second iter: %v, third iter: %v", iterations[0], iterations[1], iterations[2])
			continue
		}
		return nil
	}
	return fmt.Errorf("timeout when waiting for roundrobin distribution of RPCs across addresses: %v", addrs)
}

// CheckWeightedRoundRobinRPCs verifies that EmptyCall RPCs on the given
// ClientConn, connected to a server exposing the test.grpc_testing.TestService,
// are weighted roundrobined (with randomness) across the given backend
// addresses.
//
// Returns a non-nil error if context deadline expires before RPCs start to get
// roundrobined across the given backends.
func CheckWeightedRoundRobinRPCs(ctx context.Context, client testgrpc.TestServiceClient, addrs []resolver.Address) error {
	if err := waitForTrafficToReachBackends(ctx, client, addrs); err != nil {
		return err
	}

	// At this point, RPCs are getting successfully executed at the backends
	// that we care about. To take the randomness of the WRR into account, we
	// look for approximate distribution instead of exact.
	wantAddrCount := make(map[string]int)
	for _, addr := range addrs {
		wantAddrCount[addr.Addr]++
	}
	wantRatio := make(map[string]float64)
	for addr, count := range wantAddrCount {
		wantRatio[addr] = float64(count) / float64(len(addrs))
	}

	// There is a small possibility that RPCs are reaching backends that we
	// don't expect them to reach here. The can happen because:
	// - at time T0, the list of backends [A, B, C, D].
	// - at time T1, the test updates the list of backends to [A, B, C], and
	//   immediately starts attempting to check the distribution of RPCs to the
	//   new backends.
	// - there is no way for the test to wait for a new picker to be pushed on
	//   to the channel (which contains the updated list of backends) before
	//   starting to attempt the RPC distribution checks.
	// - This is usually a transitory state and will eventually fix itself when
	//   the new picker is pushed on the channel, and RPCs will start getting
	//   routed to only backends that we care about.
	//
	// We work around this situation by using two loops. The inner loop contains
	// the meat of the calculations, and includes the logic which factors out
	// the randomness in weighted roundrobin. If we ever see an RPCs getting
	// routed to a backend that we dont expect it to get routed to, we break
	// from the inner loop thereby resetting all state and start afresh.
	for {
		results := make(map[string]float64)
		totalCount := float64(0)
	InnerLoop:
		for {
			if ctx.Err() != nil {
				return fmt.Errorf("timeout when waiting for roundrobin distribution of RPCs across addresses: %v", addrs)
			}
			for i := 0; i < len(addrs); i++ {
				var peer peer.Peer
				if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
					return fmt.Errorf("EmptyCall() = %v, want <nil>", err)
				}
				if addr := peer.Addr.String(); wantAddrCount[addr] == 0 {
					break InnerLoop
				}
				results[peer.Addr.String()]++
			}
			totalCount += float64(len(addrs))

			gotRatio := make(map[string]float64)
			for addr, count := range results {
				gotRatio[addr] = count / totalCount
			}
			if equalApproximate(gotRatio, wantRatio) {
				return nil
			}
			logger.Infof("non-weighted-roundrobin, gotRatio: %v, wantRatio: %v", gotRatio, wantRatio)
		}
		<-time.After(time.Millisecond)
	}
}

func equalApproximate(got, want map[string]float64) bool {
	if len(got) != len(want) {
		return false
	}
	opt := cmp.Comparer(func(x, y float64) bool {
		delta := math.Abs(x - y)
		mean := math.Abs(x+y) / 2.0
		return delta/mean < 0.05
	})
	for addr := range want {
		if !cmp.Equal(got[addr], want[addr], opt) {
			return false
		}
	}
	return true
}
