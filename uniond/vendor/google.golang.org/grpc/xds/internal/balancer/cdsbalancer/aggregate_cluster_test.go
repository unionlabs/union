/*
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

package cdsbalancer

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/balancer/clusterresolver"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// makeAggregateClusterResource returns an aggregate cluster resource with the
// given name and list of child names.
func makeAggregateClusterResource(name string, childNames []string) *v3clusterpb.Cluster {
	return e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: name,
		Type:        e2e.ClusterTypeAggregate,
		ChildNames:  childNames,
	})
}

// makeLogicalDNSClusterResource returns a LOGICAL_DNS cluster resource with the
// given name and given DNS host and port.
func makeLogicalDNSClusterResource(name, dnsHost string, dnsPort uint32) *v3clusterpb.Cluster {
	return e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: name,
		Type:        e2e.ClusterTypeLogicalDNS,
		DNSHostName: dnsHost,
		DNSPort:     dnsPort,
	})
}

// Tests the case where the cluster resource requested by the cds LB policy is a
// leaf cluster. The management server sends two updates for the same leaf
// cluster resource. The test verifies that the load balancing configuration
// pushed to the cluster_resolver LB policy is contains the expected discovery
// mechanism corresponding to the leaf cluster, on both occasions.
func (s) TestAggregateClusterSuccess_LeafNode(t *testing.T) {
	tests := []struct {
		name                  string
		firstClusterResource  *v3clusterpb.Cluster
		secondClusterResource *v3clusterpb.Cluster
		wantFirstChildCfg     serviceconfig.LoadBalancingConfig
		wantSecondChildCfg    serviceconfig.LoadBalancingConfig
	}{
		{
			name:                  "eds",
			firstClusterResource:  e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone),
			secondClusterResource: e2e.DefaultCluster(clusterName, serviceName+"-new", e2e.SecurityLevelNone),
			wantFirstChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:          clusterName,
					Type:             clusterresolver.DiscoveryMechanismTypeEDS,
					EDSServiceName:   serviceName,
					OutlierDetection: json.RawMessage(`{}`),
					TelemetryLabels:  internal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
			},
			wantSecondChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:          clusterName,
					Type:             clusterresolver.DiscoveryMechanismTypeEDS,
					EDSServiceName:   serviceName + "-new",
					OutlierDetection: json.RawMessage(`{}`),
					TelemetryLabels:  internal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
			},
		},
		{
			name:                  "dns",
			firstClusterResource:  makeLogicalDNSClusterResource(clusterName, "dns_host", uint32(8080)),
			secondClusterResource: makeLogicalDNSClusterResource(clusterName, "dns_host_new", uint32(8080)),
			wantFirstChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:          clusterName,
					Type:             clusterresolver.DiscoveryMechanismTypeLogicalDNS,
					DNSHostname:      "dns_host:8080",
					OutlierDetection: json.RawMessage(`{}`),
					TelemetryLabels:  internal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
			},
			wantSecondChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:          clusterName,
					Type:             clusterresolver.DiscoveryMechanismTypeLogicalDNS,
					DNSHostname:      "dns_host_new:8080",
					OutlierDetection: json.RawMessage(`{}`),
					TelemetryLabels:  internal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
			mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

			// Push the first cluster resource through the management server and
			// verify the configuration pushed to the child policy.
			resources := e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.firstClusterResource},
				SkipValidation: true,
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}
			if err := compareLoadBalancingConfig(ctx, lbCfgCh, test.wantFirstChildCfg); err != nil {
				t.Fatal(err)
			}

			// Push the second cluster resource through the management server and
			// verify the configuration pushed to the child policy.
			resources.Clusters[0] = test.secondClusterResource
			if err := mgmtServer.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}
			if err := compareLoadBalancingConfig(ctx, lbCfgCh, test.wantSecondChildCfg); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// Tests the case where the cluster resource requested by the cds LB policy is
// an aggregate cluster root pointing to two child clusters, one of type EDS and
// the other of type LogicalDNS. The test verifies that load balancing
// configuration is pushed to the cluster_resolver LB policy only when all child
// clusters are resolved and it also verifies that the pushed configuration
// contains the expected discovery mechanisms. The test then updates the
// aggregate cluster to point to two child clusters, the same leaf cluster of
// type EDS and a different leaf cluster of type LogicalDNS and verifies that
// the load balancing configuration pushed to the cluster_resolver LB policy
// contains the expected discovery mechanisms.
func (s) TestAggregateClusterSuccess_ThenUpdateChildClusters(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

	// Configure the management server with the aggregate cluster resource
	// pointing to two child clusters, one EDS and one LogicalDNS. Include the
	// resource corresponding to the EDS cluster here, but don't include
	// resource corresponding to the LogicalDNS cluster yet.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, serviceName, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that no configuration is pushed to the child policy yet, because
	// not all clusters making up the aggregate cluster have been resolved yet.
	select {
	case cfg := <-lbCfgCh:
		t.Fatalf("Child policy received configuration when not expected to: %s", pretty.ToJSON(cfg))
	case <-time.After(defaultTestShortTimeout):
	}

	// Now configure the LogicalDNS cluster in the management server. This
	// should result in configuration being pushed down to the child policy.
	resources.Clusters = append(resources.Clusters, makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort))
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{
			{
				Cluster:          edsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeEDS,
				EDSServiceName:   serviceName,
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
			{
				Cluster:          dnsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeLogicalDNS,
				DNSHostname:      fmt.Sprintf("%s:%d", dnsHostName, dnsPort),
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
		},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}

	const dnsClusterNameNew = dnsClusterName + "-new"
	const dnsHostNameNew = dnsHostName + "-new"
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterNameNew}),
			e2e.DefaultCluster(edsClusterName, serviceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
			makeLogicalDNSClusterResource(dnsClusterNameNew, dnsHostNameNew, dnsPort),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	wantChildCfg = &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{
			{
				Cluster:          edsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeEDS,
				EDSServiceName:   serviceName,
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
			{
				Cluster:          dnsClusterNameNew,
				Type:             clusterresolver.DiscoveryMechanismTypeLogicalDNS,
				DNSHostname:      fmt.Sprintf("%s:%d", dnsHostNameNew, dnsPort),
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
		},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}
}

// Tests the case where the cluster resource requested by the cds LB policy is
// an aggregate cluster root pointing to two child clusters, one of type EDS and
// the other of type LogicalDNS. The test verifies that the load balancing
// configuration pushed to the cluster_resolver LB policy contains the discovery
// mechanisms for both child clusters. The test then updates the root cluster
// resource requested by the cds LB policy to a leaf cluster of type EDS and
// verifies the load balancing configuration pushed to the cluster_resolver LB
// policy contains a single discovery mechanism.
func (s) TestAggregateClusterSuccess_ThenChangeRootToEDS(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

	// Configure the management server with the aggregate cluster resource
	// pointing to two child clusters.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, serviceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{
			{
				Cluster:          edsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeEDS,
				EDSServiceName:   serviceName,
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
			{
				Cluster:          dnsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeLogicalDNS,
				DNSHostname:      fmt.Sprintf("%s:%d", dnsHostName, dnsPort),
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
		},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}

	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	wantChildCfg = &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:          clusterName,
			Type:             clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:   serviceName,
			OutlierDetection: json.RawMessage(`{}`),
			TelemetryLabels:  internal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}
}

// Tests the case where a requested cluster resource switches between being a
// leaf and an aggregate cluster pointing to an EDS and LogicalDNS child
// cluster. In each of these cases, the test verifies that the load balancing
// configuration pushed to the cluster_resolver LB policy contains the expected
// discovery mechanisms.
func (s) TestAggregatedClusterSuccess_SwitchBetweenLeafAndAggregate(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

	// Start off with the requested cluster being a leaf EDS cluster.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:          clusterName,
			Type:             clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:   serviceName,
			OutlierDetection: json.RawMessage(`{}`),
			TelemetryLabels:  internal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}

	// Switch the requested cluster to be an aggregate cluster pointing to two
	// child clusters.
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{edsClusterName, dnsClusterName}),
			e2e.DefaultCluster(edsClusterName, serviceName, e2e.SecurityLevelNone),
			makeLogicalDNSClusterResource(dnsClusterName, dnsHostName, dnsPort),
		},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	wantChildCfg = &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{
			{
				Cluster:          edsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeEDS,
				EDSServiceName:   serviceName,
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
			{
				Cluster:          dnsClusterName,
				Type:             clusterresolver.DiscoveryMechanismTypeLogicalDNS,
				DNSHostname:      fmt.Sprintf("%s:%d", dnsHostName, dnsPort),
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
		},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}

	// Switch the cluster back to a leaf EDS cluster.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	wantChildCfg = &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:          clusterName,
			Type:             clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:   serviceName,
			OutlierDetection: json.RawMessage(`{}`),
			TelemetryLabels:  internal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}
}

// Tests the scenario where an aggregate cluster exceeds the maximum depth,
// which is 16. Verifies that the channel moves to TRANSIENT_FAILURE, and the
// error is propagated to RPC callers. The test then modifies the graph to no
// longer exceed maximum depth, but be at the maximum allowed depth, and
// verifies that an RPC can be made successfully.
func (s) TestAggregatedClusterFailure_ExceedsMaxStackDepth(t *testing.T) {
	mgmtServer, nodeID, cc, _, _, _, _ := setupWithManagementServer(t)

	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{clusterName + "-1"}),
			makeAggregateClusterResource(clusterName+"-1", []string{clusterName + "-2"}),
			makeAggregateClusterResource(clusterName+"-2", []string{clusterName + "-3"}),
			makeAggregateClusterResource(clusterName+"-3", []string{clusterName + "-4"}),
			makeAggregateClusterResource(clusterName+"-4", []string{clusterName + "-5"}),
			makeAggregateClusterResource(clusterName+"-5", []string{clusterName + "-6"}),
			makeAggregateClusterResource(clusterName+"-6", []string{clusterName + "-7"}),
			makeAggregateClusterResource(clusterName+"-7", []string{clusterName + "-8"}),
			makeAggregateClusterResource(clusterName+"-8", []string{clusterName + "-9"}),
			makeAggregateClusterResource(clusterName+"-9", []string{clusterName + "-10"}),
			makeAggregateClusterResource(clusterName+"-10", []string{clusterName + "-11"}),
			makeAggregateClusterResource(clusterName+"-11", []string{clusterName + "-12"}),
			makeAggregateClusterResource(clusterName+"-12", []string{clusterName + "-13"}),
			makeAggregateClusterResource(clusterName+"-13", []string{clusterName + "-14"}),
			makeAggregateClusterResource(clusterName+"-14", []string{clusterName + "-15"}),
			makeAggregateClusterResource(clusterName+"-15", []string{clusterName + "-16"}),
			e2e.DefaultCluster(clusterName+"-16", serviceName, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	const wantErr = "aggregate cluster graph exceeds max depth"
	client := testgrpc.NewTestServiceClient(cc)
	_, err := client.EmptyCall(ctx, &testpb.Empty{})
	if code := status.Code(err); code != codes.Unavailable {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", code, codes.Unavailable)
	}
	if err != nil && !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("EmptyCall() failed with err: %v, want err containing: %v", err, wantErr)
	}

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	// Update the aggregate cluster resource to no longer exceed max depth, and
	// be at the maximum depth allowed.
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterName, []string{clusterName + "-1"}),
			makeAggregateClusterResource(clusterName+"-1", []string{clusterName + "-2"}),
			makeAggregateClusterResource(clusterName+"-2", []string{clusterName + "-3"}),
			makeAggregateClusterResource(clusterName+"-3", []string{clusterName + "-4"}),
			makeAggregateClusterResource(clusterName+"-4", []string{clusterName + "-5"}),
			makeAggregateClusterResource(clusterName+"-5", []string{clusterName + "-6"}),
			makeAggregateClusterResource(clusterName+"-6", []string{clusterName + "-7"}),
			makeAggregateClusterResource(clusterName+"-7", []string{clusterName + "-8"}),
			makeAggregateClusterResource(clusterName+"-8", []string{clusterName + "-9"}),
			makeAggregateClusterResource(clusterName+"-9", []string{clusterName + "-10"}),
			makeAggregateClusterResource(clusterName+"-10", []string{clusterName + "-11"}),
			makeAggregateClusterResource(clusterName+"-11", []string{clusterName + "-12"}),
			makeAggregateClusterResource(clusterName+"-12", []string{clusterName + "-13"}),
			makeAggregateClusterResource(clusterName+"-13", []string{clusterName + "-14"}),
			makeAggregateClusterResource(clusterName+"-14", []string{clusterName + "-15"}),
			e2e.DefaultCluster(clusterName+"-15", serviceName, e2e.SecurityLevelNone),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
}

// Tests a diamond shaped aggregate cluster (A->[B,C]; B->D; C->D). Verifies
// that the load balancing configuration pushed to the cluster_resolver LB
// policy specifies cluster D only once. Also verifies that configuration is
// pushed only after all child clusters are resolved.
func (s) TestAggregatedClusterSuccess_DiamondDependency(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

	// Configure the management server with an aggregate cluster resource having
	// a diamond dependency pattern, (A->[B,C]; B->D; C->D). Includes resources
	// for cluster A, B and D, but don't include the resource for cluster C yet.
	// This will help us verify that no configuration is pushed to the child
	// policy until the whole cluster graph is resolved.
	const (
		clusterNameA = clusterName // cluster name in cds LB policy config
		clusterNameB = clusterName + "-B"
		clusterNameC = clusterName + "-C"
		clusterNameD = clusterName + "-D"
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterNameA, []string{clusterNameB, clusterNameC}),
			makeAggregateClusterResource(clusterNameB, []string{clusterNameD}),
			e2e.DefaultCluster(clusterNameD, serviceName, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that no configuration is pushed to the child policy yet, because
	// not all clusters making up the aggregate cluster have been resolved yet.
	select {
	case cfg := <-lbCfgCh:
		t.Fatalf("Child policy received configuration when not expected to: %s", pretty.ToJSON(cfg))
	case <-time.After(defaultTestShortTimeout):
	}

	// Now configure the resource for cluster C in the management server,
	// thereby completing the cluster graph. This should result in configuration
	// being pushed down to the child policy.
	resources.Clusters = append(resources.Clusters, makeAggregateClusterResource(clusterNameC, []string{clusterNameD}))
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:          clusterNameD,
			Type:             clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:   serviceName,
			OutlierDetection: json.RawMessage(`{}`),
			TelemetryLabels:  internal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}
}

// Tests the case where the aggregate cluster graph contains duplicates (A->[B,
// C]; B->[C, D]). Verifies that the load balancing configuration pushed to the
// cluster_resolver LB policy does not contain duplicates, and that the
// discovery mechanism corresponding to cluster C is of higher priority than the
// discovery mechanism for cluster D. Also verifies that the configuration is
// pushed only after all child clusters are resolved.
func (s) TestAggregatedClusterSuccess_IgnoreDups(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

	// Configure the management server with an aggregate cluster resource that
	// has duplicates in the graph, (A->[B, C]; B->[C, D]). Include resources
	// for clusters A, B and D, but don't configure the resource for cluster C
	// yet. This will help us verify that no configuration is pushed to the
	// child policy until the whole cluster graph is resolved.
	const (
		clusterNameA = clusterName // cluster name in cds LB policy config
		clusterNameB = clusterName + "-B"
		clusterNameC = clusterName + "-C"
		clusterNameD = clusterName + "-D"
	)
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterNameA, []string{clusterNameB, clusterNameC}),
			makeAggregateClusterResource(clusterNameB, []string{clusterNameC, clusterNameD}),
			e2e.DefaultCluster(clusterNameD, serviceName, e2e.SecurityLevelNone),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that no configuration is pushed to the child policy yet, because
	// not all clusters making up the aggregate cluster have been resolved yet.
	select {
	case cfg := <-lbCfgCh:
		t.Fatalf("Child policy received configuration when not expected to: %s", pretty.ToJSON(cfg))
	case <-time.After(defaultTestShortTimeout):
	}

	// Now configure the resource for cluster C in the management server,
	// thereby completing the cluster graph. This should result in configuration
	// being pushed down to the child policy.
	resources.Clusters = append(resources.Clusters, e2e.DefaultCluster(clusterNameC, serviceName, e2e.SecurityLevelNone))
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{
			{
				Cluster:          clusterNameC,
				Type:             clusterresolver.DiscoveryMechanismTypeEDS,
				EDSServiceName:   serviceName,
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
			{
				Cluster:          clusterNameD,
				Type:             clusterresolver.DiscoveryMechanismTypeEDS,
				EDSServiceName:   serviceName,
				OutlierDetection: json.RawMessage(`{}`),
				TelemetryLabels:  internal.UnknownCSMLabels,
			},
		},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}
}

// Tests the scenario where the aggregate cluster graph has a node that has
// child node of itself. The case for this is A -> A, and since there is no base
// cluster (EDS or Logical DNS), no configuration should be pushed to the child
// policy.  The channel is expected to move to TRANSIENT_FAILURE and RPCs are
// expected to fail with code UNAVAILABLE and an error message specifying that
// the aggregate cluster graph has no leaf clusters.  Then the test updates A -> B,
// where B is a leaf EDS cluster. Verifies that configuration is pushed to the
// child policy and that an RPC can be successfully made.
func (s) TestAggregatedCluster_NodeChildOfItself(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, _, _, _, _ := setupWithManagementServer(t)

	const (
		clusterNameA = clusterName // cluster name in cds LB policy config
		clusterNameB = clusterName + "-B"
	)
	// Configure the management server with an aggregate cluster resource whose
	// child is itself.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{makeAggregateClusterResource(clusterNameA, []string{clusterNameA})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	select {
	case cfg := <-lbCfgCh:
		t.Fatalf("Child policy received configuration when not expected to: %s", pretty.ToJSON(cfg))
	case <-time.After(defaultTestShortTimeout):
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Verify that the RPC fails with expected code.
	client := testgrpc.NewTestServiceClient(cc)
	_, err := client.EmptyCall(ctx, &testpb.Empty{})
	if gotCode, wantCode := status.Code(err), codes.Unavailable; gotCode != wantCode {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", gotCode, wantCode)
	}
	const wantErr = "aggregate cluster graph has no leaf clusters"
	if !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("EmptyCall() failed with err: %v, want error containing %s", err, wantErr)
	}

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	// Update the aggregate cluster to point to a leaf EDS cluster.
	resources = e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterNameA, []string{clusterNameB}),
			e2e.DefaultCluster(clusterNameB, serviceName, e2e.SecurityLevelNone),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify the configuration pushed to the child policy.
	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:          clusterNameB,
			Type:             clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:   serviceName,
			OutlierDetection: json.RawMessage(`{}`),
			TelemetryLabels:  internal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
}

// Tests the scenario where the aggregate cluster graph contains a cycle and
// contains no leaf clusters. The case used here is [A -> B, B -> A]. As there
// are no leaf clusters in this graph, no configuration should be pushed to the
// child policy. The channel is expected to move to TRANSIENT_FAILURE and RPCs
// are expected to fail with code UNAVAILABLE and an error message specifying
// that the aggregate cluster graph has no leaf clusters.
func (s) TestAggregatedCluster_CycleWithNoLeafNode(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, _, _, _, _ := setupWithManagementServer(t)

	const (
		clusterNameA = clusterName // cluster name in cds LB policy config
		clusterNameB = clusterName + "-B"
	)
	// Configure the management server with an aggregate cluster resource graph
	// that contains a cycle and no leaf clusters.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterNameA, []string{clusterNameB}),
			makeAggregateClusterResource(clusterNameB, []string{clusterNameA}),
		},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	select {
	case cfg := <-lbCfgCh:
		t.Fatalf("Child policy received configuration when not expected to: %s", pretty.ToJSON(cfg))
	case <-time.After(defaultTestShortTimeout):
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Verify that the RPC fails with expected code.
	client := testgrpc.NewTestServiceClient(cc)
	_, err := client.EmptyCall(ctx, &testpb.Empty{})
	if gotCode, wantCode := status.Code(err), codes.Unavailable; gotCode != wantCode {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", gotCode, wantCode)
	}
	const wantErr = "aggregate cluster graph has no leaf clusters"
	if !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("EmptyCall() failed with err: %v, want %s", err, wantErr)
	}
}

// Tests the scenario where the aggregate cluster graph contains a cycle and
// also contains a leaf cluster. The case used here is [A -> B, B -> A, C]. As
// there is a leaf cluster in this graph , configuration should be pushed to the
// child policy and RPCs should get routed to that leaf cluster.
func (s) TestAggregatedCluster_CycleWithLeafNode(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, _, _, _, _ := setupWithManagementServer(t)

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	const (
		clusterNameA = clusterName // cluster name in cds LB policy config
		clusterNameB = clusterName + "-B"
		clusterNameC = clusterName + "-C"
	)
	// Configure the management server with an aggregate cluster resource graph
	// that contains a cycle, but also contains a leaf cluster.
	resources := e2e.UpdateOptions{
		NodeID: nodeID,
		Clusters: []*v3clusterpb.Cluster{
			makeAggregateClusterResource(clusterNameA, []string{clusterNameB}),
			makeAggregateClusterResource(clusterNameB, []string{clusterNameA, clusterNameC}),
			e2e.DefaultCluster(clusterNameC, serviceName, e2e.SecurityLevelNone),
		},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify the configuration pushed to the child policy.
	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:          clusterNameC,
			Type:             clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:   serviceName,
			OutlierDetection: json.RawMessage(`{}`),
			TelemetryLabels:  internal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}
	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
}
