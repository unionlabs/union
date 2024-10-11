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

// Package outlierdetection provides an implementation of the outlier detection
// LB policy, as defined in
// https://github.com/grpc/proposal/blob/master/A50-xds-outlier-detection.md.
package outlierdetection

import (
	"encoding/json"
	"fmt"
	"math"
	"math/rand"
	"strings"
	"sync"
	"sync/atomic"
	"time"
	"unsafe"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/balancer/gracefulswitch"
	"google.golang.org/grpc/internal/buffer"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/grpcsync"
	iserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
)

// Globals to stub out in tests.
var (
	afterFunc = time.AfterFunc
	now       = time.Now
)

// Name is the name of the outlier detection balancer.
const Name = "outlier_detection_experimental"

func init() {
	balancer.Register(bb{})
}

type bb struct{}

func (bb) Build(cc balancer.ClientConn, bOpts balancer.BuildOptions) balancer.Balancer {
	b := &outlierDetectionBalancer{
		cc:             cc,
		closed:         grpcsync.NewEvent(),
		done:           grpcsync.NewEvent(),
		addrs:          make(map[string]*addressInfo),
		scWrappers:     make(map[balancer.SubConn]*subConnWrapper),
		scUpdateCh:     buffer.NewUnbounded(),
		pickerUpdateCh: buffer.NewUnbounded(),
		channelzParent: bOpts.ChannelzParent,
	}
	b.logger = prefixLogger(b)
	b.logger.Infof("Created")
	b.child = gracefulswitch.NewBalancer(b, bOpts)
	go b.run()
	return b
}

func (bb) ParseConfig(s json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	lbCfg := &LBConfig{
		// Default top layer values as documented in A50.
		Interval:           iserviceconfig.Duration(10 * time.Second),
		BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
		MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
		MaxEjectionPercent: 10,
	}

	// This unmarshalling handles underlying layers sre and fpe which have their
	// own defaults for their fields if either sre or fpe are present.
	if err := json.Unmarshal(s, lbCfg); err != nil { // Validates child config if present as well.
		return nil, fmt.Errorf("xds: unable to unmarshal LBconfig: %s, error: %v", string(s), err)
	}

	// Note: in the xds flow, these validations will never fail. The xdsclient
	// performs the same validations as here on the xds Outlier Detection
	// resource before parsing resource into JSON which this function gets
	// called with. A50 defines two separate places for these validations to
	// take place, the xdsclient and this ParseConfig method. "When parsing a
	// config from JSON, if any of these requirements is violated, that should
	// be treated as a parsing error." - A50
	switch {
	// "The google.protobuf.Duration fields interval, base_ejection_time, and
	// max_ejection_time must obey the restrictions in the
	// google.protobuf.Duration documentation and they must have non-negative
	// values." - A50
	// Approximately 290 years is the maximum time that time.Duration (int64)
	// can represent. The restrictions on the protobuf.Duration field are to be
	// within +-10000 years. Thus, just check for negative values.
	case lbCfg.Interval < 0:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.interval = %s; must be >= 0", lbCfg.Interval)
	case lbCfg.BaseEjectionTime < 0:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.base_ejection_time = %s; must be >= 0", lbCfg.BaseEjectionTime)
	case lbCfg.MaxEjectionTime < 0:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.max_ejection_time = %s; must be >= 0", lbCfg.MaxEjectionTime)

	// "The fields max_ejection_percent,
	// success_rate_ejection.enforcement_percentage,
	// failure_percentage_ejection.threshold, and
	// failure_percentage.enforcement_percentage must have values less than or
	// equal to 100." - A50
	case lbCfg.MaxEjectionPercent > 100:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.max_ejection_percent = %v; must be <= 100", lbCfg.MaxEjectionPercent)
	case lbCfg.SuccessRateEjection != nil && lbCfg.SuccessRateEjection.EnforcementPercentage > 100:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.SuccessRateEjection.enforcement_percentage = %v; must be <= 100", lbCfg.SuccessRateEjection.EnforcementPercentage)
	case lbCfg.FailurePercentageEjection != nil && lbCfg.FailurePercentageEjection.Threshold > 100:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.FailurePercentageEjection.threshold = %v; must be <= 100", lbCfg.FailurePercentageEjection.Threshold)
	case lbCfg.FailurePercentageEjection != nil && lbCfg.FailurePercentageEjection.EnforcementPercentage > 100:
		return nil, fmt.Errorf("OutlierDetectionLoadBalancingConfig.FailurePercentageEjection.enforcement_percentage = %v; must be <= 100", lbCfg.FailurePercentageEjection.EnforcementPercentage)
	}
	return lbCfg, nil
}

func (bb) Name() string {
	return Name
}

// scUpdate wraps a subConn update to be sent to the child balancer.
type scUpdate struct {
	scw   *subConnWrapper
	state balancer.SubConnState
}

type ejectionUpdate struct {
	scw       *subConnWrapper
	isEjected bool // true for ejected, false for unejected
}

type lbCfgUpdate struct {
	lbCfg *LBConfig
	// to make sure picker is updated synchronously.
	done chan struct{}
}

type outlierDetectionBalancer struct {
	// These fields are safe to be accessed without holding any mutex because
	// they are synchronized in run(), which makes these field accesses happen
	// serially.
	//
	// childState is the latest balancer state received from the child.
	childState balancer.State
	// recentPickerNoop represents whether the most recent picker sent upward to
	// the balancer.ClientConn is a noop picker, which doesn't count RPC's. Used
	// to suppress redundant picker updates.
	recentPickerNoop bool

	closed         *grpcsync.Event
	done           *grpcsync.Event
	cc             balancer.ClientConn
	logger         *grpclog.PrefixLogger
	channelzParent channelz.Identifier

	// childMu guards calls into child (to uphold the balancer.Balancer API
	// guarantee of synchronous calls).
	childMu sync.Mutex
	child   *gracefulswitch.Balancer

	// mu guards access to the following fields. It also helps to synchronize
	// behaviors of the following events: config updates, firing of the interval
	// timer, SubConn State updates, SubConn address updates, and child state
	// updates.
	//
	// For example, when we receive a config update in the middle of the
	// interval timer algorithm, which uses knobs present in the config, the
	// balancer will wait for the interval timer algorithm to finish before
	// persisting the new configuration.
	//
	// Another example would be the updating of the addrs map, such as from a
	// SubConn address update in the middle of the interval timer algorithm
	// which uses addrs. This balancer waits for the interval timer algorithm to
	// finish before making the update to the addrs map.
	//
	// This mutex is never held at the same time as childMu (within the context
	// of a single goroutine).
	mu                    sync.Mutex
	addrs                 map[string]*addressInfo
	cfg                   *LBConfig
	scWrappers            map[balancer.SubConn]*subConnWrapper
	timerStartTime        time.Time
	intervalTimer         *time.Timer
	inhibitPickerUpdates  bool
	updateUnconditionally bool
	numAddrsEjected       int // For fast calculations of percentage of addrs ejected

	scUpdateCh     *buffer.Unbounded
	pickerUpdateCh *buffer.Unbounded
}

// noopConfig returns whether this balancer is configured with a logical no-op
// configuration or not.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) noopConfig() bool {
	return b.cfg.SuccessRateEjection == nil && b.cfg.FailurePercentageEjection == nil
}

// onIntervalConfig handles logic required specifically on the receipt of a
// configuration which specifies to count RPC's and periodically perform passive
// health checking based on heuristics defined in configuration every configured
// interval.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) onIntervalConfig() {
	var interval time.Duration
	if b.timerStartTime.IsZero() {
		b.timerStartTime = time.Now()
		for _, addrInfo := range b.addrs {
			addrInfo.callCounter.clear()
		}
		interval = time.Duration(b.cfg.Interval)
	} else {
		interval = time.Duration(b.cfg.Interval) - now().Sub(b.timerStartTime)
		if interval < 0 {
			interval = 0
		}
	}
	b.intervalTimer = afterFunc(interval, b.intervalTimerAlgorithm)
}

// onNoopConfig handles logic required specifically on the receipt of a
// configuration which specifies the balancer to be a noop.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) onNoopConfig() {
	// "If a config is provided with both the `success_rate_ejection` and
	// `failure_percentage_ejection` fields unset, skip starting the timer and
	// do the following:"
	// "Unset the timer start timestamp."
	b.timerStartTime = time.Time{}
	for _, addrInfo := range b.addrs {
		// "Uneject all currently ejected addresses."
		if !addrInfo.latestEjectionTimestamp.IsZero() {
			b.unejectAddress(addrInfo)
		}
		// "Reset each address's ejection time multiplier to 0."
		addrInfo.ejectionTimeMultiplier = 0
	}
}

func (b *outlierDetectionBalancer) UpdateClientConnState(s balancer.ClientConnState) error {
	lbCfg, ok := s.BalancerConfig.(*LBConfig)
	if !ok {
		b.logger.Errorf("received config with unexpected type %T: %v", s.BalancerConfig, s.BalancerConfig)
		return balancer.ErrBadResolverState
	}

	// Reject whole config if child policy doesn't exist, don't persist it for
	// later.
	bb := balancer.Get(lbCfg.ChildPolicy.Name)
	if bb == nil {
		return fmt.Errorf("outlier detection: child balancer %q not registered", lbCfg.ChildPolicy.Name)
	}

	// It is safe to read b.cfg here without holding the mutex, as the only
	// write to b.cfg happens later in this function. This function is part of
	// the balancer.Balancer API, so it is guaranteed to be called in a
	// synchronous manner, so it cannot race with this read.
	if b.cfg == nil || b.cfg.ChildPolicy.Name != lbCfg.ChildPolicy.Name {
		b.childMu.Lock()
		err := b.child.SwitchTo(bb)
		if err != nil {
			b.childMu.Unlock()
			return fmt.Errorf("outlier detection: error switching to child of type %q: %v", lbCfg.ChildPolicy.Name, err)
		}
		b.childMu.Unlock()
	}

	b.mu.Lock()
	// Inhibit child picker updates until this UpdateClientConnState() call
	// completes. If needed, a picker update containing the no-op config bit
	// determined from this config and most recent state from the child will be
	// sent synchronously upward at the end of this UpdateClientConnState()
	// call.
	b.inhibitPickerUpdates = true
	b.updateUnconditionally = false
	b.cfg = lbCfg

	addrs := make(map[string]bool, len(s.ResolverState.Addresses))
	for _, addr := range s.ResolverState.Addresses {
		addrs[addr.Addr] = true
		if _, ok := b.addrs[addr.Addr]; !ok {
			b.addrs[addr.Addr] = newAddressInfo()
		}
	}
	for addr := range b.addrs {
		if !addrs[addr] {
			delete(b.addrs, addr)
		}
	}

	if b.intervalTimer != nil {
		b.intervalTimer.Stop()
	}

	if b.noopConfig() {
		b.onNoopConfig()
	} else {
		b.onIntervalConfig()
	}
	b.mu.Unlock()

	b.childMu.Lock()
	err := b.child.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  s.ResolverState,
		BalancerConfig: b.cfg.ChildPolicy.Config,
	})
	b.childMu.Unlock()

	done := make(chan struct{})
	b.pickerUpdateCh.Put(lbCfgUpdate{
		lbCfg: lbCfg,
		done:  done,
	})
	<-done

	return err
}

func (b *outlierDetectionBalancer) ResolverError(err error) {
	b.childMu.Lock()
	defer b.childMu.Unlock()
	b.child.ResolverError(err)
}

func (b *outlierDetectionBalancer) updateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	b.mu.Lock()
	defer b.mu.Unlock()
	scw, ok := b.scWrappers[sc]
	if !ok {
		// Shouldn't happen if passed down a SubConnWrapper to child on SubConn
		// creation.
		b.logger.Errorf("UpdateSubConnState called with SubConn that has no corresponding SubConnWrapper")
		return
	}
	if state.ConnectivityState == connectivity.Shutdown {
		delete(b.scWrappers, scw.SubConn)
	}
	b.scUpdateCh.Put(&scUpdate{
		scw:   scw,
		state: state,
	})
}

func (b *outlierDetectionBalancer) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	b.logger.Errorf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state)
}

func (b *outlierDetectionBalancer) Close() {
	b.closed.Fire()
	<-b.done.Done()
	b.childMu.Lock()
	b.child.Close()
	b.childMu.Unlock()

	b.scUpdateCh.Close()
	b.pickerUpdateCh.Close()

	b.mu.Lock()
	defer b.mu.Unlock()
	if b.intervalTimer != nil {
		b.intervalTimer.Stop()
	}
}

func (b *outlierDetectionBalancer) ExitIdle() {
	b.childMu.Lock()
	defer b.childMu.Unlock()
	b.child.ExitIdle()
}

// wrappedPicker delegates to the child policy's picker, and when the request
// finishes, it increments the corresponding counter in the map entry referenced
// by the subConnWrapper that was picked. If both the `success_rate_ejection`
// and `failure_percentage_ejection` fields are unset in the configuration, this
// picker will not count.
type wrappedPicker struct {
	childPicker balancer.Picker
	noopPicker  bool
}

func (wp *wrappedPicker) Pick(info balancer.PickInfo) (balancer.PickResult, error) {
	pr, err := wp.childPicker.Pick(info)
	if err != nil {
		return balancer.PickResult{}, err
	}

	done := func(di balancer.DoneInfo) {
		if !wp.noopPicker {
			incrementCounter(pr.SubConn, di)
		}
		if pr.Done != nil {
			pr.Done(di)
		}
	}
	scw, ok := pr.SubConn.(*subConnWrapper)
	if !ok {
		// This can never happen, but check is present for defensive
		// programming.
		logger.Errorf("Picked SubConn from child picker is not a SubConnWrapper")
		return balancer.PickResult{
			SubConn:  pr.SubConn,
			Done:     done,
			Metadata: pr.Metadata,
		}, nil
	}
	return balancer.PickResult{
		SubConn:  scw.SubConn,
		Done:     done,
		Metadata: pr.Metadata,
	}, nil
}

func incrementCounter(sc balancer.SubConn, info balancer.DoneInfo) {
	scw, ok := sc.(*subConnWrapper)
	if !ok {
		// Shouldn't happen, as comes from child
		return
	}

	// scw.addressInfo and callCounter.activeBucket can be written to
	// concurrently (the pointers themselves). Thus, protect the reads here with
	// atomics to prevent data corruption. There exists a race in which you read
	// the addressInfo or active bucket pointer and then that pointer points to
	// deprecated memory. If this goroutine yields the processor, in between
	// reading the addressInfo pointer and writing to the active bucket,
	// UpdateAddresses can switch the addressInfo the scw points to. Writing to
	// an outdated addresses is a very small race and tolerable. After reading
	// callCounter.activeBucket in this picker a swap call can concurrently
	// change what activeBucket points to. A50 says to swap the pointer, which
	// will cause this race to write to deprecated memory the interval timer
	// algorithm will never read, which makes this race alright.
	addrInfo := (*addressInfo)(atomic.LoadPointer(&scw.addressInfo))
	if addrInfo == nil {
		return
	}
	ab := (*bucket)(atomic.LoadPointer(&addrInfo.callCounter.activeBucket))

	if info.Err == nil {
		atomic.AddUint32(&ab.numSuccesses, 1)
	} else {
		atomic.AddUint32(&ab.numFailures, 1)
	}
}

func (b *outlierDetectionBalancer) UpdateState(s balancer.State) {
	b.pickerUpdateCh.Put(s)
}

func (b *outlierDetectionBalancer) NewSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
	var sc balancer.SubConn
	oldListener := opts.StateListener
	opts.StateListener = func(state balancer.SubConnState) { b.updateSubConnState(sc, state) }
	sc, err := b.cc.NewSubConn(addrs, opts)
	if err != nil {
		return nil, err
	}
	scw := &subConnWrapper{
		SubConn:    sc,
		addresses:  addrs,
		scUpdateCh: b.scUpdateCh,
		listener:   oldListener,
	}
	b.mu.Lock()
	defer b.mu.Unlock()
	b.scWrappers[sc] = scw
	if len(addrs) != 1 {
		return scw, nil
	}
	addrInfo, ok := b.addrs[addrs[0].Addr]
	if !ok {
		return scw, nil
	}
	addrInfo.sws = append(addrInfo.sws, scw)
	atomic.StorePointer(&scw.addressInfo, unsafe.Pointer(addrInfo))
	if !addrInfo.latestEjectionTimestamp.IsZero() {
		scw.eject()
	}
	return scw, nil
}

func (b *outlierDetectionBalancer) RemoveSubConn(sc balancer.SubConn) {
	b.logger.Errorf("RemoveSubConn(%v) called unexpectedly", sc)
}

// appendIfPresent appends the scw to the address, if the address is present in
// the Outlier Detection balancers address map. Returns nil if not present, and
// the map entry if present.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) appendIfPresent(addr string, scw *subConnWrapper) *addressInfo {
	addrInfo, ok := b.addrs[addr]
	if !ok {
		return nil
	}

	addrInfo.sws = append(addrInfo.sws, scw)
	atomic.StorePointer(&scw.addressInfo, unsafe.Pointer(addrInfo))
	return addrInfo
}

// removeSubConnFromAddressesMapEntry removes the scw from its map entry if
// present.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) removeSubConnFromAddressesMapEntry(scw *subConnWrapper) {
	addrInfo := (*addressInfo)(atomic.LoadPointer(&scw.addressInfo))
	if addrInfo == nil {
		return
	}
	for i, sw := range addrInfo.sws {
		if scw == sw {
			addrInfo.sws = append(addrInfo.sws[:i], addrInfo.sws[i+1:]...)
			return
		}
	}
}

func (b *outlierDetectionBalancer) UpdateAddresses(sc balancer.SubConn, addrs []resolver.Address) {
	scw, ok := sc.(*subConnWrapper)
	if !ok {
		// Return, shouldn't happen if passed up scw
		return
	}

	b.cc.UpdateAddresses(scw.SubConn, addrs)
	b.mu.Lock()
	defer b.mu.Unlock()

	// Note that 0 addresses is a valid update/state for a SubConn to be in.
	// This is correctly handled by this algorithm (handled as part of a non singular
	// old address/new address).
	switch {
	case len(scw.addresses) == 1 && len(addrs) == 1: // single address to single address
		// If the updated address is the same, then there is nothing to do
		// past this point.
		if scw.addresses[0].Addr == addrs[0].Addr {
			return
		}
		b.removeSubConnFromAddressesMapEntry(scw)
		addrInfo := b.appendIfPresent(addrs[0].Addr, scw)
		if addrInfo == nil { // uneject unconditionally because could have come from an ejected address
			scw.uneject()
			break
		}
		if addrInfo.latestEjectionTimestamp.IsZero() { // relay new updated subconn state
			scw.uneject()
		} else {
			scw.eject()
		}
	case len(scw.addresses) == 1: // single address to multiple/no addresses
		b.removeSubConnFromAddressesMapEntry(scw)
		addrInfo := (*addressInfo)(atomic.LoadPointer(&scw.addressInfo))
		if addrInfo != nil {
			addrInfo.callCounter.clear()
		}
		scw.uneject()
	case len(addrs) == 1: // multiple/no addresses to single address
		addrInfo := b.appendIfPresent(addrs[0].Addr, scw)
		if addrInfo != nil && !addrInfo.latestEjectionTimestamp.IsZero() {
			scw.eject()
		}
	} // otherwise multiple/no addresses to multiple/no addresses; ignore

	scw.addresses = addrs
}

func (b *outlierDetectionBalancer) ResolveNow(opts resolver.ResolveNowOptions) {
	b.cc.ResolveNow(opts)
}

func (b *outlierDetectionBalancer) Target() string {
	return b.cc.Target()
}

// handleSubConnUpdate stores the recent state and forward the update
// if the SubConn is not ejected.
func (b *outlierDetectionBalancer) handleSubConnUpdate(u *scUpdate) {
	scw := u.scw
	scw.latestState = u.state
	if !scw.ejected {
		if scw.listener != nil {
			b.childMu.Lock()
			scw.listener(u.state)
			b.childMu.Unlock()
		}
	}
}

// handleEjectedUpdate handles any SubConns that get ejected/unejected, and
// forwards the appropriate corresponding subConnState to the child policy.
func (b *outlierDetectionBalancer) handleEjectedUpdate(u *ejectionUpdate) {
	scw := u.scw
	scw.ejected = u.isEjected
	// If scw.latestState has never been written to will default to connectivity
	// IDLE, which is fine.
	stateToUpdate := scw.latestState
	if u.isEjected {
		stateToUpdate = balancer.SubConnState{
			ConnectivityState: connectivity.TransientFailure,
		}
	}
	if scw.listener != nil {
		b.childMu.Lock()
		scw.listener(stateToUpdate)
		b.childMu.Unlock()
	}
}

// handleChildStateUpdate forwards the picker update wrapped in a wrapped picker
// with the noop picker bit present.
func (b *outlierDetectionBalancer) handleChildStateUpdate(u balancer.State) {
	b.childState = u
	b.mu.Lock()
	if b.inhibitPickerUpdates {
		// If a child's state is updated during the suppression of child
		// updates, the synchronous handleLBConfigUpdate function with respect
		// to UpdateClientConnState should return a picker unconditionally.
		b.updateUnconditionally = true
		b.mu.Unlock()
		return
	}
	noopCfg := b.noopConfig()
	b.mu.Unlock()
	b.recentPickerNoop = noopCfg
	b.cc.UpdateState(balancer.State{
		ConnectivityState: b.childState.ConnectivityState,
		Picker: &wrappedPicker{
			childPicker: b.childState.Picker,
			noopPicker:  noopCfg,
		},
	})
}

// handleLBConfigUpdate compares whether the new config is a noop config or not,
// to the noop bit in the picker if present. It updates the picker if this bit
// changed compared to the picker currently in use.
func (b *outlierDetectionBalancer) handleLBConfigUpdate(u lbCfgUpdate) {
	lbCfg := u.lbCfg
	noopCfg := lbCfg.SuccessRateEjection == nil && lbCfg.FailurePercentageEjection == nil
	// If the child has sent it's first update and this config flips the noop
	// bit compared to the most recent picker update sent upward, then a new
	// picker with this updated bit needs to be forwarded upward. If a child
	// update was received during the suppression of child updates within
	// UpdateClientConnState(), then a new picker needs to be forwarded with
	// this updated state, irregardless of whether this new configuration flips
	// the bit.
	if b.childState.Picker != nil && noopCfg != b.recentPickerNoop || b.updateUnconditionally {
		b.recentPickerNoop = noopCfg
		b.cc.UpdateState(balancer.State{
			ConnectivityState: b.childState.ConnectivityState,
			Picker: &wrappedPicker{
				childPicker: b.childState.Picker,
				noopPicker:  noopCfg,
			},
		})
	}
	b.inhibitPickerUpdates = false
	b.updateUnconditionally = false
	close(u.done)
}

func (b *outlierDetectionBalancer) run() {
	defer b.done.Fire()
	for {
		select {
		case update, ok := <-b.scUpdateCh.Get():
			if !ok {
				return
			}
			b.scUpdateCh.Load()
			if b.closed.HasFired() { // don't send SubConn updates to child after the balancer has been closed
				return
			}
			switch u := update.(type) {
			case *scUpdate:
				b.handleSubConnUpdate(u)
			case *ejectionUpdate:
				b.handleEjectedUpdate(u)
			}
		case update, ok := <-b.pickerUpdateCh.Get():
			if !ok {
				return
			}
			b.pickerUpdateCh.Load()
			if b.closed.HasFired() { // don't send picker updates to grpc after the balancer has been closed
				return
			}
			switch u := update.(type) {
			case balancer.State:
				b.handleChildStateUpdate(u)
			case lbCfgUpdate:
				b.handleLBConfigUpdate(u)
			}
		case <-b.closed.Done():
			return
		}
	}
}

// intervalTimerAlgorithm ejects and unejects addresses based on the Outlier
// Detection configuration and data about each address from the previous
// interval.
func (b *outlierDetectionBalancer) intervalTimerAlgorithm() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.timerStartTime = time.Now()

	for _, addrInfo := range b.addrs {
		addrInfo.callCounter.swap()
	}

	if b.cfg.SuccessRateEjection != nil {
		b.successRateAlgorithm()
	}

	if b.cfg.FailurePercentageEjection != nil {
		b.failurePercentageAlgorithm()
	}

	for _, addrInfo := range b.addrs {
		if addrInfo.latestEjectionTimestamp.IsZero() && addrInfo.ejectionTimeMultiplier > 0 {
			addrInfo.ejectionTimeMultiplier--
			continue
		}
		if addrInfo.latestEjectionTimestamp.IsZero() {
			// Address is already not ejected, so no need to check for whether
			// to uneject the address below.
			continue
		}
		et := time.Duration(b.cfg.BaseEjectionTime) * time.Duration(addrInfo.ejectionTimeMultiplier)
		met := max(time.Duration(b.cfg.BaseEjectionTime), time.Duration(b.cfg.MaxEjectionTime))
		uet := addrInfo.latestEjectionTimestamp.Add(min(et, met))
		if now().After(uet) {
			b.unejectAddress(addrInfo)
		}
	}

	// This conditional only for testing (since the interval timer algorithm is
	// called manually), will never hit in production.
	if b.intervalTimer != nil {
		b.intervalTimer.Stop()
	}
	b.intervalTimer = afterFunc(time.Duration(b.cfg.Interval), b.intervalTimerAlgorithm)
}

// addrsWithAtLeastRequestVolume returns a slice of address information of all
// addresses with at least request volume passed in.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) addrsWithAtLeastRequestVolume(requestVolume uint32) []*addressInfo {
	var addrs []*addressInfo
	for _, addrInfo := range b.addrs {
		bucket := addrInfo.callCounter.inactiveBucket
		rv := bucket.numSuccesses + bucket.numFailures
		if rv >= requestVolume {
			addrs = append(addrs, addrInfo)
		}
	}
	return addrs
}

// meanAndStdDev returns the mean and std dev of the fractions of successful
// requests of the addresses passed in.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) meanAndStdDev(addrs []*addressInfo) (float64, float64) {
	var totalFractionOfSuccessfulRequests float64
	var mean float64
	for _, addrInfo := range addrs {
		bucket := addrInfo.callCounter.inactiveBucket
		rv := bucket.numSuccesses + bucket.numFailures
		totalFractionOfSuccessfulRequests += float64(bucket.numSuccesses) / float64(rv)
	}
	mean = totalFractionOfSuccessfulRequests / float64(len(addrs))
	var sumOfSquares float64
	for _, addrInfo := range addrs {
		bucket := addrInfo.callCounter.inactiveBucket
		rv := bucket.numSuccesses + bucket.numFailures
		devFromMean := (float64(bucket.numSuccesses) / float64(rv)) - mean
		sumOfSquares += devFromMean * devFromMean
	}
	variance := sumOfSquares / float64(len(addrs))
	return mean, math.Sqrt(variance)
}

// successRateAlgorithm ejects any addresses where the success rate falls below
// the other addresses according to mean and standard deviation, and if overall
// applicable from other set heuristics.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) successRateAlgorithm() {
	addrsToConsider := b.addrsWithAtLeastRequestVolume(b.cfg.SuccessRateEjection.RequestVolume)
	if len(addrsToConsider) < int(b.cfg.SuccessRateEjection.MinimumHosts) {
		return
	}
	mean, stddev := b.meanAndStdDev(addrsToConsider)
	for _, addrInfo := range addrsToConsider {
		bucket := addrInfo.callCounter.inactiveBucket
		ejectionCfg := b.cfg.SuccessRateEjection
		if float64(b.numAddrsEjected)/float64(len(b.addrs))*100 >= float64(b.cfg.MaxEjectionPercent) {
			return
		}
		successRate := float64(bucket.numSuccesses) / float64(bucket.numSuccesses+bucket.numFailures)
		requiredSuccessRate := mean - stddev*(float64(ejectionCfg.StdevFactor)/1000)
		if successRate < requiredSuccessRate {
			channelz.Infof(logger, b.channelzParent, "SuccessRate algorithm detected outlier: %s. Parameters: successRate=%f, mean=%f, stddev=%f, requiredSuccessRate=%f", addrInfo, successRate, mean, stddev, requiredSuccessRate)
			if uint32(rand.Int31n(100)) < ejectionCfg.EnforcementPercentage {
				b.ejectAddress(addrInfo)
			}
		}
	}
}

// failurePercentageAlgorithm ejects any addresses where the failure percentage
// rate exceeds a set enforcement percentage, if overall applicable from other
// set heuristics.
//
// Caller must hold b.mu.
func (b *outlierDetectionBalancer) failurePercentageAlgorithm() {
	addrsToConsider := b.addrsWithAtLeastRequestVolume(b.cfg.FailurePercentageEjection.RequestVolume)
	if len(addrsToConsider) < int(b.cfg.FailurePercentageEjection.MinimumHosts) {
		return
	}

	for _, addrInfo := range addrsToConsider {
		bucket := addrInfo.callCounter.inactiveBucket
		ejectionCfg := b.cfg.FailurePercentageEjection
		if float64(b.numAddrsEjected)/float64(len(b.addrs))*100 >= float64(b.cfg.MaxEjectionPercent) {
			return
		}
		failurePercentage := (float64(bucket.numFailures) / float64(bucket.numSuccesses+bucket.numFailures)) * 100
		if failurePercentage > float64(b.cfg.FailurePercentageEjection.Threshold) {
			channelz.Infof(logger, b.channelzParent, "FailurePercentage algorithm detected outlier: %s, failurePercentage=%f", addrInfo, failurePercentage)
			if uint32(rand.Int31n(100)) < ejectionCfg.EnforcementPercentage {
				b.ejectAddress(addrInfo)
			}
		}
	}
}

// Caller must hold b.mu.
func (b *outlierDetectionBalancer) ejectAddress(addrInfo *addressInfo) {
	b.numAddrsEjected++
	addrInfo.latestEjectionTimestamp = b.timerStartTime
	addrInfo.ejectionTimeMultiplier++
	for _, sbw := range addrInfo.sws {
		sbw.eject()
		channelz.Infof(logger, b.channelzParent, "Subchannel ejected: %s", sbw)
	}

}

// Caller must hold b.mu.
func (b *outlierDetectionBalancer) unejectAddress(addrInfo *addressInfo) {
	b.numAddrsEjected--
	addrInfo.latestEjectionTimestamp = time.Time{}
	for _, sbw := range addrInfo.sws {
		sbw.uneject()
		channelz.Infof(logger, b.channelzParent, "Subchannel unejected: %s", sbw)
	}
}

// addressInfo contains the runtime information about an address that pertains
// to Outlier Detection. This struct and all of its fields is protected by
// outlierDetectionBalancer.mu in the case where it is accessed through the
// address map. In the case of Picker callbacks, the writes to the activeBucket
// of callCounter are protected by atomically loading and storing
// unsafe.Pointers (see further explanation in incrementCounter()).
type addressInfo struct {
	// The call result counter object.
	callCounter *callCounter

	// The latest ejection timestamp, or zero if the address is currently not
	// ejected.
	latestEjectionTimestamp time.Time

	// The current ejection time multiplier, starting at 0.
	ejectionTimeMultiplier int64

	// A list of subchannel wrapper objects that correspond to this address.
	sws []*subConnWrapper
}

func (a *addressInfo) String() string {
	var res strings.Builder
	res.WriteString("[")
	for _, sw := range a.sws {
		res.WriteString(sw.String())
	}
	res.WriteString("]")
	return res.String()
}

func newAddressInfo() *addressInfo {
	return &addressInfo{
		callCounter: newCallCounter(),
	}
}
