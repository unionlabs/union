// Copyright 2016 The go-ethereum Authors
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

// Package flowcontrol implements a client side flow control mechanism
package flowcontrol

import (
	"fmt"
	"math"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/log"
)

const (
	// fcTimeConst is the time constant applied for MinRecharge during linear
	// buffer recharge period
	fcTimeConst = time.Millisecond
	// DecParamDelay is applied at server side when decreasing capacity in order to
	// avoid a buffer underrun error due to requests sent by the client before
	// receiving the capacity update announcement
	DecParamDelay = time.Second * 2
	// keepLogs is the duration of keeping logs; logging is not used if zero
	keepLogs = 0
)

// ServerParams are the flow control parameters specified by a server for a client
//
// Note: a server can assign different amounts of capacity to each client by giving
// different parameters to them.
type ServerParams struct {
	BufLimit, MinRecharge uint64
}

// scheduledUpdate represents a delayed flow control parameter update
type scheduledUpdate struct {
	time   mclock.AbsTime
	params ServerParams
}

// ClientNode is the flow control system's representation of a client
// (used in server mode only)
type ClientNode struct {
	params         ServerParams
	bufValue       int64
	lastTime       mclock.AbsTime
	updateSchedule []scheduledUpdate
	sumCost        uint64            // sum of req costs received from this client
	accepted       map[uint64]uint64 // value = sumCost after accepting the given req
	connected      bool
	lock           sync.Mutex
	cm             *ClientManager
	log            *logger
	cmNodeFields
}

// NewClientNode returns a new ClientNode
func NewClientNode(cm *ClientManager, params ServerParams) *ClientNode {
	node := &ClientNode{
		cm:        cm,
		params:    params,
		bufValue:  int64(params.BufLimit),
		lastTime:  cm.clock.Now(),
		accepted:  make(map[uint64]uint64),
		connected: true,
	}
	if keepLogs > 0 {
		node.log = newLogger(keepLogs)
	}
	cm.connect(node)
	return node
}

// Disconnect should be called when a client is disconnected
func (node *ClientNode) Disconnect() {
	node.lock.Lock()
	defer node.lock.Unlock()

	node.connected = false
	node.cm.disconnect(node)
}

// BufferStatus returns the current buffer value and limit
func (node *ClientNode) BufferStatus() (uint64, uint64) {
	node.lock.Lock()
	defer node.lock.Unlock()

	if !node.connected {
		return 0, 0
	}
	now := node.cm.clock.Now()
	node.update(now)
	node.cm.updateBuffer(node, 0, now)
	bv := node.bufValue
	if bv < 0 {
		bv = 0
	}
	return uint64(bv), node.params.BufLimit
}

// OneTimeCost subtracts the given amount from the node's buffer.
//
// Note: this call can take the buffer into the negative region internally.
// In this case zero buffer value is returned by exported calls and no requests
// are accepted.
func (node *ClientNode) OneTimeCost(cost uint64) {
	node.lock.Lock()
	defer node.lock.Unlock()

	now := node.cm.clock.Now()
	node.update(now)
	node.bufValue -= int64(cost)
	node.cm.updateBuffer(node, -int64(cost), now)
}

// Freeze notifies the client manager about a client freeze event in which case
// the total capacity allowance is slightly reduced.
func (node *ClientNode) Freeze() {
	node.lock.Lock()
	frozenCap := node.params.MinRecharge
	node.lock.Unlock()
	node.cm.reduceTotalCapacity(frozenCap)
}

// update recalculates the buffer value at a specified time while also performing
// scheduled flow control parameter updates if necessary
func (node *ClientNode) update(now mclock.AbsTime) {
	for len(node.updateSchedule) > 0 && node.updateSchedule[0].time <= now {
		node.recalcBV(node.updateSchedule[0].time)
		node.updateParams(node.updateSchedule[0].params, now)
		node.updateSchedule = node.updateSchedule[1:]
	}
	node.recalcBV(now)
}

// recalcBV recalculates the buffer value at a specified time
func (node *ClientNode) recalcBV(now mclock.AbsTime) {
	dt := uint64(now - node.lastTime)
	if now < node.lastTime {
		dt = 0
	}
	node.bufValue += int64(node.params.MinRecharge * dt / uint64(fcTimeConst))
	if node.bufValue > int64(node.params.BufLimit) {
		node.bufValue = int64(node.params.BufLimit)
	}
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("updated  bv=%d  MRR=%d  BufLimit=%d", node.bufValue, node.params.MinRecharge, node.params.BufLimit))
	}
	node.lastTime = now
}

// UpdateParams updates the flow control parameters of a client node
func (node *ClientNode) UpdateParams(params ServerParams) {
	node.lock.Lock()
	defer node.lock.Unlock()

	now := node.cm.clock.Now()
	node.update(now)
	if params.MinRecharge >= node.params.MinRecharge {
		node.updateSchedule = nil
		node.updateParams(params, now)
	} else {
		for i, s := range node.updateSchedule {
			if params.MinRecharge >= s.params.MinRecharge {
				s.params = params
				node.updateSchedule = node.updateSchedule[:i+1]
				return
			}
		}
		node.updateSchedule = append(node.updateSchedule, scheduledUpdate{time: now.Add(DecParamDelay), params: params})
	}
}

// updateParams updates the flow control parameters of the node
func (node *ClientNode) updateParams(params ServerParams, now mclock.AbsTime) {
	diff := int64(params.BufLimit - node.params.BufLimit)
	if diff > 0 {
		node.bufValue += diff
	} else if node.bufValue > int64(params.BufLimit) {
		node.bufValue = int64(params.BufLimit)
	}
	node.cm.updateParams(node, params, now)
}

// AcceptRequest returns whether a new request can be accepted and the missing
// buffer amount if it was rejected due to a buffer underrun. If accepted, maxCost
// is deducted from the flow control buffer.
func (node *ClientNode) AcceptRequest(reqID, index, maxCost uint64) (accepted bool, bufShort uint64, priority int64) {
	node.lock.Lock()
	defer node.lock.Unlock()

	now := node.cm.clock.Now()
	node.update(now)
	if int64(maxCost) > node.bufValue {
		if node.log != nil {
			node.log.add(now, fmt.Sprintf("rejected  reqID=%d  bv=%d  maxCost=%d", reqID, node.bufValue, maxCost))
			node.log.dump(now)
		}
		return false, maxCost - uint64(node.bufValue), 0
	}
	node.bufValue -= int64(maxCost)
	node.sumCost += maxCost
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("accepted  reqID=%d  bv=%d  maxCost=%d  sumCost=%d", reqID, node.bufValue, maxCost, node.sumCost))
	}
	node.accepted[index] = node.sumCost
	return true, 0, node.cm.accepted(node, maxCost, now)
}

// RequestProcessed should be called when the request has been processed
func (node *ClientNode) RequestProcessed(reqID, index, maxCost, realCost uint64) uint64 {
	node.lock.Lock()
	defer node.lock.Unlock()

	now := node.cm.clock.Now()
	node.update(now)
	node.cm.processed(node, maxCost, realCost, now)
	bv := node.bufValue + int64(node.sumCost-node.accepted[index])
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("processed  reqID=%d  bv=%d  maxCost=%d  realCost=%d  sumCost=%d  oldSumCost=%d  reportedBV=%d", reqID, node.bufValue, maxCost, realCost, node.sumCost, node.accepted[index], bv))
	}
	delete(node.accepted, index)
	if bv < 0 {
		return 0
	}
	return uint64(bv)
}

// ServerNode is the flow control system's representation of a server
// (used in client mode only)
type ServerNode struct {
	clock       mclock.Clock
	bufEstimate uint64
	bufRecharge bool
	lastTime    mclock.AbsTime
	params      ServerParams
	sumCost     uint64            // sum of req costs sent to this server
	pending     map[uint64]uint64 // value = sumCost after sending the given req
	log         *logger
	lock        sync.RWMutex
}

// NewServerNode returns a new ServerNode
func NewServerNode(params ServerParams, clock mclock.Clock) *ServerNode {
	node := &ServerNode{
		clock:       clock,
		bufEstimate: params.BufLimit,
		bufRecharge: false,
		lastTime:    clock.Now(),
		params:      params,
		pending:     make(map[uint64]uint64),
	}
	if keepLogs > 0 {
		node.log = newLogger(keepLogs)
	}
	return node
}

// UpdateParams updates the flow control parameters of the node
func (node *ServerNode) UpdateParams(params ServerParams) {
	node.lock.Lock()
	defer node.lock.Unlock()

	node.recalcBLE(mclock.Now())
	if params.BufLimit > node.params.BufLimit {
		node.bufEstimate += params.BufLimit - node.params.BufLimit
	} else {
		if node.bufEstimate > params.BufLimit {
			node.bufEstimate = params.BufLimit
		}
	}
	node.params = params
}

// recalcBLE recalculates the lowest estimate for the client's buffer value at
// the given server at the specified time
func (node *ServerNode) recalcBLE(now mclock.AbsTime) {
	if now < node.lastTime {
		return
	}
	if node.bufRecharge {
		dt := uint64(now - node.lastTime)
		node.bufEstimate += node.params.MinRecharge * dt / uint64(fcTimeConst)
		if node.bufEstimate >= node.params.BufLimit {
			node.bufEstimate = node.params.BufLimit
			node.bufRecharge = false
		}
	}
	node.lastTime = now
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("updated  bufEst=%d  MRR=%d  BufLimit=%d", node.bufEstimate, node.params.MinRecharge, node.params.BufLimit))
	}
}

// safetyMargin is added to the flow control waiting time when estimated buffer value is low
const safetyMargin = time.Millisecond

// CanSend returns the minimum waiting time required before sending a request
// with the given maximum estimated cost. Second return value is the relative
// estimated buffer level after sending the request (divided by BufLimit).
func (node *ServerNode) CanSend(maxCost uint64) (time.Duration, float64) {
	node.lock.RLock()
	defer node.lock.RUnlock()

	if node.params.BufLimit == 0 {
		return time.Duration(math.MaxInt64), 0
	}
	now := node.clock.Now()
	node.recalcBLE(now)
	maxCost += uint64(safetyMargin) * node.params.MinRecharge / uint64(fcTimeConst)
	if maxCost > node.params.BufLimit {
		maxCost = node.params.BufLimit
	}
	if node.bufEstimate >= maxCost {
		relBuf := float64(node.bufEstimate-maxCost) / float64(node.params.BufLimit)
		if node.log != nil {
			node.log.add(now, fmt.Sprintf("canSend  bufEst=%d  maxCost=%d  true  relBuf=%f", node.bufEstimate, maxCost, relBuf))
		}
		return 0, relBuf
	}
	timeLeft := time.Duration((maxCost - node.bufEstimate) * uint64(fcTimeConst) / node.params.MinRecharge)
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("canSend  bufEst=%d  maxCost=%d  false  timeLeft=%v", node.bufEstimate, maxCost, timeLeft))
	}
	return timeLeft, 0
}

// QueuedRequest should be called when the request has been assigned to the given
// server node, before putting it in the send queue. It is mandatory that requests
// are sent in the same order as the QueuedRequest calls are made.
func (node *ServerNode) QueuedRequest(reqID, maxCost uint64) {
	node.lock.Lock()
	defer node.lock.Unlock()

	now := node.clock.Now()
	node.recalcBLE(now)
	// Note: we do not know when requests actually arrive to the server so bufRecharge
	// is not turned on here if buffer was full; in this case it is going to be turned
	// on by the first reply's bufValue feedback
	if node.bufEstimate >= maxCost {
		node.bufEstimate -= maxCost
	} else {
		log.Error("Queued request with insufficient buffer estimate")
		node.bufEstimate = 0
	}
	node.sumCost += maxCost
	node.pending[reqID] = node.sumCost
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("queued  reqID=%d  bufEst=%d  maxCost=%d  sumCost=%d", reqID, node.bufEstimate, maxCost, node.sumCost))
	}
}

// ReceivedReply adjusts estimated buffer value according to the value included in
// the latest request reply.
func (node *ServerNode) ReceivedReply(reqID, bv uint64) {
	node.lock.Lock()
	defer node.lock.Unlock()

	now := node.clock.Now()
	node.recalcBLE(now)
	if bv > node.params.BufLimit {
		bv = node.params.BufLimit
	}
	sc, ok := node.pending[reqID]
	if !ok {
		return
	}
	delete(node.pending, reqID)
	cc := node.sumCost - sc
	newEstimate := uint64(0)
	if bv > cc {
		newEstimate = bv - cc
	}
	if newEstimate > node.bufEstimate {
		// Note: we never reduce the buffer estimate based on the reported value because
		// this can only happen because of the delayed delivery of the latest reply.
		// The lowest estimate based on the previous reply can still be considered valid.
		node.bufEstimate = newEstimate
	}

	node.bufRecharge = node.bufEstimate < node.params.BufLimit
	node.lastTime = now
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("received  reqID=%d  bufEst=%d  reportedBv=%d  sumCost=%d  oldSumCost=%d", reqID, node.bufEstimate, bv, node.sumCost, sc))
	}
}

// ResumeFreeze cleans all pending requests and sets the buffer estimate to the
// reported value after resuming from a frozen state
func (node *ServerNode) ResumeFreeze(bv uint64) {
	node.lock.Lock()
	defer node.lock.Unlock()

	for reqID := range node.pending {
		delete(node.pending, reqID)
	}
	now := node.clock.Now()
	node.recalcBLE(now)
	if bv > node.params.BufLimit {
		bv = node.params.BufLimit
	}
	node.bufEstimate = bv
	node.bufRecharge = node.bufEstimate < node.params.BufLimit
	node.lastTime = now
	if node.log != nil {
		node.log.add(now, fmt.Sprintf("unfreeze  bv=%d  sumCost=%d", bv, node.sumCost))
	}
}

// DumpLogs dumps the event log if logging is used
func (node *ServerNode) DumpLogs() {
	node.lock.Lock()
	defer node.lock.Unlock()

	if node.log != nil {
		node.log.dump(node.clock.Now())
	}
}
