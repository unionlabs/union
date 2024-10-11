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
	"testing"

	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/fakeserver"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal"
	xdstestutils "google.golang.org/grpc/xds/internal/testutils"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/types/known/anypb"

	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
)

var (
	// Resource type implementations retrieved from the resource type map in the
	// internal package, which is initialized when the individual resource types
	// are created.
	listenerResourceType    = internal.ResourceTypeMapForTesting[version.V3ListenerURL].(xdsresource.Type)
	routeConfigResourceType = internal.ResourceTypeMapForTesting[version.V3RouteConfigURL].(xdsresource.Type)
)

// This route configuration watcher registers two watches corresponding to the
// names passed in at creation time on a valid update.
type testRouteConfigWatcher struct {
	client           xdsclient.XDSClient
	name1, name2     string
	rcw1, rcw2       *routeConfigWatcher
	cancel1, cancel2 func()
	updateCh         *testutils.Channel
}

func newTestRouteConfigWatcher(client xdsclient.XDSClient, name1, name2 string) *testRouteConfigWatcher {
	return &testRouteConfigWatcher{
		client:   client,
		name1:    name1,
		name2:    name2,
		rcw1:     newRouteConfigWatcher(),
		rcw2:     newRouteConfigWatcher(),
		updateCh: testutils.NewChannel(),
	}
}

func (rw *testRouteConfigWatcher) OnUpdate(update *xdsresource.RouteConfigResourceData, onDone xdsresource.OnDoneFunc) {
	rw.updateCh.Send(routeConfigUpdateErrTuple{update: update.Resource})

	rw.cancel1 = xdsresource.WatchRouteConfig(rw.client, rw.name1, rw.rcw1)
	rw.cancel2 = xdsresource.WatchRouteConfig(rw.client, rw.name2, rw.rcw2)
	onDone()
}

func (rw *testRouteConfigWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	// When used with a go-control-plane management server that continuously
	// resends resources which are NACKed by the xDS client, using a `Replace()`
	// here and in OnResourceDoesNotExist() simplifies tests which will have
	// access to the most recently received error.
	rw.updateCh.Replace(routeConfigUpdateErrTuple{err: err})
	onDone()
}

func (rw *testRouteConfigWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	rw.updateCh.Replace(routeConfigUpdateErrTuple{err: xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "RouteConfiguration not found in received response")})
	onDone()
}

func (rw *testRouteConfigWatcher) cancel() {
	rw.cancel1()
	rw.cancel2()
}

// TestWatchCallAnotherWatch tests the scenario where a watch is registered for
// a resource, and more watches are registered from the first watch's callback.
// The test verifies that this scenario does not lead to a deadlock.
func (s) TestWatchCallAnotherWatch(t *testing.T) {
	// Start an xDS management server and set the option to allow it to respond
	// to requests which only specify a subset of the configured resources.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

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

	// Configure the management server to respond with route config resources.
	ldsNameNewStyle := makeNewStyleLDSName(authority)
	rdsNameNewStyle := makeNewStyleRDSName(authority)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Routes: []*v3routepb.RouteConfiguration{
			e2e.DefaultRouteConfig(rdsName, ldsName, cdsName),
			e2e.DefaultRouteConfig(rdsNameNewStyle, ldsNameNewStyle, cdsName),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatalf("Failed to update management server with resources: %v, err: %v", resources, err)
	}

	// Create a route configuration watcher that registers two more watches from
	// the OnUpdate callback:
	// - one for the same resource name as this watch, which would be
	//   satisfied from xdsClient cache
	// - the other for a different resource name, which would be
	//   satisfied from the server
	rw := newTestRouteConfigWatcher(client, rdsName, rdsNameNewStyle)
	defer rw.cancel()
	rdsCancel := xdsresource.WatchRouteConfig(client, rdsName, rw)
	defer rdsCancel()

	// Verify the contents of the received update for the all watchers.
	wantUpdate12 := routeConfigUpdateErrTuple{
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
	wantUpdate3 := routeConfigUpdateErrTuple{
		update: xdsresource.RouteConfigUpdate{
			VirtualHosts: []*xdsresource.VirtualHost{
				{
					Domains: []string{ldsNameNewStyle},
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
	if err := verifyRouteConfigUpdate(ctx, rw.updateCh, wantUpdate12); err != nil {
		t.Fatal(err)
	}
	if err := verifyRouteConfigUpdate(ctx, rw.rcw1.updateCh, wantUpdate12); err != nil {
		t.Fatal(err)
	}
	if err := verifyRouteConfigUpdate(ctx, rw.rcw2.updateCh, wantUpdate3); err != nil {
		t.Fatal(err)
	}
}

// TestNodeProtoSentOnlyInFirstRequest verifies that a non-empty node proto gets
// sent only on the first discovery request message on the ADS stream.
//
// It also verifies the same behavior holds after a stream restart.
func (s) TestNodeProtoSentOnlyInFirstRequest(t *testing.T) {
	// Create a restartable listener which can close existing connections.
	l, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	lis := testutils.NewRestartableListener(l)

	// Start a fake xDS management server with the above restartable listener.
	//
	// We are unable to use the go-control-plane server here, because it caches
	// the node proto received in the first request message and adds it to
	// subsequent requests before invoking the OnStreamRequest() callback.
	// Therefore we cannot verify what is sent by the xDS client.
	mgmtServer, cleanup, err := fakeserver.StartServer(lis)
	if err != nil {
		t.Fatalf("Failed to start fake xDS server: %v", err)
	}
	defer cleanup()

	// Create a bootstrap file in a temporary directory.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create an xDS client with the above bootstrap contents.
	client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer close()

	const (
		serviceName     = "my-service-client-side-xds"
		routeConfigName = "route-" + serviceName
		clusterName     = "cluster-" + serviceName
	)

	// Register a watch for the Listener resource.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	watcher := xdstestutils.NewTestResourceWatcher()
	client.WatchResource(listenerResourceType, serviceName, watcher)

	// Ensure the watch results in a discovery request with an empty node proto.
	if err := readDiscoveryResponseAndCheckForNonEmptyNodeProto(ctx, mgmtServer.XDSRequestChan); err != nil {
		t.Fatal(err)
	}

	// Configure a listener resource on the fake xDS server.
	lisAny, err := anypb.New(e2e.DefaultClientListener(serviceName, routeConfigName))
	if err != nil {
		t.Fatalf("Failed to marshal listener resource into an Any proto: %v", err)
	}
	mgmtServer.XDSResponseChan <- &fakeserver.Response{
		Resp: &v3discoverypb.DiscoveryResponse{
			TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
			VersionInfo: "1",
			Resources:   []*anypb.Any{lisAny},
		},
	}

	// The xDS client is expected to ACK the Listener resource. The discovery
	// request corresponding to the ACK must contain a nil node proto.
	if err := readDiscoveryResponseAndCheckForEmptyNodeProto(ctx, mgmtServer.XDSRequestChan); err != nil {
		t.Fatal(err)
	}

	// Register a watch for a RouteConfiguration resource.
	client.WatchResource(routeConfigResourceType, routeConfigName, watcher)

	// Ensure the watch results in a discovery request with an empty node proto.
	if err := readDiscoveryResponseAndCheckForEmptyNodeProto(ctx, mgmtServer.XDSRequestChan); err != nil {
		t.Fatal(err)
	}

	// Configure the route configuration resource on the fake xDS server.
	rcAny, err := anypb.New(e2e.DefaultRouteConfig(routeConfigName, serviceName, clusterName))
	if err != nil {
		t.Fatalf("Failed to marshal route configuration resource into an Any proto: %v", err)
	}
	mgmtServer.XDSResponseChan <- &fakeserver.Response{
		Resp: &v3discoverypb.DiscoveryResponse{
			TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
			VersionInfo: "1",
			Resources:   []*anypb.Any{rcAny},
		},
	}

	// Ensure the discovery request for the ACK contains an empty node proto.
	if err := readDiscoveryResponseAndCheckForEmptyNodeProto(ctx, mgmtServer.XDSRequestChan); err != nil {
		t.Fatal(err)
	}

	// Stop the management server and expect the error callback to be invoked.
	lis.Stop()
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for the connection error to be propagated to the watcher")
	case <-watcher.ErrorCh:
	}

	// Restart the management server.
	lis.Restart()

	// The xDS client is expected to re-request previously requested resources.
	// Hence, we expect two DiscoveryRequest messages (one for the Listener and
	// one for the RouteConfiguration resource). The first message should contain
	// a non-nil node proto and the second should contain a nil-proto.
	//
	// And since we don't push any responses on the response channel of the fake
	// server, we do not expect any ACKs here.
	if err := readDiscoveryResponseAndCheckForNonEmptyNodeProto(ctx, mgmtServer.XDSRequestChan); err != nil {
		t.Fatal(err)
	}
	if err := readDiscoveryResponseAndCheckForEmptyNodeProto(ctx, mgmtServer.XDSRequestChan); err != nil {
		t.Fatal(err)
	}
}

// readDiscoveryResponseAndCheckForEmptyNodeProto reads a discovery request
// message out of the provided reqCh. It returns an error if it fails to read a
// message before the context deadline expires, or if the read message contains
// a non-empty node proto.
func readDiscoveryResponseAndCheckForEmptyNodeProto(ctx context.Context, reqCh *testutils.Channel) error {
	v, err := reqCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("Timeout when waiting for a DiscoveryRequest message")
	}
	req := v.(*fakeserver.Request).Req.(*v3discoverypb.DiscoveryRequest)
	if node := req.GetNode(); node != nil {
		return fmt.Errorf("Node proto received in DiscoveryRequest message is %v, want empty node proto", node)
	}
	return nil
}

// readDiscoveryResponseAndCheckForNonEmptyNodeProto reads a discovery request
// message out of the provided reqCh. It returns an error if it fails to read a
// message before the context deadline expires, or if the read message contains
// an empty node proto.
func readDiscoveryResponseAndCheckForNonEmptyNodeProto(ctx context.Context, reqCh *testutils.Channel) error {
	v, err := reqCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("Timeout when waiting for a DiscoveryRequest message")
	}
	req := v.(*fakeserver.Request).Req.(*v3discoverypb.DiscoveryRequest)
	if node := req.GetNode(); node == nil {
		return fmt.Errorf("Empty node proto received in DiscoveryRequest message, want non-empty node proto")
	}
	return nil
}
