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
 */

package xdsresource

import (
	"fmt"
	"strings"
	"testing"
	"time"

	v2xdspb "github.com/envoyproxy/go-control-plane/envoy/api/v2"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/xds/internal/httpfilter"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v1xdsudpatypepb "github.com/cncf/xds/go/udpa/type/v1"
	v3xdsxdstypepb "github.com/cncf/xds/go/xds/type/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	rpb "github.com/envoyproxy/go-control-plane/envoy/config/rbac/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3rbacpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/rbac/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"

	_ "google.golang.org/grpc/xds/internal/httpfilter/rbac"   // Register the RBAC HTTP filter.
	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter.
)

func (s) TestUnmarshalListener_ClientSide(t *testing.T) {
	const (
		v3LDSTarget       = "lds.target.good:3333"
		v3RouteConfigName = "v3RouteConfig"
		routeName         = "routeName"
	)

	var (
		customFilter = &v3httppb.HttpFilter{
			Name:       "customFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: customFilterConfig},
		}
		oldTypedStructFilter = &v3httppb.HttpFilter{
			Name:       "customFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: testutils.MarshalAny(t, customFilterOldTypedStructConfig)},
		}
		newTypedStructFilter = &v3httppb.HttpFilter{
			Name:       "customFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: testutils.MarshalAny(t, customFilterNewTypedStructConfig)},
		}
		customOptionalFilter = &v3httppb.HttpFilter{
			Name:       "customFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: customFilterConfig},
			IsOptional: true,
		}
		customFilter2 = &v3httppb.HttpFilter{
			Name:       "customFilter2",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: customFilterConfig},
		}
		errFilter = &v3httppb.HttpFilter{
			Name:       "errFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: errFilterConfig},
		}
		errOptionalFilter = &v3httppb.HttpFilter{
			Name:       "errFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: errFilterConfig},
			IsOptional: true,
		}
		clientOnlyCustomFilter = &v3httppb.HttpFilter{
			Name:       "clientOnlyCustomFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: clientOnlyCustomFilterConfig},
		}
		serverOnlyCustomFilter = &v3httppb.HttpFilter{
			Name:       "serverOnlyCustomFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: serverOnlyCustomFilterConfig},
		}
		serverOnlyOptionalCustomFilter = &v3httppb.HttpFilter{
			Name:       "serverOnlyOptionalCustomFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: serverOnlyCustomFilterConfig},
			IsOptional: true,
		}
		unknownFilter = &v3httppb.HttpFilter{
			Name:       "unknownFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: unknownFilterConfig},
		}
		unknownOptionalFilter = &v3httppb.HttpFilter{
			Name:       "unknownFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: unknownFilterConfig},
			IsOptional: true,
		}
		v3LisWithInlineRoute = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name: v3LDSTarget,
			ApiListener: &v3listenerpb.ApiListener{
				ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
					RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
						RouteConfig: &v3routepb.RouteConfiguration{
							Name: routeName,
							VirtualHosts: []*v3routepb.VirtualHost{{
								Domains: []string{v3LDSTarget},
								Routes: []*v3routepb.Route{{
									Match: &v3routepb.RouteMatch{
										PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
									},
									Action: &v3routepb.Route_Route{
										Route: &v3routepb.RouteAction{
											ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
										}}}}}}},
					},
					HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
					CommonHttpProtocolOptions: &v3corepb.HttpProtocolOptions{
						MaxStreamDuration: durationpb.New(time.Second),
					},
				}),
			},
		})
		v3LisWithFilters = func(fs ...*v3httppb.HttpFilter) *anypb.Any {
			fs = append(fs, emptyRouterFilter)
			return testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t,
						&v3httppb.HttpConnectionManager{
							RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
								Rds: &v3httppb.Rds{
									ConfigSource: &v3corepb.ConfigSource{
										ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
									},
									RouteConfigName: v3RouteConfigName,
								},
							},
							CommonHttpProtocolOptions: &v3corepb.HttpProtocolOptions{
								MaxStreamDuration: durationpb.New(time.Second),
							},
							HttpFilters: fs,
						}),
				},
			})
		}
		v3LisToTestRBAC = func(xffNumTrustedHops uint32, originalIpDetectionExtensions []*v3corepb.TypedExtensionConfig) *anypb.Any {
			return testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t,
						&v3httppb.HttpConnectionManager{
							RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
								Rds: &v3httppb.Rds{
									ConfigSource: &v3corepb.ConfigSource{
										ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
									},
									RouteConfigName: v3RouteConfigName,
								},
							},
							CommonHttpProtocolOptions: &v3corepb.HttpProtocolOptions{
								MaxStreamDuration: durationpb.New(time.Second),
							},
							HttpFilters:                   []*v3httppb.HttpFilter{emptyRouterFilter},
							XffNumTrustedHops:             xffNumTrustedHops,
							OriginalIpDetectionExtensions: originalIpDetectionExtensions,
						}),
				},
			})
		}

		v3ListenerWithCDSConfigSourceSelf = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name: v3LDSTarget,
			ApiListener: &v3listenerpb.ApiListener{
				ApiListener: testutils.MarshalAny(t,
					&v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
							Rds: &v3httppb.Rds{
								ConfigSource: &v3corepb.ConfigSource{
									ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{},
								},
								RouteConfigName: v3RouteConfigName,
							},
						},
						HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
					}),
			},
		})
	)

	tests := []struct {
		name       string
		resource   *anypb.Any
		wantName   string
		wantUpdate ListenerUpdate
		wantErr    bool
	}{
		{
			name:     "non-listener resource",
			resource: &anypb.Any{TypeUrl: version.V3HTTPConnManagerURL},
			wantErr:  true,
		},
		{
			name: "badly marshaled listener resource",
			resource: &anypb.Any{
				TypeUrl: version.V3ListenerURL,
				Value: func() []byte {
					lis := &v3listenerpb.Listener{
						Name: v3LDSTarget,
						ApiListener: &v3listenerpb.ApiListener{
							ApiListener: &anypb.Any{
								TypeUrl: version.V3HTTPConnManagerURL,
								Value:   []byte{1, 2, 3, 4},
							},
						},
					}
					mLis, _ := proto.Marshal(lis)
					return mLis
				}(),
			},
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name: "wrong type in apiListener",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v2xdspb.Listener{}),
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name: "empty httpConnMgr in apiListener",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
							Rds: &v3httppb.Rds{},
						},
					}),
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name: "scopedRoutes routeConfig in apiListener",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_ScopedRoutes{},
					}),
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "rds.ConfigSource in apiListener is Self",
			resource: v3ListenerWithCDSConfigSourceSelf,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName,
				HTTPFilters:     []HTTPFilter{makeRouterFilter(t)},
				Raw:             v3ListenerWithCDSConfigSourceSelf,
			},
		},
		{
			name: "rds.ConfigSource in apiListener is not ADS or Self",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
							Rds: &v3httppb.Rds{
								ConfigSource: &v3corepb.ConfigSource{
									ConfigSourceSpecifier: &v3corepb.ConfigSource_Path{
										Path: "/some/path",
									},
								},
								RouteConfigName: v3RouteConfigName,
							},
						},
					}),
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with no filters",
			resource: v3LisWithFilters(),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName:   v3RouteConfigName,
				MaxStreamDuration: time.Second,
				HTTPFilters:       makeRouterFilterList(t),
				Raw:               v3LisWithFilters(),
			},
		},
		{
			name: "v3 no terminal filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ApiListener: &v3listenerpb.ApiListener{
					ApiListener: testutils.MarshalAny(t,
						&v3httppb.HttpConnectionManager{
							RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
								Rds: &v3httppb.Rds{
									ConfigSource: &v3corepb.ConfigSource{
										ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
									},
									RouteConfigName: v3RouteConfigName,
								},
							},
							CommonHttpProtocolOptions: &v3corepb.HttpProtocolOptions{
								MaxStreamDuration: durationpb.New(time.Second),
							},
						}),
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with custom filter",
			resource: v3LisWithFilters(customFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName, MaxStreamDuration: time.Second,
				HTTPFilters: []HTTPFilter{
					{
						Name:   "customFilter",
						Filter: httpFilter{},
						Config: filterConfig{Cfg: customFilterConfig},
					},
					makeRouterFilter(t),
				},
				Raw: v3LisWithFilters(customFilter),
			},
		},
		{
			name:     "v3 with custom filter in old typed struct",
			resource: v3LisWithFilters(oldTypedStructFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName, MaxStreamDuration: time.Second,
				HTTPFilters: []HTTPFilter{
					{
						Name:   "customFilter",
						Filter: httpFilter{},
						Config: filterConfig{Cfg: customFilterOldTypedStructConfig},
					},
					makeRouterFilter(t),
				},
				Raw: v3LisWithFilters(oldTypedStructFilter),
			},
		},
		{
			name:     "v3 with custom filter in new typed struct",
			resource: v3LisWithFilters(newTypedStructFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName, MaxStreamDuration: time.Second,
				HTTPFilters: []HTTPFilter{
					{
						Name:   "customFilter",
						Filter: httpFilter{},
						Config: filterConfig{Cfg: customFilterNewTypedStructConfig},
					},
					makeRouterFilter(t),
				},
				Raw: v3LisWithFilters(newTypedStructFilter),
			},
		},
		{
			name:     "v3 with optional custom filter",
			resource: v3LisWithFilters(customOptionalFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName, MaxStreamDuration: time.Second,
				HTTPFilters: []HTTPFilter{
					{
						Name:   "customFilter",
						Filter: httpFilter{},
						Config: filterConfig{Cfg: customFilterConfig},
					},
					makeRouterFilter(t),
				},
				Raw: v3LisWithFilters(customOptionalFilter),
			},
		},
		{
			name:     "v3 with two filters with same name",
			resource: v3LisWithFilters(customFilter, customFilter),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with two filters - same type different name",
			resource: v3LisWithFilters(customFilter, customFilter2),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName, MaxStreamDuration: time.Second,
				HTTPFilters: []HTTPFilter{{
					Name:   "customFilter",
					Filter: httpFilter{},
					Config: filterConfig{Cfg: customFilterConfig},
				}, {
					Name:   "customFilter2",
					Filter: httpFilter{},
					Config: filterConfig{Cfg: customFilterConfig},
				},
					makeRouterFilter(t),
				},
				Raw: v3LisWithFilters(customFilter, customFilter2),
			},
		},
		{
			name:     "v3 with server-only filter",
			resource: v3LisWithFilters(serverOnlyCustomFilter),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with optional server-only filter",
			resource: v3LisWithFilters(serverOnlyOptionalCustomFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName:   v3RouteConfigName,
				MaxStreamDuration: time.Second,
				Raw:               v3LisWithFilters(serverOnlyOptionalCustomFilter),
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			name:     "v3 with client-only filter",
			resource: v3LisWithFilters(clientOnlyCustomFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName: v3RouteConfigName, MaxStreamDuration: time.Second,
				HTTPFilters: []HTTPFilter{
					{
						Name:   "clientOnlyCustomFilter",
						Filter: clientOnlyHTTPFilter{},
						Config: filterConfig{Cfg: clientOnlyCustomFilterConfig},
					},
					makeRouterFilter(t)},
				Raw: v3LisWithFilters(clientOnlyCustomFilter),
			},
		},
		{
			name:     "v3 with err filter",
			resource: v3LisWithFilters(errFilter),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with optional err filter",
			resource: v3LisWithFilters(errOptionalFilter),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with unknown filter",
			resource: v3LisWithFilters(unknownFilter),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 with unknown filter (optional)",
			resource: v3LisWithFilters(unknownOptionalFilter),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName:   v3RouteConfigName,
				MaxStreamDuration: time.Second,
				HTTPFilters:       makeRouterFilterList(t),
				Raw:               v3LisWithFilters(unknownOptionalFilter),
			},
		},
		{
			name:     "v3 listener resource",
			resource: v3LisWithFilters(),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName:   v3RouteConfigName,
				MaxStreamDuration: time.Second,
				HTTPFilters:       makeRouterFilterList(t),
				Raw:               v3LisWithFilters(),
			},
		},
		{
			name:     "v3 listener resource wrapped",
			resource: testutils.MarshalAny(t, &v3discoverypb.Resource{Resource: v3LisWithFilters()}),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName:   v3RouteConfigName,
				MaxStreamDuration: time.Second,
				HTTPFilters:       makeRouterFilterList(t),
				Raw:               v3LisWithFilters(),
			},
		},
		// "To allow equating RBAC's direct_remote_ip and
		// remote_ip...HttpConnectionManager.xff_num_trusted_hops must be unset
		// or zero and HttpConnectionManager.original_ip_detection_extensions
		// must be empty." - A41
		{
			name:     "rbac-allow-equating-direct-remote-ip-and-remote-ip-valid",
			resource: v3LisToTestRBAC(0, nil),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				RouteConfigName:   v3RouteConfigName,
				MaxStreamDuration: time.Second,
				HTTPFilters:       []HTTPFilter{makeRouterFilter(t)},
				Raw:               v3LisToTestRBAC(0, nil),
			},
		},
		// In order to support xDS Configured RBAC HTTPFilter equating direct
		// remote ip and remote ip, xffNumTrustedHops cannot be greater than
		// zero. This is because if you can trust a ingress proxy hop when
		// determining an origin clients ip address, direct remote ip != remote
		// ip.
		{
			name:     "rbac-allow-equating-direct-remote-ip-and-remote-ip-invalid-num-untrusted-hops",
			resource: v3LisToTestRBAC(1, nil),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		// In order to support xDS Configured RBAC HTTPFilter equating direct
		// remote ip and remote ip, originalIpDetectionExtensions must be empty.
		// This is because if you have to ask ip-detection-extension for the
		// original ip, direct remote ip might not equal remote ip.
		{
			name:     "rbac-allow-equating-direct-remote-ip-and-remote-ip-invalid-original-ip-detection-extension",
			resource: v3LisToTestRBAC(0, []*v3corepb.TypedExtensionConfig{{Name: "something"}}),
			wantName: v3LDSTarget,
			wantErr:  true,
		},
		{
			name:     "v3 listener with inline route configuration",
			resource: v3LisWithInlineRoute,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InlineRouteConfig: &RouteConfigUpdate{
					VirtualHosts: []*VirtualHost{{
						Domains: []string{v3LDSTarget},
						Routes:  []*Route{{Prefix: newStringP("/"), WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}}, ActionType: RouteActionRoute}},
					}}},
				MaxStreamDuration: time.Second,
				Raw:               v3LisWithInlineRoute,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			name, update, err := unmarshalListenerResource(test.resource)
			if (err != nil) != test.wantErr {
				t.Errorf("unmarshalListenerResource(%s), got err: %v, wantErr: %v", pretty.ToJSON(test.resource), err, test.wantErr)
			}
			if name != test.wantName {
				t.Errorf("unmarshalListenerResource(%s), got name: %s, want: %s", pretty.ToJSON(test.resource), name, test.wantName)
			}
			if diff := cmp.Diff(update, test.wantUpdate, cmpOpts); diff != "" {
				t.Errorf("unmarshalListenerResource(%s), got unexpected update, diff (-got +want): %v", pretty.ToJSON(test.resource), diff)
			}
		})
	}
}

func (s) TestUnmarshalListener_ServerSide(t *testing.T) {
	const (
		v3LDSTarget = "grpc/server?xds.resource.listening_address=0.0.0.0:9999"
		testVersion = "test-version-lds-server"
	)

	var (
		serverOnlyCustomFilter = &v3httppb.HttpFilter{
			Name:       "serverOnlyCustomFilter",
			ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: serverOnlyCustomFilterConfig},
		}
		routeConfig = &v3routepb.RouteConfiguration{
			Name: "routeName",
			VirtualHosts: []*v3routepb.VirtualHost{{
				Domains: []string{"lds.target.good:3333"},
				Routes: []*v3routepb.Route{{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
					},
					Action: &v3routepb.Route_NonForwardingAction{},
				}}}}}
		inlineRouteConfig = &RouteConfigUpdate{
			VirtualHosts: []*VirtualHost{{
				Domains: []string{"lds.target.good:3333"},
				Routes:  []*Route{{Prefix: newStringP("/"), ActionType: RouteActionNonForwardingAction}},
			}}}
		emptyValidNetworkFilters = []*v3listenerpb.Filter{
			{
				Name: "filter-1",
				ConfigType: &v3listenerpb.Filter_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
						RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
							RouteConfig: routeConfig,
						},
						HttpFilters: []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
					}),
				},
			},
		}
		localSocketAddress = &v3corepb.Address{
			Address: &v3corepb.Address_SocketAddress{
				SocketAddress: &v3corepb.SocketAddress{
					Address: "0.0.0.0",
					PortSpecifier: &v3corepb.SocketAddress_PortValue{
						PortValue: 9999,
					},
				},
			},
		}
		listenerEmptyTransportSocket = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name:    "filter-chain-1",
					Filters: emptyValidNetworkFilters,
				},
			},
		})
		listenerNoValidationContextDeprecatedFields = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name:    "filter-chain-1",
					Filters: emptyValidNetworkFilters,
					TransportSocket: &v3corepb.TransportSocket{
						Name: "envoy.transport_sockets.tls",
						ConfigType: &v3corepb.TransportSocket_TypedConfig{
							TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
								CommonTlsContext: &v3tlspb.CommonTlsContext{
									TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
										InstanceName:    "identityPluginInstance",
										CertificateName: "identityCertName",
									},
								},
							}),
						},
					},
				},
			},
			DefaultFilterChain: &v3listenerpb.FilterChain{
				Name:    "default-filter-chain-1",
				Filters: emptyValidNetworkFilters,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
									InstanceName:    "defaultIdentityPluginInstance",
									CertificateName: "defaultIdentityCertName",
								},
							},
						}),
					},
				},
			},
		})
		listenerNoValidationContextNewFields = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name:    "filter-chain-1",
					Filters: emptyValidNetworkFilters,
					TransportSocket: &v3corepb.TransportSocket{
						Name: "envoy.transport_sockets.tls",
						ConfigType: &v3corepb.TransportSocket_TypedConfig{
							TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
								CommonTlsContext: &v3tlspb.CommonTlsContext{
									TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName:    "identityPluginInstance",
										CertificateName: "identityCertName",
									},
								},
							}),
						},
					},
				},
			},
			DefaultFilterChain: &v3listenerpb.FilterChain{
				Name:    "default-filter-chain-1",
				Filters: emptyValidNetworkFilters,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
									InstanceName:    "defaultIdentityPluginInstance",
									CertificateName: "defaultIdentityCertName",
								},
							},
						}),
					},
				},
			},
		})
		listenerWithValidationContextDeprecatedFields = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name:    "filter-chain-1",
					Filters: emptyValidNetworkFilters,
					TransportSocket: &v3corepb.TransportSocket{
						Name: "envoy.transport_sockets.tls",
						ConfigType: &v3corepb.TransportSocket_TypedConfig{
							TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
								RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
								CommonTlsContext: &v3tlspb.CommonTlsContext{
									TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
										InstanceName:    "identityPluginInstance",
										CertificateName: "identityCertName",
									},
									ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
										ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    "rootPluginInstance",
											CertificateName: "rootCertName",
										},
									},
								},
							}),
						},
					},
				},
			},
			DefaultFilterChain: &v3listenerpb.FilterChain{
				Name:    "default-filter-chain-1",
				Filters: emptyValidNetworkFilters,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
							RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
									InstanceName:    "defaultIdentityPluginInstance",
									CertificateName: "defaultIdentityCertName",
								},
								ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
									ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
										InstanceName:    "defaultRootPluginInstance",
										CertificateName: "defaultRootCertName",
									},
								},
							},
						}),
					},
				},
			},
		})
		listenerWithValidationContextNewFields = testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name:    "filter-chain-1",
					Filters: emptyValidNetworkFilters,
					TransportSocket: &v3corepb.TransportSocket{
						Name: "envoy.transport_sockets.tls",
						ConfigType: &v3corepb.TransportSocket_TypedConfig{
							TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
								RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
								CommonTlsContext: &v3tlspb.CommonTlsContext{
									TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName:    "identityPluginInstance",
										CertificateName: "identityCertName",
									},
									ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
										ValidationContext: &v3tlspb.CertificateValidationContext{
											CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
												InstanceName:    "rootPluginInstance",
												CertificateName: "rootCertName",
											},
										},
									},
								},
							}),
						},
					},
				},
			},
			DefaultFilterChain: &v3listenerpb.FilterChain{
				Name:    "default-filter-chain-1",
				Filters: emptyValidNetworkFilters,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
							RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
									InstanceName:    "defaultIdentityPluginInstance",
									CertificateName: "defaultIdentityCertName",
								},
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
												InstanceName:    "defaultRootPluginInstance",
												CertificateName: "defaultRootCertName",
											},
										},
									},
								},
							},
						}),
					},
				},
			},
		})
	)
	v3LisToTestRBAC := func(xffNumTrustedHops uint32, originalIpDetectionExtensions []*v3corepb.TypedExtensionConfig) *anypb.Any {
		return testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name: "filter-chain-1",
					Filters: []*v3listenerpb.Filter{
						{
							Name: "filter-1",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
										RouteConfig: routeConfig,
									},
									HttpFilters:                   []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
									XffNumTrustedHops:             xffNumTrustedHops,
									OriginalIpDetectionExtensions: originalIpDetectionExtensions,
								}),
							},
						},
					},
				},
			},
		})
	}
	v3LisWithBadRBACConfiguration := func(rbacCfg *v3rbacpb.RBAC) *anypb.Any {
		return testutils.MarshalAny(t, &v3listenerpb.Listener{
			Name:    v3LDSTarget,
			Address: localSocketAddress,
			FilterChains: []*v3listenerpb.FilterChain{
				{
					Name: "filter-chain-1",
					Filters: []*v3listenerpb.Filter{
						{
							Name: "filter-1",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
										RouteConfig: routeConfig,
									},
									HttpFilters: []*v3httppb.HttpFilter{e2e.HTTPFilter("rbac", rbacCfg), e2e.RouterHTTPFilter},
								}),
							},
						},
					},
				},
			},
		})
	}
	badRBACCfgRegex := &v3rbacpb.RBAC{
		Rules: &rpb.RBAC{
			Action: rpb.RBAC_ALLOW,
			Policies: map[string]*rpb.Policy{
				"bad-regex-value": {
					Permissions: []*rpb.Permission{
						{Rule: &rpb.Permission_Any{Any: true}},
					},
					Principals: []*rpb.Principal{
						{Identifier: &rpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SafeRegexMatch{SafeRegexMatch: &v3matcherpb.RegexMatcher{Regex: "["}}}}},
					},
				},
			},
		},
	}
	badRBACCfgDestIP := &v3rbacpb.RBAC{
		Rules: &rpb.RBAC{
			Action: rpb.RBAC_ALLOW,
			Policies: map[string]*rpb.Policy{
				"certain-destination-ip": {
					Permissions: []*rpb.Permission{
						{Rule: &rpb.Permission_DestinationIp{DestinationIp: &v3corepb.CidrRange{AddressPrefix: "not a correct address", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
					},
					Principals: []*rpb.Principal{
						{Identifier: &rpb.Principal_Any{Any: true}},
					},
				},
			},
		},
	}

	tests := []struct {
		name       string
		resource   *anypb.Any
		wantName   string
		wantUpdate ListenerUpdate
		wantErr    string
	}{
		{
			name: "non-empty listener filters",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name: v3LDSTarget,
				ListenerFilters: []*v3listenerpb.ListenerFilter{
					{Name: "listener-filter-1"},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "unsupported field 'listener_filters'",
		},
		{
			name: "use_original_dst is set",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:           v3LDSTarget,
				UseOriginalDst: &wrapperspb.BoolValue{Value: true},
			}),
			wantName: v3LDSTarget,
			wantErr:  "unsupported field 'use_original_dst'",
		},
		{
			name:     "no address field",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{Name: v3LDSTarget}),
			wantName: v3LDSTarget,
			wantErr:  "no address field in LDS response",
		},
		{
			name: "no socket address field",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: &v3corepb.Address{},
			}),
			wantName: v3LDSTarget,
			wantErr:  "no socket_address field in LDS response",
		},
		{
			name: "no filter chains and no default filter chain",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{DestinationPort: &wrapperspb.UInt32Value{Value: 666}},
						Filters:          emptyValidNetworkFilters,
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "no supported filter chains and no default filter chain",
		},
		{
			name: "missing http connection manager network filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "missing HttpConnectionManager filter",
		},
		{
			name: "missing filter name in http filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{}),
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "missing name field in filter",
		},
		{
			name: "duplicate filter names in http filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "duplicate filter name",
		},
		{
			name: "no terminal filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
									}),
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "http filters list is empty",
		},
		{
			name: "terminal filter not last",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter, serverOnlyCustomFilter},
									}),
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "is a terminal filter but it is not last in the filter chain",
		},
		{
			name: "last not terminal filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
										HttpFilters: []*v3httppb.HttpFilter{serverOnlyCustomFilter},
									}),
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "is not a terminal filter",
		},
		{
			name: "unsupported oneof in typed config of http filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name:       "name",
								ConfigType: &v3listenerpb.Filter_ConfigDiscovery{},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "unsupported config_type",
		},
		{
			name: "overlapping filter chain match criteria",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{1, 2, 3, 4, 5}},
						Filters:          emptyValidNetworkFilters,
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{},
						Filters:          emptyValidNetworkFilters,
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{5, 6, 7}},
						Filters:          emptyValidNetworkFilters,
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "multiple filter chains with overlapping matching rules are defined",
		},
		{
			name: "unsupported network filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.LocalReplyConfig{}),
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "unsupported network filter",
		},
		{
			name: "badly marshaled network filter",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "name",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: &anypb.Any{
										TypeUrl: version.V3HTTPConnManagerURL,
										Value:   []byte{1, 2, 3, 4},
									},
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "failed unmarshalling of network filter",
		},
		{
			name: "unexpected transport socket name",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "unsupported-transport-socket-name",
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "transport_socket field has unexpected name",
		},
		{
			name: "unexpected transport socket typedConfig URL",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{}),
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "transport_socket field has unexpected typeURL",
		},
		{
			name: "badly marshaled transport socket",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: &anypb.Any{
									TypeUrl: version.V3DownstreamTLSContextURL,
									Value:   []byte{1, 2, 3, 4},
								},
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "failed to unmarshal DownstreamTlsContext in LDS response",
		},
		{
			name: "missing CommonTlsContext",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{}),
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "DownstreamTlsContext in LDS response does not contain a CommonTlsContext",
		},
		{
			name:     "rbac-allow-equating-direct-remote-ip-and-remote-ip-valid",
			resource: v3LisToTestRBAC(0, nil),
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InboundListenerCfg: &InboundListenerConfig{
					Address: "0.0.0.0",
					Port:    "9999",
					FilterChains: &FilterChainManager{
						dstPrefixMap: map[string]*destPrefixEntry{
							unspecifiedPrefixMapKey: {
								srcTypeArr: [3]*sourcePrefixes{
									{
										srcPrefixMap: map[string]*sourcePrefixEntry{
											unspecifiedPrefixMapKey: {
												srcPortMap: map[int]*FilterChain{
													0: {
														InlineRouteConfig: inlineRouteConfig,
														HTTPFilters:       makeRouterFilterList(t),
													},
												},
											},
										},
									},
								},
							},
						},
					},
				},
				Raw: listenerEmptyTransportSocket,
			},
		},
		{
			name:     "rbac-allow-equating-direct-remote-ip-and-remote-ip-invalid-num-untrusted-hops",
			resource: v3LisToTestRBAC(1, nil),
			wantName: v3LDSTarget,
			wantErr:  "xff_num_trusted_hops must be unset or zero",
		},
		{
			name:     "rbac-allow-equating-direct-remote-ip-and-remote-ip-invalid-original-ip-detection-extension",
			resource: v3LisToTestRBAC(0, []*v3corepb.TypedExtensionConfig{{Name: "something"}}),
			wantName: v3LDSTarget,
			wantErr:  "original_ip_detection_extensions must be empty",
		},
		{
			name:     "rbac-with-invalid-regex",
			resource: v3LisWithBadRBACConfiguration(badRBACCfgRegex),
			wantName: v3LDSTarget,
			wantErr:  "error parsing config for filter",
		},
		{
			name:     "rbac-with-invalid-destination-ip-matcher",
			resource: v3LisWithBadRBACConfiguration(badRBACCfgDestIP),
			wantName: v3LDSTarget,
			wantErr:  "error parsing config for filter",
		},
		{
			name: "unsupported validation context in transport socket",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									CommonTlsContext: &v3tlspb.CommonTlsContext{
										ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextSdsSecretConfig{
											ValidationContextSdsSecretConfig: &v3tlspb.SdsSecretConfig{
												Name: "foo-sds-secret",
											},
										},
									},
								}),
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "validation context contains unexpected type",
		},
		{
			name:     "empty transport socket",
			resource: listenerEmptyTransportSocket,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InboundListenerCfg: &InboundListenerConfig{
					Address: "0.0.0.0",
					Port:    "9999",
					FilterChains: &FilterChainManager{
						dstPrefixMap: map[string]*destPrefixEntry{
							unspecifiedPrefixMapKey: {
								srcTypeArr: [3]*sourcePrefixes{
									{
										srcPrefixMap: map[string]*sourcePrefixEntry{
											unspecifiedPrefixMapKey: {
												srcPortMap: map[int]*FilterChain{
													0: {
														InlineRouteConfig: inlineRouteConfig,
														HTTPFilters:       makeRouterFilterList(t),
													},
												},
											},
										},
									},
								},
							},
						},
					},
				},
				Raw: listenerEmptyTransportSocket,
			},
		},
		{
			name: "no identity and root certificate providers using deprecated fields",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
									CommonTlsContext: &v3tlspb.CommonTlsContext{
										TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    "identityPluginInstance",
											CertificateName: "identityCertName",
										},
									},
								}),
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "security configuration on the server-side does not contain root certificate provider instance name, but require_client_cert field is set",
		},
		{
			name: "no identity and root certificate providers using new fields",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
									CommonTlsContext: &v3tlspb.CommonTlsContext{
										TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
											InstanceName:    "identityPluginInstance",
											CertificateName: "identityCertName",
										},
									},
								}),
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "security configuration on the server-side does not contain root certificate provider instance name, but require_client_cert field is set",
		},
		{
			name: "no identity certificate provider with require_client_cert",
			resource: testutils.MarshalAny(t, &v3listenerpb.Listener{
				Name:    v3LDSTarget,
				Address: localSocketAddress,
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters,
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									CommonTlsContext: &v3tlspb.CommonTlsContext{},
								}),
							},
						},
					},
				},
			}),
			wantName: v3LDSTarget,
			wantErr:  "security configuration on the server-side does not contain identity certificate provider instance name",
		},
		{
			name:     "happy case with no validation context using deprecated fields",
			resource: listenerNoValidationContextDeprecatedFields,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InboundListenerCfg: &InboundListenerConfig{
					Address: "0.0.0.0",
					Port:    "9999",
					FilterChains: &FilterChainManager{
						dstPrefixMap: map[string]*destPrefixEntry{
							unspecifiedPrefixMapKey: {
								srcTypeArr: [3]*sourcePrefixes{
									{
										srcPrefixMap: map[string]*sourcePrefixEntry{
											unspecifiedPrefixMapKey: {
												srcPortMap: map[int]*FilterChain{
													0: {
														SecurityCfg: &SecurityConfig{
															IdentityInstanceName: "identityPluginInstance",
															IdentityCertName:     "identityCertName",
														},
														InlineRouteConfig: inlineRouteConfig,
														HTTPFilters:       makeRouterFilterList(t),
													},
												},
											},
										},
									},
								},
							},
						},
						def: &FilterChain{
							SecurityCfg: &SecurityConfig{
								IdentityInstanceName: "defaultIdentityPluginInstance",
								IdentityCertName:     "defaultIdentityCertName",
							},
							InlineRouteConfig: inlineRouteConfig,
							HTTPFilters:       makeRouterFilterList(t),
						},
					},
				},
				Raw: listenerNoValidationContextDeprecatedFields,
			},
		},
		{
			name:     "happy case with no validation context using new fields",
			resource: listenerNoValidationContextNewFields,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InboundListenerCfg: &InboundListenerConfig{
					Address: "0.0.0.0",
					Port:    "9999",
					FilterChains: &FilterChainManager{
						dstPrefixMap: map[string]*destPrefixEntry{
							unspecifiedPrefixMapKey: {
								srcTypeArr: [3]*sourcePrefixes{
									{
										srcPrefixMap: map[string]*sourcePrefixEntry{
											unspecifiedPrefixMapKey: {
												srcPortMap: map[int]*FilterChain{
													0: {
														SecurityCfg: &SecurityConfig{
															IdentityInstanceName: "identityPluginInstance",
															IdentityCertName:     "identityCertName",
														},
														InlineRouteConfig: inlineRouteConfig,
														HTTPFilters:       makeRouterFilterList(t),
													},
												},
											},
										},
									},
								},
							},
						},
						def: &FilterChain{
							SecurityCfg: &SecurityConfig{
								IdentityInstanceName: "defaultIdentityPluginInstance",
								IdentityCertName:     "defaultIdentityCertName",
							},
							InlineRouteConfig: inlineRouteConfig,
							HTTPFilters:       makeRouterFilterList(t),
						},
					},
				},
				Raw: listenerNoValidationContextNewFields,
			},
		},
		{
			name:     "happy case with validation context provider instance with deprecated fields",
			resource: listenerWithValidationContextDeprecatedFields,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InboundListenerCfg: &InboundListenerConfig{
					Address: "0.0.0.0",
					Port:    "9999",
					FilterChains: &FilterChainManager{
						dstPrefixMap: map[string]*destPrefixEntry{
							unspecifiedPrefixMapKey: {
								srcTypeArr: [3]*sourcePrefixes{
									{
										srcPrefixMap: map[string]*sourcePrefixEntry{
											unspecifiedPrefixMapKey: {
												srcPortMap: map[int]*FilterChain{
													0: {
														SecurityCfg: &SecurityConfig{
															RootInstanceName:     "rootPluginInstance",
															RootCertName:         "rootCertName",
															IdentityInstanceName: "identityPluginInstance",
															IdentityCertName:     "identityCertName",
															RequireClientCert:    true,
														},
														InlineRouteConfig: inlineRouteConfig,
														HTTPFilters:       makeRouterFilterList(t),
													},
												},
											},
										},
									},
								},
							},
						},
						def: &FilterChain{
							SecurityCfg: &SecurityConfig{
								RootInstanceName:     "defaultRootPluginInstance",
								RootCertName:         "defaultRootCertName",
								IdentityInstanceName: "defaultIdentityPluginInstance",
								IdentityCertName:     "defaultIdentityCertName",
								RequireClientCert:    true,
							},
							InlineRouteConfig: inlineRouteConfig,
							HTTPFilters:       makeRouterFilterList(t),
						},
					},
				},
				Raw: listenerWithValidationContextDeprecatedFields,
			},
		},
		{
			name:     "happy case with validation context provider instance with new fields",
			resource: listenerWithValidationContextNewFields,
			wantName: v3LDSTarget,
			wantUpdate: ListenerUpdate{
				InboundListenerCfg: &InboundListenerConfig{
					Address: "0.0.0.0",
					Port:    "9999",
					FilterChains: &FilterChainManager{
						dstPrefixMap: map[string]*destPrefixEntry{
							unspecifiedPrefixMapKey: {
								srcTypeArr: [3]*sourcePrefixes{
									{
										srcPrefixMap: map[string]*sourcePrefixEntry{
											unspecifiedPrefixMapKey: {
												srcPortMap: map[int]*FilterChain{
													0: {
														SecurityCfg: &SecurityConfig{
															RootInstanceName:     "rootPluginInstance",
															RootCertName:         "rootCertName",
															IdentityInstanceName: "identityPluginInstance",
															IdentityCertName:     "identityCertName",
															RequireClientCert:    true,
														},
														InlineRouteConfig: inlineRouteConfig,
														HTTPFilters:       makeRouterFilterList(t),
													},
												},
											},
										},
									},
								},
							},
						},
						def: &FilterChain{
							SecurityCfg: &SecurityConfig{
								RootInstanceName:     "defaultRootPluginInstance",
								RootCertName:         "defaultRootCertName",
								IdentityInstanceName: "defaultIdentityPluginInstance",
								IdentityCertName:     "defaultIdentityCertName",
								RequireClientCert:    true,
							},
							InlineRouteConfig: inlineRouteConfig,
							HTTPFilters:       makeRouterFilterList(t),
						},
					},
				},
				Raw: listenerWithValidationContextNewFields,
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			name, update, err := unmarshalListenerResource(test.resource)
			if err != nil && !strings.Contains(err.Error(), test.wantErr) {
				t.Errorf("unmarshalListenerResource(%s) = %v wantErr: %q", pretty.ToJSON(test.resource), err, test.wantErr)
			}
			if name != test.wantName {
				t.Errorf("unmarshalListenerResource(%s), got name: %s, want: %s", pretty.ToJSON(test.resource), name, test.wantName)
			}
			if diff := cmp.Diff(update, test.wantUpdate, cmpOpts); diff != "" {
				t.Errorf("unmarshalListenerResource(%s), got unexpected update, diff (-got +want): %v", pretty.ToJSON(test.resource), diff)
			}
		})
	}
}

type filterConfig struct {
	httpfilter.FilterConfig
	Cfg      proto.Message
	Override proto.Message
}

// httpFilter allows testing the http filter registry and parsing functionality.
type httpFilter struct {
	httpfilter.ClientInterceptorBuilder
	httpfilter.ServerInterceptorBuilder
}

func (httpFilter) TypeURLs() []string { return []string{"custom.filter"} }

func (httpFilter) ParseFilterConfig(cfg proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfig{Cfg: cfg}, nil
}

func (httpFilter) ParseFilterConfigOverride(override proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfig{Override: override}, nil
}

func (httpFilter) IsTerminal() bool {
	return false
}

// errHTTPFilter returns errors no matter what is passed to ParseFilterConfig.
type errHTTPFilter struct {
	httpfilter.ClientInterceptorBuilder
}

func (errHTTPFilter) TypeURLs() []string { return []string{"err.custom.filter"} }

func (errHTTPFilter) ParseFilterConfig(proto.Message) (httpfilter.FilterConfig, error) {
	return nil, fmt.Errorf("error from ParseFilterConfig")
}

func (errHTTPFilter) ParseFilterConfigOverride(proto.Message) (httpfilter.FilterConfig, error) {
	return nil, fmt.Errorf("error from ParseFilterConfigOverride")
}

func (errHTTPFilter) IsTerminal() bool {
	return false
}

func init() {
	httpfilter.Register(httpFilter{})
	httpfilter.Register(errHTTPFilter{})
	httpfilter.Register(serverOnlyHTTPFilter{})
	httpfilter.Register(clientOnlyHTTPFilter{})
}

// serverOnlyHTTPFilter does not implement ClientInterceptorBuilder
type serverOnlyHTTPFilter struct {
	httpfilter.ServerInterceptorBuilder
}

func (serverOnlyHTTPFilter) TypeURLs() []string { return []string{"serverOnly.custom.filter"} }

func (serverOnlyHTTPFilter) ParseFilterConfig(cfg proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfig{Cfg: cfg}, nil
}

func (serverOnlyHTTPFilter) ParseFilterConfigOverride(override proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfig{Override: override}, nil
}

func (serverOnlyHTTPFilter) IsTerminal() bool {
	return false
}

// clientOnlyHTTPFilter does not implement ServerInterceptorBuilder
type clientOnlyHTTPFilter struct {
	httpfilter.ClientInterceptorBuilder
}

func (clientOnlyHTTPFilter) TypeURLs() []string { return []string{"clientOnly.custom.filter"} }

func (clientOnlyHTTPFilter) ParseFilterConfig(cfg proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfig{Cfg: cfg}, nil
}

func (clientOnlyHTTPFilter) ParseFilterConfigOverride(override proto.Message) (httpfilter.FilterConfig, error) {
	return filterConfig{Override: override}, nil
}

func (clientOnlyHTTPFilter) IsTerminal() bool {
	return false
}

var customFilterConfig = &anypb.Any{
	TypeUrl: "custom.filter",
	Value:   []byte{1, 2, 3},
}

var errFilterConfig = &anypb.Any{
	TypeUrl: "err.custom.filter",
	Value:   []byte{1, 2, 3},
}

var serverOnlyCustomFilterConfig = &anypb.Any{
	TypeUrl: "serverOnly.custom.filter",
	Value:   []byte{1, 2, 3},
}

var clientOnlyCustomFilterConfig = &anypb.Any{
	TypeUrl: "clientOnly.custom.filter",
	Value:   []byte{1, 2, 3},
}

// This custom filter uses the old TypedStruct message from the cncf/udpa repo.
var customFilterOldTypedStructConfig = &v1xdsudpatypepb.TypedStruct{
	TypeUrl: "custom.filter",
	Value: &structpb.Struct{
		Fields: map[string]*structpb.Value{
			"foo": {Kind: &structpb.Value_StringValue{StringValue: "bar"}},
		},
	},
}

// This custom filter uses the new TypedStruct message from the cncf/xds repo.
var customFilterNewTypedStructConfig = &v3xdsxdstypepb.TypedStruct{
	TypeUrl: "custom.filter",
	Value: &structpb.Struct{
		Fields: map[string]*structpb.Value{
			"foo": {Kind: &structpb.Value_StringValue{StringValue: "bar"}},
		},
	},
}

var unknownFilterConfig = &anypb.Any{
	TypeUrl: "unknown.custom.filter",
	Value:   []byte{1, 2, 3},
}

func wrappedOptionalFilter(t *testing.T, name string) *anypb.Any {
	return testutils.MarshalAny(t, &v3routepb.FilterConfig{
		IsOptional: true,
		Config: &anypb.Any{
			TypeUrl: name,
			Value:   []byte{1, 2, 3},
		},
	})
}
