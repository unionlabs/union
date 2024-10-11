/*
 *
 * Copyright 2021 gRPC authors.
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

package e2e

import (
	"bytes"
	"context"
	"flag"
	"fmt"
	"os"
	"strconv"
	"testing"
	"time"

	"google.golang.org/grpc/internal/testutils/xds/e2e"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

var (
	clientPath = flag.String("client", "./binaries/client", "The interop client")
	serverPath = flag.String("server", "./binaries/server", "The interop server")
)

type testOpts struct {
	testName     string
	backendCount int
	clientFlags  []string
}

func setup(t *testing.T, opts testOpts) (*controlPlane, *client, []*server) {
	t.Helper()
	if _, err := os.Stat(*clientPath); os.IsNotExist(err) {
		t.Skip("skipped because client is not found")
	}
	if _, err := os.Stat(*serverPath); os.IsNotExist(err) {
		t.Skip("skipped because server is not found")
	}
	backendCount := 1
	if opts.backendCount != 0 {
		backendCount = opts.backendCount
	}

	cp, err := newControlPlane(t)
	if err != nil {
		t.Fatalf("failed to start control-plane: %v", err)
	}

	var clientLog bytes.Buffer
	c, err := newClient(fmt.Sprintf("xds:///%s", opts.testName), *clientPath, cp.bootstrapContent, &clientLog, opts.clientFlags...)
	if err != nil {
		t.Fatalf("failed to start client: %v", err)
	}
	t.Cleanup(c.stop)

	var serverLog bytes.Buffer
	servers, err := newServers(opts.testName, *serverPath, cp.bootstrapContent, &serverLog, backendCount)
	if err != nil {
		t.Fatalf("failed to start server: %v", err)
	}
	t.Cleanup(func() {
		for _, s := range servers {
			s.stop()
		}
	})
	t.Cleanup(func() {
		// TODO: find a better way to print the log. They are long, and hide the failure.
		t.Logf("\n----- client logs -----\n%v", clientLog.String())
		t.Logf("\n----- server logs -----\n%v", serverLog.String())
	})
	return cp, c, servers
}

func TestPingPong(t *testing.T) {
	const testName = "pingpong"
	cp, c, _ := setup(t, testOpts{testName: testName})

	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: testName,
		NodeID:     cp.nodeID,
		Host:       "localhost",
		Port:       serverPort,
		SecLevel:   e2e.SecurityLevelNone,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	if err := cp.server.Update(ctx, resources); err != nil {
		t.Fatalf("failed to update control plane resources: %v", err)
	}

	st, err := c.clientStats(ctx)
	if err != nil {
		t.Fatalf("failed to get client stats: %v", err)
	}
	if st.NumFailures != 0 {
		t.Fatalf("Got %v failures: %+v", st.NumFailures, st)
	}
}

// TestAffinity covers the affinity tests with ringhash policy.
// - client is configured to use ringhash, with 3 backends
// - all RPCs will hash a specific metadata header
// - verify that
//   - all RPCs with the same metadata value are sent to the same backend
//   - only one backend is Ready
//
// - send more RPCs with different metadata values until a new backend is picked, and verify that
//   - only two backends are in Ready
func TestAffinity(t *testing.T) {
	const (
		testName     = "affinity"
		backendCount = 3
		testMDKey    = "xds_md"
		testMDValue  = "unary_yranu"
	)
	cp, c, servers := setup(t, testOpts{
		testName:     testName,
		backendCount: backendCount,
		clientFlags:  []string{"--rpc=EmptyCall", fmt.Sprintf("--metadata=EmptyCall:%s:%s", testMDKey, testMDValue)},
	})

	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: testName,
		NodeID:     cp.nodeID,
		Host:       "localhost",
		Port:       serverPort,
		SecLevel:   e2e.SecurityLevelNone,
	})

	// Update EDS to multiple backends.
	var ports []uint32
	for _, s := range servers {
		ports = append(ports, uint32(s.port))
	}
	edsMsg := resources.Endpoints[0]
	resources.Endpoints[0] = e2e.DefaultEndpoint(
		edsMsg.ClusterName,
		"localhost",
		ports,
	)

	// Update CDS lbpolicy to ringhash.
	cdsMsg := resources.Clusters[0]
	cdsMsg.LbPolicy = v3clusterpb.Cluster_RING_HASH

	// Update RDS to hash the header.
	rdsMsg := resources.Routes[0]
	rdsMsg.VirtualHosts[0].Routes[0].Action = &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
		ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: cdsMsg.Name},
		HashPolicy: []*v3routepb.RouteAction_HashPolicy{{
			PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
				Header: &v3routepb.RouteAction_HashPolicy_Header{
					HeaderName: testMDKey,
				},
			},
		}},
	}}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	if err := cp.server.Update(ctx, resources); err != nil {
		t.Fatalf("failed to update control plane resources: %v", err)
	}

	// Note: We can skip CSDS check because there's no long delay as in TD.
	//
	// The client stats check doesn't race with the xds resource update because
	// there's only one version of xds resource, updated at the beginning of the
	// test. So there's no need to retry the stats call.
	//
	// In the future, we may add tests that update xds in the middle. Then we
	// either need to retry clientStats(), or make a CSDS check before so the
	// result is stable.

	st, err := c.clientStats(ctx)
	if err != nil {
		t.Fatalf("failed to get client stats: %v", err)
	}
	if st.NumFailures != 0 {
		t.Fatalf("Got %v failures: %+v", st.NumFailures, st)
	}
	if len(st.RpcsByPeer) != 1 {
		t.Fatalf("more than 1 backends got traffic: %v, want 1", st.RpcsByPeer)
	}

	// Call channelz to verify that only one subchannel is in state Ready.
	scs, err := c.channelzSubChannels(ctx)
	if err != nil {
		t.Fatalf("failed to fetch channelz: %v", err)
	}
	verifySubConnStates(t, scs, map[channelzpb.ChannelConnectivityState_State]int{
		channelzpb.ChannelConnectivityState_READY: 1,
		channelzpb.ChannelConnectivityState_IDLE:  2,
	})

	// Send Unary call with different metadata value with integers starting from
	// 0. Stop when a second peer is picked.
	var (
		diffPeerPicked bool
		mdValue        int
	)
	for !diffPeerPicked {
		if err := c.configRPCs(ctx, &testpb.ClientConfigureRequest{
			Types: []testpb.ClientConfigureRequest_RpcType{
				testpb.ClientConfigureRequest_EMPTY_CALL,
				testpb.ClientConfigureRequest_UNARY_CALL,
			},
			Metadata: []*testpb.ClientConfigureRequest_Metadata{
				{Type: testpb.ClientConfigureRequest_EMPTY_CALL, Key: testMDKey, Value: testMDValue},
				{Type: testpb.ClientConfigureRequest_UNARY_CALL, Key: testMDKey, Value: strconv.Itoa(mdValue)},
			},
		}); err != nil {
			t.Fatalf("failed to configure RPC: %v", err)
		}

		st, err := c.clientStats(ctx)
		if err != nil {
			t.Fatalf("failed to get client stats: %v", err)
		}
		if st.NumFailures != 0 {
			t.Fatalf("Got %v failures: %+v", st.NumFailures, st)
		}
		if len(st.RpcsByPeer) == 2 {
			break
		}

		mdValue++
	}

	// Call channelz to verify that only one subchannel is in state Ready.
	scs2, err := c.channelzSubChannels(ctx)
	if err != nil {
		t.Fatalf("failed to fetch channelz: %v", err)
	}
	verifySubConnStates(t, scs2, map[channelzpb.ChannelConnectivityState_State]int{
		channelzpb.ChannelConnectivityState_READY: 2,
		channelzpb.ChannelConnectivityState_IDLE:  1,
	})
}
