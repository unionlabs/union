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
	"reflect"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

var (
	testSetup     = &nodestate.Setup{}
	sfTest1       = testSetup.NewFlag("test1")
	sfTest2       = testSetup.NewFlag("test2")
	sfTest3       = testSetup.NewFlag("test3")
	sfTest4       = testSetup.NewFlag("test4")
	sfiTestWeight = testSetup.NewField("nodeWeight", reflect.TypeOf(uint64(0)))
)

const iterTestNodeCount = 6

func TestWrsIterator(t *testing.T) {
	ns := nodestate.NewNodeStateMachine(nil, nil, &mclock.Simulated{}, testSetup)
	w := NewWrsIterator(ns, sfTest2, sfTest3.Or(sfTest4), sfiTestWeight)
	ns.Start()
	for i := 1; i <= iterTestNodeCount; i++ {
		ns.SetState(testNode(i), sfTest1, nodestate.Flags{}, 0)
		ns.SetField(testNode(i), sfiTestWeight, uint64(1))
	}
	next := func() int {
		ch := make(chan struct{})
		go func() {
			w.Next()
			close(ch)
		}()
		select {
		case <-ch:
		case <-time.After(time.Second * 5):
			t.Fatalf("Iterator.Next() timeout")
		}
		node := w.Node()
		ns.SetState(node, sfTest4, nodestate.Flags{}, 0)
		return testNodeIndex(node.ID())
	}
	set := make(map[int]bool)
	expset := func() {
		for len(set) > 0 {
			n := next()
			if !set[n] {
				t.Errorf("Item returned by iterator not in the expected set (got %d)", n)
			}
			delete(set, n)
		}
	}

	ns.SetState(testNode(1), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(2), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(3), sfTest2, nodestate.Flags{}, 0)
	set[1] = true
	set[2] = true
	set[3] = true
	expset()
	ns.SetState(testNode(4), sfTest2, nodestate.Flags{}, 0)
	ns.SetState(testNode(5), sfTest2.Or(sfTest3), nodestate.Flags{}, 0)
	ns.SetState(testNode(6), sfTest2, nodestate.Flags{}, 0)
	set[4] = true
	set[6] = true
	expset()
	ns.SetField(testNode(2), sfiTestWeight, uint64(0))
	ns.SetState(testNode(1), nodestate.Flags{}, sfTest4, 0)
	ns.SetState(testNode(2), nodestate.Flags{}, sfTest4, 0)
	ns.SetState(testNode(3), nodestate.Flags{}, sfTest4, 0)
	set[1] = true
	set[3] = true
	expset()
	ns.SetField(testNode(2), sfiTestWeight, uint64(1))
	ns.SetState(testNode(2), nodestate.Flags{}, sfTest2, 0)
	ns.SetState(testNode(1), nodestate.Flags{}, sfTest4, 0)
	ns.SetState(testNode(2), sfTest2, sfTest4, 0)
	ns.SetState(testNode(3), nodestate.Flags{}, sfTest4, 0)
	set[1] = true
	set[2] = true
	set[3] = true
	expset()
	ns.Stop()
}
