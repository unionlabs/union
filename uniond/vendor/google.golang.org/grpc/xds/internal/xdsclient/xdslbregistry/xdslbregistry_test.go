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

// Package xdslbregistry_test contains test cases for the xDS LB Policy Registry.
package xdslbregistry_test

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	_ "google.golang.org/grpc/balancer/roundrobin"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/pretty"
	internalserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/internal/testutils"
	_ "google.golang.org/grpc/xds" // Register the xDS LB Registry Converters.
	"google.golang.org/grpc/xds/internal/balancer/wrrlocality"
	"google.golang.org/grpc/xds/internal/xdsclient/xdslbregistry"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v1xdsudpatypepb "github.com/cncf/xds/go/udpa/type/v1"
	v3xdsxdstypepb "github.com/cncf/xds/go/xds/type/v3"
	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3leastrequestpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/least_request/v3"
	v3pickfirstpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/pick_first/v3"
	v3ringhashpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/ring_hash/v3"
	v3roundrobinpb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/round_robin/v3"
	v3wrrlocalitypb "github.com/envoyproxy/go-control-plane/envoy/extensions/load_balancing_policies/wrr_locality/v3"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func wrrLocalityBalancerConfig(childPolicy *internalserviceconfig.BalancerConfig) *internalserviceconfig.BalancerConfig {
	return &internalserviceconfig.BalancerConfig{
		Name: wrrlocality.Name,
		Config: &wrrlocality.LBConfig{
			ChildPolicy: childPolicy,
		},
	}
}

func (s) TestConvertToServiceConfigSuccess(t *testing.T) {
	defer func(old bool) { envconfig.LeastRequestLB = old }(envconfig.LeastRequestLB)
	envconfig.LeastRequestLB = false

	const customLBPolicyName = "myorg.MyCustomLeastRequestPolicy"
	stub.Register(customLBPolicyName, stub.BalancerFuncs{})

	tests := []struct {
		name       string
		policy     *v3clusterpb.LoadBalancingPolicy
		wantConfig string // JSON config
		lrEnabled  bool
	}{
		{
			name: "ring_hash",
			policy: &v3clusterpb.LoadBalancingPolicy{
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
			wantConfig: `[{"ring_hash_experimental": { "minRingSize": 10, "maxRingSize": 100 }}]`,
		},
		{
			name: "least_request",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3leastrequestpb.LeastRequest{
								ChoiceCount: wrapperspb.UInt32(3),
							}),
						},
					},
				},
			},
			wantConfig: `[{"least_request_experimental": { "choiceCount": 3 }}]`,
			lrEnabled:  true,
		},
		{
			name: "pick_first_shuffle",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3pickfirstpb.PickFirst{
								ShuffleAddressList: true,
							}),
						},
					},
				},
			},
			wantConfig: `[{"pick_first": { "shuffleAddressList": true }}]`,
		},
		{
			name: "pick_first",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3pickfirstpb.PickFirst{}),
						},
					},
				},
			},
			wantConfig: `[{"pick_first": { "shuffleAddressList": false }}]`,
		},
		{
			name: "round_robin",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3roundrobinpb.RoundRobin{}),
						},
					},
				},
			},
			wantConfig: `[{"round_robin": {}}]`,
		},
		{
			name: "round_robin_ring_hash_use_first_supported",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3roundrobinpb.RoundRobin{}),
						},
					},
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
			wantConfig: `[{"round_robin": {}}]`,
		},
		{
			name: "pf_rr_use_pick_first",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3pickfirstpb.PickFirst{
								ShuffleAddressList: true,
							}),
						},
					},
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3roundrobinpb.RoundRobin{}),
						},
					},
				},
			},
			wantConfig: `[{"pick_first": { "shuffleAddressList": true }}]`,
		},
		{
			name: "least_request_disabled_pf_rr_use_first_supported",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3leastrequestpb.LeastRequest{
								ChoiceCount: wrapperspb.UInt32(32),
							}),
						},
					},
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3roundrobinpb.RoundRobin{}),
						},
					},
				},
			},
			wantConfig: `[{"round_robin": {}}]`,
		},
		{
			name: "custom_lb_type_v3_struct",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							// The type not registered in gRPC Policy registry.
							// Should fallback to next policy in list.
							TypedConfig: testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
								TypeUrl: "type.googleapis.com/myorg.ThisTypeDoesNotExist",
								Value:   &structpb.Struct{},
							}),
						},
					},
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
								TypeUrl: "type.googleapis.com/myorg.MyCustomLeastRequestPolicy",
								Value:   &structpb.Struct{},
							}),
						},
					},
				},
			},
			wantConfig: `[{"myorg.MyCustomLeastRequestPolicy": {}}]`,
		},
		{
			name: "custom_lb_type_v1_struct",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v1xdsudpatypepb.TypedStruct{
								TypeUrl: "type.googleapis.com/myorg.MyCustomLeastRequestPolicy",
								Value:   &structpb.Struct{},
							}),
						},
					},
				},
			},
			wantConfig: `[{"myorg.MyCustomLeastRequestPolicy": {}}]`,
		},
		{
			name: "wrr_locality_child_round_robin",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: wrrLocalityAny(t, &v3roundrobinpb.RoundRobin{}),
						},
					},
				},
			},
			wantConfig: `[{"xds_wrr_locality_experimental": { "childPolicy": [{"round_robin": {}}] }}]`,
		},
		{
			name: "wrr_locality_child_custom_lb_type_v3_struct",
			policy: &v3clusterpb.LoadBalancingPolicy{
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
			wantConfig: `[{"xds_wrr_locality_experimental": { "childPolicy": [{"myorg.MyCustomLeastRequestPolicy": {}}] }}]`,
		},
		{
			name: "on-the-boundary-of-recursive-limit",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: wrrLocalityAny(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, &v3roundrobinpb.RoundRobin{}))))))))))))))),
						},
					},
				},
			},
			wantConfig: jsonMarshal(t, wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(wrrLocalityBalancerConfig(&internalserviceconfig.BalancerConfig{
				Name: "round_robin",
			})))))))))))))))),
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if test.lrEnabled {
				defer func(old bool) { envconfig.LeastRequestLB = old }(envconfig.LeastRequestLB)
				envconfig.LeastRequestLB = true
			}
			rawJSON, err := xdslbregistry.ConvertToServiceConfig(test.policy, 0)
			if err != nil {
				t.Fatalf("ConvertToServiceConfig(%s) failed: %v", pretty.ToJSON(test.policy), err)
			}
			// got and want must be unmarshalled since JSON strings shouldn't
			// generally be directly compared.
			var got []map[string]any
			if err := json.Unmarshal(rawJSON, &got); err != nil {
				t.Fatalf("Error unmarshalling rawJSON (%q): %v", rawJSON, err)
			}
			var want []map[string]any
			if err := json.Unmarshal(json.RawMessage(test.wantConfig), &want); err != nil {
				t.Fatalf("Error unmarshalling wantConfig (%q): %v", test.wantConfig, err)
			}
			if diff := cmp.Diff(got, want); diff != "" {
				t.Fatalf("ConvertToServiceConfig() got unexpected output, diff (-got +want): %v", diff)
			}
		})
	}
}

func jsonMarshal(t *testing.T, x any) string {
	t.Helper()
	js, err := json.Marshal(x)
	if err != nil {
		t.Fatalf("Error marshalling to JSON (%+v): %v", x, err)
	}
	return string(js)
}

// TestConvertToServiceConfigFailure tests failure cases of the xDS LB registry
// of converting proto configuration to JSON configuration.
func (s) TestConvertToServiceConfigFailure(t *testing.T) {
	tests := []struct {
		name    string
		policy  *v3clusterpb.LoadBalancingPolicy
		wantErr string
	}{
		{
			name: "not xx_hash function",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: testutils.MarshalAny(t, &v3ringhashpb.RingHash{
								HashFunction:    v3ringhashpb.RingHash_MURMUR_HASH_2,
								MinimumRingSize: wrapperspb.UInt64(10),
								MaximumRingSize: wrapperspb.UInt64(100),
							}),
						},
					},
				},
			},
			wantErr: "unsupported ring_hash hash function",
		},
		{
			name: "no-supported-policy",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							// The type not registered in gRPC Policy registry.
							TypedConfig: testutils.MarshalAny(t, &v3xdsxdstypepb.TypedStruct{
								TypeUrl: "type.googleapis.com/myorg.ThisTypeDoesNotExist",
								Value:   &structpb.Struct{},
							}),
						},
					},
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							// Not supported by gRPC-Go.
							TypedConfig: testutils.MarshalAny(t, &v3leastrequestpb.LeastRequest{}),
						},
					},
				},
			},
			wantErr: "no supported policy found in policy list",
		},
		{
			name: "exceeds-boundary-of-recursive-limit-by-1",
			policy: &v3clusterpb.LoadBalancingPolicy{
				Policies: []*v3clusterpb.LoadBalancingPolicy_Policy{
					{
						TypedExtensionConfig: &v3corepb.TypedExtensionConfig{
							TypedConfig: wrrLocalityAny(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, wrrLocality(t, &v3roundrobinpb.RoundRobin{})))))))))))))))),
						},
					},
				},
			},
			wantErr: "exceeds max depth",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			_, gotErr := xdslbregistry.ConvertToServiceConfig(test.policy, 0)
			// Test the error substring to test the different root causes of
			// errors. This is more brittle over time, but it's important to
			// test the root cause of the errors emitted from the
			// ConvertToServiceConfig function call. Also, this package owns the
			// error strings so breakages won't come unexpectedly.
			if gotErr == nil || !strings.Contains(gotErr.Error(), test.wantErr) {
				t.Fatalf("ConvertToServiceConfig() = %v, wantErr %v", gotErr, test.wantErr)
			}
		})
	}
}

// wrrLocality is a helper that takes a proto message and returns a
// WrrLocalityProto with the proto message marshaled into a proto.Any as a
// child.
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

// wrrLocalityAny takes a proto message and returns a wrr locality proto
// marshaled as an any with an any child set to the marshaled proto message.
func wrrLocalityAny(t *testing.T, m proto.Message) *anypb.Any {
	return testutils.MarshalAny(t, wrrLocality(t, m))
}
