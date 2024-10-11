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

package server

import (
	"errors"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/les/vflux"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
	"github.com/ethereum/go-ethereum/rlp"
)

var (
	ErrNotConnected    = errors.New("client not connected")
	ErrNoPriority      = errors.New("priority too low to raise capacity")
	ErrCantFindMaximum = errors.New("unable to find maximum allowed capacity")
)

// ClientPool implements a client database that assigns a priority to each client
// based on a positive and negative balance. Positive balance is externally assigned
// to prioritized clients and is decreased with connection time and processed
// requests (unless the price factors are zero). If the positive balance is zero
// then negative balance is accumulated.
//
// Balance tracking and priority calculation for connected clients is done by
// balanceTracker. PriorityQueue ensures that clients with the lowest positive or
// highest negative balance get evicted when the total capacity allowance is full
// and new clients with a better balance want to connect.
//
// Already connected nodes receive a small bias in their favor in order to avoid
// accepting and instantly kicking out clients. In theory, we try to ensure that
// each client can have several minutes of connection time.
//
// Balances of disconnected clients are stored in nodeDB including positive balance
// and negative balance. Both positive balance and negative balance will decrease
// exponentially. If the balance is low enough, then the record will be dropped.
type ClientPool struct {
	*priorityPool
	*balanceTracker

	setup  *serverSetup
	clock  mclock.Clock
	ns     *nodestate.NodeStateMachine
	synced func() bool

	lock          sync.RWMutex
	connectedBias time.Duration

	minCap     uint64      // the minimal capacity value allowed for any client
	capReqNode *enode.Node // node that is requesting capacity change; only used inside NSM operation
}

// clientPeer represents a peer in the client pool. None of the callbacks should block.
type clientPeer interface {
	Node() *enode.Node
	FreeClientId() string                         // unique id for non-priority clients (typically a prefix of the network address)
	InactiveAllowance() time.Duration             // disconnection timeout for inactive non-priority peers
	UpdateCapacity(newCap uint64, requested bool) // signals a capacity update (requested is true if it is a result of a SetCapacity call on the given peer
	Disconnect()                                  // initiates disconnection (Unregister should always be called)
}

// NewClientPool creates a new client pool
func NewClientPool(balanceDb ethdb.KeyValueStore, minCap uint64, connectedBias time.Duration, clock mclock.Clock, synced func() bool) *ClientPool {
	setup := newServerSetup()
	ns := nodestate.NewNodeStateMachine(nil, nil, clock, setup.setup)
	cp := &ClientPool{
		priorityPool:   newPriorityPool(ns, setup, clock, minCap, connectedBias, 4, 100),
		balanceTracker: newBalanceTracker(ns, setup, balanceDb, clock, &utils.Expirer{}, &utils.Expirer{}),
		setup:          setup,
		ns:             ns,
		clock:          clock,
		minCap:         minCap,
		connectedBias:  connectedBias,
		synced:         synced,
	}

	ns.SubscribeState(nodestate.MergeFlags(setup.activeFlag, setup.inactiveFlag, setup.priorityFlag), func(node *enode.Node, oldState, newState nodestate.Flags) {
		if newState.Equals(setup.inactiveFlag) {
			// set timeout for non-priority inactive client
			var timeout time.Duration
			if c, ok := ns.GetField(node, setup.clientField).(clientPeer); ok {
				timeout = c.InactiveAllowance()
			}
			ns.AddTimeout(node, setup.inactiveFlag, timeout)
		}
		if oldState.Equals(setup.inactiveFlag) && newState.Equals(setup.inactiveFlag.Or(setup.priorityFlag)) {
			ns.SetStateSub(node, setup.inactiveFlag, nodestate.Flags{}, 0) // priority gained; remove timeout
		}
		if newState.Equals(setup.activeFlag) {
			// active with no priority; limit capacity to minCap
			cap, _ := ns.GetField(node, setup.capacityField).(uint64)
			if cap > minCap {
				cp.requestCapacity(node, minCap, minCap, 0)
			}
		}
		if newState.Equals(nodestate.Flags{}) {
			if c, ok := ns.GetField(node, setup.clientField).(clientPeer); ok {
				c.Disconnect()
			}
		}
	})

	ns.SubscribeField(setup.capacityField, func(node *enode.Node, state nodestate.Flags, oldValue, newValue interface{}) {
		if c, ok := ns.GetField(node, setup.clientField).(clientPeer); ok {
			newCap, _ := newValue.(uint64)
			c.UpdateCapacity(newCap, node == cp.capReqNode)
		}
	})

	// add metrics
	cp.ns.SubscribeState(nodestate.MergeFlags(cp.setup.activeFlag, cp.setup.inactiveFlag), func(node *enode.Node, oldState, newState nodestate.Flags) {
		if oldState.IsEmpty() && !newState.IsEmpty() {
			clientConnectedMeter.Mark(1)
		}
		if !oldState.IsEmpty() && newState.IsEmpty() {
			clientDisconnectedMeter.Mark(1)
		}
		if oldState.HasNone(cp.setup.activeFlag) && oldState.HasAll(cp.setup.activeFlag) {
			clientActivatedMeter.Mark(1)
		}
		if oldState.HasAll(cp.setup.activeFlag) && oldState.HasNone(cp.setup.activeFlag) {
			clientDeactivatedMeter.Mark(1)
		}
		activeCount, activeCap := cp.Active()
		totalActiveCountGauge.Update(int64(activeCount))
		totalActiveCapacityGauge.Update(int64(activeCap))
		totalInactiveCountGauge.Update(int64(cp.Inactive()))
	})
	return cp
}

// Start starts the client pool. Should be called before Register/Unregister.
func (cp *ClientPool) Start() {
	cp.ns.Start()
}

// Stop shuts the client pool down. The clientPeer interface callbacks will not be called
// after Stop. Register calls will return nil.
func (cp *ClientPool) Stop() {
	cp.balanceTracker.stop()
	cp.ns.Stop()
}

// Register registers the peer into the client pool. If the peer has insufficient
// priority and remains inactive for longer than the allowed timeout then it will be
// disconnected by calling the Disconnect function of the clientPeer interface.
func (cp *ClientPool) Register(peer clientPeer) ConnectedBalance {
	cp.ns.SetField(peer.Node(), cp.setup.clientField, peerWrapper{peer})
	balance, _ := cp.ns.GetField(peer.Node(), cp.setup.balanceField).(*nodeBalance)
	return balance
}

// Unregister removes the peer from the client pool
func (cp *ClientPool) Unregister(peer clientPeer) {
	cp.ns.SetField(peer.Node(), cp.setup.clientField, nil)
}

// SetConnectedBias sets the connection bias, which is applied to already connected clients
// So that already connected client won't be kicked out very soon and we can ensure all
// connected clients can have enough time to request or sync some data.
func (cp *ClientPool) SetConnectedBias(bias time.Duration) {
	cp.lock.Lock()
	cp.connectedBias = bias
	cp.setActiveBias(bias)
	cp.lock.Unlock()
}

// SetCapacity sets the assigned capacity of a connected client
func (cp *ClientPool) SetCapacity(node *enode.Node, reqCap uint64, bias time.Duration, requested bool) (capacity uint64, err error) {
	cp.lock.RLock()
	if cp.connectedBias > bias {
		bias = cp.connectedBias
	}
	cp.lock.RUnlock()

	cp.ns.Operation(func() {
		balance, _ := cp.ns.GetField(node, cp.setup.balanceField).(*nodeBalance)
		if balance == nil {
			err = ErrNotConnected
			return
		}
		capacity, _ = cp.ns.GetField(node, cp.setup.capacityField).(uint64)
		if capacity == 0 {
			// if the client is inactive then it has insufficient priority for the minimal capacity
			// (will be activated automatically with minCap when possible)
			return
		}
		if reqCap < cp.minCap {
			// can't request less than minCap; switching between 0 (inactive state) and minCap is
			// performed by the server automatically as soon as necessary/possible
			reqCap = cp.minCap
		}
		if reqCap > cp.minCap && cp.ns.GetState(node).HasNone(cp.setup.priorityFlag) {
			err = ErrNoPriority
			return
		}
		if reqCap == capacity {
			return
		}
		if requested {
			// mark the requested node so that the UpdateCapacity callback can signal
			// whether the update is the direct result of a SetCapacity call on the given node
			cp.capReqNode = node
			defer func() {
				cp.capReqNode = nil
			}()
		}

		var minTarget, maxTarget uint64
		if reqCap > capacity {
			// Estimate maximum available capacity at the current priority level and request
			// the estimated amount.
			// Note: requestCapacity could find the highest available capacity between the
			// current and the requested capacity but it could cost a lot of iterations with
			// fine step adjustment if the requested capacity is very high. By doing a quick
			// estimation of the maximum available capacity based on the capacity curve we
			// can limit the number of required iterations.
			curve := cp.getCapacityCurve().exclude(node.ID())
			maxTarget = curve.maxCapacity(func(capacity uint64) int64 {
				return balance.estimatePriority(capacity, 0, 0, bias, false)
			})
			if maxTarget < reqCap {
				return
			}
			maxTarget = reqCap

			// Specify a narrow target range that allows a limited number of fine step
			// iterations
			minTarget = maxTarget - maxTarget/20
			if minTarget < capacity {
				minTarget = capacity
			}
		} else {
			minTarget, maxTarget = reqCap, reqCap
		}
		if newCap := cp.requestCapacity(node, minTarget, maxTarget, bias); newCap >= minTarget && newCap <= maxTarget {
			capacity = newCap
			return
		}
		// we should be able to find the maximum allowed capacity in a few iterations
		log.Error("Unable to find maximum allowed capacity")
		err = ErrCantFindMaximum
	})
	return
}

// serveCapQuery serves a vflux capacity query. It receives multiple token amount values
// and a bias time value. For each given token amount it calculates the maximum achievable
// capacity in case the amount is added to the balance.
func (cp *ClientPool) serveCapQuery(id enode.ID, freeID string, data []byte) []byte {
	var req vflux.CapacityQueryReq
	if rlp.DecodeBytes(data, &req) != nil {
		return nil
	}
	if l := len(req.AddTokens); l == 0 || l > vflux.CapacityQueryMaxLen {
		return nil
	}
	result := make(vflux.CapacityQueryReply, len(req.AddTokens))
	if !cp.synced() {
		capacityQueryZeroMeter.Mark(1)
		reply, _ := rlp.EncodeToBytes(&result)
		return reply
	}

	bias := time.Second * time.Duration(req.Bias)
	cp.lock.RLock()
	if cp.connectedBias > bias {
		bias = cp.connectedBias
	}
	cp.lock.RUnlock()

	// use capacityCurve to answer request for multiple newly bought token amounts
	curve := cp.getCapacityCurve().exclude(id)
	cp.BalanceOperation(id, freeID, func(balance AtomicBalanceOperator) {
		pb, _ := balance.GetBalance()
		for i, addTokens := range req.AddTokens {
			add := addTokens.Int64()
			result[i] = curve.maxCapacity(func(capacity uint64) int64 {
				return balance.estimatePriority(capacity, add, 0, bias, false) / int64(capacity)
			})
			if add <= 0 && uint64(-add) >= pb && result[i] > cp.minCap {
				result[i] = cp.minCap
			}
			if result[i] < cp.minCap {
				result[i] = 0
			}
		}
	})
	// add first result to metrics (don't care about priority client multi-queries yet)
	if result[0] == 0 {
		capacityQueryZeroMeter.Mark(1)
	} else {
		capacityQueryNonZeroMeter.Mark(1)
	}
	reply, _ := rlp.EncodeToBytes(&result)
	return reply
}

// Handle implements Service
func (cp *ClientPool) Handle(id enode.ID, address string, name string, data []byte) []byte {
	switch name {
	case vflux.CapacityQueryName:
		return cp.serveCapQuery(id, address, data)
	default:
		return nil
	}
}
