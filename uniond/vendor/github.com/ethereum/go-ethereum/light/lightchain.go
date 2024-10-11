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

// Package light implements on-demand retrieval capable state and chain objects
// for the Ethereum Light Client.
package light

import (
	"context"
	"errors"
	"math/big"
	"sync"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/lru"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
)

var (
	bodyCacheLimit  = 256
	blockCacheLimit = 256
)

// LightChain represents a canonical chain that by default only handles block
// headers, downloading block bodies and receipts on demand through an ODR
// interface. It only does header validation during chain insertion.
type LightChain struct {
	hc            *core.HeaderChain
	indexerConfig *IndexerConfig
	chainDb       ethdb.Database
	engine        consensus.Engine
	odr           OdrBackend
	chainFeed     event.Feed
	chainSideFeed event.Feed
	chainHeadFeed event.Feed
	scope         event.SubscriptionScope
	genesisBlock  *types.Block
	forker        *core.ForkChoice

	bodyCache    *lru.Cache[common.Hash, *types.Body]
	bodyRLPCache *lru.Cache[common.Hash, rlp.RawValue]
	blockCache   *lru.Cache[common.Hash, *types.Block]

	chainmu sync.RWMutex // protects header inserts
	quit    chan struct{}
	wg      sync.WaitGroup

	// Atomic boolean switches:
	stopped       atomic.Bool // whether LightChain is stopped or running
	procInterrupt atomic.Bool // interrupts chain insert
}

// NewLightChain returns a fully initialised light chain using information
// available in the database. It initialises the default Ethereum header
// validator.
func NewLightChain(odr OdrBackend, config *params.ChainConfig, engine consensus.Engine) (*LightChain, error) {
	bc := &LightChain{
		chainDb:       odr.Database(),
		indexerConfig: odr.IndexerConfig(),
		odr:           odr,
		quit:          make(chan struct{}),
		bodyCache:     lru.NewCache[common.Hash, *types.Body](bodyCacheLimit),
		bodyRLPCache:  lru.NewCache[common.Hash, rlp.RawValue](bodyCacheLimit),
		blockCache:    lru.NewCache[common.Hash, *types.Block](blockCacheLimit),
		engine:        engine,
	}
	bc.forker = core.NewForkChoice(bc, nil)
	var err error
	bc.hc, err = core.NewHeaderChain(odr.Database(), config, bc.engine, bc.getProcInterrupt)
	if err != nil {
		return nil, err
	}
	bc.genesisBlock, _ = bc.GetBlockByNumber(NoOdr, 0)
	if bc.genesisBlock == nil {
		return nil, core.ErrNoGenesis
	}
	if err := bc.loadLastState(); err != nil {
		return nil, err
	}
	// Check the current state of the block hashes and make sure that we do not have any of the bad blocks in our chain
	for hash := range core.BadHashes {
		if header := bc.GetHeaderByHash(hash); header != nil {
			log.Error("Found bad hash, rewinding chain", "number", header.Number, "hash", header.ParentHash)
			bc.SetHead(header.Number.Uint64() - 1)
			log.Info("Chain rewind was successful, resuming normal operation")
		}
	}
	return bc, nil
}

func (lc *LightChain) getProcInterrupt() bool {
	return lc.procInterrupt.Load()
}

// Odr returns the ODR backend of the chain
func (lc *LightChain) Odr() OdrBackend {
	return lc.odr
}

// HeaderChain returns the underlying header chain.
func (lc *LightChain) HeaderChain() *core.HeaderChain {
	return lc.hc
}

// loadLastState loads the last known chain state from the database. This method
// assumes that the chain manager mutex is held.
func (lc *LightChain) loadLastState() error {
	if head := rawdb.ReadHeadHeaderHash(lc.chainDb); head == (common.Hash{}) {
		// Corrupt or empty database, init from scratch
		lc.Reset()
	} else {
		header := lc.GetHeaderByHash(head)
		if header == nil {
			// Corrupt or empty database, init from scratch
			lc.Reset()
		} else {
			lc.hc.SetCurrentHeader(header)
		}
	}
	// Issue a status log and return
	header := lc.hc.CurrentHeader()
	headerTd := lc.GetTd(header.Hash(), header.Number.Uint64())
	log.Info("Loaded most recent local header", "number", header.Number, "hash", header.Hash(), "td", headerTd, "age", common.PrettyAge(time.Unix(int64(header.Time), 0)))
	return nil
}

// SetHead rewinds the local chain to a new head. Everything above the new
// head will be deleted and the new one set.
func (lc *LightChain) SetHead(head uint64) error {
	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	lc.hc.SetHead(head, nil, nil)
	return lc.loadLastState()
}

// SetHeadWithTimestamp rewinds the local chain to a new head that has at max
// the given timestamp. Everything above the new head will be deleted and the
// new one set.
func (lc *LightChain) SetHeadWithTimestamp(timestamp uint64) error {
	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	lc.hc.SetHeadWithTimestamp(timestamp, nil, nil)
	return lc.loadLastState()
}

// GasLimit returns the gas limit of the current HEAD block.
func (lc *LightChain) GasLimit() uint64 {
	return lc.hc.CurrentHeader().GasLimit
}

// Reset purges the entire blockchain, restoring it to its genesis state.
func (lc *LightChain) Reset() {
	lc.ResetWithGenesisBlock(lc.genesisBlock)
}

// ResetWithGenesisBlock purges the entire blockchain, restoring it to the
// specified genesis state.
func (lc *LightChain) ResetWithGenesisBlock(genesis *types.Block) {
	// Dump the entire block chain and purge the caches
	lc.SetHead(0)

	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	// Prepare the genesis block and reinitialise the chain
	batch := lc.chainDb.NewBatch()
	rawdb.WriteTd(batch, genesis.Hash(), genesis.NumberU64(), genesis.Difficulty())
	rawdb.WriteBlock(batch, genesis)
	rawdb.WriteHeadHeaderHash(batch, genesis.Hash())
	if err := batch.Write(); err != nil {
		log.Crit("Failed to reset genesis block", "err", err)
	}
	lc.genesisBlock = genesis
	lc.hc.SetGenesis(lc.genesisBlock.Header())
	lc.hc.SetCurrentHeader(lc.genesisBlock.Header())
}

// Accessors

// Engine retrieves the light chain's consensus engine.
func (lc *LightChain) Engine() consensus.Engine { return lc.engine }

// Genesis returns the genesis block
func (lc *LightChain) Genesis() *types.Block {
	return lc.genesisBlock
}

func (lc *LightChain) StateCache() state.Database {
	panic("not implemented")
}

// GetBody retrieves a block body (transactions and uncles) from the database
// or ODR service by hash, caching it if found.
func (lc *LightChain) GetBody(ctx context.Context, hash common.Hash) (*types.Body, error) {
	// Short circuit if the body's already in the cache, retrieve otherwise
	if cached, ok := lc.bodyCache.Get(hash); ok {
		return cached, nil
	}
	number := lc.hc.GetBlockNumber(hash)
	if number == nil {
		return nil, errors.New("unknown block")
	}
	body, err := GetBody(ctx, lc.odr, hash, *number)
	if err != nil {
		return nil, err
	}
	// Cache the found body for next time and return
	lc.bodyCache.Add(hash, body)
	return body, nil
}

// GetBodyRLP retrieves a block body in RLP encoding from the database or
// ODR service by hash, caching it if found.
func (lc *LightChain) GetBodyRLP(ctx context.Context, hash common.Hash) (rlp.RawValue, error) {
	// Short circuit if the body's already in the cache, retrieve otherwise
	if cached, ok := lc.bodyRLPCache.Get(hash); ok {
		return cached, nil
	}
	number := lc.hc.GetBlockNumber(hash)
	if number == nil {
		return nil, errors.New("unknown block")
	}
	body, err := GetBodyRLP(ctx, lc.odr, hash, *number)
	if err != nil {
		return nil, err
	}
	// Cache the found body for next time and return
	lc.bodyRLPCache.Add(hash, body)
	return body, nil
}

// HasBlock checks if a block is fully present in the database or not, caching
// it if present.
func (lc *LightChain) HasBlock(hash common.Hash, number uint64) bool {
	blk, _ := lc.GetBlock(NoOdr, hash, number)
	return blk != nil
}

// GetBlock retrieves a block from the database or ODR service by hash and number,
// caching it if found.
func (lc *LightChain) GetBlock(ctx context.Context, hash common.Hash, number uint64) (*types.Block, error) {
	// Short circuit if the block's already in the cache, retrieve otherwise
	if block, ok := lc.blockCache.Get(hash); ok {
		return block, nil
	}
	block, err := GetBlock(ctx, lc.odr, hash, number)
	if err != nil {
		return nil, err
	}
	// Cache the found block for next time and return
	lc.blockCache.Add(block.Hash(), block)
	return block, nil
}

// GetBlockByHash retrieves a block from the database or ODR service by hash,
// caching it if found.
func (lc *LightChain) GetBlockByHash(ctx context.Context, hash common.Hash) (*types.Block, error) {
	number := lc.hc.GetBlockNumber(hash)
	if number == nil {
		return nil, errors.New("unknown block")
	}
	return lc.GetBlock(ctx, hash, *number)
}

// GetBlockByNumber retrieves a block from the database or ODR service by
// number, caching it (associated with its hash) if found.
func (lc *LightChain) GetBlockByNumber(ctx context.Context, number uint64) (*types.Block, error) {
	hash, err := GetCanonicalHash(ctx, lc.odr, number)
	if hash == (common.Hash{}) || err != nil {
		return nil, err
	}
	return lc.GetBlock(ctx, hash, number)
}

// Stop stops the blockchain service. If any imports are currently in progress
// it will abort them using the procInterrupt.
func (lc *LightChain) Stop() {
	if !lc.stopped.CompareAndSwap(false, true) {
		return
	}
	close(lc.quit)
	lc.StopInsert()
	lc.wg.Wait()
	log.Info("Blockchain stopped")
}

// StopInsert interrupts all insertion methods, causing them to return
// errInsertionInterrupted as soon as possible. Insertion is permanently disabled after
// calling this method.
func (lc *LightChain) StopInsert() {
	lc.procInterrupt.Store(true)
}

// Rollback is designed to remove a chain of links from the database that aren't
// certain enough to be valid.
func (lc *LightChain) Rollback(chain []common.Hash) {
	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	batch := lc.chainDb.NewBatch()
	for i := len(chain) - 1; i >= 0; i-- {
		hash := chain[i]

		// Degrade the chain markers if they are explicitly reverted.
		// In theory we should update all in-memory markers in the
		// last step, however the direction of rollback is from high
		// to low, so it's safe the update in-memory markers directly.
		if head := lc.hc.CurrentHeader(); head.Hash() == hash {
			rawdb.WriteHeadHeaderHash(batch, head.ParentHash)
			lc.hc.SetCurrentHeader(lc.GetHeader(head.ParentHash, head.Number.Uint64()-1))
		}
	}
	if err := batch.Write(); err != nil {
		log.Crit("Failed to rollback light chain", "error", err)
	}
}

func (lc *LightChain) InsertHeader(header *types.Header) error {
	// Verify the header first before obtaining the lock
	headers := []*types.Header{header}
	if _, err := lc.hc.ValidateHeaderChain(headers); err != nil {
		return err
	}
	// Make sure only one thread manipulates the chain at once
	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	lc.wg.Add(1)
	defer lc.wg.Done()

	_, err := lc.hc.WriteHeaders(headers)
	log.Info("Inserted header", "number", header.Number, "hash", header.Hash())
	return err
}

func (lc *LightChain) SetCanonical(header *types.Header) error {
	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	lc.wg.Add(1)
	defer lc.wg.Done()

	if err := lc.hc.Reorg([]*types.Header{header}); err != nil {
		return err
	}
	// Emit events
	block := types.NewBlockWithHeader(header)
	lc.chainFeed.Send(core.ChainEvent{Block: block, Hash: block.Hash()})
	lc.chainHeadFeed.Send(core.ChainHeadEvent{Block: block})
	log.Info("Set the chain head", "number", block.Number(), "hash", block.Hash())
	return nil
}

// InsertHeaderChain attempts to insert the given header chain in to the local
// chain, possibly creating a reorg. If an error is returned, it will return the
// index number of the failing header as well an error describing what went wrong.

// In the case of a light chain, InsertHeaderChain also creates and posts light
// chain events when necessary.
func (lc *LightChain) InsertHeaderChain(chain []*types.Header) (int, error) {
	if len(chain) == 0 {
		return 0, nil
	}
	start := time.Now()
	if i, err := lc.hc.ValidateHeaderChain(chain); err != nil {
		return i, err
	}

	// Make sure only one thread manipulates the chain at once
	lc.chainmu.Lock()
	defer lc.chainmu.Unlock()

	lc.wg.Add(1)
	defer lc.wg.Done()

	status, err := lc.hc.InsertHeaderChain(chain, start, lc.forker)
	if err != nil || len(chain) == 0 {
		return 0, err
	}

	// Create chain event for the new head block of this insertion.
	var (
		lastHeader = chain[len(chain)-1]
		block      = types.NewBlockWithHeader(lastHeader)
	)
	switch status {
	case core.CanonStatTy:
		lc.chainFeed.Send(core.ChainEvent{Block: block, Hash: block.Hash()})
		lc.chainHeadFeed.Send(core.ChainHeadEvent{Block: block})
	case core.SideStatTy:
		lc.chainSideFeed.Send(core.ChainSideEvent{Block: block})
	}
	return 0, err
}

// CurrentHeader retrieves the current head header of the canonical chain. The
// header is retrieved from the HeaderChain's internal cache.
func (lc *LightChain) CurrentHeader() *types.Header {
	return lc.hc.CurrentHeader()
}

// GetTd retrieves a block's total difficulty in the canonical chain from the
// database by hash and number, caching it if found.
func (lc *LightChain) GetTd(hash common.Hash, number uint64) *big.Int {
	return lc.hc.GetTd(hash, number)
}

// GetTdOdr retrieves the total difficult from the database or
// network by hash and number, caching it (associated with its hash) if found.
func (lc *LightChain) GetTdOdr(ctx context.Context, hash common.Hash, number uint64) *big.Int {
	td := lc.GetTd(hash, number)
	if td != nil {
		return td
	}
	td, _ = GetTd(ctx, lc.odr, hash, number)
	return td
}

// GetHeader retrieves a block header from the database by hash and number,
// caching it if found.
func (lc *LightChain) GetHeader(hash common.Hash, number uint64) *types.Header {
	return lc.hc.GetHeader(hash, number)
}

// GetHeaderByHash retrieves a block header from the database by hash, caching it if
// found.
func (lc *LightChain) GetHeaderByHash(hash common.Hash) *types.Header {
	return lc.hc.GetHeaderByHash(hash)
}

// HasHeader checks if a block header is present in the database or not, caching
// it if present.
func (lc *LightChain) HasHeader(hash common.Hash, number uint64) bool {
	return lc.hc.HasHeader(hash, number)
}

// GetCanonicalHash returns the canonical hash for a given block number
func (bc *LightChain) GetCanonicalHash(number uint64) common.Hash {
	return bc.hc.GetCanonicalHash(number)
}

// GetAncestor retrieves the Nth ancestor of a given block. It assumes that either the given block or
// a close ancestor of it is canonical. maxNonCanonical points to a downwards counter limiting the
// number of blocks to be individually checked before we reach the canonical chain.
//
// Note: ancestor == 0 returns the same block, 1 returns its parent and so on.
func (lc *LightChain) GetAncestor(hash common.Hash, number, ancestor uint64, maxNonCanonical *uint64) (common.Hash, uint64) {
	return lc.hc.GetAncestor(hash, number, ancestor, maxNonCanonical)
}

// GetHeaderByNumber retrieves a block header from the database by number,
// caching it (associated with its hash) if found.
func (lc *LightChain) GetHeaderByNumber(number uint64) *types.Header {
	return lc.hc.GetHeaderByNumber(number)
}

// GetHeaderByNumberOdr retrieves a block header from the database or network
// by number, caching it (associated with its hash) if found.
func (lc *LightChain) GetHeaderByNumberOdr(ctx context.Context, number uint64) (*types.Header, error) {
	if header := lc.hc.GetHeaderByNumber(number); header != nil {
		return header, nil
	}
	return GetHeaderByNumber(ctx, lc.odr, number)
}

// Config retrieves the header chain's chain configuration.
func (lc *LightChain) Config() *params.ChainConfig { return lc.hc.Config() }

// LockChain locks the chain mutex for reading so that multiple canonical hashes can be
// retrieved while it is guaranteed that they belong to the same version of the chain
func (lc *LightChain) LockChain() {
	lc.chainmu.RLock()
}

// UnlockChain unlocks the chain mutex
func (lc *LightChain) UnlockChain() {
	lc.chainmu.RUnlock()
}

// SubscribeChainEvent registers a subscription of ChainEvent.
func (lc *LightChain) SubscribeChainEvent(ch chan<- core.ChainEvent) event.Subscription {
	return lc.scope.Track(lc.chainFeed.Subscribe(ch))
}

// SubscribeChainHeadEvent registers a subscription of ChainHeadEvent.
func (lc *LightChain) SubscribeChainHeadEvent(ch chan<- core.ChainHeadEvent) event.Subscription {
	return lc.scope.Track(lc.chainHeadFeed.Subscribe(ch))
}

// SubscribeChainSideEvent registers a subscription of ChainSideEvent.
func (lc *LightChain) SubscribeChainSideEvent(ch chan<- core.ChainSideEvent) event.Subscription {
	return lc.scope.Track(lc.chainSideFeed.Subscribe(ch))
}

// SubscribeLogsEvent implements the interface of filters.Backend
// LightChain does not send logs events, so return an empty subscription.
func (lc *LightChain) SubscribeLogsEvent(ch chan<- []*types.Log) event.Subscription {
	return lc.scope.Track(new(event.Feed).Subscribe(ch))
}

// SubscribeRemovedLogsEvent implements the interface of filters.Backend
// LightChain does not send core.RemovedLogsEvent, so return an empty subscription.
func (lc *LightChain) SubscribeRemovedLogsEvent(ch chan<- core.RemovedLogsEvent) event.Subscription {
	return lc.scope.Track(new(event.Feed).Subscribe(ch))
}
