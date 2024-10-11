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

package weightedtarget

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/roundrobin"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/hierarchy"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
)

const (
	defaultTestTimeout = 5 * time.Second
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type testConfigBalancerBuilder struct {
	balancer.Builder
}

func newTestConfigBalancerBuilder() *testConfigBalancerBuilder {
	return &testConfigBalancerBuilder{
		Builder: balancer.Get(roundrobin.Name),
	}
}

// pickAndCheckError returns a function which takes a picker, invokes the Pick() method
// multiple times and ensures that the error returned by the picker matches the provided error.
func pickAndCheckError(want error) func(balancer.Picker) error {
	const rpcCount = 5
	return func(p balancer.Picker) error {
		for i := 0; i < rpcCount; i++ {
			if _, err := p.Pick(balancer.PickInfo{}); err == nil || !strings.Contains(err.Error(), want.Error()) {
				return fmt.Errorf("picker.Pick() returned error: %v, want: %v", err, want)
			}
		}
		return nil
	}
}

func (t *testConfigBalancerBuilder) Build(cc balancer.ClientConn, opts balancer.BuildOptions) balancer.Balancer {
	rr := t.Builder.Build(cc, opts)
	return &testConfigBalancer{
		Balancer: rr,
	}
}

const testConfigBalancerName = "test_config_balancer"

func (t *testConfigBalancerBuilder) Name() string {
	return testConfigBalancerName
}

type stringBalancerConfig struct {
	serviceconfig.LoadBalancingConfig
	configStr string
}

func (t *testConfigBalancerBuilder) ParseConfig(c json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	var cfg string
	if err := json.Unmarshal(c, &cfg); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config in %q: %v", testConfigBalancerName, err)
	}
	return stringBalancerConfig{configStr: cfg}, nil
}

// testConfigBalancer is a roundrobin balancer, but it takes the balancer config
// string and adds it as an address attribute to the backend addresses.
type testConfigBalancer struct {
	balancer.Balancer
}

// configKey is the type used as the key to store balancer config in the
// Attributes field of resolver.Address.
type configKey struct{}

func setConfigKey(addr resolver.Address, config string) resolver.Address {
	addr.Attributes = addr.Attributes.WithValue(configKey{}, config)
	return addr
}

func getConfigKey(attr *attributes.Attributes) (string, bool) {
	v := attr.Value(configKey{})
	name, ok := v.(string)
	return name, ok
}

func (b *testConfigBalancer) UpdateClientConnState(s balancer.ClientConnState) error {
	c, ok := s.BalancerConfig.(stringBalancerConfig)
	if !ok {
		return fmt.Errorf("unexpected balancer config with type %T", s.BalancerConfig)
	}

	addrsWithAttr := make([]resolver.Address, len(s.ResolverState.Addresses))
	for i, addr := range s.ResolverState.Addresses {
		addrsWithAttr[i] = setConfigKey(addr, c.configStr)
	}
	s.BalancerConfig = nil
	s.ResolverState.Addresses = addrsWithAttr
	return b.Balancer.UpdateClientConnState(s)
}

func (b *testConfigBalancer) Close() {
	b.Balancer.Close()
}

var (
	wtbBuilder          balancer.Builder
	wtbParser           balancer.ConfigParser
	testBackendAddrStrs []string
)

const testBackendAddrsCount = 12

func init() {
	balancer.Register(newTestConfigBalancerBuilder())
	for i := 0; i < testBackendAddrsCount; i++ {
		testBackendAddrStrs = append(testBackendAddrStrs, fmt.Sprintf("%d.%d.%d.%d:%d", i, i, i, i, i))
	}
	wtbBuilder = balancer.Get(Name)
	wtbParser = wtbBuilder.(balancer.ConfigParser)

	NewRandomWRR = testutils.NewTestWRR
}

// TestWeightedTarget covers the cases that a sub-balancer is added and a
// sub-balancer is removed. It verifies that the addresses and balancer configs
// are forwarded to the right sub-balancer. This test is intended to test the
// glue code in weighted_target. It also tests an empty target config update,
// which should trigger a transient failure state update.
func (s) TestWeightedTarget(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with "cluster_1: round_robin".
	config1, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"round_robin": ""}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and an address with hierarchy path ["cluster_1"].
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1], Attributes: nil}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addr1, []string{"cluster_1"})}},
		BalancerConfig: config1,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}
	verifyAddressInNewSubConn(t, cc, addr1)

	// Send subconn state change.
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	// Test pick with one backend.
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p.Pick(balancer.PickInfo{})
		if gotSCSt.SubConn != sc1 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc1)
		}
	}

	// Remove cluster_1, and add "cluster_2: test_config_balancer". The
	// test_config_balancer adds an address attribute whose value is set to the
	// config that is passed to it.
	config2, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_2": {
       "weight":1,
       "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and one address with hierarchy path "cluster_2".
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2], Attributes: nil}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addr2, []string{"cluster_2"})}},
		BalancerConfig: config2,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Expect a new subConn from the test_config_balancer which has an address
	// attribute set to the config that was passed to it.
	verifyAddressInNewSubConn(t, cc, setConfigKey(addr2, "cluster_2"))

	// The subconn for cluster_1 should be shut down.
	scShutdown := <-cc.ShutdownSubConnCh
	if scShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scShutdown)
	}
	scShutdown.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Shutdown})

	sc2 := <-cc.NewSubConnCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p = <-cc.NewPickerCh

	// Test pick with one backend.
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p.Pick(balancer.PickInfo{})
		if gotSCSt.SubConn != sc2 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc2)
		}
	}

	// Replace child policy of "cluster_1" to "round_robin".
	config3, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_2": {
      "weight":1,
      "childPolicy": [{"round_robin": ""}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and an address with hierarchy path ["cluster_2"].
	addr3 := resolver.Address{Addr: testBackendAddrStrs[3], Attributes: nil}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addr3, []string{"cluster_2"})}},
		BalancerConfig: config3,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}
	verifyAddressInNewSubConn(t, cc, addr3)

	// The subconn from the test_config_balancer should be shut down.
	scShutdown = <-cc.ShutdownSubConnCh
	if scShutdown != sc2 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scShutdown)
	}
	scShutdown.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Shutdown})

	// Send subconn state change.
	sc3 := <-cc.NewSubConnCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p = <-cc.NewPickerCh

	// Test pick with one backend.
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p.Pick(balancer.PickInfo{})
		if gotSCSt.SubConn != sc3 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc3)
		}
	}
	// Update the Weighted Target Balancer with an empty address list and no
	// targets. This should cause a Transient Failure State update to the Client
	// Conn.
	emptyConfig, err := wtbParser.ParseConfig([]byte(`{}`))
	if err != nil {
		t.Fatalf("Failed to parse balancer config: %v", err)
	}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{},
		BalancerConfig: emptyConfig,
	}); err != nil {
		t.Fatalf("Failed to update ClientConn state: %v", err)
	}

	state := <-cc.NewStateCh
	if state != connectivity.TransientFailure {
		t.Fatalf("Empty target update should have triggered a TF state update, got: %v", state)
	}
}

// TestWeightedTarget_OneSubBalancer_AddRemoveBackend tests the case where we
// have a weighted target balancer will one sub-balancer, and we add and remove
// backends from the subBalancer.
func (s) TestWeightedTarget_OneSubBalancer_AddRemoveBackend(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with "cluster_1: round_robin".
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"round_robin": ""}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and an address with hierarchy path ["cluster_1"].
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addr1, []string{"cluster_1"})}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}
	verifyAddressInNewSubConn(t, cc, addr1)

	// Expect one SubConn, and move it to READY.
	sc1 := <-cc.NewSubConnCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	// Test pick with one backend.
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p.Pick(balancer.PickInfo{})
		if gotSCSt.SubConn != sc1 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc1)
		}
	}

	// Send two addresses.
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_1"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}
	verifyAddressInNewSubConn(t, cc, addr2)

	// Expect one new SubConn, and move it to READY.
	sc2 := <-cc.NewSubConnCh
	// Update the SubConn to become READY.
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p = <-cc.NewPickerCh

	// Test round robin pick.
	want := []balancer.SubConn{sc1, sc2}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Remove the first address.
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addr2, []string{"cluster_1"})}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Expect one SubConn to be shut down.
	scShutdown := <-cc.ShutdownSubConnCh
	if scShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scShutdown)
	}
	scShutdown.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Shutdown})
	p = <-cc.NewPickerCh

	// Test pick with only the second SubConn.
	for i := 0; i < 5; i++ {
		gotSC, _ := p.Pick(balancer.PickInfo{})
		if gotSC.SubConn != sc2 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSC, sc2)
		}
	}
}

// TestWeightedTarget_TwoSubBalancers_OneBackend tests the case where we have a
// weighted target balancer with two sub-balancers, each with one backend.
func (s) TestWeightedTarget_TwoSubBalancers_OneBackend(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with "cluster_1: test_config_balancer, cluster_2: test_config_balancer".
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with one address for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 2)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1},
		"cluster_2": {addr2},
	})

	// We expect a single subConn on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	sc2 := scs["cluster_2"][0].sc.(*testutils.TestSubConn)

	// Send state changes for both SubConns, and wait for the picker.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	// Test roundrobin on the last picker.
	want := []balancer.SubConn{sc1, sc2}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}
}

// TestWeightedTarget_TwoSubBalancers_MoreBackends tests the case where we have
// a weighted target balancer with two sub-balancers, each with more than one
// backend.
func (s) TestWeightedTarget_TwoSubBalancers_MoreBackends(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with "cluster_1: round_robin, cluster_2: round_robin".
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with two backends for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	addr3 := resolver.Address{Addr: testBackendAddrStrs[3]}
	addr4 := resolver.Address{Addr: testBackendAddrStrs[4]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_1"}),
			hierarchy.Set(addr3, []string{"cluster_2"}),
			hierarchy.Set(addr4, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 4)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1, addr2},
		"cluster_2": {addr3, addr4},
	})

	// We expect two subConns on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	sc2 := scs["cluster_1"][1].sc.(*testutils.TestSubConn)
	sc3 := scs["cluster_2"][0].sc.(*testutils.TestSubConn)
	sc4 := scs["cluster_2"][1].sc.(*testutils.TestSubConn)

	// Send state changes for all SubConns, and wait for the picker.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	// Test roundrobin on the last picker. RPCs should be sent equally to all
	// backends.
	want := []balancer.SubConn{sc1, sc2, sc3, sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Turn sc2's connection down, should be RR between balancers.
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	p = <-cc.NewPickerCh
	want = []balancer.SubConn{sc1, sc1, sc3, sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Shut down subConn corresponding to addr3.
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_1"}),
			hierarchy.Set(addr4, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}
	scShutdown := <-cc.ShutdownSubConnCh
	if scShutdown != sc3 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc3, scShutdown)
	}
	scShutdown.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Shutdown})
	p = <-cc.NewPickerCh
	want = []balancer.SubConn{sc1, sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Turn sc1's connection down.
	wantSubConnErr := errors.New("subConn connection error")
	sc1.UpdateState(balancer.SubConnState{
		ConnectivityState: connectivity.TransientFailure,
		ConnectionError:   wantSubConnErr,
	})
	p = <-cc.NewPickerCh
	want = []balancer.SubConn{sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Turn last connection to connecting.
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	p = <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		if _, err := p.Pick(balancer.PickInfo{}); err != balancer.ErrNoSubConnAvailable {
			t.Fatalf("want pick error %v, got %v", balancer.ErrNoSubConnAvailable, err)
		}
	}

	// Turn all connections down.
	sc4.UpdateState(balancer.SubConnState{
		ConnectivityState: connectivity.TransientFailure,
		ConnectionError:   wantSubConnErr,
	})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := cc.WaitForPicker(ctx, pickAndCheckError(wantSubConnErr)); err != nil {
		t.Fatal(err)
	}
}

// TestWeightedTarget_TwoSubBalancers_DifferentWeight_MoreBackends tests the
// case where we have a weighted target balancer with two sub-balancers of
// differing weights.
func (s) TestWeightedTarget_TwoSubBalancers_DifferentWeight_MoreBackends(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with two subBalancers, one with twice the weight of the other.
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight": 2,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with two backends for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	addr3 := resolver.Address{Addr: testBackendAddrStrs[3]}
	addr4 := resolver.Address{Addr: testBackendAddrStrs[4]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_1"}),
			hierarchy.Set(addr3, []string{"cluster_2"}),
			hierarchy.Set(addr4, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 4)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1, addr2},
		"cluster_2": {addr3, addr4},
	})

	// We expect two subConns on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	sc2 := scs["cluster_1"][1].sc.(*testutils.TestSubConn)
	sc3 := scs["cluster_2"][0].sc.(*testutils.TestSubConn)
	sc4 := scs["cluster_2"][1].sc.(*testutils.TestSubConn)

	// Send state changes for all SubConns, and wait for the picker.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	// Test roundrobin on the last picker. Twice the number of RPCs should be
	// sent to cluster_1 when compared to cluster_2.
	want := []balancer.SubConn{sc1, sc1, sc2, sc2, sc3, sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}
}

// TestWeightedTarget_ThreeSubBalancers_RemoveBalancer tests the case where we
// have a weighted target balancer with three sub-balancers and we remove one of
// the subBalancers.
func (s) TestWeightedTarget_ThreeSubBalancers_RemoveBalancer(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with two subBalancers, one with twice the weight of the other.
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    },
    "cluster_3": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_3"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with one backend for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	addr3 := resolver.Address{Addr: testBackendAddrStrs[3]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_2"}),
			hierarchy.Set(addr3, []string{"cluster_3"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 3)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1},
		"cluster_2": {addr2},
		"cluster_3": {addr3},
	})

	// We expect one subConn on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	sc2 := scs["cluster_2"][0].sc.(*testutils.TestSubConn)
	sc3 := scs["cluster_3"][0].sc.(*testutils.TestSubConn)

	// Send state changes for all SubConns, and wait for the picker.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	want := []balancer.SubConn{sc1, sc2, sc3}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Remove the second balancer, while the others two are ready.
	config, err = wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_3": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_3"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr3, []string{"cluster_3"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Removing a subBalancer causes the weighted target LB policy to push a new
	// picker which ensures that the removed subBalancer is not picked for RPCs.
	p = <-cc.NewPickerCh

	scShutdown := <-cc.ShutdownSubConnCh
	if scShutdown != sc2 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc2, scShutdown)
	}
	want = []balancer.SubConn{sc1, sc3}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Move balancer 3 into transient failure.
	wantSubConnErr := errors.New("subConn connection error")
	sc3.UpdateState(balancer.SubConnState{
		ConnectivityState: connectivity.TransientFailure,
		ConnectionError:   wantSubConnErr,
	})
	<-cc.NewPickerCh

	// Remove the first balancer, while the third is transient failure.
	config, err = wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_3": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_3"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr3, []string{"cluster_3"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Removing a subBalancer causes the weighted target LB policy to push a new
	// picker which ensures that the removed subBalancer is not picked for RPCs.

	scShutdown = <-cc.ShutdownSubConnCh
	if scShutdown != sc1 {
		t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, scShutdown)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := cc.WaitForPicker(ctx, pickAndCheckError(wantSubConnErr)); err != nil {
		t.Fatal(err)
	}
}

// TestWeightedTarget_TwoSubBalancers_ChangeWeight_MoreBackends tests the case
// where we have a weighted target balancer with two sub-balancers, and we
// change the weight of these subBalancers.
func (s) TestWeightedTarget_TwoSubBalancers_ChangeWeight_MoreBackends(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with two subBalancers, one with twice the weight of the other.
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight": 2,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with two backends for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	addr3 := resolver.Address{Addr: testBackendAddrStrs[3]}
	addr4 := resolver.Address{Addr: testBackendAddrStrs[4]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_1"}),
			hierarchy.Set(addr3, []string{"cluster_2"}),
			hierarchy.Set(addr4, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 4)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1, addr2},
		"cluster_2": {addr3, addr4},
	})

	// We expect two subConns on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	sc2 := scs["cluster_1"][1].sc.(*testutils.TestSubConn)
	sc3 := scs["cluster_2"][0].sc.(*testutils.TestSubConn)
	sc4 := scs["cluster_2"][1].sc.(*testutils.TestSubConn)

	// Send state changes for all SubConns, and wait for the picker.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc3.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	<-cc.NewPickerCh
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	<-cc.NewPickerCh
	sc4.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p := <-cc.NewPickerCh

	// Test roundrobin on the last picker. Twice the number of RPCs should be
	// sent to cluster_1 when compared to cluster_2.
	want := []balancer.SubConn{sc1, sc1, sc2, sc2, sc3, sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}

	// Change the weight of cluster_1.
	config, err = wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight": 3,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight": 1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_1"}),
			hierarchy.Set(addr3, []string{"cluster_2"}),
			hierarchy.Set(addr4, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Weight change causes a new picker to be pushed to the channel.
	p = <-cc.NewPickerCh
	want = []balancer.SubConn{sc1, sc1, sc1, sc2, sc2, sc2, sc3, sc4}
	if err := testutils.IsRoundRobin(want, testutils.SubConnFromPicker(p)); err != nil {
		t.Fatalf("want %v, got %v", want, err)
	}
}

// TestWeightedTarget_InitOneSubBalancerTransientFailure tests that at init
// time, with two sub-balancers, if one sub-balancer reports transient_failure,
// the picks won't fail with transient_failure, and should instead wait for the
// other sub-balancer.
func (s) TestWeightedTarget_InitOneSubBalancerTransientFailure(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with "cluster_1: test_config_balancer, cluster_2: test_config_balancer".
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with one address for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 2)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1},
		"cluster_2": {addr2},
	})

	// We expect a single subConn on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	_ = scs["cluster_2"][0].sc

	// Set one subconn to TransientFailure, this will trigger one sub-balancer
	// to report transient failure.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})

	p := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		r, err := p.Pick(balancer.PickInfo{})
		if err != balancer.ErrNoSubConnAvailable {
			t.Fatalf("want pick to fail with %v, got result %v, err %v", balancer.ErrNoSubConnAvailable, r, err)
		}
	}
}

// Test that with two sub-balancers, both in transient_failure, if one turns
// connecting, the overall state stays in transient_failure, and all picks
// return transient failure error.
func (s) TestBalancerGroup_SubBalancerTurnsConnectingFromTransientFailure(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	// Start with "cluster_1: test_config_balancer, cluster_2: test_config_balancer".
	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_1"}]
    },
    "cluster_2": {
      "weight":1,
      "childPolicy": [{"test_config_balancer": "cluster_2"}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config with one address for each cluster.
	addr1 := resolver.Address{Addr: testBackendAddrStrs[1]}
	addr2 := resolver.Address{Addr: testBackendAddrStrs[2]}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			hierarchy.Set(addr1, []string{"cluster_1"}),
			hierarchy.Set(addr2, []string{"cluster_2"}),
		}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	scs := waitForNewSubConns(t, cc, 2)
	verifySubConnAddrs(t, scs, map[string][]resolver.Address{
		"cluster_1": {addr1},
		"cluster_2": {addr2},
	})

	// We expect a single subConn on each subBalancer.
	sc1 := scs["cluster_1"][0].sc.(*testutils.TestSubConn)
	sc2 := scs["cluster_2"][0].sc.(*testutils.TestSubConn)

	// Set both subconn to TransientFailure, this will put both sub-balancers in
	// transient failure.
	wantSubConnErr := errors.New("subConn connection error")
	sc1.UpdateState(balancer.SubConnState{
		ConnectivityState: connectivity.TransientFailure,
		ConnectionError:   wantSubConnErr,
	})
	<-cc.NewPickerCh
	sc2.UpdateState(balancer.SubConnState{
		ConnectivityState: connectivity.TransientFailure,
		ConnectionError:   wantSubConnErr,
	})
	p := <-cc.NewPickerCh

	for i := 0; i < 5; i++ {
		if _, err := p.Pick(balancer.PickInfo{}); err == nil || !strings.Contains(err.Error(), wantSubConnErr.Error()) {
			t.Fatalf("picker.Pick() returned error: %v, want: %v", err, wantSubConnErr)
		}
	}

	// Set one subconn to Connecting, it shouldn't change the overall state.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	select {
	case <-time.After(100 * time.Millisecond):
	case <-cc.NewPickerCh:
		t.Fatal("received new picker from the LB policy when expecting none")
	}

	for i := 0; i < 5; i++ {
		if _, err := p.Pick(balancer.PickInfo{}); err == nil || !strings.Contains(err.Error(), wantSubConnErr.Error()) {
			t.Fatalf("picker.Pick() returned error: %v, want: %v", err, wantSubConnErr)
		}
	}
}

// Verify that a SubConn is created with the expected address and hierarchy
// path cleared.
func verifyAddressInNewSubConn(t *testing.T, cc *testutils.BalancerClientConn, addr resolver.Address) {
	t.Helper()

	gotAddr := <-cc.NewSubConnAddrsCh
	wantAddr := []resolver.Address{hierarchy.Set(addr, []string{})}
	if diff := cmp.Diff(gotAddr, wantAddr, cmp.AllowUnexported(attributes.Attributes{})); diff != "" {
		t.Fatalf("got unexpected new subconn addrs: %v", diff)
	}
}

// subConnWithAddr wraps a subConn and the address for which it was created.
type subConnWithAddr struct {
	sc   balancer.SubConn
	addr resolver.Address
}

// waitForNewSubConns waits for `num` number of subConns to be created. This is
// expected to be used from tests using the "test_config_balancer" LB policy,
// which adds an address attribute with value set to the balancer config.
//
// Returned value is a map from subBalancer (identified by its config) to
// subConns created by it.
func waitForNewSubConns(t *testing.T, cc *testutils.BalancerClientConn, num int) map[string][]subConnWithAddr {
	t.Helper()

	scs := make(map[string][]subConnWithAddr)
	for i := 0; i < num; i++ {
		addrs := <-cc.NewSubConnAddrsCh
		if len(addrs) != 1 {
			t.Fatalf("received subConns with %d addresses, want 1", len(addrs))
		}
		cfg, ok := getConfigKey(addrs[0].Attributes)
		if !ok {
			t.Fatalf("received subConn address %v contains no attribute for balancer config", addrs[0])
		}
		sc := <-cc.NewSubConnCh
		scWithAddr := subConnWithAddr{sc: sc, addr: addrs[0]}
		scs[cfg] = append(scs[cfg], scWithAddr)
	}
	return scs
}

func verifySubConnAddrs(t *testing.T, scs map[string][]subConnWithAddr, wantSubConnAddrs map[string][]resolver.Address) {
	t.Helper()

	if len(scs) != len(wantSubConnAddrs) {
		t.Fatalf("got new subConns %+v, want %v", scs, wantSubConnAddrs)
	}
	for cfg, scsWithAddr := range scs {
		if len(scsWithAddr) != len(wantSubConnAddrs[cfg]) {
			t.Fatalf("got new subConns %+v, want %v", scs, wantSubConnAddrs)
		}
		wantAddrs := wantSubConnAddrs[cfg]
		for i, scWithAddr := range scsWithAddr {
			if diff := cmp.Diff(wantAddrs[i].Addr, scWithAddr.addr.Addr); diff != "" {
				t.Fatalf("got unexpected new subconn addrs: %v", diff)
			}
		}
	}
}

const initIdleBalancerName = "test-init-Idle-balancer"

var errTestInitIdle = fmt.Errorf("init Idle balancer error 0")

func init() {
	stub.Register(initIdleBalancerName, stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, opts balancer.ClientConnState) error {
			sc, err := bd.ClientConn.NewSubConn(opts.ResolverState.Addresses, balancer.NewSubConnOptions{
				StateListener: func(state balancer.SubConnState) {
					err := fmt.Errorf("wrong picker error")
					if state.ConnectivityState == connectivity.Idle {
						err = errTestInitIdle
					}
					bd.ClientConn.UpdateState(balancer.State{
						ConnectivityState: state.ConnectivityState,
						Picker:            &testutils.TestConstPicker{Err: err},
					})
				},
			})
			if err != nil {
				return err
			}
			sc.Connect()
			return nil
		},
	})
}

// TestInitialIdle covers the case that if the child reports Idle, the overall
// state will be Idle.
func (s) TestInitialIdle(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"test-init-Idle-balancer": ""}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and an address with hierarchy path ["cluster_1"].
	addrs := []resolver.Address{{Addr: testBackendAddrStrs[0], Attributes: nil}}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addrs[0], []string{"cds:cluster_1"})}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Verify that a subconn is created with the address, and the hierarchy path
	// in the address is cleared.
	for range addrs {
		sc := <-cc.NewSubConnCh
		sc.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})
	}

	if state := <-cc.NewStateCh; state != connectivity.Idle {
		t.Fatalf("Received aggregated state: %v, want Idle", state)
	}
}

// TestIgnoreSubBalancerStateTransitions covers the case that if the child reports a
// transition from TF to Connecting, the overall state will still be TF.
func (s) TestIgnoreSubBalancerStateTransitions(t *testing.T) {
	cc := &tcc{BalancerClientConn: testutils.NewBalancerClientConn(t)}

	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"round_robin": ""}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and an address with hierarchy path ["cluster_1"].
	addr := resolver.Address{Addr: testBackendAddrStrs[0], Attributes: nil}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addr, []string{"cluster_1"})}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	sc := <-cc.NewSubConnCh
	sc.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	sc.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})

	// Verify that the SubConnState update from TF to Connecting is ignored.
	if len(cc.states) != 2 || cc.states[0].ConnectivityState != connectivity.Connecting || cc.states[1].ConnectivityState != connectivity.TransientFailure {
		t.Fatalf("cc.states = %v; want [Connecting, TransientFailure]", cc.states)
	}
}

// tcc wraps a testutils.TestClientConn but stores all state transitions in a
// slice.
type tcc struct {
	*testutils.BalancerClientConn
	states []balancer.State
}

func (t *tcc) UpdateState(bs balancer.State) {
	t.states = append(t.states, bs)
	t.BalancerClientConn.UpdateState(bs)
}

func (s) TestUpdateStatePauses(t *testing.T) {
	cc := &tcc{BalancerClientConn: testutils.NewBalancerClientConn(t)}

	balFuncs := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, _ balancer.ClientConnState) error {
			bd.ClientConn.UpdateState(balancer.State{ConnectivityState: connectivity.TransientFailure, Picker: nil})
			bd.ClientConn.UpdateState(balancer.State{ConnectivityState: connectivity.Ready, Picker: nil})
			return nil
		},
	}
	stub.Register("update_state_balancer", balFuncs)

	wtb := wtbBuilder.Build(cc, balancer.BuildOptions{})
	defer wtb.Close()

	config, err := wtbParser.ParseConfig([]byte(`
{
  "targets": {
    "cluster_1": {
      "weight":1,
      "childPolicy": [{"update_state_balancer": ""}]
    }
  }
}`))
	if err != nil {
		t.Fatalf("failed to parse balancer config: %v", err)
	}

	// Send the config, and an address with hierarchy path ["cluster_1"].
	addrs := []resolver.Address{{Addr: testBackendAddrStrs[0], Attributes: nil}}
	if err := wtb.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: []resolver.Address{hierarchy.Set(addrs[0], []string{"cds:cluster_1"})}},
		BalancerConfig: config,
	}); err != nil {
		t.Fatalf("failed to update ClientConn state: %v", err)
	}

	// Verify that the only state update is the second one called by the child.
	if len(cc.states) != 1 || cc.states[0].ConnectivityState != connectivity.Ready {
		t.Fatalf("cc.states = %v; want [connectivity.Ready]", cc.states)
	}
}
