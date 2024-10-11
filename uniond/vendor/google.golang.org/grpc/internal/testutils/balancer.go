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

package testutils

import (
	"context"
	"errors"
	"fmt"
	"testing"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/resolver"
)

// TestSubConn implements the SubConn interface, to be used in tests.
type TestSubConn struct {
	tcc           *BalancerClientConn // the CC that owns this SubConn
	id            string
	ConnectCh     chan struct{}
	stateListener func(balancer.SubConnState)
	connectCalled *grpcsync.Event
}

// NewTestSubConn returns a newly initialized SubConn.  Typically, subconns
// should be created via TestClientConn.NewSubConn instead, but can be useful
// for some tests.
func NewTestSubConn(id string) *TestSubConn {
	return &TestSubConn{
		ConnectCh:     make(chan struct{}, 1),
		connectCalled: grpcsync.NewEvent(),
		id:            id,
	}
}

// UpdateAddresses is a no-op.
func (tsc *TestSubConn) UpdateAddresses([]resolver.Address) {}

// Connect is a no-op.
func (tsc *TestSubConn) Connect() {
	tsc.connectCalled.Fire()
	select {
	case tsc.ConnectCh <- struct{}{}:
	default:
	}
}

// GetOrBuildProducer is a no-op.
func (tsc *TestSubConn) GetOrBuildProducer(balancer.ProducerBuilder) (balancer.Producer, func()) {
	return nil, nil
}

// UpdateState pushes the state to the listener, if one is registered.
func (tsc *TestSubConn) UpdateState(state balancer.SubConnState) {
	<-tsc.connectCalled.Done()
	if tsc.stateListener != nil {
		tsc.stateListener(state)
		return
	}
}

// Shutdown pushes the SubConn to the ShutdownSubConn channel in the parent
// TestClientConn.
func (tsc *TestSubConn) Shutdown() {
	tsc.tcc.logger.Logf("SubConn %s: Shutdown", tsc)
	select {
	case tsc.tcc.ShutdownSubConnCh <- tsc:
	default:
	}
}

// String implements stringer to print human friendly error message.
func (tsc *TestSubConn) String() string {
	return tsc.id
}

// BalancerClientConn is a mock balancer.ClientConn used in tests.
type BalancerClientConn struct {
	logger Logger

	NewSubConnAddrsCh      chan []resolver.Address // the last 10 []Address to create subconn.
	NewSubConnCh           chan *TestSubConn       // the last 10 subconn created.
	ShutdownSubConnCh      chan *TestSubConn       // the last 10 subconn removed.
	UpdateAddressesAddrsCh chan []resolver.Address // last updated address via UpdateAddresses().

	NewPickerCh  chan balancer.Picker            // the last picker updated.
	NewStateCh   chan connectivity.State         // the last state.
	ResolveNowCh chan resolver.ResolveNowOptions // the last ResolveNow().

	subConnIdx int
}

// NewBalancerClientConn creates a BalancerClientConn.
func NewBalancerClientConn(t *testing.T) *BalancerClientConn {
	return &BalancerClientConn{
		logger: t,

		NewSubConnAddrsCh:      make(chan []resolver.Address, 10),
		NewSubConnCh:           make(chan *TestSubConn, 10),
		ShutdownSubConnCh:      make(chan *TestSubConn, 10),
		UpdateAddressesAddrsCh: make(chan []resolver.Address, 1),

		NewPickerCh:  make(chan balancer.Picker, 1),
		NewStateCh:   make(chan connectivity.State, 1),
		ResolveNowCh: make(chan resolver.ResolveNowOptions, 1),
	}
}

// NewSubConn creates a new SubConn.
func (tcc *BalancerClientConn) NewSubConn(a []resolver.Address, o balancer.NewSubConnOptions) (balancer.SubConn, error) {
	sc := &TestSubConn{
		tcc:           tcc,
		id:            fmt.Sprintf("sc%d", tcc.subConnIdx),
		ConnectCh:     make(chan struct{}, 1),
		stateListener: o.StateListener,
		connectCalled: grpcsync.NewEvent(),
	}
	tcc.subConnIdx++
	tcc.logger.Logf("testClientConn: NewSubConn(%v, %+v) => %s", a, o, sc)
	select {
	case tcc.NewSubConnAddrsCh <- a:
	default:
	}

	select {
	case tcc.NewSubConnCh <- sc:
	default:
	}

	return sc, nil
}

// RemoveSubConn is a nop; tests should all be updated to use sc.Shutdown()
// instead.
func (tcc *BalancerClientConn) RemoveSubConn(sc balancer.SubConn) {
	tcc.logger.Errorf("RemoveSubConn(%v) called unexpectedly", sc)
}

// UpdateAddresses updates the addresses on the SubConn.
func (tcc *BalancerClientConn) UpdateAddresses(sc balancer.SubConn, addrs []resolver.Address) {
	tcc.logger.Logf("testutils.BalancerClientConn: UpdateAddresses(%v, %+v)", sc, addrs)
	select {
	case tcc.UpdateAddressesAddrsCh <- addrs:
	default:
	}
}

// UpdateState updates connectivity state and picker.
func (tcc *BalancerClientConn) UpdateState(bs balancer.State) {
	tcc.logger.Logf("testutils.BalancerClientConn: UpdateState(%v)", bs)
	select {
	case <-tcc.NewStateCh:
	default:
	}
	tcc.NewStateCh <- bs.ConnectivityState

	select {
	case <-tcc.NewPickerCh:
	default:
	}
	tcc.NewPickerCh <- bs.Picker
}

// ResolveNow panics.
func (tcc *BalancerClientConn) ResolveNow(o resolver.ResolveNowOptions) {
	select {
	case <-tcc.ResolveNowCh:
	default:
	}
	tcc.ResolveNowCh <- o
}

// Target panics.
func (tcc *BalancerClientConn) Target() string {
	panic("not implemented")
}

// WaitForErrPicker waits until an error picker is pushed to this ClientConn.
// Returns error if the provided context expires or a non-error picker is pushed
// to the ClientConn.
func (tcc *BalancerClientConn) WaitForErrPicker(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return errors.New("timeout when waiting for an error picker")
	case picker := <-tcc.NewPickerCh:
		if _, perr := picker.Pick(balancer.PickInfo{}); perr == nil {
			return fmt.Errorf("balancer returned a picker which is not an error picker")
		}
	}
	return nil
}

// WaitForPickerWithErr waits until an error picker is pushed to this
// ClientConn with the error matching the wanted error.  Returns an error if
// the provided context expires, including the last received picker error (if
// any).
func (tcc *BalancerClientConn) WaitForPickerWithErr(ctx context.Context, want error) error {
	lastErr := errors.New("received no picker")
	for {
		select {
		case <-ctx.Done():
			return fmt.Errorf("timeout when waiting for an error picker with %v; last picker error: %v", want, lastErr)
		case picker := <-tcc.NewPickerCh:
			if _, lastErr = picker.Pick(balancer.PickInfo{}); lastErr != nil && lastErr.Error() == want.Error() {
				return nil
			}
		}
	}
}

// WaitForConnectivityState waits until the state pushed to this ClientConn
// matches the wanted state.  Returns an error if the provided context expires,
// including the last received state (if any).
func (tcc *BalancerClientConn) WaitForConnectivityState(ctx context.Context, want connectivity.State) error {
	var lastState connectivity.State = -1
	for {
		select {
		case <-ctx.Done():
			return fmt.Errorf("timeout when waiting for state to be %s; last state: %s", want, lastState)
		case s := <-tcc.NewStateCh:
			if s == want {
				return nil
			}
			lastState = s
		}
	}
}

// WaitForRoundRobinPicker waits for a picker that passes IsRoundRobin.  Also
// drains the matching state channel and requires it to be READY (if an entry
// is pending) to be considered.  Returns an error if the provided context
// expires, including the last received error from IsRoundRobin or the picker
// (if any).
func (tcc *BalancerClientConn) WaitForRoundRobinPicker(ctx context.Context, want ...balancer.SubConn) error {
	lastErr := errors.New("received no picker")
	for {
		select {
		case <-ctx.Done():
			return fmt.Errorf("timeout when waiting for round robin picker with %v; last error: %v", want, lastErr)
		case p := <-tcc.NewPickerCh:
			s := connectivity.Ready
			select {
			case s = <-tcc.NewStateCh:
			default:
			}
			if s != connectivity.Ready {
				lastErr = fmt.Errorf("received state %v instead of ready", s)
				break
			}
			var pickerErr error
			if err := IsRoundRobin(want, func() balancer.SubConn {
				sc, err := p.Pick(balancer.PickInfo{})
				if err != nil {
					pickerErr = err
				} else if sc.Done != nil {
					sc.Done(balancer.DoneInfo{})
				}
				return sc.SubConn
			}); pickerErr != nil {
				lastErr = pickerErr
				continue
			} else if err != nil {
				lastErr = err
				continue
			}
			return nil
		}
	}
}

// WaitForPicker waits for a picker that results in f returning nil.  If the
// context expires, returns the last error returned by f (if any).
func (tcc *BalancerClientConn) WaitForPicker(ctx context.Context, f func(balancer.Picker) error) error {
	lastErr := errors.New("received no picker")
	for {
		select {
		case <-ctx.Done():
			return fmt.Errorf("timeout when waiting for picker; last error: %v", lastErr)
		case p := <-tcc.NewPickerCh:
			if err := f(p); err != nil {
				lastErr = err
				continue
			}
			return nil
		}
	}
}

// IsRoundRobin checks whether f's return value is roundrobin of elements from
// want. But it doesn't check for the order. Note that want can contain
// duplicate items, which makes it weight-round-robin.
//
// Step 1. the return values of f should form a permutation of all elements in
// want, but not necessary in the same order. E.g. if want is {a,a,b}, the check
// fails if f returns:
//   - {a,a,a}: third a is returned before b
//   - {a,b,b}: second b is returned before the second a
//
// If error is found in this step, the returned error contains only the first
// iteration until where it goes wrong.
//
// Step 2. the return values of f should be repetitions of the same permutation.
// E.g. if want is {a,a,b}, the check fails if f returns:
//   - {a,b,a,b,a,a}: though it satisfies step 1, the second iteration is not
//     repeating the first iteration.
//
// If error is found in this step, the returned error contains the first
// iteration + the second iteration until where it goes wrong.
func IsRoundRobin(want []balancer.SubConn, f func() balancer.SubConn) error {
	wantSet := make(map[balancer.SubConn]int) // SubConn -> count, for weighted RR.
	for _, sc := range want {
		wantSet[sc]++
	}

	// The first iteration: makes sure f's return values form a permutation of
	// elements in want.
	//
	// Also keep the returns values in a slice, so we can compare the order in
	// the second iteration.
	gotSliceFirstIteration := make([]balancer.SubConn, 0, len(want))
	for range want {
		got := f()
		gotSliceFirstIteration = append(gotSliceFirstIteration, got)
		wantSet[got]--
		if wantSet[got] < 0 {
			return fmt.Errorf("non-roundrobin want: %v, result: %v", want, gotSliceFirstIteration)
		}
	}

	// The second iteration should repeat the first iteration.
	var gotSliceSecondIteration []balancer.SubConn
	for i := 0; i < 2; i++ {
		for _, w := range gotSliceFirstIteration {
			g := f()
			gotSliceSecondIteration = append(gotSliceSecondIteration, g)
			if w != g {
				return fmt.Errorf("non-roundrobin, first iter: %v, second iter: %v", gotSliceFirstIteration, gotSliceSecondIteration)
			}
		}
	}

	return nil
}

// SubConnFromPicker returns a function which returns a SubConn by calling the
// Pick() method of the provided picker. There is no caching of SubConns here.
// Every invocation of the returned function results in a new pick.
func SubConnFromPicker(p balancer.Picker) func() balancer.SubConn {
	return func() balancer.SubConn {
		scst, _ := p.Pick(balancer.PickInfo{})
		return scst.SubConn
	}
}

// ErrTestConstPicker is error returned by test const picker.
var ErrTestConstPicker = fmt.Errorf("const picker error")

// TestConstPicker is a const picker for tests.
type TestConstPicker struct {
	Err error
	SC  balancer.SubConn
}

// Pick returns the const SubConn or the error.
func (tcp *TestConstPicker) Pick(balancer.PickInfo) (balancer.PickResult, error) {
	if tcp.Err != nil {
		return balancer.PickResult{}, tcp.Err
	}
	return balancer.PickResult{SubConn: tcp.SC}, nil
}
