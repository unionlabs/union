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

package core

import (
	crand "crypto/rand"
	"errors"
	"fmt"
	"math"
	"math/big"
	mrand "math/rand"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/lru"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
)

const (
	headerCacheLimit = 512
	tdCacheLimit     = 1024
	numberCacheLimit = 2048
)

// HeaderChain implements the basic block header chain logic that is shared by
// core.BlockChain and light.LightChain. It is not usable in itself, only as
// a part of either structure.
//
// HeaderChain is responsible for maintaining the header chain including the
// header query and updating.
//
// The components maintained by headerchain includes: (1) total difficulty
// (2) header (3) block hash -> number mapping (4) canonical number -> hash mapping
// and (5) head header flag.
//
// It is not thread safe either, the encapsulating chain structures should do
// the necessary mutex locking/unlocking.
type HeaderChain struct {
	config        *params.ChainConfig
	chainDb       ethdb.Database
	genesisHeader *types.Header

	currentHeader     atomic.Value // Current head of the header chain (may be above the block chain!)
	currentHeaderHash common.Hash  // Hash of the current head of the header chain (prevent recomputing all the time)

	headerCache *lru.Cache[common.Hash, *types.Header]
	tdCache     *lru.Cache[common.Hash, *big.Int] // most recent total difficulties
	numberCache *lru.Cache[common.Hash, uint64]   // most recent block numbers

	procInterrupt func() bool

	rand   *mrand.Rand
	engine consensus.Engine
}

// NewHeaderChain creates a new HeaderChain structure. ProcInterrupt points
// to the parent's interrupt semaphore.
func NewHeaderChain(chainDb ethdb.Database, config *params.ChainConfig, engine consensus.Engine, procInterrupt func() bool) (*HeaderChain, error) {
	// Seed a fast but crypto originating random generator
	seed, err := crand.Int(crand.Reader, big.NewInt(math.MaxInt64))
	if err != nil {
		return nil, err
	}
	hc := &HeaderChain{
		config:        config,
		chainDb:       chainDb,
		headerCache:   lru.NewCache[common.Hash, *types.Header](headerCacheLimit),
		tdCache:       lru.NewCache[common.Hash, *big.Int](tdCacheLimit),
		numberCache:   lru.NewCache[common.Hash, uint64](numberCacheLimit),
		procInterrupt: procInterrupt,
		rand:          mrand.New(mrand.NewSource(seed.Int64())),
		engine:        engine,
	}
	hc.genesisHeader = hc.GetHeaderByNumber(0)
	if hc.genesisHeader == nil {
		return nil, ErrNoGenesis
	}
	hc.currentHeader.Store(hc.genesisHeader)
	if head := rawdb.ReadHeadBlockHash(chainDb); head != (common.Hash{}) {
		if chead := hc.GetHeaderByHash(head); chead != nil {
			hc.currentHeader.Store(chead)
		}
	}
	hc.currentHeaderHash = hc.CurrentHeader().Hash()
	headHeaderGauge.Update(hc.CurrentHeader().Number.Int64())
	return hc, nil
}

// GetBlockNumber retrieves the block number belonging to the given hash
// from the cache or database
func (hc *HeaderChain) GetBlockNumber(hash common.Hash) *uint64 {
	if cached, ok := hc.numberCache.Get(hash); ok {
		return &cached
	}
	number := rawdb.ReadHeaderNumber(hc.chainDb, hash)
	if number != nil {
		hc.numberCache.Add(hash, *number)
	}
	return number
}

type headerWriteResult struct {
	status     WriteStatus
	ignored    int
	imported   int
	lastHash   common.Hash
	lastHeader *types.Header
}

// Reorg reorgs the local canonical chain into the specified chain. The reorg
// can be classified into two cases: (a) extend the local chain (b) switch the
// head to the given header.
func (hc *HeaderChain) Reorg(headers []*types.Header) error {
	// Short circuit if nothing to reorg.
	if len(headers) == 0 {
		return nil
	}
	// If the parent of the (first) block is already the canon header,
	// we don't have to go backwards to delete canon blocks, but simply
	// pile them onto the existing chain. Otherwise, do the necessary
	// reorgs.
	var (
		first = headers[0]
		last  = headers[len(headers)-1]
		batch = hc.chainDb.NewBatch()
	)
	if first.ParentHash != hc.currentHeaderHash {
		// Delete any canonical number assignments above the new head
		for i := last.Number.Uint64() + 1; ; i++ {
			hash := rawdb.ReadCanonicalHash(hc.chainDb, i)
			if hash == (common.Hash{}) {
				break
			}
			rawdb.DeleteCanonicalHash(batch, i)
		}
		// Overwrite any stale canonical number assignments, going
		// backwards from the first header in this import until the
		// cross link between two chains.
		var (
			header     = first
			headNumber = header.Number.Uint64()
			headHash   = header.Hash()
		)
		for rawdb.ReadCanonicalHash(hc.chainDb, headNumber) != headHash {
			rawdb.WriteCanonicalHash(batch, headHash, headNumber)
			if headNumber == 0 {
				break // It shouldn't be reached
			}
			headHash, headNumber = header.ParentHash, header.Number.Uint64()-1
			header = hc.GetHeader(headHash, headNumber)
			if header == nil {
				return fmt.Errorf("missing parent %d %x", headNumber, headHash)
			}
		}
	}
	// Extend the canonical chain with the new headers
	for i := 0; i < len(headers)-1; i++ {
		hash := headers[i+1].ParentHash // Save some extra hashing
		num := headers[i].Number.Uint64()
		rawdb.WriteCanonicalHash(batch, hash, num)
		rawdb.WriteHeadHeaderHash(batch, hash)
	}
	// Write the last header
	hash := headers[len(headers)-1].Hash()
	num := headers[len(headers)-1].Number.Uint64()
	rawdb.WriteCanonicalHash(batch, hash, num)
	rawdb.WriteHeadHeaderHash(batch, hash)

	if err := batch.Write(); err != nil {
		return err
	}
	// Last step update all in-memory head header markers
	hc.currentHeaderHash = last.Hash()
	hc.currentHeader.Store(types.CopyHeader(last))
	headHeaderGauge.Update(last.Number.Int64())
	return nil
}

// WriteHeaders writes a chain of headers into the local chain, given that the
// parents are already known. The chain head header won't be updated in this
// function, the additional SetCanonical is expected in order to finish the entire
// procedure.
func (hc *HeaderChain) WriteHeaders(headers []*types.Header) (int, error) {
	if len(headers) == 0 {
		return 0, nil
	}
	ptd := hc.GetTd(headers[0].ParentHash, headers[0].Number.Uint64()-1)
	if ptd == nil {
		return 0, consensus.ErrUnknownAncestor
	}
	var (
		newTD       = new(big.Int).Set(ptd) // Total difficulty of inserted chain
		inserted    []rawdb.NumberHash      // Ephemeral lookup of number/hash for the chain
		parentKnown = true                  // Set to true to force hc.HasHeader check the first iteration
		batch       = hc.chainDb.NewBatch()
	)
	for i, header := range headers {
		var hash common.Hash
		// The headers have already been validated at this point, so we already
		// know that it's a contiguous chain, where
		// headers[i].Hash() == headers[i+1].ParentHash
		if i < len(headers)-1 {
			hash = headers[i+1].ParentHash
		} else {
			hash = header.Hash()
		}
		number := header.Number.Uint64()
		newTD.Add(newTD, header.Difficulty)

		// If the parent was not present, store it
		// If the header is already known, skip it, otherwise store
		alreadyKnown := parentKnown && hc.HasHeader(hash, number)
		if !alreadyKnown {
			// Irrelevant of the canonical status, write the TD and header to the database.
			rawdb.WriteTd(batch, hash, number, newTD)
			hc.tdCache.Add(hash, new(big.Int).Set(newTD))

			rawdb.WriteHeader(batch, header)
			inserted = append(inserted, rawdb.NumberHash{Number: number, Hash: hash})
			hc.headerCache.Add(hash, header)
			hc.numberCache.Add(hash, number)
		}
		parentKnown = alreadyKnown
	}
	// Skip the slow disk write of all headers if interrupted.
	if hc.procInterrupt() {
		log.Debug("Premature abort during headers import")
		return 0, errors.New("aborted")
	}
	// Commit to disk!
	if err := batch.Write(); err != nil {
		log.Crit("Failed to write headers", "error", err)
	}
	return len(inserted), nil
}

// writeHeadersAndSetHead writes a batch of block headers and applies the last
// header as the chain head if the fork choicer says it's ok to update the chain.
// Note: This method is not concurrent-safe with inserting blocks simultaneously
// into the chain, as side effects caused by reorganisations cannot be emulated
// without the real blocks. Hence, writing headers directly should only be done
// in two scenarios: pure-header mode of operation (light clients), or properly
// separated header/block phases (non-archive clients).
func (hc *HeaderChain) writeHeadersAndSetHead(headers []*types.Header, forker *ForkChoice) (*headerWriteResult, error) {
	inserted, err := hc.WriteHeaders(headers)
	if err != nil {
		return nil, err
	}
	var (
		lastHeader = headers[len(headers)-1]
		lastHash   = headers[len(headers)-1].Hash()
		result     = &headerWriteResult{
			status:     NonStatTy,
			ignored:    len(headers) - inserted,
			imported:   inserted,
			lastHash:   lastHash,
			lastHeader: lastHeader,
		}
	)
	// Ask the fork choicer if the reorg is necessary
	if reorg, err := forker.ReorgNeeded(hc.CurrentHeader(), lastHeader); err != nil {
		return nil, err
	} else if !reorg {
		if inserted != 0 {
			result.status = SideStatTy
		}
		return result, nil
	}
	// Special case, all the inserted headers are already on the canonical
	// header chain, skip the reorg operation.
	if hc.GetCanonicalHash(lastHeader.Number.Uint64()) == lastHash && lastHeader.Number.Uint64() <= hc.CurrentHeader().Number.Uint64() {
		return result, nil
	}
	// Apply the reorg operation
	if err := hc.Reorg(headers); err != nil {
		return nil, err
	}
	result.status = CanonStatTy
	return result, nil
}

func (hc *HeaderChain) ValidateHeaderChain(chain []*types.Header) (int, error) {
	// Do a sanity check that the provided chain is actually ordered and linked
	for i := 1; i < len(chain); i++ {
		if chain[i].Number.Uint64() != chain[i-1].Number.Uint64()+1 {
			hash := chain[i].Hash()
			parentHash := chain[i-1].Hash()
			// Chain broke ancestry, log a message (programming error) and skip insertion
			log.Error("Non contiguous header insert", "number", chain[i].Number, "hash", hash,
				"parent", chain[i].ParentHash, "prevnumber", chain[i-1].Number, "prevhash", parentHash)

			return 0, fmt.Errorf("non contiguous insert: item %d is #%d [%x..], item %d is #%d [%x..] (parent [%x..])", i-1, chain[i-1].Number,
				parentHash.Bytes()[:4], i, chain[i].Number, hash.Bytes()[:4], chain[i].ParentHash[:4])
		}
		// If the header is a banned one, straight out abort
		if BadHashes[chain[i].ParentHash] {
			return i - 1, ErrBannedHash
		}
		// If it's the last header in the cunk, we need to check it too
		if i == len(chain)-1 && BadHashes[chain[i].Hash()] {
			return i, ErrBannedHash
		}
	}
	// Start the parallel verifier
	abort, results := hc.engine.VerifyHeaders(hc, chain)
	defer close(abort)

	// Iterate over the headers and ensure they all check out
	for i := range chain {
		// If the chain is terminating, stop processing blocks
		if hc.procInterrupt() {
			log.Debug("Premature abort during headers verification")
			return 0, errors.New("aborted")
		}
		// Otherwise wait for headers checks and ensure they pass
		if err := <-results; err != nil {
			return i, err
		}
	}

	return 0, nil
}

// InsertHeaderChain inserts the given headers and does the reorganisations.
//
// The validity of the headers is NOT CHECKED by this method, i.e. they need to be
// validated by ValidateHeaderChain before calling InsertHeaderChain.
//
// This insert is all-or-nothing. If this returns an error, no headers were written,
// otherwise they were all processed successfully.
//
// The returned 'write status' says if the inserted headers are part of the canonical chain
// or a side chain.
func (hc *HeaderChain) InsertHeaderChain(chain []*types.Header, start time.Time, forker *ForkChoice) (WriteStatus, error) {
	if hc.procInterrupt() {
		return 0, errors.New("aborted")
	}
	res, err := hc.writeHeadersAndSetHead(chain, forker)
	if err != nil {
		return 0, err
	}
	// Report some public statistics so the user has a clue what's going on
	context := []interface{}{
		"count", res.imported,
		"elapsed", common.PrettyDuration(time.Since(start)),
	}
	if last := res.lastHeader; last != nil {
		context = append(context, "number", last.Number, "hash", res.lastHash)
		if timestamp := time.Unix(int64(last.Time), 0); time.Since(timestamp) > time.Minute {
			context = append(context, []interface{}{"age", common.PrettyAge(timestamp)}...)
		}
	}
	if res.ignored > 0 {
		context = append(context, []interface{}{"ignored", res.ignored}...)
	}
	log.Debug("Imported new block headers", context...)
	return res.status, err
}

// GetAncestor retrieves the Nth ancestor of a given block. It assumes that either the given block or
// a close ancestor of it is canonical. maxNonCanonical points to a downwards counter limiting the
// number of blocks to be individually checked before we reach the canonical chain.
//
// Note: ancestor == 0 returns the same block, 1 returns its parent and so on.
func (hc *HeaderChain) GetAncestor(hash common.Hash, number, ancestor uint64, maxNonCanonical *uint64) (common.Hash, uint64) {
	if ancestor > number {
		return common.Hash{}, 0
	}
	if ancestor == 1 {
		// in this case it is cheaper to just read the header
		if header := hc.GetHeader(hash, number); header != nil {
			return header.ParentHash, number - 1
		}
		return common.Hash{}, 0
	}
	for ancestor != 0 {
		if rawdb.ReadCanonicalHash(hc.chainDb, number) == hash {
			ancestorHash := rawdb.ReadCanonicalHash(hc.chainDb, number-ancestor)
			if rawdb.ReadCanonicalHash(hc.chainDb, number) == hash {
				number -= ancestor
				return ancestorHash, number
			}
		}
		if *maxNonCanonical == 0 {
			return common.Hash{}, 0
		}
		*maxNonCanonical--
		ancestor--
		header := hc.GetHeader(hash, number)
		if header == nil {
			return common.Hash{}, 0
		}
		hash = header.ParentHash
		number--
	}
	return hash, number
}

// GetTd retrieves a block's total difficulty in the canonical chain from the
// database by hash and number, caching it if found.
func (hc *HeaderChain) GetTd(hash common.Hash, number uint64) *big.Int {
	// Short circuit if the td's already in the cache, retrieve otherwise
	if cached, ok := hc.tdCache.Get(hash); ok {
		return cached
	}
	td := rawdb.ReadTd(hc.chainDb, hash, number)
	if td == nil {
		return nil
	}
	// Cache the found body for next time and return
	hc.tdCache.Add(hash, td)
	return td
}

// GetHeader retrieves a block header from the database by hash and number,
// caching it if found.
func (hc *HeaderChain) GetHeader(hash common.Hash, number uint64) *types.Header {
	// Short circuit if the header's already in the cache, retrieve otherwise
	if header, ok := hc.headerCache.Get(hash); ok {
		return header
	}
	header := rawdb.ReadHeader(hc.chainDb, hash, number)
	if header == nil {
		return nil
	}
	// Cache the found header for next time and return
	hc.headerCache.Add(hash, header)
	return header
}

// GetHeaderByHash retrieves a block header from the database by hash, caching it if
// found.
func (hc *HeaderChain) GetHeaderByHash(hash common.Hash) *types.Header {
	number := hc.GetBlockNumber(hash)
	if number == nil {
		return nil
	}
	return hc.GetHeader(hash, *number)
}

// HasHeader checks if a block header is present in the database or not.
// In theory, if header is present in the database, all relative components
// like td and hash->number should be present too.
func (hc *HeaderChain) HasHeader(hash common.Hash, number uint64) bool {
	if hc.numberCache.Contains(hash) || hc.headerCache.Contains(hash) {
		return true
	}
	return rawdb.HasHeader(hc.chainDb, hash, number)
}

// GetHeaderByNumber retrieves a block header from the database by number,
// caching it (associated with its hash) if found.
func (hc *HeaderChain) GetHeaderByNumber(number uint64) *types.Header {
	hash := rawdb.ReadCanonicalHash(hc.chainDb, number)
	if hash == (common.Hash{}) {
		return nil
	}
	return hc.GetHeader(hash, number)
}

// GetHeadersFrom returns a contiguous segment of headers, in rlp-form, going
// backwards from the given number.
// If the 'number' is higher than the highest local header, this method will
// return a best-effort response, containing the headers that we do have.
func (hc *HeaderChain) GetHeadersFrom(number, count uint64) []rlp.RawValue {
	// If the request is for future headers, we still return the portion of
	// headers that we are able to serve
	if current := hc.CurrentHeader().Number.Uint64(); current < number {
		if count > number-current {
			count -= number - current
			number = current
		} else {
			return nil
		}
	}
	var headers []rlp.RawValue
	// If we have some of the headers in cache already, use that before going to db.
	hash := rawdb.ReadCanonicalHash(hc.chainDb, number)
	if hash == (common.Hash{}) {
		return nil
	}
	for count > 0 {
		header, ok := hc.headerCache.Get(hash)
		if !ok {
			break
		}
		rlpData, _ := rlp.EncodeToBytes(header)
		headers = append(headers, rlpData)
		hash = header.ParentHash
		count--
		number--
	}
	// Read remaining from db
	if count > 0 {
		headers = append(headers, rawdb.ReadHeaderRange(hc.chainDb, number, count)...)
	}
	return headers
}

func (hc *HeaderChain) GetCanonicalHash(number uint64) common.Hash {
	return rawdb.ReadCanonicalHash(hc.chainDb, number)
}

// CurrentHeader retrieves the current head header of the canonical chain. The
// header is retrieved from the HeaderChain's internal cache.
func (hc *HeaderChain) CurrentHeader() *types.Header {
	return hc.currentHeader.Load().(*types.Header)
}

// SetCurrentHeader sets the in-memory head header marker of the canonical chan
// as the given header.
func (hc *HeaderChain) SetCurrentHeader(head *types.Header) {
	hc.currentHeader.Store(head)
	hc.currentHeaderHash = head.Hash()
	headHeaderGauge.Update(head.Number.Int64())
}

type (
	// UpdateHeadBlocksCallback is a callback function that is called by SetHead
	// before head header is updated. The method will return the actual block it
	// updated the head to (missing state) and a flag if setHead should continue
	// rewinding till that forcefully (exceeded ancient limits)
	UpdateHeadBlocksCallback func(ethdb.KeyValueWriter, *types.Header) (*types.Header, bool)

	// DeleteBlockContentCallback is a callback function that is called by SetHead
	// before each header is deleted.
	DeleteBlockContentCallback func(ethdb.KeyValueWriter, common.Hash, uint64)
)

// SetHead rewinds the local chain to a new head. Everything above the new head
// will be deleted and the new one set.
func (hc *HeaderChain) SetHead(head uint64, updateFn UpdateHeadBlocksCallback, delFn DeleteBlockContentCallback) {
	hc.setHead(head, 0, updateFn, delFn)
}

// SetHeadWithTimestamp rewinds the local chain to a new head timestamp. Everything
// above the new head will be deleted and the new one set.
func (hc *HeaderChain) SetHeadWithTimestamp(time uint64, updateFn UpdateHeadBlocksCallback, delFn DeleteBlockContentCallback) {
	hc.setHead(0, time, updateFn, delFn)
}

// setHead rewinds the local chain to a new head block or a head timestamp.
// Everything above the new head will be deleted and the new one set.
func (hc *HeaderChain) setHead(headBlock uint64, headTime uint64, updateFn UpdateHeadBlocksCallback, delFn DeleteBlockContentCallback) {
	// Sanity check that there's no attempt to undo the genesis block. This is
	// a fairly synthetic case where someone enables a timestamp based fork
	// below the genesis timestamp. It's nice to not allow that instead of the
	// entire chain getting deleted.
	if headTime > 0 && hc.genesisHeader.Time > headTime {
		// Note, a critical error is quite brutal, but we should really not reach
		// this point. Since pre-timestamp based forks it was impossible to have
		// a fork before block 0, the setHead would always work. With timestamp
		// forks it becomes possible to specify below the genesis. That said, the
		// only time we setHead via timestamp is with chain config changes on the
		// startup, so failing hard there is ok.
		log.Crit("Rejecting genesis rewind via timestamp", "target", headTime, "genesis", hc.genesisHeader.Time)
	}
	var (
		parentHash common.Hash
		batch      = hc.chainDb.NewBatch()
		origin     = true
	)
	done := func(header *types.Header) bool {
		if headTime > 0 {
			return header.Time <= headTime
		}
		return header.Number.Uint64() <= headBlock
	}
	for hdr := hc.CurrentHeader(); hdr != nil && !done(hdr); hdr = hc.CurrentHeader() {
		num := hdr.Number.Uint64()

		// Rewind chain to new head
		parent := hc.GetHeader(hdr.ParentHash, num-1)
		if parent == nil {
			parent = hc.genesisHeader
		}
		parentHash = parent.Hash()

		// Notably, since geth has the possibility for setting the head to a low
		// height which is even lower than ancient head.
		// In order to ensure that the head is always no higher than the data in
		// the database (ancient store or active store), we need to update head
		// first then remove the relative data from the database.
		//
		// Update head first(head fast block, head full block) before deleting the data.
		markerBatch := hc.chainDb.NewBatch()
		if updateFn != nil {
			newHead, force := updateFn(markerBatch, parent)
			if force && ((headTime > 0 && newHead.Time < headTime) || (headTime == 0 && newHead.Number.Uint64() < headBlock)) {
				log.Warn("Force rewinding till ancient limit", "head", newHead.Number.Uint64())
				headBlock, headTime = newHead.Number.Uint64(), 0 // Target timestamp passed, continue rewind in block mode (cleaner)
			}
		}
		// Update head header then.
		rawdb.WriteHeadHeaderHash(markerBatch, parentHash)
		if err := markerBatch.Write(); err != nil {
			log.Crit("Failed to update chain markers", "error", err)
		}
		hc.currentHeader.Store(parent)
		hc.currentHeaderHash = parentHash
		headHeaderGauge.Update(parent.Number.Int64())

		// If this is the first iteration, wipe any leftover data upwards too so
		// we don't end up with dangling daps in the database
		var nums []uint64
		if origin {
			for n := num + 1; len(rawdb.ReadAllHashes(hc.chainDb, n)) > 0; n++ {
				nums = append([]uint64{n}, nums...) // suboptimal, but we don't really expect this path
			}
			origin = false
		}
		nums = append(nums, num)

		// Remove the related data from the database on all sidechains
		for _, num := range nums {
			// Gather all the side fork hashes
			hashes := rawdb.ReadAllHashes(hc.chainDb, num)
			if len(hashes) == 0 {
				// No hashes in the database whatsoever, probably frozen already
				hashes = append(hashes, hdr.Hash())
			}
			for _, hash := range hashes {
				if delFn != nil {
					delFn(batch, hash, num)
				}
				rawdb.DeleteHeader(batch, hash, num)
				rawdb.DeleteTd(batch, hash, num)
			}
			rawdb.DeleteCanonicalHash(batch, num)
		}
	}
	// Flush all accumulated deletions.
	if err := batch.Write(); err != nil {
		log.Crit("Failed to rewind block", "error", err)
	}
	// Clear out any stale content from the caches
	hc.headerCache.Purge()
	hc.tdCache.Purge()
	hc.numberCache.Purge()
}

// SetGenesis sets a new genesis block header for the chain
func (hc *HeaderChain) SetGenesis(head *types.Header) {
	hc.genesisHeader = head
}

// Config retrieves the header chain's chain configuration.
func (hc *HeaderChain) Config() *params.ChainConfig { return hc.config }

// Engine retrieves the header chain's consensus engine.
func (hc *HeaderChain) Engine() consensus.Engine { return hc.engine }

// GetBlock implements consensus.ChainReader, and returns nil for every input as
// a header chain does not have blocks available for retrieval.
func (hc *HeaderChain) GetBlock(hash common.Hash, number uint64) *types.Block {
	return nil
}
