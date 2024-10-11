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
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
)

type noopRouteConfigWatcher struct{}

func (noopRouteConfigWatcher) OnUpdate(update *xdsresource.RouteConfigResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (noopRouteConfigWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (noopRouteConfigWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

type routeConfigUpdateErrTuple struct {
	update xdsresource.RouteConfigUpdate
	err    error
}

type routeConfigWatcher struct {
	updateCh *testutils.Channel
}

func newRouteConfigWatcher() *routeConfigWatcher {
	return &routeConfigWatcher{updateCh: testutils.NewChannel()}
}

func (rw *routeConfigWatcher) OnUpdate(update *xdsresource.RouteConfigResourceData, onDone xdsresource.OnDoneFunc) {
	rw.updateCh.Send(routeConfigUpdateErrTuple{update: update.Resource})
	onDone()
}

func (rw *routeConfigWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	// When used with a go-control-plane management server that continuously
	// resends resources which are NACKed by the xDS client, using a `Replace()`
	// here and in OnResourceDoesNotExist() simplifies tests which will have
	// access to the most recently received error.
	rw.updateCh.Replace(routeConfigUpdateErrTuple{err: err})
	onDone()
}

func (rw *routeConfigWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	rw.updateCh.Replace(routeConfigUpdateErrTuple{err: xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "RouteConfiguration not found in received response")})
	onDone()
}

// badRouteConfigResource returns a RouteConfiguration resource for the given
// routeName which contains a retry config with num_retries set to `0`. This is
// expected to be NACK'ed by the xDS client.
func badRouteConfigResource(routeName, ldsTarget, clusterName string) *v3routepb.RouteConfiguration {
	return &v3routepb.RouteConfiguration{
		Name: routeName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			Domains: []string{ldsTarget},
			Routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
				Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
					ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
				}}}},
			RetryPolicy: &v3routepb.RetryPolicy{
				NumRetries: &wrapperspb.UInt32Value{Value: 0},
			},
		}},
	}
}

// xdsClient is expected to produce an error containing this string when an
// update is received containing a route configuration resource created using
// `badRouteConfigResource`.
const wantRouteConfigNACKErr = "received route is invalid: retry_policy.num_retries = 0; must be >= 1"

// verifyRouteConfigUpdate waits for an update to be received on the provided
// update channel and verifies that it matches the expected update.
//
// Returns an error if no update is received before the context deadline expires
// or the received update does not match the expected one.
func verifyRouteConfigUpdate(ctx context.Context, updateCh *testutils.Channel, wantUpdate routeConfigUpdateErrTuple) error {
	u, err := updateCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("timeout when waiting for a route configuration resource from the management server: %v", err)
	}
	got := u.(routeConfigUpdateErrTuple)
	if wantUpdate.err != nil {
		if gotType, wantType := xdsresource.ErrType(got.err), xdsresource.ErrType(wantUpdate.err); gotType != wantType {
			return fmt.Errorf("received update with error type %v, want %v", gotType, wantType)
		}
	}
	cmpOpts := []cmp.Option{cmpopts.EquateEmpty(), cmpopts.IgnoreFields(xdsresource.RouteConfigUpdate{}, "Raw")}
	if diff := cmp.Diff(wantUpdate.update, got.update, cmpOpts...); diff != "" {
		return fmt.Errorf("received unexpected diff in the route configuration resource update: (-want, got):\n%s", diff)
	}
	return nil
}

// verifyNoRouteConfigUpdate verifies that no route configuration update is
// received on the provided update channel, and returns an error if an update is
// received.
//
// A very short deadline is used while waiting for the update, as this function
// is intended to be used when an update is not expected.
func verifyNoRouteConfigUpdate(ctx context.Context, updateCh *testutils.Channel) error {
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if u, err := updateCh.Receive(sCtx); err != context.DeadlineExceeded {
		return fmt.Errorf("unexpected RouteConfigUpdate: %v", u)
	}
	return nil
}

// TestRDSWatch covers the case where a single watcher exists for a single route
// configuration resource. The test verifies the following scenarios:
//  1. An update from the management server containing the resource being
//     watched should result in the invocation of the watch callback.
//  2. An update from the management server containing a resource *not* being
//     watched should not result in the invocation of the watch callback.
//  3. After the watch is cancelled, an update from the management server
//     containing the resource that was being watched should not result in the
//     invocation of the watch callback.
//
// The test is run for old and new style names.
func (s) TestRDSWatch(t *testing.T) {
	tests := []struct {
		desc                   string
		resourceName           string
		watchedResource        *v3routepb.RouteConfiguration // The resource being watched.
		updatedWatchedResource *v3routepb.RouteConfiguration // The watched resource after an update.
		notWatchedResource     *v3routepb.RouteConfiguration // A resource which is not being watched.
		wantUpdate             routeConfigUpdateErrTuple
	}{
		{
			desc:                   "old style resource",
			resourceName:           rdsName,
			watchedResource:        e2e.DefaultRouteConfig(rdsName, ldsName, cdsName),
			updatedWatchedResource: e2e.DefaultRouteConfig(rdsName, ldsName, "new-cds-resource"),
			notWatchedResource:     e2e.DefaultRouteConfig("unsubscribed-rds-resource", ldsName, cdsName),
			wantUpdate: routeConfigUpdateErrTuple{
				update: xdsresource.RouteConfigUpdate{
					VirtualHosts: []*xdsresource.VirtualHost{
						{
							Domains: []string{ldsName},
							Routes: []*xdsresource.Route{
								{
									Prefix:           newStringP("/"),
									ActionType:       xdsresource.RouteActionRoute,
									WeightedClusters: map[string]xdsresource.WeightedCluster{cdsName: {Weight: 100}},
								},
							},
						},
					},
				},
			},
		},
		{
			desc:                   "new style resource",
			resourceName:           rdsNameNewStyle,
			watchedResource:        e2e.DefaultRouteConfig(rdsNameNewStyle, ldsNameNewStyle, cdsNameNewStyle),
			updatedWatchedResource: e2e.DefaultRouteConfig(rdsNameNewStyle, ldsNameNewStyle, "new-cds-resource"),
			notWatchedResource:     e2e.DefaultRouteConfig("unsubscribed-rds-resource", ldsNameNewStyle, cdsNameNewStyle),
			wantUpdate: routeConfigUpdateErrTuple{
				update: xdsresource.RouteConfigUpdate{
					VirtualHosts: []*xdsresource.VirtualHost{
						{
							Domains: []string{ldsNameNewStyle},
							Routes: []*xdsresource.Route{
								{
									Prefix:           newStringP("/"),
									ActionType:       xdsresource.RouteActionRoute,
									WeightedClusters: map[string]xdsresource.WeightedCluster{cdsNameNewStyle: {Weight: 100}},
								},
							},
						},
					},
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

			nodeID := uuid.New().String()
			bc, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
				Servers: []byte(fmt.Sprintf(`[{
					"server_uri": %q,
					"channel_creds": [{"type": "insecure"}]
				}]`, mgmtServer.Address)),
				Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
				Authorities: map[string]json.RawMessage{
					// Xdstp resource names used in this test do not specify an
					// authority. These will end up looking up an entry with the
					// empty key in the authorities map. Having an entry with an
					// empty key and empty configuration, results in these
					// resources also using the top-level configuration.
					"": []byte(`{}`),
				},
			})
			if err != nil {
				t.Fatalf("Failed to create bootstrap configuration: %v", err)
			}
			testutils.CreateBootstrapFileForTesting(t, bc)

			// Create an xDS client with the above bootstrap contents.
			client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
				Name:     t.Name(),
				Contents: bc,
			})
			if err != nil {
				t.Fatalf("Failed to create xDS client: %v", err)
			}
			defer close()

			// Register a watch for a route configuration resource and have the
			// watch callback push the received update on to a channel.
			rw := newRouteConfigWatcher()
			rdsCancel := xdsresource.WatchRouteConfig(client, test.resourceName, rw)

			// Configure the management server to return a single route
			// configuration resource, corresponding to the one being watched.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Routes:         []*v3routepb.RouteConfiguration{test.watchedResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}

			// Verify the contents of the received update.
			if err := verifyRouteConfigUpdate(ctx, rw.updateCh, test.wantUpdate); err != nil {
				t.Fatal(err)
			}

			// Configure the management server to return an additional route
			// configuration resource, one that we are not interested in.
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Routes:         []*v3routepb.RouteConfiguration{test.watchedResource, test.notWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoRouteConfigUpdate(ctx, rw.updateCh); err != nil {
				t.Fatal(err)
			}

			// Cancel the watch and update the resource corresponding to the original
			// watch.  Ensure that the cancelled watch callback is not invoked.
			rdsCancel()
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Routes:         []*v3routepb.RouteConfiguration{test.updatedWatchedResource, test.notWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoRouteConfigUpdate(ctx, rw.updateCh); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestRDSWatch_TwoWatchesForSameResourceName covers the case where two watchers
// exist for a single route configuration resource.  The test verifies the
// following scenarios:
//  1. An update from the management server containing the resource being
//     watched should result in the invocation of both watch callbacks.
//  2. After one of the watches is cancelled, a redundant update from the
//     management server should not result in the invocation of either of the
//     watch callbacks.
//  3. An update from the management server containing the resource being
//     watched should result in the invocation of the un-cancelled watch
//     callback.
//
// The test is run for old and new style names.
func (s) TestRDSWatch_TwoWatchesForSameResourceName(t *testing.T) {
	tests := []struct {
		desc                   string
		resourceName           string
		watchedResource        *v3routepb.RouteConfiguration // The resource being watched.
		updatedWatchedResource *v3routepb.RouteConfiguration // The watched resource after an update.
		wantUpdateV1           routeConfigUpdateErrTuple
		wantUpdateV2           routeConfigUpdateErrTuple
	}{
		{
			desc:                   "old style resource",
			resourceName:           rdsName,
			watchedResource:        e2e.DefaultRouteConfig(rdsName, ldsName, cdsName),
			updatedWatchedResource: e2e.DefaultRouteConfig(rdsName, ldsName, "new-cds-resource"),
			wantUpdateV1: routeConfigUpdateErrTuple{
				update: xdsresource.RouteConfigUpdate{
					VirtualHosts: []*xdsresource.VirtualHost{
						{
							Domains: []string{ldsName},
							Routes: []*xdsresource.Route{
								{
									Prefix:           newStringP("/"),
									ActionType:       xdsresource.RouteActionRoute,
									WeightedClusters: map[string]xdsresource.WeightedCluster{cdsName: {Weight: 100}},
								},
							},
						},
					},
				},
			},
			wantUpdateV2: routeConfigUpdateErrTuple{
				update: xdsresource.RouteConfigUpdate{
					VirtualHosts: []*xdsresource.VirtualHost{
						{
							Domains: []string{ldsName},
							Routes: []*xdsresource.Route{
								{
									Prefix:           newStringP("/"),
									ActionType:       xdsresource.RouteActionRoute,
									WeightedClusters: map[string]xdsresource.WeightedCluster{"new-cds-resource": {Weight: 100}},
								},
							},
						},
					},
				},
			},
		},
		{
			desc:                   "new style resource",
			resourceName:           rdsNameNewStyle,
			watchedResource:        e2e.DefaultRouteConfig(rdsNameNewStyle, ldsNameNewStyle, cdsNameNewStyle),
			updatedWatchedResource: e2e.DefaultRouteConfig(rdsNameNewStyle, ldsNameNewStyle, "new-cds-resource"),
			wantUpdateV1: routeConfigUpdateErrTuple{
				update: xdsresource.RouteConfigUpdate{
					VirtualHosts: []*xdsresource.VirtualHost{
						{
							Domains: []string{ldsNameNewStyle},
							Routes: []*xdsresource.Route{
								{
									Prefix:           newStringP("/"),
									ActionType:       xdsresource.RouteActionRoute,
									WeightedClusters: map[string]xdsresource.WeightedCluster{cdsNameNewStyle: {Weight: 100}},
								},
							},
						},
					},
				},
			},
			wantUpdateV2: routeConfigUpdateErrTuple{
				update: xdsresource.RouteConfigUpdate{
					VirtualHosts: []*xdsresource.VirtualHost{
						{
							Domains: []string{ldsNameNewStyle},
							Routes: []*xdsresource.Route{
								{
									Prefix:           newStringP("/"),
									ActionType:       xdsresource.RouteActionRoute,
									WeightedClusters: map[string]xdsresource.WeightedCluster{"new-cds-resource": {Weight: 100}},
								},
							},
						},
					},
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

			nodeID := uuid.New().String()
			bc, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
				Servers: []byte(fmt.Sprintf(`[{
					"server_uri": %q,
					"channel_creds": [{"type": "insecure"}]
				}]`, mgmtServer.Address)),
				Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
				Authorities: map[string]json.RawMessage{
					// Xdstp resource names used in this test do not specify an
					// authority. These will end up looking up an entry with the
					// empty key in the authorities map. Having an entry with an
					// empty key and empty configuration, results in these
					// resources also using the top-level configuration.
					"": []byte(`{}`),
				},
			})
			if err != nil {
				t.Fatalf("Failed to create bootstrap configuration: %v", err)
			}
			testutils.CreateBootstrapFileForTesting(t, bc)

			// Create an xDS client with the above bootstrap contents.
			client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
				Name:     t.Name(),
				Contents: bc,
			})
			if err != nil {
				t.Fatalf("Failed to create xDS client: %v", err)
			}
			defer close()

			// Register two watches for the same route configuration resource
			// and have the callbacks push the received updates on to a channel.
			rw1 := newRouteConfigWatcher()
			rdsCancel1 := xdsresource.WatchRouteConfig(client, test.resourceName, rw1)
			defer rdsCancel1()
			rw2 := newRouteConfigWatcher()
			rdsCancel2 := xdsresource.WatchRouteConfig(client, test.resourceName, rw2)

			// Configure the management server to return a single route
			// configuration resource, corresponding to the one being watched.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Routes:         []*v3routepb.RouteConfiguration{test.watchedResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}

			// Verify the contents of the received update.
			if err := verifyRouteConfigUpdate(ctx, rw1.updateCh, test.wantUpdateV1); err != nil {
				t.Fatal(err)
			}
			if err := verifyRouteConfigUpdate(ctx, rw2.updateCh, test.wantUpdateV1); err != nil {
				t.Fatal(err)
			}

			// Cancel the second watch and force the management server to push a
			// redundant update for the resource being watched. Neither of the
			// two watch callbacks should be invoked.
			rdsCancel2()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoRouteConfigUpdate(ctx, rw1.updateCh); err != nil {
				t.Fatal(err)
			}
			if err := verifyNoRouteConfigUpdate(ctx, rw2.updateCh); err != nil {
				t.Fatal(err)
			}

			// Update to the resource being watched. The un-cancelled callback
			// should be invoked while the cancelled one should not be.
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Routes:         []*v3routepb.RouteConfiguration{test.updatedWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyRouteConfigUpdate(ctx, rw1.updateCh, test.wantUpdateV2); err != nil {
				t.Fatal(err)
			}
			if err := verifyNoRouteConfigUpdate(ctx, rw2.updateCh); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestRDSWatch_ThreeWatchesForDifferentResourceNames covers the case with three
// watchers (two watchers for one resource, and the third watcher for another
// resource), exist across two route configuration resources.  The test verifies
// that an update from the management server containing both resources results
// in the invocation of all watch callbacks.
//
// The test is run with both old and new style names.
func (s) TestRDSWatch_ThreeWatchesForDifferentResourceNames(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	authority := makeAuthorityName(t.Name())
	bc, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		Authorities: map[string]json.RawMessage{
			// Xdstp style resource names used in this test use a slash removed
			// version of t.Name as their authority, and the empty config
			// results in the top-level xds server configuration being used for
			// this authority.
			authority: []byte(`{}`),
		},
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS client with the above bootstrap contents.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Register two watches for the same route configuration resource
	// and have the callbacks push the received updates on to a channel.
	rw1 := newRouteConfigWatcher()
	rdsCancel1 := xdsresource.WatchRouteConfig(client, rdsName, rw1)
	defer rdsCancel1()
	rw2 := newRouteConfigWatcher()
	rdsCancel2 := xdsresource.WatchRouteConfig(client, rdsName, rw2)
	defer rdsCancel2()

	// Register the third watch for a different route configuration resource.
	rdsNameNewStyle := makeNewStyleRDSName(authority)
	rw3 := newRouteConfigWatcher()
	rdsCancel3 := xdsresource.WatchRouteConfig(client, rdsNameNewStyle, rw3)
	defer rdsCancel3()

	// Configure the management server to return two route configuration
	// resources, corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Routes: []*v3routepb.RouteConfiguration{
			e2e.DefaultRouteConfig(rdsName, ldsName, cdsName),
			e2e.DefaultRouteConfig(rdsNameNewStyle, ldsName, cdsName),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for the all watchers. The two
	// resources returned differ only in the resource name. Therefore the
	// expected update is the same for all the watchers.
	wantUpdate := routeConfigUpdateErrTuple{
		update: xdsresource.RouteConfigUpdate{
			VirtualHosts: []*xdsresource.VirtualHost{
				{
					Domains: []string{ldsName},
					Routes: []*xdsresource.Route{
						{
							Prefix:           newStringP("/"),
							ActionType:       xdsresource.RouteActionRoute,
							WeightedClusters: map[string]xdsresource.WeightedCluster{cdsName: {Weight: 100}},
						},
					},
				},
			},
		},
	}
	if err := verifyRouteConfigUpdate(ctx, rw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyRouteConfigUpdate(ctx, rw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyRouteConfigUpdate(ctx, rw3.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestRDSWatch_ResourceCaching covers the case where a watch is registered for
// a resource which is already present in the cache.  The test verifies that the
// watch callback is invoked with the contents from the cache, instead of a
// request being sent to the management server.
func (s) TestRDSWatch_ResourceCaching(t *testing.T) {
	firstRequestReceived := false
	firstAckReceived := grpcsync.NewEvent()
	secondRequestReceived := grpcsync.NewEvent()

	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(id int64, req *v3discoverypb.DiscoveryRequest) error {
			// The first request has an empty version string.
			if !firstRequestReceived && req.GetVersionInfo() == "" {
				firstRequestReceived = true
				return nil
			}
			// The first ack has a non-empty version string.
			if !firstAckReceived.HasFired() && req.GetVersionInfo() != "" {
				firstAckReceived.Fire()
				return nil
			}
			// Any requests after the first request and ack, are not expected.
			secondRequestReceived.Fire()
			return nil
		},
	})

	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS client with the above bootstrap contents.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Register a watch for a route configuration resource and have the watch
	// callback push the received update on to a channel.
	rw1 := newRouteConfigWatcher()
	rdsCancel1 := xdsresource.WatchRouteConfig(client, rdsName, rw1)
	defer rdsCancel1()

	// Configure the management server to return a single route configuration
	// resource, corresponding to the one we registered a watch for.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Routes:         []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(rdsName, ldsName, cdsName)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update.
	wantUpdate := routeConfigUpdateErrTuple{
		update: xdsresource.RouteConfigUpdate{
			VirtualHosts: []*xdsresource.VirtualHost{
				{
					Domains: []string{ldsName},
					Routes: []*xdsresource.Route{
						{
							Prefix:           newStringP("/"),
							ActionType:       xdsresource.RouteActionRoute,
							WeightedClusters: map[string]xdsresource.WeightedCluster{cdsName: {Weight: 100}},
						},
					},
				},
			},
		},
	}
	if err := verifyRouteConfigUpdate(ctx, rw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	select {
	case <-ctx.Done():
		t.Fatal("timeout when waiting for receipt of ACK at the management server")
	case <-firstAckReceived.Done():
	}

	// Register another watch for the same resource. This should get the update
	// from the cache.
	rw2 := newRouteConfigWatcher()
	rdsCancel2 := xdsresource.WatchRouteConfig(client, rdsName, rw2)
	defer rdsCancel2()
	if err := verifyRouteConfigUpdate(ctx, rw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	// No request should get sent out as part of this watch.
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-secondRequestReceived.Done():
		t.Fatal("xdsClient sent out request instead of using update from cache")
	}
}

// TestRDSWatch_ExpiryTimerFiresBeforeResponse tests the case where the client
// does not receive an RDS response for the request that it sends. The test
// verifies that the watch callback is invoked with an error once the
// watchExpiryTimer fires.
func (s) TestRDSWatch_ExpiryTimerFiresBeforeResponse(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS client talking to the above management server.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:               t.Name(),
		Contents:           bc,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer close()

	// Register a watch for a resource which is expected to fail with an error
	// after the watch expiry timer fires.
	rw := newRouteConfigWatcher()
	rdsCancel := xdsresource.WatchRouteConfig(client, rdsName, rw)
	defer rdsCancel()

	// Wait for the watch expiry timer to fire.
	<-time.After(defaultTestWatchExpiryTimeout)

	// Verify that an empty update with the expected error is received.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantErr := xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "")
	if err := verifyRouteConfigUpdate(ctx, rw.updateCh, routeConfigUpdateErrTuple{err: wantErr}); err != nil {
		t.Fatal(err)
	}
}

// TestRDSWatch_ValidResponseCancelsExpiryTimerBehavior tests the case where the
// client receives a valid RDS response for the request that it sends. The test
// verifies that the behavior associated with the expiry timer (i.e, callback
// invocation with error) does not take place.
func (s) TestRDSWatch_ValidResponseCancelsExpiryTimerBehavior(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS client talking to the above management server.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:               t.Name(),
		Contents:           bc,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer close()

	// Register a watch for a route configuration resource and have the watch
	// callback push the received update on to a channel.
	rw := newRouteConfigWatcher()
	rdsCancel := xdsresource.WatchRouteConfig(client, rdsName, rw)
	defer rdsCancel()

	// Configure the management server to return a single route configuration
	// resource, corresponding to the one we registered a watch for.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Routes:         []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(rdsName, ldsName, cdsName)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update.
	wantUpdate := routeConfigUpdateErrTuple{
		update: xdsresource.RouteConfigUpdate{
			VirtualHosts: []*xdsresource.VirtualHost{
				{
					Domains: []string{ldsName},
					Routes: []*xdsresource.Route{
						{
							Prefix:           newStringP("/"),
							ActionType:       xdsresource.RouteActionRoute,
							WeightedClusters: map[string]xdsresource.WeightedCluster{cdsName: {Weight: 100}},
						},
					},
				},
			},
		},
	}
	if err := verifyRouteConfigUpdate(ctx, rw.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}

	// Wait for the watch expiry timer to fire, and verify that the callback is
	// not invoked.
	<-time.After(defaultTestWatchExpiryTimeout)
	if err := verifyNoRouteConfigUpdate(ctx, rw.updateCh); err != nil {
		t.Fatal(err)
	}
}

// TestRDSWatch_NACKError covers the case where an update from the management
// server is NACK'ed by the xdsclient. The test verifies that the error is
// propagated to the watcher.
func (s) TestRDSWatch_NACKError(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS client with the above bootstrap contents.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Register a watch for a route configuration resource and have the watch
	// callback push the received update on to a channel.
	rw := newRouteConfigWatcher()
	rdsCancel := xdsresource.WatchRouteConfig(client, rdsName, rw)
	defer rdsCancel()

	// Configure the management server to return a single route configuration
	// resource which is expected to be NACKed by the client.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Routes:         []*v3routepb.RouteConfiguration{badRouteConfigResource(rdsName, ldsName, cdsName)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify that the expected error is propagated to the watcher.
	u, err := rw.updateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for a route configuration resource from the management server: %v", err)
	}
	gotErr := u.(routeConfigUpdateErrTuple).err
	if gotErr == nil || !strings.Contains(gotErr.Error(), wantRouteConfigNACKErr) {
		t.Fatalf("update received with error: %v, want %q", gotErr, wantRouteConfigNACKErr)
	}
}

// TestRDSWatch_PartialValid covers the case where a response from the
// management server contains both valid and invalid resources and is expected
// to be NACK'ed by the xdsclient. The test verifies that watchers corresponding
// to the valid resource receive the update, while watchers corresponding to the
// invalid resource receive an error.
func (s) TestRDSWatch_PartialValid(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	authority := makeAuthorityName(t.Name())
	bc, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		Authorities: map[string]json.RawMessage{
			// Xdstp style resource names used in this test use a slash removed
			// version of t.Name as their authority, and the empty config
			// results in the top-level xds server configuration being used for
			// this authority.
			authority: []byte(`{}`),
		},
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}
	testutils.CreateBootstrapFileForTesting(t, bc)

	// Create an xDS client with the above bootstrap contents.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	// Register two watches for route configuration resources. The first watch
	// is expected to receive an error because the received resource is NACKed.
	// The second watch is expected to get a good update.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	badResourceName := rdsName
	rw1 := newRouteConfigWatcher()
	rdsCancel1 := xdsresource.WatchRouteConfig(client, badResourceName, rw1)
	defer rdsCancel1()
	goodResourceName := makeNewStyleRDSName(authority)
	rw2 := newRouteConfigWatcher()
	rdsCancel2 := xdsresource.WatchRouteConfig(client, goodResourceName, rw2)
	defer rdsCancel2()

	// Configure the management server to return two route configuration
	// resources, corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Routes: []*v3routepb.RouteConfiguration{
			badRouteConfigResource(badResourceName, ldsName, cdsName),
			e2e.DefaultRouteConfig(goodResourceName, ldsName, cdsName),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify that the expected error is propagated to the watcher which
	// requested for the bad resource.
	u, err := rw1.updateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for a route configuration resource from the management server: %v", err)
	}
	gotErr := u.(routeConfigUpdateErrTuple).err
	if gotErr == nil || !strings.Contains(gotErr.Error(), wantRouteConfigNACKErr) {
		t.Fatalf("update received with error: %v, want %q", gotErr, wantRouteConfigNACKErr)
	}

	// Verify that the watcher watching the good resource receives a good
	// update.
	wantUpdate := routeConfigUpdateErrTuple{
		update: xdsresource.RouteConfigUpdate{
			VirtualHosts: []*xdsresource.VirtualHost{
				{
					Domains: []string{ldsName},
					Routes: []*xdsresource.Route{
						{
							Prefix:           newStringP("/"),
							ActionType:       xdsresource.RouteActionRoute,
							WeightedClusters: map[string]xdsresource.WeightedCluster{cdsName: {Weight: 100}},
						},
					},
				},
			},
		},
	}
	if err := verifyRouteConfigUpdate(ctx, rw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}
