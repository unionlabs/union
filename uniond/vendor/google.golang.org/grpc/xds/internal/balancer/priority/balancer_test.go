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
 *
 */

package priority

import (
	"context"
	"fmt"
	"testing"
	"time"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/roundrobin"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/hierarchy"
	internalserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
)

const (
	defaultTestTimeout      = 5 * time.Second
	defaultTestShortTimeout = 100 * time.Millisecond
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

var testBackendAddrStrs []string

const (
	testBackendAddrsCount = 12
	testRRBalancerName    = "another-round-robin"
)

type anotherRR struct {
	balancer.Builder
}

func (*anotherRR) Name() string {
	return testRRBalancerName
}

func init() {
	for i := 0; i < testBackendAddrsCount; i++ {
		testBackendAddrStrs = append(testBackendAddrStrs, fmt.Sprintf("%d.%d.%d.%d:%d", i, i, i, i, i))
	}
	// Disable sub-balancer caching for all but the tests which exercise the
	// caching behavior.
	DefaultSubBalancerCloseTimeout = time.Duration(0)
	balancer.Register(&anotherRR{Builder: balancer.Get(roundrobin.Name)})
}

// When a high priority is ready, adding/removing lower locality doesn't cause
// changes.
//
// Init 0 and 1; 0 is up, use 0; add 2, use 0; remove 2, use 0.
func (s) TestPriority_HighPriorityReady(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two children, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// p0 is ready.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Add p2, it shouldn't cause any updates.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-2"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-2": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1", "child-2"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	select {
	case sc := <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn: %s", sc)
	case sc := <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn: %v", sc)
	case <-time.After(time.Millisecond * 100):
	}

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Remove p2, no updates.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	select {
	case <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn")
	case <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn")
	case <-time.After(time.Millisecond * 100):
	}

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}
}

// Lower priority is used when higher priority is not ready.
//
// Init 0 and 1; 0 is up, use 0; 0 is down, 1 is up, use 1; add 2, use 1; 1 is
// down, use 2; remove 2, use 1.
func (s) TestPriority_SwitchPriority(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	t.Log("Two localities, with priorities [0, 1], each with one backend.")
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	t.Log("Make p0 ready.")
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc0); err != nil {
		t.Fatal(err.Error())
	}

	t.Log("Turn down 0, will start and use 1.")
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	t.Log("Handle SubConn creation from 1.")
	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 1.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	t.Log("Add p2, it shouldn't cause any updates.")
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-2"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-2": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1", "child-2"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	select {
	case sc := <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn, %s", sc)
	case <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn")
	case <-time.After(time.Millisecond * 100):
	}

	t.Log("Turn down 1, use 2.")
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	// Before 2 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs2 := <-cc.NewSubConnAddrsCh
	if got, want := addrs2[0].Addr, testBackendAddrStrs[2]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc2 := <-cc.NewSubConnCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 2.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}

	t.Log("Remove 2, use 1.")
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// p2 SubConns are shut down.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc2 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc2, scToShutdown)
	}

	// Should get an update with 1's old transient failure picker, to override
	// 2's old picker.
	if err := cc.WaitForErrPicker(ctx); err != nil {
		t.Fatal(err.Error())
	}
	<-cc.NewStateCh // Drain to match picker

	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	// Does not change the aggregate state, because round robin does not leave
	// TRANSIENT_FAILURE if a subconn goes CONNECTING.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}
}

// Lower priority is used when higher priority turns Connecting from Ready.
// Because changing from Ready to Connecting is a failure.
//
// Init 0 and 1; 0 is up, use 0; 0 is connecting, 1 is up, use 1; 0 is ready,
// use 0.
func (s) TestPriority_HighPriorityToConnectingFromReady(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two localities, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// p0 is ready.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc0); err != nil {
		t.Fatal(err.Error())
	}

	// Turn 0 to TransientFailure, will start and use 1.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	// Handle SubConn creation from 1.
	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 1.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Turn 0 back to Ready.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// p1 subconn should be shut down.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc0, scToShutdown)
	}

	if err := cc.WaitForRoundRobinPicker(ctx, sc0); err != nil {
		t.Fatal(err.Error())
	}
}

// Add a lower priority while the higher priority is down.
//
// Init 0 and 1; 0 and 1 both down; add 2, use 2.
func (s) TestPriority_HigherDownWhileAddingLower(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two localities, with different priorities, each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	t.Log("Turn down 0, 1 is used.")
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	t.Log("Turn down 1, pick should error.")
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	// Test pick failure.
	if err := cc.WaitForErrPicker(ctx); err != nil {
		t.Fatal(err.Error())
	}
	<-cc.NewStateCh // Drain to match picker

	t.Log("Add p2, it should create a new SubConn.")
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-2"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-2": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1", "child-2"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// A new connecting picker should be updated for the new priority.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs2 := <-cc.NewSubConnAddrsCh
	if got, want := addrs2[0].Addr, testBackendAddrStrs[2]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc2 := <-cc.NewSubConnCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 2.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}
}

// When a higher priority becomes available, all lower priorities are closed.
//
// Init 0,1,2; 0 and 1 down, use 2; 0 up, close 1 and 2.
func (s) TestPriority_HigherReadyCloseAllLower(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Three localities, with priorities [0,1,2], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-2"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-2": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1", "child-2"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// Turn down 0, 1 is used.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// Turn down 1, 2 is used.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	// Before 2 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs2 := <-cc.NewSubConnAddrsCh
	if got, want := addrs2[0].Addr, testBackendAddrStrs[2]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc2 := <-cc.NewSubConnCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 2.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}

	// When 0 becomes ready, 0 should be used, 1 and 2 should all be closed.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// sc1 and sc2 should be shut down.
	//
	// With localities caching, the lower priorities are closed after a timeout,
	// in goroutines. The order is no longer guaranteed.
	scToShutdown := []balancer.SubConn{<-cc.ShutdownSubConnCh, <-cc.ShutdownSubConnCh}
	if !(scToShutdown[0] == sc1 && scToShutdown[1] == sc2) && !(scToShutdown[0] == sc2 && scToShutdown[1] == sc1) {
		t.Errorf("ShutdownSubConn, want [%v, %v], got %v", sc1, sc2, scToShutdown)
	}

	// Test pick with 0.
	if err := cc.WaitForRoundRobinPicker(ctx, sc0); err != nil {
		t.Fatal(err.Error())
	}
}

// At init, start the next lower priority after timeout if the higher priority
// doesn't get ready.
//
// Init 0,1; 0 is not ready (in connecting), after timeout, use 1.
func (s) TestPriority_InitTimeout(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	const testPriorityInitTimeout = 200 * time.Millisecond
	defer func() func() {
		old := DefaultPriorityInitTimeout
		DefaultPriorityInitTimeout = testPriorityInitTimeout
		return func() {
			DefaultPriorityInitTimeout = old
		}
	}()()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two localities, with different priorities, each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// Keep 0 in connecting, 1 will be used after init timeout.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})

	// Make sure new SubConn is created before timeout.
	select {
	case <-time.After(testPriorityInitTimeout * 3 / 4):
	case <-cc.NewSubConnAddrsCh:
		t.Fatalf("Got a new SubConn too early (Within timeout). Expect a new SubConn only after timeout")
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// After the init timer of p0, when switching to p1, a connecting picker
	// will be sent to the parent. Clear it here.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 1.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}
}

// EDS removes all priorities, and re-adds them.
func (s) TestPriority_RemovesAllPriorities(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	const testPriorityInitTimeout = 200 * time.Millisecond
	defer func() func() {
		old := DefaultPriorityInitTimeout
		DefaultPriorityInitTimeout = testPriorityInitTimeout
		return func() {
			DefaultPriorityInitTimeout = old
		}
	}()()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two localities, with different priorities, each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc0); err != nil {
		t.Fatal(err.Error())
	}

	// Remove all priorities.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: nil,
		},
		BalancerConfig: &LBConfig{
			Children:   nil,
			Priorities: nil,
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// p0 subconn should be shut down.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc0 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc0, scToShutdown)
	}

	// Test pick return TransientFailure.
	if err := cc.WaitForPickerWithErr(ctx, ErrAllPrioritiesRemoved); err != nil {
		t.Fatal(err.Error())
	}

	// Re-add two localities, with previous priorities, but different backends.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[3]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs01 := <-cc.NewSubConnAddrsCh
	if got, want := addrs01[0].Addr, testBackendAddrStrs[2]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc01 := <-cc.NewSubConnCh

	// Don't send any update to p0, so to not override the old state of p0.
	// Later, connect to p1 and then remove p1. This will fallback to p0, and
	// will send p0's old picker if they are not correctly removed.

	// p1 will be used after priority init timeout.
	addrs11 := <-cc.NewSubConnAddrsCh
	if got, want := addrs11[0].Addr, testBackendAddrStrs[3]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc11 := <-cc.NewSubConnCh
	sc11.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc11.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p1 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc11); err != nil {
		t.Fatal(err.Error())
	}

	// Remove p1, to fallback to p0.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// p1 subconn should be shut down.
	scToShutdown1 := <-cc.ShutdownSubConnCh
	if scToShutdown1 != sc11 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc11, scToShutdown1)
	}

	// Test pick return NoSubConn.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	// Send an ready update for the p0 sc that was received when re-adding
	// priorities.
	sc01.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc01.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc01); err != nil {
		t.Fatal(err.Error())
	}

	select {
	case <-cc.NewPickerCh:
		t.Fatalf("got unexpected new picker")
	case <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn")
	case <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn")
	case <-time.After(time.Millisecond * 100):
	}
}

// Test the case where the high priority contains no backends. The low priority
// will be used.
func (s) TestPriority_HighPriorityNoEndpoints(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two localities, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// p0 is ready.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Remove addresses from priority 0, should use p1.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// p0 will shutdown the subconn, and ClientConn will send a sc update to
	// shutdown.
	scToShutdown := <-cc.ShutdownSubConnCh
	scToShutdown.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Shutdown})

	addrs2 := <-cc.NewSubConnAddrsCh
	if got, want := addrs2[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc2 := <-cc.NewSubConnCh

	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	// p1 is ready.
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p1 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}
}

// Test the case where the first and only priority is removed.
func (s) TestPriority_FirstPriorityUnavailable(t *testing.T) {
	const testPriorityInitTimeout = 200 * time.Millisecond
	defer func(t time.Duration) {
		DefaultPriorityInitTimeout = t
	}(DefaultPriorityInitTimeout)
	DefaultPriorityInitTimeout = testPriorityInitTimeout

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One localities, with priorities [0], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Remove the only localities.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: nil,
		},
		BalancerConfig: &LBConfig{
			Children:   nil,
			Priorities: nil,
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Wait after double the init timer timeout, to ensure it doesn't panic.
	time.Sleep(testPriorityInitTimeout * 2)
}

// When a child is moved from low priority to high.
//
// Init a(p0) and b(p1); a(p0) is up, use a; move b to p0, a to p1, use b.
func (s) TestPriority_MoveChildToHigherPriority(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two children, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// p0 is ready.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Swap child with p0 and p1, the child at lower priority should now be the
	// higher priority, and be used. The old SubConn should be closed.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-1", "child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// When the new child for p0 is changed from the previous child, the
	// balancer should immediately update the picker so the picker from old
	// child is not used. In this case, the picker becomes a
	// no-subconn-available picker because this child is just started.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	// Old subconn should be shut down.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scToShutdown)
	}

	addrs2 := <-cc.NewSubConnAddrsCh
	if got, want := addrs2[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc2 := <-cc.NewSubConnCh

	// New p0 child is ready.
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only new subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}
}

// When a child is in lower priority, and in use (because higher is down),
// move it from low priority to high.
//
// Init a(p0) and b(p1); a(p0) is down, use b; move b to p0, a to p1, use b.
func (s) TestPriority_MoveReadyChildToHigherPriority(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two children, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// p0 is down.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p1 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Swap child with p0 and p1, the child at lower priority should now be the
	// higher priority, and be used. The old SubConn should be closed.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-1", "child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Old subconn from child-0 should be removed.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc0 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc0, scToShutdown)
	}

	// Because this was a ready child moved to a higher priority, no new subconn
	// or picker should be updated.
	select {
	case <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn")
	case <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn")
	case <-time.After(time.Millisecond * 100):
	}
}

// When the lowest child is in use, and is removed, should use the higher
// priority child even though it's not ready.
//
// Init a(p0) and b(p1); a(p0) is down, use b; move b to p0, a to p1, use b.
func (s) TestPriority_RemoveReadyLowestChild(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two children, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// p0 is down.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p1 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Remove child with p1, the child at higher priority should now be used.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Old subconn from child-1 should be shut down.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scToShutdown)
	}

	if err := cc.WaitForErrPicker(ctx); err != nil {
		t.Fatal(err.Error())
	}
	<-cc.NewStateCh // Drain to match picker

	// Because there was no new child, no new subconn should be created.
	select {
	case <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn")
	case <-time.After(time.Millisecond * 100):
	}
}

// When a ready child is removed, it's kept in cache. Re-adding doesn't create subconns.
//
// Init 0; 0 is up, use 0; remove 0, only picker is updated, no subconn is
// removed; re-add 0, picker is updated.
func (s) TestPriority_ReadyChildRemovedButInCache(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	const testChildCacheTimeout = time.Second
	defer func() func() {
		old := DefaultSubBalancerCloseTimeout
		DefaultSubBalancerCloseTimeout = testChildCacheTimeout
		return func() {
			DefaultSubBalancerCloseTimeout = old
		}
	}()()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One children, with priorities [0], with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// p0 is ready.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Remove the child, it shouldn't cause any conn changed, but picker should
	// be different.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{},
		BalancerConfig: &LBConfig{},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	if err := cc.WaitForPickerWithErr(ctx, ErrAllPrioritiesRemoved); err != nil {
		t.Fatal(err.Error())
	}

	// But no conn changes should happen. Child balancer is in cache.
	select {
	case sc := <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn: %s", sc)
	case sc := <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn: %v", sc)
	case <-time.After(time.Millisecond * 100):
	}

	// Re-add the child, shouldn't create new connections.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// But no conn changes should happen. Child balancer is just taken out from
	// the cache.
	select {
	case sc := <-cc.NewSubConnCh:
		t.Fatalf("got unexpected new SubConn: %s", sc)
	case sc := <-cc.ShutdownSubConnCh:
		t.Fatalf("got unexpected shutdown SubConn: %v", sc)
	case <-time.After(time.Millisecond * 100):
	}
}

// When the policy of a child is changed.
//
// Init 0; 0 is up, use 0; change 0's policy, 0 is used.
func (s) TestPriority_ChildPolicyChange(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One children, with priorities [0], with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh

	// p0 is ready.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test roundrobin with only p0 subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	// Change the policy for the child (still roundrobin, but with a different
	// name).
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: testRRBalancerName}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Old subconn should be shut down.
	scToShutdown := <-cc.ShutdownSubConnCh
	if scToShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scToShutdown)
	}

	// A new subconn should be created.
	addrs2 := <-cc.NewSubConnAddrsCh
	if got, want := addrs2[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc2 := <-cc.NewSubConnCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pickfirst with the new subconns.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}
}

const inlineUpdateBalancerName = "test-inline-update-balancer"

var errTestInlineStateUpdate = fmt.Errorf("don't like addresses, empty or not")

func init() {
	stub.Register(inlineUpdateBalancerName, stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, _ balancer.ClientConnState) error {
			bd.ClientConn.UpdateState(balancer.State{
				ConnectivityState: connectivity.Ready,
				Picker:            &testutils.TestConstPicker{Err: errTestInlineStateUpdate},
			})
			return nil
		},
	})
}

// When the child policy update picker inline in a handleClientUpdate call
// (e.g., roundrobin handling empty addresses). There could be deadlock caused
// by acquiring a locked mutex.
func (s) TestPriority_ChildPolicyUpdatePickerInline(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One children, with priorities [0], with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: inlineUpdateBalancerName}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	if err := cc.WaitForPickerWithErr(ctx, errTestInlineStateUpdate); err != nil {
		t.Fatal(err.Error())
	}
}

// TestPriority_IgnoreReresolutionRequest tests the case where the priority
// policy has a single child policy. The test verifies that ResolveNow() calls
// from the child policy are ignored based on the value of the
// IgnoreReresolutionRequests field in the configuration.
func (s) TestPriority_IgnoreReresolutionRequest(t *testing.T) {
	// Register a stub balancer to act the child policy of the priority policy.
	// Provide an init function to the stub balancer to capture the ClientConn
	// passed to the child policy.
	ccCh := testutils.NewChannel()
	childPolicyName := t.Name()
	stub.Register(childPolicyName, stub.BalancerFuncs{
		Init: func(data *stub.BalancerData) {
			ccCh.Send(data.ClientConn)
		},
	})

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One children, with priorities [0], with one backend, reresolution is
	// ignored.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {
					Config:                     &internalserviceconfig.BalancerConfig{Name: childPolicyName},
					IgnoreReresolutionRequests: true,
				},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Retrieve the ClientConn passed to the child policy.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	val, err := ccCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout waiting for ClientConn from the child policy")
	}
	balancerCC := val.(balancer.ClientConn)

	// Since IgnoreReresolutionRequests was set to true, all ResolveNow() calls
	// should be ignored.
	for i := 0; i < 5; i++ {
		balancerCC.ResolveNow(resolver.ResolveNowOptions{})
	}
	select {
	case <-cc.ResolveNowCh:
		t.Fatalf("got unexpected ResolveNow() call")
	case <-time.After(defaultTestShortTimeout):
	}

	// Send another update to set IgnoreReresolutionRequests to false.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {
					Config:                     &internalserviceconfig.BalancerConfig{Name: childPolicyName},
					IgnoreReresolutionRequests: false,
				},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Call ResolveNow() on the CC, it should be forwarded.
	balancerCC.ResolveNow(resolver.ResolveNowOptions{})
	select {
	case <-cc.ResolveNowCh:
	case <-time.After(time.Second):
		t.Fatalf("timeout waiting for ResolveNow()")
	}

}

// TestPriority_IgnoreReresolutionRequestTwoChildren tests the case where the
// priority policy has two child policies, one of them has the
// IgnoreReresolutionRequests field set to true while the other one has it set
// to false. The test verifies that ResolveNow() calls from the child which is
// set to ignore reresolution requests are ignored, while calls from the other
// child are processed.
func (s) TestPriority_IgnoreReresolutionRequestTwoChildren(t *testing.T) {
	// Register a stub balancer to act the child policy of the priority policy.
	// Provide an init function to the stub balancer to capture the ClientConn
	// passed to the child policy.
	ccCh := testutils.NewChannel()
	childPolicyName := t.Name()
	stub.Register(childPolicyName, stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			ccCh.Send(bd.ClientConn)
			bd.Data = balancer.Get(roundrobin.Name).Build(bd.ClientConn, bd.BuildOptions)
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			bal := bd.Data.(balancer.Balancer)
			return bal.UpdateClientConnState(ccs)
		},
	})

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One children, with priorities [0, 1], each with one backend.
	// Reresolution is ignored for p0.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {
					Config:                     &internalserviceconfig.BalancerConfig{Name: childPolicyName},
					IgnoreReresolutionRequests: true,
				},
				"child-1": {
					Config: &internalserviceconfig.BalancerConfig{Name: childPolicyName},
				},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Retrieve the ClientConn passed to the child policy from p0.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	val, err := ccCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout waiting for ClientConn from the child policy")
	}
	balancerCC0 := val.(balancer.ClientConn)

	// Set p0 to transient failure, p1 will be started.
	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	// Retrieve the ClientConn passed to the child policy from p1.
	val, err = ccCh.Receive(ctx)
	if err != nil {
		t.Fatalf("timeout waiting for ClientConn from the child policy")
	}
	balancerCC1 := val.(balancer.ClientConn)

	// Since IgnoreReresolutionRequests was set to true for p0, ResolveNow()
	// from p0 should all be ignored.
	for i := 0; i < 5; i++ {
		balancerCC0.ResolveNow(resolver.ResolveNowOptions{})
	}
	select {
	case <-cc.ResolveNowCh:
		t.Fatalf("got unexpected ResolveNow() call")
	case <-time.After(defaultTestShortTimeout):
	}

	// But IgnoreReresolutionRequests was false for p1, ResolveNow() from p1
	// should be forwarded.
	balancerCC1.ResolveNow(resolver.ResolveNowOptions{})
	select {
	case <-cc.ResolveNowCh:
	case <-time.After(defaultTestShortTimeout):
		t.Fatalf("timeout waiting for ResolveNow()")
	}
}

const initIdleBalancerName = "test-init-Idle-balancer"

var errsTestInitIdle = []error{
	fmt.Errorf("init Idle balancer error 0"),
	fmt.Errorf("init Idle balancer error 1"),
}

func init() {
	for i := 0; i < 2; i++ {
		ii := i
		stub.Register(fmt.Sprintf("%s-%d", initIdleBalancerName, ii), stub.BalancerFuncs{
			UpdateClientConnState: func(bd *stub.BalancerData, opts balancer.ClientConnState) error {
				lis := func(state balancer.SubConnState) {
					err := fmt.Errorf("wrong picker error")
					if state.ConnectivityState == connectivity.Idle {
						err = errsTestInitIdle[ii]
					}
					bd.ClientConn.UpdateState(balancer.State{
						ConnectivityState: state.ConnectivityState,
						Picker:            &testutils.TestConstPicker{Err: err},
					})
				}

				sc, err := bd.ClientConn.NewSubConn(opts.ResolverState.Addresses, balancer.NewSubConnOptions{StateListener: lis})
				if err != nil {
					return err
				}
				sc.Connect()
				bd.ClientConn.UpdateState(balancer.State{
					ConnectivityState: connectivity.Connecting,
					Picker:            &testutils.TestConstPicker{Err: balancer.ErrNoSubConnAvailable},
				})
				return nil
			},
		})
	}
}

// If the high priorities send initial pickers with Idle state, their pickers
// should get picks, because policies like ringhash starts in Idle, and doesn't
// connect.
//
// Init 0, 1; 0 is Idle, use 0; 0 is down, start 1; 1 is Idle, use 1.
func (s) TestPriority_HighPriorityInitIdle(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// Two children, with priorities [0, 1], each with one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: fmt.Sprintf("%s-%d", initIdleBalancerName, 0)}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: fmt.Sprintf("%s-%d", initIdleBalancerName, 1)}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// Send an Idle state update to trigger an Idle picker update.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})
	if err := cc.WaitForPickerWithErr(ctx, errsTestInitIdle[0]); err != nil {
		t.Fatal(err.Error())
	}

	// Turn p0 down, to start p1.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh
	// Idle picker from p1 should also be forwarded.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})
	if err := cc.WaitForPickerWithErr(ctx, errsTestInitIdle[1]); err != nil {
		t.Fatal(err.Error())
	}
}

// If the high priorities send initial pickers with Idle state, their pickers
// should get picks, because policies like ringhash starts in Idle, and doesn't
// connect. In this case, if a lower priority is added, it shouldn't switch to
// the lower priority.
//
// Init 0; 0 is Idle, use 0; add 1, use 0.
func (s) TestPriority_AddLowPriorityWhenHighIsInIdle(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	// One child, with priorities [0], one backend.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: fmt.Sprintf("%s-%d", initIdleBalancerName, 0)}},
			},
			Priorities: []string{"child-0"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	// Send an Idle state update to trigger an Idle picker update.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})
	if err := cc.WaitForPickerWithErr(ctx, errsTestInitIdle[0]); err != nil {
		t.Fatal(err.Error())
	}

	// Add 1, should keep using 0.
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: fmt.Sprintf("%s-%d", initIdleBalancerName, 0)}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: fmt.Sprintf("%s-%d", initIdleBalancerName, 1)}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// The ClientConn state update triggers a priority switch, from p0 -> p0
	// (since p0 is still in use). Along with this the update, p0 also gets a
	// ClientConn state update, with the addresses, which didn't change in this
	// test (this update to the child is necessary in case the addresses are
	// different).
	//
	// The test child policy, initIdleBalancer, blindly calls NewSubConn with
	// all the addresses it receives, so this will trigger a NewSubConn with the
	// old p0 addresses. (Note that in a real balancer, like roundrobin, no new
	// SubConn will be created because the addresses didn't change).
	//
	// The check below makes sure that the addresses are still from p0, and not
	// from p1. This is good enough for the purpose of this test.
	addrsNew := <-cc.NewSubConnAddrsCh
	if got, want := addrsNew[0].Addr, testBackendAddrStrs[0]; got != want {
		// Fail if p1 is started and creates a SubConn.
		t.Fatalf("got unexpected call to NewSubConn with addr: %v, want %v", addrsNew, want)
	}
}

// Lower priority is used when higher priority is not ready; higher priority
// still gets updates.
//
// Init 0 and 1; 0 is down, 1 is up, use 1; update 0; 0 is up, use 0
func (s) TestPriority_HighPriorityUpdatesWhenLowInUse(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	cc := testutils.NewBalancerClientConn(t)
	bb := balancer.Get(Name)
	pb := bb.Build(cc, balancer.BuildOptions{})
	defer pb.Close()

	t.Log("Two localities, with priorities [0, 1], each with one backend.")
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[0]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[1]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	addrs0 := <-cc.NewSubConnAddrsCh
	if got, want := addrs0[0].Addr, testBackendAddrStrs[0]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc0 := <-cc.NewSubConnCh

	t.Log("Make p0 fail.")
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	// Before 1 gets READY, picker should return NoSubConnAvailable, so RPCs
	// will retry.
	if err := cc.WaitForPickerWithErr(ctx, balancer.ErrNoSubConnAvailable); err != nil {
		t.Fatal(err.Error())
	}

	t.Log("Make p1 ready.")
	addrs1 := <-cc.NewSubConnAddrsCh
	if got, want := addrs1[0].Addr, testBackendAddrStrs[1]; got != want {
		t.Fatalf("sc is created with addr %v, want %v", got, want)
	}
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with 1.
	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	// Does not change the aggregate state, because round robin does not leave
	// TRANSIENT_FAILURE if a subconn goes CONNECTING.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	if err := cc.WaitForRoundRobinPicker(ctx, sc1); err != nil {
		t.Fatal(err.Error())
	}

	t.Log("Change p0 to use new address.")
	if err := pb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{
			Addresses: []resolver.Address{
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[2]}, []string{"child-0"}),
				hierarchy.Set(resolver.Address{Addr: testBackendAddrStrs[3]}, []string{"child-1"}),
			},
		},
		BalancerConfig: &LBConfig{
			Children: map[string]*Child{
				"child-0": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
				"child-1": {Config: &internalserviceconfig.BalancerConfig{Name: roundrobin.Name}},
			},
			Priorities: []string{"child-0", "child-1"},
		},
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Two new subconns are created by the previous update; one by p0 and one
	// by p1.  They don't happen concurrently, but they could happen in any
	// order.
	t.Log("Make p0 and p1 both ready; p0 should be used.")
	var sc2, sc3 balancer.SubConn
	for i := 0; i < 2; i++ {
		addr := <-cc.NewSubConnAddrsCh
		sc := <-cc.NewSubConnCh
		switch addr[0].Addr {
		case testBackendAddrStrs[2]:
			sc2 = sc
		case testBackendAddrStrs[3]:
			sc3 = sc
		default:
			t.Fatalf("sc is created with addr %v, want %v or %v", addr[0].Addr, testBackendAddrStrs[2], testBackendAddrStrs[3])
		}
		sc.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
		sc.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	}
	if sc2 == nil {
		t.Fatalf("sc not created with addr %v", testBackendAddrStrs[2])
	}
	if sc3 == nil {
		t.Fatalf("sc not created with addr %v", testBackendAddrStrs[3])
	}

	// Test pick with 0.
	if err := cc.WaitForRoundRobinPicker(ctx, sc2); err != nil {
		t.Fatal(err.Error())
	}
}
