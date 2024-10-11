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

// Package ringhash implements the ringhash balancer.
package ringhash

import (
	"encoding/json"
	"errors"
	"fmt"
	"sync"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/base"
	"google.golang.org/grpc/balancer/weightedroundrobin"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
)

// Name is the name of the ring_hash balancer.
const Name = "ring_hash_experimental"

func init() {
	balancer.Register(bb{})
}

type bb struct{}

func (bb) Build(cc balancer.ClientConn, _ balancer.BuildOptions) balancer.Balancer {
	b := &ringhashBalancer{
		cc:              cc,
		subConns:        resolver.NewAddressMap(),
		scStates:        make(map[balancer.SubConn]*subConn),
		csEvltr:         &connectivityStateEvaluator{},
		orderedSubConns: make([]*subConn, 0),
	}
	b.logger = prefixLogger(b)
	b.logger.Infof("Created")
	return b
}

func (bb) Name() string {
	return Name
}

func (bb) ParseConfig(c json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	return parseConfig(c)
}

type subConn struct {
	addr   string
	weight uint32
	sc     balancer.SubConn
	logger *grpclog.PrefixLogger

	mu sync.RWMutex
	// This is the actual state of this SubConn (as updated by the ClientConn).
	// The effective state can be different, see comment of attemptedToConnect.
	state connectivity.State
	// failing is whether this SubConn is in a failing state. A subConn is
	// considered to be in a failing state if it was previously in
	// TransientFailure.
	//
	// This affects the effective connectivity state of this SubConn, e.g.
	// - if the actual state is Idle or Connecting, but this SubConn is failing,
	// the effective state is TransientFailure.
	//
	// This is used in pick(). E.g. if a subConn is Idle, but has failing as
	// true, pick() will
	// - consider this SubConn as TransientFailure, and check the state of the
	// next SubConn.
	// - trigger Connect() (note that normally a SubConn in real
	// TransientFailure cannot Connect())
	//
	// A subConn starts in non-failing (failing is false). A transition to
	// TransientFailure sets failing to true (and it stays true). A transition
	// to Ready sets failing to false.
	failing bool
	// connectQueued is true if a Connect() was queued for this SubConn while
	// it's not in Idle (most likely was in TransientFailure). A Connect() will
	// be triggered on this SubConn when it turns Idle.
	//
	// When connectivity state is updated to Idle for this SubConn, if
	// connectQueued is true, Connect() will be called on the SubConn.
	connectQueued bool
	// attemptingToConnect indicates if this subconn is attempting to connect.
	// It's set when queueConnect is called. It's unset when the state is
	// changed to Ready/Shutdown, or Idle (and if connectQueued is false).
	attemptingToConnect bool
}

// setState updates the state of this SubConn.
//
// It also handles the queued Connect(). If the new state is Idle, and a
// Connect() was queued, this SubConn will be triggered to Connect().
func (sc *subConn) setState(s connectivity.State) {
	sc.mu.Lock()
	defer sc.mu.Unlock()
	switch s {
	case connectivity.Idle:
		// Trigger Connect() if new state is Idle, and there is a queued connect.
		if sc.connectQueued {
			sc.connectQueued = false
			sc.logger.Infof("Executing a queued connect for subConn moving to state: %v", sc.state)
			sc.sc.Connect()
		} else {
			sc.attemptingToConnect = false
		}
	case connectivity.Connecting:
		// Clear connectQueued if the SubConn isn't failing. This state
		// transition is unlikely to happen, but handle this just in case.
		sc.connectQueued = false
	case connectivity.Ready:
		// Clear connectQueued if the SubConn isn't failing. This state
		// transition is unlikely to happen, but handle this just in case.
		sc.connectQueued = false
		sc.attemptingToConnect = false
		// Set to a non-failing state.
		sc.failing = false
	case connectivity.TransientFailure:
		// Set to a failing state.
		sc.failing = true
	case connectivity.Shutdown:
		sc.attemptingToConnect = false
	}
	sc.state = s
}

// effectiveState returns the effective state of this SubConn. It can be
// different from the actual state, e.g. Idle while the subConn is failing is
// considered TransientFailure. Read comment of field failing for other cases.
func (sc *subConn) effectiveState() connectivity.State {
	sc.mu.RLock()
	defer sc.mu.RUnlock()
	if sc.failing && (sc.state == connectivity.Idle || sc.state == connectivity.Connecting) {
		return connectivity.TransientFailure
	}
	return sc.state
}

// queueConnect sets a boolean so that when the SubConn state changes to Idle,
// it's Connect() will be triggered. If the SubConn state is already Idle, it
// will just call Connect().
func (sc *subConn) queueConnect() {
	sc.mu.Lock()
	defer sc.mu.Unlock()
	sc.attemptingToConnect = true
	if sc.state == connectivity.Idle {
		sc.logger.Infof("Executing a queued connect for subConn in state: %v", sc.state)
		sc.sc.Connect()
		return
	}
	// Queue this connect, and when this SubConn switches back to Idle (happens
	// after backoff in TransientFailure), it will Connect().
	sc.logger.Infof("Queueing a connect for subConn in state: %v", sc.state)
	sc.connectQueued = true
}

func (sc *subConn) isAttemptingToConnect() bool {
	sc.mu.Lock()
	defer sc.mu.Unlock()
	return sc.attemptingToConnect
}

type ringhashBalancer struct {
	cc     balancer.ClientConn
	logger *grpclog.PrefixLogger

	config   *LBConfig
	subConns *resolver.AddressMap // Map from resolver.Address to `*subConn`.
	scStates map[balancer.SubConn]*subConn

	// ring is always in sync with subConns. When subConns change, a new ring is
	// generated. Note that address weights updates (they are keys in the
	// subConns map) also regenerates the ring.
	ring    *ring
	picker  balancer.Picker
	csEvltr *connectivityStateEvaluator
	state   connectivity.State

	resolverErr error // the last error reported by the resolver; cleared on successful resolution
	connErr     error // the last connection error; cleared upon leaving TransientFailure

	// orderedSubConns contains the list of subconns in the order that addresses
	// appear from the resolver. Together with lastInternallyTriggeredSCIndex,
	// this allows triggering connection attempts to all SubConns independently
	// of the order they appear on the ring. Always in sync with ring and
	// subConns. The index is reset when addresses change.
	orderedSubConns                []*subConn
	lastInternallyTriggeredSCIndex int
}

// updateAddresses creates new SubConns and removes SubConns, based on the
// address update.
//
// The return value is whether the new address list is different from the
// previous. True if
// - an address was added
// - an address was removed
// - an address's weight was updated
//
// Note that this function doesn't trigger SubConn connecting, so all the new
// SubConn states are Idle.
func (b *ringhashBalancer) updateAddresses(addrs []resolver.Address) bool {
	var addrsUpdated bool
	// addrsSet is the set converted from addrs, used for quick lookup.
	addrsSet := resolver.NewAddressMap()

	b.orderedSubConns = b.orderedSubConns[:0] // reuse the underlying array.

	for _, addr := range addrs {
		addrsSet.Set(addr, true)
		newWeight := getWeightAttribute(addr)
		if val, ok := b.subConns.Get(addr); !ok {
			var sc balancer.SubConn
			opts := balancer.NewSubConnOptions{
				HealthCheckEnabled: true,
				StateListener:      func(state balancer.SubConnState) { b.updateSubConnState(sc, state) },
			}
			sc, err := b.cc.NewSubConn([]resolver.Address{addr}, opts)
			if err != nil {
				b.logger.Warningf("Failed to create new SubConn: %v", err)
				continue
			}
			scs := &subConn{addr: addr.Addr, weight: newWeight, sc: sc}
			scs.logger = subConnPrefixLogger(b, scs)
			scs.setState(connectivity.Idle)
			b.state = b.csEvltr.recordTransition(connectivity.Shutdown, connectivity.Idle)
			b.subConns.Set(addr, scs)
			b.scStates[sc] = scs
			b.orderedSubConns = append(b.orderedSubConns, scs)
			addrsUpdated = true
		} else {
			// We have seen this address before and created a subConn for it. If the
			// weight associated with the address has changed, update the subConns map
			// with the new weight. This will be used when a new ring is created.
			//
			// There is no need to call UpdateAddresses on the subConn at this point
			// since *only* the weight attribute has changed, and that does not affect
			// subConn uniqueness.
			scInfo := val.(*subConn)
			b.orderedSubConns = append(b.orderedSubConns, scInfo)
			if oldWeight := scInfo.weight; oldWeight != newWeight {
				scInfo.weight = newWeight
				b.subConns.Set(addr, scInfo)
				// Return true to force recreation of the ring.
				addrsUpdated = true
			}
		}
	}
	for _, addr := range b.subConns.Keys() {
		// addr was removed by resolver.
		if _, ok := addrsSet.Get(addr); !ok {
			v, _ := b.subConns.Get(addr)
			scInfo := v.(*subConn)
			scInfo.sc.Shutdown()
			b.subConns.Delete(addr)
			addrsUpdated = true
			// Keep the state of this sc in b.scStates until sc's state becomes Shutdown.
			// The entry will be deleted in updateSubConnState.
		}
	}
	if addrsUpdated {
		b.lastInternallyTriggeredSCIndex = 0
	}
	return addrsUpdated
}

func (b *ringhashBalancer) UpdateClientConnState(s balancer.ClientConnState) error {
	b.logger.Infof("Received update from resolver, balancer config: %+v", pretty.ToJSON(s.BalancerConfig))
	newConfig, ok := s.BalancerConfig.(*LBConfig)
	if !ok {
		return fmt.Errorf("unexpected balancer config with type: %T", s.BalancerConfig)
	}

	// If addresses were updated, whether it resulted in SubConn
	// creation/deletion, or just weight update, we need to regenerate the ring
	// and send a new picker.
	regenerateRing := b.updateAddresses(s.ResolverState.Addresses)

	// If the ring configuration has changed, we need to regenerate the ring and
	// send a new picker.
	if b.config == nil || b.config.MinRingSize != newConfig.MinRingSize || b.config.MaxRingSize != newConfig.MaxRingSize {
		regenerateRing = true
	}
	b.config = newConfig

	// If resolver state contains no addresses, return an error so ClientConn
	// will trigger re-resolve. Also records this as an resolver error, so when
	// the overall state turns transient failure, the error message will have
	// the zero address information.
	if len(s.ResolverState.Addresses) == 0 {
		b.ResolverError(errors.New("produced zero addresses"))
		return balancer.ErrBadResolverState
	}

	if regenerateRing {
		// Ring creation is guaranteed to not fail because we call newRing()
		// with a non-empty subConns map.
		b.ring = newRing(b.subConns, b.config.MinRingSize, b.config.MaxRingSize, b.logger)
		b.regeneratePicker()
		b.cc.UpdateState(balancer.State{ConnectivityState: b.state, Picker: b.picker})
	}

	// Successful resolution; clear resolver error and return nil.
	b.resolverErr = nil
	return nil
}

func (b *ringhashBalancer) ResolverError(err error) {
	b.resolverErr = err
	if b.subConns.Len() == 0 {
		b.state = connectivity.TransientFailure
	}

	if b.state != connectivity.TransientFailure {
		// The picker will not change since the balancer does not currently
		// report an error.
		return
	}
	b.regeneratePicker()
	b.cc.UpdateState(balancer.State{
		ConnectivityState: b.state,
		Picker:            b.picker,
	})
}

func (b *ringhashBalancer) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	b.logger.Errorf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state)
}

// updateSubConnState updates the per-SubConn state stored in the ring, and also
// the aggregated state.
//
//	It triggers an update to cc when:
//	- the new state is TransientFailure, to update the error message
//	  - it's possible that this is a noop, but sending an extra update is easier
//	    than comparing errors
//
//	- the aggregated state is changed
//	  - the same picker will be sent again, but this update may trigger a re-pick
//	    for some RPCs.
func (b *ringhashBalancer) updateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	s := state.ConnectivityState
	if logger.V(2) {
		b.logger.Infof("Handle SubConn state change: %p, %v", sc, s)
	}
	scs, ok := b.scStates[sc]
	if !ok {
		b.logger.Infof("Received state change for an unknown SubConn: %p, %v", sc, s)
		return
	}
	oldSCState := scs.effectiveState()
	scs.setState(s)
	newSCState := scs.effectiveState()
	b.logger.Infof("SubConn's effective old state was: %v, new state is %v", oldSCState, newSCState)

	b.state = b.csEvltr.recordTransition(oldSCState, newSCState)

	switch s {
	case connectivity.TransientFailure:
		// Save error to be reported via picker.
		b.connErr = state.ConnectionError
	case connectivity.Shutdown:
		// When an address was removed by resolver, b called Shutdown but kept
		// the sc's state in scStates. Remove state for this sc here.
		delete(b.scStates, sc)
	}

	if oldSCState != newSCState {
		// Because the picker caches the state of the subconns, we always
		// regenerate and update the picker when the effective SubConn state
		// changes.
		b.regeneratePicker()
		b.logger.Infof("Pushing new state %v and picker %p", b.state, b.picker)
		b.cc.UpdateState(balancer.State{ConnectivityState: b.state, Picker: b.picker})
	}

	switch b.state {
	case connectivity.Connecting, connectivity.TransientFailure:
		// When overall state is TransientFailure, we need to make sure at least
		// one SubConn is attempting to connect, otherwise this balancer may
		// never get picks if the parent is priority.
		//
		// Because we report Connecting as the overall state when only one
		// SubConn is in TransientFailure, we do the same check for Connecting
		// here.
		//
		// Note that this check also covers deleting SubConns due to address
		// change. E.g. if the SubConn attempting to connect is deleted, and the
		// overall state is TF. Since there must be at least one SubConn
		// attempting to connect, we need to trigger one. But since the deleted
		// SubConn will eventually send a shutdown update, this code will run
		// and trigger the next SubConn to connect.
		for _, v := range b.subConns.Values() {
			sc := v.(*subConn)
			if sc.isAttemptingToConnect() {
				return
			}
		}

		// Trigger a SubConn (the next in the order addresses appear in the
		// resolver) to connect if nobody is attempting to connect.
		b.lastInternallyTriggeredSCIndex = (b.lastInternallyTriggeredSCIndex + 1) % len(b.orderedSubConns)
		b.orderedSubConns[b.lastInternallyTriggeredSCIndex].queueConnect()
	}
}

// mergeErrors builds an error from the last connection error and the last
// resolver error.  Must only be called if b.state is TransientFailure.
func (b *ringhashBalancer) mergeErrors() error {
	// connErr must always be non-nil unless there are no SubConns, in which
	// case resolverErr must be non-nil.
	if b.connErr == nil {
		return fmt.Errorf("last resolver error: %v", b.resolverErr)
	}
	if b.resolverErr == nil {
		return fmt.Errorf("last connection error: %v", b.connErr)
	}
	return fmt.Errorf("last connection error: %v; last resolver error: %v", b.connErr, b.resolverErr)
}

func (b *ringhashBalancer) regeneratePicker() {
	if b.state == connectivity.TransientFailure {
		b.picker = base.NewErrPicker(b.mergeErrors())
		return
	}
	b.picker = newPicker(b.ring, b.logger)
}

func (b *ringhashBalancer) Close() {
	b.logger.Infof("Shutdown")
}

func (b *ringhashBalancer) ExitIdle() {
	// ExitIdle implementation is a no-op because connections are either
	// triggers from picks or from subConn state changes.
}

// connectivityStateEvaluator takes the connectivity states of multiple SubConns
// and returns one aggregated connectivity state.
//
// It's not thread safe.
type connectivityStateEvaluator struct {
	sum  uint64
	nums [5]uint64
}

// recordTransition records state change happening in subConn and based on that
// it evaluates what aggregated state should be.
//
// - If there is at least one subchannel in READY state, report READY.
// - If there are 2 or more subchannels in TRANSIENT_FAILURE state, report TRANSIENT_FAILURE.
// - If there is at least one subchannel in CONNECTING state, report CONNECTING.
// - If there is one subchannel in TRANSIENT_FAILURE and there is more than one subchannel, report state CONNECTING.
// - If there is at least one subchannel in Idle state, report Idle.
// - Otherwise, report TRANSIENT_FAILURE.
//
// Note that if there are 1 connecting, 2 transient failure, the overall state
// is transient failure. This is because the second transient failure is a
// fallback of the first failing SubConn, and we want to report transient
// failure to failover to the lower priority.
func (cse *connectivityStateEvaluator) recordTransition(oldState, newState connectivity.State) connectivity.State {
	// Update counters.
	for idx, state := range []connectivity.State{oldState, newState} {
		updateVal := 2*uint64(idx) - 1 // -1 for oldState and +1 for new.
		cse.nums[state] += updateVal
	}
	if oldState == connectivity.Shutdown {
		// There's technically no transition from Shutdown. But we record a
		// Shutdown->Idle transition when a new SubConn is created.
		cse.sum++
	}
	if newState == connectivity.Shutdown {
		cse.sum--
	}

	if cse.nums[connectivity.Ready] > 0 {
		return connectivity.Ready
	}
	if cse.nums[connectivity.TransientFailure] > 1 {
		return connectivity.TransientFailure
	}
	if cse.nums[connectivity.Connecting] > 0 {
		return connectivity.Connecting
	}
	if cse.nums[connectivity.TransientFailure] > 0 && cse.sum > 1 {
		return connectivity.Connecting
	}
	if cse.nums[connectivity.Idle] > 0 {
		return connectivity.Idle
	}
	return connectivity.TransientFailure
}

// getWeightAttribute is a convenience function which returns the value of the
// weight attribute stored in the BalancerAttributes field of addr, using the
// weightedroundrobin package.
//
// When used in the xDS context, the weight attribute is guaranteed to be
// non-zero. But, when used in a non-xDS context, the weight attribute could be
// unset. A Default of 1 is used in the latter case.
func getWeightAttribute(addr resolver.Address) uint32 {
	w := weightedroundrobin.GetAddrInfo(addr).Weight
	if w == 0 {
		return 1
	}
	return w
}
