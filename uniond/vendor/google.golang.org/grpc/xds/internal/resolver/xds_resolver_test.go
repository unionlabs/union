/*
 *
 * Copyright 2019 gRPC authors.
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
	"strings"
	"testing"
	"time"

	xxhash "github.com/cespare/xxhash/v2"
	"github.com/envoyproxy/go-control-plane/pkg/wellknown"
	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpcsync"
	iresolver "google.golang.org/grpc/internal/resolver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/xds/internal/balancer/clustermanager"
	"google.golang.org/grpc/xds/internal/balancer/ringhash"
	"google.golang.org/grpc/xds/internal/httpfilter"
	xdsresolver "google.golang.org/grpc/xds/internal/resolver"
	rinternal "google.golang.org/grpc/xds/internal/resolver/internal"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3xdsxdstypepb "github.com/cncf/xds/go/xds/type/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3routerpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/router/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"

	_ "google.golang.org/grpc/xds/internal/balancer/cdsbalancer" // Register the cds LB policy
	_ "google.golang.org/grpc/xds/internal/httpfilter/router"    // Register the router filter
)

// Tests the case where xDS client creation is expected to fail because the
// bootstrap configuration is not specified. The test verifies that xDS resolver
// build fails as well.
func (s) TestResolverBuilder_ClientCreationFails_NoBootstrap(t *testing.T) {
	// Build an xDS resolver without specifying bootstrap env vars.
	builder := resolver.Get(xdsresolver.Scheme)
	if builder == nil {
		t.Fatalf("Scheme %q is not registered", xdsresolver.Scheme)
	}

	target := resolver.Target{URL: *testutils.MustParseURL("xds:///target")}
	if _, err := builder.Build(target, nil, resolver.BuildOptions{}); err == nil {
		t.Fatalf("xds Resolver Build(%v) succeeded when expected to fail, because there is not bootstrap configuration for the xDS client", target)
	}
}

// Tests the case where the specified dial target contains an authority that is
// not specified in the bootstrap file. Verifies that the resolver.Build method
// fails with the expected error string.
func (s) TestResolverBuilder_AuthorityNotDefinedInBootstrap(t *testing.T) {
	contents := e2e.DefaultBootstrapContents(t, "node-id", "dummy-management-server")
	testutils.CreateBootstrapFileForTesting(t, contents)

	builder := resolver.Get(xdsresolver.Scheme)
	if builder == nil {
		t.Fatalf("Scheme %q is not registered", xdsresolver.Scheme)
	}

	target := resolver.Target{URL: *testutils.MustParseURL("xds://non-existing-authority/target")}
	const wantErr = `authority "non-existing-authority" specified in dial target "xds://non-existing-authority/target" is not found in the bootstrap file`

	r, err := builder.Build(target, &testutils.ResolverClientConn{Logger: t}, resolver.BuildOptions{})
	if r != nil {
		r.Close()
	}
	if err == nil {
		t.Fatalf("xds Resolver Build(%v) succeeded for target with authority not specified in bootstrap", target)
	}
	if !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("xds Resolver Build(%v) returned err: %v, wantErr: %v", target, err, wantErr)
	}
}

// Test builds an xDS resolver and verifies that the resource name specified in
// the discovery request matches expectations.
func (s) TestResolverResourceName(t *testing.T) {
	tests := []struct {
		name                         string
		listenerResourceNameTemplate string
		extraAuthority               string
		dialTarget                   string
		wantResourceNames            []string
	}{
		{
			name:                         "default %s old style",
			listenerResourceNameTemplate: "%s",
			dialTarget:                   "xds:///target",
			wantResourceNames:            []string{"target"},
		},
		{
			name:                         "old style no percent encoding",
			listenerResourceNameTemplate: "/path/to/%s",
			dialTarget:                   "xds:///target",
			wantResourceNames:            []string{"/path/to/target"},
		},
		{
			name:                         "new style with %s",
			listenerResourceNameTemplate: "xdstp://authority.com/%s",
			dialTarget:                   "xds:///0.0.0.0:8080",
			wantResourceNames:            []string{"xdstp://authority.com/0.0.0.0:8080"},
		},
		{
			name:                         "new style percent encoding",
			listenerResourceNameTemplate: "xdstp://authority.com/%s",
			dialTarget:                   "xds:///[::1]:8080",
			wantResourceNames:            []string{"xdstp://authority.com/%5B::1%5D:8080"},
		},
		{
			name:                         "new style different authority",
			listenerResourceNameTemplate: "xdstp://authority.com/%s",
			extraAuthority:               "test-authority",
			dialTarget:                   "xds://test-authority/target",
			wantResourceNames:            []string{"xdstp://test-authority/envoy.config.listener.v3.Listener/target"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Spin up an xDS management server for the test.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			nodeID := uuid.New().String()
			mgmtServer, lisCh, _ := setupManagementServerForTest(ctx, t, nodeID)

			// Create a bootstrap configuration with test options.
			opts := bootstrap.ConfigOptionsForTesting{
				Servers: []byte(fmt.Sprintf(`[{
					"server_uri": %q,
					"channel_creds": [{"type": "insecure"}]
				}]`, mgmtServer.Address)),
				ClientDefaultListenerResourceNameTemplate: tt.listenerResourceNameTemplate,
				Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
			}
			if tt.extraAuthority != "" {
				// In this test, we really don't care about having multiple
				// management servers. All we need to verify is whether the
				// resource name matches expectation.
				opts.Authorities = map[string]json.RawMessage{
					tt.extraAuthority: []byte(fmt.Sprintf(`{
						"server_uri": %q,
						"channel_creds": [{"type": "insecure"}]
					}`, mgmtServer.Address)),
				}
			}
			contents, err := bootstrap.NewContentsForTesting(opts)
			if err != nil {
				t.Fatalf("Failed to create bootstrap configuration: %v", err)
			}
			testutils.CreateBootstrapFileForTesting(t, contents)

			buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL(tt.dialTarget)})
			waitForResourceNames(ctx, t, lisCh, tt.wantResourceNames)
		})
	}
}

// Tests the case where a service update from the underlying xDS client is
// received after the resolver is closed, and verifies that the update is not
// propagated to the ClientConn.
func (s) TestResolverWatchCallbackAfterClose(t *testing.T) {
	// Setup the management server that synchronizes with the test goroutine
	// using two channels. The management server signals the test goroutine when
	// it receives a discovery request for a route configuration resource. And
	// the test goroutine signals the management server when the resolver is
	// closed.
	routeConfigResourceNamesCh := make(chan []string, 1)
	waitForResolverCloseCh := make(chan struct{})
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() == version.V3RouteConfigURL {
				select {
				case <-routeConfigResourceNamesCh:
				default:
				}
				select {
				case routeConfigResourceNamesCh <- req.GetResourceNames():
				default:
				}
				<-waitForResolverCloseCh
			}
			return nil
		},
	})

	// Create a bootstrap configuration specifying the above management server.
	nodeID := uuid.New().String()
	contents := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
	testutils.CreateBootstrapFileForTesting(t, contents)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Wait for a discovery request for a route configuration resource.
	stateCh, _, r := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})
	waitForResourceNames(ctx, t, routeConfigResourceNamesCh, []string{defaultTestRouteConfigName})

	// Close the resolver and unblock the management server.
	r.Close()
	close(waitForResolverCloseCh)

	// Verify that the update from the management server is not propagated to
	// the ClientConn. The xDS resolver, once closed, is expected to drop
	// updates from the xDS client.
	verifyNoUpdateFromResolver(ctx, t, stateCh)
}

// Tests that the xDS resolver's Close method closes the xDS client.
func (s) TestResolverCloseClosesXDSClient(t *testing.T) {
	// Override xDS client creation to use bootstrap configuration pointing to a
	// dummy management server. Also close a channel when the returned xDS
	// client is closed.
	origNewClient := rinternal.NewXDSClient
	closeCh := make(chan struct{})
	rinternal.NewXDSClient = func(string) (xdsclient.XDSClient, func(), error) {
		bc := e2e.DefaultBootstrapContents(t, uuid.New().String(), "dummy-management-server-address")
		c, cancel, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
			Name:               t.Name(),
			Contents:           bc,
			WatchExpiryTimeout: defaultTestTimeout,
		})
		return c, grpcsync.OnceFunc(func() {
			close(closeCh)
			cancel()
		}), err
	}
	defer func() { rinternal.NewXDSClient = origNewClient }()

	_, _, r := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///my-service-client-side-xds")})
	r.Close()

	select {
	case <-closeCh:
	case <-time.After(defaultTestTimeout):
		t.Fatal("Timeout when waiting for xDS client to be closed")
	}
}

// Tests the case where a resource returned by the management server is NACKed
// by the xDS client, which then returns an update containing an error to the
// resolver. Verifies that the update is propagated to the ClientConn by the
// resolver. It also tests the cases where the resolver gets a good update
// subsequently, and another error after the good update. The test also verifies
// that these are propagated to the ClientConn.
func (s) TestResolverBadServiceUpdate(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure a listener resource that is expected to be NACKed because it
	// does not contain the `RouteSpecifier` field in the HTTPConnectionManager.
	hcm := testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
		HttpFilters: []*v3httppb.HttpFilter{e2e.HTTPFilter("router", &v3routerpb.Router{})},
	})
	lis := &v3listenerpb.Listener{
		Name:        defaultTestServiceName,
		ApiListener: &v3listenerpb.ApiListener{ApiListener: hcm},
		FilterChains: []*v3listenerpb.FilterChain{{
			Name: "filter-chain-name",
			Filters: []*v3listenerpb.Filter{{
				Name:       wellknown.HTTPConnectionManager,
				ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: hcm},
			}},
		}},
	}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, []*v3listenerpb.Listener{lis}, nil)

	// Build the resolver and expect an error update from it.
	stateCh, errCh, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})
	wantErr := "no RouteSpecifier"
	verifyErrorFromResolver(ctx, t, errCh, wantErr)

	// Configure good listener and route configuration resources on the
	// management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Expect a good update from the resolver.
	verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// Configure another bad resource on the management server and expect an
	// error update from the resolver.
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, []*v3listenerpb.Listener{lis}, nil)
	verifyErrorFromResolver(ctx, t, errCh, wantErr)
}

// TestResolverGoodServiceUpdate tests the case where the resource returned by
// the management server is ACKed by the xDS client, which then returns a good
// service update to the resolver. The test verifies that the service config
// returned by the resolver matches expectations, and that the config selector
// returned by the resolver picks clusters based on the route configuration
// received from the management server.
func (s) TestResolverGoodServiceUpdate(t *testing.T) {
	for _, tt := range []struct {
		name              string
		routeConfig       *v3routepb.RouteConfiguration
		wantServiceConfig string
		wantClusters      map[string]bool
	}{
		{
			name: "single cluster",
			routeConfig: e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
				RouteConfigName:      defaultTestRouteConfigName,
				ListenerName:         defaultTestServiceName,
				ClusterSpecifierType: e2e.RouteConfigClusterSpecifierTypeCluster,
				ClusterName:          defaultTestClusterName,
			}),
			wantServiceConfig: wantDefaultServiceConfig,
			wantClusters:      map[string]bool{fmt.Sprintf("cluster:%s", defaultTestClusterName): true},
		},
		{
			name: "two clusters",
			routeConfig: e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
				RouteConfigName:      defaultTestRouteConfigName,
				ListenerName:         defaultTestServiceName,
				ClusterSpecifierType: e2e.RouteConfigClusterSpecifierTypeWeightedCluster,
				WeightedClusters:     map[string]int{"cluster_1": 75, "cluster_2": 25},
			}),
			// This update contains the cluster from the previous update as well
			// as this update, as the previous config selector still references
			// the old cluster when the new one is pushed.
			wantServiceConfig: `{
  "loadBalancingConfig": [{
    "xds_cluster_manager_experimental": {
      "children": {
        "cluster:cluster_1": {
          "childPolicy": [{
			"cds_experimental": {
			  "cluster": "cluster_1"
			}
		  }]
        },
        "cluster:cluster_2": {
          "childPolicy": [{
			"cds_experimental": {
			  "cluster": "cluster_2"
			}
		  }]
        }
      }
    }
  }]}`,
			wantClusters: map[string]bool{"cluster:cluster_1": true, "cluster:cluster_2": true},
		},
	} {
		t.Run(tt.name, func(t *testing.T) {
			// Spin up an xDS management server for the test.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			nodeID := uuid.New().String()
			mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

			// Configure the management server with a good listener resource and a
			// route configuration resource, as specified by the test case.
			listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
			routes := []*v3routepb.RouteConfiguration{tt.routeConfig}
			configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

			stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

			// Read the update pushed by the resolver to the ClientConn.
			cs := verifyUpdateFromResolver(ctx, t, stateCh, tt.wantServiceConfig)

			pickedClusters := make(map[string]bool)
			// Odds of picking 75% cluster 100 times in a row: 1 in 3E-13.  And
			// with the random number generator stubbed out, we can rely on this
			// to be 100% reproducible.
			for i := 0; i < 100; i++ {
				res, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
				if err != nil {
					t.Fatalf("cs.SelectConfig(): %v", err)
				}
				cluster := clustermanager.GetPickedClusterForTesting(res.Context)
				pickedClusters[cluster] = true
				res.OnCommitted()
			}
			if !cmp.Equal(pickedClusters, tt.wantClusters) {
				t.Errorf("Picked clusters: %v; want: %v", pickedClusters, tt.wantClusters)
			}
		})
	}
}

// Tests a case where a resolver receives a RouteConfig update with a HashPolicy
// specifying to generate a hash. The configSelector generated should
// successfully generate a Hash.
func (s) TestResolverRequestHash(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure the management server with a good listener resource and a
	// route configuration resource that specifies a hash policy.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{{
		Name: defaultTestRouteConfigName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			Domains: []string{defaultTestServiceName},
			Routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
				Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
					ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{WeightedClusters: &v3routepb.WeightedCluster{
						Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
							{
								Name:   defaultTestClusterName,
								Weight: &wrapperspb.UInt32Value{Value: 100},
							},
						},
					}},
					HashPolicy: []*v3routepb.RouteAction_HashPolicy{{
						PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
							Header: &v3routepb.RouteAction_HashPolicy_Header{
								HeaderName: ":path",
							},
						},
						Terminal: true,
					}},
				}},
			}},
		}},
	}}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Build the resolver and read the config selector out of it.
	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})
	cs := verifyUpdateFromResolver(ctx, t, stateCh, "")

	// Selecting a config when there was a hash policy specified in the route
	// that will be selected should put a request hash in the config's context.
	res, err := cs.SelectConfig(iresolver.RPCInfo{
		Context: metadata.NewOutgoingContext(ctx, metadata.Pairs(":path", "/products")),
		Method:  "/service/method",
	})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}
	gotHash := ringhash.GetRequestHashForTesting(res.Context)
	wantHash := xxhash.Sum64String("/products")
	if gotHash != wantHash {
		t.Fatalf("Got request hash: %v, want: %v", gotHash, wantHash)
	}
}

// Tests the case where resources are removed from the management server,
// causing it to send an empty update to the xDS client, which returns a
// resource-not-found error to the xDS resolver. The test verifies that an
// ongoing RPC is handled to completion when this happens.
func (s) TestResolverRemovedWithRPCs(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Read the update pushed by the resolver to the ClientConn.
	cs := verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	res, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}

	// Delete the resources on the management server. This should result in a
	// resource-not-found error from the xDS client.
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{NodeID: nodeID}); err != nil {
		t.Fatal(err)
	}

	// The RPC started earlier is still in progress. So, the xDS resolver will
	// not produce an empty service config at this point. Instead it will retain
	// the cluster to which the RPC is ongoing in the service config, but will
	// return an erroring config selector which will fail new RPCs.
	cs = verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)
	_, err = cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err == nil || status.Code(err) != codes.Unavailable {
		t.Fatalf("cs.SelectConfig() returned: %v, want: %v", err, codes.Unavailable)
	}

	// "Finish the RPC"; this could cause a panic if the resolver doesn't
	// handle it correctly.
	res.OnCommitted()

	// Now that the RPC is committed, the xDS resolver is expected to send an
	// update with an empty service config.
	var state resolver.State
	select {
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for an update from the resolver: %v", ctx.Err())
	case state = <-stateCh:
		if err := state.ServiceConfig.Err; err != nil {
			t.Fatalf("Received error in service config: %v", state.ServiceConfig.Err)
		}
		wantSCParsed := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)("{}")
		if !internal.EqualServiceConfigForTesting(state.ServiceConfig.Config, wantSCParsed.Config) {
			t.Fatalf("Got service config:\n%s \nWant service config:\n%s", cmp.Diff(nil, state.ServiceConfig.Config), cmp.Diff(nil, wantSCParsed.Config))
		}
	}

	// Re-add the listener and expect everything to work again.
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)
	// Read the update pushed by the resolver to the ClientConn.
	cs = verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	res, err = cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}
	res.OnCommitted()
}

// Tests the case where resources returned by the management server are removed.
// The test verifies that the resolver pushes the expected config selector and
// service config in this case.
func (s) TestResolverRemovedResource(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Read the update pushed by the resolver to the ClientConn.
	cs := verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// "Make an RPC" by invoking the config selector.
	res, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}

	// "Finish the RPC"; this could cause a panic if the resolver doesn't
	// handle it correctly.
	res.OnCommitted()

	// Delete the resources on the management server, resulting in a
	// resource-not-found error from the xDS client.
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{NodeID: nodeID}); err != nil {
		t.Fatal(err)
	}

	// The channel should receive the existing service config with the original
	// cluster but with an erroring config selector.
	cs = verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// "Make another RPC" by invoking the config selector.
	res, err = cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err == nil || status.Code(err) != codes.Unavailable {
		t.Fatalf("cs.SelectConfig() got %v, %v, expected UNAVAILABLE error", res, err)
	}

	// In the meantime, an empty ServiceConfig update should have been sent.
	var state resolver.State
	select {
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for an update from the resolver: %v", ctx.Err())
	case state = <-stateCh:
		if err := state.ServiceConfig.Err; err != nil {
			t.Fatalf("Received error in service config: %v", state.ServiceConfig.Err)
		}
		wantSCParsed := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)("{}")
		if !internal.EqualServiceConfigForTesting(state.ServiceConfig.Config, wantSCParsed.Config) {
			t.Fatalf("Got service config:\n%s \nWant service config:\n%s", cmp.Diff(nil, state.ServiceConfig.Config), cmp.Diff(nil, wantSCParsed.Config))
		}
	}
}

// Tests the case where the resolver receives max stream duration as part of the
// listener and route configuration resources.  The test verifies that the RPC
// timeout returned by the config selector matches expectations. A non-nil max
// stream duration (this includes an explicit zero value) in a matching route
// overrides the value specified in the listener resource.
func (s) TestResolverMaxStreamDuration(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Configure the management server with a listener resource that specifies a
	// max stream duration as part of its HTTP connection manager. Also
	// configure a route configuration resource, which has multiple routes with
	// different values of max stream duration.
	hcm := testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
		RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{Rds: &v3httppb.Rds{
			ConfigSource: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
			},
			RouteConfigName: defaultTestRouteConfigName,
		}},
		HttpFilters: []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
		CommonHttpProtocolOptions: &v3corepb.HttpProtocolOptions{
			MaxStreamDuration: durationpb.New(1 * time.Second),
		},
	})
	listeners := []*v3listenerpb.Listener{{
		Name:        defaultTestServiceName,
		ApiListener: &v3listenerpb.ApiListener{ApiListener: hcm},
		FilterChains: []*v3listenerpb.FilterChain{{
			Name: "filter-chain-name",
			Filters: []*v3listenerpb.Filter{{
				Name:       wellknown.HTTPConnectionManager,
				ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: hcm},
			}},
		}},
	}}
	routes := []*v3routepb.RouteConfiguration{{
		Name: defaultTestRouteConfigName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			Domains: []string{defaultTestServiceName},
			Routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/foo"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{WeightedClusters: &v3routepb.WeightedCluster{
							Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
								{
									Name:   "A",
									Weight: &wrapperspb.UInt32Value{Value: 100},
								},
							}},
						},
						MaxStreamDuration: &v3routepb.RouteAction_MaxStreamDuration{
							MaxStreamDuration: durationpb.New(5 * time.Second),
						},
					}},
				},
				{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/bar"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{WeightedClusters: &v3routepb.WeightedCluster{
							Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
								{
									Name:   "B",
									Weight: &wrapperspb.UInt32Value{Value: 100},
								},
							}},
						},
						MaxStreamDuration: &v3routepb.RouteAction_MaxStreamDuration{
							MaxStreamDuration: durationpb.New(0 * time.Second),
						},
					}},
				},
				{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{WeightedClusters: &v3routepb.WeightedCluster{
							Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
								{
									Name:   "C",
									Weight: &wrapperspb.UInt32Value{Value: 100},
								},
							}},
						},
					}},
				},
			},
		}},
	}}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Read the update pushed by the resolver to the ClientConn.
	cs := verifyUpdateFromResolver(ctx, t, stateCh, "")

	testCases := []struct {
		name   string
		method string
		want   *time.Duration
	}{{
		name:   "RDS setting",
		method: "/foo/method",
		want:   newDurationP(5 * time.Second),
	}, {
		name:   "explicit zero in RDS; ignore LDS",
		method: "/bar/method",
		want:   nil,
	}, {
		name:   "no config in RDS; fallback to LDS",
		method: "/baz/method",
		want:   newDurationP(time.Second),
	}}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			req := iresolver.RPCInfo{
				Method:  tc.method,
				Context: ctx,
			}
			res, err := cs.SelectConfig(req)
			if err != nil {
				t.Errorf("cs.SelectConfig(%v): %v", req, err)
				return
			}
			res.OnCommitted()
			got := res.MethodConfig.Timeout
			if !cmp.Equal(got, tc.want) {
				t.Errorf("For method %q: res.MethodConfig.Timeout = %v; want %v", tc.method, got, tc.want)
			}
		})
	}
}

// Tests that clusters remain in service config if RPCs are in flight.
func (s) TestResolverDelayedOnCommitted(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Read the update pushed by the resolver to the ClientConn.
	cs := verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// Make an RPC, but do not commit it yet.
	resOld, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}
	wantClusterName := fmt.Sprintf("cluster:%s", defaultTestClusterName)
	if cluster := clustermanager.GetPickedClusterForTesting(resOld.Context); cluster != wantClusterName {
		t.Fatalf("Picked cluster is %q, want %q", cluster, wantClusterName)
	}

	// Delay resOld.OnCommitted(). As long as there are pending RPCs to removed
	// clusters, they still appear in the service config.

	// Update the route configuration resource on the management server to
	// return a new cluster.
	newClusterName := "new-" + defaultTestClusterName
	routes = []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, newClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Read the update pushed by the resolver to the ClientConn and ensure the
	// old cluster is present in the service config. Also ensure that the newly
	// returned config selector does not hold a reference to the old cluster.
	wantSC := fmt.Sprintf(`
{
	"loadBalancingConfig": [
		{
		  "xds_cluster_manager_experimental": {
			"children": {
			  "cluster:%s": {
				"childPolicy": [
				  {
					"cds_experimental": {
					  "cluster": "%s"
					}
				  }
				]
			  },
			  "cluster:%s": {
				"childPolicy": [
				  {
					"cds_experimental": {
					  "cluster": "%s"
					}
				  }
				]
			  }
			}
		  }
		}
	  ]
}`, defaultTestClusterName, defaultTestClusterName, newClusterName, newClusterName)
	cs = verifyUpdateFromResolver(ctx, t, stateCh, wantSC)

	resNew, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
	if err != nil {
		t.Fatalf("cs.SelectConfig(): %v", err)
	}
	wantClusterName = fmt.Sprintf("cluster:%s", newClusterName)
	if cluster := clustermanager.GetPickedClusterForTesting(resNew.Context); cluster != wantClusterName {
		t.Fatalf("Picked cluster is %q, want %q", cluster, wantClusterName)
	}

	// Invoke OnCommitted on the old RPC; should lead to a service config update
	// that deletes the old cluster, as the old cluster no longer has any
	// pending RPCs.
	resOld.OnCommitted()

	wantSC = fmt.Sprintf(`
{
	"loadBalancingConfig": [
		{
		  "xds_cluster_manager_experimental": {
			"children": {
			  "cluster:%s": {
				"childPolicy": [
				  {
					"cds_experimental": {
					  "cluster": "%s"
					}
				  }
				]
			  }
			}
		  }
		}
	  ]
}`, newClusterName, newClusterName)
	verifyUpdateFromResolver(ctx, t, stateCh, wantSC)
}

// Tests the case where two LDS updates with the same RDS name to watch are
// received without an RDS in between. Those LDS updates shouldn't trigger a
// service config update.
func (s) TestResolverMultipleLDSUpdates(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	// Configure the management server with a listener resource, but no route
	// configuration resource.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, nil)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Ensure there is no update from the resolver.
	verifyNoUpdateFromResolver(ctx, t, stateCh)

	// Configure the management server with a listener resource that points to
	// the same route configuration resource but has different values for max
	// stream duration field. There is still no route configuration resource on
	// the management server.
	hcm := testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
		RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{Rds: &v3httppb.Rds{
			ConfigSource: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
			},
			RouteConfigName: defaultTestRouteConfigName,
		}},
		HttpFilters: []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
		CommonHttpProtocolOptions: &v3corepb.HttpProtocolOptions{
			MaxStreamDuration: durationpb.New(1 * time.Second),
		},
	})
	listeners = []*v3listenerpb.Listener{{
		Name:        defaultTestServiceName,
		ApiListener: &v3listenerpb.ApiListener{ApiListener: hcm},
		FilterChains: []*v3listenerpb.FilterChain{{
			Name: "filter-chain-name",
			Filters: []*v3listenerpb.Filter{{
				Name:       wellknown.HTTPConnectionManager,
				ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: hcm},
			}},
		}},
	}}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, nil)

	// Ensure that there is no update from the resolver.
	verifyNoUpdateFromResolver(ctx, t, stateCh)
}

// TestResolverWRR tests the case where the route configuration returned by the
// management server contains a set of weighted clusters. The test performs a
// bunch of RPCs using the cluster specifier returned by the resolver, and
// verifies the cluster distribution.
func (s) TestResolverWRR(t *testing.T) {
	origNewWRR := rinternal.NewWRR
	rinternal.NewWRR = testutils.NewTestWRR
	defer func() { rinternal.NewWRR = origNewWRR }()

	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
		RouteConfigName:      defaultTestRouteConfigName,
		ListenerName:         defaultTestServiceName,
		ClusterSpecifierType: e2e.RouteConfigClusterSpecifierTypeWeightedCluster,
		WeightedClusters:     map[string]int{"A": 75, "B": 25},
	})}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Read the update pushed by the resolver to the ClientConn.
	cs := verifyUpdateFromResolver(ctx, t, stateCh, "")

	// Make RPCs to verify WRR behavior in the cluster specifier.
	picks := map[string]int{}
	for i := 0; i < 100; i++ {
		res, err := cs.SelectConfig(iresolver.RPCInfo{Context: ctx, Method: "/service/method"})
		if err != nil {
			t.Fatalf("cs.SelectConfig(): %v", err)
		}
		picks[clustermanager.GetPickedClusterForTesting(res.Context)]++
		res.OnCommitted()
	}
	want := map[string]int{"cluster:A": 75, "cluster:B": 25}
	if !cmp.Equal(picks, want) {
		t.Errorf("Picked clusters: %v; want: %v", picks, want)
	}
}

const filterCfgPathFieldName = "path"
const filterCfgErrorFieldName = "new_stream_error"

type filterCfg struct {
	httpfilter.FilterConfig
	path         string
	newStreamErr error
}

type filterBuilder struct {
	paths   []string
	typeURL string
}

func (fb *filterBuilder) TypeURLs() []string { return []string{fb.typeURL} }

func filterConfigFromProto(cfg proto.Message) (httpfilter.FilterConfig, error) {
	ts, ok := cfg.(*v3xdsxdstypepb.TypedStruct)
	if !ok {
		return nil, fmt.Errorf("unsupported filter config type: %T, want %T", cfg, &v3xdsxdstypepb.TypedStruct{})
	}

	if ts.GetValue() == nil {
		return filterCfg{}, nil
	}
	ret := filterCfg{}
	if v := ts.GetValue().GetFields()[filterCfgPathFieldName]; v != nil {
		ret.path = v.GetStringValue()
	}
	if v := ts.GetValue().GetFields()[filterCfgErrorFieldName]; v != nil {
		if v.GetStringValue() == "" {
			ret.newStreamErr = nil
		} else {
			ret.newStreamErr = fmt.Errorf("%s", v.GetStringValue())
		}
	}
	return ret, nil
}

func (*filterBuilder) ParseFilterConfig(cfg proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfigFromProto(cfg)
}

func (*filterBuilder) ParseFilterConfigOverride(override proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfigFromProto(override)
}

func (*filterBuilder) IsTerminal() bool { return false }

var _ httpfilter.ClientInterceptorBuilder = &filterBuilder{}

func (fb *filterBuilder) BuildClientInterceptor(config, override httpfilter.FilterConfig) (iresolver.ClientInterceptor, error) {
	if config == nil {
		panic("unexpected missing config")
	}

	fi := &filterInterceptor{
		parent: fb,
		pathCh: make(chan string, 10),
	}

	fb.paths = append(fb.paths, "build:"+config.(filterCfg).path)
	err := config.(filterCfg).newStreamErr
	if override != nil {
		fb.paths = append(fb.paths, "override:"+override.(filterCfg).path)
		err = override.(filterCfg).newStreamErr
	}

	fi.cfgPath = config.(filterCfg).path
	fi.err = err
	return fi, nil
}

type filterInterceptor struct {
	parent  *filterBuilder
	pathCh  chan string
	cfgPath string
	err     error
}

func (fi *filterInterceptor) NewStream(ctx context.Context, ri iresolver.RPCInfo, done func(), newStream func(ctx context.Context, done func()) (iresolver.ClientStream, error)) (iresolver.ClientStream, error) {
	fi.parent.paths = append(fi.parent.paths, "newstream:"+fi.cfgPath)
	if fi.err != nil {
		return nil, fi.err
	}
	d := func() {
		fi.parent.paths = append(fi.parent.paths, "done:"+fi.cfgPath)
		done()
	}
	cs, err := newStream(ctx, d)
	if err != nil {
		return nil, err
	}
	return &clientStream{ClientStream: cs}, nil
}

type clientStream struct {
	iresolver.ClientStream
}

func (s) TestConfigSelector_FailureCases(t *testing.T) {
	const methodName = "1"

	tests := []struct {
		name     string
		listener *v3listenerpb.Listener
		wantErr  string
	}{
		{
			name: "route type RouteActionUnsupported invalid for client",
			listener: &v3listenerpb.Listener{
				Name: defaultTestServiceName,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
							RouteConfig: &v3routepb.RouteConfiguration{
								Name: defaultTestRouteConfigName,
								VirtualHosts: []*v3routepb.VirtualHost{{
									Domains: []string{defaultTestServiceName},
									Routes: []*v3routepb.Route{{
										Match: &v3routepb.RouteMatch{
											PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: methodName},
										},
										Action: &v3routepb.Route_FilterAction{},
									}},
								}},
							}},
						HttpFilters: []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
					}),
				},
			},
			wantErr: "matched route does not have a supported route action type",
		},
		{
			name: "route type RouteActionNonForwardingAction invalid for client",
			listener: &v3listenerpb.Listener{
				Name: defaultTestServiceName,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
							RouteConfig: &v3routepb.RouteConfiguration{
								Name: defaultTestRouteConfigName,
								VirtualHosts: []*v3routepb.VirtualHost{{
									Domains: []string{defaultTestServiceName},
									Routes: []*v3routepb.Route{{
										Match: &v3routepb.RouteMatch{
											PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: methodName},
										},
										Action: &v3routepb.Route_NonForwardingAction{},
									}},
								}},
							}},
						HttpFilters: []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
					}),
				},
			},
			wantErr: "matched route does not have a supported route action type",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			// Spin up an xDS management server.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			nodeID := uuid.New().String()
			mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

			// Build an xDS resolver.
			stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

			// Update the management server with a listener resource that
			// contains inline route configuration.
			configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, []*v3listenerpb.Listener{test.listener}, nil)

			// Ensure that the resolver pushes a state update to the channel.
			cs := verifyUpdateFromResolver(ctx, t, stateCh, "")

			// Ensure that it returns the expected error.
			_, err := cs.SelectConfig(iresolver.RPCInfo{Method: methodName, Context: ctx})
			if err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Errorf("SelectConfig(_) = _, %v; want _, Contains(%v)", err, test.wantErr)
			}
		})
	}
}

func newHTTPFilter(t *testing.T, name, typeURL, path, err string) *v3httppb.HttpFilter {
	return &v3httppb.HttpFilter{
		Name: name,
		ConfigType: &v3httppb.HttpFilter_TypedConfig{
			TypedConfig: testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
				TypeUrl: typeURL,
				Value: &structpb.Struct{
					Fields: map[string]*structpb.Value{
						filterCfgPathFieldName:  {Kind: &structpb.Value_StringValue{StringValue: path}},
						filterCfgErrorFieldName: {Kind: &structpb.Value_StringValue{StringValue: err}},
					},
				},
			}),
		},
	}
}

func (s) TestXDSResolverHTTPFilters(t *testing.T) {
	const methodName1 = "1"
	const methodName2 = "2"
	testFilterName := t.Name()

	testCases := []struct {
		name          string
		listener      *v3listenerpb.Listener
		rpcRes        map[string][][]string
		wantStreamErr string
	}{
		{
			name: "NewStream error - ensure earlier interceptor Done is still called",
			listener: &v3listenerpb.Listener{
				Name: defaultTestServiceName,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
							RouteConfig: &v3routepb.RouteConfiguration{
								Name: defaultTestRouteConfigName,
								VirtualHosts: []*v3routepb.VirtualHost{{
									Domains: []string{defaultTestServiceName},
									Routes: []*v3routepb.Route{{
										Match: &v3routepb.RouteMatch{
											PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: methodName1},
										},
										Action: &v3routepb.Route_Route{
											Route: &v3routepb.RouteAction{
												ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
													WeightedClusters: &v3routepb.WeightedCluster{
														Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
															{Name: "A", Weight: wrapperspb.UInt32(1)},
															{Name: "B", Weight: wrapperspb.UInt32(1)},
														},
													},
												},
											},
										},
									}},
								}},
							}},
						HttpFilters: []*v3httppb.HttpFilter{
							newHTTPFilter(t, "foo", testFilterName, "foo1", ""),
							newHTTPFilter(t, "bar", testFilterName, "bar1", "bar newstream err"),
							e2e.RouterHTTPFilter,
						},
					}),
				},
			},
			rpcRes: map[string][][]string{
				methodName1: {
					{"build:foo1", "build:bar1", "newstream:foo1", "newstream:bar1", "done:foo1"}, // err in bar1 NewStream()
				},
			},
			wantStreamErr: "bar newstream err",
		},
		{
			name: "all overrides",
			listener: &v3listenerpb.Listener{
				Name: defaultTestServiceName,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
							RouteConfig: &v3routepb.RouteConfiguration{
								Name: defaultTestRouteConfigName,
								VirtualHosts: []*v3routepb.VirtualHost{{
									Domains: []string{defaultTestServiceName},
									Routes: []*v3routepb.Route{
										{
											Match: &v3routepb.RouteMatch{
												PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: methodName1},
											},
											Action: &v3routepb.Route_Route{
												Route: &v3routepb.RouteAction{
													ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
														WeightedClusters: &v3routepb.WeightedCluster{
															Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
																{Name: "A", Weight: wrapperspb.UInt32(1)},
																{Name: "B", Weight: wrapperspb.UInt32(1)},
															},
														},
													},
												},
											},
										},
										{
											Match: &v3routepb.RouteMatch{
												PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: methodName2},
											},
											Action: &v3routepb.Route_Route{
												Route: &v3routepb.RouteAction{
													ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
														WeightedClusters: &v3routepb.WeightedCluster{
															Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
																{Name: "A", Weight: wrapperspb.UInt32(1)},
																{
																	Name:   "B",
																	Weight: wrapperspb.UInt32(1),
																	TypedPerFilterConfig: map[string]*anypb.Any{
																		"foo": testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
																			TypeUrl: testFilterName,
																			Value: &structpb.Struct{
																				Fields: map[string]*structpb.Value{
																					filterCfgPathFieldName: {Kind: &structpb.Value_StringValue{StringValue: "foo4"}},
																				},
																			},
																		}),
																		"bar": testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
																			TypeUrl: testFilterName,
																			Value: &structpb.Struct{
																				Fields: map[string]*structpb.Value{
																					filterCfgPathFieldName: {Kind: &structpb.Value_StringValue{StringValue: "bar4"}},
																				},
																			},
																		}),
																	},
																},
															},
														},
													},
												},
											},
											TypedPerFilterConfig: map[string]*anypb.Any{
												"foo": testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
													TypeUrl: testFilterName,
													Value: &structpb.Struct{
														Fields: map[string]*structpb.Value{
															filterCfgPathFieldName:  {Kind: &structpb.Value_StringValue{StringValue: "foo3"}},
															filterCfgErrorFieldName: {Kind: &structpb.Value_StringValue{StringValue: ""}},
														},
													},
												}),
												"bar": testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
													TypeUrl: testFilterName,
													Value: &structpb.Struct{
														Fields: map[string]*structpb.Value{
															filterCfgPathFieldName: {Kind: &structpb.Value_StringValue{StringValue: "bar3"}},
														},
													},
												}),
											},
										},
									},
									TypedPerFilterConfig: map[string]*anypb.Any{
										"foo": testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
											TypeUrl: testFilterName,
											Value: &structpb.Struct{
												Fields: map[string]*structpb.Value{
													filterCfgPathFieldName:  {Kind: &structpb.Value_StringValue{StringValue: "foo2"}},
													filterCfgErrorFieldName: {Kind: &structpb.Value_StringValue{StringValue: ""}},
												},
											},
										}),
										"bar": testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
											TypeUrl: testFilterName,
											Value: &structpb.Struct{
												Fields: map[string]*structpb.Value{
													filterCfgPathFieldName: {Kind: &structpb.Value_StringValue{StringValue: "bar2"}},
												},
											},
										}),
									},
								}},
							}},
						HttpFilters: []*v3httppb.HttpFilter{
							newHTTPFilter(t, "foo", testFilterName, "foo1", "this is overridden to nil"),
							newHTTPFilter(t, "bar", testFilterName, "bar1", ""),
							e2e.RouterHTTPFilter,
						},
					}),
				},
			},
			rpcRes: map[string][][]string{
				methodName1: {
					{"build:foo1", "override:foo2", "build:bar1", "override:bar2", "newstream:foo1", "newstream:bar1", "done:bar1", "done:foo1"},
					{"build:foo1", "override:foo2", "build:bar1", "override:bar2", "newstream:foo1", "newstream:bar1", "done:bar1", "done:foo1"},
				},
				methodName2: {
					{"build:foo1", "override:foo3", "build:bar1", "override:bar3", "newstream:foo1", "newstream:bar1", "done:bar1", "done:foo1"},
					{"build:foo1", "override:foo4", "build:bar1", "override:bar4", "newstream:foo1", "newstream:bar1", "done:bar1", "done:foo1"},
					{"build:foo1", "override:foo3", "build:bar1", "override:bar3", "newstream:foo1", "newstream:bar1", "done:bar1", "done:foo1"},
					{"build:foo1", "override:foo4", "build:bar1", "override:bar4", "newstream:foo1", "newstream:bar1", "done:bar1", "done:foo1"},
				},
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			origNewWRR := rinternal.NewWRR
			rinternal.NewWRR = testutils.NewTestWRR
			defer func() { rinternal.NewWRR = origNewWRR }()

			// Register a custom httpFilter builder for the test.
			fb := &filterBuilder{typeURL: testFilterName}
			httpfilter.Register(fb)

			// Spin up an xDS management server.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			nodeID := uuid.New().String()
			mgmtServer, _, _ := setupManagementServerForTest(ctx, t, nodeID)

			// Build an xDS resolver.
			stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

			// Update the management server with a listener resource that
			// contains an inline route configuration.
			configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, []*v3listenerpb.Listener{tc.listener}, nil)

			// Ensure that the resolver pushes a state update to the channel.
			cs := verifyUpdateFromResolver(ctx, t, stateCh, "")

			for method, wants := range tc.rpcRes {
				// Order of wants is non-deterministic.
				remainingWant := make([][]string, len(wants))
				copy(remainingWant, wants)
				for n := range wants {
					res, err := cs.SelectConfig(iresolver.RPCInfo{Method: method, Context: ctx})
					if err != nil {
						t.Fatalf("Unexpected error from cs.SelectConfig(_): %v", err)
					}

					var doneFunc func()
					_, err = res.Interceptor.NewStream(context.Background(), iresolver.RPCInfo{}, func() {}, func(ctx context.Context, done func()) (iresolver.ClientStream, error) {
						doneFunc = done
						return nil, nil
					})
					if tc.wantStreamErr != "" {
						if err == nil || !strings.Contains(err.Error(), tc.wantStreamErr) {
							t.Errorf("NewStream(...) = _, %v; want _, Contains(%v)", err, tc.wantStreamErr)
						}
						if err == nil {
							res.OnCommitted()
							doneFunc()
						}
						continue
					}
					if err != nil {
						t.Fatalf("unexpected error from Interceptor.NewStream: %v", err)

					}
					res.OnCommitted()
					doneFunc()

					gotPaths := fb.paths
					fb.paths = []string{}

					// Confirm the desired path is found in remainingWant, and remove it.
					pass := false
					for i := range remainingWant {
						if cmp.Equal(gotPaths, remainingWant[i]) {
							remainingWant[i] = remainingWant[len(remainingWant)-1]
							remainingWant = remainingWant[:len(remainingWant)-1]
							pass = true
							break
						}
					}
					if !pass {
						t.Errorf("%q:%v - path:\n%v\nwant one of:\n%v", method, n, gotPaths, remainingWant)
					}
				}
			}
		})
	}
}

func newDurationP(d time.Duration) *time.Duration {
	return &d
}
