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

package test

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/pickfirst"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// TestResolverUpdateDuringBuild_ServiceConfigParseError makes the
// resolver.Builder call into the ClientConn, during the Build call, with a
// service config parsing error.
//
// We use two separate mutexes in the code which make sure there is no data race
// in this code path, and also that there is no deadlock.
func (s) TestResolverUpdateDuringBuild_ServiceConfigParseError(t *testing.T) {
	// Setting InitialState on the manual resolver makes it call into the
	// ClientConn during the Build call.
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{ServiceConfig: &serviceconfig.ParseResult{Err: errors.New("resolver build err")}})

	cc, err := grpc.NewClient(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("NewClient(_, _) = _, %v; want _, nil", err)
	}
	defer cc.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	const wantMsg = "error parsing service config"
	const wantCode = codes.Unavailable
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != wantCode || !strings.Contains(status.Convert(err).Message(), wantMsg) {
		t.Fatalf("EmptyCall RPC failed: %v; want code: %v, want message: %q", err, wantCode, wantMsg)
	}
}

type fakeConfig struct {
	serviceconfig.Config
}

// TestResolverUpdateDuringBuild_ServiceConfigInvalidTypeError makes the
// resolver.Builder call into the ClientConn, during the Build call, with an
// invalid service config type.
//
// We use two separate mutexes in the code which make sure there is no data race
// in this code path, and also that there is no deadlock.
func (s) TestResolverUpdateDuringBuild_ServiceConfigInvalidTypeError(t *testing.T) {
	// Setting InitialState on the manual resolver makes it call into the
	// ClientConn during the Build call.
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{ServiceConfig: &serviceconfig.ParseResult{Config: fakeConfig{}}})

	cc, err := grpc.NewClient(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("NewClient(_, _) = _, %v; want _, nil", err)
	}
	defer cc.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	const wantMsg = "illegal service config type"
	const wantCode = codes.Unavailable
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != wantCode || !strings.Contains(status.Convert(err).Message(), wantMsg) {
		t.Fatalf("EmptyCall RPC failed: %v; want code: %v, want message: %q", err, wantCode, wantMsg)
	}
}

// TestResolverUpdate_InvalidServiceConfigAsFirstUpdate makes the resolver send
// an update with an invalid service config as its first update. This should
// make the ClientConn apply the failing LB policy, and should result in RPC
// errors indicating the failing service config.
func (s) TestResolverUpdate_InvalidServiceConfigAsFirstUpdate(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")

	cc, err := grpc.Dial(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("Dial(_, _) = _, %v; want _, nil", err)
	}
	defer cc.Close()

	scpr := r.CC.ParseServiceConfig("bad json service config")
	r.UpdateState(resolver.State{ServiceConfig: scpr})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	const wantMsg = "error parsing service config"
	const wantCode = codes.Unavailable
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != wantCode || !strings.Contains(status.Convert(err).Message(), wantMsg) {
		t.Fatalf("EmptyCall RPC failed: %v; want code: %v, want message: %q", err, wantCode, wantMsg)
	}
}

func verifyClientConnStateUpdate(got, want balancer.ClientConnState) error {
	if got, want := got.ResolverState.Addresses, want.ResolverState.Addresses; !cmp.Equal(got, want) {
		return fmt.Errorf("update got unexpected addresses: %v, want %v", got, want)
	}
	if got, want := got.ResolverState.ServiceConfig.Config, want.ResolverState.ServiceConfig.Config; !internal.EqualServiceConfigForTesting(got, want) {
		return fmt.Errorf("received unexpected service config: \ngot: %v \nwant: %v", got, want)
	}
	if got, want := got.BalancerConfig, want.BalancerConfig; !cmp.Equal(got, want) {
		return fmt.Errorf("received unexpected balancer config: \ngot: %v \nwant: %v", cmp.Diff(nil, got), cmp.Diff(nil, want))
	}
	return nil
}

// TestResolverUpdate_InvalidServiceConfigAfterGoodUpdate tests the scenario
// where the resolver sends an update with an invalid service config after
// having sent a good update. This should result in the ClientConn discarding
// the new invalid service config, and continuing to use the old good config.
func (s) TestResolverUpdate_InvalidServiceConfigAfterGoodUpdate(t *testing.T) {
	type wrappingBalancerConfig struct {
		serviceconfig.LoadBalancingConfig
		Config string `json:"config,omitempty"`
	}

	// Register a stub balancer which uses a "pick_first" balancer underneath and
	// signals on a channel when it receives ClientConn updates.
	ccUpdateCh := testutils.NewChannel()
	stub.Register(t.Name(), stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			pf := balancer.Get(pickfirst.Name)
			bd.Data = pf.Build(bd.ClientConn, bd.BuildOptions)
		},
		ParseConfig: func(lbCfg json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			cfg := &wrappingBalancerConfig{}
			if err := json.Unmarshal(lbCfg, cfg); err != nil {
				return nil, err
			}
			return cfg, nil
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			if _, ok := ccs.BalancerConfig.(*wrappingBalancerConfig); !ok {
				return fmt.Errorf("received balancer config of unsupported type %T", ccs.BalancerConfig)
			}
			bal := bd.Data.(balancer.Balancer)
			ccUpdateCh.Send(ccs)
			ccs.BalancerConfig = nil
			return bal.UpdateClientConnState(ccs)
		},
	})

	// Start a backend exposing the test service.
	backend := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
	}
	if err := backend.StartServer(); err != nil {
		t.Fatalf("Failed to start backend: %v", err)
	}
	t.Logf("Started TestService backend at: %q", backend.Address)
	defer backend.Stop()

	r := manual.NewBuilderWithScheme("whatever")

	cc, err := grpc.Dial(r.Scheme()+":///test.server", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("Dial(_, _) = _, %v; want _, nil", err)
	}
	defer cc.Close()

	// Push a resolver update and verify that our balancer receives the update.
	addrs := []resolver.Address{{Addr: backend.Address}}
	const lbCfg = "wrapping balancer LB policy config"
	goodSC := r.CC.ParseServiceConfig(fmt.Sprintf(`
{
  "loadBalancingConfig": [
    {
      "%v": {
        "config": "%s"
      }
    }
  ]
}`, t.Name(), lbCfg))
	r.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: goodSC})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	wantCCS := balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses:     addrs,
			ServiceConfig: goodSC,
		},
		BalancerConfig: &wrappingBalancerConfig{Config: lbCfg},
	}
	ccs, err := ccUpdateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for ClientConnState update from grpc")
	}
	gotCCS := ccs.(balancer.ClientConnState)
	if err := verifyClientConnStateUpdate(gotCCS, wantCCS); err != nil {
		t.Fatal(err)
	}

	// Ensure RPCs are successful.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("EmptyCall RPC failed: %v", err)
	}

	// Push a bad resolver update and ensure that the update is propagated to our
	// stub balancer. But since the pushed update contains an invalid service
	// config, our balancer should continue to see the old loadBalancingConfig.
	badSC := r.CC.ParseServiceConfig("bad json service config")
	wantCCS.ResolverState.ServiceConfig = badSC
	r.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: badSC})
	ccs, err = ccUpdateCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for ClientConnState update from grpc")
	}
	gotCCS = ccs.(balancer.ClientConnState)
	if err := verifyClientConnStateUpdate(gotCCS, wantCCS); err != nil {
		t.Fatal(err)
	}

	// RPCs should continue to be successful since the ClientConn is using the old
	// good service config.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("EmptyCall RPC failed: %v", err)
	}
}
