// Copyright 2019 The go-ethereum Authors
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

package flowcontrol

import (
	"math"
	"math/rand"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
)

type testNode struct {
	node               *ClientNode
	bufLimit, capacity uint64
	waitUntil          mclock.AbsTime
	index, totalCost   uint64
}

const (
	testMaxCost = 1000000
	testLength  = 100000
)

// testConstantTotalCapacity simulates multiple request sender nodes and verifies
// whether the total amount of served requests matches the expected value based on
// the total capacity and the duration of the test.
// Some nodes are sending requests occasionally so that their buffer should regularly
// reach the maximum while other nodes (the "max capacity nodes") are sending at the
// maximum permitted rate. The max capacity nodes are changed multiple times during
// a single test.
func TestConstantTotalCapacity(t *testing.T) {
	testConstantTotalCapacity(t, 10, 1, 0, false)
	testConstantTotalCapacity(t, 10, 1, 1, false)
	testConstantTotalCapacity(t, 30, 1, 0, false)
	testConstantTotalCapacity(t, 30, 2, 3, false)
	testConstantTotalCapacity(t, 100, 1, 0, false)
	testConstantTotalCapacity(t, 100, 3, 5, false)
	testConstantTotalCapacity(t, 100, 5, 10, false)
	testConstantTotalCapacity(t, 100, 3, 5, true)
}

func testConstantTotalCapacity(t *testing.T, nodeCount, maxCapacityNodes, randomSend int, priorityOverflow bool) {
	clock := &mclock.Simulated{}
	nodes := make([]*testNode, nodeCount)
	var totalCapacity uint64
	for i := range nodes {
		nodes[i] = &testNode{capacity: uint64(50000 + rand.Intn(100000))}
		totalCapacity += nodes[i].capacity
	}
	m := NewClientManager(PieceWiseLinear{{0, totalCapacity}}, clock)
	if priorityOverflow {
		// provoke a situation where rcLastUpdate overflow needs to be handled
		m.rcLastIntValue = math.MaxInt64 - 10000000000
	}
	for _, n := range nodes {
		n.bufLimit = n.capacity * 6000
		n.node = NewClientNode(m, ServerParams{BufLimit: n.bufLimit, MinRecharge: n.capacity})
	}
	maxNodes := make([]int, maxCapacityNodes)
	for i := range maxNodes {
		// we don't care if some indexes are selected multiple times
		// in that case we have fewer max nodes
		maxNodes[i] = rand.Intn(nodeCount)
	}

	var sendCount int
	for i := 0; i < testLength; i++ {
		now := clock.Now()
		for _, idx := range maxNodes {
			for nodes[idx].send(t, now) {
			}
		}
		if rand.Intn(testLength) < maxCapacityNodes*3 {
			maxNodes[rand.Intn(maxCapacityNodes)] = rand.Intn(nodeCount)
		}

		sendCount += randomSend
		failCount := randomSend * 10
		for sendCount > 0 && failCount > 0 {
			if nodes[rand.Intn(nodeCount)].send(t, now) {
				sendCount--
			} else {
				failCount--
			}
		}
		clock.Run(time.Millisecond)
	}

	var totalCost uint64
	for _, n := range nodes {
		totalCost += n.totalCost
	}
	ratio := float64(totalCost) / float64(totalCapacity) / testLength
	if ratio < 0.98 || ratio > 1.02 {
		t.Errorf("totalCost/totalCapacity/testLength ratio incorrect (expected: 1, got: %f)", ratio)
	}
}

func (n *testNode) send(t *testing.T, now mclock.AbsTime) bool {
	if now < n.waitUntil {
		return false
	}
	n.index++
	if ok, _, _ := n.node.AcceptRequest(0, n.index, testMaxCost); !ok {
		t.Fatalf("Rejected request after expected waiting time has passed")
	}
	rcost := uint64(rand.Int63n(testMaxCost))
	bv := n.node.RequestProcessed(0, n.index, testMaxCost, rcost)
	if bv < testMaxCost {
		n.waitUntil = now + mclock.AbsTime((testMaxCost-bv)*1001000/n.capacity)
	}
	n.totalCost += rcost
	return true
}
