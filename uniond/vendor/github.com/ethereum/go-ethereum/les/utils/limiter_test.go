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
	"crypto/rand"
	"testing"

	"github.com/ethereum/go-ethereum/p2p/enode"
)

const (
	ltTolerance = 0.03
	ltRounds    = 7
)

type (
	ltNode struct {
		addr, id         int
		value, exp       float64
		cost             uint
		reqRate          float64
		reqMax, runCount int
		lastTotalCost    uint

		served, dropped int
	}

	ltResult struct {
		node *ltNode
		ch   chan struct{}
	}

	limTest struct {
		limiter            *Limiter
		results            chan ltResult
		runCount           int
		expCost, totalCost uint
	}
)

func (lt *limTest) request(n *ltNode) {
	var (
		address string
		id      enode.ID
	)
	if n.addr >= 0 {
		address = string([]byte{byte(n.addr)})
	} else {
		var b [32]byte
		rand.Read(b[:])
		address = string(b[:])
	}
	if n.id >= 0 {
		id = enode.ID{byte(n.id)}
	} else {
		rand.Read(id[:])
	}
	lt.runCount++
	n.runCount++
	cch := lt.limiter.Add(id, address, n.value, n.cost)
	go func() {
		lt.results <- ltResult{n, <-cch}
	}()
}

func (lt *limTest) moreRequests(n *ltNode) {
	maxStart := int(float64(lt.totalCost-n.lastTotalCost) * n.reqRate)
	if maxStart != 0 {
		n.lastTotalCost = lt.totalCost
	}
	for n.reqMax > n.runCount && maxStart > 0 {
		lt.request(n)
		maxStart--
	}
}

func (lt *limTest) process() {
	res := <-lt.results
	lt.runCount--
	res.node.runCount--
	if res.ch != nil {
		res.node.served++
		if res.node.exp != 0 {
			lt.expCost += res.node.cost
		}
		lt.totalCost += res.node.cost
		close(res.ch)
	} else {
		res.node.dropped++
	}
}

func TestLimiter(t *testing.T) {
	limTests := [][]*ltNode{
		{ // one id from an individual address and two ids from a shared address
			{addr: 0, id: 0, value: 0, cost: 1, reqRate: 1, reqMax: 1, exp: 0.5},
			{addr: 1, id: 1, value: 0, cost: 1, reqRate: 1, reqMax: 1, exp: 0.25},
			{addr: 1, id: 2, value: 0, cost: 1, reqRate: 1, reqMax: 1, exp: 0.25},
		},
		{ // varying request costs
			{addr: 0, id: 0, value: 0, cost: 10, reqRate: 0.2, reqMax: 1, exp: 0.5},
			{addr: 1, id: 1, value: 0, cost: 3, reqRate: 0.5, reqMax: 1, exp: 0.25},
			{addr: 1, id: 2, value: 0, cost: 1, reqRate: 1, reqMax: 1, exp: 0.25},
		},
		{ // different request rate
			{addr: 0, id: 0, value: 0, cost: 1, reqRate: 2, reqMax: 2, exp: 0.5},
			{addr: 1, id: 1, value: 0, cost: 1, reqRate: 10, reqMax: 10, exp: 0.25},
			{addr: 1, id: 2, value: 0, cost: 1, reqRate: 1, reqMax: 1, exp: 0.25},
		},
		{ // adding value
			{addr: 0, id: 0, value: 3, cost: 1, reqRate: 1, reqMax: 1, exp: (0.5 + 0.3) / 2},
			{addr: 1, id: 1, value: 0, cost: 1, reqRate: 1, reqMax: 1, exp: 0.25 / 2},
			{addr: 1, id: 2, value: 7, cost: 1, reqRate: 1, reqMax: 1, exp: (0.25 + 0.7) / 2},
		},
		{ // DoS attack from a single address with a single id
			{addr: 0, id: 0, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 1, id: 1, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 2, id: 2, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 3, id: 3, value: 0, cost: 1, reqRate: 10, reqMax: 1000000000, exp: 0},
		},
		{ // DoS attack from a single address with different ids
			{addr: 0, id: 0, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 1, id: 1, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 2, id: 2, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 3, id: -1, value: 0, cost: 1, reqRate: 1, reqMax: 1000000000, exp: 0},
		},
		{ // DDoS attack from different addresses with a single id
			{addr: 0, id: 0, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 1, id: 1, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 2, id: 2, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: -1, id: 3, value: 0, cost: 1, reqRate: 1, reqMax: 1000000000, exp: 0},
		},
		{ // DDoS attack from different addresses with different ids
			{addr: 0, id: 0, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 1, id: 1, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: 2, id: 2, value: 1, cost: 1, reqRate: 1, reqMax: 1, exp: 0.3333},
			{addr: -1, id: -1, value: 0, cost: 1, reqRate: 1, reqMax: 1000000000, exp: 0},
		},
	}

	lt := &limTest{
		limiter: NewLimiter(100),
		results: make(chan ltResult),
	}
	for _, test := range limTests {
		lt.expCost, lt.totalCost = 0, 0
		iterCount := 10000
		for j := 0; j < ltRounds; j++ {
			// try to reach expected target range in multiple rounds with increasing iteration counts
			last := j == ltRounds-1
			for _, n := range test {
				lt.request(n)
			}
			for i := 0; i < iterCount; i++ {
				lt.process()
				for _, n := range test {
					lt.moreRequests(n)
				}
			}
			for lt.runCount > 0 {
				lt.process()
			}
			if spamRatio := 1 - float64(lt.expCost)/float64(lt.totalCost); spamRatio > 0.5*(1+ltTolerance) {
				t.Errorf("Spam ratio too high (%f)", spamRatio)
			}
			fail, success := false, true
			for _, n := range test {
				if n.exp != 0 {
					if n.dropped > 0 {
						t.Errorf("Dropped %d requests of non-spam node", n.dropped)
						fail = true
					}
					r := float64(n.served) * float64(n.cost) / float64(lt.expCost)
					if r < n.exp*(1-ltTolerance) || r > n.exp*(1+ltTolerance) {
						if last {
							// print error only if the target is still not reached in the last round
							t.Errorf("Request ratio (%f) does not match expected value (%f)", r, n.exp)
						}
						success = false
					}
				}
			}
			if fail || success {
				break
			}
			// neither failed nor succeeded; try more iterations to reach probability targets
			iterCount *= 2
		}
	}
	lt.limiter.Stop()
}
