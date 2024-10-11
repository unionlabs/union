// Package iface defines the actual database interface used
// by a Prysm beacon node, also containing useful, scoped interfaces such as
// a ReadOnlyDatabase.
package iface

import (
	"context"
	"io"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filters"
	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/monitoring/backup"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ReadOnlyDatabase defines a struct which only has read access to database methods.
type ReadOnlyDatabase interface {
	// Block related methods.
	Block(ctx context.Context, blockRoot [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error)
	Blocks(ctx context.Context, f *filters.QueryFilter) ([]interfaces.ReadOnlySignedBeaconBlock, [][32]byte, error)
	BlockRoots(ctx context.Context, f *filters.QueryFilter) ([][32]byte, error)
	BlocksBySlot(ctx context.Context, slot primitives.Slot) ([]interfaces.ReadOnlySignedBeaconBlock, error)
	BlockRootsBySlot(ctx context.Context, slot primitives.Slot) (bool, [][32]byte, error)
	HasBlock(ctx context.Context, blockRoot [32]byte) bool
	GenesisBlock(ctx context.Context) (interfaces.ReadOnlySignedBeaconBlock, error)
	GenesisBlockRoot(ctx context.Context) ([32]byte, error)
	IsFinalizedBlock(ctx context.Context, blockRoot [32]byte) bool
	FinalizedChildBlock(ctx context.Context, blockRoot [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error)
	HighestRootsBelowSlot(ctx context.Context, slot primitives.Slot) (primitives.Slot, [][32]byte, error)
	// State related methods.
	State(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error)
	StateOrError(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error)
	GenesisState(ctx context.Context) (state.BeaconState, error)
	HasState(ctx context.Context, blockRoot [32]byte) bool
	StateSummary(ctx context.Context, blockRoot [32]byte) (*ethpb.StateSummary, error)
	HasStateSummary(ctx context.Context, blockRoot [32]byte) bool
	HighestSlotStatesBelow(ctx context.Context, slot primitives.Slot) ([]state.ReadOnlyBeaconState, error)
	// Checkpoint operations.
	JustifiedCheckpoint(ctx context.Context) (*ethpb.Checkpoint, error)
	FinalizedCheckpoint(ctx context.Context) (*ethpb.Checkpoint, error)
	ArchivedPointRoot(ctx context.Context, slot primitives.Slot) [32]byte
	HasArchivedPoint(ctx context.Context, slot primitives.Slot) bool
	LastArchivedRoot(ctx context.Context) [32]byte
	LastArchivedSlot(ctx context.Context) (primitives.Slot, error)
	LastValidatedCheckpoint(ctx context.Context) (*ethpb.Checkpoint, error)
	// Deposit contract related handlers.
	DepositContractAddress(ctx context.Context) ([]byte, error)
	// ExecutionChainData operations.
	ExecutionChainData(ctx context.Context) (*ethpb.ETH1ChainData, error)
	// Fee recipients operations.
	FeeRecipientByValidatorID(ctx context.Context, id primitives.ValidatorIndex) (common.Address, error)
	RegistrationByValidatorID(ctx context.Context, id primitives.ValidatorIndex) (*ethpb.ValidatorRegistrationV1, error)

	// origin checkpoint sync support
	OriginCheckpointBlockRoot(ctx context.Context) ([32]byte, error)
	BackfillStatus(context.Context) (*dbval.BackfillStatus, error)
}

// NoHeadAccessDatabase defines a struct without access to chain head data.
type NoHeadAccessDatabase interface {
	ReadOnlyDatabase

	// Block related methods.
	DeleteBlock(ctx context.Context, root [32]byte) error
	SaveBlock(ctx context.Context, block interfaces.ReadOnlySignedBeaconBlock) error
	SaveBlocks(ctx context.Context, blocks []interfaces.ReadOnlySignedBeaconBlock) error
	SaveROBlocks(ctx context.Context, blks []blocks.ROBlock, cache bool) error
	SaveGenesisBlockRoot(ctx context.Context, blockRoot [32]byte) error
	// State related methods.
	SaveState(ctx context.Context, state state.ReadOnlyBeaconState, blockRoot [32]byte) error
	SaveStates(ctx context.Context, states []state.ReadOnlyBeaconState, blockRoots [][32]byte) error
	DeleteState(ctx context.Context, blockRoot [32]byte) error
	DeleteStates(ctx context.Context, blockRoots [][32]byte) error
	SaveStateSummary(ctx context.Context, summary *ethpb.StateSummary) error
	SaveStateSummaries(ctx context.Context, summaries []*ethpb.StateSummary) error
	// Checkpoint operations.
	SaveJustifiedCheckpoint(ctx context.Context, checkpoint *ethpb.Checkpoint) error
	SaveFinalizedCheckpoint(ctx context.Context, checkpoint *ethpb.Checkpoint) error
	SaveLastValidatedCheckpoint(ctx context.Context, checkpoint *ethpb.Checkpoint) error
	// Deposit contract related handlers.
	SaveDepositContractAddress(ctx context.Context, addr common.Address) error
	// SaveExecutionChainData operations.
	SaveExecutionChainData(ctx context.Context, data *ethpb.ETH1ChainData) error
	// Run any required database migrations.
	RunMigrations(ctx context.Context) error
	// Fee recipients operations.
	SaveFeeRecipientsByValidatorIDs(ctx context.Context, ids []primitives.ValidatorIndex, addrs []common.Address) error
	SaveRegistrationsByValidatorIDs(ctx context.Context, ids []primitives.ValidatorIndex, regs []*ethpb.ValidatorRegistrationV1) error

	CleanUpDirtyStates(ctx context.Context, slotsPerArchivedPoint primitives.Slot) error
}

// HeadAccessDatabase defines a struct with access to reading chain head data.
type HeadAccessDatabase interface {
	NoHeadAccessDatabase

	// Block related methods.
	HeadBlock(ctx context.Context) (interfaces.ReadOnlySignedBeaconBlock, error)
	SaveHeadBlockRoot(ctx context.Context, blockRoot [32]byte) error

	// Genesis operations.
	LoadGenesis(ctx context.Context, stateBytes []byte) error
	SaveGenesisData(ctx context.Context, state state.BeaconState) error
	EnsureEmbeddedGenesis(ctx context.Context) error

	// Support for checkpoint sync and backfill.
	SaveOrigin(ctx context.Context, serState, serBlock []byte) error
	SaveBackfillStatus(context.Context, *dbval.BackfillStatus) error
	BackfillFinalizedIndex(ctx context.Context, blocks []blocks.ROBlock, finalizedChildRoot [32]byte) error
}

// SlasherDatabase interface for persisting data related to detecting slashable offenses on Ethereum.
type SlasherDatabase interface {
	io.Closer
	SaveLastEpochsWrittenForValidators(
		ctx context.Context, epochByValidator map[primitives.ValidatorIndex]primitives.Epoch,
	) error
	SaveAttestationRecordsForValidators(
		ctx context.Context,
		attestations []*slashertypes.IndexedAttestationWrapper,
	) error
	SaveSlasherChunks(
		ctx context.Context, kind slashertypes.ChunkKind, chunkKeys [][]byte, chunks [][]uint16,
	) error
	SaveBlockProposals(
		ctx context.Context, proposal []*slashertypes.SignedBlockHeaderWrapper,
	) error
	LastEpochWrittenForValidators(
		ctx context.Context, validatorIndices []primitives.ValidatorIndex,
	) ([]*slashertypes.AttestedEpochForValidator, error)
	AttestationRecordForValidator(
		ctx context.Context, validatorIdx primitives.ValidatorIndex, targetEpoch primitives.Epoch,
	) (*slashertypes.IndexedAttestationWrapper, error)
	BlockProposalForValidator(
		ctx context.Context, validatorIdx primitives.ValidatorIndex, slot primitives.Slot,
	) (*slashertypes.SignedBlockHeaderWrapper, error)
	CheckAttesterDoubleVotes(
		ctx context.Context, attestations []*slashertypes.IndexedAttestationWrapper,
	) ([]*slashertypes.AttesterDoubleVote, error)
	LoadSlasherChunks(
		ctx context.Context, kind slashertypes.ChunkKind, diskKeys [][]byte,
	) ([][]uint16, []bool, error)
	CheckDoubleBlockProposals(
		ctx context.Context, proposals []*slashertypes.SignedBlockHeaderWrapper,
	) ([]*ethpb.ProposerSlashing, error)
	PruneAttestationsAtEpoch(
		ctx context.Context, maxEpoch primitives.Epoch,
	) (numPruned uint, err error)
	PruneProposalsAtEpoch(
		ctx context.Context, maxEpoch primitives.Epoch,
	) (numPruned uint, err error)
	HighestAttestations(
		ctx context.Context,
		indices []primitives.ValidatorIndex,
	) ([]*ethpb.HighestAttestation, error)
	DatabasePath() string
	ClearDB() error
}

// Database interface with full access.
type Database interface {
	io.Closer
	backup.Exporter
	HeadAccessDatabase

	DatabasePath() string
	ClearDB() error
}
