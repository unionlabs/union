package blockchain

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
)

// This saves a beacon block to the initial sync blocks cache. It rate limits how many blocks
// the cache keeps in memory (2 epochs worth of blocks) and saves them to DB when it hits this limit.
func (s *Service) saveInitSyncBlock(ctx context.Context, r [32]byte, b interfaces.ReadOnlySignedBeaconBlock) error {
	s.initSyncBlocksLock.Lock()
	s.initSyncBlocks[r] = b
	numBlocks := len(s.initSyncBlocks)
	s.initSyncBlocksLock.Unlock()
	if uint64(numBlocks) > initialSyncBlockCacheSize {
		if err := s.cfg.BeaconDB.SaveBlocks(ctx, s.getInitSyncBlocks()); err != nil {
			return err
		}
		s.clearInitSyncBlocks()
	}
	return nil
}

// This checks if a beacon block exists in the initial sync blocks cache using the root
// of the block.
func (s *Service) hasInitSyncBlock(r [32]byte) bool {
	s.initSyncBlocksLock.RLock()
	defer s.initSyncBlocksLock.RUnlock()
	_, ok := s.initSyncBlocks[r]
	return ok
}

// Returns true if a block for root `r` exists in the initial sync blocks cache or the DB.
func (s *Service) hasBlockInInitSyncOrDB(ctx context.Context, r [32]byte) bool {
	if s.hasInitSyncBlock(r) {
		return true
	}
	return s.cfg.BeaconDB.HasBlock(ctx, r)
}

// Returns block for a given root `r` from either the initial sync blocks cache or the DB.
// Error is returned if the block is not found in either cache or DB.
func (s *Service) getBlock(ctx context.Context, r [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
	s.initSyncBlocksLock.RLock()

	// Check cache first because it's faster.
	b, ok := s.initSyncBlocks[r]
	s.initSyncBlocksLock.RUnlock()
	var err error
	if !ok {
		b, err = s.cfg.BeaconDB.Block(ctx, r)
		if err != nil {
			return nil, errors.Wrap(err, "could not retrieve block from db")
		}
	}
	if err := blocks.BeaconBlockIsNil(b); err != nil {
		return nil, errBlockNotFoundInCacheOrDB
	}
	return b, nil
}

// This retrieves all the beacon blocks from the initial sync blocks cache, the returned
// blocks are unordered.
func (s *Service) getInitSyncBlocks() []interfaces.ReadOnlySignedBeaconBlock {
	s.initSyncBlocksLock.RLock()
	defer s.initSyncBlocksLock.RUnlock()

	blks := make([]interfaces.ReadOnlySignedBeaconBlock, 0, len(s.initSyncBlocks))
	for _, b := range s.initSyncBlocks {
		blks = append(blks, b)
	}
	return blks
}

// This clears out the initial sync blocks cache.
func (s *Service) clearInitSyncBlocks() {
	s.initSyncBlocksLock.Lock()
	defer s.initSyncBlocksLock.Unlock()
	s.initSyncBlocks = make(map[[32]byte]interfaces.ReadOnlySignedBeaconBlock)
}
