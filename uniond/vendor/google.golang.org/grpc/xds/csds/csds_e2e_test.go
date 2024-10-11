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

package csds_test

import (
	"context"
	"fmt"
	"io"
	"slices"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/xds/csds"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"

	v3adminpb "github.com/envoyproxy/go-control-plane/envoy/admin/v3"
	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3statuspb "github.com/envoyproxy/go-control-plane/envoy/service/status/v3"
	v3statuspbgrpc "github.com/envoyproxy/go-control-plane/envoy/service/status/v3"

	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter
)

const defaultTestTimeout = 5 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// The following watcher implementations are no-ops since we don't really care
// about the callback received by these watchers in the test. We only care
// whether CSDS reports the expected state.

type nopListenerWatcher struct{}

func (nopListenerWatcher) OnUpdate(_ *xdsresource.ListenerResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopListenerWatcher) OnError(_ error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopListenerWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

type nopRouteConfigWatcher struct{}

func (nopRouteConfigWatcher) OnUpdate(_ *xdsresource.RouteConfigResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopRouteConfigWatcher) OnError(_ error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopRouteConfigWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

type nopClusterWatcher struct{}

func (nopClusterWatcher) OnUpdate(_ *xdsresource.ClusterResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopClusterWatcher) OnError(_ error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopClusterWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

type nopEndpointsWatcher struct{}

func (nopEndpointsWatcher) OnUpdate(_ *xdsresource.EndpointsResourceData, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopEndpointsWatcher) OnError(_ error, onDone xdsresource.OnDoneFunc) {
	onDone()
}
func (nopEndpointsWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	onDone()
}

// This watcher writes the onDone callback on to a channel for the test to
// invoke it when it wants to unblock the next read on the ADS stream in the xDS
// client. This is particularly useful when a resource is NACKed, because the
// go-control-plane management server continuously resends the same resource in
// this case, and applying flow control from these watchers ensures that xDS
// client does not spend all of its time receiving and NACKing updates from the
// management server. This was indeed the case on arm64 (before we had support
// for ADS stream level flow control), and was causing CSDS to not receive any
// updates from the xDS client.
type blockingListenerWatcher struct {
	testCtxDone <-chan struct{}             // Closed when the test is done.
	onDoneCh    chan xdsresource.OnDoneFunc // Channel to write the onDone callback to.
}

func newBlockingListenerWatcher(testCtxDone <-chan struct{}) *blockingListenerWatcher {
	return &blockingListenerWatcher{
		testCtxDone: testCtxDone,
		onDoneCh:    make(chan xdsresource.OnDoneFunc, 1),
	}
}

func (w *blockingListenerWatcher) OnUpdate(_ *xdsresource.ListenerResourceData, onDone xdsresource.OnDoneFunc) {
	writeOnDone(w.testCtxDone, w.onDoneCh, onDone)
}
func (w *blockingListenerWatcher) OnError(_ error, onDone xdsresource.OnDoneFunc) {
	writeOnDone(w.testCtxDone, w.onDoneCh, onDone)
}
func (w *blockingListenerWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	writeOnDone(w.testCtxDone, w.onDoneCh, onDone)
}

// writeOnDone attempts to writes the onDone callback on the onDone channel. It
// returns when it can successfully write to the channel or when the test is
// done, which is signalled by testCtxDone being closed.
func writeOnDone(testCtxDone <-chan struct{}, onDoneCh chan xdsresource.OnDoneFunc, onDone xdsresource.OnDoneFunc) {
	select {
	case <-testCtxDone:
	case onDoneCh <- onDone:
	}
}

// Creates a gRPC server and starts serving a CSDS service implementation on it.
// Returns the address of the newly created gRPC server.
//
// Registers cleanup functions on t to stop the gRPC server and the CSDS
// implementation.
func startCSDSServer(t *testing.T) string {
	t.Helper()

	server := grpc.NewServer()
	t.Cleanup(server.Stop)

	csdss, err := csds.NewClientStatusDiscoveryServer()
	if err != nil {
		t.Fatalf("Failed to create CSDS service implementation: %v", err)
	}
	v3statuspbgrpc.RegisterClientStatusDiscoveryServiceServer(server, csdss)
	t.Cleanup(csdss.Close)

	// Create a local listener and pass it to Serve().
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	go func() {
		if err := server.Serve(lis); err != nil {
			t.Errorf("Serve() failed: %v", err)
		}
	}()
	return lis.Addr().String()
}

func startCSDSClientStream(ctx context.Context, t *testing.T, serverAddr string) v3statuspbgrpc.ClientStatusDiscoveryService_StreamClientStatusClient {
	conn, err := grpc.NewClient(serverAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial CSDS server %q: %v", serverAddr, err)
	}

	client := v3statuspbgrpc.NewClientStatusDiscoveryServiceClient(conn)
	stream, err := client.StreamClientStatus(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("Failed to create a stream for CSDS: %v", err)
	}
	t.Cleanup(func() { conn.Close() })
	return stream
}

// Tests CSDS functionality. The test performs the following:
//   - Spins up a management server and creates two xDS clients talking to it.
//   - Registers a set of watches on the xDS clients, and verifies that the CSDS
//     response reports resources in REQUESTED state.
//   - Configures resources on the management server corresponding to the ones
//     being watched by the clients, and verifies that the CSDS response reports
//     resources in ACKED state.
//
// For the above operations, the test also verifies that the client_scope field
// in the CSDS response is populated appropriately.
func (s) TestCSDS(t *testing.T) {
	// Spin up a xDS management server on a local port.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create a bootstrap contents pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create two xDS clients, with different names. These should end up
	// creating two different xDS clients.
	const xdsClient1Name = "xds-csds-client-1"
	xdsClient1, xdsClose1, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     xdsClient1Name,
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer xdsClose1()
	const xdsClient2Name = "xds-csds-client-2"
	xdsClient2, xdsClose2, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     xdsClient2Name,
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer xdsClose2()

	// Start a CSDS server and create a client stream to it.
	addr := startCSDSServer(t)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream := startCSDSClientStream(ctx, t, addr)

	// Verify that the xDS client reports an empty config.
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
				ClientScope: xdsClient1Name,
			},
			{
				Node:        wantNode,
				ClientScope: xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}

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

	// Register watches on the xDS clients for two resources of each type.
	for _, xdsC := range []xdsclient.XDSClient{xdsClient1, xdsClient2} {
		for _, target := range ldsTargets {
			xdsresource.WatchListener(xdsC, target, nopListenerWatcher{})
		}
		for _, target := range rdsTargets {
			xdsresource.WatchRouteConfig(xdsC, target, nopRouteConfigWatcher{})
		}
		for _, target := range cdsTargets {
			xdsresource.WatchCluster(xdsC, target, nopClusterWatcher{})
		}
		for _, target := range edsTargets {
			xdsresource.WatchEndpoints(xdsC, target, nopEndpointsWatcher{})
		}
	}

	// Verify that the xDS client reports the resources as being in "Requested"
	// state, and in version "0".
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
				ClientScope:       xdsClient1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}

	// Configure the management server with two resources of each type,
	// corresponding to the watches registered above.
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: listeners,
		Routes:    routes,
		Clusters:  clusters,
		Endpoints: endpoints,
	}); err != nil {
		t.Fatal(err)
	}

	// Verify that the xDS client reports the resources as being in "ACKed"
	// state, and in version "1".
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
				ClientScope:       xdsClient1Name,
			},
			{
				Node:              wantNode,
				GenericXdsConfigs: wantConfigs,
				ClientScope:       xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}
}

// Tests CSDS functionality. The test performs the following:
//   - Spins up a management server and creates two xDS clients talking to it.
//   - Registers one watch on each xDS client, and verifies that the CSDS
//     response reports resources in REQUESTED state.
//   - Configures two resources on the management server and verifies that the
//     CSDS response reports the resources as being in ACKED state.
//   - Updates one of two resources on the management server such that it is
//     expected to be NACKed by the client. Verifies that the CSDS response
//     contains one resource in ACKED state and one in NACKED state.
//
// For the above operations, the test also verifies that the client_scope field
// in the CSDS response is populated appropriately.
//
// This test does a bunch of similar things to the previous test, but has
// reduced complexity because of having to deal with a single resource type.
// This makes is possible to test the NACKing a resource (which results in
// continuous resending of the resource by the go-control-plane management
// server), in an easier and less flaky way.
func (s) TestCSDS_NACK(t *testing.T) {
	// Spin up a xDS management server on a local port.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create a bootstrap contents pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create two xDS clients, with different names. These should end up
	// creating two different xDS clients.
	const xdsClient1Name = "xds-csds-client-1"
	xdsClient1, xdsClose1, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     xdsClient1Name,
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer xdsClose1()
	const xdsClient2Name = "xds-csds-client-2"
	xdsClient2, xdsClose2, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     xdsClient2Name,
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	defer xdsClose2()

	// Start a CSDS server and create a client stream to it.
	addr := startCSDSServer(t)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream := startCSDSClientStream(ctx, t, addr)

	// Verify that the xDS client reports an empty config.
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
				ClientScope: xdsClient1Name,
			},
			{
				Node:        wantNode,
				ClientScope: xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}

	// Initialize the xDS resources to be used in this test.
	const ldsTarget0, ldsTarget1 = "lds.target.good:0000", "lds.target.good:1111"
	listener0 := e2e.DefaultClientListener(ldsTarget0, "rds-name")
	listener1 := e2e.DefaultClientListener(ldsTarget1, "rds-name")
	listenerAny0 := testutils.MarshalAny(t, listener0)
	listenerAny1 := testutils.MarshalAny(t, listener1)

	// Register the watchers, one for each xDS client.
	watcher1 := nopListenerWatcher{}
	watcher2 := newBlockingListenerWatcher(ctx.Done())
	xdsresource.WatchListener(xdsClient1, ldsTarget0, watcher1)
	xdsresource.WatchListener(xdsClient2, ldsTarget1, watcher2)

	// Verify that the xDS client reports the resources as being in "Requested"
	// state, and in version "0".
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node: wantNode,
				GenericXdsConfigs: []*v3statuspb.ClientConfig_GenericXdsConfig{
					makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTarget0, "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
				},
				ClientScope: xdsClient1Name,
			},
			{
				Node: wantNode,
				GenericXdsConfigs: []*v3statuspb.ClientConfig_GenericXdsConfig{
					makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTarget1, "", v3adminpb.ClientResourceStatus_REQUESTED, nil, nil),
				},
				ClientScope: xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}

	// Configure the management server with two listener resources corresponding
	// to the watches registered above.
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{listener0, listener1},
		SkipValidation: true,
	}); err != nil {
		t.Fatal(err)
	}

	// Verify that the xDS client reports the resources as being in "ACKed"
	// state, and in version "1".
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node: wantNode,
				GenericXdsConfigs: []*v3statuspb.ClientConfig_GenericXdsConfig{
					makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTarget0, "1", v3adminpb.ClientResourceStatus_ACKED, listenerAny0, nil),
				},
				ClientScope: xdsClient1Name,
			},
			{
				Node: wantNode,
				GenericXdsConfigs: []*v3statuspb.ClientConfig_GenericXdsConfig{
					makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTarget1, "1", v3adminpb.ClientResourceStatus_ACKED, listenerAny1, nil),
				},
				ClientScope: xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}

	// Unblock reads on the ADS stream by calling the onDone callback sent to
	// the watcher.
	select {
	case <-ctx.Done():
		t.Fatal("Timed out waiting for watch callback")
	case onDone := <-watcher2.onDoneCh:
		onDone()
	}

	// Update the second resource with an empty ApiListener field which is
	// expected to be NACK'ed by the xDS client.
	listener1.ApiListener = nil
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{listener0, listener1},
		SkipValidation: true,
	}); err != nil {
		t.Fatal(err)
	}

	// Wait for the update to reach the watchers.
	select {
	case <-ctx.Done():
		t.Fatal("Timed out waiting for watch callback")
	case onDone := <-watcher2.onDoneCh:
		onDone()
	}

	// Verify that the xDS client reports the first listener resource as being
	// ACKed and the second listener resource as being NACKed.  The version for
	// the ACKed resource would be "2", while that for the NACKed resource would
	// be "1". In the NACKed resource, the version which is NACKed is stored in
	// the ErrorState field.
	wantResp = &v3statuspb.ClientStatusResponse{
		Config: []*v3statuspb.ClientConfig{
			{
				Node: wantNode,
				GenericXdsConfigs: []*v3statuspb.ClientConfig_GenericXdsConfig{
					makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTarget0, "2", v3adminpb.ClientResourceStatus_ACKED, listenerAny0, nil),
				},
				ClientScope: xdsClient1Name,
			},
			{
				Node: wantNode,
				GenericXdsConfigs: []*v3statuspb.ClientConfig_GenericXdsConfig{
					makeGenericXdsConfig("type.googleapis.com/envoy.config.listener.v3.Listener", ldsTarget1, "1", v3adminpb.ClientResourceStatus_NACKED, listenerAny1, &v3adminpb.UpdateFailureState{VersionInfo: "2"}),
				},
				ClientScope: xdsClient2Name,
			},
		},
	}
	if err := checkClientStatusResponse(ctx, stream, wantResp); err != nil {
		t.Fatal(err)
	}
}

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

// Repeatedly sends CSDS requests and receives CSDS responses on the provided
// stream and verifies that the response matches `want`. Returns an error if
// sending or receiving on the stream fails, or if the context expires before a
// response matching `want` is received.
//
// Expects client configs in `want` to be sorted on `client_scope` and the
// resource dump to be sorted on type_url and resource name.
func checkClientStatusResponse(ctx context.Context, stream v3statuspbgrpc.ClientStatusDiscoveryService_StreamClientStatusClient, want *v3statuspb.ClientStatusResponse) error {
	var cmpOpts = cmp.Options{
		protocmp.Transform(),
		protocmp.IgnoreFields((*v3statuspb.ClientConfig_GenericXdsConfig)(nil), "last_updated"),
		protocmp.IgnoreFields((*v3adminpb.UpdateFailureState)(nil), "last_update_attempt", "details"),
	}

	var lastErr error
	for ; ctx.Err() == nil; <-time.After(100 * time.Millisecond) {
		if err := stream.Send(&v3statuspb.ClientStatusRequest{Node: nil}); err != nil {
			if err != io.EOF {
				return fmt.Errorf("failed to send ClientStatusRequest: %v", err)
			}
			// If the stream has closed, we call Recv() until it returns a non-nil
			// error to get the actual error on the stream.
			for {
				if _, err := stream.Recv(); err != nil {
					return fmt.Errorf("failed to recv ClientStatusResponse: %v", err)
				}
			}
		}
		got, err := stream.Recv()
		if err != nil {
			return fmt.Errorf("failed to recv ClientStatusResponse: %v", err)
		}
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
	return fmt.Errorf("timeout when waiting for resource dump to reach expected state: ctxErr: %v, otherErr: %v", ctx.Err(), lastErr)
}

func (s) TestCSDSNoXDSClient(t *testing.T) {
	// Create a bootstrap file in a temporary directory. Since we pass an empty
	// bootstrap configuration, it will fail xDS client creation because the
	// `server_uri` field is unset.
	testutils.CreateBootstrapFileForTesting(t, []byte(``))

	// Start a CSDS server and create a client stream to it.
	addr := startCSDSServer(t)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream := startCSDSClientStream(ctx, t, addr)

	if err := stream.Send(&v3statuspb.ClientStatusRequest{Node: nil}); err != nil {
		t.Fatalf("Failed to send ClientStatusRequest: %v", err)
	}
	r, err := stream.Recv()
	if err != nil {
		// io.EOF is not ok.
		t.Fatalf("Failed to recv ClientStatusResponse: %v", err)
	}
	if n := len(r.Config); n != 0 {
		t.Fatalf("got %d configs, want 0: %v", n, prototext.Format(r))
	}
}
