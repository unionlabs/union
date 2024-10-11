/*
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

package e2e_test

import (
	"context"
	"fmt"
	"net"
	"sort"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils/pickfirst"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// makeAggregateClusterResource returns an aggregate cluster resource with the
// given name and list of child names.
func makeAggregateClusterResource(name string, childNames []string) *v3clusterpb.Cluster {
	return e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: name,
		Type:        e2e.ClusterTypeAggregate,
		ChildNames:  childNames,
	})
}

// makeLogicalDNSClusterResource returns a LOGICAL_DNS cluster resource with the
// given name and given DNS host and port.
func makeLogicalDNSClusterResource(name, dnsHost string, dnsPort uint32) *v3clusterpb.Cluster {
	return e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: name,
		Type:        e2e.ClusterTypeLogicalDNS,
		DNSHostName: dnsHost,
		DNSPort:     dnsPort,
	})
}

// setupDNS unregisters the DNS resolver and registers a manual resolver for the
// same scheme. This allows the test to mock the DNS resolution by supplying the
// addresses of the test backends.
//
// Returns the following:
//   - a channel onto which the DNS target being resolved is written to by the
//     mock DNS resolver
//   - a manual resolver which is used to mock the actual DNS resolution
func setupDNS(t *testing.T) (chan resolver.Target, *manual.Resolver) {
	targetCh := make(chan resolver.Target, 1)

	mr := manual.NewBuilderWithScheme("dns")
	mr.BuildCallback = func(target resolver.Target, _ resolver.ClientConn, _ resolver.BuildOptions) { targetCh <- target }

	dnsResolverBuilder := resolver.Get("dns")
	resolver.Register(mr)

	t.Cleanup(func() { resolver.Register(dnsResolverBuilder) })
	return targetCh, mr
}

// TestAggregateCluster_WithTwoEDSClusters tests the case where the top-level
// cluster resource is an aggregate cluster. It verifies that RPCs fail when the
// management server has not responded to all requested EDS resources, and also
// that RPCs are routed to the highest priority cluster once all requested EDS
// resources have been sent by the management server.
func (s) TestAggregateCluster_WithTwoEDSClusters(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Start an xDS management server that pushes the EDS resource names onto a
	// channel when requested.
	edsResourceNameCh := make(chan []string, 1)
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() != version.V3EndpointsURL {
				return nil
			}
			if len(req.GetResourceNames()) == 0 {
				// This is the case for ACKs. Do nothing here.
				return nil
			}
			select {
			case edsResourceNameCh <- req.GetResourceNames():
			case <-ctx.Done():
			}
			return nil
		},
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend belongs to EDS cluster "cluster-1", while the second backend
	// belongs to EDS cluster "cluster-2".
	servers, cleanup2 := startTestServiceBackends(t, 2)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Configure an aggregate cluster, two EDS clusters and only one endpoints
	// resource (corresponding to the first EDS cluster) in the management
	// server.
	const clusterName1 = clusterName + "-cluster-1"
	const clusterName2 = clusterName + "-cluster-2"
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{clusterName1, clusterName2}),
			e2e.DefaultCluster(clusterName1, "", e2e.SecurityLevelNone),
			e2e.DefaultCluster(clusterName2, "", e2e.SecurityLevelNone),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(clusterName1, "localhost", []uint32{uint32(ports[0])})},
		SkipValidation: true,
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Wait for both EDS resources to be requested.
	func() {
		for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
			select {
			case names := <-edsResourceNameCh:
				// Copy and sort the sortedNames to avoid racing with an
				// OnStreamRequest call.
				sortedNames := make([]string, len(names))
				copy(sortedNames, names)
				sort.Strings(sortedNames)
				if cmp.Equal(sortedNames, []string{clusterName1, clusterName2}) {
					return
				}
			default:
			}
		}
	}()
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for all EDS resources %v to be requested", []string{clusterName1, clusterName2})
	}

	// Make an RPC with a short deadline. We expect this RPC to not succeed
	// because the management server has not responded with all EDS resources
	// requested.
	client := testgrpc.NewTestServiceClient(cc)
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("EmptyCall() code %s, want %s", status.Code(err), codes.DeadlineExceeded)
	}

	// Update the management server with the second EDS resource.
	resources.Endpoints = append(resources.Endpoints, e2e.DefaultEndpoint(clusterName2, "localhost", []uint32{uint32(ports[1])}))
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Make an RPC and ensure that it gets routed to cluster-1, implicitly
	// higher priority than cluster-2.
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[0].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
	}
}

// TestAggregateCluster_WithTwoEDSClusters_PrioritiesChange tests the case where
// the top-level cluster resource is an aggregate cluster. It verifies that RPCs
// are routed to the highest priority EDS cluster.
func (s) TestAggregateCluster_WithTwoEDSClusters_PrioritiesChange(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend belongs to EDS cluster "cluster-1", while the second backend
	// belongs to EDS cluster "cluster-2".
	servers, cleanup2 := startTestServiceBackends(t, 2)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Configure an aggregate cluster, two EDS clusters and the corresponding
	// endpoints resources in the management server.
	const clusterName1 = clusterName + "cluster-1"
	const clusterName2 = clusterName + "cluster-2"
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{clusterName1, clusterName2}),
			e2e.DefaultCluster(clusterName1, "", e2e.SecurityLevelNone),
			e2e.DefaultCluster(clusterName2, "", e2e.SecurityLevelNone),
		},
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{
			e2e.DefaultEndpoint(clusterName1, "localhost", []uint32{uint32(ports[0])}),
			e2e.DefaultEndpoint(clusterName2, "localhost", []uint32{uint32(ports[1])}),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Make an RPC and ensure that it gets routed to cluster-1, implicitly
	// higher priority than cluster-2.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[0].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
	}

	// Swap the priorities of the EDS clusters in the aggregate cluster.
	resources.Clusters = []*v3clusterpb.Cluster{
		makeAggregateClusterResource(clusterName, []string{clusterName2, clusterName1}),
		e2e.DefaultCluster(clusterName1, "", e2e.SecurityLevelNone),
		e2e.DefaultCluster(clusterName2, "", e2e.SecurityLevelNone),
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Wait for RPCs to get routed to cluster-2, which is now implicitly higher
	// priority than cluster-1, after the priority switch above.
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
		if peer.Addr.String() == addrs[1].Addr {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatal("Timeout waiting for RPCs to be routed to cluster-2 after priority switch")
	}
}

func hostAndPortFromAddress(t *testing.T, addr string) (string, uint32) {
	t.Helper()

	host, p, err := net.SplitHostPort(addr)
	if err != nil {
		t.Fatalf("Invalid serving address: %v", addr)
	}
	port, err := strconv.ParseUint(p, 10, 32)
	if err != nil {
		t.Fatalf("Invalid serving port %q: %v", p, err)
	}
	return host, uint32(port)
}

// TestAggregateCluster_WithOneDNSCluster tests the case where the top-level
// cluster resource is an aggregate cluster that resolves to a single
// LOGICAL_DNS cluster. The test verifies that RPCs can be made to backends that
// make up the LOGICAL_DNS cluster.
func (s) TestAggregateCluster_WithOneDNSCluster(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()
	host, port := hostAndPortFromAddress(t, server.Address)

	// Configure an aggregate cluster pointing to a single LOGICAL_DNS cluster.
	const dnsClusterName = clusterName + "-dns"
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{dnsClusterName}),
			makeLogicalDNSClusterResource(dnsClusterName, host, uint32(port)),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Make an RPC and ensure that it gets routed to the first backend since the
	// child policy for a LOGICAL_DNS cluster is pick_first by default.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != server.Address {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, server.Address)
	}
}

// Tests the case where the top-level cluster resource is an aggregate cluster
// that resolves to a single LOGICAL_DNS cluster. The specified dns hostname is
// expected to fail url parsing. The test verifies that the channel moves to
// TRANSIENT_FAILURE.
func (s) TestAggregateCluster_WithOneDNSCluster_ParseFailure(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Configure an aggregate cluster pointing to a single LOGICAL_DNS cluster.
	const dnsClusterName = clusterName + "-dns"
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{dnsClusterName}),
			makeLogicalDNSClusterResource(dnsClusterName, "%gh&%ij", uint32(8080)),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Ensure that the ClientConn moves to TransientFailure.
	for state := cc.GetState(); state != connectivity.TransientFailure; state = cc.GetState() {
		if !cc.WaitForStateChange(ctx, state) {
			t.Fatalf("Timed out waiting for state change. got %v; want %v", state, connectivity.TransientFailure)
		}
	}
}

// Tests the case where the top-level cluster resource is an aggregate cluster
// that resolves to a single LOGICAL_DNS cluster. The test verifies that RPCs
// can be made to backends that make up the LOGICAL_DNS cluster. The hostname of
// the LOGICAL_DNS cluster is updated, and the test verifies that RPCs can be
// made to backends that the new hostname resolves to.
func (s) TestAggregateCluster_WithOneDNSCluster_HostnameChange(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend is used initially for the LOGICAL_DNS cluster and an update
	// switches the cluster to use the second backend.
	servers, cleanup2 := startTestServiceBackends(t, 2)
	defer cleanup2()

	// Configure an aggregate cluster pointing to a single LOGICAL_DNS cluster.
	const dnsClusterName = clusterName + "-dns"
	dnsHostName, dnsPort := hostAndPortFromAddress(t, servers[0].Address)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{dnsClusterName}),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Make an RPC and ensure that it gets routed to the first backend since the
	// child policy for a LOGICAL_DNS cluster is pick_first by default.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != servers[0].Address {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, servers[0].Address)
	}

	// Update the LOGICAL_DNS cluster's hostname to point to the second backend.
	dnsHostName, dnsPort = hostAndPortFromAddress(t, servers[1].Address)
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{dnsClusterName}),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		SkipValidation: true,
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that traffic moves to the second backend eventually.
	for ctx.Err() == nil {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
		if peer.Addr.String() == servers[1].Address {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatal("Timeout when waiting for RPCs to switch to the second backend")
	}
}

// TestAggregateCluster_WithEDSAndDNS tests the case where the top-level cluster
// resource is an aggregate cluster that resolves to an EDS and a LOGICAL_DNS
// cluster. The test verifies that RPCs fail until both clusters are resolved to
// endpoints, and RPCs are routed to the higher priority EDS cluster.
func (s) TestAggregateCluster_WithEDSAndDNS(t *testing.T) {
	dnsTargetCh, dnsR := setupDNS(t)

	// Start an xDS management server that pushes the name of the requested EDS
	// resource onto a channel.
	edsResourceCh := make(chan string, 1)
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() != version.V3EndpointsURL {
				return nil
			}
			if len(req.GetResourceNames()) > 0 {
				select {
				case edsResourceCh <- req.GetResourceNames()[0]:
				default:
				}
			}
			return nil
		},
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend is used for the EDS cluster and the second backend is used for
	// the LOGICAL_DNS cluster.
	servers, cleanup3 := startTestServiceBackends(t, 2)
	defer cleanup3()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Configure an aggregate cluster pointing to an EDS and DNS cluster. Also
	// configure an endpoints resource for the EDS cluster.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
		dnsHostName    = "dns_host"
		dnsPort        = uint32(8080)
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, "", e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(edsClusterName, "localhost", []uint32{uint32(ports[0])})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Ensure that an EDS request is sent for the expected resource name.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for EDS request to be received on the management server")
	case name := <-edsResourceCh:
		if name != edsClusterName {
			t.Fatalf("Received EDS request with resource name %q, want %q", name, edsClusterName)
		}
	}

	// Ensure that the DNS resolver is started for the expected target.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for DNS resolver to be started")
	case target := <-dnsTargetCh:
		got, want := target.Endpoint(), fmt.Sprintf("%s:%d", dnsHostName, dnsPort)
		if got != want {
			t.Fatalf("DNS resolution started for target %q, want %q", got, want)
		}
	}

	// Make an RPC with a short deadline. We expect this RPC to not succeed
	// because the DNS resolver has not responded with endpoint addresses.
	client := testgrpc.NewTestServiceClient(cc)
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("EmptyCall() code %s, want %s", status.Code(err), codes.DeadlineExceeded)
	}

	// Update DNS resolver with test backend addresses.
	dnsR.UpdateState(resolver.State{Addresses: addrs[1:]})

	// Make an RPC and ensure that it gets routed to the first backend since the
	// EDS cluster is of higher priority than the LOGICAL_DNS cluster.
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[0].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
	}
}

// TestAggregateCluster_SwitchEDSAndDNS tests the case where the top-level
// cluster resource is an aggregate cluster. It initially resolves to a single
// EDS cluster. The test verifies that RPCs are routed to backends in the EDS
// cluster. Subsequently, the aggregate cluster resolves to a single DNS
// cluster. The test verifies that RPCs are successful, this time to backends in
// the DNS cluster.
func (s) TestAggregateCluster_SwitchEDSAndDNS(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend is used for the EDS cluster and the second backend is used for
	// the LOGICAL_DNS cluster.
	servers, cleanup3 := startTestServiceBackends(t, 2)
	defer cleanup3()
	addrs, ports := backendAddressesAndPorts(t, servers)
	dnsHostName, dnsPort := hostAndPortFromAddress(t, addrs[1].Addr)

	// Configure an aggregate cluster pointing to a single EDS cluster. Also,
	// configure the underlying EDS cluster (and the corresponding endpoints
	// resource) and DNS cluster (will be used later in the test).
	const dnsClusterName = clusterName + "-dns"
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsServiceName}),
			e2e.DefaultCluster(edsServiceName, "", e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(edsServiceName, "localhost", []uint32{uint32(ports[0])})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Ensure that the RPC is routed to the appropriate backend.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[0].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
	}

	// Update the aggregate cluster to point to a single DNS cluster.
	resources.Clusters = []*v3clusterpb.Cluster{
		makeAggregateClusterResource(clusterName, []string{dnsClusterName}),
		e2e.DefaultCluster(edsServiceName, "", e2e.SecurityLevelNone),
		makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that start getting routed to the backend corresponding to the
	// LOGICAL_DNS cluster.
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer))
		if peer.Addr.String() == addrs[1].Addr {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for RPCs to be routed to backend %q in the DNS cluster", addrs[1].Addr)
	}
}

// TestAggregateCluster_BadEDS_GoodToBadDNS tests the case where the top-level
// cluster is an aggregate cluster that resolves to an EDS and LOGICAL_DNS
// cluster. The test first asserts that no RPCs can be made after receiving an
// EDS response with zero endpoints because no update has been received from the
// DNS resolver yet. Once the DNS resolver pushes an update, the test verifies
// that we switch to the DNS cluster and can make a successful RPC. At this
// point when the DNS cluster returns an error, the test verifies that RPCs are
// still successful. This is the expected behavior because the cluster resolver
// policy eats errors from DNS Resolver after it has returned an error.
func (s) TestAggregateCluster_BadEDS_GoodToBadDNS(t *testing.T) {
	dnsTargetCh, dnsR := setupDNS(t)

	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends.
	servers, cleanup3 := startTestServiceBackends(t, 2)
	defer cleanup3()
	addrs, _ := backendAddressesAndPorts(t, servers)

	// Configure an aggregate cluster pointing to an EDS and LOGICAL_DNS
	// cluster. Also configure an endpoints resource for the EDS cluster which
	// triggers a NACK.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
		dnsHostName    = "dns_host"
		dnsPort        = uint32(8080)
	)
	emptyEndpointResource := e2e.DefaultEndpoint(edsServiceName, "localhost", nil)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, edsServiceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{emptyEndpointResource},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Make an RPC with a short deadline. We expect this RPC to not succeed
	// because the EDS resource came back with no endpoints, and we are yet to
	// push an update through the DNS resolver.
	client := testgrpc.NewTestServiceClient(cc)
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := client.EmptyCall(sCtx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("EmptyCall() code %s, want %s", status.Code(err), codes.DeadlineExceeded)
	}

	// Ensure that the DNS resolver is started for the expected target.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for DNS resolver to be started")
	case target := <-dnsTargetCh:
		got, want := target.Endpoint(), fmt.Sprintf("%s:%d", dnsHostName, dnsPort)
		if got != want {
			t.Fatalf("DNS resolution started for target %q, want %q", got, want)
		}
	}

	// Update DNS resolver with test backend addresses.
	dnsR.UpdateState(resolver.State{Addresses: addrs})

	// Ensure that RPCs start getting routed to the first backend since the
	// child policy for a LOGICAL_DNS cluster is pick_first by default.
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		peer := &peer.Peer{}
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			t.Logf("EmptyCall() failed: %v", err)
			continue
		}
		if peer.Addr.String() == addrs[0].Addr {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for RPCs to be routed to backend %q in the DNS cluster", addrs[0].Addr)
	}

	// Push an error from the DNS resolver as well.
	dnsErr := fmt.Errorf("DNS error")
	dnsR.ReportError(dnsErr)

	// Ensure that RPCs continue to succeed for the next second.
	for end := time.Now().Add(time.Second); time.Now().Before(end); <-time.After(defaultTestShortTimeout) {
		peer := &peer.Peer{}
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
		if peer.Addr.String() != addrs[0].Addr {
			t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
		}
	}
}

// TestAggregateCluster_BadEDS_GoodToBadDNS tests the case where the top-level
// cluster is an aggregate cluster that resolves to an EDS and LOGICAL_DNS
// cluster. The test first sends an EDS response which triggers an NACK. Once
// the DNS resolver pushes an update, the test verifies that we switch to the
// DNS cluster and can make a successful RPC.
func (s) TestAggregateCluster_BadEDSFromError_GoodToBadDNS(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()
	dnsHostName, dnsPort := hostAndPortFromAddress(t, server.Address)

	// Configure an aggregate cluster pointing to an EDS and LOGICAL_DNS
	// cluster. Also configure an empty endpoints resource for the EDS cluster
	// that contains no endpoints.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
	)
	nackEndpointResource := e2e.DefaultEndpoint(edsServiceName, "localhost", nil)
	nackEndpointResource.Endpoints = []*v3endpointpb.LocalityLbEndpoints{
		{
			LoadBalancingWeight: &wrapperspb.UInt32Value{
				Value: 0, // causes an NACK
			},
		},
	}
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, edsServiceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{nackEndpointResource},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Ensure that RPCs start getting routed to the first backend since the
	// child policy for a LOGICAL_DNS cluster is pick_first by default.
	pickfirst.CheckRPCsToBackend(ctx, cc, resolver.Address{Addr: server.Address})
}

// TestAggregateCluster_BadDNS_GoodEDS tests the case where the top-level
// cluster is an aggregate cluster that resolves to an LOGICAL_DNS and EDS
// cluster. When the DNS Resolver returns an error and EDS cluster returns a
// good update, this test verifies the cluster_resolver balancer correctly falls
// back from the LOGICAL_DNS cluster to the EDS cluster.
func (s) TestAggregateCluster_BadDNS_GoodEDS(t *testing.T) {
	dnsTargetCh, dnsR := setupDNS(t)
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()
	_, edsPort := hostAndPortFromAddress(t, server.Address)

	// Configure an aggregate cluster pointing to an LOGICAL_DNS and EDS
	// cluster. Also configure an endpoints resource for the EDS cluster.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
		dnsHostName    = "bad.ip.v4.address"
		dnsPort        = 8080
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{dnsClusterName, edsClusterName}),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
			e2e.DefaultCluster(edsClusterName, edsServiceName, e2e.SecurityLevelNone),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(edsServiceName, "localhost", []uint32{uint32(edsPort)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Ensure that the DNS resolver is started for the expected target.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for DNS resolver to be started")
	case target := <-dnsTargetCh:
		got, want := target.Endpoint(), fmt.Sprintf("%s:%d", dnsHostName, dnsPort)
		if got != want {
			t.Fatalf("DNS resolution started for target %q, want %q", got, want)
		}
	}

	// Produce a bad resolver update from the DNS resolver.
	dnsErr := fmt.Errorf("DNS error")
	dnsR.ReportError(dnsErr)

	// RPCs should work, higher level DNS cluster errors so should fallback to
	// EDS cluster.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != server.Address {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, server.Address)
	}
}

// TestAggregateCluster_BadEDS_BadDNS tests the case where the top-level cluster
// is an aggregate cluster that resolves to an EDS and LOGICAL_DNS cluster. When
// the EDS request returns a resource that contains no endpoints, the test
// verifies that we switch to the DNS cluster. When the DNS cluster returns an
// error, the test verifies that RPCs fail with the error triggered by the DNS
// Discovery Mechanism (from sending an empty address list down).
func (s) TestAggregateCluster_BadEDS_BadDNS(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Configure an aggregate cluster pointing to an EDS and LOGICAL_DNS
	// cluster. Also configure an empty endpoints resource for the EDS cluster
	// that contains no endpoints.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
	)
	emptyEndpointResource := e2e.DefaultEndpoint(edsServiceName, "localhost", nil)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, edsServiceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, "bad.ip.v4.address", 8080),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{emptyEndpointResource},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Ensure that the error from the DNS Resolver leads to an empty address
	// update for both priorities.
	client := testgrpc.NewTestServiceClient(cc)
	for ctx.Err() == nil {
		_, err := client.EmptyCall(ctx, &testpb.Empty{})
		if err == nil {
			t.Fatal("EmptyCall() succeeded when expected to fail")
		}
		if status.Code(err) == codes.Unavailable && strings.Contains(err.Error(), "produced zero addresses") {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for RPCs to fail with expected code and error")
	}
}

// TestAggregateCluster_NoFallback_EDSNackedWithPreviousGoodUpdate tests the
// scenario where the top-level cluster is an aggregate cluster that resolves to
// an EDS and LOGICAL_DNS cluster. The management server first sends a good EDS
// response for the EDS cluster and the test verifies that RPCs get routed to
// the EDS cluster. The management server then sends a bad EDS response. The
// test verifies that the cluster_resolver LB policy continues to use the
// previously received good update and that RPCs still get routed to the EDS
// cluster.
func (s) TestAggregateCluster_NoFallback_EDSNackedWithPreviousGoodUpdate(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend is used for the EDS cluster and the second backend is used for
	// the LOGICAL_DNS cluster.
	servers, cleanup3 := startTestServiceBackends(t, 2)
	defer cleanup3()
	addrs, ports := backendAddressesAndPorts(t, servers)
	dnsHostName, dnsPort := hostAndPortFromAddress(t, servers[1].Address)

	// Configure an aggregate cluster pointing to an EDS and DNS cluster. Also
	// configure an endpoints resource for the EDS cluster.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, "", e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(edsClusterName, "localhost", []uint32{uint32(ports[0])})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Make an RPC and ensure that it gets routed to the first backend since the
	// EDS cluster is of higher priority than the LOGICAL_DNS cluster.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[0].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
	}

	// Push an EDS resource from the management server that is expected to be
	// NACKed by the xDS client. Since the cluster_resolver LB policy has a
	// previously received good EDS resource, it will continue to use that.
	resources.Endpoints[0].Endpoints[0].LbEndpoints[0].LoadBalancingWeight = &wrapperspb.UInt32Value{Value: 0}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that RPCs continue to get routed to the EDS cluster for the next
	// second.
	for end := time.Now().Add(time.Second); time.Now().Before(end); <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
		if peer.Addr.String() != addrs[0].Addr {
			t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
		}
	}
}

// TestAggregateCluster_Fallback_EDSNackedWithoutPreviousGoodUpdate tests the
// scenario where the top-level cluster is an aggregate cluster that resolves to
// an EDS and LOGICAL_DNS cluster.  The management server sends a bad EDS
// response. The test verifies that the cluster_resolver LB policy falls back to
// the LOGICAL_DNS cluster, because it is supposed to treat the bad EDS response
// as though it received an update with no endpoints.
func (s) TestAggregateCluster_Fallback_EDSNackedWithoutPreviousGoodUpdate(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend is used for the EDS cluster and the second backend is used for
	// the LOGICAL_DNS cluster.
	servers, cleanup3 := startTestServiceBackends(t, 2)
	defer cleanup3()
	addrs, ports := backendAddressesAndPorts(t, servers)
	dnsHostName, dnsPort := hostAndPortFromAddress(t, servers[1].Address)

	// Configure an aggregate cluster pointing to an EDS and DNS cluster.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, "", e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(edsClusterName, "localhost", []uint32{uint32(ports[0])})},
		SkipValidation: true,
	}

	// Set a load balancing weight of 0 for the backend in the EDS resource.
	// This is expected to be NACKed by the xDS client. Since the
	// cluster_resolver LB policy has no previously received good EDS resource,
	// it will treat this as though it received an update with no endpoints.
	resources.Endpoints[0].Endpoints[0].LbEndpoints[0].LoadBalancingWeight = &wrapperspb.UInt32Value{Value: 0}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	// Make an RPC and ensure that it gets routed to the LOGICAL_DNS cluster.
	peer := &peer.Peer{}
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[1].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[1].Addr)
	}
}

// TestAggregateCluster_Fallback_EDS_ResourceNotFound tests the scenario where
// the top-level cluster is an aggregate cluster that resolves to an EDS and
// LOGICAL_DNS cluster.  The management server does not respond with the EDS
// cluster. The test verifies that the cluster_resolver LB policy falls back to
// the LOGICAL_DNS cluster in this case.
func (s) TestAggregateCluster_Fallback_EDS_ResourceNotFound(t *testing.T) {
	// Start an xDS management server.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start a test backend for the LOGICAL_DNS cluster.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()
	dnsHostName, dnsPort := hostAndPortFromAddress(t, server.Address)

	// Configure an aggregate cluster pointing to an EDS and DNS cluster. No
	// endpoints are configured for the EDS cluster.
	const (
		edsClusterName = clusterName + "-eds"
		dnsClusterName = clusterName + "-dns"
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, "", e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client talking to the above management server, configured
	// with a short watch expiry timeout.
	xdsClient, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:               t.Name(),
		Contents:           bootstrapContents,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push a service config specifying the use of
	// the cds LB policy as the top-level LB policy, and a corresponding config
	// with a single cluster.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": "%s"
				}
			}]
		}`, clusterName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Create a ClientConn.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure that it gets routed to the LOGICAL_DNS cluster.
	// Even though the EDS cluster is of higher priority, since the management
	// server does not respond with an EDS resource, the cluster_resolver LB
	// policy is expected to fallback to the LOGICAL_DNS cluster once the watch
	// timeout expires.
	peer := &peer.Peer{}
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != server.Address {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, server.Address)
	}
}
