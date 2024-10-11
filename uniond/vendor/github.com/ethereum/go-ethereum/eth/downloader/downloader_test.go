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
	"os"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/protocols/eth"
	"github.com/ethereum/go-ethereum/eth/protocols/snap"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

// downloadTester is a test simulator for mocking out local block chain.
type downloadTester struct {
	freezer    string
	chain      *core.BlockChain
	downloader *Downloader

	peers map[string]*downloadTesterPeer
	lock  sync.RWMutex
}

// newTester creates a new downloader test mocker.
func newTester(t *testing.T) *downloadTester {
	return newTesterWithNotification(t, nil)
}

// newTester creates a new downloader test mocker.
func newTesterWithNotification(t *testing.T, success func()) *downloadTester {
	freezer := t.TempDir()
	db, err := rawdb.NewDatabaseWithFreezer(rawdb.NewMemoryDatabase(), freezer, "", false)
	if err != nil {
		panic(err)
	}
	t.Cleanup(func() {
		db.Close()
	})
	gspec := &core.Genesis{
		Config:  params.TestChainConfig,
		Alloc:   core.GenesisAlloc{testAddress: {Balance: big.NewInt(1000000000000000)}},
		BaseFee: big.NewInt(params.InitialBaseFee),
	}
	chain, err := core.NewBlockChain(db, nil, gspec, nil, ethash.NewFaker(), vm.Config{}, nil, nil)
	if err != nil {
		panic(err)
	}
	tester := &downloadTester{
		freezer: freezer,
		chain:   chain,
		peers:   make(map[string]*downloadTesterPeer),
	}
	tester.downloader = New(db, new(event.TypeMux), tester.chain, nil, tester.dropPeer, success)
	return tester
}

// terminate aborts any operations on the embedded downloader and releases all
// held resources.
func (dl *downloadTester) terminate() {
	dl.downloader.Terminate()
	dl.chain.Stop()

	os.RemoveAll(dl.freezer)
}

// sync starts synchronizing with a remote peer, blocking until it completes.
func (dl *downloadTester) sync(id string, td *big.Int, mode SyncMode) error {
	head := dl.peers[id].chain.CurrentBlock()
	if td == nil {
		// If no particular TD was requested, load from the peer's blockchain
		td = dl.peers[id].chain.GetTd(head.Hash(), head.Number.Uint64())
	}
	// Synchronise with the chosen peer and ensure proper cleanup afterwards
	err := dl.downloader.synchronise(id, head.Hash(), td, nil, mode, false, nil)
	select {
	case <-dl.downloader.cancelCh:
		// Ok, downloader fully cancelled after sync cycle
	default:
		// Downloader is still accepting packets, can block a peer up
		panic("downloader active post sync cycle") // panic will be caught by tester
	}
	return err
}

// newPeer registers a new block download source into the downloader.
func (dl *downloadTester) newPeer(id string, version uint, blocks []*types.Block) *downloadTesterPeer {
	dl.lock.Lock()
	defer dl.lock.Unlock()

	peer := &downloadTesterPeer{
		dl:              dl,
		id:              id,
		chain:           newTestBlockchain(blocks),
		withholdHeaders: make(map[common.Hash]struct{}),
	}
	dl.peers[id] = peer

	if err := dl.downloader.RegisterPeer(id, version, peer); err != nil {
		panic(err)
	}
	if err := dl.downloader.SnapSyncer.Register(peer); err != nil {
		panic(err)
	}
	return peer
}

// dropPeer simulates a hard peer removal from the connection pool.
func (dl *downloadTester) dropPeer(id string) {
	dl.lock.Lock()
	defer dl.lock.Unlock()

	delete(dl.peers, id)
	dl.downloader.SnapSyncer.Unregister(id)
	dl.downloader.UnregisterPeer(id)
}

type downloadTesterPeer struct {
	dl    *downloadTester
	id    string
	chain *core.BlockChain

	withholdHeaders map[common.Hash]struct{}
}

// Head constructs a function to retrieve a peer's current head hash
// and total difficulty.
func (dlp *downloadTesterPeer) Head() (common.Hash, *big.Int) {
	head := dlp.chain.CurrentBlock()
	return head.Hash(), dlp.chain.GetTd(head.Hash(), head.Number.Uint64())
}

func unmarshalRlpHeaders(rlpdata []rlp.RawValue) []*types.Header {
	var headers = make([]*types.Header, len(rlpdata))
	for i, data := range rlpdata {
		var h types.Header
		if err := rlp.DecodeBytes(data, &h); err != nil {
			panic(err)
		}
		headers[i] = &h
	}
	return headers
}

// RequestHeadersByHash constructs a GetBlockHeaders function based on a hashed
// origin; associated with a particular peer in the download tester. The returned
// function can be used to retrieve batches of headers from the particular peer.
func (dlp *downloadTesterPeer) RequestHeadersByHash(origin common.Hash, amount int, skip int, reverse bool, sink chan *eth.Response) (*eth.Request, error) {
	// Service the header query via the live handler code
	rlpHeaders := eth.ServiceGetBlockHeadersQuery(dlp.chain, &eth.GetBlockHeadersRequest{
		Origin: eth.HashOrNumber{
			Hash: origin,
		},
		Amount:  uint64(amount),
		Skip:    uint64(skip),
		Reverse: reverse,
	}, nil)
	headers := unmarshalRlpHeaders(rlpHeaders)
	// If a malicious peer is simulated withholding headers, delete them
	for hash := range dlp.withholdHeaders {
		for i, header := range headers {
			if header.Hash() == hash {
				headers = append(headers[:i], headers[i+1:]...)
				break
			}
		}
	}
	hashes := make([]common.Hash, len(headers))
	for i, header := range headers {
		hashes[i] = header.Hash()
	}
	// Deliver the headers to the downloader
	req := &eth.Request{
		Peer: dlp.id,
	}
	res := &eth.Response{
		Req:  req,
		Res:  (*eth.BlockHeadersRequest)(&headers),
		Meta: hashes,
		Time: 1,
		Done: make(chan error, 1), // Ignore the returned status
	}
	go func() {
		sink <- res
	}()
	return req, nil
}

// RequestHeadersByNumber constructs a GetBlockHeaders function based on a numbered
// origin; associated with a particular peer in the download tester. The returned
// function can be used to retrieve batches of headers from the particular peer.
func (dlp *downloadTesterPeer) RequestHeadersByNumber(origin uint64, amount int, skip int, reverse bool, sink chan *eth.Response) (*eth.Request, error) {
	// Service the header query via the live handler code
	rlpHeaders := eth.ServiceGetBlockHeadersQuery(dlp.chain, &eth.GetBlockHeadersRequest{
		Origin: eth.HashOrNumber{
			Number: origin,
		},
		Amount:  uint64(amount),
		Skip:    uint64(skip),
		Reverse: reverse,
	}, nil)
	headers := unmarshalRlpHeaders(rlpHeaders)
	// If a malicious peer is simulated withholding headers, delete them
	for hash := range dlp.withholdHeaders {
		for i, header := range headers {
			if header.Hash() == hash {
				headers = append(headers[:i], headers[i+1:]...)
				break
			}
		}
	}
	hashes := make([]common.Hash, len(headers))
	for i, header := range headers {
		hashes[i] = header.Hash()
	}
	// Deliver the headers to the downloader
	req := &eth.Request{
		Peer: dlp.id,
	}
	res := &eth.Response{
		Req:  req,
		Res:  (*eth.BlockHeadersRequest)(&headers),
		Meta: hashes,
		Time: 1,
		Done: make(chan error, 1), // Ignore the returned status
	}
	go func() {
		sink <- res
	}()
	return req, nil
}

// RequestBodies constructs a getBlockBodies method associated with a particular
// peer in the download tester. The returned function can be used to retrieve
// batches of block bodies from the particularly requested peer.
func (dlp *downloadTesterPeer) RequestBodies(hashes []common.Hash, sink chan *eth.Response) (*eth.Request, error) {
	blobs := eth.ServiceGetBlockBodiesQuery(dlp.chain, hashes)

	bodies := make([]*eth.BlockBody, len(blobs))
	for i, blob := range blobs {
		bodies[i] = new(eth.BlockBody)
		rlp.DecodeBytes(blob, bodies[i])
	}
	var (
		txsHashes        = make([]common.Hash, len(bodies))
		uncleHashes      = make([]common.Hash, len(bodies))
		withdrawalHashes = make([]common.Hash, len(bodies))
	)
	hasher := trie.NewStackTrie(nil)
	for i, body := range bodies {
		txsHashes[i] = types.DeriveSha(types.Transactions(body.Transactions), hasher)
		uncleHashes[i] = types.CalcUncleHash(body.Uncles)
	}
	req := &eth.Request{
		Peer: dlp.id,
	}
	res := &eth.Response{
		Req:  req,
		Res:  (*eth.BlockBodiesResponse)(&bodies),
		Meta: [][]common.Hash{txsHashes, uncleHashes, withdrawalHashes},
		Time: 1,
		Done: make(chan error, 1), // Ignore the returned status
	}
	go func() {
		sink <- res
	}()
	return req, nil
}

// RequestReceipts constructs a getReceipts method associated with a particular
// peer in the download tester. The returned function can be used to retrieve
// batches of block receipts from the particularly requested peer.
func (dlp *downloadTesterPeer) RequestReceipts(hashes []common.Hash, sink chan *eth.Response) (*eth.Request, error) {
	blobs := eth.ServiceGetReceiptsQuery(dlp.chain, hashes)

	receipts := make([][]*types.Receipt, len(blobs))
	for i, blob := range blobs {
		rlp.DecodeBytes(blob, &receipts[i])
	}
	hasher := trie.NewStackTrie(nil)
	hashes = make([]common.Hash, len(receipts))
	for i, receipt := range receipts {
		hashes[i] = types.DeriveSha(types.Receipts(receipt), hasher)
	}
	req := &eth.Request{
		Peer: dlp.id,
	}
	res := &eth.Response{
		Req:  req,
		Res:  (*eth.ReceiptsResponse)(&receipts),
		Meta: hashes,
		Time: 1,
		Done: make(chan error, 1), // Ignore the returned status
	}
	go func() {
		sink <- res
	}()
	return req, nil
}

// ID retrieves the peer's unique identifier.
func (dlp *downloadTesterPeer) ID() string {
	return dlp.id
}

// RequestAccountRange fetches a batch of accounts rooted in a specific account
// trie, starting with the origin.
func (dlp *downloadTesterPeer) RequestAccountRange(id uint64, root, origin, limit common.Hash, bytes uint64) error {
	// Create the request and service it
	req := &snap.GetAccountRangePacket{
		ID:     id,
		Root:   root,
		Origin: origin,
		Limit:  limit,
		Bytes:  bytes,
	}
	slimaccs, proofs := snap.ServiceGetAccountRangeQuery(dlp.chain, req)

	// We need to convert to non-slim format, delegate to the packet code
	res := &snap.AccountRangePacket{
		ID:       id,
		Accounts: slimaccs,
		Proof:    proofs,
	}
	hashes, accounts, _ := res.Unpack()

	go dlp.dl.downloader.SnapSyncer.OnAccounts(dlp, id, hashes, accounts, proofs)
	return nil
}

// RequestStorageRanges fetches a batch of storage slots belonging to one or
// more accounts. If slots from only one account is requested, an origin marker
// may also be used to retrieve from there.
func (dlp *downloadTesterPeer) RequestStorageRanges(id uint64, root common.Hash, accounts []common.Hash, origin, limit []byte, bytes uint64) error {
	// Create the request and service it
	req := &snap.GetStorageRangesPacket{
		ID:       id,
		Accounts: accounts,
		Root:     root,
		Origin:   origin,
		Limit:    limit,
		Bytes:    bytes,
	}
	storage, proofs := snap.ServiceGetStorageRangesQuery(dlp.chain, req)

	// We need to convert to demultiplex, delegate to the packet code
	res := &snap.StorageRangesPacket{
		ID:    id,
		Slots: storage,
		Proof: proofs,
	}
	hashes, slots := res.Unpack()

	go dlp.dl.downloader.SnapSyncer.OnStorage(dlp, id, hashes, slots, proofs)
	return nil
}

// RequestByteCodes fetches a batch of bytecodes by hash.
func (dlp *downloadTesterPeer) RequestByteCodes(id uint64, hashes []common.Hash, bytes uint64) error {
	req := &snap.GetByteCodesPacket{
		ID:     id,
		Hashes: hashes,
		Bytes:  bytes,
	}
	codes := snap.ServiceGetByteCodesQuery(dlp.chain, req)
	go dlp.dl.downloader.SnapSyncer.OnByteCodes(dlp, id, codes)
	return nil
}

// RequestTrieNodes fetches a batch of account or storage trie nodes rooted in
// a specific state trie.
func (dlp *downloadTesterPeer) RequestTrieNodes(id uint64, root common.Hash, paths []snap.TrieNodePathSet, bytes uint64) error {
	req := &snap.GetTrieNodesPacket{
		ID:    id,
		Root:  root,
		Paths: paths,
		Bytes: bytes,
	}
	nodes, _ := snap.ServiceGetTrieNodesQuery(dlp.chain, req, time.Now())
	go dlp.dl.downloader.SnapSyncer.OnTrieNodes(dlp, id, nodes)
	return nil
}

// Log retrieves the peer's own contextual logger.
func (dlp *downloadTesterPeer) Log() log.Logger {
	return log.New("peer", dlp.id)
}

// assertOwnChain checks if the local chain contains the correct number of items
// of the various chain components.
func assertOwnChain(t *testing.T, tester *downloadTester, length int) {
	// Mark this method as a helper to report errors at callsite, not in here
	t.Helper()

	headers, blocks, receipts := length, length, length
	if tester.downloader.getMode() == LightSync {
		blocks, receipts = 1, 1
	}
	if hs := int(tester.chain.CurrentHeader().Number.Uint64()) + 1; hs != headers {
		t.Fatalf("synchronised headers mismatch: have %v, want %v", hs, headers)
	}
	if bs := int(tester.chain.CurrentBlock().Number.Uint64()) + 1; bs != blocks {
		t.Fatalf("synchronised blocks mismatch: have %v, want %v", bs, blocks)
	}
	if rs := int(tester.chain.CurrentSnapBlock().Number.Uint64()) + 1; rs != receipts {
		t.Fatalf("synchronised receipts mismatch: have %v, want %v", rs, receipts)
	}
}

func TestCanonicalSynchronisation68Full(t *testing.T)  { testCanonSync(t, eth.ETH68, FullSync) }
func TestCanonicalSynchronisation68Snap(t *testing.T)  { testCanonSync(t, eth.ETH68, SnapSync) }
func TestCanonicalSynchronisation68Light(t *testing.T) { testCanonSync(t, eth.ETH68, LightSync) }
func TestCanonicalSynchronisation67Full(t *testing.T)  { testCanonSync(t, eth.ETH67, FullSync) }
func TestCanonicalSynchronisation67Snap(t *testing.T)  { testCanonSync(t, eth.ETH67, SnapSync) }
func TestCanonicalSynchronisation67Light(t *testing.T) { testCanonSync(t, eth.ETH67, LightSync) }

func testCanonSync(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	// Create a small enough block chain to download
	chain := testChainBase.shorten(blockCacheMaxItems - 15)
	tester.newPeer("peer", protocol, chain.blocks[1:])

	// Synchronise with the peer and make sure all relevant data was retrieved
	if err := tester.sync("peer", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chain.blocks))
}

// Tests that if a large batch of blocks are being downloaded, it is throttled
// until the cached blocks are retrieved.
func TestThrottling68Full(t *testing.T) { testThrottling(t, eth.ETH68, FullSync) }
func TestThrottling68Snap(t *testing.T) { testThrottling(t, eth.ETH68, SnapSync) }
func TestThrottling67Full(t *testing.T) { testThrottling(t, eth.ETH67, FullSync) }
func TestThrottling67Snap(t *testing.T) { testThrottling(t, eth.ETH67, SnapSync) }

func testThrottling(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	// Create a long block chain to download and the tester
	targetBlocks := len(testChainBase.blocks) - 1
	tester.newPeer("peer", protocol, testChainBase.blocks[1:])

	// Wrap the importer to allow stepping
	var blocked atomic.Uint32
	proceed := make(chan struct{})
	tester.downloader.chainInsertHook = func(results []*fetchResult) {
		blocked.Store(uint32(len(results)))
		<-proceed
	}
	// Start a synchronisation concurrently
	errc := make(chan error, 1)
	go func() {
		errc <- tester.sync("peer", nil, mode)
	}()
	// Iteratively take some blocks, always checking the retrieval count
	for {
		// Check the retrieval count synchronously (! reason for this ugly block)
		tester.lock.RLock()
		retrieved := int(tester.chain.CurrentSnapBlock().Number.Uint64()) + 1
		tester.lock.RUnlock()
		if retrieved >= targetBlocks+1 {
			break
		}
		// Wait a bit for sync to throttle itself
		var cached, frozen int
		for start := time.Now(); time.Since(start) < 3*time.Second; {
			time.Sleep(25 * time.Millisecond)

			tester.lock.Lock()
			tester.downloader.queue.lock.Lock()
			tester.downloader.queue.resultCache.lock.Lock()
			{
				cached = tester.downloader.queue.resultCache.countCompleted()
				frozen = int(blocked.Load())
				retrieved = int(tester.chain.CurrentSnapBlock().Number.Uint64()) + 1
			}
			tester.downloader.queue.resultCache.lock.Unlock()
			tester.downloader.queue.lock.Unlock()
			tester.lock.Unlock()

			if cached == blockCacheMaxItems ||
				cached == blockCacheMaxItems-reorgProtHeaderDelay ||
				retrieved+cached+frozen == targetBlocks+1 ||
				retrieved+cached+frozen == targetBlocks+1-reorgProtHeaderDelay {
				break
			}
		}
		// Make sure we filled up the cache, then exhaust it
		time.Sleep(25 * time.Millisecond) // give it a chance to screw up
		tester.lock.RLock()
		retrieved = int(tester.chain.CurrentSnapBlock().Number.Uint64()) + 1
		tester.lock.RUnlock()
		if cached != blockCacheMaxItems && cached != blockCacheMaxItems-reorgProtHeaderDelay && retrieved+cached+frozen != targetBlocks+1 && retrieved+cached+frozen != targetBlocks+1-reorgProtHeaderDelay {
			t.Fatalf("block count mismatch: have %v, want %v (owned %v, blocked %v, target %v)", cached, blockCacheMaxItems, retrieved, frozen, targetBlocks+1)
		}
		// Permit the blocked blocks to import
		if blocked.Load() > 0 {
			blocked.Store(uint32(0))
			proceed <- struct{}{}
		}
	}
	// Check that we haven't pulled more blocks than available
	assertOwnChain(t, tester, targetBlocks+1)
	if err := <-errc; err != nil {
		t.Fatalf("block synchronization failed: %v", err)
	}
}

// Tests that simple synchronization against a forked chain works correctly. In
// this test common ancestor lookup should *not* be short circuited, and a full
// binary search should be executed.
func TestForkedSync68Full(t *testing.T)  { testForkedSync(t, eth.ETH68, FullSync) }
func TestForkedSync68Snap(t *testing.T)  { testForkedSync(t, eth.ETH68, SnapSync) }
func TestForkedSync68Light(t *testing.T) { testForkedSync(t, eth.ETH68, LightSync) }
func TestForkedSync67Full(t *testing.T)  { testForkedSync(t, eth.ETH67, FullSync) }
func TestForkedSync67Snap(t *testing.T)  { testForkedSync(t, eth.ETH67, SnapSync) }
func TestForkedSync67Light(t *testing.T) { testForkedSync(t, eth.ETH67, LightSync) }

func testForkedSync(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chainA := testChainForkLightA.shorten(len(testChainBase.blocks) + 80)
	chainB := testChainForkLightB.shorten(len(testChainBase.blocks) + 81)
	tester.newPeer("fork A", protocol, chainA.blocks[1:])
	tester.newPeer("fork B", protocol, chainB.blocks[1:])
	// Synchronise with the peer and make sure all blocks were retrieved
	if err := tester.sync("fork A", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chainA.blocks))

	// Synchronise with the second peer and make sure that fork is pulled too
	if err := tester.sync("fork B", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chainB.blocks))
}

// Tests that synchronising against a much shorter but much heavier fork works
// currently and is not dropped.
func TestHeavyForkedSync68Full(t *testing.T)  { testHeavyForkedSync(t, eth.ETH68, FullSync) }
func TestHeavyForkedSync68Snap(t *testing.T)  { testHeavyForkedSync(t, eth.ETH68, SnapSync) }
func TestHeavyForkedSync68Light(t *testing.T) { testHeavyForkedSync(t, eth.ETH68, LightSync) }
func TestHeavyForkedSync67Full(t *testing.T)  { testHeavyForkedSync(t, eth.ETH67, FullSync) }
func TestHeavyForkedSync67Snap(t *testing.T)  { testHeavyForkedSync(t, eth.ETH67, SnapSync) }
func TestHeavyForkedSync67Light(t *testing.T) { testHeavyForkedSync(t, eth.ETH67, LightSync) }

func testHeavyForkedSync(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chainA := testChainForkLightA.shorten(len(testChainBase.blocks) + 80)
	chainB := testChainForkHeavy.shorten(len(testChainBase.blocks) + 79)
	tester.newPeer("light", protocol, chainA.blocks[1:])
	tester.newPeer("heavy", protocol, chainB.blocks[1:])

	// Synchronise with the peer and make sure all blocks were retrieved
	if err := tester.sync("light", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chainA.blocks))

	// Synchronise with the second peer and make sure that fork is pulled too
	if err := tester.sync("heavy", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chainB.blocks))
}

// Tests that chain forks are contained within a certain interval of the current
// chain head, ensuring that malicious peers cannot waste resources by feeding
// long dead chains.
func TestBoundedForkedSync68Full(t *testing.T)  { testBoundedForkedSync(t, eth.ETH68, FullSync) }
func TestBoundedForkedSync68Snap(t *testing.T)  { testBoundedForkedSync(t, eth.ETH68, SnapSync) }
func TestBoundedForkedSync68Light(t *testing.T) { testBoundedForkedSync(t, eth.ETH68, LightSync) }
func TestBoundedForkedSync67Full(t *testing.T)  { testBoundedForkedSync(t, eth.ETH67, FullSync) }
func TestBoundedForkedSync67Snap(t *testing.T)  { testBoundedForkedSync(t, eth.ETH67, SnapSync) }
func TestBoundedForkedSync67Light(t *testing.T) { testBoundedForkedSync(t, eth.ETH67, LightSync) }

func testBoundedForkedSync(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chainA := testChainForkLightA
	chainB := testChainForkLightB
	tester.newPeer("original", protocol, chainA.blocks[1:])
	tester.newPeer("rewriter", protocol, chainB.blocks[1:])

	// Synchronise with the peer and make sure all blocks were retrieved
	if err := tester.sync("original", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chainA.blocks))

	// Synchronise with the second peer and ensure that the fork is rejected to being too old
	if err := tester.sync("rewriter", nil, mode); err != errInvalidAncestor {
		t.Fatalf("sync failure mismatch: have %v, want %v", err, errInvalidAncestor)
	}
}

// Tests that chain forks are contained within a certain interval of the current
// chain head for short but heavy forks too. These are a bit special because they
// take different ancestor lookup paths.
func TestBoundedHeavyForkedSync68Full(t *testing.T) {
	testBoundedHeavyForkedSync(t, eth.ETH68, FullSync)
}
func TestBoundedHeavyForkedSync68Snap(t *testing.T) {
	testBoundedHeavyForkedSync(t, eth.ETH68, SnapSync)
}
func TestBoundedHeavyForkedSync68Light(t *testing.T) {
	testBoundedHeavyForkedSync(t, eth.ETH68, LightSync)
}
func TestBoundedHeavyForkedSync67Full(t *testing.T) {
	testBoundedHeavyForkedSync(t, eth.ETH67, FullSync)
}
func TestBoundedHeavyForkedSync67Snap(t *testing.T) {
	testBoundedHeavyForkedSync(t, eth.ETH67, SnapSync)
}
func TestBoundedHeavyForkedSync67Light(t *testing.T) {
	testBoundedHeavyForkedSync(t, eth.ETH67, LightSync)
}

func testBoundedHeavyForkedSync(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	// Create a long enough forked chain
	chainA := testChainForkLightA
	chainB := testChainForkHeavy
	tester.newPeer("original", protocol, chainA.blocks[1:])

	// Synchronise with the peer and make sure all blocks were retrieved
	if err := tester.sync("original", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chainA.blocks))

	tester.newPeer("heavy-rewriter", protocol, chainB.blocks[1:])
	// Synchronise with the second peer and ensure that the fork is rejected to being too old
	if err := tester.sync("heavy-rewriter", nil, mode); err != errInvalidAncestor {
		t.Fatalf("sync failure mismatch: have %v, want %v", err, errInvalidAncestor)
	}
}

// Tests that a canceled download wipes all previously accumulated state.
func TestCancel68Full(t *testing.T)  { testCancel(t, eth.ETH68, FullSync) }
func TestCancel68Snap(t *testing.T)  { testCancel(t, eth.ETH68, SnapSync) }
func TestCancel68Light(t *testing.T) { testCancel(t, eth.ETH68, LightSync) }
func TestCancel67Full(t *testing.T)  { testCancel(t, eth.ETH67, FullSync) }
func TestCancel67Snap(t *testing.T)  { testCancel(t, eth.ETH67, SnapSync) }
func TestCancel67Light(t *testing.T) { testCancel(t, eth.ETH67, LightSync) }

func testCancel(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(MaxHeaderFetch)
	tester.newPeer("peer", protocol, chain.blocks[1:])

	// Make sure canceling works with a pristine downloader
	tester.downloader.Cancel()
	if !tester.downloader.queue.Idle() {
		t.Errorf("download queue not idle")
	}
	// Synchronise with the peer, but cancel afterwards
	if err := tester.sync("peer", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	tester.downloader.Cancel()
	if !tester.downloader.queue.Idle() {
		t.Errorf("download queue not idle")
	}
}

// Tests that synchronisation from multiple peers works as intended (multi thread sanity test).
func TestMultiSynchronisation68Full(t *testing.T)  { testMultiSynchronisation(t, eth.ETH68, FullSync) }
func TestMultiSynchronisation68Snap(t *testing.T)  { testMultiSynchronisation(t, eth.ETH68, SnapSync) }
func TestMultiSynchronisation68Light(t *testing.T) { testMultiSynchronisation(t, eth.ETH68, LightSync) }
func TestMultiSynchronisation67Full(t *testing.T)  { testMultiSynchronisation(t, eth.ETH67, FullSync) }
func TestMultiSynchronisation67Snap(t *testing.T)  { testMultiSynchronisation(t, eth.ETH67, SnapSync) }
func TestMultiSynchronisation67Light(t *testing.T) { testMultiSynchronisation(t, eth.ETH67, LightSync) }

func testMultiSynchronisation(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	// Create various peers with various parts of the chain
	targetPeers := 8
	chain := testChainBase.shorten(targetPeers * 100)

	for i := 0; i < targetPeers; i++ {
		id := fmt.Sprintf("peer #%d", i)
		tester.newPeer(id, protocol, chain.shorten(len(chain.blocks) / (i + 1)).blocks[1:])
	}
	if err := tester.sync("peer #0", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chain.blocks))
}

// Tests that synchronisations behave well in multi-version protocol environments
// and not wreak havoc on other nodes in the network.
func TestMultiProtoSynchronisation68Full(t *testing.T)  { testMultiProtoSync(t, eth.ETH68, FullSync) }
func TestMultiProtoSynchronisation68Snap(t *testing.T)  { testMultiProtoSync(t, eth.ETH68, SnapSync) }
func TestMultiProtoSynchronisation68Light(t *testing.T) { testMultiProtoSync(t, eth.ETH68, LightSync) }
func TestMultiProtoSynchronisation67Full(t *testing.T)  { testMultiProtoSync(t, eth.ETH67, FullSync) }
func TestMultiProtoSynchronisation67Snap(t *testing.T)  { testMultiProtoSync(t, eth.ETH67, SnapSync) }
func TestMultiProtoSynchronisation67Light(t *testing.T) { testMultiProtoSync(t, eth.ETH67, LightSync) }

func testMultiProtoSync(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	// Create a small enough block chain to download
	chain := testChainBase.shorten(blockCacheMaxItems - 15)

	// Create peers of every type
	tester.newPeer("peer 68", eth.ETH68, chain.blocks[1:])
	tester.newPeer("peer 67", eth.ETH67, chain.blocks[1:])

	// Synchronise with the requested peer and make sure all blocks were retrieved
	if err := tester.sync(fmt.Sprintf("peer %d", protocol), nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chain.blocks))

	// Check that no peers have been dropped off
	for _, version := range []int{68, 67} {
		peer := fmt.Sprintf("peer %d", version)
		if _, ok := tester.peers[peer]; !ok {
			t.Errorf("%s dropped", peer)
		}
	}
}

// Tests that if a block is empty (e.g. header only), no body request should be
// made, and instead the header should be assembled into a whole block in itself.
func TestEmptyShortCircuit68Full(t *testing.T)  { testEmptyShortCircuit(t, eth.ETH68, FullSync) }
func TestEmptyShortCircuit68Snap(t *testing.T)  { testEmptyShortCircuit(t, eth.ETH68, SnapSync) }
func TestEmptyShortCircuit68Light(t *testing.T) { testEmptyShortCircuit(t, eth.ETH68, LightSync) }
func TestEmptyShortCircuit67Full(t *testing.T)  { testEmptyShortCircuit(t, eth.ETH67, FullSync) }
func TestEmptyShortCircuit67Snap(t *testing.T)  { testEmptyShortCircuit(t, eth.ETH67, SnapSync) }
func TestEmptyShortCircuit67Light(t *testing.T) { testEmptyShortCircuit(t, eth.ETH67, LightSync) }

func testEmptyShortCircuit(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	// Create a block chain to download
	chain := testChainBase
	tester.newPeer("peer", protocol, chain.blocks[1:])

	// Instrument the downloader to signal body requests
	var bodiesHave, receiptsHave atomic.Int32
	tester.downloader.bodyFetchHook = func(headers []*types.Header) {
		bodiesHave.Add(int32(len(headers)))
	}
	tester.downloader.receiptFetchHook = func(headers []*types.Header) {
		receiptsHave.Add(int32(len(headers)))
	}
	// Synchronise with the peer and make sure all blocks were retrieved
	if err := tester.sync("peer", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chain.blocks))

	// Validate the number of block bodies that should have been requested
	bodiesNeeded, receiptsNeeded := 0, 0
	for _, block := range chain.blocks[1:] {
		if mode != LightSync && (len(block.Transactions()) > 0 || len(block.Uncles()) > 0) {
			bodiesNeeded++
		}
	}
	for _, block := range chain.blocks[1:] {
		if mode == SnapSync && len(block.Transactions()) > 0 {
			receiptsNeeded++
		}
	}
	if int(bodiesHave.Load()) != bodiesNeeded {
		t.Errorf("body retrieval count mismatch: have %v, want %v", bodiesHave.Load(), bodiesNeeded)
	}
	if int(receiptsHave.Load()) != receiptsNeeded {
		t.Errorf("receipt retrieval count mismatch: have %v, want %v", receiptsHave.Load(), receiptsNeeded)
	}
}

// Tests that headers are enqueued continuously, preventing malicious nodes from
// stalling the downloader by feeding gapped header chains.
func TestMissingHeaderAttack68Full(t *testing.T)  { testMissingHeaderAttack(t, eth.ETH68, FullSync) }
func TestMissingHeaderAttack68Snap(t *testing.T)  { testMissingHeaderAttack(t, eth.ETH68, SnapSync) }
func TestMissingHeaderAttack68Light(t *testing.T) { testMissingHeaderAttack(t, eth.ETH68, LightSync) }
func TestMissingHeaderAttack67Full(t *testing.T)  { testMissingHeaderAttack(t, eth.ETH67, FullSync) }
func TestMissingHeaderAttack67Snap(t *testing.T)  { testMissingHeaderAttack(t, eth.ETH67, SnapSync) }
func TestMissingHeaderAttack67Light(t *testing.T) { testMissingHeaderAttack(t, eth.ETH67, LightSync) }

func testMissingHeaderAttack(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(blockCacheMaxItems - 15)

	attacker := tester.newPeer("attack", protocol, chain.blocks[1:])
	attacker.withholdHeaders[chain.blocks[len(chain.blocks)/2-1].Hash()] = struct{}{}

	if err := tester.sync("attack", nil, mode); err == nil {
		t.Fatalf("succeeded attacker synchronisation")
	}
	// Synchronise with the valid peer and make sure sync succeeds
	tester.newPeer("valid", protocol, chain.blocks[1:])
	if err := tester.sync("valid", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chain.blocks))
}

// Tests that if requested headers are shifted (i.e. first is missing), the queue
// detects the invalid numbering.
func TestShiftedHeaderAttack68Full(t *testing.T)  { testShiftedHeaderAttack(t, eth.ETH68, FullSync) }
func TestShiftedHeaderAttack68Snap(t *testing.T)  { testShiftedHeaderAttack(t, eth.ETH68, SnapSync) }
func TestShiftedHeaderAttack68Light(t *testing.T) { testShiftedHeaderAttack(t, eth.ETH68, LightSync) }
func TestShiftedHeaderAttack67Full(t *testing.T)  { testShiftedHeaderAttack(t, eth.ETH67, FullSync) }
func TestShiftedHeaderAttack67Snap(t *testing.T)  { testShiftedHeaderAttack(t, eth.ETH67, SnapSync) }
func TestShiftedHeaderAttack67Light(t *testing.T) { testShiftedHeaderAttack(t, eth.ETH67, LightSync) }

func testShiftedHeaderAttack(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(blockCacheMaxItems - 15)

	// Attempt a full sync with an attacker feeding shifted headers
	attacker := tester.newPeer("attack", protocol, chain.blocks[1:])
	attacker.withholdHeaders[chain.blocks[1].Hash()] = struct{}{}

	if err := tester.sync("attack", nil, mode); err == nil {
		t.Fatalf("succeeded attacker synchronisation")
	}
	// Synchronise with the valid peer and make sure sync succeeds
	tester.newPeer("valid", protocol, chain.blocks[1:])
	if err := tester.sync("valid", nil, mode); err != nil {
		t.Fatalf("failed to synchronise blocks: %v", err)
	}
	assertOwnChain(t, tester, len(chain.blocks))
}

// Tests that a peer advertising a high TD doesn't get to stall the downloader
// afterwards by not sending any useful hashes.
func TestHighTDStarvationAttack68Full(t *testing.T) {
	testHighTDStarvationAttack(t, eth.ETH68, FullSync)
}
func TestHighTDStarvationAttack68Snap(t *testing.T) {
	testHighTDStarvationAttack(t, eth.ETH68, SnapSync)
}
func TestHighTDStarvationAttack68Light(t *testing.T) {
	testHighTDStarvationAttack(t, eth.ETH68, LightSync)
}
func TestHighTDStarvationAttack67Full(t *testing.T) {
	testHighTDStarvationAttack(t, eth.ETH67, FullSync)
}
func TestHighTDStarvationAttack67Snap(t *testing.T) {
	testHighTDStarvationAttack(t, eth.ETH67, SnapSync)
}
func TestHighTDStarvationAttack67Light(t *testing.T) {
	testHighTDStarvationAttack(t, eth.ETH67, LightSync)
}

func testHighTDStarvationAttack(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(1)
	tester.newPeer("attack", protocol, chain.blocks[1:])
	if err := tester.sync("attack", big.NewInt(1000000), mode); err != errStallingPeer {
		t.Fatalf("synchronisation error mismatch: have %v, want %v", err, errStallingPeer)
	}
}

// Tests that misbehaving peers are disconnected, whilst behaving ones are not.
func TestBlockHeaderAttackerDropping68(t *testing.T) { testBlockHeaderAttackerDropping(t, eth.ETH68) }
func TestBlockHeaderAttackerDropping67(t *testing.T) { testBlockHeaderAttackerDropping(t, eth.ETH67) }

func testBlockHeaderAttackerDropping(t *testing.T, protocol uint) {
	// Define the disconnection requirement for individual hash fetch errors
	tests := []struct {
		result error
		drop   bool
	}{
		{nil, false},                        // Sync succeeded, all is well
		{errBusy, false},                    // Sync is already in progress, no problem
		{errUnknownPeer, false},             // Peer is unknown, was already dropped, don't double drop
		{errBadPeer, true},                  // Peer was deemed bad for some reason, drop it
		{errStallingPeer, true},             // Peer was detected to be stalling, drop it
		{errUnsyncedPeer, true},             // Peer was detected to be unsynced, drop it
		{errNoPeers, false},                 // No peers to download from, soft race, no issue
		{errTimeout, true},                  // No hashes received in due time, drop the peer
		{errEmptyHeaderSet, true},           // No headers were returned as a response, drop as it's a dead end
		{errPeersUnavailable, true},         // Nobody had the advertised blocks, drop the advertiser
		{errInvalidAncestor, true},          // Agreed upon ancestor is not acceptable, drop the chain rewriter
		{errInvalidChain, true},             // Hash chain was detected as invalid, definitely drop
		{errInvalidBody, false},             // A bad peer was detected, but not the sync origin
		{errInvalidReceipt, false},          // A bad peer was detected, but not the sync origin
		{errCancelContentProcessing, false}, // Synchronisation was canceled, origin may be innocent, don't drop
	}
	// Run the tests and check disconnection status
	tester := newTester(t)
	defer tester.terminate()
	chain := testChainBase.shorten(1)

	for i, tt := range tests {
		// Register a new peer and ensure its presence
		id := fmt.Sprintf("test %d", i)
		tester.newPeer(id, protocol, chain.blocks[1:])
		if _, ok := tester.peers[id]; !ok {
			t.Fatalf("test %d: registered peer not found", i)
		}
		// Simulate a synchronisation and check the required result
		tester.downloader.synchroniseMock = func(string, common.Hash) error { return tt.result }

		tester.downloader.LegacySync(id, tester.chain.Genesis().Hash(), big.NewInt(1000), nil, FullSync)
		if _, ok := tester.peers[id]; !ok != tt.drop {
			t.Errorf("test %d: peer drop mismatch for %v: have %v, want %v", i, tt.result, !ok, tt.drop)
		}
	}
}

// Tests that synchronisation progress (origin block number, current block number
// and highest block number) is tracked and updated correctly.
func TestSyncProgress68Full(t *testing.T)  { testSyncProgress(t, eth.ETH68, FullSync) }
func TestSyncProgress68Snap(t *testing.T)  { testSyncProgress(t, eth.ETH68, SnapSync) }
func TestSyncProgress68Light(t *testing.T) { testSyncProgress(t, eth.ETH68, LightSync) }
func TestSyncProgress67Full(t *testing.T)  { testSyncProgress(t, eth.ETH67, FullSync) }
func TestSyncProgress67Snap(t *testing.T)  { testSyncProgress(t, eth.ETH67, SnapSync) }
func TestSyncProgress67Light(t *testing.T) { testSyncProgress(t, eth.ETH67, LightSync) }

func testSyncProgress(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(blockCacheMaxItems - 15)

	// Set a sync init hook to catch progress changes
	starting := make(chan struct{})
	progress := make(chan struct{})

	tester.downloader.syncInitHook = func(origin, latest uint64) {
		starting <- struct{}{}
		<-progress
	}
	checkProgress(t, tester.downloader, "pristine", ethereum.SyncProgress{})

	// Synchronise half the blocks and check initial progress
	tester.newPeer("peer-half", protocol, chain.shorten(len(chain.blocks) / 2).blocks[1:])
	pending := new(sync.WaitGroup)
	pending.Add(1)

	go func() {
		defer pending.Done()
		if err := tester.sync("peer-half", nil, mode); err != nil {
			panic(fmt.Sprintf("failed to synchronise blocks: %v", err))
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "initial", ethereum.SyncProgress{
		HighestBlock: uint64(len(chain.blocks)/2 - 1),
	})
	progress <- struct{}{}
	pending.Wait()

	// Synchronise all the blocks and check continuation progress
	tester.newPeer("peer-full", protocol, chain.blocks[1:])
	pending.Add(1)
	go func() {
		defer pending.Done()
		if err := tester.sync("peer-full", nil, mode); err != nil {
			panic(fmt.Sprintf("failed to synchronise blocks: %v", err))
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "completing", ethereum.SyncProgress{
		StartingBlock: uint64(len(chain.blocks)/2 - 1),
		CurrentBlock:  uint64(len(chain.blocks)/2 - 1),
		HighestBlock:  uint64(len(chain.blocks) - 1),
	})

	// Check final progress after successful sync
	progress <- struct{}{}
	pending.Wait()
	checkProgress(t, tester.downloader, "final", ethereum.SyncProgress{
		StartingBlock: uint64(len(chain.blocks)/2 - 1),
		CurrentBlock:  uint64(len(chain.blocks) - 1),
		HighestBlock:  uint64(len(chain.blocks) - 1),
	})
}

func checkProgress(t *testing.T, d *Downloader, stage string, want ethereum.SyncProgress) {
	// Mark this method as a helper to report errors at callsite, not in here
	t.Helper()

	p := d.Progress()
	if p.StartingBlock != want.StartingBlock || p.CurrentBlock != want.CurrentBlock || p.HighestBlock != want.HighestBlock {
		t.Fatalf("%s progress mismatch:\nhave %+v\nwant %+v", stage, p, want)
	}
}

// Tests that synchronisation progress (origin block number and highest block
// number) is tracked and updated correctly in case of a fork (or manual head
// revertal).
func TestForkedSyncProgress68Full(t *testing.T)  { testForkedSyncProgress(t, eth.ETH68, FullSync) }
func TestForkedSyncProgress68Snap(t *testing.T)  { testForkedSyncProgress(t, eth.ETH68, SnapSync) }
func TestForkedSyncProgress68Light(t *testing.T) { testForkedSyncProgress(t, eth.ETH68, LightSync) }
func TestForkedSyncProgress67Full(t *testing.T)  { testForkedSyncProgress(t, eth.ETH67, FullSync) }
func TestForkedSyncProgress67Snap(t *testing.T)  { testForkedSyncProgress(t, eth.ETH67, SnapSync) }
func TestForkedSyncProgress67Light(t *testing.T) { testForkedSyncProgress(t, eth.ETH67, LightSync) }

func testForkedSyncProgress(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chainA := testChainForkLightA.shorten(len(testChainBase.blocks) + MaxHeaderFetch)
	chainB := testChainForkLightB.shorten(len(testChainBase.blocks) + MaxHeaderFetch)

	// Set a sync init hook to catch progress changes
	starting := make(chan struct{})
	progress := make(chan struct{})

	tester.downloader.syncInitHook = func(origin, latest uint64) {
		starting <- struct{}{}
		<-progress
	}
	checkProgress(t, tester.downloader, "pristine", ethereum.SyncProgress{})

	// Synchronise with one of the forks and check progress
	tester.newPeer("fork A", protocol, chainA.blocks[1:])
	pending := new(sync.WaitGroup)
	pending.Add(1)
	go func() {
		defer pending.Done()
		if err := tester.sync("fork A", nil, mode); err != nil {
			panic(fmt.Sprintf("failed to synchronise blocks: %v", err))
		}
	}()
	<-starting

	checkProgress(t, tester.downloader, "initial", ethereum.SyncProgress{
		HighestBlock: uint64(len(chainA.blocks) - 1),
	})
	progress <- struct{}{}
	pending.Wait()

	// Simulate a successful sync above the fork
	tester.downloader.syncStatsChainOrigin = tester.downloader.syncStatsChainHeight

	// Synchronise with the second fork and check progress resets
	tester.newPeer("fork B", protocol, chainB.blocks[1:])
	pending.Add(1)
	go func() {
		defer pending.Done()
		if err := tester.sync("fork B", nil, mode); err != nil {
			panic(fmt.Sprintf("failed to synchronise blocks: %v", err))
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "forking", ethereum.SyncProgress{
		StartingBlock: uint64(len(testChainBase.blocks)) - 1,
		CurrentBlock:  uint64(len(chainA.blocks) - 1),
		HighestBlock:  uint64(len(chainB.blocks) - 1),
	})

	// Check final progress after successful sync
	progress <- struct{}{}
	pending.Wait()
	checkProgress(t, tester.downloader, "final", ethereum.SyncProgress{
		StartingBlock: uint64(len(testChainBase.blocks)) - 1,
		CurrentBlock:  uint64(len(chainB.blocks) - 1),
		HighestBlock:  uint64(len(chainB.blocks) - 1),
	})
}

// Tests that if synchronisation is aborted due to some failure, then the progress
// origin is not updated in the next sync cycle, as it should be considered the
// continuation of the previous sync and not a new instance.
func TestFailedSyncProgress68Full(t *testing.T)  { testFailedSyncProgress(t, eth.ETH68, FullSync) }
func TestFailedSyncProgress68Snap(t *testing.T)  { testFailedSyncProgress(t, eth.ETH68, SnapSync) }
func TestFailedSyncProgress68Light(t *testing.T) { testFailedSyncProgress(t, eth.ETH68, LightSync) }
func TestFailedSyncProgress67Full(t *testing.T)  { testFailedSyncProgress(t, eth.ETH67, FullSync) }
func TestFailedSyncProgress67Snap(t *testing.T)  { testFailedSyncProgress(t, eth.ETH67, SnapSync) }
func TestFailedSyncProgress67Light(t *testing.T) { testFailedSyncProgress(t, eth.ETH67, LightSync) }

func testFailedSyncProgress(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(blockCacheMaxItems - 15)

	// Set a sync init hook to catch progress changes
	starting := make(chan struct{})
	progress := make(chan struct{})

	tester.downloader.syncInitHook = func(origin, latest uint64) {
		starting <- struct{}{}
		<-progress
	}
	checkProgress(t, tester.downloader, "pristine", ethereum.SyncProgress{})

	// Attempt a full sync with a faulty peer
	missing := len(chain.blocks)/2 - 1

	faulter := tester.newPeer("faulty", protocol, chain.blocks[1:])
	faulter.withholdHeaders[chain.blocks[missing].Hash()] = struct{}{}

	pending := new(sync.WaitGroup)
	pending.Add(1)
	go func() {
		defer pending.Done()
		if err := tester.sync("faulty", nil, mode); err == nil {
			panic("succeeded faulty synchronisation")
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "initial", ethereum.SyncProgress{
		HighestBlock: uint64(len(chain.blocks) - 1),
	})
	progress <- struct{}{}
	pending.Wait()
	afterFailedSync := tester.downloader.Progress()

	// Synchronise with a good peer and check that the progress origin remind the same
	// after a failure
	tester.newPeer("valid", protocol, chain.blocks[1:])
	pending.Add(1)
	go func() {
		defer pending.Done()
		if err := tester.sync("valid", nil, mode); err != nil {
			panic(fmt.Sprintf("failed to synchronise blocks: %v", err))
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "completing", afterFailedSync)

	// Check final progress after successful sync
	progress <- struct{}{}
	pending.Wait()
	checkProgress(t, tester.downloader, "final", ethereum.SyncProgress{
		CurrentBlock: uint64(len(chain.blocks) - 1),
		HighestBlock: uint64(len(chain.blocks) - 1),
	})
}

// Tests that if an attacker fakes a chain height, after the attack is detected,
// the progress height is successfully reduced at the next sync invocation.
func TestFakedSyncProgress68Full(t *testing.T)  { testFakedSyncProgress(t, eth.ETH68, FullSync) }
func TestFakedSyncProgress68Snap(t *testing.T)  { testFakedSyncProgress(t, eth.ETH68, SnapSync) }
func TestFakedSyncProgress68Light(t *testing.T) { testFakedSyncProgress(t, eth.ETH68, LightSync) }
func TestFakedSyncProgress67Full(t *testing.T)  { testFakedSyncProgress(t, eth.ETH67, FullSync) }
func TestFakedSyncProgress67Snap(t *testing.T)  { testFakedSyncProgress(t, eth.ETH67, SnapSync) }
func TestFakedSyncProgress67Light(t *testing.T) { testFakedSyncProgress(t, eth.ETH67, LightSync) }

func testFakedSyncProgress(t *testing.T, protocol uint, mode SyncMode) {
	tester := newTester(t)
	defer tester.terminate()

	chain := testChainBase.shorten(blockCacheMaxItems - 15)

	// Set a sync init hook to catch progress changes
	starting := make(chan struct{})
	progress := make(chan struct{})
	tester.downloader.syncInitHook = func(origin, latest uint64) {
		starting <- struct{}{}
		<-progress
	}
	checkProgress(t, tester.downloader, "pristine", ethereum.SyncProgress{})

	// Create and sync with an attacker that promises a higher chain than available.
	attacker := tester.newPeer("attack", protocol, chain.blocks[1:])
	numMissing := 5
	for i := len(chain.blocks) - 2; i > len(chain.blocks)-numMissing; i-- {
		attacker.withholdHeaders[chain.blocks[i].Hash()] = struct{}{}
	}
	pending := new(sync.WaitGroup)
	pending.Add(1)
	go func() {
		defer pending.Done()
		if err := tester.sync("attack", nil, mode); err == nil {
			panic("succeeded attacker synchronisation")
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "initial", ethereum.SyncProgress{
		HighestBlock: uint64(len(chain.blocks) - 1),
	})
	progress <- struct{}{}
	pending.Wait()
	afterFailedSync := tester.downloader.Progress()

	// Synchronise with a good peer and check that the progress height has been reduced to
	// the true value.
	validChain := chain.shorten(len(chain.blocks) - numMissing)
	tester.newPeer("valid", protocol, validChain.blocks[1:])
	pending.Add(1)

	go func() {
		defer pending.Done()
		if err := tester.sync("valid", nil, mode); err != nil {
			panic(fmt.Sprintf("failed to synchronise blocks: %v", err))
		}
	}()
	<-starting
	checkProgress(t, tester.downloader, "completing", ethereum.SyncProgress{
		CurrentBlock: afterFailedSync.CurrentBlock,
		HighestBlock: uint64(len(validChain.blocks) - 1),
	})
	// Check final progress after successful sync.
	progress <- struct{}{}
	pending.Wait()
	checkProgress(t, tester.downloader, "final", ethereum.SyncProgress{
		CurrentBlock: uint64(len(validChain.blocks) - 1),
		HighestBlock: uint64(len(validChain.blocks) - 1),
	})
}

func TestRemoteHeaderRequestSpan(t *testing.T) {
	testCases := []struct {
		remoteHeight uint64
		localHeight  uint64
		expected     []int
	}{
		// Remote is way higher. We should ask for the remote head and go backwards
		{1500, 1000,
			[]int{1323, 1339, 1355, 1371, 1387, 1403, 1419, 1435, 1451, 1467, 1483, 1499},
		},
		{15000, 13006,
			[]int{14823, 14839, 14855, 14871, 14887, 14903, 14919, 14935, 14951, 14967, 14983, 14999},
		},
		// Remote is pretty close to us. We don't have to fetch as many
		{1200, 1150,
			[]int{1149, 1154, 1159, 1164, 1169, 1174, 1179, 1184, 1189, 1194, 1199},
		},
		// Remote is equal to us (so on a fork with higher td)
		// We should get the closest couple of ancestors
		{1500, 1500,
			[]int{1497, 1499},
		},
		// We're higher than the remote! Odd
		{1000, 1500,
			[]int{997, 999},
		},
		// Check some weird edgecases that it behaves somewhat rationally
		{0, 1500,
			[]int{0, 2},
		},
		{6000000, 0,
			[]int{5999823, 5999839, 5999855, 5999871, 5999887, 5999903, 5999919, 5999935, 5999951, 5999967, 5999983, 5999999},
		},
		{0, 0,
			[]int{0, 2},
		},
	}
	reqs := func(from, count, span int) []int {
		var r []int
		num := from
		for len(r) < count {
			r = append(r, num)
			num += span + 1
		}
		return r
	}
	for i, tt := range testCases {
		from, count, span, max := calculateRequestSpan(tt.remoteHeight, tt.localHeight)
		data := reqs(int(from), count, span)

		if max != uint64(data[len(data)-1]) {
			t.Errorf("test %d: wrong last value %d != %d", i, data[len(data)-1], max)
		}
		failed := false
		if len(data) != len(tt.expected) {
			failed = true
			t.Errorf("test %d: length wrong, expected %d got %d", i, len(tt.expected), len(data))
		} else {
			for j, n := range data {
				if n != tt.expected[j] {
					failed = true
					break
				}
			}
		}
		if failed {
			res := strings.ReplaceAll(fmt.Sprint(data), " ", ",")
			exp := strings.ReplaceAll(fmt.Sprint(tt.expected), " ", ",")
			t.Logf("got: %v\n", res)
			t.Logf("exp: %v\n", exp)
			t.Errorf("test %d: wrong values", i)
		}
	}
}

// Tests that peers below a pre-configured checkpoint block are prevented from
// being fast-synced from, avoiding potential cheap eclipse attacks.
func TestBeaconSync68Full(t *testing.T) { testBeaconSync(t, eth.ETH68, FullSync) }
func TestBeaconSync68Snap(t *testing.T) { testBeaconSync(t, eth.ETH68, SnapSync) }
func TestBeaconSync67Full(t *testing.T) { testBeaconSync(t, eth.ETH67, FullSync) }
func TestBeaconSync67Snap(t *testing.T) { testBeaconSync(t, eth.ETH67, SnapSync) }

func testBeaconSync(t *testing.T, protocol uint, mode SyncMode) {
	//log.Root().SetHandler(log.LvlFilterHandler(log.LvlInfo, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	var cases = []struct {
		name  string // The name of testing scenario
		local int    // The length of local chain(canonical chain assumed), 0 means genesis is the head
	}{
		{name: "Beacon sync since genesis", local: 0},
		{name: "Beacon sync with short local chain", local: 1},
		{name: "Beacon sync with long local chain", local: blockCacheMaxItems - 15 - fsMinFullBlocks/2},
		{name: "Beacon sync with full local chain", local: blockCacheMaxItems - 15 - 1},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			success := make(chan struct{})
			tester := newTesterWithNotification(t, func() {
				close(success)
			})
			defer tester.terminate()

			chain := testChainBase.shorten(blockCacheMaxItems - 15)
			tester.newPeer("peer", protocol, chain.blocks[1:])

			// Build the local chain segment if it's required
			if c.local > 0 {
				tester.chain.InsertChain(chain.blocks[1 : c.local+1])
			}
			if err := tester.downloader.BeaconSync(mode, chain.blocks[len(chain.blocks)-1].Header(), nil); err != nil {
				t.Fatalf("Failed to beacon sync chain %v %v", c.name, err)
			}
			select {
			case <-success:
				// Ok, downloader fully cancelled after sync cycle
				if bs := int(tester.chain.CurrentBlock().Number.Uint64()) + 1; bs != len(chain.blocks) {
					t.Fatalf("synchronised blocks mismatch: have %v, want %v", bs, len(chain.blocks))
				}
			case <-time.NewTimer(time.Second * 3).C:
				t.Fatalf("Failed to sync chain in three seconds")
			}
		})
	}
}
