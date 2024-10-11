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
 *
 */

package resolver_test

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/google/uuid"
	"google.golang.org/grpc/balancer"
	iresolver "google.golang.org/grpc/internal/resolver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/xds/internal/balancer/clustermanager"
	"google.golang.org/grpc/xds/internal/clusterspecifier"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
)

func init() {
	balancer.Register(cspBalancerBuilder{})
	clusterspecifier.Register(testClusterSpecifierPlugin{})
}

// cspBalancerBuilder is a no-op LB policy which is referenced by the
// testClusterSpecifierPlugin.
type cspBalancerBuilder struct{}

func (cspBalancerBuilder) Build(cc balancer.ClientConn, opts balancer.BuildOptions) balancer.Balancer {
	return nil
}

func (cspBalancerBuilder) Name() string {
	return "csp_experimental"
}

type cspBalancerConfig struct {
	serviceconfig.LoadBalancingConfig
	ArbitraryField string `json:"arbitrary_field"`
}

func (cspBalancerBuilder) ParseConfig(lbCfg json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	cfg := &cspBalancerConfig{}
	if err := json.Unmarshal(lbCfg, cfg); err != nil {
		return nil, err
	}
	return cfg, nil

}

// testClusterSpecifierPlugin is a test cluster specifier plugin which returns
// an LB policy configuration specifying the cspBalancer.
type testClusterSpecifierPlugin struct {
}

func (testClusterSpecifierPlugin) TypeURLs() []string {
	// The config for this plugin contains a wrapperspb.StringValue, and since
	// we marshal that proto as an Any proto, the type URL on the latter gets
	// set to "type.googleapis.com/google.protobuf.StringValue". If we wanted a
	// more descriptive type URL for this test plugin, we would have to define a
	// proto package with a message for the configuration. That would be
	// overkill for a test. Therefore, this seems to be an acceptable tradeoff.
	return []string{"type.googleapis.com/google.protobuf.StringValue"}
}

func (testClusterSpecifierPlugin) ParseClusterSpecifierConfig(cfg proto.Message) (clusterspecifier.BalancerConfig, error) {
	if cfg == nil {
		return nil, fmt.Errorf("testClusterSpecifierPlugin: nil configuration message provided")
	}
	anyp, ok := cfg.(*anypb.Any)
	if !ok {
		return nil, fmt.Errorf("testClusterSpecifierPlugin: error parsing config %v: got type %T, want *anypb.Any", cfg, cfg)
	}
	lbCfg := new(wrapperspb.StringValue)
	if err := anypb.UnmarshalTo(anyp, lbCfg, proto.UnmarshalOptions{}); err != nil {
		return nil, fmt.Errorf("testClusterSpecifierPlugin: error parsing config %v: %v", cfg, err)
	}
	return []map[string]any{{"csp_experimental": cspBalancerConfig{ArbitraryField: lbCfg.GetValue()}}}, nil
}

// TestResolverClusterSpecifierPlugin tests the case where a route configuration
// containing cluster specifier plugins is sent by the management server. The
// test verifies that the service config output by the resolver contains the LB
// policy specified by the cluster specifier plugin, and the config selector
// returns the cluster associated with the cluster specifier plugin.
//
// The test also verifies that a change in the cluster specifier plugin config
// result in appropriate change in the service config pushed by the resolver.
func (s) TestResolverClusterSpecifierPlugin(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
		RouteConfigName:              defaultTestRouteConfigName,
		ListenerName:                 defaultTestServiceName,
		ClusterSpecifierType:         e2e.RouteConfigClusterSpecifierTypeClusterSpecifierPlugin,
		ClusterSpecifierPluginName:   "cspA",
		ClusterSpecifierPluginConfig: testutils.MarshalAny(t, &wrapperspb.StringValue{Value: "anything"}),
	})}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Wait for an update from the resolver, and verify the service config.
	wantSC := `
 {
	 "loadBalancingConfig": [
		 {
		   "xds_cluster_manager_experimental": {
			 "children": {
			   "cluster_specifier_plugin:cspA": {
				 "childPolicy": [
				   {
					 "csp_experimental": {
					   "arbitrary_field": "anything"
					 }
				   }
				 ]
			   }
			 }
		   }
		 }
	   ]
 }`
	cs := verifyUpdateFromResolver(ctx, t, stateCh, wantSC)
	res, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}

	gotCluster := clustermanager.GetPickedClusterForTesting(res.Context)
	wantCluster := "cluster_specifier_plugin:cspA"
	if gotCluster != wantCluster {
		t.Fatalf("config selector returned cluster: %v, want: %v", gotCluster, wantCluster)
	}

	// Change the cluster specifier plugin configuration.
	routes = []*v3routepb.RouteConfiguration{e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
		RouteConfigName:              defaultTestRouteConfigName,
		ListenerName:                 defaultTestServiceName,
		ClusterSpecifierType:         e2e.RouteConfigClusterSpecifierTypeClusterSpecifierPlugin,
		ClusterSpecifierPluginName:   "cspA",
		ClusterSpecifierPluginConfig: testutils.MarshalAny(t, &wrapperspb.StringValue{Value: "changed"}),
	})}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Wait for an update from the resolver, and verify the service config.
	wantSC = `
 {
	 "loadBalancingConfig": [
		 {
		   "xds_cluster_manager_experimental": {
			 "children": {
			   "cluster_specifier_plugin:cspA": {
				 "childPolicy": [
				   {
					 "csp_experimental": {
					   "arbitrary_field": "changed"
					 }
				   }
				 ]
			   }
			 }
		   }
		 }
	   ]
 }`
	verifyUpdateFromResolver(ctx, t, stateCh, wantSC)
}

// TestXDSResolverDelayedOnCommittedCSP tests that cluster specifier plugins and
// their corresponding configurations remain in service config if RPCs are in
// flight.
func (s) TestXDSResolverDelayedOnCommittedCSP(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
		RouteConfigName:              defaultTestRouteConfigName,
		ListenerName:                 defaultTestServiceName,
		ClusterSpecifierType:         e2e.RouteConfigClusterSpecifierTypeClusterSpecifierPlugin,
		ClusterSpecifierPluginName:   "cspA",
		ClusterSpecifierPluginConfig: testutils.MarshalAny(t, &wrapperspb.StringValue{Value: "anythingA"}),
	})}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Wait for an update from the resolver, and verify the service config.
	wantSC := `
 {
	 "loadBalancingConfig": [
		 {
		   "xds_cluster_manager_experimental": {
			 "children": {
			   "cluster_specifier_plugin:cspA": {
				 "childPolicy": [
				   {
					 "csp_experimental": {
					   "arbitrary_field": "anythingA"
					 }
				   }
				 ]
			   }
			 }
		   }
		 }
	   ]
 }`
	cs := verifyUpdateFromResolver(ctx, t, stateCh, wantSC)

	resOld, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}

	gotCluster := clustermanager.GetPickedClusterForTesting(resOld.Context)
	wantCluster := "cluster_specifier_plugin:cspA"
	if gotCluster != wantCluster {
		t.Fatalf("config selector returned cluster: %v, want: %v", gotCluster, wantCluster)
	}

	// Delay resOld.OnCommitted(). As long as there are pending RPCs to removed
	// clusters, they still appear in the service config.

	// Change the cluster specifier plugin configuration.
	routes = []*v3routepb.RouteConfiguration{e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
		RouteConfigName:              defaultTestRouteConfigName,
		ListenerName:                 defaultTestServiceName,
		ClusterSpecifierType:         e2e.RouteConfigClusterSpecifierTypeClusterSpecifierPlugin,
		ClusterSpecifierPluginName:   "cspB",
		ClusterSpecifierPluginConfig: testutils.MarshalAny(t, &wrapperspb.StringValue{Value: "anythingB"}),
	})}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Wait for an update from the resolver, and verify the service config.
	wantSC = `
 {
	 "loadBalancingConfig": [
		 {
		   "xds_cluster_manager_experimental": {
			 "children": {
			   "cluster_specifier_plugin:cspA": {
				 "childPolicy": [
				   {
					 "csp_experimental": {
					   "arbitrary_field": "anythingA"
					 }
				   }
				 ]
			   },
			   "cluster_specifier_plugin:cspB": {
				 "childPolicy": [
				   {
					 "csp_experimental": {
					   "arbitrary_field": "anythingB"
					 }
				   }
				 ]
			   }
			 }
		   }
		 }
	   ]
 }`
	cs = verifyUpdateFromResolver(ctx, t, stateCh, wantSC)

	// Perform an RPC and ensure that it is routed to the new cluster.
	resNew, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}

	gotCluster = clustermanager.GetPickedClusterForTesting(resNew.Context)
	wantCluster = "cluster_specifier_plugin:cspB"
	if gotCluster != wantCluster {
		t.Fatalf("config selector returned cluster: %v, want: %v", gotCluster, wantCluster)
	}

	// Invoke resOld.OnCommitted; should lead to a service config update that deletes
	// cspA.
	resOld.OnCommitted()

	wantSC = `
 {
	 "loadBalancingConfig": [
		 {
		   "xds_cluster_manager_experimental": {
			 "children": {
			   "cluster_specifier_plugin:cspB": {
				 "childPolicy": [
				   {
					 "csp_experimental": {
					   "arbitrary_field": "anythingB"
					 }
				   }
				 ]
			   }
			 }
		   }
		 }
	   ]
 }`
	verifyUpdateFromResolver(ctx, t, stateCh, wantSC)
}
