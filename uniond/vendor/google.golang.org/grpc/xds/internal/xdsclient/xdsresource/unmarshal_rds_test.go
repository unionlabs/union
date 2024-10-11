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
	"errors"
	"fmt"
	"math"
	"regexp"
	"testing"
	"time"

	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/xds/matcher"
	"google.golang.org/grpc/xds/internal/clusterspecifier"
	"google.golang.org/grpc/xds/internal/httpfilter"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	rpb "github.com/envoyproxy/go-control-plane/envoy/config/rbac/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3rbacpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/rbac/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"
	v3typepb "github.com/envoyproxy/go-control-plane/envoy/type/v3"
)

func (s) TestRDSGenerateRDSUpdateFromRouteConfiguration(t *testing.T) {
	const (
		uninterestingDomain      = "uninteresting.domain"
		uninterestingClusterName = "uninterestingClusterName"
		ldsTarget                = "lds.target.good:1111"
		routeName                = "routeName"
		clusterName              = "clusterName"
	)

	var (
		goodRouteConfigWithFilterConfigs = func(cfgs map[string]*anypb.Any) *v3routepb.RouteConfiguration {
			return &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{{
					Domains: []string{ldsTarget},
					Routes: []*v3routepb.Route{{
						Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName}},
						},
					}},
					TypedPerFilterConfig: cfgs,
				}},
			}
		}
		goodRouteConfigWithClusterSpecifierPlugins = func(csps []*v3routepb.ClusterSpecifierPlugin, cspReferences []string) *v3routepb.RouteConfiguration {
			var rs []*v3routepb.Route

			for i, cspReference := range cspReferences {
				rs = append(rs, &v3routepb.Route{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: fmt.Sprint(i + 1)}},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_ClusterSpecifierPlugin{ClusterSpecifierPlugin: cspReference},
						},
					},
				})
			}

			rc := &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{{
					Domains: []string{ldsTarget},
					Routes:  rs,
				}},
				ClusterSpecifierPlugins: csps,
			}

			return rc
		}
		goodRouteConfigWithClusterSpecifierPluginsAndNormalRoute = func(csps []*v3routepb.ClusterSpecifierPlugin, cspReferences []string) *v3routepb.RouteConfiguration {
			rs := goodRouteConfigWithClusterSpecifierPlugins(csps, cspReferences)
			rs.VirtualHosts[0].Routes = append(rs.VirtualHosts[0].Routes, &v3routepb.Route{
				Match: &v3routepb.RouteMatch{
					PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
					CaseSensitive: &wrapperspb.BoolValue{Value: false},
				},
				Action: &v3routepb.Route_Route{
					Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
					}}})
			return rs
		}
		goodRouteConfigWithUnsupportedClusterSpecifier = &v3routepb.RouteConfiguration{
			Name: routeName,
			VirtualHosts: []*v3routepb.VirtualHost{{
				Domains: []string{ldsTarget},
				Routes: []*v3routepb.Route{
					{
						Match: &v3routepb.RouteMatch{
							PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
							CaseSensitive: &wrapperspb.BoolValue{Value: false},
						},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName}},
						}},
					{
						Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "|"}},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{ClusterSpecifier: &v3routepb.RouteAction_ClusterHeader{}},
						}},
				},
			},
			},
		}

		goodUpdateWithFilterConfigs = func(cfgs map[string]httpfilter.FilterConfig) RouteConfigUpdate {
			return RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{{
					Domains: []string{ldsTarget},
					Routes: []*Route{{
						Prefix:           newStringP("/"),
						WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}},
						ActionType:       RouteActionRoute,
					}},
					HTTPFilterConfigOverride: cfgs,
				}},
			}
		}
		goodUpdateWithNormalRoute = RouteConfigUpdate{
			VirtualHosts: []*VirtualHost{
				{
					Domains: []string{ldsTarget},
					Routes: []*Route{{Prefix: newStringP("/"),
						CaseInsensitive:  true,
						WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}},
						ActionType:       RouteActionRoute}},
				},
			},
		}
		goodUpdateWithClusterSpecifierPluginA = RouteConfigUpdate{
			VirtualHosts: []*VirtualHost{{
				Domains: []string{ldsTarget},
				Routes: []*Route{{
					Prefix:                 newStringP("1"),
					ActionType:             RouteActionRoute,
					ClusterSpecifierPlugin: "cspA",
				}},
			}},
			ClusterSpecifierPlugins: map[string]clusterspecifier.BalancerConfig{
				"cspA": nil,
			},
		}
		clusterSpecifierPlugin = func(name string, config *anypb.Any, isOptional bool) *v3routepb.ClusterSpecifierPlugin {
			return &v3routepb.ClusterSpecifierPlugin{
				Extension: &v3corepb.TypedExtensionConfig{
					Name:        name,
					TypedConfig: config,
				},
				IsOptional: isOptional,
			}
		}
		goodRouteConfigWithRetryPolicy = func(vhrp *v3routepb.RetryPolicy, rrp *v3routepb.RetryPolicy) *v3routepb.RouteConfiguration {
			return &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{{
					Domains: []string{ldsTarget},
					Routes: []*v3routepb.Route{{
						Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
								RetryPolicy:      rrp,
							},
						},
					}},
					RetryPolicy: vhrp,
				}},
			}
		}
		goodUpdateWithRetryPolicy = func(vhrc *RetryConfig, rrc *RetryConfig) RouteConfigUpdate {
			return RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{{
					Domains: []string{ldsTarget},
					Routes: []*Route{{
						Prefix:           newStringP("/"),
						WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}},
						ActionType:       RouteActionRoute,
						RetryConfig:      rrc,
					}},
					RetryConfig: vhrc,
				}},
			}
		}
		defaultRetryBackoff = RetryBackoff{BaseInterval: 25 * time.Millisecond, MaxInterval: 250 * time.Millisecond}
	)

	tests := []struct {
		name       string
		rc         *v3routepb.RouteConfiguration
		wantUpdate RouteConfigUpdate
		wantError  bool
	}{
		{
			name: "default-route-match-field-is-nil",
			rc: &v3routepb.RouteConfiguration{
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
									},
								},
							},
						},
					},
				},
			},
			wantError: true,
		},
		{
			name: "default-route-match-field-is-non-nil",
			rc: &v3routepb.RouteConfiguration{
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match:  &v3routepb.RouteMatch{},
								Action: &v3routepb.Route_Route{},
							},
						},
					},
				},
			},
			wantError: true,
		},
		{
			name: "default-route-routeaction-field-is-nil",
			rc: &v3routepb.RouteConfiguration{
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes:  []*v3routepb.Route{{}},
					},
				},
			},
			wantError: true,
		},
		{
			name: "default-route-cluster-field-is-empty",
			rc: &v3routepb.RouteConfiguration{
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier: &v3routepb.RouteAction_ClusterHeader{},
									},
								},
							},
						},
					},
				},
			},
			wantError: true,
		},
		{
			// default route's match sets case-sensitive to false.
			name: "good-route-config-but-with-casesensitive-false",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{{
					Domains: []string{ldsTarget},
					Routes: []*v3routepb.Route{{
						Match: &v3routepb.RouteMatch{
							PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
							CaseSensitive: &wrapperspb.BoolValue{Value: false},
						},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
							}}}}}}},
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{Prefix: newStringP("/"),
							CaseInsensitive:  true,
							WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
				},
			},
		},
		{
			name: "good-route-config-with-empty-string-route",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{uninterestingDomain},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: ""}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: uninterestingClusterName},
									},
								},
							},
						},
					},
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: ""}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
									},
								},
							},
						},
					},
				},
			},
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{uninterestingDomain},
						Routes: []*Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]WeightedCluster{uninterestingClusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
				},
			},
		},
		{
			// default route's match is not empty string, but "/".
			name: "good-route-config-with-slash-string-route",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
									},
								},
							},
						},
					},
				},
			},
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{Prefix: newStringP("/"),
							WeightedClusters: map[string]WeightedCluster{clusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
				},
			},
		},
		{
			name: "good-route-config-with-weighted_clusters",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
											WeightedClusters: &v3routepb.WeightedCluster{
												Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
													{Name: "a", Weight: &wrapperspb.UInt32Value{Value: 2}},
													{Name: "b", Weight: &wrapperspb.UInt32Value{Value: 3}},
													{Name: "c", Weight: &wrapperspb.UInt32Value{Value: 5}},
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
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{
							Prefix: newStringP("/"),
							WeightedClusters: map[string]WeightedCluster{
								"a": {Weight: 2},
								"b": {Weight: 3},
								"c": {Weight: 5},
							},
							ActionType: RouteActionRoute,
						}},
					},
				},
			},
		},
		{
			name: "good-route-config-with-max-stream-duration",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier:  &v3routepb.RouteAction_Cluster{Cluster: clusterName},
										MaxStreamDuration: &v3routepb.RouteAction_MaxStreamDuration{MaxStreamDuration: durationpb.New(time.Second)},
									},
								},
							},
						},
					},
				},
			},
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{
							Prefix:            newStringP("/"),
							WeightedClusters:  map[string]WeightedCluster{clusterName: {Weight: 1}},
							MaxStreamDuration: newDurationP(time.Second),
							ActionType:        RouteActionRoute,
						}},
					},
				},
			},
		},
		{
			name: "good-route-config-with-grpc-timeout-header-max",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier:  &v3routepb.RouteAction_Cluster{Cluster: clusterName},
										MaxStreamDuration: &v3routepb.RouteAction_MaxStreamDuration{GrpcTimeoutHeaderMax: durationpb.New(time.Second)},
									},
								},
							},
						},
					},
				},
			},
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{
							Prefix:            newStringP("/"),
							WeightedClusters:  map[string]WeightedCluster{clusterName: {Weight: 1}},
							MaxStreamDuration: newDurationP(time.Second),
							ActionType:        RouteActionRoute,
						}},
					},
				},
			},
		},
		{
			name: "good-route-config-with-both-timeouts",
			rc: &v3routepb.RouteConfiguration{
				Name: routeName,
				VirtualHosts: []*v3routepb.VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*v3routepb.Route{
							{
								Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
								Action: &v3routepb.Route_Route{
									Route: &v3routepb.RouteAction{
										ClusterSpecifier:  &v3routepb.RouteAction_Cluster{Cluster: clusterName},
										MaxStreamDuration: &v3routepb.RouteAction_MaxStreamDuration{MaxStreamDuration: durationpb.New(2 * time.Second), GrpcTimeoutHeaderMax: durationpb.New(0)},
									},
								},
							},
						},
					},
				},
			},
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{
							Prefix:            newStringP("/"),
							WeightedClusters:  map[string]WeightedCluster{clusterName: {Weight: 1}},
							MaxStreamDuration: newDurationP(0),
							ActionType:        RouteActionRoute,
						}},
					},
				},
			},
		},
		{
			name:       "good-route-config-with-http-filter-config",
			rc:         goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": customFilterConfig}),
			wantUpdate: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterConfig}}),
		},
		{
			name:       "good-route-config-with-http-filter-config-in-old-typed-struct",
			rc:         goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": testutils.MarshalAny(t, customFilterOldTypedStructConfig)}),
			wantUpdate: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterOldTypedStructConfig}}),
		},
		{
			name:       "good-route-config-with-http-filter-config-in-new-typed-struct",
			rc:         goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": testutils.MarshalAny(t, customFilterNewTypedStructConfig)}),
			wantUpdate: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterNewTypedStructConfig}}),
		},
		{
			name:       "good-route-config-with-optional-http-filter-config",
			rc:         goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": wrappedOptionalFilter(t, "custom.filter")}),
			wantUpdate: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterConfig}}),
		},
		{
			name:      "good-route-config-with-http-err-filter-config",
			rc:        goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": errFilterConfig}),
			wantError: true,
		},
		{
			name:      "good-route-config-with-http-optional-err-filter-config",
			rc:        goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": wrappedOptionalFilter(t, "err.custom.filter")}),
			wantError: true,
		},
		{
			name:      "good-route-config-with-http-unknown-filter-config",
			rc:        goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": unknownFilterConfig}),
			wantError: true,
		},
		{
			name:       "good-route-config-with-http-optional-unknown-filter-config",
			rc:         goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"foo": wrappedOptionalFilter(t, "unknown.custom.filter")}),
			wantUpdate: goodUpdateWithFilterConfigs(nil),
		},
		{
			name: "good-route-config-with-bad-rbac-http-filter-configuration",
			rc: goodRouteConfigWithFilterConfigs(map[string]*anypb.Any{"rbac": testutils.MarshalAny(t, &v3rbacpb.RBACPerRoute{Rbac: &v3rbacpb.RBAC{
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
			}})}),
			wantError: true,
		},
		{
			name: "good-route-config-with-retry-policy",
			rc: goodRouteConfigWithRetryPolicy(
				&v3routepb.RetryPolicy{RetryOn: "cancelled"},
				&v3routepb.RetryPolicy{RetryOn: "deadline-exceeded,unsupported", NumRetries: &wrapperspb.UInt32Value{Value: 2}}),
			wantUpdate: goodUpdateWithRetryPolicy(
				&RetryConfig{RetryOn: map[codes.Code]bool{codes.Canceled: true}, NumRetries: 1, RetryBackoff: defaultRetryBackoff},
				&RetryConfig{RetryOn: map[codes.Code]bool{codes.DeadlineExceeded: true}, NumRetries: 2, RetryBackoff: defaultRetryBackoff}),
		},
		{
			name: "good-route-config-with-retry-backoff",
			rc: goodRouteConfigWithRetryPolicy(
				&v3routepb.RetryPolicy{RetryOn: "internal", RetryBackOff: &v3routepb.RetryPolicy_RetryBackOff{BaseInterval: durationpb.New(10 * time.Millisecond), MaxInterval: durationpb.New(10 * time.Millisecond)}},
				&v3routepb.RetryPolicy{RetryOn: "resource-exhausted", RetryBackOff: &v3routepb.RetryPolicy_RetryBackOff{BaseInterval: durationpb.New(10 * time.Millisecond)}}),
			wantUpdate: goodUpdateWithRetryPolicy(
				&RetryConfig{RetryOn: map[codes.Code]bool{codes.Internal: true}, NumRetries: 1, RetryBackoff: RetryBackoff{BaseInterval: 10 * time.Millisecond, MaxInterval: 10 * time.Millisecond}},
				&RetryConfig{RetryOn: map[codes.Code]bool{codes.ResourceExhausted: true}, NumRetries: 1, RetryBackoff: RetryBackoff{BaseInterval: 10 * time.Millisecond, MaxInterval: 100 * time.Millisecond}}),
		},
		{
			name:       "bad-retry-policy-0-retries",
			rc:         goodRouteConfigWithRetryPolicy(&v3routepb.RetryPolicy{RetryOn: "cancelled", NumRetries: &wrapperspb.UInt32Value{Value: 0}}, nil),
			wantUpdate: RouteConfigUpdate{},
			wantError:  true,
		},
		{
			name:       "bad-retry-policy-0-base-interval",
			rc:         goodRouteConfigWithRetryPolicy(&v3routepb.RetryPolicy{RetryOn: "cancelled", RetryBackOff: &v3routepb.RetryPolicy_RetryBackOff{BaseInterval: durationpb.New(0)}}, nil),
			wantUpdate: RouteConfigUpdate{},
			wantError:  true,
		},
		{
			name:       "bad-retry-policy-negative-max-interval",
			rc:         goodRouteConfigWithRetryPolicy(&v3routepb.RetryPolicy{RetryOn: "cancelled", RetryBackOff: &v3routepb.RetryPolicy_RetryBackOff{MaxInterval: durationpb.New(-time.Second)}}, nil),
			wantUpdate: RouteConfigUpdate{},
			wantError:  true,
		},
		{
			name:       "bad-retry-policy-negative-max-interval-no-known-retry-on",
			rc:         goodRouteConfigWithRetryPolicy(&v3routepb.RetryPolicy{RetryOn: "something", RetryBackOff: &v3routepb.RetryPolicy_RetryBackOff{MaxInterval: durationpb.New(-time.Second)}}, nil),
			wantUpdate: RouteConfigUpdate{},
			wantError:  true,
		},
		{
			name: "cluster-specifier-declared-which-not-registered",
			rc: goodRouteConfigWithClusterSpecifierPlugins([]*v3routepb.ClusterSpecifierPlugin{
				clusterSpecifierPlugin("cspA", configOfClusterSpecifierDoesntExist, false),
			}, []string{"cspA"}),
			wantError: true,
		},
		{
			name: "error-in-cluster-specifier-plugin-conversion-method",
			rc: goodRouteConfigWithClusterSpecifierPlugins([]*v3routepb.ClusterSpecifierPlugin{
				clusterSpecifierPlugin("cspA", errorClusterSpecifierConfig, false),
			}, []string{"cspA"}),
			wantError: true,
		},
		{
			name: "route-action-that-references-undeclared-cluster-specifier-plugin",
			rc: goodRouteConfigWithClusterSpecifierPlugins([]*v3routepb.ClusterSpecifierPlugin{
				clusterSpecifierPlugin("cspA", mockClusterSpecifierConfig, false),
			}, []string{"cspA", "cspB"}),
			wantError: true,
		},
		{
			name: "emitted-cluster-specifier-plugins",
			rc: goodRouteConfigWithClusterSpecifierPlugins([]*v3routepb.ClusterSpecifierPlugin{
				clusterSpecifierPlugin("cspA", mockClusterSpecifierConfig, false),
			}, []string{"cspA"}),
			wantUpdate: goodUpdateWithClusterSpecifierPluginA,
		},
		{
			name: "deleted-cluster-specifier-plugins-not-referenced",
			rc: goodRouteConfigWithClusterSpecifierPlugins([]*v3routepb.ClusterSpecifierPlugin{
				clusterSpecifierPlugin("cspA", mockClusterSpecifierConfig, false),
				clusterSpecifierPlugin("cspB", mockClusterSpecifierConfig, false),
			}, []string{"cspA"}),
			wantUpdate: goodUpdateWithClusterSpecifierPluginA,
		},
		// This tests a scenario where a cluster specifier plugin is not found
		// and is optional. Any routes referencing that not found optional
		// cluster specifier plugin should be ignored. The config has two
		// routes, and only one of them should be present in the update.
		{
			name: "cluster-specifier-plugin-not-found-and-optional-route-should-ignore",
			rc: goodRouteConfigWithClusterSpecifierPluginsAndNormalRoute([]*v3routepb.ClusterSpecifierPlugin{
				clusterSpecifierPlugin("cspA", configOfClusterSpecifierDoesntExist, true),
			}, []string{"cspA"}),
			wantUpdate: goodUpdateWithNormalRoute,
		},
		// This tests a scenario where a route has an unsupported cluster
		// specifier. Any routes with an unsupported cluster specifier should be
		// ignored. The config has two routes, and only one of them should be
		// present in the update.
		{
			name:       "unsupported-cluster-specifier-route-should-ignore",
			rc:         goodRouteConfigWithUnsupportedClusterSpecifier,
			wantUpdate: goodUpdateWithNormalRoute,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			gotUpdate, gotError := generateRDSUpdateFromRouteConfiguration(test.rc)
			if (gotError != nil) != test.wantError ||
				!cmp.Equal(gotUpdate, test.wantUpdate, cmpopts.EquateEmpty(),
					cmp.Transformer("FilterConfig", func(fc httpfilter.FilterConfig) string {
						return fmt.Sprint(fc)
					})) {
				t.Errorf("generateRDSUpdateFromRouteConfiguration(%+v, %v) returned unexpected, diff (-want +got):\\n%s", test.rc, ldsTarget, cmp.Diff(test.wantUpdate, gotUpdate, cmpopts.EquateEmpty()))
			}
		})
	}
}

var configOfClusterSpecifierDoesntExist = &anypb.Any{
	TypeUrl: "does.not.exist",
	Value:   []byte{1, 2, 3},
}

var mockClusterSpecifierConfig = &anypb.Any{
	TypeUrl: "mock.cluster.specifier.plugin",
	Value:   []byte{1, 2, 3},
}

var errorClusterSpecifierConfig = &anypb.Any{
	TypeUrl: "error.cluster.specifier.plugin",
	Value:   []byte{1, 2, 3},
}

func init() {
	clusterspecifier.Register(mockClusterSpecifierPlugin{})
	clusterspecifier.Register(errorClusterSpecifierPlugin{})
}

type mockClusterSpecifierPlugin struct {
}

func (mockClusterSpecifierPlugin) TypeURLs() []string {
	return []string{"mock.cluster.specifier.plugin"}
}

func (mockClusterSpecifierPlugin) ParseClusterSpecifierConfig(proto.Message) (clusterspecifier.BalancerConfig, error) {
	return []map[string]any{}, nil
}

type errorClusterSpecifierPlugin struct{}

func (errorClusterSpecifierPlugin) TypeURLs() []string {
	return []string{"error.cluster.specifier.plugin"}
}

func (errorClusterSpecifierPlugin) ParseClusterSpecifierConfig(proto.Message) (clusterspecifier.BalancerConfig, error) {
	return nil, errors.New("error from cluster specifier conversion function")
}

func (s) TestUnmarshalRouteConfig(t *testing.T) {
	const (
		ldsTarget                = "lds.target.good:1111"
		uninterestingDomain      = "uninteresting.domain"
		uninterestingClusterName = "uninterestingClusterName"
		v3RouteConfigName        = "v3RouteConfig"
		v3ClusterName            = "v3Cluster"
	)

	var (
		v3VirtualHost = []*v3routepb.VirtualHost{
			{
				Domains: []string{uninterestingDomain},
				Routes: []*v3routepb.Route{
					{
						Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: ""}},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: uninterestingClusterName},
							},
						},
					},
				},
			},
			{
				Domains: []string{ldsTarget},
				Routes: []*v3routepb.Route{
					{
						Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: ""}},
						Action: &v3routepb.Route_Route{
							Route: &v3routepb.RouteAction{
								ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: v3ClusterName},
							},
						},
					},
				},
			},
		}
		v3RouteConfig = testutils.MarshalAny(t, &v3routepb.RouteConfiguration{
			Name:         v3RouteConfigName,
			VirtualHosts: v3VirtualHost,
		})
	)

	tests := []struct {
		name       string
		resource   *anypb.Any
		wantName   string
		wantUpdate RouteConfigUpdate
		wantErr    bool
	}{
		{
			name:     "non-routeConfig resource type",
			resource: &anypb.Any{TypeUrl: version.V3HTTPConnManagerURL},
			wantErr:  true,
		},
		{
			name: "badly marshaled routeconfig resource",
			resource: &anypb.Any{
				TypeUrl: version.V3RouteConfigURL,
				Value:   []byte{1, 2, 3, 4},
			},
			wantErr: true,
		},
		{
			name:     "v3 routeConfig resource",
			resource: v3RouteConfig,
			wantName: v3RouteConfigName,
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{uninterestingDomain},
						Routes: []*Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]WeightedCluster{uninterestingClusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]WeightedCluster{v3ClusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
				},
				Raw: v3RouteConfig,
			},
		},
		{
			name:     "v3 routeConfig resource wrapped",
			resource: testutils.MarshalAny(t, &v3discoverypb.Resource{Resource: v3RouteConfig}),
			wantName: v3RouteConfigName,
			wantUpdate: RouteConfigUpdate{
				VirtualHosts: []*VirtualHost{
					{
						Domains: []string{uninterestingDomain},
						Routes: []*Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]WeightedCluster{uninterestingClusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
					{
						Domains: []string{ldsTarget},
						Routes: []*Route{{Prefix: newStringP(""),
							WeightedClusters: map[string]WeightedCluster{v3ClusterName: {Weight: 1}},
							ActionType:       RouteActionRoute}},
					},
				},
				Raw: v3RouteConfig,
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			name, update, err := unmarshalRouteConfigResource(test.resource)
			if (err != nil) != test.wantErr {
				t.Errorf("unmarshalRouteConfigResource(%s), got err: %v, wantErr: %v", pretty.ToJSON(test.resource), err, test.wantErr)
			}
			if name != test.wantName {
				t.Errorf("unmarshalRouteConfigResource(%s), got name: %s, want: %s", pretty.ToJSON(test.resource), name, test.wantName)
			}
			if diff := cmp.Diff(update, test.wantUpdate, cmpOpts); diff != "" {
				t.Errorf("unmarshalRouteConfigResource(%s), got unexpected update, diff (-got +want): %v", pretty.ToJSON(test.resource), diff)
			}
		})
	}
}

func (s) TestRoutesProtoToSlice(t *testing.T) {
	sm, _ := matcher.StringMatcherFromProto(&v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "tv"}})
	var (
		goodRouteWithFilterConfigs = func(cfgs map[string]*anypb.Any) []*v3routepb.Route {
			// Sets per-filter config in cluster "B" and in the route.
			return []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{
					PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
					CaseSensitive: &wrapperspb.BoolValue{Value: false},
				},
				Action: &v3routepb.Route_Route{
					Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
							WeightedClusters: &v3routepb.WeightedCluster{
								Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
									{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}, TypedPerFilterConfig: cfgs},
									{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
								},
							}}}},
				TypedPerFilterConfig: cfgs,
			}}
		}
		goodUpdateWithFilterConfigs = func(cfgs map[string]httpfilter.FilterConfig) []*Route {
			// Sets per-filter config in cluster "B" and in the route.
			return []*Route{{
				Prefix:                   newStringP("/"),
				CaseInsensitive:          true,
				WeightedClusters:         map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60, HTTPFilterConfigOverride: cfgs}},
				HTTPFilterConfigOverride: cfgs,
				ActionType:               RouteActionRoute,
			}}
		}
	)

	tests := []struct {
		name       string
		routes     []*v3routepb.Route
		wantRoutes []*Route
		wantErr    bool
	}{
		{
			name: "no path",
			routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{},
			}},
			wantErr: true,
		},
		{
			name: "case_sensitive is false",
			routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{
					PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
					CaseSensitive: &wrapperspb.BoolValue{Value: false},
				},
				Action: &v3routepb.Route_Route{
					Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
							WeightedClusters: &v3routepb.WeightedCluster{
								Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
									{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
									{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
								},
							}}}},
			}},
			wantRoutes: []*Route{{
				Prefix:           newStringP("/"),
				CaseInsensitive:  true,
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				ActionType:       RouteActionRoute,
			}},
		},
		{
			name: "good",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
						Headers: []*v3routepb.HeaderMatcher{
							{
								Name: "th",
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{
									PrefixMatch: "tv",
								},
								InvertMatch: true,
							},
						},
						RuntimeFraction: &v3corepb.RuntimeFractionalPercent{
							DefaultValue: &v3typepb.FractionalPercent{
								Numerator:   1,
								Denominator: v3typepb.FractionalPercent_HUNDRED,
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}}}},
				},
			},
			wantRoutes: []*Route{{
				Prefix: newStringP("/a/"),
				Headers: []*HeaderMatcher{
					{
						Name:        "th",
						InvertMatch: newBoolP(true),
						PrefixMatch: newStringP("tv"),
					},
				},
				Fraction:         newUInt32P(10000),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				ActionType:       RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name: "good with regex matchers",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: "/a/"}},
						Headers: []*v3routepb.HeaderMatcher{
							{
								Name:                 "th",
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SafeRegexMatch{SafeRegexMatch: &v3matcherpb.RegexMatcher{Regex: "tv"}},
							},
						},
						RuntimeFraction: &v3corepb.RuntimeFractionalPercent{
							DefaultValue: &v3typepb.FractionalPercent{
								Numerator:   1,
								Denominator: v3typepb.FractionalPercent_HUNDRED,
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}}}},
				},
			},
			wantRoutes: []*Route{{
				Regex: func() *regexp.Regexp { return regexp.MustCompile("/a/") }(),
				Headers: []*HeaderMatcher{
					{
						Name:        "th",
						InvertMatch: newBoolP(false),
						RegexMatch:  func() *regexp.Regexp { return regexp.MustCompile("tv") }(),
					},
				},
				Fraction:         newUInt32P(10000),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				ActionType:       RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name: "good with string matcher",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: "/a/"}},
						Headers: []*v3routepb.HeaderMatcher{
							{
								Name:                 "th",
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_StringMatch{StringMatch: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "tv"}}},
							},
						},
						RuntimeFraction: &v3corepb.RuntimeFractionalPercent{
							DefaultValue: &v3typepb.FractionalPercent{
								Numerator:   1,
								Denominator: v3typepb.FractionalPercent_HUNDRED,
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}}}},
				},
			},
			wantRoutes: []*Route{{
				Regex: func() *regexp.Regexp { return regexp.MustCompile("/a/") }(),
				Headers: []*HeaderMatcher{
					{
						Name:        "th",
						InvertMatch: newBoolP(false),
						StringMatch: &sm,
					},
				},
				Fraction:         newUInt32P(10000),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				ActionType:       RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name: "query is ignored",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}}}},
				},
				{
					Name: "with_query",
					Match: &v3routepb.RouteMatch{
						PathSpecifier:   &v3routepb.RouteMatch_Prefix{Prefix: "/b/"},
						QueryParameters: []*v3routepb.QueryParameterMatcher{{Name: "route_will_be_ignored"}},
					},
				},
			},
			// Only one route in the result, because the second one with query
			// parameters is ignored.
			wantRoutes: []*Route{{
				Prefix:           newStringP("/a/"),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				ActionType:       RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name: "unrecognized path specifier",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_ConnectMatcher_{},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "bad regex in path specifier",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: "??"}},
						Headers: []*v3routepb.HeaderMatcher{
							{
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{PrefixMatch: "tv"},
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName}},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "bad regex in header specifier",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
						Headers: []*v3routepb.HeaderMatcher{
							{
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SafeRegexMatch{SafeRegexMatch: &v3matcherpb.RegexMatcher{Regex: "??"}},
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName}},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "unrecognized header match specifier",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
						Headers: []*v3routepb.HeaderMatcher{
							{
								Name:                 "th",
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_StringMatch{},
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "no cluster in weighted clusters action",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{}}}},
				},
			},
			wantErr: true,
		},
		{
			name: "all 0-weight clusters in weighted clusters action",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 0}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 0}},
									},
								}}}},
				},
			},
			wantErr: true,
		},
		{
			name: "The sum of all weighted clusters is more than uint32",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: math.MaxUint32}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: math.MaxUint32}},
									},
								}}}},
				},
			},
			wantErr: true,
		},
		{
			name: "unsupported cluster specifier",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_ClusterSpecifierPlugin{}}},
				},
			},
			wantErr: true,
		},
		{
			name: "default totalWeight is 100 in weighted clusters action",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}}}},
				},
			},
			wantRoutes: []*Route{{
				Prefix:           newStringP("/a/"),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				ActionType:       RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name: "default totalWeight is 100 in weighted clusters action",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 30}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 20}},
									},
								}}}},
				},
			},
			wantRoutes: []*Route{{
				Prefix:           newStringP("/a/"),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 20}, "B": {Weight: 30}},
				ActionType:       RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name: "good-with-channel-id-hash-policy",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
						Headers: []*v3routepb.HeaderMatcher{
							{
								Name: "th",
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{
									PrefixMatch: "tv",
								},
								InvertMatch: true,
							},
						},
						RuntimeFraction: &v3corepb.RuntimeFractionalPercent{
							DefaultValue: &v3typepb.FractionalPercent{
								Numerator:   1,
								Denominator: v3typepb.FractionalPercent_HUNDRED,
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}},
							HashPolicy: []*v3routepb.RouteAction_HashPolicy{
								{PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{FilterState: &v3routepb.RouteAction_HashPolicy_FilterState{Key: "io.grpc.channel_id"}}},
							},
						}},
				},
			},
			wantRoutes: []*Route{{
				Prefix: newStringP("/a/"),
				Headers: []*HeaderMatcher{
					{
						Name:        "th",
						InvertMatch: newBoolP(true),
						PrefixMatch: newStringP("tv"),
					},
				},
				Fraction:         newUInt32P(10000),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				HashPolicies: []*HashPolicy{
					{HashPolicyType: HashPolicyTypeChannelID},
				},
				ActionType: RouteActionRoute,
			}},
			wantErr: false,
		},
		// This tests that policy.Regex ends up being nil if RegexRewrite is not
		// set in xds response.
		{
			name: "good-with-header-hash-policy-no-regex-specified",
			routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{
						PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/a/"},
						Headers: []*v3routepb.HeaderMatcher{
							{
								Name: "th",
								HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{
									PrefixMatch: "tv",
								},
								InvertMatch: true,
							},
						},
						RuntimeFraction: &v3corepb.RuntimeFractionalPercent{
							DefaultValue: &v3typepb.FractionalPercent{
								Numerator:   1,
								Denominator: v3typepb.FractionalPercent_HUNDRED,
							},
						},
					},
					Action: &v3routepb.Route_Route{
						Route: &v3routepb.RouteAction{
							ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{
								WeightedClusters: &v3routepb.WeightedCluster{
									Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
										{Name: "B", Weight: &wrapperspb.UInt32Value{Value: 60}},
										{Name: "A", Weight: &wrapperspb.UInt32Value{Value: 40}},
									},
								}},
							HashPolicy: []*v3routepb.RouteAction_HashPolicy{
								{PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{Header: &v3routepb.RouteAction_HashPolicy_Header{HeaderName: ":path"}}},
							},
						}},
				},
			},
			wantRoutes: []*Route{{
				Prefix: newStringP("/a/"),
				Headers: []*HeaderMatcher{
					{
						Name:        "th",
						InvertMatch: newBoolP(true),
						PrefixMatch: newStringP("tv"),
					},
				},
				Fraction:         newUInt32P(10000),
				WeightedClusters: map[string]WeightedCluster{"A": {Weight: 40}, "B": {Weight: 60}},
				HashPolicies: []*HashPolicy{
					{HashPolicyType: HashPolicyTypeHeader,
						HeaderName: ":path"},
				},
				ActionType: RouteActionRoute,
			}},
			wantErr: false,
		},
		{
			name:       "with custom HTTP filter config",
			routes:     goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": customFilterConfig}),
			wantRoutes: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterConfig}}),
		},
		{
			name:       "with custom HTTP filter config in typed struct",
			routes:     goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": testutils.MarshalAny(t, customFilterOldTypedStructConfig)}),
			wantRoutes: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterOldTypedStructConfig}}),
		},
		{
			name:       "with optional custom HTTP filter config",
			routes:     goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": wrappedOptionalFilter(t, "custom.filter")}),
			wantRoutes: goodUpdateWithFilterConfigs(map[string]httpfilter.FilterConfig{"foo": filterConfig{Override: customFilterConfig}}),
		},
		{
			name:    "with erroring custom HTTP filter config",
			routes:  goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": errFilterConfig}),
			wantErr: true,
		},
		{
			name:    "with optional erroring custom HTTP filter config",
			routes:  goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": wrappedOptionalFilter(t, "err.custom.filter")}),
			wantErr: true,
		},
		{
			name:    "with unknown custom HTTP filter config",
			routes:  goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": unknownFilterConfig}),
			wantErr: true,
		},
		{
			name:       "with optional unknown custom HTTP filter config",
			routes:     goodRouteWithFilterConfigs(map[string]*anypb.Any{"foo": wrappedOptionalFilter(t, "unknown.custom.filter")}),
			wantRoutes: goodUpdateWithFilterConfigs(nil),
		},
	}

	cmpOpts := []cmp.Option{
		cmp.AllowUnexported(Route{}, HeaderMatcher{}, Int64Range{}, regexp.Regexp{}),
		cmpopts.EquateEmpty(),
		cmp.Transformer("FilterConfig", func(fc httpfilter.FilterConfig) string {
			return fmt.Sprint(fc)
		}),
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, _, err := routesProtoToSlice(tt.routes, nil)
			if (err != nil) != tt.wantErr {
				t.Fatalf("routesProtoToSlice() error = %v, wantErr %v", err, tt.wantErr)
			}
			if diff := cmp.Diff(got, tt.wantRoutes, cmpOpts...); diff != "" {
				t.Fatalf("routesProtoToSlice() returned unexpected diff (-got +want):\n%s", diff)
			}
		})
	}
}

func (s) TestHashPoliciesProtoToSlice(t *testing.T) {
	tests := []struct {
		name             string
		hashPolicies     []*v3routepb.RouteAction_HashPolicy
		wantHashPolicies []*HashPolicy
		wantErr          bool
	}{
		// header-hash-policy tests a basic hash policy that specifies to hash a
		// certain header.
		{
			name: "header-hash-policy",
			hashPolicies: []*v3routepb.RouteAction_HashPolicy{
				{
					PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
						Header: &v3routepb.RouteAction_HashPolicy_Header{
							HeaderName: ":path",
							RegexRewrite: &v3matcherpb.RegexMatchAndSubstitute{
								Pattern:      &v3matcherpb.RegexMatcher{Regex: "/products"},
								Substitution: "/products",
							},
						},
					},
				},
			},
			wantHashPolicies: []*HashPolicy{
				{
					HashPolicyType:    HashPolicyTypeHeader,
					HeaderName:        ":path",
					Regex:             func() *regexp.Regexp { return regexp.MustCompile("/products") }(),
					RegexSubstitution: "/products",
				},
			},
		},
		// channel-id-hash-policy tests a basic hash policy that specifies to
		// hash a unique identifier of the channel.
		{
			name: "channel-id-hash-policy",
			hashPolicies: []*v3routepb.RouteAction_HashPolicy{
				{PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{FilterState: &v3routepb.RouteAction_HashPolicy_FilterState{Key: "io.grpc.channel_id"}}},
			},
			wantHashPolicies: []*HashPolicy{
				{HashPolicyType: HashPolicyTypeChannelID},
			},
		},
		// unsupported-filter-state-key tests that an unsupported key in the
		// filter state hash policy are treated as a no-op.
		{
			name: "wrong-filter-state-key",
			hashPolicies: []*v3routepb.RouteAction_HashPolicy{
				{PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{FilterState: &v3routepb.RouteAction_HashPolicy_FilterState{Key: "unsupported key"}}},
			},
		},
		// no-op-hash-policy tests that hash policies that are not supported by
		// grpc are treated as a no-op.
		{
			name: "no-op-hash-policy",
			hashPolicies: []*v3routepb.RouteAction_HashPolicy{
				{PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{}},
			},
		},
		// header-and-channel-id-hash-policy test that a list of header and
		// channel id hash policies are successfully converted to an internal
		// struct.
		{
			name: "header-and-channel-id-hash-policy",
			hashPolicies: []*v3routepb.RouteAction_HashPolicy{
				{
					PolicySpecifier: &v3routepb.RouteAction_HashPolicy_Header_{
						Header: &v3routepb.RouteAction_HashPolicy_Header{
							HeaderName: ":path",
							RegexRewrite: &v3matcherpb.RegexMatchAndSubstitute{
								Pattern:      &v3matcherpb.RegexMatcher{Regex: "/products"},
								Substitution: "/products",
							},
						},
					},
				},
				{
					PolicySpecifier: &v3routepb.RouteAction_HashPolicy_FilterState_{FilterState: &v3routepb.RouteAction_HashPolicy_FilterState{Key: "io.grpc.channel_id"}},
					Terminal:        true,
				},
			},
			wantHashPolicies: []*HashPolicy{
				{
					HashPolicyType:    HashPolicyTypeHeader,
					HeaderName:        ":path",
					Regex:             func() *regexp.Regexp { return regexp.MustCompile("/products") }(),
					RegexSubstitution: "/products",
				},
				{
					HashPolicyType: HashPolicyTypeChannelID,
					Terminal:       true,
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := hashPoliciesProtoToSlice(tt.hashPolicies)
			if (err != nil) != tt.wantErr {
				t.Fatalf("hashPoliciesProtoToSlice() error = %v, wantErr %v", err, tt.wantErr)
			}
			if diff := cmp.Diff(got, tt.wantHashPolicies, cmp.AllowUnexported(regexp.Regexp{})); diff != "" {
				t.Fatalf("hashPoliciesProtoToSlice() returned unexpected diff (-got +want):\n%s", diff)
			}
		})
	}
}

func newStringP(s string) *string {
	return &s
}

func newUInt32P(i uint32) *uint32 {
	return &i
}

func newBoolP(b bool) *bool {
	return &b
}

func newDurationP(d time.Duration) *time.Duration {
	return &d
}
