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

// Package tests_test contains test cases for unmarshalling of CDS resources.
package tests_test

import (
	"encoding/json"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/balancer/leastrequest"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/internal/grpctest"
	iserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/balancer/ringhash"
	"google.golang.org/grpc/xds/internal/balancer/wrrlocality"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3xdsxdstypepb "github.com/cncf/xds/go/xds/type/v3"
	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3aggregateclusterpb "github.com/envoyproxy/go-control-plane/envoy/extensions/clusters/aggregate/v3"
	v3ringhashpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/ring_hash/v3"
	v3roundrobinpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/round_robin/v3"
	v3wrrlocalitypb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/wrr_locality/v3"

	_ "google.golang.org/grpc/balancer/roundrobin" // To register round_robin load balancer.
	_ "google.golang.org/grpc/xds"                 // Register the xDS LB Registry Converters.
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const (
	clusterName = "clusterName"
	serviceName = "service"
)

func wrrLocality(t *testing.T, m proto.Message) *v3wrrlocalitypb.WrrLocality {
	return &v3wrrlocalitypb.WrrLocality{
		EndpointPickingPolicy: &v3clusterpb.LoadBalancingPolicy{
			Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
				{
					TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
						TypedConfig: testutils.MarshalAny(t, m),
					},
				},
			},
		},
	}
}

func wrrLocalityAny(t *testing.T, m proto.Message) *anypb.Any {
	return testutils.MarshalAny(t, wrrLocality(t, m))
}

type customLBConfig struct {
	serviceconfig.LoadBalancingConfig
}

// We have this test in a separate test package in order to not take a
// dependency on the internal xDS balancer packages within the xDS Client.
func (s) TestValidateCluster_Success(t *testing.T) {
	const customLBPolicyName = "myorg.MyCustomLeastRequestPolicy"
	stub.Register(customLBPolicyName, stub.BalancerFuncs{
		ParseConfig: func(json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			return customLBConfig{}, nil
		},
	})
	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: "test-server"})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	defer func(old bool) { envconfig.LeastRequestLB = old }(envconfig.LeastRequestLB)
	envconfig.LeastRequestLB = true
	tests := []struct {
		name         string
		cluster      *v3clusterpb.Cluster
		serverCfg    *bootstrap.ServerConfig
		wantUpdate   xdsresource.ClusterUpdate
		wantLBConfig *iserviceconfig.BalancerConfig
	}{
		{
			name: "happy-case-logical-dns",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_LOGICAL_DNS},
				LbPolicy:             v3clusterpb.Cluster_ROUND_ROBIN,
				LoadAssignment: &v3endpointpb.ClusterLoadAssignment{
					Endpoints: []*v3endpointpb.LocalityLbEndpoints{{
						LbEndpoints: []*v3endpointpb.LbEndpoint{{
							HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{
								Endpoint: &v3endpointpb.Endpoint{
									Address: &v3corepb.Address{
										Address: &v3corepb.Address_SocketAddress{
											SocketAddress: &v3corepb.SocketAddress{
												Address: "dns_host",
												PortSpecifier: &v3corepb.SocketAddress_PortValue{
													PortValue: 8080,
												},
											},
										},
									},
								},
							},
						}},
					}},
				},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				ClusterType:     xdsresource.ClusterTypeLogicalDNS,
				DNSHostName:     "dns_host:8080",
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name: "happy-case-aggregate-v3",
			cluster: &v3clusterpb.Cluster{
				Name: clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_ClusterType{
					ClusterType: &v3clusterpb.Cluster_CustomClusterType{
						Name: "envoy.clusters.aggregate",
						TypedConfig: testutils.MarshalAny(t, &v3aggregateclusterpb.ClusterConfig{
							Clusters: []string{"a", "b", "c"},
						}),
					},
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:             clusterName,
				ClusterType:             xdsresource.ClusterTypeAggregate,
				PrioritizedClusterNames: []string{"a", "b", "c"},
				TelemetryLabels:         internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name:    "happy-case-no-service-name-no-lrs",
			cluster: e2e.DefaultCluster(clusterName, "", e2e.SecurityLevelNone),
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name:    "happy-case-no-lrs",
			cluster: e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone),
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name: "happiest-case-with-lrs",
			cluster: e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
				ClusterName: clusterName,
				ServiceName: serviceName,
				EnableLRS:   true,
			}),
			serverCfg: serverCfg,
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				LRSServerConfig: serverCfg,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name: "happiest-case-with-circuitbreakers",
			cluster: func() *v3clusterpb.Cluster {
				c := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
					ClusterName: clusterName,
					ServiceName: serviceName,
					EnableLRS:   true,
				})
				c.CircuitBreakers = &v3clusterpb.CircuitBreakers{
					Thresholds: []*v3clusterpb.CircuitBreakers_Thresholds{
						{
							Priority:    v3corepb.RoutingPriority_DEFAULT,
							MaxRequests: wrapperspb.UInt32(512),
						},
						{
							Priority:    v3corepb.RoutingPriority_HIGH,
							MaxRequests: nil,
						},
					},
				}
				return c
			}(),
			serverCfg: serverCfg,
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				LRSServerConfig: serverCfg,
				MaxRequests:     func() *uint32 { i := uint32(512); return &i }(),
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name: "happiest-case-with-ring-hash-lb-policy-with-default-config",
			cluster: func() *v3clusterpb.Cluster {
				c := e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)
				c.LbPolicy = v3clusterpb.Cluster_RING_HASH
				return c
			}(),
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: "ring_hash_experimental",
				Config: &ringhash.LBConfig{
					MinRingSize: 1024,
					MaxRingSize: 4096,
				},
			},
		},
		{
			name: "happiest-case-with-least-request-lb-policy-with-default-config",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_LEAST_REQUEST,
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: "least_request_experimental",
				Config: &leastrequest.LBConfig{
					ChoiceCount: 2,
				},
			},
		},
		{
			name: "happiest-case-with-ring-hash-lb-policy-with-none-default-config",
			cluster: func() *v3clusterpb.Cluster {
				c := e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)
				c.LbPolicy = v3clusterpb.Cluster_RING_HASH
				c.LbConfig = &v3clusterpb.Cluster_RingHashLbConfig_{
					RingHashLbConfig: &v3clusterpb.Cluster_RingHashLbConfig{
						MinimumRingSize: wrapperspb.UInt64(10),
						MaximumRingSize: wrapperspb.UInt64(100),
					},
				}
				return c
			}(),
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: "ring_hash_experimental",
				Config: &ringhash.LBConfig{
					MinRingSize: 10,
					MaxRingSize: 100,
				},
			},
		},
		{
			name: "happiest-case-with-least-request-lb-policy-with-none-default-config",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_LEAST_REQUEST,
				LbConfig: &v3clusterpb.Cluster_LeastRequestLbConfig_{
					LeastRequestLbConfig: &v3clusterpb.Cluster_LeastRequestLbConfig{
						ChoiceCount: wrapperspb.UInt32(3),
					},
				},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: "least_request_experimental",
				Config: &leastrequest.LBConfig{
					ChoiceCount: 3,
				},
			},
		},
		{
			name: "happiest-case-with-ring-hash-lb-policy-configured-through-LoadBalancingPolicy",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LoadBalancingPolicy: &v3clusterpb.LoadBalancingPolicy{
					Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
						{
							TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
								TypedConfig: testutils.MarshalAny(t, &v3ringhashpb.RingHash{
									HashFunction:    v3ringhashpb.RingHash_XX_HASH,
									MinimumRingSize: wrapperspb.UInt64(10),
									MaximumRingSize: wrapperspb.UInt64(100),
								}),
							},
						},
					},
				},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: "ring_hash_experimental",
				Config: &ringhash.LBConfig{
					MinRingSize: 10,
					MaxRingSize: 100,
				},
			},
		},
		{
			name: "happiest-case-with-wrrlocality-rr-child-configured-through-LoadBalancingPolicy",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LoadBalancingPolicy: &v3clusterpb.LoadBalancingPolicy{
					Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
						{
							TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
								TypedConfig: wrrLocalityAny(t, &v3roundrobinpb.RoundRobin{}),
							},
						},
					},
				},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name: "round_robin",
					},
				},
			},
		},
		{
			name: "happiest-case-with-custom-lb-configured-through-LoadBalancingPolicy",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LoadBalancingPolicy: &v3clusterpb.LoadBalancingPolicy{
					Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
						{
							TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
								TypedConfig: wrrLocalityAny(t, &v3xdsxdstypepb.TypedStruct{
									TypeUrl: "type.googleapis.com/myorg.MyCustomLeastRequestPolicy",
									Value:   &structpb.Struct{},
								}),
							},
						},
					},
				},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: wrrlocality.Name,
				Config: &wrrlocality.LBConfig{
					ChildPolicy: &iserviceconfig.BalancerConfig{
						Name:   "myorg.MyCustomLeastRequestPolicy",
						Config: customLBConfig{},
					},
				},
			},
		},
		{
			name: "load-balancing-policy-takes-precedence-over-lb-policy-and-enum",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_RING_HASH,
				LbConfig: &v3clusterpb.Cluster_RingHashLbConfig_{
					RingHashLbConfig: &v3clusterpb.Cluster_RingHashLbConfig{
						MinimumRingSize: wrapperspb.UInt64(20),
						MaximumRingSize: wrapperspb.UInt64(200),
					},
				},
				LoadBalancingPolicy: &v3clusterpb.LoadBalancingPolicy{
					Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
						{
							TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
								TypedConfig: testutils.MarshalAny(t, &v3ringhashpb.RingHash{
									HashFunction:    v3ringhashpb.RingHash_XX_HASH,
									MinimumRingSize: wrapperspb.UInt64(10),
									MaximumRingSize: wrapperspb.UInt64(100),
								}),
							},
						},
					},
				},
			},
			wantUpdate: xdsresource.ClusterUpdate{
				ClusterName:     clusterName,
				EDSServiceName:  serviceName,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
			wantLBConfig: &iserviceconfig.BalancerConfig{
				Name: "ring_hash_experimental",
				Config: &ringhash.LBConfig{
					MinRingSize: 10,
					MaxRingSize: 100,
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			update, err := xdsresource.ValidateClusterAndConstructClusterUpdateForTesting(test.cluster, test.serverCfg)
			if err != nil {
				t.Errorf("validateClusterAndConstructClusterUpdate(%+v) failed: %v", test.cluster, err)
			}
			// Ignore the raw JSON string into the cluster update. JSON bytes
			// are nondeterministic (whitespace etc.) so we cannot reliably
			// compare JSON bytes in a test. Thus, marshal into a Balancer
			// Config struct and compare on that. Only need to test this JSON
			// emission here, as this covers the possible output space.
			if diff := cmp.Diff(update, test.wantUpdate, cmpopts.EquateEmpty(), cmpopts.IgnoreFields(xdsresource.ClusterUpdate{}, "LBPolicy")); diff != "" {
				t.Errorf("validateClusterAndConstructClusterUpdate(%+v) got diff: %v (-got, +want)", test.cluster, diff)
			}
			bc := &iserviceconfig.BalancerConfig{}
			if err := json.Unmarshal(update.LBPolicy, bc); err != nil {
				t.Fatalf("failed to unmarshal JSON: %v", err)
			}
			if diff := cmp.Diff(bc, test.wantLBConfig); diff != "" {
				t.Fatalf("update.LBConfig got unexpected output, diff (-got +want): %v", diff)
			}
		})
	}
}
