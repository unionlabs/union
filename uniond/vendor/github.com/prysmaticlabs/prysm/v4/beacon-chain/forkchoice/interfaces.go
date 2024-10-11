package forkchoice

import (
	"context"

	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	forkchoice2 "github.com/prysmaticlabs/prysm/v4/consensus-types/forkchoice"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// BalancesByRooter is a handler to obtain the effective balances of the state
// with the given block root
type BalancesByRooter func(context.Context, [32]byte) ([]uint64, error)

// ForkChoicer represents the full fork choice interface composed of all the sub-interfaces.
type ForkChoicer interface {
	RLocker // separate interface isolates  read locking for ROForkChoice.
	Lock()
	Unlock()
	HeadRetriever        // to compute head.
	BlockProcessor       // to track new block for fork choice.
	AttestationProcessor // to track new attestation for fork choice.
	Getter               // to retrieve fork choice information.
	Setter               // to set fork choice information.
}

// RLocker represents forkchoice's internal RWMutex read-only lock/unlock methods.
type RLocker interface {
	RLock()
	RUnlock()
}

// HeadRetriever retrieves head root and optimistic info of the current chain.
type HeadRetriever interface {
	Head(context.Context) ([32]byte, error)
	GetProposerHead() [32]byte
	CachedHeadRoot() [32]byte
}

// BlockProcessor processes the block that's used for accounting fork choice.
type BlockProcessor interface {
	InsertNode(context.Context, state.BeaconState, [32]byte) error
	InsertChain(context.Context, []*forkchoicetypes.BlockAndCheckpoints) error
}

// AttestationProcessor processes the attestation that's used for accounting fork choice.
type AttestationProcessor interface {
	ProcessAttestation(context.Context, []uint64, [32]byte, primitives.Epoch)
}

// Getter returns fork choice related information.
type Getter interface {
	FastGetter
	AncestorRoot(ctx context.Context, root [32]byte, slot primitives.Slot) ([32]byte, error)
	CommonAncestor(ctx context.Context, root1 [32]byte, root2 [32]byte) ([32]byte, primitives.Slot, error)
	ForkChoiceDump(context.Context) (*forkchoice2.Dump, error)
	Tips() ([][32]byte, []primitives.Slot)
}

type FastGetter interface {
	FinalizedCheckpoint() *forkchoicetypes.Checkpoint
	FinalizedPayloadBlockHash() [32]byte
	HasNode([32]byte) bool
	HighestReceivedBlockSlot() primitives.Slot
	HighestReceivedBlockDelay() primitives.Slot
	IsCanonical(root [32]byte) bool
	IsOptimistic(root [32]byte) (bool, error)
	IsViableForCheckpoint(*forkchoicetypes.Checkpoint) (bool, error)
	JustifiedCheckpoint() *forkchoicetypes.Checkpoint
	JustifiedPayloadBlockHash() [32]byte
	LastRoot(primitives.Epoch) [32]byte
	NodeCount() int
	PreviousJustifiedCheckpoint() *forkchoicetypes.Checkpoint
	ProposerBoost() [fieldparams.RootLength]byte
	ReceivedBlocksLastEpoch() (uint64, error)
	ShouldOverrideFCU() bool
	Slot([32]byte) (primitives.Slot, error)
	TargetRootForEpoch([32]byte, primitives.Epoch) ([32]byte, error)
	UnrealizedJustifiedPayloadBlockHash() [32]byte
	Weight(root [32]byte) (uint64, error)
}

// Setter allows to set forkchoice information
type Setter interface {
	SetOptimisticToValid(context.Context, [fieldparams.RootLength]byte) error
	SetOptimisticToInvalid(context.Context, [fieldparams.RootLength]byte, [fieldparams.RootLength]byte, [fieldparams.RootLength]byte) ([][32]byte, error)
	UpdateJustifiedCheckpoint(context.Context, *forkchoicetypes.Checkpoint) error
	UpdateFinalizedCheckpoint(*forkchoicetypes.Checkpoint) error
	SetGenesisTime(uint64)
	SetOriginRoot([32]byte)
	NewSlot(context.Context, primitives.Slot) error
	SetBalancesByRooter(BalancesByRooter)
	InsertSlashedIndex(context.Context, primitives.ValidatorIndex)
}
