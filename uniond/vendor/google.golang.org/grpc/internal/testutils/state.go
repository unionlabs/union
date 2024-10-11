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

package testutils

import (
	"context"
	"testing"

	"google.golang.org/grpc/connectivity"
)

// A StateChanger reports state changes, e.g. a grpc.ClientConn.
type StateChanger interface {
	// Connect begins connecting the StateChanger.
	Connect()
	// GetState returns the current state of the StateChanger.
	GetState() connectivity.State
	// WaitForStateChange returns true when the state becomes s, or returns
	// false if ctx is canceled first.
	WaitForStateChange(ctx context.Context, s connectivity.State) bool
}

// StayConnected makes sc stay connected by repeatedly calling sc.Connect()
// until the state becomes Shutdown or until the context expires.
func StayConnected(ctx context.Context, sc StateChanger) {
	for {
		state := sc.GetState()
		switch state {
		case connectivity.Idle:
			sc.Connect()
		case connectivity.Shutdown:
			return
		}
		if !sc.WaitForStateChange(ctx, state) {
			return
		}
	}
}

// AwaitState waits for sc to enter stateWant or fatal errors if it doesn't
// happen before ctx expires.
func AwaitState(ctx context.Context, t *testing.T, sc StateChanger, stateWant connectivity.State) {
	t.Helper()
	for state := sc.GetState(); state != stateWant; state = sc.GetState() {
		if !sc.WaitForStateChange(ctx, state) {
			t.Fatalf("Timed out waiting for state change.  got %v; want %v", state, stateWant)
		}
	}
}

// AwaitNotState waits for sc to leave stateDoNotWant or fatal errors if it
// doesn't happen before ctx expires.
func AwaitNotState(ctx context.Context, t *testing.T, sc StateChanger, stateDoNotWant connectivity.State) {
	t.Helper()
	for state := sc.GetState(); state == stateDoNotWant; state = sc.GetState() {
		if !sc.WaitForStateChange(ctx, state) {
			t.Fatalf("Timed out waiting for state change.  got %v; want NOT %v", state, stateDoNotWant)
		}
	}
}

// AwaitNoStateChange expects ctx to be canceled before sc's state leaves
// currState, and fatal errors otherwise.
func AwaitNoStateChange(ctx context.Context, t *testing.T, sc StateChanger, currState connectivity.State) {
	t.Helper()
	if sc.WaitForStateChange(ctx, currState) {
		t.Fatalf("State changed from %q to %q when no state change was expected", currState, sc.GetState())
	}
}
