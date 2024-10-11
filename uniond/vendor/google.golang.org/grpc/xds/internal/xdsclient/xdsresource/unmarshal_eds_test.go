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
	"net"
	"strconv"
	"testing"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3typepb "github.com/envoyproxy/go-control-plane/envoy/type/v3"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

func (s) TestEDSParseRespProto(t *testing.T) {
	tests := []struct {
		name    string
		m       *v3endpointpb.ClusterLoadAssignment
		want    EndpointsUpdate
		wantErr bool
	}{
		{
			name: "missing-priority",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 1, 0, []string{"addr1:314"}, nil)
				clab0.addLocality("locality-2", 1, 2, []string{"addr2:159"}, nil)
				return clab0.Build()
			}(),
			want:    EndpointsUpdate{},
			wantErr: true,
		},
		{
			name: "missing-locality-ID",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("", 1, 0, []string{"addr1:314"}, nil)
				return clab0.Build()
			}(),
			want:    EndpointsUpdate{},
			wantErr: true,
		},
		{
			name: "zero-endpoint-weight",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-0", 1, 0, []string{"addr1:314"}, &addLocalityOptions{Weight: []uint32{0}})
				return clab0.Build()
			}(),
			want:    EndpointsUpdate{},
			wantErr: true,
		},
		{
			name: "duplicate-locality-in-the-same-priority",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-0", 1, 0, []string{"addr1:314"}, nil)
				clab0.addLocality("locality-0", 1, 0, []string{"addr1:314"}, nil) // Duplicate locality with the same priority.
				return clab0.Build()
			}(),
			want:    EndpointsUpdate{},
			wantErr: true,
		},
		{
			name: "missing locality weight",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 0, 1, []string{"addr1:314"}, &addLocalityOptions{
					Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_HEALTHY},
				})
				clab0.addLocality("locality-2", 0, 0, []string{"addr2:159"}, &addLocalityOptions{
					Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_HEALTHY},
				})
				return clab0.Build()
			}(),
			want: EndpointsUpdate{},
		},
		{
			name: "max sum of weights at the same priority exceeded",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 1, 0, []string{"addr1:314"}, nil)
				clab0.addLocality("locality-2", 4294967295, 1, []string{"addr2:159"}, nil)
				clab0.addLocality("locality-3", 1, 1, []string{"addr2:88"}, nil)
				return clab0.Build()
			}(),
			want:    EndpointsUpdate{},
			wantErr: true,
		},
		{
			name: "duplicate endpoint address",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 1, 1, []string{"addr:997"}, nil)
				clab0.addLocality("locality-2", 1, 0, []string{"addr:997"}, nil)
				return clab0.Build()
			}(),
			want:    EndpointsUpdate{},
			wantErr: true,
		},
		{
			name: "good",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 1, 1, []string{"addr1:314"}, &addLocalityOptions{
					Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_UNHEALTHY},
					Weight: []uint32{271},
				})
				clab0.addLocality("locality-2", 1, 0, []string{"addr2:159"}, &addLocalityOptions{
					Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_DRAINING},
					Weight: []uint32{828},
				})
				return clab0.Build()
			}(),
			want: EndpointsUpdate{
				Drops: nil,
				Localities: []Locality{
					{
						Endpoints: []Endpoint{{
							Address:      "addr1:314",
							HealthStatus: EndpointHealthStatusUnhealthy,
							Weight:       271,
						}},
						ID:       internal.LocalityID{SubZone: "locality-1"},
						Priority: 1,
						Weight:   1,
					},
					{
						Endpoints: []Endpoint{{
							Address:      "addr2:159",
							HealthStatus: EndpointHealthStatusDraining,
							Weight:       828,
						}},
						ID:       internal.LocalityID{SubZone: "locality-2"},
						Priority: 0,
						Weight:   1,
					},
				},
			},
			wantErr: false,
		},
		{
			name: "good duplicate locality with different priority",
			m: func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 1, 1, []string{"addr1:314"}, &addLocalityOptions{
					Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_UNHEALTHY},
					Weight: []uint32{271},
				})
				// Same locality name, but with different priority.
				clab0.addLocality("locality-1", 1, 0, []string{"addr2:159"}, &addLocalityOptions{
					Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_DRAINING},
					Weight: []uint32{828},
				})
				return clab0.Build()
			}(),
			want: EndpointsUpdate{
				Drops: nil,
				Localities: []Locality{
					{
						Endpoints: []Endpoint{{
							Address:      "addr1:314",
							HealthStatus: EndpointHealthStatusUnhealthy,
							Weight:       271,
						}},
						ID:       internal.LocalityID{SubZone: "locality-1"},
						Priority: 1,
						Weight:   1,
					},
					{
						Endpoints: []Endpoint{{
							Address:      "addr2:159",
							HealthStatus: EndpointHealthStatusDraining,
							Weight:       828,
						}},
						ID:       internal.LocalityID{SubZone: "locality-1"},
						Priority: 0,
						Weight:   1,
					},
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := parseEDSRespProto(tt.m)
			if (err != nil) != tt.wantErr {
				t.Errorf("parseEDSRespProto() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if d := cmp.Diff(got, tt.want); d != "" {
				t.Errorf("parseEDSRespProto() got = %v, want %v, diff: %v", got, tt.want, d)
			}
		})
	}
}

func (s) TestUnmarshalEndpoints(t *testing.T) {
	var v3EndpointsAny = testutils.MarshalAny(t, func() *v3endpointpb.ClusterLoadAssignment {
		clab0 := newClaBuilder("test", nil)
		clab0.addLocality("locality-1", 1, 1, []string{"addr1:314"}, &addLocalityOptions{
			Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_UNHEALTHY},
			Weight: []uint32{271},
		})
		clab0.addLocality("locality-2", 1, 0, []string{"addr2:159"}, &addLocalityOptions{
			Health: []v3corepb.HealthStatus{v3corepb.HealthStatus_DRAINING},
			Weight: []uint32{828},
		})
		return clab0.Build()
	}())

	tests := []struct {
		name       string
		resource   *anypb.Any
		wantName   string
		wantUpdate EndpointsUpdate
		wantErr    bool
	}{
		{
			name:     "non-clusterLoadAssignment resource type",
			resource: &anypb.Any{TypeUrl: version.V3HTTPConnManagerURL},
			wantErr:  true,
		},
		{
			name: "badly marshaled clusterLoadAssignment resource",
			resource: &anypb.Any{
				TypeUrl: version.V3EndpointsURL,
				Value:   []byte{1, 2, 3, 4},
			},
			wantErr: true,
		},
		{
			name: "bad endpoints resource",
			resource: testutils.MarshalAny(t, func() *v3endpointpb.ClusterLoadAssignment {
				clab0 := newClaBuilder("test", nil)
				clab0.addLocality("locality-1", 1, 0, []string{"addr1:314"}, nil)
				clab0.addLocality("locality-2", 1, 2, []string{"addr2:159"}, nil)
				return clab0.Build()
			}()),
			wantName: "test",
			wantErr:  true,
		},
		{
			name:     "v3 endpoints",
			resource: v3EndpointsAny,
			wantName: "test",
			wantUpdate: EndpointsUpdate{
				Drops: nil,
				Localities: []Locality{
					{
						Endpoints: []Endpoint{{
							Address:      "addr1:314",
							HealthStatus: EndpointHealthStatusUnhealthy,
							Weight:       271,
						}},
						ID:       internal.LocalityID{SubZone: "locality-1"},
						Priority: 1,
						Weight:   1,
					},
					{
						Endpoints: []Endpoint{{
							Address:      "addr2:159",
							HealthStatus: EndpointHealthStatusDraining,
							Weight:       828,
						}},
						ID:       internal.LocalityID{SubZone: "locality-2"},
						Priority: 0,
						Weight:   1,
					},
				},
				Raw: v3EndpointsAny,
			},
		},
		{
			name:     "v3 endpoints wrapped",
			resource: testutils.MarshalAny(t, &v3discoverypb.Resource{Resource: v3EndpointsAny}),
			wantName: "test",
			wantUpdate: EndpointsUpdate{
				Drops: nil,
				Localities: []Locality{
					{
						Endpoints: []Endpoint{{
							Address:      "addr1:314",
							HealthStatus: EndpointHealthStatusUnhealthy,
							Weight:       271,
						}},
						ID:       internal.LocalityID{SubZone: "locality-1"},
						Priority: 1,
						Weight:   1,
					},
					{
						Endpoints: []Endpoint{{
							Address:      "addr2:159",
							HealthStatus: EndpointHealthStatusDraining,
							Weight:       828,
						}},
						ID:       internal.LocalityID{SubZone: "locality-2"},
						Priority: 0,
						Weight:   1,
					},
				},
				Raw: v3EndpointsAny,
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			name, update, err := unmarshalEndpointsResource(test.resource)
			if (err != nil) != test.wantErr {
				t.Fatalf("unmarshalEndpointsResource(%s), got err: %v, wantErr: %v", pretty.ToJSON(test.resource), err, test.wantErr)
			}
			if name != test.wantName {
				t.Errorf("unmarshalEndpointsResource(%s), got name: %s, want: %s", pretty.ToJSON(test.resource), name, test.wantName)
			}
			if diff := cmp.Diff(update, test.wantUpdate, cmpOpts); diff != "" {
				t.Errorf("unmarshalEndpointsResource(%s), got unexpected update, diff (-got +want): %v", pretty.ToJSON(test.resource), diff)
			}
		})
	}
}

// claBuilder builds a ClusterLoadAssignment, aka EDS
// response.
type claBuilder struct {
	v *v3endpointpb.ClusterLoadAssignment
}

// newClaBuilder creates a claBuilder.
func newClaBuilder(clusterName string, dropPercents []uint32) *claBuilder {
	var drops []*v3endpointpb.ClusterLoadAssignment_Policy_DropOverload
	for i, d := range dropPercents {
		drops = append(drops, &v3endpointpb.ClusterLoadAssignment_Policy_DropOverload{
			Category: fmt.Sprintf("test-drop-%d", i),
			DropPercentage: &v3typepb.FractionalPercent{
				Numerator:   d,
				Denominator: v3typepb.FractionalPercent_HUNDRED,
			},
		})
	}

	return &claBuilder{
		v: &v3endpointpb.ClusterLoadAssignment{
			ClusterName: clusterName,
			Policy: &v3endpointpb.ClusterLoadAssignment_Policy{
				DropOverloads: drops,
			},
		},
	}
}

// addLocalityOptions contains options when adding locality to the builder.
type addLocalityOptions struct {
	Health []v3corepb.HealthStatus
	Weight []uint32
}

// addLocality adds a locality to the builder.
func (clab *claBuilder) addLocality(subzone string, weight uint32, priority uint32, addrsWithPort []string, opts *addLocalityOptions) {
	var lbEndPoints []*v3endpointpb.LbEndpoint
	for i, a := range addrsWithPort {
		host, portStr, err := net.SplitHostPort(a)
		if err != nil {
			panic("failed to split " + a)
		}
		port, err := strconv.Atoi(portStr)
		if err != nil {
			panic("failed to atoi " + portStr)
		}

		lbe := &v3endpointpb.LbEndpoint{
			HostIdentifier: &v3endpointpb.LbEndpoint_Endpoint{
				Endpoint: &v3endpointpb.Endpoint{
					Address: &v3corepb.Address{
						Address: &v3corepb.Address_SocketAddress{
							SocketAddress: &v3corepb.SocketAddress{
								Protocol: v3corepb.SocketAddress_TCP,
								Address:  host,
								PortSpecifier: &v3corepb.SocketAddress_PortValue{
									PortValue: uint32(port)}}}}}},
		}
		if opts != nil {
			if i < len(opts.Health) {
				lbe.HealthStatus = opts.Health[i]
			}
			if i < len(opts.Weight) {
				lbe.LoadBalancingWeight = &wrapperspb.UInt32Value{Value: opts.Weight[i]}
			}
		}
		lbEndPoints = append(lbEndPoints, lbe)
	}

	var localityID *v3corepb.Locality
	if subzone != "" {
		localityID = &v3corepb.Locality{
			Region:  "",
			Zone:    "",
			SubZone: subzone,
		}
	}

	clab.v.Endpoints = append(clab.v.Endpoints, &v3endpointpb.LocalityLbEndpoints{
		Locality:            localityID,
		LbEndpoints:         lbEndPoints,
		LoadBalancingWeight: &wrapperspb.UInt32Value{Value: weight},
		Priority:            priority,
	})
}

// Build builds ClusterLoadAssignment.
func (clab *claBuilder) Build() *v3endpointpb.ClusterLoadAssignment {
	return clab.v
}
