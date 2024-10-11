package doublylinkedtree

import (
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// orphanLateBlockProposingEarly determines the maximum threshold that we
// consider the node is proposing early and sure to receive proposer boost
const orphanLateBlockProposingEarly = 2

// ShouldOverrideFCU returns whether the current forkchoice head is weak
// and thus may be reorged when proposing the next block.
// This function should only be called if the following two conditions are
// satisfied:
// 1-   It is immediately after receiving a block that may be subject to a reorg
//
//	or
//
//	It is right after processAttestationsThreshold and we have processed the
//	current slots attestations.
//
// 2- The caller has already called Forkchoice.Head() so that forkchoice has
// been updated.
// 3- The beacon node is serving a validator that will propose during the next
// slot.
//
// This function only applies a heuristic to decide if the beacon will update
// the engine's view of head with the parent block or the incoming block. It
// does not guarantee an attempted reorg. This will only be decided later at
// proposal time by calling GetProposerHead.
func (f *ForkChoice) ShouldOverrideFCU() (override bool) {
	override = false

	// We only need to override FCU if our current head is from the current
	// slot. This differs from the spec implementation in that we assume
	// that we will call this function in the previous slot to proposing.
	head := f.store.headNode
	if head == nil {
		return
	}

	if head.slot != slots.CurrentSlot(f.store.genesisTime) {
		return
	}

	// Do not reorg on epoch boundaries
	if (head.slot+1)%params.BeaconConfig().SlotsPerEpoch == 0 {
		return
	}
	// Only reorg blocks that arrive late
	early, err := head.arrivedEarly(f.store.genesisTime)
	if err != nil {
		log.WithError(err).Error("could not check if block arrived early")
		return
	}
	if early {
		return
	}
	// Only reorg if we have been finalizing
	finalizedEpoch := f.store.finalizedCheckpoint.Epoch
	if slots.ToEpoch(head.slot+1) > finalizedEpoch+params.BeaconConfig().ReorgMaxEpochsSinceFinalization {
		return
	}
	// Only orphan a single block
	parent := head.parent
	if parent == nil {
		return
	}
	if head.slot > parent.slot+1 {
		return
	}
	// Do not orphan a block that has higher justification than the parent
	// if head.unrealizedJustifiedEpoch > parent.unrealizedJustifiedEpoch {
	//		return
	// }

	// Only orphan a block if the head LMD vote is weak
	if head.weight*100 > f.store.committeeWeight*params.BeaconConfig().ReorgWeightThreshold {
		return
	}

	// Only orphan a block if the parent LMD vote is strong
	if parent.weight*100 < f.store.committeeWeight*params.BeaconConfig().ReorgParentWeightThreshold {
		return
	}
	return true
}

// GetProposerHead returns the block root that has to be used as ParentRoot by a
// proposer. It may not be the actual head of the canonical chain, in certain
// cases it may be its parent, when the last head block has arrived early and is
// considered safe to be orphaned.
//
// This function needs to be called only when proposing a block and all
// attestation processing has already happened.
func (f *ForkChoice) GetProposerHead() [32]byte {
	head := f.store.headNode
	if head == nil {
		return [32]byte{}
	}

	// Only reorg blocks from the previous slot.
	if head.slot+1 != slots.CurrentSlot(f.store.genesisTime) {
		return head.root
	}
	// Do not reorg on epoch boundaries
	if (head.slot+1)%params.BeaconConfig().SlotsPerEpoch == 0 {
		return head.root
	}
	// Only reorg blocks that arrive late
	early, err := head.arrivedEarly(f.store.genesisTime)
	if err != nil {
		log.WithError(err).Error("could not check if block arrived early")
		return head.root
	}
	if early {
		return head.root
	}
	// Only reorg if we have been finalizing
	finalizedEpoch := f.store.finalizedCheckpoint.Epoch
	if slots.ToEpoch(head.slot+1) > finalizedEpoch+params.BeaconConfig().ReorgMaxEpochsSinceFinalization {
		return head.root
	}
	// Only orphan a single block
	parent := head.parent
	if parent == nil {
		return head.root
	}
	if head.slot > parent.slot+1 {
		return head.root
	}

	// Only orphan a block if the head LMD vote is weak
	if head.weight*100 > f.store.committeeWeight*params.BeaconConfig().ReorgWeightThreshold {
		return head.root
	}

	// Only orphan a block if the parent LMD vote is strong
	if parent.weight*100 < f.store.committeeWeight*params.BeaconConfig().ReorgParentWeightThreshold {
		return head.root
	}

	// Only reorg if we are proposing early
	secs, err := slots.SecondsSinceSlotStart(head.slot+1, f.store.genesisTime, uint64(time.Now().Unix()))
	if err != nil {
		log.WithError(err).Error("could not check if proposing early")
		return head.root
	}
	if secs >= orphanLateBlockProposingEarly {
		return head.root
	}
	return parent.root
}
