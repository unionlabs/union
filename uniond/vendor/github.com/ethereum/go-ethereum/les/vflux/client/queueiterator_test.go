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

package client

import (
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

func testNode(i int) *enode.Node {
	return enode.SignNull(new(enr.Record), testNodeID(i))
}

func TestQueueIteratorFIFO(t *testing.T) {
	testQueueIterator(t, true)
}

func TestQueueIteratorLIFO(t *testing.T) {
	testQueueIterator(t, false)
}

func testQueueIterator(t *testing.T, fifo bool) {
	ns := nodestate.NewNodeStateMachine(nil, nil, &mclock.Simulated{}, testSetup)
	qi := NewQueueIterator(ns, sfTest2, sfTest3.Or(sfTest4), fifo, nil)
	ns.Start()
	for i := 1; i <= iterTestNodeCount; i++ {
		ns.SetState(testNode(i), sfTest1, nodestate.Flags{}, 0)
	}
	next := func() int {
		ch := make(chan struct{})
		go func() {
			qi.Next()
			close(ch)
		}()
		select {
		case <-ch:
		case <-time.After(time.Second * 5):
			t.Fatalf("Iterator.Next() timeout")
		}
		node := qi.Node()
		ns.SetState(node, sfTest4, nodestate.Flags{}, 0)
		return testNodeIndex(node.ID())
	}
	exp := func(i int) {
		n := next()
		if n != i {
			t.Errorf("Wrong item returned by iterator (expected %d, got %d)", i, n)
		}
	}
	explist := func(list []int) {
		for i := range list {
			if fifo {
				exp(list[i])
			} else {
				exp(list[len(list)-1-i])
			}
		}
	}

	ns.SetState(testNode(1), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(2), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(3), sfTest2, nodestate.Flags{}, 0)
	explist([]int{1, 2, 3})
	ns.SetState(testNode(4), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(5), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(6), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(5), sfTest3, nodestate.Flags{}, 0)
	explist([]int{4, 6})
	ns.SetState(testNode(1), nodestate.Flags{}, sfTest4, 0)
	ns.SetState(testNode(2), nodestate.Flags{}, sfTest4, 0)
	ns.SetState(testNode(3), nodestate.Flags{}, sfTest4, 0)
	ns.SetState(testNode(2), sfTest3, nodestate.Flags{}, 0)
	ns.SetState(testNode(2), nodestate.Flags{}, sfTest3, 0)
	explist([]int{1, 3, 2})
	ns.Stop()
}
