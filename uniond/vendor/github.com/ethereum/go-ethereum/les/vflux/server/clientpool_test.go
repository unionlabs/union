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

package server

import (
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

const defaultConnectedBias = time.Minute * 3

func TestClientPoolL10C100Free(t *testing.T) {
	testClientPool(t, 10, 100, 0, true)
}

func TestClientPoolL40C200Free(t *testing.T) {
	testClientPool(t, 40, 200, 0, true)
}

func TestClientPoolL100C300Free(t *testing.T) {
	testClientPool(t, 100, 300, 0, true)
}

func TestClientPoolL10C100P4(t *testing.T) {
	testClientPool(t, 10, 100, 4, false)
}

func TestClientPoolL40C200P30(t *testing.T) {
	testClientPool(t, 40, 200, 30, false)
}

func TestClientPoolL100C300P20(t *testing.T) {
	testClientPool(t, 100, 300, 20, false)
}

const testClientPoolTicks = 100000

type poolTestPeer struct {
	node            *enode.Node
	index           int
	disconnCh       chan int
	cap             uint64
	inactiveAllowed bool
}

func newPoolTestPeer(i int, disconnCh chan int) *poolTestPeer {
	return &poolTestPeer{
		index:     i,
		disconnCh: disconnCh,
		node:      enode.SignNull(&enr.Record{}, enode.ID{byte(i % 256), byte(i >> 8)}),
	}
}

func (i *poolTestPeer) Node() *enode.Node {
	return i.node
}

func (i *poolTestPeer) FreeClientId() string {
	return fmt.Sprintf("addr #%d", i.index)
}

func (i *poolTestPeer) InactiveAllowance() time.Duration {
	if i.inactiveAllowed {
		return time.Second * 10
	}
	return 0
}

func (i *poolTestPeer) UpdateCapacity(capacity uint64, requested bool) {
	i.cap = capacity
}

func (i *poolTestPeer) Disconnect() {
	if i.disconnCh == nil {
		return
	}
	id := i.node.ID()
	i.disconnCh <- int(id[0]) + int(id[1])<<8
}

func getBalance(pool *ClientPool, p *poolTestPeer) (pos, neg uint64) {
	pool.BalanceOperation(p.node.ID(), p.FreeClientId(), func(nb AtomicBalanceOperator) {
		pos, neg = nb.GetBalance()
	})
	return
}

func addBalance(pool *ClientPool, id enode.ID, amount int64) {
	pool.BalanceOperation(id, "", func(nb AtomicBalanceOperator) {
		nb.AddBalance(amount)
	})
}

func checkDiff(a, b uint64) bool {
	maxDiff := (a + b) / 2000
	if maxDiff < 1 {
		maxDiff = 1
	}
	return a > b+maxDiff || b > a+maxDiff
}

func connect(pool *ClientPool, peer *poolTestPeer) uint64 {
	pool.Register(peer)
	return peer.cap
}

func disconnect(pool *ClientPool, peer *poolTestPeer) {
	pool.Unregister(peer)
}

func alwaysTrueFn() bool {
	return true
}

func testClientPool(t *testing.T, activeLimit, clientCount, paidCount int, randomDisconnect bool) {
	var (
		clock     mclock.Simulated
		db        = rawdb.NewMemoryDatabase()
		connected = make([]bool, clientCount)
		connTicks = make([]int, clientCount)
		disconnCh = make(chan int, clientCount)
		pool      = NewClientPool(db, 1, 0, &clock, alwaysTrueFn)
	)
	pool.Start()
	pool.SetExpirationTCs(0, 1000)

	pool.SetLimits(uint64(activeLimit), uint64(activeLimit))
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	// pool should accept new peers up to its connected limit
	for i := 0; i < activeLimit; i++ {
		if cap := connect(pool, newPoolTestPeer(i, disconnCh)); cap != 0 {
			connected[i] = true
		} else {
			t.Fatalf("Test peer #%d rejected", i)
		}
	}
	// randomly connect and disconnect peers, expect to have a similar total connection time at the end
	for tickCounter := 0; tickCounter < testClientPoolTicks; tickCounter++ {
		clock.Run(1 * time.Second)

		if tickCounter == testClientPoolTicks/4 {
			// give a positive balance to some of the peers
			amount := testClientPoolTicks / 2 * int64(time.Second) // enough for half of the simulation period
			for i := 0; i < paidCount; i++ {
				addBalance(pool, newPoolTestPeer(i, disconnCh).node.ID(), amount)
			}
		}

		i := rand.Intn(clientCount)
		if connected[i] {
			if randomDisconnect {
				disconnect(pool, newPoolTestPeer(i, disconnCh))
				connected[i] = false
				connTicks[i] += tickCounter
			}
		} else {
			if cap := connect(pool, newPoolTestPeer(i, disconnCh)); cap != 0 {
				connected[i] = true
				connTicks[i] -= tickCounter
			} else {
				disconnect(pool, newPoolTestPeer(i, disconnCh))
			}
		}
	pollDisconnects:
		for {
			select {
			case i := <-disconnCh:
				disconnect(pool, newPoolTestPeer(i, disconnCh))
				if connected[i] {
					connTicks[i] += tickCounter
					connected[i] = false
				}
			default:
				break pollDisconnects
			}
		}
	}

	expTicks := testClientPoolTicks/2*activeLimit/clientCount + testClientPoolTicks/2*(activeLimit-paidCount)/(clientCount-paidCount)
	expMin := expTicks - expTicks/5
	expMax := expTicks + expTicks/5
	paidTicks := testClientPoolTicks/2*activeLimit/clientCount + testClientPoolTicks/2
	paidMin := paidTicks - paidTicks/5
	paidMax := paidTicks + paidTicks/5

	// check if the total connected time of peers are all in the expected range
	for i, c := range connected {
		if c {
			connTicks[i] += testClientPoolTicks
		}
		min, max := expMin, expMax
		if i < paidCount {
			// expect a higher amount for clients with a positive balance
			min, max = paidMin, paidMax
		}
		if connTicks[i] < min || connTicks[i] > max {
			t.Errorf("Total connected time of test node #%d (%d) outside expected range (%d to %d)", i, connTicks[i], min, max)
		}
	}
	pool.Stop()
}

func testPriorityConnect(t *testing.T, pool *ClientPool, p *poolTestPeer, cap uint64, expSuccess bool) {
	if cap := connect(pool, p); cap == 0 {
		if expSuccess {
			t.Fatalf("Failed to connect paid client")
		} else {
			return
		}
	}
	if newCap, _ := pool.SetCapacity(p.node, cap, defaultConnectedBias, true); newCap != cap {
		if expSuccess {
			t.Fatalf("Failed to raise capacity of paid client")
		} else {
			return
		}
	}
	if !expSuccess {
		t.Fatalf("Should reject high capacity paid client")
	}
}

func TestConnectPaidClient(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10))
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	// Add balance for an external client and mark it as paid client
	addBalance(pool, newPoolTestPeer(0, nil).node.ID(), int64(time.Minute))
	testPriorityConnect(t, pool, newPoolTestPeer(0, nil), 10, true)
}

func TestConnectPaidClientToSmallPool(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	// Add balance for an external client and mark it as paid client
	addBalance(pool, newPoolTestPeer(0, nil).node.ID(), int64(time.Minute))

	// connect a fat paid client to pool, should reject it.
	testPriorityConnect(t, pool, newPoolTestPeer(0, nil), 100, false)
}

func TestConnectPaidClientToFullPool(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	for i := 0; i < 10; i++ {
		addBalance(pool, newPoolTestPeer(i, nil).node.ID(), int64(time.Second*20))
		connect(pool, newPoolTestPeer(i, nil))
	}
	addBalance(pool, newPoolTestPeer(11, nil).node.ID(), int64(time.Second*2)) // Add low balance to new paid client
	if cap := connect(pool, newPoolTestPeer(11, nil)); cap != 0 {
		t.Fatalf("Low balance paid client should be rejected")
	}
	clock.Run(time.Second)
	addBalance(pool, newPoolTestPeer(12, nil).node.ID(), int64(time.Minute*5)) // Add high balance to new paid client
	if cap := connect(pool, newPoolTestPeer(12, nil)); cap == 0 {
		t.Fatalf("High balance paid client should be accepted")
	}
}

func TestPaidClientKickedOut(t *testing.T) {
	var (
		clock    mclock.Simulated
		db       = rawdb.NewMemoryDatabase()
		kickedCh = make(chan int, 100)
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	pool.SetExpirationTCs(0, 0)
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	for i := 0; i < 10; i++ {
		addBalance(pool, newPoolTestPeer(i, kickedCh).node.ID(), 10000000000) // 10 second allowance
		connect(pool, newPoolTestPeer(i, kickedCh))
		clock.Run(time.Millisecond)
	}
	clock.Run(defaultConnectedBias + time.Second*11)
	if cap := connect(pool, newPoolTestPeer(11, kickedCh)); cap == 0 {
		t.Fatalf("Free client should be accepted")
	}
	clock.Run(0)
	select {
	case id := <-kickedCh:
		if id != 0 {
			t.Fatalf("Kicked client mismatch, want %v, got %v", 0, id)
		}
	default:
		t.Fatalf("timeout")
	}
}

func TestConnectFreeClient(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10))
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})
	if cap := connect(pool, newPoolTestPeer(0, nil)); cap == 0 {
		t.Fatalf("Failed to connect free client")
	}
	testPriorityConnect(t, pool, newPoolTestPeer(0, nil), 2, false)
}

func TestConnectFreeClientToFullPool(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	for i := 0; i < 10; i++ {
		connect(pool, newPoolTestPeer(i, nil))
	}
	if cap := connect(pool, newPoolTestPeer(11, nil)); cap != 0 {
		t.Fatalf("New free client should be rejected")
	}
	clock.Run(time.Minute)
	if cap := connect(pool, newPoolTestPeer(12, nil)); cap != 0 {
		t.Fatalf("New free client should be rejected")
	}
	clock.Run(time.Millisecond)
	clock.Run(4 * time.Minute)
	if cap := connect(pool, newPoolTestPeer(13, nil)); cap == 0 {
		t.Fatalf("Old client connects more than 5min should be kicked")
	}
}

func TestFreeClientKickedOut(t *testing.T) {
	var (
		clock  mclock.Simulated
		db     = rawdb.NewMemoryDatabase()
		kicked = make(chan int, 100)
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	for i := 0; i < 10; i++ {
		connect(pool, newPoolTestPeer(i, kicked))
		clock.Run(time.Millisecond)
	}
	if cap := connect(pool, newPoolTestPeer(10, kicked)); cap != 0 {
		t.Fatalf("New free client should be rejected")
	}
	clock.Run(0)
	select {
	case <-kicked:
	default:
		t.Fatalf("timeout")
	}
	disconnect(pool, newPoolTestPeer(10, kicked))
	clock.Run(5 * time.Minute)
	for i := 0; i < 10; i++ {
		connect(pool, newPoolTestPeer(i+10, kicked))
	}
	clock.Run(0)

	for i := 0; i < 10; i++ {
		select {
		case id := <-kicked:
			if id >= 10 {
				t.Fatalf("Old client should be kicked, now got: %d", id)
			}
		default:
			t.Fatalf("timeout")
		}
	}
}

func TestPositiveBalanceCalculation(t *testing.T) {
	var (
		clock  mclock.Simulated
		db     = rawdb.NewMemoryDatabase()
		kicked = make(chan int, 10)
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	addBalance(pool, newPoolTestPeer(0, kicked).node.ID(), int64(time.Minute*3))
	testPriorityConnect(t, pool, newPoolTestPeer(0, kicked), 10, true)
	clock.Run(time.Minute)

	disconnect(pool, newPoolTestPeer(0, kicked))
	pb, _ := getBalance(pool, newPoolTestPeer(0, kicked))
	if checkDiff(pb, uint64(time.Minute*2)) {
		t.Fatalf("Positive balance mismatch, want %v, got %v", uint64(time.Minute*2), pb)
	}
}

func TestDowngradePriorityClient(t *testing.T) {
	var (
		clock  mclock.Simulated
		db     = rawdb.NewMemoryDatabase()
		kicked = make(chan int, 10)
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 1})

	p := newPoolTestPeer(0, kicked)
	addBalance(pool, p.node.ID(), int64(time.Minute))
	testPriorityConnect(t, pool, p, 10, true)
	if p.cap != 10 {
		t.Fatalf("The capacity of priority peer hasn't been updated, got: %d", p.cap)
	}

	clock.Run(time.Minute)             // All positive balance should be used up.
	time.Sleep(300 * time.Millisecond) // Ensure the callback is called
	if p.cap != 1 {
		t.Fatalf("The capcacity of peer should be downgraded, got: %d", p.cap)
	}
	pb, _ := getBalance(pool, newPoolTestPeer(0, kicked))
	if pb != 0 {
		t.Fatalf("Positive balance mismatch, want %v, got %v", 0, pb)
	}

	addBalance(pool, newPoolTestPeer(0, kicked).node.ID(), int64(time.Minute))
	pb, _ = getBalance(pool, newPoolTestPeer(0, kicked))
	if checkDiff(pb, uint64(time.Minute)) {
		t.Fatalf("Positive balance mismatch, want %v, got %v", uint64(time.Minute), pb)
	}
}

func TestNegativeBalanceCalculation(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetExpirationTCs(0, 3600)
	pool.SetLimits(10, uint64(10)) // Total capacity limit is 10
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1e-3, CapacityFactor: 0, RequestFactor: 1}, PriceFactors{TimeFactor: 1e-3, CapacityFactor: 0, RequestFactor: 1})

	for i := 0; i < 10; i++ {
		connect(pool, newPoolTestPeer(i, nil))
	}
	clock.Run(time.Second)

	for i := 0; i < 10; i++ {
		disconnect(pool, newPoolTestPeer(i, nil))
		_, nb := getBalance(pool, newPoolTestPeer(i, nil))
		if nb != 0 {
			t.Fatalf("Short connection shouldn't be recorded")
		}
	}
	for i := 0; i < 10; i++ {
		connect(pool, newPoolTestPeer(i, nil))
	}
	clock.Run(time.Minute)
	for i := 0; i < 10; i++ {
		disconnect(pool, newPoolTestPeer(i, nil))
		_, nb := getBalance(pool, newPoolTestPeer(i, nil))
		exp := uint64(time.Minute) / 1000
		exp -= exp / 120 // correct for negative balance expiration
		if checkDiff(nb, exp) {
			t.Fatalf("Negative balance mismatch, want %v, got %v", exp, nb)
		}
	}
}

func TestInactiveClient(t *testing.T) {
	var (
		clock mclock.Simulated
		db    = rawdb.NewMemoryDatabase()
	)
	pool := NewClientPool(db, 1, defaultConnectedBias, &clock, alwaysTrueFn)
	pool.Start()
	defer pool.Stop()
	pool.SetLimits(2, uint64(2))

	p1 := newPoolTestPeer(1, nil)
	p1.inactiveAllowed = true
	p2 := newPoolTestPeer(2, nil)
	p2.inactiveAllowed = true
	p3 := newPoolTestPeer(3, nil)
	p3.inactiveAllowed = true
	addBalance(pool, p1.node.ID(), 1000*int64(time.Second))
	addBalance(pool, p3.node.ID(), 2000*int64(time.Second))
	// p1: 1000  p2: 0  p3: 2000
	p1.cap = connect(pool, p1)
	if p1.cap != 1 {
		t.Fatalf("Failed to connect peer #1")
	}
	p2.cap = connect(pool, p2)
	if p2.cap != 1 {
		t.Fatalf("Failed to connect peer #2")
	}
	p3.cap = connect(pool, p3)
	if p3.cap != 1 {
		t.Fatalf("Failed to connect peer #3")
	}
	if p2.cap != 0 {
		t.Fatalf("Failed to deactivate peer #2")
	}
	addBalance(pool, p2.node.ID(), 3000*int64(time.Second))
	// p1: 1000  p2: 3000  p3: 2000
	if p2.cap != 1 {
		t.Fatalf("Failed to activate peer #2")
	}
	if p1.cap != 0 {
		t.Fatalf("Failed to deactivate peer #1")
	}
	addBalance(pool, p2.node.ID(), -2500*int64(time.Second))
	// p1: 1000  p2: 500  p3: 2000
	if p1.cap != 1 {
		t.Fatalf("Failed to activate peer #1")
	}
	if p2.cap != 0 {
		t.Fatalf("Failed to deactivate peer #2")
	}
	pool.SetDefaultFactors(PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 0}, PriceFactors{TimeFactor: 1, CapacityFactor: 0, RequestFactor: 0})
	p4 := newPoolTestPeer(4, nil)
	addBalance(pool, p4.node.ID(), 1500*int64(time.Second))
	// p1: 1000  p2: 500  p3: 2000  p4: 1500
	p4.cap = connect(pool, p4)
	if p4.cap != 1 {
		t.Fatalf("Failed to activate peer #4")
	}
	if p1.cap != 0 {
		t.Fatalf("Failed to deactivate peer #1")
	}
	clock.Run(time.Second * 600)
	// manually trigger a check to avoid a long real-time wait
	pool.ns.SetState(p1.node, pool.setup.updateFlag, nodestate.Flags{}, 0)
	pool.ns.SetState(p1.node, nodestate.Flags{}, pool.setup.updateFlag, 0)
	// p1: 1000  p2: 500  p3: 2000  p4: 900
	if p1.cap != 1 {
		t.Fatalf("Failed to activate peer #1")
	}
	if p4.cap != 0 {
		t.Fatalf("Failed to deactivate peer #4")
	}
	disconnect(pool, p2)
	disconnect(pool, p4)
	addBalance(pool, p1.node.ID(), -1000*int64(time.Second))
	if p1.cap != 1 {
		t.Fatalf("Should not deactivate peer #1")
	}
	if p2.cap != 0 {
		t.Fatalf("Should not activate peer #2")
	}
}
