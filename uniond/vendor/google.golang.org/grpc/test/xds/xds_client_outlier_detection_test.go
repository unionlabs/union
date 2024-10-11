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

package xds_test

import (
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/e2e/setup"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

// TestOutlierDetection_NoopConfig tests the scenario where the Outlier
// Detection feature is enabled on the gRPC client, but it receives no Outlier
// Detection configuration from the management server. This should result in a
// no-op Outlier Detection configuration being used to configure the Outlier
// Detection balancer. This test verifies that an RPC is able to proceed
// normally with this configuration.
func (s) TestOutlierDetection_NoopConfig(t *testing.T) {
	managementServer, nodeID, _, xdsResolver := setup.ManagementServerAndResolver(t)

	server := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	server.StartServer()
	t.Logf("Started test service backend at %q", server.Address)
	defer server.Stop()

	const serviceName = "my-service-client-side-xds"
	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: serviceName,
		NodeID:     nodeID,
		Host:       "localhost",
		Port:       testutils.ParsePort(t, server.Address),
		SecLevel:   e2e.SecurityLevelNone,
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsResolver))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}
}

// clientResourcesMultipleBackendsAndOD returns xDS resources which correspond
// to multiple upstreams, corresponding different backends listening on
// different localhost:port combinations. The resources also configure an
// Outlier Detection Balancer configured through the passed in Outlier Detection
// proto.
func clientResourcesMultipleBackendsAndOD(params e2e.ResourceParams, ports []uint32, od *v3clusterpb.OutlierDetection) e2e.UpdateOptions {
	routeConfigName := "route-" + params.DialTarget
	clusterName := "cluster-" + params.DialTarget
	endpointsName := "endpoints-" + params.DialTarget
	return e2e.UpdateOptions{
		NodeID:    params.NodeID,
		Listeners: []*v3listenerpb.Listener{e2e.DefaultClientListener(params.DialTarget, routeConfigName)},
		Routes:    []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(routeConfigName, params.DialTarget, clusterName)},
		Clusters:  []*v3clusterpb.Cluster{clusterWithOutlierDetection(clusterName, endpointsName, params.SecLevel, od)},
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(endpointsName, params.Host, ports)},
	}
}

func clusterWithOutlierDetection(clusterName, edsServiceName string, secLevel e2e.SecurityLevel, od *v3clusterpb.OutlierDetection) *v3clusterpb.Cluster {
	cluster := e2e.DefaultCluster(clusterName, edsServiceName, secLevel)
	cluster.OutlierDetection = od
	return cluster
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
	for ; ctx.Err() == nil; <-time.After(time.Millisecond) {
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
		gotAddrCount := make(map[string]int)
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
	return fmt.Errorf("timeout when waiting for roundrobin distribution of RPCs across addresses: %v", addrs)
}

// TestOutlierDetectionWithOutlier tests the Outlier Detection Balancer e2e. It
// spins up three backends, one which consistently errors, and configures the
// ClientConn using xDS to connect to all three of those backends. The Outlier
// Detection Balancer should eject the connection to the backend which
// constantly errors, causing RPC's to not be routed to that upstream, and only
// be Round Robined across the two healthy upstreams. Other than the intervals
// the unhealthy upstream is ejected, RPC's should regularly round robin across
// all three upstreams.
func (s) TestOutlierDetectionWithOutlier(t *testing.T) {
	managementServer, nodeID, _, xdsResolver := setup.ManagementServerAndResolver(t)

	// Working backend 1.
	backend1 := stubserver.StartTestService(t, nil)
	port1 := testutils.ParsePort(t, backend1.Address)
	defer backend1.Stop()

	// Working backend 2.
	backend2 := stubserver.StartTestService(t, nil)
	port2 := testutils.ParsePort(t, backend2.Address)
	defer backend2.Stop()

	// Backend 3 that will always return an error and eventually ejected.
	backend3 := stubserver.StartTestService(t, &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return nil, errors.New("some error") },
	})
	port3 := testutils.ParsePort(t, backend3.Address)
	defer backend3.Stop()

	const serviceName = "my-service-client-side-xds"
	resources := clientResourcesMultipleBackendsAndOD(e2e.ResourceParams{
		DialTarget: serviceName,
		NodeID:     nodeID,
		Host:       "localhost",
		SecLevel:   e2e.SecurityLevelNone,
	}, []uint32{port1, port2, port3}, &v3clusterpb.OutlierDetection{
		Interval:                       &durationpb.Duration{Nanos: 50000000}, // .5 seconds
		BaseEjectionTime:               &durationpb.Duration{Seconds: 30},
		MaxEjectionTime:                &durationpb.Duration{Seconds: 300},
		MaxEjectionPercent:             &wrapperspb.UInt32Value{Value: 1},
		FailurePercentageThreshold:     &wrapperspb.UInt32Value{Value: 50},
		EnforcingFailurePercentage:     &wrapperspb.UInt32Value{Value: 100},
		FailurePercentageRequestVolume: &wrapperspb.UInt32Value{Value: 8},
		FailurePercentageMinimumHosts:  &wrapperspb.UInt32Value{Value: 3},
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsResolver))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)

	fullAddresses := []resolver.Address{
		{Addr: backend1.Address},
		{Addr: backend2.Address},
		{Addr: backend3.Address},
	}
	// At first, due to no statistics on each of the backends, the 3
	// upstreams should all be round robined across.
	if err = checkRoundRobinRPCs(ctx, client, fullAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}

	// The addresses which don't return errors.
	okAddresses := []resolver.Address{
		{Addr: backend1.Address},
		{Addr: backend2.Address},
	}
	// After calling the three upstreams, one of them constantly error
	// and should eventually be ejected for a period of time. This
	// period of time should cause the RPC's to be round robined only
	// across the two that are healthy.
	if err = checkRoundRobinRPCs(ctx, client, okAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}
}

// TestOutlierDetectionXDSDefaultOn tests that Outlier Detection is by default
// configured on in the xDS Flow. If the Outlier Detection proto message is
// present with SuccessRateEjection unset, then Outlier Detection should be
// turned on. The test setups and xDS system with xDS resources with Outlier
// Detection present in the CDS update, but with SuccessRateEjection unset, and
// asserts that Outlier Detection is turned on and ejects upstreams.
func (s) TestOutlierDetectionXDSDefaultOn(t *testing.T) {
	managementServer, nodeID, _, xdsResolver := setup.ManagementServerAndResolver(t)

	// Working backend 1.
	backend1 := stubserver.StartTestService(t, nil)
	port1 := testutils.ParsePort(t, backend1.Address)
	defer backend1.Stop()

	// Working backend 2.
	backend2 := stubserver.StartTestService(t, nil)
	port2 := testutils.ParsePort(t, backend2.Address)
	defer backend2.Stop()

	// Backend 3 that will always return an error and eventually ejected.
	backend3 := stubserver.StartTestService(t, &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return nil, errors.New("some error") },
	})
	port3 := testutils.ParsePort(t, backend3.Address)
	defer backend3.Stop()

	// Configure CDS resources with Outlier Detection set but
	// EnforcingSuccessRate unset. This should cause Outlier Detection to be
	// configured with SuccessRateEjection present in configuration, which will
	// eventually be populated with its default values along with the knobs set
	// as SuccessRate fields in the proto, and thus Outlier Detection should be
	// on and actively eject upstreams.
	const serviceName = "my-service-client-side-xds"
	resources := clientResourcesMultipleBackendsAndOD(e2e.ResourceParams{
		DialTarget: serviceName,
		NodeID:     nodeID,
		Host:       "localhost",
		SecLevel:   e2e.SecurityLevelNone,
	}, []uint32{port1, port2, port3}, &v3clusterpb.OutlierDetection{
		// Need to set knobs to trigger ejection within the test time frame.
		Interval: &durationpb.Duration{Nanos: 50000000},
		// EnforcingSuccessRateSet to nil, causes success rate algorithm to be
		// turned on.
		SuccessRateMinimumHosts:  &wrapperspb.UInt32Value{Value: 1},
		SuccessRateRequestVolume: &wrapperspb.UInt32Value{Value: 8},
		SuccessRateStdevFactor:   &wrapperspb.UInt32Value{Value: 1},
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsResolver))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)

	fullAddresses := []resolver.Address{
		{Addr: backend1.Address},
		{Addr: backend2.Address},
		{Addr: backend3.Address},
	}
	// At first, due to no statistics on each of the backends, the 3
	// upstreams should all be round robined across.
	if err = checkRoundRobinRPCs(ctx, client, fullAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}

	// The addresses which don't return errors.
	okAddresses := []resolver.Address{
		{Addr: backend1.Address},
		{Addr: backend2.Address},
	}
	// After calling the three upstreams, one of them constantly error
	// and should eventually be ejected for a period of time. This
	// period of time should cause the RPC's to be round robined only
	// across the two that are healthy.
	if err = checkRoundRobinRPCs(ctx, client, okAddresses); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}
}
