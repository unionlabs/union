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
	"math"
	"math/rand"
	"strconv"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/ethdb/memorydb"
	"github.com/ethereum/go-ethereum/p2p/enode"

	"github.com/ethereum/go-ethereum/les/utils"
)

const (
	testReqTypes  = 3
	testNodeCount = 5
	testReqCount  = 10000
	testRounds    = 10
)

func TestValueTracker(t *testing.T) {
	db := memorydb.New()
	clock := &mclock.Simulated{}
	requestList := make([]RequestInfo, testReqTypes)
	relPrices := make([]float64, testReqTypes)
	totalAmount := make([]uint64, testReqTypes)
	for i := range requestList {
		requestList[i] = RequestInfo{Name: "testreq" + strconv.Itoa(i), InitAmount: 1, InitValue: 1}
		totalAmount[i] = 1
		relPrices[i] = rand.Float64() + 0.1
	}
	nodes := make([]*NodeValueTracker, testNodeCount)
	for round := 0; round < testRounds; round++ {
		makeRequests := round < testRounds-2
		useExpiration := round == testRounds-1
		var expRate float64
		if useExpiration {
			expRate = math.Log(2) / float64(time.Hour*100)
		}

		vt := NewValueTracker(db, clock, requestList, time.Minute, 1/float64(time.Hour), expRate, expRate)
		updateCosts := func(i int) {
			costList := make([]uint64, testReqTypes)
			baseCost := rand.Float64()*10000000 + 100000
			for j := range costList {
				costList[j] = uint64(baseCost * relPrices[j])
			}
			nodes[i].UpdateCosts(costList)
		}
		for i := range nodes {
			nodes[i] = vt.Register(enode.ID{byte(i)})
			updateCosts(i)
		}
		if makeRequests {
			for i := 0; i < testReqCount; i++ {
				reqType := rand.Intn(testReqTypes)
				reqAmount := rand.Intn(10) + 1
				node := rand.Intn(testNodeCount)
				respTime := time.Duration((rand.Float64() + 1) * float64(time.Second) * float64(node+1) / testNodeCount)
				totalAmount[reqType] += uint64(reqAmount)
				nodes[node].Served([]ServedRequest{{uint32(reqType), uint32(reqAmount)}}, respTime)
				clock.Run(time.Second)
			}
		} else {
			clock.Run(time.Hour * 100)
			if useExpiration {
				for i, a := range totalAmount {
					totalAmount[i] = a / 2
				}
			}
		}
		vt.Stop()
		var sumrp, sumrv float64
		for i, rp := range relPrices {
			sumrp += rp
			sumrv += vt.refBasket.reqValues[i]
		}
		for i, rp := range relPrices {
			ratio := vt.refBasket.reqValues[i] * sumrp / (rp * sumrv)
			if ratio < 0.99 || ratio > 1.01 {
				t.Errorf("reqValues (%v) does not match relPrices (%v)", vt.refBasket.reqValues, relPrices)
				break
			}
		}
		exp := utils.ExpFactor(vt.StatsExpirer().LogOffset(clock.Now()))
		basketAmount := make([]uint64, testReqTypes)
		for i, bi := range vt.refBasket.basket.items {
			basketAmount[i] += uint64(exp.Value(float64(bi.amount), vt.refBasket.basket.exp))
		}
		if makeRequests {
			// if we did not make requests in this round then we expect all amounts to be
			// in the reference basket
			for _, node := range nodes {
				for i, bi := range node.basket.basket.items {
					basketAmount[i] += uint64(exp.Value(float64(bi.amount), node.basket.basket.exp))
				}
			}
		}
		for i, a := range basketAmount {
			amount := a / basketFactor
			if amount+10 < totalAmount[i] || amount > totalAmount[i]+10 {
				t.Errorf("totalAmount[%d] mismatch in round %d (expected %d, got %d)", i, round, totalAmount[i], amount)
			}
		}
		var sumValue float64
		for _, node := range nodes {
			s := node.RtStats()
			sumValue += s.Value(maxResponseWeights, exp)
		}
		s := vt.RtStats()
		mainValue := s.Value(maxResponseWeights, exp)
		if sumValue < mainValue-10 || sumValue > mainValue+10 {
			t.Errorf("Main rtStats value does not match sum of node rtStats values in round %d (main %v, sum %v)", round, mainValue, sumValue)
		}
	}
}
