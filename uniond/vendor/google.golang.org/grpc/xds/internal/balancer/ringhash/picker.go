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
	"fmt"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/status"
)

type picker struct {
	ring          *ring
	logger        *grpclog.PrefixLogger
	subConnStates map[*subConn]connectivity.State
}

func newPicker(ring *ring, logger *grpclog.PrefixLogger) *picker {
	states := make(map[*subConn]connectivity.State)
	for _, e := range ring.items {
		states[e.sc] = e.sc.effectiveState()
	}
	return &picker{ring: ring, logger: logger, subConnStates: states}
}

// handleRICSResult is the return type of handleRICS. It's needed to wrap the
// returned error from Pick() in a struct. With this, if the return values are
// `balancer.PickResult, error, bool`, linter complains because error is not the
// last return value.
type handleRICSResult struct {
	pr  balancer.PickResult
	err error
}

// handleRICS generates pick result if the entry is in Ready, Idle, Connecting
// or Shutdown. TransientFailure will be handled specifically after this
// function returns.
//
// The first return value indicates if the state is in Ready, Idle, Connecting
// or Shutdown. If it's true, the PickResult and error should be returned from
// Pick() as is.
func (p *picker) handleRICS(e *ringEntry) (handleRICSResult, bool) {
	switch state := p.subConnStates[e.sc]; state {
	case connectivity.Ready:
		return handleRICSResult{pr: balancer.PickResult{SubConn: e.sc.sc}}, true
	case connectivity.Idle:
		// Trigger Connect() and queue the pick.
		e.sc.queueConnect()
		return handleRICSResult{err: balancer.ErrNoSubConnAvailable}, true
	case connectivity.Connecting:
		return handleRICSResult{err: balancer.ErrNoSubConnAvailable}, true
	case connectivity.TransientFailure:
		// Return ok==false, so TransientFailure will be handled afterwards.
		return handleRICSResult{}, false
	case connectivity.Shutdown:
		// Shutdown can happen in a race where the old picker is called. A new
		// picker should already be sent.
		return handleRICSResult{err: balancer.ErrNoSubConnAvailable}, true
	default:
		// Should never reach this. All the connectivity states are already
		// handled in the cases.
		p.logger.Errorf("SubConn has undefined connectivity state: %v", state)
		return handleRICSResult{err: status.Errorf(codes.Unavailable, "SubConn has undefined connectivity state: %v", state)}, true
	}
}

func (p *picker) Pick(info balancer.PickInfo) (balancer.PickResult, error) {
	e := p.ring.pick(getRequestHash(info.Ctx))
	if hr, ok := p.handleRICS(e); ok {
		return hr.pr, hr.err
	}
	// ok was false, the entry is in transient failure.
	return p.handleTransientFailure(e)
}

func (p *picker) handleTransientFailure(e *ringEntry) (balancer.PickResult, error) {
	// Queue a connect on the first picked SubConn.
	e.sc.queueConnect()

	// Find next entry in the ring, skipping duplicate SubConns.
	e2 := nextSkippingDuplicates(p.ring, e)
	if e2 == nil {
		// There's no next entry available, fail the pick.
		return balancer.PickResult{}, fmt.Errorf("the only SubConn is in Transient Failure")
	}

	// For the second SubConn, also check Ready/Idle/Connecting as if it's the
	// first entry.
	if hr, ok := p.handleRICS(e2); ok {
		return hr.pr, hr.err
	}

	// The second SubConn is also in TransientFailure. Queue a connect on it.
	e2.sc.queueConnect()

	// If it gets here, this is after the second SubConn, and the second SubConn
	// was in TransientFailure.
	//
	// Loop over all other SubConns:
	// - If all SubConns so far are all TransientFailure, trigger Connect() on
	// the TransientFailure SubConns, and keep going.
	// - If there's one SubConn that's not in TransientFailure, keep checking
	// the remaining SubConns (in case there's a Ready, which will be returned),
	// but don't not trigger Connect() on the other SubConns.
	var firstNonFailedFound bool
	for ee := nextSkippingDuplicates(p.ring, e2); ee != e; ee = nextSkippingDuplicates(p.ring, ee) {
		scState := p.subConnStates[ee.sc]
		if scState == connectivity.Ready {
			return balancer.PickResult{SubConn: ee.sc.sc}, nil
		}
		if firstNonFailedFound {
			continue
		}
		if scState == connectivity.TransientFailure {
			// This will queue a connect.
			ee.sc.queueConnect()
			continue
		}
		// This is a SubConn in a non-failure state. We continue to check the
		// other SubConns, but remember that there was a non-failed SubConn
		// seen. After this, Pick() will never trigger any SubConn to Connect().
		firstNonFailedFound = true
		if scState == connectivity.Idle {
			// This is the first non-failed SubConn, and it is in a real Idle
			// state. Trigger it to Connect().
			ee.sc.queueConnect()
		}
	}
	return balancer.PickResult{}, fmt.Errorf("no connection is Ready")
}

// nextSkippingDuplicates finds the next entry in the ring, with a different
// subconn from the given entry.
func nextSkippingDuplicates(ring *ring, entry *ringEntry) *ringEntry {
	for next := ring.next(entry); next != entry; next = ring.next(next) {
		if next.sc != entry.sc {
			return next
		}
	}
	// There's no qualifying next entry.
	return nil
}
