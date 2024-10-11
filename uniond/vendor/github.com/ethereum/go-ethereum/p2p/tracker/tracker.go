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

package tracker

import (
	"container/list"
	"fmt"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
)

const (
	// trackedGaugeName is the prefix of the per-packet request tracking.
	trackedGaugeName = "p2p/tracked"

	// lostMeterName is the prefix of the per-packet request expirations.
	lostMeterName = "p2p/lost"

	// staleMeterName is the prefix of the per-packet stale responses.
	staleMeterName = "p2p/stale"

	// waitHistName is the prefix of the per-packet (req only) waiting time histograms.
	waitHistName = "p2p/wait"

	// maxTrackedPackets is a huge number to act as a failsafe on the number of
	// pending requests the node will track. It should never be hit unless an
	// attacker figures out a way to spin requests.
	maxTrackedPackets = 100000
)

// request tracks sent network requests which have not yet received a response.
type request struct {
	peer    string
	version uint // Protocol version

	reqCode uint64 // Protocol message code of the request
	resCode uint64 // Protocol message code of the expected response

	time   time.Time     // Timestamp when the request was made
	expire *list.Element // Expiration marker to untrack it
}

// Tracker is a pending network request tracker to measure how much time it takes
// a remote peer to respond.
type Tracker struct {
	protocol string        // Protocol capability identifier for the metrics
	timeout  time.Duration // Global timeout after which to drop a tracked packet

	pending map[uint64]*request // Currently pending requests
	expire  *list.List          // Linked list tracking the expiration order
	wake    *time.Timer         // Timer tracking the expiration of the next item

	lock sync.Mutex // Lock protecting from concurrent updates
}

// New creates a new network request tracker to monitor how much time it takes to
// fill certain requests and how individual peers perform.
func New(protocol string, timeout time.Duration) *Tracker {
	return &Tracker{
		protocol: protocol,
		timeout:  timeout,
		pending:  make(map[uint64]*request),
		expire:   list.New(),
	}
}

// Track adds a network request to the tracker to wait for a response to arrive
// or until the request it cancelled or times out.
func (t *Tracker) Track(peer string, version uint, reqCode uint64, resCode uint64, id uint64) {
	if !metrics.Enabled {
		return
	}
	t.lock.Lock()
	defer t.lock.Unlock()

	// If there's a duplicate request, we've just random-collided (or more probably,
	// we have a bug), report it. We could also add a metric, but we're not really
	// expecting ourselves to be buggy, so a noisy warning should be enough.
	if _, ok := t.pending[id]; ok {
		log.Error("Network request id collision", "protocol", t.protocol, "version", version, "code", reqCode, "id", id)
		return
	}
	// If we have too many pending requests, bail out instead of leaking memory
	if pending := len(t.pending); pending >= maxTrackedPackets {
		log.Error("Request tracker exceeded allowance", "pending", pending, "peer", peer, "protocol", t.protocol, "version", version, "code", reqCode)
		return
	}
	// Id doesn't exist yet, start tracking it
	t.pending[id] = &request{
		peer:    peer,
		version: version,
		reqCode: reqCode,
		resCode: resCode,
		time:    time.Now(),
		expire:  t.expire.PushBack(id),
	}
	g := fmt.Sprintf("%s/%s/%d/%#02x", trackedGaugeName, t.protocol, version, reqCode)
	metrics.GetOrRegisterGauge(g, nil).Inc(1)

	// If we've just inserted the first item, start the expiration timer
	if t.wake == nil {
		t.wake = time.AfterFunc(t.timeout, t.clean)
	}
}

// clean is called automatically when a preset time passes without a response
// being delivered for the first network request.
func (t *Tracker) clean() {
	t.lock.Lock()
	defer t.lock.Unlock()

	// Expire anything within a certain threshold (might be no items at all if
	// we raced with the delivery)
	for t.expire.Len() > 0 {
		// Stop iterating if the next pending request is still alive
		var (
			head = t.expire.Front()
			id   = head.Value.(uint64)
			req  = t.pending[id]
		)
		if time.Since(req.time) < t.timeout+5*time.Millisecond {
			break
		}
		// Nope, dead, drop it
		t.expire.Remove(head)
		delete(t.pending, id)

		g := fmt.Sprintf("%s/%s/%d/%#02x", trackedGaugeName, t.protocol, req.version, req.reqCode)
		metrics.GetOrRegisterGauge(g, nil).Dec(1)

		m := fmt.Sprintf("%s/%s/%d/%#02x", lostMeterName, t.protocol, req.version, req.reqCode)
		metrics.GetOrRegisterMeter(m, nil).Mark(1)
	}
	t.schedule()
}

// schedule starts a timer to trigger on the expiration of the first network
// packet.
func (t *Tracker) schedule() {
	if t.expire.Len() == 0 {
		t.wake = nil
		return
	}
	t.wake = time.AfterFunc(time.Until(t.pending[t.expire.Front().Value.(uint64)].time.Add(t.timeout)), t.clean)
}

// Fulfil fills a pending request, if any is available, reporting on various metrics.
func (t *Tracker) Fulfil(peer string, version uint, code uint64, id uint64) {
	if !metrics.Enabled {
		return
	}
	t.lock.Lock()
	defer t.lock.Unlock()

	// If it's a non existing request, track as stale response
	req, ok := t.pending[id]
	if !ok {
		m := fmt.Sprintf("%s/%s/%d/%#02x", staleMeterName, t.protocol, version, code)
		metrics.GetOrRegisterMeter(m, nil).Mark(1)
		return
	}
	// If the response is funky, it might be some active attack
	if req.peer != peer || req.version != version || req.resCode != code {
		log.Warn("Network response id collision",
			"have", fmt.Sprintf("%s:%s/%d:%d", peer, t.protocol, version, code),
			"want", fmt.Sprintf("%s:%s/%d:%d", peer, t.protocol, req.version, req.resCode),
		)
		return
	}
	// Everything matches, mark the request serviced and meter it
	t.expire.Remove(req.expire)
	delete(t.pending, id)
	if req.expire.Prev() == nil {
		if t.wake.Stop() {
			t.schedule()
		}
	}
	g := fmt.Sprintf("%s/%s/%d/%#02x", trackedGaugeName, t.protocol, req.version, req.reqCode)
	metrics.GetOrRegisterGauge(g, nil).Dec(1)

	h := fmt.Sprintf("%s/%s/%d/%#02x", waitHistName, t.protocol, req.version, req.reqCode)
	sampler := func() metrics.Sample {
		return metrics.ResettingSample(
			metrics.NewExpDecaySample(1028, 0.015),
		)
	}
	metrics.GetOrRegisterHistogramLazy(h, nil, sampler).Update(time.Since(req.time).Microseconds())
}
