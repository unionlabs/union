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
	"context"
	"errors"
	"fmt"
	"net"
	"strings"
	"testing"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3routerpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/router/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	iresolver "google.golang.org/grpc/internal/resolver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/xds/internal/httpfilter"
	"google.golang.org/grpc/xds/internal/httpfilter/router"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
)

const (
	topLevel = "top level"
	vhLevel  = "virtual host level"
	rLevel   = "route level"
)

func emptyValidNetworkFilters(t *testing.T) []*v3listenerpb.Filter {
	return []*v3listenerpb.Filter{
		{
			Name: "filter-1",
			ConfigType: &v3listenerpb.Filter_TypedConfig{
				TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
					RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
						RouteConfig: routeConfig,
					},
					HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
				}),
			},
		},
	}
}

var (
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

	validServerSideHTTPFilter1 = &v3httppb.HttpFilter{
		Name:       "serverOnlyCustomFilter",
		ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: serverOnlyCustomFilterConfig},
	}
	validServerSideHTTPFilter2 = &v3httppb.HttpFilter{
		Name:       "serverOnlyCustomFilter2",
		ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: serverOnlyCustomFilterConfig},
	}
	emptyRouterFilter = e2e.RouterHTTPFilter
)

func makeRouterFilter(t *testing.T) HTTPFilter {
	routerBuilder := httpfilter.Get(router.TypeURL)
	routerConfig, _ := routerBuilder.ParseFilterConfig(testutils.MarshalAny(t, &v3routerpb.Router{}))
	return HTTPFilter{Name: "router", Filter: routerBuilder, Config: routerConfig}
}

func makeRouterFilterList(t *testing.T) []HTTPFilter {
	return []HTTPFilter{makeRouterFilter(t)}
}

// TestNewFilterChainImpl_Failure_BadMatchFields verifies cases where we have a
// single filter chain with match criteria that contains unsupported fields.
func (s) TestNewFilterChainImpl_Failure_BadMatchFields(t *testing.T) {
	tests := []struct {
		desc string
		lis  *v3listenerpb.Listener
	}{
		{
			desc: "unsupported destination port field",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{DestinationPort: &wrapperspb.UInt32Value{Value: 666}},
					},
				},
			},
		},
		{
			desc: "unsupported server names field",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{ServerNames: []string{"example-server"}},
					},
				},
			},
		},
		{
			desc: "unsupported transport protocol ",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{TransportProtocol: "tls"},
					},
				},
			},
		},
		{
			desc: "unsupported application protocol field",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{ApplicationProtocols: []string{"h2"}},
					},
				},
			},
		},
		{
			desc: "bad dest address prefix",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{{AddressPrefix: "a.b.c.d"}}},
					},
				},
			},
		},
		{
			desc: "bad dest prefix length",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.1.1.0", 50)}},
					},
				},
			},
		},
		{
			desc: "bad source address prefix",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePrefixRanges: []*v3corepb.CidrRange{{AddressPrefix: "a.b.c.d"}}},
					},
				},
			},
		},
		{
			desc: "bad source prefix length",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.1.1.0", 50)}},
					},
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if fci, err := NewFilterChainManager(test.lis); err == nil {
				t.Fatalf("NewFilterChainManager() returned %v when expected to fail", fci)
			}
		})
	}
}

// TestNewFilterChainImpl_Failure_OverlappingMatchingRules verifies cases where
// there are multiple filter chains and they have overlapping match rules.
func (s) TestNewFilterChainImpl_Failure_OverlappingMatchingRules(t *testing.T) {
	tests := []struct {
		desc string
		lis  *v3listenerpb.Listener
	}{
		{
			desc: "matching destination prefixes with no other matchers",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16), cidrRangeFromAddressAndPrefixLen("10.0.0.0", 0)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.2.2", 16)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
		},
		{
			desc: "matching source type",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourceType: v3listenerpb.FilterChainMatch_ANY},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourceType: v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourceType: v3listenerpb.FilterChainMatch_EXTERNAL},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourceType: v3listenerpb.FilterChainMatch_EXTERNAL},
						Filters:          emptyValidNetworkFilters(t),
					},
				},
			},
		},
		{
			desc: "matching source prefixes",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16), cidrRangeFromAddressAndPrefixLen("10.0.0.0", 0)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.2.2", 16)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
		},
		{
			desc: "matching source ports",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{1, 2, 3, 4, 5}},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{5, 6, 7}},
						Filters:          emptyValidNetworkFilters(t),
					},
				},
			},
		},
	}

	const wantErr = "multiple filter chains with overlapping matching rules are defined"
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if _, err := NewFilterChainManager(test.lis); err == nil || !strings.Contains(err.Error(), wantErr) {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: %s", err, wantErr)
			}
		})
	}
}

// TestNewFilterChainImpl_Failure_BadSecurityConfig verifies cases where the
// security configuration in the filter chain is invalid.
func (s) TestNewFilterChainImpl_Failure_BadSecurityConfig(t *testing.T) {
	tests := []struct {
		desc    string
		lis     *v3listenerpb.Listener
		wantErr string
	}{
		{
			desc:    "no filter chains",
			lis:     &v3listenerpb.Listener{},
			wantErr: "no supported filter chains and no default filter chain",
		},
		{
			desc: "unexpected transport socket name",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{Name: "unsupported-transport-socket-name"},
						Filters:         emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "transport_socket field has unexpected name",
		},
		{
			desc: "unexpected transport socket URL",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{}),
							},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "transport_socket field has unexpected typeURL",
		},
		{
			desc: "badly marshaled transport socket",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: &anypb.Any{
									TypeUrl: version.V3DownstreamTLSContextURL,
									Value:   []byte{1, 2, 3, 4},
								},
							},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "failed to unmarshal DownstreamTlsContext in LDS response",
		},
		{
			desc: "missing CommonTlsContext",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{}),
							},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "DownstreamTlsContext in LDS response does not contain a CommonTlsContext",
		},
		{
			desc: "require_sni-set-to-true-in-downstreamTlsContext",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									RequireSni: &wrapperspb.BoolValue{Value: true},
								}),
							},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "require_sni field set to true in DownstreamTlsContext message",
		},
		{
			desc: "unsupported-ocsp_staple_policy-in-downstreamTlsContext",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									OcspStaplePolicy: v3tlspb.DownstreamTlsContext_STRICT_STAPLING,
								}),
							},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "ocsp_staple_policy field set to unsupported value in DownstreamTlsContext message",
		},
		{
			desc: "unsupported validation context in transport socket",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
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
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "validation context contains unexpected type",
		},
		{
			desc: "unsupported match_subject_alt_names field in transport socket",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
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
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "validation context contains unexpected type",
		},
		{
			desc: "no root certificate provider with require_client_cert",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
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
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "security configuration on the server-side does not contain root certificate provider instance name, but require_client_cert field is set",
		},
		{
			desc: "no identity certificate provider",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						TransportSocket: &v3corepb.TransportSocket{
							Name: "envoy.transport_sockets.tls",
							ConfigType: &v3corepb.TransportSocket_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
									CommonTlsContext: &v3tlspb.CommonTlsContext{},
								}),
							},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			wantErr: "security configuration on the server-side does not contain identity certificate provider instance name",
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			_, err := NewFilterChainManager(test.lis)
			if err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: %s", err, test.wantErr)
			}
		})
	}
}

// TestNewFilterChainImpl_Success_RouteUpdate tests the construction of the
// filter chain with valid HTTP Filters present.
func (s) TestNewFilterChainImpl_Success_RouteUpdate(t *testing.T) {
	tests := []struct {
		name   string
		lis    *v3listenerpb.Listener
		wantFC *FilterChainManager
	}{
		{
			name: "rds",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
											Rds: &v3httppb.Rds{
												ConfigSource: &v3corepb.ConfigSource{
													ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
												},
												RouteConfigName: "route-1",
											},
										},
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
										Rds: &v3httppb.Rds{
											ConfigSource: &v3corepb.ConfigSource{
												ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
											},
											RouteConfigName: "route-1",
										},
									},
									HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
								}),
							},
						},
					},
				},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									unspecifiedPrefixMapKey: {
										srcPortMap: map[int]*FilterChain{
											0: {
												RouteConfigName: "route-1",
												HTTPFilters:     makeRouterFilterList(t),
											},
										},
									},
								},
							},
						},
					},
				},
				def: &FilterChain{
					RouteConfigName: "route-1",
					HTTPFilters:     makeRouterFilterList(t),
				},
				RouteConfigNames: map[string]bool{"route-1": true},
			},
		},
		{
			name: "inline route config",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
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
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
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
			wantFC: &FilterChainManager{
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
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		// two rds tests whether the Filter Chain Manager successfully persists
		// the two RDS names that need to be dynamically queried.
		{
			name: "two rds",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
											Rds: &v3httppb.Rds{
												ConfigSource: &v3corepb.ConfigSource{
													ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
												},
												RouteConfigName: "route-1",
											},
										},
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
										Rds: &v3httppb.Rds{
											ConfigSource: &v3corepb.ConfigSource{
												ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
											},
											RouteConfigName: "route-2",
										},
									},
									HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
								}),
							},
						},
					},
				},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									unspecifiedPrefixMapKey: {
										srcPortMap: map[int]*FilterChain{
											0: {
												RouteConfigName: "route-1",
												HTTPFilters:     makeRouterFilterList(t),
											},
										},
									},
								},
							},
						},
					},
				},
				def: &FilterChain{
					RouteConfigName: "route-2",
					HTTPFilters:     makeRouterFilterList(t),
				},
				RouteConfigNames: map[string]bool{
					"route-1": true,
					"route-2": true,
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			gotFC, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: nil", err)
			}
			if !cmp.Equal(gotFC, test.wantFC, cmp.AllowUnexported(FilterChainManager{}, destPrefixEntry{}, sourcePrefixes{}, sourcePrefixEntry{}), cmpOpts) {
				t.Fatalf("NewFilterChainManager() returned %+v, want: %+v", gotFC, test.wantFC)
			}
		})
	}
}

// TestNewFilterChainImpl_Failure_BadRouteUpdate verifies cases where the Route
// Update in the filter chain are invalid.
func (s) TestNewFilterChainImpl_Failure_BadRouteUpdate(t *testing.T) {
	tests := []struct {
		name    string
		lis     *v3listenerpb.Listener
		wantErr string
	}{
		{
			name: "missing-route-specifier",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{

									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
								}),
							},
						},
					},
				},
			},
			wantErr: "no RouteSpecifier",
		},
		{
			name: "not-ads",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{

									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
											Rds: &v3httppb.Rds{
												RouteConfigName: "route-1",
											},
										},
										HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
										Rds: &v3httppb.Rds{
											RouteConfigName: "route-1",
										},
									},
									HttpFilters: []*v3httppb.HttpFilter{emptyRouterFilter},
								}),
							},
						},
					},
				},
			},
			wantErr: "ConfigSource is not ADS",
		},
		{
			name: "unsupported-route-specifier",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										RouteSpecifier: &v3httppb.HttpConnectionManager_ScopedRoutes{},
										HttpFilters:    []*v3httppb.HttpFilter{emptyRouterFilter},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									RouteSpecifier: &v3httppb.HttpConnectionManager_ScopedRoutes{},
									HttpFilters:    []*v3httppb.HttpFilter{emptyRouterFilter},
								}),
							},
						},
					},
				},
			},
			wantErr: "unsupported type",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			_, err := NewFilterChainManager(test.lis)
			if err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: %s", err, test.wantErr)
			}
		})
	}
}

// TestNewFilterChainImpl_Failure_BadHTTPFilters verifies cases where the HTTP
// Filters in the filter chain are invalid.
func (s) TestNewFilterChainImpl_Failure_BadHTTPFilters(t *testing.T) {
	tests := []struct {
		name    string
		lis     *v3listenerpb.Listener
		wantErr string
	}{
		{
			name: "client side HTTP filter",
			lis: &v3listenerpb.Listener{
				Name: "grpc/server?xds.resource.listening_address=0.0.0.0:9999",
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{
											{
												Name:       "clientOnlyCustomFilter",
												ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: clientOnlyCustomFilterConfig},
											},
										},
									}),
								},
							},
						},
					},
				},
			},
			wantErr: "invalid server side HTTP Filters",
		},
		{
			name: "one valid then one invalid HTTP filter",
			lis: &v3listenerpb.Listener{
				Name: "grpc/server?xds.resource.listening_address=0.0.0.0:9999",
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{
											validServerSideHTTPFilter1,
											{
												Name:       "clientOnlyCustomFilter",
												ConfigType: &v3httppb.HttpFilter_TypedConfig{TypedConfig: clientOnlyCustomFilterConfig},
											},
										},
									}),
								},
							},
						},
					},
				},
			},
			wantErr: "invalid server side HTTP Filters",
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			_, err := NewFilterChainManager(test.lis)
			if err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: %s", err, test.wantErr)
			}
		})
	}
}

// TestNewFilterChainImpl_Success_HTTPFilters tests the construction of the
// filter chain with valid HTTP Filters present.
func (s) TestNewFilterChainImpl_Success_HTTPFilters(t *testing.T) {
	tests := []struct {
		name   string
		lis    *v3listenerpb.Listener
		wantFC *FilterChainManager
	}{
		{
			name: "singular valid http filter",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{
											validServerSideHTTPFilter1,
											emptyRouterFilter,
										},
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									HttpFilters: []*v3httppb.HttpFilter{
										validServerSideHTTPFilter1,
										emptyRouterFilter,
									},
									RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
										RouteConfig: routeConfig,
									},
								}),
							},
						},
					},
				},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									unspecifiedPrefixMapKey: {
										srcPortMap: map[int]*FilterChain{
											0: {HTTPFilters: []HTTPFilter{
												{
													Name:   "serverOnlyCustomFilter",
													Filter: serverOnlyHTTPFilter{},
													Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
												},
												makeRouterFilter(t),
											},
												InlineRouteConfig: inlineRouteConfig,
											},
										},
									},
								},
							},
						},
					},
				},
				def: &FilterChain{
					HTTPFilters: []HTTPFilter{
						{
							Name:   "serverOnlyCustomFilter",
							Filter: serverOnlyHTTPFilter{},
							Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
						},
						makeRouterFilter(t),
					},
					InlineRouteConfig: inlineRouteConfig,
				},
			},
		},
		{
			name: "two valid http filters",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{
											validServerSideHTTPFilter1,
											validServerSideHTTPFilter2,
											emptyRouterFilter,
										},
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									HttpFilters: []*v3httppb.HttpFilter{
										validServerSideHTTPFilter1,
										validServerSideHTTPFilter2,
										emptyRouterFilter,
									},
									RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
										RouteConfig: routeConfig,
									},
								}),
							},
						},
					},
				},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									unspecifiedPrefixMapKey: {
										srcPortMap: map[int]*FilterChain{
											0: {HTTPFilters: []HTTPFilter{
												{
													Name:   "serverOnlyCustomFilter",
													Filter: serverOnlyHTTPFilter{},
													Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
												},
												{
													Name:   "serverOnlyCustomFilter2",
													Filter: serverOnlyHTTPFilter{},
													Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
												},
												makeRouterFilter(t),
											},
												InlineRouteConfig: inlineRouteConfig,
											},
										},
									},
								},
							},
						},
					},
				},
				def: &FilterChain{HTTPFilters: []HTTPFilter{
					{
						Name:   "serverOnlyCustomFilter",
						Filter: serverOnlyHTTPFilter{},
						Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
					},
					{
						Name:   "serverOnlyCustomFilter2",
						Filter: serverOnlyHTTPFilter{},
						Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
					},
					makeRouterFilter(t),
				},
					InlineRouteConfig: inlineRouteConfig,
				},
			},
		},
		// In the case of two HTTP Connection Manager's being present, the
		// second HTTP Connection Manager should be validated, but ignored.
		{
			name: "two hcms",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name: "filter-chain-1",
						Filters: []*v3listenerpb.Filter{
							{
								Name: "hcm",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{
											validServerSideHTTPFilter1,
											validServerSideHTTPFilter2,
											emptyRouterFilter,
										},
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
									}),
								},
							},
							{
								Name: "hcm2",
								ConfigType: &v3listenerpb.Filter_TypedConfig{
									TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
										HttpFilters: []*v3httppb.HttpFilter{
											validServerSideHTTPFilter1,
											emptyRouterFilter,
										},
										RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
											RouteConfig: routeConfig,
										},
									}),
								},
							},
						},
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: []*v3listenerpb.Filter{
						{
							Name: "hcm",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									HttpFilters: []*v3httppb.HttpFilter{
										validServerSideHTTPFilter1,
										validServerSideHTTPFilter2,
										emptyRouterFilter,
									},
									RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
										RouteConfig: routeConfig,
									},
								}),
							},
						},
						{
							Name: "hcm2",
							ConfigType: &v3listenerpb.Filter_TypedConfig{
								TypedConfig: testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
									HttpFilters: []*v3httppb.HttpFilter{
										validServerSideHTTPFilter1,
										emptyRouterFilter,
									},
									RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
										RouteConfig: routeConfig,
									},
								}),
							},
						},
					},
				},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									unspecifiedPrefixMapKey: {
										srcPortMap: map[int]*FilterChain{
											0: {HTTPFilters: []HTTPFilter{
												{
													Name:   "serverOnlyCustomFilter",
													Filter: serverOnlyHTTPFilter{},
													Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
												},
												{
													Name:   "serverOnlyCustomFilter2",
													Filter: serverOnlyHTTPFilter{},
													Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
												},
												makeRouterFilter(t),
											},
												InlineRouteConfig: inlineRouteConfig,
											},
										},
									},
								},
							},
						},
					},
				},
				def: &FilterChain{HTTPFilters: []HTTPFilter{
					{
						Name:   "serverOnlyCustomFilter",
						Filter: serverOnlyHTTPFilter{},
						Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
					},
					{
						Name:   "serverOnlyCustomFilter2",
						Filter: serverOnlyHTTPFilter{},
						Config: filterConfig{Cfg: serverOnlyCustomFilterConfig},
					},
					makeRouterFilter(t),
				},
					InlineRouteConfig: inlineRouteConfig,
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			gotFC, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: nil", err)
			}
			if !cmp.Equal(gotFC, test.wantFC, cmp.AllowUnexported(FilterChainManager{}, destPrefixEntry{}, sourcePrefixes{}, sourcePrefixEntry{}), cmpOpts) {
				t.Fatalf("NewFilterChainManager() returned %+v, want: %+v", gotFC, test.wantFC)
			}
		})
	}
}

// TestNewFilterChainImpl_Success_SecurityConfig verifies cases where the
// security configuration in the filter chain contains valid data.
func (s) TestNewFilterChainImpl_Success_SecurityConfig(t *testing.T) {
	tests := []struct {
		desc   string
		lis    *v3listenerpb.Listener
		wantFC *FilterChainManager
	}{
		{
			desc: "empty transport socket",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "filter-chain-1",
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Filters: emptyValidNetworkFilters(t),
				},
			},
			wantFC: &FilterChainManager{
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
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "no validation context",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
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
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
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
					Filters: emptyValidNetworkFilters(t),
				},
			},
			wantFC: &FilterChainManager{
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
		{
			desc: "validation context with certificate provider",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
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
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{
					Name: "default-filter-chain-1",
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
					Filters: emptyValidNetworkFilters(t),
				},
			},
			wantFC: &FilterChainManager{
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
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			gotFC, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: nil", err)
			}
			if !cmp.Equal(gotFC, test.wantFC, cmp.AllowUnexported(FilterChainManager{}, destPrefixEntry{}, sourcePrefixes{}, sourcePrefixEntry{}), cmpopts.EquateEmpty()) {
				t.Fatalf("NewFilterChainManager() returned %+v, want: %+v", gotFC, test.wantFC)
			}
		})
	}
}

// TestNewFilterChainImpl_Success_UnsupportedMatchFields verifies cases where
// there are multiple filter chains, and one of them is valid while the other
// contains unsupported match fields. These configurations should lead to
// success at config validation time and the filter chains which contains
// unsupported match fields will be skipped at lookup time.
func (s) TestNewFilterChainImpl_Success_UnsupportedMatchFields(t *testing.T) {
	unspecifiedEntry := &destPrefixEntry{
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
	}

	tests := []struct {
		desc   string
		lis    *v3listenerpb.Listener
		wantFC *FilterChainManager
	}{
		{
			desc: "unsupported destination port",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "good-chain",
						Filters: emptyValidNetworkFilters(t),
					},
					{
						Name: "unsupported-destination-port",
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:    []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							DestinationPort: &wrapperspb.UInt32Value{Value: 666},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: unspecifiedEntry,
				},
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "unsupported server names",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "good-chain",
						Filters: emptyValidNetworkFilters(t),
					},
					{
						Name: "unsupported-server-names",
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							ServerNames:  []string{"example-server"},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: unspecifiedEntry,
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
					},
				},
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "unsupported transport protocol",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "good-chain",
						Filters: emptyValidNetworkFilters(t),
					},
					{
						Name: "unsupported-transport-protocol",
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:      []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							TransportProtocol: "tls",
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: unspecifiedEntry,
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
					},
				},
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "unsupported application protocol",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						Name:    "good-chain",
						Filters: emptyValidNetworkFilters(t),
					},
					{
						Name: "unsupported-application-protocol",
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:         []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							ApplicationProtocols: []string{"h2"},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: unspecifiedEntry,
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
					},
				},
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			gotFC, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: nil", err)
			}
			if !cmp.Equal(gotFC, test.wantFC, cmp.AllowUnexported(FilterChainManager{}, destPrefixEntry{}, sourcePrefixes{}, sourcePrefixEntry{}), cmpopts.EquateEmpty()) {
				t.Fatalf("NewFilterChainManager() returned %+v, want: %+v", gotFC, test.wantFC)
			}
		})
	}
}

// TestNewFilterChainImpl_Success_AllCombinations verifies different
// combinations of the supported match criteria.
func (s) TestNewFilterChainImpl_Success_AllCombinations(t *testing.T) {
	tests := []struct {
		desc   string
		lis    *v3listenerpb.Listener
		wantFC *FilterChainManager
	}{
		{
			desc: "multiple destination prefixes",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						// Unspecified destination prefix.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						// v4 wildcard destination prefix.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("0.0.0.0", 0)},
							SourceType:   v3listenerpb.FilterChainMatch_EXTERNAL,
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						// v6 wildcard destination prefix.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("::", 0)},
							SourceType:   v3listenerpb.FilterChainMatch_EXTERNAL,
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)}},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.0.0.0", 8)}},
						Filters:          emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
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
					"0.0.0.0/0": {
						net: ipNetFromCIDR("0.0.0.0/0"),
						srcTypeArr: [3]*sourcePrefixes{
							nil,
							nil,
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
					"::/0": {
						net: ipNetFromCIDR("::/0"),
						srcTypeArr: [3]*sourcePrefixes{
							nil,
							nil,
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
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
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
					"10.0.0.0/8": {
						net: ipNetFromCIDR("10.0.0.0/8"),
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
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "multiple source types",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourceType: v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							SourceType:   v3listenerpb.FilterChainMatch_EXTERNAL,
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							nil,
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
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
						srcTypeArr: [3]*sourcePrefixes{
							nil,
							nil,
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
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "multiple source prefixes",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.0.0.0", 8)}},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:       []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									"10.0.0.0/8": {
										net: ipNetFromCIDR("10.0.0.0/8"),
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
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									"192.168.0.0/16": {
										net: ipNetFromCIDR("192.168.0.0/16"),
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
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "multiple source ports",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{1, 2, 3}},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:       []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							SourceType:         v3listenerpb.FilterChainMatch_EXTERNAL,
							SourcePorts:        []uint32{1, 2, 3},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
				dstPrefixMap: map[string]*destPrefixEntry{
					unspecifiedPrefixMapKey: {
						srcTypeArr: [3]*sourcePrefixes{
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									unspecifiedPrefixMapKey: {
										srcPortMap: map[int]*FilterChain{
											1: {
												InlineRouteConfig: inlineRouteConfig,
												HTTPFilters:       makeRouterFilterList(t),
											},
											2: {
												InlineRouteConfig: inlineRouteConfig,
												HTTPFilters:       makeRouterFilterList(t),
											},
											3: {
												InlineRouteConfig: inlineRouteConfig,
												HTTPFilters:       makeRouterFilterList(t),
											},
										},
									},
								},
							},
						},
					},
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
						srcTypeArr: [3]*sourcePrefixes{
							nil,
							nil,
							{
								srcPrefixMap: map[string]*sourcePrefixEntry{
									"192.168.0.0/16": {
										net: ipNetFromCIDR("192.168.0.0/16"),
										srcPortMap: map[int]*FilterChain{
											1: {
												InlineRouteConfig: inlineRouteConfig,
												HTTPFilters:       makeRouterFilterList(t),
											},
											2: {
												InlineRouteConfig: inlineRouteConfig,
												HTTPFilters:       makeRouterFilterList(t),
											},
											3: {
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
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
		{
			desc: "some chains have unsupported fields",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)}},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:      []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.0.0.0", 8)},
							TransportProtocol: "raw_buffer",
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						// This chain will be dropped in favor of the above
						// filter chain because they both have the same
						// destination prefix, but this one has an empty
						// transport protocol while the above chain has the more
						// preferred "raw_buffer".
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:       []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.0.0.0", 8)},
							TransportProtocol:  "",
							SourceType:         v3listenerpb.FilterChainMatch_EXTERNAL,
							SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("10.0.0.0", 16)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						// This chain will be dropped for unsupported server
						// names.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.100.1", 32)},
							ServerNames:  []string{"foo", "bar"},
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						// This chain will be dropped for unsupported transport
						// protocol.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:      []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.100.2", 32)},
							TransportProtocol: "not-raw-buffer",
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						// This chain will be dropped for unsupported
						// application protocol.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges:         []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.100.3", 32)},
							ApplicationProtocols: []string{"h2"},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
				DefaultFilterChain: &v3listenerpb.FilterChain{Filters: emptyValidNetworkFilters(t)},
			},
			wantFC: &FilterChainManager{
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
					"192.168.0.0/16": {
						net: ipNetFromCIDR("192.168.2.2/16"),
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
					"10.0.0.0/8": {
						net: ipNetFromCIDR("10.0.0.0/8"),
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
					"192.168.100.1/32": {
						net:        ipNetFromCIDR("192.168.100.1/32"),
						srcTypeArr: [3]*sourcePrefixes{},
					},
					"192.168.100.2/32": {
						net:        ipNetFromCIDR("192.168.100.2/32"),
						srcTypeArr: [3]*sourcePrefixes{},
					},
					"192.168.100.3/32": {
						net:        ipNetFromCIDR("192.168.100.3/32"),
						srcTypeArr: [3]*sourcePrefixes{},
					},
				},
				def: &FilterChain{
					InlineRouteConfig: inlineRouteConfig,
					HTTPFilters:       makeRouterFilterList(t),
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			gotFC, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() returned err: %v, wantErr: nil", err)
			}
			if !cmp.Equal(gotFC, test.wantFC, cmp.AllowUnexported(FilterChainManager{}, destPrefixEntry{}, sourcePrefixes{}, sourcePrefixEntry{})) {
				t.Fatalf("NewFilterChainManager() returned %+v, want: %+v", gotFC, test.wantFC)
			}
		})
	}
}

func (s) TestLookup_Failures(t *testing.T) {
	tests := []struct {
		desc    string
		lis     *v3listenerpb.Listener
		params  FilterChainLookupParams
		wantErr string
	}{
		{
			desc: "no destination prefix match",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)}},
						Filters:          emptyValidNetworkFilters(t),
					},
				},
			},
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(10, 1, 1, 1),
			},
			wantErr: "no matching filter chain based on destination prefix match",
		},
		{
			desc: "no source type match",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							SourceType:   v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 100, 1),
				SourceAddr:            net.IPv4(192, 168, 100, 2),
			},
			wantErr: "no matching filter chain based on source type match",
		},
		{
			desc: "no source prefix match",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 24)},
							SourceType:         v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 100, 1),
				SourceAddr:            net.IPv4(192, 168, 100, 1),
			},
			wantErr: "no matching filter chain after all match criteria",
		},
		{
			desc: "multiple matching filter chains",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{1, 2, 3}},
						Filters:          emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)},
							SourcePorts:  []uint32{1},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			params: FilterChainLookupParams{
				// IsUnspecified is not set. This means that the destination
				// prefix matchers will be ignored.
				DestAddr:   net.IPv4(192, 168, 100, 1),
				SourceAddr: net.IPv4(192, 168, 100, 1),
				SourcePort: 1,
			},
			wantErr: "multiple matching filter chains",
		},
		{
			desc: "no default filter chain",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{SourcePorts: []uint32{1, 2, 3}},
						Filters:          emptyValidNetworkFilters(t),
					},
				},
			},
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 100, 1),
				SourceAddr:            net.IPv4(192, 168, 100, 1),
				SourcePort:            80,
			},
			wantErr: "no matching filter chain after all match criteria",
		},
		{
			desc: "most specific match dropped for unsupported field",
			lis: &v3listenerpb.Listener{
				FilterChains: []*v3listenerpb.FilterChain{
					{
						// This chain will be picked in the destination prefix
						// stage, but will be dropped at the server names stage.
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.100.1", 32)},
							ServerNames:  []string{"foo"},
						},
						Filters: emptyValidNetworkFilters(t),
					},
					{
						FilterChainMatch: &v3listenerpb.FilterChainMatch{
							PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.100.0", 16)},
						},
						Filters: emptyValidNetworkFilters(t),
					},
				},
			},
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 100, 1),
				SourceAddr:            net.IPv4(192, 168, 100, 1),
				SourcePort:            80,
			},
			wantErr: "no matching filter chain based on source type match",
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			fci, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() failed: %v", err)
			}
			fc, err := fci.Lookup(test.params)
			if err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("FilterChainManager.Lookup(%v) = (%v, %v) want (nil, %s)", test.params, fc, err, test.wantErr)
			}
		})
	}
}

func (s) TestLookup_Successes(t *testing.T) {
	lisWithDefaultChain := &v3listenerpb.Listener{
		FilterChains: []*v3listenerpb.FilterChain{
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)}},
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{InstanceName: "instance1"},
							},
						}),
					},
				},
				Filters: emptyValidNetworkFilters(t),
			},
		},
		// A default filter chain with an empty transport socket.
		DefaultFilterChain: &v3listenerpb.FilterChain{
			TransportSocket: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{InstanceName: "default"},
						},
					}),
				},
			},
			Filters: emptyValidNetworkFilters(t),
		},
	}
	lisWithoutDefaultChain := &v3listenerpb.Listener{
		FilterChains: []*v3listenerpb.FilterChain{
			{
				TransportSocket: transportSocketWithInstanceName(t, "unspecified-dest-and-source-prefix"),
				Filters:         emptyValidNetworkFilters(t),
			},
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges:       []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("0.0.0.0", 0)},
					SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("0.0.0.0", 0)},
				},
				TransportSocket: transportSocketWithInstanceName(t, "wildcard-prefixes-v4"),
				Filters:         emptyValidNetworkFilters(t),
			},
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("::", 0)},
				},
				TransportSocket: transportSocketWithInstanceName(t, "wildcard-source-prefix-v6"),
				Filters:         emptyValidNetworkFilters(t),
			},
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 16)}},
				TransportSocket:  transportSocketWithInstanceName(t, "specific-destination-prefix-unspecified-source-type"),
				Filters:          emptyValidNetworkFilters(t),
			},
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 24)},
					SourceType:   v3listenerpb.FilterChainMatch_EXTERNAL,
				},
				TransportSocket: transportSocketWithInstanceName(t, "specific-destination-prefix-specific-source-type"),
				Filters:         emptyValidNetworkFilters(t),
			},
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges:       []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 24)},
					SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.92.1", 24)},
					SourceType:         v3listenerpb.FilterChainMatch_EXTERNAL,
				},
				TransportSocket: transportSocketWithInstanceName(t, "specific-destination-prefix-specific-source-type-specific-source-prefix"),
				Filters:         emptyValidNetworkFilters(t),
			},
			{
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges:       []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.1.1", 24)},
					SourcePrefixRanges: []*v3corepb.CidrRange{cidrRangeFromAddressAndPrefixLen("192.168.92.1", 24)},
					SourceType:         v3listenerpb.FilterChainMatch_EXTERNAL,
					SourcePorts:        []uint32{80},
				},
				TransportSocket: transportSocketWithInstanceName(t, "specific-destination-prefix-specific-source-type-specific-source-prefix-specific-source-port"),
				Filters:         emptyValidNetworkFilters(t),
			},
		},
	}

	tests := []struct {
		desc   string
		lis    *v3listenerpb.Listener
		params FilterChainLookupParams
		wantFC *FilterChain
	}{
		{
			desc: "default filter chain",
			lis:  lisWithDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(10, 1, 1, 1),
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "default"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "unspecified destination match",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.ParseIP("2001:68::db8"),
				SourceAddr:            net.IPv4(10, 1, 1, 1),
				SourcePort:            1,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "unspecified-dest-and-source-prefix"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "wildcard destination match v4",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(10, 1, 1, 1),
				SourceAddr:            net.IPv4(10, 1, 1, 1),
				SourcePort:            1,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "wildcard-prefixes-v4"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "wildcard source match v6",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.ParseIP("2001:68::1"),
				SourceAddr:            net.ParseIP("2001:68::2"),
				SourcePort:            1,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "wildcard-source-prefix-v6"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "specific destination and wildcard source type match",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 100, 1),
				SourceAddr:            net.IPv4(192, 168, 100, 1),
				SourcePort:            80,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "specific-destination-prefix-unspecified-source-type"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "specific destination and source type match",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 1, 1),
				SourceAddr:            net.IPv4(10, 1, 1, 1),
				SourcePort:            80,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "specific-destination-prefix-specific-source-type"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "specific destination source type and source prefix",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 1, 1),
				SourceAddr:            net.IPv4(192, 168, 92, 100),
				SourcePort:            70,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "specific-destination-prefix-specific-source-type-specific-source-prefix"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
		{
			desc: "specific destination source type source prefix and source port",
			lis:  lisWithoutDefaultChain,
			params: FilterChainLookupParams{
				IsUnspecifiedListener: true,
				DestAddr:              net.IPv4(192, 168, 1, 1),
				SourceAddr:            net.IPv4(192, 168, 92, 100),
				SourcePort:            80,
			},
			wantFC: &FilterChain{
				SecurityCfg:       &SecurityConfig{IdentityInstanceName: "specific-destination-prefix-specific-source-type-specific-source-prefix-specific-source-port"},
				InlineRouteConfig: inlineRouteConfig,
				HTTPFilters:       makeRouterFilterList(t),
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			fci, err := NewFilterChainManager(test.lis)
			if err != nil {
				t.Fatalf("NewFilterChainManager() failed: %v", err)
			}
			gotFC, err := fci.Lookup(test.params)
			if err != nil {
				t.Fatalf("FilterChainManager.Lookup(%v) failed: %v", test.params, err)
			}
			if !cmp.Equal(gotFC, test.wantFC, cmpopts.EquateEmpty()) {
				t.Fatalf("FilterChainManager.Lookup(%v) = %v, want %v", test.params, gotFC, test.wantFC)
			}
		})
	}
}

type filterCfg struct {
	httpfilter.FilterConfig
	// Level is what differentiates top level filters ("top level") vs. second
	// level ("virtual host level"), and third level ("route level").
	level string
}

type filterBuilder struct {
	httpfilter.Filter
}

var _ httpfilter.ServerInterceptorBuilder = &filterBuilder{}

func (fb *filterBuilder) BuildServerInterceptor(config httpfilter.FilterConfig, override httpfilter.FilterConfig) (iresolver.ServerInterceptor, error) {
	var level string
	level = config.(filterCfg).level

	if override != nil {
		level = override.(filterCfg).level
	}
	return &serverInterceptor{level: level}, nil
}

type serverInterceptor struct {
	level string
}

func (si *serverInterceptor) AllowRPC(context.Context) error {
	return errors.New(si.level)
}

func (s) TestHTTPFilterInstantiation(t *testing.T) {
	tests := []struct {
		name        string
		filters     []HTTPFilter
		routeConfig RouteConfigUpdate
		// A list of strings which will be built from iterating through the
		// filters ["top level", "vh level", "route level", "route level"...]
		// wantErrs is the list of error strings that will be constructed from
		// the deterministic iteration through the vh list and route list. The
		// error string will be determined by the level of config that the
		// filter builder receives (i.e. top level, vs. virtual host level vs.
		// route level).
		wantErrs []string
	}{
		{
			name: "one http filter no overrides",
			filters: []HTTPFilter{
				{Name: "server-interceptor", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
			},
			routeConfig: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{"target"},
						Routes: []*Route{{
							Prefix: newStringP("1"),
						},
						},
					},
				}},
			wantErrs: []string{topLevel},
		},
		{
			name: "one http filter vh override",
			filters: []HTTPFilter{
				{Name: "server-interceptor", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
			},
			routeConfig: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{"target"},
						Routes: []*Route{{
							Prefix: newStringP("1"),
						},
						},
						HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
							"server-interceptor": filterCfg{level: vhLevel},
						},
					},
				}},
			wantErrs: []string{vhLevel},
		},
		{
			name: "one http filter route override",
			filters: []HTTPFilter{
				{Name: "server-interceptor", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
			},
			routeConfig: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{"target"},
						Routes: []*Route{{
							Prefix: newStringP("1"),
							HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
								"server-interceptor": filterCfg{level: rLevel},
							},
						},
						},
					},
				}},
			wantErrs: []string{rLevel},
		},
		// This tests the scenario where there are three http filters, and one
		// gets overridden by route and one by virtual host.
		{
			name: "three http filters vh override route override",
			filters: []HTTPFilter{
				{Name: "server-interceptor1", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
				{Name: "server-interceptor2", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
				{Name: "server-interceptor3", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
			},
			routeConfig: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{"target"},
						Routes: []*Route{{
							Prefix: newStringP("1"),
							HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
								"server-interceptor3": filterCfg{level: rLevel},
							},
						},
						},
						HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
							"server-interceptor2": filterCfg{level: vhLevel},
						},
					},
				}},
			wantErrs: []string{topLevel, vhLevel, rLevel},
		},
		// This tests the scenario where there are three http filters, and two
		// virtual hosts with different vh + route overrides for each virtual
		// host.
		{
			name: "three http filters two vh",
			filters: []HTTPFilter{
				{Name: "server-interceptor1", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
				{Name: "server-interceptor2", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
				{Name: "server-interceptor3", Filter: &filterBuilder{}, Config: filterCfg{level: topLevel}},
			},
			routeConfig: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{"target"},
						Routes: []*Route{{
							Prefix: newStringP("1"),
							HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
								"server-interceptor3": filterCfg{level: rLevel},
							},
						},
						},
						HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
							"server-interceptor2": filterCfg{level: vhLevel},
						},
					},
					{
						Domains: []string{"target"},
						Routes: []*Route{{
							Prefix: newStringP("1"),
							HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
								"server-interceptor1": filterCfg{level: rLevel},
								"server-interceptor2": filterCfg{level: rLevel},
							},
						},
						},
						HTTPFilterConfigOverride: map[string]httpfilter.FilterConfig{
							"server-interceptor2": filterCfg{level: vhLevel},
							"server-interceptor3": filterCfg{level: vhLevel},
						},
					},
				}},
			wantErrs: []string{topLevel, vhLevel, rLevel, rLevel, rLevel, vhLevel},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			fc := FilterChain{
				HTTPFilters: test.filters,
			}
			urc := fc.ConstructUsableRouteConfiguration(test.routeConfig)
			if urc.Err != nil {
				t.Fatalf("Error constructing usable route configuration: %v", urc.Err)
			}
			// Build out list of errors by iterating through the virtual hosts and routes,
			// and running the filters in route configurations.
			var errs []string
			for _, vh := range urc.VHS {
				for _, r := range vh.Routes {
					for _, int := range r.Interceptors {
						errs = append(errs, int.AllowRPC(context.Background()).Error())
					}
				}
			}
			if !cmp.Equal(errs, test.wantErrs) {
				t.Fatalf("List of errors %v, want %v", errs, test.wantErrs)
			}
		})
	}
}

// The Equal() methods defined below help with using cmp.Equal() on these types
// which contain all unexported fields.

func (fcm *FilterChainManager) Equal(other *FilterChainManager) bool {
	if (fcm == nil) != (other == nil) {
		return false
	}
	if fcm == nil {
		return true
	}
	switch {
	case !cmp.Equal(fcm.dstPrefixMap, other.dstPrefixMap, cmpopts.EquateEmpty()):
		return false
	// TODO: Support comparing dstPrefixes slice?
	case !cmp.Equal(fcm.def, other.def, cmpopts.EquateEmpty(), protocmp.Transform()):
		return false
	case !cmp.Equal(fcm.RouteConfigNames, other.RouteConfigNames, cmpopts.EquateEmpty()):
		return false
	}
	return true
}

func (fc *FilterChain) Equal(other *FilterChain) bool {
	if (fc == nil) != (other == nil) {
		return false
	}
	if fc == nil {
		return true
	}
	if !cmp.Equal(fc.SecurityCfg, other.SecurityCfg, cmpopts.EquateEmpty()) {
		return false
	}
	if !cmp.Equal(fc.RouteConfigName, other.RouteConfigName) {
		return false
	}
	if !cmp.Equal(fc.HTTPFilters, other.HTTPFilters, cmpopts.EquateEmpty(), protocmp.Transform()) {
		return false
	}
	return cmp.Equal(fc.InlineRouteConfig, other.InlineRouteConfig, cmpopts.EquateEmpty())
}

func (dpe *destPrefixEntry) Equal(other *destPrefixEntry) bool {
	if (dpe == nil) != (other == nil) {
		return false
	}
	if dpe == nil {
		return true
	}
	if !cmp.Equal(dpe.net, other.net) {
		return false
	}
	for i, st := range dpe.srcTypeArr {
		if !cmp.Equal(st, other.srcTypeArr[i], cmpopts.EquateEmpty()) {
			return false
		}
	}
	return true
}

func (sp *sourcePrefixes) Equal(other *sourcePrefixes) bool {
	if (sp == nil) != (other == nil) {
		return false
	}
	if sp == nil {
		return true
	}
	// TODO: Support comparing srcPrefixes slice?
	return cmp.Equal(sp.srcPrefixMap, other.srcPrefixMap, cmpopts.EquateEmpty())
}

func (spe *sourcePrefixEntry) Equal(other *sourcePrefixEntry) bool {
	if (spe == nil) != (other == nil) {
		return false
	}
	if spe == nil {
		return true
	}
	switch {
	case !cmp.Equal(spe.net, other.net):
		return false
	case !cmp.Equal(spe.srcPortMap, other.srcPortMap, cmpopts.EquateEmpty(), protocmp.Transform()):
		return false
	}
	return true
}

// The String() methods defined below help with debugging test failures as the
// regular %v or %+v formatting directives do not expands pointer fields inside
// structs, and these types have a lot of pointers pointing to other structs.
func (fcm *FilterChainManager) String() string {
	if fcm == nil {
		return ""
	}

	var sb strings.Builder
	if fcm.dstPrefixMap != nil {
		sb.WriteString("destination_prefix_map: map {\n")
		for k, v := range fcm.dstPrefixMap {
			sb.WriteString(fmt.Sprintf("%q: %v\n", k, v))
		}
		sb.WriteString("}\n")
	}
	if fcm.dstPrefixes != nil {
		sb.WriteString("destination_prefixes: [")
		for _, p := range fcm.dstPrefixes {
			sb.WriteString(fmt.Sprintf("%v ", p))
		}
		sb.WriteString("]")
	}
	if fcm.def != nil {
		sb.WriteString(fmt.Sprintf("default_filter_chain: %+v ", fcm.def))
	}
	return sb.String()
}

func (dpe *destPrefixEntry) String() string {
	if dpe == nil {
		return ""
	}
	var sb strings.Builder
	if dpe.net != nil {
		sb.WriteString(fmt.Sprintf("destination_prefix: %s ", dpe.net.String()))
	}
	sb.WriteString("source_types_array: [")
	for _, st := range dpe.srcTypeArr {
		sb.WriteString(fmt.Sprintf("%v ", st))
	}
	sb.WriteString("]")
	return sb.String()
}

func (sp *sourcePrefixes) String() string {
	if sp == nil {
		return ""
	}
	var sb strings.Builder
	if sp.srcPrefixMap != nil {
		sb.WriteString("source_prefix_map: map {")
		for k, v := range sp.srcPrefixMap {
			sb.WriteString(fmt.Sprintf("%q: %v ", k, v))
		}
		sb.WriteString("}")
	}
	if sp.srcPrefixes != nil {
		sb.WriteString("source_prefixes: [")
		for _, p := range sp.srcPrefixes {
			sb.WriteString(fmt.Sprintf("%v ", p))
		}
		sb.WriteString("]")
	}
	return sb.String()
}

func (spe *sourcePrefixEntry) String() string {
	if spe == nil {
		return ""
	}
	var sb strings.Builder
	if spe.net != nil {
		sb.WriteString(fmt.Sprintf("source_prefix: %s ", spe.net.String()))
	}
	if spe.srcPortMap != nil {
		sb.WriteString("source_ports_map: map {")
		for k, v := range spe.srcPortMap {
			sb.WriteString(fmt.Sprintf("%d: %+v ", k, v))
		}
		sb.WriteString("}")
	}
	return sb.String()
}

func (fc *FilterChain) String() string {
	if fc == nil || fc.SecurityCfg == nil {
		return ""
	}
	return fmt.Sprintf("security_config: %v", fc.SecurityCfg)
}

func ipNetFromCIDR(cidr string) *net.IPNet {
	_, ipnet, err := net.ParseCIDR(cidr)
	if err != nil {
		panic(err)
	}
	return ipnet
}

func transportSocketWithInstanceName(t *testing.T, name string) *v3corepb.TransportSocket {
	return &v3corepb.TransportSocket{
		Name: "envoy.transport_sockets.tls",
		ConfigType: &v3corepb.TransportSocket_TypedConfig{
			TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
				CommonTlsContext: &v3tlspb.CommonTlsContext{
					TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{InstanceName: name},
				},
			}),
		},
	}
}

func cidrRangeFromAddressAndPrefixLen(address string, len int) *v3corepb.CidrRange {
	return &v3corepb.CidrRange{
		AddressPrefix: address,
		PrefixLen: &wrapperspb.UInt32Value{
			Value: uint32(len),
		},
	}
}
