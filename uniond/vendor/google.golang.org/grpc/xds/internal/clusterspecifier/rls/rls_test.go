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

package rls

import (
	"encoding/json"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/internal/grpctest"
	rlspb "google.golang.org/grpc/internal/proto/grpc_lookup_v1"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/xds/internal/clusterspecifier"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"

	_ "google.golang.org/grpc/balancer/rls"                      // Register the RLS LB policy.
	_ "google.golang.org/grpc/xds/internal/balancer/cdsbalancer" // Register the CDS LB policy.
)

func init() {
	clusterspecifier.Register(rls{})
}

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// TestParseClusterSpecifierConfig tests the parsing functionality of the RLS
// Cluster Specifier Plugin.
func (s) TestParseClusterSpecifierConfig(t *testing.T) {
	tests := []struct {
		name       string
		rlcs       proto.Message
		wantConfig clusterspecifier.BalancerConfig
		wantErr    bool
	}{
		{

			// This will error because the required_match field is set in grpc key builder
			name: "invalid-rls-cluster-specifier",
			rlcs: testutils.MarshalAny(t, &rlspb.RouteLookupClusterSpecifier{
				RouteLookupConfig: &rlspb.RouteLookupConfig{
					GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
						{
							Names: []*rlspb.GrpcKeyBuilder_Name{
								{
									Service: "service",
									Method:  "method",
								},
							},
							Headers: []*rlspb.NameMatcher{
								{
									Key:           "k1",
									RequiredMatch: true,
									Names:         []string{"v1"},
								},
							},
						},
					},
				},
			}),
			wantErr: true,
		},
		{
			name: "valid-rls-cluster-specifier",
			rlcs: testutils.MarshalAny(t, &rlspb.RouteLookupClusterSpecifier{
				RouteLookupConfig: &rlspb.RouteLookupConfig{
					GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
						{
							Names: []*rlspb.GrpcKeyBuilder_Name{
								{
									Service: "service",
									Method:  "method",
								},
							},
							Headers: []*rlspb.NameMatcher{
								{
									Key:   "k1",
									Names: []string{"v1"},
								},
							},
						},
					},
					LookupService:        "target",
					LookupServiceTimeout: &durationpb.Duration{Seconds: 100},
					MaxAge:               &durationpb.Duration{Seconds: 60},
					StaleAge:             &durationpb.Duration{Seconds: 50},
					CacheSizeBytes:       1000,
					DefaultTarget:        "passthrough:///default",
				},
			}),
			wantConfig: configWithoutTransformationsWant,
		},
	}
	for _, test := range tests {
		cs := clusterspecifier.Get("type.googleapis.com/grpc.lookup.v1.RouteLookupClusterSpecifier")
		if cs == nil {
			t.Fatal("Error getting cluster specifier")
		}
		lbCfg, err := cs.ParseClusterSpecifierConfig(test.rlcs)

		if (err != nil) != test.wantErr {
			t.Fatalf("ParseClusterSpecifierConfig(%+v) returned err: %v, wantErr: %v", test.rlcs, err, test.wantErr)
		}
		if test.wantErr { // Successfully received an error.
			return
		}
		// Marshal and then unmarshal into any to get rid of nondeterministic
		// protojson Marshaling.
		lbCfgJSON, err := json.Marshal(lbCfg)
		if err != nil {
			t.Fatalf("json.Marshal(%+v) returned err %v", lbCfg, err)
		}
		var got any
		err = json.Unmarshal(lbCfgJSON, got)
		if err != nil {
			t.Fatalf("json.Unmarshal(%+v) returned err %v", lbCfgJSON, err)
		}
		wantCfgJSON, err := json.Marshal(test.wantConfig)
		if err != nil {
			t.Fatalf("json.Marshal(%+v) returned err %v", test.wantConfig, err)
		}
		var want any
		err = json.Unmarshal(wantCfgJSON, want)
		if err != nil {
			t.Fatalf("json.Unmarshal(%+v) returned err %v", lbCfgJSON, err)
		}
		if diff := cmp.Diff(want, got, cmpopts.EquateEmpty()); diff != "" {
			t.Fatalf("ParseClusterSpecifierConfig(%+v) returned expected, diff (-want +got) %v", test.rlcs, diff)
		}
	}
}

var configWithoutTransformationsWant = clusterspecifier.BalancerConfig{{"rls_experimental": &lbConfigJSON{
	RouteLookupConfig: []byte(`{"grpcKeybuilders":[{"names":[{"service":"service","method":"method"}],"headers":[{"key":"k1","names":["v1"]}]}],"lookupService":"target","lookupServiceTimeout":"100s","maxAge":"60s","staleAge":"50s","cacheSizeBytes":"1000","defaultTarget":"passthrough:///default"}`),
	ChildPolicy: []map[string]json.RawMessage{
		{
			"cds_experimental": []byte(`{}`),
		},
	},
	ChildPolicyConfigTargetFieldName: "cluster",
}}}
