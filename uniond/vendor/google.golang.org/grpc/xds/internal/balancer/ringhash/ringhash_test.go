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

package ringhash

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/weightedroundrobin"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/xds/internal"
)

var (
	cmpOpts = cmp.Options{
		cmp.AllowUnexported(testutils.TestSubConn{}, ringEntry{}, subConn{}),
		cmpopts.IgnoreFields(subConn{}, "mu"),
		cmpopts.IgnoreFields(testutils.TestSubConn{}, "connectCalled"),
	}
)

const (
	defaultTestTimeout      = 10 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond

	testBackendAddrsCount = 12
)

var (
	testBackendAddrStrs []string
	testConfig          = &LBConfig{MinRingSize: 1, MaxRingSize: 10}
)

func init() {
	for i := 0; i < testBackendAddrsCount; i++ {
		testBackendAddrStrs = append(testBackendAddrStrs, fmt.Sprintf("%d.%d.%d.%d:%d", i, i, i, i, i))
	}
}

func ctxWithHash(h uint64) context.Context {
	return SetRequestHash(context.Background(), h)
}

// setupTest creates the balancer, and does an initial sanity check.
func setupTest(t *testing.T, addrs []resolver.Address) (*testutils.BalancerClientConn, balancer.Balancer, balancer.Picker) {
	t.Helper()
	cc := testutils.NewBalancerClientConn(t)
	builder := balancer.Get(Name)
	b := builder.Build(cc, balancer.BuildOptions{})
	if b == nil {
		t.Fatalf("builder.Build(%s) failed and returned nil", Name)
	}
	if err := b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: addrs},
		BalancerConfig: testConfig,
	}); err != nil {
		t.Fatalf("UpdateClientConnState returned err: %v", err)
	}

	for _, addr := range addrs {
		addr1 := <-cc.NewSubConnAddrsCh
		if want := []resolver.Address{addr}; !cmp.Equal(addr1, want, cmp.AllowUnexported(attributes.Attributes{})) {
			t.Fatalf("got unexpected new subconn addrs: %v", cmp.Diff(addr1, want, cmp.AllowUnexported(attributes.Attributes{})))
		}
		sc1 := <-cc.NewSubConnCh
		// All the SubConns start in Idle, and should not Connect().
		select {
		case <-sc1.ConnectCh:
			t.Errorf("unexpected Connect() from SubConn %v", sc1)
		case <-time.After(defaultTestShortTimeout):
		}
	}

	// Should also have a picker, with all SubConns in Idle.
	p1 := <-cc.NewPickerCh
	return cc, b, p1
}

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// TestUpdateClientConnState_NewRingSize tests the scenario where the ringhash
// LB policy receives new configuration which specifies new values for the ring
// min and max sizes. The test verifies that a new ring is created and a new
// picker is sent to the ClientConn.
func (s) TestUpdateClientConnState_NewRingSize(t *testing.T) {
	origMinRingSize, origMaxRingSize := 1, 10 // Configured from `testConfig` in `setupTest`
	newMinRingSize, newMaxRingSize := 20, 100

	addrs := []resolver.Address{{Addr: testBackendAddrStrs[0]}}
	cc, b, p1 := setupTest(t, addrs)
	ring1 := p1.(*picker).ring
	if ringSize := len(ring1.items); ringSize < origMinRingSize || ringSize > origMaxRingSize {
		t.Fatalf("Ring created with size %d, want between [%d, %d]", ringSize, origMinRingSize, origMaxRingSize)
	}

	if err := b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: addrs},
		BalancerConfig: &LBConfig{MinRingSize: uint64(newMinRingSize), MaxRingSize: uint64(newMaxRingSize)},
	}); err != nil {
		t.Fatalf("UpdateClientConnState returned err: %v", err)
	}

	var ring2 *ring
	select {
	case <-time.After(defaultTestTimeout):
		t.Fatal("Timeout when waiting for a picker update after a configuration update")
	case p2 := <-cc.NewPickerCh:
		ring2 = p2.(*picker).ring
	}
	if ringSize := len(ring2.items); ringSize < newMinRingSize || ringSize > newMaxRingSize {
		t.Fatalf("Ring created with size %d, want between [%d, %d]", ringSize, newMinRingSize, newMaxRingSize)
	}
}

func (s) TestOneSubConn(t *testing.T) {
	wantAddr1 := resolver.Address{Addr: testBackendAddrStrs[0]}
	cc, _, p0 := setupTest(t, []resolver.Address{wantAddr1})
	ring0 := p0.(*picker).ring

	firstHash := ring0.items[0].hash
	// firstHash-1 will pick the first (and only) SubConn from the ring.
	testHash := firstHash - 1
	// The first pick should be queued, and should trigger Connect() on the only
	// SubConn.
	if _, err := p0.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)}); err != balancer.ErrNoSubConnAvailable {
		t.Fatalf("first pick returned err %v, want %v", err, balancer.ErrNoSubConnAvailable)
	}
	sc0 := ring0.items[0].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc0.ConnectCh:
	case <-time.After(defaultTestTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", sc0)
	}

	// Send state updates to Ready.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// Test pick with one backend.
	p1 := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p1.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)})
		if gotSCSt.SubConn != sc0 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc0)
		}
	}
}

// TestThreeBackendsAffinity covers that there are 3 SubConns, RPCs with the
// same hash always pick the same SubConn. When the one picked is down, another
// one will be picked.
func (s) TestThreeSubConnsAffinity(t *testing.T) {
	wantAddrs := []resolver.Address{
		{Addr: testBackendAddrStrs[0]},
		{Addr: testBackendAddrStrs[1]},
		{Addr: testBackendAddrStrs[2]},
	}
	cc, _, p0 := setupTest(t, wantAddrs)
	// This test doesn't update addresses, so this ring will be used by all the
	// pickers.
	ring0 := p0.(*picker).ring

	firstHash := ring0.items[0].hash
	// firstHash+1 will pick the second SubConn from the ring.
	testHash := firstHash + 1
	// The first pick should be queued, and should trigger Connect() on the only
	// SubConn.
	if _, err := p0.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)}); err != balancer.ErrNoSubConnAvailable {
		t.Fatalf("first pick returned err %v, want %v", err, balancer.ErrNoSubConnAvailable)
	}
	// The picked SubConn should be the second in the ring.
	sc0 := ring0.items[1].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc0.ConnectCh:
	case <-time.After(defaultTestTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", sc0)
	}

	// Send state updates to Ready.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p1 := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p1.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)})
		if gotSCSt.SubConn != sc0 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc0)
		}
	}

	// Turn down the subConn in use.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	p2 := <-cc.NewPickerCh
	// Pick with the same hash should be queued, because the SubConn after the
	// first picked is Idle.
	if _, err := p2.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)}); err != balancer.ErrNoSubConnAvailable {
		t.Fatalf("first pick returned err %v, want %v", err, balancer.ErrNoSubConnAvailable)
	}

	// The third SubConn in the ring should connect.
	sc1 := ring0.items[2].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc1.ConnectCh:
	case <-time.After(defaultTestTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", sc1)
	}

	// Send state updates to Ready.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	// New picks should all return this SubConn.
	p3 := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p3.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)})
		if gotSCSt.SubConn != sc1 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc1)
		}
	}

	// Now, after backoff, the first picked SubConn will turn Idle.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})
	// The picks above should have queued Connect() for the first picked
	// SubConn, so this Idle state change will trigger a Connect().
	select {
	case <-sc0.ConnectCh:
	case <-time.After(defaultTestTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", sc0)
	}

	// After the first picked SubConn turn Ready, new picks should return it
	// again (even though the second picked SubConn is also Ready).
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})
	p4 := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p4.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)})
		if gotSCSt.SubConn != sc0 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc0)
		}
	}
}

// TestThreeBackendsAffinity covers that there are 3 SubConns, RPCs with the
// same hash always pick the same SubConn. Then try different hash to pick
// another backend, and verify the first hash still picks the first backend.
func (s) TestThreeSubConnsAffinityMultiple(t *testing.T) {
	wantAddrs := []resolver.Address{
		{Addr: testBackendAddrStrs[0]},
		{Addr: testBackendAddrStrs[1]},
		{Addr: testBackendAddrStrs[2]},
	}
	cc, _, p0 := setupTest(t, wantAddrs)
	// This test doesn't update addresses, so this ring will be used by all the
	// pickers.
	ring0 := p0.(*picker).ring

	firstHash := ring0.items[0].hash
	// firstHash+1 will pick the second SubConn from the ring.
	testHash := firstHash + 1
	// The first pick should be queued, and should trigger Connect() on the only
	// SubConn.
	if _, err := p0.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)}); err != balancer.ErrNoSubConnAvailable {
		t.Fatalf("first pick returned err %v, want %v", err, balancer.ErrNoSubConnAvailable)
	}
	sc0 := ring0.items[1].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc0.ConnectCh:
	case <-time.After(defaultTestTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", sc0)
	}

	// Send state updates to Ready.
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// First hash should always pick sc0.
	p1 := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p1.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)})
		if gotSCSt.SubConn != sc0 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc0)
		}
	}

	secondHash := ring0.items[1].hash
	// secondHash+1 will pick the third SubConn from the ring.
	testHash2 := secondHash + 1
	if _, err := p0.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash2)}); err != balancer.ErrNoSubConnAvailable {
		t.Fatalf("first pick returned err %v, want %v", err, balancer.ErrNoSubConnAvailable)
	}
	sc1 := ring0.items[2].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc1.ConnectCh:
	case <-time.After(defaultTestTimeout):
		t.Errorf("timeout waiting for Connect() from SubConn %v", sc1)
	}
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Connecting})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Ready})

	// With the new generated picker, hash2 always picks sc1.
	p2 := <-cc.NewPickerCh
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p2.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash2)})
		if gotSCSt.SubConn != sc1 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc1)
		}
	}
	// But the first hash still picks sc0.
	for i := 0; i < 5; i++ {
		gotSCSt, _ := p2.Pick(balancer.PickInfo{Ctx: ctxWithHash(testHash)})
		if gotSCSt.SubConn != sc0 {
			t.Fatalf("picker.Pick, got %v, want SubConn=%v", gotSCSt, sc0)
		}
	}
}

// TestAddrWeightChange covers the following scenarios after setting up the
// balancer with 3 addresses [A, B, C]:
//   - updates balancer with [A, B, C], a new Picker should not be sent.
//   - updates balancer with [A, B] (C removed), a new Picker is sent and the
//     ring is updated.
//   - updates balancer with [A, B], but B has a weight of 2, a new Picker is
//     sent.  And the new ring should contain the correct number of entries
//     and weights.
func (s) TestAddrWeightChange(t *testing.T) {
	addrs := []resolver.Address{
		{Addr: testBackendAddrStrs[0]},
		{Addr: testBackendAddrStrs[1]},
		{Addr: testBackendAddrStrs[2]},
	}
	cc, b, p0 := setupTest(t, addrs)
	ring0 := p0.(*picker).ring

	// Update with the same addresses, should not send a new Picker.
	if err := b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: addrs},
		BalancerConfig: testConfig,
	}); err != nil {
		t.Fatalf("UpdateClientConnState returned err: %v", err)
	}
	select {
	case <-cc.NewPickerCh:
		t.Fatalf("unexpected picker after UpdateClientConn with the same addresses")
	case <-time.After(defaultTestShortTimeout):
	}

	// Delete an address, should send a new Picker.
	if err := b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			{Addr: testBackendAddrStrs[0]},
			{Addr: testBackendAddrStrs[1]},
		}},
		BalancerConfig: testConfig,
	}); err != nil {
		t.Fatalf("UpdateClientConnState returned err: %v", err)
	}
	var p1 balancer.Picker
	select {
	case p1 = <-cc.NewPickerCh:
	case <-time.After(defaultTestTimeout):
		t.Fatalf("timeout waiting for picker after UpdateClientConn with different addresses")
	}
	ring1 := p1.(*picker).ring
	if ring1 == ring0 {
		t.Fatalf("new picker after removing address has the same ring as before, want different")
	}

	// Another update with the same addresses, but different weight.
	if err := b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{
			{Addr: testBackendAddrStrs[0]},
			weightedroundrobin.SetAddrInfo(
				resolver.Address{Addr: testBackendAddrStrs[1]},
				weightedroundrobin.AddrInfo{Weight: 2}),
		}},
		BalancerConfig: testConfig,
	}); err != nil {
		t.Fatalf("UpdateClientConnState returned err: %v", err)
	}
	var p2 balancer.Picker
	select {
	case p2 = <-cc.NewPickerCh:
	case <-time.After(defaultTestTimeout):
		t.Fatalf("timeout waiting for picker after UpdateClientConn with different addresses")
	}
	if p2.(*picker).ring == ring1 {
		t.Fatalf("new picker after changing address weight has the same ring as before, want different")
	}
	// With the new update, the ring must look like this:
	//   [
	//     {idx:0 sc: {addr: testBackendAddrStrs[0], weight: 1}},
	//     {idx:1 sc: {addr: testBackendAddrStrs[1], weight: 2}},
	//     {idx:2 sc: {addr: testBackendAddrStrs[2], weight: 2}},
	//   ].
	if len(p2.(*picker).ring.items) != 3 {
		t.Fatalf("new picker after changing address weight has %d entries, want 3", len(p2.(*picker).ring.items))
	}
	for _, i := range p2.(*picker).ring.items {
		if i.sc.addr == testBackendAddrStrs[0] {
			if i.sc.weight != 1 {
				t.Fatalf("new picker after changing address weight has weight %d for %v, want 1", i.sc.weight, i.sc.addr)
			}
		}
		if i.sc.addr == testBackendAddrStrs[1] {
			if i.sc.weight != 2 {
				t.Fatalf("new picker after changing address weight has weight %d for %v, want 2", i.sc.weight, i.sc.addr)
			}
		}
	}
}

// TestSubConnToConnectWhenOverallTransientFailure covers the situation when the
// overall state is TransientFailure, the SubConns turning Idle will trigger the
// next SubConn in the ring to Connect(). But not when the overall state is not
// TransientFailure.
func (s) TestSubConnToConnectWhenOverallTransientFailure(t *testing.T) {
	wantAddrs := []resolver.Address{
		{Addr: testBackendAddrStrs[0]},
		{Addr: testBackendAddrStrs[1]},
		{Addr: testBackendAddrStrs[2]},
	}
	_, _, p0 := setupTest(t, wantAddrs)
	ring0 := p0.(*picker).ring

	// ringhash won't tell SCs to connect until there is an RPC, so simulate
	// one now.
	p0.Pick(balancer.PickInfo{Ctx: context.Background()})

	// Turn the first subconn to transient failure.
	sc0 := ring0.items[0].sc.sc.(*testutils.TestSubConn)
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	sc0.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})

	// It will trigger the second subconn to connect (because overall state is
	// Connect (when one subconn is TF)).
	sc1 := ring0.items[1].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc1.ConnectCh:
	case <-time.After(defaultTestShortTimeout):
		t.Fatalf("timeout waiting for Connect() from SubConn %v", sc1)
	}

	// Turn the second subconn to TF. This will set the overall state to TF.
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	sc1.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})

	// It will trigger the third subconn to connect.
	sc2 := ring0.items[2].sc.sc.(*testutils.TestSubConn)
	select {
	case <-sc2.ConnectCh:
	case <-time.After(defaultTestShortTimeout):
		t.Fatalf("timeout waiting for Connect() from SubConn %v", sc2)
	}

	// Turn the third subconn to TF. This will set the overall state to TF.
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})

	// It will trigger the first subconn to connect.
	select {
	case <-sc0.ConnectCh:
	case <-time.After(defaultTestShortTimeout):
		t.Fatalf("timeout waiting for Connect() from SubConn %v", sc0)
	}

	// Turn the third subconn to TF again.
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.TransientFailure})
	sc2.UpdateState(balancer.SubConnState{ConnectivityState: connectivity.Idle})

	// This will not trigger any new Connect() on the SubConns, because sc0 is
	// still attempting to connect, and we only need one SubConn to connect.
	select {
	case <-sc0.ConnectCh:
		t.Fatalf("unexpected Connect() from SubConn %v", sc0)
	case <-sc1.ConnectCh:
		t.Fatalf("unexpected Connect() from SubConn %v", sc1)
	case <-sc2.ConnectCh:
		t.Fatalf("unexpected Connect() from SubConn %v", sc2)
	case <-time.After(defaultTestShortTimeout):
	}
}

func (s) TestConnectivityStateEvaluatorRecordTransition(t *testing.T) {
	tests := []struct {
		name     string
		from, to []connectivity.State
		want     connectivity.State
	}{
		{
			name: "one ready",
			from: []connectivity.State{connectivity.Idle},
			to:   []connectivity.State{connectivity.Ready},
			want: connectivity.Ready,
		},
		{
			name: "one connecting",
			from: []connectivity.State{connectivity.Idle},
			to:   []connectivity.State{connectivity.Connecting},
			want: connectivity.Connecting,
		},
		{
			name: "one ready one transient failure",
			from: []connectivity.State{connectivity.Idle, connectivity.Idle},
			to:   []connectivity.State{connectivity.Ready, connectivity.TransientFailure},
			want: connectivity.Ready,
		},
		{
			name: "one connecting one transient failure",
			from: []connectivity.State{connectivity.Idle, connectivity.Idle},
			to:   []connectivity.State{connectivity.Connecting, connectivity.TransientFailure},
			want: connectivity.Connecting,
		},
		{
			name: "one connecting two transient failure",
			from: []connectivity.State{connectivity.Idle, connectivity.Idle, connectivity.Idle},
			to:   []connectivity.State{connectivity.Connecting, connectivity.TransientFailure, connectivity.TransientFailure},
			want: connectivity.TransientFailure,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cse := &connectivityStateEvaluator{}
			var got connectivity.State
			for i, fff := range tt.from {
				ttt := tt.to[i]
				got = cse.recordTransition(fff, ttt)
			}
			if got != tt.want {
				t.Errorf("recordTransition() = %v, want %v", got, tt.want)
			}
		})
	}
}

// TestAddrBalancerAttributesChange tests the case where the ringhash balancer
// receives a ClientConnUpdate with the same config and addresses as received in
// the previous update. Although the `BalancerAttributes` contents are the same,
// the pointer is different. This test verifies that subConns are not recreated
// in this scenario.
func (s) TestAddrBalancerAttributesChange(t *testing.T) {
	addrs1 := []resolver.Address{internal.SetLocalityID(resolver.Address{Addr: testBackendAddrStrs[0]}, internal.LocalityID{Region: "americas"})}
	cc, b, _ := setupTest(t, addrs1)

	addrs2 := []resolver.Address{internal.SetLocalityID(resolver.Address{Addr: testBackendAddrStrs[0]}, internal.LocalityID{Region: "americas"})}
	if err := b.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  resolver.State{Addresses: addrs2},
		BalancerConfig: testConfig,
	}); err != nil {
		t.Fatalf("UpdateClientConnState returned err: %v", err)
	}
	select {
	case <-cc.NewSubConnCh:
		t.Fatal("new subConn created for an update with the same addresses")
	case <-time.After(defaultTestShortTimeout):
	}
}
