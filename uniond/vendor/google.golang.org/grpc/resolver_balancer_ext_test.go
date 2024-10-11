/*
 *
 * Copyright 2023 gRPC authors.
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
	"errors"
	"fmt"
	"runtime"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
)

// TestResolverBalancerInteraction tests:
// 1. resolver.Builder.Build() ->
// 2. resolver.ClientConn.UpdateState() ->
// 3. balancer.Balancer.UpdateClientConnState() ->
// 4. balancer.ClientConn.ResolveNow() ->
// 5. resolver.Resolver.ResolveNow() ->
func (s) TestResolverBalancerInteraction(t *testing.T) {
	name := strings.ReplaceAll(strings.ToLower(t.Name()), "/", "")
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			bd.ClientConn.ResolveNow(resolver.ResolveNowOptions{})
			return nil
		},
	}
	stub.Register(name, bf)

	rb := manual.NewBuilderWithScheme(name)
	rb.BuildCallback = func(_ resolver.Target, cc resolver.ClientConn, _ resolver.BuildOptions) {
		sc := cc.ParseServiceConfig(`{"loadBalancingConfig": [{"` + name + `":{}}]}`)
		cc.UpdateState(resolver.State{
			Addresses:     []resolver.Address{{Addr: "test"}},
			ServiceConfig: sc,
		})
	}
	rnCh := make(chan struct{})
	rb.ResolveNowCallback = func(resolver.ResolveNowOptions) { close(rnCh) }
	resolver.Register(rb)

	cc, err := grpc.Dial(name+":///", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial error: %v", err)
	}
	defer cc.Close()
	select {
	case <-rnCh:
	case <-time.After(defaultTestTimeout):
		t.Fatalf("timed out waiting for resolver.ResolveNow")
	}
}

type resolverBuilderWithErr struct {
	resolver.Resolver
	errCh  <-chan error
	scheme string
}

func (b *resolverBuilderWithErr) Build(_ resolver.Target, cc resolver.ClientConn, _ resolver.BuildOptions) (resolver.Resolver, error) {
	if err := <-b.errCh; err != nil {
		return nil, err
	}
	return b, nil
}

func (b *resolverBuilderWithErr) Scheme() string {
	return b.scheme
}

func (b *resolverBuilderWithErr) Close() {}

// TestResolverBuildFailure tests:
// 1. resolver.Builder.Build() passes.
// 2. Channel enters idle mode.
// 3. An RPC happens.
// 4. resolver.Builder.Build() fails.
func (s) TestResolverBuildFailure(t *testing.T) {
	enterIdle := internal.EnterIdleModeForTesting.(func(*grpc.ClientConn))
	name := strings.ReplaceAll(strings.ToLower(t.Name()), "/", "")
	resErrCh := make(chan error, 1)
	resolver.Register(&resolverBuilderWithErr{errCh: resErrCh, scheme: name})

	resErrCh <- nil
	cc, err := grpc.Dial(name+":///", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial error: %v", err)
	}
	defer cc.Close()
	enterIdle(cc)
	const errStr = "test error from resolver builder"
	t.Log("pushing res err")
	resErrCh <- errors.New(errStr)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := cc.Invoke(ctx, "/a/b", nil, nil); err == nil || !strings.Contains(err.Error(), errStr) {
		t.Fatalf("Invoke = %v; want %v", err, errStr)
	}
}

// TestEnterIdleDuringResolverUpdateState tests a scenario that used to deadlock
// while calling UpdateState at the same time as the resolver being closed while
// the channel enters idle mode.
func (s) TestEnterIdleDuringResolverUpdateState(t *testing.T) {
	enterIdle := internal.EnterIdleModeForTesting.(func(*grpc.ClientConn))
	name := strings.ReplaceAll(strings.ToLower(t.Name()), "/", "")

	// Create a manual resolver that spams UpdateState calls until it is closed.
	rb := manual.NewBuilderWithScheme(name)
	var cancel context.CancelFunc
	rb.BuildCallback = func(_ resolver.Target, cc resolver.ClientConn, _ resolver.BuildOptions) {
		var ctx context.Context
		ctx, cancel = context.WithCancel(context.Background())
		go func() {
			for ctx.Err() == nil {
				cc.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "test"}}})
			}
		}()
	}
	rb.CloseCallback = func() {
		cancel()
	}
	resolver.Register(rb)

	cc, err := grpc.NewClient(name+":///", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient error: %v", err)
	}
	defer cc.Close()

	// Enter/exit idle mode repeatedly.
	for i := 0; i < 2000; i++ {
		// Start a timer so we panic out of the deadlock and can see all the
		// stack traces to debug the problem.
		p := time.AfterFunc(time.Second, func() {
			buf := make([]byte, 8192)
			buf = buf[0:runtime.Stack(buf, true)]
			t.Error("Timed out waiting for enterIdle")
			panic(fmt.Sprint("Stack trace:\n", string(buf)))
		})
		enterIdle(cc)
		p.Stop()
		cc.Connect()
	}
}

// TestEnterIdleDuringBalancerUpdateState tests calling UpdateState at the same
// time as the balancer being closed while the channel enters idle mode.
func (s) TestEnterIdleDuringBalancerUpdateState(t *testing.T) {
	enterIdle := internal.EnterIdleModeForTesting.(func(*grpc.ClientConn))
	name := strings.ReplaceAll(strings.ToLower(t.Name()), "/", "")

	// Create a balancer that calls UpdateState once asynchronously, attempting
	// to make the channel appear ready even after entering idle.
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			go func() {
				bd.ClientConn.UpdateState(balancer.State{ConnectivityState: connectivity.Ready})
			}()
			return nil
		},
	}
	stub.Register(name, bf)

	rb := manual.NewBuilderWithScheme(name)
	rb.BuildCallback = func(_ resolver.Target, cc resolver.ClientConn, _ resolver.BuildOptions) {
		cc.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "test"}}})
	}
	resolver.Register(rb)

	cc, err := grpc.NewClient(
		name+":///",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"`+name+`":{}}]}`))
	if err != nil {
		t.Fatalf("grpc.NewClient error: %v", err)
	}
	defer cc.Close()

	// Enter/exit idle mode repeatedly.
	for i := 0; i < 2000; i++ {
		enterIdle(cc)
		if got, want := cc.GetState(), connectivity.Idle; got != want {
			t.Fatalf("cc state = %v; want %v", got, want)
		}
		cc.Connect()
	}
}

// TestEnterIdleDuringBalancerNewSubConn tests calling NewSubConn at the same
// time as the balancer being closed while the channel enters idle mode.
func (s) TestEnterIdleDuringBalancerNewSubConn(t *testing.T) {
	channelz.TurnOn()
	defer internal.ChannelzTurnOffForTesting()
	enterIdle := internal.EnterIdleModeForTesting.(func(*grpc.ClientConn))
	name := strings.ReplaceAll(strings.ToLower(t.Name()), "/", "")

	// Create a balancer that calls NewSubConn once asynchronously, attempting
	// to create a subchannel after going idle.
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			go func() {
				bd.ClientConn.NewSubConn([]resolver.Address{{Addr: "test"}}, balancer.NewSubConnOptions{})
			}()
			return nil
		},
	}
	stub.Register(name, bf)

	rb := manual.NewBuilderWithScheme(name)
	rb.BuildCallback = func(_ resolver.Target, cc resolver.ClientConn, _ resolver.BuildOptions) {
		cc.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "test"}}})
	}
	resolver.Register(rb)

	cc, err := grpc.NewClient(
		name+":///",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"`+name+`":{}}]}`))
	if err != nil {
		t.Fatalf("grpc.NewClient error: %v", err)
	}
	defer cc.Close()

	// Enter/exit idle mode repeatedly.
	for i := 0; i < 2000; i++ {
		enterIdle(cc)
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			t.Fatalf("Found channels: %v; expected 1 entry", tcs)
		}
		if got := tcs[0].SubChans(); len(got) != 0 {
			t.Fatalf("Found subchannels: %v; expected 0 entries", got)
		}
		cc.Connect()
	}
}
