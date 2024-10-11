// Package state defines the actual beacon state interface used
// by a Prysm beacon node, also containing useful, scoped interfaces such as
// a ReadOnlyState and WriteOnlyBeaconState.
package state

import (
	"context"
	"encoding/json"

	"github.com/prysmaticlabs/go-bitfield"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// BeaconState has read and write access to beacon state methods.
type BeaconState interface {
	SpecParametersProvider
	ReadOnlyBeaconState
	WriteOnlyBeaconState
	Copy() BeaconState
	CopyAllTries()
	Defragment()
	HashTreeRoot(ctx context.Context) ([32]byte, error)
	Prover
	json.Marshaler
}

// SpecParametersProvider provides fork-specific configuration parameters as
// defined in the consensus specification for the beacon chain.
type SpecParametersProvider interface {
	InactivityPenaltyQuotient() (uint64, error)
	ProportionalSlashingMultiplier() (uint64, error)
}

// StateProver defines the ability to create Merkle proofs for beacon state fields.
type Prover interface {
	FinalizedRootProof(ctx context.Context) ([][]byte, error)
	CurrentSyncCommitteeProof(ctx context.Context) ([][]byte, error)
	NextSyncCommitteeProof(ctx context.Context) ([][]byte, error)
}

// ReadOnlyBeaconState defines a struct which only has read access to beacon state methods.
type ReadOnlyBeaconState interface {
	ReadOnlyBlockRoots
	ReadOnlyStateRoots
	ReadOnlyRandaoMixes
	ReadOnlyEth1Data
	ReadOnlyValidators
	ReadOnlyBalances
	ReadOnlyCheckpoint
	ReadOnlyAttestations
	ReadOnlyWithdrawals
	ReadOnlyParticipation
	ReadOnlyInactivity
	ReadOnlySyncCommittee
	ToProtoUnsafe() interface{}
	ToProto() interface{}
	GenesisTime() uint64
	GenesisValidatorsRoot() []byte
	Slot() primitives.Slot
	Fork() *ethpb.Fork
	LatestBlockHeader() *ethpb.BeaconBlockHeader
	HistoricalRoots() ([][]byte, error)
	HistoricalSummaries() ([]*ethpb.HistoricalSummary, error)
	Slashings() []uint64
	FieldReferencesCount() map[string]uint64
	RecordStateMetrics()
	MarshalSSZ() ([]byte, error)
	IsNil() bool
	Version() int
	LatestExecutionPayloadHeader() (interfaces.ExecutionData, error)
}

// WriteOnlyBeaconState defines a struct which only has write access to beacon state methods.
type WriteOnlyBeaconState interface {
	WriteOnlyBlockRoots
	WriteOnlyStateRoots
	WriteOnlyRandaoMixes
	WriteOnlyEth1Data
	WriteOnlyValidators
	WriteOnlyBalances
	WriteOnlyCheckpoint
	WriteOnlyAttestations
	WriteOnlyParticipation
	WriteOnlyInactivity
	WriteOnlySyncCommittee
	SetGenesisTime(val uint64) error
	SetGenesisValidatorsRoot(val []byte) error
	SetSlot(val primitives.Slot) error
	SetFork(val *ethpb.Fork) error
	SetLatestBlockHeader(val *ethpb.BeaconBlockHeader) error
	SetHistoricalRoots(val [][]byte) error
	SetSlashings(val []uint64) error
	UpdateSlashingsAtIndex(idx, val uint64) error
	AppendHistoricalRoots(root [32]byte) error
	AppendHistoricalSummaries(*ethpb.HistoricalSummary) error
	SetLatestExecutionPayloadHeader(payload interfaces.ExecutionData) error
	SetNextWithdrawalIndex(i uint64) error
	SetNextWithdrawalValidatorIndex(i primitives.ValidatorIndex) error
}

// ReadOnlyValidator defines a struct which only has read access to validator methods.
type ReadOnlyValidator interface {
	EffectiveBalance() uint64
	ActivationEligibilityEpoch() primitives.Epoch
	ActivationEpoch() primitives.Epoch
	WithdrawableEpoch() primitives.Epoch
	ExitEpoch() primitives.Epoch
	PublicKey() [fieldparams.BLSPubkeyLength]byte
	WithdrawalCredentials() []byte
	Slashed() bool
	IsNil() bool
}

// ReadOnlyValidators defines a struct which only has read access to validators methods.
type ReadOnlyValidators interface {
	Validators() []*ethpb.Validator
	ValidatorAtIndex(idx primitives.ValidatorIndex) (*ethpb.Validator, error)
	ValidatorAtIndexReadOnly(idx primitives.ValidatorIndex) (ReadOnlyValidator, error)
	ValidatorIndexByPubkey(key [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool)
	PublicKeys() ([][fieldparams.BLSPubkeyLength]byte, error)
	PubkeyAtIndex(idx primitives.ValidatorIndex) [fieldparams.BLSPubkeyLength]byte
	NumValidators() int
	ReadFromEveryValidator(f func(idx int, val ReadOnlyValidator) error) error
}

// ReadOnlyBalances defines a struct which only has read access to balances methods.
type ReadOnlyBalances interface {
	Balances() []uint64
	BalanceAtIndex(idx primitives.ValidatorIndex) (uint64, error)
	BalancesLength() int
}

// ReadOnlyCheckpoint defines a struct which only has read access to checkpoint methods.
type ReadOnlyCheckpoint interface {
	PreviousJustifiedCheckpoint() *ethpb.Checkpoint
	CurrentJustifiedCheckpoint() *ethpb.Checkpoint
	MatchCurrentJustifiedCheckpoint(c *ethpb.Checkpoint) bool
	MatchPreviousJustifiedCheckpoint(c *ethpb.Checkpoint) bool
	FinalizedCheckpoint() *ethpb.Checkpoint
	FinalizedCheckpointEpoch() primitives.Epoch
	JustificationBits() bitfield.Bitvector4
	UnrealizedCheckpointBalances() (uint64, uint64, uint64, error)
}

// ReadOnlyBlockRoots defines a struct which only has read access to block roots methods.
type ReadOnlyBlockRoots interface {
	BlockRoots() [][]byte
	BlockRootAtIndex(idx uint64) ([]byte, error)
}

// ReadOnlyStateRoots defines a struct which only has read access to state roots methods.
type ReadOnlyStateRoots interface {
	StateRoots() [][]byte
	StateRootAtIndex(idx uint64) ([]byte, error)
}

// ReadOnlyRandaoMixes defines a struct which only has read access to randao mixes methods.
type ReadOnlyRandaoMixes interface {
	RandaoMixes() [][]byte
	RandaoMixAtIndex(idx uint64) ([]byte, error)
	RandaoMixesLength() int
}

// ReadOnlyEth1Data defines a struct which only has read access to eth1 data methods.
type ReadOnlyEth1Data interface {
	Eth1Data() *ethpb.Eth1Data
	Eth1DataVotes() []*ethpb.Eth1Data
	Eth1DepositIndex() uint64
}

// ReadOnlyAttestations defines a struct which only has read access to attestations methods.
type ReadOnlyAttestations interface {
	PreviousEpochAttestations() ([]*ethpb.PendingAttestation, error)
	CurrentEpochAttestations() ([]*ethpb.PendingAttestation, error)
}

// ReadOnlyWithdrawals defines a struct which only has read access to withdrawal methods.
type ReadOnlyWithdrawals interface {
	ExpectedWithdrawals() ([]*enginev1.Withdrawal, error)
	NextWithdrawalValidatorIndex() (primitives.ValidatorIndex, error)
	NextWithdrawalIndex() (uint64, error)
}

// ReadOnlyParticipation defines a struct which only has read access to participation methods.
type ReadOnlyParticipation interface {
	CurrentEpochParticipation() ([]byte, error)
	PreviousEpochParticipation() ([]byte, error)
}

// ReadOnlyInactivity defines a struct which only has read access to inactivity methods.
type ReadOnlyInactivity interface {
	InactivityScores() ([]uint64, error)
}

// ReadOnlySyncCommittee defines a struct which only has read access to sync committee methods.
type ReadOnlySyncCommittee interface {
	CurrentSyncCommittee() (*ethpb.SyncCommittee, error)
	NextSyncCommittee() (*ethpb.SyncCommittee, error)
}

// WriteOnlyBlockRoots defines a struct which only has write access to block roots methods.
type WriteOnlyBlockRoots interface {
	SetBlockRoots(val [][]byte) error
	UpdateBlockRootAtIndex(idx uint64, blockRoot [32]byte) error
}

// WriteOnlyStateRoots defines a struct which only has write access to state roots methods.
type WriteOnlyStateRoots interface {
	SetStateRoots(val [][]byte) error
	UpdateStateRootAtIndex(idx uint64, stateRoot [32]byte) error
}

// WriteOnlyEth1Data defines a struct which only has write access to eth1 data methods.
type WriteOnlyEth1Data interface {
	SetEth1Data(val *ethpb.Eth1Data) error
	SetEth1DataVotes(val []*ethpb.Eth1Data) error
	AppendEth1DataVotes(val *ethpb.Eth1Data) error
	SetEth1DepositIndex(val uint64) error
}

// WriteOnlyValidators defines a struct which only has write access to validators methods.
type WriteOnlyValidators interface {
	SetValidators(val []*ethpb.Validator) error
	ApplyToEveryValidator(f func(idx int, val *ethpb.Validator) (bool, *ethpb.Validator, error)) error
	UpdateValidatorAtIndex(idx primitives.ValidatorIndex, val *ethpb.Validator) error
	AppendValidator(val *ethpb.Validator) error
}

// WriteOnlyBalances defines a struct which only has write access to balances methods.
type WriteOnlyBalances interface {
	SetBalances(val []uint64) error
	UpdateBalancesAtIndex(idx primitives.ValidatorIndex, val uint64) error
	AppendBalance(bal uint64) error
}

// WriteOnlyRandaoMixes defines a struct which only has write access to randao mixes methods.
type WriteOnlyRandaoMixes interface {
	SetRandaoMixes(val [][]byte) error
	UpdateRandaoMixesAtIndex(idx uint64, val [32]byte) error
}

// WriteOnlyCheckpoint defines a struct which only has write access to check point methods.
type WriteOnlyCheckpoint interface {
	SetFinalizedCheckpoint(val *ethpb.Checkpoint) error
	SetPreviousJustifiedCheckpoint(val *ethpb.Checkpoint) error
	SetCurrentJustifiedCheckpoint(val *ethpb.Checkpoint) error
	SetJustificationBits(val bitfield.Bitvector4) error
}

// WriteOnlyAttestations defines a struct which only has write access to attestations methods.
type WriteOnlyAttestations interface {
	AppendCurrentEpochAttestations(val *ethpb.PendingAttestation) error
	AppendPreviousEpochAttestations(val *ethpb.PendingAttestation) error
	SetPreviousEpochAttestations([]*ethpb.PendingAttestation) error
	SetCurrentEpochAttestations([]*ethpb.PendingAttestation) error
	RotateAttestations() error
}

// WriteOnlyParticipation defines a struct which only has write access to participation methods.
type WriteOnlyParticipation interface {
	AppendCurrentParticipationBits(val byte) error
	AppendPreviousParticipationBits(val byte) error
	SetPreviousParticipationBits(val []byte) error
	SetCurrentParticipationBits(val []byte) error
	ModifyCurrentParticipationBits(func(val []byte) ([]byte, error)) error
	ModifyPreviousParticipationBits(func(val []byte) ([]byte, error)) error
}

// WriteOnlyInactivity defines a struct which only has write access to inactivity methods.
type WriteOnlyInactivity interface {
	AppendInactivityScore(s uint64) error
	SetInactivityScores(val []uint64) error
}

// WriteOnlySyncCommittee defines a struct which only has write access to sync committee methods.
type WriteOnlySyncCommittee interface {
	SetCurrentSyncCommittee(val *ethpb.SyncCommittee) error
	SetNextSyncCommittee(val *ethpb.SyncCommittee) error
}
