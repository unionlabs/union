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

package grpc_test

import (
	"context"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/stubserver"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
)

type producerBuilder struct{}

type producer struct {
	client  testgrpc.TestServiceClient
	stopped chan struct{}
}

// Build constructs and returns a producer and its cleanup function
func (*producerBuilder) Build(cci any) (balancer.Producer, func()) {
	p := &producer{
		client:  testgrpc.NewTestServiceClient(cci.(grpc.ClientConnInterface)),
		stopped: make(chan struct{}),
	}
	return p, func() {
		<-p.stopped
	}
}

func (p *producer) TestStreamStart(t *testing.T, streamStarted chan<- struct{}) {
	go func() {
		defer close(p.stopped)
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()
		if _, err := p.client.FullDuplexCall(ctx); err != nil {
			t.Errorf("Unexpected error starting stream: %v", err)
		}
		close(streamStarted)
	}()
}

var producerBuilderSingleton = &producerBuilder{}

// TestProducerStreamStartsAfterReady ensures producer streams only start after
// the subchannel reports as READY to the LB policy.
func (s) TestProducerStreamStartsAfterReady(t *testing.T) {
	name := strings.ReplaceAll(strings.ToLower(t.Name()), "/", "")
	producerCh := make(chan balancer.Producer)
	streamStarted := make(chan struct{})
	done := make(chan struct{})
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			sc, err := bd.ClientConn.NewSubConn(ccs.ResolverState.Addresses, balancer.NewSubConnOptions{
				StateListener: func(scs balancer.SubConnState) {
					if scs.ConnectivityState == connectivity.Ready {
						timer := time.NewTimer(5 * time.Millisecond)
						select {
						case <-streamStarted:
							t.Errorf("Producer stream started before Ready listener returned")
						case <-timer.C:
						}
						close(done)
					}
				},
			})
			if err != nil {
				return err
			}
			producer, _ := sc.GetOrBuildProducer(producerBuilderSingleton)
			producerCh <- producer
			sc.Connect()
			return nil
		},
	}
	stub.Register(name, bf)

	ss := stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			return nil
		},
	}
	if err := ss.StartServer(); err != nil {
		t.Fatal("Error starting server:", err)
	}
	defer ss.Stop()

	cc, err := grpc.NewClient("dns:///"+ss.Address,
		grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"`+name+`":{}}]}`),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("Error creating client: %v", err)
	}
	defer cc.Close()

	go cc.Connect()
	p := <-producerCh
	p.(*producer).TestStreamStart(t, streamStarted)

	<-done
}
