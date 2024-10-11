// Copyright 2015 The go-ethereum Authors
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

package downloader

import (
	"fmt"
	"math/big"
	"math/rand"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/trie"
)

// makeChain creates a chain of n blocks starting at and including parent.
// the returned hash chain is ordered head->parent. In addition, every 3rd block
// contains a transaction and every 5th an uncle to allow testing correct block
// reassembly.
func makeChain(n int, seed byte, parent *types.Block, empty bool) ([]*types.Block, []types.Receipts) {
	blocks, receipts := core.GenerateChain(params.TestChainConfig, parent, ethash.NewFaker(), testDB, n, func(i int, block *core.BlockGen) {
		block.SetCoinbase(common.Address{seed})
		// Add one tx to every secondblock
		if !empty && i%2 == 0 {
			signer := types.MakeSigner(params.TestChainConfig, block.Number(), block.Timestamp())
			tx, err := types.SignTx(types.NewTransaction(block.TxNonce(testAddress), common.Address{seed}, big.NewInt(1000), params.TxGas, block.BaseFee(), nil), signer, testKey)
			if err != nil {
				panic(err)
			}
			block.AddTx(tx)
		}
	})
	return blocks, receipts
}

type chainData struct {
	blocks []*types.Block
	offset int
}

var chain *chainData
var emptyChain *chainData

func init() {
	// Create a chain of blocks to import
	targetBlocks := 128
	blocks, _ := makeChain(targetBlocks, 0, testGenesis, false)
	chain = &chainData{blocks, 0}

	blocks, _ = makeChain(targetBlocks, 0, testGenesis, true)
	emptyChain = &chainData{blocks, 0}
}

func (chain *chainData) headers() []*types.Header {
	hdrs := make([]*types.Header, len(chain.blocks))
	for i, b := range chain.blocks {
		hdrs[i] = b.Header()
	}
	return hdrs
}

func (chain *chainData) Len() int {
	return len(chain.blocks)
}

func dummyPeer(id string) *peerConnection {
	p := &peerConnection{
		id:      id,
		lacking: make(map[common.Hash]struct{}),
	}
	return p
}

func TestBasics(t *testing.T) {
	numOfBlocks := len(emptyChain.blocks)
	numOfReceipts := len(emptyChain.blocks) / 2

	q := newQueue(10, 10)
	if !q.Idle() {
		t.Errorf("new queue should be idle")
	}
	q.Prepare(1, SnapSync)
	if res := q.Results(false); len(res) != 0 {
		t.Fatal("new queue should have 0 results")
	}

	// Schedule a batch of headers
	headers := chain.headers()
	hashes := make([]common.Hash, len(headers))
	for i, header := range headers {
		hashes[i] = header.Hash()
	}
	q.Schedule(headers, hashes, 1)
	if q.Idle() {
		t.Errorf("queue should not be idle")
	}
	if got, exp := q.PendingBodies(), chain.Len(); got != exp {
		t.Errorf("wrong pending block count, got %d, exp %d", got, exp)
	}
	// Only non-empty receipts get added to task-queue
	if got, exp := q.PendingReceipts(), 64; got != exp {
		t.Errorf("wrong pending receipt count, got %d, exp %d", got, exp)
	}
	// Items are now queued for downloading, next step is that we tell the
	// queue that a certain peer will deliver them for us
	{
		peer := dummyPeer("peer-1")
		fetchReq, _, throttle := q.ReserveBodies(peer, 50)
		if !throttle {
			// queue size is only 10, so throttling should occur
			t.Fatal("should throttle")
		}
		// But we should still get the first things to fetch
		if got, exp := len(fetchReq.Headers), 5; got != exp {
			t.Fatalf("expected %d requests, got %d", exp, got)
		}
		if got, exp := fetchReq.Headers[0].Number.Uint64(), uint64(1); got != exp {
			t.Fatalf("expected header %d, got %d", exp, got)
		}
	}
	if exp, got := q.blockTaskQueue.Size(), numOfBlocks-10; exp != got {
		t.Errorf("expected block task queue to be %d, got %d", exp, got)
	}
	if exp, got := q.receiptTaskQueue.Size(), numOfReceipts; exp != got {
		t.Errorf("expected receipt task queue to be %d, got %d", exp, got)
	}
	{
		peer := dummyPeer("peer-2")
		fetchReq, _, throttle := q.ReserveBodies(peer, 50)

		// The second peer should hit throttling
		if !throttle {
			t.Fatalf("should throttle")
		}
		// And not get any fetches at all, since it was throttled to begin with
		if fetchReq != nil {
			t.Fatalf("should have no fetches, got %d", len(fetchReq.Headers))
		}
	}
	if exp, got := q.blockTaskQueue.Size(), numOfBlocks-10; exp != got {
		t.Errorf("expected block task queue to be %d, got %d", exp, got)
	}
	if exp, got := q.receiptTaskQueue.Size(), numOfReceipts; exp != got {
		t.Errorf("expected receipt task queue to be %d, got %d", exp, got)
	}
	{
		// The receipt delivering peer should not be affected
		// by the throttling of body deliveries
		peer := dummyPeer("peer-3")
		fetchReq, _, throttle := q.ReserveReceipts(peer, 50)
		if !throttle {
			// queue size is only 10, so throttling should occur
			t.Fatal("should throttle")
		}
		// But we should still get the first things to fetch
		if got, exp := len(fetchReq.Headers), 5; got != exp {
			t.Fatalf("expected %d requests, got %d", exp, got)
		}
		if got, exp := fetchReq.Headers[0].Number.Uint64(), uint64(1); got != exp {
			t.Fatalf("expected header %d, got %d", exp, got)
		}
	}
	if exp, got := q.blockTaskQueue.Size(), numOfBlocks-10; exp != got {
		t.Errorf("expected block task queue to be %d, got %d", exp, got)
	}
	if exp, got := q.receiptTaskQueue.Size(), numOfReceipts-5; exp != got {
		t.Errorf("expected receipt task queue to be %d, got %d", exp, got)
	}
	if got, exp := q.resultCache.countCompleted(), 0; got != exp {
		t.Errorf("wrong processable count, got %d, exp %d", got, exp)
	}
}

func TestEmptyBlocks(t *testing.T) {
	numOfBlocks := len(emptyChain.blocks)

	q := newQueue(10, 10)

	q.Prepare(1, SnapSync)

	// Schedule a batch of headers
	headers := emptyChain.headers()
	hashes := make([]common.Hash, len(headers))
	for i, header := range headers {
		hashes[i] = header.Hash()
	}
	q.Schedule(headers, hashes, 1)
	if q.Idle() {
		t.Errorf("queue should not be idle")
	}
	if got, exp := q.PendingBodies(), len(emptyChain.blocks); got != exp {
		t.Errorf("wrong pending block count, got %d, exp %d", got, exp)
	}
	if got, exp := q.PendingReceipts(), 0; got != exp {
		t.Errorf("wrong pending receipt count, got %d, exp %d", got, exp)
	}
	// They won't be processable, because the fetchresults haven't been
	// created yet
	if got, exp := q.resultCache.countCompleted(), 0; got != exp {
		t.Errorf("wrong processable count, got %d, exp %d", got, exp)
	}

	// Items are now queued for downloading, next step is that we tell the
	// queue that a certain peer will deliver them for us
	// That should trigger all of them to suddenly become 'done'
	{
		// Reserve blocks
		peer := dummyPeer("peer-1")
		fetchReq, _, _ := q.ReserveBodies(peer, 50)

		// there should be nothing to fetch, blocks are empty
		if fetchReq != nil {
			t.Fatal("there should be no body fetch tasks remaining")
		}
	}
	if q.blockTaskQueue.Size() != numOfBlocks-10 {
		t.Errorf("expected block task queue to be %d, got %d", numOfBlocks-10, q.blockTaskQueue.Size())
	}
	if q.receiptTaskQueue.Size() != 0 {
		t.Errorf("expected receipt task queue to be %d, got %d", 0, q.receiptTaskQueue.Size())
	}
	{
		peer := dummyPeer("peer-3")
		fetchReq, _, _ := q.ReserveReceipts(peer, 50)

		// there should be nothing to fetch, blocks are empty
		if fetchReq != nil {
			t.Fatal("there should be no receipt fetch tasks remaining")
		}
	}
	if q.blockTaskQueue.Size() != numOfBlocks-10 {
		t.Errorf("expected block task queue to be %d, got %d", numOfBlocks-10, q.blockTaskQueue.Size())
	}
	if q.receiptTaskQueue.Size() != 0 {
		t.Errorf("expected receipt task queue to be %d, got %d", 0, q.receiptTaskQueue.Size())
	}
	if got, exp := q.resultCache.countCompleted(), 10; got != exp {
		t.Errorf("wrong processable count, got %d, exp %d", got, exp)
	}
}

// XTestDelivery does some more extensive testing of events that happen,
// blocks that become known and peers that make reservations and deliveries.
// disabled since it's not really a unit-test, but can be executed to test
// some more advanced scenarios
func XTestDelivery(t *testing.T) {
	// the outside network, holding blocks
	blo, rec := makeChain(128, 0, testGenesis, false)
	world := newNetwork()
	world.receipts = rec
	world.chain = blo
	world.progress(10)
	if false {
		log.Root().SetHandler(log.StdoutHandler)
	}
	q := newQueue(10, 10)
	var wg sync.WaitGroup
	q.Prepare(1, SnapSync)
	wg.Add(1)
	go func() {
		// deliver headers
		defer wg.Done()
		c := 1
		for {
			//fmt.Printf("getting headers from %d\n", c)
			headers := world.headers(c)
			hashes := make([]common.Hash, len(headers))
			for i, header := range headers {
				hashes[i] = header.Hash()
			}
			l := len(headers)
			//fmt.Printf("scheduling %d headers, first %d last %d\n",
			//	l, headers[0].Number.Uint64(), headers[len(headers)-1].Number.Uint64())
			q.Schedule(headers, hashes, uint64(c))
			c += l
		}
	}()
	wg.Add(1)
	go func() {
		// collect results
		defer wg.Done()
		tot := 0
		for {
			res := q.Results(true)
			tot += len(res)
			fmt.Printf("got %d results, %d tot\n", len(res), tot)
			// Now we can forget about these
			world.forget(res[len(res)-1].Header.Number.Uint64())
		}
	}()
	wg.Add(1)
	go func() {
		defer wg.Done()
		// reserve body fetch
		i := 4
		for {
			peer := dummyPeer(fmt.Sprintf("peer-%d", i))
			f, _, _ := q.ReserveBodies(peer, rand.Intn(30))
			if f != nil {
				var (
					emptyList []*types.Header
					txset     [][]*types.Transaction
					uncleset  [][]*types.Header
				)
				numToSkip := rand.Intn(len(f.Headers))
				for _, hdr := range f.Headers[0 : len(f.Headers)-numToSkip] {
					txset = append(txset, world.getTransactions(hdr.Number.Uint64()))
					uncleset = append(uncleset, emptyList)
				}
				var (
					txsHashes   = make([]common.Hash, len(txset))
					uncleHashes = make([]common.Hash, len(uncleset))
				)
				hasher := trie.NewStackTrie(nil)
				for i, txs := range txset {
					txsHashes[i] = types.DeriveSha(types.Transactions(txs), hasher)
				}
				for i, uncles := range uncleset {
					uncleHashes[i] = types.CalcUncleHash(uncles)
				}
				time.Sleep(100 * time.Millisecond)
				_, err := q.DeliverBodies(peer.id, txset, txsHashes, uncleset, uncleHashes, nil, nil)
				if err != nil {
					fmt.Printf("delivered %d bodies %v\n", len(txset), err)
				}
			} else {
				i++
				time.Sleep(200 * time.Millisecond)
			}
		}
	}()
	go func() {
		defer wg.Done()
		// reserve receiptfetch
		peer := dummyPeer("peer-3")
		for {
			f, _, _ := q.ReserveReceipts(peer, rand.Intn(50))
			if f != nil {
				var rcs [][]*types.Receipt
				for _, hdr := range f.Headers {
					rcs = append(rcs, world.getReceipts(hdr.Number.Uint64()))
				}
				hasher := trie.NewStackTrie(nil)
				hashes := make([]common.Hash, len(rcs))
				for i, receipt := range rcs {
					hashes[i] = types.DeriveSha(types.Receipts(receipt), hasher)
				}
				_, err := q.DeliverReceipts(peer.id, rcs, hashes)
				if err != nil {
					fmt.Printf("delivered %d receipts %v\n", len(rcs), err)
				}
				time.Sleep(100 * time.Millisecond)
			} else {
				time.Sleep(200 * time.Millisecond)
			}
		}
	}()
	wg.Add(1)
	go func() {
		defer wg.Done()
		for i := 0; i < 50; i++ {
			time.Sleep(300 * time.Millisecond)
			//world.tick()
			//fmt.Printf("trying to progress\n")
			world.progress(rand.Intn(100))
		}
		for i := 0; i < 50; i++ {
			time.Sleep(2990 * time.Millisecond)
		}
	}()
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			time.Sleep(990 * time.Millisecond)
			fmt.Printf("world block tip is %d\n",
				world.chain[len(world.chain)-1].Header().Number.Uint64())
			fmt.Println(q.Stats())
		}
	}()
	wg.Wait()
}

func newNetwork() *network {
	var l sync.RWMutex
	return &network{
		cond:   sync.NewCond(&l),
		offset: 1, // block 1 is at blocks[0]
	}
}

// represents the network
type network struct {
	offset   int
	chain    []*types.Block
	receipts []types.Receipts
	lock     sync.RWMutex
	cond     *sync.Cond
}

func (n *network) getTransactions(blocknum uint64) types.Transactions {
	index := blocknum - uint64(n.offset)
	return n.chain[index].Transactions()
}
func (n *network) getReceipts(blocknum uint64) types.Receipts {
	index := blocknum - uint64(n.offset)
	if got := n.chain[index].Header().Number.Uint64(); got != blocknum {
		fmt.Printf("Err, got %d exp %d\n", got, blocknum)
		panic("sd")
	}
	return n.receipts[index]
}

func (n *network) forget(blocknum uint64) {
	index := blocknum - uint64(n.offset)
	n.chain = n.chain[index:]
	n.receipts = n.receipts[index:]
	n.offset = int(blocknum)
}
func (n *network) progress(numBlocks int) {
	n.lock.Lock()
	defer n.lock.Unlock()
	//fmt.Printf("progressing...\n")
	newBlocks, newR := makeChain(numBlocks, 0, n.chain[len(n.chain)-1], false)
	n.chain = append(n.chain, newBlocks...)
	n.receipts = append(n.receipts, newR...)
	n.cond.Broadcast()
}

func (n *network) headers(from int) []*types.Header {
	numHeaders := 128
	var hdrs []*types.Header
	index := from - n.offset

	for index >= len(n.chain) {
		// wait for progress
		n.cond.L.Lock()
		//fmt.Printf("header going into wait\n")
		n.cond.Wait()
		index = from - n.offset
		n.cond.L.Unlock()
	}
	n.lock.RLock()
	defer n.lock.RUnlock()
	for i, b := range n.chain[index:] {
		hdrs = append(hdrs, b.Header())
		if i >= numHeaders {
			break
		}
	}
	return hdrs
}
