//go:build !minimal

package state_native

import (
	"encoding/json"
	"sync"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/fieldtrie"
	customtypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/custom-types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// BeaconState defines a struct containing utilities for the Ethereum Beacon Chain state, defining
// getters and setters for its respective values and helpful functions such as HashTreeRoot().
type BeaconState struct {
	version                             int
	genesisTime                         uint64
	genesisValidatorsRoot               [32]byte
	slot                                primitives.Slot
	fork                                *ethpb.Fork
	latestBlockHeader                   *ethpb.BeaconBlockHeader
	blockRoots                          customtypes.BlockRoots
	blockRootsMultiValue                *MultiValueBlockRoots
	stateRoots                          customtypes.StateRoots
	stateRootsMultiValue                *MultiValueStateRoots
	historicalRoots                     customtypes.HistoricalRoots
	historicalSummaries                 []*ethpb.HistoricalSummary
	eth1Data                            *ethpb.Eth1Data
	eth1DataVotes                       []*ethpb.Eth1Data
	eth1DepositIndex                    uint64
	validators                          []*ethpb.Validator
	validatorsMultiValue                *MultiValueValidators
	balances                            []uint64
	balancesMultiValue                  *MultiValueBalances
	randaoMixes                         customtypes.RandaoMixes
	randaoMixesMultiValue               *MultiValueRandaoMixes
	slashings                           []uint64
	previousEpochAttestations           []*ethpb.PendingAttestation
	currentEpochAttestations            []*ethpb.PendingAttestation
	previousEpochParticipation          []byte
	currentEpochParticipation           []byte
	justificationBits                   bitfield.Bitvector4
	previousJustifiedCheckpoint         *ethpb.Checkpoint
	currentJustifiedCheckpoint          *ethpb.Checkpoint
	finalizedCheckpoint                 *ethpb.Checkpoint
	inactivityScores                    []uint64
	inactivityScoresMultiValue          *MultiValueInactivityScores
	currentSyncCommittee                *ethpb.SyncCommittee
	nextSyncCommittee                   *ethpb.SyncCommittee
	latestExecutionPayloadHeader        *enginev1.ExecutionPayloadHeader
	latestExecutionPayloadHeaderCapella *enginev1.ExecutionPayloadHeaderCapella
	latestExecutionPayloadHeaderDeneb   *enginev1.ExecutionPayloadHeaderDeneb
	nextWithdrawalIndex                 uint64
	nextWithdrawalValidatorIndex        primitives.ValidatorIndex

	id                    uint64
	lock                  sync.RWMutex
	dirtyFields           map[types.FieldIndex]bool
	dirtyIndices          map[types.FieldIndex][]uint64
	stateFieldLeaves      map[types.FieldIndex]*fieldtrie.FieldTrie
	rebuildTrie           map[types.FieldIndex]bool
	valMapHandler         *stateutil.ValidatorMapHandler
	merkleLayers          [][][]byte
	sharedFieldReferences map[types.FieldIndex]*stateutil.Reference
}

type beaconStateMarshalable struct {
	Version                             int                                     `json:"version" yaml:"version"`
	GenesisTime                         uint64                                  `json:"genesis_time" yaml:"genesis_time"`
	GenesisValidatorsRoot               [32]byte                                `json:"genesis_validators_root" yaml:"genesis_validators_root"`
	Slot                                primitives.Slot                         `json:"slot" yaml:"slot"`
	Fork                                *ethpb.Fork                             `json:"fork" yaml:"fork"`
	LatestBlockHeader                   *ethpb.BeaconBlockHeader                `json:"latest_block_header" yaml:"latest_block_header"`
	BlockRoots                          customtypes.BlockRoots                  `json:"block_roots" yaml:"block_roots"`
	StateRoots                          customtypes.StateRoots                  `json:"state_roots" yaml:"state_roots"`
	HistoricalRoots                     customtypes.HistoricalRoots             `json:"historical_roots" yaml:"historical_roots"`
	HistoricalSummaries                 []*ethpb.HistoricalSummary              `json:"historical_summaries" yaml:"historical_summaries"`
	Eth1Data                            *ethpb.Eth1Data                         `json:"eth_1_data" yaml:"eth_1_data"`
	Eth1DataVotes                       []*ethpb.Eth1Data                       `json:"eth_1_data_votes" yaml:"eth_1_data_votes"`
	Eth1DepositIndex                    uint64                                  `json:"eth_1_deposit_index" yaml:"eth_1_deposit_index"`
	Validators                          []*ethpb.Validator                      `json:"validators" yaml:"validators"`
	Balances                            []uint64                                `json:"balances" yaml:"balances"`
	RandaoMixes                         customtypes.RandaoMixes                 `json:"randao_mixes" yaml:"randao_mixes"`
	Slashings                           []uint64                                `json:"slashings" yaml:"slashings"`
	PreviousEpochAttestations           []*ethpb.PendingAttestation             `json:"previous_epoch_attestations" yaml:"previous_epoch_attestations"`
	CurrentEpochAttestations            []*ethpb.PendingAttestation             `json:"current_epoch_attestations" yaml:"current_epoch_attestations"`
	PreviousEpochParticipation          []byte                                  `json:"previous_epoch_participation" yaml:"previous_epoch_participation"`
	CurrentEpochParticipation           []byte                                  `json:"current_epoch_participation" yaml:"current_epoch_participation"`
	JustificationBits                   bitfield.Bitvector4                     `json:"justification_bits" yaml:"justification_bits"`
	PreviousJustifiedCheckpoint         *ethpb.Checkpoint                       `json:"previous_justified_checkpoint" yaml:"previous_justified_checkpoint"`
	CurrentJustifiedCheckpoint          *ethpb.Checkpoint                       `json:"current_justified_checkpoint" yaml:"current_justified_checkpoint"`
	FinalizedCheckpoint                 *ethpb.Checkpoint                       `json:"finalized_checkpoint" yaml:"finalized_checkpoint"`
	InactivityScores                    []uint64                                `json:"inactivity_scores" yaml:"inactivity_scores"`
	CurrentSyncCommittee                *ethpb.SyncCommittee                    `json:"current_sync_committee" yaml:"current_sync_committee"`
	NextSyncCommittee                   *ethpb.SyncCommittee                    `json:"next_sync_committee" yaml:"next_sync_committee"`
	LatestExecutionPayloadHeader        *enginev1.ExecutionPayloadHeader        `json:"latest_execution_payload_header" yaml:"latest_execution_payload_header"`
	LatestExecutionPayloadHeaderCapella *enginev1.ExecutionPayloadHeaderCapella `json:"latest_execution_payload_header_capella" yaml:"latest_execution_payload_header_capella"`
	NextWithdrawalIndex                 uint64                                  `json:"next_withdrawal_index" yaml:"next_withdrawal_index"`
	NextWithdrawalValidatorIndex        primitives.ValidatorIndex               `json:"next_withdrawal_validator_index" yaml:"next_withdrawal_validator_index"`
}

func (b *BeaconState) MarshalJSON() ([]byte, error) {
	var bRoots customtypes.BlockRoots
	var sRoots customtypes.StateRoots
	var mixes customtypes.RandaoMixes
	var balances []uint64
	var inactivityScores []uint64
	var vals []*ethpb.Validator

	if features.Get().EnableExperimentalState {
		bRoots = b.blockRootsMultiValue.Value(b)
		sRoots = b.stateRootsMultiValue.Value(b)
		mixes = b.randaoMixesMultiValue.Value(b)
		balances = b.balancesMultiValue.Value(b)
		inactivityScores = b.inactivityScoresMultiValue.Value(b)
		vals = b.validatorsMultiValue.Value(b)
	} else {
		bRoots = b.blockRoots
		sRoots = b.stateRoots
		mixes = b.randaoMixes
		balances = b.balances
		inactivityScores = b.inactivityScores
		vals = b.validators
	}

	marshalable := &beaconStateMarshalable{
		Version:                             b.version,
		GenesisTime:                         b.genesisTime,
		GenesisValidatorsRoot:               b.genesisValidatorsRoot,
		Slot:                                b.slot,
		Fork:                                b.fork,
		LatestBlockHeader:                   b.latestBlockHeader,
		BlockRoots:                          bRoots,
		StateRoots:                          sRoots,
		HistoricalRoots:                     b.historicalRoots,
		HistoricalSummaries:                 b.historicalSummaries,
		Eth1Data:                            b.eth1Data,
		Eth1DataVotes:                       b.eth1DataVotes,
		Eth1DepositIndex:                    b.eth1DepositIndex,
		Validators:                          vals,
		Balances:                            balances,
		RandaoMixes:                         mixes,
		Slashings:                           b.slashings,
		PreviousEpochAttestations:           b.previousEpochAttestations,
		CurrentEpochAttestations:            b.currentEpochAttestations,
		PreviousEpochParticipation:          b.previousEpochParticipation,
		CurrentEpochParticipation:           b.currentEpochParticipation,
		JustificationBits:                   b.justificationBits,
		PreviousJustifiedCheckpoint:         b.previousJustifiedCheckpoint,
		CurrentJustifiedCheckpoint:          b.currentJustifiedCheckpoint,
		FinalizedCheckpoint:                 b.finalizedCheckpoint,
		InactivityScores:                    inactivityScores,
		CurrentSyncCommittee:                b.currentSyncCommittee,
		NextSyncCommittee:                   b.nextSyncCommittee,
		LatestExecutionPayloadHeader:        b.latestExecutionPayloadHeader,
		LatestExecutionPayloadHeaderCapella: b.latestExecutionPayloadHeaderCapella,
		NextWithdrawalIndex:                 b.nextWithdrawalIndex,
		NextWithdrawalValidatorIndex:        b.nextWithdrawalValidatorIndex,
	}
	return json.Marshal(marshalable)
}
