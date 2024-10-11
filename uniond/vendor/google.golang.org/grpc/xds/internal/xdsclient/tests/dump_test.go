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

package xdsclient_test

import (
	"context"
	"encoding/json"
	"fmt"
	"slices"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"

	v3adminpb "github.com/envoyproxy/go-control-plane/envoy/admin/v3"
	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3routerpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/router/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3statuspb "github.com/envoyproxy/go-control-plane/envoy/service/status/v3"
	"github.com/envoyproxy/go-control-plane/pkg/wellknown"
)

func makeGenericXdsConfig(typeURL, name, version string, status v3adminpb.ClientResourceStatus, config *anypb.Any, failure *v3adminpb.UpdateFailureState) *v3statuspb.ClientConfig_GenericXdsConfig {
	return &v3statuspb.ClientConfig_GenericXdsConfig{
		TypeUrl:      typeURL,
		Name:         name,
		VersionInfo:  version,
		ClientStatus: status,
		XdsConfig:    config,
		ErrorState:   failure,
	}
}

func checkResourceDump(ctx context.Context, want *v3statuspb.ClientStatusResponse) error {
	var cmpOpts = cmp.Options{
		protocmp.Transform(),
		protocmp.IgnoreFields((*v3statuspb.ClientConfig_GenericXdsConfig)(nil), "last_updated"),
		protocmp.IgnoreFields((*v3adminpb.UpdateFailureState)(nil), "last_update_attempt", "details"),
	}

	var lastErr error
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		got := xdsclient.DumpResources()
		// Sort the client configs based on the `client_scope` field.
		slices.SortFunc(got.GetConfig(), func(a, b *v3statuspb.ClientConfig) int {
			return strings.Compare(a.ClientScope, b.ClientScope)
		})
		// Sort the resource configs based on the type_url and name fields.
		for _, cc := range got.GetConfig() {
			slices.SortFunc(cc.GetGenericXdsConfigs(), func(a, b *v3statuspb.ClientConfig_GenericXdsConfig) int {
				if strings.Compare(a.TypeUrl, b.TypeUrl) == 0 {
					return strings.Compare(a.Name, b.Name)
				}
				return strings.Compare(a.TypeUrl, b.TypeUrl)
			})
		}
		diff := cmp.Diff(want, got, cmpOpts)
		if diff == "" {
			return nil
		}
		lastErr = fmt.Errorf("received unexpected resource dump, diff (-got, +want):\n%s, got: %s\n want:%s", diff, pretty.ToJSON(got), pretty.ToJSON(want))
	}
	return fmt.Errorf("timeout when waiting for resource dump to reach expected state: %v", lastErr)
}

// Tests the scenario where there are multiple xDS clients talking to the same
// management server, and requesting the same set of resources. Verifies that
// under all circumstances, both xDS clients receive the same configuration from
// the server.
func (s) TestDumpResources_ManyToOne(t *testing.T) {
	// Initialize the xDS resources to be used in this test.
	ldsTargets := []string{"lds.target.good:0000", "lds.target.good:1111"}
	rdsTargets := []string{"route-config-0", "route-config-1"}
	cdsTargets := []string{"cluster-0", "cluster-1"}
	edsTargets := []string{"endpoints-0", "endpoints-1"}
	listeners := make([]*v3listenerpb.Listener, len(ldsTargets))
	listenerAnys := make([]*anypb.Any, len(ldsTargets))
	for i := range ldsTargets {
		listeners[i] = e2e.DefaultClientListener(ldsTargets[i], rdsTargets[i])
		listenerAnys[i] = testutils.MarshalAny(t, listeners[i])
	}
	routes := make([]*v3routepb.RouteConfiguration, len(rdsTargets))
	routeAnys := make([]*anypb.Any, len(rdsTargets))
	for i := range rdsTargets {
		routes[i] = e2e.DefaultRouteConfig(rdsTargets[i], ldsTargets[i], cdsTargets[i])
		routeAnys[i] = testutils.MarshalAny(t, routes[i])
	}
	clusters := make([]*v3clusterpb.Cluster, len(cdsTargets))
	clusterAnys := make([]*anypb.Any, len(cdsTargets))
	for i := range cdsTargets {
		clusters[i] = e2e.DefaultCluster(cdsTargets[i], edsTargets[i], e2e.SecurityLevelNone)
		clusterAnys[i] = testutils.MarshalAny(t, clusters[i])
	}
	endpoints := make([]*v3endpointpb.ClusterLoadAssignment, len(edsTargets))
	endpointAnys := make([]*anypb.Any, len(edsTargets))
	ips := []string{"0.0.0.0", "1.1.1.1"}
	ports := []uint32{123, 456}
	for i := range edsTargets {
		endpoints[i] = e2e.DefaultEndpoint(edsTargets[i], ips[i], ports[i:i+1])
		endpointAnys[i] = testutils.MarshalAny(t, endpoints[i])
	}

	// Spin up an xDS management server on a local port.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create two xDS clients with the above bootstrap contents.
	client1Name := t.Name() + "-1"
	client1, close1, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     client1Name,
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close1()
	client2Name := t.Name() + "-2"
	client2, close2, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     client2Name,
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close2()

	// Dump resources and expect empty configs.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantNode := &v3corepb.Node{
		Id:                   nodeID,
		UserAgentName:        "gRPC Go",
		UserAgentVersionType: &v3corepb.Node_UserAgentVersion{UserAgentVersion: grpc.Version},
		ClientFeatures:       []string{"envoy.lb.does_not_support_overprovisioning", "xds.config.resource-in-sotw"},
	}
	wantResp := &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:        wantNode,
				ClientScope: client1Name,
			},
			{
				Node:        wantNode,
				ClientScope: client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}

	// Register watches, dump resources and expect configs in requested state.
	for _, xdsC := range []xdsclient.XDSClient{client1, client2} {
		for _, target := range ldsTargets {
			xdsresource.WatchListener(xdsC, target, noopListenerWatcher{})
		}
		for _, target := range rdsTargets {
			xdsresource.WatchRouteConfig(xdsC, target, noopRouteConfigWatcher{})
		}
		for _, target := range cdsTargets {
			xdsresource.WatchCluster(xdsC, target, noopClusterWatcher{})
		}
		for _, target := range edsTargets {
			xdsresource.WatchEndpoints(xdsC, target, noopEndpointsWatcher{})
		}
	}
	wantConfigs := []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
	}
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       client1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}

	// Configure the resources on the management server.
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: listeners,
		Routes:    routes,
		Clusters:  clusters,
		Endpoints: endpoints,
	}); err != nil {
		t.Fatal(err)
	}

	// Dump resources and expect ACK configs.
	wantConfigs = []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, clusterAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, clusterAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, endpointAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, endpointAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, listenerAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, listenerAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, routeAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, routeAnys[1], nil),
	}
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       client1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}

	// Update the first resource of each type in the management server to a
	// value which is expected to be NACK'ed by the xDS client.
	listeners[0] = func() *v3listenerpb.Listener {
		hcm := testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
			HttpFilters: []*v3httppb.HttpFilter{e2e.HTTPFilter("router", &v3routerpb.Router{})},
		})
		return &v3listenerpb.Listener{
			Name:        ldsTargets[0],
			ApiListener: &v3listenerpb.ApiListener{ApiListener: hcm},
			FilterChains: []*v3listenerpb.FilterChain{{
				Name: "filter-chain-name",
				Filters: []*v3listenerpb.Filter{{
					Name:       wellknown.HTTPConnectionManager,
					ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: hcm},
				}},
			}},
		}
	}()
	routes[0].VirtualHosts = []*v3routepb.VirtualHost{{Routes: []*v3routepb.Route{{}}}}
	clusters[0].ClusterDiscoveryType = &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_STATIC}
	endpoints[0].Endpoints = []*v3endpointpb.LocalityLbEndpoints{{}}
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      listeners,
		Routes:         routes,
		Clusters:       clusters,
		Endpoints:      endpoints,
		SkipValidation: true,
	}); err != nil {
		t.Fatal(err)
	}

	wantConfigs = []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[0], "1", v3adminpb.ClientResourceStatus_NACKED, clusterAnys[0], &v3adminpb.UpdateFailureState{VersionInfo: "2"}),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[1], "2", v3adminpb.ClientResourceStatus_ACKED, clusterAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[0], "1", v3adminpb.ClientResourceStatus_NACKED, endpointAnys[0], &v3adminpb.UpdateFailureState{VersionInfo: "2"}),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[1], "2", v3adminpb.ClientResourceStatus_ACKED, endpointAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[0], "1", v3adminpb.ClientResourceStatus_NACKED, listenerAnys[0], &v3adminpb.UpdateFailureState{VersionInfo: "2"}),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[1], "2", v3adminpb.ClientResourceStatus_ACKED, listenerAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[0], "1", v3adminpb.ClientResourceStatus_NACKED, routeAnys[0], &v3adminpb.UpdateFailureState{VersionInfo: "2"}),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[1], "2", v3adminpb.ClientResourceStatus_ACKED, routeAnys[1], nil),
	}
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       client1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}
}

// Tests the scenario where there are multiple xDS client talking to different
// management server, and requesting different set of resources.
func (s) TestDumpResources_ManyToMany(t *testing.T) {
	// Initialize the xDS resources to be used in this test:
	// - The first xDS client watches old style resource names, and thereby
	//   requests these resources from the top-level xDS server.
	// - The second xDS client watches new style resource names with a non-empty
	//   authority, and thereby requests these resources from the server
	//   configuration for that authority.
	authority := strings.Join(strings.Split(t.Name(), "/"), "")
	ldsTargets := []string{
		"lds.target.good:0000",
		fmt.Sprintf("xdstp://%s/envoy.config.listener.v3.Listener/lds.targer.good:1111", authority),
	}
	rdsTargets := []string{
		"route-config-0",
		fmt.Sprintf("xdstp://%s/envoy.config.route.v3.RouteConfiguration/route-config-1", authority),
	}
	cdsTargets := []string{
		"cluster-0",
		fmt.Sprintf("xdstp://%s/envoy.config.cluster.v3.Cluster/cluster-1", authority),
	}
	edsTargets := []string{
		"endpoints-0",
		fmt.Sprintf("xdstp://%s/envoy.config.endpoint.v3.ClusterLoadAssignment/endpoints-1", authority),
	}
	listeners := make([]*v3listenerpb.Listener, len(ldsTargets))
	listenerAnys := make([]*anypb.Any, len(ldsTargets))
	for i := range ldsTargets {
		listeners[i] = e2e.DefaultClientListener(ldsTargets[i], rdsTargets[i])
		listenerAnys[i] = testutils.MarshalAny(t, listeners[i])
	}
	routes := make([]*v3routepb.RouteConfiguration, len(rdsTargets))
	routeAnys := make([]*anypb.Any, len(rdsTargets))
	for i := range rdsTargets {
		routes[i] = e2e.DefaultRouteConfig(rdsTargets[i], ldsTargets[i], cdsTargets[i])
		routeAnys[i] = testutils.MarshalAny(t, routes[i])
	}
	clusters := make([]*v3clusterpb.Cluster, len(cdsTargets))
	clusterAnys := make([]*anypb.Any, len(cdsTargets))
	for i := range cdsTargets {
		clusters[i] = e2e.DefaultCluster(cdsTargets[i], edsTargets[i], e2e.SecurityLevelNone)
		clusterAnys[i] = testutils.MarshalAny(t, clusters[i])
	}
	endpoints := make([]*v3endpointpb.ClusterLoadAssignment, len(edsTargets))
	endpointAnys := make([]*anypb.Any, len(edsTargets))
	ips := []string{"0.0.0.0", "1.1.1.1"}
	ports := []uint32{123, 456}
	for i := range edsTargets {
		endpoints[i] = e2e.DefaultEndpoint(edsTargets[i], ips[i], ports[i:i+1])
		endpointAnys[i] = testutils.MarshalAny(t, endpoints[i])
	}

	// Start two management servers.
	mgmtServer1 := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})
	mgmtServer2 := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// The first of the above management servers will be the top-level xDS
	// server in the bootstrap configuration, and the second will be the xDS
	// server corresponding to the test authority.
	nodeID := uuid.New().String()
	bc, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer1.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		Authorities: map[string]json.RawMessage{
			authority: []byte(fmt.Sprintf(`{
				"xds_servers": [{
					"server_uri": %q,
					"channel_creds": [{"type": "insecure"}]
				}]}`, mgmtServer2.Address)),
		},
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create two xDS clients with the above bootstrap contents.
	client1Name := t.Name() + "-1"
	client1, close1, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     client1Name,
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close1()
	client2Name := t.Name() + "-2"
	client2, close2, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     client2Name,
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close2()

	// Check the resource dump before configuring resources on the management server.
	// Dump resources and expect empty configs.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantNode := &v3corepb.Node{
		Id:                   nodeID,
		UserAgentName:        "gRPC Go",
		UserAgentVersionType: &v3corepb.Node_UserAgentVersion{UserAgentVersion: grpc.Version},
		ClientFeatures:       []string{"envoy.lb.does_not_support_overprovisioning", "xds.config.resource-in-sotw"},
	}
	wantResp := &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:        wantNode,
				ClientScope: client1Name,
			},
			{
				Node:        wantNode,
				ClientScope: client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}

	// Register watches, the first xDS client watches old style resource names,
	// while the second xDS client watches new style resource names.
	xdsresource.WatchListener(client1, ldsTargets[0], noopListenerWatcher{})
	xdsresource.WatchRouteConfig(client1, rdsTargets[0], noopRouteConfigWatcher{})
	xdsresource.WatchCluster(client1, cdsTargets[0], noopClusterWatcher{})
	xdsresource.WatchEndpoints(client1, edsTargets[0], noopEndpointsWatcher{})
	xdsresource.WatchListener(client2, ldsTargets[1], noopListenerWatcher{})
	xdsresource.WatchRouteConfig(client2, rdsTargets[1], noopRouteConfigWatcher{})
	xdsresource.WatchCluster(client2, cdsTargets[1], noopClusterWatcher{})
	xdsresource.WatchEndpoints(client2, edsTargets[1], noopEndpointsWatcher{})

	// Check the resource dump. Both clients should have all resources in
	// REQUESTED state.
	wantConfigs1 := []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[0], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
	}
	wantConfigs2 := []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[1], "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
	}
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs1,
				ClientScope:       client1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs2,
				ClientScope:       client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}

	// Configure resources on the first management server.
	if err := mgmtServer1.Update(ctx, e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: listeners[:1],
		Routes:    routes[:1],
		Clusters:  clusters[:1],
		Endpoints: endpoints[:1],
	}); err != nil {
		t.Fatal(err)
	}

	// Check the resource dump. One client should have resources in ACKED state,
	// while the other should still have resources in REQUESTED state.
	wantConfigs1 = []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, clusterAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, endpointAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, listenerAnys[0], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[0], "1", v3adminpb.ClientResourceStatus_ACKED, routeAnys[0], nil),
	}
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs1,
				ClientScope:       client1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs2,
				ClientScope:       client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}

	// Configure resources on the second management server.
	if err := mgmtServer2.Update(ctx, e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: listeners[1:],
		Routes:    routes[1:],
		Clusters:  clusters[1:],
		Endpoints: endpoints[1:],
	}); err != nil {
		t.Fatal(err)
	}

	// Check the resource dump. Both clients should have appropriate resources
	// in REQUESTED state.
	wantConfigs2 = []*v3statuspb.ClientConfig_GenericXdsConfig{
		makeGenericXdsConfig("type.googleapis.com/envoy.config.cluster.v3.Cluster", cdsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, clusterAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment", edsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, endpointAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, listenerAnys[1], nil),
		makeGenericXdsConfig("type.googleapis.com/envoy.config.route.v3.RouteConfiguration", rdsTargets[1], "1", v3adminpb.ClientResourceStatus_ACKED, routeAnys[1], nil),
	}
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs1,
				ClientScope:       client1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs2,
				ClientScope:       client2Name,
			},
		},
	}
	if err := checkResourceDump(ctx, wantResp); err != nil {
		t.Fatal(err)
	}
}
