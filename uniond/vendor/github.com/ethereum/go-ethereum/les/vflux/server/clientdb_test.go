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

package server

import (
	"reflect"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

func expval(v uint64) utils.ExpiredValue {
	return utils.ExpiredValue{Base: v}
}

func TestNodeDB(t *testing.T) {
	ndb := newNodeDB(rawdb.NewMemoryDatabase(), mclock.System{})
	defer ndb.close()

	var cases = []struct {
		id       enode.ID
		ip       string
		balance  utils.ExpiredValue
		positive bool
	}{
		{enode.ID{0x00, 0x01, 0x02}, "", expval(100), true},
		{enode.ID{0x00, 0x01, 0x02}, "", expval(200), true},
		{enode.ID{}, "127.0.0.1", expval(100), false},
		{enode.ID{}, "127.0.0.1", expval(200), false},
	}
	for _, c := range cases {
		if c.positive {
			ndb.setBalance(c.id.Bytes(), false, c.balance)
			if pb := ndb.getOrNewBalance(c.id.Bytes(), false); !reflect.DeepEqual(pb, c.balance) {
				t.Fatalf("Positive balance mismatch, want %v, got %v", c.balance, pb)
			}
		} else {
			ndb.setBalance([]byte(c.ip), true, c.balance)
			if nb := ndb.getOrNewBalance([]byte(c.ip), true); !reflect.DeepEqual(nb, c.balance) {
				t.Fatalf("Negative balance mismatch, want %v, got %v", c.balance, nb)
			}
		}
	}
	for _, c := range cases {
		if c.positive {
			ndb.delBalance(c.id.Bytes(), false)
			if pb := ndb.getOrNewBalance(c.id.Bytes(), false); !reflect.DeepEqual(pb, utils.ExpiredValue{}) {
				t.Fatalf("Positive balance mismatch, want %v, got %v", utils.ExpiredValue{}, pb)
			}
		} else {
			ndb.delBalance([]byte(c.ip), true)
			if nb := ndb.getOrNewBalance([]byte(c.ip), true); !reflect.DeepEqual(nb, utils.ExpiredValue{}) {
				t.Fatalf("Negative balance mismatch, want %v, got %v", utils.ExpiredValue{}, nb)
			}
		}
	}
	posExp, negExp := utils.Fixed64(1000), utils.Fixed64(2000)
	ndb.setExpiration(posExp, negExp)
	if pos, neg := ndb.getExpiration(); pos != posExp || neg != negExp {
		t.Fatalf("Expiration mismatch, want %v / %v, got %v / %v", posExp, negExp, pos, neg)
	}
	/*	curBalance := currencyBalance{typ: "ETH", amount: 10000}
		ndb.setCurrencyBalance(enode.ID{0x01, 0x02}, curBalance)
		if got := ndb.getCurrencyBalance(enode.ID{0x01, 0x02}); !reflect.DeepEqual(got, curBalance) {
			t.Fatalf("Currency balance mismatch, want %v, got %v", curBalance, got)
		}*/
}

func TestNodeDBExpiration(t *testing.T) {
	var (
		iterated int
		done     = make(chan struct{}, 1)
	)
	callback := func(now mclock.AbsTime, neg bool, b utils.ExpiredValue) bool {
		iterated += 1
		return true
	}
	clock := &mclock.Simulated{}
	ndb := newNodeDB(rawdb.NewMemoryDatabase(), clock)
	defer ndb.close()
	ndb.evictCallBack = callback
	ndb.cleanupHook = func() { done <- struct{}{} }

	var cases = []struct {
		id      []byte
		neg     bool
		balance utils.ExpiredValue
	}{
		{[]byte{0x01, 0x02}, false, expval(1)},
		{[]byte{0x03, 0x04}, false, expval(1)},
		{[]byte{0x05, 0x06}, false, expval(1)},
		{[]byte{0x07, 0x08}, false, expval(1)},

		{[]byte("127.0.0.1"), true, expval(1)},
		{[]byte("127.0.0.2"), true, expval(1)},
		{[]byte("127.0.0.3"), true, expval(1)},
		{[]byte("127.0.0.4"), true, expval(1)},
	}
	for _, c := range cases {
		ndb.setBalance(c.id, c.neg, c.balance)
	}
	clock.WaitForTimers(1)
	clock.Run(time.Hour + time.Minute)
	select {
	case <-done:
	case <-time.NewTimer(time.Second).C:
		t.Fatalf("timeout")
	}
	if iterated != 8 {
		t.Fatalf("Failed to evict useless balances, want %v, got %d", 8, iterated)
	}

	for _, c := range cases {
		ndb.setBalance(c.id, c.neg, c.balance)
	}
	clock.WaitForTimers(1)
	clock.Run(time.Hour + time.Minute)
	select {
	case <-done:
	case <-time.NewTimer(time.Second).C:
		t.Fatalf("timeout")
	}
	if iterated != 16 {
		t.Fatalf("Failed to evict useless balances, want %v, got %d", 16, iterated)
	}
}
