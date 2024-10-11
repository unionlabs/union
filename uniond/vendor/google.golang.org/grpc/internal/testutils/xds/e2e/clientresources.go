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

package e2e

import (
	"fmt"
	"net"
	"strconv"

	"github.com/envoyproxy/go-control-plane/pkg/wellknown"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3aggregateclusterpb "github.com/envoyproxy/go-control-plane/envoy/extensions/clusters/aggregate/v3"
	v3routerpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/router/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	v3typepb "github.com/envoyproxy/go-control-plane/envoy/type/v3"
)

const (
	// ServerListenerResourceNameTemplate is the Listener resource name template
	// used on the server side.
	ServerListenerResourceNameTemplate = "grpc/server?xds.resource.listening_address=%s"
	// ClientSideCertProviderInstance is the certificate provider instance name
	// used in the Cluster resource on the client side.
	ClientSideCertProviderInstance = "client-side-certificate-provider-instance"
	// ServerSideCertProviderInstance is the certificate provider instance name
	// used in the Listener resource on the server side.
	ServerSideCertProviderInstance = "server-side-certificate-provider-instance"
)

// SecurityLevel allows the test to control the security level to be used in the
// resource returned by this package.
type SecurityLevel int

const (
	// SecurityLevelNone is used when no security configuration is required.
	SecurityLevelNone SecurityLevel = iota
	// SecurityLevelTLS is used when security configuration corresponding to TLS
	// is required. Only the server presents an identity certificate in this
	// configuration.
	SecurityLevelTLS
	// SecurityLevelMTLS is used when security configuration corresponding to
	// mTLS is required. Both client and server present identity certificates in
	// this configuration.
	SecurityLevelMTLS
)

// ResourceParams wraps the arguments to be passed to DefaultClientResources.
type ResourceParams struct {
	// DialTarget is the client's dial target. This is used as the name of the
	// Listener resource.
	DialTarget string
	// NodeID is the id of the xdsClient to which this update is to be pushed.
	NodeID string
	// Host is the host of the default Endpoint resource.
	Host string
	// port is the port of the default Endpoint resource.
	Port uint32
	// SecLevel controls the security configuration in the Cluster resource.
	SecLevel SecurityLevel
}

// DefaultClientResources returns a set of resources (LDS, RDS, CDS, EDS) for a
// client to generically connect to one server.
func DefaultClientResources(params ResourceParams) UpdateOptions {
	routeConfigName := "route-" + params.DialTarget
	clusterName := "cluster-" + params.DialTarget
	endpointsName := "endpoints-" + params.DialTarget
	return UpdateOptions{
		NodeID:    params.NodeID,
		Listeners: []*v3listenerpb.Listener{DefaultClientListener(params.DialTarget, routeConfigName)},
		Routes:    []*v3routepb.RouteConfiguration{DefaultRouteConfig(routeConfigName, params.DialTarget, clusterName)},
		Clusters:  []*v3clusterpb.Cluster{DefaultCluster(clusterName, endpointsName, params.SecLevel)},
		Endpoints: []*v3endpointpb.ClusterLoadAssignment{DefaultEndpoint(endpointsName, params.Host, []uint32{params.Port})},
	}
}

// RouterHTTPFilter is the HTTP Filter configuration for the Router filter.
var RouterHTTPFilter = HTTPFilter("router", &v3routerpb.Router{})

// DefaultClientListener returns a basic xds Listener resource to be used on
// the client side.
func DefaultClientListener(target, routeName string) *v3listenerpb.Listener {
	hcm := marshalAny(&v3httppb.HttpConnectionManager{
		RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{Rds: &v3httppb.Rds{
			ConfigSource: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
			},
			RouteConfigName: routeName,
		}},
		HttpFilters: []*v3httppb.HttpFilter{HTTPFilter("router", &v3routerpb.Router{})}, // router fields are unused by grpc
	})
	return &v3listenerpb.Listener{
		Name:        target,
		ApiListener: &v3listenerpb.ApiListener{ApiListener: hcm},
		FilterChains: []*v3listenerpb.FilterChain{{
			Name: "filter-chain-name",
			Filters: []*v3listenerpb.Filter{{
				Name:       wellknown.HTTPConnectionManager,
				ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: hcm},
			}},
		}},
	}
}

func marshalAny(m proto.Message) *anypb.Any {
	a, err := anypb.New(m)
	if err != nil {
		panic(fmt.Sprintf("anypb.New(%+v) failed: %v", m, err))
	}
	return a
}

// filterChainWontMatch returns a filter chain that won't match if running the
// test locally.
func filterChainWontMatch(routeName string, addressPrefix string, srcPorts []uint32) *v3listenerpb.FilterChain {
	hcm := &v3httppb.HttpConnectionManager{
		RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
			Rds: &v3httppb.Rds{
				ConfigSource: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
				},
				RouteConfigName: routeName,
			},
		},
		HttpFilters: []*v3httppb.HttpFilter{RouterHTTPFilter},
	}
	return &v3listenerpb.FilterChain{
		Name: routeName + "-wont-match",
		FilterChainMatch: &v3listenerpb.FilterChainMatch{
			PrefixRanges: []*v3corepb.CidrRange{
				{
					AddressPrefix: addressPrefix,
					PrefixLen: &wrapperspb.UInt32Value{
						Value: uint32(0),
					},
				},
			},
			SourceType:  v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
			SourcePorts: srcPorts,
			SourcePrefixRanges: []*v3corepb.CidrRange{
				{
					AddressPrefix: addressPrefix,
					PrefixLen: &wrapperspb.UInt32Value{
						Value: uint32(0),
					},
				},
			},
		},
		Filters: []*v3listenerpb.Filter{
			{
				Name:       "filter-1",
				ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: marshalAny(hcm)},
			},
		},
	}
}

// ListenerResourceThreeRouteResources returns a listener resource that points
// to three route configurations. Only the filter chain that points to the first
// route config can be matched to.
func ListenerResourceThreeRouteResources(host string, port uint32, secLevel SecurityLevel, routeName string) *v3listenerpb.Listener {
	lis := defaultServerListenerCommon(host, port, secLevel, routeName, false)
	lis.FilterChains = append(lis.FilterChains, filterChainWontMatch("routeName2", "1.1.1.1", []uint32{1}))
	lis.FilterChains = append(lis.FilterChains, filterChainWontMatch("routeName3", "2.2.2.2", []uint32{2}))
	return lis
}

// ListenerResourceFallbackToDefault returns a listener resource that contains a
// filter chain that will never get chosen to process traffic and a default
// filter chain. The default filter chain points to routeName2.
func ListenerResourceFallbackToDefault(host string, port uint32, secLevel SecurityLevel) *v3listenerpb.Listener {
	lis := defaultServerListenerCommon(host, port, secLevel, "", false)
	lis.FilterChains = nil
	lis.FilterChains = append(lis.FilterChains, filterChainWontMatch("routeName", "1.1.1.1", []uint32{1}))
	lis.DefaultFilterChain = filterChainWontMatch("routeName2", "2.2.2.2", []uint32{2})
	return lis
}

// DefaultServerListener returns a basic xds Listener resource to be used on the
// server side. The returned Listener resource contains an inline route
// configuration with the name of routeName.
func DefaultServerListener(host string, port uint32, secLevel SecurityLevel, routeName string) *v3listenerpb.Listener {
	return defaultServerListenerCommon(host, port, secLevel, routeName, true)
}

func defaultServerListenerCommon(host string, port uint32, secLevel SecurityLevel, routeName string, inlineRouteConfig bool) *v3listenerpb.Listener {
	var hcm *v3httppb.HttpConnectionManager
	if inlineRouteConfig {
		hcm = &v3httppb.HttpConnectionManager{
			RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
				RouteConfig: &v3routepb.RouteConfiguration{
					Name: routeName,
					VirtualHosts: []*v3routepb.VirtualHost{{
						// This "*" string matches on any incoming authority. This is to ensure any
						// incoming RPC matches to Route_NonForwardingAction and will proceed as
						// normal.
						Domains: []string{"*"},
						Routes: []*v3routepb.Route{{
							Match: &v3routepb.RouteMatch{
								PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
							},
							Action: &v3routepb.Route_NonForwardingAction{},
						}}}}},
			},
			HttpFilters: []*v3httppb.HttpFilter{RouterHTTPFilter},
		}
	} else {
		hcm = &v3httppb.HttpConnectionManager{
			RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
				Rds: &v3httppb.Rds{
					ConfigSource: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
					},
					RouteConfigName: routeName,
				},
			},
			HttpFilters: []*v3httppb.HttpFilter{RouterHTTPFilter},
		}
	}

	var tlsContext *v3tlspb.DownstreamTlsContext
	switch secLevel {
	case SecurityLevelNone:
	case SecurityLevelTLS:
		tlsContext = &v3tlspb.DownstreamTlsContext{
			CommonTlsContext: &v3tlspb.CommonTlsContext{
				TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
					InstanceName: ServerSideCertProviderInstance,
				},
			},
		}
	case SecurityLevelMTLS:
		tlsContext = &v3tlspb.DownstreamTlsContext{
			RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
			CommonTlsContext: &v3tlspb.CommonTlsContext{
				TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
					InstanceName: ServerSideCertProviderInstance,
				},
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
					ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
						InstanceName: ServerSideCertProviderInstance,
					},
				},
			},
		}
	}

	var ts *v3corepb.TransportSocket
	if tlsContext != nil {
		ts = &v3corepb.TransportSocket{
			Name: "envoy.transport_sockets.tls",
			ConfigType: &v3corepb.TransportSocket_TypedConfig{
				TypedConfig: marshalAny(tlsContext),
			},
		}
	}
	return &v3listenerpb.Listener{
		Name: fmt.Sprintf(ServerListenerResourceNameTemplate, net.JoinHostPort(host, strconv.Itoa(int(port)))),
		Address: &v3corepb.Address{
			Address: &v3corepb.Address_SocketAddress{
				SocketAddress: &v3corepb.SocketAddress{
					Address: host,
					PortSpecifier: &v3corepb.SocketAddress_PortValue{
						PortValue: port,
					},
				},
			},
		},
		FilterChains: []*v3listenerpb.FilterChain{
			{
				Name: "v4-wildcard",
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "0.0.0.0",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
					SourceType: v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
					SourcePrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "0.0.0.0",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
				},
				Filters: []*v3listenerpb.Filter{
					{
						Name:       "filter-1",
						ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: marshalAny(hcm)},
					},
				},
				TransportSocket: ts,
			},
			{
				Name: "v6-wildcard",
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "::",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
					SourceType: v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
					SourcePrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "::",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
				},
				Filters: []*v3listenerpb.Filter{
					{
						Name:       "filter-1",
						ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: marshalAny(hcm)},
					},
				},
				TransportSocket: ts,
			},
		},
	}
}

// HTTPFilter constructs an xds HttpFilter with the provided name and config.
func HTTPFilter(name string, config proto.Message) *v3httppb.HttpFilter {
	return &v3httppb.HttpFilter{
		Name: name,
		ConfigType: &v3httppb.HttpFilter_TypedConfig{
			TypedConfig: marshalAny(config),
		},
	}
}

// DefaultRouteConfig returns a basic xds RouteConfig resource.
func DefaultRouteConfig(routeName, vhDomain, clusterName string) *v3routepb.RouteConfiguration {
	return &v3routepb.RouteConfiguration{
		Name: routeName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			Domains: []string{vhDomain},
			Routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
				Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
					ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{WeightedClusters: &v3routepb.WeightedCluster{
						Clusters: []*v3routepb.WeightedCluster_ClusterWeight{
							{
								Name:   clusterName,
								Weight: &wrapperspb.UInt32Value{Value: 100},
							},
						},
					}},
				}},
			}},
		}},
	}
}

// RouteConfigClusterSpecifierType determines the cluster specifier type for the
// route actions configured in the returned RouteConfiguration resource.
type RouteConfigClusterSpecifierType int

const (
	// RouteConfigClusterSpecifierTypeCluster results in the cluster specifier
	// being set to a RouteAction_Cluster.
	RouteConfigClusterSpecifierTypeCluster RouteConfigClusterSpecifierType = iota
	// RouteConfigClusterSpecifierTypeWeightedCluster results in the cluster
	// specifier being set to RouteAction_WeightedClusters.
	RouteConfigClusterSpecifierTypeWeightedCluster
	// RouteConfigClusterSpecifierTypeClusterSpecifierPlugin results in the
	// cluster specifier being set to a RouteAction_ClusterSpecifierPlugin.
	RouteConfigClusterSpecifierTypeClusterSpecifierPlugin
)

// RouteConfigOptions contains options to configure a RouteConfiguration
// resource.
type RouteConfigOptions struct {
	// RouteConfigName is the name of the RouteConfiguration resource.
	RouteConfigName string
	// ListenerName is the name of the Listener resource which uses this
	// RouteConfiguration.
	ListenerName string
	// ClusterSpecifierType determines the cluster specifier type.
	ClusterSpecifierType RouteConfigClusterSpecifierType
	// ClusterName is name of the cluster resource used when the cluster
	// specifier type is set to RouteConfigClusterSpecifierTypeCluster.
	//
	// Default value of "A" is used if left unspecified.
	ClusterName string
	// WeightedClusters is a map from cluster name to weights, and is used when
	// the cluster specifier type is set to
	// RouteConfigClusterSpecifierTypeWeightedCluster.
	//
	// Default value of {"A": 75, "B": 25} is used if left unspecified.
	WeightedClusters map[string]int
	// The below two fields specify the name of the cluster specifier plugin and
	// its configuration, and are used when the cluster specifier type is set to
	// RouteConfigClusterSpecifierTypeClusterSpecifierPlugin. Tests are expected
	// to provide valid values for these fields when appropriate.
	ClusterSpecifierPluginName   string
	ClusterSpecifierPluginConfig *anypb.Any
}

// RouteConfigResourceWithOptions returns a RouteConfiguration resource
// configured with the provided options.
func RouteConfigResourceWithOptions(opts RouteConfigOptions) *v3routepb.RouteConfiguration {
	switch opts.ClusterSpecifierType {
	case RouteConfigClusterSpecifierTypeCluster:
		clusterName := opts.ClusterName
		if clusterName == "" {
			clusterName = "A"
		}
		return &v3routepb.RouteConfiguration{
			Name: opts.RouteConfigName,
			VirtualHosts: []*v3routepb.VirtualHost{{
				Domains: []string{opts.ListenerName},
				Routes: []*v3routepb.Route{{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName},
					}},
				}},
			}},
		}
	case RouteConfigClusterSpecifierTypeWeightedCluster:
		weightedClusters := opts.WeightedClusters
		if weightedClusters == nil {
			weightedClusters = map[string]int{"A": 75, "B": 25}
		}
		clusters := []*v3routepb.WeightedCluster_ClusterWeight{}
		for name, weight := range weightedClusters {
			clusters = append(clusters, &v3routepb.WeightedCluster_ClusterWeight{
				Name:   name,
				Weight: &wrapperspb.UInt32Value{Value: uint32(weight)},
			})
		}
		return &v3routepb.RouteConfiguration{
			Name: opts.RouteConfigName,
			VirtualHosts: []*v3routepb.VirtualHost{{
				Domains: []string{opts.ListenerName},
				Routes: []*v3routepb.Route{{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_WeightedClusters{WeightedClusters: &v3routepb.WeightedCluster{Clusters: clusters}},
					}},
				}},
			}},
		}
	case RouteConfigClusterSpecifierTypeClusterSpecifierPlugin:
		return &v3routepb.RouteConfiguration{
			Name: opts.RouteConfigName,
			ClusterSpecifierPlugins: []*v3routepb.ClusterSpecifierPlugin{{
				Extension: &v3corepb.TypedExtensionConfig{
					Name:        opts.ClusterSpecifierPluginName,
					TypedConfig: opts.ClusterSpecifierPluginConfig,
				}},
			},
			VirtualHosts: []*v3routepb.VirtualHost{{
				Domains: []string{opts.ListenerName},
				Routes: []*v3routepb.Route{{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_ClusterSpecifierPlugin{ClusterSpecifierPlugin: opts.ClusterSpecifierPluginName},
					}},
				}},
			}},
		}
	default:
		panic(fmt.Sprintf("unsupported cluster specifier plugin type: %v", opts.ClusterSpecifierType))
	}
}

// DefaultCluster returns a basic xds Cluster resource.
func DefaultCluster(clusterName, edsServiceName string, secLevel SecurityLevel) *v3clusterpb.Cluster {
	return ClusterResourceWithOptions(ClusterOptions{
		ClusterName:   clusterName,
		ServiceName:   edsServiceName,
		Policy:        LoadBalancingPolicyRoundRobin,
		SecurityLevel: secLevel,
	})
}

// LoadBalancingPolicy determines the policy used for balancing load across
// endpoints in the Cluster.
type LoadBalancingPolicy int

const (
	// LoadBalancingPolicyRoundRobin results in the use of the weighted_target
	// LB policy to balance load across localities and endpoints in the cluster.
	LoadBalancingPolicyRoundRobin LoadBalancingPolicy = iota
	// LoadBalancingPolicyRingHash results in the use of the ring_hash LB policy
	// as the leaf policy.
	LoadBalancingPolicyRingHash
)

// ClusterType specifies the type of the Cluster resource.
type ClusterType int

const (
	// ClusterTypeEDS specifies a Cluster that uses EDS to resolve endpoints.
	ClusterTypeEDS ClusterType = iota
	// ClusterTypeLogicalDNS specifies a Cluster that uses DNS to resolve
	// endpoints.
	ClusterTypeLogicalDNS
	// ClusterTypeAggregate specifies a Cluster that is made up of child
	// clusters.
	ClusterTypeAggregate
)

// ClusterOptions contains options to configure a Cluster resource.
type ClusterOptions struct {
	Type ClusterType
	// ClusterName is the name of the Cluster resource.
	ClusterName string
	// ServiceName is the EDS service name of the Cluster. Applicable only when
	// cluster type is EDS.
	ServiceName string
	// ChildNames is the list of child Cluster names. Applicable only when
	// cluster type is Aggregate.
	ChildNames []string
	// DNSHostName is the dns host name of the Cluster. Applicable only when the
	// cluster type is DNS.
	DNSHostName string
	// DNSPort is the port number of the Cluster. Applicable only when the
	// cluster type is DNS.
	DNSPort uint32
	// Policy is the LB policy to be used.
	Policy LoadBalancingPolicy
	// SecurityLevel determines the security configuration for the Cluster.
	SecurityLevel SecurityLevel
	// EnableLRS adds a load reporting configuration with a config source
	// pointing to self.
	EnableLRS bool
}

// ClusterResourceWithOptions returns an xDS Cluster resource configured with
// the provided options.
func ClusterResourceWithOptions(opts ClusterOptions) *v3clusterpb.Cluster {
	var tlsContext *v3tlspb.UpstreamTlsContext
	switch opts.SecurityLevel {
	case SecurityLevelNone:
	case SecurityLevelTLS:
		tlsContext = &v3tlspb.UpstreamTlsContext{
			CommonTlsContext: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
					ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
						InstanceName: ClientSideCertProviderInstance,
					},
				},
			},
		}
	case SecurityLevelMTLS:
		tlsContext = &v3tlspb.UpstreamTlsContext{
			CommonTlsContext: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
					ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
						InstanceName: ClientSideCertProviderInstance,
					},
				},
				TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
					InstanceName: ClientSideCertProviderInstance,
				},
			},
		}
	}

	var lbPolicy v3clusterpb.Cluster_LbPolicy
	switch opts.Policy {
	case LoadBalancingPolicyRoundRobin:
		lbPolicy = v3clusterpb.Cluster_ROUND_ROBIN
	case LoadBalancingPolicyRingHash:
		lbPolicy = v3clusterpb.Cluster_RING_HASH
	}
	cluster := &v3clusterpb.Cluster{
		Name:     opts.ClusterName,
		LbPolicy: lbPolicy,
	}
	switch opts.Type {
	case ClusterTypeEDS:
		cluster.ClusterDiscoveryType = &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS}
		cluster.EdsClusterConfig = &v3clusterpb.Cluster_EdsClusterConfig{
			EdsConfig: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
					Ads: &v3corepb.AggregatedConfigSource{},
				},
			},
			ServiceName: opts.ServiceName,
		}
	case ClusterTypeLogicalDNS:
		cluster.ClusterDiscoveryType = &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_LOGICAL_DNS}
		cluster.LoadAssignment = &v3endpointpb.ClusterLoadAssignment{
			Endpoints: []*v3endpointpb.LocalityLbEndpoints{{
				LbEndpoints: []*v3endpointpb.LbEndpoint{{
					HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{
						Endpoint: &v3endpointpb.Endpoint{
							Address: &v3corepb.Address{
								Address: &v3corepb.Address_SocketAddress{
									SocketAddress: &v3corepb.SocketAddress{
										Address: opts.DNSHostName,
										PortSpecifier: &v3corepb.SocketAddress_PortValue{
											PortValue: opts.DNSPort,
										},
									},
								},
							},
						},
					},
				}},
			}},
		}
	case ClusterTypeAggregate:
		cluster.ClusterDiscoveryType = &v3clusterpb.Cluster_ClusterType{
			ClusterType: &v3clusterpb.Cluster_CustomClusterType{
				Name: "envoy.clusters.aggregate",
				TypedConfig: marshalAny(&v3aggregateclusterpb.ClusterConfig{
					Clusters: opts.ChildNames,
				}),
			},
		}
	}
	if tlsContext != nil {
		cluster.TransportSocket = &v3corepb.TransportSocket{
			Name: "envoy.transport_sockets.tls",
			ConfigType: &v3corepb.TransportSocket_TypedConfig{
				TypedConfig: marshalAny(tlsContext),
			},
		}
	}
	if opts.EnableLRS {
		cluster.LrsServer = &v3corepb.ConfigSource{
			ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
				Self: &v3corepb.SelfConfigSource{},
			},
		}
	}
	return cluster
}

// LocalityOptions contains options to configure a Locality.
type LocalityOptions struct {
	// Name is the unique locality name.
	Name string
	// Weight is the weight of the locality, used for load balancing.
	Weight uint32
	// Backends is a set of backends belonging to this locality.
	Backends []BackendOptions
	// Priority is the priority of the locality. Defaults to 0.
	Priority uint32
}

// BackendOptions contains options to configure individual backends in a
// locality.
type BackendOptions struct {
	// Port number on which the backend is accepting connections. All backends
	// are expected to run on localhost, hence host name is not stored here.
	Port uint32
	// Health status of the backend. Default is UNKNOWN which is treated the
	// same as HEALTHY.
	HealthStatus v3corepb.HealthStatus
	// Weight sets the backend weight. Defaults to 1.
	Weight uint32
}

// EndpointOptions contains options to configure an Endpoint (or
// ClusterLoadAssignment) resource.
type EndpointOptions struct {
	// ClusterName is the name of the Cluster resource (or EDS service name)
	// containing the endpoints specified below.
	ClusterName string
	// Host is the hostname of the endpoints. In our e2e tests, hostname must
	// always be "localhost".
	Host string
	// Localities is a set of localities belonging to this resource.
	Localities []LocalityOptions
	// DropPercents is a map from drop category to a drop percentage. If unset,
	// no drops are configured.
	DropPercents map[string]int
}

// DefaultEndpoint returns a basic xds Endpoint resource.
func DefaultEndpoint(clusterName string, host string, ports []uint32) *v3endpointpb.ClusterLoadAssignment {
	var bOpts []BackendOptions
	for _, p := range ports {
		bOpts = append(bOpts, BackendOptions{Port: p, Weight: 1})
	}
	return EndpointResourceWithOptions(EndpointOptions{
		ClusterName: clusterName,
		Host:        host,
		Localities: []LocalityOptions{
			{
				Backends: bOpts,
				Weight:   1,
			},
		},
	})
}

// EndpointResourceWithOptions returns an xds Endpoint resource configured with
// the provided options.
func EndpointResourceWithOptions(opts EndpointOptions) *v3endpointpb.ClusterLoadAssignment {
	var endpoints []*v3endpointpb.LocalityLbEndpoints
	for i, locality := range opts.Localities {
		var lbEndpoints []*v3endpointpb.LbEndpoint
		for _, b := range locality.Backends {
			// Weight defaults to 1.
			if b.Weight == 0 {
				b.Weight = 1
			}
			lbEndpoints = append(lbEndpoints, &v3endpointpb.LbEndpoint{
				HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{Endpoint: &v3endpointpb.Endpoint{
					Address: &v3corepb.Address{Address: &v3corepb.Address_SocketAddress{
						SocketAddress: &v3corepb.SocketAddress{
							Protocol:      v3corepb.SocketAddress_TCP,
							Address:       opts.Host,
							PortSpecifier: &v3corepb.SocketAddress_PortValue{PortValue: b.Port},
						},
					}},
				}},
				HealthStatus:        b.HealthStatus,
				LoadBalancingWeight: &wrapperspb.UInt32Value{Value: b.Weight},
			})
		}

		endpoints = append(endpoints, &v3endpointpb.LocalityLbEndpoints{
			Locality: &v3corepb.Locality{
				Region:  fmt.Sprintf("region-%d", i+1),
				Zone:    fmt.Sprintf("zone-%d", i+1),
				SubZone: fmt.Sprintf("subzone-%d", i+1),
			},
			LbEndpoints:         lbEndpoints,
			LoadBalancingWeight: &wrapperspb.UInt32Value{Value: locality.Weight},
			Priority:            locality.Priority,
		})
	}

	cla := &v3endpointpb.ClusterLoadAssignment{
		ClusterName: opts.ClusterName,
		Endpoints:   endpoints,
	}

	var drops []*v3endpointpb.ClusterLoadAssignment_Policy_DropOverload
	for category, val := range opts.DropPercents {
		drops = append(drops, &v3endpointpb.ClusterLoadAssignment_Policy_DropOverload{
			Category: category,
			DropPercentage: &v3typepb.FractionalPercent{
				Numerator:   uint32(val),
				Denominator: v3typepb.FractionalPercent_HUNDRED,
			},
		})
	}
	if len(drops) != 0 {
		cla.Policy = &v3endpointpb.ClusterLoadAssignment_Policy{
			DropOverloads: drops,
		}
	}
	return cla
}

// DefaultServerListenerWithRouteConfigName returns a basic xds Listener
// resource to be used on the server side. The returned Listener resource
// contains a RouteConfiguration resource name that needs to be resolved.
func DefaultServerListenerWithRouteConfigName(host string, port uint32, secLevel SecurityLevel, routeName string) *v3listenerpb.Listener {
	return defaultServerListenerCommon(host, port, secLevel, routeName, false)
}

// RouteConfigNoRouteMatch returns an xDS RouteConfig resource which a route
// with no route match. This will be NACKed by the xDS Client.
func RouteConfigNoRouteMatch(routeName string) *v3routepb.RouteConfiguration {
	return &v3routepb.RouteConfiguration{
		Name: routeName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			// This "*" string matches on any incoming authority. This is to ensure any
			// incoming RPC matches to Route_NonForwardingAction and will proceed as
			// normal.
			Domains: []string{"*"},
			Routes: []*v3routepb.Route{{
				Action: &v3routepb.Route_NonForwardingAction{},
			}}}}}
}

// RouteConfigNonForwardingAction returns an xDS RouteConfig resource which
// specifies to route to a route specifying non forwarding action. This is
// intended to be used on the server side for RDS requests, and corresponds to
// the inline route configuration in DefaultServerListener.
func RouteConfigNonForwardingAction(routeName string) *v3routepb.RouteConfiguration {
	return &v3routepb.RouteConfiguration{
		Name: routeName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			// This "*" string matches on any incoming authority. This is to ensure any
			// incoming RPC matches to Route_NonForwardingAction and will proceed as
			// normal.
			Domains: []string{"*"},
			Routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{
					PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
				},
				Action: &v3routepb.Route_NonForwardingAction{},
			}}}}}
}

// RouteConfigFilterAction returns an xDS RouteConfig resource which specifies
// to route to a route specifying route filter action. Since this is not type
// non forwarding action, this should fail requests that match to this server
// side.
func RouteConfigFilterAction(routeName string) *v3routepb.RouteConfiguration {
	return &v3routepb.RouteConfiguration{
		Name: routeName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			// This "*" string matches on any incoming authority. This is to
			// ensure any incoming RPC matches to Route_Route and will fail with
			// UNAVAILABLE.
			Domains: []string{"*"},
			Routes: []*v3routepb.Route{{
				Match: &v3routepb.RouteMatch{
					PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
				},
				Action: &v3routepb.Route_FilterAction{},
			}}}}}
}
