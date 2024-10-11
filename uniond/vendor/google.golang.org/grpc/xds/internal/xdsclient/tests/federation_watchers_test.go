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

package xdsclient_test

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
)

const testNonDefaultAuthority = "non-default-authority"

// setupForFederationWatchersTest spins up two management servers, one for the
// default (empty) authority and another for a non-default authority.
//
// Returns the management server associated with the non-default authority, the
// nodeID to use, and the xDS client.
func setupForFederationWatchersTest(t *testing.T) (*e2e.ManagementServer, string, xdsclient.XDSClient) {
	// Start a management server as the default authority.
	serverDefaultAuthority := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Start another management server as the other authority.
	serverNonDefaultAuthority := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bootstrapContents, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, serverDefaultAuthority.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		Authorities: map[string]json.RawMessage{
			testNonDefaultAuthority: []byte(fmt.Sprintf(`{
				"xds_servers": [{
					"server_uri": %q,
					"channel_creds": [{"type": "insecure"}]
				}]}`, serverNonDefaultAuthority.Address)),
		},
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}
	// Create an xDS client with the above bootstrap contents.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	t.Cleanup(close)
	return serverNonDefaultAuthority, nodeID, client
}

// TestFederation_ListenerResourceContextParamOrder covers the case of watching
// a Listener resource with the new style resource name and context parameters.
// The test registers watches for two resources which differ only in the order
// of context parameters in their URI. The server is configured to respond with
// a single resource with canonicalized context parameters. The test verifies
// that both watchers are notified.
func (s) TestFederation_ListenerResourceContextParamOrder(t *testing.T) {
	serverNonDefaultAuthority, nodeID, client := setupForFederationWatchersTest(t)

	var (
		// Two resource names only differ in context parameter order.
		resourceName1 = fmt.Sprintf("xdstp://%s/envoy.config.listener.v3.Listener/xdsclient-test-lds-resource?a=1&b=2", testNonDefaultAuthority)
		resourceName2 = fmt.Sprintf("xdstp://%s/envoy.config.listener.v3.Listener/xdsclient-test-lds-resource?b=2&a=1", testNonDefaultAuthority)
	)

	// Register two watches for listener resources with the same query string,
	// but context parameters in different order.
	lw1 := newListenerWatcher()
	ldsCancel1 := xdsresource.WatchListener(client, resourceName1, lw1)
	defer ldsCancel1()
	lw2 := newListenerWatcher()
	ldsCancel2 := xdsresource.WatchListener(client, resourceName2, lw2)
	defer ldsCancel2()

	// Configure the management server for the non-default authority to return a
	// single listener resource, corresponding to the watches registered above.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultClientListener(resourceName1, "rds-resource")},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := serverNonDefaultAuthority.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	wantUpdate := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: "rds-resource",
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	// Verify the contents of the received update.
	if err := verifyListenerUpdate(ctx, lw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestFederation_RouteConfigResourceContextParamOrder covers the case of
// watching a RouteConfiguration resource with the new style resource name and
// context parameters. The test registers watches for two resources which
// differ only in the order of context parameters in their URI. The server is
// configured to respond with a single resource with canonicalized context
// parameters. The test verifies that both watchers are notified.
func (s) TestFederation_RouteConfigResourceContextParamOrder(t *testing.T) {
	serverNonDefaultAuthority, nodeID, client := setupForFederationWatchersTest(t)

	var (
		// Two resource names only differ in context parameter order.
		resourceName1 = fmt.Sprintf("xdstp://%s/envoy.config.route.v3.RouteConfiguration/xdsclient-test-rds-resource?a=1&b=2", testNonDefaultAuthority)
		resourceName2 = fmt.Sprintf("xdstp://%s/envoy.config.route.v3.RouteConfiguration/xdsclient-test-rds-resource?b=2&a=1", testNonDefaultAuthority)
	)

	// Register two watches for route configuration resources with the same
	// query string, but context parameters in different order.
	rw1 := newRouteConfigWatcher()
	rdsCancel1 := xdsresource.WatchRouteConfig(client, resourceName1, rw1)
	defer rdsCancel1()
	rw2 := newRouteConfigWatcher()
	rdsCancel2 := xdsresource.WatchRouteConfig(client, resourceName2, rw2)
	defer rdsCancel2()

	// Configure the management server for the non-default authority to return a
	// single route config resource, corresponding to the watches registered.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Routes:         []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(resourceName1, "listener-resource", "cluster-resource")},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := serverNonDefaultAuthority.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	wantUpdate := routeConfigUpdateErrTuple{
		update: xdsresource.RouteConfigUpdate{
			VirtualHosts: []*xdsresource.VirtualHost{
				{
					Domains: []string{"listener-resource"},
					Routes: []*xdsresource.Route{
						{
							Prefix:           newStringP("/"),
							ActionType:       xdsresource.RouteActionRoute,
							WeightedClusters: map[string]xdsresource.WeightedCluster{"cluster-resource": {Weight: 100}},
						},
					},
				},
			},
		},
	}
	// Verify the contents of the received update.
	if err := verifyRouteConfigUpdate(ctx, rw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyRouteConfigUpdate(ctx, rw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestFederation_ClusterResourceContextParamOrder covers the case of watching a
// Cluster resource with the new style resource name and context parameters.
// The test registers watches for two resources which differ only in the order
// of context parameters in their URI. The server is configured to respond with
// a single resource with canonicalized context parameters. The test verifies
// that both watchers are notified.
func (s) TestFederation_ClusterResourceContextParamOrder(t *testing.T) {
	serverNonDefaultAuthority, nodeID, client := setupForFederationWatchersTest(t)

	var (
		// Two resource names only differ in context parameter order.
		resourceName1 = fmt.Sprintf("xdstp://%s/envoy.config.cluster.v3.Cluster/xdsclient-test-cds-resource?a=1&b=2", testNonDefaultAuthority)
		resourceName2 = fmt.Sprintf("xdstp://%s/envoy.config.cluster.v3.Cluster/xdsclient-test-cds-resource?b=2&a=1", testNonDefaultAuthority)
	)

	// Register two watches for cluster resources with the same query string,
	// but context parameters in different order.
	cw1 := newClusterWatcher()
	cdsCancel1 := xdsresource.WatchCluster(client, resourceName1, cw1)
	defer cdsCancel1()
	cw2 := newClusterWatcher()
	cdsCancel2 := xdsresource.WatchCluster(client, resourceName2, cw2)
	defer cdsCancel2()

	// Configure the management server for the non-default authority to return a
	// single cluster resource, corresponding to the watches registered.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(resourceName1, "eds-service-name", e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := serverNonDefaultAuthority.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	wantUpdate := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    "xdstp://non-default-authority/envoy.config.cluster.v3.Cluster/xdsclient-test-cds-resource?a=1&b=2",
			EDSServiceName: "eds-service-name",
		},
	}
	// Verify the contents of the received update.
	if err := verifyClusterUpdate(ctx, cw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestFederation_EndpointsResourceContextParamOrder covers the case of watching
// an Endpoints resource with the new style resource name and context parameters.
// The test registers watches for two resources which differ only in the order
// of context parameters in their URI. The server is configured to respond with
// a single resource with canonicalized context parameters. The test verifies
// that both watchers are notified.
func (s) TestFederation_EndpointsResourceContextParamOrder(t *testing.T) {
	serverNonDefaultAuthority, nodeID, client := setupForFederationWatchersTest(t)

	var (
		// Two resource names only differ in context parameter order.
		resourceName1 = fmt.Sprintf("xdstp://%s/envoy.config.endpoint.v3.ClusterLoadAssignment/xdsclient-test-eds-resource?a=1&b=2", testNonDefaultAuthority)
		resourceName2 = fmt.Sprintf("xdstp://%s/envoy.config.endpoint.v3.ClusterLoadAssignment/xdsclient-test-eds-resource?b=2&a=1", testNonDefaultAuthority)
	)

	// Register two watches for endpoint resources with the same query string,
	// but context parameters in different order.
	ew1 := newEndpointsWatcher()
	edsCancel1 := xdsresource.WatchEndpoints(client, resourceName1, ew1)
	defer edsCancel1()
	ew2 := newEndpointsWatcher()
	edsCancel2 := xdsresource.WatchEndpoints(client, resourceName2, ew2)
	defer edsCancel2()

	// Configure the management server for the non-default authority to return a
	// single endpoints resource, corresponding to the watches registered.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(resourceName1, "localhost", []uint32{666})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := serverNonDefaultAuthority.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	wantUpdate := endpointsUpdateErrTuple{
		update: xdsresource.EndpointsUpdate{
			Localities: []xdsresource.Locality{
				{
					Endpoints: []xdsresource.Endpoint{{Address: "localhost:666", Weight: 1}},
					Weight:    1,
					ID: internal.LocalityID{
						Region:  "region-1",
						Zone:    "zone-1",
						SubZone: "subzone-1",
					},
				},
			},
		},
	}
	// Verify the contents of the received update.
	if err := verifyEndpointsUpdate(ctx, ew1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyEndpointsUpdate(ctx, ew2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

func newStringP(s string) *string {
	return &s
}
