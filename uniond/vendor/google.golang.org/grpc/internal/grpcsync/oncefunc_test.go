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
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// TestOnceFunc tests that a OnceFunc is executed only once even with multiple
// simultaneous callers of it.
func (s) TestOnceFunc(t *testing.T) {
	var v int32
	inc := OnceFunc(func() { atomic.AddInt32(&v, 1) })

	const numWorkers = 100
	var wg sync.WaitGroup // Blocks until all workers have called inc.
	wg.Add(numWorkers)

	block := NewEvent() // Signal to worker goroutines to call inc

	for i := 0; i < numWorkers; i++ {
		go func() {
			<-block.Done() // Wait for a signal.
			inc()          // Call the OnceFunc.
			wg.Done()
		}()
	}
	time.Sleep(time.Millisecond) // Allow goroutines to get to the block.
	block.Fire()                 // Unblock them.
	wg.Wait()                    // Wait for them to complete.
	if v != 1 {
		t.Fatalf("OnceFunc() called %v times; want 1", v)
	}
}
