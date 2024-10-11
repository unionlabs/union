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

package server

import (
	"context"
	"fmt"
	"net"
	"strconv"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	xdsinternal "google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"

	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"

	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter
)

type verifyMode struct {
	modeCh chan connectivity.ServingMode
}

func (vm *verifyMode) verifyModeCallback(_ net.Addr, mode connectivity.ServingMode, _ error) {
	vm.modeCh <- mode
}

func hostPortFromListener(t *testing.T, lis net.Listener) (string, uint32) {
	t.Helper()

	host, p, err := net.SplitHostPort(lis.Addr().String())
	if err != nil {
		t.Fatalf("net.SplitHostPort(%s) failed: %v", lis.Addr().String(), err)
	}
	port, err := strconv.ParseInt(p, 10, 32)
	if err != nil {
		t.Fatalf("strconv.ParseInt(%s, 10, 32) failed: %v", p, err)
	}
	return host, uint32(port)
}

// TestListenerWrapper tests the listener wrapper. It configures the listener
// wrapper with a certain LDS, and makes sure that it requests the LDS name. It
// then receives an LDS resource that points to an RDS for Route Configuration.
// The listener wrapper should then start a watch for the RDS name. This should
// not trigger a mode change (the mode starts out non serving). Then a RDS
// resource is configured to return for the RDS name. This should transition the
// Listener Wrapper to READY.
func (s) TestListenerWrapper(t *testing.T) {
	mgmtServer, nodeID, ldsResourceNamesCh, rdsResourceNamesCh, xdsC := xdsSetupForTests(t)
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create a local TCP listener: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	modeCh := make(chan connectivity.ServingMode, 1)
	vm := verifyMode{
		modeCh: modeCh,
	}
	host, port := hostPortFromListener(t, lis)
	lisResourceName := fmt.Sprintf(e2e.ServerListenerResourceNameTemplate, net.JoinHostPort(host, strconv.Itoa(int(port))))
	params := ListenerWrapperParams{
		Listener:             lis,
		ListenerResourceName: lisResourceName,
		XDSClient:            xdsC,
		ModeCallback:         vm.verifyModeCallback,
	}
	l := NewListenerWrapper(params)
	if l == nil {
		t.Fatalf("NewListenerWrapper(%+v) returned nil", params)
	}
	defer l.Close()
	waitForResourceNames(ctx, t, ldsResourceNamesCh, []string{lisResourceName})
	// Configure the management server with a listener resource that specifies
	// the name of RDS resources that need to be resolved.
	listener := e2e.DefaultServerListenerWithRouteConfigName(host, port, e2e.SecurityLevelNone, route1)
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultServerListenerWithRouteConfigName(host, port, e2e.SecurityLevelNone, route1)},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	waitForResourceNames(ctx, t, rdsResourceNamesCh, []string{route1})

	// Verify that there is no mode change.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	select {
	case mode := <-modeCh:
		t.Fatalf("received mode change to %v when no mode expected", mode)
	case <-sCtx.Done():
	}

	// Configure the management server with the route configuration resource
	// specified by the listener resource.
	resources.Routes = []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(route1, lisResourceName, clusterName)}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Mode should go serving.
	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for mode change")
	case mode := <-modeCh:
		if mode != connectivity.ServingModeServing {
			t.Fatalf("mode change received: %v, want: %v", mode, connectivity.ServingModeServing)
		}
	}

	// Invoke lds resource not found - should go back to non serving.
	triggerResourceNotFound := internal.TriggerXDSResourceNotFoundForTesting.(func(xdsclient.XDSClient, xdsresource.Type, string) error)
	listenerResourceType := xdsinternal.ResourceTypeMapForTesting[version.V3ListenerURL].(xdsresource.Type)
	if err := triggerResourceNotFound(xdsC, listenerResourceType, listener.GetName()); err != nil {
		t.Fatalf("Failed to trigger resource name not found for testing: %v", err)
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for mode change")
	case mode := <-modeCh:
		if mode != connectivity.ServingModeNotServing {
			t.Fatalf("mode change received: %v, want: %v", mode, connectivity.ServingModeNotServing)
		}
	}
}

type testService struct {
	testgrpc.TestServiceServer
}

func (*testService) EmptyCall(context.Context, *testpb.Empty) (*testpb.Empty, error) {
	return &testpb.Empty{}, nil
}

// TestConnsCleanup tests that the listener wrapper clears it's connection
// references when connections close. It sets up a listener wrapper and gRPC
// Server, and connects to the server 100 times and makes an RPC each time, and
// then closes the connection. After these 100 connections Close, the listener
// wrapper should have no more references to any connections.
func (s) TestConnsCleanup(t *testing.T) {
	mgmtServer, nodeID, _, _, xdsC := xdsSetupForTests(t)
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to create a local TCP listener: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	modeCh := make(chan connectivity.ServingMode, 1)
	vm := verifyMode{
		modeCh: modeCh,
	}

	host, port := hostPortFromListener(t, lis)
	lisResourceName := fmt.Sprintf(e2e.ServerListenerResourceNameTemplate, net.JoinHostPort(host, strconv.Itoa(int(port))))
	params := ListenerWrapperParams{
		Listener:             lis,
		ListenerResourceName: lisResourceName,
		XDSClient:            xdsC,
		ModeCallback:         vm.verifyModeCallback,
	}
	lw := NewListenerWrapper(params)
	if lw == nil {
		t.Fatalf("NewListenerWrapper(%+v) returned nil", params)
	}
	defer lw.Close()

	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultServerListener(host, port, e2e.SecurityLevelNone, route1)},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Wait for Listener Mode to go serving.
	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for mode change")
	case mode := <-modeCh:
		if mode != connectivity.ServingModeServing {
			t.Fatalf("mode change received: %v, want: %v", mode, connectivity.ServingModeServing)
		}
	}

	server := grpc.NewServer(grpc.Creds(insecure.NewCredentials()))
	testgrpc.RegisterTestServiceServer(server, &testService{})
	wg := sync.WaitGroup{}
	go func() {
		if err := server.Serve(lw); err != nil {
			t.Errorf("failed to serve: %v", err)
		}
	}()

	// Make 100 connections to the server, and make an RPC on each one.
	for i := 0; i < 100; i++ {
		cc, err := grpc.NewClient(lw.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			t.Fatalf("grpc.NewClient failed with err: %v", err)
		}
		client := testgrpc.NewTestServiceClient(cc)
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			t.Fatalf("client.EmptyCall() failed: %v", err)
		}
		cc.Close()
	}

	lisWrapper := lw.(*listenerWrapper)
	// Eventually when the server processes the connection shutdowns, the
	// listener wrapper should clear its references to the wrapped connections.
	lenConns := 1
	for ; ctx.Err() == nil && lenConns > 0; <-time.After(time.Millisecond) {
		lisWrapper.mu.Lock()
		lenConns = len(lisWrapper.conns)
		lisWrapper.mu.Unlock()
	}
	if lenConns > 0 {
		t.Fatalf("timeout waiting for lis wrapper conns to clear, size: %v", lenConns)
	}

	server.Stop()
	wg.Wait()
}
