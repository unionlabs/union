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
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

const (
	posThreshold             = 1000000         // minimum positive balance that is persisted in the database
	negThreshold             = 1000000         // minimum negative balance that is persisted in the database
	persistExpirationRefresh = time.Minute * 5 // refresh period of the token expiration persistence
)

// balanceTracker tracks positive and negative balances for connected nodes.
// After clientField is set externally, a nodeBalance is created and previous
// balance values are loaded from the database. Both balances are exponentially expired
// values. Costs are deducted from the positive balance if present, otherwise added to
// the negative balance. If the capacity is non-zero then a time cost is applied
// continuously while individual request costs are applied immediately.
// The two balances are translated into a single priority value that also depends
// on the actual capacity.
type balanceTracker struct {
	setup          *serverSetup
	clock          mclock.Clock
	lock           sync.Mutex
	ns             *nodestate.NodeStateMachine
	ndb            *nodeDB
	posExp, negExp utils.ValueExpirer

	posExpTC, negExpTC                   uint64
	defaultPosFactors, defaultNegFactors PriceFactors

	active, inactive utils.ExpiredValue
	balanceTimer     *utils.UpdateTimer
	quit             chan struct{}
}

// newBalanceTracker creates a new balanceTracker
func newBalanceTracker(ns *nodestate.NodeStateMachine, setup *serverSetup, db ethdb.KeyValueStore, clock mclock.Clock, posExp, negExp utils.ValueExpirer) *balanceTracker {
	ndb := newNodeDB(db, clock)
	bt := &balanceTracker{
		ns:           ns,
		setup:        setup,
		ndb:          ndb,
		clock:        clock,
		posExp:       posExp,
		negExp:       negExp,
		balanceTimer: utils.NewUpdateTimer(clock, time.Second*10),
		quit:         make(chan struct{}),
	}
	posOffset, negOffset := bt.ndb.getExpiration()
	posExp.SetLogOffset(clock.Now(), posOffset)
	negExp.SetLogOffset(clock.Now(), negOffset)

	// Load all persisted balance entries of priority nodes,
	// calculate the total number of issued service tokens.
	bt.ndb.forEachBalance(false, func(id enode.ID, balance utils.ExpiredValue) bool {
		bt.inactive.AddExp(balance)
		return true
	})

	ns.SubscribeField(bt.setup.capacityField, func(node *enode.Node, state nodestate.Flags, oldValue, newValue interface{}) {
		n, _ := ns.GetField(node, bt.setup.balanceField).(*nodeBalance)
		if n == nil {
			return
		}

		ov, _ := oldValue.(uint64)
		nv, _ := newValue.(uint64)
		if ov == 0 && nv != 0 {
			n.activate()
		}
		if nv != 0 {
			n.setCapacity(nv)
		}
		if ov != 0 && nv == 0 {
			n.deactivate()
		}
	})
	ns.SubscribeField(bt.setup.clientField, func(node *enode.Node, state nodestate.Flags, oldValue, newValue interface{}) {
		type peer interface {
			FreeClientId() string
		}
		if newValue != nil {
			n := bt.newNodeBalance(node, newValue.(peer).FreeClientId(), true)
			bt.lock.Lock()
			n.SetPriceFactors(bt.defaultPosFactors, bt.defaultNegFactors)
			bt.lock.Unlock()
			ns.SetFieldSub(node, bt.setup.balanceField, n)
		} else {
			ns.SetStateSub(node, nodestate.Flags{}, bt.setup.priorityFlag, 0)
			if b, _ := ns.GetField(node, bt.setup.balanceField).(*nodeBalance); b != nil {
				b.deactivate()
			}
			ns.SetFieldSub(node, bt.setup.balanceField, nil)
		}
	})

	// The positive and negative balances of clients are stored in database
	// and both of these decay exponentially over time. Delete them if the
	// value is small enough.
	bt.ndb.evictCallBack = bt.canDropBalance

	go func() {
		for {
			select {
			case <-clock.After(persistExpirationRefresh):
				now := clock.Now()
				bt.ndb.setExpiration(posExp.LogOffset(now), negExp.LogOffset(now))
			case <-bt.quit:
				return
			}
		}
	}()
	return bt
}

// Stop saves expiration offset and unsaved node balances and shuts balanceTracker down
func (bt *balanceTracker) stop() {
	now := bt.clock.Now()
	bt.ndb.setExpiration(bt.posExp.LogOffset(now), bt.negExp.LogOffset(now))
	close(bt.quit)
	bt.ns.ForEach(nodestate.Flags{}, nodestate.Flags{}, func(node *enode.Node, state nodestate.Flags) {
		if n, ok := bt.ns.GetField(node, bt.setup.balanceField).(*nodeBalance); ok {
			n.lock.Lock()
			n.storeBalance(true, true)
			n.lock.Unlock()
			bt.ns.SetField(node, bt.setup.balanceField, nil)
		}
	})
	bt.ndb.close()
}

// TotalTokenAmount returns the current total amount of service tokens in existence
func (bt *balanceTracker) TotalTokenAmount() uint64 {
	bt.lock.Lock()
	defer bt.lock.Unlock()

	bt.balanceTimer.Update(func(_ time.Duration) bool {
		bt.active = utils.ExpiredValue{}
		bt.ns.ForEach(nodestate.Flags{}, nodestate.Flags{}, func(node *enode.Node, state nodestate.Flags) {
			if n, ok := bt.ns.GetField(node, bt.setup.balanceField).(*nodeBalance); ok && n.active {
				pos, _ := n.GetRawBalance()
				bt.active.AddExp(pos)
			}
		})
		return true
	})
	total := bt.active
	total.AddExp(bt.inactive)
	return total.Value(bt.posExp.LogOffset(bt.clock.Now()))
}

// GetPosBalanceIDs lists node IDs with an associated positive balance
func (bt *balanceTracker) GetPosBalanceIDs(start, stop enode.ID, maxCount int) (result []enode.ID) {
	return bt.ndb.getPosBalanceIDs(start, stop, maxCount)
}

// SetDefaultFactors sets the default price factors applied to subsequently connected clients
func (bt *balanceTracker) SetDefaultFactors(posFactors, negFactors PriceFactors) {
	bt.lock.Lock()
	bt.defaultPosFactors = posFactors
	bt.defaultNegFactors = negFactors
	bt.lock.Unlock()
}

// SetExpirationTCs sets positive and negative token expiration time constants.
// Specified in seconds, 0 means infinite (no expiration).
func (bt *balanceTracker) SetExpirationTCs(pos, neg uint64) {
	bt.lock.Lock()
	defer bt.lock.Unlock()

	bt.posExpTC, bt.negExpTC = pos, neg
	now := bt.clock.Now()
	if pos > 0 {
		bt.posExp.SetRate(now, 1/float64(pos*uint64(time.Second)))
	} else {
		bt.posExp.SetRate(now, 0)
	}
	if neg > 0 {
		bt.negExp.SetRate(now, 1/float64(neg*uint64(time.Second)))
	} else {
		bt.negExp.SetRate(now, 0)
	}
}

// GetExpirationTCs returns the current positive and negative token expiration
// time constants
func (bt *balanceTracker) GetExpirationTCs() (pos, neg uint64) {
	bt.lock.Lock()
	defer bt.lock.Unlock()

	return bt.posExpTC, bt.negExpTC
}

// BalanceOperation allows atomic operations on the balance of a node regardless of whether
// it is currently connected or not
func (bt *balanceTracker) BalanceOperation(id enode.ID, connAddress string, cb func(AtomicBalanceOperator)) {
	bt.ns.Operation(func() {
		var nb *nodeBalance
		if node := bt.ns.GetNode(id); node != nil {
			nb, _ = bt.ns.GetField(node, bt.setup.balanceField).(*nodeBalance)
		}
		if nb == nil {
			node := enode.SignNull(&enr.Record{}, id)
			nb = bt.newNodeBalance(node, connAddress, false)
		}
		cb(nb)
	})
}

// newNodeBalance loads balances from the database and creates a nodeBalance instance
// for the given node. It also sets the priorityFlag and adds balanceCallbackZero if
// the node has a positive balance.
// Note: this function should run inside a NodeStateMachine operation
func (bt *balanceTracker) newNodeBalance(node *enode.Node, connAddress string, setFlags bool) *nodeBalance {
	pb := bt.ndb.getOrNewBalance(node.ID().Bytes(), false)
	nb := bt.ndb.getOrNewBalance([]byte(connAddress), true)
	n := &nodeBalance{
		bt:          bt,
		node:        node,
		setFlags:    setFlags,
		connAddress: connAddress,
		balance:     balance{pos: pb, neg: nb, posExp: bt.posExp, negExp: bt.negExp},
		initTime:    bt.clock.Now(),
		lastUpdate:  bt.clock.Now(),
	}
	for i := range n.callbackIndex {
		n.callbackIndex[i] = -1
	}
	if setFlags && n.checkPriorityStatus() {
		n.bt.ns.SetStateSub(n.node, n.bt.setup.priorityFlag, nodestate.Flags{}, 0)
	}
	return n
}

// storeBalance stores either a positive or a negative balance in the database
func (bt *balanceTracker) storeBalance(id []byte, neg bool, value utils.ExpiredValue) {
	if bt.canDropBalance(bt.clock.Now(), neg, value) {
		bt.ndb.delBalance(id, neg) // balance is small enough, drop it directly.
	} else {
		bt.ndb.setBalance(id, neg, value)
	}
}

// canDropBalance tells whether a positive or negative balance is below the threshold
// and therefore can be dropped from the database
func (bt *balanceTracker) canDropBalance(now mclock.AbsTime, neg bool, b utils.ExpiredValue) bool {
	if neg {
		return b.Value(bt.negExp.LogOffset(now)) <= negThreshold
	}
	return b.Value(bt.posExp.LogOffset(now)) <= posThreshold
}

// updateTotalBalance adjusts the total balance after executing given callback.
func (bt *balanceTracker) updateTotalBalance(n *nodeBalance, callback func() bool) {
	bt.lock.Lock()
	defer bt.lock.Unlock()

	n.lock.Lock()
	defer n.lock.Unlock()

	original, active := n.balance.pos, n.active
	if !callback() {
		return
	}
	if active {
		bt.active.SubExp(original)
	} else {
		bt.inactive.SubExp(original)
	}
	if n.active {
		bt.active.AddExp(n.balance.pos)
	} else {
		bt.inactive.AddExp(n.balance.pos)
	}
}
