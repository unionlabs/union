/*
 *
 * Copyright 2020 gRPC authors.
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
	"testing"

	"github.com/envoyproxy/go-control-plane/pkg/wellknown"
	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/resolver"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3routerpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/router/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
)

// Tests the case where the listener resource starts pointing to a new route
// configuration resource after the xDS resolver has successfully resolved the
// service name and pushed an update on the channel. The test verifies that the
// resolver stops requesting the old route configuration resource and requests
// the new resource, and once successfully resolved, verifies that the update
// from the resolver matches expected service config.
func (s) TestServiceWatch_ListenerPointsToNewRouteConfiguration(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, lisCh, routeCfgCh := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Verify initial update from the resolver.
	waitForResourceNames(ctx, t, lisCh, []string{defaultTestServiceName})
	waitForResourceNames(ctx, t, routeCfgCh, []string{defaultTestRouteConfigName})
	verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// Update the listener resource to point to a new route configuration name.
	// Leave the old route configuration resource unchanged.
	newTestRouteConfigName := defaultTestRouteConfigName + "-new"
	listeners = []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, newTestRouteConfigName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Verify that the new route configuration resource is requested.
	waitForResourceNames(ctx, t, routeCfgCh, []string{newTestRouteConfigName})

	// Update the old route configuration resource by adding a new route.
	routes[0].VirtualHosts[0].Routes = append(routes[0].VirtualHosts[0].Routes, &v3routepb.Route{
		Match: &v3routepb.RouteMatch{
			PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/foo/bar"},
			CaseSensitive: &wrapperspb.BoolValue{Value: false},
		},
		Action: &v3routepb.Route_Route{
			Route: &v3routepb.RouteAction{
				ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: "some-random-cluster"},
			},
		},
	})
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Wait for no update from the resolver.
	verifyNoUpdateFromResolver(ctx, t, stateCh)

	// Update the management server with the new route configuration resource.
	routes = append(routes, e2e.DefaultRouteConfig(newTestRouteConfigName, defaultTestServiceName, defaultTestClusterName))
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Ensure update from the resolver.
	verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)
}

// Tests the case where the listener resource changes to contain an inline route
// configuration and changes back to having a route configuration resource name.
// Verifies that the expected xDS resource names are requested by the resolver
// and that the update from the resolver matches expected service config.
func (s) TestServiceWatch_ListenerPointsToInlineRouteConfiguration(t *testing.T) {
	// Spin up an xDS management server for the test.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	nodeID := uuid.New().String()
	mgmtServer, lisCh, routeCfgCh := setupManagementServerForTest(ctx, t, nodeID)

	// Configure resources on the management server.
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	routes := []*v3routepb.RouteConfiguration{e2e.DefaultRouteConfig(defaultTestRouteConfigName, defaultTestServiceName, defaultTestClusterName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	stateCh, _, _ := buildResolverForTarget(t, resolver.Target{URL: *testutils.MustParseURL("xds:///" + defaultTestServiceName)})

	// Verify initial update from the resolver.
	waitForResourceNames(ctx, t, lisCh, []string{defaultTestServiceName})
	waitForResourceNames(ctx, t, routeCfgCh, []string{defaultTestRouteConfigName})
	verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// Update listener to contain an inline route configuration.
	hcm := testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
		RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
			RouteConfig: &v3routepb.RouteConfiguration{
				Name: defaultTestRouteConfigName,
				VirtualHosts: []*v3routepb.VirtualHost{{
					Domains: []string{defaultTestServiceName},
					Routes: []*v3routepb.Route{{
						Match: &v3routepb.RouteMatch{
							PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
						},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: defaultTestClusterName},
							},
						},
					}},
				}},
			},
		},
		HttpFilters: []*v3httppb.HttpFilter{e2e.HTTPFilter("router", &v3routerpb.Router{})},
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

	// Verify that the old route configuration is not requested anymore.
	waitForResourceNames(ctx, t, routeCfgCh, []string{})
	verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)

	// Update listener back to contain a route configuration name.
	listeners = []*v3listenerpb.Listener{e2e.DefaultClientListener(defaultTestServiceName, defaultTestRouteConfigName)}
	configureResourcesOnManagementServer(ctx, t, mgmtServer, nodeID, listeners, routes)

	// Verify that that route configuration resource is requested.
	waitForResourceNames(ctx, t, routeCfgCh, []string{defaultTestRouteConfigName})

	// Verify that appropriate SC is pushed on the channel.
	verifyUpdateFromResolver(ctx, t, stateCh, wantDefaultServiceConfig)
}
