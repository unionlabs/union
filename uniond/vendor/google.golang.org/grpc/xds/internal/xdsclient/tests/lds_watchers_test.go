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

	"github.com/envoyproxy/go-control-plane/pkg/wellknown"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"

	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routerpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/router/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"

	_ "google.golang.org/grpc/xds"                            // To ensure internal.NewXDSResolverWithConfigForTesting is set.
	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter.
)

type noopListenerWatcher struct{}

func (noopListenerWatcher) OnUpdate(update *xdsresource.ListenerResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (noopListenerWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (noopListenerWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

type listenerUpdateErrTuple struct {
	update xdsresource.ListenerUpdate
	err    error
}

type listenerWatcher struct {
	updateCh *testutils.Channel
}

func newListenerWatcher() *listenerWatcher {
	return &listenerWatcher{updateCh: testutils.NewChannel()}
}

func (cw *listenerWatcher) OnUpdate(update *xdsresource.ListenerResourceData, onDone xdsresource.OnDoneFunc) {
	cw.updateCh.Send(listenerUpdateErrTuple{update: update.Resource})
	onDone()
}

func (cw *listenerWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	// When used with a go-control-plane management server that continuously
	// resends resources which are NACKed by the xDS client, using a `Replace()`
	// here and in OnResourceDoesNotExist() simplifies tests which will have
	// access to the most recently received error.
	cw.updateCh.Replace(listenerUpdateErrTuple{err: err})
	onDone()
}

func (cw *listenerWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	cw.updateCh.Replace(listenerUpdateErrTuple{err: xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "Listener not found in received response")})
	onDone()
}

// badListenerResource returns a listener resource for the given name which does
// not contain the `RouteSpecifier` field in the HTTPConnectionManager, and
// hence is expected to be NACKed by the client.
func badListenerResource(t *testing.T, name string) *v3listenerpb.Listener {
	hcm := testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
		HttpFilters: []*v3httppb.HttpFilter{e2e.HTTPFilter("router", &v3routerpb.Router{})},
	})
	return &v3listenerpb.Listener{
		Name:        name,
		ApiListener: &v3listenerpb.ApiListener{ApiListener: hcm},
		FilterChains: []*v3listenerpb.FilterChain{{
			Name: "filter-chain-name",
			Filters: []*v3listenerpb.Filter{{
				Name:       wellknown.HTTPConnectionManager,
				ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: hcm},
			}},
		}},
	}
}

// xdsClient is expected to produce an error containing this string when an
// update is received containing a listener created using `badListenerResource`.
const wantListenerNACKErr = "no RouteSpecifier"

// verifyNoListenerUpdate verifies that no listener update is received on the
// provided update channel, and returns an error if an update is received.
//
// A very short deadline is used while waiting for the update, as this function
// is intended to be used when an update is not expected.
func verifyNoListenerUpdate(ctx context.Context, updateCh *testutils.Channel) error {
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	if u, err := updateCh.Receive(sCtx); err != context.DeadlineExceeded {
		return fmt.Errorf("unexpected ListenerUpdate: %v", u)
	}
	return nil
}

// verifyListenerUpdate waits for an update to be received on the provided
// update channel and verifies that it matches the expected update.
//
// Returns an error if no update is received before the context deadline expires
// or the received update does not match the expected one.
func verifyListenerUpdate(ctx context.Context, updateCh *testutils.Channel, wantUpdate listenerUpdateErrTuple) error {
	u, err := updateCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("timeout when waiting for a listener resource from the management server: %v", err)
	}
	got := u.(listenerUpdateErrTuple)
	if wantUpdate.err != nil {
		if gotType, wantType := xdsresource.ErrType(got.err), xdsresource.ErrType(wantUpdate.err); gotType != wantType {
			return fmt.Errorf("received update with error type %v, want %v", gotType, wantType)
		}
	}
	cmpOpts := []cmp.Option{
		cmpopts.EquateEmpty(),
		cmpopts.IgnoreFields(xdsresource.HTTPFilter{}, "Filter", "Config"),
		cmpopts.IgnoreFields(xdsresource.ListenerUpdate{}, "Raw"),
	}
	if diff := cmp.Diff(wantUpdate.update, got.update, cmpOpts...); diff != "" {
		return fmt.Errorf("received unexpected diff in the listener resource update: (-want, got):\n%s", diff)
	}
	return nil
}

// TestLDSWatch covers the case where a single watcher exists for a single
// listener resource. The test verifies the following scenarios:
//  1. An update from the management server containing the resource being
//     watched should result in the invocation of the watch callback.
//  2. An update from the management server containing a resource *not* being
//     watched should not result in the invocation of the watch callback.
//  3. After the watch is cancelled, an update from the management server
//     containing the resource that was being watched should not result in the
//     invocation of the watch callback.
//
// The test is run for old and new style names.
func (s) TestLDSWatch(t *testing.T) {
	tests := []struct {
		desc                   string
		resourceName           string
		watchedResource        *v3listenerpb.Listener // The resource being watched.
		updatedWatchedResource *v3listenerpb.Listener // The watched resource after an update.
		notWatchedResource     *v3listenerpb.Listener // A resource which is not being watched.
		wantUpdate             listenerUpdateErrTuple
	}{
		{
			desc:                   "old style resource",
			resourceName:           ldsName,
			watchedResource:        e2e.DefaultClientListener(ldsName, rdsName),
			updatedWatchedResource: e2e.DefaultClientListener(ldsName, "new-rds-resource"),
			notWatchedResource:     e2e.DefaultClientListener("unsubscribed-lds-resource", rdsName),
			wantUpdate: listenerUpdateErrTuple{
				update: xdsresource.ListenerUpdate{
					RouteConfigName: rdsName,
					HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
				},
			},
		},
		{
			desc:                   "new style resource",
			resourceName:           ldsNameNewStyle,
			watchedResource:        e2e.DefaultClientListener(ldsNameNewStyle, rdsNameNewStyle),
			updatedWatchedResource: e2e.DefaultClientListener(ldsNameNewStyle, "new-rds-resource"),
			notWatchedResource:     e2e.DefaultClientListener("unsubscribed-lds-resource", rdsNameNewStyle),
			wantUpdate: listenerUpdateErrTuple{
				update: xdsresource.ListenerUpdate{
					RouteConfigName: rdsNameNewStyle,
					HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
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

			// Register a watch for a listener resource and have the watch
			// callback push the received update on to a channel.
			lw := newListenerWatcher()
			ldsCancel := xdsresource.WatchListener(client, test.resourceName, lw)

			// Configure the management server to return a single listener
			// resource, corresponding to the one we registered a watch for.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Listeners:      []*v3listenerpb.Listener{test.watchedResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}

			// Verify the contents of the received update.
			if err := verifyListenerUpdate(ctx, lw.updateCh, test.wantUpdate); err != nil {
				t.Fatal(err)
			}

			// Configure the management server to return an additional listener
			// resource, one that we are not interested in.
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Listeners:      []*v3listenerpb.Listener{test.watchedResource, test.notWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoListenerUpdate(ctx, lw.updateCh); err != nil {
				t.Fatal(err)
			}

			// Cancel the watch and update the resource corresponding to the original
			// watch.  Ensure that the cancelled watch callback is not invoked.
			ldsCancel()
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Listeners:      []*v3listenerpb.Listener{test.updatedWatchedResource, test.notWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoListenerUpdate(ctx, lw.updateCh); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestLDSWatch_TwoWatchesForSameResourceName covers the case where two watchers
// exist for a single listener resource.  The test verifies the following
// scenarios:
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
func (s) TestLDSWatch_TwoWatchesForSameResourceName(t *testing.T) {
	tests := []struct {
		desc                   string
		resourceName           string
		watchedResource        *v3listenerpb.Listener // The resource being watched.
		updatedWatchedResource *v3listenerpb.Listener // The watched resource after an update.
		wantUpdateV1           listenerUpdateErrTuple
		wantUpdateV2           listenerUpdateErrTuple
	}{
		{
			desc:                   "old style resource",
			resourceName:           ldsName,
			watchedResource:        e2e.DefaultClientListener(ldsName, rdsName),
			updatedWatchedResource: e2e.DefaultClientListener(ldsName, "new-rds-resource"),
			wantUpdateV1: listenerUpdateErrTuple{
				update: xdsresource.ListenerUpdate{
					RouteConfigName: rdsName,
					HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
				},
			},
			wantUpdateV2: listenerUpdateErrTuple{
				update: xdsresource.ListenerUpdate{
					RouteConfigName: "new-rds-resource",
					HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
				},
			},
		},
		{
			desc:                   "new style resource",
			resourceName:           ldsNameNewStyle,
			watchedResource:        e2e.DefaultClientListener(ldsNameNewStyle, rdsNameNewStyle),
			updatedWatchedResource: e2e.DefaultClientListener(ldsNameNewStyle, "new-rds-resource"),
			wantUpdateV1: listenerUpdateErrTuple{
				update: xdsresource.ListenerUpdate{
					RouteConfigName: rdsNameNewStyle,
					HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
				},
			},
			wantUpdateV2: listenerUpdateErrTuple{
				update: xdsresource.ListenerUpdate{
					RouteConfigName: "new-rds-resource",
					HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
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

			// Register two watches for the same listener resource and have the
			// callbacks push the received updates on to a channel.
			lw1 := newListenerWatcher()
			ldsCancel1 := xdsresource.WatchListener(client, test.resourceName, lw1)
			defer ldsCancel1()
			lw2 := newListenerWatcher()
			ldsCancel2 := xdsresource.WatchListener(client, test.resourceName, lw2)

			// Configure the management server to return a single listener
			// resource, corresponding to the one we registered watches for.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Listeners:      []*v3listenerpb.Listener{test.watchedResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}

			// Verify the contents of the received update.
			if err := verifyListenerUpdate(ctx, lw1.updateCh, test.wantUpdateV1); err != nil {
				t.Fatal(err)
			}
			if err := verifyListenerUpdate(ctx, lw2.updateCh, test.wantUpdateV1); err != nil {
				t.Fatal(err)
			}

			// Cancel the second watch and force the management server to push a
			// redundant update for the resource being watched. Neither of the
			// two watch callbacks should be invoked.
			ldsCancel2()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyNoListenerUpdate(ctx, lw1.updateCh); err != nil {
				t.Fatal(err)
			}
			if err := verifyNoListenerUpdate(ctx, lw2.updateCh); err != nil {
				t.Fatal(err)
			}

			// Update to the resource being watched. The un-cancelled callback
			// should be invoked while the cancelled one should not be.
			resources = e2e.UpdateOptions{
				NodeID:         nodeID,
				Listeners:      []*v3listenerpb.Listener{test.updatedWatchedResource},
				SkipValidation: true,
			}
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
			}
			if err := verifyListenerUpdate(ctx, lw1.updateCh, test.wantUpdateV2); err != nil {
				t.Fatal(err)
			}
			if err := verifyNoListenerUpdate(ctx, lw2.updateCh); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestLDSWatch_ThreeWatchesForDifferentResourceNames covers the case with three
// watchers (two watchers for one resource, and the third watcher for another
// resource), exist across two listener resources.  The test verifies that an
// update from the management server containing both resources results in the
// invocation of all watch callbacks.
//
// The test is run with both old and new style names.
func (s) TestLDSWatch_ThreeWatchesForDifferentResourceNames(t *testing.T) {
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

	// Register two watches for the same listener resource and have the
	// callbacks push the received updates on to a channel.
	lw1 := newListenerWatcher()
	ldsCancel1 := xdsresource.WatchListener(client, ldsName, lw1)
	defer ldsCancel1()
	lw2 := newListenerWatcher()
	ldsCancel2 := xdsresource.WatchListener(client, ldsName, lw2)
	defer ldsCancel2()

	// Register the third watch for a different listener resource.
	ldsNameNewStyle := makeNewStyleLDSName(authority)
	lw3 := newListenerWatcher()
	ldsCancel3 := xdsresource.WatchListener(client, ldsNameNewStyle, lw3)
	defer ldsCancel3()

	// Configure the management server to return two listener resources,
	// corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Listeners: []*v3listenerpb.Listener{
			e2e.DefaultClientListener(ldsName, rdsName),
			e2e.DefaultClientListener(ldsNameNewStyle, rdsName),
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
	wantUpdate := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyListenerUpdate(ctx, lw3.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestLDSWatch_ResourceCaching covers the case where a watch is registered for
// a resource which is already present in the cache.  The test verifies that the
// watch callback is invoked with the contents from the cache, instead of a
// request being sent to the management server.
func (s) TestLDSWatch_ResourceCaching(t *testing.T) {
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

	// Register a watch for a listener resource and have the watch
	// callback push the received update on to a channel.
	lw1 := newListenerWatcher()
	ldsCancel1 := xdsresource.WatchListener(client, ldsName, lw1)
	defer ldsCancel1()

	// Configure the management server to return a single listener
	// resource, corresponding to the one we registered a watch for.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultClientListener(ldsName, rdsName)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update.
	wantUpdate := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	select {
	case <-ctx.Done():
		t.Fatal("timeout when waiting for receipt of ACK at the management server")
	case <-firstAckReceived.Done():
	}

	// Register another watch for the same resource. This should get the update
	// from the cache.
	lw2 := newListenerWatcher()
	ldsCancel2 := xdsresource.WatchListener(client, ldsName, lw2)
	defer ldsCancel2()
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate); err != nil {
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

// TestLDSWatch_ExpiryTimerFiresBeforeResponse tests the case where the client
// does not receive an LDS response for the request that it sends. The test
// verifies that the watch callback is invoked with an error once the
// watchExpiryTimer fires.
func (s) TestLDSWatch_ExpiryTimerFiresBeforeResponse(t *testing.T) {
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
	lw := newListenerWatcher()
	ldsCancel := xdsresource.WatchListener(client, ldsName, lw)
	defer ldsCancel()

	// Wait for the watch expiry timer to fire.
	<-time.After(defaultTestWatchExpiryTimeout)

	// Verify that an empty update with the expected error is received.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantErr := xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "")
	if err := verifyListenerUpdate(ctx, lw.updateCh, listenerUpdateErrTuple{err: wantErr}); err != nil {
		t.Fatal(err)
	}
}

// TestLDSWatch_ValidResponseCancelsExpiryTimerBehavior tests the case where the
// client receives a valid LDS response for the request that it sends. The test
// verifies that the behavior associated with the expiry timer (i.e, callback
// invocation with error) does not take place.
func (s) TestLDSWatch_ValidResponseCancelsExpiryTimerBehavior(t *testing.T) {
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

	// Register a watch for a listener resource and have the watch
	// callback push the received update on to a channel.
	lw := newListenerWatcher()
	ldsCancel := xdsresource.WatchListener(client, ldsName, lw)
	defer ldsCancel()

	// Configure the management server to return a single listener
	// resource, corresponding to the one we registered a watch for.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultClientListener(ldsName, rdsName)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update.
	wantUpdate := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}

	// Wait for the watch expiry timer to fire, and verify that the callback is
	// not invoked.
	<-time.After(defaultTestWatchExpiryTimeout)
	if err := verifyNoListenerUpdate(ctx, lw.updateCh); err != nil {
		t.Fatal(err)
	}
}

// TestLDSWatch_ResourceRemoved covers the cases where a resource being watched
// is removed from the management server. The test verifies the following
// scenarios:
//  1. Removing a resource should trigger the watch callback with a resource
//     removed error. It should not trigger the watch callback for an unrelated
//     resource.
//  2. An update to another resource should result in the invocation of the watch
//     callback associated with that resource.  It should not result in the
//     invocation of the watch callback associated with the deleted resource.
//
// The test is run with both old and new style names.
func (s) TestLDSWatch_ResourceRemoved(t *testing.T) {
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

	// Register two watches for two listener resources and have the
	// callbacks push the received updates on to a channel.
	resourceName1 := ldsName
	lw1 := newListenerWatcher()
	ldsCancel1 := xdsresource.WatchListener(client, resourceName1, lw1)
	defer ldsCancel1()

	resourceName2 := makeNewStyleLDSName(authority)
	lw2 := newListenerWatcher()
	ldsCancel2 := xdsresource.WatchListener(client, resourceName2, lw2)
	defer ldsCancel2()

	// Configure the management server to return two listener resources,
	// corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Listeners: []*v3listenerpb.Listener{
			e2e.DefaultClientListener(resourceName1, rdsName),
			e2e.DefaultClientListener(resourceName2, rdsName),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for both watchers. The two
	// resources returned differ only in the resource name. Therefore the
	// expected update is the same for both watchers.
	wantUpdate := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw1.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}

	// Remove the first listener resource on the management server.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultClientListener(resourceName2, rdsName)},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// The first watcher should receive a resource removed error, while the
	// second watcher should not see an update.
	if err := verifyListenerUpdate(ctx, lw1.updateCh, listenerUpdateErrTuple{
		err: xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, ""),
	}); err != nil {
		t.Fatal(err)
	}
	if err := verifyNoListenerUpdate(ctx, lw2.updateCh); err != nil {
		t.Fatal(err)
	}

	// Update the second listener resource on the management server. The first
	// watcher should not see an update, while the second watcher should.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultClientListener(resourceName2, "new-rds-resource")},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}
	if err := verifyNoListenerUpdate(ctx, lw1.updateCh); err != nil {
		t.Fatal(err)
	}
	wantUpdate = listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: "new-rds-resource",
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestLDSWatch_NACKError covers the case where an update from the management
// server is NACK'ed by the xdsclient. The test verifies that the error is
// propagated to the watcher.
func (s) TestLDSWatch_NACKError(t *testing.T) {
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

	// Register a watch for a listener resource and have the watch
	// callback push the received update on to a channel.
	lw := newListenerWatcher()
	ldsCancel := xdsresource.WatchListener(client, ldsName, lw)
	defer ldsCancel()

	// Configure the management server to return a single listener resource
	// which is expected to be NACKed by the client.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{badListenerResource(t, ldsName)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify that the expected error is propagated to the watcher.
	u, err := lw.updateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for a listener resource from the management server: %v", err)
	}
	gotErr := u.(listenerUpdateErrTuple).err
	if gotErr == nil || !strings.Contains(gotErr.Error(), wantListenerNACKErr) {
		t.Fatalf("update received with error: %v, want %q", gotErr, wantListenerNACKErr)
	}
}

// TestLDSWatch_PartialValid covers the case where a response from the
// management server contains both valid and invalid resources and is expected
// to be NACK'ed by the xdsclient. The test verifies that watchers corresponding
// to the valid resource receive the update, while watchers corresponding to the
// invalid resource receive an error.
func (s) TestLDSWatch_PartialValid(t *testing.T) {
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

	// Register two watches for listener resources. The first watch is expected
	// to receive an error because the received resource is NACKed. The second
	// watch is expected to get a good update.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	badResourceName := ldsName
	lw1 := newListenerWatcher()
	ldsCancel1 := xdsresource.WatchListener(client, badResourceName, lw1)
	defer ldsCancel1()
	goodResourceName := makeNewStyleLDSName(authority)
	lw2 := newListenerWatcher()
	ldsCancel2 := xdsresource.WatchListener(client, goodResourceName, lw2)
	defer ldsCancel2()

	// Configure the management server with two listener resources. One of these
	// is a bad resource causing the update to be NACKed.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Listeners: []*v3listenerpb.Listener{
			badListenerResource(t, badResourceName),
			e2e.DefaultClientListener(goodResourceName, rdsName),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify that the expected error is propagated to the watcher which
	// requested for the bad resource.
	u, err := lw1.updateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout when waiting for a listener resource from the management server: %v", err)
	}
	gotErr := u.(listenerUpdateErrTuple).err
	if gotErr == nil || !strings.Contains(gotErr.Error(), wantListenerNACKErr) {
		t.Fatalf("update received with error: %v, want %q", gotErr, wantListenerNACKErr)
	}

	// Verify that the watcher watching the good resource receives a good
	// update.
	wantUpdate := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate); err != nil {
		t.Fatal(err)
	}
}

// TestLDSWatch_PartialResponse covers the case where a response from the
// management server does not contain all requested resources. LDS responses are
// supposed to contain all requested resources, and the absence of one usually
// indicates that the management server does not know about it. In cases where
// the server has never responded with this resource before, the xDS client is
// expected to wait for the watch timeout to expire before concluding that the
// resource does not exist on the server
func (s) TestLDSWatch_PartialResponse(t *testing.T) {
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

	// Register two watches for two listener resources and have the
	// callbacks push the received updates on to a channel.
	resourceName1 := ldsName
	lw1 := newListenerWatcher()
	ldsCancel1 := xdsresource.WatchListener(client, resourceName1, lw1)
	defer ldsCancel1()

	resourceName2 := makeNewStyleLDSName(authority)
	lw2 := newListenerWatcher()
	ldsCancel2 := xdsresource.WatchListener(client, resourceName2, lw2)
	defer ldsCancel2()

	// Configure the management server to return only one of the two listener
	// resources, corresponding to the registered watches.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Listeners: []*v3listenerpb.Listener{
			e2e.DefaultClientListener(resourceName1, rdsName),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for first watcher.
	wantUpdate1 := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw1.updateCh, wantUpdate1); err != nil {
		t.Fatal(err)
	}

	// Verify that the second watcher does not get an update with an error.
	if err := verifyNoListenerUpdate(ctx, lw2.updateCh); err != nil {
		t.Fatal(err)
	}

	// Configure the management server to return two listener resources,
	// corresponding to the registered watches.
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Listeners: []*v3listenerpb.Listener{
			e2e.DefaultClientListener(resourceName1, rdsName),
			e2e.DefaultClientListener(resourceName2, rdsName),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Verify the contents of the received update for the second watcher.
	wantUpdate2 := listenerUpdateErrTuple{
		update: xdsresource.ListenerUpdate{
			RouteConfigName: rdsName,
			HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
		},
	}
	if err := verifyListenerUpdate(ctx, lw2.updateCh, wantUpdate2); err != nil {
		t.Fatal(err)
	}

	// Verify that the first watcher gets no update, as the first resource did
	// not change.
	if err := verifyNoListenerUpdate(ctx, lw1.updateCh); err != nil {
		t.Fatal(err)
	}
}
