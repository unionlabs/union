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

package balancer

import (
	"testing"

	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/grpctest"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// TestRecordTransition_FirstStateChange tests the first call to
// RecordTransition where the `oldState` is usually set to `Shutdown` (a state
// that the ConnectivityStateEvaluator is set to ignore).
func (s) TestRecordTransition_FirstStateChange(t *testing.T) {
	tests := []struct {
		newState  connectivity.State
		wantState connectivity.State
	}{
		{
			newState:  connectivity.Idle,
			wantState: connectivity.Idle,
		},
		{
			newState:  connectivity.Connecting,
			wantState: connectivity.Connecting,
		},
		{
			newState:  connectivity.Ready,
			wantState: connectivity.Ready,
		},
		{
			newState:  connectivity.TransientFailure,
			wantState: connectivity.TransientFailure,
		},
		{
			newState:  connectivity.Shutdown,
			wantState: connectivity.TransientFailure,
		},
	}
	for _, test := range tests {
		cse := &ConnectivityStateEvaluator{}
		if gotState := cse.RecordTransition(connectivity.Shutdown, test.newState); gotState != test.wantState {
			t.Fatalf("RecordTransition(%v, %v) = %v, want %v", connectivity.Shutdown, test.newState, gotState, test.wantState)
		}
	}
}

// TestRecordTransition_SameState tests the scenario where state transitions to
// the same state are recorded multiple times.
func (s) TestRecordTransition_SameState(t *testing.T) {
	tests := []struct {
		newState  connectivity.State
		wantState connectivity.State
	}{
		{
			newState:  connectivity.Idle,
			wantState: connectivity.Idle,
		},
		{
			newState:  connectivity.Connecting,
			wantState: connectivity.Connecting,
		},
		{
			newState:  connectivity.Ready,
			wantState: connectivity.Ready,
		},
		{
			newState:  connectivity.TransientFailure,
			wantState: connectivity.TransientFailure,
		},
		{
			newState:  connectivity.Shutdown,
			wantState: connectivity.TransientFailure,
		},
	}
	const numStateChanges = 5
	for _, test := range tests {
		cse := &ConnectivityStateEvaluator{}
		var prevState, gotState connectivity.State
		prevState = connectivity.Shutdown
		for i := 0; i < numStateChanges; i++ {
			gotState = cse.RecordTransition(prevState, test.newState)
			prevState = test.newState
		}
		if gotState != test.wantState {
			t.Fatalf("RecordTransition() = %v, want %v", gotState, test.wantState)
		}
	}
}

// TestRecordTransition_SingleSubConn_DifferentStates tests some common
// connectivity state change scenarios, on a single subConn.
func (s) TestRecordTransition_SingleSubConn_DifferentStates(t *testing.T) {
	tests := []struct {
		name      string
		states    []connectivity.State
		wantState connectivity.State
	}{
		{
			name:      "regular transition to ready",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.Ready},
			wantState: connectivity.Ready,
		},
		{
			name:      "regular transition to transient failure",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.TransientFailure},
			wantState: connectivity.TransientFailure,
		},
		{
			name:      "regular transition to ready",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.Idle},
			wantState: connectivity.Idle,
		},
		{
			name:      "transition from ready to transient failure",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.TransientFailure},
			wantState: connectivity.TransientFailure,
		},
		{
			name:      "transition from transient failure back to ready",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.TransientFailure, connectivity.Ready},
			wantState: connectivity.Ready,
		},
		{
			// This state transition is usually suppressed at the LB policy level, by
			// not calling RecordTransition.
			name:      "transition from transient failure back to idle",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.TransientFailure, connectivity.Idle},
			wantState: connectivity.Idle,
		},
		{
			// This state transition is usually suppressed at the LB policy level, by
			// not calling RecordTransition.
			name:      "transition from transient failure back to connecting",
			states:    []connectivity.State{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.TransientFailure, connectivity.Connecting},
			wantState: connectivity.Connecting,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			cse := &ConnectivityStateEvaluator{}
			var prevState, gotState connectivity.State
			prevState = connectivity.Shutdown
			for _, newState := range test.states {
				gotState = cse.RecordTransition(prevState, newState)
				prevState = newState
			}
			if gotState != test.wantState {
				t.Fatalf("RecordTransition() = %v, want %v", gotState, test.wantState)
			}
		})
	}
}

// TestRecordTransition_MultipleSubConns_DifferentStates tests state transitions
// among multiple subConns, and verifies that the connectivity state aggregation
// algorithm produces the expected aggregate connectivity state.
func (s) TestRecordTransition_MultipleSubConns_DifferentStates(t *testing.T) {
	tests := []struct {
		name string
		// Each entry in this slice corresponds to the state changes happening on an
		// individual subConn.
		subConnStates [][]connectivity.State
		wantState     connectivity.State
	}{
		{
			name: "atleast one ready",
			subConnStates: [][]connectivity.State{
				{connectivity.Idle, connectivity.Connecting, connectivity.Ready},
				{connectivity.Idle},
				{connectivity.Idle, connectivity.Connecting},
				{connectivity.Idle, connectivity.Connecting, connectivity.TransientFailure},
			},
			wantState: connectivity.Ready,
		},
		{
			name: "atleast one connecting",
			subConnStates: [][]connectivity.State{
				{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.Connecting},
				{connectivity.Idle},
				{connectivity.Idle, connectivity.Connecting, connectivity.TransientFailure},
			},
			wantState: connectivity.Connecting,
		},
		{
			name: "atleast one idle",
			subConnStates: [][]connectivity.State{
				{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.Idle},
				{connectivity.Idle, connectivity.Connecting, connectivity.TransientFailure},
			},
			wantState: connectivity.Idle,
		},
		{
			name: "atleast one transient failure",
			subConnStates: [][]connectivity.State{
				{connectivity.Idle, connectivity.Connecting, connectivity.Ready, connectivity.TransientFailure},
				{connectivity.TransientFailure},
			},
			wantState: connectivity.TransientFailure,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			cse := &ConnectivityStateEvaluator{}
			var prevState, gotState connectivity.State
			for _, scStates := range test.subConnStates {
				prevState = connectivity.Shutdown
				for _, newState := range scStates {
					gotState = cse.RecordTransition(prevState, newState)
					prevState = newState
				}
			}
			if gotState != test.wantState {
				t.Fatalf("RecordTransition() = %v, want %v", gotState, test.wantState)
			}
		})
	}
}
