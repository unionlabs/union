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

package grpcsync

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
)

const (
	defaultTestTimeout      = 5 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond // For events expected to *not* happen.
)

// TestCallbackSerializer_Schedule_FIFO verifies that callbacks are executed in
// the same order in which they were scheduled.
func (s) TestCallbackSerializer_Schedule_FIFO(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	cs := NewCallbackSerializer(ctx)
	defer cancel()

	// We have two channels, one to record the order of scheduling, and the
	// other to record the order of execution. We spawn a bunch of goroutines
	// which record the order of scheduling and call the actual Schedule()
	// method as well.  The callbacks record the order of execution.
	//
	// We need to grab a lock to record order of scheduling to guarantee that
	// the act of recording and the act of calling Schedule() happen atomically.
	const numCallbacks = 100
	var mu sync.Mutex
	scheduleOrderCh := make(chan int, numCallbacks)
	executionOrderCh := make(chan int, numCallbacks)
	for i := 0; i < numCallbacks; i++ {
		go func(id int) {
			mu.Lock()
			defer mu.Unlock()
			scheduleOrderCh <- id
			cs.TrySchedule(func(ctx context.Context) {
				select {
				case <-ctx.Done():
					return
				case executionOrderCh <- id:
				}
			})
		}(i)
	}

	// Spawn a couple of goroutines to capture the order or scheduling and the
	// order of execution.
	scheduleOrder := make([]int, numCallbacks)
	executionOrder := make([]int, numCallbacks)
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		defer wg.Done()
		for i := 0; i < numCallbacks; i++ {
			select {
			case <-ctx.Done():
				return
			case id := <-scheduleOrderCh:
				scheduleOrder[i] = id
			}
		}
	}()
	go func() {
		defer wg.Done()
		for i := 0; i < numCallbacks; i++ {
			select {
			case <-ctx.Done():
				return
			case id := <-executionOrderCh:
				executionOrder[i] = id
			}
		}
	}()
	wg.Wait()

	if diff := cmp.Diff(executionOrder, scheduleOrder); diff != "" {
		t.Fatalf("Callbacks are not executed in scheduled order. diff(-want, +got):\n%s", diff)
	}
}

// TestCallbackSerializer_Schedule_Concurrent verifies that all concurrently
// scheduled callbacks get executed.
func (s) TestCallbackSerializer_Schedule_Concurrent(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	cs := NewCallbackSerializer(ctx)
	defer cancel()

	// Schedule callbacks concurrently by calling Schedule() from goroutines.
	// The execution of the callbacks call Done() on the waitgroup, which
	// eventually unblocks the test and allows it to complete.
	const numCallbacks = 100
	var wg sync.WaitGroup
	wg.Add(numCallbacks)
	for i := 0; i < numCallbacks; i++ {
		go func() {
			cs.TrySchedule(func(context.Context) {
				wg.Done()
			})
		}()
	}

	// We call Wait() on the waitgroup from a goroutine so that we can select on
	// the Wait() being unblocked and the overall test deadline expiring.
	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	select {
	case <-ctx.Done():
		t.Fatal("Timeout waiting for all scheduled callbacks to be executed")
	case <-done:
	}
}

// TestCallbackSerializer_Schedule_Close verifies that callbacks in the queue
// are not executed once Close() returns.
func (s) TestCallbackSerializer_Schedule_Close(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	serializerCtx, serializerCancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	cs := NewCallbackSerializer(serializerCtx)

	// Schedule a callback which blocks until the context passed to it is
	// canceled. It also closes a channel to signal that it has started.
	firstCallbackStartedCh := make(chan struct{})
	cs.TrySchedule(func(ctx context.Context) {
		close(firstCallbackStartedCh)
		<-ctx.Done()
	})

	// Schedule a bunch of callbacks. These should be executed since they are
	// scheduled before the serializer is closed.
	const numCallbacks = 10
	callbackCh := make(chan int, numCallbacks)
	for i := 0; i < numCallbacks; i++ {
		num := i
		callback := func(context.Context) { callbackCh <- num }
		onFailure := func() { t.Fatal("Schedule failed to accept a callback when the serializer is yet to be closed") }
		cs.ScheduleOr(callback, onFailure)
	}

	// Ensure that none of the newer callbacks are executed at this point.
	select {
	case <-time.After(defaultTestShortTimeout):
	case <-callbackCh:
		t.Fatal("Newer callback executed when older one is still executing")
	}

	// Wait for the first callback to start before closing the scheduler.
	<-firstCallbackStartedCh

	// Cancel the context which will unblock the first callback. All of the
	// other callbacks (which have not started executing at this point) should
	// be executed after this.
	serializerCancel()

	// Ensure that the newer callbacks are executed.
	for i := 0; i < numCallbacks; i++ {
		select {
		case <-ctx.Done():
			t.Fatal("Timeout when waiting for callback scheduled before close to be executed")
		case num := <-callbackCh:
			if num != i {
				t.Fatalf("Executing callback %d, want %d", num, i)
			}
		}
	}
	<-cs.Done()

	// Ensure that a callback cannot be scheduled after the serializer is
	// closed.
	done := make(chan struct{})
	callback := func(context.Context) { t.Fatal("Scheduled a callback after closing the serializer") }
	onFailure := func() { close(done) }
	cs.ScheduleOr(callback, onFailure)
	select {
	case <-time.After(defaultTestTimeout):
		t.Fatal("Successfully scheduled callback after serializer is closed")
	case <-done:
	}
}
