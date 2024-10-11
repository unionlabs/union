package initialsync

import (
	"context"
	"fmt"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	p2pTypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	p2ppb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// forkData represents alternative chain path supported by a given peer.
// Blocks are stored in an ascending slot order. The first block is guaranteed to have parent
// either in DB or initial sync cache.
type forkData struct {
	peer peer.ID
	bwb  []blocks.BlockWithROBlobs
}

// nonSkippedSlotAfter checks slots after the given one in an attempt to find a non-empty future slot.
// For efficiency only one random slot is checked per epoch, so returned slot might not be the first
// non-skipped slot. This shouldn't be a problem, as in case of adversary peer, we might get incorrect
// data anyway, so code that relies on this function must be robust enough to re-request, if no progress
// is possible with a returned value.
func (f *blocksFetcher) nonSkippedSlotAfter(ctx context.Context, slot primitives.Slot) (primitives.Slot, error) {
	ctx, span := trace.StartSpan(ctx, "initialsync.nonSkippedSlotAfter")
	defer span.End()

	headEpoch, targetEpoch, peers := f.calculateHeadAndTargetEpochs()
	log.WithFields(logrus.Fields{
		"start":       slot,
		"headEpoch":   headEpoch,
		"targetEpoch": targetEpoch,
	}).Debug("Searching for non-skipped slot")

	// Exit early if no peers with epoch higher than our known head are found.
	if targetEpoch <= headEpoch {
		return 0, errSlotIsTooHigh
	}

	// Transform peer list to avoid eclipsing (filter, shuffle, trim).
	peers = f.filterPeers(ctx, peers, peersPercentagePerRequest)
	return f.nonSkippedSlotAfterWithPeersTarget(ctx, slot, peers, targetEpoch)
}

// nonSkippedSlotWithPeersTarget traverse peers (supporting a given target epoch), in an attempt
// to find non-skipped slot among returned blocks.
func (f *blocksFetcher) nonSkippedSlotAfterWithPeersTarget(
	ctx context.Context, slot primitives.Slot, peers []peer.ID, targetEpoch primitives.Epoch,
) (primitives.Slot, error) {
	// Exit early if no peers are ready.
	if len(peers) == 0 {
		return 0, errNoPeersAvailable
	}

	slotsPerEpoch := params.BeaconConfig().SlotsPerEpoch
	pidInd := 0

	fetch := func(pid peer.ID, start primitives.Slot, count, step uint64) (primitives.Slot, error) {
		req := &p2ppb.BeaconBlocksByRangeRequest{
			StartSlot: start,
			Count:     count,
			Step:      step,
		}
		blocks, err := f.requestBlocks(ctx, req, pid)
		if err != nil {
			return 0, err
		}
		if len(blocks) > 0 {
			for _, block := range blocks {
				if block.Block().Slot() > slot {
					return block.Block().Slot(), nil
				}
			}
		}
		return 0, nil
	}

	// Start by checking several epochs fully, w/o resorting to random sampling.
	start := slot + 1
	end := start + nonSkippedSlotsFullSearchEpochs*slotsPerEpoch
	for ind := start; ind < end; ind += slotsPerEpoch {
		nextSlot, err := fetch(peers[pidInd%len(peers)], ind, uint64(slotsPerEpoch), 1)
		if err != nil {
			return 0, err
		}
		if nextSlot > slot {
			return nextSlot, nil
		}
		pidInd++
	}

	// Quickly find the close enough epoch where a non-empty slot definitely exists.
	// Only single random slot per epoch is checked - allowing to move forward relatively quickly.
	// This method has been changed to account for our spec change where step can only be 1 in a
	// block by range request. https://github.com/ethereum/consensus-specs/pull/2856
	// The downside is that this method will be less effective during periods without
	// finality.
	slot += nonSkippedSlotsFullSearchEpochs * slotsPerEpoch
	upperBoundSlot, err := slots.EpochStart(targetEpoch + 1)
	if err != nil {
		return 0, err
	}
	for ind := slot + 1; ind < upperBoundSlot; ind += slotsPerEpoch {
		nextSlot, err := fetch(peers[pidInd%len(peers)], ind, uint64(slotsPerEpoch), 1)
		if err != nil {
			return 0, err
		}
		pidInd++
		if nextSlot > slot && upperBoundSlot >= nextSlot {
			upperBoundSlot = nextSlot
			break
		}
	}

	// Epoch with non-empty slot is located. Check all slots within two nearby epochs.
	if upperBoundSlot > slotsPerEpoch {
		upperBoundSlot -= slotsPerEpoch
	}
	upperBoundSlot, err = slots.EpochStart(slots.ToEpoch(upperBoundSlot))
	if err != nil {
		return 0, err
	}
	nextSlot, err := fetch(peers[pidInd%len(peers)], upperBoundSlot, uint64(slotsPerEpoch*2), 1)
	if err != nil {
		return 0, err
	}
	s, err := slots.EpochStart(targetEpoch + 1)
	if err != nil {
		return 0, err
	}
	if nextSlot < slot || s < nextSlot {
		return 0, errors.New("invalid range for non-skipped slot")
	}
	return nextSlot, nil
}

// findFork queries all peers that have higher head slot, in an attempt to find
// ones that feature blocks from alternative branches. Once found, peer is further queried
// to find common ancestor slot. On success, all obtained blocks and peer is returned.
func (f *blocksFetcher) findFork(ctx context.Context, slot primitives.Slot) (*forkData, error) {
	ctx, span := trace.StartSpan(ctx, "initialsync.findFork")
	defer span.End()

	// Safe-guard, since previous epoch is used when calculating.
	slotsPerEpoch := params.BeaconConfig().SlotsPerEpoch
	if slot < slotsPerEpoch*2 {
		return nil, fmt.Errorf("slot is too low to backtrack, min. expected %d", slotsPerEpoch*2)
	}

	// The current slot's epoch must be after the finalization epoch,
	// triggering backtracking on earlier epochs is unnecessary.
	cp := f.chain.FinalizedCheckpt()
	finalizedEpoch := cp.Epoch
	epoch := slots.ToEpoch(slot)
	if epoch <= finalizedEpoch {
		return nil, errors.New("slot is not after the finalized epoch, no backtracking is necessary")
	}
	// Update slot to the beginning of the current epoch (preserve original slot for comparison).
	slot, err := slots.EpochStart(epoch)
	if err != nil {
		return nil, err
	}

	// Select peers that have higher head slot, and potentially blocks from more favourable fork.
	// Exit early if no peers are ready.
	_, peers := f.p2p.Peers().BestNonFinalized(1, epoch+1)
	if len(peers) == 0 {
		return nil, errNoPeersAvailable
	}
	f.rand.Shuffle(len(peers), func(i, j int) {
		peers[i], peers[j] = peers[j], peers[i]
	})

	// Query all found peers, stop on peer with alternative blocks, and try backtracking.
	for i, pid := range peers {
		log.WithFields(logrus.Fields{
			"peer": pid,
			"step": fmt.Sprintf("%d/%d", i+1, len(peers)),
		}).Debug("Searching for alternative blocks")
		fork, err := f.findForkWithPeer(ctx, pid, slot)
		if err != nil {
			log.WithFields(logrus.Fields{
				"peer":  pid,
				"error": err.Error(),
			}).Debug("No alternative blocks found for peer")
			continue
		}
		return fork, nil
	}
	return nil, errNoPeersWithAltBlocks
}

var errNoAlternateBlocks = errors.New("no alternative blocks exist within scanned range")

func findForkReqRangeSize() uint64 {
	return uint64(params.BeaconConfig().SlotsPerEpoch.Mul(2))
}

// findForkWithPeer loads some blocks from a peer in an attempt to find alternative blocks.
func (f *blocksFetcher) findForkWithPeer(ctx context.Context, pid peer.ID, slot primitives.Slot) (*forkData, error) {
	reqCount := findForkReqRangeSize()
	// Safe-guard, since previous epoch is used when calculating.
	if uint64(slot) < reqCount {
		return nil, fmt.Errorf("slot is too low to backtrack, min. expected %d", reqCount)
	}
	slotsPerEpoch := params.BeaconConfig().SlotsPerEpoch

	// Locate non-skipped slot, supported by a given peer (can survive long periods of empty slots).
	// When searching for non-empty slot, start an epoch earlier - for those blocks we
	// definitely have roots. So, spotting a fork will be easier. It is not a problem if unknown
	// block of the current fork is found: we are searching for forks when FSMs are stuck, so
	// being able to progress on any fork is good.
	pidState, err := f.p2p.Peers().ChainState(pid)
	if err != nil {
		return nil, fmt.Errorf("cannot obtain peer's status: %w", err)
	}
	nonSkippedSlot, err := f.nonSkippedSlotAfterWithPeersTarget(
		ctx, slot-slotsPerEpoch, []peer.ID{pid}, slots.ToEpoch(pidState.HeadSlot))
	if err != nil {
		return nil, fmt.Errorf("cannot locate non-empty slot for a peer: %w", err)
	}

	// Request blocks starting from the first non-empty slot.
	req := &p2ppb.BeaconBlocksByRangeRequest{
		StartSlot: nonSkippedSlot,
		Count:     reqCount,
		Step:      1,
	}
	blocks, err := f.requestBlocks(ctx, req, pid)
	if err != nil {
		return nil, fmt.Errorf("cannot fetch blocks: %w", err)
	}
	if len(blocks) == 0 {
		return nil, errNoAlternateBlocks
	}

	// If the first block is not connected to the current canonical chain, we'll stop processing this batch.
	// Instead, we'll work backwards from the first block until we find a common ancestor,
	// and then begin processing from there.
	first := blocks[0]
	if !f.chain.HasBlock(ctx, first.Block().ParentRoot()) {
		// Backtrack on a root, to find a common ancestor from which we can resume syncing.
		fork, err := f.findAncestor(ctx, pid, first)
		if err != nil {
			return nil, fmt.Errorf("failed to find common ancestor: %w", err)
		}
		return fork, nil
	}

	// Traverse blocks, and if we've got one that doesn't have parent in DB, backtrack on it.
	// Note that we start from the second element in the array, because we know that the first element is in the db,
	// otherwise we would have gone into the findAncestor early return path above.
	for i := 1; i < len(blocks); i++ {
		block := blocks[i]
		parentRoot := block.Block().ParentRoot()
		// Step through blocks until we find one that is not in the chain. The goal is to find the point where the
		// chain observed in the peer diverges from the locally known chain, and then collect up the remainder of the
		// observed chain chunk to start initial-sync processing from the fork point.
		if f.chain.HasBlock(ctx, parentRoot) {
			continue
		}
		log.WithFields(logrus.Fields{
			"peer": pid,
			"slot": block.Block().Slot(),
			"root": fmt.Sprintf("%#x", parentRoot),
		}).Debug("Block with unknown parent root has been found")
		altBlocks, err := sortedBlockWithVerifiedBlobSlice(blocks[i-1:])
		if err != nil {
			return nil, errors.Wrap(err, "invalid blocks received in findForkWithPeer")
		}
		// We need to fetch the blobs for the given alt-chain if any exist, so that we can try to verify and import
		// the blocks.
		bwb, err := f.fetchBlobsFromPeer(ctx, altBlocks, pid)
		if err != nil {
			return nil, errors.Wrap(err, "unable to retrieve blobs for blocks found in findForkWithPeer")
		}
		// The caller will use the BlocksWith VerifiedBlobs in bwb as the starting point for
		// round-robin syncing the alternate chain.
		return &forkData{peer: pid, bwb: bwb}, nil
	}
	return nil, errNoAlternateBlocks
}

// findAncestor tries to figure out common ancestor slot that connects a given root to known block.
func (f *blocksFetcher) findAncestor(ctx context.Context, pid peer.ID, b interfaces.ReadOnlySignedBeaconBlock) (*forkData, error) {
	outBlocks := []interfaces.ReadOnlySignedBeaconBlock{b}
	for i := uint64(0); i < backtrackingMaxHops; i++ {
		parentRoot := outBlocks[len(outBlocks)-1].Block().ParentRoot()
		if f.chain.HasBlock(ctx, parentRoot) {
			// Common ancestor found, forward blocks back to processor.
			bwb, err := sortedBlockWithVerifiedBlobSlice(outBlocks)
			if err != nil {
				return nil, errors.Wrap(err, "received invalid blocks in findAncestor")
			}
			bwb, err = f.fetchBlobsFromPeer(ctx, bwb, pid)
			if err != nil {
				return nil, errors.Wrap(err, "unable to retrieve blobs for blocks found in findAncestor")
			}
			return &forkData{
				peer: pid,
				bwb:  bwb,
			}, nil
		}
		// Request block's parent.
		req := &p2pTypes.BeaconBlockByRootsReq{parentRoot}
		blocks, err := f.requestBlocksByRoot(ctx, req, pid)
		if err != nil {
			return nil, err
		}
		if len(blocks) == 0 {
			break
		}
		outBlocks = append(outBlocks, blocks[0])
	}
	return nil, errors.New("no common ancestor found")
}

// bestFinalizedSlot returns the highest finalized slot of the majority of connected peers.
func (f *blocksFetcher) bestFinalizedSlot() primitives.Slot {
	cp := f.chain.FinalizedCheckpt()
	finalizedEpoch, _ := f.p2p.Peers().BestFinalized(
		params.BeaconConfig().MaxPeersToSync, cp.Epoch)
	return params.BeaconConfig().SlotsPerEpoch.Mul(uint64(finalizedEpoch))
}

// bestNonFinalizedSlot returns the highest non-finalized slot of enough number of connected peers.
func (f *blocksFetcher) bestNonFinalizedSlot() primitives.Slot {
	headEpoch := slots.ToEpoch(f.chain.HeadSlot())
	targetEpoch, _ := f.p2p.Peers().BestNonFinalized(flags.Get().MinimumSyncPeers*2, headEpoch)
	return params.BeaconConfig().SlotsPerEpoch.Mul(uint64(targetEpoch))
}

// calculateHeadAndTargetEpochs return node's current head epoch, along with the best known target
// epoch. For the latter peers supporting that target epoch are returned as well.
func (f *blocksFetcher) calculateHeadAndTargetEpochs() (headEpoch, targetEpoch primitives.Epoch, peers []peer.ID) {
	if f.mode == modeStopOnFinalizedEpoch {
		cp := f.chain.FinalizedCheckpt()
		headEpoch = cp.Epoch
		targetEpoch, peers = f.p2p.Peers().BestFinalized(params.BeaconConfig().MaxPeersToSync, headEpoch)
	} else {
		headEpoch = slots.ToEpoch(f.chain.HeadSlot())
		targetEpoch, peers = f.p2p.Peers().BestNonFinalized(flags.Get().MinimumSyncPeers, headEpoch)
	}
	return headEpoch, targetEpoch, peers
}
