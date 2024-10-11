// Copyright 2021 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

// Package msgrate allows estimating the throughput of peers for more balanced syncs.
package msgrate

import (
	"errors"
	"math"
	"sort"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/log"
)

// measurementImpact is the impact a single measurement has on a peer's final
// capacity value. A value closer to 0 reacts slower to sudden network changes,
// but it is also more stable against temporary hiccups. 0.1 worked well for
// most of Ethereum's existence, so might as well go with it.
const measurementImpact = 0.1

// capacityOverestimation is the ratio of items to over-estimate when retrieving
// a peer's capacity to avoid locking into a lower value due to never attempting
// to fetch more than some local stable value.
const capacityOverestimation = 1.01

// rttMinEstimate is the minimal round trip time to target requests for. Since
// every request entails a 2 way latency + bandwidth + serving database lookups,
// it should be generous enough to permit meaningful work to be done on top of
// the transmission costs.
const rttMinEstimate = 2 * time.Second

// rttMaxEstimate is the maximal round trip time to target requests for. Although
// the expectation is that a well connected node will never reach this, certain
// special connectivity ones might experience significant delays (e.g. satellite
// uplink with 3s RTT). This value should be low enough to forbid stalling the
// pipeline too long, but large enough to cover the worst of the worst links.
const rttMaxEstimate = 20 * time.Second

// rttPushdownFactor is a multiplier to attempt forcing quicker requests than
// what the message rate tracker estimates. The reason is that message rate
// tracking adapts queries to the RTT, but multiple RTT values can be perfectly
// valid, they just result in higher packet sizes. Since smaller packets almost
// always result in stabler download streams, this factor hones in on the lowest
// RTT from all the functional ones.
const rttPushdownFactor = 0.9

// rttMinConfidence is the minimum value the roundtrip confidence factor may drop
// to. Since the target timeouts are based on how confident the tracker is in the
// true roundtrip, it's important to not allow too huge fluctuations.
const rttMinConfidence = 0.1

// ttlScaling is the multiplier that converts the estimated roundtrip time to a
// timeout cap for network requests. The expectation is that peers' response time
// will fluctuate around the estimated roundtrip, but depending in their load at
// request time, it might be higher than anticipated. This scaling factor ensures
// that we allow remote connections some slack but at the same time do enforce a
// behavior similar to our median peers.
const ttlScaling = 3

// ttlLimit is the maximum timeout allowance to prevent reaching crazy numbers
// if some unforeseen network events happen. As much as we try to hone in on
// the most optimal values, it doesn't make any sense to go above a threshold,
// even if everything is slow and screwy.
const ttlLimit = time.Minute

// tuningConfidenceCap is the number of active peers above which to stop detuning
// the confidence number. The idea here is that once we hone in on the capacity
// of a meaningful number of peers, adding one more should ot have a significant
// impact on things, so just ron with the originals.
const tuningConfidenceCap = 10

// tuningImpact is the influence that a new tuning target has on the previously
// cached value. This number is mostly just an out-of-the-blue heuristic that
// prevents the estimates from jumping around. There's no particular reason for
// the current value.
const tuningImpact = 0.25

// Tracker estimates the throughput capacity of a peer with regard to each data
// type it can deliver. The goal is to dynamically adjust request sizes to max
// out network throughput without overloading either the peer or the local node.
//
// By tracking in real time the latencies and bandwidths peers exhibit for each
// packet type, it's possible to prevent overloading by detecting a slowdown on
// one type when another type is pushed too hard.
//
// Similarly, real time measurements also help avoid overloading the local net
// connection if our peers would otherwise be capable to deliver more, but the
// local link is saturated. In that case, the live measurements will force us
// to reduce request sizes until the throughput gets stable.
//
// Lastly, message rate measurements allows us to detect if a peer is unusually
// slow compared to other peers, in which case we can decide to keep it around
// or free up the slot so someone closer.
//
// Since throughput tracking and estimation adapts dynamically to live network
// conditions, it's fine to have multiple trackers locally track the same peer
// in different subsystem. The throughput will simply be distributed across the
// two trackers if both are highly active.
type Tracker struct {
	// capacity is the number of items retrievable per second of a given type.
	// It is analogous to bandwidth, but we deliberately avoided using bytes
	// as the unit, since serving nodes also spend a lot of time loading data
	// from disk, which is linear in the number of items, but mostly constant
	// in their sizes.
	//
	// Callers of course are free to use the item counter as a byte counter if
	// or when their protocol of choice if capped by bytes instead of items.
	// (eg. eth.getHeaders vs snap.getAccountRange).
	capacity map[uint64]float64

	// roundtrip is the latency a peer in general responds to data requests.
	// This number is not used inside the tracker, but is exposed to compare
	// peers to each other and filter out slow ones. Note however, it only
	// makes sense to compare RTTs if the caller caters request sizes for
	// each peer to target the same RTT. There's no need to make this number
	// the real networking RTT, we just need a number to compare peers with.
	roundtrip time.Duration

	lock sync.RWMutex
}

// NewTracker creates a new message rate tracker for a specific peer. An initial
// RTT is needed to avoid a peer getting marked as an outlier compared to others
// right after joining. It's suggested to use the median rtt across all peers to
// init a new peer tracker.
func NewTracker(caps map[uint64]float64, rtt time.Duration) *Tracker {
	if caps == nil {
		caps = make(map[uint64]float64)
	}
	return &Tracker{
		capacity:  caps,
		roundtrip: rtt,
	}
}

// Capacity calculates the number of items the peer is estimated to be able to
// retrieve within the allotted time slot. The method will round up any division
// errors and will add an additional overestimation ratio on top. The reason for
// overshooting the capacity is because certain message types might not increase
// the load proportionally to the requested items, so fetching a bit more might
// still take the same RTT. By forcefully overshooting by a small amount, we can
// avoid locking into a lower-that-real capacity.
func (t *Tracker) Capacity(kind uint64, targetRTT time.Duration) int {
	t.lock.RLock()
	defer t.lock.RUnlock()

	// Calculate the actual measured throughput
	throughput := t.capacity[kind] * float64(targetRTT) / float64(time.Second)

	// Return an overestimation to force the peer out of a stuck minima, adding
	// +1 in case the item count is too low for the overestimator to dent
	return roundCapacity(1 + capacityOverestimation*throughput)
}

// roundCapacity gives the integer value of a capacity.
// The result fits int32, and is guaranteed to be positive.
func roundCapacity(cap float64) int {
	const maxInt32 = float64(1<<31 - 1)
	return int(math.Min(maxInt32, math.Max(1, math.Ceil(cap))))
}

// Update modifies the peer's capacity values for a specific data type with a new
// measurement. If the delivery is zero, the peer is assumed to have either timed
// out or to not have the requested data, resulting in a slash to 0 capacity. This
// avoids assigning the peer retrievals that it won't be able to honour.
func (t *Tracker) Update(kind uint64, elapsed time.Duration, items int) {
	t.lock.Lock()
	defer t.lock.Unlock()

	// If nothing was delivered (timeout / unavailable data), reduce throughput
	// to minimum
	if items == 0 {
		t.capacity[kind] = 0
		return
	}
	// Otherwise update the throughput with a new measurement
	if elapsed <= 0 {
		elapsed = 1 // +1 (ns) to ensure non-zero divisor
	}
	measured := float64(items) / (float64(elapsed) / float64(time.Second))

	t.capacity[kind] = (1-measurementImpact)*(t.capacity[kind]) + measurementImpact*measured
	t.roundtrip = time.Duration((1-measurementImpact)*float64(t.roundtrip) + measurementImpact*float64(elapsed))
}

// Trackers is a set of message rate trackers across a number of peers with the
// goal of aggregating certain measurements across the entire set for outlier
// filtering and newly joining initialization.
type Trackers struct {
	trackers map[string]*Tracker

	// roundtrip is the current best guess as to what is a stable round trip time
	// across the entire collection of connected peers. This is derived from the
	// various trackers added, but is used as a cache to avoid recomputing on each
	// network request. The value is updated once every RTT to avoid fluctuations
	// caused by hiccups or peer events.
	roundtrip time.Duration

	// confidence represents the probability that the estimated roundtrip value
	// is the real one across all our peers. The confidence value is used as an
	// impact factor of new measurements on old estimates. As our connectivity
	// stabilizes, this value gravitates towards 1, new measurements having
	// almost no impact. If there's a large peer churn and few peers, then new
	// measurements will impact it more. The confidence is increased with every
	// packet and dropped with every new connection.
	confidence float64

	// tuned is the time instance the tracker recalculated its cached roundtrip
	// value and confidence values. A cleaner way would be to have a heartbeat
	// goroutine do it regularly, but that requires a lot of maintenance to just
	// run every now and again.
	tuned time.Time

	// The fields below can be used to override certain default values. Their
	// purpose is to allow quicker tests. Don't use them in production.
	OverrideTTLLimit time.Duration

	log  log.Logger
	lock sync.RWMutex
}

// NewTrackers creates an empty set of trackers to be filled with peers.
func NewTrackers(log log.Logger) *Trackers {
	return &Trackers{
		trackers:         make(map[string]*Tracker),
		roundtrip:        rttMaxEstimate,
		confidence:       1,
		tuned:            time.Now(),
		OverrideTTLLimit: ttlLimit,
		log:              log,
	}
}

// Track inserts a new tracker into the set.
func (t *Trackers) Track(id string, tracker *Tracker) error {
	t.lock.Lock()
	defer t.lock.Unlock()

	if _, ok := t.trackers[id]; ok {
		return errors.New("already tracking")
	}
	t.trackers[id] = tracker
	t.detune()

	return nil
}

// Untrack stops tracking a previously added peer.
func (t *Trackers) Untrack(id string) error {
	t.lock.Lock()
	defer t.lock.Unlock()

	if _, ok := t.trackers[id]; !ok {
		return errors.New("not tracking")
	}
	delete(t.trackers, id)
	return nil
}

// MedianRoundTrip returns the median RTT across all known trackers. The purpose
// of the median RTT is to initialize a new peer with sane statistics that it will
// hopefully outperform. If it seriously underperforms, there's a risk of dropping
// the peer, but that is ok as we're aiming for a strong median.
func (t *Trackers) MedianRoundTrip() time.Duration {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.medianRoundTrip()
}

// medianRoundTrip is the internal lockless version of MedianRoundTrip to be used
// by the QoS tuner.
func (t *Trackers) medianRoundTrip() time.Duration {
	// Gather all the currently measured round trip times
	rtts := make([]float64, 0, len(t.trackers))
	for _, tt := range t.trackers {
		tt.lock.RLock()
		rtts = append(rtts, float64(tt.roundtrip))
		tt.lock.RUnlock()
	}
	sort.Float64s(rtts)

	var median time.Duration
	switch len(rtts) {
	case 0:
		median = rttMaxEstimate
	case 1:
		median = time.Duration(rtts[0])
	default:
		idx := int(math.Sqrt(float64(len(rtts))))
		median = time.Duration(rtts[idx])
	}
	// Restrict the RTT into some QoS defaults, irrelevant of true RTT
	if median < rttMinEstimate {
		median = rttMinEstimate
	}
	if median > rttMaxEstimate {
		median = rttMaxEstimate
	}
	return median
}

// MeanCapacities returns the capacities averaged across all the added trackers.
// The purpose of the mean capacities are to initialize a new peer with some sane
// starting values that it will hopefully outperform. If the mean overshoots, the
// peer will be cut back to minimal capacity and given another chance.
func (t *Trackers) MeanCapacities() map[uint64]float64 {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.meanCapacities()
}

// meanCapacities is the internal lockless version of MeanCapacities used for
// debug logging.
func (t *Trackers) meanCapacities() map[uint64]float64 {
	capacities := make(map[uint64]float64, len(t.trackers))
	for _, tt := range t.trackers {
		tt.lock.RLock()
		for key, val := range tt.capacity {
			capacities[key] += val
		}
		tt.lock.RUnlock()
	}
	for key, val := range capacities {
		capacities[key] = val / float64(len(t.trackers))
	}
	return capacities
}

// TargetRoundTrip returns the current target round trip time for a request to
// complete in.The returned RTT is slightly under the estimated RTT. The reason
// is that message rate estimation is a 2 dimensional problem which is solvable
// for any RTT. The goal is to gravitate towards smaller RTTs instead of large
// messages, to result in a stabler download stream.
func (t *Trackers) TargetRoundTrip() time.Duration {
	// Recalculate the internal caches if it's been a while
	t.tune()

	// Caches surely recent, return target roundtrip
	t.lock.RLock()
	defer t.lock.RUnlock()

	return time.Duration(float64(t.roundtrip) * rttPushdownFactor)
}

// TargetTimeout returns the timeout allowance for a single request to finish
// under. The timeout is proportional to the roundtrip, but also takes into
// consideration the tracker's confidence in said roundtrip and scales it
// accordingly. The final value is capped to avoid runaway requests.
func (t *Trackers) TargetTimeout() time.Duration {
	// Recalculate the internal caches if it's been a while
	t.tune()

	// Caches surely recent, return target timeout
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.targetTimeout()
}

// targetTimeout is the internal lockless version of TargetTimeout to be used
// during QoS tuning.
func (t *Trackers) targetTimeout() time.Duration {
	timeout := time.Duration(ttlScaling * float64(t.roundtrip) / t.confidence)
	if timeout > t.OverrideTTLLimit {
		timeout = t.OverrideTTLLimit
	}
	return timeout
}

// tune gathers the individual tracker statistics and updates the estimated
// request round trip time.
func (t *Trackers) tune() {
	// Tune may be called concurrently all over the place, but we only want to
	// periodically update and even then only once. First check if it was updated
	// recently and abort if so.
	t.lock.RLock()
	dirty := time.Since(t.tuned) > t.roundtrip
	t.lock.RUnlock()
	if !dirty {
		return
	}
	// If an update is needed, obtain a write lock but make sure we don't update
	// it on all concurrent threads one by one.
	t.lock.Lock()
	defer t.lock.Unlock()

	if dirty := time.Since(t.tuned) > t.roundtrip; !dirty {
		return // A concurrent request beat us to the tuning
	}
	// First thread reaching the tuning point, update the estimates and return
	t.roundtrip = time.Duration((1-tuningImpact)*float64(t.roundtrip) + tuningImpact*float64(t.medianRoundTrip()))
	t.confidence = t.confidence + (1-t.confidence)/2

	t.tuned = time.Now()
	t.log.Debug("Recalculated msgrate QoS values", "rtt", t.roundtrip, "confidence", t.confidence, "ttl", t.targetTimeout(), "next", t.tuned.Add(t.roundtrip))
	t.log.Trace("Debug dump of mean capacities", "caps", log.Lazy{Fn: t.meanCapacities})
}

// detune reduces the tracker's confidence in order to make fresh measurements
// have a larger impact on the estimates. It is meant to be used during new peer
// connections so they can have a proper impact on the estimates.
func (t *Trackers) detune() {
	// If we have a single peer, confidence is always 1
	if len(t.trackers) == 1 {
		t.confidence = 1
		return
	}
	// If we have a ton of peers, don't drop the confidence since there's enough
	// remaining to retain the same throughput
	if len(t.trackers) >= tuningConfidenceCap {
		return
	}
	// Otherwise drop the confidence factor
	peers := float64(len(t.trackers))

	t.confidence = t.confidence * (peers - 1) / peers
	if t.confidence < rttMinConfidence {
		t.confidence = rttMinConfidence
	}
	t.log.Debug("Relaxed msgrate QoS values", "rtt", t.roundtrip, "confidence", t.confidence, "ttl", t.targetTimeout())
}

// Capacity is a helper function to access a specific tracker without having to
// track it explicitly outside.
func (t *Trackers) Capacity(id string, kind uint64, targetRTT time.Duration) int {
	t.lock.RLock()
	defer t.lock.RUnlock()

	tracker := t.trackers[id]
	if tracker == nil {
		return 1 // Unregister race, don't return 0, it's a dangerous number
	}
	return tracker.Capacity(kind, targetRTT)
}

// Update is a helper function to access a specific tracker without having to
// track it explicitly outside.
func (t *Trackers) Update(id string, kind uint64, elapsed time.Duration, items int) {
	t.lock.RLock()
	defer t.lock.RUnlock()

	if tracker := t.trackers[id]; tracker != nil {
		tracker.Update(kind, elapsed, items)
	}
}
