// Copyright 2019 The go-ethereum Authors
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

package p2p

import (
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
)

func TestExpHeap(t *testing.T) {
	var h expHeap

	var (
		basetime = mclock.AbsTime(10)
		exptimeA = basetime.Add(2 * time.Second)
		exptimeB = basetime.Add(3 * time.Second)
		exptimeC = basetime.Add(4 * time.Second)
	)
	h.add("b", exptimeB)
	h.add("a", exptimeA)
	h.add("c", exptimeC)

	if h.nextExpiry() != exptimeA {
		t.Fatal("wrong nextExpiry")
	}
	if !h.contains("a") || !h.contains("b") || !h.contains("c") {
		t.Fatal("heap doesn't contain all live items")
	}

	h.expire(exptimeA.Add(1), nil)
	if h.nextExpiry() != exptimeB {
		t.Fatal("wrong nextExpiry")
	}
	if h.contains("a") {
		t.Fatal("heap contains a even though it has already expired")
	}
	if !h.contains("b") || !h.contains("c") {
		t.Fatal("heap doesn't contain all live items")
	}
}
