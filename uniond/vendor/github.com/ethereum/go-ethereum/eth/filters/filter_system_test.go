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

package filters

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"math/rand"
	"reflect"
	"runtime"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/bloombits"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rpc"
)

type testBackend struct {
	db              ethdb.Database
	sections        uint64
	txFeed          event.Feed
	logsFeed        event.Feed
	rmLogsFeed      event.Feed
	pendingLogsFeed event.Feed
	chainFeed       event.Feed
	pendingBlock    *types.Block
	pendingReceipts types.Receipts
}

func (b *testBackend) ChainConfig() *params.ChainConfig {
	return params.TestChainConfig
}

func (b *testBackend) CurrentHeader() *types.Header {
	hdr, _ := b.HeaderByNumber(context.TODO(), rpc.LatestBlockNumber)
	return hdr
}

func (b *testBackend) ChainDb() ethdb.Database {
	return b.db
}

func (b *testBackend) HeaderByNumber(ctx context.Context, blockNr rpc.BlockNumber) (*types.Header, error) {
	var (
		hash common.Hash
		num  uint64
	)
	switch blockNr {
	case rpc.LatestBlockNumber:
		hash = rawdb.ReadHeadBlockHash(b.db)
		number := rawdb.ReadHeaderNumber(b.db, hash)
		if number == nil {
			return nil, nil
		}
		num = *number
	case rpc.FinalizedBlockNumber:
		hash = rawdb.ReadFinalizedBlockHash(b.db)
		number := rawdb.ReadHeaderNumber(b.db, hash)
		if number == nil {
			return nil, nil
		}
		num = *number
	case rpc.SafeBlockNumber:
		return nil, errors.New("safe block not found")
	default:
		num = uint64(blockNr)
		hash = rawdb.ReadCanonicalHash(b.db, num)
	}
	return rawdb.ReadHeader(b.db, hash, num), nil
}

func (b *testBackend) HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error) {
	number := rawdb.ReadHeaderNumber(b.db, hash)
	if number == nil {
		return nil, nil
	}
	return rawdb.ReadHeader(b.db, hash, *number), nil
}

func (b *testBackend) GetBody(ctx context.Context, hash common.Hash, number rpc.BlockNumber) (*types.Body, error) {
	if body := rawdb.ReadBody(b.db, hash, uint64(number)); body != nil {
		return body, nil
	}
	return nil, errors.New("block body not found")
}

func (b *testBackend) GetReceipts(ctx context.Context, hash common.Hash) (types.Receipts, error) {
	if number := rawdb.ReadHeaderNumber(b.db, hash); number != nil {
		if header := rawdb.ReadHeader(b.db, hash, *number); header != nil {
			return rawdb.ReadReceipts(b.db, hash, *number, header.Time, params.TestChainConfig), nil
		}
	}
	return nil, nil
}

func (b *testBackend) GetLogs(ctx context.Context, hash common.Hash, number uint64) ([][]*types.Log, error) {
	logs := rawdb.ReadLogs(b.db, hash, number)
	return logs, nil
}

func (b *testBackend) PendingBlockAndReceipts() (*types.Block, types.Receipts) {
	return b.pendingBlock, b.pendingReceipts
}

func (b *testBackend) SubscribeNewTxsEvent(ch chan<- core.NewTxsEvent) event.Subscription {
	return b.txFeed.Subscribe(ch)
}

func (b *testBackend) SubscribeRemovedLogsEvent(ch chan<- core.RemovedLogsEvent) event.Subscription {
	return b.rmLogsFeed.Subscribe(ch)
}

func (b *testBackend) SubscribeLogsEvent(ch chan<- []*types.Log) event.Subscription {
	return b.logsFeed.Subscribe(ch)
}

func (b *testBackend) SubscribePendingLogsEvent(ch chan<- []*types.Log) event.Subscription {
	return b.pendingLogsFeed.Subscribe(ch)
}

func (b *testBackend) SubscribeChainEvent(ch chan<- core.ChainEvent) event.Subscription {
	return b.chainFeed.Subscribe(ch)
}

func (b *testBackend) BloomStatus() (uint64, uint64) {
	return params.BloomBitsBlocks, b.sections
}

func (b *testBackend) ServiceFilter(ctx context.Context, session *bloombits.MatcherSession) {
	requests := make(chan chan *bloombits.Retrieval)

	go session.Multiplex(16, 0, requests)
	go func() {
		for {
			// Wait for a service request or a shutdown
			select {
			case <-ctx.Done():
				return

			case request := <-requests:
				task := <-request

				task.Bitsets = make([][]byte, len(task.Sections))
				for i, section := range task.Sections {
					if rand.Int()%4 != 0 { // Handle occasional missing deliveries
						head := rawdb.ReadCanonicalHash(b.db, (section+1)*params.BloomBitsBlocks-1)
						task.Bitsets[i], _ = rawdb.ReadBloomBits(b.db, task.Bit, section, head)
					}
				}
				request <- task
			}
		}
	}()
}

func newTestFilterSystem(t testing.TB, db ethdb.Database, cfg Config) (*testBackend, *FilterSystem) {
	backend := &testBackend{db: db}
	sys := NewFilterSystem(backend, cfg)
	return backend, sys
}

// TestBlockSubscription tests if a block subscription returns block hashes for posted chain events.
// It creates multiple subscriptions:
// - one at the start and should receive all posted chain events and a second (blockHashes)
// - one that is created after a cutoff moment and uninstalled after a second cutoff moment (blockHashes[cutoff1:cutoff2])
// - one that is created after the second cutoff moment (blockHashes[cutoff2:])
func TestBlockSubscription(t *testing.T) {
	t.Parallel()

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{})
		api          = NewFilterAPI(sys, false)
		genesis      = &core.Genesis{
			Config:  params.TestChainConfig,
			BaseFee: big.NewInt(params.InitialBaseFee),
		}
		_, chain, _ = core.GenerateChainWithGenesis(genesis, ethash.NewFaker(), 10, func(i int, gen *core.BlockGen) {})
		chainEvents = []core.ChainEvent{}
	)

	for _, blk := range chain {
		chainEvents = append(chainEvents, core.ChainEvent{Hash: blk.Hash(), Block: blk})
	}

	chan0 := make(chan *types.Header)
	sub0 := api.events.SubscribeNewHeads(chan0)
	chan1 := make(chan *types.Header)
	sub1 := api.events.SubscribeNewHeads(chan1)

	go func() { // simulate client
		i1, i2 := 0, 0
		for i1 != len(chainEvents) || i2 != len(chainEvents) {
			select {
			case header := <-chan0:
				if chainEvents[i1].Hash != header.Hash() {
					t.Errorf("sub0 received invalid hash on index %d, want %x, got %x", i1, chainEvents[i1].Hash, header.Hash())
				}
				i1++
			case header := <-chan1:
				if chainEvents[i2].Hash != header.Hash() {
					t.Errorf("sub1 received invalid hash on index %d, want %x, got %x", i2, chainEvents[i2].Hash, header.Hash())
				}
				i2++
			}
		}

		sub0.Unsubscribe()
		sub1.Unsubscribe()
	}()

	time.Sleep(1 * time.Second)
	for _, e := range chainEvents {
		backend.chainFeed.Send(e)
	}

	<-sub0.Err()
	<-sub1.Err()
}

// TestPendingTxFilter tests whether pending tx filters retrieve all pending transactions that are posted to the event mux.
func TestPendingTxFilter(t *testing.T) {
	t.Parallel()

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{})
		api          = NewFilterAPI(sys, false)

		transactions = []*types.Transaction{
			types.NewTransaction(0, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(1, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(2, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(3, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(4, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
		}

		hashes []common.Hash
	)

	fid0 := api.NewPendingTransactionFilter(nil)

	time.Sleep(1 * time.Second)
	backend.txFeed.Send(core.NewTxsEvent{Txs: transactions})

	timeout := time.Now().Add(1 * time.Second)
	for {
		results, err := api.GetFilterChanges(fid0)
		if err != nil {
			t.Fatalf("Unable to retrieve logs: %v", err)
		}

		h := results.([]common.Hash)
		hashes = append(hashes, h...)
		if len(hashes) >= len(transactions) {
			break
		}
		// check timeout
		if time.Now().After(timeout) {
			break
		}

		time.Sleep(100 * time.Millisecond)
	}

	if len(hashes) != len(transactions) {
		t.Errorf("invalid number of transactions, want %d transactions(s), got %d", len(transactions), len(hashes))
		return
	}
	for i := range hashes {
		if hashes[i] != transactions[i].Hash() {
			t.Errorf("hashes[%d] invalid, want %x, got %x", i, transactions[i].Hash(), hashes[i])
		}
	}
}

// TestPendingTxFilterFullTx tests whether pending tx filters retrieve all pending transactions that are posted to the event mux.
func TestPendingTxFilterFullTx(t *testing.T) {
	t.Parallel()

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{})
		api          = NewFilterAPI(sys, false)

		transactions = []*types.Transaction{
			types.NewTransaction(0, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(1, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(2, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(3, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
			types.NewTransaction(4, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil),
		}

		txs []*ethapi.RPCTransaction
	)

	fullTx := true
	fid0 := api.NewPendingTransactionFilter(&fullTx)

	time.Sleep(1 * time.Second)
	backend.txFeed.Send(core.NewTxsEvent{Txs: transactions})

	timeout := time.Now().Add(1 * time.Second)
	for {
		results, err := api.GetFilterChanges(fid0)
		if err != nil {
			t.Fatalf("Unable to retrieve logs: %v", err)
		}

		tx := results.([]*ethapi.RPCTransaction)
		txs = append(txs, tx...)
		if len(txs) >= len(transactions) {
			break
		}
		// check timeout
		if time.Now().After(timeout) {
			break
		}

		time.Sleep(100 * time.Millisecond)
	}

	if len(txs) != len(transactions) {
		t.Errorf("invalid number of transactions, want %d transactions(s), got %d", len(transactions), len(txs))
		return
	}
	for i := range txs {
		if txs[i].Hash != transactions[i].Hash() {
			t.Errorf("hashes[%d] invalid, want %x, got %x", i, transactions[i].Hash(), txs[i].Hash)
		}
	}
}

// TestLogFilterCreation test whether a given filter criteria makes sense.
// If not it must return an error.
func TestLogFilterCreation(t *testing.T) {
	var (
		db     = rawdb.NewMemoryDatabase()
		_, sys = newTestFilterSystem(t, db, Config{})
		api    = NewFilterAPI(sys, false)

		testCases = []struct {
			crit    FilterCriteria
			success bool
		}{
			// defaults
			{FilterCriteria{}, true},
			// valid block number range
			{FilterCriteria{FromBlock: big.NewInt(1), ToBlock: big.NewInt(2)}, true},
			// "mined" block range to pending
			{FilterCriteria{FromBlock: big.NewInt(1), ToBlock: big.NewInt(rpc.LatestBlockNumber.Int64())}, true},
			// new mined and pending blocks
			{FilterCriteria{FromBlock: big.NewInt(rpc.LatestBlockNumber.Int64()), ToBlock: big.NewInt(rpc.PendingBlockNumber.Int64())}, true},
			// from block "higher" than to block
			{FilterCriteria{FromBlock: big.NewInt(2), ToBlock: big.NewInt(1)}, false},
			// from block "higher" than to block
			{FilterCriteria{FromBlock: big.NewInt(rpc.LatestBlockNumber.Int64()), ToBlock: big.NewInt(100)}, false},
			// from block "higher" than to block
			{FilterCriteria{FromBlock: big.NewInt(rpc.PendingBlockNumber.Int64()), ToBlock: big.NewInt(100)}, false},
			// from block "higher" than to block
			{FilterCriteria{FromBlock: big.NewInt(rpc.PendingBlockNumber.Int64()), ToBlock: big.NewInt(rpc.LatestBlockNumber.Int64())}, false},
		}
	)

	for i, test := range testCases {
		id, err := api.NewFilter(test.crit)
		if err != nil && test.success {
			t.Errorf("expected filter creation for case %d to success, got %v", i, err)
		}
		if err == nil {
			api.UninstallFilter(id)
			if !test.success {
				t.Errorf("expected testcase %d to fail with an error", i)
			}
		}
	}
}

// TestInvalidLogFilterCreation tests whether invalid filter log criteria results in an error
// when the filter is created.
func TestInvalidLogFilterCreation(t *testing.T) {
	t.Parallel()

	var (
		db     = rawdb.NewMemoryDatabase()
		_, sys = newTestFilterSystem(t, db, Config{})
		api    = NewFilterAPI(sys, false)
	)

	// different situations where log filter creation should fail.
	// Reason: fromBlock > toBlock
	testCases := []FilterCriteria{
		0: {FromBlock: big.NewInt(rpc.PendingBlockNumber.Int64()), ToBlock: big.NewInt(rpc.LatestBlockNumber.Int64())},
		1: {FromBlock: big.NewInt(rpc.PendingBlockNumber.Int64()), ToBlock: big.NewInt(100)},
		2: {FromBlock: big.NewInt(rpc.LatestBlockNumber.Int64()), ToBlock: big.NewInt(100)},
	}

	for i, test := range testCases {
		if _, err := api.NewFilter(test); err == nil {
			t.Errorf("Expected NewFilter for case #%d to fail", i)
		}
	}
}

func TestInvalidGetLogsRequest(t *testing.T) {
	var (
		db        = rawdb.NewMemoryDatabase()
		_, sys    = newTestFilterSystem(t, db, Config{})
		api       = NewFilterAPI(sys, false)
		blockHash = common.HexToHash("0x1111111111111111111111111111111111111111111111111111111111111111")
	)

	// Reason: Cannot specify both BlockHash and FromBlock/ToBlock)
	testCases := []FilterCriteria{
		0: {BlockHash: &blockHash, FromBlock: big.NewInt(100)},
		1: {BlockHash: &blockHash, ToBlock: big.NewInt(500)},
		2: {BlockHash: &blockHash, FromBlock: big.NewInt(rpc.LatestBlockNumber.Int64())},
	}

	for i, test := range testCases {
		if _, err := api.GetLogs(context.Background(), test); err == nil {
			t.Errorf("Expected Logs for case #%d to fail", i)
		}
	}
}

// TestLogFilter tests whether log filters match the correct logs that are posted to the event feed.
func TestLogFilter(t *testing.T) {
	t.Parallel()

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{})
		api          = NewFilterAPI(sys, false)

		firstAddr      = common.HexToAddress("0x1111111111111111111111111111111111111111")
		secondAddr     = common.HexToAddress("0x2222222222222222222222222222222222222222")
		thirdAddress   = common.HexToAddress("0x3333333333333333333333333333333333333333")
		notUsedAddress = common.HexToAddress("0x9999999999999999999999999999999999999999")
		firstTopic     = common.HexToHash("0x1111111111111111111111111111111111111111111111111111111111111111")
		secondTopic    = common.HexToHash("0x2222222222222222222222222222222222222222222222222222222222222222")
		notUsedTopic   = common.HexToHash("0x9999999999999999999999999999999999999999999999999999999999999999")

		// posted twice, once as regular logs and once as pending logs.
		allLogs = []*types.Log{
			{Address: firstAddr},
			{Address: firstAddr, Topics: []common.Hash{firstTopic}, BlockNumber: 1},
			{Address: secondAddr, Topics: []common.Hash{firstTopic}, BlockNumber: 1},
			{Address: thirdAddress, Topics: []common.Hash{secondTopic}, BlockNumber: 2},
			{Address: thirdAddress, Topics: []common.Hash{secondTopic}, BlockNumber: 3},
		}

		expectedCase7  = []*types.Log{allLogs[3], allLogs[4], allLogs[0], allLogs[1], allLogs[2], allLogs[3], allLogs[4]}
		expectedCase11 = []*types.Log{allLogs[1], allLogs[2], allLogs[1], allLogs[2]}

		testCases = []struct {
			crit     FilterCriteria
			expected []*types.Log
			id       rpc.ID
		}{
			// match all
			0: {FilterCriteria{}, allLogs, ""},
			// match none due to no matching addresses
			1: {FilterCriteria{Addresses: []common.Address{{}, notUsedAddress}, Topics: [][]common.Hash{nil}}, []*types.Log{}, ""},
			// match logs based on addresses, ignore topics
			2: {FilterCriteria{Addresses: []common.Address{firstAddr}}, allLogs[:2], ""},
			// match none due to no matching topics (match with address)
			3: {FilterCriteria{Addresses: []common.Address{secondAddr}, Topics: [][]common.Hash{{notUsedTopic}}}, []*types.Log{}, ""},
			// match logs based on addresses and topics
			4: {FilterCriteria{Addresses: []common.Address{thirdAddress}, Topics: [][]common.Hash{{firstTopic, secondTopic}}}, allLogs[3:5], ""},
			// match logs based on multiple addresses and "or" topics
			5: {FilterCriteria{Addresses: []common.Address{secondAddr, thirdAddress}, Topics: [][]common.Hash{{firstTopic, secondTopic}}}, allLogs[2:5], ""},
			// logs in the pending block
			6: {FilterCriteria{Addresses: []common.Address{firstAddr}, FromBlock: big.NewInt(rpc.PendingBlockNumber.Int64()), ToBlock: big.NewInt(rpc.PendingBlockNumber.Int64())}, allLogs[:2], ""},
			// mined logs with block num >= 2 or pending logs
			7: {FilterCriteria{FromBlock: big.NewInt(2), ToBlock: big.NewInt(rpc.PendingBlockNumber.Int64())}, expectedCase7, ""},
			// all "mined" logs with block num >= 2
			8: {FilterCriteria{FromBlock: big.NewInt(2), ToBlock: big.NewInt(rpc.LatestBlockNumber.Int64())}, allLogs[3:], ""},
			// all "mined" logs
			9: {FilterCriteria{ToBlock: big.NewInt(rpc.LatestBlockNumber.Int64())}, allLogs, ""},
			// all "mined" logs with 1>= block num <=2 and topic secondTopic
			10: {FilterCriteria{FromBlock: big.NewInt(1), ToBlock: big.NewInt(2), Topics: [][]common.Hash{{secondTopic}}}, allLogs[3:4], ""},
			// all "mined" and pending logs with topic firstTopic
			11: {FilterCriteria{FromBlock: big.NewInt(rpc.LatestBlockNumber.Int64()), ToBlock: big.NewInt(rpc.PendingBlockNumber.Int64()), Topics: [][]common.Hash{{firstTopic}}}, expectedCase11, ""},
			// match all logs due to wildcard topic
			12: {FilterCriteria{Topics: [][]common.Hash{nil}}, allLogs[1:], ""},
		}
	)

	// create all filters
	for i := range testCases {
		testCases[i].id, _ = api.NewFilter(testCases[i].crit)
	}

	// raise events
	time.Sleep(1 * time.Second)
	if nsend := backend.logsFeed.Send(allLogs); nsend == 0 {
		t.Fatal("Logs event not delivered")
	}
	if nsend := backend.pendingLogsFeed.Send(allLogs); nsend == 0 {
		t.Fatal("Pending logs event not delivered")
	}

	for i, tt := range testCases {
		var fetched []*types.Log
		timeout := time.Now().Add(1 * time.Second)
		for { // fetch all expected logs
			results, err := api.GetFilterChanges(tt.id)
			if err != nil {
				t.Fatalf("Unable to fetch logs: %v", err)
			}

			fetched = append(fetched, results.([]*types.Log)...)
			if len(fetched) >= len(tt.expected) {
				break
			}
			// check timeout
			if time.Now().After(timeout) {
				break
			}

			time.Sleep(100 * time.Millisecond)
		}

		if len(fetched) != len(tt.expected) {
			t.Errorf("invalid number of logs for case %d, want %d log(s), got %d", i, len(tt.expected), len(fetched))
			return
		}

		for l := range fetched {
			if fetched[l].Removed {
				t.Errorf("expected log not to be removed for log %d in case %d", l, i)
			}
			if !reflect.DeepEqual(fetched[l], tt.expected[l]) {
				t.Errorf("invalid log on index %d for case %d", l, i)
			}
		}
	}
}

// TestPendingLogsSubscription tests if a subscription receives the correct pending logs that are posted to the event feed.
func TestPendingLogsSubscription(t *testing.T) {
	t.Parallel()

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{})
		api          = NewFilterAPI(sys, false)

		firstAddr      = common.HexToAddress("0x1111111111111111111111111111111111111111")
		secondAddr     = common.HexToAddress("0x2222222222222222222222222222222222222222")
		thirdAddress   = common.HexToAddress("0x3333333333333333333333333333333333333333")
		notUsedAddress = common.HexToAddress("0x9999999999999999999999999999999999999999")
		firstTopic     = common.HexToHash("0x1111111111111111111111111111111111111111111111111111111111111111")
		secondTopic    = common.HexToHash("0x2222222222222222222222222222222222222222222222222222222222222222")
		thirdTopic     = common.HexToHash("0x3333333333333333333333333333333333333333333333333333333333333333")
		fourthTopic    = common.HexToHash("0x4444444444444444444444444444444444444444444444444444444444444444")
		notUsedTopic   = common.HexToHash("0x9999999999999999999999999999999999999999999999999999999999999999")

		allLogs = [][]*types.Log{
			{{Address: firstAddr, Topics: []common.Hash{}, BlockNumber: 0}},
			{{Address: firstAddr, Topics: []common.Hash{firstTopic}, BlockNumber: 1}},
			{{Address: secondAddr, Topics: []common.Hash{firstTopic}, BlockNumber: 2}},
			{{Address: thirdAddress, Topics: []common.Hash{secondTopic}, BlockNumber: 3}},
			{{Address: thirdAddress, Topics: []common.Hash{secondTopic}, BlockNumber: 4}},
			{
				{Address: thirdAddress, Topics: []common.Hash{firstTopic}, BlockNumber: 5},
				{Address: thirdAddress, Topics: []common.Hash{thirdTopic}, BlockNumber: 5},
				{Address: thirdAddress, Topics: []common.Hash{fourthTopic}, BlockNumber: 5},
				{Address: firstAddr, Topics: []common.Hash{firstTopic}, BlockNumber: 5},
			},
		}

		pendingBlockNumber = big.NewInt(rpc.PendingBlockNumber.Int64())

		testCases = []struct {
			crit     ethereum.FilterQuery
			expected []*types.Log
			c        chan []*types.Log
			sub      *Subscription
			err      chan error
		}{
			// match all
			{
				ethereum.FilterQuery{FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				flattenLogs(allLogs),
				nil, nil, nil,
			},
			// match none due to no matching addresses
			{
				ethereum.FilterQuery{Addresses: []common.Address{{}, notUsedAddress}, Topics: [][]common.Hash{nil}, FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				nil,
				nil, nil, nil,
			},
			// match logs based on addresses, ignore topics
			{
				ethereum.FilterQuery{Addresses: []common.Address{firstAddr}, FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				append(flattenLogs(allLogs[:2]), allLogs[5][3]),
				nil, nil, nil,
			},
			// match none due to no matching topics (match with address)
			{
				ethereum.FilterQuery{Addresses: []common.Address{secondAddr}, Topics: [][]common.Hash{{notUsedTopic}}, FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				nil,
				nil, nil, nil,
			},
			// match logs based on addresses and topics
			{
				ethereum.FilterQuery{Addresses: []common.Address{thirdAddress}, Topics: [][]common.Hash{{firstTopic, secondTopic}}, FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				append(flattenLogs(allLogs[3:5]), allLogs[5][0]),
				nil, nil, nil,
			},
			// match logs based on multiple addresses and "or" topics
			{
				ethereum.FilterQuery{Addresses: []common.Address{secondAddr, thirdAddress}, Topics: [][]common.Hash{{firstTopic, secondTopic}}, FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				append(flattenLogs(allLogs[2:5]), allLogs[5][0]),
				nil, nil, nil,
			},
			// multiple pending logs, should match only 2 topics from the logs in block 5
			{
				ethereum.FilterQuery{Addresses: []common.Address{thirdAddress}, Topics: [][]common.Hash{{firstTopic, fourthTopic}}, FromBlock: pendingBlockNumber, ToBlock: pendingBlockNumber},
				[]*types.Log{allLogs[5][0], allLogs[5][2]},
				nil, nil, nil,
			},
			// match none due to only matching new mined logs
			{
				ethereum.FilterQuery{},
				nil,
				nil, nil, nil,
			},
			// match none due to only matching mined logs within a specific block range
			{
				ethereum.FilterQuery{FromBlock: big.NewInt(1), ToBlock: big.NewInt(2)},
				nil,
				nil, nil, nil,
			},
			// match all due to matching mined and pending logs
			{
				ethereum.FilterQuery{FromBlock: big.NewInt(rpc.LatestBlockNumber.Int64()), ToBlock: big.NewInt(rpc.PendingBlockNumber.Int64())},
				flattenLogs(allLogs),
				nil, nil, nil,
			},
			// match none due to matching logs from a specific block number to new mined blocks
			{
				ethereum.FilterQuery{FromBlock: big.NewInt(1), ToBlock: big.NewInt(rpc.LatestBlockNumber.Int64())},
				nil,
				nil, nil, nil,
			},
		}
	)

	// create all subscriptions, this ensures all subscriptions are created before the events are posted.
	// on slow machines this could otherwise lead to missing events when the subscription is created after
	// (some) events are posted.
	for i := range testCases {
		testCases[i].c = make(chan []*types.Log)
		testCases[i].err = make(chan error, 1)

		var err error
		testCases[i].sub, err = api.events.SubscribeLogs(testCases[i].crit, testCases[i].c)
		if err != nil {
			t.Fatalf("SubscribeLogs %d failed: %v\n", i, err)
		}
	}

	for n, test := range testCases {
		i := n
		tt := test
		go func() {
			defer tt.sub.Unsubscribe()

			var fetched []*types.Log

			timeout := time.After(1 * time.Second)
		fetchLoop:
			for {
				select {
				case logs := <-tt.c:
					// Do not break early if we've fetched greater, or equal,
					// to the number of logs expected. This ensures we do not
					// deadlock the filter system because it will do a blocking
					// send on this channel if another log arrives.
					fetched = append(fetched, logs...)
				case <-timeout:
					break fetchLoop
				}
			}

			if len(fetched) != len(tt.expected) {
				tt.err <- fmt.Errorf("invalid number of logs for case %d, want %d log(s), got %d", i, len(tt.expected), len(fetched))
				return
			}

			for l := range fetched {
				if fetched[l].Removed {
					tt.err <- fmt.Errorf("expected log not to be removed for log %d in case %d", l, i)
					return
				}
				if !reflect.DeepEqual(fetched[l], tt.expected[l]) {
					tt.err <- fmt.Errorf("invalid log on index %d for case %d\n", l, i)
					return
				}
			}
			tt.err <- nil
		}()
	}

	// raise events
	for _, ev := range allLogs {
		backend.pendingLogsFeed.Send(ev)
	}

	for i := range testCases {
		err := <-testCases[i].err
		if err != nil {
			t.Fatalf("test %d failed: %v", i, err)
		}
		<-testCases[i].sub.Err()
	}
}

func TestLightFilterLogs(t *testing.T) {
	t.Parallel()

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{})
		api          = NewFilterAPI(sys, true)
		signer       = types.HomesteadSigner{}

		firstAddr      = common.HexToAddress("0x1111111111111111111111111111111111111111")
		secondAddr     = common.HexToAddress("0x2222222222222222222222222222222222222222")
		thirdAddress   = common.HexToAddress("0x3333333333333333333333333333333333333333")
		notUsedAddress = common.HexToAddress("0x9999999999999999999999999999999999999999")
		firstTopic     = common.HexToHash("0x1111111111111111111111111111111111111111111111111111111111111111")
		secondTopic    = common.HexToHash("0x2222222222222222222222222222222222222222222222222222222222222222")

		// posted twice, once as regular logs and once as pending logs.
		allLogs = []*types.Log{
			// Block 1
			{Address: firstAddr, Topics: []common.Hash{}, Data: []byte{}, BlockNumber: 2, Index: 0},
			// Block 2
			{Address: firstAddr, Topics: []common.Hash{firstTopic}, Data: []byte{}, BlockNumber: 3, Index: 0},
			{Address: secondAddr, Topics: []common.Hash{firstTopic}, Data: []byte{}, BlockNumber: 3, Index: 1},
			{Address: thirdAddress, Topics: []common.Hash{secondTopic}, Data: []byte{}, BlockNumber: 3, Index: 2},
			// Block 3
			{Address: thirdAddress, Topics: []common.Hash{secondTopic}, Data: []byte{}, BlockNumber: 4, Index: 0},
		}

		testCases = []struct {
			crit     FilterCriteria
			expected []*types.Log
			id       rpc.ID
		}{
			// match all
			0: {FilterCriteria{}, allLogs, ""},
			// match none due to no matching addresses
			1: {FilterCriteria{Addresses: []common.Address{{}, notUsedAddress}, Topics: [][]common.Hash{nil}}, []*types.Log{}, ""},
			// match logs based on addresses, ignore topics
			2: {FilterCriteria{Addresses: []common.Address{firstAddr}}, allLogs[:2], ""},
			// match logs based on addresses and topics
			3: {FilterCriteria{Addresses: []common.Address{thirdAddress}, Topics: [][]common.Hash{{firstTopic, secondTopic}}}, allLogs[3:5], ""},
			// all logs with block num >= 3
			4: {FilterCriteria{FromBlock: big.NewInt(3), ToBlock: big.NewInt(5)}, allLogs[1:], ""},
			// all logs
			5: {FilterCriteria{FromBlock: big.NewInt(0), ToBlock: big.NewInt(5)}, allLogs, ""},
			// all logs with 1>= block num <=2 and topic secondTopic
			6: {FilterCriteria{FromBlock: big.NewInt(2), ToBlock: big.NewInt(3), Topics: [][]common.Hash{{secondTopic}}}, allLogs[3:4], ""},
		}

		key, _  = crypto.GenerateKey()
		addr    = crypto.PubkeyToAddress(key.PublicKey)
		genesis = &core.Genesis{Config: params.TestChainConfig,
			Alloc: core.GenesisAlloc{
				addr: {Balance: big.NewInt(params.Ether)},
			},
		}
		receipts = []*types.Receipt{{
			Logs: []*types.Log{allLogs[0]},
		}, {
			Logs: []*types.Log{allLogs[1], allLogs[2], allLogs[3]},
		}, {
			Logs: []*types.Log{allLogs[4]},
		}}
	)

	_, blocks, _ := core.GenerateChainWithGenesis(genesis, ethash.NewFaker(), 4, func(i int, b *core.BlockGen) {
		if i == 0 {
			return
		}
		receipts[i-1].Bloom = types.CreateBloom(types.Receipts{receipts[i-1]})
		b.AddUncheckedReceipt(receipts[i-1])
		tx, _ := types.SignTx(types.NewTx(&types.LegacyTx{Nonce: uint64(i - 1), To: &common.Address{}, Value: big.NewInt(1000), Gas: params.TxGas, GasPrice: b.BaseFee(), Data: nil}), signer, key)
		b.AddTx(tx)
	})
	for i, block := range blocks {
		rawdb.WriteBlock(db, block)
		rawdb.WriteCanonicalHash(db, block.Hash(), block.NumberU64())
		rawdb.WriteHeadBlockHash(db, block.Hash())
		if i > 0 {
			rawdb.WriteReceipts(db, block.Hash(), block.NumberU64(), []*types.Receipt{receipts[i-1]})
		}
	}
	// create all filters
	for i := range testCases {
		id, err := api.NewFilter(testCases[i].crit)
		if err != nil {
			t.Fatal(err)
		}
		testCases[i].id = id
	}

	// raise events
	time.Sleep(1 * time.Second)
	for _, block := range blocks {
		backend.chainFeed.Send(core.ChainEvent{Block: block, Hash: common.Hash{}, Logs: allLogs})
	}

	for i, tt := range testCases {
		var fetched []*types.Log
		timeout := time.Now().Add(1 * time.Second)
		for { // fetch all expected logs
			results, err := api.GetFilterChanges(tt.id)
			if err != nil {
				t.Fatalf("Unable to fetch logs: %v", err)
			}
			fetched = append(fetched, results.([]*types.Log)...)
			if len(fetched) >= len(tt.expected) {
				break
			}
			// check timeout
			if time.Now().After(timeout) {
				break
			}

			time.Sleep(100 * time.Millisecond)
		}

		if len(fetched) != len(tt.expected) {
			t.Errorf("invalid number of logs for case %d, want %d log(s), got %d", i, len(tt.expected), len(fetched))
			return
		}

		for l := range fetched {
			if fetched[l].Removed {
				t.Errorf("expected log not to be removed for log %d in case %d", l, i)
			}
			expected := *tt.expected[l]
			blockNum := expected.BlockNumber - 1
			expected.BlockHash = blocks[blockNum].Hash()
			expected.TxHash = blocks[blockNum].Transactions()[0].Hash()
			if !reflect.DeepEqual(fetched[l], &expected) {
				t.Errorf("invalid log on index %d for case %d", l, i)
			}
		}
	}
}

// TestPendingTxFilterDeadlock tests if the event loop hangs when pending
// txes arrive at the same time that one of multiple filters is timing out.
// Please refer to #22131 for more details.
func TestPendingTxFilterDeadlock(t *testing.T) {
	t.Parallel()
	timeout := 100 * time.Millisecond

	var (
		db           = rawdb.NewMemoryDatabase()
		backend, sys = newTestFilterSystem(t, db, Config{Timeout: timeout})
		api          = NewFilterAPI(sys, false)
		done         = make(chan struct{})
	)

	go func() {
		// Bombard feed with txes until signal was received to stop
		i := uint64(0)
		for {
			select {
			case <-done:
				return
			default:
			}

			tx := types.NewTransaction(i, common.HexToAddress("0xb794f5ea0ba39494ce83a213fffba74279579268"), new(big.Int), 0, new(big.Int), nil)
			backend.txFeed.Send(core.NewTxsEvent{Txs: []*types.Transaction{tx}})
			i++
		}
	}()

	// Create a bunch of filters that will
	// timeout either in 100ms or 200ms
	subs := make([]*Subscription, 20)
	for i := 0; i < len(subs); i++ {
		fid := api.NewPendingTransactionFilter(nil)
		f, ok := api.filters[fid]
		if !ok {
			t.Fatalf("Filter %s should exist", fid)
		}
		subs[i] = f.s
		// Wait for at least one tx to arrive in filter
		for {
			hashes, err := api.GetFilterChanges(fid)
			if err != nil {
				t.Fatalf("Filter should exist: %v\n", err)
			}
			if len(hashes.([]common.Hash)) > 0 {
				break
			}
			runtime.Gosched()
		}
	}

	// Wait until filters have timed out and have been uninstalled.
	for _, sub := range subs {
		select {
		case <-sub.Err():
		case <-time.After(1 * time.Second):
			t.Fatalf("Filter timeout is hanging")
		}
	}
}

func flattenLogs(pl [][]*types.Log) []*types.Log {
	var logs []*types.Log
	for _, l := range pl {
		logs = append(logs, l...)
	}
	return logs
}
