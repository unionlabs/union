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

package ringhash_test

import (
	"context"
	"errors"
	"fmt"
	"math"
	"math/rand"
	"net"
	"slices"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/backoff"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3ringhashpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/ring_hash/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/protobuf/types/known/wrapperspb"

	_ "google.golang.org/grpc/xds"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const (
	defaultTestTimeout = 10 * time.Second

	errorTolerance = .05 // For tests that rely on statistical significance.

	virtualHostName = "test.server"
)

// fastConnectParams disables connection attempts backoffs and lowers delays.
// This speeds up tests that rely on subchannel to move to transient failure.
var fastConnectParams = grpc.ConnectParams{
	Backoff: backoff.Config{
		BaseDelay: 10 * time.Millisecond,
	},
	MinConnectTimeout: 100 * time.Millisecond,
}

// Tests the case where the ring contains a single subConn, and verifies that
// when the server goes down, the LB policy on the client automatically
// reconnects until the subChannel moves out of TRANSIENT_FAILURE.
func (s) TestRingHash_ReconnectToMoveOutOfTransientFailure(t *testing.T) {
	// Create a restartable listener to simulate server being down.
	l, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	lis := testutils.NewRestartableListener(l)
	srv := stubserver.StartTestService(t, &stubserver.StubServer{
		Listener:   lis,
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	})
	defer srv.Stop()

	// Create a clientConn with a manual resolver (which is used to push the
	// address of the test backend), and a default service config pointing to
	// the use of the ring_hash_experimental LB policy.
	const ringHashServiceConfig = `{"loadBalancingConfig": [{"ring_hash_experimental":{}}]}`
	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(r),
		grpc.WithDefaultServiceConfig(ringHashServiceConfig),
		grpc.WithConnectParams(fastConnectParams),
	}
	cc, err := grpc.NewClient(r.Scheme()+":///test.server", dopts...)
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Push the address of the test backend through the manual resolver.
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: lis.Addr().String()}}})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}

	// Stopping the server listener will close the transport on the client,
	// which will lead to the channel eventually moving to IDLE. The ring_hash
	// LB policy is not expected to reconnect by itself at this point.
	lis.Stop()

	testutils.AwaitState(ctx, t, cc, connectivity.Idle)

	// Make an RPC to get the ring_hash LB policy to reconnect and thereby move
	// to TRANSIENT_FAILURE upon connection failure.
	client.EmptyCall(ctx, &testpb.Empty{})

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// An RPC at this point is expected to fail.
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); err == nil {
		t.Fatal("EmptyCall RPC succeeded when the channel is in TRANSIENT_FAILURE")
	}

	// Restart the server listener. The ring_hash LB policy is expected to
	// attempt to reconnect on its own and come out of TRANSIENT_FAILURE, even
	// without an RPC attempt.
	lis.Restart()
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)

	// An RPC at this point is expected to succeed.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}
}

// startTestServiceBackends starts num stub servers. It returns their addresses.
// Servers are closed when the test is stopped.
func startTestServiceBackends(t *testing.T, num int) []string {
	t.Helper()

	addrs := make([]string, 0, num)
	for i := 0; i < num; i++ {
		server := stubserver.StartTestService(t, nil)
		t.Cleanup(server.Stop)
		addrs = append(addrs, server.Address)
	}
	return addrs
}

// backendOptions returns a slice of e2e.BackendOptions for the given server
// addresses.
func backendOptions(t *testing.T, serverAddrs []string) []e2e.BackendOptions {
	t.Helper()

	var backendOpts []e2e.BackendOptions
	for _, addr := range serverAddrs {
		backendOpts = append(backendOpts, e2e.BackendOptions{Port: testutils.ParsePort(t, addr)})
	}
	return backendOpts
}

// channelIDHashRoute returns a RouteConfiguration with a hash policy that
// hashes based on the channel ID.
func channelIDHashRoute(routeName, virtualHostDomain, clusterName string) *v3routepb.RouteConfiguration {
	route := e2e.DefaultRouteConfig(routeName, virtualHostDomain, clusterName)
	hashPolicy := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{
			FilterState: &v3routepb.RouteAction_HashPolicy_FilterState{
				Key: "io.grpc.channel_id",
			},
		},
	}
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy = []*v3routepb.RouteAction_HashPolicy{&hashPolicy}
	return route
}

// checkRPCSendOK sends num RPCs to the client. It returns a map of backend
// addresses as keys and number of RPCs sent to this address as value. Abort the
// test if any RPC fails.
func checkRPCSendOK(ctx context.Context, t *testing.T, client testgrpc.TestServiceClient, num int) map[string]int {
	t.Helper()

	backendCount := make(map[string]int)
	for i := 0; i < num; i++ {
		var remote peer.Peer
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&remote)); err != nil {
			t.Fatalf("rpc EmptyCall() failed: %v", err)
		}
		backendCount[remote.Addr.String()]++
	}
	return backendCount
}

// makeNonExistentBackends returns a slice of strings with num listeners, each
// of which is closed immediately. Useful to simulate servers that are
// unreachable.
func makeNonExistentBackends(t *testing.T, num int) []string {
	t.Helper()

	closedListeners := make([]net.Listener, 0, num)
	for i := 0; i < num; i++ {
		lis, err := testutils.LocalTCPListener()
		if err != nil {
			t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
		}
		closedListeners = append(closedListeners, lis)
	}

	// Stop the servers that we want to be unreachable and collect their
	// addresses. We don't close them in the loop above to make sure ports are
	// not reused across them.
	addrs := make([]string, 0, num)
	for _, lis := range closedListeners {
		addrs = append(addrs, lis.Addr().String())
		lis.Close()
	}
	return addrs
}

// setupManagementServerAndResolver sets up an xDS management server, creates
// bootstrap configuration pointing to that server and creates an xDS resolver
// using that configuration.
//
// Registers a cleanup function on t to stop the management server.
//
// Returns the management server, node ID and the xDS resolver builder.
func setupManagementServerAndResolver(t *testing.T) (*e2e.ManagementServer, string, resolver.Builder) {
	t.Helper()

	// Start an xDS management server.
	xdsServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, xdsServer.Address)

	// Create an xDS resolver with the above bootstrap configuration.
	var r resolver.Builder
	var err error
	if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
		r, err = newResolver.(func([]byte) (resolver.Builder, error))(bc)
		if err != nil {
			t.Fatalf("Failed to create xDS resolver for testing: %v", err)
		}
	}

	return xdsServer, nodeID, r
}

// xdsUpdateOpts returns an e2e.UpdateOptions for the given node ID with the given xDS resources.
func xdsUpdateOpts(nodeID string, endpoints *v3endpointpb.ClusterLoadAssignment, cluster *v3clusterpb.Cluster, route *v3routepb.RouteConfiguration, listener *v3listenerpb.Listener) e2e.UpdateOptions {
	return e2e.UpdateOptions{
		NodeID:    nodeID,
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{endpoints},
		Clusters:  []*v3clusterpb.Cluster{cluster},
		Routes:    []*v3routepb.RouteConfiguration{route},
		Listeners: []*v3listenerpb.Listener{listener},
	}
}

// Tests that when an aggregate cluster is configured with ring hash policy, and
// the first cluster is in transient failure, all RPCs are sent to the second
// cluster using the ring hash policy.
func (s) TestRingHash_AggregateClusterFallBackFromRingHashAtStartup(t *testing.T) {
	addrs := startTestServiceBackends(t, 2)

	const primaryClusterName = "new_cluster_1"
	const primaryServiceName = "new_eds_service_1"
	const secondaryClusterName = "new_cluster_2"
	const secondaryServiceName = "new_eds_service_2"
	const clusterName = "aggregate_cluster"

	ep1 := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: primaryServiceName,
		Localities: []e2e.LocalityOptions{{
			Name:     "locality0",
			Weight:   1,
			Backends: backendOptions(t, makeNonExistentBackends(t, 2)),
		}},
	})
	ep2 := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: secondaryServiceName,
		Localities: []e2e.LocalityOptions{{
			Name:     "locality0",
			Weight:   1,
			Backends: backendOptions(t, addrs),
		}},
	})
	primaryCluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: primaryClusterName,
		ServiceName: primaryServiceName,
	})
	secondaryCluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: secondaryClusterName,
		ServiceName: secondaryServiceName,
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		Type:        e2e.ClusterTypeAggregate,
		// TODO: when "A75: xDS Aggregate Cluster Behavior Fixes" is implemented, the
		// policy will have to be set on the child clusters.
		Policy:     e2e.LoadBalancingPolicyRingHash,
		ChildNames: []string{primaryClusterName, secondaryClusterName},
	})
	route := channelIDHashRoute("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	updateOpts := e2e.UpdateOptions{
		NodeID:    nodeID,
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{ep1, ep2},
		Clusters:  []*v3clusterpb.Cluster{cluster, primaryCluster, secondaryCluster},
		Routes:    []*v3routepb.RouteConfiguration{route},
		Listeners: []*v3listenerpb.Listener{listener},
	}
	if err := xdsServer.Update(ctx, updateOpts); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	const numRPCs = 100
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)

	// Since this is using ring hash with the channel ID as the key, all RPCs
	// are routed to the same backend of the secondary locality.
	if len(gotPerBackend) != 1 {
		t.Errorf("Got RPCs routed to %v backends, want %v", len(gotPerBackend), 1)
	}

	var backend string
	var got int
	for backend, got = range gotPerBackend {
	}
	if !slices.Contains(addrs, backend) {
		t.Errorf("Got RPCs routed to an unexpected backend: %v, want one of %v", backend, addrs)
	}
	if got != numRPCs {
		t.Errorf("Got %v RPCs routed to a backend, want %v", got, 100)
	}
}

func replaceDNSResolver(t *testing.T) *manual.Resolver {
	mr := manual.NewBuilderWithScheme("dns")

	dnsResolverBuilder := resolver.Get("dns")
	resolver.Register(mr)

	t.Cleanup(func() { resolver.Register(dnsResolverBuilder) })
	return mr
}

// Tests that when an aggregate cluster is configured with ring hash policy, and
// the first is an EDS cluster in transient failure, and the fallback is a
// logical DNS cluster, all RPCs are sent to the second cluster using the ring
// hash policy.
func (s) TestRingHash_AggregateClusterFallBackFromRingHashToLogicalDnsAtStartup(t *testing.T) {
	const edsClusterName = "eds_cluster"
	const logicalDNSClusterName = "logical_dns_cluster"
	const clusterName = "aggregate_cluster"

	backends := startTestServiceBackends(t, 1)

	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: edsClusterName,
		Localities: []e2e.LocalityOptions{{
			Name:     "locality0",
			Weight:   1,
			Backends: backendOptions(t, makeNonExistentBackends(t, 1)),
			Priority: 0,
		}},
	})
	edsCluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: edsClusterName,
		ServiceName: edsClusterName,
	})

	logicalDNSCluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		Type:        e2e.ClusterTypeLogicalDNS,
		ClusterName: logicalDNSClusterName,
		// The DNS values are not used because we fake DNS later on, but they
		// are required to be present for the resource to be valid.
		DNSHostName: "server.example.com",
		DNSPort:     443,
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		Type:        e2e.ClusterTypeAggregate,
		// TODO: when "A75: xDS Aggregate Cluster Behavior Fixes" is merged, the
		// policy will have to be set on the child clusters.
		Policy:     e2e.LoadBalancingPolicyRingHash,
		ChildNames: []string{edsClusterName, logicalDNSClusterName},
	})
	route := channelIDHashRoute("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	updateOpts := e2e.UpdateOptions{
		NodeID:    nodeID,
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{endpoints},
		Clusters:  []*v3clusterpb.Cluster{cluster, edsCluster, logicalDNSCluster},
		Routes:    []*v3routepb.RouteConfiguration{route},
		Listeners: []*v3listenerpb.Listener{listener},
	}
	if err := xdsServer.Update(ctx, updateOpts); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	dnsR := replaceDNSResolver(t)
	dnsR.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: backends[0]}}})

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	gotPerBackend := checkRPCSendOK(ctx, t, client, 1)
	var got string
	for got = range gotPerBackend {
	}
	if want := backends[0]; got != want {
		t.Errorf("Got RPCs routed to an unexpected got: %v, want %v", got, want)
	}
}

// Tests that when an aggregate cluster is configured with ring hash policy, and
// it's first child is in transient failure, and the fallback is a logical DNS,
// the later recovers from transient failure when its backend becomes available.
func (s) TestRingHash_AggregateClusterFallBackFromRingHashToLogicalDnsAtStartupNoFailedRPCs(t *testing.T) {
	const edsClusterName = "eds_cluster"
	const logicalDNSClusterName = "logical_dns_cluster"
	const clusterName = "aggregate_cluster"

	backends := startTestServiceBackends(t, 1)

	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: edsClusterName,
		Localities: []e2e.LocalityOptions{{
			Name:     "locality0",
			Weight:   1,
			Backends: backendOptions(t, makeNonExistentBackends(t, 1)),
			Priority: 0,
		}},
	})
	edsCluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: edsClusterName,
		ServiceName: edsClusterName,
	})

	logicalDNSCluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		Type:        e2e.ClusterTypeLogicalDNS,
		ClusterName: logicalDNSClusterName,
		// The DNS values are not used because we fake DNS later on, but they
		// are required to be present for the resource to be valid.
		DNSHostName: "server.example.com",
		DNSPort:     443,
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		Type:        e2e.ClusterTypeAggregate,
		// TODO: when "A75: xDS Aggregate Cluster Behavior Fixes" is merged, the
		// policy will have to be set on the child clusters.
		Policy:     e2e.LoadBalancingPolicyRingHash,
		ChildNames: []string{edsClusterName, logicalDNSClusterName},
	})
	route := channelIDHashRoute("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	updateOpts := e2e.UpdateOptions{
		NodeID:    nodeID,
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{endpoints},
		Clusters:  []*v3clusterpb.Cluster{cluster, edsCluster, logicalDNSCluster},
		Routes:    []*v3routepb.RouteConfiguration{route},
		Listeners: []*v3listenerpb.Listener{listener},
	}
	if err := xdsServer.Update(ctx, updateOpts); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	dnsR := replaceDNSResolver(t)
	dnsR.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: backends[0]}}})

	dialer := testutils.NewBlockingDialer()
	cp := grpc.ConnectParams{
		// Increase backoff time, so that subconns stay in TRANSIENT_FAILURE
		// for long enough to trigger potential problems.
		Backoff: backoff.Config{
			BaseDelay: defaultTestTimeout,
		},
		MinConnectTimeout: 0,
	}
	dopts := []grpc.DialOption{
		grpc.WithResolvers(xdsResolver),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithContextDialer(dialer.DialContext),
		grpc.WithConnectParams(cp)}
	conn, err := grpc.NewClient("xds:///test.server", dopts...)
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	hold := dialer.Hold(backends[0])

	errCh := make(chan error, 2)
	go func() {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			errCh <- fmt.Errorf("first rpc UnaryCall() failed: %v", err)
			return
		}
		errCh <- nil
	}()

	testutils.AwaitState(ctx, t, conn, connectivity.Connecting)

	go func() {
		// Start a second RPC at this point, which should be queued as well.
		// This will fail if the priority policy fails to update the picker to
		// point to the LOGICAL_DNS child; if it leaves it pointing to the EDS
		// priority 1, then the RPC will fail, because all subchannels are in
		// transient failure.
		//
		// Note that sending only the first RPC does not catch this case,
		// because if the priority policy fails to update the picker, then the
		// pick for the first RPC will not be retried.
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			errCh <- fmt.Errorf("second UnaryCall() failed: %v", err)
			return
		}
		errCh <- nil
	}()

	// Wait for a connection attempt to backends[0].
	if !hold.Wait(ctx) {
		t.Fatalf("Timeout while waiting for a connection attempt to %s", backends[0])
	}
	// Allow the connection attempts to complete.
	hold.Resume()

	// RPCs should complete successfully.
	for range []int{0, 1} {
		select {
		case err := <-errCh:
			if err != nil {
				t.Errorf("Expected 2 rpc to succeed, but at least one failed: %v", err)
			}
		case <-ctx.Done():
			t.Fatalf("Timed out waiting for RPCs to complete")
		}
	}
}

// endpointResource creates a ClusterLoadAssignment containing a single locality
// with the given addresses.
func endpointResource(t *testing.T, clusterName string, addrs []string) *v3endpointpb.ClusterLoadAssignment {
	t.Helper()

	// We must set the host name socket address in EDS, as the ring hash policy
	// uses it to construct the ring.
	host, _, err := net.SplitHostPort(addrs[0])
	if err != nil {
		t.Fatalf("Failed to split host and port from stubserver: %v", err)
	}

	return e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Host:        host,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOptions(t, addrs),
			Weight:   1,
		}},
	})
}

// Tests that ring hash policy that hashes using channel id ensures all RPCs to
// go 1 particular backend.
func (s) TestRingHash_ChannelIdHashing(t *testing.T) {
	backends := startTestServiceBackends(t, 4)

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := channelIDHashRoute("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	const numRPCs = 100
	received := checkRPCSendOK(ctx, t, client, numRPCs)
	if len(received) != 1 {
		t.Errorf("Got RPCs routed to %v backends, want %v", len(received), 1)
	}
	var got int
	for _, got = range received {
	}
	if got != numRPCs {
		t.Errorf("Got %v RPCs routed to a backend, want %v", got, numRPCs)
	}
}

// headerHashRoute creates a RouteConfiguration with a hash policy that uses the
// provided header.
func headerHashRoute(routeName, virtualHostName, clusterName, header string) *v3routepb.RouteConfiguration {
	route := e2e.DefaultRouteConfig(routeName, virtualHostName, clusterName)
	hashPolicy := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
			Header: &v3routepb.RouteAction_HashPolicy_Header{
				HeaderName: header,
			},
		},
	}
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy = []*v3routepb.RouteAction_HashPolicy{&hashPolicy}
	return route
}

// Tests that ring hash policy that hashes using a header value can send RPCs
// to specific backends based on their hash.
func (s) TestRingHash_HeaderHashing(t *testing.T) {
	backends := startTestServiceBackends(t, 4)

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Note each type of RPC contains a header value that will always be hashed
	// to a specific backend as the header value matches the value used to
	// create the entry in the ring.
	for _, backend := range backends {
		ctx := metadata.NewOutgoingContext(ctx, metadata.Pairs("address_hash", backend+"_0"))
		numRPCs := 10
		reqPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
		if reqPerBackend[backend] != numRPCs {
			t.Errorf("Got RPC routed to addresses %v, want all RPCs routed to %v", reqPerBackend, backend)
		}
	}
}

// Tests that ring hash policy that hashes using a header value and regex
// rewrite to aggregate RPCs to 1 backend.
func (s) TestRingHash_HeaderHashingWithRegexRewrite(t *testing.T) {
	backends := startTestServiceBackends(t, 4)

	clusterName := "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy[0].GetHeader().RegexRewrite = &v3matcherpb.RegexMatchAndSubstitute{
		Pattern: &v3matcherpb.RegexMatcher{
			EngineType: &v3matcherpb.RegexMatcher_GoogleRe2{},
			Regex:      "[0-9]+",
		},
		Substitution: "foo",
	}
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Note each type of RPC contains a header value that would always be hashed
	// to a specific backend as the header value matches the value used to
	// create the entry in the ring. However, the regex rewrites all numbers to
	// "foo", and header values only differ by numbers, so they all end up
	// hashing to the same value.
	gotPerBackend := make(map[string]int)
	for _, backend := range backends {
		ctx := metadata.NewOutgoingContext(ctx, metadata.Pairs("address_hash", backend+"_0"))
		res := checkRPCSendOK(ctx, t, client, 100)
		for addr, count := range res {
			gotPerBackend[addr] += count
		}
	}
	if want := 1; len(gotPerBackend) != want {
		t.Errorf("Got RPCs routed to %v backends, want %v", len(gotPerBackend), want)
	}
	var got int
	for _, got = range gotPerBackend {
	}
	if want := 400; got != want {
		t.Errorf("Got %v RPCs routed to a backend, want %v", got, want)
	}
}

// computeIdealNumberOfRPCs computes the ideal number of RPCs to send so that
// we can observe an event happening with probability p, and the result will
// have value p with the given error tolerance.
//
// See https://github.com/grpc/grpc/blob/4f6e13bdda9e8c26d6027af97db4b368ca2b3069/test/cpp/end2end/xds/xds_end2end_test_lib.h#L941
// for an explanation of the formula.
func computeIdealNumberOfRPCs(t *testing.T, p, errorTolerance float64) int {
	if p < 0 || p > 1 {
		t.Fatal("p must be in (0, 1)")
	}
	numRPCs := math.Ceil(p * (1 - p) * 5. * 5. / errorTolerance / errorTolerance)
	return int(numRPCs + 1000.) // add 1k as a buffer to avoid flakiness.
}

// setRingHashLBPolicyWithHighMinRingSize sets the ring hash policy with a high
// minimum ring size to ensure that the ring is large enough to distribute
// requests more uniformly across endpoints when a random hash is used.
func setRingHashLBPolicyWithHighMinRingSize(t *testing.T, cluster *v3clusterpb.Cluster) {
	const minRingSize = 100000
	oldVal := envconfig.RingHashCap
	envconfig.RingHashCap = minRingSize
	t.Cleanup(func() {
		envconfig.RingHashCap = oldVal
	})
	// Increasing min ring size for random distribution.
	config := testutils.MarshalAny(t, &v3ringhashpb.RingHash{
		HashFunction:    v3ringhashpb.RingHash_XX_HASH,
		MinimumRingSize: &wrapperspb.UInt64Value{Value: minRingSize},
	})
	cluster.LoadBalancingPolicy = &v3clusterpb.LoadBalancingPolicy{
		Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{{
			TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
				Name:        "envoy.load_balancing_policies.ring_hash",
				TypedConfig: config,
			},
		}},
	}
}

// Tests that ring hash policy that hashes using a random value.
func (s) TestRingHash_NoHashPolicy(t *testing.T) {
	backends := startTestServiceBackends(t, 2)
	numRPCs := computeIdealNumberOfRPCs(t, .5, errorTolerance)

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
	})
	setRingHashLBPolicyWithHighMinRingSize(t, cluster)
	route := e2e.DefaultRouteConfig("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Send a large number of RPCs and check that they are distributed randomly.
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
	for _, backend := range backends {
		got := float64(gotPerBackend[backend]) / float64(numRPCs)
		want := .5
		if !cmp.Equal(got, want, cmpopts.EquateApprox(0, errorTolerance)) {
			t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backend, got, want, errorTolerance)
		}
	}
}

// Tests that we observe endpoint weights.
func (s) TestRingHash_EndpointWeights(t *testing.T) {
	backends := startTestServiceBackends(t, 3)

	const clusterName = "cluster"
	backendOpts := []e2e.BackendOptions{
		{Port: testutils.ParsePort(t, backends[0])},
		{Port: testutils.ParsePort(t, backends[1])},
		{Port: testutils.ParsePort(t, backends[2]), Weight: 2},
	}

	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOpts,
			Weight:   1,
		}},
	})
	endpoints.Endpoints[0].LbEndpoints[0].LoadBalancingWeight = wrapperspb.UInt32(uint32(1))
	endpoints.Endpoints[0].LbEndpoints[1].LoadBalancingWeight = wrapperspb.UInt32(uint32(1))
	endpoints.Endpoints[0].LbEndpoints[2].LoadBalancingWeight = wrapperspb.UInt32(uint32(2))
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
	})
	// Increasing min ring size for random distribution.
	setRingHashLBPolicyWithHighMinRingSize(t, cluster)
	route := e2e.DefaultRouteConfig("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Send a large number of RPCs and check that they are distributed randomly.
	numRPCs := computeIdealNumberOfRPCs(t, .25, errorTolerance)
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)

	got := float64(gotPerBackend[backends[0]]) / float64(numRPCs)
	want := .25
	if !cmp.Equal(got, want, cmpopts.EquateApprox(0, errorTolerance)) {
		t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backends[0], got, want, errorTolerance)
	}
	got = float64(gotPerBackend[backends[1]]) / float64(numRPCs)
	if !cmp.Equal(got, want, cmpopts.EquateApprox(0, errorTolerance)) {
		t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backends[1], got, want, errorTolerance)
	}
	got = float64(gotPerBackend[backends[2]]) / float64(numRPCs)
	want = .50
	if !cmp.Equal(got, want, cmpopts.EquateApprox(0, errorTolerance)) {
		t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backends[2], got, want, errorTolerance)
	}
}

// Tests that ring hash policy evaluation will continue past the terminal hash
// policy if no results are produced yet.
func (s) TestRingHash_ContinuesPastTerminalPolicyThatDoesNotProduceResult(t *testing.T) {
	backends := startTestServiceBackends(t, 2)

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})

	route := e2e.DefaultRouteConfig("new_route", "test.server", clusterName)

	// Even though this hash policy is terminal, since it produces no result, we
	// continue past it to find a policy that produces results.
	hashPolicy := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
			Header: &v3routepb.RouteAction_HashPolicy_Header{
				HeaderName: "header_not_present",
			},
		},
		Terminal: true,
	}
	hashPolicy2 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
			Header: &v3routepb.RouteAction_HashPolicy_Header{
				HeaderName: "address_hash",
			},
		},
	}
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy = []*v3routepb.RouteAction_HashPolicy{&hashPolicy, &hashPolicy2}

	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// - The first hash policy does not match because the header is not present.
	//   If this hash policy was applied, it would spread the load across
	//   backend 0 and 1, since a random hash would be used.
	// - In the second hash policy, each type of RPC contains a header
	//   value that always hashes to backend 0, as the header value
	//   matches the value used to create the entry in the ring.
	// We verify that the second hash policy is used by checking that all RPCs
	// are being routed to backend 0.
	wantBackend := backends[0]
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("address_hash", wantBackend+"_0"))
	const numRPCs = 100
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
	if got := gotPerBackend[wantBackend]; got != numRPCs {
		t.Errorf("Got %v RPCs routed to backend %v, want %v", got, wantBackend, numRPCs)
	}
}

// Tests that a random hash is used when header hashing policy specified a
// header field that the RPC did not have.
func (s) TestRingHash_HashOnHeaderThatIsNotPresent(t *testing.T) {
	backends := startTestServiceBackends(t, 2)
	wantFractionPerBackend := .5
	numRPCs := computeIdealNumberOfRPCs(t, wantFractionPerBackend, errorTolerance)

	const clusterName = "cluster"
	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOptions(t, backends),
			Weight:   1,
		}},
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
	})
	setRingHashLBPolicyWithHighMinRingSize(t, cluster)
	route := headerHashRoute("new_route", virtualHostName, clusterName, "header_not_present")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// The first hash policy does not apply because the header is not present in
	// the RPCs that we are about to send. As a result, a random hash should be
	// used instead, resulting in a random request distribution.
	// We verify this by checking that the RPCs are distributed randomly.
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
	for _, backend := range backends {
		got := float64(gotPerBackend[backend]) / float64(numRPCs)
		if !cmp.Equal(got, wantFractionPerBackend, cmpopts.EquateApprox(0, errorTolerance)) {
			t.Errorf("fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backend, got, wantFractionPerBackend, errorTolerance)
		}
	}
}

// Tests that a random hash is used when only unsupported hash policies are
// configured.
func (s) TestRingHash_UnsupportedHashPolicyDefaultToRandomHashing(t *testing.T) {
	backends := startTestServiceBackends(t, 2)
	wantFractionPerBackend := .5
	numRPCs := computeIdealNumberOfRPCs(t, wantFractionPerBackend, errorTolerance)

	const clusterName = "cluster"
	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOptions(t, backends),
			Weight:   1,
		}},
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
	})
	setRingHashLBPolicyWithHighMinRingSize(t, cluster)
	route := e2e.DefaultRouteConfig("new_route", "test.server", clusterName)
	unsupportedHashPolicy1 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Cookie_{
			Cookie: &v3routepb.RouteAction_HashPolicy_Cookie{Name: "cookie"},
		},
	}
	unsupportedHashPolicy2 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_ConnectionProperties_{
			ConnectionProperties: &v3routepb.RouteAction_HashPolicy_ConnectionProperties{SourceIp: true},
		},
	}
	unsupportedHashPolicy3 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_QueryParameter_{
			QueryParameter: &v3routepb.RouteAction_HashPolicy_QueryParameter{Name: "query_parameter"},
		},
	}
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy = []*v3routepb.RouteAction_HashPolicy{&unsupportedHashPolicy1, &unsupportedHashPolicy2, &unsupportedHashPolicy3}
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Since none of the hash policy are supported, a random hash should be
	// generated for every request.
	// We verify this by checking that the RPCs are distributed randomly.
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
	for _, backend := range backends {
		got := float64(gotPerBackend[backend]) / float64(numRPCs)
		if !cmp.Equal(got, wantFractionPerBackend, cmpopts.EquateApprox(0, errorTolerance)) {
			t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backend, got, wantFractionPerBackend, errorTolerance)
		}
	}
}

// Tests that unsupported hash policy types are all ignored before a supported
// hash policy.
func (s) TestRingHash_UnsupportedHashPolicyUntilChannelIdHashing(t *testing.T) {
	backends := startTestServiceBackends(t, 2)

	const clusterName = "cluster"
	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOptions(t, backends),
			Weight:   1,
		}},
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
	})
	setRingHashLBPolicyWithHighMinRingSize(t, cluster)
	route := e2e.DefaultRouteConfig("new_route", "test.server", clusterName)
	unsupportedHashPolicy1 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Cookie_{
			Cookie: &v3routepb.RouteAction_HashPolicy_Cookie{Name: "cookie"},
		},
	}
	unsupportedHashPolicy2 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_ConnectionProperties_{
			ConnectionProperties: &v3routepb.RouteAction_HashPolicy_ConnectionProperties{SourceIp: true},
		},
	}
	unsupportedHashPolicy3 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_QueryParameter_{
			QueryParameter: &v3routepb.RouteAction_HashPolicy_QueryParameter{Name: "query_parameter"},
		},
	}
	channelIDhashPolicy := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{
			FilterState: &v3routepb.RouteAction_HashPolicy_FilterState{
				Key: "io.grpc.channel_id",
			},
		},
	}
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy = []*v3routepb.RouteAction_HashPolicy{&unsupportedHashPolicy1, &unsupportedHashPolicy2, &unsupportedHashPolicy3, &channelIDhashPolicy}
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Since only unsupported policies are present except for the last one
	// which is using the channel ID hashing policy, all requests should be
	// routed to the same backend.
	const numRPCs = 100
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
	if len(gotPerBackend) != 1 {
		t.Errorf("Got RPCs routed to %v backends, want 1", len(gotPerBackend))
	}
	var got int
	for _, got = range gotPerBackend {
	}
	if got != numRPCs {
		t.Errorf("Got %v RPCs routed to a backend, want %v", got, numRPCs)
	}
}

// Tests that ring hash policy that hashes using a random value can spread RPCs
// across all the backends according to locality weight.
func (s) TestRingHash_RandomHashingDistributionAccordingToLocalityAndEndpointWeight(t *testing.T) {
	backends := startTestServiceBackends(t, 2)

	const clusterName = "cluster"
	const locality1Weight = uint32(1)
	const endpoint1Weight = uint32(1)
	const locality2Weight = uint32(2)
	const endpoint2Weight = uint32(2)
	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities: []e2e.LocalityOptions{
			{
				Backends: []e2e.BackendOptions{{
					Port:   testutils.ParsePort(t, backends[0]),
					Weight: endpoint1Weight,
				}},
				Weight: locality1Weight,
			},
			{
				Backends: []e2e.BackendOptions{{
					Port:   testutils.ParsePort(t, backends[1]),
					Weight: endpoint2Weight,
				}},
				Weight: locality2Weight,
			},
		},
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
	})
	setRingHashLBPolicyWithHighMinRingSize(t, cluster)
	route := e2e.DefaultRouteConfig("new_route", "test.server", clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	const weight1 = endpoint1Weight * locality1Weight
	const weight2 = endpoint2Weight * locality2Weight
	const wantRPCs1 = float64(weight1) / float64(weight1+weight2)
	const wantRPCs2 = float64(weight2) / float64(weight1+weight2)
	numRPCs := computeIdealNumberOfRPCs(t, math.Min(wantRPCs1, wantRPCs2), errorTolerance)

	// Send a large number of RPCs and check that they are distributed randomly.
	gotPerBackend := checkRPCSendOK(ctx, t, client, numRPCs)
	got := float64(gotPerBackend[backends[0]]) / float64(numRPCs)
	if !cmp.Equal(got, wantRPCs1, cmpopts.EquateApprox(0, errorTolerance)) {
		t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backends[2], got, wantRPCs1, errorTolerance)
	}
	got = float64(gotPerBackend[backends[1]]) / float64(numRPCs)
	if !cmp.Equal(got, wantRPCs2, cmpopts.EquateApprox(0, errorTolerance)) {
		t.Errorf("Fraction of RPCs to backend %s: got %v, want %v (margin: +-%v)", backends[2], got, wantRPCs2, errorTolerance)
	}
}

// Tests that ring hash policy that hashes using a fixed string ensures all RPCs
// to go 1 particular backend; and that subsequent hashing policies are ignored
// due to the setting of terminal.
func (s) TestRingHash_FixedHashingTerminalPolicy(t *testing.T) {
	backends := startTestServiceBackends(t, 2)
	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})

	route := e2e.DefaultRouteConfig("new_route", "test.server", clusterName)

	hashPolicy := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
			Header: &v3routepb.RouteAction_HashPolicy_Header{
				HeaderName: "fixed_string",
			},
		},
		Terminal: true,
	}
	hashPolicy2 := v3routepb.RouteAction_HashPolicy{
		PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
			Header: &v3routepb.RouteAction_HashPolicy_Header{
				HeaderName: "random_string",
			},
		},
	}
	action := route.VirtualHosts[0].Routes[0].Action.(*v3routepb.Route_Route)
	action.Route.HashPolicy = []*v3routepb.RouteAction_HashPolicy{&hashPolicy, &hashPolicy2}

	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Check that despite the matching random string header, since the fixed
	// string hash policy is terminal, only the fixed string hash policy applies
	// and requests all get routed to the same host.
	gotPerBackend := make(map[string]int)
	const numRPCs = 100
	for i := 0; i < numRPCs; i++ {
		ctx := metadata.NewOutgoingContext(ctx, metadata.Pairs(
			"fixed_string", backends[0]+"_0",
			"random_string", fmt.Sprintf("%d", rand.Int())),
		)
		var remote peer.Peer
		_, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&remote))
		if err != nil {
			t.Fatalf("rpc EmptyCall() failed: %v", err)
		}
		gotPerBackend[remote.Addr.String()]++
	}

	if len(gotPerBackend) != 1 {
		t.Error("Got RPCs routed to multiple backends, want a single backend")
	}
	if got := gotPerBackend[backends[0]]; got != numRPCs {
		t.Errorf("Got %v RPCs routed to %v, want %v", got, backends[0], numRPCs)
	}
}

// TestRingHash_IdleToReady tests that the channel will go from idle to ready
// via connecting; (though it is not possible to catch the connecting state
// before moving to ready via the public API).
// TODO: we should be able to catch all state transitions by using the internal.SubscribeToConnectivityStateChanges API.
func (s) TestRingHash_IdleToReady(t *testing.T) {
	backends := startTestServiceBackends(t, 1)

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := channelIDHashRoute("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	if got, want := conn.GetState(), connectivity.Idle; got != want {
		t.Errorf("conn.GetState(): got %v, want %v", got, want)
	}

	checkRPCSendOK(ctx, t, client, 1)

	if got, want := conn.GetState(), connectivity.Ready; got != want {
		t.Errorf("conn.GetState(): got %v, want %v", got, want)
	}
}

// Test that the channel will transition to READY once it starts
// connecting even if there are no RPCs being sent to the picker.
func (s) TestRingHash_ContinuesConnectingWithoutPicks(t *testing.T) {
	backend := stubserver.StartTestService(t, &stubserver.StubServer{
		// We expect the server EmptyCall to not be call here because the
		// aggregated channel state is never READY when the call is pending.
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			t.Errorf("EmptyCall() should not have been called")
			return &testpb.Empty{}, nil
		},
	})
	defer backend.Stop()

	nonExistentServerAddr := makeNonExistentBackends(t, 1)[0]

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, []string{backend.Address, nonExistentServerAddr})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	dialer := testutils.NewBlockingDialer()
	dopts := []grpc.DialOption{
		grpc.WithResolvers(xdsResolver),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithContextDialer(dialer.DialContext),
	}
	conn, err := grpc.NewClient("xds:///test.server", dopts...)
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	hold := dialer.Hold(backend.Address)

	rpcCtx, rpcCancel := context.WithCancel(ctx)
	go func() {
		rpcCtx = metadata.NewOutgoingContext(rpcCtx, metadata.Pairs("address_hash", nonExistentServerAddr+"_0"))
		_, err := client.EmptyCall(rpcCtx, &testpb.Empty{})
		if status.Code(err) != codes.Canceled {
			t.Errorf("Expected RPC to be canceled, got error: %v", err)
		}
	}()

	// Wait for the connection attempt to the real backend.
	if !hold.Wait(ctx) {
		t.Fatalf("Timeout waiting for connection attempt to backend %v.", backend.Address)
	}
	// Now cancel the RPC while we are still connecting.
	rpcCancel()

	// This allows the connection attempts to continue. The RPC was cancelled
	// before the backend was connected, but the backend is up. The conn
	// becomes Ready due to the connection attempt to the existing backend
	// succeeding, despite no new RPC being sent.
	hold.Resume()

	testutils.AwaitState(ctx, t, conn, connectivity.Ready)
}

// Tests that when the first pick is down leading to a transient failure, we
// will move on to the next ring hash entry.
func (s) TestRingHash_TransientFailureCheckNextOne(t *testing.T) {
	backends := startTestServiceBackends(t, 1)
	nonExistentBackends := makeNonExistentBackends(t, 1)

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, append(nonExistentBackends, backends...))
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Note each type of RPC contains a header value that will always be hashed
	// the value that was used to place the non-existent endpoint on the ring,
	// but it still gets routed to the backend that is up.
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("address_hash", nonExistentBackends[0]+"_0"))
	reqPerBackend := checkRPCSendOK(ctx, t, client, 1)
	var got string
	for got = range reqPerBackend {
	}
	if want := backends[0]; got != want {
		t.Errorf("Got RPC routed to addr %v, want %v", got, want)
	}
}

// Tests for a bug seen in the wild in c-core, where ring_hash started with no
// endpoints and reported TRANSIENT_FAILURE, then got an update with endpoints
// and reported IDLE, but the picker update was squelched, so it failed to ever
// get reconnected.
func (s) TestRingHash_ReattemptWhenGoingFromTransientFailureToIdle(t *testing.T) {
	const clusterName = "cluster"
	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities:  []e2e.LocalityOptions{{}}, // note the empty locality (no endpoint).
	})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := e2e.DefaultRouteConfig("new_route", virtualHostName, clusterName)
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	conn, err := grpc.NewClient("xds:///test.server", grpc.WithResolvers(xdsResolver), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	if got, want := conn.GetState(), connectivity.Idle; got != want {
		t.Errorf("conn.GetState(): got %v, want %v", got, want)
	}

	// There are no endpoints in EDS. RPCs should fail and the channel should
	// transition to transient failure.
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); err == nil {
		t.Errorf("rpc EmptyCall() succeeded, want error")
	}
	if got, want := conn.GetState(), connectivity.TransientFailure; got != want {
		t.Errorf("conn.GetState(): got %v, want %v", got, want)
	}

	backends := startTestServiceBackends(t, 1)

	t.Log("Updating EDS with a new backend endpoint.")
	endpoints = e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOptions(t, backends),
			Weight:   1,
		}},
	})
	if err = xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	// A WaitForReady RPC should succeed, and the channel should report READY.
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Errorf("rpc EmptyCall() failed: %v", err)
	}
	if got, want := conn.GetState(), connectivity.Ready; got != want {
		t.Errorf("conn.GetState(): got %v, want %v", got, want)
	}
}

// Tests that when all backends are down and then up, we may pick a TF backend
// and we will then jump to ready backend.
func (s) TestRingHash_TransientFailureSkipToAvailableReady(t *testing.T) {
	emptyCallF := func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
		return &testpb.Empty{}, nil
	}
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}
	restartableListener1 := testutils.NewRestartableListener(lis)
	restartableServer1 := stubserver.StartTestService(t, &stubserver.StubServer{
		Listener:   restartableListener1,
		EmptyCallF: emptyCallF,
	})
	defer restartableServer1.Stop()

	lis, err = testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}
	restartableListener2 := testutils.NewRestartableListener(lis)
	restartableServer2 := stubserver.StartTestService(t, &stubserver.StubServer{
		Listener:   restartableListener2,
		EmptyCallF: emptyCallF,
	})
	defer restartableServer2.Stop()

	nonExistantBackends := makeNonExistentBackends(t, 2)

	const clusterName = "cluster"
	backends := []string{restartableServer1.Address, restartableServer2.Address}
	backends = append(backends, nonExistantBackends...)
	endpoints := endpointResource(t, clusterName, backends)
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}
	opts := []grpc.DialOption{
		grpc.WithConnectParams(grpc.ConnectParams{
			// Disable backoff to speed up the test.
			MinConnectTimeout: 100 * time.Millisecond,
		}),
		grpc.WithResolvers(xdsResolver),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	}
	conn, err := grpc.NewClient("xds:///test.server", opts...)
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	testutils.AwaitState(ctx, t, conn, connectivity.Idle)

	// Test starts with backends not listening.
	restartableListener1.Stop()
	restartableListener2.Stop()

	// Send a request with a hash that should go to restartableServer1.
	// Because it is not accepting connections, and no other backend is
	// listening, the RPC fails.
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("address_hash", restartableServer1.Address+"_0"))
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); err == nil {
		t.Fatalf("rpc EmptyCall() succeeded, want error")
	}

	testutils.AwaitState(ctx, t, conn, connectivity.TransientFailure)

	// Bring up first backend. The channel should become Ready without any
	// picks, because in TF, we are always trying to connect to at least one
	// backend at all times.
	restartableListener1.Restart()
	testutils.AwaitState(ctx, t, conn, connectivity.Ready)

	// Bring down backend 1 and bring up backend 2.
	// Note the RPC contains a header value that will always be hashed to
	// backend 1. So by purposely bringing down backend 1 and bringing up
	// another backend, this will ensure Picker's first choice of backend 1
	// fails and it will go through the remaining subchannels to find one in
	// READY. Since the entries in the ring are pretty distributed and we have
	// unused ports to fill the ring, it is almost guaranteed that the Picker
	// will go through some non-READY entries and skip them as per design.
	t.Logf("bringing down backend 1")
	restartableListener1.Stop()

	testutils.AwaitState(ctx, t, conn, connectivity.TransientFailure)
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); err == nil {
		t.Fatalf("rpc EmptyCall() succeeded, want error")
	}

	t.Logf("bringing up backend 2")
	restartableListener2.Restart()
	testutils.AwaitState(ctx, t, conn, connectivity.Ready)

	wantPeerAddr := ""
	for wantPeerAddr != restartableServer2.Address {
		p := peer.Peer{}
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&p)); errors.Is(err, context.DeadlineExceeded) {
			t.Fatalf("Timed out waiting for rpc EmptyCall() to be routed to the expected backend")
		}
		wantPeerAddr = p.Addr.String()
	}
}

// Tests that when all backends are down, we keep reattempting.
func (s) TestRingHash_ReattemptWhenAllEndpointsUnreachable(t *testing.T) {
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}
	restartableListener := testutils.NewRestartableListener(lis)
	restartableServer := stubserver.StartTestService(t, &stubserver.StubServer{
		Listener: restartableListener,
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	})
	defer restartableServer.Stop()

	const clusterName = "cluster"
	endpoints := endpointResource(t, clusterName, []string{restartableServer.Address})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	dopts := []grpc.DialOption{
		grpc.WithResolvers(xdsResolver),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithConnectParams(fastConnectParams),
	}
	conn, err := grpc.NewClient("xds:///test.server", dopts...)
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	testutils.AwaitState(ctx, t, conn, connectivity.Idle)

	t.Log("Stopping the backend server")
	restartableListener.Stop()

	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("rpc EmptyCall() succeeded, want Unavailable error")
	}

	// Wait for channel to fail.
	testutils.AwaitState(ctx, t, conn, connectivity.TransientFailure)

	t.Log("Restarting the backend server")
	restartableListener.Restart()

	// Wait for channel to become connected without any pending RPC.
	testutils.AwaitState(ctx, t, conn, connectivity.Ready)
}

// Tests that when a backend goes down, we will move on to the next subchannel
// (with a lower priority).  When the backend comes back up, traffic will move
// back.
func (s) TestRingHash_SwitchToLowerPriorityAndThenBack(t *testing.T) {
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}
	restartableListener := testutils.NewRestartableListener(lis)
	restartableServer := stubserver.StartTestService(t, &stubserver.StubServer{
		Listener: restartableListener,
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	})
	defer restartableServer.Stop()

	otherBackend := startTestServiceBackends(t, 1)[0]

	// We must set the host name socket address in EDS, as the ring hash policy
	// uses it to construct the ring.
	host, _, err := net.SplitHostPort(otherBackend)
	if err != nil {
		t.Fatalf("Failed to split host and port from stubserver: %v", err)
	}

	const clusterName = "cluster"
	endpoints := e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
		ClusterName: clusterName,
		Host:        host,
		Localities: []e2e.LocalityOptions{{
			Backends: backendOptions(t, []string{restartableServer.Address}),
			Weight:   1,
		}, {
			Backends: backendOptions(t, []string{otherBackend}),
			Weight:   1,
			Priority: 1,
		}}})
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	dopts := []grpc.DialOption{
		grpc.WithResolvers(xdsResolver),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithConnectParams(fastConnectParams),
	}
	conn, err := grpc.NewClient("xds:///test.server", dopts...)
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	// Note each type of RPC contains a header value that will always be hashed
	// to the value that was used to place the non-existent endpoint on the ring.
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("address_hash", restartableServer.Address+"_0"))
	var got string
	for got = range checkRPCSendOK(ctx, t, client, 1) {
	}
	if want := restartableServer.Address; got != want {
		t.Fatalf("Got RPC routed to addr %v, want %v", got, want)
	}

	// Trigger failure with the existing backend, which should cause the
	// balancer to go in transient failure and the priority balancer to move
	// to the lower priority.
	restartableListener.Stop()

	for {
		p := peer.Peer{}
		_, err = client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(&p))

		// Ignore errors: we may need to attempt to send an RPC to detect the
		// failure (the next write on connection fails).
		if err == nil {
			if got, want := p.Addr.String(), otherBackend; got != want {
				t.Fatalf("Got RPC routed to addr %v, want %v", got, want)
			}
			break
		}
	}

	// Now we start the backend with the address hash that is used in the
	// metadata, so eventually RPCs should be routed to it, since it is in a
	// locality with higher priority.
	peerAddr := ""
	restartableListener.Restart()
	for peerAddr != restartableServer.Address {
		p := peer.Peer{}
		_, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&p))
		if errors.Is(err, context.DeadlineExceeded) {
			t.Fatalf("Timed out waiting for rpc EmptyCall() to be routed to the expected backend")
		}
		peerAddr = p.Addr.String()
	}
}

// Tests that when we trigger internal connection attempts without picks, we do
// so for only one subchannel at a time.
func (s) TestRingHash_ContinuesConnectingWithoutPicksOneSubchannelAtATime(t *testing.T) {
	backends := startTestServiceBackends(t, 1)
	nonExistantBackends := makeNonExistentBackends(t, 3)

	const clusterName = "cluster"

	endpoints := endpointResource(t, clusterName, append(nonExistantBackends, backends...))
	cluster := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: clusterName,
		Policy:      e2e.LoadBalancingPolicyRingHash,
	})
	route := headerHashRoute("new_route", virtualHostName, clusterName, "address_hash")
	listener := e2e.DefaultClientListener(virtualHostName, route.Name)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	xdsServer, nodeID, xdsResolver := setupManagementServerAndResolver(t)
	if err := xdsServer.Update(ctx, xdsUpdateOpts(nodeID, endpoints, cluster, route, listener)); err != nil {
		t.Fatalf("Failed to update xDS resources: %v", err)
	}

	dialer := testutils.NewBlockingDialer()
	dialOpts := []grpc.DialOption{
		grpc.WithResolvers(xdsResolver),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithContextDialer(dialer.DialContext),
		grpc.WithConnectParams(fastConnectParams),
	}
	conn, err := grpc.NewClient("xds:///test.server", dialOpts...)
	if err != nil {
		t.Fatalf("Failed to create client: %s", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)

	holdNonExistant0 := dialer.Hold(nonExistantBackends[0])
	holdNonExistant1 := dialer.Hold(nonExistantBackends[1])
	holdNonExistant2 := dialer.Hold(nonExistantBackends[2])
	holdGood := dialer.Hold(backends[0])

	rpcCtx, rpcCancel := context.WithCancel(ctx)
	errCh := make(chan error, 1)
	go func() {
		rpcCtx = metadata.NewOutgoingContext(rpcCtx, metadata.Pairs("address_hash", nonExistantBackends[0]+"_0"))
		_, err := client.EmptyCall(rpcCtx, &testpb.Empty{})
		if status.Code(err) == codes.Canceled {
			errCh <- nil
			return
		}
		errCh <- err
	}()

	// Wait for the RPC to trigger a connection attempt to the first address,
	// then cancel the RPC.  No other connection attempts should be started yet.
	if !holdNonExistant0.Wait(ctx) {
		t.Fatalf("Timeout waiting for connection attempt to backend 0")
	}
	rpcCancel()
	if err := <-errCh; err != nil {
		t.Fatalf("Expected RPC to fail be canceled, got %v", err)
	}

	// Since the connection attempt to the first address is still blocked, no
	// other connection attempts should be started yet.
	if holdNonExistant1.IsStarted() {
		t.Errorf("Got connection attempt to backend 1, expected no connection attempt.")
	}
	if holdNonExistant2.IsStarted() {
		t.Errorf("Got connection attempt to backend 2, expected no connection attempt.")
	}
	if holdGood.IsStarted() {
		t.Errorf("Got connection attempt to good backend, expected no connection attempt.")
	}

	// Allow the connection attempt to the first address to resume and wait for
	// the attempt for the second address. No other connection attempts should
	// be started yet.
	holdNonExistant0Again := dialer.Hold(nonExistantBackends[0])
	holdNonExistant0.Resume()
	if !holdNonExistant1.Wait(ctx) {
		t.Fatalf("Timeout waiting for connection attempt to backend 1")
	}
	if holdNonExistant0Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 0 again, expected no connection attempt.")
	}
	if holdNonExistant2.IsStarted() {
		t.Errorf("Got connection attempt to backend 2, expected no connection attempt.")
	}
	if holdGood.IsStarted() {
		t.Errorf("Got connection attempt to good backend, expected no connection attempt.")
	}

	// Allow the connection attempt to the second address to resume and wait for
	// the attempt for the third address.  No other connection attempts should
	// be started yet.
	holdNonExistant1Again := dialer.Hold(nonExistantBackends[1])
	holdNonExistant1.Resume()
	if !holdNonExistant2.Wait(ctx) {
		t.Fatalf("Timeout waiting for connection attempt to backend 2")
	}
	if holdNonExistant0Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 0 again, expected no connection attempt.")
	}
	if holdNonExistant1Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 1 again, expected no connection attempt.")
	}
	if holdGood.IsStarted() {
		t.Errorf("Got connection attempt to good backend, expected no connection attempt.")
	}

	// Allow the connection attempt to the third address to resume and wait
	// for the attempt for the final address. No other connection attempts
	// should be started yet.
	holdNonExistant2Again := dialer.Hold(nonExistantBackends[2])
	holdNonExistant2.Resume()
	if !holdGood.Wait(ctx) {
		t.Fatalf("Timeout waiting for connection attempt to good backend")
	}
	if holdNonExistant0Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 0 again, expected no connection attempt.")
	}
	if holdNonExistant1Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 1 again, expected no connection attempt.")
	}
	if holdNonExistant2Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 2 again, expected no connection attempt.")
	}

	// Allow the final attempt to resume.
	holdGood.Resume()

	// Wait for channel to become connected without any pending RPC.
	testutils.AwaitState(ctx, t, conn, connectivity.Ready)

	// No other connection attempts should have been started
	if holdNonExistant0Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 0 again, expected no connection attempt.")
	}
	if holdNonExistant1Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 1 again, expected no connection attempt.")
	}
	if holdNonExistant2Again.IsStarted() {
		t.Errorf("Got connection attempt to backend 2 again, expected no connection attempt.")
	}
}
