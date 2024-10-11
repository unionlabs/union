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
 *
 */

package base

import (
	"context"
	"testing"
	"time"

	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/resolver"
)

type testClientConn struct {
	balancer.ClientConn
	newSubConn func([]resolver.Address, balancer.NewSubConnOptions) (balancer.SubConn, error)
}

func (c *testClientConn) NewSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
	return c.newSubConn(addrs, opts)
}

func (c *testClientConn) UpdateState(balancer.State) {}

type testSubConn struct {
	updateState func(balancer.SubConnState)
}

func (sc *testSubConn) UpdateAddresses([]resolver.Address) {}

func (sc *testSubConn) Connect() {}

func (sc *testSubConn) Shutdown() {}

func (sc *testSubConn) GetOrBuildProducer(balancer.ProducerBuilder) (balancer.Producer, func()) {
	return nil, nil
}

// testPickBuilder creates balancer.Picker for test.
type testPickBuilder struct {
	validate func(info PickerBuildInfo)
}

func (p *testPickBuilder) Build(info PickerBuildInfo) balancer.Picker {
	p.validate(info)
	return nil
}

func TestBaseBalancerReserveAttributes(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	validated := make(chan struct{}, 1)
	v := func(info PickerBuildInfo) {
		defer func() { validated <- struct{}{} }()
		for _, sc := range info.ReadySCs {
			if sc.Address.Addr == "1.1.1.1" {
				if sc.Address.Attributes == nil {
					t.Errorf("in picker.validate, got address %+v with nil attributes, want not nil", sc.Address)
				}
				foo, ok := sc.Address.Attributes.Value("foo").(string)
				if !ok || foo != "2233niang" {
					t.Errorf("in picker.validate, got address[1.1.1.1] with invalid attributes value %v, want 2233niang", sc.Address.Attributes.Value("foo"))
				}
			} else if sc.Address.Addr == "2.2.2.2" {
				if sc.Address.Attributes != nil {
					t.Error("in b.subConns, got address[2.2.2.2] with not nil attributes, want nil")
				}
			}
		}
	}
	pickBuilder := &testPickBuilder{validate: v}
	b := (&baseBuilder{pickerBuilder: pickBuilder}).Build(&testClientConn{
		newSubConn: func(_ []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
			return &testSubConn{updateState: opts.StateListener}, nil
		},
	}, balancer.BuildOptions{}).(*baseBalancer)

	b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				{Addr: "1.1.1.1", Attributes: attributes.New("foo", "2233niang")},
				{Addr: "2.2.2.2", Attributes: nil},
			},
		},
	})
	select {
	case <-validated:
	case <-ctx.Done():
		t.Fatalf("timed out waiting for UpdateClientConnState to call picker.Build")
	}

	for sc := range b.scStates {
		sc.(*testSubConn).updateState(balancer.SubConnState{ConnectivityState: connectivity.Ready, ConnectionError: nil})
		select {
		case <-validated:
		case <-ctx.Done():
			t.Fatalf("timed out waiting for UpdateClientConnState to call picker.Build")
		}
	}
}
