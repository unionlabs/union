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
	"encoding/json"
	"regexp"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/internal/xds/matcher"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3aggregateclusterpb "github.com/envoyproxy/go-control-plane/envoy/extensions/clusters/aggregate/v3"
	v3leastrequestpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/least_request/v3"
	v3ringhashpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/ring_hash/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

const (
	clusterName = "clusterName"
	serviceName = "service"
)

func (s) TestValidateCluster_Failure(t *testing.T) {
	tests := []struct {
		name    string
		cluster *v3clusterpb.Cluster
		wantErr bool
	}{
		{
			name: "non-supported-cluster-type-static",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_STATIC},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
				},
				LbPolicy: v3clusterpb.Cluster_LEAST_REQUEST,
			},
			wantErr: true,
		},
		{
			name: "non-supported-cluster-type-original-dst",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_ORIGINAL_DST},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
				},
				LbPolicy: v3clusterpb.Cluster_LEAST_REQUEST,
			},
			wantErr: true,
		},
		{
			name: "no-eds-config",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				LbPolicy:             v3clusterpb.Cluster_ROUND_ROBIN,
			},
			wantErr: true,
		},
		{
			name: "no-ads-config-source",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig:     &v3clusterpb.Cluster_EdsClusterConfig{},
				LbPolicy:             v3clusterpb.Cluster_ROUND_ROBIN,
			},
			wantErr: true,
		},
		{
			name: "non-round-robin-or-ring-hash-lb-policy",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
				},
				LbPolicy: v3clusterpb.Cluster_LEAST_REQUEST,
			},
			wantErr: true,
		},
		{
			name: "logical-dns-multiple-localities",
			cluster: &v3clusterpb.Cluster{
				Name:                 clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_LOGICAL_DNS},
				LbPolicy:             v3clusterpb.Cluster_ROUND_ROBIN,
				LoadAssignment: &v3endpointpb.ClusterLoadAssignment{
					Endpoints: []*v3endpointpb.LocalityLbEndpoints{
						// Invalid if there are more than one locality.
						{LbEndpoints: nil},
						{LbEndpoints: nil},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "ring-hash-hash-function-not-xx-hash",
			cluster: &v3clusterpb.Cluster{
				LbPolicy: v3clusterpb.Cluster_RING_HASH,
				LbConfig: &v3clusterpb.Cluster_RingHashLbConfig_{
					RingHashLbConfig: &v3clusterpb.Cluster_RingHashLbConfig{
						HashFunction: v3clusterpb.Cluster_RingHashLbConfig_MURMUR_HASH_2,
					},
				},
			},
			wantErr: true,
		},
		{
			name: "least-request-choice-count-less-than-two",
			cluster: &v3clusterpb.Cluster{
				LbPolicy: v3clusterpb.Cluster_RING_HASH,
				LbConfig: &v3clusterpb.Cluster_LeastRequestLbConfig_{
					LeastRequestLbConfig: &v3clusterpb.Cluster_LeastRequestLbConfig{
						ChoiceCount: wrapperspb.UInt32(1),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "ring-hash-max-bound-greater-than-upper-bound",
			cluster: &v3clusterpb.Cluster{
				LbPolicy: v3clusterpb.Cluster_RING_HASH,
				LbConfig: &v3clusterpb.Cluster_RingHashLbConfig_{
					RingHashLbConfig: &v3clusterpb.Cluster_RingHashLbConfig{
						MaximumRingSize: wrapperspb.UInt64(ringHashSizeUpperBound + 1),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "ring-hash-max-bound-greater-than-upper-bound-load-balancing-policy",
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
									MaximumRingSize: wrapperspb.UInt64(ringHashSizeUpperBound + 1),
								}),
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "least-request-unsupported-in-converter-since-env-var-unset",
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
								TypedConfig: testutils.MarshalAny(t, &v3leastrequestpb.LeastRequest{}),
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "aggregate-nil-clusters",
			cluster: &v3clusterpb.Cluster{
				Name: clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_ClusterType{
					ClusterType: &v3clusterpb.Cluster_CustomClusterType{
						Name:        "envoy.clusters.aggregate",
						TypedConfig: testutils.MarshalAny(t, &v3aggregateclusterpb.ClusterConfig{}),
					},
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			},
			wantErr: true,
		},
		{
			name: "aggregate-empty-clusters",
			cluster: &v3clusterpb.Cluster{
				Name: clusterName,
				ClusterDiscoveryType: &v3clusterpb.Cluster_ClusterType{
					ClusterType: &v3clusterpb.Cluster_CustomClusterType{
						Name: "envoy.clusters.aggregate",
						TypedConfig: testutils.MarshalAny(t, &v3aggregateclusterpb.ClusterConfig{
							Clusters: []string{},
						}),
					},
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			},
			wantErr: true,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if update, err := validateClusterAndConstructClusterUpdate(test.cluster, nil); err == nil {
				t.Errorf("validateClusterAndConstructClusterUpdate(%+v) = %v, wanted error", test.cluster, update)
			}
		})
	}
}

func (s) TestSecurityConfigFromCommonTLSContextUsingNewFields_ErrorCases(t *testing.T) {
	tests := []struct {
		name    string
		common  *v3tlspb.CommonTlsContext
		server  bool
		wantErr string
	}{
		{
			name: "unsupported-tls_certificates-field-for-identity-certs",
			common: &v3tlspb.CommonTlsContext{
				TlsCertificates: []*v3tlspb.TlsCertificate{
					{CertificateChain: &v3corepb.DataSource{}},
				},
			},
			wantErr: "unsupported field tls_certificates is set in CommonTlsContext message",
		},
		{
			name: "unsupported-tls_certificates_sds_secret_configs-field-for-identity-certs",
			common: &v3tlspb.CommonTlsContext{
				TlsCertificateSdsSecretConfigs: []*v3tlspb.SdsSecretConfig{
					{Name: "sds-secrets-config"},
				},
			},
			wantErr: "unsupported field tls_certificate_sds_secret_configs is set in CommonTlsContext message",
		},
		{
			name: "unsupported-sds-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextSdsSecretConfig{
					ValidationContextSdsSecretConfig: &v3tlspb.SdsSecretConfig{
						Name: "foo-sds-secret",
					},
				},
			},
			wantErr: "validation context contains unexpected type",
		},
		{
			name: "missing-ca_certificate_provider_instance-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{},
				},
			},
			wantErr: "expected field ca_certificate_provider_instance is missing in CommonTlsContext message",
		},
		{
			name: "unsupported-field-verify_certificate_spki-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						VerifyCertificateSpki: []string{"spki"},
					},
				},
			},
			wantErr: "unsupported verify_certificate_spki field in CommonTlsContext message",
		},
		{
			name: "unsupported-field-verify_certificate_hash-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						VerifyCertificateHash: []string{"hash"},
					},
				},
			},
			wantErr: "unsupported verify_certificate_hash field in CommonTlsContext message",
		},
		{
			name: "unsupported-field-require_signed_certificate_timestamp-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						RequireSignedCertificateTimestamp: &wrapperspb.BoolValue{Value: true},
					},
				},
			},
			wantErr: "unsupported require_signed_certificate_timestamp field in CommonTlsContext message",
		},
		{
			name: "unsupported-field-crl-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						Crl: &v3corepb.DataSource{},
					},
				},
			},
			wantErr: "unsupported crl field in CommonTlsContext message",
		},
		{
			name: "unsupported-field-custom_validator_config-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						CustomValidatorConfig: &v3corepb.TypedExtensionConfig{},
					},
				},
			},
			wantErr: "unsupported custom_validator_config field in CommonTlsContext message",
		},
		{
			name: "invalid-match_subject_alt_names-field-in-validation-context",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
							{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: ""}},
						},
					},
				},
			},
			wantErr: "empty prefix is not allowed in StringMatcher",
		},
		{
			name: "unsupported-field-matching-subject-alt-names-in-validation-context-of-server",
			common: &v3tlspb.CommonTlsContext{
				ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
					ValidationContext: &v3tlspb.CertificateValidationContext{
						CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
							InstanceName:    "rootPluginInstance",
							CertificateName: "rootCertName",
						},
						MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
							{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "sanPrefix"}},
						},
					},
				},
			},
			server:  true,
			wantErr: "match_subject_alt_names field in validation context is not supported on the server",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			_, err := securityConfigFromCommonTLSContextUsingNewFields(test.common, test.server)
			if err == nil {
				t.Fatal("securityConfigFromCommonTLSContextUsingNewFields() succeeded when expected to fail")
			}
			if !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("securityConfigFromCommonTLSContextUsingNewFields() returned err: %v, wantErr: %v", err, test.wantErr)
			}
		})
	}
}

func (s) TestValidateClusterWithSecurityConfig(t *testing.T) {
	const (
		identityPluginInstance = "identityPluginInstance"
		identityCertName       = "identityCert"
		rootPluginInstance     = "rootPluginInstance"
		rootCertName           = "rootCert"
		clusterName            = "cluster"
		serviceName            = "service"
		sanExact               = "san-exact"
		sanPrefix              = "san-prefix"
		sanSuffix              = "san-suffix"
		sanRegexBad            = "??"
		sanRegexGood           = "san?regex?"
		sanContains            = "san-contains"
	)
	var sanRE = regexp.MustCompile(sanRegexGood)

	tests := []struct {
		name       string
		cluster    *v3clusterpb.Cluster
		wantUpdate ClusterUpdate
		wantErr    bool
	}{
		{
			name: "transport-socket-matches",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocketMatches: []*v3clusterpb.Cluster_TransportSocketMatch{
					{Name: "transport-socket-match-1"},
				},
			},
			wantErr: true,
		},
		{
			name: "transport-socket-unsupported-name",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "unsupported-foo",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: &anypb.Any{
							TypeUrl: version.V3UpstreamTLSContextURL,
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "transport-socket-unsupported-typeURL",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: &anypb.Any{
							TypeUrl: version.V3HTTPConnManagerURL,
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "transport-socket-unsupported-type",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: &anypb.Any{
							TypeUrl: version.V3UpstreamTLSContextURL,
							Value:   []byte{1, 2, 3, 4},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "transport-socket-unsupported-tls-params-field",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsParams: &v3tlspb.TlsParameters{},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "transport-socket-unsupported-custom-handshaker-field",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								CustomHandshaker: &v3corepb.TypedExtensionConfig{},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "transport-socket-unsupported-validation-context",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
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
			wantErr: true,
		},
		{
			name: "transport-socket-without-validation-context",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "empty-prefix-in-matching-SAN",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: ""}},
											},
										},
										ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "empty-suffix-in-matching-SAN",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{MatchPattern: &v3matcherpb.StringMatcher_Suffix{Suffix: ""}},
											},
										},
										ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "empty-contains-in-matching-SAN",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{MatchPattern: &v3matcherpb.StringMatcher_Contains{Contains: ""}},
											},
										},
										ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "invalid-regex-in-matching-SAN",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: sanRegexBad}}},
											},
										},
										ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "invalid-regex-in-matching-SAN-with-new-fields",
			cluster: &v3clusterpb.Cluster{
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: serviceName,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: sanRegexBad}}},
											},
											CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
												InstanceName:    rootPluginInstance,
												CertificateName: rootCertName,
											},
										},
									},
								},
							},
						}),
					},
				},
			},
			wantErr: true,
		},
		{
			name: "happy-case-with-no-identity-certs-using-deprecated-fields",
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
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
									ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
										InstanceName:    rootPluginInstance,
										CertificateName: rootCertName,
									},
								},
							},
						}),
					},
				},
			},
			wantUpdate: ClusterUpdate{
				ClusterName:    clusterName,
				EDSServiceName: serviceName,
				SecurityCfg: &SecurityConfig{
					RootInstanceName: rootPluginInstance,
					RootCertName:     rootCertName,
				},
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name: "happy-case-with-no-identity-certs-using-new-fields",
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
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
									ValidationContext: &v3tlspb.CertificateValidationContext{
										CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantUpdate: ClusterUpdate{
				ClusterName:    clusterName,
				EDSServiceName: serviceName,
				SecurityCfg: &SecurityConfig{
					RootInstanceName: rootPluginInstance,
					RootCertName:     rootCertName,
				},
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name: "happy-case-with-validation-context-provider-instance-using-deprecated-fields",
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
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
									InstanceName:    identityPluginInstance,
									CertificateName: identityCertName,
								},
								ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
									ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
										InstanceName:    rootPluginInstance,
										CertificateName: rootCertName,
									},
								},
							},
						}),
					},
				},
			},
			wantUpdate: ClusterUpdate{
				ClusterName:    clusterName,
				EDSServiceName: serviceName,
				SecurityCfg: &SecurityConfig{
					RootInstanceName:     rootPluginInstance,
					RootCertName:         rootCertName,
					IdentityInstanceName: identityPluginInstance,
					IdentityCertName:     identityCertName,
				},
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name: "happy-case-with-validation-context-provider-instance-using-new-fields",
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
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
									InstanceName:    identityPluginInstance,
									CertificateName: identityCertName,
								},
								ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
									ValidationContext: &v3tlspb.CertificateValidationContext{
										CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantUpdate: ClusterUpdate{
				ClusterName:    clusterName,
				EDSServiceName: serviceName,
				SecurityCfg: &SecurityConfig{
					RootInstanceName:     rootPluginInstance,
					RootCertName:         rootCertName,
					IdentityInstanceName: identityPluginInstance,
					IdentityCertName:     identityCertName,
				},
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name: "happy-case-with-combined-validation-context-using-deprecated-fields",
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
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
									InstanceName:    identityPluginInstance,
									CertificateName: identityCertName,
								},
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{
													MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: sanExact},
													IgnoreCase:   true,
												},
												{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: sanPrefix}},
												{MatchPattern: &v3matcherpb.StringMatcher_Suffix{Suffix: sanSuffix}},
												{MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: sanRegexGood}}},
												{MatchPattern: &v3matcherpb.StringMatcher_Contains{Contains: sanContains}},
											},
										},
										ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
											InstanceName:    rootPluginInstance,
											CertificateName: rootCertName,
										},
									},
								},
							},
						}),
					},
				},
			},
			wantUpdate: ClusterUpdate{
				ClusterName:    clusterName,
				EDSServiceName: serviceName,
				SecurityCfg: &SecurityConfig{
					RootInstanceName:     rootPluginInstance,
					RootCertName:         rootCertName,
					IdentityInstanceName: identityPluginInstance,
					IdentityCertName:     identityCertName,
					SubjectAltNameMatchers: []matcher.StringMatcher{
						matcher.StringMatcherForTesting(newStringP(sanExact), nil, nil, nil, nil, true),
						matcher.StringMatcherForTesting(nil, newStringP(sanPrefix), nil, nil, nil, false),
						matcher.StringMatcherForTesting(nil, nil, newStringP(sanSuffix), nil, nil, false),
						matcher.StringMatcherForTesting(nil, nil, nil, nil, sanRE, false),
						matcher.StringMatcherForTesting(nil, nil, nil, newStringP(sanContains), nil, false),
					},
				},
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name: "happy-case-with-combined-validation-context-using-new-fields",
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
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				TransportSocket: &v3corepb.TransportSocket{
					Name: "envoy.transport_sockets.tls",
					ConfigType: &v3corepb.TransportSocket_TypedConfig{
						TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
							CommonTlsContext: &v3tlspb.CommonTlsContext{
								TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
									InstanceName:    identityPluginInstance,
									CertificateName: identityCertName,
								},
								ValidationContextType: &v3tlspb.CommonTlsContext_CombinedValidationContext{
									CombinedValidationContext: &v3tlspb.CommonTlsContext_CombinedCertificateValidationContext{
										DefaultValidationContext: &v3tlspb.CertificateValidationContext{
											MatchSubjectAltNames: []*v3matcherpb.StringMatcher{
												{
													MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: sanExact},
													IgnoreCase:   true,
												},
												{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: sanPrefix}},
												{MatchPattern: &v3matcherpb.StringMatcher_Suffix{Suffix: sanSuffix}},
												{MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: sanRegexGood}}},
												{MatchPattern: &v3matcherpb.StringMatcher_Contains{Contains: sanContains}},
											},
											CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
												InstanceName:    rootPluginInstance,
												CertificateName: rootCertName,
											},
										},
									},
								},
							},
						}),
					},
				},
			},
			wantUpdate: ClusterUpdate{
				ClusterName:    clusterName,
				EDSServiceName: serviceName,
				SecurityCfg: &SecurityConfig{
					RootInstanceName:     rootPluginInstance,
					RootCertName:         rootCertName,
					IdentityInstanceName: identityPluginInstance,
					IdentityCertName:     identityCertName,
					SubjectAltNameMatchers: []matcher.StringMatcher{
						matcher.StringMatcherForTesting(newStringP(sanExact), nil, nil, nil, nil, true),
						matcher.StringMatcherForTesting(nil, newStringP(sanPrefix), nil, nil, nil, false),
						matcher.StringMatcherForTesting(nil, nil, newStringP(sanSuffix), nil, nil, false),
						matcher.StringMatcherForTesting(nil, nil, nil, nil, sanRE, false),
						matcher.StringMatcherForTesting(nil, nil, nil, newStringP(sanContains), nil, false),
					},
				},
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			update, err := validateClusterAndConstructClusterUpdate(test.cluster, nil)
			if (err != nil) != test.wantErr {
				t.Errorf("validateClusterAndConstructClusterUpdate() returned err %v wantErr %v)", err, test.wantErr)
			}
			if diff := cmp.Diff(test.wantUpdate, update, cmpopts.EquateEmpty(), cmp.AllowUnexported(regexp.Regexp{}), cmpopts.IgnoreFields(ClusterUpdate{}, "LBPolicy")); diff != "" {
				t.Errorf("validateClusterAndConstructClusterUpdate() returned unexpected diff (-want, +got):\n%s", diff)
			}
		})
	}
}

func (s) TestUnmarshalCluster(t *testing.T) {
	const (
		v3ClusterName = "v3clusterName"
		v3Service     = "v3Service"
	)
	var (
		v3ClusterAny = testutils.MarshalAny(t, &v3clusterpb.Cluster{
			Name:                 v3ClusterName,
			ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
			EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
				EdsConfig: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
						Ads: &v3corepb.AggregatedConfigSource{},
					},
				},
				ServiceName: v3Service,
			},
			LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			LrsServer: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
					Self: &v3corepb.SelfConfigSource{},
				},
			},
		})

		v3ClusterAnyWithEDSConfigSourceSelf = testutils.MarshalAny(t, &v3clusterpb.Cluster{
			Name:                 v3ClusterName,
			ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
			EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
				EdsConfig: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{},
				},
				ServiceName: v3Service,
			},
			LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			LrsServer: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
					Self: &v3corepb.SelfConfigSource{},
				},
			},
		})

		v3ClusterAnyWithTelemetryLabels = testutils.MarshalAny(t, &v3clusterpb.Cluster{
			Name:                 v3ClusterName,
			ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
			EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
				EdsConfig: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
						Ads: &v3corepb.AggregatedConfigSource{},
					},
				},
				ServiceName: v3Service,
			},
			LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			LrsServer: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
					Self: &v3corepb.SelfConfigSource{},
				},
			},
			Metadata: &v3corepb.Metadata{
				FilterMetadata: map[string]*structpb.Struct{
					"com.google.csm.telemetry_labels": {
						Fields: map[string]*structpb.Value{
							"service_name":      structpb.NewStringValue("grpc-service"),
							"service_namespace": structpb.NewStringValue("grpc-service-namespace"),
						},
					},
				},
			},
		})
		v3ClusterAnyWithTelemetryLabelsIgnoreSome = testutils.MarshalAny(t, &v3clusterpb.Cluster{
			Name:                 v3ClusterName,
			ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
			EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
				EdsConfig: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
						Ads: &v3corepb.AggregatedConfigSource{},
					},
				},
				ServiceName: v3Service,
			},
			LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
			LrsServer: &v3corepb.ConfigSource{
				ConfigSourceSpecifier: &v3corepb.ConfigSource_Self{
					Self: &v3corepb.SelfConfigSource{},
				},
			},
			Metadata: &v3corepb.Metadata{
				FilterMetadata: map[string]*structpb.Struct{
					"com.google.csm.telemetry_labels": {
						Fields: map[string]*structpb.Value{
							"string-value-should-ignore": structpb.NewStringValue("string-val"),
							"float-value-ignore":         structpb.NewNumberValue(3),
							"bool-value-ignore":          structpb.NewBoolValue(false),
							"service_name":               structpb.NewStringValue("grpc-service"), // shouldn't ignore
							"service_namespace":          structpb.NewNullValue(),                 // should ignore - wrong type
						},
					},
					"ignore-this-metadata": { // should ignore this filter_metadata
						Fields: map[string]*structpb.Value{
							"service_namespace": structpb.NewStringValue("string-val-should-ignore"),
						},
					},
				},
			},
		})
	)

	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: "test-server"})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	tests := []struct {
		name       string
		resource   *anypb.Any
		serverCfg  *bootstrap.ServerConfig
		wantName   string
		wantUpdate ClusterUpdate
		wantErr    bool
	}{
		{
			name:     "non-cluster resource type",
			resource: &anypb.Any{TypeUrl: version.V3HTTPConnManagerURL},
			wantErr:  true,
		},
		{
			name: "badly marshaled cluster resource",
			resource: &anypb.Any{
				TypeUrl: version.V3ClusterURL,
				Value:   []byte{1, 2, 3, 4},
			},
			wantErr: true,
		},
		{
			name: "bad cluster resource",
			resource: testutils.MarshalAny(t, &v3clusterpb.Cluster{
				Name:                 "test",
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_STATIC},
			}),
			wantName: "test",
			wantErr:  true,
		},
		{
			name: "cluster resource with non-self lrs_server field",
			resource: testutils.MarshalAny(t, &v3clusterpb.Cluster{
				Name:                 "test",
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: v3Service,
				},
				LbPolicy: v3clusterpb.Cluster_ROUND_ROBIN,
				LrsServer: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
						Ads: &v3corepb.AggregatedConfigSource{},
					},
				},
			}),
			wantName: "test",
			wantErr:  true,
		},
		{
			name:      "v3 cluster",
			resource:  v3ClusterAny,
			serverCfg: serverCfg,
			wantName:  v3ClusterName,
			wantUpdate: ClusterUpdate{
				ClusterName:     v3ClusterName,
				EDSServiceName:  v3Service,
				LRSServerConfig: serverCfg,
				Raw:             v3ClusterAny,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name:      "v3 cluster wrapped",
			resource:  testutils.MarshalAny(t, &v3discoverypb.Resource{Resource: v3ClusterAny}),
			serverCfg: serverCfg,
			wantName:  v3ClusterName,
			wantUpdate: ClusterUpdate{
				ClusterName:     v3ClusterName,
				EDSServiceName:  v3Service,
				LRSServerConfig: serverCfg,
				Raw:             v3ClusterAny,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name:      "v3 cluster with EDS config source self",
			resource:  v3ClusterAnyWithEDSConfigSourceSelf,
			serverCfg: serverCfg,
			wantName:  v3ClusterName,
			wantUpdate: ClusterUpdate{
				ClusterName:     v3ClusterName,
				EDSServiceName:  v3Service,
				LRSServerConfig: serverCfg,
				Raw:             v3ClusterAnyWithEDSConfigSourceSelf,
				TelemetryLabels: internal.UnknownCSMLabels,
			},
		},
		{
			name:      "v3 cluster with telemetry case",
			resource:  v3ClusterAnyWithTelemetryLabels,
			serverCfg: serverCfg,
			wantName:  v3ClusterName,
			wantUpdate: ClusterUpdate{
				ClusterName:     v3ClusterName,
				EDSServiceName:  v3Service,
				LRSServerConfig: serverCfg,
				Raw:             v3ClusterAnyWithTelemetryLabels,
				TelemetryLabels: map[string]string{
					"csm.service_name":           "grpc-service",
					"csm.service_namespace_name": "grpc-service-namespace",
				},
			},
		},
		{
			name:      "v3 metadata ignore other types not string and not com.google.csm.telemetry_labels",
			resource:  v3ClusterAnyWithTelemetryLabelsIgnoreSome,
			serverCfg: serverCfg,
			wantName:  v3ClusterName,
			wantUpdate: ClusterUpdate{
				ClusterName:     v3ClusterName,
				EDSServiceName:  v3Service,
				LRSServerConfig: serverCfg,
				Raw:             v3ClusterAnyWithTelemetryLabelsIgnoreSome,
				TelemetryLabels: map[string]string{
					"csm.service_name":           "grpc-service",
					"csm.service_namespace_name": "unknown",
				},
			},
		},
		{
			name: "xdstp cluster resource with unset EDS service name",
			resource: testutils.MarshalAny(t, &v3clusterpb.Cluster{
				Name:                 "xdstp:foo",
				ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
				EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
					EdsConfig: &v3corepb.ConfigSource{
						ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
							Ads: &v3corepb.AggregatedConfigSource{},
						},
					},
					ServiceName: "",
				},
			}),
			wantName: "xdstp:foo",
			wantErr:  true,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			name, update, err := unmarshalClusterResource(test.resource, test.serverCfg)
			if (err != nil) != test.wantErr {
				t.Fatalf("unmarshalClusterResource(%s), got err: %v, wantErr: %v", pretty.ToJSON(test.resource), err, test.wantErr)
			}
			if name != test.wantName {
				t.Errorf("unmarshalClusterResource(%s), got name: %s, want: %s", pretty.ToJSON(test.resource), name, test.wantName)
			}
			if diff := cmp.Diff(update, test.wantUpdate, cmpOpts, cmpopts.IgnoreFields(ClusterUpdate{}, "LBPolicy")); diff != "" {
				t.Errorf("unmarshalClusterResource(%s), got unexpected update, diff (-got +want): %v", pretty.ToJSON(test.resource), diff)
			}
		})
	}
}

func (s) TestValidateClusterWithOutlierDetection(t *testing.T) {
	odToClusterProto := func(od *v3clusterpb.OutlierDetection) *v3clusterpb.Cluster {
		// Cluster parsing doesn't fail with respect to fields orthogonal to
		// outlier detection.
		return &v3clusterpb.Cluster{
			Name:                 clusterName,
			ClusterDiscoveryType: &v3clusterpb.Cluster_Type{Type: v3clusterpb.Cluster_EDS},
			EdsClusterConfig: &v3clusterpb.Cluster_EdsClusterConfig{
				EdsConfig: &v3corepb.ConfigSource{
					ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{
						Ads: &v3corepb.AggregatedConfigSource{},
					},
				},
			},
			LbPolicy:         v3clusterpb.Cluster_ROUND_ROBIN,
			OutlierDetection: od,
		}
	}

	tests := []struct {
		name      string
		cluster   *v3clusterpb.Cluster
		wantODCfg string
		wantErr   bool
	}{
		{
			name:      "success-and-failure-null",
			cluster:   odToClusterProto(&v3clusterpb.OutlierDetection{}),
			wantODCfg: `{"successRateEjection": {}}`,
		},
		{
			name: "success-and-failure-zero",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{
				EnforcingSuccessRate:       &wrapperspb.UInt32Value{Value: 0}, // Thus doesn't create sre - to focus on fpe
				EnforcingFailurePercentage: &wrapperspb.UInt32Value{Value: 0},
			}),
			wantODCfg: `{}`,
		},
		{
			name: "some-fields-set",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{
				Interval:                       &durationpb.Duration{Seconds: 1},
				MaxEjectionTime:                &durationpb.Duration{Seconds: 3},
				EnforcingSuccessRate:           &wrapperspb.UInt32Value{Value: 3},
				SuccessRateRequestVolume:       &wrapperspb.UInt32Value{Value: 5},
				EnforcingFailurePercentage:     &wrapperspb.UInt32Value{Value: 7},
				FailurePercentageRequestVolume: &wrapperspb.UInt32Value{Value: 9},
			}),
			wantODCfg: `{
				"interval": "1s",
				"maxEjectionTime": "3s",
				"successRateEjection": {
					"enforcementPercentage": 3,
					"requestVolume": 5
				},
				"failurePercentageEjection": {
					"enforcementPercentage": 7,
					"requestVolume": 9
				}
			}`,
		},
		{
			name: "every-field-set-non-zero",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{
				// all fields set (including ones that will be layered) should
				// pick up those too and explicitly all fields, including those
				// put in layers, in the JSON generated.
				Interval:                       &durationpb.Duration{Seconds: 1},
				BaseEjectionTime:               &durationpb.Duration{Seconds: 2},
				MaxEjectionTime:                &durationpb.Duration{Seconds: 3},
				MaxEjectionPercent:             &wrapperspb.UInt32Value{Value: 1},
				SuccessRateStdevFactor:         &wrapperspb.UInt32Value{Value: 2},
				EnforcingSuccessRate:           &wrapperspb.UInt32Value{Value: 3},
				SuccessRateMinimumHosts:        &wrapperspb.UInt32Value{Value: 4},
				SuccessRateRequestVolume:       &wrapperspb.UInt32Value{Value: 5},
				FailurePercentageThreshold:     &wrapperspb.UInt32Value{Value: 6},
				EnforcingFailurePercentage:     &wrapperspb.UInt32Value{Value: 7},
				FailurePercentageMinimumHosts:  &wrapperspb.UInt32Value{Value: 8},
				FailurePercentageRequestVolume: &wrapperspb.UInt32Value{Value: 9},
			}),
			wantODCfg: `{
				"interval": "1s",
				"baseEjectionTime": "2s",
				"maxEjectionTime": "3s",
				"maxEjectionPercent": 1,
				"successRateEjection": {
					"stdevFactor": 2,
					"enforcementPercentage": 3,
					"minimumHosts": 4,
					"requestVolume": 5
				},
				"failurePercentageEjection": {
					"threshold": 6,
					"enforcementPercentage": 7,
					"minimumHosts": 8,
					"requestVolume": 9
				}
			}`,
		},
		{
			name:    "interval-is-negative",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{Interval: &durationpb.Duration{Seconds: -10}}),
			wantErr: true,
		},
		{
			name:    "interval-overflows",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{Interval: &durationpb.Duration{Seconds: 315576000001}}),
			wantErr: true,
		},
		{
			name:    "base-ejection-time-is-negative",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{BaseEjectionTime: &durationpb.Duration{Seconds: -10}}),
			wantErr: true,
		},
		{
			name:    "base-ejection-time-overflows",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{BaseEjectionTime: &durationpb.Duration{Seconds: 315576000001}}),
			wantErr: true,
		},
		{
			name:    "max-ejection-time-is-negative",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{MaxEjectionTime: &durationpb.Duration{Seconds: -10}}),
			wantErr: true,
		},
		{
			name:    "max-ejection-time-overflows",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{MaxEjectionTime: &durationpb.Duration{Seconds: 315576000001}}),
			wantErr: true,
		},
		{
			name:    "max-ejection-percent-is-greater-than-100",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{MaxEjectionPercent: &wrapperspb.UInt32Value{Value: 150}}),
			wantErr: true,
		},
		{
			name:    "enforcing-success-rate-is-greater-than-100",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{EnforcingSuccessRate: &wrapperspb.UInt32Value{Value: 150}}),
			wantErr: true,
		},
		{
			name:    "failure-percentage-threshold-is-greater-than-100",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{FailurePercentageThreshold: &wrapperspb.UInt32Value{Value: 150}}),
			wantErr: true,
		},
		{
			name:    "enforcing-failure-percentage-is-greater-than-100",
			cluster: odToClusterProto(&v3clusterpb.OutlierDetection{EnforcingFailurePercentage: &wrapperspb.UInt32Value{Value: 150}}),
			wantErr: true,
		},
		// A Outlier Detection proto not present should lead to a nil
		// OutlierDetection field in the ClusterUpdate, which is implicitly
		// tested in every other test in this file.
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			update, err := validateClusterAndConstructClusterUpdate(test.cluster, nil)
			if (err != nil) != test.wantErr {
				t.Errorf("validateClusterAndConstructClusterUpdate() returned err %v wantErr %v)", err, test.wantErr)
			}
			if test.wantErr {
				return
			}
			// got and want must be unmarshalled since JSON strings shouldn't
			// generally be directly compared.
			var got map[string]any
			if err := json.Unmarshal(update.OutlierDetection, &got); err != nil {
				t.Fatalf("Error unmarshalling update.OutlierDetection (%q): %v", update.OutlierDetection, err)
			}
			var want map[string]any
			if err := json.Unmarshal(json.RawMessage(test.wantODCfg), &want); err != nil {
				t.Fatalf("Error unmarshalling wantODCfg (%q): %v", test.wantODCfg, err)
			}
			if diff := cmp.Diff(got, want); diff != "" {
				t.Fatalf("cluster.OutlierDetection got unexpected output, diff (-got, +want): %v", diff)
			}
		})
	}
}
