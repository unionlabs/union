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
	"math/rand"
	"reflect"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

const (
	testCapacityStepDiv      = 100
	testCapacityToleranceDiv = 10
	testMinCap               = 100
)

type ppTestClient struct {
	node         *enode.Node
	balance, cap uint64
}

func (c *ppTestClient) priority(cap uint64) int64 {
	return int64(c.balance / cap)
}

func (c *ppTestClient) estimatePriority(cap uint64, addBalance int64, future, bias time.Duration, update bool) int64 {
	return int64(c.balance / cap)
}

func TestPriorityPool(t *testing.T) {
	clock := &mclock.Simulated{}
	setup := newServerSetup()
	setup.balanceField = setup.setup.NewField("ppTestClient", reflect.TypeOf(&ppTestClient{}))
	ns := nodestate.NewNodeStateMachine(nil, nil, clock, setup.setup)

	ns.SubscribeField(setup.capacityField, func(node *enode.Node, state nodestate.Flags, oldValue, newValue interface{}) {
		if n := ns.GetField(node, setup.balanceField); n != nil {
			c := n.(*ppTestClient)
			c.cap = newValue.(uint64)
		}
	})
	pp := newPriorityPool(ns, setup, clock, testMinCap, 0, testCapacityStepDiv, testCapacityStepDiv)
	ns.Start()
	pp.SetLimits(100, 1000000)
	clients := make([]*ppTestClient, 100)
	raise := func(c *ppTestClient) {
		for {
			var ok bool
			ns.Operation(func() {
				newCap := c.cap + c.cap/testCapacityStepDiv
				ok = pp.requestCapacity(c.node, newCap, newCap, 0) == newCap
			})
			if !ok {
				return
			}
		}
	}
	var sumBalance uint64
	check := func(c *ppTestClient) {
		expCap := 1000000 * c.balance / sumBalance
		capTol := expCap / testCapacityToleranceDiv
		if c.cap < expCap-capTol || c.cap > expCap+capTol {
			t.Errorf("Wrong node capacity (expected %d, got %d)", expCap, c.cap)
		}
	}

	for i := range clients {
		c := &ppTestClient{
			node:    enode.SignNull(&enr.Record{}, enode.ID{byte(i)}),
			balance: 100000000000,
			cap:     1000,
		}
		sumBalance += c.balance
		clients[i] = c
		ns.SetField(c.node, setup.balanceField, c)
		ns.SetState(c.node, setup.inactiveFlag, nodestate.Flags{}, 0)
		raise(c)
		check(c)
	}

	for count := 0; count < 100; count++ {
		c := clients[rand.Intn(len(clients))]
		oldBalance := c.balance
		c.balance = uint64(rand.Int63n(100000000000) + 100000000000)
		sumBalance += c.balance - oldBalance
		pp.ns.SetState(c.node, setup.updateFlag, nodestate.Flags{}, 0)
		pp.ns.SetState(c.node, nodestate.Flags{}, setup.updateFlag, 0)
		if c.balance > oldBalance {
			raise(c)
		} else {
			for _, c := range clients {
				raise(c)
			}
		}
		// check whether capacities are proportional to balances
		for _, c := range clients {
			check(c)
		}
		if count%10 == 0 {
			// test available capacity calculation with capacity curve
			c = clients[rand.Intn(len(clients))]
			curve := pp.getCapacityCurve().exclude(c.node.ID())

			add := uint64(rand.Int63n(10000000000000))
			c.balance += add
			sumBalance += add
			expCap := curve.maxCapacity(func(cap uint64) int64 {
				return int64(c.balance / cap)
			})
			var ok bool
			expFail := expCap + 10
			if expFail < testMinCap {
				expFail = testMinCap
			}
			ns.Operation(func() {
				ok = pp.requestCapacity(c.node, expFail, expFail, 0) == expFail
			})
			if ok {
				t.Errorf("Request for more than expected available capacity succeeded")
			}
			if expCap >= testMinCap {
				ns.Operation(func() {
					ok = pp.requestCapacity(c.node, expCap, expCap, 0) == expCap
				})
				if !ok {
					t.Errorf("Request for expected available capacity failed")
				}
			}
			c.balance -= add
			sumBalance -= add
			pp.ns.SetState(c.node, setup.updateFlag, nodestate.Flags{}, 0)
			pp.ns.SetState(c.node, nodestate.Flags{}, setup.updateFlag, 0)
			for _, c := range clients {
				raise(c)
			}
		}
	}

	ns.Stop()
}

func TestCapacityCurve(t *testing.T) {
	clock := &mclock.Simulated{}
	setup := newServerSetup()
	setup.balanceField = setup.setup.NewField("ppTestClient", reflect.TypeOf(&ppTestClient{}))
	ns := nodestate.NewNodeStateMachine(nil, nil, clock, setup.setup)

	pp := newPriorityPool(ns, setup, clock, 400000, 0, 2, 2)
	ns.Start()
	pp.SetLimits(10, 10000000)
	clients := make([]*ppTestClient, 10)

	for i := range clients {
		c := &ppTestClient{
			node:    enode.SignNull(&enr.Record{}, enode.ID{byte(i)}),
			balance: 100000000000 * uint64(i+1),
			cap:     1000000,
		}
		clients[i] = c
		ns.SetField(c.node, setup.balanceField, c)
		ns.SetState(c.node, setup.inactiveFlag, nodestate.Flags{}, 0)
		ns.Operation(func() {
			pp.requestCapacity(c.node, c.cap, c.cap, 0)
		})
	}

	curve := pp.getCapacityCurve()
	check := func(balance, expCap uint64) {
		cap := curve.maxCapacity(func(cap uint64) int64 {
			return int64(balance / cap)
		})
		var fail bool
		if cap == 0 || expCap == 0 {
			fail = cap != expCap
		} else {
			pri := balance / cap
			expPri := balance / expCap
			fail = pri != expPri && pri != expPri+1
		}
		if fail {
			t.Errorf("Incorrect capacity for %d balance (got %d, expected %d)", balance, cap, expCap)
		}
	}

	check(0, 0)
	check(10000000000, 100000)
	check(50000000000, 500000)
	check(100000000000, 1000000)
	check(200000000000, 1000000)
	check(300000000000, 1500000)
	check(450000000000, 1500000)
	check(600000000000, 2000000)
	check(800000000000, 2000000)
	check(1000000000000, 2500000)

	pp.SetLimits(11, 10000000)
	curve = pp.getCapacityCurve()

	check(0, 0)
	check(10000000000, 100000)
	check(50000000000, 500000)
	check(150000000000, 750000)
	check(200000000000, 1000000)
	check(220000000000, 1100000)
	check(275000000000, 1100000)
	check(375000000000, 1500000)
	check(450000000000, 1500000)
	check(600000000000, 2000000)
	check(800000000000, 2000000)
	check(1000000000000, 2500000)

	ns.Stop()
}
