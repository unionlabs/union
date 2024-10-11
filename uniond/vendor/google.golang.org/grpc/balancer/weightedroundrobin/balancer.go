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

package weightedroundrobin

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"math/rand"
	"sync"
	"sync/atomic"
	"time"
	"unsafe"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/base"
	"google.golang.org/grpc/balancer/weightedroundrobin/internal"
	"google.golang.org/grpc/balancer/weightedtarget"
	"google.golang.org/grpc/connectivity"
	estats "google.golang.org/grpc/experimental/stats"
	"google.golang.org/grpc/internal/grpclog"
	iserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/orca"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"

	v3orcapb "github.com/cncf/xds/go/xds/data/orca/v3"
)

// Name is the name of the weighted round robin balancer.
const Name = "weighted_round_robin"

var (
	rrFallbackMetric = estats.RegisterInt64Count(estats.MetricDescriptor{
		Name:           "grpc.lb.wrr.rr_fallback",
		Description:    "EXPERIMENTAL. Number of scheduler updates in which there were not enough endpoints with valid weight, which caused the WRR policy to fall back to RR behavior.",
		Unit:           "update",
		Labels:         []string{"grpc.target"},
		OptionalLabels: []string{"grpc.lb.locality"},
		Default:        false,
	})

	endpointWeightNotYetUsableMetric = estats.RegisterInt64Count(estats.MetricDescriptor{
		Name:           "grpc.lb.wrr.endpoint_weight_not_yet_usable",
		Description:    "EXPERIMENTAL. Number of endpoints from each scheduler update that don't yet have usable weight information (i.e., either the load report has not yet been received, or it is within the blackout period).",
		Unit:           "endpoint",
		Labels:         []string{"grpc.target"},
		OptionalLabels: []string{"grpc.lb.locality"},
		Default:        false,
	})

	endpointWeightStaleMetric = estats.RegisterInt64Count(estats.MetricDescriptor{
		Name:           "grpc.lb.wrr.endpoint_weight_stale",
		Description:    "EXPERIMENTAL. Number of endpoints from each scheduler update whose latest weight is older than the expiration period.",
		Unit:           "endpoint",
		Labels:         []string{"grpc.target"},
		OptionalLabels: []string{"grpc.lb.locality"},
		Default:        false,
	})
	endpointWeightsMetric = estats.RegisterFloat64Histo(estats.MetricDescriptor{
		Name:           "grpc.lb.wrr.endpoint_weights",
		Description:    "EXPERIMENTAL. Weight of each endpoint, recorded on every scheduler update. Endpoints without usable weights will be recorded as weight 0.",
		Unit:           "endpoint",
		Labels:         []string{"grpc.target"},
		OptionalLabels: []string{"grpc.lb.locality"},
		Default:        false,
	})
)

func init() {
	balancer.Register(bb{})
}

type bb struct{}

func (bb) Build(cc balancer.ClientConn, bOpts balancer.BuildOptions) balancer.Balancer {
	b := &wrrBalancer{
		cc:                cc,
		subConns:          resolver.NewAddressMap(),
		csEvltr:           &balancer.ConnectivityStateEvaluator{},
		scMap:             make(map[balancer.SubConn]*weightedSubConn),
		connectivityState: connectivity.Connecting,
		target:            bOpts.Target.String(),
		metricsRecorder:   bOpts.MetricsRecorder,
	}

	b.logger = prefixLogger(b)
	b.logger.Infof("Created")
	return b
}

func (bb) ParseConfig(js json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	lbCfg := &lbConfig{
		// Default values as documented in A58.
		OOBReportingPeriod:      iserviceconfig.Duration(10 * time.Second),
		BlackoutPeriod:          iserviceconfig.Duration(10 * time.Second),
		WeightExpirationPeriod:  iserviceconfig.Duration(3 * time.Minute),
		WeightUpdatePeriod:      iserviceconfig.Duration(time.Second),
		ErrorUtilizationPenalty: 1,
	}
	if err := json.Unmarshal(js, lbCfg); err != nil {
		return nil, fmt.Errorf("wrr: unable to unmarshal LB policy config: %s, error: %v", string(js), err)
	}

	if lbCfg.ErrorUtilizationPenalty < 0 {
		return nil, fmt.Errorf("wrr: errorUtilizationPenalty must be non-negative")
	}

	// For easier comparisons later, ensure the OOB reporting period is unset
	// (0s) when OOB reports are disabled.
	if !lbCfg.EnableOOBLoadReport {
		lbCfg.OOBReportingPeriod = 0
	}

	// Impose lower bound of 100ms on weightUpdatePeriod.
	if !internal.AllowAnyWeightUpdatePeriod && lbCfg.WeightUpdatePeriod < iserviceconfig.Duration(100*time.Millisecond) {
		lbCfg.WeightUpdatePeriod = iserviceconfig.Duration(100 * time.Millisecond)
	}

	return lbCfg, nil
}

func (bb) Name() string {
	return Name
}

// wrrBalancer implements the weighted round robin LB policy.
type wrrBalancer struct {
	// The following fields are immutable.
	cc              balancer.ClientConn
	logger          *grpclog.PrefixLogger
	target          string
	metricsRecorder estats.MetricsRecorder

	// The following fields are only accessed on calls into the LB policy, and
	// do not need a mutex.
	cfg               *lbConfig            // active config
	subConns          *resolver.AddressMap // active weightedSubConns mapped by address
	scMap             map[balancer.SubConn]*weightedSubConn
	connectivityState connectivity.State // aggregate state
	csEvltr           *balancer.ConnectivityStateEvaluator
	resolverErr       error // the last error reported by the resolver; cleared on successful resolution
	connErr           error // the last connection error; cleared upon leaving TransientFailure
	stopPicker        func()
	locality          string
}

func (b *wrrBalancer) UpdateClientConnState(ccs balancer.ClientConnState) error {
	b.logger.Infof("UpdateCCS: %v", ccs)
	b.resolverErr = nil
	cfg, ok := ccs.BalancerConfig.(*lbConfig)
	if !ok {
		return fmt.Errorf("wrr: received nil or illegal BalancerConfig (type %T): %v", ccs.BalancerConfig, ccs.BalancerConfig)
	}

	b.cfg = cfg
	b.locality = weightedtarget.LocalityFromResolverState(ccs.ResolverState)
	b.updateAddresses(ccs.ResolverState.Addresses)

	if len(ccs.ResolverState.Addresses) == 0 {
		b.ResolverError(errors.New("resolver produced zero addresses")) // will call regeneratePicker
		return balancer.ErrBadResolverState
	}

	b.regeneratePicker()

	return nil
}

func (b *wrrBalancer) updateAddresses(addrs []resolver.Address) {
	addrsSet := resolver.NewAddressMap()

	// Loop through new address list and create subconns for any new addresses.
	for _, addr := range addrs {
		if _, ok := addrsSet.Get(addr); ok {
			// Redundant address; skip.
			continue
		}
		addrsSet.Set(addr, nil)

		var wsc *weightedSubConn
		wsci, ok := b.subConns.Get(addr)
		if ok {
			wsc = wsci.(*weightedSubConn)
		} else {
			// addr is a new address (not existing in b.subConns).
			var sc balancer.SubConn
			sc, err := b.cc.NewSubConn([]resolver.Address{addr}, balancer.NewSubConnOptions{
				StateListener: func(state balancer.SubConnState) {
					b.updateSubConnState(sc, state)
				},
			})
			if err != nil {
				b.logger.Warningf("Failed to create new SubConn for address %v: %v", addr, err)
				continue
			}
			wsc = &weightedSubConn{
				SubConn:           sc,
				logger:            b.logger,
				connectivityState: connectivity.Idle,
				// Initially, we set load reports to off, because they are not
				// running upon initial weightedSubConn creation.
				cfg: &lbConfig{EnableOOBLoadReport: false},

				metricsRecorder: b.metricsRecorder,
				target:          b.target,
				locality:        b.locality,
			}
			b.subConns.Set(addr, wsc)
			b.scMap[sc] = wsc
			b.csEvltr.RecordTransition(connectivity.Shutdown, connectivity.Idle)
			sc.Connect()
		}
		// Update config for existing weightedSubConn or send update for first
		// time to new one.  Ensures an OOB listener is running if needed
		// (and stops the existing one if applicable).
		wsc.updateConfig(b.cfg)
	}

	// Loop through existing subconns and remove ones that are not in addrs.
	for _, addr := range b.subConns.Keys() {
		if _, ok := addrsSet.Get(addr); ok {
			// Existing address also in new address list; skip.
			continue
		}
		// addr was removed by resolver.  Remove.
		wsci, _ := b.subConns.Get(addr)
		wsc := wsci.(*weightedSubConn)
		wsc.SubConn.Shutdown()
		b.subConns.Delete(addr)
	}
}

func (b *wrrBalancer) ResolverError(err error) {
	b.resolverErr = err
	if b.subConns.Len() == 0 {
		b.connectivityState = connectivity.TransientFailure
	}
	if b.connectivityState != connectivity.TransientFailure {
		// No need to update the picker since no error is being returned.
		return
	}
	b.regeneratePicker()
}

func (b *wrrBalancer) UpdateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	b.logger.Errorf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, state)
}

func (b *wrrBalancer) updateSubConnState(sc balancer.SubConn, state balancer.SubConnState) {
	wsc := b.scMap[sc]
	if wsc == nil {
		b.logger.Errorf("UpdateSubConnState called with an unknown SubConn: %p, %v", sc, state)
		return
	}
	if b.logger.V(2) {
		logger.Infof("UpdateSubConnState(%+v, %+v)", sc, state)
	}

	cs := state.ConnectivityState

	if cs == connectivity.TransientFailure {
		// Save error to be reported via picker.
		b.connErr = state.ConnectionError
	}

	if cs == connectivity.Shutdown {
		delete(b.scMap, sc)
		// The subconn was removed from b.subConns when the address was removed
		// in updateAddresses.
	}

	oldCS := wsc.updateConnectivityState(cs)
	b.connectivityState = b.csEvltr.RecordTransition(oldCS, cs)

	// Regenerate picker when one of the following happens:
	//  - this sc entered or left ready
	//  - the aggregated state of balancer is TransientFailure
	//    (may need to update error message)
	if (cs == connectivity.Ready) != (oldCS == connectivity.Ready) ||
		b.connectivityState == connectivity.TransientFailure {
		b.regeneratePicker()
	}
}

// Close stops the balancer.  It cancels any ongoing scheduler updates and
// stops any ORCA listeners.
func (b *wrrBalancer) Close() {
	if b.stopPicker != nil {
		b.stopPicker()
		b.stopPicker = nil
	}
	for _, wsc := range b.scMap {
		// Ensure any lingering OOB watchers are stopped.
		wsc.updateConnectivityState(connectivity.Shutdown)
	}
}

// ExitIdle is ignored; we always connect to all backends.
func (b *wrrBalancer) ExitIdle() {}

func (b *wrrBalancer) readySubConns() []*weightedSubConn {
	var ret []*weightedSubConn
	for _, v := range b.subConns.Values() {
		wsc := v.(*weightedSubConn)
		if wsc.connectivityState == connectivity.Ready {
			ret = append(ret, wsc)
		}
	}
	return ret
}

// mergeErrors builds an error from the last connection error and the last
// resolver error.  Must only be called if b.connectivityState is
// TransientFailure.
func (b *wrrBalancer) mergeErrors() error {
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

func (b *wrrBalancer) regeneratePicker() {
	if b.stopPicker != nil {
		b.stopPicker()
		b.stopPicker = nil
	}

	switch b.connectivityState {
	case connectivity.TransientFailure:
		b.cc.UpdateState(balancer.State{
			ConnectivityState: connectivity.TransientFailure,
			Picker:            base.NewErrPicker(b.mergeErrors()),
		})
		return
	case connectivity.Connecting, connectivity.Idle:
		// Idle could happen very briefly if all subconns are Idle and we've
		// asked them to connect but they haven't reported Connecting yet.
		// Report the same as Connecting since this is temporary.
		b.cc.UpdateState(balancer.State{
			ConnectivityState: connectivity.Connecting,
			Picker:            base.NewErrPicker(balancer.ErrNoSubConnAvailable),
		})
		return
	case connectivity.Ready:
		b.connErr = nil
	}

	p := &picker{
		v:               rand.Uint32(), // start the scheduler at a random point
		cfg:             b.cfg,
		subConns:        b.readySubConns(),
		metricsRecorder: b.metricsRecorder,
		locality:        b.locality,
		target:          b.target,
	}
	var ctx context.Context
	ctx, b.stopPicker = context.WithCancel(context.Background())
	p.start(ctx)
	b.cc.UpdateState(balancer.State{
		ConnectivityState: b.connectivityState,
		Picker:            p,
	})
}

// picker is the WRR policy's picker.  It uses live-updating backend weights to
// update the scheduler periodically and ensure picks are routed proportional
// to those weights.
type picker struct {
	scheduler unsafe.Pointer     // *scheduler; accessed atomically
	v         uint32             // incrementing value used by the scheduler; accessed atomically
	cfg       *lbConfig          // active config when picker created
	subConns  []*weightedSubConn // all READY subconns

	// The following fields are immutable.
	target          string
	locality        string
	metricsRecorder estats.MetricsRecorder
}

func (p *picker) scWeights(recordMetrics bool) []float64 {
	ws := make([]float64, len(p.subConns))
	now := internal.TimeNow()
	for i, wsc := range p.subConns {
		ws[i] = wsc.weight(now, time.Duration(p.cfg.WeightExpirationPeriod), time.Duration(p.cfg.BlackoutPeriod), recordMetrics)
	}

	return ws
}

func (p *picker) inc() uint32 {
	return atomic.AddUint32(&p.v, 1)
}

func (p *picker) regenerateScheduler() {
	s := p.newScheduler(true)
	atomic.StorePointer(&p.scheduler, unsafe.Pointer(&s))
}

func (p *picker) start(ctx context.Context) {
	p.regenerateScheduler()
	if len(p.subConns) == 1 {
		// No need to regenerate weights with only one backend.
		return
	}

	go func() {
		ticker := time.NewTicker(time.Duration(p.cfg.WeightUpdatePeriod))
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				p.regenerateScheduler()
			}
		}
	}()
}

func (p *picker) Pick(balancer.PickInfo) (balancer.PickResult, error) {
	// Read the scheduler atomically.  All scheduler operations are threadsafe,
	// and if the scheduler is replaced during this usage, we want to use the
	// scheduler that was live when the pick started.
	sched := *(*scheduler)(atomic.LoadPointer(&p.scheduler))

	pickedSC := p.subConns[sched.nextIndex()]
	pr := balancer.PickResult{SubConn: pickedSC.SubConn}
	if !p.cfg.EnableOOBLoadReport {
		pr.Done = func(info balancer.DoneInfo) {
			if load, ok := info.ServerLoad.(*v3orcapb.OrcaLoadReport); ok && load != nil {
				pickedSC.OnLoadReport(load)
			}
		}
	}
	return pr, nil
}

// weightedSubConn is the wrapper of a subconn that holds the subconn and its
// weight (and other parameters relevant to computing the effective weight).
// When needed, it also tracks connectivity state, listens for metrics updates
// by implementing the orca.OOBListener interface and manages that listener.
type weightedSubConn struct {
	// The following fields are immutable.
	balancer.SubConn
	logger          *grpclog.PrefixLogger
	target          string
	metricsRecorder estats.MetricsRecorder
	locality        string

	// The following fields are only accessed on calls into the LB policy, and
	// do not need a mutex.
	connectivityState connectivity.State
	stopORCAListener  func()

	// The following fields are accessed asynchronously and are protected by
	// mu.  Note that mu may not be held when calling into the stopORCAListener
	// or when registering a new listener, as those calls require the ORCA
	// producer mu which is held when calling the listener, and the listener
	// holds mu.
	mu            sync.Mutex
	weightVal     float64
	nonEmptySince time.Time
	lastUpdated   time.Time
	cfg           *lbConfig
}

func (w *weightedSubConn) OnLoadReport(load *v3orcapb.OrcaLoadReport) {
	if w.logger.V(2) {
		w.logger.Infof("Received load report for subchannel %v: %v", w.SubConn, load)
	}
	// Update weights of this subchannel according to the reported load
	utilization := load.ApplicationUtilization
	if utilization == 0 {
		utilization = load.CpuUtilization
	}
	if utilization == 0 || load.RpsFractional == 0 {
		if w.logger.V(2) {
			w.logger.Infof("Ignoring empty load report for subchannel %v", w.SubConn)
		}
		return
	}

	w.mu.Lock()
	defer w.mu.Unlock()

	errorRate := load.Eps / load.RpsFractional
	w.weightVal = load.RpsFractional / (utilization + errorRate*w.cfg.ErrorUtilizationPenalty)
	if w.logger.V(2) {
		w.logger.Infof("New weight for subchannel %v: %v", w.SubConn, w.weightVal)
	}

	w.lastUpdated = internal.TimeNow()
	if w.nonEmptySince.Equal(time.Time{}) {
		w.nonEmptySince = w.lastUpdated
	}
}

// updateConfig updates the parameters of the WRR policy and
// stops/starts/restarts the ORCA OOB listener.
func (w *weightedSubConn) updateConfig(cfg *lbConfig) {
	w.mu.Lock()
	oldCfg := w.cfg
	w.cfg = cfg
	w.mu.Unlock()

	newPeriod := cfg.OOBReportingPeriod
	if cfg.EnableOOBLoadReport == oldCfg.EnableOOBLoadReport &&
		newPeriod == oldCfg.OOBReportingPeriod {
		// Load reporting wasn't enabled before or after, or load reporting was
		// enabled before and after, and had the same period.  (Note that with
		// load reporting disabled, OOBReportingPeriod is always 0.)
		return
	}
	// (Optionally stop and) start the listener to use the new config's
	// settings for OOB reporting.

	if w.stopORCAListener != nil {
		w.stopORCAListener()
	}
	if !cfg.EnableOOBLoadReport {
		w.stopORCAListener = nil
		return
	}
	if w.logger.V(2) {
		w.logger.Infof("Registering ORCA listener for %v with interval %v", w.SubConn, newPeriod)
	}
	opts := orca.OOBListenerOptions{ReportInterval: time.Duration(newPeriod)}
	w.stopORCAListener = orca.RegisterOOBListener(w.SubConn, w, opts)
}

func (w *weightedSubConn) updateConnectivityState(cs connectivity.State) connectivity.State {
	switch cs {
	case connectivity.Idle:
		// Always reconnect when idle.
		w.SubConn.Connect()
	case connectivity.Ready:
		// If we transition back to READY state, reset nonEmptySince so that we
		// apply the blackout period after we start receiving load data. Also
		// reset lastUpdated to trigger endpoint weight not yet usable in the
		// case endpoint gets asked what weight it is before receiving a new
		// load report. Note that we cannot guarantee that we will never receive
		// lingering callbacks for backend metric reports from the previous
		// connection after the new connection has been established, but they
		// should be masked by new backend metric reports from the new
		// connection by the time the blackout period ends.
		w.mu.Lock()
		w.nonEmptySince = time.Time{}
		w.lastUpdated = time.Time{}
		w.mu.Unlock()
	case connectivity.Shutdown:
		if w.stopORCAListener != nil {
			w.stopORCAListener()
		}
	}

	oldCS := w.connectivityState

	if oldCS == connectivity.TransientFailure &&
		(cs == connectivity.Connecting || cs == connectivity.Idle) {
		// Once a subconn enters TRANSIENT_FAILURE, ignore subsequent IDLE or
		// CONNECTING transitions to prevent the aggregated state from being
		// always CONNECTING when many backends exist but are all down.
		return oldCS
	}

	w.connectivityState = cs

	return oldCS
}

// weight returns the current effective weight of the subconn, taking into
// account the parameters.  Returns 0 for blacked out or expired data, which
// will cause the backend weight to be treated as the mean of the weights of the
// other backends. If forScheduler is set to true, this function will emit
// metrics through the metrics registry.
func (w *weightedSubConn) weight(now time.Time, weightExpirationPeriod, blackoutPeriod time.Duration, recordMetrics bool) (weight float64) {
	w.mu.Lock()
	defer w.mu.Unlock()

	if recordMetrics {
		defer func() {
			endpointWeightsMetric.Record(w.metricsRecorder, weight, w.target, w.locality)
		}()
	}

	// The SubConn has not received a load report (i.e. just turned READY with
	// no load report).
	if w.lastUpdated.Equal(time.Time{}) {
		endpointWeightNotYetUsableMetric.Record(w.metricsRecorder, 1, w.target, w.locality)
		return 0
	}

	// If the most recent update was longer ago than the expiration period,
	// reset nonEmptySince so that we apply the blackout period again if we
	// start getting data again in the future, and return 0.
	if now.Sub(w.lastUpdated) >= weightExpirationPeriod {
		if recordMetrics {
			endpointWeightStaleMetric.Record(w.metricsRecorder, 1, w.target, w.locality)
		}
		w.nonEmptySince = time.Time{}
		return 0
	}

	// If we don't have at least blackoutPeriod worth of data, return 0.
	if blackoutPeriod != 0 && (w.nonEmptySince.Equal(time.Time{}) || now.Sub(w.nonEmptySince) < blackoutPeriod) {
		if recordMetrics {
			endpointWeightNotYetUsableMetric.Record(w.metricsRecorder, 1, w.target, w.locality)
		}
		return 0
	}

	return w.weightVal
}
