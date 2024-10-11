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
	"encoding/json"
	"fmt"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/xds"

	clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const (
	serviceName = "my-service-xds"
	rdsName     = "route-" + serviceName
	cdsName1    = "cluster1-" + serviceName
	cdsName2    = "cluster2-" + serviceName
	edsName1    = "eds1-" + serviceName
	edsName2    = "eds2-" + serviceName
)

var (
	// This route configuration resource contains two routes:
	// - a route for the EmptyCall rpc, to be sent to cluster1
	// - a route for the UnaryCall rpc, to be sent to cluster2
	defaultRouteConfigWithTwoRoutes = &routepb.RouteConfiguration{
		Name: rdsName,
		VirtualHosts: []*routepb.VirtualHost{{
			Domains: []string{serviceName},
			Routes: []*routepb.Route{
				{
					Match: &routepb.RouteMatch{PathSpecifier: &routepb.RouteMatch_Prefix{Prefix: "/grpc.testing.TestService/EmptyCall"}},
					Action: &routepb.Route_Route{Route: &routepb.RouteAction{
						ClusterSpecifier: &routepb.RouteAction_Cluster{Cluster: cdsName1},
					}},
				},
				{
					Match: &routepb.RouteMatch{PathSpecifier: &routepb.RouteMatch_Prefix{Prefix: "/grpc.testing.TestService/UnaryCall"}},
					Action: &routepb.Route_Route{Route: &routepb.RouteAction{
						ClusterSpecifier: &routepb.RouteAction_Cluster{Cluster: cdsName2},
					}},
				},
			},
		}},
	}
)

// This test runs subtest each for a Listener resource and a Cluster resource deletion
// in the response from the server for the following cases:
//   - testResourceDeletionIgnored: When ignore_resource_deletion is set, the
//     xDSClient should not delete the resource.
//   - testResourceDeletionNotIgnored: When ignore_resource_deletion is unset,
//     the xDSClient should delete the resource.
//
// Resource deletion is only applicable to Listener and Cluster resources.
func (s) TestIgnoreResourceDeletionOnClient(t *testing.T) {
	server1 := stubserver.StartTestService(t, nil)
	t.Cleanup(server1.Stop)

	server2 := stubserver.StartTestService(t, nil)
	t.Cleanup(server2.Stop)

	initialResourceOnServer := func(nodeID string) e2e.UpdateOptions {
		return e2e.UpdateOptions{
			NodeID:    nodeID,
			Listeners: []*listenerpb.Listener{e2e.DefaultClientListener(serviceName, rdsName)},
			Routes:    []*routepb.RouteConfiguration{defaultRouteConfigWithTwoRoutes},
			Clusters: []*clusterpb.Cluster{
				e2e.DefaultCluster(cdsName1, edsName1, e2e.SecurityLevelNone),
				e2e.DefaultCluster(cdsName2, edsName2, e2e.SecurityLevelNone),
			},
			Endpoints: []*endpointpb.ClusterLoadAssignment{
				e2e.DefaultEndpoint(edsName1, "localhost", []uint32{testutils.ParsePort(t, server1.Address)}),
				e2e.DefaultEndpoint(edsName2, "localhost", []uint32{testutils.ParsePort(t, server2.Address)}),
			},
			SkipValidation: true,
		}
	}

	tests := []struct {
		name           string
		updateResource func(r *e2e.UpdateOptions)
	}{
		{
			name: "listener",
			updateResource: func(r *e2e.UpdateOptions) {
				r.Listeners = nil
			},
		},
		{
			name: "cluster",
			updateResource: func(r *e2e.UpdateOptions) {
				r.Clusters = nil
			},
		},
	}
	for _, test := range tests {
		t.Run(fmt.Sprintf("%s resource deletion ignored", test.name), func(t *testing.T) {
			testResourceDeletionIgnored(t, initialResourceOnServer, test.updateResource)
		})
		t.Run(fmt.Sprintf("%s resource deletion not ignored", test.name), func(t *testing.T) {
			testResourceDeletionNotIgnored(t, initialResourceOnServer, test.updateResource)
		})
	}
}

// This subtest tests the scenario where the bootstrap config has "ignore_resource_deletion"
// set in "server_features" field. This subtest verifies that the resource was
// not deleted by the xDSClient when a resource is missing the xDS response and
// RPCs continue to succeed.
func testResourceDeletionIgnored(t *testing.T, initialResource func(string) e2e.UpdateOptions, updateResource func(r *e2e.UpdateOptions)) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	t.Cleanup(cancel)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})
	nodeID := uuid.New().String()
	bs := generateBootstrapContents(t, mgmtServer.Address, true, nodeID)
	xdsR := xdsResolverBuilder(t, bs)
	resources := initialResource(nodeID)

	// Update the management server with initial resources setup.
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsR))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v.", err)
	}
	t.Cleanup(func() { cc.Close() })

	if err := verifyRPCtoAllEndpoints(cc); err != nil {
		t.Fatal(err)
	}

	// Mutate resource and update on the server.
	updateResource(&resources)
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Make an RPC every 50ms for the next 500ms. This is to ensure that the
	// updated resource is received from the management server and is processed by
	// gRPC. Since resource deletions are ignored by the xDS client, we expect RPCs
	// to all endpoints to keep succeeding.
	timer := time.NewTimer(500 * time.Millisecond)
	ticker := time.NewTicker(50 * time.Millisecond)
	t.Cleanup(ticker.Stop)
	for {
		if err := verifyRPCtoAllEndpoints(cc); err != nil {
			t.Fatal(err)
		}
		select {
		case <-ctx.Done():
			return
		case <-timer.C:
			return
		case <-ticker.C:
		}
	}
}

// This subtest tests the scenario where the bootstrap config has "ignore_resource_deletion"
// not set in "server_features" field. This subtest verifies that the resource was
// deleted by the xDSClient when a resource is missing the xDS response and subsequent
// RPCs fail.
func testResourceDeletionNotIgnored(t *testing.T, initialResource func(string) e2e.UpdateOptions, updateResource func(r *e2e.UpdateOptions)) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout*1000)
	t.Cleanup(cancel)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})
	nodeID := uuid.New().String()
	bs := generateBootstrapContents(t, mgmtServer.Address, false, nodeID)
	xdsR := xdsResolverBuilder(t, bs)
	resources := initialResource(nodeID)

	// Update the management server with initial resources setup.
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsR))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	if err := verifyRPCtoAllEndpoints(cc); err != nil {
		t.Fatal(err)
	}

	// Mutate resource and update on the server.
	updateResource(&resources)
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Spin up go routines to verify RPCs fail after the update.
	client := testgrpc.NewTestServiceClient(cc)
	wg := sync.WaitGroup{}
	wg.Add(2)
	go func() {
		defer wg.Done()
		for ; ctx.Err() == nil; <-time.After(10 * time.Millisecond) {
			if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
				return
			}
		}
	}()
	go func() {
		defer wg.Done()
		for ; ctx.Err() == nil; <-time.After(10 * time.Millisecond) {
			if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
				return
			}
		}
	}()

	wg.Wait()
	if ctx.Err() != nil {
		t.Fatal("Context expired before RPCs failed.")
	}
}

// This helper generates a custom bootstrap config for the test.
func generateBootstrapContents(t *testing.T, serverURI string, ignoreResourceDeletion bool, nodeID string) []byte {
	t.Helper()
	var serverCfgs json.RawMessage
	if ignoreResourceDeletion {
		serverCfgs = []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}],
			"server_features": ["ignore_resource_deletion"]
		}]`, serverURI))
	} else {
		serverCfgs = []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, serverURI))

	}
	bootstrapContents, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers:                            serverCfgs,
		Node:                               []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		ServerListenerResourceNameTemplate: e2e.ServerListenerResourceNameTemplate,
	})
	if err != nil {
		t.Fatal(err)
	}
	return bootstrapContents
}

// This helper creates an XDS resolver Builder from the bootstrap config passed
// as parameter.
func xdsResolverBuilder(t *testing.T, bs []byte) resolver.Builder {
	t.Helper()
	resolverBuilder := internal.NewXDSResolverWithConfigForTesting.(func([]byte) (resolver.Builder, error))
	xdsR, err := resolverBuilder(bs)
	if err != nil {
		t.Fatalf("Creating xDS resolver for testing failed for config %q: %v", string(bs), err)
	}
	return xdsR
}

// This helper creates an xDS-enabled gRPC server using the listener and the
// bootstrap config passed. It then registers the test service on the newly
// created gRPC server and starts serving.
func setupGRPCServerWithModeChangeChannelAndServe(t *testing.T, bootstrapContents []byte, lis net.Listener) chan connectivity.ServingMode {
	t.Helper()
	updateCh := make(chan connectivity.ServingMode, 1)

	// Create a server option to get notified about serving mode changes.
	modeChangeOpt := xds.ServingModeCallback(func(addr net.Addr, args xds.ServingModeChangeArgs) {
		t.Logf("Serving mode for listener %q changed to %q, err: %v", addr.String(), args.Mode, args.Err)
		updateCh <- args.Mode
	})
	server, err := xds.NewGRPCServer(grpc.Creds(insecure.NewCredentials()), modeChangeOpt, xds.BootstrapContentsForTesting(bootstrapContents))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	t.Cleanup(server.Stop)
	testgrpc.RegisterTestServiceServer(server, &testService{})

	// Serve.
	go func() {
		if err := server.Serve(lis); err != nil {
			t.Errorf("Serve() failed: %v", err)
		}
	}()

	return updateCh
}

// This helper creates a new TCP listener. This helper also uses this listener to
// create a resource update with a listener resource. This helper returns the
// resource update and the TCP listener.
func resourceWithListenerForGRPCServer(t *testing.T, nodeID string) (e2e.UpdateOptions, net.Listener) {
	t.Helper()
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	t.Cleanup(func() { lis.Close() })
	host, port, err := hostPortFromListener(lis)
	if err != nil {
		t.Fatalf("Failed to retrieve host and port of listener at %q: %v", lis.Addr(), err)
	}
	listener := e2e.DefaultServerListener(host, port, e2e.SecurityLevelNone, "routeName")
	resources := e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*listenerpb.Listener{listener},
	}
	return resources, lis
}

// This test creates a gRPC server which provides server-side xDS functionality
// by talking to a custom management server. This tests the scenario where bootstrap
// config with "server_features" includes "ignore_resource_deletion". In which
// case, when the listener resource is deleted on the management server, the gRPC
// server should continue to serve RPCs.
func (s) TestListenerResourceDeletionOnServerIgnored(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})
	nodeID := uuid.New().String()
	bs := generateBootstrapContents(t, mgmtServer.Address, true, nodeID)
	xdsR := xdsResolverBuilder(t, bs)
	resources, lis := resourceWithListenerForGRPCServer(t, nodeID)
	modeChangeCh := setupGRPCServerWithModeChangeChannelAndServe(t, bs, lis)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Wait for the server to update to ServingModeServing mode.
	select {
	case <-ctx.Done():
		t.Fatal("Test timed out waiting for a server to change to ServingModeServing.")
	case mode := <-modeChangeCh:
		if mode != connectivity.ServingModeServing {
			t.Fatalf("Server switched to mode %v, want %v", mode, connectivity.ServingModeServing)
		}
	}

	// Create a ClientConn and make a successful RPCs.
	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsR))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	if err := verifyRPCtoAllEndpoints(cc); err != nil {
		t.Fatal(err)
	}

	// Update without a listener resource.
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*listenerpb.Listener{},
	}); err != nil {
		t.Fatal(err)
	}

	// Perform RPCs every 100 ms for 1s and verify that the serving mode does not
	// change on gRPC server.
	timer := time.NewTimer(500 * time.Millisecond)
	ticker := time.NewTicker(50 * time.Millisecond)
	t.Cleanup(ticker.Stop)
	for {
		if err := verifyRPCtoAllEndpoints(cc); err != nil {
			t.Fatal(err)
		}
		select {
		case <-timer.C:
			return
		case mode := <-modeChangeCh:
			t.Fatalf("Server switched to mode: %v when no switch was expected", mode)
		case <-ticker.C:
		}
	}
}

// This test creates a gRPC server which provides server-side xDS functionality
// by talking to a custom management server. This tests the scenario where bootstrap
// config with "server_features" does not include "ignore_resource_deletion". In
// which case, when the listener resource is deleted on the management server, the
// gRPC server should stop serving RPCs and switch mode to ServingModeNotServing.
func (s) TestListenerResourceDeletionOnServerNotIgnored(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})
	nodeID := uuid.New().String()
	bs := generateBootstrapContents(t, mgmtServer.Address, false, nodeID)
	xdsR := xdsResolverBuilder(t, bs)
	resources, lis := resourceWithListenerForGRPCServer(t, nodeID)
	updateCh := setupGRPCServerWithModeChangeChannelAndServe(t, bs, lis)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Wait for the listener to move to "serving" mode.
	select {
	case <-ctx.Done():
		t.Fatal("Test timed out waiting for a mode change update.")
	case mode := <-updateCh:
		if mode != connectivity.ServingModeServing {
			t.Fatalf("Listener received new mode %v, want %v", mode, connectivity.ServingModeServing)
		}
	}

	// Create a ClientConn and make a successful RPCs.
	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsR))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()
	if err := verifyRPCtoAllEndpoints(cc); err != nil {
		t.Fatal(err)
	}

	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*listenerpb.Listener{}, // empty listener resource
	}); err != nil {
		t.Fatal(err)
	}

	select {
	case <-ctx.Done():
		t.Fatalf("timed out waiting for a mode change update: %v", err)
	case mode := <-updateCh:
		if mode != connectivity.ServingModeNotServing {
			t.Fatalf("listener received new mode %v, want %v", mode, connectivity.ServingModeNotServing)
		}
	}
}

// This helper makes both UnaryCall and EmptyCall RPCs using the ClientConn that
// is passed to this function. This helper panics for any failed RPCs.
func verifyRPCtoAllEndpoints(cc grpc.ClientConnInterface) error {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		return fmt.Errorf("rpc EmptyCall() failed: %v", err)
	}
	if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		return fmt.Errorf("rpc UnaryCall() failed: %v", err)
	}
	return nil
}
