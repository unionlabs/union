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

	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

// WrsIterator returns nodes from the specified selectable set with a weighted random
// selection. Selection weights are provided by a callback function.
type WrsIterator struct {
	lock sync.Mutex
	cond *sync.Cond

	ns       *nodestate.NodeStateMachine
	wrs      *utils.WeightedRandomSelect
	nextNode *enode.Node
	closed   bool
}

// NewWrsIterator creates a new WrsIterator. Nodes are selectable if they have all the required
// and none of the disabled flags set. When a node is selected the selectedFlag is set which also
// disables further selectability until it is removed or times out.
func NewWrsIterator(ns *nodestate.NodeStateMachine, requireFlags, disableFlags nodestate.Flags, weightField nodestate.Field) *WrsIterator {
	wfn := func(i interface{}) uint64 {
		n := ns.GetNode(i.(enode.ID))
		if n == nil {
			return 0
		}
		wt, _ := ns.GetField(n, weightField).(uint64)
		return wt
	}

	w := &WrsIterator{
		ns:  ns,
		wrs: utils.NewWeightedRandomSelect(wfn),
	}
	w.cond = sync.NewCond(&w.lock)

	ns.SubscribeField(weightField, func(n *enode.Node, state nodestate.Flags, oldValue, newValue interface{}) {
		if state.HasAll(requireFlags) && state.HasNone(disableFlags) {
			w.lock.Lock()
			w.wrs.Update(n.ID())
			w.lock.Unlock()
			w.cond.Signal()
		}
	})

	ns.SubscribeState(requireFlags.Or(disableFlags), func(n *enode.Node, oldState, newState nodestate.Flags) {
		oldMatch := oldState.HasAll(requireFlags) && oldState.HasNone(disableFlags)
		newMatch := newState.HasAll(requireFlags) && newState.HasNone(disableFlags)
		if newMatch == oldMatch {
			return
		}

		w.lock.Lock()
		if newMatch {
			w.wrs.Update(n.ID())
		} else {
			w.wrs.Remove(n.ID())
		}
		w.lock.Unlock()
		w.cond.Signal()
	})
	return w
}

// Next selects the next node.
func (w *WrsIterator) Next() bool {
	w.nextNode = w.chooseNode()
	return w.nextNode != nil
}

func (w *WrsIterator) chooseNode() *enode.Node {
	w.lock.Lock()
	defer w.lock.Unlock()

	for {
		for !w.closed && w.wrs.IsEmpty() {
			w.cond.Wait()
		}
		if w.closed {
			return nil
		}
		// Choose the next node at random. Even though w.wrs is guaranteed
		// non-empty here, Choose might return nil if all items have weight
		// zero.
		if c := w.wrs.Choose(); c != nil {
			id := c.(enode.ID)
			w.wrs.Remove(id)
			return w.ns.GetNode(id)
		}
	}
}

// Close ends the iterator.
func (w *WrsIterator) Close() {
	w.lock.Lock()
	w.closed = true
	w.lock.Unlock()
	w.cond.Signal()
}

// Node returns the current node.
func (w *WrsIterator) Node() *enode.Node {
	w.lock.Lock()
	defer w.lock.Unlock()
	return w.nextNode
}
