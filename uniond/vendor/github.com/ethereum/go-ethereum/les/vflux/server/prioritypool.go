// Copyright 2020 The go-ethereum Authors
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

package server

import (
	"math"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/common/prque"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

const (
	lazyQueueRefresh = time.Second * 10 // refresh period of the active queue
)

// priorityPool handles a set of nodes where each node has a capacity (a scalar value)
// and a priority (which can change over time and can also depend on the capacity).
// A node is active if it has at least the necessary minimal amount of capacity while
// inactive nodes have 0 capacity (values between 0 and the minimum are not allowed).
// The pool ensures that the number and total capacity of all active nodes are limited
// and the highest priority nodes are active at all times (limits can be changed
// during operation with immediate effect).
//
// When activating clients a priority bias is applied in favor of the already active
// nodes in order to avoid nodes quickly alternating between active and inactive states
// when their priorities are close to each other. The bias is specified in terms of
// duration (time) because priorities are expected to usually get lower over time and
// therefore a future minimum prediction (see EstMinPriority) should monotonously
// decrease with the specified time parameter.
// This time bias can be interpreted as minimum expected active time at the given
// capacity (if the threshold priority stays the same).
//
// Nodes in the pool always have either inactiveFlag or activeFlag set. A new node is
// added to the pool by externally setting inactiveFlag. priorityPool can switch a node
// between inactiveFlag and activeFlag at any time. Nodes can be removed from the pool
// by externally resetting both flags. activeFlag should not be set externally.
//
// The highest priority nodes in "inactive" state are moved to "active" state as soon as
// the minimum capacity can be granted for them. The capacity of lower priority active
// nodes is reduced or they are demoted to "inactive" state if their priority is
// insufficient even at minimal capacity.
type priorityPool struct {
	setup                        *serverSetup
	ns                           *nodestate.NodeStateMachine
	clock                        mclock.Clock
	lock                         sync.Mutex
	maxCount, maxCap             uint64
	minCap                       uint64
	activeBias                   time.Duration
	capacityStepDiv, fineStepDiv uint64

	// The snapshot of priority pool for query.
	cachedCurve    *capacityCurve
	ccUpdatedAt    mclock.AbsTime
	ccUpdateForced bool

	// Runtime status of prioritypool, represents the
	// temporary state if tempState is not empty
	tempState              []*ppNodeInfo
	activeCount, activeCap uint64
	activeQueue            *prque.LazyQueue[int64, *ppNodeInfo]
	inactiveQueue          *prque.Prque[int64, *ppNodeInfo]
}

// ppNodeInfo is the internal node descriptor of priorityPool
type ppNodeInfo struct {
	nodePriority               nodePriority
	node                       *enode.Node
	connected                  bool
	capacity                   uint64 // only changed when temporary state is committed
	activeIndex, inactiveIndex int

	tempState    bool   // should only be true while the priorityPool lock is held
	tempCapacity uint64 // equals capacity when tempState is false

	// the following fields only affect the temporary state and they are set to their
	// default value when leaving the temp state
	minTarget, stepDiv uint64
	bias               time.Duration
}

// newPriorityPool creates a new priorityPool
func newPriorityPool(ns *nodestate.NodeStateMachine, setup *serverSetup, clock mclock.Clock, minCap uint64, activeBias time.Duration, capacityStepDiv, fineStepDiv uint64) *priorityPool {
	pp := &priorityPool{
		setup:           setup,
		ns:              ns,
		clock:           clock,
		inactiveQueue:   prque.New[int64, *ppNodeInfo](inactiveSetIndex),
		minCap:          minCap,
		activeBias:      activeBias,
		capacityStepDiv: capacityStepDiv,
		fineStepDiv:     fineStepDiv,
	}
	if pp.activeBias < time.Duration(1) {
		pp.activeBias = time.Duration(1)
	}
	pp.activeQueue = prque.NewLazyQueue(activeSetIndex, activePriority, pp.activeMaxPriority, clock, lazyQueueRefresh)

	ns.SubscribeField(pp.setup.balanceField, func(node *enode.Node, state nodestate.Flags, oldValue, newValue interface{}) {
		if newValue != nil {
			c := &ppNodeInfo{
				node:          node,
				nodePriority:  newValue.(nodePriority),
				activeIndex:   -1,
				inactiveIndex: -1,
			}
			ns.SetFieldSub(node, pp.setup.queueField, c)
			ns.SetStateSub(node, setup.inactiveFlag, nodestate.Flags{}, 0)
		} else {
			ns.SetStateSub(node, nodestate.Flags{}, pp.setup.activeFlag.Or(pp.setup.inactiveFlag), 0)
			if n, _ := pp.ns.GetField(node, pp.setup.queueField).(*ppNodeInfo); n != nil {
				pp.disconnectNode(n)
			}
			ns.SetFieldSub(node, pp.setup.capacityField, nil)
			ns.SetFieldSub(node, pp.setup.queueField, nil)
		}
	})
	ns.SubscribeState(pp.setup.activeFlag.Or(pp.setup.inactiveFlag), func(node *enode.Node, oldState, newState nodestate.Flags) {
		if c, _ := pp.ns.GetField(node, pp.setup.queueField).(*ppNodeInfo); c != nil {
			if oldState.IsEmpty() {
				pp.connectNode(c)
			}
			if newState.IsEmpty() {
				pp.disconnectNode(c)
			}
		}
	})
	ns.SubscribeState(pp.setup.updateFlag, func(node *enode.Node, oldState, newState nodestate.Flags) {
		if !newState.IsEmpty() {
			pp.updatePriority(node)
		}
	})
	return pp
}

// requestCapacity tries to set the capacity of a connected node to the highest possible
// value inside the given target range. If maxTarget is not reachable then the capacity is
// iteratively reduced in fine steps based on the fineStepDiv parameter until minTarget is reached.
// The function returns the new capacity if successful and the original capacity otherwise.
// Note: this function should run inside a NodeStateMachine operation
func (pp *priorityPool) requestCapacity(node *enode.Node, minTarget, maxTarget uint64, bias time.Duration) uint64 {
	pp.lock.Lock()
	pp.activeQueue.Refresh()

	if minTarget < pp.minCap {
		minTarget = pp.minCap
	}
	if maxTarget < minTarget {
		maxTarget = minTarget
	}
	if bias < pp.activeBias {
		bias = pp.activeBias
	}
	c, _ := pp.ns.GetField(node, pp.setup.queueField).(*ppNodeInfo)
	if c == nil {
		log.Error("requestCapacity called for unknown node", "id", node.ID())
		pp.lock.Unlock()
		return 0
	}
	pp.setTempState(c)
	if maxTarget > c.capacity {
		pp.setTempStepDiv(c, pp.fineStepDiv)
		pp.setTempBias(c, bias)
	}
	pp.setTempCapacity(c, maxTarget)
	c.minTarget = minTarget
	pp.removeFromQueues(c)
	pp.activeQueue.Push(c)
	pp.enforceLimits()
	updates := pp.finalizeChanges(c.tempCapacity >= minTarget && c.tempCapacity <= maxTarget && c.tempCapacity != c.capacity)
	pp.lock.Unlock()
	pp.updateFlags(updates)
	return c.capacity
}

// SetLimits sets the maximum number and total capacity of simultaneously active nodes
func (pp *priorityPool) SetLimits(maxCount, maxCap uint64) {
	pp.lock.Lock()
	pp.activeQueue.Refresh()
	inc := (maxCount > pp.maxCount) || (maxCap > pp.maxCap)
	dec := (maxCount < pp.maxCount) || (maxCap < pp.maxCap)
	pp.maxCount, pp.maxCap = maxCount, maxCap

	var updates []capUpdate
	if dec {
		pp.enforceLimits()
		updates = pp.finalizeChanges(true)
	}
	if inc {
		updates = append(updates, pp.tryActivate(false)...)
	}
	pp.lock.Unlock()
	pp.ns.Operation(func() { pp.updateFlags(updates) })
}

// setActiveBias sets the bias applied when trying to activate inactive nodes
func (pp *priorityPool) setActiveBias(bias time.Duration) {
	pp.lock.Lock()
	pp.activeBias = bias
	if pp.activeBias < time.Duration(1) {
		pp.activeBias = time.Duration(1)
	}
	updates := pp.tryActivate(false)
	pp.lock.Unlock()
	pp.ns.Operation(func() { pp.updateFlags(updates) })
}

// Active returns the number and total capacity of currently active nodes
func (pp *priorityPool) Active() (uint64, uint64) {
	pp.lock.Lock()
	defer pp.lock.Unlock()

	return pp.activeCount, pp.activeCap
}

// Inactive returns the number of currently inactive nodes
func (pp *priorityPool) Inactive() int {
	pp.lock.Lock()
	defer pp.lock.Unlock()

	return pp.inactiveQueue.Size()
}

// Limits returns the maximum allowed number and total capacity of active nodes
func (pp *priorityPool) Limits() (uint64, uint64) {
	pp.lock.Lock()
	defer pp.lock.Unlock()

	return pp.maxCount, pp.maxCap
}

// inactiveSetIndex callback updates ppNodeInfo item index in inactiveQueue
func inactiveSetIndex(a *ppNodeInfo, index int) {
	a.inactiveIndex = index
}

// activeSetIndex callback updates ppNodeInfo item index in activeQueue
func activeSetIndex(a *ppNodeInfo, index int) {
	a.activeIndex = index
}

// invertPriority inverts a priority value. The active queue uses inverted priorities
// because the node on the top is the first to be deactivated.
func invertPriority(p int64) int64 {
	if p == math.MinInt64 {
		return math.MaxInt64
	}
	return -p
}

// activePriority callback returns actual priority of ppNodeInfo item in activeQueue
func activePriority(c *ppNodeInfo) int64 {
	if c.bias == 0 {
		return invertPriority(c.nodePriority.priority(c.tempCapacity))
	} else {
		return invertPriority(c.nodePriority.estimatePriority(c.tempCapacity, 0, 0, c.bias, true))
	}
}

// activeMaxPriority callback returns estimated maximum priority of ppNodeInfo item in activeQueue
func (pp *priorityPool) activeMaxPriority(c *ppNodeInfo, until mclock.AbsTime) int64 {
	future := time.Duration(until - pp.clock.Now())
	if future < 0 {
		future = 0
	}
	return invertPriority(c.nodePriority.estimatePriority(c.tempCapacity, 0, future, c.bias, false))
}

// inactivePriority callback returns actual priority of ppNodeInfo item in inactiveQueue
func (pp *priorityPool) inactivePriority(p *ppNodeInfo) int64 {
	return p.nodePriority.priority(pp.minCap)
}

// removeFromQueues removes the node from the active/inactive queues
func (pp *priorityPool) removeFromQueues(c *ppNodeInfo) {
	if c.activeIndex >= 0 {
		pp.activeQueue.Remove(c.activeIndex)
	}
	if c.inactiveIndex >= 0 {
		pp.inactiveQueue.Remove(c.inactiveIndex)
	}
}

// connectNode is called when a new node has been added to the pool (inactiveFlag set)
// Note: this function should run inside a NodeStateMachine operation
func (pp *priorityPool) connectNode(c *ppNodeInfo) {
	pp.lock.Lock()
	pp.activeQueue.Refresh()
	if c.connected {
		pp.lock.Unlock()
		return
	}
	c.connected = true
	pp.inactiveQueue.Push(c, pp.inactivePriority(c))
	updates := pp.tryActivate(false)
	pp.lock.Unlock()
	pp.updateFlags(updates)
}

// disconnectNode is called when a node has been removed from the pool (both inactiveFlag
// and activeFlag reset)
// Note: this function should run inside a NodeStateMachine operation
func (pp *priorityPool) disconnectNode(c *ppNodeInfo) {
	pp.lock.Lock()
	pp.activeQueue.Refresh()
	if !c.connected {
		pp.lock.Unlock()
		return
	}
	c.connected = false
	pp.removeFromQueues(c)

	var updates []capUpdate
	if c.capacity != 0 {
		pp.setTempState(c)
		pp.setTempCapacity(c, 0)
		updates = pp.tryActivate(true)
	}
	pp.lock.Unlock()
	pp.updateFlags(updates)
}

// setTempState internally puts a node in a temporary state that can either be reverted
// or confirmed later. This temporary state allows changing the capacity of a node and
// moving it between the active and inactive queue. activeFlag/inactiveFlag and
// capacityField are not changed while the changes are still temporary.
func (pp *priorityPool) setTempState(c *ppNodeInfo) {
	if c.tempState {
		return
	}
	c.tempState = true
	if c.tempCapacity != c.capacity { // should never happen
		log.Error("tempCapacity != capacity when entering tempState")
	}
	// Assign all the defaults to the temp state.
	c.minTarget = pp.minCap
	c.stepDiv = pp.capacityStepDiv
	c.bias = 0
	pp.tempState = append(pp.tempState, c)
}

// unsetTempState revokes the temp status of the node and reset all internal
// fields to the default value.
func (pp *priorityPool) unsetTempState(c *ppNodeInfo) {
	if !c.tempState {
		return
	}
	c.tempState = false
	if c.tempCapacity != c.capacity { // should never happen
		log.Error("tempCapacity != capacity when leaving tempState")
	}
	c.minTarget = pp.minCap
	c.stepDiv = pp.capacityStepDiv
	c.bias = 0
}

// setTempCapacity changes the capacity of a node in the temporary state and adjusts
// activeCap and activeCount accordingly. Since this change is performed in the temporary
// state it should be called after setTempState and before finalizeChanges.
func (pp *priorityPool) setTempCapacity(c *ppNodeInfo, cap uint64) {
	if !c.tempState { // should never happen
		log.Error("Node is not in temporary state")
		return
	}
	pp.activeCap += cap - c.tempCapacity
	if c.tempCapacity == 0 {
		pp.activeCount++
	}
	if cap == 0 {
		pp.activeCount--
	}
	c.tempCapacity = cap
}

// setTempBias changes the connection bias of a node in the temporary state.
func (pp *priorityPool) setTempBias(c *ppNodeInfo, bias time.Duration) {
	if !c.tempState { // should never happen
		log.Error("Node is not in temporary state")
		return
	}
	c.bias = bias
}

// setTempStepDiv changes the capacity divisor of a node in the temporary state.
func (pp *priorityPool) setTempStepDiv(c *ppNodeInfo, stepDiv uint64) {
	if !c.tempState { // should never happen
		log.Error("Node is not in temporary state")
		return
	}
	c.stepDiv = stepDiv
}

// enforceLimits enforces active node count and total capacity limits. It returns the
// lowest active node priority. Note that this function is performed on the temporary
// internal state.
func (pp *priorityPool) enforceLimits() (*ppNodeInfo, int64) {
	if pp.activeCap <= pp.maxCap && pp.activeCount <= pp.maxCount {
		return nil, math.MinInt64
	}
	var (
		lastNode          *ppNodeInfo
		maxActivePriority int64
	)
	pp.activeQueue.MultiPop(func(c *ppNodeInfo, priority int64) bool {
		lastNode = c
		pp.setTempState(c)
		maxActivePriority = priority
		if c.tempCapacity == c.minTarget || pp.activeCount > pp.maxCount {
			pp.setTempCapacity(c, 0)
		} else {
			sub := c.tempCapacity / c.stepDiv
			if sub == 0 {
				sub = 1
			}
			if c.tempCapacity-sub < c.minTarget {
				sub = c.tempCapacity - c.minTarget
			}
			pp.setTempCapacity(c, c.tempCapacity-sub)
			pp.activeQueue.Push(c)
		}
		return pp.activeCap > pp.maxCap || pp.activeCount > pp.maxCount
	})
	return lastNode, invertPriority(maxActivePriority)
}

// finalizeChanges either commits or reverts temporary changes. The necessary capacity
// field and according flag updates are not performed here but returned in a list because
// they should be performed while the mutex is not held.
func (pp *priorityPool) finalizeChanges(commit bool) (updates []capUpdate) {
	for _, c := range pp.tempState {
		// always remove and push back in order to update biased priority
		pp.removeFromQueues(c)
		oldCapacity := c.capacity
		if commit {
			c.capacity = c.tempCapacity
		} else {
			pp.setTempCapacity(c, c.capacity) // revert activeCount/activeCap
		}
		pp.unsetTempState(c)

		if c.connected {
			if c.capacity != 0 {
				pp.activeQueue.Push(c)
			} else {
				pp.inactiveQueue.Push(c, pp.inactivePriority(c))
			}
			if c.capacity != oldCapacity {
				updates = append(updates, capUpdate{c.node, oldCapacity, c.capacity})
			}
		}
	}
	pp.tempState = nil
	if commit {
		pp.ccUpdateForced = true
	}
	return
}

// capUpdate describes a capacityField and activeFlag/inactiveFlag update
type capUpdate struct {
	node           *enode.Node
	oldCap, newCap uint64
}

// updateFlags performs capacityField and activeFlag/inactiveFlag updates while the
// pool mutex is not held
// Note: this function should run inside a NodeStateMachine operation
func (pp *priorityPool) updateFlags(updates []capUpdate) {
	for _, f := range updates {
		if f.oldCap == 0 {
			pp.ns.SetStateSub(f.node, pp.setup.activeFlag, pp.setup.inactiveFlag, 0)
		}
		if f.newCap == 0 {
			pp.ns.SetStateSub(f.node, pp.setup.inactiveFlag, pp.setup.activeFlag, 0)
			pp.ns.SetFieldSub(f.node, pp.setup.capacityField, nil)
		} else {
			pp.ns.SetFieldSub(f.node, pp.setup.capacityField, f.newCap)
		}
	}
}

// tryActivate tries to activate inactive nodes if possible
func (pp *priorityPool) tryActivate(commit bool) []capUpdate {
	for pp.inactiveQueue.Size() > 0 {
		c := pp.inactiveQueue.PopItem()
		pp.setTempState(c)
		pp.setTempBias(c, pp.activeBias)
		pp.setTempCapacity(c, pp.minCap)
		pp.activeQueue.Push(c)
		pp.enforceLimits()
		if c.tempCapacity > 0 {
			commit = true
			pp.setTempBias(c, 0)
		} else {
			break
		}
	}
	pp.ccUpdateForced = true
	return pp.finalizeChanges(commit)
}

// updatePriority gets the current priority value of the given node from the nodePriority
// interface and performs the necessary changes. It is triggered by updateFlag.
// Note: this function should run inside a NodeStateMachine operation
func (pp *priorityPool) updatePriority(node *enode.Node) {
	pp.lock.Lock()
	pp.activeQueue.Refresh()
	c, _ := pp.ns.GetField(node, pp.setup.queueField).(*ppNodeInfo)
	if c == nil || !c.connected {
		pp.lock.Unlock()
		return
	}
	pp.removeFromQueues(c)
	if c.capacity != 0 {
		pp.activeQueue.Push(c)
	} else {
		pp.inactiveQueue.Push(c, pp.inactivePriority(c))
	}
	updates := pp.tryActivate(false)
	pp.lock.Unlock()
	pp.updateFlags(updates)
}

// capacityCurve is a snapshot of the priority pool contents in a format that can efficiently
// estimate how much capacity could be granted to a given node at a given priority level.
type capacityCurve struct {
	points       []curvePoint       // curve points sorted in descending order of priority
	index        map[enode.ID][]int // curve point indexes belonging to each node
	excludeList  []int              // curve point indexes of excluded node
	excludeFirst bool               // true if activeCount == maxCount
}

type curvePoint struct {
	freeCap uint64 // available capacity and node count at the current priority level
	nextPri int64  // next priority level where more capacity will be available
}

// getCapacityCurve returns a new or recently cached capacityCurve based on the contents of the pool
func (pp *priorityPool) getCapacityCurve() *capacityCurve {
	pp.lock.Lock()
	defer pp.lock.Unlock()

	now := pp.clock.Now()
	dt := time.Duration(now - pp.ccUpdatedAt)
	if !pp.ccUpdateForced && pp.cachedCurve != nil && dt < time.Second*10 {
		return pp.cachedCurve
	}

	pp.ccUpdateForced = false
	pp.ccUpdatedAt = now
	curve := &capacityCurve{
		index: make(map[enode.ID][]int),
	}
	pp.cachedCurve = curve

	var excludeID enode.ID
	excludeFirst := pp.maxCount == pp.activeCount
	// reduce node capacities or remove nodes until nothing is left in the queue;
	// record the available capacity and the necessary priority after each step
	lastPri := int64(math.MinInt64)
	for pp.activeCap > 0 {
		cp := curvePoint{}
		if pp.activeCap > pp.maxCap {
			log.Error("Active capacity is greater than allowed maximum", "active", pp.activeCap, "maximum", pp.maxCap)
		} else {
			cp.freeCap = pp.maxCap - pp.activeCap
		}
		// temporarily increase activeCap to enforce reducing or removing a node capacity
		tempCap := cp.freeCap + 1
		pp.activeCap += tempCap
		var next *ppNodeInfo
		// enforceLimits removes the lowest priority node if it has minimal capacity,
		// otherwise reduces its capacity
		next, cp.nextPri = pp.enforceLimits()
		if cp.nextPri < lastPri {
			// enforce monotonicity which may be broken by continuously changing priorities
			cp.nextPri = lastPri
		} else {
			lastPri = cp.nextPri
		}
		pp.activeCap -= tempCap
		if next == nil {
			log.Error("getCapacityCurve: cannot remove next element from the priority queue")
			break
		}
		id := next.node.ID()
		if excludeFirst {
			// if the node count limit is already reached then mark the node with the
			// lowest priority for exclusion
			curve.excludeFirst = true
			excludeID = id
			excludeFirst = false
		}
		// multiple curve points and therefore multiple indexes may belong to a node
		// if it was removed in multiple steps (if its capacity was more than the minimum)
		curve.index[id] = append(curve.index[id], len(curve.points))
		curve.points = append(curve.points, cp)
	}
	// restore original state of the queue
	pp.finalizeChanges(false)
	curve.points = append(curve.points, curvePoint{
		freeCap: pp.maxCap,
		nextPri: math.MaxInt64,
	})
	if curve.excludeFirst {
		curve.excludeList = curve.index[excludeID]
	}
	return curve
}

// exclude returns a capacityCurve with the given node excluded from the original curve
func (cc *capacityCurve) exclude(id enode.ID) *capacityCurve {
	if excludeList, ok := cc.index[id]; ok {
		// return a new version of the curve (only one excluded node can be selected)
		// Note: if the first node was excluded by default (excludeFirst == true) then
		// we can forget about that and exclude the node with the given id instead.
		return &capacityCurve{
			points:      cc.points,
			index:       cc.index,
			excludeList: excludeList,
		}
	}
	return cc
}

func (cc *capacityCurve) getPoint(i int) curvePoint {
	cp := cc.points[i]
	if i == 0 && cc.excludeFirst {
		cp.freeCap = 0
		return cp
	}
	for ii := len(cc.excludeList) - 1; ii >= 0; ii-- {
		ei := cc.excludeList[ii]
		if ei < i {
			break
		}
		e1, e2 := cc.points[ei], cc.points[ei+1]
		cp.freeCap += e2.freeCap - e1.freeCap
	}
	return cp
}

// maxCapacity calculates the maximum capacity available for a node with a given
// (monotonically decreasing) priority vs. capacity function. Note that if the requesting
// node is already in the pool then it should be excluded from the curve in order to get
// the correct result.
func (cc *capacityCurve) maxCapacity(priority func(cap uint64) int64) uint64 {
	min, max := 0, len(cc.points)-1 // the curve always has at least one point
	for min < max {
		mid := (min + max) / 2
		cp := cc.getPoint(mid)
		if cp.freeCap == 0 || priority(cp.freeCap) > cp.nextPri {
			min = mid + 1
		} else {
			max = mid
		}
	}
	cp2 := cc.getPoint(min)
	if cp2.freeCap == 0 || min == 0 {
		return cp2.freeCap
	}
	cp1 := cc.getPoint(min - 1)
	if priority(cp2.freeCap) > cp1.nextPri {
		return cp2.freeCap
	}
	minc, maxc := cp1.freeCap, cp2.freeCap-1
	for minc < maxc {
		midc := (minc + maxc + 1) / 2
		if midc == 0 || priority(midc) > cp1.nextPri {
			minc = midc
		} else {
			maxc = midc - 1
		}
	}
	return maxc
}
