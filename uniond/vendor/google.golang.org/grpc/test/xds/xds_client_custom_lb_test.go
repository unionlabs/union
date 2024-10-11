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

package xds_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"google.golang.org/grpc"
	_ "google.golang.org/grpc/balancer/leastrequest"       // To register least_request
	_ "google.golang.org/grpc/balancer/weightedroundrobin" // To register weighted_round_robin
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/roundrobin"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/e2e/setup"
	"google.golang.org/grpc/resolver"

	v3xdsxdstypepb "github.com/cncf/xds/go/xds/type/v3"
	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3clientsideweightedroundrobinpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/client_side_weighted_round_robin/v3"
	v3leastrequestpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/least_request/v3"
	v3roundrobinpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/round_robin/v3"
	v3wrrlocalitypb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/wrr_locality/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

// wrrLocality is a helper that takes a proto message and returns a
// WrrLocalityProto with the proto message marshaled into a proto.Any as a
// child.
func wrrLocality(t *testing.T, m proto.Message) *v3wrrlocalitypb.WrrLocality {
	return &v3wrrlocalitypb.WrrLocality{
		EndpointPickingPolicy: &v3clusterpb.LoadBalancingPolicy{
			Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
				{
					TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
						TypedConfig: testutils.MarshalAny(t, m),
					},
				},
			},
		},
	}
}

// clusterWithLBConfiguration returns a cluster resource with the proto message
// passed Marshaled to an any and specified through the load_balancing_policy
// field.
func clusterWithLBConfiguration(t *testing.T, clusterName, edsServiceName string, secLevel e2e.SecurityLevel, m proto.Message) *v3clusterpb.Cluster {
	cluster := e2e.DefaultCluster(clusterName, edsServiceName, secLevel)
	cluster.LoadBalancingPolicy = &v3clusterpb.LoadBalancingPolicy{
		Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
			{
				TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
					TypedConfig: testutils.MarshalAny(t, m),
				},
			},
		},
	}
	return cluster
}

// TestWRRLocality tests RPC distribution across a scenario with 5 backends,
// with 2 backends in a locality with weight 1, and 3 backends in a second
// locality with weight 2. Through xDS, the test configures a
// wrr_locality_balancer with either a round robin or custom (specifying pick
// first) child load balancing policy, and asserts the correct distribution
// based on the locality weights and the endpoint picking policy specified.
func (s) TestWrrLocality(t *testing.T) {
	oldLeastRequestLBSupport := envconfig.LeastRequestLB
	envconfig.LeastRequestLB = true
	defer func() {
		envconfig.LeastRequestLB = oldLeastRequestLBSupport
	}()

	backend1 := stubserver.StartTestService(t, nil)
	port1 := testutils.ParsePort(t, backend1.Address)
	defer backend1.Stop()
	backend2 := stubserver.StartTestService(t, nil)
	port2 := testutils.ParsePort(t, backend2.Address)
	defer backend2.Stop()
	backend3 := stubserver.StartTestService(t, nil)
	port3 := testutils.ParsePort(t, backend3.Address)
	defer backend3.Stop()
	backend4 := stubserver.StartTestService(t, nil)
	port4 := testutils.ParsePort(t, backend4.Address)
	defer backend4.Stop()
	backend5 := stubserver.StartTestService(t, nil)
	port5 := testutils.ParsePort(t, backend5.Address)
	defer backend5.Stop()
	const serviceName = "my-service-client-side-xds"

	tests := []struct {
		name string
		// Configuration will be specified through load_balancing_policy field.
		wrrLocalityConfiguration *v3wrrlocalitypb.WrrLocality
		addressDistributionWant  []struct {
			addr  string
			count int
		}
	}{
		{
			name:                     "rr_child",
			wrrLocalityConfiguration: wrrLocality(t, &v3roundrobinpb.RoundRobin{}),
			// Each addresses expected probability is locality weight of
			// locality / total locality weights multiplied by 1 / number of
			// endpoints in each locality (due to round robin across endpoints
			// in a locality). Thus, address 1 and address 2 have 1/3 * 1/2
			// probability, and addresses 3 4 5 have 2/3 * 1/3 probability of
			// being routed to.
			addressDistributionWant: []struct {
				addr  string
				count int
			}{
				{addr: backend1.Address, count: 6},
				{addr: backend2.Address, count: 6},
				{addr: backend3.Address, count: 8},
				{addr: backend4.Address, count: 8},
				{addr: backend5.Address, count: 8},
			},
		},
		// This configures custom lb as the child of wrr_locality, which points
		// to our pick_first implementation. Thus, the expected distribution of
		// addresses is locality weight of locality / total locality weights as
		// the probability of picking the first backend within the locality
		// (e.g. Address 1 for locality 1, and Address 3 for locality 2).
		{
			name: "custom_lb_child_pick_first",
			wrrLocalityConfiguration: wrrLocality(t, &v3xdsxdstypepb.TypedStruct{
				TypeUrl: "type.googleapis.com/pick_first",
				Value:   &structpb.Struct{},
			}),
			addressDistributionWant: []struct {
				addr  string
				count int
			}{
				{addr: backend1.Address, count: 1},
				{addr: backend3.Address, count: 2},
			},
		},
		// Sanity check for weighted round robin. Don't need to test super
		// specific behaviors, as that is covered in unit tests. Set up weighted
		// round robin as the endpoint picking policy with per RPC load reports
		// enabled. Due the server not sending trailers with load reports, the
		// weighted round robin policy should essentially function as round
		// robin, and thus should have the same distribution as round robin
		// above.
		{
			name: "custom_lb_child_wrr/",
			wrrLocalityConfiguration: wrrLocality(t, &v3clientsideweightedroundrobinpb.ClientSideWeightedRoundRobin{
				EnableOobLoadReport: &wrapperspb.BoolValue{
					Value: false,
				},
				// BlackoutPeriod long enough to cause load report weights to
				// trigger in the scope of test case, but no load reports
				// configured anyway.
				BlackoutPeriod:          durationpb.New(10 * time.Second),
				WeightExpirationPeriod:  durationpb.New(10 * time.Second),
				WeightUpdatePeriod:      durationpb.New(time.Second),
				ErrorUtilizationPenalty: &wrapperspb.FloatValue{Value: 1},
			}),
			addressDistributionWant: []struct {
				addr  string
				count int
			}{
				{addr: backend1.Address, count: 6},
				{addr: backend2.Address, count: 6},
				{addr: backend3.Address, count: 8},
				{addr: backend4.Address, count: 8},
				{addr: backend5.Address, count: 8},
			},
		},
		{
			name: "custom_lb_least_request",
			wrrLocalityConfiguration: wrrLocality(t, &v3leastrequestpb.LeastRequest{
				ChoiceCount: wrapperspb.UInt32(2),
			}),
			// The test performs a Unary RPC, and blocks until the RPC returns,
			// and then makes the next Unary RPC. Thus, over iterations, no RPC
			// counts are present. This causes least request's randomness of
			// indexes to sample to converge onto a round robin distribution per
			// locality. Thus, expect the same distribution as round robin
			// above.
			addressDistributionWant: []struct {
				addr  string
				count int
			}{
				{addr: backend1.Address, count: 6},
				{addr: backend2.Address, count: 6},
				{addr: backend3.Address, count: 8},
				{addr: backend4.Address, count: 8},
				{addr: backend5.Address, count: 8},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			// Start an xDS management server.
			managementServer, nodeID, _, xdsResolver := setup.ManagementServerAndResolver(t)

			routeConfigName := "route-" + serviceName
			clusterName := "cluster-" + serviceName
			endpointsName := "endpoints-" + serviceName
			resources := e2e.UpdateOptions{
				NodeID:    nodeID,
				Listeners: []*v3listenerpb.Listener{e2e.DefaultClientListener(serviceName, routeConfigName)},
				Routes:    []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(routeConfigName, serviceName, clusterName)},
				Clusters:  []*v3clusterpb.Cluster{clusterWithLBConfiguration(t, clusterName, endpointsName, e2e.SecurityLevelNone, test.wrrLocalityConfiguration)},
				Endpoints: []*v3endpointpb.ClusterLoadAssignment{e2e.EndpointResourceWithOptions(e2e.EndpointOptions{
					ClusterName: endpointsName,
					Host:        "localhost",
					Localities: []e2e.LocalityOptions{
						{
							Backends: []e2e.BackendOptions{{Port: port1}, {Port: port2}},
							Weight:   1,
						},
						{
							Backends: []e2e.BackendOptions{{Port: port3}, {Port: port4}, {Port: port5}},
							Weight:   2,
						},
					},
				})},
			}

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := managementServer.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}

			cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsResolver))
			if err != nil {
				t.Fatalf("Failed to dial local test server: %v", err)
			}
			defer cc.Close()

			client := testgrpc.NewTestServiceClient(cc)
			var addrDistWant []resolver.Address
			for _, addrAndCount := range test.addressDistributionWant {
				for i := 0; i < addrAndCount.count; i++ {
					addrDistWant = append(addrDistWant, resolver.Address{Addr: addrAndCount.addr})
				}
			}
			if err := roundrobin.CheckWeightedRoundRobinRPCs(ctx, client, addrDistWant); err != nil {
				t.Fatalf("Error in expected round robin: %v", err)
			}
		})
	}
}
