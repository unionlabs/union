/*
 *
 * Copyright 2022 gRPC authors.
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
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/pickfirst"
	"google.golang.org/grpc/balancer/rls/internal/test/e2e"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	internalserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/internal/testutils"
	rlstest "google.golang.org/grpc/internal/testutils/rls"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/testdata"

	rlspb "google.golang.org/grpc/internal/proto/grpc_lookup_v1"
	"google.golang.org/protobuf/types/known/durationpb"
)

// TestConfigUpdate_ControlChannel tests the scenario where a config update
// changes the RLS server name. Verifies that the new control channel is created
// and the old one is closed.
func (s) TestConfigUpdate_ControlChannel(t *testing.T) {
	// Start two RLS servers.
	lis1 := testutils.NewListenerWrapper(t, nil)
	rlsServer1, rlsReqCh1 := rlstest.SetupFakeRLSServer(t, lis1)
	lis2 := testutils.NewListenerWrapper(t, nil)
	rlsServer2, rlsReqCh2 := rlstest.SetupFakeRLSServer(t, lis2)

	// Build RLS service config with the RLS server pointing to the first one.
	// Set a very low value for maxAge to ensure that the entry expires soon.
	rlsConfig := buildBasicRLSConfigWithChildPolicy(t, t.Name(), rlsServer1.Address)
	rlsConfig.RouteLookupConfig.MaxAge = durationpb.New(defaultTestShortTimeout)

	// Start a couple of test backends, and set up the fake RLS servers to return
	// these as a target in the RLS response.
	backendCh1, backendAddress1 := startBackend(t)
	rlsServer1.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress1}}}
	})
	backendCh2, backendAddress2 := startBackend(t)
	rlsServer2.SetResponseCallback(func(context.Context, *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress2}}}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure it gets routed to the test backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh1)

	// Ensure a connection is established to the first RLS server.
	val, err := lis1.NewConnCh.Receive(ctx)
	if err != nil {
		t.Fatal("Timeout expired when waiting for LB policy to create control channel")
	}
	conn1 := val.(*testutils.ConnWrapper)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh1, true)

	// Change lookup_service field of the RLS config to point to the second one.
	rlsConfig.RouteLookupConfig.LookupService = rlsServer2.Address

	// Push the config update through the manual resolver.
	scJSON, err := rlsConfig.ServiceConfigJSON()
	if err != nil {
		t.Fatal(err)
	}
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.UpdateState(resolver.State{ServiceConfig: sc})

	// Ensure a connection is established to the second RLS server.
	if _, err := lis2.NewConnCh.Receive(ctx); err != nil {
		t.Fatal("Timeout expired when waiting for LB policy to create control channel")
	}

	// Ensure the connection to the old one is closed.
	if _, err := conn1.CloseCh.Receive(ctx); err != nil {
		t.Fatal("Timeout expired when waiting for LB policy to close control channel")
	}

	// Make an RPC and expect it to get routed to the second test backend through
	// the second RLS server.
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh2)
	verifyRLSRequest(t, rlsReqCh2, true)
}

// TestConfigUpdate_ControlChannelWithCreds tests the scenario where a config
// update specified an RLS server name, and the parent ClientConn specifies
// transport credentials. The RLS server and the test backend are configured to
// accept those transport credentials. This test verifies that the parent
// channel credentials are correctly propagated to the control channel.
func (s) TestConfigUpdate_ControlChannelWithCreds(t *testing.T) {
	serverCreds, err := credentials.NewServerTLSFromFile(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatalf("credentials.NewServerTLSFromFile(server1.pem, server1.key) = %v", err)
	}
	clientCreds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "")
	if err != nil {
		t.Fatalf("credentials.NewClientTLSFromFile(ca.pem) = %v", err)
	}

	// Start an RLS server with the wrapped listener and credentials.
	lis := testutils.NewListenerWrapper(t, nil)
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, lis, grpc.Creds(serverCreds))
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Build RLS service config.
	rlsConfig := buildBasicRLSConfigWithChildPolicy(t, t.Name(), rlsServer.Address)

	// Start a test backend which uses the same credentials as the RLS server,
	// and set up the fake RLS server to return this as the target in the RLS
	// response.
	backendCh, backendAddress := startBackend(t, grpc.Creds(serverCreds))
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress}}}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	// Dial with credentials and expect the RLS server to receive the same. The
	// server certificate used for the RLS server and the backend specifies a
	// DNS SAN of "*.test.example.com". Hence we use a dial target which is a
	// subdomain of the same here.
	cc, err := grpc.Dial(r.Scheme()+":///rls.test.example.com", grpc.WithResolvers(r), grpc.WithTransportCredentials(clientCreds))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure it gets routed to the test backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Ensure a connection is established to the first RLS server.
	if _, err := lis.NewConnCh.Receive(ctx); err != nil {
		t.Fatal("Timeout expired when waiting for LB policy to create control channel")
	}
}

// TestConfigUpdate_ControlChannelServiceConfig tests the scenario where RLS LB
// policy's configuration specifies the service config for the control channel
// via the `routeLookupChannelServiceConfig` field. This test verifies that the
// provided service config is applied for the control channel.
func (s) TestConfigUpdate_ControlChannelServiceConfig(t *testing.T) {
	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Register a balancer to be used for the control channel, and set up a
	// callback to get notified when the balancer receives a clientConn updates.
	ccUpdateCh := testutils.NewChannel()
	bf := &e2e.BalancerFuncs{
		UpdateClientConnState: func(cfg *e2e.RLSChildPolicyConfig) error {
			if cfg.Backend != rlsServer.Address {
				return fmt.Errorf("control channel LB policy received config with backend %q, want %q", cfg.Backend, rlsServer.Address)
			}
			ccUpdateCh.Replace(nil)
			return nil
		},
	}
	controlChannelPolicyName := "test-control-channel-" + t.Name()
	e2e.RegisterRLSChildPolicy(controlChannelPolicyName, bf)
	t.Logf("Registered child policy with name %q", controlChannelPolicyName)

	// Build RLS service config and set the `routeLookupChannelServiceConfig`
	// field to a service config which uses the above balancer.
	rlsConfig := buildBasicRLSConfigWithChildPolicy(t, t.Name(), rlsServer.Address)
	rlsConfig.RouteLookupChannelServiceConfig = fmt.Sprintf(`{"loadBalancingConfig" : [{%q: {"backend": %q} }]}`, controlChannelPolicyName, rlsServer.Address)

	// Start a test backend, and set up the fake RLS server to return this as a
	// target in the RLS response.
	backendCh, backendAddress := startBackend(t)
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress}}}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///rls.test.example.com", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure it gets routed to the test backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Verify that the control channel is using the LB policy we injected via the
	// routeLookupChannelServiceConfig field.
	if _, err := ccUpdateCh.Receive(ctx); err != nil {
		t.Fatalf("timeout when waiting for control channel LB policy to receive a clientConn update")
	}
}

// TestConfigUpdate_DefaultTarget tests the scenario where a config update
// changes the default target. Verifies that RPCs get routed to the new default
// target after the config has been applied.
func (s) TestConfigUpdate_DefaultTarget(t *testing.T) {
	// Start an RLS server and set the throttler to always throttle requests.
	rlsServer, _ := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, alwaysThrottlingThrottler())

	// Build RLS service config with a default target.
	rlsConfig := buildBasicRLSConfigWithChildPolicy(t, t.Name(), rlsServer.Address)
	backendCh1, backendAddress1 := startBackend(t)
	rlsConfig.RouteLookupConfig.DefaultTarget = backendAddress1

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure it gets routed to the default target.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh1)

	// Change default_target field of the RLS config.
	backendCh2, backendAddress2 := startBackend(t)
	rlsConfig.RouteLookupConfig.DefaultTarget = backendAddress2

	// Push the config update through the manual resolver.
	scJSON, err := rlsConfig.ServiceConfigJSON()
	if err != nil {
		t.Fatal(err)
	}
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.UpdateState(resolver.State{ServiceConfig: sc})
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh2)
}

// TestConfigUpdate_ChildPolicyConfigs verifies that config changes which affect
// child policy configuration are propagated correctly.
func (s) TestConfigUpdate_ChildPolicyConfigs(t *testing.T) {
	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Start a default backend and a test backend.
	_, defBackendAddress := startBackend(t)
	testBackendCh, testBackendAddress := startBackend(t)

	// Set up the RLS server to respond with the test backend.
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{testBackendAddress}}}
	})

	// Set up a test balancer callback to push configs received by child policies.
	defBackendConfigsCh := make(chan *e2e.RLSChildPolicyConfig, 1)
	testBackendConfigsCh := make(chan *e2e.RLSChildPolicyConfig, 1)
	bf := &e2e.BalancerFuncs{
		UpdateClientConnState: func(cfg *e2e.RLSChildPolicyConfig) error {
			switch cfg.Backend {
			case defBackendAddress:
				defBackendConfigsCh <- cfg
			case testBackendAddress:
				testBackendConfigsCh <- cfg
			default:
				t.Errorf("Received child policy configs for unknown target %q", cfg.Backend)
			}
			return nil
		},
	}

	// Register an LB policy to act as the child policy for RLS LB policy.
	childPolicyName := "test-child-policy" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName, bf)
	t.Logf("Registered child policy with name %q", childPolicyName)

	// Build RLS service config with default target.
	rlsConfig := buildBasicRLSConfig(childPolicyName, rlsServer.Address)
	rlsConfig.RouteLookupConfig.DefaultTarget = defBackendAddress

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// At this point, the RLS LB policy should have received its config, and
	// should have created a child policy for the default target.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantCfg := &e2e.RLSChildPolicyConfig{Backend: defBackendAddress}
	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the default target child policy to receive its config")
	case gotCfg := <-defBackendConfigsCh:
		if !cmp.Equal(gotCfg, wantCfg) {
			t.Fatalf("Default target child policy received config %+v, want %+v", gotCfg, wantCfg)
		}
	}

	// Make an RPC and ensure it gets routed to the test backend.
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, testBackendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// As part of handling the above RPC, the RLS LB policy should have created
	// a child policy for the test target.
	wantCfg = &e2e.RLSChildPolicyConfig{Backend: testBackendAddress}
	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the test target child policy to receive its config")
	case gotCfg := <-testBackendConfigsCh:
		if !cmp.Equal(gotCfg, wantCfg) {
			t.Fatalf("Test target child policy received config %+v, want %+v", gotCfg, wantCfg)
		}
	}

	// Push an RLS config update with a change in the child policy config.
	childPolicyBuilder := balancer.Get(childPolicyName)
	childPolicyParser := childPolicyBuilder.(balancer.ConfigParser)
	lbCfg, err := childPolicyParser.ParseConfig([]byte(`{"Random": "random"}`))
	if err != nil {
		t.Fatal(err)
	}
	rlsConfig.ChildPolicy.Config = lbCfg
	scJSON, err := rlsConfig.ServiceConfigJSON()
	if err != nil {
		t.Fatal(err)
	}
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.UpdateState(resolver.State{ServiceConfig: sc})

	// Expect the child policy for the test backend to receive the update.
	wantCfg = &e2e.RLSChildPolicyConfig{
		Backend: testBackendAddress,
		Random:  "random",
	}
	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the test target child policy to receive its config")
	case gotCfg := <-testBackendConfigsCh:
		if !cmp.Equal(gotCfg, wantCfg) {
			t.Fatalf("Test target child policy received config %+v, want %+v", gotCfg, wantCfg)
		}
	}

	// Expect the child policy for the default backend to receive the update.
	wantCfg = &e2e.RLSChildPolicyConfig{
		Backend: defBackendAddress,
		Random:  "random",
	}
	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the default target child policy to receive its config")
	case gotCfg := <-defBackendConfigsCh:
		if !cmp.Equal(gotCfg, wantCfg) {
			t.Fatalf("Default target child policy received config %+v, want %+v", gotCfg, wantCfg)
		}
	}
}

// TestConfigUpdate_ChildPolicyChange verifies that a child policy change is
// handled by closing the old balancer and creating a new one.
func (s) TestConfigUpdate_ChildPolicyChange(t *testing.T) {
	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, _ := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Set up balancer callbacks.
	configsCh1 := make(chan *e2e.RLSChildPolicyConfig, 1)
	closeCh1 := make(chan struct{}, 1)
	bf := &e2e.BalancerFuncs{
		UpdateClientConnState: func(cfg *e2e.RLSChildPolicyConfig) error {
			configsCh1 <- cfg
			return nil
		},
		Close: func() {
			closeCh1 <- struct{}{}
		},
	}

	// Register an LB policy to act as the child policy for RLS LB policy.
	childPolicyName1 := "test-child-policy-1" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName1, bf)
	t.Logf("Registered child policy with name %q", childPolicyName1)

	// Build RLS service config with a dummy default target.
	const defaultBackend = "default-backend"
	rlsConfig := buildBasicRLSConfig(childPolicyName1, rlsServer.Address)
	rlsConfig.RouteLookupConfig.DefaultTarget = defaultBackend

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// At this point, the RLS LB policy should have received its config, and
	// should have created a child policy for the default target.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantCfg := &e2e.RLSChildPolicyConfig{Backend: defaultBackend}
	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the first child policy to receive its config")
	case gotCfg := <-configsCh1:
		if !cmp.Equal(gotCfg, wantCfg) {
			t.Fatalf("First child policy received config %+v, want %+v", gotCfg, wantCfg)
		}
	}

	// Set up balancer callbacks for the second policy.
	configsCh2 := make(chan *e2e.RLSChildPolicyConfig, 1)
	bf = &e2e.BalancerFuncs{
		UpdateClientConnState: func(cfg *e2e.RLSChildPolicyConfig) error {
			configsCh2 <- cfg
			return nil
		},
	}

	// Register a second LB policy to act as the child policy for RLS LB policy.
	childPolicyName2 := "test-child-policy-2" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName2, bf)
	t.Logf("Registered child policy with name %q", childPolicyName2)

	// Push an RLS config update with a change in the child policy name.
	rlsConfig.ChildPolicy = &internalserviceconfig.BalancerConfig{Name: childPolicyName2}
	scJSON, err := rlsConfig.ServiceConfigJSON()
	if err != nil {
		t.Fatal(err)
	}
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.UpdateState(resolver.State{ServiceConfig: sc})

	// The above update should result in the first LB policy being shutdown and
	// the second LB policy receiving a config update.
	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the first child policy to be shutdown")
	case <-closeCh1:
	}

	select {
	case <-ctx.Done():
		t.Fatal("Timed out when waiting for the second child policy to receive its config")
	case gotCfg := <-configsCh2:
		if !cmp.Equal(gotCfg, wantCfg) {
			t.Fatalf("First child policy received config %+v, want %+v", gotCfg, wantCfg)
		}
	}
}

// TestConfigUpdate_BadChildPolicyConfigs tests the scenario where a config
// update is rejected by the child policy. Verifies that the child policy
// wrapper goes "lame" and the error from the child policy is reported back to
// the caller of the RPC.
func (s) TestConfigUpdate_BadChildPolicyConfigs(t *testing.T) {
	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Set up the RLS server to respond with a bad target field which is expected
	// to cause the child policy's ParseTarget to fail and should result in the LB
	// policy creating a lame child policy wrapper.
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{e2e.RLSChildPolicyBadTarget}}}
	})

	// Build RLS service config with a default target. This default backend is
	// expected to be healthy (even though we don't attempt to route RPCs to it)
	// and ensures that the overall connectivity state of the RLS LB policy is not
	// TRANSIENT_FAILURE. This is required to make sure that the pick for the bad
	// child policy actually gets delegated to the child policy picker.
	rlsConfig := buildBasicRLSConfigWithChildPolicy(t, t.Name(), rlsServer.Address)
	_, addr := startBackend(t)
	rlsConfig.RouteLookupConfig.DefaultTarget = addr

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure that if fails with the expected error.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	makeTestRPCAndVerifyError(ctx, t, cc, codes.Unavailable, e2e.ErrParseConfigBadTarget)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)
}

// TestConfigUpdate_DataCacheSizeDecrease tests the scenario where a config
// update decreases the data cache size. Verifies that entries are evicted from
// the cache.
func (s) TestConfigUpdate_DataCacheSizeDecrease(t *testing.T) {
	// Override the clientConn update hook to get notified.
	clientConnUpdateDone := make(chan struct{}, 1)
	origClientConnUpdateHook := clientConnUpdateHook
	clientConnUpdateHook = func() { clientConnUpdateDone <- struct{}{} }
	defer func() { clientConnUpdateHook = origClientConnUpdateHook }()

	// Override the cache entry size func, and always return 1.
	origEntrySizeFunc := computeDataCacheEntrySize
	computeDataCacheEntrySize = func(cacheKey, *cacheEntry) int64 { return 1 }
	defer func() { computeDataCacheEntrySize = origEntrySizeFunc }()

	// Override the minEvictionDuration to ensure that when the config update
	// reduces the cache size, the resize operation is not stopped because
	// we find an entry whose minExpiryDuration has not elapsed.
	origMinEvictDuration := minEvictDuration
	minEvictDuration = time.Duration(0)
	defer func() { minEvictDuration = origMinEvictDuration }()

	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Register an LB policy to act as the child policy for RLS LB policy.
	childPolicyName := "test-child-policy" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName, nil)
	t.Logf("Registered child policy with name %q", childPolicyName)

	// Build RLS service config with header matchers.
	rlsConfig := buildBasicRLSConfig(childPolicyName, rlsServer.Address)

	// Start a couple of test backends, and set up the fake RLS server to return
	// these as targets in the RLS response, based on request keys.
	backendCh1, backendAddress1 := startBackend(t)
	backendCh2, backendAddress2 := startBackend(t)
	rlsServer.SetResponseCallback(func(_ context.Context, req *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		if req.KeyMap["k1"] == "v1" {
			return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress1}}}
		}
		if req.KeyMap["k2"] == "v2" {
			return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress2}}}
		}
		return &rlstest.RouteLookupResponse{Err: errors.New("no keys in request metadata")}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	<-clientConnUpdateDone

	// Make an RPC and ensure it gets routed to the first backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctxOutgoing := metadata.AppendToOutgoingContext(ctx, "n1", "v1")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh1)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Make another RPC with a different set of headers. This will force the LB
	// policy to send out a new RLS request, resulting in a new data cache
	// entry.
	ctxOutgoing = metadata.AppendToOutgoingContext(ctx, "n2", "v2")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh2)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// We currently have two cache entries. Setting the size to 1, will cause
	// the entry corresponding to backend1 to be evicted.
	rlsConfig.RouteLookupConfig.CacheSizeBytes = 1

	// Push the config update through the manual resolver.
	scJSON, err := rlsConfig.ServiceConfigJSON()
	if err != nil {
		t.Fatal(err)
	}
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.UpdateState(resolver.State{ServiceConfig: sc})

	<-clientConnUpdateDone

	// Make an RPC to match the cache entry which got evicted above, and expect
	// an RLS request to be made to fetch the targets.
	ctxOutgoing = metadata.AppendToOutgoingContext(ctx, "n1", "v1")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh1)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)
}

// Test that when a data cache entry is evicted due to config change
// in cache size, the picker is updated accordingly.
func (s) TestPickerUpdateOnDataCacheSizeDecrease(t *testing.T) {
	// Override the clientConn update hook to get notified.
	clientConnUpdateDone := make(chan struct{}, 1)
	origClientConnUpdateHook := clientConnUpdateHook
	clientConnUpdateHook = func() { clientConnUpdateDone <- struct{}{} }
	defer func() { clientConnUpdateHook = origClientConnUpdateHook }()

	// Override the cache entry size func, and always return 1.
	origEntrySizeFunc := computeDataCacheEntrySize
	computeDataCacheEntrySize = func(cacheKey, *cacheEntry) int64 { return 1 }
	defer func() { computeDataCacheEntrySize = origEntrySizeFunc }()

	// Override the backoff strategy to return a large backoff which
	// will make sure the date cache entry remains in backoff for the
	// duration of the test.
	origBackoffStrategy := defaultBackoffStrategy
	defaultBackoffStrategy = &fakeBackoffStrategy{backoff: defaultTestTimeout}
	defer func() { defaultBackoffStrategy = origBackoffStrategy }()

	// Override the minEvictionDuration to ensure that when the config update
	// reduces the cache size, the resize operation is not stopped because
	// we find an entry whose minExpiryDuration has not elapsed.
	origMinEvictDuration := minEvictDuration
	minEvictDuration = time.Duration(0)
	defer func() { minEvictDuration = origMinEvictDuration }()

	// Register the top-level wrapping balancer which forwards calls to RLS.
	topLevelBalancerName := t.Name() + "top-level"
	var ccWrapper *testCCWrapper
	stub.Register(topLevelBalancerName, stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			ccWrapper = &testCCWrapper{ClientConn: bd.ClientConn}
			bd.Data = balancer.Get(Name).Build(ccWrapper, bd.BuildOptions)
		},
		ParseConfig: func(sc json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			parser := balancer.Get(Name).(balancer.ConfigParser)
			return parser.ParseConfig(sc)
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

	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Register an LB policy to act as the child policy for RLS LB policy.
	childPolicyName := "test-child-policy" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName, nil)
	t.Logf("Registered child policy with name %q", childPolicyName)

	// Start a couple of test backends, and set up the fake RLS server to return
	// these as targets in the RLS response, based on request keys.
	// Start a couple of test backends, and set up the fake RLS server to return
	// these as targets in the RLS response, based on request keys.
	backendCh1, backendAddress1 := startBackend(t)
	backendCh2, backendAddress2 := startBackend(t)
	rlsServer.SetResponseCallback(func(_ context.Context, req *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		if req.KeyMap["k1"] == "v1" {
			return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress1}}}
		}
		if req.KeyMap["k2"] == "v2" {
			return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress2}}}
		}
		return &rlstest.RouteLookupResponse{Err: errors.New("no keys in request metadata")}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := manual.NewBuilderWithScheme("rls-e2e")
	headers := `
    [
        {
            "key": "k1",
            "names": [
                "n1"
            ]
        },
        {
            "key": "k2",
            "names": [
                "n2"
            ]
        }
    ]
    `

	configJSON := `
	{
	  "loadBalancingConfig": [
		{
		  "%s": {
			"routeLookupConfig": {
				"grpcKeybuilders": [{
					"names": [{"service": "grpc.testing.TestService"}],
					"headers": %s
				}],
				"lookupService": "%s",
				"cacheSizeBytes": %d
			},
			"childPolicy": [{"%s": {}}],
			"childPolicyConfigTargetFieldName": "Backend"
		  }
		}
	  ]
	}`
	scJSON := fmt.Sprintf(configJSON, topLevelBalancerName, headers, rlsServer.Address, 1000, childPolicyName)
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.InitialState(resolver.State{ServiceConfig: sc})

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("create grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	<-clientConnUpdateDone

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Make an RPC call with empty metadata, which will eventually throw
	// the error as no metadata will match from rlsServer response
	// callback defined above. This will cause the control channel to
	// throw the error and cause the item to get into backoff.
	makeTestRPCAndVerifyError(ctx, t, cc, codes.Unavailable, nil)

	ctxOutgoing := metadata.AppendToOutgoingContext(ctx, "n1", "v1")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh1)
	verifyRLSRequest(t, rlsReqCh, true)

	ctxOutgoing = metadata.AppendToOutgoingContext(ctx, "n2", "v2")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh2)
	verifyRLSRequest(t, rlsReqCh, true)

	initialStateCnt := len(ccWrapper.getStates())
	// Setting the size to 1 will cause the entries to be
	// evicted.
	scJSON1 := fmt.Sprintf(`
{
  "loadBalancingConfig": [
    {
      "%s": {
		"routeLookupConfig": {
			"grpcKeybuilders": [{
				"names": [{"service": "grpc.testing.TestService"}],
				"headers": %s
			}],
			"lookupService": "%s",
			"cacheSizeBytes": 2
		},
		"childPolicy": [{"%s": {}}],
		"childPolicyConfigTargetFieldName": "Backend"
      }
    }
  ]
}`, topLevelBalancerName, headers, rlsServer.Address, childPolicyName)
	sc1 := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON1)
	r.UpdateState(resolver.State{ServiceConfig: sc1})
	<-clientConnUpdateDone
	finalStateCnt := len(ccWrapper.getStates())

	if finalStateCnt != initialStateCnt+1 {
		t.Errorf("Unexpected balancer state count: got %v, want %v", finalStateCnt, initialStateCnt)
	}
}

// TestDataCachePurging verifies that the LB policy periodically evicts expired
// entries from the data cache.
func (s) TestDataCachePurging(t *testing.T) {
	// Override the frequency of the data cache purger to a small one.
	origDataCachePurgeTicker := dataCachePurgeTicker
	ticker := time.NewTicker(defaultTestShortTimeout)
	defer ticker.Stop()
	dataCachePurgeTicker = func() *time.Ticker { return ticker }
	defer func() { dataCachePurgeTicker = origDataCachePurgeTicker }()

	// Override the data cache purge hook to get notified.
	dataCachePurgeDone := make(chan struct{}, 1)
	origDataCachePurgeHook := dataCachePurgeHook
	dataCachePurgeHook = func() { dataCachePurgeDone <- struct{}{} }
	defer func() { dataCachePurgeHook = origDataCachePurgeHook }()

	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Register an LB policy to act as the child policy for RLS LB policy.
	childPolicyName := "test-child-policy" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName, nil)
	t.Logf("Registered child policy with name %q", childPolicyName)

	// Build RLS service config with header matchers and lookupService pointing to
	// the fake RLS server created above. Set a very low value for maxAge to
	// ensure that the entry expires soon.
	rlsConfig := buildBasicRLSConfig(childPolicyName, rlsServer.Address)
	rlsConfig.RouteLookupConfig.MaxAge = durationpb.New(time.Millisecond)

	// Start a test backend, and set up the fake RLS server to return this as a
	// target in the RLS response.
	backendCh, backendAddress := startBackend(t)
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress}}}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure it gets routed to the test backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctxOutgoing := metadata.AppendToOutgoingContext(ctx, "n1", "v1")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Make another RPC with different headers. This will force the LB policy to
	// send out a new RLS request, resulting in a new data cache entry.
	ctxOutgoing = metadata.AppendToOutgoingContext(ctx, "n2", "v2")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Wait for the data cache purging to happen before proceeding.
	<-dataCachePurgeDone

	// Perform the same RPCs again and verify that they result in RLS requests.
	ctxOutgoing = metadata.AppendToOutgoingContext(ctx, "n1", "v1")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Make another RPC with different headers. This will force the LB policy to
	// send out a new RLS request, resulting in a new data cache entry.
	ctxOutgoing = metadata.AppendToOutgoingContext(ctx, "n2", "v2")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)
}

// TestControlChannelConnectivityStateMonitoring tests the scenario where the
// control channel goes down and comes back up again and verifies that backoff
// state is reset for cache entries in this scenario.
func (s) TestControlChannelConnectivityStateMonitoring(t *testing.T) {
	// Create a restartable listener which can close existing connections.
	l, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("net.Listen() failed: %v", err)
	}
	lis := testutils.NewRestartableListener(l)

	// Start an RLS server with the restartable listener and set the throttler to
	// never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, lis)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Override the reset backoff hook to get notified.
	resetBackoffDone := make(chan struct{}, 1)
	origResetBackoffHook := resetBackoffHook
	resetBackoffHook = func() { resetBackoffDone <- struct{}{} }
	defer func() { resetBackoffHook = origResetBackoffHook }()

	// Override the backoff strategy to return a large backoff which
	// will make sure the date cache entry remains in backoff for the
	// duration of the test.
	origBackoffStrategy := defaultBackoffStrategy
	defaultBackoffStrategy = &fakeBackoffStrategy{backoff: defaultTestTimeout}
	defer func() { defaultBackoffStrategy = origBackoffStrategy }()

	// Register an LB policy to act as the child policy for RLS LB policy.
	childPolicyName := "test-child-policy" + t.Name()
	e2e.RegisterRLSChildPolicy(childPolicyName, nil)
	t.Logf("Registered child policy with name %q", childPolicyName)

	// Build RLS service config with header matchers, and a very low value for
	// maxAge to ensure that cache entries become invalid very soon.
	rlsConfig := buildBasicRLSConfig(childPolicyName, rlsServer.Address)
	rlsConfig.RouteLookupConfig.MaxAge = durationpb.New(defaultTestShortTimeout)

	// Start a test backend, and set up the fake RLS server to return this as a
	// target in the RLS response.
	backendCh, backendAddress := startBackend(t)
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{backendAddress}}}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := startManualResolverWithConfig(t, rlsConfig)

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Make an RPC and ensure it gets routed to the test backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Stop the RLS server.
	lis.Stop()

	// Make another RPC similar to the first one. Since the above cache entry
	// would have expired by now, this should trigger another RLS request. And
	// since the RLS server is down, RLS request will fail and the cache entry
	// will enter backoff, and we have overridden the default backoff strategy to
	// return a value which will keep this entry in backoff for the whole duration
	// of the test.
	makeTestRPCAndVerifyError(ctx, t, cc, codes.Unavailable, nil)

	// Restart the RLS server.
	lis.Restart()

	// When we closed the RLS server earlier, the existing transport to the RLS
	// server would have closed, and the RLS control channel would have moved to
	// TRANSIENT_FAILURE with a subConn backoff before moving to IDLE. This
	// backoff will last for about a second. We need to keep retrying RPCs for the
	// subConn to eventually come out of backoff and attempt to reconnect.
	//
	// Make this RPC with a different set of headers leading to the creation of
	// a new cache entry and a new RLS request. This RLS request will also fail
	// till the control channel comes moves back to READY. So, override the
	// backoff strategy to perform a small backoff on this entry.
	defaultBackoffStrategy = &fakeBackoffStrategy{backoff: defaultTestShortTimeout}
	ctxOutgoing := metadata.AppendToOutgoingContext(ctx, "n1", "v1")
	makeTestRPCAndExpectItToReachBackend(ctxOutgoing, t, cc, backendCh)

	select {
	case <-ctx.Done():
		t.Fatalf("Timed out waiting for resetBackoffDone")
	case <-resetBackoffDone:
	}

	// The fact that the above RPC succeeded indicates that the control channel
	// has moved back to READY. The connectivity state monitoring code should have
	// realized this and should have reset all backoff timers (which in this case
	// is the cache entry corresponding to the first RPC). Retrying that RPC now
	// should succeed with an RLS request being sent out.
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, backendCh)
	verifyRLSRequest(t, rlsReqCh, true)
}

// testCCWrapper wraps a balancer.ClientConn and overrides UpdateState and
// stores all state updates pushed by the RLS LB policy.
type testCCWrapper struct {
	balancer.ClientConn

	mu     sync.Mutex
	states []balancer.State
}

func (t *testCCWrapper) UpdateState(bs balancer.State) {
	t.mu.Lock()
	t.states = append(t.states, bs)
	t.mu.Unlock()
	t.ClientConn.UpdateState(bs)
}

func (t *testCCWrapper) getStates() []balancer.State {
	t.mu.Lock()
	defer t.mu.Unlock()

	states := make([]balancer.State, len(t.states))
	copy(states, t.states)
	return states
}

// TestUpdateStatePauses tests the scenario where a config update received by
// the RLS LB policy results in multiple UpdateState calls from the child
// policies. This test verifies that picker updates are paused when the config
// update is being processed by RLS LB policy and its child policies.
//
// The test uses a wrapping balancer as the top-level LB policy on the channel.
// The wrapping balancer wraps an RLS LB policy as a child policy and forwards
// all calls to it. It also records the UpdateState() calls from the RLS LB
// policy and makes it available for inspection by the test.
//
// The test uses another wrapped balancer (which wraps a pickfirst balancer) as
// the child policy of the RLS LB policy. This balancer makes multiple
// UpdateState calls when handling an update from its parent in
// UpdateClientConnState.
func (s) TestUpdateStatePauses(t *testing.T) {
	// Override the hook to get notified when UpdateClientConnState is done.
	clientConnUpdateDone := make(chan struct{}, 1)
	origClientConnUpdateHook := clientConnUpdateHook
	clientConnUpdateHook = func() { clientConnUpdateDone <- struct{}{} }
	defer func() { clientConnUpdateHook = origClientConnUpdateHook }()

	// Register the top-level wrapping balancer which forwards calls to RLS.
	topLevelBalancerName := t.Name() + "top-level"
	var ccWrapper *testCCWrapper
	stub.Register(topLevelBalancerName, stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			ccWrapper = &testCCWrapper{ClientConn: bd.ClientConn}
			bd.Data = balancer.Get(Name).Build(ccWrapper, bd.BuildOptions)
		},
		ParseConfig: func(sc json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			parser := balancer.Get(Name).(balancer.ConfigParser)
			return parser.ParseConfig(sc)
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

	// Register a child policy that wraps a pickfirst balancer and makes multiple calls
	// to UpdateState when handling a config update in UpdateClientConnState. When
	// this policy is used as a child policy of the RLS LB policy, it is expected
	// that the latter suppress these updates and push a single picker update on the
	// channel (after the config has been processed by all child policies).
	childPolicyName := t.Name() + "child"
	type childPolicyConfig struct {
		serviceconfig.LoadBalancingConfig
		Backend string // `json:"backend,omitempty"`
	}
	stub.Register(childPolicyName, stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			bd.Data = balancer.Get(pickfirst.Name).Build(bd.ClientConn, bd.BuildOptions)
		},
		ParseConfig: func(sc json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			cfg := &childPolicyConfig{}
			if err := json.Unmarshal(sc, cfg); err != nil {
				return nil, err
			}
			return cfg, nil
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			bal := bd.Data.(balancer.Balancer)
			bd.ClientConn.UpdateState(balancer.State{ConnectivityState: connectivity.Idle, Picker: &testutils.TestConstPicker{Err: balancer.ErrNoSubConnAvailable}})
			bd.ClientConn.UpdateState(balancer.State{ConnectivityState: connectivity.Connecting, Picker: &testutils.TestConstPicker{Err: balancer.ErrNoSubConnAvailable}})

			cfg := ccs.BalancerConfig.(*childPolicyConfig)
			return bal.UpdateClientConnState(balancer.ClientConnState{
				ResolverState: resolver.State{Addresses: []resolver.Address{{Addr: cfg.Backend}}},
			})
		},
	})

	// Start an RLS server and set the throttler to never throttle requests.
	rlsServer, rlsReqCh := rlstest.SetupFakeRLSServer(t, nil)
	overrideAdaptiveThrottler(t, neverThrottlingThrottler())

	// Start a test backend and set the RLS server to respond with it.
	testBackendCh, testBackendAddress := startBackend(t)
	rlsServer.SetResponseCallback(func(_ context.Context, _ *rlspb.RouteLookupRequest) *rlstest.RouteLookupResponse {
		return &rlstest.RouteLookupResponse{Resp: &rlspb.RouteLookupResponse{Targets: []string{testBackendAddress}}}
	})

	// Register a manual resolver and push the RLS service config through it.
	r := manual.NewBuilderWithScheme("rls-e2e")
	scJSON := fmt.Sprintf(`
{
  "loadBalancingConfig": [
    {
      "%s": {
		"routeLookupConfig": {
			"grpcKeybuilders": [{
				"names": [{"service": "grpc.testing.TestService"}]
			}],
			"lookupService": "%s",
			"cacheSizeBytes": 1000
		},
		"childPolicy": [{"%s": {}}],
		"childPolicyConfigTargetFieldName": "Backend"
      }
    }
  ]
}`, topLevelBalancerName, rlsServer.Address, childPolicyName)
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.InitialState(resolver.State{ServiceConfig: sc})

	cc, err := grpc.Dial(r.Scheme()+":///", grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()

	// Wait for the clientconn update to be processed by the RLS LB policy.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
	case <-clientConnUpdateDone:
	}

	// It is important to note that at this point no child policies have been
	// created because we have not attempted any RPC so far. When we attempt an
	// RPC (below), child policies will be created and their configs will be
	// pushed to them. But this config update will not happen in the context of
	// a config update on the parent.

	// Make an RPC and ensure it gets routed to the test backend.
	makeTestRPCAndExpectItToReachBackend(ctx, t, cc, testBackendCh)

	// Make sure an RLS request is sent out.
	verifyRLSRequest(t, rlsReqCh, true)

	// Wait for the control channel to become READY, before reading the states
	// out of the wrapping top-level balancer.
	//
	// makeTestRPCAndExpectItToReachBackend repeatedly sends RPCs with short
	// deadlines until one succeeds. See its docstring for details.
	//
	// The following sequence of events is possible:
	// 1. When the first RPC is attempted above, a pending cache entry is
	//    created, an RLS request is sent out, and the pick is queued. The
	//    channel is in CONNECTING state.
	// 2. When the RLS response arrives, the pending cache entry is moved to the
	//    data cache, a child policy is created for the target specified in the
	//    response and a new picker is returned. The channel is still in
	//    CONNECTING, and retried pick is again queued.
	// 3. The child policy moves through the standard set of states, IDLE -->
	//    CONNECTING --> READY. And for each of these state changes, a new
	//    picker is sent on the channel. But the overall connectivity state of
	//    the channel is still CONNECTING.
	// 4. Right around the time when the child policy becomes READY, the
	//    deadline associated with the first RPC made by
	//    makeTestRPCAndExpectItToReachBackend() could expire, and it could send
	//    a new one. And because the internal state of the LB policy now
	//    contains a child policy which is READY, this RPC will succeed. But the
	//    RLS LB policy has yet to push a new picker on the channel.
	// 5. If we read the states seen by the top-level wrapping LB policy without
	//    waiting for the channel to become READY, there is a possibility that we
	//    might not see the READY state in there. And if that happens, we will
	//    see two extra states in the last check made in the test, and thereby
	//    the test would fail. Waiting for the channel to become READY here
	//    ensures that the test does not flake because of this rare sequence of
	//    events.
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)

	// Cache the state changes seen up to this point.
	states0 := ccWrapper.getStates()

	// Push an updated service config. As mentioned earlier, the previous config
	// updates on the child policies did not happen in the context of a config
	// update on the parent. Hence, this update is required to force the
	// scenario which we are interesting in testing here, i.e child policies get
	// config updates as part of the parent policy getting its config update.
	scJSON = fmt.Sprintf(`
{
  "loadBalancingConfig": [
    {
      "%s": {
		"routeLookupConfig": {
			"grpcKeybuilders": [{
				"names": [
					{"service": "grpc.testing.TestService"},
					{"service": "grpc.health.v1.Health"}
				]
			}],
			"lookupService": "%s",
			"cacheSizeBytes": 1000
		},
		"childPolicy": [{"%s": {}}],
		"childPolicyConfigTargetFieldName": "Backend"
      }
    }
  ]
}`, topLevelBalancerName, rlsServer.Address, childPolicyName)
	sc = internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(scJSON)
	r.UpdateState(resolver.State{ServiceConfig: sc})

	// Wait for the clientconn update to be processed by the RLS LB policy.
	select {
	case <-ctx.Done():
	case <-clientConnUpdateDone:
	}

	// Even though the child policies used in this test make multiple calls to
	// UpdateState as part of handling their configs, we expect the RLS policy
	// to inhibit picker updates during this time frame, and send a single
	// picker once the config update is completely handled.
	states1 := ccWrapper.getStates()
	if len(states1) != len(states0)+1 {
		t.Fatalf("more than one state update seen. before %v, after %v", states0, states1)
	}
}
