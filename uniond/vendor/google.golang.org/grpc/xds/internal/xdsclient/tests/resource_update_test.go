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
	"fmt"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/fakeserver"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3adminpb "github.com/envoyproxy/go-control-plane/envoy/admin/v3"
	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3statuspb "github.com/envoyproxy/go-control-plane/envoy/service/status/v3"

	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter.
)

// startFakeManagementServer starts a fake xDS management server and returns a
// cleanup function to close the fake server.
func startFakeManagementServer(t *testing.T) (*fakeserver.Server, func()) {
	t.Helper()
	fs, sCleanup, err := fakeserver.StartServer(nil)
	if err != nil {
		t.Fatalf("Failed to start fake xDS server: %v", err)
	}
	return fs, sCleanup
}

func compareUpdateMetadata(ctx context.Context, dumpFunc func() *v3statuspb.ClientStatusResponse, want []*v3statuspb.ClientConfig_GenericXdsConfig) error {
	var cmpOpts = cmp.Options{
		cmp.Transformer("sort", func(in []*v3statuspb.ClientConfig_GenericXdsConfig) []*v3statuspb.ClientConfig_GenericXdsConfig {
			out := append([]*v3statuspb.ClientConfig_GenericXdsConfig(nil), in...)
			sort.Slice(out, func(i, j int) bool {
				a, b := out[i], out[j]
				if a == nil {
					return true
				}
				if b == nil {
					return false
				}
				if strings.Compare(a.TypeUrl, b.TypeUrl) == 0 {
					return strings.Compare(a.Name, b.Name) < 0
				}
				return strings.Compare(a.TypeUrl, b.TypeUrl) < 0
			})
			return out
		}),
		protocmp.Transform(),
		protocmp.IgnoreFields((*v3statuspb.ClientConfig_GenericXdsConfig)(nil), "last_updated"),
		protocmp.IgnoreFields((*v3adminpb.UpdateFailureState)(nil), "last_update_attempt", "details"),
	}

	var lastErr error
	for ; ctx.Err() == nil; <-time.After(100 * time.Millisecond) {
		var got []*v3statuspb.ClientConfig_GenericXdsConfig
		for _, cfg := range dumpFunc().GetConfig() {
			got = append(got, cfg.GetGenericXdsConfigs()...)
		}
		diff := cmp.Diff(want, got, cmpOpts)
		if diff == "" {
			return nil
		}
		lastErr = fmt.Errorf("unexpected diff in metadata, diff (-want +got):\n%s\n want: %+v\n got: %+v", diff, want, got)
	}
	return fmt.Errorf("timeout when waiting for expected update metadata: %v", lastErr)
}

// TestHandleListenerResponseFromManagementServer covers different scenarios
// involving receipt of an LDS response from the management server. The test
// verifies that the internal state of the xDS client (parsed resource and
// metadata) matches expectations.
func (s) TestHandleListenerResponseFromManagementServer(t *testing.T) {
	const (
		resourceName1 = "resource-name-1"
		resourceName2 = "resource-name-2"
	)
	var (
		emptyRouterFilter = e2e.RouterHTTPFilter
		apiListener       = &v3listenerpb.ApiListener{
			ApiListener: func() *anypb.Any {
				return testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
					RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
						Rds: &v3httppb.Rds{
							ConfigSource: &v3corepb.ConfigSource{
								ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
							},
							RouteConfigName: "route-configuration-name",
						},
					},
					HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
				})
			}(),
		}
		resource1 = &v3listenerpb.Listener{
			Name:        resourceName1,
			ApiListener: apiListener,
		}
		resource2 = &v3listenerpb.Listener{
			Name:        resourceName2,
			ApiListener: apiListener,
		}
	)

	tests := []struct {
		desc                     string
		resourceName             string
		managementServerResponse *v3discoverypb.DiscoveryResponse
		wantUpdate               xdsresource.ListenerUpdate
		wantErr                  string
		wantGenericXDSConfig     []*v3statuspb.ClientConfig_GenericXdsConfig
	}{
		{
			desc:         "badly-marshaled-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
				VersionInfo: "1",
				Resources: []*anypb.Any{{
					TypeUrl: "type.googleapis.com/envoy.config.listener.v3.Listener",
					Value:   []byte{1, 2, 3, 4},
				}},
			},
			wantErr: "Listener not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.listener.v3.Listener",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "empty-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
				VersionInfo: "1",
			},
			wantErr: "Listener not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.listener.v3.Listener",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "unexpected-type-in-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, &v3routepb.RouteConfiguration{})},
			},
			wantErr: "Listener not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.listener.v3.Listener",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "one-bad-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
				VersionInfo: "1",
				Resources: []*anypb.Any{testutils.MarshalAny(t, &v3listenerpb.Listener{
					Name: resourceName1,
					ApiListener: &v3listenerpb.ApiListener{
						ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{}),
					}}),
				},
			},
			wantErr: "no RouteSpecifier",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.listener.v3.Listener",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_NACKED,
					ErrorState: &v3adminpb.UpdateFailureState{
						VersionInfo: "1",
					},
				},
			},
		},
		{
			desc:         "one-good-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1)},
			},
			wantUpdate: xdsresource.ListenerUpdate{
				RouteConfigName: "route-configuration-name",
				HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.listener.v3.Listener",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
		{
			desc:         "two-resources-when-we-requested-one",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.listener.v3.Listener",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1), testutils.MarshalAny(t, resource2)},
			},
			wantUpdate: xdsresource.ListenerUpdate{
				RouteConfigName: "route-configuration-name",
				HTTPFilters:     []xdsresource.HTTPFilter{{Name: "router"}},
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.listener.v3.Listener",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			// Create a fake xDS management server listening on a local port,
			// and set it up with the response to send.
			mgmtServer, cleanup := startFakeManagementServer(t)
			defer cleanup()
			t.Logf("Started xDS management server on %s", mgmtServer.Address)

			// Create an xDS client talking to the above management server.
			nodeID := uuid.New().String()
			bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
			client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
				Name:               t.Name(),
				Contents:           bc,
				WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
			})
			if err != nil {
				t.Fatalf("Failed to create an xDS client: %v", err)
			}
			defer close()
			t.Logf("Created xDS client to %s", mgmtServer.Address)

			// Register a watch, and push the results on to a channel.
			lw := newListenerWatcher()
			cancel := xdsresource.WatchListener(client, test.resourceName, lw)
			defer cancel()
			t.Logf("Registered a watch for Listener %q", test.resourceName)

			// Wait for the discovery request to be sent out.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			val, err := mgmtServer.XDSRequestChan.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for discovery request at the management server: %v", ctx)
			}
			wantReq := &fakeserver.Request{Req: &v3discoverypb.DiscoveryRequest{
				Node: &v3corepb.Node{
					Id:            nodeID,
					UserAgentName: "gRPC Go",
					ClientFeatures: []string{
						"envoy.lb.does_not_support_overprovisioning",
						"xds.config.resource-in-sotw",
					},
				},
				ResourceNames: []string{test.resourceName},
				TypeUrl:       "type.googleapis.com/envoy.config.listener.v3.Listener",
			}}
			gotReq := val.(*fakeserver.Request)
			if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), protocmp.IgnoreFields(&v3corepb.Node{}, "user_agent_version")); diff != "" {
				t.Fatalf("Discovery request received with unexpected diff (-got +want):\n%s\n got: %+v, want: %+v", diff, gotReq, wantReq)
			}
			t.Logf("Discovery request received at management server")

			// Configure the fake management server with a response.
			mgmtServer.XDSResponseChan <- &fakeserver.Response{Resp: test.managementServerResponse}

			// Wait for an update from the xDS client and compare with expected
			// update.
			val, err = lw.updateCh.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for watch callback to invoked after response from management server: %v", err)
			}
			gotUpdate := val.(listenerUpdateErrTuple).update
			gotErr := val.(listenerUpdateErrTuple).err
			if (gotErr != nil) != (test.wantErr != "") {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			if gotErr != nil && !strings.Contains(gotErr.Error(), test.wantErr) {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			cmpOpts := []cmp.Option{
				cmpopts.EquateEmpty(),
				cmpopts.IgnoreFields(xdsresource.HTTPFilter{}, "Filter", "Config"),
				cmpopts.IgnoreFields(xdsresource.ListenerUpdate{}, "Raw"),
			}
			if diff := cmp.Diff(test.wantUpdate, gotUpdate, cmpOpts...); diff != "" {
				t.Fatalf("Unexpected diff in metadata, diff (-want +got):\n%s", diff)
			}
			if err := compareUpdateMetadata(ctx, xdsclient.DumpResources, test.wantGenericXDSConfig); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestHandleRouteConfigResponseFromManagementServer covers different scenarios
// involving receipt of an RDS response from the management server. The test
// verifies that the internal state of the xDS client (parsed resource and
// metadata) matches expectations.
func (s) TestHandleRouteConfigResponseFromManagementServer(t *testing.T) {
	const (
		resourceName1 = "resource-name-1"
		resourceName2 = "resource-name-2"
	)
	var (
		virtualHosts = []*v3routepb.VirtualHost{
			{
				Domains: []string{"lds-target-name"},
				Routes: []*v3routepb.Route{
					{
						Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: ""}},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: "cluster-name"},
							},
						},
					},
				},
			},
		}
		resource1 = &v3routepb.RouteConfiguration{
			Name:         resourceName1,
			VirtualHosts: virtualHosts,
		}
		resource2 = &v3routepb.RouteConfiguration{
			Name:         resourceName2,
			VirtualHosts: virtualHosts,
		}
	)

	tests := []struct {
		desc                     string
		resourceName             string
		managementServerResponse *v3discoverypb.DiscoveryResponse
		wantUpdate               xdsresource.RouteConfigUpdate
		wantErr                  string
		wantGenericXDSConfig     []*v3statuspb.ClientConfig_GenericXdsConfig
	}{
		// The first three tests involve scenarios where the response fails
		// protobuf deserialization (because it contains an invalid data or type
		// in the anypb.Any) or the requested resource is not present in the
		// response.  In either case, no resource update makes its way to the
		// top-level xDS client. An RDS response without a requested resource
		// does not mean that the resource does not exist in the server. It
		// could be part of a future update.  Therefore, the only failure mode
		// for this resource is for the watch to timeout.
		{
			desc:         "badly-marshaled-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
				Resources: []*anypb.Any{{
					TypeUrl: "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Value:   []byte{1, 2, 3, 4},
				}},
			},
			wantErr: "RouteConfiguration not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "empty-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
			},
			wantErr: "RouteConfiguration not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "unexpected-type-in-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, &v3clusterpb.Cluster{})},
			},
			wantErr: "RouteConfiguration not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "one-bad-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
				Resources: []*anypb.Any{testutils.MarshalAny(t, &v3routepb.RouteConfiguration{
					Name: resourceName1,
					VirtualHosts: []*v3routepb.VirtualHost{{
						Domains: []string{"lds-resource-name"},
						Routes: []*v3routepb.Route{{
							Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
							Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: "cluster-resource-name"},
							}}}},
						RetryPolicy: &v3routepb.RetryPolicy{
							NumRetries: &wrapperspb.UInt32Value{Value: 0},
						},
					}},
				})},
			},
			wantErr: "received route is invalid: retry_policy.num_retries = 0; must be >= 1",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_NACKED,
					ErrorState: &v3adminpb.UpdateFailureState{
						VersionInfo: "1",
					},
				},
			},
		},
		{
			desc:         "one-good-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1)},
			},
			wantUpdate: xdsresource.RouteConfigUpdate{
				VirtualHosts: []*xdsresource.VirtualHost{
					{
						Domains: []string{"lds-target-name"},
						Routes: []*xdsresource.Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]xdsresource.WeightedCluster{"cluster-name": {Weight: 1}},
							ActionType:       xdsresource.RouteActionRoute}},
					},
				},
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
		{
			desc:         "two-resources-when-we-requested-one",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1), testutils.MarshalAny(t, resource2)},
			},
			wantUpdate: xdsresource.RouteConfigUpdate{
				VirtualHosts: []*xdsresource.VirtualHost{
					{
						Domains: []string{"lds-target-name"},
						Routes: []*xdsresource.Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]xdsresource.WeightedCluster{"cluster-name": {Weight: 1}},
							ActionType:       xdsresource.RouteActionRoute}},
					},
				},
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			// Create a fake xDS management server listening on a local port,
			// and set it up with the response to send.
			mgmtServer, cleanup := startFakeManagementServer(t)
			defer cleanup()
			t.Logf("Started xDS management server on %s", mgmtServer.Address)

			// Create an xDS client talking to the above management server.
			nodeID := uuid.New().String()
			bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
			client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
				Name:               t.Name(),
				Contents:           bc,
				WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
			})
			if err != nil {
				t.Fatalf("Failed to create an xDS client: %v", err)
			}
			defer close()
			t.Logf("Created xDS client to %s", mgmtServer.Address)

			// Register a watch, and push the results on to a channel.
			rw := newRouteConfigWatcher()
			cancel := xdsresource.WatchRouteConfig(client, test.resourceName, rw)
			defer cancel()
			t.Logf("Registered a watch for Route Configuration %q", test.resourceName)

			// Wait for the discovery request to be sent out.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			val, err := mgmtServer.XDSRequestChan.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for discovery request at the management server: %v", ctx)
			}
			wantReq := &fakeserver.Request{Req: &v3discoverypb.DiscoveryRequest{
				Node: &v3corepb.Node{
					Id:            nodeID,
					UserAgentName: "gRPC Go",
					ClientFeatures: []string{
						"envoy.lb.does_not_support_overprovisioning",
						"xds.config.resource-in-sotw",
					},
				},
				ResourceNames: []string{test.resourceName},
				TypeUrl:       "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
			}}
			gotReq := val.(*fakeserver.Request)
			if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), protocmp.IgnoreFields(&v3corepb.Node{}, "user_agent_version")); diff != "" {
				t.Fatalf("Discovery request received with unexpected diff (-got +want):\n%s\n got: %+v, want: %+v", diff, gotReq, wantReq)
			}
			t.Logf("Discovery request received at management server")

			// Configure the fake management server with a response.
			mgmtServer.XDSResponseChan <- &fakeserver.Response{Resp: test.managementServerResponse}

			// Wait for an update from the xDS client and compare with expected
			// update.
			val, err = rw.updateCh.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for watch callback to invoked after response from management server: %v", err)
			}
			gotUpdate := val.(routeConfigUpdateErrTuple).update
			gotErr := val.(routeConfigUpdateErrTuple).err
			if (gotErr != nil) != (test.wantErr != "") {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			if gotErr != nil && !strings.Contains(gotErr.Error(), test.wantErr) {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			cmpOpts := []cmp.Option{
				cmpopts.EquateEmpty(),
				cmpopts.IgnoreFields(xdsresource.RouteConfigUpdate{}, "Raw"),
			}
			if diff := cmp.Diff(test.wantUpdate, gotUpdate, cmpOpts...); diff != "" {
				t.Fatalf("Unexpected diff in metadata, diff (-want +got):\n%s", diff)
			}
			if err := compareUpdateMetadata(ctx, xdsclient.DumpResources, test.wantGenericXDSConfig); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestHandleClusterResponseFromManagementServer covers different scenarios
// involving receipt of a CDS response from the management server. The test
// verifies that the internal state of the xDS client (parsed resource and
// metadata) matches expectations.
func (s) TestHandleClusterResponseFromManagementServer(t *testing.T) {
	const (
		resourceName1 = "resource-name-1"
		resourceName2 = "resource-name-2"
	)
	resource1 := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: resourceName1,
		ServiceName: "eds-service-name",
		EnableLRS:   true,
	})
	resource2 := proto.Clone(resource1).(*v3clusterpb.Cluster)
	resource2.Name = resourceName2

	tests := []struct {
		desc                     string
		resourceName             string
		managementServerResponse *v3discoverypb.DiscoveryResponse
		wantUpdate               xdsresource.ClusterUpdate
		wantErr                  string
		wantGenericXDSConfig     []*v3statuspb.ClientConfig_GenericXdsConfig
	}{
		{
			desc:         "badly-marshaled-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.cluster.v3.Cluster",
				VersionInfo: "1",
				Resources: []*anypb.Any{{
					TypeUrl: "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Value:   []byte{1, 2, 3, 4},
				}},
			},
			wantErr: "Cluster not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "empty-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.cluster.v3.Cluster",
				VersionInfo: "1",
			},
			wantErr: "Cluster not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "unexpected-type-in-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.cluster.v3.Cluster",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, &v3endpointpb.ClusterLoadAssignment{})},
			},
			wantErr: "Cluster not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "one-bad-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.cluster.v3.Cluster",
				VersionInfo: "1",
				Resources: []*anypb.Any{testutils.MarshalAny(t, &v3clusterpb.Cluster{
					Name:                 resourceName1,
					ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
					EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
						EdsConfig: &v3corepb.ConfigSource{
							ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
								Ads: &v3corepb.AggregatedConfigSource{},
							},
						},
						ServiceName: "eds-service-name",
					},
					LbPolicy: v3clusterpb.Cluster_MAGLEV,
				})},
			},
			wantErr: "unexpected lbPolicy MAGLEV",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_NACKED,
					ErrorState: &v3adminpb.UpdateFailureState{
						VersionInfo: "1",
					},
				},
			},
		},
		{
			desc:         "one-good-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.cluster.v3.Cluster",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1)},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:    "resource-name-1",
				EDSServiceName: "eds-service-name",
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
		{
			desc:         "two-resources-when-we-requested-one",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.cluster.v3.Cluster",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1), testutils.MarshalAny(t, resource2)},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:    "resource-name-1",
				EDSServiceName: "eds-service-name",
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.cluster.v3.Cluster",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			// Create a fake xDS management server listening on a local port,
			// and set it up with the response to send.
			mgmtServer, cleanup := startFakeManagementServer(t)
			defer cleanup()
			t.Logf("Started xDS management server on %s", mgmtServer.Address)

			// Create an xDS client talking to the above management server.
			nodeID := uuid.New().String()
			bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
			client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
				Name:               t.Name(),
				Contents:           bc,
				WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
			})
			if err != nil {
				t.Fatalf("Failed to create an xDS client: %v", err)
			}
			defer close()
			t.Logf("Created xDS client to %s", mgmtServer.Address)

			// Register a watch, and push the results on to a channel.
			cw := newClusterWatcher()
			cancel := xdsresource.WatchCluster(client, test.resourceName, cw)
			defer cancel()
			t.Logf("Registered a watch for Cluster %q", test.resourceName)

			// Wait for the discovery request to be sent out.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			val, err := mgmtServer.XDSRequestChan.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for discovery request at the management server: %v", ctx)
			}
			wantReq := &fakeserver.Request{Req: &v3discoverypb.DiscoveryRequest{
				Node: &v3corepb.Node{
					Id:            nodeID,
					UserAgentName: "gRPC Go",
					ClientFeatures: []string{
						"envoy.lb.does_not_support_overprovisioning",
						"xds.config.resource-in-sotw",
					},
				},
				ResourceNames: []string{test.resourceName},
				TypeUrl:       "type.googleapis.com/envoy.config.cluster.v3.Cluster",
			}}
			gotReq := val.(*fakeserver.Request)
			if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), protocmp.IgnoreFields(&v3corepb.Node{}, "user_agent_version")); diff != "" {
				t.Fatalf("Discovery request received with unexpected diff (-got +want):\n%s\n got: %+v, want: %+v", diff, gotReq, wantReq)
			}
			t.Logf("Discovery request received at management server")

			// Configure the fake management server with a response.
			mgmtServer.XDSResponseChan <- &fakeserver.Response{Resp: test.managementServerResponse}

			// Wait for an update from the xDS client and compare with expected
			// update.
			val, err = cw.updateCh.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for watch callback to invoked after response from management server: %v", err)
			}
			gotUpdate := val.(clusterUpdateErrTuple).update
			gotErr := val.(clusterUpdateErrTuple).err
			if (gotErr != nil) != (test.wantErr != "") {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			if gotErr != nil && !strings.Contains(gotErr.Error(), test.wantErr) {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}

			// For tests expected to succeed, we expect an LRS server config in
			// the update from the xDS client, because the LRS bit is turned on
			// in the cluster resource. We *cannot* set the LRS server config in
			// the test table because we do not have the address of the xDS
			// server at that point, hence we do it here before verifying the
			// received update.
			if test.wantErr == "" {
				serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: fmt.Sprintf("passthrough:///%s", mgmtServer.Address)})
				if err != nil {
					t.Fatalf("Failed to create server config for testing: %v", err)
				}
				test.wantUpdate.LRSServerConfig = serverCfg
			}
			cmpOpts := []cmp.Option{
				cmpopts.EquateEmpty(),
				cmpopts.IgnoreFields(xdsresource.ClusterUpdate{}, "Raw", "LBPolicy", "TelemetryLabels"),
			}
			if diff := cmp.Diff(test.wantUpdate, gotUpdate, cmpOpts...); diff != "" {
				t.Fatalf("Unexpected diff in metadata, diff (-want +got):\n%s", diff)
			}
			if err := compareUpdateMetadata(ctx, xdsclient.DumpResources, test.wantGenericXDSConfig); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestHandleEndpointsResponseFromManagementServer covers different scenarios
// involving receipt of a CDS response from the management server. The test
// verifies that the internal state of the xDS client (parsed resource and
// metadata) matches expectations.
func (s) TestHandleEndpointsResponseFromManagementServer(t *testing.T) {
	const (
		resourceName1 = "resource-name-1"
		resourceName2 = "resource-name-2"
	)
	resource1 := &v3endpointpb.ClusterLoadAssignment{
		ClusterName: resourceName1,
		Endpoints: []*v3endpointpb.LocalityLbEndpoints{
			{
				Locality: &v3corepb.Locality{SubZone: "locality-1"},
				LbEndpoints: []*v3endpointpb.LbEndpoint{
					{
						HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{
							Endpoint: &v3endpointpb.Endpoint{
								Address: &v3corepb.Address{
									Address: &v3corepb.Address_SocketAddress{
										SocketAddress: &v3corepb.SocketAddress{
											Protocol: v3corepb.SocketAddress_TCP,
											Address:  "addr1",
											PortSpecifier: &v3corepb.SocketAddress_PortValue{
												PortValue: uint32(314),
											},
										},
									},
								},
							},
						},
					},
				},
				LoadBalancingWeight: &wrapperspb.UInt32Value{Value: 1},
				Priority:            1,
			},
			{
				Locality: &v3corepb.Locality{SubZone: "locality-2"},
				LbEndpoints: []*v3endpointpb.LbEndpoint{
					{
						HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{
							Endpoint: &v3endpointpb.Endpoint{
								Address: &v3corepb.Address{
									Address: &v3corepb.Address_SocketAddress{
										SocketAddress: &v3corepb.SocketAddress{
											Protocol: v3corepb.SocketAddress_TCP,
											Address:  "addr2",
											PortSpecifier: &v3corepb.SocketAddress_PortValue{
												PortValue: uint32(159),
											},
										},
									},
								},
							},
						},
					},
				},
				LoadBalancingWeight: &wrapperspb.UInt32Value{Value: 1},
				Priority:            0,
			},
		},
	}
	resource2 := proto.Clone(resource1).(*v3endpointpb.ClusterLoadAssignment)
	resource2.ClusterName = resourceName2

	tests := []struct {
		desc                     string
		resourceName             string
		managementServerResponse *v3discoverypb.DiscoveryResponse
		wantUpdate               xdsresource.EndpointsUpdate
		wantErr                  string
		wantGenericXDSConfig     []*v3statuspb.ClientConfig_GenericXdsConfig
	}{
		// The first three tests involve scenarios where the response fails
		// protobuf deserialization (because it contains an invalid data or type
		// in the anypb.Any) or the requested resource is not present in the
		// response.  In either case, no resource update makes its way to the
		// top-level xDS client. An EDS response without a requested resource
		// does not mean that the resource does not exist in the server. It
		// could be part of a future update.  Therefore, the only failure mode
		// for this resource is for the watch to timeout.
		{
			desc:         "badly-marshaled-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
				VersionInfo: "1",
				Resources: []*anypb.Any{{
					TypeUrl: "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Value:   []byte{1, 2, 3, 4},
				}},
			},
			wantErr: "Endpoints not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "empty-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
				VersionInfo: "1",
			},
			wantErr: "Endpoints not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "unexpected-type-in-response",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.route.v3.RouteConfiguration",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, &v3listenerpb.Listener{})},
			},
			wantErr: "Endpoints not found in received response",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_DOES_NOT_EXIST,
				},
			},
		},
		{
			desc:         "one-bad-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
				VersionInfo: "1",
				Resources: []*anypb.Any{testutils.MarshalAny(t, &v3endpointpb.ClusterLoadAssignment{
					ClusterName: resourceName1,
					Endpoints: []*v3endpointpb.LocalityLbEndpoints{
						{
							Locality: &v3corepb.Locality{SubZone: "locality-1"},
							LbEndpoints: []*v3endpointpb.LbEndpoint{
								{
									HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{
										Endpoint: &v3endpointpb.Endpoint{
											Address: &v3corepb.Address{
												Address: &v3corepb.Address_SocketAddress{
													SocketAddress: &v3corepb.SocketAddress{
														Protocol: v3corepb.SocketAddress_TCP,
														Address:  "addr1",
														PortSpecifier: &v3corepb.SocketAddress_PortValue{
															PortValue: uint32(314),
														},
													},
												},
											},
										},
									},
									LoadBalancingWeight: &wrapperspb.UInt32Value{Value: 0},
								},
							},
							LoadBalancingWeight: &wrapperspb.UInt32Value{Value: 1},
							Priority:            1,
						},
					},
				}),
				},
			},
			wantErr: "EDS response contains an endpoint with zero weight",
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_NACKED,
					ErrorState: &v3adminpb.UpdateFailureState{
						VersionInfo: "1",
					},
				},
			},
		},
		{
			desc:         "one-good-resource",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1)},
			},
			wantUpdate: xdsresource.EndpointsUpdate{
				Localities: []xdsresource.Locality{
					{
						Endpoints: []xdsresource.Endpoint{{Address: "addr1:314", Weight: 1}},
						ID:        internal.LocalityID{SubZone: "locality-1"},
						Priority:  1,
						Weight:    1,
					},
					{
						Endpoints: []xdsresource.Endpoint{{Address: "addr2:159", Weight: 1}},
						ID:        internal.LocalityID{SubZone: "locality-2"},
						Priority:  0,
						Weight:    1,
					},
				},
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
		{
			desc:         "two-resources-when-we-requested-one",
			resourceName: resourceName1,
			managementServerResponse: &v3discoverypb.DiscoveryResponse{
				TypeUrl:     "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
				VersionInfo: "1",
				Resources:   []*anypb.Any{testutils.MarshalAny(t, resource1), testutils.MarshalAny(t, resource2)},
			},
			wantUpdate: xdsresource.EndpointsUpdate{
				Localities: []xdsresource.Locality{
					{
						Endpoints: []xdsresource.Endpoint{{Address: "addr1:314", Weight: 1}},
						ID:        internal.LocalityID{SubZone: "locality-1"},
						Priority:  1,
						Weight:    1,
					},
					{
						Endpoints: []xdsresource.Endpoint{{Address: "addr2:159", Weight: 1}},
						ID:        internal.LocalityID{SubZone: "locality-2"},
						Priority:  0,
						Weight:    1,
					},
				},
			},
			wantGenericXDSConfig: []*v3statuspb.ClientConfig_GenericXdsConfig{
				{
					TypeUrl:      "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
					Name:         resourceName1,
					ClientStatus: v3adminpb.ClientResourceStatus_ACKED,
					VersionInfo:  "1",
					XdsConfig:    testutils.MarshalAny(t, resource1),
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			// Create a fake xDS management server listening on a local port,
			// and set it up with the response to send.
			mgmtServer, cleanup := startFakeManagementServer(t)
			defer cleanup()
			t.Logf("Started xDS management server on %s", mgmtServer.Address)

			// Create an xDS client talking to the above management server.
			nodeID := uuid.New().String()
			bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)
			client, close, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
				Name:               t.Name(),
				Contents:           bc,
				WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
			})
			if err != nil {
				t.Fatalf("Failed to create an xDS client: %v", err)
			}
			defer close()
			t.Logf("Created xDS client to %s", mgmtServer.Address)

			// Register a watch, and push the results on to a channel.
			ew := newEndpointsWatcher()
			cancel := xdsresource.WatchEndpoints(client, test.resourceName, ew)
			defer cancel()
			t.Logf("Registered a watch for Endpoint %q", test.resourceName)

			// Wait for the discovery request to be sent out.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			val, err := mgmtServer.XDSRequestChan.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for discovery request at the management server: %v", ctx)
			}
			wantReq := &fakeserver.Request{Req: &v3discoverypb.DiscoveryRequest{
				Node: &v3corepb.Node{
					Id:            nodeID,
					UserAgentName: "gRPC Go",
					ClientFeatures: []string{
						"envoy.lb.does_not_support_overprovisioning",
						"xds.config.resource-in-sotw",
					},
				},
				ResourceNames: []string{test.resourceName},
				TypeUrl:       "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment",
			}}
			gotReq := val.(*fakeserver.Request)
			if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), protocmp.IgnoreFields(&v3corepb.Node{}, "user_agent_version")); diff != "" {
				t.Fatalf("Discovery request received with unexpected diff (-got +want):\n%s\n got: %+v, want: %+v", diff, gotReq, wantReq)
			}
			t.Logf("Discovery request received at management server")

			// Configure the fake management server with a response.
			mgmtServer.XDSResponseChan <- &fakeserver.Response{Resp: test.managementServerResponse}

			// Wait for an update from the xDS client and compare with expected
			// update.
			val, err = ew.updateCh.Receive(ctx)
			if err != nil {
				t.Fatalf("Timeout when waiting for watch callback to invoked after response from management server: %v", err)
			}
			gotUpdate := val.(endpointsUpdateErrTuple).update
			gotErr := val.(endpointsUpdateErrTuple).err
			if (gotErr != nil) != (test.wantErr != "") {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			if gotErr != nil && !strings.Contains(gotErr.Error(), test.wantErr) {
				t.Fatalf("Got error from handling update: %v, want %v", gotErr, test.wantErr)
			}
			cmpOpts := []cmp.Option{
				cmpopts.EquateEmpty(),
				cmpopts.IgnoreFields(xdsresource.EndpointsUpdate{}, "Raw"),
			}
			if diff := cmp.Diff(test.wantUpdate, gotUpdate, cmpOpts...); diff != "" {
				t.Fatalf("Unexpected diff in metadata, diff (-want +got):\n%s", diff)
			}
			if err := compareUpdateMetadata(ctx, xdsclient.DumpResources, test.wantGenericXDSConfig); err != nil {
				t.Fatal(err)
			}
		})
	}
}
