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
	"sync"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

// QueueIterator returns nodes from the specified selectable set in the same order as
// they entered the set.
type QueueIterator struct {
	lock sync.Mutex
	cond *sync.Cond

	ns           *nodestate.NodeStateMachine
	queue        []*enode.Node
	nextNode     *enode.Node
	waitCallback func(bool)
	fifo, closed bool
}

// NewQueueIterator creates a new QueueIterator. Nodes are selectable if they have all the required
// and none of the disabled flags set. When a node is selected the selectedFlag is set which also
// disables further selectability until it is removed or times out.
func NewQueueIterator(ns *nodestate.NodeStateMachine, requireFlags, disableFlags nodestate.Flags, fifo bool, waitCallback func(bool)) *QueueIterator {
	qi := &QueueIterator{
		ns:           ns,
		fifo:         fifo,
		waitCallback: waitCallback,
	}
	qi.cond = sync.NewCond(&qi.lock)

	ns.SubscribeState(requireFlags.Or(disableFlags), func(n *enode.Node, oldState, newState nodestate.Flags) {
		oldMatch := oldState.HasAll(requireFlags) && oldState.HasNone(disableFlags)
		newMatch := newState.HasAll(requireFlags) && newState.HasNone(disableFlags)
		if newMatch == oldMatch {
			return
		}

		qi.lock.Lock()
		defer qi.lock.Unlock()

		if newMatch {
			qi.queue = append(qi.queue, n)
		} else {
			id := n.ID()
			for i, qn := range qi.queue {
				if qn.ID() == id {
					copy(qi.queue[i:len(qi.queue)-1], qi.queue[i+1:])
					qi.queue = qi.queue[:len(qi.queue)-1]
					break
				}
			}
		}
		qi.cond.Signal()
	})
	return qi
}

// Next moves to the next selectable node.
func (qi *QueueIterator) Next() bool {
	qi.lock.Lock()
	if !qi.closed && len(qi.queue) == 0 {
		if qi.waitCallback != nil {
			qi.waitCallback(true)
		}
		for !qi.closed && len(qi.queue) == 0 {
			qi.cond.Wait()
		}
		if qi.waitCallback != nil {
			qi.waitCallback(false)
		}
	}
	if qi.closed {
		qi.nextNode = nil
		qi.lock.Unlock()
		return false
	}
	// Move to the next node in queue.
	if qi.fifo {
		qi.nextNode = qi.queue[0]
		copy(qi.queue[:len(qi.queue)-1], qi.queue[1:])
		qi.queue = qi.queue[:len(qi.queue)-1]
	} else {
		qi.nextNode = qi.queue[len(qi.queue)-1]
		qi.queue = qi.queue[:len(qi.queue)-1]
	}
	qi.lock.Unlock()
	return true
}

// Close ends the iterator.
func (qi *QueueIterator) Close() {
	qi.lock.Lock()
	qi.closed = true
	qi.lock.Unlock()
	qi.cond.Signal()
}

// Node returns the current node.
func (qi *QueueIterator) Node() *enode.Node {
	qi.lock.Lock()
	defer qi.lock.Unlock()

	return qi.nextNode
}
