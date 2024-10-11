package sync

import (
	"context"
	"encoding/hex"
	"fmt"
	"sort"
	"sync"
	"time"

	"github.com/libp2p/go-libp2p/core"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz/equality"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"github.com/trailofbits/go-mutexasserts"
	"go.opencensus.io/trace"
)

var processPendingBlocksPeriod = slots.DivideSlotBy(3 /* times per slot */)

const maxPeerRequest = 50
const numOfTries = 5
const maxBlocksPerSlot = 3

// processes pending blocks queue on every processPendingBlocksPeriod
func (s *Service) processPendingBlocksQueue() {
	// Prevents multiple queue processing goroutines (invoked by RunEvery) from contending for data.
	locker := new(sync.Mutex)
	async.RunEvery(s.ctx, processPendingBlocksPeriod, func() {
		// Don't process the pending blocks if genesis time has not been set. The chain is not ready.
		if !s.chainIsStarted() {
			return
		}
		locker.Lock()
		if err := s.processPendingBlocks(s.ctx); err != nil {
			log.WithError(err).Debug("Could not process pending blocks")
		}
		locker.Unlock()
	})
}

// processPendingBlocks validates, processes, and broadcasts pending blocks.
func (s *Service) processPendingBlocks(ctx context.Context) error {
	ctx, span := trace.StartSpan(ctx, "processPendingBlocks")
	defer span.End()

	// Remove old blocks from our expiration cache.
	s.deleteExpiredBlocksFromCache()

	// Validate pending slots before processing.
	if err := s.validatePendingSlots(); err != nil {
		return errors.Wrap(err, "could not validate pending slots")
	}

	// Sort slots for ordered processing.
	sortedSlots := s.sortedPendingSlots()

	span.AddAttributes(trace.Int64Attribute("numSlots", int64(len(sortedSlots))), trace.Int64Attribute("numPeers", int64(len(s.cfg.p2p.Peers().Connected()))))

	randGen := rand.NewGenerator()
	var parentRoots [][32]byte

	// Iterate through sorted slots.
	for _, slot := range sortedSlots {
		// Skip processing if slot is in the future.
		if slot > s.cfg.clock.CurrentSlot() {
			continue
		}

		ctx, span := startInnerSpan(ctx, slot)

		// Get blocks in the pending queue for the current slot.
		blocksInCache := s.getBlocksInQueue(slot)
		if len(blocksInCache) == 0 {
			span.End()
			continue
		}

		// Process each block in the queue.
		for _, b := range blocksInCache {
			if err := blocks.BeaconBlockIsNil(b); err != nil {
				continue
			}
			blkRoot, err := b.Block().HashTreeRoot()
			if err != nil {
				return err
			}

			// Skip blocks that are already being processed.
			if s.cfg.chain.BlockBeingSynced(blkRoot) {
				log.WithField("BlockRoot", fmt.Sprintf("%#x", blkRoot)).Info("Skipping pending block already being processed")
				continue
			}

			// Remove and skip blocks already in the database.
			if s.cfg.beaconDB.HasBlock(ctx, blkRoot) {
				if err := s.removeBlockFromQueue(b, blkRoot); err != nil {
					return err
				}
				continue
			}

			parentRoot := b.Block().ParentRoot()
			inPendingQueue := s.isBlockInQueue(parentRoot)

			// Check if block is bad.
			keepProcessing, err := s.checkIfBlockIsBad(ctx, span, slot, b, blkRoot)
			if err != nil {
				return err
			}
			if !keepProcessing {
				continue
			}

			// Request parent block if not in the pending queue and not in the database.
			isParentBlockInDB := s.cfg.beaconDB.HasBlock(ctx, parentRoot)
			if !inPendingQueue && !isParentBlockInDB && s.hasPeer() {
				parentRoots = append(parentRoots, parentRoot)
				continue
			}
			if !isParentBlockInDB {
				continue
			}

			// Calculate the deadline time by adding three slots duration to the current time
			secondsPerSlot := params.BeaconConfig().SecondsPerSlot
			threeSlotDuration := 3 * time.Duration(secondsPerSlot) * time.Second
			ctxWithTimeout, cancelFunction := context.WithTimeout(ctx, threeSlotDuration)
			// Process and broadcast the block.
			if err := s.processAndBroadcastBlock(ctxWithTimeout, b, blkRoot); err != nil {
				s.handleBlockProcessingError(ctxWithTimeout, err, b, blkRoot)
				cancelFunction()
				continue
			}
			cancelFunction()

			// Remove the processed block from the queue.
			if err := s.removeBlockFromQueue(b, blkRoot); err != nil {
				return err
			}
			log.WithFields(logrus.Fields{"slot": slot, "blockRoot": hex.EncodeToString(bytesutil.Trunc(blkRoot[:]))}).Debug("Processed pending block and cleared it in cache")
		}
		span.End()
	}
	return s.sendBatchRootRequest(ctx, parentRoots, randGen)
}

// startInnerSpan starts a new tracing span for an inner loop and returns the new context and span.
func startInnerSpan(ctx context.Context, slot primitives.Slot) (context.Context, *trace.Span) {
	ctx, span := trace.StartSpan(ctx, "processPendingBlocks.InnerLoop")
	span.AddAttributes(trace.Int64Attribute("slot", int64(slot))) // lint:ignore uintcast -- This conversion is OK for tracing.
	return ctx, span
}

// getBlocksInQueue retrieves the blocks in the pending queue for a given slot.
func (s *Service) getBlocksInQueue(slot primitives.Slot) []interfaces.ReadOnlySignedBeaconBlock {
	s.pendingQueueLock.RLock()
	defer s.pendingQueueLock.RUnlock()
	return s.pendingBlocksInCache(slot)
}

// removeBlockFromQueue removes a block from the pending queue.
func (s *Service) removeBlockFromQueue(b interfaces.ReadOnlySignedBeaconBlock, blkRoot [32]byte) error {
	s.pendingQueueLock.Lock()
	defer s.pendingQueueLock.Unlock()
	if err := s.deleteBlockFromPendingQueue(b.Block().Slot(), b, blkRoot); err != nil {
		return err
	}
	return nil
}

// isBlockInQueue checks if a block's parent root is in the pending queue.
func (s *Service) isBlockInQueue(parentRoot [32]byte) bool {
	s.pendingQueueLock.RLock()
	defer s.pendingQueueLock.RUnlock()

	return s.seenPendingBlocks[parentRoot]
}

func (s *Service) hasPeer() bool {
	return len(s.cfg.p2p.Peers().Connected()) > 0
}

// processAndBroadcastBlock validates, processes, and broadcasts a block.
// part of the function is to request missing blobs from peers if the block contains kzg commitments.
func (s *Service) processAndBroadcastBlock(ctx context.Context, b interfaces.ReadOnlySignedBeaconBlock, blkRoot [32]byte) error {
	if err := s.validateBeaconBlock(ctx, b, blkRoot); err != nil {
		if !errors.Is(ErrOptimisticParent, err) {
			log.WithError(err).WithField("slot", b.Block().Slot()).Debug("Could not validate block")
			return err
		}
	}

	peers := s.getBestPeers()
	peerCount := len(peers)
	if peerCount > 0 {
		if err := s.requestPendingBlobs(ctx, b, blkRoot, peers[rand.NewGenerator().Int()%peerCount]); err != nil {
			return err
		}
	}

	if err := s.cfg.chain.ReceiveBlock(ctx, b, blkRoot, nil); err != nil {
		return err
	}

	s.setSeenBlockIndexSlot(b.Block().Slot(), b.Block().ProposerIndex())

	pb, err := b.Proto()
	if err != nil {
		log.WithError(err).Debug("Could not get protobuf block")
		return err
	}
	if err := s.cfg.p2p.Broadcast(ctx, pb); err != nil {
		log.WithError(err).Debug("Could not broadcast block")
		return err
	}

	return nil
}

// handleBlockProcessingError handles errors during block processing.
func (s *Service) handleBlockProcessingError(ctx context.Context, err error, b interfaces.ReadOnlySignedBeaconBlock, blkRoot [32]byte) {
	if blockchain.IsInvalidBlock(err) {
		s.setBadBlock(ctx, blkRoot)
	}
	log.WithError(err).WithField("slot", b.Block().Slot()).Debug("Could not process block")
}

// getBestPeers returns the list of best peers based on finalized checkpoint epoch.
func (s *Service) getBestPeers() []core.PeerID {
	_, bestPeers := s.cfg.p2p.Peers().BestFinalized(maxPeerRequest, s.cfg.chain.FinalizedCheckpt().Epoch)
	return bestPeers
}

func (s *Service) checkIfBlockIsBad(
	ctx context.Context,
	span *trace.Span,
	slot primitives.Slot,
	b interfaces.ReadOnlySignedBeaconBlock,
	blkRoot [32]byte,
) (keepProcessing bool, err error) {
	parentIsBad := s.hasBadBlock(b.Block().ParentRoot())
	blockIsBad := s.hasBadBlock(blkRoot)
	// Check if parent is a bad block.
	if parentIsBad || blockIsBad {
		// Set block as bad if its parent block is bad too.
		if parentIsBad {
			s.setBadBlock(ctx, blkRoot)
		}
		// Remove block from queue.
		s.pendingQueueLock.Lock()
		if err = s.deleteBlockFromPendingQueue(slot, b, blkRoot); err != nil {
			s.pendingQueueLock.Unlock()
			return false, err
		}
		s.pendingQueueLock.Unlock()
		span.End()
		return false, nil
	}

	return true, nil
}

func (s *Service) sendBatchRootRequest(ctx context.Context, roots [][32]byte, randGen *rand.Rand) error {
	ctx, span := trace.StartSpan(ctx, "sendBatchRootRequest")
	defer span.End()

	roots = dedupRoots(roots)
	s.pendingQueueLock.RLock()
	for i := len(roots) - 1; i >= 0; i-- {
		r := roots[i]
		if s.seenPendingBlocks[r] || s.cfg.chain.BlockBeingSynced(r) {
			roots = append(roots[:i], roots[i+1:]...)
		} else {
			log.WithField("blockRoot", fmt.Sprintf("%#x", r)).Debug("Requesting block by root")
		}
	}
	s.pendingQueueLock.RUnlock()

	if len(roots) == 0 {
		return nil
	}
	bestPeers := s.getBestPeers()
	if len(bestPeers) == 0 {
		return nil
	}
	// Randomly choose a peer to query from our best peers. If that peer cannot return
	// all the requested blocks, we randomly select another peer.
	pid := bestPeers[randGen.Int()%len(bestPeers)]
	for i := 0; i < numOfTries; i++ {
		req := p2ptypes.BeaconBlockByRootsReq(roots)
		currentEpoch := slots.ToEpoch(s.cfg.clock.CurrentSlot())
		maxReqBlock := params.MaxRequestBlock(currentEpoch)
		if uint64(len(roots)) > maxReqBlock {
			req = roots[:maxReqBlock]
		}
		if err := s.sendRecentBeaconBlocksRequest(ctx, &req, pid); err != nil {
			tracing.AnnotateError(span, err)
			log.WithError(err).Debug("Could not send recent block request")
		}
		newRoots := make([][32]byte, 0, len(roots))
		s.pendingQueueLock.RLock()
		for _, rt := range roots {
			if !s.seenPendingBlocks[rt] {
				newRoots = append(newRoots, rt)
			}
		}
		s.pendingQueueLock.RUnlock()
		if len(newRoots) == 0 {
			break
		}
		// Choosing a new peer with the leftover set of
		// roots to request.
		roots = newRoots
		pid = bestPeers[randGen.Int()%len(bestPeers)]
	}
	return nil
}

func (s *Service) sortedPendingSlots() []primitives.Slot {
	s.pendingQueueLock.RLock()
	defer s.pendingQueueLock.RUnlock()

	items := s.slotToPendingBlocks.Items()

	ss := make([]primitives.Slot, 0, len(items))
	for k := range items {
		slot := cacheKeyToSlot(k)
		ss = append(ss, slot)
	}
	sort.Slice(ss, func(i, j int) bool {
		return ss[i] < ss[j]
	})
	return ss
}

// validatePendingSlots validates the pending blocks
// by their slot. If they are before the current finalized
// checkpoint, these blocks are removed from the queue.
func (s *Service) validatePendingSlots() error {
	s.pendingQueueLock.Lock()
	defer s.pendingQueueLock.Unlock()
	oldBlockRoots := make(map[[32]byte]bool)

	cp := s.cfg.chain.FinalizedCheckpt()
	finalizedEpoch := cp.Epoch
	if s.slotToPendingBlocks == nil {
		return errors.New("slotToPendingBlocks cache can't be nil")
	}
	items := s.slotToPendingBlocks.Items()
	for k := range items {
		slot := cacheKeyToSlot(k)
		blks := s.pendingBlocksInCache(slot)
		for _, b := range blks {
			epoch := slots.ToEpoch(slot)
			// remove all descendant blocks of old blocks
			if oldBlockRoots[b.Block().ParentRoot()] {
				root, err := b.Block().HashTreeRoot()
				if err != nil {
					return err
				}
				oldBlockRoots[root] = true
				if err := s.deleteBlockFromPendingQueue(slot, b, root); err != nil {
					return err
				}
				continue
			}
			// don't process old blocks
			if finalizedEpoch > 0 && epoch <= finalizedEpoch {
				blkRoot, err := b.Block().HashTreeRoot()
				if err != nil {
					return err
				}
				oldBlockRoots[blkRoot] = true
				if err := s.deleteBlockFromPendingQueue(slot, b, blkRoot); err != nil {
					return err
				}
			}
		}
	}
	return nil
}

func (s *Service) clearPendingSlots() {
	s.pendingQueueLock.Lock()
	defer s.pendingQueueLock.Unlock()
	s.slotToPendingBlocks.Flush()
	s.seenPendingBlocks = make(map[[32]byte]bool)
}

// Delete block from the list from the pending queue using the slot as key.
// Note: this helper is not thread safe.
func (s *Service) deleteBlockFromPendingQueue(slot primitives.Slot, b interfaces.ReadOnlySignedBeaconBlock, r [32]byte) error {
	mutexasserts.AssertRWMutexLocked(&s.pendingQueueLock)

	blks := s.pendingBlocksInCache(slot)
	if len(blks) == 0 {
		return nil
	}

	// Defensive check to ignore nil blocks
	if err := blocks.BeaconBlockIsNil(b); err != nil {
		return err
	}

	newBlks := make([]interfaces.ReadOnlySignedBeaconBlock, 0, len(blks))
	for _, blk := range blks {
		blkPb, err := blk.Proto()
		if err != nil {
			return err
		}
		bPb, err := b.Proto()
		if err != nil {
			return err
		}
		if equality.DeepEqual(blkPb, bPb) {
			continue
		}
		newBlks = append(newBlks, blk)
	}
	if len(newBlks) == 0 {
		s.slotToPendingBlocks.Delete(slotToCacheKey(slot))
		delete(s.seenPendingBlocks, r)
		return nil
	}

	// Decrease exp time in proportion to how many blocks are still in the cache for slot key.
	d := pendingBlockExpTime / time.Duration(len(newBlks))
	if err := s.slotToPendingBlocks.Replace(slotToCacheKey(slot), newBlks, d); err != nil {
		return err
	}
	delete(s.seenPendingBlocks, r)
	return nil
}

// This method manually clears our cache so that all expired
// entries are correctly removed.
func (s *Service) deleteExpiredBlocksFromCache() {
	s.pendingQueueLock.Lock()
	defer s.pendingQueueLock.Unlock()

	s.slotToPendingBlocks.DeleteExpired()
}

// Insert block to the list in the pending queue using the slot as key.
// Note: this helper is not thread safe.
func (s *Service) insertBlockToPendingQueue(_ primitives.Slot, b interfaces.ReadOnlySignedBeaconBlock, r [32]byte) error {
	mutexasserts.AssertRWMutexLocked(&s.pendingQueueLock)

	if s.seenPendingBlocks[r] {
		return nil
	}

	if err := s.addPendingBlockToCache(b); err != nil {
		return err
	}

	s.seenPendingBlocks[r] = true
	return nil
}

// This returns signed beacon blocks given input key from slotToPendingBlocks.
func (s *Service) pendingBlocksInCache(slot primitives.Slot) []interfaces.ReadOnlySignedBeaconBlock {
	k := slotToCacheKey(slot)
	value, ok := s.slotToPendingBlocks.Get(k)
	if !ok {
		return []interfaces.ReadOnlySignedBeaconBlock{}
	}
	blks, ok := value.([]interfaces.ReadOnlySignedBeaconBlock)
	if !ok {
		return []interfaces.ReadOnlySignedBeaconBlock{}
	}
	return blks
}

// This adds input signed beacon block to slotToPendingBlocks cache.
func (s *Service) addPendingBlockToCache(b interfaces.ReadOnlySignedBeaconBlock) error {
	if err := blocks.BeaconBlockIsNil(b); err != nil {
		return err
	}

	blks := s.pendingBlocksInCache(b.Block().Slot())

	if len(blks) >= maxBlocksPerSlot {
		return nil
	}

	blks = append(blks, b)
	k := slotToCacheKey(b.Block().Slot())
	s.slotToPendingBlocks.Set(k, blks, pendingBlockExpTime)
	return nil
}

// This converts input string to slot.
func cacheKeyToSlot(s string) primitives.Slot {
	b := []byte(s)
	return bytesutil.BytesToSlotBigEndian(b)
}

// This converts input slot to a key to be used for slotToPendingBlocks cache.
func slotToCacheKey(s primitives.Slot) string {
	b := bytesutil.SlotToBytesBigEndian(s)
	return string(b)
}

func dedupRoots(roots [][32]byte) [][32]byte {
	newRoots := make([][32]byte, 0, len(roots))
	rootMap := make(map[[32]byte]bool, len(roots))
	for i, r := range roots {
		if rootMap[r] {
			continue
		}
		rootMap[r] = true
		newRoots = append(newRoots, roots[i])
	}
	return newRoots
}
