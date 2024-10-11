package store

import (
	"errors"
	"fmt"
	"strconv"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/go-kit/kit/metrics"
	lru "github.com/hashicorp/golang-lru/v2"

	dbm "github.com/cometbft/cometbft-db"
	cmtstore "github.com/cometbft/cometbft/api/cometbft/store/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/internal/evidence"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
	cmterrors "github.com/cometbft/cometbft/types/errors"
)

// Assuming the length of a block part is 64kB (`types.BlockPartSizeBytes`),
// the maximum size of a block, that will be batch saved, is 640kB. The
// benchmarks have shown that `goleveldb` still performs well with blocks of
// this size. However, if the block is larger than 1MB, the performance degrades.
const maxBlockPartsToBatch = 10

/*
BlockStore is a simple low level store for blocks.

There are three types of information stored:
  - BlockMeta:   Meta information about each block
  - Block part:  Parts of each block, aggregated w/ PartSet
  - Commit:      The commit part of each block, for gossiping precommit votes

Currently the precommit signatures are duplicated in the Block parts as
well as the Commit.  In the future this may change, perhaps by moving
the Commit data outside the Block. (TODO)

The store can be assumed to contain all contiguous blocks between base and height (inclusive).

// NOTE: BlockStore methods will panic if they encounter errors
// deserializing loaded data, indicating probable corruption on disk.
*/
type BlockStore struct {
	db      dbm.DB
	metrics *Metrics

	// mtx guards access to the struct fields listed below it. Although we rely on the database
	// to enforce fine-grained concurrency control for its data, we need to make sure that
	// no external observer can get data from the database that is not in sync with the fields below,
	// and vice-versa. Hence, when updating the fields below, we use the mutex to make sure
	// that the database is also up to date. This prevents any concurrent external access from
	// obtaining inconsistent data.
	// The only reason for keeping these fields in the struct is that the data
	// can't efficiently be queried from the database since the key encoding we use is not
	// lexicographically ordered (see https://github.com/tendermint/tendermint/issues/4567).
	mtx    cmtsync.RWMutex
	base   int64
	height int64

	dbKeyLayout BlockKeyLayout

	blocksDeleted      int64
	compact            bool
	compactionInterval int64

	seenCommitCache          *lru.Cache[int64, *types.Commit]
	blockCommitCache         *lru.Cache[int64, *types.Commit]
	blockExtendedCommitCache *lru.Cache[int64, *types.ExtendedCommit]
	blockPartCache           *lru.Cache[blockPartIndex, *types.Part]
}

type BlockStoreOption func(*BlockStore)

// WithCompaction sets the compaciton parameters.
func WithCompaction(compact bool, compactionInterval int64) BlockStoreOption {
	return func(bs *BlockStore) {
		bs.compact = compact
		bs.compactionInterval = compactionInterval
	}
}

// WithMetrics sets the metrics.
func WithMetrics(metrics *Metrics) BlockStoreOption {
	return func(bs *BlockStore) { bs.metrics = metrics }
}

// WithDBKeyLayout the metrics.
func WithDBKeyLayout(dbKeyLayout string) BlockStoreOption {
	return func(bs *BlockStore) { setDBLayout(bs, dbKeyLayout) }
}

func setDBLayout(bStore *BlockStore, dbKeyLayoutVersion string) {
	if !bStore.IsEmpty() {
		var version []byte
		var err error
		if version, err = bStore.db.Get([]byte("version")); err != nil {
			// WARN: This is because currently cometBFT DB does not return an error if the key does not exist
			// If this behavior changes we need to account for that.
			panic(err)
		}
		if len(version) != 0 {
			dbKeyLayoutVersion = string(version)
		}
	}
	switch dbKeyLayoutVersion {
	case "v1", "":
		bStore.dbKeyLayout = &v1LegacyLayout{}
		dbKeyLayoutVersion = "v1"
	case "v2":
		bStore.dbKeyLayout = &v2Layout{}
	default:
		panic("unknown key layout version")
	}
	if err := bStore.db.SetSync([]byte("version"), []byte(dbKeyLayoutVersion)); err != nil {
		panic(err)
	}
}

// NewBlockStore returns a new BlockStore with the given DB,
// initialized to the last height that was committed to the DB.
func NewBlockStore(db dbm.DB, options ...BlockStoreOption) *BlockStore {
	start := time.Now()

	bs := LoadBlockStoreState(db)

	bStore := &BlockStore{
		base:    bs.Base,
		height:  bs.Height,
		db:      db,
		metrics: NopMetrics(),
	}
	bStore.addCaches()

	for _, option := range options {
		option(bStore)
	}

	if bStore.dbKeyLayout == nil {
		setDBLayout(bStore, "v1")
	}

	addTimeSample(bStore.metrics.BlockStoreAccessDurationSeconds.With("method", "new_block_store"), start)()
	return bStore
}

func (bs *BlockStore) addCaches() {
	var err error
	// err can only occur if the argument is non-positive, so is impossible in context.
	bs.blockCommitCache, err = lru.New[int64, *types.Commit](100)
	if err != nil {
		panic(err)
	}
	bs.blockExtendedCommitCache, err = lru.New[int64, *types.ExtendedCommit](100)
	if err != nil {
		panic(err)
	}
	bs.seenCommitCache, err = lru.New[int64, *types.Commit](100)
	if err != nil {
		panic(err)
	}
	bs.blockPartCache, err = lru.New[blockPartIndex, *types.Part](500)
	if err != nil {
		panic(err)
	}
}

func (bs *BlockStore) GetVersion() string {
	switch bs.dbKeyLayout.(type) {
	case *v1LegacyLayout:
		return "v1"
	case *v2Layout:
		return "v2"
	}
	return "no version set"
}

func (bs *BlockStore) IsEmpty() bool {
	bs.mtx.RLock()
	defer bs.mtx.RUnlock()
	return bs.base == 0 && bs.height == 0
}

// Base returns the first known contiguous block height, or 0 for empty block stores.
func (bs *BlockStore) Base() int64 {
	bs.mtx.RLock()
	defer bs.mtx.RUnlock()
	return bs.base
}

// Height returns the last known contiguous block height, or 0 for empty block stores.
func (bs *BlockStore) Height() int64 {
	bs.mtx.RLock()
	defer bs.mtx.RUnlock()
	return bs.height
}

// Size returns the number of blocks in the block store.
func (bs *BlockStore) Size() int64 {
	bs.mtx.RLock()
	defer bs.mtx.RUnlock()
	if bs.height == 0 {
		return 0
	}
	return bs.height - bs.base + 1
}

// LoadBase atomically loads the base block meta, or returns nil if no base is found.
func (bs *BlockStore) LoadBaseMeta() *types.BlockMeta {
	bs.mtx.RLock()
	defer bs.mtx.RUnlock()
	if bs.base == 0 {
		return nil
	}
	return bs.LoadBlockMeta(bs.base)
}

// LoadBlock returns the block with the given height.
// If no block is found for that height, it returns nil.
func (bs *BlockStore) LoadBlock(height int64) (*types.Block, *types.BlockMeta) {
	start := time.Now()
	blockMeta := bs.LoadBlockMeta(height)
	if blockMeta == nil {
		return nil, nil
	}
	pbb := new(cmtproto.Block)
	buf := []byte{}
	for i := 0; i < int(blockMeta.BlockID.PartSetHeader.Total); i++ {
		part := bs.LoadBlockPart(height, i)
		// If the part is missing (e.g. since it has been deleted after we
		// loaded the block meta) we consider the whole block to be missing.
		if part == nil {
			return nil, nil
		}
		buf = append(buf, part.Bytes...)
	}
	addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block"), start)()

	err := proto.Unmarshal(buf, pbb)
	if err != nil {
		// NOTE: The existence of meta should imply the existence of the
		// block. So, make sure meta is only saved after blocks are saved.
		panic(fmt.Sprintf("Error reading block: %v", err))
	}
	block, err := types.BlockFromProto(pbb)
	if err != nil {
		panic(cmterrors.ErrMsgFromProto{MessageName: "Block", Err: err})
	}

	return block, blockMeta
}

// LoadBlockByHash returns the block with the given hash.
// If no block is found for that hash, it returns nil.
// Panics if it fails to parse height associated with the given hash.
func (bs *BlockStore) LoadBlockByHash(hash []byte) (*types.Block, *types.BlockMeta) {
	// WARN this function includes the time for LoadBlock and will count the time it takes to load the entire block, block parts
	// AND unmarshall
	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block_by_hash"), time.Now())()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcBlockHashKey(hash))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		return nil, nil
	}

	s := string(bz)
	height, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		panic(fmt.Sprintf("failed to extract height from %s: %v", s, err))
	}
	return bs.LoadBlock(height)
}

type blockPartIndex struct {
	height int64
	index  int
}

// LoadBlockPart returns the Part at the given index
// from the block at the given height.
// If no part is found for the given height and index, it returns nil.
// The returned part should not be modified by the caller. Take a copy if you need to modify it.
func (bs *BlockStore) LoadBlockPart(height int64, index int) *types.Part {
	part, ok := bs.blockPartCache.Get(blockPartIndex{height, index})
	if ok {
		return part
	}
	pbpart := new(cmtproto.Part)
	start := time.Now()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcBlockPartKey(height, index))
	if err != nil {
		panic(err)
	}

	addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block_part"), start)()

	if len(bz) == 0 {
		return nil
	}
	err = proto.Unmarshal(bz, pbpart)
	if err != nil {
		panic(fmt.Errorf("unmarshal to cmtproto.Part failed: %w", err))
	}
	part, err = types.PartFromProto(pbpart)
	if err != nil {
		panic(fmt.Sprintf("Error reading block part: %v", err))
	}
	bs.blockPartCache.Add(blockPartIndex{height, index}, part)
	return part
}

// LoadBlockMeta returns the BlockMeta for the given height.
// If no block is found for the given height, it returns nil.
func (bs *BlockStore) LoadBlockMeta(height int64) *types.BlockMeta {
	pbbm := new(cmtproto.BlockMeta)
	start := time.Now()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcBlockMetaKey(height))
	if err != nil {
		panic(err)
	}

	addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block_meta"), start)()

	if len(bz) == 0 {
		return nil
	}

	err = proto.Unmarshal(bz, pbbm)
	if err != nil {
		panic(fmt.Errorf("unmarshal to cmtproto.BlockMeta: %w", err))
	}
	blockMeta, err := types.BlockMetaFromTrustedProto(pbbm)
	if err != nil {
		panic(cmterrors.ErrMsgFromProto{MessageName: "BlockMetadata", Err: err})
	}

	return blockMeta
}

// LoadBlockMetaByHash returns the blockmeta who's header corresponds to the given
// hash. If none is found, returns nil.
func (bs *BlockStore) LoadBlockMetaByHash(hash []byte) *types.BlockMeta {
	// WARN Same as for block by hash, this includes the time to get the block metadata and unmarshall it
	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block_meta_by_hash"), time.Now())()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcBlockHashKey(hash))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		return nil
	}

	s := string(bz)
	height, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		panic(fmt.Sprintf("failed to extract height from %s: %v", s, err))
	}
	return bs.LoadBlockMeta(height)
}

// LoadBlockCommit returns the Commit for the given height.
// This commit consists of the +2/3 and other Precommit-votes for block at `height`,
// and it comes from the block.LastCommit for `height+1`.
// If no commit is found for the given height, it returns nil.
//
// This return value should not be modified. If you need to modify it,
// do bs.LoadBlockCommit(height).Clone().
func (bs *BlockStore) LoadBlockCommit(height int64) *types.Commit {
	comm, ok := bs.blockCommitCache.Get(height)
	if ok {
		return comm
	}
	pbc := new(cmtproto.Commit)

	start := time.Now()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcBlockCommitKey(height))
	if err != nil {
		panic(err)
	}

	addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block_commit"), start)()

	if len(bz) == 0 {
		return nil
	}

	err = proto.Unmarshal(bz, pbc)
	if err != nil {
		panic(fmt.Errorf("error reading block commit: %w", err))
	}
	commit, err := types.CommitFromProto(pbc)
	if err != nil {
		panic(cmterrors.ErrMsgToProto{MessageName: "Commit", Err: err})
	}
	bs.blockCommitCache.Add(height, commit)
	return commit
}

// LoadExtendedCommit returns the ExtendedCommit for the given height.
// The extended commit is not guaranteed to contain the same +2/3 precommits data
// as the commit in the block.
func (bs *BlockStore) LoadBlockExtendedCommit(height int64) *types.ExtendedCommit {
	comm, ok := bs.blockExtendedCommitCache.Get(height)
	if ok {
		return comm.Clone()
	}
	pbec := new(cmtproto.ExtendedCommit)

	start := time.Now()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcExtCommitKey(height))
	if err != nil {
		panic(fmt.Errorf("fetching extended commit: %w", err))
	}

	addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_block_ext_commit"), start)()

	if len(bz) == 0 {
		return nil
	}

	err = proto.Unmarshal(bz, pbec)
	if err != nil {
		panic(fmt.Errorf("decoding extended commit: %w", err))
	}
	extCommit, err := types.ExtendedCommitFromProto(pbec)
	if err != nil {
		panic(fmt.Errorf("converting extended commit: %w", err))
	}
	bs.blockExtendedCommitCache.Add(height, extCommit)
	return extCommit.Clone()
}

// LoadSeenCommit returns the locally seen Commit for the given height.
// This is useful when we've seen a commit, but there has not yet been
// a new block at `height + 1` that includes this commit in its block.LastCommit.
func (bs *BlockStore) LoadSeenCommit(height int64) *types.Commit {
	comm, ok := bs.seenCommitCache.Get(height)
	if ok {
		return comm.Clone()
	}
	pbc := new(cmtproto.Commit)
	start := time.Now()
	bz, err := bs.db.Get(bs.dbKeyLayout.CalcSeenCommitKey(height))
	if err != nil {
		panic(err)
	}

	addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "load_seen_commit"), start)()

	if len(bz) == 0 {
		return nil
	}

	err = proto.Unmarshal(bz, pbc)
	if err != nil {
		panic(fmt.Sprintf("error reading block seen commit: %v", err))
	}

	commit, err := types.CommitFromProto(pbc)
	if err != nil {
		panic(fmt.Errorf("converting seen commit: %w", err))
	}
	bs.seenCommitCache.Add(height, commit)
	return commit.Clone()
}

// PruneBlocks removes block up to (but not including) a height. It returns the
// number of blocks pruned and the evidence retain height - the height at which
// data needed to prove evidence must not be removed.
func (bs *BlockStore) PruneBlocks(height int64, state sm.State) (uint64, int64, error) {
	if height <= 0 {
		return 0, -1, errors.New("height must be greater than 0")
	}
	bs.mtx.RLock()
	if height > bs.height {
		bs.mtx.RUnlock()
		return 0, -1, fmt.Errorf("cannot prune beyond the latest height %v", bs.height)
	}
	base := bs.base
	bs.mtx.RUnlock()
	if height < base {
		return 0, -1, fmt.Errorf("cannot prune to height %v, it is lower than base height %v",
			height, base)
	}

	pruned := uint64(0)
	batch := bs.db.NewBatch()
	defer batch.Close()
	flush := func(batch dbm.Batch, base int64) error {
		// We can't trust batches to be atomic, so update base first to make sure no one
		// tries to access missing blocks.
		bs.mtx.Lock()
		defer batch.Close()
		defer bs.mtx.Unlock()
		bs.base = base
		return bs.saveStateAndWriteDB(batch, "failed to prune")
	}

	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "prune_blocks"), time.Now())()

	evidencePoint := height
	for h := base; h < height; h++ {
		meta := bs.LoadBlockMeta(h)
		if meta == nil { // assume already deleted
			continue
		}

		// This logic is in place to protect data that proves malicious behavior.
		// If the height is within the evidence age, we continue to persist the header and commit data.

		if evidencePoint == height && !evidence.IsEvidenceExpired(state.LastBlockHeight, state.LastBlockTime, h, meta.Header.Time, state.ConsensusParams.Evidence) {
			evidencePoint = h
		}

		// if height is beyond the evidence point we dont delete the header
		if h < evidencePoint {
			if err := batch.Delete(bs.dbKeyLayout.CalcBlockMetaKey(h)); err != nil {
				return 0, -1, err
			}
		}
		if err := batch.Delete(bs.dbKeyLayout.CalcBlockHashKey(meta.BlockID.Hash)); err != nil {
			return 0, -1, err
		}
		// if height is beyond the evidence point we dont delete the commit data
		if h < evidencePoint {
			if err := batch.Delete(bs.dbKeyLayout.CalcBlockCommitKey(h)); err != nil {
				return 0, -1, err
			}
			bs.blockCommitCache.Remove(h)
		}
		if err := batch.Delete(bs.dbKeyLayout.CalcSeenCommitKey(h)); err != nil {
			return 0, -1, err
		}
		bs.seenCommitCache.Remove(h)
		for p := 0; p < int(meta.BlockID.PartSetHeader.Total); p++ {
			if err := batch.Delete(bs.dbKeyLayout.CalcBlockPartKey(h, p)); err != nil {
				return 0, -1, err
			}
			bs.blockPartCache.Remove(blockPartIndex{h, p})
		}
		pruned++

		// flush every 1000 blocks to avoid batches becoming too large
		if pruned%1000 == 0 && pruned > 0 {
			err := flush(batch, h)
			if err != nil {
				return 0, -1, err
			}
			batch = bs.db.NewBatch()
			defer batch.Close()
		}
	}

	err := flush(batch, height)
	if err != nil {
		return 0, -1, err
	}
	bs.blocksDeleted += int64(pruned)

	if bs.compact && bs.blocksDeleted >= bs.compactionInterval {
		// When the range is nil,nil, the database will try to compact
		// ALL levels. Another option is to set a predefined range of
		// specific keys.
		err = bs.db.Compact(nil, nil)
		if err == nil {
			// If there was no error in compaction we reset the counter.
			// Otherwise we preserve the number of blocks deleted so
			// we can trigger compaction in the next pruning iteration
			bs.blocksDeleted = 0
		}
	}
	return pruned, evidencePoint, err
}

// SaveBlock persists the given block, blockParts, and seenCommit to the underlying db.
// blockParts: Must be parts of the block
// seenCommit: The +2/3 precommits that were seen which committed at height.
//
//	If all the nodes restart after committing a block,
//	we need this to reload the precommits to catch-up nodes to the
//	most recent height.  Otherwise they'd stall at H-1.
func (bs *BlockStore) SaveBlock(block *types.Block, blockParts *types.PartSet, seenCommit *types.Commit) {
	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "save_block"), time.Now())()
	if block == nil {
		panic("BlockStore can only save a non-nil block")
	}

	batch := bs.db.NewBatch()
	defer batch.Close()

	if err := bs.saveBlockToBatch(block, blockParts, seenCommit, batch); err != nil {
		panic(err)
	}

	bs.mtx.Lock()
	defer bs.mtx.Unlock()
	bs.height = block.Height
	if bs.base == 0 {
		bs.base = block.Height
	}

	// Save new BlockStoreState descriptor. This also flushes the database.
	err := bs.saveStateAndWriteDB(batch, "failed to save block")
	if err != nil {
		panic(err)
	}
}

// SaveBlockWithExtendedCommit persists the given block, blockParts, and
// seenExtendedCommit to the underlying db. seenExtendedCommit is stored under
// two keys in the database: as the seenCommit and as the ExtendedCommit data for the
// height. This allows the vote extension data to be persisted for all blocks
// that are saved.
func (bs *BlockStore) SaveBlockWithExtendedCommit(block *types.Block, blockParts *types.PartSet, seenExtendedCommit *types.ExtendedCommit) {
	// WARN includes marshaling the blockstore state
	start := time.Now()

	if block == nil {
		panic("BlockStore can only save a non-nil block")
	}
	if err := seenExtendedCommit.EnsureExtensions(true); err != nil {
		panic(fmt.Errorf("problems saving block with extensions: %w", err))
	}

	batch := bs.db.NewBatch()
	defer batch.Close()

	if err := bs.saveBlockToBatch(block, blockParts, seenExtendedCommit.ToCommit(), batch); err != nil {
		panic(err)
	}
	height := block.Height

	marshallingTime := time.Now()

	pbec := seenExtendedCommit.ToProto()

	extCommitBytes := mustEncode(pbec)

	extCommitMarshallTDiff := time.Since(marshallingTime).Seconds()

	if err := batch.Set(bs.dbKeyLayout.CalcExtCommitKey(height), extCommitBytes); err != nil {
		panic(err)
	}

	bs.mtx.Lock()
	defer bs.mtx.Unlock()
	bs.height = height
	if bs.base == 0 {
		bs.base = height
	}

	// Save new BlockStoreState descriptor. This also flushes the database.
	err := bs.saveStateAndWriteDB(batch, "failed to save block with extended commit")
	if err != nil {
		panic(err)
	}

	bs.metrics.BlockStoreAccessDurationSeconds.With("method", "save_block_ext_commit").Observe(time.Since(start).Seconds() - extCommitMarshallTDiff)
}

func (bs *BlockStore) saveBlockToBatch(
	block *types.Block,
	blockParts *types.PartSet,
	seenCommit *types.Commit,
	batch dbm.Batch,
) error {
	if block == nil {
		panic("BlockStore can only save a non-nil block")
	}

	height := block.Height
	hash := block.Hash()

	if g, w := height, bs.Height()+1; bs.Base() > 0 && g != w {
		return fmt.Errorf("BlockStore can only save contiguous blocks. Wanted %v, got %v", w, g)
	}
	if !blockParts.IsComplete() {
		return errors.New("BlockStore can only save complete block part sets")
	}
	if height != seenCommit.Height {
		return fmt.Errorf("BlockStore cannot save seen commit of a different height (block: %d, commit: %d)", height, seenCommit.Height)
	}

	// If the block is small, batch save the block parts. Otherwise, save the
	// parts individually.
	saveBlockPartsToBatch := blockParts.Count() <= maxBlockPartsToBatch

	start := time.Now()

	// Save block parts. This must be done before the block meta, since callers
	// typically load the block meta first as an indication that the block exists
	// and then go on to load block parts - we must make sure the block is
	// complete as soon as the block meta is written.
	for i := 0; i < int(blockParts.Total()); i++ {
		part := blockParts.GetPart(i)
		bs.saveBlockPart(height, i, part, batch, saveBlockPartsToBatch)
		bs.blockPartCache.Add(blockPartIndex{height, i}, part)
	}

	marshallTime := time.Now()
	// Save block meta
	blockMeta := types.NewBlockMeta(block, blockParts)
	pbm := blockMeta.ToProto()
	if pbm == nil {
		return errors.New("nil blockmeta")
	}
	metaBytes := mustEncode(pbm)
	blockMetaMarshallDiff := time.Since(marshallTime).Seconds()

	if err := batch.Set(bs.dbKeyLayout.CalcBlockMetaKey(height), metaBytes); err != nil {
		return err
	}
	if err := batch.Set(bs.dbKeyLayout.CalcBlockHashKey(hash), []byte(strconv.FormatInt(height, 10))); err != nil {
		return err
	}

	marshallTime = time.Now()
	// Save block commit (duplicate and separate from the Block)
	pbc := block.LastCommit.ToProto()
	blockCommitBytes := mustEncode(pbc)

	blockMetaMarshallDiff += time.Since(marshallTime).Seconds()

	if err := batch.Set(bs.dbKeyLayout.CalcBlockCommitKey(height-1), blockCommitBytes); err != nil {
		return err
	}

	marshallTime = time.Now()

	// Save seen commit (seen +2/3 precommits for block)
	// NOTE: we can delete this at a later height
	pbsc := seenCommit.ToProto()
	seenCommitBytes := mustEncode(pbsc)

	blockMetaMarshallDiff += time.Since(marshallTime).Seconds()
	if err := batch.Set(bs.dbKeyLayout.CalcSeenCommitKey(height), seenCommitBytes); err != nil {
		return err
	}

	bs.metrics.BlockStoreAccessDurationSeconds.With("method", "save_block_to_batch").Observe(time.Since(start).Seconds() - blockMetaMarshallDiff)
	return nil
}

func (bs *BlockStore) saveBlockPart(height int64, index int, part *types.Part, batch dbm.Batch, saveBlockPartsToBatch bool) {
	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "save_block_part"), time.Now())()
	pbp, err := part.ToProto()
	if err != nil {
		panic(cmterrors.ErrMsgToProto{MessageName: "Part", Err: err})
	}

	partBytes := mustEncode(pbp)

	if saveBlockPartsToBatch {
		err = batch.Set(bs.dbKeyLayout.CalcBlockPartKey(height, index), partBytes)
	} else {
		err = bs.db.Set(bs.dbKeyLayout.CalcBlockPartKey(height, index), partBytes)
	}
	if err != nil {
		panic(err)
	}
}

// Contract: the caller MUST have, at least, a read lock on `bs`.
func (bs *BlockStore) saveStateAndWriteDB(batch dbm.Batch, errMsg string) error {
	bss := cmtstore.BlockStoreState{
		Base:   bs.base,
		Height: bs.height,
	}
	start := time.Now()

	SaveBlockStoreState(&bss, batch)

	err := batch.WriteSync()
	if err != nil {
		return fmt.Errorf("error writing batch to DB %q: (base %d, height %d): %w",
			errMsg, bs.base, bs.height, err)
	}
	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "save_bs_state"), start)()

	return nil
}

// SaveSeenCommit saves a seen commit, used by e.g. the state sync reactor when bootstrapping node.
func (bs *BlockStore) SaveSeenCommit(height int64, seenCommit *types.Commit) error {
	pbc := seenCommit.ToProto()
	seenCommitBytes, err := proto.Marshal(pbc)

	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "save_seen_commit"), time.Now())()

	if err != nil {
		return fmt.Errorf("unable to marshal commit: %w", err)
	}
	return bs.db.Set(bs.dbKeyLayout.CalcSeenCommitKey(height), seenCommitBytes)
}

func (bs *BlockStore) Close() error {
	return bs.db.Close()
}

// -----------------------------------------------------------------------------

var blockStoreKey = []byte("blockStore")

// SaveBlockStoreState persists the blockStore state to the database.
func SaveBlockStoreState(bsj *cmtstore.BlockStoreState, batch dbm.Batch) {
	bytes, err := proto.Marshal(bsj)
	if err != nil {
		panic(fmt.Sprintf("Could not marshal state bytes: %v", err))
	}
	if err := batch.Set(blockStoreKey, bytes); err != nil {
		panic(err)
	}
}

// LoadBlockStoreState returns the BlockStoreState as loaded from disk.
// If no BlockStoreState was previously persisted, it returns the zero value.
func LoadBlockStoreState(db dbm.DB) cmtstore.BlockStoreState {
	bytes, err := db.Get(blockStoreKey)
	if err != nil {
		panic(err)
	}

	if len(bytes) == 0 {
		return cmtstore.BlockStoreState{
			Base:   0,
			Height: 0,
		}
	}

	var bsj cmtstore.BlockStoreState
	if err := proto.Unmarshal(bytes, &bsj); err != nil {
		panic(fmt.Sprintf("Could not unmarshal bytes: %X", bytes))
	}

	// Backwards compatibility with persisted data from before Base existed.
	if bsj.Height > 0 && bsj.Base == 0 {
		bsj.Base = 1
	}
	return bsj
}

// mustEncode proto encodes a proto.message and panics if fails.
func mustEncode(pb proto.Message) []byte {
	bz, err := proto.Marshal(pb)
	if err != nil {
		panic(fmt.Errorf("unable to marshal: %w", err))
	}
	return bz
}

// -----------------------------------------------------------------------------

// DeleteLatestBlock removes the block pointed to by height,
// lowering height by one.
func (bs *BlockStore) DeleteLatestBlock() error {
	defer addTimeSample(bs.metrics.BlockStoreAccessDurationSeconds.With("method", "delete_latest_block"), time.Now())()

	bs.mtx.RLock()
	targetHeight := bs.height
	bs.mtx.RUnlock()

	batch := bs.db.NewBatch()
	defer batch.Close()

	// delete what we can, skipping what's already missing, to ensure partial
	// blocks get deleted fully.
	if meta := bs.LoadBlockMeta(targetHeight); meta != nil {
		if err := batch.Delete(bs.dbKeyLayout.CalcBlockHashKey(meta.BlockID.Hash)); err != nil {
			return err
		}
		for p := 0; p < int(meta.BlockID.PartSetHeader.Total); p++ {
			if err := batch.Delete(bs.dbKeyLayout.CalcBlockPartKey(targetHeight, p)); err != nil {
				return err
			}
		}
	}
	if err := batch.Delete(bs.dbKeyLayout.CalcBlockCommitKey(targetHeight)); err != nil {
		return err
	}
	if err := batch.Delete(bs.dbKeyLayout.CalcSeenCommitKey(targetHeight)); err != nil {
		return err
	}
	// delete last, so as to not leave keys built on meta.BlockID dangling
	if err := batch.Delete(bs.dbKeyLayout.CalcBlockMetaKey(targetHeight)); err != nil {
		return err
	}

	bs.mtx.Lock()
	defer bs.mtx.Unlock()
	bs.height = targetHeight - 1
	return bs.saveStateAndWriteDB(batch, "failed to delete the latest block")
}

// addTimeSample returns a function that, when called, adds an observation to m.
// The observation added to m is the number of seconds elapsed since addTimeSample
// was initially called. addTimeSample is meant to be called in a defer to calculate
// the amount of time a function takes to complete.
func addTimeSample(h metrics.Histogram, start time.Time) func() {
	return func() {
		h.Observe(time.Since(start).Seconds())
	}
}
