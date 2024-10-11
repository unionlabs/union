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
	"math/rand"
	"testing"

	"github.com/ethereum/go-ethereum/les/utils"
)

func checkU64(t *testing.T, name string, value, exp uint64) {
	if value != exp {
		t.Errorf("Incorrect value for %s: got %d, expected %d", name, value, exp)
	}
}

func checkF64(t *testing.T, name string, value, exp, tol float64) {
	if value < exp-tol || value > exp+tol {
		t.Errorf("Incorrect value for %s: got %f, expected %f", name, value, exp)
	}
}

func TestServerBasket(t *testing.T) {
	var s serverBasket
	s.init(2)
	// add some requests with different request value factors
	s.updateRvFactor(1)
	noexp := utils.ExpirationFactor{Factor: 1}
	s.add(0, 1000, 10000, noexp)
	s.add(1, 3000, 60000, noexp)
	s.updateRvFactor(10)
	s.add(0, 4000, 4000, noexp)
	s.add(1, 2000, 4000, noexp)
	s.updateRvFactor(10)
	// check basket contents directly
	checkU64(t, "s.basket[0].amount", s.basket.items[0].amount, 5000*basketFactor)
	checkU64(t, "s.basket[0].value", s.basket.items[0].value, 50000)
	checkU64(t, "s.basket[1].amount", s.basket.items[1].amount, 5000*basketFactor)
	checkU64(t, "s.basket[1].value", s.basket.items[1].value, 100000)
	// transfer 50% of the contents of the basket
	transfer1 := s.transfer(0.5)
	checkU64(t, "transfer1[0].amount", transfer1.items[0].amount, 2500*basketFactor)
	checkU64(t, "transfer1[0].value", transfer1.items[0].value, 25000)
	checkU64(t, "transfer1[1].amount", transfer1.items[1].amount, 2500*basketFactor)
	checkU64(t, "transfer1[1].value", transfer1.items[1].value, 50000)
	// add more requests
	s.updateRvFactor(100)
	s.add(0, 1000, 100, noexp)
	// transfer 25% of the contents of the basket
	transfer2 := s.transfer(0.25)
	checkU64(t, "transfer2[0].amount", transfer2.items[0].amount, (2500+1000)/4*basketFactor)
	checkU64(t, "transfer2[0].value", transfer2.items[0].value, (25000+10000)/4)
	checkU64(t, "transfer2[1].amount", transfer2.items[1].amount, 2500/4*basketFactor)
	checkU64(t, "transfer2[1].value", transfer2.items[1].value, 50000/4)
}

func TestConvertMapping(t *testing.T) {
	b := requestBasket{items: []basketItem{{3, 3}, {1, 1}, {2, 2}}}
	oldMap := []string{"req3", "req1", "req2"}
	newMap := []string{"req1", "req2", "req3", "req4"}
	init := requestBasket{items: []basketItem{{2, 2}, {4, 4}, {6, 6}, {8, 8}}}
	bc := b.convertMapping(oldMap, newMap, init)
	checkU64(t, "bc[0].amount", bc.items[0].amount, 1)
	checkU64(t, "bc[1].amount", bc.items[1].amount, 2)
	checkU64(t, "bc[2].amount", bc.items[2].amount, 3)
	checkU64(t, "bc[3].amount", bc.items[3].amount, 4) // 8 should be scaled down to 4
}

func TestReqValueFactor(t *testing.T) {
	var ref referenceBasket
	ref.basket = requestBasket{items: make([]basketItem, 4)}
	for i := range ref.basket.items {
		ref.basket.items[i].amount = uint64(i+1) * basketFactor
		ref.basket.items[i].value = uint64(i+1) * basketFactor
	}
	ref.init(4)
	rvf := ref.reqValueFactor([]uint64{1000, 2000, 3000, 4000})
	// expected value is (1000000+2000000+3000000+4000000) / (1*1000+2*2000+3*3000+4*4000) = 10000000/30000 = 333.333
	checkF64(t, "reqValueFactor", rvf, 333.333, 1)
}

func TestNormalize(t *testing.T) {
	for cycle := 0; cycle < 100; cycle += 1 {
		// Initialize data for testing
		valueRange, lower := 1000000, 1000000
		ref := referenceBasket{basket: requestBasket{items: make([]basketItem, 10)}}
		for i := 0; i < 10; i++ {
			ref.basket.items[i].amount = uint64(rand.Intn(valueRange) + lower)
			ref.basket.items[i].value = uint64(rand.Intn(valueRange) + lower)
		}
		ref.normalize()

		// Check whether SUM(amount) ~= SUM(value)
		var sumAmount, sumValue uint64
		for i := 0; i < 10; i++ {
			sumAmount += ref.basket.items[i].amount
			sumValue += ref.basket.items[i].value
		}
		var epsilon = 0.01
		if float64(sumAmount)*(1+epsilon) < float64(sumValue) || float64(sumAmount)*(1-epsilon) > float64(sumValue) {
			t.Fatalf("Failed to normalize sumAmount: %d sumValue: %d", sumAmount, sumValue)
		}
	}
}

func TestReqValueAdjustment(t *testing.T) {
	var s1, s2 serverBasket
	s1.init(3)
	s2.init(3)
	cost1 := []uint64{30000, 60000, 90000}
	cost2 := []uint64{100000, 200000, 300000}
	var ref referenceBasket
	ref.basket = requestBasket{items: make([]basketItem, 3)}
	for i := range ref.basket.items {
		ref.basket.items[i].amount = 123 * basketFactor
		ref.basket.items[i].value = 123 * basketFactor
	}
	ref.init(3)
	// initial reqValues are expected to be {1, 1, 1}
	checkF64(t, "reqValues[0]", ref.reqValues[0], 1, 0.01)
	checkF64(t, "reqValues[1]", ref.reqValues[1], 1, 0.01)
	checkF64(t, "reqValues[2]", ref.reqValues[2], 1, 0.01)
	var logOffset utils.Fixed64
	for period := 0; period < 1000; period++ {
		exp := utils.ExpFactor(logOffset)
		s1.updateRvFactor(ref.reqValueFactor(cost1))
		s2.updateRvFactor(ref.reqValueFactor(cost2))
		// throw in random requests into each basket using their internal pricing
		for i := 0; i < 1000; i++ {
			reqType, reqAmount := uint32(rand.Intn(3)), uint32(rand.Intn(10)+1)
			reqCost := uint64(reqAmount) * cost1[reqType]
			s1.add(reqType, reqAmount, reqCost, exp)
			reqType, reqAmount = uint32(rand.Intn(3)), uint32(rand.Intn(10)+1)
			reqCost = uint64(reqAmount) * cost2[reqType]
			s2.add(reqType, reqAmount, reqCost, exp)
		}
		ref.add(s1.transfer(0.1))
		ref.add(s2.transfer(0.1))
		ref.normalize()
		ref.updateReqValues()
		logOffset += utils.Float64ToFixed64(0.1)
	}
	checkF64(t, "reqValues[0]", ref.reqValues[0], 0.5, 0.01)
	checkF64(t, "reqValues[1]", ref.reqValues[1], 1, 0.01)
	checkF64(t, "reqValues[2]", ref.reqValues[2], 1.5, 0.01)
}
