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

package utils

import (
	"math"
	"math/rand"

	"github.com/ethereum/go-ethereum/log"
)

type (
	// WeightedRandomSelect is capable of weighted random selection from a set of items
	WeightedRandomSelect struct {
		root *wrsNode
		idx  map[WrsItem]int
		wfn  WeightFn
	}
	WrsItem  interface{}
	WeightFn func(interface{}) uint64
)

// NewWeightedRandomSelect returns a new WeightedRandomSelect structure
func NewWeightedRandomSelect(wfn WeightFn) *WeightedRandomSelect {
	return &WeightedRandomSelect{root: &wrsNode{maxItems: wrsBranches}, idx: make(map[WrsItem]int), wfn: wfn}
}

// Update updates an item's weight, adds it if it was non-existent or removes it if
// the new weight is zero. Note that explicitly updating decreasing weights is not necessary.
func (w *WeightedRandomSelect) Update(item WrsItem) {
	w.setWeight(item, w.wfn(item))
}

// Remove removes an item from the set
func (w *WeightedRandomSelect) Remove(item WrsItem) {
	w.setWeight(item, 0)
}

// IsEmpty returns true if the set is empty
func (w *WeightedRandomSelect) IsEmpty() bool {
	return w.root.sumCost == 0
}

// setWeight sets an item's weight to a specific value (removes it if zero)
func (w *WeightedRandomSelect) setWeight(item WrsItem, weight uint64) {
	if weight > math.MaxInt64-w.root.sumCost {
		// old weight is still included in sumCost, remove and check again
		w.setWeight(item, 0)
		if weight > math.MaxInt64-w.root.sumCost {
			log.Error("WeightedRandomSelect overflow", "sumCost", w.root.sumCost, "new weight", weight)
			weight = math.MaxInt64 - w.root.sumCost
		}
	}
	idx, ok := w.idx[item]
	if ok {
		w.root.setWeight(idx, weight)
		if weight == 0 {
			delete(w.idx, item)
		}
	} else {
		if weight != 0 {
			if w.root.itemCnt == w.root.maxItems {
				// add a new level
				newRoot := &wrsNode{sumCost: w.root.sumCost, itemCnt: w.root.itemCnt, level: w.root.level + 1, maxItems: w.root.maxItems * wrsBranches}
				newRoot.items[0] = w.root
				newRoot.weights[0] = w.root.sumCost
				w.root = newRoot
			}
			w.idx[item] = w.root.insert(item, weight)
		}
	}
}

// Choose randomly selects an item from the set, with a chance proportional to its
// current weight. If the weight of the chosen element has been decreased since the
// last stored value, returns it with a newWeight/oldWeight chance, otherwise just
// updates its weight and selects another one
func (w *WeightedRandomSelect) Choose() WrsItem {
	for {
		if w.root.sumCost == 0 {
			return nil
		}
		val := uint64(rand.Int63n(int64(w.root.sumCost)))
		choice, lastWeight := w.root.choose(val)
		weight := w.wfn(choice)
		if weight != lastWeight {
			w.setWeight(choice, weight)
		}
		if weight >= lastWeight || uint64(rand.Int63n(int64(lastWeight))) < weight {
			return choice
		}
	}
}

const wrsBranches = 8 // max number of branches in the wrsNode tree

// wrsNode is a node of a tree structure that can store WrsItems or further wrsNodes.
type wrsNode struct {
	items                    [wrsBranches]interface{}
	weights                  [wrsBranches]uint64
	sumCost                  uint64
	level, itemCnt, maxItems int
}

// insert recursively inserts a new item to the tree and returns the item index
func (n *wrsNode) insert(item WrsItem, weight uint64) int {
	branch := 0
	for n.items[branch] != nil && (n.level == 0 || n.items[branch].(*wrsNode).itemCnt == n.items[branch].(*wrsNode).maxItems) {
		branch++
		if branch == wrsBranches {
			panic(nil)
		}
	}
	n.itemCnt++
	n.sumCost += weight
	n.weights[branch] += weight
	if n.level == 0 {
		n.items[branch] = item
		return branch
	}
	var subNode *wrsNode
	if n.items[branch] == nil {
		subNode = &wrsNode{maxItems: n.maxItems / wrsBranches, level: n.level - 1}
		n.items[branch] = subNode
	} else {
		subNode = n.items[branch].(*wrsNode)
	}
	subIdx := subNode.insert(item, weight)
	return subNode.maxItems*branch + subIdx
}

// setWeight updates the weight of a certain item (which should exist) and returns
// the change of the last weight value stored in the tree
func (n *wrsNode) setWeight(idx int, weight uint64) uint64 {
	if n.level == 0 {
		oldWeight := n.weights[idx]
		n.weights[idx] = weight
		diff := weight - oldWeight
		n.sumCost += diff
		if weight == 0 {
			n.items[idx] = nil
			n.itemCnt--
		}
		return diff
	}
	branchItems := n.maxItems / wrsBranches
	branch := idx / branchItems
	diff := n.items[branch].(*wrsNode).setWeight(idx-branch*branchItems, weight)
	n.weights[branch] += diff
	n.sumCost += diff
	if weight == 0 {
		n.itemCnt--
	}
	return diff
}

// choose recursively selects an item from the tree and returns it along with its weight
func (n *wrsNode) choose(val uint64) (WrsItem, uint64) {
	for i, w := range n.weights {
		if val < w {
			if n.level == 0 {
				return n.items[i].(WrsItem), n.weights[i]
			}
			return n.items[i].(*wrsNode).choose(val)
		}
		val -= w
	}
	panic(nil)
}
