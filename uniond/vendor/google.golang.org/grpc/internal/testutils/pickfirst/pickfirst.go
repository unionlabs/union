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

// Package pickfirst contains helper functions to check for pickfirst load
// balancing of RPCs in tests.
package pickfirst

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// CheckRPCsToBackend makes a bunch of RPCs on the given ClientConn and verifies
// if the RPCs are routed to a peer matching wantAddr.
//
// Returns a non-nil error if context deadline expires before all RPCs begin to
// be routed to the peer matching wantAddr, or if the backend returns RPC errors.
func CheckRPCsToBackend(ctx context.Context, cc *grpc.ClientConn, wantAddr resolver.Address) error {
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	// Make sure that 20 RPCs in a row reach the expected backend. Some
	// tests switch from round_robin back to pick_first and call this
	// function. None of our tests spin up more than 10 backends. So,
	// waiting for 20 RPCs to reach a single backend would a decent
	// indicator of having switched to pick_first.
	count := 0
	for {
		time.Sleep(time.Millisecond)
		if ctx.Err() != nil {
			return fmt.Errorf("timeout waiting for RPC to be routed to %s", wantAddr.Addr)
		}
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			// Some tests remove backends and check if pick_first is happening across
			// the remaining backends. In such cases, RPCs can initially fail on the
			// connection using the removed backend. Just keep retrying and eventually
			// the connection using the removed backend will shutdown and will be
			// removed.
			continue
		}
		if peer.Addr.String() != wantAddr.Addr {
			count = 0
			continue
		}
		count++
		if count > 20 {
			break
		}
	}
	// Make sure subsequent RPCs are all routed to the same backend.
	for i := 0; i < 10; i++ {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			return fmt.Errorf("EmptyCall() = %v, want <nil>", err)
		}
		if gotAddr := peer.Addr.String(); gotAddr != wantAddr.Addr {
			return fmt.Errorf("rpc sent to peer %q, want peer %q", gotAddr, wantAddr)
		}
	}
	return nil
}
