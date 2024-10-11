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

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
)

type noopClusterWatcher struct{}

func (noopClusterWatcher) OnUpdate(update *xdsresource.ClusterResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (noopClusterWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (noopClusterWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

type clusterUpdateErrTuple struct {
	update xdsresource.ClusterUpdate
	err    error
}

type clusterWatcher struct {
	updateCh *testutils.Channel
}

func newClusterWatcher() *clusterWatcher {
	return &clusterWatcher{updateCh: testutils.NewChannel()}
}

func (cw *clusterWatcher) OnUpdate(update *xdsresource.ClusterResourceData, onDone xdsresource.OnDoneFunc) {
	cw.updateCh.Send(clusterUpdateErrTuple{update: update.Resource})
	onDone()
}

func (cw *clusterWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	// When used with a go-control-plane management server that continuously
	// resends resources which are NACKed by the xDS client, using a `Replace()`
	// here and in OnResourceDoesNotExist() simplifies tests which will have
	// access to the most recently received error.
	cw.updateCh.Replace(clusterUpdateErrTuple{err: err})
	onDone()
}

func (cw *clusterWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	cw.updateCh.Replace(clusterUpdateErrTuple{err: xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "Cluster not found in received response")})
	onDone()
}

// badClusterResource returns a cluster resource for the given name which
// contains a config_source_specifier for the `lrs_server` field which is not
// set to `self`, and hence is expected to be NACKed by the client.
func badClusterResource(clusterName, edsServiceName string, secLevel e2e.SecurityLevel) *v3clusterpb.Cluster {
	cluster := e2e.DefaultCluster(clusterName, edsServiceName, secLevel)
	cluster.LrsServer = &v3corepb.ConfigSource{ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{}}
	return cluster
}

// xdsClient is expected to produce an error containing this string when an
// update is received containing a cluster created using `badClusterResource`.
const wantClusterNACKErr = "unsupported config_source_specifier"

// verifyClusterUpdate waits for an update to be received on the provided update
// channel and verifies that it matches the expected update.
//
// Returns an error if no update is received before the context deadline expires
// or the received update does not match the expected one.
func verifyClusterUpdate(ctx context.Context, updateCh *testutils.Channel, wantUpdate clusterUpdateErrTuple) error {
	u, err := updateCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("timeout when waiting for a cluster resource from the management server: %v", err)
	}
	got := u.(clusterUpdateErrTuple)
	if wantUpdate.err != nil {
		if gotType, wantType := xdsresource.ErrType(got.err), xdsresource.ErrType(wantUpdate.err); gotType != wantType {
			return fmt.Errorf("received update with error type %v, want %v", gotType, wantType)
		}
	}
	cmpOpts := []cmp.Option{cmpopts.EquateEmpty(), cmpopts.IgnoreFields(xdsresource.ClusterUpdate{}, "Raw", "LBPolicy", "TelemetryLabels")}
	if diff := cmp.Diff(wantUpdate.update, got.update, cmpOpts...); diff != "" {
		return fmt.Errorf("received unexpected diff in the cluster resource update: (-want, got):\n%s", diff)
	}
	return nil
}

// verifyNoClusterUpdate verifies that no cluster update is received on the
// provided update channel, and returns an error if an update is received.
//
// A very short deadline is used while waiting for the update, as this function
// is intended to be used when an update is not expected.
func verifyNoClusterUpdate(ctx context.Context, updateCh *testutils.Channel) error {
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if u, err := updateCh.Receive(sCtx); err != context.DeadlineExceeded {
		return fmt.Errorf("received unexpected ClusterUpdate when expecting none: %v", u)
	}
	return nil
}

// TestCDSWatch covers the case where a single watcher exists for a single
// cluster resource. The test verifies the following scenarios:
//  1. An update from the management server containing the resource being
//     watched should result in the invocation of the watch callback.
//  2. An update from the management server containing a resource *not* being
//     watched should not result in the invocation of the watch callback.
//  3. After the watch is cancelled, an update from the management server
//     containing the resource that was being watched should not result in the
//     invocation of the watch callback.
//
// The test is run for old and new style names.
func (s) TestCDSWatch(t *testing.T) {
	tests := []struct {
		desc                   string
		resourceName           string
		watchedResource        *v3clusterpb.Cluster // The resource being watched.
		updatedWatchedResource *v3clusterpb.Cluster // The watched resource after an update.
		notWatchedResource     *v3clusterpb.Cluster // A resource which is not being watched.
		wantUpdate             clusterUpdateErrTuple
	}{
		{
			desc:                   "old style resource",
			resourceName:           cdsName,
			watchedResource:        e2e.DefaultCluster(cdsName, edsName, e2e.SecurityLevelNone),
			updatedWatchedResource: e2e.DefaultCluster(cdsName, "new-eds-resource", e2e.SecurityLevelNone),
			notWatchedResource:     e2e.DefaultCluster("unsubscribed-cds-resource", edsName, e2e.SecurityLevelNone),
			wantUpdate: clusterUpdateErrTuple{
				update: xdsresource.ClusterUpdate{
					ClusterName:    cdsName,
					EDSServiceName: edsName,
				},
			},
		},
		{
			desc:                   "new style resource",
			resourceName:           cdsNameNewStyle,
			watchedResource:        e2e.DefaultCluster(cdsNameNewStyle, edsNameNewStyle, e2e.SecurityLevelNone),
			updatedWatchedResource: e2e.DefaultCluster(cdsNameNewStyle, "new-eds-resource", e2e.SecurityLevelNone),
			notWatchedResource:     e2e.DefaultCluster("unsubscribed-cds-resource", edsNameNewStyle, e2e.SecurityLevelNone),
			wantUpdate: clusterUpdateErrTuple{
				update: xdsresource.ClusterUpdate{
					ClusterName:    cdsNameNewStyle,
					EDSServiceName: edsNameNewStyle,
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

			// Register a watch for a cluster resource and have the watch
			// callback push the received update on to a channel.
			cw := newClusterWatcher()
			cdsCancel := xdsresource.WatchCluster(client, test.resourceName, cw)

			// Configure the management server to return a single cluster
			// resource, corresponding to the one we registered a watch for.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.watchedResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}

			// Verify the contents of the received update.
			if err := verifyClusterUpdate(ctx, cw.updateCh, test.wantUpdate); err != nil {
				t.Fatal(err)
			}

			// Configure the management server to return an additional cluster
			// resource, one that we are not interested in.
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.watchedResource, test.notWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoClusterUpdate(ctx, cw.updateCh); err != nil {
				t.Fatal(err)
			}

			// Cancel the watch and update the resource corresponding to the original
			// watch.  Ensure that the cancelled watch callback is not invoked.
			cdsCancel()
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.updatedWatchedResource, test.notWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoClusterUpdate(ctx, cw.updateCh); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestCDSWatch_TwoWatchesForSameResourceName covers the case where two watchers
// exist for a single cluster resource.  The test verifies the following
// scenarios:
//  1. An update from the management server containing the resource being
//     watched should result in the invocation of both watch callbacks.
//  2. After one of the watches is cancelled, a redundant update from the
//     management server should not result in the invocation of either of the
//     watch callbacks.
//  3. A new update from the management server containing the resource being
//     watched should result in the invocation of the un-cancelled watch
//     callback.
//
// The test is run for old and new style names.
func (s) TestCDSWatch_TwoWatchesForSameResourceName(t *testing.T) {
	tests := []struct {
		desc                   string
		resourceName           string
		watchedResource        *v3clusterpb.Cluster // The resource being watched.
		updatedWatchedResource *v3clusterpb.Cluster // The watched resource after an update.
		wantUpdateV1           clusterUpdateErrTuple
		wantUpdateV2           clusterUpdateErrTuple
	}{
		{
			desc:                   "old style resource",
			resourceName:           cdsName,
			watchedResource:        e2e.DefaultCluster(cdsName, edsName, e2e.SecurityLevelNone),
			updatedWatchedResource: e2e.DefaultCluster(cdsName, "new-eds-resource", e2e.SecurityLevelNone),
			wantUpdateV1: clusterUpdateErrTuple{
				update: xdsresource.ClusterUpdate{
					ClusterName:    cdsName,
					EDSServiceName: edsName,
				},
			},
			wantUpdateV2: clusterUpdateErrTuple{
				update: xdsresource.ClusterUpdate{
					ClusterName:    cdsName,
					EDSServiceName: "new-eds-resource",
				},
			},
		},
		{
			desc:                   "new style resource",
			resourceName:           cdsNameNewStyle,
			watchedResource:        e2e.DefaultCluster(cdsNameNewStyle, edsNameNewStyle, e2e.SecurityLevelNone),
			updatedWatchedResource: e2e.DefaultCluster(cdsNameNewStyle, "new-eds-resource", e2e.SecurityLevelNone),
			wantUpdateV1: clusterUpdateErrTuple{
				update: xdsresource.ClusterUpdate{
					ClusterName:    cdsNameNewStyle,
					EDSServiceName: edsNameNewStyle,
				},
			},
			wantUpdateV2: clusterUpdateErrTuple{
				update: xdsresource.ClusterUpdate{
					ClusterName:    cdsNameNewStyle,
					EDSServiceName: "new-eds-resource",
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

			// Register two watches for the same cluster resource and have the
			// callbacks push the received updates on to a channel.
			cw1 := newClusterWatcher()
			cdsCancel1 := xdsresource.WatchCluster(client, test.resourceName, cw1)
			defer cdsCancel1()
			cw2 := newClusterWatcher()
			cdsCancel2 := xdsresource.WatchCluster(client, test.resourceName, cw2)

			// Configure the management server to return a single cluster
			// resource, corresponding to the one we registered watches for.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.watchedResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}

			// Verify the contents of the received update.
			if err := verifyClusterUpdate(ctx, cw1.updateCh, test.wantUpdateV1); err != nil {
				t.Fatal(err)
			}
			if err := verifyClusterUpdate(ctx, cw2.updateCh, test.wantUpdateV1); err != nil {
				t.Fatal(err)
			}

			// Cancel the second watch and force the management server to push a
			// redundant update for the resource being watched. Neither of the
			// two watch callbacks should be invoked.
			cdsCancel2()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoClusterUpdate(ctx, cw1.updateCh); err != nil {
				t.Fatal(err)
			}
			if err := verifyNoClusterUpdate(ctx, cw2.updateCh); err != nil {
				t.Fatal(err)
			}

			// Update to the resource being watched. The un-cancelled callback
			// should be invoked while the cancelled one should not be.
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.updatedWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyClusterUpdate(ctx, cw1.updateCh, test.wantUpdateV2); err != nil {
				t.Fatal(err)
			}
			if err := verifyNoClusterUpdate(ctx, cw2.updateCh); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestCDSWatch_ThreeWatchesForDifferentResourceNames covers the case where
// three watchers (two watchers for one resource, and the third watcher for
// another resource) exist across two cluster resources (one with an old style
// name and one with a new style name).  The test verifies that an update from
// the management server containing both resources results in the invocation of
// all watch callbacks.
func (s) TestCDSWatch_ThreeWatchesForDifferentResourceNames(t *testing.T) {
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

	// Register two watches for the same cluster resource and have the
	// callbacks push the received updates on to a channel.
	cw1 := newClusterWatcher()
	cdsCancel1 := xdsresource.WatchCluster(client, cdsName, cw1)
	defer cdsCancel1()
	cw2 := newClusterWatcher()
	cdsCancel2 := xdsresource.WatchCluster(client, cdsName, cw2)
	defer cdsCancel2()

	// Register the third watch for a different cluster resource, and push the
	// received updates onto a channel.
	cdsNameNewStyle := makeNewStyleCDSName(authority)
	cw3 := newClusterWatcher()
	cdsCancel3 := xdsresource.WatchCluster(client, cdsNameNewStyle, cw3)
	defer cdsCancel3()

	// Configure the management server to return two cluster resources,
	// corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			e2e.DefaultCluster(cdsName, edsName, e2e.SecurityLevelNone),
			e2e.DefaultCluster(cdsNameNewStyle, edsNameNewStyle, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for the all watchers.
	wantUpdate12 := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    cdsName,
			EDSServiceName: edsName,
		},
	}
	wantUpdate3 := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    cdsNameNewStyle,
			EDSServiceName: edsNameNewStyle,
		},
	}
	if err := verifyClusterUpdate(ctx, cw1.updateCh, wantUpdate12); err != nil {
		t.Fatal(err)
	}
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate12); err != nil {
		t.Fatal(err)
	}
	if err := verifyClusterUpdate(ctx, cw3.updateCh, wantUpdate3); err != nil {
		t.Fatal(err)
	}
}

// TestCDSWatch_ResourceCaching covers the case where a watch is registered for
// a resource which is already present in the cache.  The test verifies that the
// watch callback is invoked with the contents from the cache, instead of a
// request being sent to the management server.
func (s) TestCDSWatch_ResourceCaching(t *testing.T) {
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

	// Register a watch for a cluster resource and have the watch
	// callback push the received update on to a channel.
	cw1 := newClusterWatcher()
	cdsCancel1 := xdsresource.WatchCluster(client, cdsName, cw1)
	defer cdsCancel1()

	// Configure the management server to return a single cluster
	// resource, corresponding to the one we registered a watch for.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(cdsName, edsName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update.
	wantUpdate := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    cdsName,
			EDSServiceName: edsName,
		},
	}
	if err := verifyClusterUpdate(ctx, cw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	select {
	case <-ctx.Done():
		t.Fatal("timeout when waiting for receipt of ACK at the management server")
	case <-firstAckReceived.Done():
	}

	// Register another watch for the same resource. This should get the update
	// from the cache.
	cw2 := newClusterWatcher()
	cdsCancel2 := xdsresource.WatchCluster(client, cdsName, cw2)
	defer cdsCancel2()
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate); err != nil {
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

// TestCDSWatch_ExpiryTimerFiresBeforeResponse tests the case where the client
// does not receive an CDS response for the request that it sends. The test
// verifies that the watch callback is invoked with an error once the
// watchExpiryTimer fires.
func (s) TestCDSWatch_ExpiryTimerFiresBeforeResponse(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:               t.Name(),
		Contents:           bc,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer close()

	// Register a watch for a resource which is expected to be invoked with an
	// error after the watch expiry timer fires.
	cw := newClusterWatcher()
	cdsCancel := xdsresource.WatchCluster(client, cdsName, cw)
	defer cdsCancel()

	// Wait for the watch expiry timer to fire.
	<-time.After(defaultTestWatchExpiryTimeout)

	// Verify that an empty update with the expected error is received.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantErr := xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "")
	if err := verifyClusterUpdate(ctx, cw.updateCh, clusterUpdateErrTuple{err: wantErr}); err != nil {
		t.Fatal(err)
	}
}

// TestCDSWatch_ValidResponseCancelsExpiryTimerBehavior tests the case where the
// client receives a valid LDS response for the request that it sends. The test
// verifies that the behavior associated with the expiry timer (i.e, callback
// invocation with error) does not take place.
func (s) TestCDSWatch_ValidResponseCancelsExpiryTimerBehavior(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create an xDS client talking to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bc)

	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:               t.Name(),
		Contents:           bc,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer close()

	// Register a watch for a cluster resource and have the watch
	// callback push the received update on to a channel.
	cw := newClusterWatcher()
	cdsCancel := xdsresource.WatchCluster(client, cdsName, cw)
	defer cdsCancel()

	// Configure the management server to return a single cluster resource,
	// corresponding to the one we registered a watch for.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(cdsName, edsName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update.
	wantUpdate := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    cdsName,
			EDSServiceName: edsName,
		},
	}
	if err := verifyClusterUpdate(ctx, cw.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}

	// Wait for the watch expiry timer to fire, and verify that the callback is
	// not invoked.
	<-time.After(defaultTestWatchExpiryTimeout)
	if err := verifyNoClusterUpdate(ctx, cw.updateCh); err != nil {
		t.Fatal(err)
	}
}

// TestCDSWatch_ResourceRemoved covers the cases where two watchers exists for
// two different resources (one with an old style name and one with a new style
// name). One of these resources being watched is removed from the management
// server. The test verifies the following scenarios:
//  1. Removing a resource should trigger the watch callback associated with that
//     resource with a resource removed error. It should not trigger the watch
//     callback for an unrelated resource.
//  2. An update to other resource should result in the invocation of the watch
//     callback associated with that resource.  It should not result in the
//     invocation of the watch callback associated with the deleted resource.
func (s) TestCDSWatch_ResourceRemoved(t *testing.T) {
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

	// Register two watches for two cluster resources and have the
	// callbacks push the received updates on to a channel.
	resourceName1 := cdsName
	cw1 := newClusterWatcher()
	cdsCancel1 := xdsresource.WatchCluster(client, resourceName1, cw1)
	defer cdsCancel1()

	resourceName2 := makeNewStyleCDSName(authority)
	cw2 := newClusterWatcher()
	cdsCancel2 := xdsresource.WatchCluster(client, resourceName2, cw2)
	defer cdsCancel2()

	// Configure the management server to return two cluster resources,
	// corresponding to the registered watches.
	edsNameNewStyle := makeNewStyleEDSName(authority)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			e2e.DefaultCluster(resourceName1, edsName, e2e.SecurityLevelNone),
			e2e.DefaultCluster(resourceName2, edsNameNewStyle, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for both watchers.
	wantUpdate1 := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    resourceName1,
			EDSServiceName: edsName,
		},
	}
	wantUpdate2 := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    resourceName2,
			EDSServiceName: edsNameNewStyle,
		},
	}
	if err := verifyClusterUpdate(ctx, cw1.updateCh, wantUpdate1); err != nil {
		t.Fatal(err)
	}
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate2); err != nil {
		t.Fatal(err)
	}

	// Remove the first cluster resource on the management server.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(resourceName2, edsNameNewStyle, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// The first watcher should receive a resource removed error, while the
	// second watcher should not receive an update.
	if err := verifyClusterUpdate(ctx, cw1.updateCh, clusterUpdateErrTuple{err: xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "")}); err != nil {
		t.Fatal(err)
	}
	if err := verifyNoClusterUpdate(ctx, cw2.updateCh); err != nil {
		t.Fatal(err)
	}

	// Update the second cluster resource on the management server. The first
	// watcher should not receive an update, while the second watcher should.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(resourceName2, "new-eds-resource", e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}
	if err := verifyNoClusterUpdate(ctx, cw1.updateCh); err != nil {
		t.Fatal(err)
	}
	wantUpdate := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    resourceName2,
			EDSServiceName: "new-eds-resource",
		},
	}
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestCDSWatch_NACKError covers the case where an update from the management
// server is NACK'ed by the xdsclient. The test verifies that the error is
// propagated to the watcher.
func (s) TestCDSWatch_NACKError(t *testing.T) {
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

	// Register a watch for a cluster resource and have the watch
	// callback push the received update on to a channel.
	cw := newClusterWatcher()
	cdsCancel := xdsresource.WatchCluster(client, cdsName, cw)
	defer cdsCancel()

	// Configure the management server to return a single cluster resource
	// which is expected to be NACK'ed by the client.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{badClusterResource(cdsName, edsName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify that the expected error is propagated to the watcher.
	u, err := cw.updateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for a cluster resource from the management server: %v", err)
	}
	gotErr := u.(clusterUpdateErrTuple).err
	if gotErr == nil || !strings.Contains(gotErr.Error(), wantClusterNACKErr) {
		t.Fatalf("update received with error: %v, want %q", gotErr, wantClusterNACKErr)
	}
}

// TestCDSWatch_PartialValid covers the case where a response from the
// management server contains both valid and invalid resources and is expected
// to be NACK'ed by the xdsclient. The test verifies that watchers corresponding
// to the valid resource receive the update, while watchers corresponding to the
// invalid resource receive an error.
func (s) TestCDSWatch_PartialValid(t *testing.T) {
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

	// Register two watches for cluster resources. The first watch is expected
	// to receive an error because the received resource is NACK'ed. The second
	// watch is expected to get a good update.
	badResourceName := cdsName
	cw1 := newClusterWatcher()
	cdsCancel1 := xdsresource.WatchCluster(client, badResourceName, cw1)
	defer cdsCancel1()
	goodResourceName := makeNewStyleCDSName(authority)
	cw2 := newClusterWatcher()
	cdsCancel2 := xdsresource.WatchCluster(client, goodResourceName, cw2)
	defer cdsCancel2()

	// Configure the management server with two cluster resources. One of these
	// is a bad resource causing the update to be NACKed.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			badClusterResource(badResourceName, edsName, e2e.SecurityLevelNone),
			e2e.DefaultCluster(goodResourceName, edsName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify that the expected error is propagated to the watcher which is
	// watching the bad resource.
	u, err := cw1.updateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for a cluster resource from the management server: %v", err)
	}
	gotErr := u.(clusterUpdateErrTuple).err
	if gotErr == nil || !strings.Contains(gotErr.Error(), wantClusterNACKErr) {
		t.Fatalf("update received with error: %v, want %q", gotErr, wantClusterNACKErr)
	}

	// Verify that the watcher watching the good resource receives a good
	// update.
	wantUpdate := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    goodResourceName,
			EDSServiceName: edsName,
		},
	}
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestCDSWatch_PartialResponse covers the case where a response from the
// management server does not contain all requested resources. CDS responses are
// supposed to contain all requested resources, and the absence of one usually
// indicates that the management server does not know about it. In cases where
// the server has never responded with this resource before, the xDS client is
// expected to wait for the watch timeout to expire before concluding that the
// resource does not exist on the server
func (s) TestCDSWatch_PartialResponse(t *testing.T) {
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

	// Register two watches for two cluster resources and have the
	// callbacks push the received updates on to a channel.
	resourceName1 := cdsName
	cw1 := newClusterWatcher()
	cdsCancel1 := xdsresource.WatchCluster(client, resourceName1, cw1)
	defer cdsCancel1()
	resourceName2 := makeNewStyleCDSName(authority)
	cw2 := newClusterWatcher()
	cdsCancel2 := xdsresource.WatchCluster(client, resourceName2, cw2)
	defer cdsCancel2()

	// Configure the management server to return only one of the two cluster
	// resources, corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(resourceName1, edsName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for first watcher.
	wantUpdate1 := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    resourceName1,
			EDSServiceName: edsName,
		},
	}
	if err := verifyClusterUpdate(ctx, cw1.updateCh, wantUpdate1); err != nil {
		t.Fatal(err)
	}

	// Verify that the second watcher does not get an update with an error.
	if err := verifyNoClusterUpdate(ctx, cw2.updateCh); err != nil {
		t.Fatal(err)
	}

	// Configure the management server to return two cluster resources,
	// corresponding to the registered watches.
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			e2e.DefaultCluster(resourceName1, edsName, e2e.SecurityLevelNone),
			e2e.DefaultCluster(resourceName2, edsNameNewStyle, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for the second watcher.
	wantUpdate2 := clusterUpdateErrTuple{
		update: xdsresource.ClusterUpdate{
			ClusterName:    resourceName2,
			EDSServiceName: edsNameNewStyle,
		},
	}
	if err := verifyClusterUpdate(ctx, cw2.updateCh, wantUpdate2); err != nil {
		t.Fatal(err)
	}

	// Verify that the first watcher gets no update, as the first resource did
	// not change.
	if err := verifyNoClusterUpdate(ctx, cw1.updateCh); err != nil {
		t.Fatal(err)
	}
}
