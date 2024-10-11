/*
 *
 * Copyright 2024 gRPC authors.
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
	"net"
	"sync"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/interop"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/stats"
)

// TestPeerForClientStatsHandler configures a stats handler that
// verifies that peer is sent all stats handler callouts instead
// of Begin and PickerUpdated.
func (s) TestPeerForClientStatsHandler(t *testing.T) {
	psh := &peerStatsHandler{}

	// Stats callouts & peer object population.
	// Note:
	// * Begin stats lack peer info (RPC starts pre-resolution).
	// * PickerUpdated: no peer info (picker lacks transport details).
	expectedCallouts := map[stats.RPCStats]bool{
		&stats.OutPayload{}:    true,
		&stats.InHeader{}:      true,
		&stats.OutHeader{}:     true,
		&stats.InTrailer{}:     true,
		&stats.OutTrailer{}:    true,
		&stats.End{}:           true,
		&stats.Begin{}:         false,
		&stats.PickerUpdated{}: false,
	}

	// Start server.
	l, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}
	s := grpc.NewServer()
	testgrpc.RegisterTestServiceServer(s, interop.NewTestServer())
	errCh := make(chan error)
	go func() {
		errCh <- s.Serve(l)
	}()
	defer func() {
		s.Stop()
		if err := <-errCh; err != nil {
			t.Error(err)
		}
	}()

	// Create client with stats handler and do some calls.
	cc, err := grpc.NewClient(
		l.Addr().String(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithStatsHandler(psh))
	if err != nil {
		t.Fatal(err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	interop.DoEmptyUnaryCall(ctx, client)

	psh.mu.Lock()
	pshArgs := psh.args
	psh.mu.Unlock()

	// Fetch the total unique stats handlers with peer != nil
	uniqueStatsTypes := make(map[string]struct{})
	for _, callbackArgs := range pshArgs {
		key := fmt.Sprintf("%T", callbackArgs.rpcStats)
		if _, exists := uniqueStatsTypes[key]; exists {
			continue
		}
		uniqueStatsTypes[fmt.Sprintf("%T", callbackArgs.rpcStats)] = struct{}{}
	}
	if len(uniqueStatsTypes) != len(expectedCallouts) {
		t.Errorf("Unexpected number of stats handler callouts. Got %v, want %v", len(uniqueStatsTypes), len(expectedCallouts))
	}

	for _, callbackArgs := range pshArgs {
		expectedPeer, found := expectedCallouts[callbackArgs.rpcStats]
		// In case expectation is set to false and still we got the peer,
		// then it's good to have it. So no need to assert those conditions.
		if found && expectedPeer && callbackArgs.peer != nil {
			continue
		} else if expectedPeer && callbackArgs.peer == nil {
			t.Errorf("peer not populated for: %T", callbackArgs.rpcStats)
		}
	}
}

type peerStats struct {
	rpcStats stats.RPCStats
	peer     *peer.Peer
}

type peerStatsHandler struct {
	args []peerStats
	mu   sync.Mutex
}

func (h *peerStatsHandler) TagRPC(ctx context.Context, _ *stats.RPCTagInfo) context.Context {
	return ctx
}

func (h *peerStatsHandler) HandleRPC(ctx context.Context, rs stats.RPCStats) {
	p, _ := peer.FromContext(ctx)
	h.mu.Lock()
	defer h.mu.Unlock()
	h.args = append(h.args, peerStats{rs, p})
}

func (h *peerStatsHandler) TagConn(ctx context.Context, _ *stats.ConnTagInfo) context.Context {
	return ctx
}

func (h *peerStatsHandler) HandleConn(context.Context, stats.ConnStats) {}
