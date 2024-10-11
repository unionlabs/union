/*
 * Copyright 2019 gRPC authors.
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
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"
	xdsinternal "google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/balancer/clusterresolver"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/xds/internal/balancer/ringhash" // Register the ring_hash LB policy
)

const (
	clusterName             = "cluster1"
	edsClusterName          = clusterName + "-eds"
	dnsClusterName          = clusterName + "-dns"
	serviceName             = "service1"
	dnsHostName             = "dns_host"
	dnsPort                 = uint32(8080)
	defaultTestTimeout      = 5 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond // For events expected to *not* happen.
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func waitForResourceNames(ctx context.Context, resourceNamesCh chan []string, wantNames []string) error {
	for ctx.Err() == nil {
		select {
		case <-ctx.Done():
		case gotNames := <-resourceNamesCh:
			if cmp.Equal(gotNames, wantNames) {
				return nil
			}
		}
	}
	if ctx.Err() != nil {
		return fmt.Errorf("Timeout when waiting for appropriate Cluster resources to be requested")
	}
	return nil
}

// Registers a wrapped cluster_resolver LB policy (child policy of the cds LB
// policy) for the duration of this test that retains all the functionality of
// the former, but makes certain events available for inspection by the test.
//
// Returns the following:
// - a channel to read received load balancing configuration
// - a channel to read received resolver error
// - a channel that is closed when ExitIdle() is called
// - a channel that is closed when the balancer is closed
func registerWrappedClusterResolverPolicy(t *testing.T) (chan serviceconfig.LoadBalancingConfig, chan error, chan struct{}, chan struct{}) {
	clusterresolverBuilder := balancer.Get(clusterresolver.Name)
	internal.BalancerUnregister(clusterresolverBuilder.Name())

	lbCfgCh := make(chan serviceconfig.LoadBalancingConfig, 1)
	resolverErrCh := make(chan error, 1)
	exitIdleCh := make(chan struct{})
	closeCh := make(chan struct{})

	stub.Register(clusterresolver.Name, stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			bd.Data = clusterresolverBuilder.Build(bd.ClientConn, bd.BuildOptions)
		},
		ParseConfig: func(lbCfg json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			return clusterresolverBuilder.(balancer.ConfigParser).ParseConfig(lbCfg)
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			select {
			case lbCfgCh <- ccs.BalancerConfig:
			default:
			}
			bal := bd.Data.(balancer.Balancer)
			return bal.UpdateClientConnState(ccs)
		},
		ResolverError: func(bd *stub.BalancerData, err error) {
			select {
			case resolverErrCh <- err:
			default:
			}
			bal := bd.Data.(balancer.Balancer)
			bal.ResolverError(err)
		},
		ExitIdle: func(bd *stub.BalancerData) {
			bal := bd.Data.(balancer.Balancer)
			bal.(balancer.ExitIdler).ExitIdle()
			close(exitIdleCh)
		},
		Close: func(bd *stub.BalancerData) {
			bal := bd.Data.(balancer.Balancer)
			bal.Close()
			close(closeCh)
		},
	})
	t.Cleanup(func() { balancer.Register(clusterresolverBuilder) })

	return lbCfgCh, resolverErrCh, exitIdleCh, closeCh
}

// Registers a wrapped cds LB policy for the duration of this test that retains
// all the functionality of the original cds LB policy, but makes the newly
// built policy available to the test to directly invoke any balancer methods.
//
// Returns a channel on which the newly built cds LB policy is written to.
func registerWrappedCDSPolicy(t *testing.T) chan balancer.Balancer {
	cdsBuilder := balancer.Get(cdsName)
	internal.BalancerUnregister(cdsBuilder.Name())
	cdsBalancerCh := make(chan balancer.Balancer, 1)
	stub.Register(cdsBuilder.Name(), stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			bal := cdsBuilder.Build(bd.ClientConn, bd.BuildOptions)
			bd.Data = bal
			cdsBalancerCh <- bal
		},
		ParseConfig: func(lbCfg json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			return cdsBuilder.(balancer.ConfigParser).ParseConfig(lbCfg)
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			bal := bd.Data.(balancer.Balancer)
			return bal.UpdateClientConnState(ccs)
		},
		Close: func(bd *stub.BalancerData) {
			bal := bd.Data.(balancer.Balancer)
			bal.Close()
		},
	})
	t.Cleanup(func() { balancer.Register(cdsBuilder) })

	return cdsBalancerCh
}

// Performs the following setup required for tests:
//   - Spins up an xDS management server
//   - Creates an xDS client talking to this management server
//   - Creates a manual resolver that configures the cds LB policy as the
//     top-level policy, and pushes an initial configuration to it
//   - Creates a gRPC channel with the above manual resolver
//
// Returns the following:
//   - the xDS management server
//   - the nodeID expected by the management server
//   - the grpc channel to the test backend service
//   - the manual resolver configured on the channel
//   - the xDS client used the grpc channel
//   - a channel on which requested cluster resource names are sent
//   - a channel used to signal that previously requested cluster resources are
//     no longer requested
func setupWithManagementServer(t *testing.T) (*e2e.ManagementServer, string, *grpc.ClientConn, *manual.Resolver, xdsclient.XDSClient, chan []string, chan struct{}) {
	t.Helper()

	cdsResourceRequestedCh := make(chan []string, 1)
	cdsResourceCanceledCh := make(chan struct{}, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() == version.V3ClusterURL {
				switch len(req.GetResourceNames()) {
				case 0:
					select {
					case cdsResourceCanceledCh <- struct{}{}:
					default:
					}
				default:
					select {
					case cdsResourceRequestedCh <- req.GetResourceNames():
					default:
					}
				}
			}
			return nil
		},
		// Required for aggregate clusters as all resources cannot be requested
		// at once.
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	xdsC, xdsClose, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	t.Cleanup(xdsClose)

	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": "%s"
				}
			}]
		}`, clusterName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsC))

	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	return mgmtServer, nodeID, cc, r, xdsC, cdsResourceRequestedCh, cdsResourceCanceledCh
}

// Helper function to compare the load balancing configuration received on the
// channel with the expected one. Both configs are marshalled to JSON and then
// compared.
//
// Returns an error if marshalling to JSON fails, or if the load balancing
// configurations don't match, or if the context deadline expires before reading
// a child policy configuration off of the lbCfgCh.
func compareLoadBalancingConfig(ctx context.Context, lbCfgCh chan serviceconfig.LoadBalancingConfig, wantChildCfg serviceconfig.LoadBalancingConfig) error {
	wantJSON, err := json.Marshal(wantChildCfg)
	if err != nil {
		return fmt.Errorf("failed to marshal expected child config to JSON: %v", err)
	}
	select {
	case lbCfg := <-lbCfgCh:
		gotJSON, err := json.Marshal(lbCfg)
		if err != nil {
			return fmt.Errorf("failed to marshal received LB config into JSON: %v", err)
		}
		if diff := cmp.Diff(wantJSON, gotJSON); diff != "" {
			return fmt.Errorf("child policy received unexpected diff in config (-want +got):\n%s", diff)
		}
	case <-ctx.Done():
		return fmt.Errorf("timeout when waiting for child policy to receive its configuration")
	}
	return nil
}

// Tests the functionality that handles LB policy configuration. Verifies that
// the appropriate xDS resource is requested corresponding to the provided LB
// policy configuration. Also verifies that when the LB policy receives the same
// configuration again, it does not send out a new request, and when the
// configuration changes, it stops requesting the old cluster resource and
// starts requesting the new one.
func (s) TestConfigurationUpdate_Success(t *testing.T) {
	_, _, _, r, xdsClient, cdsResourceRequestedCh, _ := setupWithManagementServer(t)

	// Verify that the specified cluster resource is requested.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantNames := []string{clusterName}
	if err := waitForResourceNames(ctx, cdsResourceRequestedCh, wantNames); err != nil {
		t.Fatal(err)
	}

	// Push the same configuration again.
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": "%s"
				}
			}]
		}`, clusterName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.UpdateState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Verify that a new CDS request is not sent.
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case gotNames := <-cdsResourceRequestedCh:
		t.Fatalf("CDS resources %v requested when none expected", gotNames)
	}

	// Push an updated configuration with a different cluster name.
	newClusterName := clusterName + "-new"
	jsonSC = fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": "%s"
				}
			}]
		}`, newClusterName)
	scpr = internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.UpdateState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Verify that the new cluster name is requested and the old one is no
	// longer requested.
	wantNames = []string{newClusterName}
	if err := waitForResourceNames(ctx, cdsResourceRequestedCh, wantNames); err != nil {
		t.Fatal(err)
	}
}

// Tests the case where a configuration with an empty cluster name is pushed to
// the CDS LB policy. Verifies that ErrBadResolverState is returned.
func (s) TestConfigurationUpdate_EmptyCluster(t *testing.T) {
	// Setup a management server and an xDS client to talk to it.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	xdsClient, xdsClose, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bc,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	t.Cleanup(xdsClose)

	// Create a manual resolver that configures the CDS LB policy as the
	// top-level LB policy on the channel, and pushes a configuration with an
	// empty cluster name. Also, register a callback with the manual resolver to
	// receive the error returned by the balancer when a configuration with an
	// empty cluster name is pushed.
	r := manual.NewBuilderWithScheme("whatever")
	updateStateErrCh := make(chan error, 1)
	r.UpdateStateCallback = func(err error) { updateStateErrCh <- err }
	jsonSC := `{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": ""
				}
			}]
		}`
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient))

	// Create a ClientConn with the above manual resolver.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("Failed to dial: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	select {
	case <-time.After(defaultTestTimeout):
		t.Fatalf("Timed out waiting for error from the LB policy")
	case err := <-updateStateErrCh:
		if err != balancer.ErrBadResolverState {
			t.Fatalf("For a configuration update with an empty cluster name, got error %v from the LB policy, want %v", err, balancer.ErrBadResolverState)
		}
	}
}

// Tests the case where a configuration with a missing xDS client is pushed to
// the CDS LB policy. Verifies that ErrBadResolverState is returned.
func (s) TestConfigurationUpdate_MissingXdsClient(t *testing.T) {
	// Create a manual resolver that configures the CDS LB policy as the
	// top-level LB policy on the channel, and pushes a configuration that is
	// missing the xDS client.  Also, register a callback with the manual
	// resolver to receive the error returned by the balancer.
	r := manual.NewBuilderWithScheme("whatever")
	updateStateErrCh := make(chan error, 1)
	r.UpdateStateCallback = func(err error) { updateStateErrCh <- err }
	jsonSC := `{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": "foo"
				}
			}]
		}`
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	r.InitialState(resolver.State{ServiceConfig: scpr})

	// Create a ClientConn with the above manual resolver.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("Failed to dial: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	select {
	case <-time.After(defaultTestTimeout):
		t.Fatalf("Timed out waiting for error from the LB policy")
	case err := <-updateStateErrCh:
		if err != balancer.ErrBadResolverState {
			t.Fatalf("For a configuration update missing the xDS client, got error %v from the LB policy, want %v", err, balancer.ErrBadResolverState)
		}
	}
}

// Tests success scenarios where the cds LB policy receives a cluster resource
// from the management server. Verifies that the load balancing configuration
// pushed to the child is as expected.
func (s) TestClusterUpdate_Success(t *testing.T) {
	tests := []struct {
		name            string
		clusterResource *v3clusterpb.Cluster
		wantChildCfg    serviceconfig.LoadBalancingConfig
	}{
		{
			name: "happy-case-with-circuit-breakers",
			clusterResource: func() *v3clusterpb.Cluster {
				c := e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)
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
			wantChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:               clusterName,
					Type:                  clusterresolver.DiscoveryMechanismTypeEDS,
					EDSServiceName:        serviceName,
					MaxConcurrentRequests: newUint32(512),
					OutlierDetection:      json.RawMessage(`{}`),
					TelemetryLabels:       xdsinternal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
			},
		},
		{
			name: "happy-case-with-ring-hash-lb-policy",
			clusterResource: func() *v3clusterpb.Cluster {
				c := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
					ClusterName:   clusterName,
					ServiceName:   serviceName,
					SecurityLevel: e2e.SecurityLevelNone,
					Policy:        e2e.LoadBalancingPolicyRingHash,
				})
				c.LbConfig = &v3clusterpb.Cluster_RingHashLbConfig_{
					RingHashLbConfig: &v3clusterpb.Cluster_RingHashLbConfig{
						MinimumRingSize: &wrapperspb.UInt64Value{Value: 100},
						MaximumRingSize: &wrapperspb.UInt64Value{Value: 1000},
					},
				}
				return c
			}(),
			wantChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:          clusterName,
					Type:             clusterresolver.DiscoveryMechanismTypeEDS,
					EDSServiceName:   serviceName,
					OutlierDetection: json.RawMessage(`{}`),
					TelemetryLabels:  xdsinternal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"ring_hash_experimental": {"minRingSize":100, "maxRingSize":1000}}]`),
			},
		},
		{
			name: "happy-case-outlier-detection-xds-defaults", // OD proto set but no proto fields set
			clusterResource: func() *v3clusterpb.Cluster {
				c := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
					ClusterName:   clusterName,
					ServiceName:   serviceName,
					SecurityLevel: e2e.SecurityLevelNone,
					Policy:        e2e.LoadBalancingPolicyRingHash,
				})
				c.OutlierDetection = &v3clusterpb.OutlierDetection{}
				return c
			}(),
			wantChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:          clusterName,
					Type:             clusterresolver.DiscoveryMechanismTypeEDS,
					EDSServiceName:   serviceName,
					OutlierDetection: json.RawMessage(`{"successRateEjection":{}}`),
					TelemetryLabels:  xdsinternal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"ring_hash_experimental": {"minRingSize":1024, "maxRingSize":8388608}}]`),
			},
		},
		{
			name: "happy-case-outlier-detection-all-fields-set",
			clusterResource: func() *v3clusterpb.Cluster {
				c := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
					ClusterName:   clusterName,
					ServiceName:   serviceName,
					SecurityLevel: e2e.SecurityLevelNone,
					Policy:        e2e.LoadBalancingPolicyRingHash,
				})
				c.OutlierDetection = &v3clusterpb.OutlierDetection{
					Interval:                       durationpb.New(10 * time.Second),
					BaseEjectionTime:               durationpb.New(30 * time.Second),
					MaxEjectionTime:                durationpb.New(300 * time.Second),
					MaxEjectionPercent:             wrapperspb.UInt32(10),
					SuccessRateStdevFactor:         wrapperspb.UInt32(1900),
					EnforcingSuccessRate:           wrapperspb.UInt32(100),
					SuccessRateMinimumHosts:        wrapperspb.UInt32(5),
					SuccessRateRequestVolume:       wrapperspb.UInt32(100),
					FailurePercentageThreshold:     wrapperspb.UInt32(85),
					EnforcingFailurePercentage:     wrapperspb.UInt32(5),
					FailurePercentageMinimumHosts:  wrapperspb.UInt32(5),
					FailurePercentageRequestVolume: wrapperspb.UInt32(50),
				}
				return c
			}(),
			wantChildCfg: &clusterresolver.LBConfig{
				DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
					Cluster:        clusterName,
					Type:           clusterresolver.DiscoveryMechanismTypeEDS,
					EDSServiceName: serviceName,
					OutlierDetection: json.RawMessage(`{
						"interval": "10s",
						"baseEjectionTime": "30s",
						"maxEjectionTime": "300s",
						"maxEjectionPercent": 10,
						"successRateEjection": {
							"stdevFactor": 1900,
							"enforcementPercentage": 100,
							"minimumHosts": 5,
							"requestVolume": 100
						},
						"failurePercentageEjection": {
							"threshold": 85,
							"enforcementPercentage": 5,
							"minimumHosts": 5,
							"requestVolume": 50
						}
					}`),
					TelemetryLabels: xdsinternal.UnknownCSMLabels,
				}},
				XDSLBPolicy: json.RawMessage(`[{"ring_hash_experimental": {"minRingSize":1024, "maxRingSize":8388608}}]`),
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
			mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
				NodeID:         nodeID,
				Clusters:       []*v3clusterpb.Cluster{test.clusterResource},
				SkipValidation: true,
			}); err != nil {
				t.Fatal(err)
			}

			if err := compareLoadBalancingConfig(ctx, lbCfgCh, test.wantChildCfg); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// Tests a single success scenario where the cds LB policy receives a cluster
// resource from the management server with LRS enabled. Verifies that the load
// balancing configuration pushed to the child is as expected.
func (s) TestClusterUpdate_SuccessWithLRS(t *testing.T) {
	lbCfgCh, _, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, _, _, _, _, _ := setupWithManagementServer(t)

	clusterResource := e2e.ClusterResourceWithOptions(e2e.ClusterOptions{
		ClusterName: clusterName,
		ServiceName: serviceName,
		EnableLRS:   true,
	})
	lrsServerCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: fmt.Sprintf("passthrough:///%s", mgmtServer.Address)})
	if err != nil {
		t.Fatalf("Failed to create LRS server config for testing: %v", err)
	}

	wantChildCfg := &clusterresolver.LBConfig{
		DiscoveryMechanisms: []clusterresolver.DiscoveryMechanism{{
			Cluster:             clusterName,
			Type:                clusterresolver.DiscoveryMechanismTypeEDS,
			EDSServiceName:      serviceName,
			LoadReportingServer: lrsServerCfg,
			OutlierDetection:    json.RawMessage(`{}`),
			TelemetryLabels:     xdsinternal.UnknownCSMLabels,
		}},
		XDSLBPolicy: json.RawMessage(`[{"xds_wrr_locality_experimental": {"childPolicy": [{"round_robin": {}}]}}]`),
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{clusterResource},
		SkipValidation: true,
	}); err != nil {
		t.Fatal(err)
	}

	if err := compareLoadBalancingConfig(ctx, lbCfgCh, wantChildCfg); err != nil {
		t.Fatal(err)
	}
}

// Tests scenarios for a bad cluster update received from the management server.
//
//   - when a bad cluster resource update is received without any previous good
//     update from the management server, the cds LB policy is expected to put
//     the channel in TRANSIENT_FAILURE.
//   - when a bad cluster resource update is received after a previous good
//     update from the management server, the cds LB policy is expected to
//     continue using the previous good update.
//   - when the cluster resource is removed after a previous good
//     update from the management server, the cds LB policy is expected to put
//     the channel in TRANSIENT_FAILURE.
func (s) TestClusterUpdate_Failure(t *testing.T) {
	_, resolverErrCh, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, _, _, cdsResourceRequestedCh, cdsResourceCanceledCh := setupWithManagementServer(t)

	// Verify that the specified cluster resource is requested.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantNames := []string{clusterName}
	if err := waitForResourceNames(ctx, cdsResourceRequestedCh, wantNames); err != nil {
		t.Fatal(err)
	}

	// Configure the management server to return a cluster resource that
	// contains a config_source_specifier for the `lrs_server` field which is not
	// set to `self`, and hence is expected to be NACKed by the client.
	cluster := e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)
	cluster.LrsServer = &v3corepb.ConfigSource{ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{}}
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{cluster},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that the watch for the cluster resource is not cancelled.
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-cdsResourceCanceledCh:
		t.Fatal("Watch for cluster resource is cancelled when not expected to")
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Ensure that the NACK error is propagated to the RPC caller.
	const wantClusterNACKErr = "unsupported config_source_specifier"
	client := testgrpc.NewTestServiceClient(cc)
	_, err := client.EmptyCall(ctx, &testpb.Empty{})
	if code := status.Code(err); code != codes.Unavailable {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", code, codes.Unavailable)
	}
	if err != nil && !strings.Contains(err.Error(), wantClusterNACKErr) {
		t.Fatalf("EmptyCall() failed with err: %v, want err containing: %v", err, wantClusterNACKErr)
	}

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	// Configure cluster and endpoints resources in the management server.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
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

	// Send the bad cluster resource again.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{cluster},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that the watch for the cluster resource is not cancelled.
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-cdsResourceCanceledCh:
		t.Fatal("Watch for cluster resource is cancelled when not expected to")
	}

	// Verify that a successful RPC can be made, using the previously received
	// good configuration.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Verify that the resolver error is pushed to the child policy.
	select {
	case err := <-resolverErrCh:
		if !strings.Contains(err.Error(), wantClusterNACKErr) {
			t.Fatalf("Error pushed to child policy is %v, want %v", err, wantClusterNACKErr)
		}
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for resolver error to be pushed to the child policy")
	}

	// Remove the cluster resource from the management server, triggering a
	// resource-not-found error.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that the watch for the cluster resource is not cancelled.
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-cdsResourceCanceledCh:
		t.Fatal("Watch for cluster resource is cancelled when not expected to")
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Ensure RPC fails with Unavailable. The actual error message depends on
	// the picker returned from the priority LB policy, and therefore not
	// checking for it here.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", status.Code(err), codes.Unavailable)
	}
}

// Tests the following scenarios for resolver errors:
//   - when a resolver error is received without any previous good update from the
//     management server, the cds LB policy is expected to put the channel in
//     TRANSIENT_FAILURE.
//   - when a resolver error is received (one that is not a resource-not-found
//     error), with a previous good update from the management server, the cds LB
//     policy is expected to push the error down the child policy, but is expected
//     to continue to use the previously received good configuration.
//   - when a resolver error is received (one that is a resource-not-found
//     error, which is usually the case when the LDS resource is removed),
//     with a previous good update from the management server, the cds LB policy
//     is expected to push the error down the child policy and put the channel in
//     TRANSIENT_FAILURE. It is also expected to cancel the CDS watch.
func (s) TestResolverError(t *testing.T) {
	_, resolverErrCh, _, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, r, _, cdsResourceRequestedCh, cdsResourceCanceledCh := setupWithManagementServer(t)

	// Verify that the specified cluster resource is requested.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantNames := []string{clusterName}
	if err := waitForResourceNames(ctx, cdsResourceRequestedCh, wantNames); err != nil {
		t.Fatal(err)
	}

	// Push a resolver error that is not a resource-not-found error.
	resolverErr := errors.New("resolver-error-not-a-resource-not-found-error")
	r.ReportError(resolverErr)

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Drain the resolver error channel.
	select {
	case <-resolverErrCh:
	default:
	}

	// Ensure that the resolver error is propagated to the RPC caller.
	client := testgrpc.NewTestServiceClient(cc)
	_, err := client.EmptyCall(ctx, &testpb.Empty{})
	if code := status.Code(err); code != codes.Unavailable {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", code, codes.Unavailable)
	}
	if err != nil && !strings.Contains(err.Error(), resolverErr.Error()) {
		t.Fatalf("EmptyCall() failed with err: %v, want %v", err, resolverErr)
	}

	// Also verify that the watch for the cluster resource is not cancelled.
	sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-cdsResourceCanceledCh:
		t.Fatal("Watch for cluster resource is cancelled when not expected to")
	}

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	// Configure good cluster and endpoints resources in the management server.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
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

	// Again push a resolver error that is not a resource-not-found error.
	r.ReportError(resolverErr)

	// And again verify that the watch for the cluster resource is not
	// cancelled.
	sCtx, sCancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-cdsResourceCanceledCh:
		t.Fatal("Watch for cluster resource is cancelled when not expected to")
	}

	// Verify that a successful RPC can be made, using the previously received
	// good configuration.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Verify that the resolver error is pushed to the child policy.
	select {
	case err := <-resolverErrCh:
		if err != resolverErr {
			t.Fatalf("Error pushed to child policy is %v, want %v", err, resolverErr)
		}
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for resolver error to be pushed to the child policy")
	}

	// Push a resource-not-found-error this time around.
	resolverErr = xdsresource.NewErrorf(xdsresource.ErrorTypeResourceNotFound, "xds resource not found error")
	r.ReportError(resolverErr)

	// Wait for the CDS resource to be not requested anymore.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for CDS resource to be not requested")
	case <-cdsResourceCanceledCh:
	}

	// Verify that the resolver error is pushed to the child policy.
	select {
	case err := <-resolverErrCh:
		if err != resolverErr {
			t.Fatalf("Error pushed to child policy is %v, want %v", err, resolverErr)
		}
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for resolver error to be pushed to the child policy")
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Ensure RPC fails with Unavailable. The actual error message depends on
	// the picker returned from the priority LB policy, and therefore not
	// checking for it here.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("EmptyCall() failed with code: %v, want %v", status.Code(err), codes.Unavailable)
	}
}

// Tests that closing the cds LB policy results in the cluster resource watch
// being cancelled and the child policy being closed.
func (s) TestClose(t *testing.T) {
	cdsBalancerCh := registerWrappedCDSPolicy(t)
	_, _, _, childPolicyCloseCh := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, _, _, _, cdsResourceCanceledCh := setupWithManagementServer(t)

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	// Configure cluster and endpoints resources in the management server.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Retrieve the cds LB policy and close it.
	var cdsBal balancer.Balancer
	select {
	case cdsBal = <-cdsBalancerCh:
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for cds LB policy to be created")
	}
	cdsBal.Close()

	// Wait for the CDS resource to be not requested anymore.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for CDS resource to be not requested")
	case <-cdsResourceCanceledCh:
	}
	// Wait for the child policy to be closed.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for the child policy to be closed")
	case <-childPolicyCloseCh:
	}
}

// Tests that calling ExitIdle on the cds LB policy results in the call being
// propagated to the child policy.
func (s) TestExitIdle(t *testing.T) {
	cdsBalancerCh := registerWrappedCDSPolicy(t)
	_, _, exitIdleCh, _ := registerWrappedClusterResolverPolicy(t)
	mgmtServer, nodeID, cc, _, _, _, _ := setupWithManagementServer(t)

	// Start a test service backend.
	server := stubserver.StartTestService(t, nil)
	t.Cleanup(server.Stop)

	// Configure cluster and endpoints resources in the management server.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, server.Address)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Retrieve the cds LB policy and call ExitIdle() on it.
	var cdsBal balancer.Balancer
	select {
	case cdsBal = <-cdsBalancerCh:
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for cds LB policy to be created")
	}
	cdsBal.(balancer.ExitIdler).ExitIdle()

	// Wait for ExitIdle to be called on the child policy.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for the child policy to be closed")
	case <-exitIdleCh:
	}
}

// TestParseConfig verifies the ParseConfig() method in the CDS balancer.
func (s) TestParseConfig(t *testing.T) {
	bb := balancer.Get(cdsName)
	if bb == nil {
		t.Fatalf("balancer.Get(%q) returned nil", cdsName)
	}
	parser, ok := bb.(balancer.ConfigParser)
	if !ok {
		t.Fatalf("balancer %q does not implement the ConfigParser interface", cdsName)
	}

	tests := []struct {
		name    string
		input   json.RawMessage
		wantCfg serviceconfig.LoadBalancingConfig
		wantErr bool
	}{
		{
			name:    "good-config",
			input:   json.RawMessage(`{"Cluster": "cluster1"}`),
			wantCfg: &lbConfig{ClusterName: "cluster1"},
		},
		{
			name:    "unknown-fields-in-config",
			input:   json.RawMessage(`{"Unknown": "foobar"}`),
			wantCfg: &lbConfig{ClusterName: ""},
		},
		{
			name:    "empty-config",
			input:   json.RawMessage(""),
			wantErr: true,
		},
		{
			name:    "bad-config",
			input:   json.RawMessage(`{"Cluster": 5}`),
			wantErr: true,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			gotCfg, gotErr := parser.ParseConfig(test.input)
			if (gotErr != nil) != test.wantErr {
				t.Fatalf("ParseConfig(%v) = %v, wantErr %v", string(test.input), gotErr, test.wantErr)
			}
			if test.wantErr {
				return
			}
			if !cmp.Equal(gotCfg, test.wantCfg) {
				t.Fatalf("ParseConfig(%v) = %v, want %v", string(test.input), gotCfg, test.wantCfg)
			}
		})
	}
}

func newUint32(i uint32) *uint32 {
	return &i
}
