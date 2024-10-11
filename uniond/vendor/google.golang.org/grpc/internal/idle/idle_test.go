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

package idle

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"google.golang.org/grpc/internal/grpctest"
)

const (
	defaultTestTimeout      = 10 * time.Second
	defaultTestIdleTimeout  = 500 * time.Millisecond // A short idle_timeout for tests.
	defaultTestShortTimeout = 10 * time.Millisecond  // A small deadline to wait for events expected to not happen.
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type testEnforcer struct {
	exitIdleCh  chan struct{}
	enterIdleCh chan struct{}
}

func (ti *testEnforcer) ExitIdleMode() error {
	ti.exitIdleCh <- struct{}{}
	return nil

}

func (ti *testEnforcer) EnterIdleMode() {
	ti.enterIdleCh <- struct{}{}
}

func newTestEnforcer() *testEnforcer {
	return &testEnforcer{
		exitIdleCh:  make(chan struct{}, 1),
		enterIdleCh: make(chan struct{}, 1),
	}
}

// overrideNewTimer overrides the new timer creation function by ensuring that a
// message is pushed on the returned channel everytime the timer fires.
func overrideNewTimer(t *testing.T) <-chan struct{} {
	t.Helper()

	ch := make(chan struct{}, 1)
	origTimeAfterFunc := timeAfterFunc
	timeAfterFunc = func(d time.Duration, callback func()) *time.Timer {
		return time.AfterFunc(d, func() {
			select {
			case ch <- struct{}{}:
			default:
			}
			callback()
		})
	}
	t.Cleanup(func() { timeAfterFunc = origTimeAfterFunc })
	return ch
}

// TestManager_Disabled tests the case where the idleness manager is
// disabled by passing an idle_timeout of 0. Verifies the following things:
//   - timer callback does not fire
//   - an RPC triggers a call to ExitIdleMode on the ClientConn
//   - more calls to RPC termination (as compared to RPC initiation) does not
//     result in an error log
func (s) TestManager_Disabled(t *testing.T) {
	callbackCh := overrideNewTimer(t)

	// Create an idleness manager that is disabled because of idleTimeout being
	// set to `0`.
	enforcer := newTestEnforcer()
	mgr := NewManager(enforcer, time.Duration(0))

	// Ensure that the timer callback does not fire within a short deadline.
	select {
	case <-callbackCh:
		t.Fatal("Idle timer callback fired when manager is disabled")
	case <-time.After(defaultTestShortTimeout):
	}

	// The first invocation of OnCallBegin() should lead to a call to
	// ExitIdleMode() on the enforcer.
	go mgr.OnCallBegin()
	select {
	case <-enforcer.exitIdleCh:
	case <-time.After(defaultTestShortTimeout):
		t.Fatal("Timeout waiting for channel to move out of idle mode")
	}

	// If the number of calls to OnCallEnd() exceeds the number of calls to
	// OnCallBegin(), the idleness manager is expected to throw an error log
	// (which will cause our TestLogger to fail the test). But since the manager
	// is disabled, this should not happen.
	mgr.OnCallEnd()
	mgr.OnCallEnd()

	// The idleness manager is explicitly not closed here. But since the manager
	// is disabled, it will not start the run goroutine, and hence we expect the
	// leak checker to not find any leaked goroutines.
}

// TestManager_Enabled_TimerFires tests the case where the idle manager
// is enabled. Ensures that when there are no RPCs, the timer callback is
// invoked and the EnterIdleMode() method is invoked on the enforcer.
func (s) TestManager_Enabled_TimerFires(t *testing.T) {
	callbackCh := overrideNewTimer(t)

	enforcer := newTestEnforcer()
	mgr := NewManager(enforcer, time.Duration(defaultTestIdleTimeout))
	defer mgr.Close()
	mgr.ExitIdleMode()

	// Ensure that the timer callback fires within a appropriate amount of time.
	select {
	case <-callbackCh:
	case <-time.After(2 * defaultTestIdleTimeout):
		t.Fatal("Timeout waiting for idle timer callback to fire")
	}

	// Ensure that the channel moves to idle mode eventually.
	select {
	case <-enforcer.enterIdleCh:
	case <-time.After(defaultTestTimeout):
		t.Fatal("Timeout waiting for channel to move to idle")
	}
}

// TestManager_Enabled_OngoingCall tests the case where the idle manager
// is enabled. Ensures that when there is an ongoing RPC, the channel does not
// enter idle mode.
func (s) TestManager_Enabled_OngoingCall(t *testing.T) {
	callbackCh := overrideNewTimer(t)

	enforcer := newTestEnforcer()
	mgr := NewManager(enforcer, time.Duration(defaultTestIdleTimeout))
	defer mgr.Close()
	mgr.ExitIdleMode()

	// Fire up a goroutine that simulates an ongoing RPC that is terminated
	// after the timer callback fires for the first time.
	timerFired := make(chan struct{})
	go func() {
		mgr.OnCallBegin()
		<-timerFired
		mgr.OnCallEnd()
	}()

	// Ensure that the timer callback fires and unblock the above goroutine.
	select {
	case <-callbackCh:
		close(timerFired)
	case <-time.After(2 * defaultTestIdleTimeout):
		t.Fatal("Timeout waiting for idle timer callback to fire")
	}

	// The invocation of the timer callback should not put the channel in idle
	// mode since we had an ongoing RPC.
	select {
	case <-enforcer.enterIdleCh:
		t.Fatalf("EnterIdleMode() called on enforcer when active RPC exists")
	case <-time.After(defaultTestShortTimeout):
	}

	// Since we terminated the ongoing RPC and we have no other active RPCs, the
	// channel must move to idle eventually.
	select {
	case <-enforcer.enterIdleCh:
	case <-time.After(defaultTestTimeout):
		t.Fatal("Timeout waiting for channel to move to idle")
	}
}

// TestManager_Enabled_ActiveSinceLastCheck tests the case where the
// idle manager is enabled. Ensures that when there are active RPCs in the last
// period (even though there is no active call when the timer fires), the
// channel does not enter idle mode.
func (s) TestManager_Enabled_ActiveSinceLastCheck(t *testing.T) {
	callbackCh := overrideNewTimer(t)

	enforcer := newTestEnforcer()
	mgr := NewManager(enforcer, time.Duration(defaultTestIdleTimeout))
	defer mgr.Close()
	mgr.ExitIdleMode()

	// Fire up a goroutine that simulates unary RPCs until the timer callback
	// fires.
	timerFired := make(chan struct{})
	go func() {
		for ; ; <-time.After(defaultTestShortTimeout) {
			mgr.OnCallBegin()
			mgr.OnCallEnd()

			select {
			case <-timerFired:
				return
			default:
			}
		}
	}()

	// Ensure that the timer callback fires, and that we don't enter idle as
	// part of this invocation of the timer callback, since we had some RPCs in
	// this period.
	select {
	case <-callbackCh:
		close(timerFired)
	case <-time.After(2 * defaultTestIdleTimeout):
		close(timerFired)
		t.Fatal("Timeout waiting for idle timer callback to fire")
	}
	select {
	case <-enforcer.enterIdleCh:
		t.Fatalf("EnterIdleMode() called on enforcer when one RPC completed in the last period")
	case <-time.After(defaultTestShortTimeout):
	}

	// Since the unary RPC terminated and we have no other active RPCs, the
	// channel must move to idle eventually.
	select {
	case <-enforcer.enterIdleCh:
	case <-time.After(defaultTestTimeout):
		t.Fatal("Timeout waiting for channel to move to idle")
	}
}

// TestManager_Enabled_ExitIdleOnRPC tests the case where the idle
// manager is enabled. Ensures that the channel moves out of idle when an RPC is
// initiated.
func (s) TestManager_Enabled_ExitIdleOnRPC(t *testing.T) {
	overrideNewTimer(t)

	enforcer := newTestEnforcer()
	mgr := NewManager(enforcer, time.Duration(defaultTestIdleTimeout))
	defer mgr.Close()

	mgr.ExitIdleMode()
	<-enforcer.exitIdleCh
	// Ensure that the channel moves to idle since there are no RPCs.
	select {
	case <-enforcer.enterIdleCh:
	case <-time.After(2 * defaultTestIdleTimeout):
		t.Fatal("Timeout waiting for channel to move to idle mode")
	}

	for i := 0; i < 100; i++ {
		// A call to OnCallBegin and OnCallEnd simulates an RPC.
		go func() {
			if err := mgr.OnCallBegin(); err != nil {
				t.Errorf("OnCallBegin() failed: %v", err)
			}
			mgr.OnCallEnd()
		}()
	}

	// Ensure that the channel moves out of idle as a result of the above RPC.
	select {
	case <-enforcer.exitIdleCh:
	case <-time.After(2 * defaultTestIdleTimeout):
		t.Fatal("Timeout waiting for channel to move out of idle mode")
	}

	// Ensure that only one call to exit idle mode is made to the CC.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	select {
	case <-enforcer.exitIdleCh:
		t.Fatal("More than one call to exit idle mode on the ClientConn; only one expected")
	case <-sCtx.Done():
	}
}

type racyState int32

const (
	stateInitial racyState = iota
	stateEnteredIdle
	stateExitedIdle
	stateActiveRPCs
)

// racyIdlnessEnforcer is a test idleness enforcer used specifically to test the
// race between idle timeout and incoming RPCs.
type racyEnforcer struct {
	t       *testing.T
	state   *racyState // Accessed atomically.
	started bool
}

// ExitIdleMode sets the internal state to stateExitedIdle. We should only ever
// exit idle when we are currently in idle.
func (ri *racyEnforcer) ExitIdleMode() error {
	// Set only on the initial ExitIdleMode
	if ri.started == false {
		if !atomic.CompareAndSwapInt32((*int32)(ri.state), int32(stateInitial), int32(stateInitial)) {
			return fmt.Errorf("idleness enforcer's first ExitIdleMode after EnterIdleMode")
		}
		ri.started = true
		return nil
	}
	if !atomic.CompareAndSwapInt32((*int32)(ri.state), int32(stateEnteredIdle), int32(stateExitedIdle)) {
		return fmt.Errorf("idleness enforcer asked to exit idle when it did not enter idle earlier")
	}
	return nil
}

// EnterIdleMode attempts to set the internal state to stateEnteredIdle. We should only ever enter idle before RPCs start.
func (ri *racyEnforcer) EnterIdleMode() {
	if !atomic.CompareAndSwapInt32((*int32)(ri.state), int32(stateInitial), int32(stateEnteredIdle)) {
		ri.t.Errorf("idleness enforcer asked to enter idle after rpcs started")
	}
}

// TestManager_IdleTimeoutRacesWithOnCallBegin tests the case where firing of
// the idle timeout races with an incoming RPC. The test verifies that if the
// timer callback wins the race and puts the channel in idle, the RPCs can kick
// it out of idle. And if the RPCs win the race and keep the channel active,
// then the timer callback should not attempt to put the channel in idle mode.
func (s) TestManager_IdleTimeoutRacesWithOnCallBegin(t *testing.T) {
	// Run multiple iterations to simulate different possibilities.
	for i := 0; i < 20; i++ {
		t.Run(fmt.Sprintf("iteration=%d", i), func(t *testing.T) {
			var idlenessState racyState
			enforcer := &racyEnforcer{t: t, state: &idlenessState}

			// Configure a large idle timeout so that we can control the
			// race between the timer callback and RPCs.
			mgr := NewManager(enforcer, time.Duration(10*time.Minute))
			defer mgr.Close()
			mgr.ExitIdleMode()

			var wg sync.WaitGroup
			wg.Add(1)
			go func() {
				defer wg.Done()
				<-time.After(defaultTestIdleTimeout / 50)
				mgr.handleIdleTimeout()
			}()
			for j := 0; j < 5; j++ {
				wg.Add(1)
				go func() {
					defer wg.Done()
					// Wait for the configured idle timeout and simulate an RPC to
					// race with the idle timeout timer callback.
					<-time.After(defaultTestIdleTimeout / 50)
					if err := mgr.OnCallBegin(); err != nil {
						t.Errorf("OnCallBegin() failed: %v", err)
					}
					atomic.StoreInt32((*int32)(&idlenessState), int32(stateActiveRPCs))
					mgr.OnCallEnd()
				}()
			}
			wg.Wait()
		})
	}
}
