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

package gracefulswitch

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
)

const (
	defaultTestTimeout      = 5 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func setup(t *testing.T) (*testutils.BalancerClientConn, *Balancer) {
	tcc := testutils.NewBalancerClientConn(t)
	return tcc, NewBalancer(tcc, balancer.BuildOptions{})
}

// TestSuccessfulFirstUpdate tests a basic scenario for the graceful switch load
// balancer, where it is setup with a balancer which should populate the current
// load balancer. Any ClientConn updates should then be forwarded to this
// current load balancer.
func (s) TestSuccessfulFirstUpdate(t *testing.T) {
	_, gsb := setup(t)
	if err := gsb.SwitchTo(mockBalancerBuilder1{}); err != nil {
		t.Fatalf("Balancer.SwitchTo failed with error: %v", err)
	}
	if gsb.balancerCurrent == nil {
		t.Fatal("current balancer not populated after a successful call to SwitchTo()")
	}
	// This will be used to update the graceful switch balancer. This update
	// should simply be forwarded down to the current load balancing policy.
	ccs := balancer.ClientConnState{
		BalancerConfig: mockBalancerConfig{},
	}

	// Updating ClientConnState should forward the update exactly as is to the
	// current balancer.
	if err := gsb.UpdateClientConnState(ccs); err != nil {
		t.Fatalf("Balancer.UpdateClientConnState(%v) failed: %v", ccs, err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := gsb.balancerCurrent.Balancer.(*mockBalancer).waitForClientConnUpdate(ctx, ccs); err != nil {
		t.Fatal(err)
	}
}

// TestTwoBalancersSameType tests the scenario where there is a graceful switch
// load balancer setup with a current and pending load balancer of the same
// type. Any ClientConn update should be forwarded to the current lb if there is
// a current lb and no pending lb, and the only the pending lb if the graceful
// switch balancer contains both a current lb and a pending lb. The pending load
// balancer should also swap into current whenever it updates with a
// connectivity state other than CONNECTING.
func (s) TestTwoBalancersSameType(t *testing.T) {
	tcc, gsb := setup(t)
	// This will be used to update the graceful switch balancer. This update
	// should simply be forwarded down to either the current or pending load
	// balancing policy.
	ccs := balancer.ClientConnState{
		BalancerConfig: mockBalancerConfig{},
	}

	gsb.SwitchTo(mockBalancerBuilder1{})
	gsb.UpdateClientConnState(ccs)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := gsb.balancerCurrent.Balancer.(*mockBalancer).waitForClientConnUpdate(ctx, ccs); err != nil {
		t.Fatal(err)
	}

	// The current balancer reporting READY should cause this state
	// to be forwarded to the ClientConn.
	gsb.balancerCurrent.Balancer.(*mockBalancer).updateState(balancer.State{
		ConnectivityState: connectivity.Ready,
		Picker:            &neverErrPicker{},
	})

	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a UpdateState call on the ClientConn")
	case state := <-tcc.NewStateCh:
		if state != connectivity.Ready {
			t.Fatalf("current balancer reports connectivity state %v, want %v", state, connectivity.Ready)
		}
	}

	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a UpdateState call on the ClientConn")
	case picker := <-tcc.NewPickerCh:
		// Should receive a never err picker.
		if _, err := picker.Pick(balancer.PickInfo{}); err != nil {
			t.Fatalf("ClientConn should have received a never err picker from an UpdateState call")
		}
	}

	// An explicit call to switchTo, even if the same type, should cause the
	// balancer to build a new balancer for pending.
	gsb.SwitchTo(mockBalancerBuilder1{})
	if gsb.balancerPending == nil {
		t.Fatal("pending balancer not populated after another call to SwitchTo()")
	}

	// A ClientConn update received should be forwarded to the new pending LB
	// policy, and not the current one.
	gsb.UpdateClientConnState(ccs)
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if err := gsb.balancerCurrent.Balancer.(*mockBalancer).waitForClientConnUpdate(sCtx, ccs); err == nil {
		t.Fatal("current balancer received a ClientConn update when there is a pending balancer")
	}
	if err := gsb.balancerPending.Balancer.(*mockBalancer).waitForClientConnUpdate(ctx, ccs); err != nil {
		t.Fatal(err)
	}

	// If the pending load balancer reports that is CONNECTING, no update should
	// be sent to the ClientConn.
	gsb.balancerPending.Balancer.(*mockBalancer).updateState(balancer.State{
		ConnectivityState: connectivity.Connecting,
	})
	sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-tcc.NewStateCh:
		t.Fatal("balancerPending reporting CONNECTING should not forward up to the ClientConn")
	case <-sCtx.Done():
	}

	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	// If the pending load balancer reports a state other than CONNECTING, the
	// pending load balancer is logically warmed up, and the ClientConn should
	// be updated with the State and Picker to start using the new policy. The
	// pending load balancing policy should also be switched into the current
	// load balancer.
	gsb.balancerPending.Balancer.(*mockBalancer).updateState(balancer.State{
		ConnectivityState: connectivity.Ready,
		Picker:            &neverErrPicker{},
	})

	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a UpdateState call on the ClientConn")
	case state := <-tcc.NewStateCh:
		if state != connectivity.Ready {
			t.Fatalf("pending balancer reports connectivity state %v, want %v", state, connectivity.Ready)
		}
	}

	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a UpdateState call on the ClientConn")
	case picker := <-tcc.NewPickerCh:
		// This picker should be the recent one sent from UpdateState(), a never
		// err picker, not the nil picker from two updateState() calls previous.
		if picker == nil {
			t.Fatalf("ClientConn should have received a never err picker, which is the most recent picker, from an UpdateState call")
		}
		if _, err := picker.Pick(balancer.PickInfo{}); err != nil {
			t.Fatalf("ClientConn should have received a never err picker, which is the most recent picker, from an UpdateState call")
		}
	}
	// The current balancer should be closed as a result of the swap.
	if err := currBal.waitForClose(ctx); err != nil {
		t.Fatal(err)
	}
}

// TestCurrentNotReadyPendingUpdate tests the scenario where there is a current
// and pending load balancer setup in the graceful switch load balancer, and the
// current LB is not in the connectivity state READY. Any update from the
// pending load balancer should cause the graceful switch load balancer to swap
// the pending into current, and update the ClientConn with the pending load
// balancers state.
func (s) TestCurrentNotReadyPendingUpdate(t *testing.T) {
	tcc, gsb := setup(t)
	gsb.SwitchTo(mockBalancerBuilder1{})
	gsb.SwitchTo(mockBalancerBuilder1{})
	if gsb.balancerPending == nil {
		t.Fatal("pending balancer not populated after another call to SwitchTo()")
	}
	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	// Due to the current load balancer not being in state READY, any update
	// from the pending load balancer should cause that update to be forwarded
	// to the ClientConn and also cause the pending load balancer to swap into
	// the current one.
	gsb.balancerPending.Balancer.(*mockBalancer).updateState(balancer.State{
		ConnectivityState: connectivity.Connecting,
		Picker:            &neverErrPicker{},
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for an UpdateState call on the ClientConn")
	case state := <-tcc.NewStateCh:
		if state != connectivity.Connecting {
			t.Fatalf("ClientConn received connectivity state %v, want %v (from pending)", state, connectivity.Connecting)
		}
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout waiting for an UpdateState call on the ClientConn")
	case picker := <-tcc.NewPickerCh:
		// Should receive a never err picker.
		if _, err := picker.Pick(balancer.PickInfo{}); err != nil {
			t.Fatalf("ClientConn should have received a never err picker from an UpdateState call")
		}
	}

	// The current balancer should be closed as a result of the swap.
	if err := currBal.waitForClose(ctx); err != nil {
		t.Fatal(err)
	}
}

// TestCurrentLeavingReady tests the scenario where there is a current and
// pending load balancer setup in the graceful switch load balancer, with the
// current load balancer being in the state READY, and the current load balancer
// then transitions into a state other than READY. This should cause the pending
// load balancer to swap into the current load balancer, and the ClientConn to
// be updated with the cached pending load balancing state. Also, once the
// current is cleared from the graceful switch load balancer, any updates sent
// should be intercepted and not forwarded to the ClientConn, as the balancer
// has already been cleared.
func (s) TestCurrentLeavingReady(t *testing.T) {
	tcc, gsb := setup(t)
	gsb.SwitchTo(mockBalancerBuilder1{})
	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	currBal.updateState(balancer.State{
		ConnectivityState: connectivity.Ready,
	})

	gsb.SwitchTo(mockBalancerBuilder2{})
	// Sends CONNECTING, shouldn't make it's way to ClientConn.
	gsb.balancerPending.Balancer.(*mockBalancer).updateState(balancer.State{
		ConnectivityState: connectivity.Connecting,
		Picker:            &neverErrPicker{},
	})

	// The current balancer leaving READY should cause the pending balancer to
	// swap to the current balancer. This swap from current to pending should
	// also update the ClientConn with the pending balancers cached state and
	// picker.
	currBal.updateState(balancer.State{
		ConnectivityState: connectivity.Idle,
	})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a UpdateState call on the ClientConn")
	case state := <-tcc.NewStateCh:
		if state != connectivity.Connecting {
			t.Fatalf("current balancer reports connectivity state %v, want %v", state, connectivity.Connecting)
		}
	}

	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a UpdateState call on the ClientConn")
	case picker := <-tcc.NewPickerCh:
		// Should receive a never err picker cached from pending LB's updateState() call, which
		// was cached.
		if _, err := picker.Pick(balancer.PickInfo{}); err != nil {
			t.Fatalf("ClientConn should have received a never err picker, the cached picker, from an UpdateState call")
		}
	}

	// The current balancer should be closed as a result of the swap.
	if err := currBal.waitForClose(ctx); err != nil {
		t.Fatal(err)
	}

	// The current balancer is now cleared from the graceful switch load
	// balancer. Thus, any update from the old current should be intercepted by
	// the graceful switch load balancer and not forward up to the ClientConn.
	currBal.updateState(balancer.State{
		ConnectivityState: connectivity.Ready,
		Picker:            &neverErrPicker{},
	})

	// This update should not be forwarded to the ClientConn.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-tcc.NewStateCh:
		t.Fatal("UpdateState() from a cleared balancer should not make it's way to ClientConn")
	}

	if _, err := currBal.newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{}); err == nil {
		t.Fatal("newSubConn() from a cleared balancer should have returned an error")
	}

	// This newSubConn call should also not reach the ClientConn.
	sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-sCtx.Done():
	case <-tcc.NewSubConnCh:
		t.Fatal("newSubConn() from a cleared balancer should not make it's way to ClientConn")
	}
}

// TestBalancerSubconns tests the SubConn functionality of the graceful switch
// load balancer. This tests the SubConn update flow in both directions, and
// make sure updates end up at the correct component.
func (s) TestBalancerSubconns(t *testing.T) {
	tcc, gsb := setup(t)
	gsb.SwitchTo(mockBalancerBuilder1{})
	gsb.SwitchTo(mockBalancerBuilder2{})

	// A child balancer creating a new SubConn should eventually be forwarded to
	// the ClientConn held by the graceful switch load balancer.
	sc1, err := gsb.balancerCurrent.Balancer.(*mockBalancer).newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	if err != nil {
		t.Fatalf("error constructing newSubConn in gsb: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an NewSubConn call on the ClientConn")
	case sc := <-tcc.NewSubConnCh:
		if sc != sc1 {
			t.Fatalf("NewSubConn, want %v, got %v", sc1, sc)
		}
	}

	// The other child balancer creating a new SubConn should also be eventually
	// be forwarded to the ClientConn held by the graceful switch load balancer.
	sc2, err := gsb.balancerPending.Balancer.(*mockBalancer).newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	if err != nil {
		t.Fatalf("error constructing newSubConn in gsb: %v", err)
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an NewSubConn call on the ClientConn")
	case sc := <-tcc.NewSubConnCh:
		if sc != sc2 {
			t.Fatalf("NewSubConn, want %v, got %v", sc2, sc)
		}
	}
	scState := balancer.SubConnState{ConnectivityState: connectivity.Ready}
	// Updating the SubConnState for sc1 should cause the graceful switch
	// balancer to forward the Update to balancerCurrent for sc1, as that is the
	// balancer that created this SubConn.
	sc1.(*testutils.TestSubConn).UpdateState(scState)

	// Updating the SubConnState for sc2 should cause the graceful switch
	// balancer to forward the Update to balancerPending for sc2, as that is the
	// balancer that created this SubConn.
	sc2.(*testutils.TestSubConn).UpdateState(scState)

	// Updating the addresses for both SubConns and removing both SubConns
	// should get forwarded to the ClientConn.

	// Updating the addresses for sc1 should get forwarded to the ClientConn.
	gsb.balancerCurrent.Balancer.(*mockBalancer).updateAddresses(sc1, []resolver.Address{})

	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses call on the ClientConn")
	case <-tcc.UpdateAddressesAddrsCh:
	}

	// Updating the addresses for sc2 should also get forwarded to the ClientConn.
	gsb.balancerPending.Balancer.(*mockBalancer).updateAddresses(sc2, []resolver.Address{})
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses call on the ClientConn")
	case <-tcc.UpdateAddressesAddrsCh:
	}

	// balancerCurrent removing sc1 should get forwarded to the ClientConn.
	sc1.Shutdown()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses call on the ClientConn")
	case sc := <-tcc.ShutdownSubConnCh:
		if sc != sc1 {
			t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, sc)
		}
	}
	// balancerPending removing sc2 should get forwarded to the ClientConn.
	sc2.Shutdown()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses call on the ClientConn")
	case sc := <-tcc.ShutdownSubConnCh:
		if sc != sc2 {
			t.Fatalf("ShutdownSubConn, want %v, got %v", sc2, sc)
		}
	}
}

// TestBalancerClose tests the graceful switch balancer's Close()
// functionality.  From the Close() call, the graceful switch balancer should
// shut down any created Subconns and Close() the current and pending load
// balancers. This Close() call should also cause any other events (calls to
// entrance functions) to be no-ops.
func (s) TestBalancerClose(t *testing.T) {
	// Setup gsb balancer with current, pending, and one created SubConn on both
	// current and pending.
	tcc, gsb := setup(t)
	gsb.SwitchTo(mockBalancerBuilder1{})
	gsb.SwitchTo(mockBalancerBuilder2{})

	sc1, err := gsb.balancerCurrent.Balancer.(*mockBalancer).newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	// Will eventually get back a SubConn with an identifying property id 1
	if err != nil {
		t.Fatalf("error constructing newSubConn in gsb: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an NewSubConn call on the ClientConn")
	case <-tcc.NewSubConnCh:
	}

	sc2, err := gsb.balancerPending.Balancer.(*mockBalancer).newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	// Will eventually get back a SubConn with an identifying property id 2
	if err != nil {
		t.Fatalf("error constructing newSubConn in gsb: %v", err)
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an NewSubConn call on the ClientConn")
	case <-tcc.NewSubConnCh:
	}

	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	pendBal := gsb.balancerPending.Balancer.(*mockBalancer)

	// Closing the graceful switch load balancer should lead to removing any
	// created SubConns, and closing both the current and pending load balancer.
	gsb.Close()

	// The order of SubConns the graceful switch load balancer tells the Client
	// Conn to shut down is non deterministic, as it is stored in a
	// map. However, the first SubConn shut down should be either sc1 or sc2.
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses call on the ClientConn")
	case sc := <-tcc.ShutdownSubConnCh:
		if sc != sc1 && sc != sc2 {
			t.Fatalf("ShutdownSubConn, want either %v or %v, got %v", sc1, sc2, sc)
		}
	}

	// The graceful switch load balancer should then tell the ClientConn to
	// shut down the other SubConn.
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses call on the ClientConn")
	case sc := <-tcc.ShutdownSubConnCh:
		if sc != sc1 && sc != sc2 {
			t.Fatalf("ShutdownSubConn, want either %v or %v, got %v", sc1, sc2, sc)
		}
	}

	// The current balancer should get closed as a result of the graceful switch balancer being closed.
	if err := currBal.waitForClose(ctx); err != nil {
		t.Fatal(err)
	}
	// The pending balancer should also get closed as a result of the graceful switch balancer being closed.
	if err := pendBal.waitForClose(ctx); err != nil {
		t.Fatal(err)
	}

	// Once the graceful switch load balancer has been closed, any entrance
	// function should be a no-op and return errBalancerClosed if the function
	// returns an error.

	// SwitchTo() should return an error due to the graceful switch load
	// balancer having been closed already.
	if err := gsb.SwitchTo(mockBalancerBuilder1{}); err != errBalancerClosed {
		t.Fatalf("gsb.SwitchTo(%v) returned error %v, want %v", mockBalancerBuilder1{}, err, errBalancerClosed)
	}

	// UpdateClientConnState() should return an error due to the graceful switch
	// load balancer having been closed already.
	ccs := balancer.ClientConnState{
		BalancerConfig: mockBalancerConfig{},
	}
	if err := gsb.UpdateClientConnState(ccs); err != errBalancerClosed {
		t.Fatalf("gsb.UpdateClientConnState(%v) returned error %v, want %v", ccs, err, errBalancerClosed)
	}

	// After the graceful switch load balancer has been closed, any resolver error
	// shouldn't forward to either balancer, as the resolver error is a no-op
	// and also even if not, the balancers should have been cleared from the
	// graceful switch load balancer.
	gsb.ResolverError(balancer.ErrBadResolverState)
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if err := currBal.waitForResolverError(sCtx, balancer.ErrBadResolverState); !strings.Contains(err.Error(), sCtx.Err().Error()) {
		t.Fatal("the current balancer should not have received the resolver error after close")
	}
	sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if err := pendBal.waitForResolverError(sCtx, balancer.ErrBadResolverState); !strings.Contains(err.Error(), sCtx.Err().Error()) {
		t.Fatal("the pending balancer should not have received the resolver error after close")
	}
}

// TestResolverError tests the functionality of a Resolver Error. If there is a
// current balancer, but no pending, the error should be forwarded to the
// current balancer. If there is both a current and pending balancer, the error
// should be forwarded to only the pending balancer.
func (s) TestResolverError(t *testing.T) {
	_, gsb := setup(t)
	gsb.SwitchTo(mockBalancerBuilder1{})
	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	// If there is only a current balancer present, the resolver error should be
	// forwarded to the current balancer.
	gsb.ResolverError(balancer.ErrBadResolverState)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := currBal.waitForResolverError(ctx, balancer.ErrBadResolverState); err != nil {
		t.Fatal(err)
	}

	gsb.SwitchTo(mockBalancerBuilder1{})

	// If there is a pending balancer present, then a resolver error should be
	// forwarded to only the pending balancer, not the current.
	pendBal := gsb.balancerPending.Balancer.(*mockBalancer)
	gsb.ResolverError(balancer.ErrBadResolverState)

	// The Resolver Error should not be forwarded to the current load balancer.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if err := currBal.waitForResolverError(sCtx, balancer.ErrBadResolverState); !strings.Contains(err.Error(), sCtx.Err().Error()) {
		t.Fatal("the current balancer should not have received the resolver error after close")
	}

	// The Resolver Error should be forwarded to the pending load balancer.
	if err := pendBal.waitForResolverError(ctx, balancer.ErrBadResolverState); err != nil {
		t.Fatal(err)
	}
}

// TestPendingReplacedByAnotherPending tests the scenario where a graceful
// switch balancer has a current and pending load balancer, and receives a
// SwitchTo() call, which then replaces the pending. This should cause the
// graceful switch balancer to clear pending state, close old pending SubConns,
// and Close() the pending balancer being replaced.
func (s) TestPendingReplacedByAnotherPending(t *testing.T) {
	tcc, gsb := setup(t)
	gsb.SwitchTo(mockBalancerBuilder1{})
	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	currBal.updateState(balancer.State{
		ConnectivityState: connectivity.Ready,
	})

	// Populate pending with a SwitchTo() call.
	gsb.SwitchTo(mockBalancerBuilder2{})

	pendBal := gsb.balancerPending.Balancer.(*mockBalancer)
	sc1, err := pendBal.newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	if err != nil {
		t.Fatalf("error constructing newSubConn in gsb: %v", err)
	}
	// This picker never returns an error, which can help this test verify
	// whether this cached state will get cleared on a new pending balancer
	// (will replace it with a picker that always errors).
	pendBal.updateState(balancer.State{
		ConnectivityState: connectivity.Connecting,
		Picker:            &neverErrPicker{},
	})

	// Replace pending with a SwitchTo() call.
	gsb.SwitchTo(mockBalancerBuilder2{})
	// The pending balancer being replaced should cause the graceful switch
	// balancer to Shutdown() any created SubConns for the old pending balancer
	// and also Close() the old pending balancer.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a SubConn.Shutdown")
	case sc := <-tcc.ShutdownSubConnCh:
		if sc != sc1 {
			t.Fatalf("ShutdownSubConn, want %v, got %v", sc1, sc)
		}
	}

	if err := pendBal.waitForClose(ctx); err != nil {
		t.Fatal(err)
	}

	// Switching the current out of READY should cause the pending LB to swap
	// into current, causing the graceful switch balancer to update the
	// ClientConn with the cached pending state. Since the new pending hasn't
	// sent an Update, the default state with connectivity state CONNECTING and
	// an errPicker should be sent to the ClientConn.
	currBal.updateState(balancer.State{
		ConnectivityState: connectivity.Idle,
	})

	// The update should contain a default connectivity state CONNECTING for the
	// state of the new pending LB policy.
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateState() call on the ClientConn")
	case state := <-tcc.NewStateCh:
		if state != connectivity.Connecting {
			t.Fatalf("UpdateState(), want connectivity state %v, got %v", connectivity.Connecting, state)
		}
	}
	// The update should contain a default picker ErrPicker in the picker sent
	// for the state of the new pending LB policy.
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateState() call on the ClientConn")
	case picker := <-tcc.NewPickerCh:
		if _, err := picker.Pick(balancer.PickInfo{}); err != balancer.ErrNoSubConnAvailable {
			t.Fatalf("ClientConn should have received a never err picker from an UpdateState call")
		}
	}
}

// Picker which never errors here for test purposes (can fill up tests further up with this)
type neverErrPicker struct{}

func (p *neverErrPicker) Pick(balancer.PickInfo) (balancer.PickResult, error) {
	return balancer.PickResult{}, nil
}

// TestUpdateSubConnStateRace tests the race condition when the graceful switch
// load balancer receives a SubConnUpdate concurrently with an UpdateState()
// call, which can cause the balancer to forward the update to be closed and
// cleared. The balancer API guarantees to never call any method the balancer
// after a Close() call, and the test verifies that doesn't happen within the
// graceful switch load balancer.
func (s) TestUpdateSubConnStateRace(t *testing.T) {
	tcc, gsb := setup(t)
	gsb.SwitchTo(verifyBalancerBuilder{})
	gsb.SwitchTo(mockBalancerBuilder1{})
	currBal := gsb.balancerCurrent.Balancer.(*verifyBalancer)
	currBal.t = t
	pendBal := gsb.balancerPending.Balancer.(*mockBalancer)
	sc, err := currBal.newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	if err != nil {
		t.Fatalf("error constructing newSubConn in gsb: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an NewSubConn call on the ClientConn")
	case <-tcc.NewSubConnCh:
	}
	// Spawn a goroutine that constantly calls UpdateSubConn for the current
	// balancer, which will get deleted in this testing goroutine.
	finished := make(chan struct{})
	go func() {
		for {
			select {
			case <-finished:
				return
			default:
			}
			sc.(*testutils.TestSubConn).UpdateState(balancer.SubConnState{
				ConnectivityState: connectivity.Ready,
			})
		}
	}()
	time.Sleep(time.Millisecond)
	// This UpdateState call causes current to be closed/cleared.
	pendBal.updateState(balancer.State{
		ConnectivityState: connectivity.Ready,
	})
	// From this, either one of two things happen. Either the graceful switch
	// load balancer doesn't Close() the current balancer before it forwards the
	// SubConn update to the child, and the call gets forwarded down to the
	// current balancer, or it can Close() the current balancer in between
	// reading the balancer pointer and writing to it, and in that case the old
	// current balancer should not be updated, as the balancer has already been
	// closed and the balancer API guarantees it.
	close(finished)
}

// TestInlineCallbackInBuild tests the scenario where a balancer calls back into
// the balancer.ClientConn API inline from it's build function.
func (s) TestInlineCallbackInBuild(t *testing.T) {
	tcc, gsb := setup(t)
	// This build call should cause all of the inline updates to forward to the
	// ClientConn.
	gsb.SwitchTo(buildCallbackBalancerBuilder{})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateState() call on the ClientConn")
	case <-tcc.NewStateCh:
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a NewSubConn() call on the ClientConn")
	case <-tcc.NewSubConnCh:
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses() call on the ClientConn")
	case <-tcc.UpdateAddressesAddrsCh:
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a Shutdown() call on the SubConn")
	case <-tcc.ShutdownSubConnCh:
	}
	oldCurrent := gsb.balancerCurrent.Balancer.(*buildCallbackBal)

	// Since the callback reports a state READY, this new inline balancer should
	// be swapped to the current.
	gsb.SwitchTo(buildCallbackBalancerBuilder{})
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateState() call on the ClientConn")
	case <-tcc.NewStateCh:
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a NewSubConn() call on the ClientConn")
	case <-tcc.NewSubConnCh:
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for an UpdateAddresses() call on the ClientConn")
	case <-tcc.UpdateAddressesAddrsCh:
	}
	select {
	case <-ctx.Done():
		t.Fatalf("timeout while waiting for a Shutdown() call on the SubConn")
	case <-tcc.ShutdownSubConnCh:
	}

	// The current balancer should be closed as a result of the swap.
	if err := oldCurrent.waitForClose(ctx); err != nil {
		t.Fatalf("error waiting for balancer close: %v", err)
	}

	// The old balancer should be deprecated and any calls from it should be a no-op.
	oldCurrent.newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-tcc.NewSubConnCh:
		t.Fatal("Deprecated LB calling NewSubConn() should not forward up to the ClientConn")
	case <-sCtx.Done():
	}
}

// TestExitIdle tests the ExitIdle operation on the Graceful Switch Balancer for
// both possible codepaths, one where the child implements ExitIdler interface
// and one where the child doesn't implement ExitIdler interface.
func (s) TestExitIdle(t *testing.T) {
	_, gsb := setup(t)
	// switch to a balancer that implements ExitIdle{} (will populate current).
	gsb.SwitchTo(mockBalancerBuilder1{})
	currBal := gsb.balancerCurrent.Balancer.(*mockBalancer)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// exitIdle on the Graceful Switch Balancer should get forwarded to the
	// current child as it implements exitIdle.
	gsb.ExitIdle()
	if err := currBal.waitForExitIdle(ctx); err != nil {
		t.Fatal(err)
	}

	// switch to a balancer that doesn't implement ExitIdle{} (will populate
	// pending).
	gsb.SwitchTo(verifyBalancerBuilder{})
	// call exitIdle concurrently with newSubConn to make sure there is not a
	// data race.
	done := make(chan struct{})
	go func() {
		gsb.ExitIdle()
		close(done)
	}()
	pendBal := gsb.balancerPending.Balancer.(*verifyBalancer)
	for i := 0; i < 10; i++ {
		pendBal.newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	}
	<-done
}

const balancerName1 = "mock_balancer_1"
const balancerName2 = "mock_balancer_2"
const verifyBalName = "verifyNoSubConnUpdateAfterCloseBalancer"
const buildCallbackBalName = "callbackInBuildBalancer"

type mockBalancerBuilder1 struct{}

func (mockBalancerBuilder1) Build(cc balancer.ClientConn, _ balancer.BuildOptions) balancer.Balancer {
	return &mockBalancer{
		ccsCh:         testutils.NewChannel(),
		scStateCh:     testutils.NewChannel(),
		resolverErrCh: testutils.NewChannel(),
		closeCh:       testutils.NewChannel(),
		exitIdleCh:    testutils.NewChannel(),
		cc:            cc,
	}
}

func (mockBalancerBuilder1) Name() string {
	return balancerName1
}

type mockBalancerConfig struct {
	serviceconfig.LoadBalancingConfig
}

// mockBalancer is a fake balancer used to verify different actions from
// the gracefulswitch. It contains a bunch of channels to signal different events
// to the test.
type mockBalancer struct {
	// ccsCh is a channel used to signal the receipt of a ClientConn update.
	ccsCh *testutils.Channel
	// scStateCh is a channel used to signal the receipt of a SubConn update.
	scStateCh *testutils.Channel
	// resolverErrCh is a channel used to signal a resolver error.
	resolverErrCh *testutils.Channel
	// closeCh is a channel used to signal the closing of this balancer.
	closeCh *testutils.Channel
	// exitIdleCh is a channel used to signal the receipt of an ExitIdle call.
	exitIdleCh *testutils.Channel
	// Hold onto ClientConn wrapper to communicate with it
	cc balancer.ClientConn
}

type subConnWithState struct {
	sc    balancer.SubConn
	state balancer.SubConnState
}

func (mb1 *mockBalancer) UpdateClientConnState(ccs balancer.ClientConnState) error {
	// Need to verify this call...use a channel?...all of these will need verification
	mb1.ccsCh.Send(ccs)
	return nil
}

func (mb1 *mockBalancer) ResolverError(err error) {
	mb1.resolverErrCh.Send(err)
}

func (mb1 *mockBalancer) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	panic(fmt.Sprintf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state))
}

func (mb1 *mockBalancer) Close() {
	mb1.closeCh.Send(struct{}{})
}

func (mb1 *mockBalancer) ExitIdle() {
	mb1.exitIdleCh.Send(struct{}{})
}

// waitForClientConnUpdate verifies if the mockBalancer receives the
// provided ClientConnState within a reasonable amount of time.
func (mb1 *mockBalancer) waitForClientConnUpdate(ctx context.Context, wantCCS balancer.ClientConnState) error {
	ccs, err := mb1.ccsCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("error waiting for ClientConnUpdate: %v", err)
	}
	gotCCS := ccs.(balancer.ClientConnState)
	if diff := cmp.Diff(gotCCS, wantCCS, cmpopts.IgnoreFields(resolver.State{}, "Attributes")); diff != "" {
		return fmt.Errorf("error in ClientConnUpdate: received unexpected ClientConnState, diff (-got +want): %v", diff)
	}
	return nil
}

// waitForResolverError verifies if the mockBalancer receives the provided
// resolver error before the context expires.
func (mb1 *mockBalancer) waitForResolverError(ctx context.Context, wantErr error) error {
	gotErr, err := mb1.resolverErrCh.Receive(ctx)
	if err != nil {
		return fmt.Errorf("error waiting for resolver error: %v", err)
	}
	if gotErr != wantErr {
		return fmt.Errorf("received resolver error: %v, want %v", gotErr, wantErr)
	}
	return nil
}

// waitForClose verifies that the mockBalancer is closed before the context
// expires.
func (mb1 *mockBalancer) waitForClose(ctx context.Context) error {
	if _, err := mb1.closeCh.Receive(ctx); err != nil {
		return fmt.Errorf("error waiting for Close(): %v", err)
	}
	return nil
}

// waitForExitIdle verifies that ExitIdle gets called on the mockBalancer before
// the context expires.
func (mb1 *mockBalancer) waitForExitIdle(ctx context.Context) error {
	if _, err := mb1.exitIdleCh.Receive(ctx); err != nil {
		return fmt.Errorf("error waiting for ExitIdle(): %v", err)
	}
	return nil
}

func (mb1 *mockBalancer) updateState(state balancer.State) {
	mb1.cc.UpdateState(state)
}

func (mb1 *mockBalancer) newSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (sc balancer.SubConn, err error) {
	if opts.StateListener == nil {
		opts.StateListener = func(state balancer.SubConnState) {
			mb1.scStateCh.Send(subConnWithState{sc: sc, state: state})
		}
	}
	defer func() {
		if sc != nil {
			sc.Connect()
		}
	}()
	return mb1.cc.NewSubConn(addrs, opts)
}

func (mb1 *mockBalancer) updateAddresses(sc balancer.SubConn, addrs []resolver.Address) {
	mb1.cc.UpdateAddresses(sc, addrs)
}

type mockBalancerBuilder2 struct{}

func (mockBalancerBuilder2) Build(cc balancer.ClientConn, _ balancer.BuildOptions) balancer.Balancer {
	return &mockBalancer{
		ccsCh:         testutils.NewChannel(),
		scStateCh:     testutils.NewChannel(),
		resolverErrCh: testutils.NewChannel(),
		closeCh:       testutils.NewChannel(),
		cc:            cc,
	}
}

func (mockBalancerBuilder2) Name() string {
	return balancerName2
}

type verifyBalancerBuilder struct{}

func (verifyBalancerBuilder) Build(cc balancer.ClientConn, _ balancer.BuildOptions) balancer.Balancer {
	return &verifyBalancer{
		closed: grpcsync.NewEvent(),
		cc:     cc,
	}
}

func (verifyBalancerBuilder) Name() string {
	return verifyBalName
}

// verifyBalancer is a balancer that verifies that after a Close() call, a
// StateListener() call never happens.
type verifyBalancer struct {
	closed *grpcsync.Event
	// Hold onto the ClientConn wrapper to communicate with it.
	cc balancer.ClientConn
	// To fail the test if StateListener gets called after Close().
	t *testing.T
}

func (vb *verifyBalancer) UpdateClientConnState(balancer.ClientConnState) error {
	return nil
}

func (vb *verifyBalancer) ResolverError(error) {}

func (vb *verifyBalancer) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	panic(fmt.Sprintf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state))
}

func (vb *verifyBalancer) Close() {
	vb.closed.Fire()
}

func (vb *verifyBalancer) newSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (sc balancer.SubConn, err error) {
	if opts.StateListener == nil {
		opts.StateListener = func(state balancer.SubConnState) {
			if vb.closed.HasFired() {
				vb.t.Fatalf("StateListener(%+v) was called after Close(), which breaks the balancer API", state)
			}
		}
	}
	defer func() { sc.Connect() }()
	return vb.cc.NewSubConn(addrs, opts)
}

type buildCallbackBalancerBuilder struct{}

func (buildCallbackBalancerBuilder) Build(cc balancer.ClientConn, _ balancer.BuildOptions) balancer.Balancer {
	b := &buildCallbackBal{
		cc:      cc,
		closeCh: testutils.NewChannel(),
	}
	b.updateState(balancer.State{
		ConnectivityState: connectivity.Connecting,
	})
	sc, err := b.newSubConn([]resolver.Address{}, balancer.NewSubConnOptions{})
	if err != nil {
		return nil
	}
	b.updateAddresses(sc, []resolver.Address{})
	sc.Shutdown()
	return b
}

func (buildCallbackBalancerBuilder) Name() string {
	return buildCallbackBalName
}

type buildCallbackBal struct {
	// Hold onto the ClientConn wrapper to communicate with it.
	cc balancer.ClientConn
	// closeCh is a channel used to signal the closing of this balancer.
	closeCh *testutils.Channel
}

func (bcb *buildCallbackBal) UpdateClientConnState(balancer.ClientConnState) error {
	return nil
}

func (bcb *buildCallbackBal) ResolverError(error) {}

func (bcb *buildCallbackBal) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	panic(fmt.Sprintf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state))
}

func (bcb *buildCallbackBal) Close() {
	bcb.closeCh.Send(struct{}{})
}

func (bcb *buildCallbackBal) updateState(state balancer.State) {
	bcb.cc.UpdateState(state)
}

func (bcb *buildCallbackBal) newSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (sc balancer.SubConn, err error) {
	defer func() {
		if sc != nil {
			sc.Connect()
		}
	}()
	return bcb.cc.NewSubConn(addrs, opts)
}

func (bcb *buildCallbackBal) updateAddresses(sc balancer.SubConn, addrs []resolver.Address) {
	bcb.cc.UpdateAddresses(sc, addrs)
}

// waitForClose verifies that the mockBalancer is closed before the context
// expires.
func (bcb *buildCallbackBal) waitForClose(ctx context.Context) error {
	if _, err := bcb.closeCh.Receive(ctx); err != nil {
		return err
	}
	return nil
}
