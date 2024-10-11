/*
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
 */

package e2e_test

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	rrutil "google.golang.org/grpc/internal/testutils/roundrobin"
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
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/xds/internal/balancer/clusterresolver" // Register the "cluster_resolver_experimental" LB policy.
	"google.golang.org/grpc/xds/internal/balancer/priority"
)

const (
	clusterName    = "cluster-my-service-client-side-xds"
	edsServiceName = "endpoints-my-service-client-side-xds"
	localityName1  = "my-locality-1"
	localityName2  = "my-locality-2"
	localityName3  = "my-locality-3"

	defaultTestTimeout            = 5 * time.Second
	defaultTestShortTimeout       = 10 * time.Millisecond
	defaultTestWatchExpiryTimeout = 500 * time.Millisecond
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// backendAddressesAndPorts extracts the address and port of each of the
// StubServers passed in and returns them. Fails the test if any of the
// StubServers passed have an invalid address.
func backendAddressesAndPorts(t *testing.T, servers []*stubserver.StubServer) ([]resolver.Address, []uint32) {
	addrs := make([]resolver.Address, len(servers))
	ports := make([]uint32, len(servers))
	for i := 0; i < len(servers); i++ {
		addrs[i] = resolver.Address{Addr: servers[i].Address}
		ports[i] = testutils.ParsePort(t, servers[i].Address)
	}
	return addrs, ports
}

func startTestServiceBackends(t *testing.T, numBackends int) ([]*stubserver.StubServer, func()) {
	var servers []*stubserver.StubServer
	for i := 0; i < numBackends; i++ {
		servers = append(servers, stubserver.StartTestService(t, nil))
		servers[i].StartServer()
	}

	return servers, func() {
		for _, server := range servers {
			server.Stop()
		}
	}
}

// clientEndpointsResource returns an EDS resource for the specified nodeID,
// service name and localities.
func clientEndpointsResource(nodeID, edsServiceName string, localities []e2e.LocalityOptions) e2e.UpdateOptions {
	return e2e.UpdateOptions{
		NodeID: nodeID,
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
			ClusterName: edsServiceName,
			Host:        "localhost",
			Localities:  localities,
		})},
		SkipValidation: true,
	}
}

// TestEDS_OneLocality tests the cluster_resolver LB policy using an EDS
// resource with one locality. The following scenarios are tested:
//  1. Single backend. Test verifies that RPCs reach this backend.
//  2. Add a backend. Test verifies that RPCs are roundrobined across the two
//     backends.
//  3. Remove one backend. Test verifies that all RPCs reach the other backend.
//  4. Replace the backend. Test verifies that all RPCs reach the new backend.
func (s) TestEDS_OneLocality(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start backend servers which provide an implementation of the TestService.
	servers, cleanup2 := startTestServiceBackends(t, 3)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Create xDS resources for consumption by the test. We start off with a
	// single backend in a single EDS locality.
	resources := clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: ports[0]}},
	}})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client for use by the cluster_resolver LB policy.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push a service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, client))

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Ensure RPCs are being roundrobined across the single backend.
	testClient := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, testClient, addrs[:1]); err != nil {
		t.Fatal(err)
	}

	// Add a backend to the same locality, and ensure RPCs are sent in a
	// roundrobin fashion across the two backends.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: ports[0]}, {Port: ports[1]}},
	}})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := rrutil.CheckRoundRobinRPCs(ctx, testClient, addrs[:2]); err != nil {
		t.Fatal(err)
	}

	// Remove the first backend, and ensure all RPCs are sent to the second
	// backend.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: ports[1]}},
	}})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := rrutil.CheckRoundRobinRPCs(ctx, testClient, addrs[1:2]); err != nil {
		t.Fatal(err)
	}

	// Replace the backend, and ensure all RPCs are sent to the new backend.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: ports[2]}},
	}})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := rrutil.CheckRoundRobinRPCs(ctx, testClient, addrs[2:3]); err != nil {
		t.Fatal(err)
	}
}

// TestEDS_MultipleLocalities tests the cluster_resolver LB policy using an EDS
// resource with multiple localities. The following scenarios are tested:
//  1. Two localities, each with a single backend. Test verifies that RPCs are
//     weighted roundrobined across these two backends.
//  2. Add another locality, with a single backend. Test verifies that RPCs are
//     weighted roundrobined across all the backends.
//  3. Remove one locality. Test verifies that RPCs are weighted roundrobined
//     across backends from the remaining localities.
//  4. Add a backend to one locality. Test verifies that RPCs are weighted
//     roundrobined across localities.
//  5. Change the weight of one of the localities. Test verifies that RPCs are
//     weighted roundrobined across the localities.
//
// In our LB policy tree, one of the descendents of the "cluster_resolver" LB
// policy is the "weighted_target" LB policy which performs weighted roundrobin
// across localities (and this has a randomness component associated with it).
// Therefore, the moment we have backends from more than one locality, RPCs are
// weighted roundrobined across them.
func (s) TestEDS_MultipleLocalities(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start backend servers which provide an implementation of the TestService.
	servers, cleanup2 := startTestServiceBackends(t, 4)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Create xDS resources for consumption by the test. We start off with two
	// localities, and single backend in each of them.
	resources := clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{
		{
			Name:     localityName1,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[0]}},
		},
		{
			Name:     localityName2,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[1]}},
		},
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client for use by the cluster_resolver LB policy.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, client))

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Ensure RPCs are being weighted roundrobined across the two backends.
	testClient := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckWeightedRoundRobinRPCs(ctx, testClient, addrs[0:2]); err != nil {
		t.Fatal(err)
	}

	// Add another locality with a single backend, and ensure RPCs are being
	// weighted roundrobined across the three backends.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{
		{
			Name:     localityName1,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[0]}},
		},
		{
			Name:     localityName2,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[1]}},
		},
		{
			Name:     localityName3,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[2]}},
		},
	})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := rrutil.CheckWeightedRoundRobinRPCs(ctx, testClient, addrs[0:3]); err != nil {
		t.Fatal(err)
	}

	// Remove the first locality, and ensure RPCs are being weighted
	// roundrobined across the remaining two backends.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{
		{
			Name:     localityName2,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[1]}},
		},
		{
			Name:     localityName3,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[2]}},
		},
	})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := rrutil.CheckWeightedRoundRobinRPCs(ctx, testClient, addrs[1:3]); err != nil {
		t.Fatal(err)
	}

	// Add a backend to one locality, and ensure weighted roundrobin. Since RPCs
	// are roundrobined across localities, locality2's backend will receive
	// twice the traffic.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{
		{
			Name:     localityName2,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[1]}},
		},
		{
			Name:     localityName3,
			Weight:   1,
			Backends: []e2e.BackendOptions{{Port: ports[2]}, {Port: ports[3]}},
		},
	})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	wantAddrs := []resolver.Address{addrs[1], addrs[1], addrs[2], addrs[3]}
	if err := rrutil.CheckWeightedRoundRobinRPCs(ctx, testClient, wantAddrs); err != nil {
		t.Fatal(err)
	}
}

// TestEDS_EndpointsHealth tests the cluster_resolver LB policy using an EDS
// resource which specifies endpoint health information and verifies that
// traffic is routed only to backends deemed capable of receiving traffic.
func (s) TestEDS_EndpointsHealth(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start backend servers which provide an implementation of the TestService.
	servers, cleanup2 := startTestServiceBackends(t, 12)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Create xDS resources for consumption by the test.  Two localities with
	// six backends each, with two of the six backends being healthy. Both
	// UNKNOWN and HEALTHY are considered by gRPC for load balancing.
	resources := clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{
		{
			Name:   localityName1,
			Weight: 1,
			Backends: []e2e.BackendOptions{
				{Port: ports[0], HealthStatus: v3corepb.HealthStatus_UNKNOWN},
				{Port: ports[1], HealthStatus: v3corepb.HealthStatus_HEALTHY},
				{Port: ports[2], HealthStatus: v3corepb.HealthStatus_UNHEALTHY},
				{Port: ports[3], HealthStatus: v3corepb.HealthStatus_DRAINING},
				{Port: ports[4], HealthStatus: v3corepb.HealthStatus_TIMEOUT},
				{Port: ports[5], HealthStatus: v3corepb.HealthStatus_DEGRADED},
			},
		},
		{
			Name:   localityName2,
			Weight: 1,
			Backends: []e2e.BackendOptions{
				{Port: ports[6], HealthStatus: v3corepb.HealthStatus_UNKNOWN},
				{Port: ports[7], HealthStatus: v3corepb.HealthStatus_HEALTHY},
				{Port: ports[8], HealthStatus: v3corepb.HealthStatus_UNHEALTHY},
				{Port: ports[9], HealthStatus: v3corepb.HealthStatus_DRAINING},
				{Port: ports[10], HealthStatus: v3corepb.HealthStatus_TIMEOUT},
				{Port: ports[11], HealthStatus: v3corepb.HealthStatus_DEGRADED},
			},
		},
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client for use by the cluster_resolver LB policy.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, client))

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Ensure RPCs are being weighted roundrobined across healthy backends from
	// both localities.
	testClient := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckWeightedRoundRobinRPCs(ctx, testClient, append(addrs[0:2], addrs[6:8]...)); err != nil {
		t.Fatal(err)
	}
}

// TestEDS_EmptyUpdate tests the cluster_resolver LB policy using an EDS
// resource with no localities and verifies that RPCs fail with "all priorities
// removed" error.
func (s) TestEDS_EmptyUpdate(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start backend servers which provide an implementation of the TestService.
	servers, cleanup2 := startTestServiceBackends(t, 4)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	oldCacheTimeout := priority.DefaultSubBalancerCloseTimeout
	priority.DefaultSubBalancerCloseTimeout = 100 * time.Microsecond
	defer func() { priority.DefaultSubBalancerCloseTimeout = oldCacheTimeout }()

	// Create xDS resources for consumption by the test. The first update is an
	// empty update. This should put the channel in TRANSIENT_FAILURE.
	resources := clientEndpointsResource(nodeID, edsServiceName, nil)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client for use by the cluster_resolver LB policy.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, client))

	// Create a ClientConn and ensure that RPCs fail with "all priorities
	// removed" error. This is the expected error when the cluster_resolver LB
	// policy receives an EDS update with no localities.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()
	testClient := testgrpc.NewTestServiceClient(cc)
	if err := waitForProducedZeroAddressesError(ctx, t, testClient); err != nil {
		t.Fatal(err)
	}

	// Add a locality with one backend and ensure RPCs are successful.
	resources = clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: ports[0]}},
	}})
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := rrutil.CheckRoundRobinRPCs(ctx, testClient, addrs[:1]); err != nil {
		t.Fatal(err)
	}

	// Push another empty update and ensure that RPCs fail with "all priorities
	// removed" error again.
	resources = clientEndpointsResource(nodeID, edsServiceName, nil)
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	if err := waitForProducedZeroAddressesError(ctx, t, testClient); err != nil {
		t.Fatal(err)
	}
}

// TestEDS_ResourceRemoved tests the case where the EDS resource requested by
// the clusterresolver LB policy is removed from the management server. The test
// verifies that the EDS watch is not canceled and that RPCs continue to succeed
// with the previously received configuration.
func (s) TestEDS_ResourceRemoved(t *testing.T) {
	// Start an xDS management server that uses a couple of channels to
	// notify the test about the following events:
	// - an EDS requested with the expected resource name is requested
	// - EDS resource is unrequested, i.e, an EDS request with no resource name
	//   is received, which indicates that we are not longer interested in that
	//   resource.
	edsResourceRequestedCh := make(chan struct{}, 1)
	edsResourceCanceledCh := make(chan struct{}, 1)
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() == version.V3EndpointsURL {
				switch len(req.GetResourceNames()) {
				case 0:
					select {
					case edsResourceCanceledCh <- struct{}{}:
					default:
					}
				case 1:
					if req.GetResourceNames()[0] == edsServiceName {
						select {
						case edsResourceRequestedCh <- struct{}{}:
						default:
						}
					}
				default:
					t.Errorf("Unexpected number of resources, %d, in an EDS request", len(req.GetResourceNames()))
				}
			}
			return nil
		},
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Configure cluster and endpoints resources in the management server.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, edsServiceName, e2e.SecurityLevelNone)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(edsServiceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
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

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Delete the endpoints resource from the management server.
	resources.Endpoints = nil
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that RPCs continue to succeed for the next second, and that the
	// EDS watch is not canceled.
	for end := time.Now().Add(time.Second); time.Now().Before(end); <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
		select {
		case <-edsResourceCanceledCh:
			t.Fatal("EDS watch canceled when not expected to be canceled")
		default:
		}
	}
}

// TestEDS_ClusterResourceDoesNotContainEDSServiceName tests the case where the
// Cluster resource sent by the management server does not contain an EDS
// service name. The test verifies that the cluster_resolver LB policy uses the
// cluster name for the EDS resource.
func (s) TestEDS_ClusterResourceDoesNotContainEDSServiceName(t *testing.T) {
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
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Configure cluster and endpoints resources with the same name in the management server. The cluster resource does not specify an EDS service name.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, "", e2e.SecurityLevelNone)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(clusterName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
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

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for EDS request to be received on the management server")
	case name := <-edsResourceCh:
		if name != clusterName {
			t.Fatalf("Received EDS request with resource name %q, want %q", name, clusterName)
		}
	}
}

// TestEDS_ClusterResourceUpdates verifies different scenarios with regards to
// cluster resource updates.
//
//   - The first cluster resource contains an eds_service_name. The test verifies
//     that an EDS request is sent for the received eds_service_name. It also
//     verifies that a subsequent RPC gets routed to a backend belonging to that
//     service name.
//   - The next cluster resource update contains no eds_service_name. The test
//     verifies that a subsequent EDS request is sent for the cluster_name and
//     that the previously received eds_service_name is no longer requested. It
//     also verifies that a subsequent RPC gets routed to a backend belonging to
//     the service represented by the cluster_name.
//   - The next cluster resource update changes the circuit breaking
//     configuration, but does not change the service name. The test verifies
//     that a subsequent RPC gets routed to the same backend as before.
func (s) TestEDS_ClusterResourceUpdates(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Start an xDS management server that pushes the EDS resource names onto a
	// channel.
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
			case <-ctx.Done():
			case edsResourceNameCh <- req.GetResourceNames():
			}
			return nil
		},
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start two test backends and extract their host and port. The first
	// backend is used for the EDS resource identified by the eds_service_name,
	// and the second backend is used for the EDS resource identified by the
	// cluster_name.
	servers, cleanup2 := startTestServiceBackends(t, 2)
	defer cleanup2()
	addrs, ports := backendAddressesAndPorts(t, servers)

	// Configure cluster and endpoints resources in the management server.
	resources := e2e.UpdateOptions{
		NodeID:   nodeID,
		Clusters: []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, edsServiceName, e2e.SecurityLevelNone)},
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{
			e2e.DefaultEndpoint(edsServiceName, "localhost", []uint32{uint32(ports[0])}),
			e2e.DefaultEndpoint(clusterName, "localhost", []uint32{uint32(ports[1])}),
		},
		SkipValidation: true,
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create xDS client, configure cds_experimental LB policy with a manual
	// resolver, and dial the test backends.
	cc, cleanup := setupAndDial(t, bootstrapContents)
	defer cleanup()

	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if peer.Addr.String() != addrs[0].Addr {
		t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[0].Addr)
	}

	// Ensure EDS watch is registered for eds_service_name.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for EDS request to be received on the management server")
	case names := <-edsResourceNameCh:
		if !cmp.Equal(names, []string{edsServiceName}) {
			t.Fatalf("Received EDS request with resource names %v, want %v", names, []string{edsServiceName})
		}
	}

	// Change the cluster resource to not contain an eds_service_name.
	resources.Clusters = []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, "", e2e.SecurityLevelNone)}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that an EDS watch for eds_service_name is canceled and new watch
	// for cluster_name is registered. The actual order in which this happens is
	// not deterministic, i.e the watch for old resource could be canceled
	// before the new one is registered or vice-versa. In either case,
	// eventually, we want to see a request to the management server for just
	// the cluster_name.
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		names := <-edsResourceNameCh
		if cmp.Equal(names, []string{clusterName}) {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for old EDS watch %q to be canceled and new one %q to be registered", edsServiceName, clusterName)
	}

	// Make a RPC, and ensure that it gets routed to second backend,
	// corresponding to the cluster_name.
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			continue
		}
		if peer.Addr.String() == addrs[1].Addr {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for EmptyCall() to be routed to correct backend %q", addrs[1].Addr)
	}

	// Change cluster resource circuit breaking count.
	resources.Clusters[0].CircuitBreakers = &v3clusterpb.CircuitBreakers{
		Thresholds: []*v3clusterpb.CircuitBreakers_Thresholds{
			{
				Priority:    v3corepb.RoutingPriority_DEFAULT,
				MaxRequests: wrapperspb.UInt32(512),
			},
		},
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that RPCs continue to get routed to the second backend for the
	// next second.
	for end := time.Now().Add(time.Second); time.Now().Before(end); <-time.After(defaultTestShortTimeout) {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer)); err != nil {
			t.Fatalf("EmptyCall() failed: %v", err)
		}
		if peer.Addr.String() != addrs[1].Addr {
			t.Fatalf("EmptyCall() routed to backend %q, want %q", peer.Addr, addrs[1].Addr)
		}
	}
}

// TestEDS_BadUpdateWithoutPreviousGoodUpdate tests the case where the
// management server sends a bad update (one that is NACKed by the xDS client).
// Since the cluster_resolver LB policy does not have a previously received good
// update, it is expected to treat this bad update as though it received an
// update with no endpoints. Hence RPCs are expected to fail with "all
// priorities removed" error.
func (s) TestEDS_BadUpdateWithoutPreviousGoodUpdate(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start a backend server that implements the TestService.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Create an EDS resource with a load balancing weight of 0. This will
	// result in the resource being NACKed by the xDS client. Since the
	// cluster_resolver LB policy does not have a previously received good EDS
	// update, it should treat this update as an empty EDS update.
	resources := clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: testutils.ParsePort(t, server.Address)}},
	}})
	resources.Endpoints[0].Endpoints[0].LbEndpoints[0].LoadBalancingWeight = &wrapperspb.UInt32Value{Value: 0}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client for use by the cluster_resolver LB policy.
	xdsClient, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push a service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Create a ClientConn and verify that RPCs fail with "all priorities
	// removed" error.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()
	client := testgrpc.NewTestServiceClient(cc)
	if err := waitForProducedZeroAddressesError(ctx, t, client); err != nil {
		t.Fatal(err)
	}
}

// TestEDS_BadUpdateWithPreviousGoodUpdate tests the case where the
// cluster_resolver LB policy receives a good EDS update from the management
// server and the test verifies that RPCs are successful. Then, a bad update is
// received from the management server (one that is NACKed by the xDS client).
// The test verifies that the previously received good update is still being
// used and that RPCs are still successful.
func (s) TestEDS_BadUpdateWithPreviousGoodUpdate(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Start a backend server that implements the TestService.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Create an EDS resource for consumption by the test.
	resources := clientEndpointsResource(nodeID, edsServiceName, []e2e.LocalityOptions{{
		Name:     localityName1,
		Weight:   1,
		Backends: []e2e.BackendOptions{{Port: testutils.ParsePort(t, server.Address)}},
	}})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create an xDS client for use by the cluster_resolver LB policy.
	xdsClient, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Create a manual resolver and push a service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Ensure RPCs are being roundrobined across the single backend.
	client := testgrpc.NewTestServiceClient(cc)
	if err := rrutil.CheckRoundRobinRPCs(ctx, client, []resolver.Address{{Addr: server.Address}}); err != nil {
		t.Fatal(err)
	}

	// Update the endpoints resource in the management server with a load
	// balancing weight of 0. This will result in the resource being NACKed by
	// the xDS client. But since the cluster_resolver LB policy has a previously
	// received good EDS update, it should continue using it.
	resources.Endpoints[0].Endpoints[0].LbEndpoints[0].LoadBalancingWeight = &wrapperspb.UInt32Value{Value: 0}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that RPCs continue to succeed for the next second.
	for end := time.Now().Add(time.Second); time.Now().Before(end); <-time.After(defaultTestShortTimeout) {
		if err := rrutil.CheckRoundRobinRPCs(ctx, client, []resolver.Address{{Addr: server.Address}}); err != nil {
			t.Fatal(err)
		}
	}
}

// TestEDS_ResourceNotFound tests the case where the requested EDS resource does
// not exist on the management server. Once the watch timer associated with the
// requested resource expires, the cluster_resolver LB policy receives a
// "resource-not-found" callback from the xDS client and is expected to treat it
// as though it received an update with no endpoints. Hence RPCs are expected to
// fail with "all priorities removed" error.
func (s) TestEDS_ResourceNotFound(t *testing.T) {
	// Spin up a management server to receive xDS resources from.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create an xDS client talking to the above management server, configured
	// with a short watch expiry timeout.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	xdsClient, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:               t.Name(),
		Contents:           bc,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer close()

	// Configure no resources on the management server.
	resources := e2e.UpdateOptions{NodeID: nodeID, SkipValidation: true}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Create a manual resolver and push a service config specifying the use of
	// the cluster_resolver LB policy with a single discovery mechanism.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cluster_resolver_experimental":{
					"discoveryMechanisms": [{
						"cluster": "%s",
						"type": "EDS",
						"edsServiceName": "%s",
						"outlierDetection": {}
					}],
					"xdsLbPolicy":[{"round_robin":{}}]
				}
			}]
		}`, clusterName, edsServiceName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Create a ClientConn and verify that RPCs fail with "all priorities
	// removed" error.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()
	client := testgrpc.NewTestServiceClient(cc)
	if err := waitForProducedZeroAddressesError(ctx, t, client); err != nil {
		t.Fatal(err)
	}
}

// waitForAllPrioritiesRemovedError repeatedly makes RPCs using the
// TestServiceClient until they fail with an error which indicates that no
// resolver addresses have been produced. A non-nil error is returned if the
// context expires before RPCs fail with the expected error.
func waitForProducedZeroAddressesError(ctx context.Context, t *testing.T, client testgrpc.TestServiceClient) error {
	for ; ctx.Err() == nil; <-time.After(time.Millisecond) {
		_, err := client.EmptyCall(ctx, &testpb.Empty{})
		if err == nil {
			t.Log("EmptyCall() succeeded after error in Discovery Mechanism")
			continue
		}
		if code := status.Code(err); code != codes.Unavailable {
			t.Logf("EmptyCall() returned code: %v, want: %v", code, codes.Unavailable)
			continue
		}
		if !strings.Contains(err.Error(), "produced zero addresses") {
			t.Logf("EmptyCall() = %v, want %v", err, "produced zero addresses")
			continue
		}
		return nil
	}
	return errors.New("timeout when waiting for RPCs to fail with UNAVAILABLE status and produced zero addresses")
}
