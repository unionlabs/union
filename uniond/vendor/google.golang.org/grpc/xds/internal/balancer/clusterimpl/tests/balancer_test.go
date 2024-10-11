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
 *
 */

package clusterimpl_test

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/fakeserver"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3pickfirstpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/pick_first/v3"
	v3lrspb "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/xds"
)

const (
	defaultTestTimeout      = 5 * time.Second
	defaultTestShortTimeout = 100 * time.Millisecond
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// TestConfigUpdateWithSameLoadReportingServerConfig tests the scenario where
// the clusterimpl LB policy receives a config update with no change in the load
// reporting server configuration. The test verifies that the existing load
// reporting stream is not terminated and that a new load reporting stream is not
// created.
func (s) TestConfigUpdateWithSameLoadReportingServerConfig(t *testing.T) {
	// Create an xDS management server that serves ADS and LRS requests.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{SupportLoadReportingService: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS resolver with the above bootstrap configuration.
	var resolverBuilder resolver.Builder
	var err error
	if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
		resolverBuilder, err = newResolver.(func([]byte) (resolver.Builder, error))(bc)
		if err != nil {
			t.Fatalf("Failed to create xDS resolver for testing: %v", err)
		}
	}

	// Start a server backend exposing the test service.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Configure the xDS management server with default resources. Override the
	// default cluster to include an LRS server config pointing to self.
	const serviceName = "my-test-xds-service"
	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: serviceName,
		NodeID:     nodeID,
		Host:       "localhost",
		Port:       testutils.ParsePort(t, server.Address),
		SecLevel:   e2e.SecurityLevelNone,
	})
	resources.Clusters[0].LrsServer = &v3corepb.ConfigSource{
		ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
			Self: &v3corepb.SelfConfigSource{},
		},
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(resolverBuilder))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}

	// Ensure that an LRS stream is created.
	if _, err := mgmtServer.LRSServer.LRSStreamOpenChan.Receive(ctx); err != nil {
		t.Fatalf("Failure when waiting for an LRS stream to be opened: %v", err)
	}

	// Configure a new resource on the management server with drop config that
	// drops all RPCs, but with no change in the load reporting server config.
	resources.Endpoints = []*v3endpointpb.ClusterLoadAssignment{
		e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
			ClusterName: "endpoints-" + serviceName,
			Host:        "localhost",
			Localities: []e2e.LocalityOptions{
				{
					Backends: []e2e.BackendOptions{{Port: testutils.ParsePort(t, server.Address)}},
					Weight:   1,
				},
			},
			DropPercents: map[string]int{"test-drop-everything": 100},
		}),
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Repeatedly send RPCs until we sees that they are getting dropped, or the
	// test context deadline expires. The former indicates that new config with
	// drops has been applied.
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		_, err := client.EmptyCall(ctx, &testpb.Empty{})
		if err != nil && status.Code(err) == codes.Unavailable && strings.Contains(err.Error(), "RPC is dropped") {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatalf("Timeout when waiting for RPCs to be dropped after config update")
	}

	// Ensure that the old LRS stream is not closed.
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := mgmtServer.LRSServer.LRSStreamCloseChan.Receive(sCtx); err == nil {
		t.Fatal("LRS stream closed when expected not to")
	}

	// Also ensure that a new LRS stream is not created.
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if _, err := mgmtServer.LRSServer.LRSStreamOpenChan.Receive(sCtx); err == nil {
		t.Fatal("New LRS stream created when expected not to")
	}
}

// Tests whether load is reported correctly when using pickfirst with endpoints
// in multiple localities.
func (s) TestLoadReportingPickFirstMultiLocality(t *testing.T) {
	// Create an xDS management server that serves ADS and LRS requests.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{SupportLoadReportingService: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create an xDS resolver with the above bootstrap configuration.
	var resolverBuilder resolver.Builder
	var err error
	if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
		resolverBuilder, err = newResolver.(func([]byte) (resolver.Builder, error))(bc)
		if err != nil {
			t.Fatalf("Failed to create xDS resolver for testing: %v", err)
		}
	}

	// Start two server backends exposing the test service.
	server1 := stubserver.StartTestService(t, nil)
	defer server1.Stop()

	server2 := stubserver.StartTestService(t, nil)
	defer server2.Stop()

	// Configure the xDS management server.
	const serviceName = "my-test-xds-service"
	routeConfigName := "route-" + serviceName
	clusterName := "cluster-" + serviceName
	endpointsName := "endpoints-" + serviceName
	resources := e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*v3listenerpb.Listener{e2e.DefaultClientListener(serviceName, routeConfigName)},
		Routes:    []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(routeConfigName, serviceName, clusterName)},
		Clusters: []*v3clusterpb.Cluster{
			{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: endpointsName,
				},
				// Specify a custom load balancing policy to use pickfirst.
				LoadBalancingPolicy: &v3clusterpb.LoadBalancingPolicy{
					Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
						{
							TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
								TypedConfig: testutils.MarshalAny(t, &v3pickfirstpb.PickFirst{}),
							},
						},
					},
				},
				// Include a fake LRS server config pointing to self.
				LrsServer: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
						Self: &v3corepb.SelfConfigSource{},
					},
				},
			},
		},
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
			ClusterName: endpointsName,
			Host:        "localhost",
			Localities: []e2e.LocalityOptions{
				{
					Backends: []e2e.BackendOptions{
						{Port: testutils.ParsePort(t, server1.Address)},
					},
					Weight: 1,
				},
				{
					Backends: []e2e.BackendOptions{
						{Port: testutils.ParsePort(t, server2.Address)},
					},
					Weight: 2,
				},
			},
		})},
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithResolvers(resolverBuilder))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)
	var peer peer.Peer
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}

	// Verify that the request was sent to server 1.
	if got, want := peer.Addr.String(), server1.Address; got != want {
		t.Errorf("peer.Addr = %q, want = %q", got, want)
	}

	// Ensure that an LRS stream is created.
	if _, err = mgmtServer.LRSServer.LRSStreamOpenChan.Receive(ctx); err != nil {
		t.Fatalf("Failure when waiting for an LRS stream to be opened: %v", err)
	}

	// Handle the initial LRS request from the xDS client.
	if _, err = mgmtServer.LRSServer.LRSRequestChan.Receive(ctx); err != nil {
		t.Fatalf("Failure waiting for initial LRS request: %v", err)
	}

	resp := fakeserver.Response{
		Resp: &v3lrspb.LoadStatsResponse{
			SendAllClusters:       true,
			LoadReportingInterval: durationpb.New(10 * time.Millisecond),
		},
	}
	mgmtServer.LRSServer.LRSResponseChan <- &resp

	// Wait for load to be reported for locality of server 1.
	if err := waitForSuccessfulLoadReport(ctx, mgmtServer.LRSServer, "region-1"); err != nil {
		t.Fatalf("Server 1 did not receive load due to error: %v", err)
	}

	// Stop server 1 and send one more rpc. Now the request should go to server 2.
	server1.Stop()

	// Wait for the balancer to pick up the server state change.
	testutils.AwaitState(ctx, t, cc, connectivity.Idle)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}

	// Verify that the request was sent to server 2.
	if got, want := peer.Addr.String(), server2.Address; got != want {
		t.Errorf("peer.Addr = %q, want = %q", got, want)
	}

	// Wait for load to be reported for locality of server 2.
	if err := waitForSuccessfulLoadReport(ctx, mgmtServer.LRSServer, "region-2"); err != nil {
		t.Fatalf("Server 2 did not receive load due to error: %v", err)
	}
}

// waitForSuccessfulLoadReport waits for a successful request to be reported for
// the specified locality region.
func waitForSuccessfulLoadReport(ctx context.Context, lrsServer *fakeserver.Server, region string) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case req := <-lrsServer.LRSRequestChan.C:
			loadStats := req.(*fakeserver.Request).Req.(*v3lrspb.LoadStatsRequest)
			for _, load := range loadStats.ClusterStats {
				for _, locality := range load.UpstreamLocalityStats {
					if locality.TotalSuccessfulRequests > 0 && locality.Locality.Region == region {
						return nil
					}
				}
			}
		}
	}
}
