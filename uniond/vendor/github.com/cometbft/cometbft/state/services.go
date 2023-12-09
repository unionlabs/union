package state

import (
	"github.com/cometbft/cometbft/types"
)

//------------------------------------------------------
// blockchain services types
// NOTE: Interfaces used by RPC must be thread safe!
//------------------------------------------------------

//------------------------------------------------------
// blockstore

//go:generate ../scripts/mockery_generate.sh BlockStore

// BlockStore defines the interface used by the ConsensusState.
type BlockStore interface {
	Base() int64
	Height() int64
	Size() int64

	LoadBaseMeta() *types.BlockMeta
	LoadBlockMeta(height int64) *types.BlockMeta
	LoadBlock(height int64) *types.Block

	SaveBlock(block *types.Block, blockParts *types.PartSet, seenCommit *types.Commit)
	SaveBlockWithExtendedCommit(block *types.Block, blockParts *types.PartSet, seenCommit *types.ExtendedCommit)

	PruneBlocks(height int64, state State) (uint64, int64, error)

	LoadBlockByHash(hash []byte) *types.Block
	LoadBlockMetaByHash(hash []byte) *types.BlockMeta
	LoadBlockPart(height int64, index int) *types.Part

	LoadBlockCommit(height int64) *types.Commit
	LoadSeenCommit(height int64) *types.Commit
	LoadBlockExtendedCommit(height int64) *types.ExtendedCommit

	DeleteLatestBlock() error

	Close() error
}

//-----------------------------------------------------------------------------
// evidence pool

//go:generate ../scripts/mockery_generate.sh EvidencePool

// EvidencePool defines the EvidencePool interface used by State.
type EvidencePool interface {
	PendingEvidence(maxBytes int64) (ev []types.Evidence, size int64)
	AddEvidence(types.Evidence) error
	Update(State, types.EvidenceList)
	CheckEvidence(types.EvidenceList) error
}

// EmptyEvidencePool is an empty implementation of EvidencePool, useful for testing. It also complies
// to the consensus evidence pool interface
type EmptyEvidencePool struct{}

func (EmptyEvidencePool) PendingEvidence(int64) (ev []types.Evidence, size int64) {
	return nil, 0
}
func (EmptyEvidencePool) AddEvidence(types.Evidence) error                { return nil }
func (EmptyEvidencePool) Update(State, types.EvidenceList)                {}
func (EmptyEvidencePool) CheckEvidence(types.EvidenceList) error          { return nil }
func (EmptyEvidencePool) ReportConflictingVotes(*types.Vote, *types.Vote) {}
