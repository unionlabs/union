package doublylinkedtree

import (
	"context"
	"fmt"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	forkchoice2 "github.com/prysmaticlabs/prysm/v4/consensus-types/forkchoice"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// New initializes a new fork choice store.
func New() *ForkChoice {
	s := &Store{
		justifiedCheckpoint:           &forkchoicetypes.Checkpoint{},
		unrealizedJustifiedCheckpoint: &forkchoicetypes.Checkpoint{},
		unrealizedFinalizedCheckpoint: &forkchoicetypes.Checkpoint{},
		prevJustifiedCheckpoint:       &forkchoicetypes.Checkpoint{},
		finalizedCheckpoint:           &forkchoicetypes.Checkpoint{},
		proposerBoostRoot:             [32]byte{},
		nodeByRoot:                    make(map[[fieldparams.RootLength]byte]*Node),
		nodeByPayload:                 make(map[[fieldparams.RootLength]byte]*Node),
		slashedIndices:                make(map[primitives.ValidatorIndex]bool),
		receivedBlocksLastEpoch:       [fieldparams.SlotsPerEpoch]primitives.Slot{},
	}

	b := make([]uint64, 0)
	v := make([]Vote, 0)
	return &ForkChoice{store: s, balances: b, votes: v}
}

// NodeCount returns the current number of nodes in the Store.
func (f *ForkChoice) NodeCount() int {
	return len(f.store.nodeByRoot)
}

// Head returns the head root from fork choice store.
// It firsts computes validator's balance changes then recalculates block tree from leaves to root.
func (f *ForkChoice) Head(
	ctx context.Context,
) ([32]byte, error) {
	ctx, span := trace.StartSpan(ctx, "doublyLinkedForkchoice.Head")
	defer span.End()

	calledHeadCount.Inc()

	if err := f.updateBalances(); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not update balances")
	}

	if err := f.applyProposerBoostScore(); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not apply proposer boost score")
	}

	if err := f.store.treeRootNode.applyWeightChanges(ctx); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not apply weight changes")
	}

	jc := f.JustifiedCheckpoint()
	fc := f.FinalizedCheckpoint()
	currentEpoch := slots.EpochsSinceGenesis(time.Unix(int64(f.store.genesisTime), 0))
	if err := f.store.treeRootNode.updateBestDescendant(ctx, jc.Epoch, fc.Epoch, currentEpoch); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not update best descendant")
	}
	return f.store.head(ctx)
}

// ProcessAttestation processes attestation for vote accounting, it iterates around validator indices
// and update their votes accordingly.
func (f *ForkChoice) ProcessAttestation(ctx context.Context, validatorIndices []uint64, blockRoot [32]byte, targetEpoch primitives.Epoch) {
	_, span := trace.StartSpan(ctx, "doublyLinkedForkchoice.ProcessAttestation")
	defer span.End()

	for _, index := range validatorIndices {
		// Validator indices will grow the vote cache.
		for index >= uint64(len(f.votes)) {
			f.votes = append(f.votes, Vote{currentRoot: params.BeaconConfig().ZeroHash, nextRoot: params.BeaconConfig().ZeroHash})
		}

		// Newly allocated vote if the root fields are untouched.
		newVote := f.votes[index].nextRoot == params.BeaconConfig().ZeroHash &&
			f.votes[index].currentRoot == params.BeaconConfig().ZeroHash

		// Vote gets updated if it's newly allocated or high target epoch.
		if newVote || targetEpoch > f.votes[index].nextEpoch {
			f.votes[index].nextEpoch = targetEpoch
			f.votes[index].nextRoot = blockRoot
		}
	}

	processedAttestationCount.Inc()
}

// InsertNode processes a new block by inserting it to the fork choice store.
func (f *ForkChoice) InsertNode(ctx context.Context, state state.BeaconState, root [32]byte) error {
	ctx, span := trace.StartSpan(ctx, "doublyLinkedForkchoice.InsertNode")
	defer span.End()

	slot := state.Slot()
	bh := state.LatestBlockHeader()
	if bh == nil {
		return errNilBlockHeader
	}
	parentRoot := bytesutil.ToBytes32(bh.ParentRoot)
	var payloadHash [32]byte
	if state.Version() >= version.Bellatrix {
		ph, err := state.LatestExecutionPayloadHeader()
		if err != nil {
			return err
		}
		if ph != nil {
			copy(payloadHash[:], ph.BlockHash())
		}
	}
	jc := state.CurrentJustifiedCheckpoint()
	if jc == nil {
		return errInvalidNilCheckpoint
	}
	justifiedEpoch := jc.Epoch
	fc := state.FinalizedCheckpoint()
	if fc == nil {
		return errInvalidNilCheckpoint
	}
	finalizedEpoch := fc.Epoch
	node, err := f.store.insert(ctx, slot, root, parentRoot, payloadHash, justifiedEpoch, finalizedEpoch)
	if err != nil {
		return err
	}

	jc, fc = f.store.pullTips(state, node, jc, fc)
	return f.updateCheckpoints(ctx, jc, fc)
}

// updateCheckpoints update the checkpoints when inserting a new node.
func (f *ForkChoice) updateCheckpoints(ctx context.Context, jc, fc *ethpb.Checkpoint) error {
	if jc.Epoch > f.store.justifiedCheckpoint.Epoch {
		f.store.prevJustifiedCheckpoint = f.store.justifiedCheckpoint
		jcRoot := bytesutil.ToBytes32(jc.Root)
		f.store.justifiedCheckpoint = &forkchoicetypes.Checkpoint{Epoch: jc.Epoch, Root: jcRoot}
		if err := f.updateJustifiedBalances(ctx, jcRoot); err != nil {
			return errors.Wrap(err, "could not update justified balances")
		}
	}
	// Update finalization
	if fc.Epoch <= f.store.finalizedCheckpoint.Epoch {
		return nil
	}
	f.store.finalizedCheckpoint = &forkchoicetypes.Checkpoint{Epoch: fc.Epoch,
		Root: bytesutil.ToBytes32(fc.Root)}
	return f.store.prune(ctx)
}

// HasNode returns true if the node exists in fork choice store,
// false else wise.
func (f *ForkChoice) HasNode(root [32]byte) bool {
	_, ok := f.store.nodeByRoot[root]
	return ok
}

// IsCanonical returns true if the given root is part of the canonical chain.
func (f *ForkChoice) IsCanonical(root [32]byte) bool {
	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		return false
	}

	if node.bestDescendant == nil {
		if f.store.headNode.bestDescendant == nil {
			return node == f.store.headNode
		}
		return node == f.store.headNode.bestDescendant
	}
	if f.store.headNode.bestDescendant == nil {
		return node.bestDescendant == f.store.headNode
	}
	return node.bestDescendant == f.store.headNode.bestDescendant
}

// IsOptimistic returns true if the given root has been optimistically synced.
func (f *ForkChoice) IsOptimistic(root [32]byte) (bool, error) {
	if f.store.allTipsAreInvalid {
		return true, nil
	}

	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		return true, ErrNilNode
	}

	return node.optimistic, nil
}

// AncestorRoot returns the ancestor root of input block root at a given slot.
func (f *ForkChoice) AncestorRoot(ctx context.Context, root [32]byte, slot primitives.Slot) ([32]byte, error) {
	ctx, span := trace.StartSpan(ctx, "doublyLinkedForkchoice.AncestorRoot")
	defer span.End()

	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		return [32]byte{}, errors.Wrap(ErrNilNode, "could not determine ancestor root")
	}

	n := node
	for n != nil && n.slot > slot {
		if ctx.Err() != nil {
			return [32]byte{}, ctx.Err()
		}
		n = n.parent
	}

	if n == nil {
		return [32]byte{}, errors.Wrap(ErrNilNode, "could not determine ancestor root")
	}

	return n.root, nil
}

// IsViableForCheckpoint returns whether the root passed is a checkpoint root for any
// known chain in forkchoice.
func (f *ForkChoice) IsViableForCheckpoint(cp *forkchoicetypes.Checkpoint) (bool, error) {
	node, ok := f.store.nodeByRoot[cp.Root]
	if !ok || node == nil {
		return false, nil
	}
	epochStart, err := slots.EpochStart(cp.Epoch)
	if err != nil {
		return false, err
	}
	if node.slot > epochStart {
		return false, nil
	}

	if len(node.children) == 0 {
		return true, nil
	}
	if node.slot == epochStart {
		return true, nil
	}
	nodeEpoch := slots.ToEpoch(node.slot)
	if nodeEpoch >= cp.Epoch {
		return false, nil
	}
	for _, child := range node.children {
		if child.slot > epochStart {
			return true, nil
		}
	}
	return false, nil
}

// updateBalances updates the balances that directly voted for each block taking into account the
// validators' latest votes.
func (f *ForkChoice) updateBalances() error {
	newBalances := f.justifiedBalances
	zHash := params.BeaconConfig().ZeroHash

	for index, vote := range f.votes {
		// Skip if validator has been slashed
		if f.store.slashedIndices[primitives.ValidatorIndex(index)] {
			continue
		}
		// Skip if validator has never voted for current root and next root (i.e. if the
		// votes are zero hash aka genesis block), there's nothing to compute.
		if vote.currentRoot == zHash && vote.nextRoot == zHash {
			continue
		}

		oldBalance := uint64(0)
		newBalance := uint64(0)
		// If the validator index did not exist in `f.balances` or
		// `newBalances` list above, the balance is just 0.
		if index < len(f.balances) {
			oldBalance = f.balances[index]
		}
		if index < len(newBalances) {
			newBalance = newBalances[index]
		}

		// Update only if the validator's balance or vote has changed.
		if vote.currentRoot != vote.nextRoot || oldBalance != newBalance {
			// Ignore the vote if the root is not in fork choice
			// store, that means we have not seen the block before.
			nextNode, ok := f.store.nodeByRoot[vote.nextRoot]
			if ok && vote.nextRoot != zHash {
				// Protection against nil node
				if nextNode == nil {
					return errors.Wrap(ErrNilNode, "could not update balances")
				}
				nextNode.balance += newBalance
			}

			currentNode, ok := f.store.nodeByRoot[vote.currentRoot]
			if ok && vote.currentRoot != zHash {
				// Protection against nil node
				if currentNode == nil {
					return errors.Wrap(ErrNilNode, "could not update balances")
				}
				if currentNode.balance < oldBalance {
					log.WithFields(logrus.Fields{
						"nodeRoot":                   fmt.Sprintf("%#x", bytesutil.Trunc(vote.currentRoot[:])),
						"oldBalance":                 oldBalance,
						"nodeBalance":                currentNode.balance,
						"nodeWeight":                 currentNode.weight,
						"proposerBoostRoot":          fmt.Sprintf("%#x", bytesutil.Trunc(f.store.proposerBoostRoot[:])),
						"previousProposerBoostRoot":  fmt.Sprintf("%#x", bytesutil.Trunc(f.store.previousProposerBoostRoot[:])),
						"previousProposerBoostScore": f.store.previousProposerBoostScore,
					}).Warning("node with invalid balance, setting it to zero")
					currentNode.balance = 0
				} else {
					currentNode.balance -= oldBalance
				}
			}
		}

		// Rotate the validator vote.
		f.votes[index].currentRoot = vote.nextRoot
	}
	f.balances = newBalances
	return nil
}

// Tips returns a list of possible heads from fork choice store, it returns the
// roots and the slots of the leaf nodes.
func (f *ForkChoice) Tips() ([][32]byte, []primitives.Slot) {
	return f.store.tips()
}

// ProposerBoost returns the proposerBoost of the store
func (f *ForkChoice) ProposerBoost() [fieldparams.RootLength]byte {
	return f.store.proposerBoost()
}

// SetOptimisticToValid sets the node with the given root as a fully validated node
func (f *ForkChoice) SetOptimisticToValid(ctx context.Context, root [fieldparams.RootLength]byte) error {
	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		return errors.Wrap(ErrNilNode, "could not set node to valid")
	}
	return node.setNodeAndParentValidated(ctx)
}

// PreviousJustifiedCheckpoint of fork choice store.
func (f *ForkChoice) PreviousJustifiedCheckpoint() *forkchoicetypes.Checkpoint {
	return f.store.prevJustifiedCheckpoint
}

// JustifiedCheckpoint of fork choice store.
func (f *ForkChoice) JustifiedCheckpoint() *forkchoicetypes.Checkpoint {
	return f.store.justifiedCheckpoint
}

// FinalizedCheckpoint of fork choice store.
func (f *ForkChoice) FinalizedCheckpoint() *forkchoicetypes.Checkpoint {
	return f.store.finalizedCheckpoint
}

// SetOptimisticToInvalid removes a block with an invalid execution payload from fork choice store
func (f *ForkChoice) SetOptimisticToInvalid(ctx context.Context, root, parentRoot, payloadHash [fieldparams.RootLength]byte) ([][32]byte, error) {
	return f.store.setOptimisticToInvalid(ctx, root, parentRoot, payloadHash)
}

// InsertSlashedIndex adds the given slashed validator index to the
// store-tracked list. Votes from these validators are not accounted for
// in forkchoice.
func (f *ForkChoice) InsertSlashedIndex(_ context.Context, index primitives.ValidatorIndex) {
	// return early if the index was already included:
	if f.store.slashedIndices[index] {
		return
	}
	f.store.slashedIndices[index] = true

	// Subtract last vote from this equivocating validator

	if index >= primitives.ValidatorIndex(len(f.balances)) {
		return
	}

	if index >= primitives.ValidatorIndex(len(f.votes)) {
		return
	}

	node, ok := f.store.nodeByRoot[f.votes[index].currentRoot]
	if !ok || node == nil {
		return
	}

	if node.balance < f.balances[index] {
		node.balance = 0
	} else {
		node.balance -= f.balances[index]
	}
}

// UpdateJustifiedCheckpoint sets the justified checkpoint to the given one
func (f *ForkChoice) UpdateJustifiedCheckpoint(ctx context.Context, jc *forkchoicetypes.Checkpoint) error {
	if jc == nil {
		return errInvalidNilCheckpoint
	}
	f.store.prevJustifiedCheckpoint = f.store.justifiedCheckpoint
	f.store.justifiedCheckpoint = jc
	if err := f.updateJustifiedBalances(ctx, jc.Root); err != nil {
		return errors.Wrap(err, "could not update justified balances")
	}
	return nil
}

// UpdateFinalizedCheckpoint sets the finalized checkpoint to the given one
func (f *ForkChoice) UpdateFinalizedCheckpoint(fc *forkchoicetypes.Checkpoint) error {
	if fc == nil {
		return errInvalidNilCheckpoint
	}
	f.store.finalizedCheckpoint = fc
	return nil
}

// CommonAncestor returns the common ancestor root and slot between the two block roots r1 and r2.
func (f *ForkChoice) CommonAncestor(ctx context.Context, r1 [32]byte, r2 [32]byte) ([32]byte, primitives.Slot, error) {
	ctx, span := trace.StartSpan(ctx, "doublyLinkedForkchoice.CommonAncestorRoot")
	defer span.End()

	n1, ok := f.store.nodeByRoot[r1]
	if !ok || n1 == nil {
		return [32]byte{}, 0, forkchoice.ErrUnknownCommonAncestor
	}

	// Do nothing if the input roots are the same.
	if r1 == r2 {
		return r1, n1.slot, nil
	}

	n2, ok := f.store.nodeByRoot[r2]
	if !ok || n2 == nil {
		return [32]byte{}, 0, forkchoice.ErrUnknownCommonAncestor
	}

	for {
		if ctx.Err() != nil {
			return [32]byte{}, 0, ctx.Err()
		}
		if n1.slot > n2.slot {
			n1 = n1.parent
			// Reaches the end of the tree and unable to find common ancestor.
			// This should not happen at runtime as the finalized
			// node has to be a common ancestor
			if n1 == nil {
				return [32]byte{}, 0, forkchoice.ErrUnknownCommonAncestor
			}
		} else {
			n2 = n2.parent
			// Reaches the end of the tree and unable to find common ancestor.
			if n2 == nil {
				return [32]byte{}, 0, forkchoice.ErrUnknownCommonAncestor
			}
		}
		if n1 == n2 {
			return n1.root, n1.slot, nil
		}
	}
}

// InsertChain inserts all nodes corresponding to blocks in the slice
// `blocks`. This slice must be ordered from child to parent. It includes all
// blocks **except** the first one (that is the one with the highest slot
// number). All blocks are assumed to be a strict chain
// where blocks[i].Parent = blocks[i+1]. Also, we assume that the parent of the
// last block in this list is already included in forkchoice store.
func (f *ForkChoice) InsertChain(ctx context.Context, chain []*forkchoicetypes.BlockAndCheckpoints) error {
	if len(chain) == 0 {
		return nil
	}
	for i := len(chain) - 1; i > 0; i-- {
		b := chain[i].Block
		r := chain[i-1].Block.ParentRoot()
		parentRoot := b.ParentRoot()
		payloadHash, err := blocks.GetBlockPayloadHash(b)
		if err != nil {
			return err
		}
		if _, err := f.store.insert(ctx,
			b.Slot(), r, parentRoot, payloadHash,
			chain[i].JustifiedCheckpoint.Epoch, chain[i].FinalizedCheckpoint.Epoch); err != nil {
			return err
		}
		if err := f.updateCheckpoints(ctx, chain[i].JustifiedCheckpoint, chain[i].FinalizedCheckpoint); err != nil {
			return err
		}
	}
	return nil
}

// SetGenesisTime sets the genesisTime tracked by forkchoice
func (f *ForkChoice) SetGenesisTime(genesisTime uint64) {
	f.store.genesisTime = genesisTime
}

// SetOriginRoot sets the genesis block root
func (f *ForkChoice) SetOriginRoot(root [32]byte) {
	f.store.originRoot = root
}

// CachedHeadRoot returns the last cached head root
func (f *ForkChoice) CachedHeadRoot() [32]byte {
	node := f.store.headNode
	if node == nil {
		return [32]byte{}
	}
	return f.store.headNode.root
}

// FinalizedPayloadBlockHash returns the hash of the payload at the finalized checkpoint
func (f *ForkChoice) FinalizedPayloadBlockHash() [32]byte {
	root := f.FinalizedCheckpoint().Root
	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		// This should not happen
		return [32]byte{}
	}
	return node.payloadHash
}

// JustifiedPayloadBlockHash returns the hash of the payload at the justified checkpoint
func (f *ForkChoice) JustifiedPayloadBlockHash() [32]byte {
	root := f.JustifiedCheckpoint().Root
	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		// This should not happen
		return [32]byte{}
	}
	return node.payloadHash
}

// UnrealizedJustifiedPayloadBlockHash returns the hash of the payload at the unrealized justified checkpoint
func (f *ForkChoice) UnrealizedJustifiedPayloadBlockHash() [32]byte {
	root := f.store.unrealizedJustifiedCheckpoint.Root
	node, ok := f.store.nodeByRoot[root]
	if !ok || node == nil {
		// This should not happen
		return [32]byte{}
	}
	return node.payloadHash
}

// ForkChoiceDump returns a full dump of forkchoice.
func (f *ForkChoice) ForkChoiceDump(ctx context.Context) (*forkchoice2.Dump, error) {
	jc := &ethpb.Checkpoint{
		Epoch: f.store.justifiedCheckpoint.Epoch,
		Root:  f.store.justifiedCheckpoint.Root[:],
	}
	ujc := &ethpb.Checkpoint{
		Epoch: f.store.unrealizedJustifiedCheckpoint.Epoch,
		Root:  f.store.unrealizedJustifiedCheckpoint.Root[:],
	}
	fc := &ethpb.Checkpoint{
		Epoch: f.store.finalizedCheckpoint.Epoch,
		Root:  f.store.finalizedCheckpoint.Root[:],
	}
	ufc := &ethpb.Checkpoint{
		Epoch: f.store.unrealizedFinalizedCheckpoint.Epoch,
		Root:  f.store.unrealizedFinalizedCheckpoint.Root[:],
	}
	nodes := make([]*forkchoice2.Node, 0, f.NodeCount())
	var err error
	if f.store.treeRootNode != nil {
		nodes, err = f.store.treeRootNode.nodeTreeDump(ctx, nodes)
		if err != nil {
			return nil, err
		}
	}
	var headRoot [32]byte
	if f.store.headNode != nil {
		headRoot = f.store.headNode.root
	}
	resp := &forkchoice2.Dump{
		JustifiedCheckpoint:           jc,
		UnrealizedJustifiedCheckpoint: ujc,
		FinalizedCheckpoint:           fc,
		UnrealizedFinalizedCheckpoint: ufc,
		ProposerBoostRoot:             f.store.proposerBoostRoot[:],
		PreviousProposerBoostRoot:     f.store.previousProposerBoostRoot[:],
		HeadRoot:                      headRoot[:],
		ForkChoiceNodes:               nodes,
	}
	return resp, nil
}

// SetBalancesByRooter sets the balanceByRoot handler in forkchoice
func (f *ForkChoice) SetBalancesByRooter(handler forkchoice.BalancesByRooter) {
	f.balancesByRoot = handler
}

// Weight returns the weight of the given root if found on the store
func (f *ForkChoice) Weight(root [32]byte) (uint64, error) {
	n, ok := f.store.nodeByRoot[root]
	if !ok || n == nil {
		return 0, ErrNilNode
	}
	return n.weight, nil
}

// updateJustifiedBalances updates the validators balances on the justified checkpoint pointed by root.
func (f *ForkChoice) updateJustifiedBalances(ctx context.Context, root [32]byte) error {
	balances, err := f.balancesByRoot(ctx, root)
	if err != nil {
		return errors.Wrap(err, "could not get justified balances")
	}
	f.justifiedBalances = balances
	f.store.committeeWeight = 0
	f.numActiveValidators = 0
	for _, val := range balances {
		if val > 0 {
			f.store.committeeWeight += val
			f.numActiveValidators++
		}
	}
	f.store.committeeWeight /= uint64(params.BeaconConfig().SlotsPerEpoch)
	return nil
}

// Slot returns the slot of the given root if it's known to forkchoice
func (f *ForkChoice) Slot(root [32]byte) (primitives.Slot, error) {
	n, ok := f.store.nodeByRoot[root]
	if !ok || n == nil {
		return 0, ErrNilNode
	}
	return n.slot, nil
}

// TargetRootForEpoch returns the root of the target block for a given epoch.
// The epoch parameter is crucial to identify the correct target root. For example:
// When inserting a block at slot 63 with block root 0xA and target root 0xB (pointing to the block at slot 32),
// and at slot 64, where the block is skipped, the attestation will reference the target root as 0xA (for slot 63), not 0xB (for slot 32).
// This implies that if the input slot exceeds the block slot, the target root will be the same as the block root.
// We also allow for the epoch to be below the current target for this root, in
// which case we return the root of the checkpoint of the chain containing the
// passed root, at the given epoch
func (f *ForkChoice) TargetRootForEpoch(root [32]byte, epoch primitives.Epoch) ([32]byte, error) {
	n, ok := f.store.nodeByRoot[root]
	if !ok || n == nil {
		return [32]byte{}, ErrNilNode
	}
	nodeEpoch := slots.ToEpoch(n.slot)
	if epoch > nodeEpoch {
		return n.root, nil
	}
	if n.target == nil {
		return [32]byte{}, nil
	}
	targetRoot := n.target.root
	if epoch == nodeEpoch {
		return targetRoot, nil
	}
	targetNode, ok := f.store.nodeByRoot[targetRoot]
	if !ok || targetNode == nil {
		return [32]byte{}, ErrNilNode
	}
	// If slot 0 was not missed we consider a previous block to go back at least one epoch
	if nodeEpoch == slots.ToEpoch(targetNode.slot) {
		targetNode = targetNode.parent
		if targetNode == nil {
			return [32]byte{}, ErrNilNode
		}
	}
	return f.TargetRootForEpoch(targetNode.root, epoch)
}
