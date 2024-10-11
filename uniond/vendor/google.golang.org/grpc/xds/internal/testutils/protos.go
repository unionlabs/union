/*
 *
 * Copyright 2020 gRPC authors.
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

package testutils

import (
	"net"
	"strconv"

	v2xdspb "github.com/envoyproxy/go-control-plane/envoy/api/v2"
	v2corepb "github.com/envoyproxy/go-control-plane/envoy/api/v2/core"
	v2endpointpb "github.com/envoyproxy/go-control-plane/envoy/api/v2/endpoint"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v2typepb "github.com/envoyproxy/go-control-plane/envoy/type"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

// EmptyNodeProtoV2 is a v2 Node proto with no fields set.
var EmptyNodeProtoV2 = &v2corepb.Node{}

// EmptyNodeProtoV3 is a v3 Node proto with no fields set.
var EmptyNodeProtoV3 = &v3corepb.Node{}

// LocalityIDToProto converts a LocalityID to its proto representation.
func LocalityIDToProto(l internal.LocalityID) *v2corepb.Locality {
	return &v2corepb.Locality{
		Region:  l.Region,
		Zone:    l.Zone,
		SubZone: l.SubZone,
	}
}

// The helper structs/functions related to EDS protos are used in EDS balancer
// tests now, to generate test inputs. Eventually, EDS balancer tests should
// generate EndpointsUpdate directly, instead of generating and parsing the
// proto message.
// TODO: Once EDS balancer tests don't use these, these can be moved to v2 client code.

// ClusterLoadAssignmentBuilder builds a ClusterLoadAssignment, aka EDS
// response.
type ClusterLoadAssignmentBuilder struct {
	v *v2xdspb.ClusterLoadAssignment
}

// NewClusterLoadAssignmentBuilder creates a ClusterLoadAssignmentBuilder.
func NewClusterLoadAssignmentBuilder(clusterName string, dropPercents map[string]uint32) *ClusterLoadAssignmentBuilder {
	drops := make([]*v2xdspb.ClusterLoadAssignment_Policy_DropOverload, 0, len(dropPercents))
	for n, d := range dropPercents {
		drops = append(drops, &v2xdspb.ClusterLoadAssignment_Policy_DropOverload{
			Category: n,
			DropPercentage: &v2typepb.FractionalPercent{
				Numerator:   d,
				Denominator: v2typepb.FractionalPercent_HUNDRED,
			},
		})
	}

	return &ClusterLoadAssignmentBuilder{
		v: &v2xdspb.ClusterLoadAssignment{
			ClusterName: clusterName,
			Policy: &v2xdspb.ClusterLoadAssignment_Policy{
				DropOverloads: drops,
			},
		},
	}
}

// AddLocalityOptions contains options when adding locality to the builder.
type AddLocalityOptions struct {
	Health []v2corepb.HealthStatus
	Weight []uint32
}

// AddLocality adds a locality to the builder.
func (clab *ClusterLoadAssignmentBuilder) AddLocality(subzone string, weight uint32, priority uint32, addrsWithPort []string, opts *AddLocalityOptions) {
	lbEndPoints := make([]*v2endpointpb.LbEndpoint, 0, len(addrsWithPort))
	for i, a := range addrsWithPort {
		host, portStr, err := net.SplitHostPort(a)
		if err != nil {
			panic("failed to split " + a)
		}
		port, err := strconv.Atoi(portStr)
		if err != nil {
			panic("failed to atoi " + portStr)
		}

		lbe := &v2endpointpb.LbEndpoint{
			HostIdentifier: &v2endpointpb.LbEndpoint_Endpoint{
				Endpoint: &v2endpointpb.Endpoint{
					Address: &v2corepb.Address{
						Address: &v2corepb.Address_SocketAddress{
							SocketAddress: &v2corepb.SocketAddress{
								Protocol: v2corepb.SocketAddress_TCP,
								Address:  host,
								PortSpecifier: &v2corepb.SocketAddress_PortValue{
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

	var localityID *v2corepb.Locality
	if subzone != "" {
		localityID = &v2corepb.Locality{
			Region:  "",
			Zone:    "",
			SubZone: subzone,
		}
	}

	clab.v.Endpoints = append(clab.v.Endpoints, &v2endpointpb.LocalityLbEndpoints{
		Locality:            localityID,
		LbEndpoints:         lbEndPoints,
		LoadBalancingWeight: &wrapperspb.UInt32Value{Value: weight},
		Priority:            priority,
	})
}

// Build builds ClusterLoadAssignment.
func (clab *ClusterLoadAssignmentBuilder) Build() *v2xdspb.ClusterLoadAssignment {
	return clab.v
}
