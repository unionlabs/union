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

package utils

import (
	"sync"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"golang.org/x/exp/slices"
)

const maxSelectionWeight = 1000000000 // maximum selection weight of each individual node/address group

// Limiter protects a network request serving mechanism from denial-of-service attacks.
// It limits the total amount of resources used for serving requests while ensuring that
// the most valuable connections always have a reasonable chance of being served.
type Limiter struct {
	lock sync.Mutex
	cond *sync.Cond
	quit bool

	nodes                          map[enode.ID]*nodeQueue
	addresses                      map[string]*addressGroup
	addressSelect, valueSelect     *WeightedRandomSelect
	maxValue                       float64
	maxCost, sumCost, sumCostLimit uint
	selectAddressNext              bool
}

// nodeQueue represents queued requests coming from a single node ID
type nodeQueue struct {
	queue                   []request // always nil if penaltyCost != 0
	id                      enode.ID
	address                 string
	value                   float64
	flatWeight, valueWeight uint64 // current selection weights in the address/value selectors
	sumCost                 uint   // summed cost of requests queued by the node
	penaltyCost             uint   // cumulative cost of dropped requests since last processed request
	groupIndex              int
}

// addressGroup is a group of node IDs that have sent their last requests from the same
// network address
type addressGroup struct {
	nodes                      []*nodeQueue
	nodeSelect                 *WeightedRandomSelect
	sumFlatWeight, groupWeight uint64
}

// request represents an incoming request scheduled for processing
type request struct {
	process chan chan struct{}
	cost    uint
}

// flatWeight distributes weights equally between each active network address
func flatWeight(item interface{}) uint64 { return item.(*nodeQueue).flatWeight }

// add adds the node queue to the address group. It is the caller's responsibility to
// add the address group to the address map and the address selector if it wasn't
// there before.
func (ag *addressGroup) add(nq *nodeQueue) {
	if nq.groupIndex != -1 {
		panic("added node queue is already in an address group")
	}
	l := len(ag.nodes)
	nq.groupIndex = l
	ag.nodes = append(ag.nodes, nq)
	ag.sumFlatWeight += nq.flatWeight
	ag.groupWeight = ag.sumFlatWeight / uint64(l+1)
	ag.nodeSelect.Update(ag.nodes[l])
}

// update updates the selection weight of the node queue inside the address group.
// It is the caller's responsibility to update the group's selection weight in the
// address selector.
func (ag *addressGroup) update(nq *nodeQueue, weight uint64) {
	if nq.groupIndex == -1 || nq.groupIndex >= len(ag.nodes) || ag.nodes[nq.groupIndex] != nq {
		panic("updated node queue is not in this address group")
	}
	ag.sumFlatWeight += weight - nq.flatWeight
	nq.flatWeight = weight
	ag.groupWeight = ag.sumFlatWeight / uint64(len(ag.nodes))
	ag.nodeSelect.Update(nq)
}

// remove removes the node queue from the address group. It is the caller's responsibility
// to remove the address group from the address map if it is empty.
func (ag *addressGroup) remove(nq *nodeQueue) {
	if nq.groupIndex == -1 || nq.groupIndex >= len(ag.nodes) || ag.nodes[nq.groupIndex] != nq {
		panic("removed node queue is not in this address group")
	}

	l := len(ag.nodes) - 1
	if nq.groupIndex != l {
		ag.nodes[nq.groupIndex] = ag.nodes[l]
		ag.nodes[nq.groupIndex].groupIndex = nq.groupIndex
	}
	nq.groupIndex = -1
	ag.nodes = ag.nodes[:l]
	ag.sumFlatWeight -= nq.flatWeight
	if l >= 1 {
		ag.groupWeight = ag.sumFlatWeight / uint64(l)
	} else {
		ag.groupWeight = 0
	}
	ag.nodeSelect.Remove(nq)
}

// choose selects one of the node queues belonging to the address group
func (ag *addressGroup) choose() *nodeQueue {
	return ag.nodeSelect.Choose().(*nodeQueue)
}

// NewLimiter creates a new Limiter
func NewLimiter(sumCostLimit uint) *Limiter {
	l := &Limiter{
		addressSelect: NewWeightedRandomSelect(func(item interface{}) uint64 { return item.(*addressGroup).groupWeight }),
		valueSelect:   NewWeightedRandomSelect(func(item interface{}) uint64 { return item.(*nodeQueue).valueWeight }),
		nodes:         make(map[enode.ID]*nodeQueue),
		addresses:     make(map[string]*addressGroup),
		sumCostLimit:  sumCostLimit,
	}
	l.cond = sync.NewCond(&l.lock)
	go l.processLoop()
	return l
}

// selectionWeights calculates the selection weights of a node for both the address and
// the value selector. The selection weight depends on the next request cost or the
// summed cost of recently dropped requests.
func (l *Limiter) selectionWeights(reqCost uint, value float64) (flatWeight, valueWeight uint64) {
	if value > l.maxValue {
		l.maxValue = value
	}
	if value > 0 {
		// normalize value to <= 1
		value /= l.maxValue
	}
	if reqCost > l.maxCost {
		l.maxCost = reqCost
	}
	relCost := float64(reqCost) / float64(l.maxCost)
	var f float64
	if relCost <= 0.001 {
		f = 1
	} else {
		f = 0.001 / relCost
	}
	f *= maxSelectionWeight
	flatWeight, valueWeight = uint64(f), uint64(f*value)
	if flatWeight == 0 {
		flatWeight = 1
	}
	return
}

// Add adds a new request to the node queue belonging to the given id. Value belongs
// to the requesting node. A higher value gives the request a higher chance of being
// served quickly in case of heavy load or a DDoS attack. Cost is a rough estimate
// of the serving cost of the request. A lower cost also gives the request a
// better chance.
func (l *Limiter) Add(id enode.ID, address string, value float64, reqCost uint) chan chan struct{} {
	l.lock.Lock()
	defer l.lock.Unlock()

	process := make(chan chan struct{}, 1)
	if l.quit {
		close(process)
		return process
	}
	if reqCost == 0 {
		reqCost = 1
	}
	if nq, ok := l.nodes[id]; ok {
		if nq.queue != nil {
			nq.queue = append(nq.queue, request{process, reqCost})
			nq.sumCost += reqCost
			nq.value = value
			if address != nq.address {
				// known id sending request from a new address, move to different address group
				l.removeFromGroup(nq)
				l.addToGroup(nq, address)
			}
		} else {
			// already waiting on a penalty, just add to the penalty cost and drop the request
			nq.penaltyCost += reqCost
			l.update(nq)
			close(process)
			return process
		}
	} else {
		nq := &nodeQueue{
			queue:      []request{{process, reqCost}},
			id:         id,
			value:      value,
			sumCost:    reqCost,
			groupIndex: -1,
		}
		nq.flatWeight, nq.valueWeight = l.selectionWeights(reqCost, value)
		if len(l.nodes) == 0 {
			l.cond.Signal()
		}
		l.nodes[id] = nq
		if nq.valueWeight != 0 {
			l.valueSelect.Update(nq)
		}
		l.addToGroup(nq, address)
	}
	l.sumCost += reqCost
	if l.sumCost > l.sumCostLimit {
		l.dropRequests()
	}
	return process
}

// update updates the selection weights of the node queue
func (l *Limiter) update(nq *nodeQueue) {
	var cost uint
	if nq.queue != nil {
		cost = nq.queue[0].cost
	} else {
		cost = nq.penaltyCost
	}
	flatWeight, valueWeight := l.selectionWeights(cost, nq.value)
	ag := l.addresses[nq.address]
	ag.update(nq, flatWeight)
	l.addressSelect.Update(ag)
	nq.valueWeight = valueWeight
	l.valueSelect.Update(nq)
}

// addToGroup adds the node queue to the given address group. The group is created if
// it does not exist yet.
func (l *Limiter) addToGroup(nq *nodeQueue, address string) {
	nq.address = address
	ag := l.addresses[address]
	if ag == nil {
		ag = &addressGroup{nodeSelect: NewWeightedRandomSelect(flatWeight)}
		l.addresses[address] = ag
	}
	ag.add(nq)
	l.addressSelect.Update(ag)
}

// removeFromGroup removes the node queue from its address group
func (l *Limiter) removeFromGroup(nq *nodeQueue) {
	ag := l.addresses[nq.address]
	ag.remove(nq)
	if len(ag.nodes) == 0 {
		delete(l.addresses, nq.address)
	}
	l.addressSelect.Update(ag)
}

// remove removes the node queue from its address group, the nodes map and the value
// selector
func (l *Limiter) remove(nq *nodeQueue) {
	l.removeFromGroup(nq)
	if nq.valueWeight != 0 {
		l.valueSelect.Remove(nq)
	}
	delete(l.nodes, nq.id)
}

// choose selects the next node queue to process.
func (l *Limiter) choose() *nodeQueue {
	if l.valueSelect.IsEmpty() || l.selectAddressNext {
		if ag, ok := l.addressSelect.Choose().(*addressGroup); ok {
			l.selectAddressNext = false
			return ag.choose()
		}
	}
	nq, _ := l.valueSelect.Choose().(*nodeQueue)
	l.selectAddressNext = true
	return nq
}

// processLoop processes requests sequentially
func (l *Limiter) processLoop() {
	l.lock.Lock()
	defer l.lock.Unlock()

	for {
		if l.quit {
			for _, nq := range l.nodes {
				for _, request := range nq.queue {
					close(request.process)
				}
			}
			return
		}
		nq := l.choose()
		if nq == nil {
			l.cond.Wait()
			continue
		}
		if nq.queue != nil {
			request := nq.queue[0]
			nq.queue = nq.queue[1:]
			nq.sumCost -= request.cost
			l.sumCost -= request.cost
			l.lock.Unlock()
			ch := make(chan struct{})
			request.process <- ch
			<-ch
			l.lock.Lock()
			if len(nq.queue) > 0 {
				l.update(nq)
			} else {
				l.remove(nq)
			}
		} else {
			// penalized queue removed, next request will be added to a clean queue
			l.remove(nq)
		}
	}
}

// Stop stops the processing loop. All queued and future requests are rejected.
func (l *Limiter) Stop() {
	l.lock.Lock()
	defer l.lock.Unlock()

	l.quit = true
	l.cond.Signal()
}

type dropListItem struct {
	nq       *nodeQueue
	priority float64
}

// dropRequests selects the nodes with the highest queued request cost to selection
// weight ratio and drops their queued request. The empty node queues stay in the
// selectors with a low selection weight in order to penalize these nodes.
func (l *Limiter) dropRequests() {
	var (
		sumValue float64
		list     []dropListItem
	)
	for _, nq := range l.nodes {
		sumValue += nq.value
	}
	for _, nq := range l.nodes {
		if nq.sumCost == 0 {
			continue
		}
		w := 1 / float64(len(l.addresses)*len(l.addresses[nq.address].nodes))
		if sumValue > 0 {
			w += nq.value / sumValue
		}
		list = append(list, dropListItem{
			nq:       nq,
			priority: w / float64(nq.sumCost),
		})
	}
	slices.SortFunc(list, func(a, b dropListItem) int {
		if a.priority < b.priority {
			return -1
		}
		if a.priority < b.priority {
			return 1
		}
		return 0
	})
	for _, item := range list {
		for _, request := range item.nq.queue {
			close(request.process)
		}
		// make the queue penalized; no more requests are accepted until the node is
		// selected based on the penalty cost which is the cumulative cost of all dropped
		// requests. This ensures that sending excess requests is always penalized
		// and incentivizes the sender to stop for a while if no replies are received.
		item.nq.queue = nil
		item.nq.penaltyCost = item.nq.sumCost
		l.sumCost -= item.nq.sumCost // penalty costs are not counted in sumCost
		item.nq.sumCost = 0
		l.update(item.nq)
		if l.sumCost <= l.sumCostLimit/2 {
			return
		}
	}
}
