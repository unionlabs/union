/*
 *
 * Copyright 2024 gRPC authors.
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

package endpointsharding

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils/roundrobin"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

var gracefulSwitchPickFirst serviceconfig.LoadBalancingConfig

var logger = grpclog.Component("endpoint-sharding-test")

func init() {
	var err error
	gracefulSwitchPickFirst, err = ParseConfig(json.RawMessage(PickFirstConfig))
	if err != nil {
		logger.Fatal(err)
	}
	balancer.Register(fakePetioleBuilder{})
}

const fakePetioleName = "fake_petiole"

type fakePetioleBuilder struct{}

func (fakePetioleBuilder) Name() string {
	return fakePetioleName
}

func (fakePetioleBuilder) Build(cc balancer.ClientConn, opts balancer.BuildOptions) balancer.Balancer {
	fp := &fakePetiole{
		ClientConn: cc,
		bOpts:      opts,
	}
	fp.Balancer = NewBalancer(fp, opts)
	return fp
}

func (fakePetioleBuilder) ParseConfig(json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	return nil, nil
}

// fakePetiole is a load balancer that wraps the endpointShardingBalancer, and
// forwards ClientConnUpdates with a child config of graceful switch that wraps
// pick first. It also intercepts UpdateState to make sure it can access the
// child state maintained by EndpointSharding.
type fakePetiole struct {
	balancer.Balancer
	balancer.ClientConn
	bOpts balancer.BuildOptions
}

func (fp *fakePetiole) UpdateClientConnState(state balancer.ClientConnState) error {
	if el := state.ResolverState.Endpoints; len(el) != 2 {
		return fmt.Errorf("UpdateClientConnState wants two endpoints, got: %v", el)
	}

	return fp.Balancer.UpdateClientConnState(balancer.ClientConnState{
		BalancerConfig: gracefulSwitchPickFirst,
		ResolverState:  state.ResolverState,
	})
}

func (fp *fakePetiole) UpdateState(state balancer.State) {
	childStates := ChildStatesFromPicker(state.Picker)
	// Both child states should be present in the child picker. States and
	// picker change over the lifecycle of test, but there should always be two.
	if len(childStates) != 2 {
		logger.Fatal(fmt.Errorf("length of child states received: %v, want 2", len(childStates)))
	}

	fp.ClientConn.UpdateState(state)
}

// TestEndpointShardingBasic tests the basic functionality of the endpoint
// sharding balancer. It specifies a petiole policy that is essentially a
// wrapper around the endpoint sharder. Two backends are started, with each
// backend's address specified in an endpoint. The petiole does not have a
// special picker, so it should fallback to the default behavior, which is to
// round_robin amongst the endpoint children that are in the aggregated state.
// It also verifies the petiole has access to the raw child state in case it
// wants to implement a custom picker.
func (s) TestEndpointShardingBasic(t *testing.T) {
	backend1 := stubserver.StartTestService(t, nil)
	defer backend1.Stop()
	backend2 := stubserver.StartTestService(t, nil)
	defer backend2.Stop()

	mr := manual.NewBuilderWithScheme("e2e-test")
	defer mr.Close()

	json := `{"loadBalancingConfig": [{"fake_petiole":{}}]}`
	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(json)
	mr.InitialState(resolver.State{
		Endpoints: []resolver.Endpoint{
			{Addresses: []resolver.Address{{Addr: backend1.Address}}},
			{Addresses: []resolver.Address{{Addr: backend2.Address}}},
		},
		ServiceConfig: sc,
	})

	cc, err := grpc.Dial(mr.Scheme()+":///", grpc.WithResolvers(mr), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to dial: %v", err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), time.Second*10)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	// Assert a round robin distribution between the two spun up backends. This
	// requires a poll and eventual consistency as both endpoint children do not
	// start in state READY.
	if err = roundrobin.CheckRoundRobinRPCs(ctx, client, []resolver.Address{{Addr: backend1.Address}, {Addr: backend2.Address}}); err != nil {
		t.Fatalf("error in expected round robin: %v", err)
	}
}
