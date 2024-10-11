package state_native

import (
	"context"
	"runtime"
	"sort"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/fieldtrie"
	customtypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/custom-types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	mvslice "github.com/prysmaticlabs/prysm/v4/container/multi-value-slice"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/proto"
)

var phase0Fields = []types.FieldIndex{
	types.GenesisTime,
	types.GenesisValidatorsRoot,
	types.Slot,
	types.Fork,
	types.LatestBlockHeader,
	types.BlockRoots,
	types.StateRoots,
	types.HistoricalRoots,
	types.Eth1Data,
	types.Eth1DataVotes,
	types.Eth1DepositIndex,
	types.Validators,
	types.Balances,
	types.RandaoMixes,
	types.Slashings,
	types.PreviousEpochAttestations,
	types.CurrentEpochAttestations,
	types.JustificationBits,
	types.PreviousJustifiedCheckpoint,
	types.CurrentJustifiedCheckpoint,
	types.FinalizedCheckpoint,
}

var altairFields = []types.FieldIndex{
	types.GenesisTime,
	types.GenesisValidatorsRoot,
	types.Slot,
	types.Fork,
	types.LatestBlockHeader,
	types.BlockRoots,
	types.StateRoots,
	types.HistoricalRoots,
	types.Eth1Data,
	types.Eth1DataVotes,
	types.Eth1DepositIndex,
	types.Validators,
	types.Balances,
	types.RandaoMixes,
	types.Slashings,
	types.PreviousEpochParticipationBits,
	types.CurrentEpochParticipationBits,
	types.JustificationBits,
	types.PreviousJustifiedCheckpoint,
	types.CurrentJustifiedCheckpoint,
	types.FinalizedCheckpoint,
	types.InactivityScores,
	types.CurrentSyncCommittee,
	types.NextSyncCommittee,
}

var bellatrixFields = append(altairFields, types.LatestExecutionPayloadHeader)

var capellaFields = append(
	altairFields,
	types.LatestExecutionPayloadHeaderCapella,
	types.NextWithdrawalIndex,
	types.NextWithdrawalValidatorIndex,
	types.HistoricalSummaries,
)

var denebFields = append(
	altairFields,
	types.LatestExecutionPayloadHeaderDeneb,
	types.NextWithdrawalIndex,
	types.NextWithdrawalValidatorIndex,
	types.HistoricalSummaries,
)

const (
	phase0SharedFieldRefCount                     = 10
	altairSharedFieldRefCount                     = 11
	bellatrixSharedFieldRefCount                  = 12
	capellaSharedFieldRefCount                    = 14
	denebSharedFieldRefCount                      = 14
	experimentalStatePhase0SharedFieldRefCount    = 5
	experimentalStateAltairSharedFieldRefCount    = 5
	experimentalStateBellatrixSharedFieldRefCount = 6
	experimentalStateCapellaSharedFieldRefCount   = 8
	experimentalStateDenebSharedFieldRefCount     = 8
)

// InitializeFromProtoPhase0 the beacon state from a protobuf representation.
func InitializeFromProtoPhase0(st *ethpb.BeaconState) (state.BeaconState, error) {
	return InitializeFromProtoUnsafePhase0(proto.Clone(st).(*ethpb.BeaconState))
}

// InitializeFromProtoAltair the beacon state from a protobuf representation.
func InitializeFromProtoAltair(st *ethpb.BeaconStateAltair) (state.BeaconState, error) {
	return InitializeFromProtoUnsafeAltair(proto.Clone(st).(*ethpb.BeaconStateAltair))
}

// InitializeFromProtoBellatrix the beacon state from a protobuf representation.
func InitializeFromProtoBellatrix(st *ethpb.BeaconStateBellatrix) (state.BeaconState, error) {
	return InitializeFromProtoUnsafeBellatrix(proto.Clone(st).(*ethpb.BeaconStateBellatrix))
}

// InitializeFromProtoCapella the beacon state from a protobuf representation.
func InitializeFromProtoCapella(st *ethpb.BeaconStateCapella) (state.BeaconState, error) {
	return InitializeFromProtoUnsafeCapella(proto.Clone(st).(*ethpb.BeaconStateCapella))
}

// InitializeFromProtoDeneb the beacon state from a protobuf representation.
func InitializeFromProtoDeneb(st *ethpb.BeaconStateDeneb) (state.BeaconState, error) {
	return InitializeFromProtoUnsafeDeneb(proto.Clone(st).(*ethpb.BeaconStateDeneb))
}

// InitializeFromProtoUnsafePhase0 directly uses the beacon state protobuf fields
// and sets them as fields of the BeaconState type.
func InitializeFromProtoUnsafePhase0(st *ethpb.BeaconState) (state.BeaconState, error) {
	if st == nil {
		return nil, errors.New("received nil state")
	}

	hRoots := customtypes.HistoricalRoots(make([][32]byte, len(st.HistoricalRoots)))
	for i, r := range st.HistoricalRoots {
		copy(hRoots[i][:], r)
	}

	fieldCount := params.BeaconConfig().BeaconStateFieldCount
	b := &BeaconState{
		version:                     version.Phase0,
		genesisTime:                 st.GenesisTime,
		genesisValidatorsRoot:       bytesutil.ToBytes32(st.GenesisValidatorsRoot),
		slot:                        st.Slot,
		fork:                        st.Fork,
		latestBlockHeader:           st.LatestBlockHeader,
		historicalRoots:             hRoots,
		eth1Data:                    st.Eth1Data,
		eth1DataVotes:               st.Eth1DataVotes,
		eth1DepositIndex:            st.Eth1DepositIndex,
		slashings:                   st.Slashings,
		previousEpochAttestations:   st.PreviousEpochAttestations,
		currentEpochAttestations:    st.CurrentEpochAttestations,
		justificationBits:           st.JustificationBits,
		previousJustifiedCheckpoint: st.PreviousJustifiedCheckpoint,
		currentJustifiedCheckpoint:  st.CurrentJustifiedCheckpoint,
		finalizedCheckpoint:         st.FinalizedCheckpoint,

		id: types.Enumerator.Inc(),

		dirtyFields:      make(map[types.FieldIndex]bool, fieldCount),
		dirtyIndices:     make(map[types.FieldIndex][]uint64, fieldCount),
		stateFieldLeaves: make(map[types.FieldIndex]*fieldtrie.FieldTrie, fieldCount),
		rebuildTrie:      make(map[types.FieldIndex]bool, fieldCount),
		valMapHandler:    stateutil.NewValMapHandler(st.Validators),
	}

	if features.Get().EnableExperimentalState {
		b.blockRootsMultiValue = NewMultiValueBlockRoots(st.BlockRoots)
		b.stateRootsMultiValue = NewMultiValueStateRoots(st.StateRoots)
		b.randaoMixesMultiValue = NewMultiValueRandaoMixes(st.RandaoMixes)
		b.balancesMultiValue = NewMultiValueBalances(st.Balances)
		b.validatorsMultiValue = NewMultiValueValidators(st.Validators)
		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStatePhase0SharedFieldRefCount)
	} else {
		bRoots := make([][32]byte, fieldparams.BlockRootsLength)
		for i, r := range st.BlockRoots {
			bRoots[i] = bytesutil.ToBytes32(r)
		}
		b.blockRoots = bRoots

		sRoots := make([][32]byte, fieldparams.StateRootsLength)
		for i, r := range st.StateRoots {
			sRoots[i] = bytesutil.ToBytes32(r)
		}
		b.stateRoots = sRoots

		mixes := make([][32]byte, fieldparams.RandaoMixesLength)
		for i, m := range st.RandaoMixes {
			mixes[i] = bytesutil.ToBytes32(m)
		}
		b.randaoMixes = mixes

		b.balances = st.Balances
		b.validators = st.Validators

		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, phase0SharedFieldRefCount)
	}

	for _, f := range phase0Fields {
		b.dirtyFields[f] = true
		b.rebuildTrie[f] = true
		b.dirtyIndices[f] = []uint64{}
		trie, err := fieldtrie.NewFieldTrie(f, types.BasicArray, nil, 0)
		if err != nil {
			return nil, err
		}
		b.stateFieldLeaves[f] = trie
	}

	// Initialize field reference tracking for shared data.
	b.sharedFieldReferences[types.HistoricalRoots] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Eth1DataVotes] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Slashings] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.PreviousEpochAttestations] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.CurrentEpochAttestations] = stateutil.NewRef(1)
	if !features.Get().EnableExperimentalState {
		b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Balances] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Validators] = stateutil.NewRef(1)
	}

	state.Count.Inc()
	// Finalizer runs when dst is being destroyed in garbage collection.
	runtime.SetFinalizer(b, finalizerCleanup)
	return b, nil
}

// InitializeFromProtoUnsafeAltair directly uses the beacon state protobuf fields
// and sets them as fields of the BeaconState type.
func InitializeFromProtoUnsafeAltair(st *ethpb.BeaconStateAltair) (state.BeaconState, error) {
	if st == nil {
		return nil, errors.New("received nil state")
	}

	hRoots := customtypes.HistoricalRoots(make([][32]byte, len(st.HistoricalRoots)))
	for i, r := range st.HistoricalRoots {
		hRoots[i] = bytesutil.ToBytes32(r)
	}

	fieldCount := params.BeaconConfig().BeaconStateAltairFieldCount
	b := &BeaconState{
		version:                     version.Altair,
		genesisTime:                 st.GenesisTime,
		genesisValidatorsRoot:       bytesutil.ToBytes32(st.GenesisValidatorsRoot),
		slot:                        st.Slot,
		fork:                        st.Fork,
		latestBlockHeader:           st.LatestBlockHeader,
		historicalRoots:             hRoots,
		eth1Data:                    st.Eth1Data,
		eth1DataVotes:               st.Eth1DataVotes,
		eth1DepositIndex:            st.Eth1DepositIndex,
		slashings:                   st.Slashings,
		previousEpochParticipation:  st.PreviousEpochParticipation,
		currentEpochParticipation:   st.CurrentEpochParticipation,
		justificationBits:           st.JustificationBits,
		previousJustifiedCheckpoint: st.PreviousJustifiedCheckpoint,
		currentJustifiedCheckpoint:  st.CurrentJustifiedCheckpoint,
		finalizedCheckpoint:         st.FinalizedCheckpoint,
		currentSyncCommittee:        st.CurrentSyncCommittee,
		nextSyncCommittee:           st.NextSyncCommittee,

		id: types.Enumerator.Inc(),

		dirtyFields:      make(map[types.FieldIndex]bool, fieldCount),
		dirtyIndices:     make(map[types.FieldIndex][]uint64, fieldCount),
		stateFieldLeaves: make(map[types.FieldIndex]*fieldtrie.FieldTrie, fieldCount),
		rebuildTrie:      make(map[types.FieldIndex]bool, fieldCount),
		valMapHandler:    stateutil.NewValMapHandler(st.Validators),
	}

	if features.Get().EnableExperimentalState {
		b.blockRootsMultiValue = NewMultiValueBlockRoots(st.BlockRoots)
		b.stateRootsMultiValue = NewMultiValueStateRoots(st.StateRoots)
		b.randaoMixesMultiValue = NewMultiValueRandaoMixes(st.RandaoMixes)
		b.balancesMultiValue = NewMultiValueBalances(st.Balances)
		b.validatorsMultiValue = NewMultiValueValidators(st.Validators)
		b.inactivityScoresMultiValue = NewMultiValueInactivityScores(st.InactivityScores)
		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateAltairSharedFieldRefCount)
	} else {
		bRoots := make([][32]byte, fieldparams.BlockRootsLength)
		for i, r := range st.BlockRoots {
			bRoots[i] = bytesutil.ToBytes32(r)
		}
		b.blockRoots = bRoots

		sRoots := make([][32]byte, fieldparams.StateRootsLength)
		for i, r := range st.StateRoots {
			sRoots[i] = bytesutil.ToBytes32(r)
		}
		b.stateRoots = sRoots

		mixes := make([][32]byte, fieldparams.RandaoMixesLength)
		for i, m := range st.RandaoMixes {
			mixes[i] = bytesutil.ToBytes32(m)
		}
		b.randaoMixes = mixes

		b.balances = st.Balances
		b.validators = st.Validators
		b.inactivityScores = st.InactivityScores

		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, altairSharedFieldRefCount)
	}

	for _, f := range altairFields {
		b.dirtyFields[f] = true
		b.rebuildTrie[f] = true
		b.dirtyIndices[f] = []uint64{}
		trie, err := fieldtrie.NewFieldTrie(f, types.BasicArray, nil, 0)
		if err != nil {
			return nil, err
		}
		b.stateFieldLeaves[f] = trie
	}

	// Initialize field reference tracking for shared data.
	b.sharedFieldReferences[types.HistoricalRoots] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Eth1DataVotes] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Slashings] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.PreviousEpochParticipationBits] = stateutil.NewRef(1) // New in Altair.
	b.sharedFieldReferences[types.CurrentEpochParticipationBits] = stateutil.NewRef(1)  // New in Altair.
	if !features.Get().EnableExperimentalState {
		b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Balances] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Validators] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.InactivityScores] = stateutil.NewRef(1)
	}

	state.Count.Inc()
	// Finalizer runs when dst is being destroyed in garbage collection.
	runtime.SetFinalizer(b, finalizerCleanup)
	return b, nil
}

// InitializeFromProtoUnsafeBellatrix directly uses the beacon state protobuf fields
// and sets them as fields of the BeaconState type.
func InitializeFromProtoUnsafeBellatrix(st *ethpb.BeaconStateBellatrix) (state.BeaconState, error) {
	if st == nil {
		return nil, errors.New("received nil state")
	}

	hRoots := customtypes.HistoricalRoots(make([][32]byte, len(st.HistoricalRoots)))
	for i, r := range st.HistoricalRoots {
		hRoots[i] = bytesutil.ToBytes32(r)
	}

	fieldCount := params.BeaconConfig().BeaconStateBellatrixFieldCount
	b := &BeaconState{
		version:                      version.Bellatrix,
		genesisTime:                  st.GenesisTime,
		genesisValidatorsRoot:        bytesutil.ToBytes32(st.GenesisValidatorsRoot),
		slot:                         st.Slot,
		fork:                         st.Fork,
		latestBlockHeader:            st.LatestBlockHeader,
		historicalRoots:              hRoots,
		eth1Data:                     st.Eth1Data,
		eth1DataVotes:                st.Eth1DataVotes,
		eth1DepositIndex:             st.Eth1DepositIndex,
		slashings:                    st.Slashings,
		previousEpochParticipation:   st.PreviousEpochParticipation,
		currentEpochParticipation:    st.CurrentEpochParticipation,
		justificationBits:            st.JustificationBits,
		previousJustifiedCheckpoint:  st.PreviousJustifiedCheckpoint,
		currentJustifiedCheckpoint:   st.CurrentJustifiedCheckpoint,
		finalizedCheckpoint:          st.FinalizedCheckpoint,
		currentSyncCommittee:         st.CurrentSyncCommittee,
		nextSyncCommittee:            st.NextSyncCommittee,
		latestExecutionPayloadHeader: st.LatestExecutionPayloadHeader,

		id: types.Enumerator.Inc(),

		dirtyFields:      make(map[types.FieldIndex]bool, fieldCount),
		dirtyIndices:     make(map[types.FieldIndex][]uint64, fieldCount),
		stateFieldLeaves: make(map[types.FieldIndex]*fieldtrie.FieldTrie, fieldCount),
		rebuildTrie:      make(map[types.FieldIndex]bool, fieldCount),
		valMapHandler:    stateutil.NewValMapHandler(st.Validators),
	}

	if features.Get().EnableExperimentalState {
		b.blockRootsMultiValue = NewMultiValueBlockRoots(st.BlockRoots)
		b.stateRootsMultiValue = NewMultiValueStateRoots(st.StateRoots)
		b.randaoMixesMultiValue = NewMultiValueRandaoMixes(st.RandaoMixes)
		b.balancesMultiValue = NewMultiValueBalances(st.Balances)
		b.validatorsMultiValue = NewMultiValueValidators(st.Validators)
		b.inactivityScoresMultiValue = NewMultiValueInactivityScores(st.InactivityScores)
		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateBellatrixSharedFieldRefCount)
	} else {
		bRoots := make([][32]byte, fieldparams.BlockRootsLength)
		for i, r := range st.BlockRoots {
			bRoots[i] = bytesutil.ToBytes32(r)
		}
		b.blockRoots = bRoots

		sRoots := make([][32]byte, fieldparams.StateRootsLength)
		for i, r := range st.StateRoots {
			sRoots[i] = bytesutil.ToBytes32(r)
		}
		b.stateRoots = sRoots

		mixes := make([][32]byte, fieldparams.RandaoMixesLength)
		for i, m := range st.RandaoMixes {
			mixes[i] = bytesutil.ToBytes32(m)
		}
		b.randaoMixes = mixes

		b.balances = st.Balances
		b.validators = st.Validators
		b.inactivityScores = st.InactivityScores

		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, bellatrixSharedFieldRefCount)
	}

	for _, f := range bellatrixFields {
		b.dirtyFields[f] = true
		b.rebuildTrie[f] = true
		b.dirtyIndices[f] = []uint64{}
		trie, err := fieldtrie.NewFieldTrie(f, types.BasicArray, nil, 0)
		if err != nil {
			return nil, err
		}
		b.stateFieldLeaves[f] = trie
	}

	// Initialize field reference tracking for shared data.
	b.sharedFieldReferences[types.HistoricalRoots] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Eth1DataVotes] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Slashings] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.PreviousEpochParticipationBits] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.CurrentEpochParticipationBits] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.LatestExecutionPayloadHeader] = stateutil.NewRef(1) // New in Bellatrix.
	if !features.Get().EnableExperimentalState {
		b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Balances] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Validators] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.InactivityScores] = stateutil.NewRef(1)
	}

	state.Count.Inc()
	// Finalizer runs when dst is being destroyed in garbage collection.
	runtime.SetFinalizer(b, finalizerCleanup)
	return b, nil
}

// InitializeFromProtoUnsafeCapella directly uses the beacon state protobuf fields
// and sets them as fields of the BeaconState type.
func InitializeFromProtoUnsafeCapella(st *ethpb.BeaconStateCapella) (state.BeaconState, error) {
	if st == nil {
		return nil, errors.New("received nil state")
	}

	hRoots := customtypes.HistoricalRoots(make([][32]byte, len(st.HistoricalRoots)))
	for i, r := range st.HistoricalRoots {
		hRoots[i] = bytesutil.ToBytes32(r)
	}

	fieldCount := params.BeaconConfig().BeaconStateCapellaFieldCount
	b := &BeaconState{
		version:                             version.Capella,
		genesisTime:                         st.GenesisTime,
		genesisValidatorsRoot:               bytesutil.ToBytes32(st.GenesisValidatorsRoot),
		slot:                                st.Slot,
		fork:                                st.Fork,
		latestBlockHeader:                   st.LatestBlockHeader,
		historicalRoots:                     hRoots,
		eth1Data:                            st.Eth1Data,
		eth1DataVotes:                       st.Eth1DataVotes,
		eth1DepositIndex:                    st.Eth1DepositIndex,
		slashings:                           st.Slashings,
		previousEpochParticipation:          st.PreviousEpochParticipation,
		currentEpochParticipation:           st.CurrentEpochParticipation,
		justificationBits:                   st.JustificationBits,
		previousJustifiedCheckpoint:         st.PreviousJustifiedCheckpoint,
		currentJustifiedCheckpoint:          st.CurrentJustifiedCheckpoint,
		finalizedCheckpoint:                 st.FinalizedCheckpoint,
		currentSyncCommittee:                st.CurrentSyncCommittee,
		nextSyncCommittee:                   st.NextSyncCommittee,
		latestExecutionPayloadHeaderCapella: st.LatestExecutionPayloadHeader,
		nextWithdrawalIndex:                 st.NextWithdrawalIndex,
		nextWithdrawalValidatorIndex:        st.NextWithdrawalValidatorIndex,
		historicalSummaries:                 st.HistoricalSummaries,

		id: types.Enumerator.Inc(),

		dirtyFields:      make(map[types.FieldIndex]bool, fieldCount),
		dirtyIndices:     make(map[types.FieldIndex][]uint64, fieldCount),
		stateFieldLeaves: make(map[types.FieldIndex]*fieldtrie.FieldTrie, fieldCount),
		rebuildTrie:      make(map[types.FieldIndex]bool, fieldCount),
		valMapHandler:    stateutil.NewValMapHandler(st.Validators),
	}

	if features.Get().EnableExperimentalState {
		b.blockRootsMultiValue = NewMultiValueBlockRoots(st.BlockRoots)
		b.stateRootsMultiValue = NewMultiValueStateRoots(st.StateRoots)
		b.randaoMixesMultiValue = NewMultiValueRandaoMixes(st.RandaoMixes)
		b.balancesMultiValue = NewMultiValueBalances(st.Balances)
		b.validatorsMultiValue = NewMultiValueValidators(st.Validators)
		b.inactivityScoresMultiValue = NewMultiValueInactivityScores(st.InactivityScores)
		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateCapellaSharedFieldRefCount)
	} else {
		bRoots := make([][32]byte, fieldparams.BlockRootsLength)
		for i, r := range st.BlockRoots {
			bRoots[i] = bytesutil.ToBytes32(r)
		}
		b.blockRoots = bRoots

		sRoots := make([][32]byte, fieldparams.StateRootsLength)
		for i, r := range st.StateRoots {
			sRoots[i] = bytesutil.ToBytes32(r)
		}
		b.stateRoots = sRoots

		mixes := make([][32]byte, fieldparams.RandaoMixesLength)
		for i, m := range st.RandaoMixes {
			mixes[i] = bytesutil.ToBytes32(m)
		}
		b.randaoMixes = mixes

		b.balances = st.Balances
		b.validators = st.Validators
		b.inactivityScores = st.InactivityScores

		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, capellaSharedFieldRefCount)
	}

	for _, f := range capellaFields {
		b.dirtyFields[f] = true
		b.rebuildTrie[f] = true
		b.dirtyIndices[f] = []uint64{}
		trie, err := fieldtrie.NewFieldTrie(f, types.BasicArray, nil, 0)
		if err != nil {
			return nil, err
		}
		b.stateFieldLeaves[f] = trie
	}

	// Initialize field reference tracking for shared data.
	b.sharedFieldReferences[types.HistoricalRoots] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Eth1DataVotes] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Slashings] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.PreviousEpochParticipationBits] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.CurrentEpochParticipationBits] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.LatestExecutionPayloadHeaderCapella] = stateutil.NewRef(1) // New in Capella.
	b.sharedFieldReferences[types.HistoricalSummaries] = stateutil.NewRef(1)                 // New in Capella.
	if !features.Get().EnableExperimentalState {
		b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Balances] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Validators] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.InactivityScores] = stateutil.NewRef(1)
	}

	state.Count.Inc()
	// Finalizer runs when dst is being destroyed in garbage collection.
	runtime.SetFinalizer(b, finalizerCleanup)
	return b, nil
}

// InitializeFromProtoUnsafeDeneb directly uses the beacon state protobuf fields
// and sets them as fields of the BeaconState type.
func InitializeFromProtoUnsafeDeneb(st *ethpb.BeaconStateDeneb) (state.BeaconState, error) {
	if st == nil {
		return nil, errors.New("received nil state")
	}

	hRoots := customtypes.HistoricalRoots(make([][32]byte, len(st.HistoricalRoots)))
	for i, r := range st.HistoricalRoots {
		hRoots[i] = bytesutil.ToBytes32(r)
	}

	fieldCount := params.BeaconConfig().BeaconStateDenebFieldCount
	b := &BeaconState{
		version:                           version.Deneb,
		genesisTime:                       st.GenesisTime,
		genesisValidatorsRoot:             bytesutil.ToBytes32(st.GenesisValidatorsRoot),
		slot:                              st.Slot,
		fork:                              st.Fork,
		latestBlockHeader:                 st.LatestBlockHeader,
		historicalRoots:                   hRoots,
		eth1Data:                          st.Eth1Data,
		eth1DataVotes:                     st.Eth1DataVotes,
		eth1DepositIndex:                  st.Eth1DepositIndex,
		slashings:                         st.Slashings,
		previousEpochParticipation:        st.PreviousEpochParticipation,
		currentEpochParticipation:         st.CurrentEpochParticipation,
		justificationBits:                 st.JustificationBits,
		previousJustifiedCheckpoint:       st.PreviousJustifiedCheckpoint,
		currentJustifiedCheckpoint:        st.CurrentJustifiedCheckpoint,
		finalizedCheckpoint:               st.FinalizedCheckpoint,
		currentSyncCommittee:              st.CurrentSyncCommittee,
		nextSyncCommittee:                 st.NextSyncCommittee,
		latestExecutionPayloadHeaderDeneb: st.LatestExecutionPayloadHeader,
		nextWithdrawalIndex:               st.NextWithdrawalIndex,
		nextWithdrawalValidatorIndex:      st.NextWithdrawalValidatorIndex,
		historicalSummaries:               st.HistoricalSummaries,

		dirtyFields:      make(map[types.FieldIndex]bool, fieldCount),
		dirtyIndices:     make(map[types.FieldIndex][]uint64, fieldCount),
		stateFieldLeaves: make(map[types.FieldIndex]*fieldtrie.FieldTrie, fieldCount),
		rebuildTrie:      make(map[types.FieldIndex]bool, fieldCount),
		valMapHandler:    stateutil.NewValMapHandler(st.Validators),
	}

	if features.Get().EnableExperimentalState {
		b.blockRootsMultiValue = NewMultiValueBlockRoots(st.BlockRoots)
		b.stateRootsMultiValue = NewMultiValueStateRoots(st.StateRoots)
		b.randaoMixesMultiValue = NewMultiValueRandaoMixes(st.RandaoMixes)
		b.balancesMultiValue = NewMultiValueBalances(st.Balances)
		b.validatorsMultiValue = NewMultiValueValidators(st.Validators)
		b.inactivityScoresMultiValue = NewMultiValueInactivityScores(st.InactivityScores)
		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateDenebSharedFieldRefCount)
	} else {
		bRoots := make([][32]byte, fieldparams.BlockRootsLength)
		for i, r := range st.BlockRoots {
			bRoots[i] = bytesutil.ToBytes32(r)
		}
		b.blockRoots = bRoots

		sRoots := make([][32]byte, fieldparams.StateRootsLength)
		for i, r := range st.StateRoots {
			sRoots[i] = bytesutil.ToBytes32(r)
		}
		b.stateRoots = sRoots

		mixes := make([][32]byte, fieldparams.RandaoMixesLength)
		for i, m := range st.RandaoMixes {
			mixes[i] = bytesutil.ToBytes32(m)
		}
		b.randaoMixes = mixes

		b.balances = st.Balances
		b.validators = st.Validators
		b.inactivityScores = st.InactivityScores

		b.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, denebSharedFieldRefCount)
	}

	for _, f := range denebFields {
		b.dirtyFields[f] = true
		b.rebuildTrie[f] = true
		b.dirtyIndices[f] = []uint64{}
		trie, err := fieldtrie.NewFieldTrie(f, types.BasicArray, nil, 0)
		if err != nil {
			return nil, err
		}
		b.stateFieldLeaves[f] = trie
	}

	// Initialize field reference tracking for shared data.
	b.sharedFieldReferences[types.HistoricalRoots] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Eth1DataVotes] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.Slashings] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.PreviousEpochParticipationBits] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.CurrentEpochParticipationBits] = stateutil.NewRef(1)
	b.sharedFieldReferences[types.LatestExecutionPayloadHeaderDeneb] = stateutil.NewRef(1) // New in Deneb.
	b.sharedFieldReferences[types.HistoricalSummaries] = stateutil.NewRef(1)               // New in Capella.
	if !features.Get().EnableExperimentalState {
		b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Balances] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.Validators] = stateutil.NewRef(1)
		b.sharedFieldReferences[types.InactivityScores] = stateutil.NewRef(1)
	}

	state.Count.Inc()
	// Finalizer runs when dst is being destroyed in garbage collection.
	runtime.SetFinalizer(b, finalizerCleanup)
	return b, nil
}

// Copy returns a deep copy of the beacon state.
func (b *BeaconState) Copy() state.BeaconState {
	b.lock.RLock()
	defer b.lock.RUnlock()

	var fieldCount int
	switch b.version {
	case version.Phase0:
		fieldCount = params.BeaconConfig().BeaconStateFieldCount
	case version.Altair:
		fieldCount = params.BeaconConfig().BeaconStateAltairFieldCount
	case version.Bellatrix:
		fieldCount = params.BeaconConfig().BeaconStateBellatrixFieldCount
	case version.Capella:
		fieldCount = params.BeaconConfig().BeaconStateCapellaFieldCount
	case version.Deneb:
		fieldCount = params.BeaconConfig().BeaconStateDenebFieldCount
	}

	dst := &BeaconState{
		version: b.version,

		// Primitive types, safe to copy.
		genesisTime:                  b.genesisTime,
		slot:                         b.slot,
		eth1DepositIndex:             b.eth1DepositIndex,
		nextWithdrawalIndex:          b.nextWithdrawalIndex,
		nextWithdrawalValidatorIndex: b.nextWithdrawalValidatorIndex,

		// Large arrays, infrequently changed, constant size.
		blockRoots:                b.blockRoots,
		blockRootsMultiValue:      b.blockRootsMultiValue,
		stateRoots:                b.stateRoots,
		stateRootsMultiValue:      b.stateRootsMultiValue,
		randaoMixes:               b.randaoMixes,
		randaoMixesMultiValue:     b.randaoMixesMultiValue,
		previousEpochAttestations: b.previousEpochAttestations,
		currentEpochAttestations:  b.currentEpochAttestations,
		eth1DataVotes:             b.eth1DataVotes,
		slashings:                 b.slashings,

		// Large arrays, increases over time.
		balances:                   b.balances,
		balancesMultiValue:         b.balancesMultiValue,
		historicalRoots:            b.historicalRoots,
		historicalSummaries:        b.historicalSummaries,
		validators:                 b.validators,
		validatorsMultiValue:       b.validatorsMultiValue,
		previousEpochParticipation: b.previousEpochParticipation,
		currentEpochParticipation:  b.currentEpochParticipation,
		inactivityScores:           b.inactivityScores,
		inactivityScoresMultiValue: b.inactivityScoresMultiValue,

		// Everything else, too small to be concerned about, constant size.
		genesisValidatorsRoot:               b.genesisValidatorsRoot,
		justificationBits:                   b.justificationBitsVal(),
		fork:                                b.forkVal(),
		latestBlockHeader:                   b.latestBlockHeaderVal(),
		eth1Data:                            b.eth1DataVal(),
		previousJustifiedCheckpoint:         b.previousJustifiedCheckpointVal(),
		currentJustifiedCheckpoint:          b.currentJustifiedCheckpointVal(),
		finalizedCheckpoint:                 b.finalizedCheckpointVal(),
		currentSyncCommittee:                b.currentSyncCommitteeVal(),
		nextSyncCommittee:                   b.nextSyncCommitteeVal(),
		latestExecutionPayloadHeader:        b.latestExecutionPayloadHeaderVal(),
		latestExecutionPayloadHeaderCapella: b.latestExecutionPayloadHeaderCapellaVal(),
		latestExecutionPayloadHeaderDeneb:   b.latestExecutionPayloadHeaderDenebVal(),

		id: types.Enumerator.Inc(),

		dirtyFields:      make(map[types.FieldIndex]bool, fieldCount),
		dirtyIndices:     make(map[types.FieldIndex][]uint64, fieldCount),
		rebuildTrie:      make(map[types.FieldIndex]bool, fieldCount),
		stateFieldLeaves: make(map[types.FieldIndex]*fieldtrie.FieldTrie, fieldCount),

		// Share the reference to validator index map.
		valMapHandler: b.valMapHandler,
	}

	if features.Get().EnableExperimentalState {
		b.blockRootsMultiValue.Copy(b, dst)
		b.stateRootsMultiValue.Copy(b, dst)
		b.randaoMixesMultiValue.Copy(b, dst)
		b.balancesMultiValue.Copy(b, dst)
		if b.version > version.Phase0 {
			b.inactivityScoresMultiValue.Copy(b, dst)
		}
		b.validatorsMultiValue.Copy(b, dst)
	}

	if features.Get().EnableExperimentalState {
		switch b.version {
		case version.Phase0:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStatePhase0SharedFieldRefCount)
		case version.Altair:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateAltairSharedFieldRefCount)
		case version.Bellatrix:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateBellatrixSharedFieldRefCount)
		case version.Capella:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateCapellaSharedFieldRefCount)
		case version.Deneb:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, experimentalStateDenebSharedFieldRefCount)
		}
	} else {
		switch b.version {
		case version.Phase0:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, phase0SharedFieldRefCount)
		case version.Altair:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, altairSharedFieldRefCount)
		case version.Bellatrix:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, bellatrixSharedFieldRefCount)
		case version.Capella:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, capellaSharedFieldRefCount)
		case version.Deneb:
			dst.sharedFieldReferences = make(map[types.FieldIndex]*stateutil.Reference, denebSharedFieldRefCount)
		}
	}

	for field, ref := range b.sharedFieldReferences {
		ref.AddRef()
		dst.sharedFieldReferences[field] = ref
	}

	// Increment ref for validator map
	b.valMapHandler.AddRef()

	for i := range b.dirtyFields {
		dst.dirtyFields[i] = true
	}

	for i := range b.dirtyIndices {
		indices := make([]uint64, len(b.dirtyIndices[i]))
		copy(indices, b.dirtyIndices[i])
		dst.dirtyIndices[i] = indices
	}

	for i := range b.rebuildTrie {
		dst.rebuildTrie[i] = true
	}

	for fldIdx, fieldTrie := range b.stateFieldLeaves {
		dst.stateFieldLeaves[fldIdx] = fieldTrie
		if fieldTrie.FieldReference() != nil {
			fieldTrie.Lock()
			fieldTrie.FieldReference().AddRef()
			fieldTrie.Unlock()
		}
	}

	if b.merkleLayers != nil {
		dst.merkleLayers = make([][][]byte, len(b.merkleLayers))
		for i, layer := range b.merkleLayers {
			dst.merkleLayers[i] = make([][]byte, len(layer))
			for j, content := range layer {
				dst.merkleLayers[i][j] = make([]byte, len(content))
				copy(dst.merkleLayers[i][j], content)
			}
		}
	}

	state.Count.Inc()
	// Finalizer runs when dst is being destroyed in garbage collection.
	runtime.SetFinalizer(dst, finalizerCleanup)
	return dst
}

// HashTreeRoot of the beacon state retrieves the Merkle root of the trie
// representation of the beacon state based on the Ethereum Simple Serialize specification.
func (b *BeaconState) HashTreeRoot(ctx context.Context) ([32]byte, error) {
	ctx, span := trace.StartSpan(ctx, "beaconState.HashTreeRoot")
	defer span.End()

	b.lock.Lock()
	defer b.lock.Unlock()
	if err := b.initializeMerkleLayers(ctx); err != nil {
		return [32]byte{}, err
	}
	if err := b.recomputeDirtyFields(ctx); err != nil {
		return [32]byte{}, err
	}
	return bytesutil.ToBytes32(b.merkleLayers[len(b.merkleLayers)-1][0]), nil
}

// Initializes the Merkle layers for the beacon state if they are empty.
//
// WARNING: Caller must acquire the mutex before using.
func (b *BeaconState) initializeMerkleLayers(ctx context.Context) error {
	if len(b.merkleLayers) > 0 {
		return nil
	}
	fieldRoots, err := ComputeFieldRootsWithHasher(ctx, b)
	if err != nil {
		return err
	}
	layers := stateutil.Merkleize(fieldRoots)
	b.merkleLayers = layers
	switch b.version {
	case version.Phase0:
		b.dirtyFields = make(map[types.FieldIndex]bool, params.BeaconConfig().BeaconStateFieldCount)
	case version.Altair:
		b.dirtyFields = make(map[types.FieldIndex]bool, params.BeaconConfig().BeaconStateAltairFieldCount)
	case version.Bellatrix:
		b.dirtyFields = make(map[types.FieldIndex]bool, params.BeaconConfig().BeaconStateBellatrixFieldCount)
	case version.Capella:
		b.dirtyFields = make(map[types.FieldIndex]bool, params.BeaconConfig().BeaconStateCapellaFieldCount)
	case version.Deneb:
		b.dirtyFields = make(map[types.FieldIndex]bool, params.BeaconConfig().BeaconStateDenebFieldCount)
	}

	return nil
}

// Recomputes the Merkle layers for the dirty fields in the state.
//
// WARNING: Caller must acquire the mutex before using.
func (b *BeaconState) recomputeDirtyFields(ctx context.Context) error {
	for field := range b.dirtyFields {
		root, err := b.rootSelector(ctx, field)
		if err != nil {
			return err
		}
		idx := field.RealPosition()
		b.merkleLayers[0][idx] = root[:]
		b.recomputeRoot(idx)
		delete(b.dirtyFields, field)
	}
	return nil
}

// FieldReferencesCount returns the reference count held by each field. This
// also includes the field trie held by each field.
func (b *BeaconState) FieldReferencesCount() map[string]uint64 {
	refMap := make(map[string]uint64)
	b.lock.RLock()
	defer b.lock.RUnlock()
	for i, f := range b.sharedFieldReferences {
		refMap[i.String()] = uint64(f.Refs())
	}
	for i, f := range b.stateFieldLeaves {
		numOfRefs := uint64(f.FieldReference().Refs())
		f.RLock()
		if !f.Empty() {
			refMap[i.String()+"_trie"] = numOfRefs
		}
		f.RUnlock()
	}
	return refMap
}

// RecordStateMetrics proceeds to record any state related metrics data.
func (b *BeaconState) RecordStateMetrics() {
	b.lock.RLock()
	defer b.lock.RUnlock()
	// Only run this for nodes running with the experimental state.
	if !features.Get().EnableExperimentalState {
		return
	}

	// Validators
	if b.validatorsMultiValue != nil {
		stats := b.validatorsMultiValue.MultiValueStatistics()
		multiValueIndividualElementsCountGauge.WithLabelValues(types.Validators.String()).Set(float64(stats.TotalIndividualElements))
		multiValueIndividualElementReferencesCountGauge.WithLabelValues(types.Validators.String()).Set(float64(stats.TotalIndividualElemReferences))
		multiValueAppendedElementsCountGauge.WithLabelValues(types.Validators.String()).Set(float64(stats.TotalAppendedElements))
		multiValueAppendedElementReferencesCountGauge.WithLabelValues(types.Validators.String()).Set(float64(stats.TotalAppendedElemReferences))
	}

	// Balances
	if b.balancesMultiValue != nil {
		stats := b.balancesMultiValue.MultiValueStatistics()
		multiValueIndividualElementsCountGauge.WithLabelValues(types.Balances.String()).Set(float64(stats.TotalIndividualElements))
		multiValueIndividualElementReferencesCountGauge.WithLabelValues(types.Balances.String()).Set(float64(stats.TotalIndividualElemReferences))
		multiValueAppendedElementsCountGauge.WithLabelValues(types.Balances.String()).Set(float64(stats.TotalAppendedElements))
		multiValueAppendedElementReferencesCountGauge.WithLabelValues(types.Balances.String()).Set(float64(stats.TotalAppendedElemReferences))
	}

	// InactivityScores
	if b.inactivityScoresMultiValue != nil {
		stats := b.inactivityScoresMultiValue.MultiValueStatistics()
		multiValueIndividualElementsCountGauge.WithLabelValues(types.InactivityScores.String()).Set(float64(stats.TotalIndividualElements))
		multiValueIndividualElementReferencesCountGauge.WithLabelValues(types.InactivityScores.String()).Set(float64(stats.TotalIndividualElemReferences))
		multiValueAppendedElementsCountGauge.WithLabelValues(types.InactivityScores.String()).Set(float64(stats.TotalAppendedElements))
		multiValueAppendedElementReferencesCountGauge.WithLabelValues(types.InactivityScores.String()).Set(float64(stats.TotalAppendedElemReferences))
	}
	// BlockRoots
	if b.blockRootsMultiValue != nil {
		stats := b.blockRootsMultiValue.MultiValueStatistics()
		multiValueIndividualElementsCountGauge.WithLabelValues(types.BlockRoots.String()).Set(float64(stats.TotalIndividualElements))
		multiValueIndividualElementReferencesCountGauge.WithLabelValues(types.BlockRoots.String()).Set(float64(stats.TotalIndividualElemReferences))
		multiValueAppendedElementsCountGauge.WithLabelValues(types.BlockRoots.String()).Set(float64(stats.TotalAppendedElements))
		multiValueAppendedElementReferencesCountGauge.WithLabelValues(types.BlockRoots.String()).Set(float64(stats.TotalAppendedElemReferences))
	}

	// StateRoots
	if b.stateRootsMultiValue != nil {
		stats := b.stateRootsMultiValue.MultiValueStatistics()
		multiValueIndividualElementsCountGauge.WithLabelValues(types.StateRoots.String()).Set(float64(stats.TotalIndividualElements))
		multiValueIndividualElementReferencesCountGauge.WithLabelValues(types.StateRoots.String()).Set(float64(stats.TotalIndividualElemReferences))
		multiValueAppendedElementsCountGauge.WithLabelValues(types.StateRoots.String()).Set(float64(stats.TotalAppendedElements))
		multiValueAppendedElementReferencesCountGauge.WithLabelValues(types.StateRoots.String()).Set(float64(stats.TotalAppendedElemReferences))
	}
	// RandaoMixes
	if b.randaoMixesMultiValue != nil {
		stats := b.randaoMixesMultiValue.MultiValueStatistics()
		multiValueIndividualElementsCountGauge.WithLabelValues(types.RandaoMixes.String()).Set(float64(stats.TotalIndividualElements))
		multiValueIndividualElementReferencesCountGauge.WithLabelValues(types.RandaoMixes.String()).Set(float64(stats.TotalIndividualElemReferences))
		multiValueAppendedElementsCountGauge.WithLabelValues(types.RandaoMixes.String()).Set(float64(stats.TotalAppendedElements))
		multiValueAppendedElementReferencesCountGauge.WithLabelValues(types.RandaoMixes.String()).Set(float64(stats.TotalAppendedElemReferences))
	}
}

// IsNil checks if the state and the underlying proto
// object are nil.
func (b *BeaconState) IsNil() bool {
	return b == nil
}

func (b *BeaconState) rootSelector(ctx context.Context, field types.FieldIndex) ([32]byte, error) {
	_, span := trace.StartSpan(ctx, "beaconState.rootSelector")
	defer span.End()
	span.AddAttributes(trace.StringAttribute("field", field.String()))

	switch field {
	case types.GenesisTime:
		return ssz.Uint64Root(b.genesisTime), nil
	case types.GenesisValidatorsRoot:
		return b.genesisValidatorsRoot, nil
	case types.Slot:
		return ssz.Uint64Root(uint64(b.slot)), nil
	case types.Eth1DepositIndex:
		return ssz.Uint64Root(b.eth1DepositIndex), nil
	case types.Fork:
		return ssz.ForkRoot(b.fork)
	case types.LatestBlockHeader:
		return stateutil.BlockHeaderRoot(b.latestBlockHeader)
	case types.BlockRoots:
		return b.blockRootsRootSelector(field)
	case types.StateRoots:
		return b.stateRootsRootSelector(field)
	case types.HistoricalRoots:
		hRoots := make([][]byte, len(b.historicalRoots))
		for i := range hRoots {
			hRoots[i] = b.historicalRoots[i][:]
		}
		return ssz.ByteArrayRootWithLimit(hRoots, fieldparams.HistoricalRootsLength)
	case types.Eth1Data:
		return stateutil.Eth1Root(b.eth1Data)
	case types.Eth1DataVotes:
		if b.rebuildTrie[field] {
			err := b.resetFieldTrie(
				field,
				b.eth1DataVotes,
				params.BeaconConfig().Eth1DataVotesLength(),
			)
			if err != nil {
				return [32]byte{}, err
			}
			delete(b.rebuildTrie, field)
			return b.stateFieldLeaves[field].TrieRoot()
		}
		return b.recomputeFieldTrie(field, b.eth1DataVotes)
	case types.Validators:
		return b.validatorsRootSelector(field)
	case types.Balances:
		return b.balancesRootSelector(field)
	case types.RandaoMixes:
		return b.randaoMixesRootSelector(field)
	case types.Slashings:
		return ssz.SlashingsRoot(b.slashings)
	case types.PreviousEpochAttestations:
		if b.rebuildTrie[field] {
			err := b.resetFieldTrie(
				field,
				b.previousEpochAttestations,
				params.BeaconConfig().PreviousEpochAttestationsLength(),
			)
			if err != nil {
				return [32]byte{}, err
			}
			delete(b.rebuildTrie, field)
			return b.stateFieldLeaves[field].TrieRoot()
		}
		return b.recomputeFieldTrie(field, b.previousEpochAttestations)
	case types.CurrentEpochAttestations:
		if b.rebuildTrie[field] {
			err := b.resetFieldTrie(
				field,
				b.currentEpochAttestations,
				params.BeaconConfig().CurrentEpochAttestationsLength(),
			)
			if err != nil {
				return [32]byte{}, err
			}
			delete(b.rebuildTrie, field)
			return b.stateFieldLeaves[field].TrieRoot()
		}
		return b.recomputeFieldTrie(field, b.currentEpochAttestations)
	case types.PreviousEpochParticipationBits:
		return stateutil.ParticipationBitsRoot(b.previousEpochParticipation)
	case types.CurrentEpochParticipationBits:
		return stateutil.ParticipationBitsRoot(b.currentEpochParticipation)
	case types.JustificationBits:
		return bytesutil.ToBytes32(b.justificationBits), nil
	case types.PreviousJustifiedCheckpoint:
		return ssz.CheckpointRoot(b.previousJustifiedCheckpoint)
	case types.CurrentJustifiedCheckpoint:
		return ssz.CheckpointRoot(b.currentJustifiedCheckpoint)
	case types.FinalizedCheckpoint:
		return ssz.CheckpointRoot(b.finalizedCheckpoint)
	case types.InactivityScores:
		if features.Get().EnableExperimentalState {
			return stateutil.Uint64ListRootWithRegistryLimit(b.inactivityScoresMultiValue.Value(b))
		} else {
			return stateutil.Uint64ListRootWithRegistryLimit(b.inactivityScores)
		}
	case types.CurrentSyncCommittee:
		return stateutil.SyncCommitteeRoot(b.currentSyncCommittee)
	case types.NextSyncCommittee:
		return stateutil.SyncCommitteeRoot(b.nextSyncCommittee)
	case types.LatestExecutionPayloadHeader:
		return b.latestExecutionPayloadHeader.HashTreeRoot()
	case types.LatestExecutionPayloadHeaderCapella:
		return b.latestExecutionPayloadHeaderCapella.HashTreeRoot()
	case types.LatestExecutionPayloadHeaderDeneb:
		return b.latestExecutionPayloadHeaderDeneb.HashTreeRoot()
	case types.NextWithdrawalIndex:
		return ssz.Uint64Root(b.nextWithdrawalIndex), nil
	case types.NextWithdrawalValidatorIndex:
		return ssz.Uint64Root(uint64(b.nextWithdrawalValidatorIndex)), nil
	case types.HistoricalSummaries:
		return stateutil.HistoricalSummariesRoot(b.historicalSummaries)
	}
	return [32]byte{}, errors.New("invalid field index provided")
}

// CopyAllTries copies our field tries from the state. This is used to
// remove shared field tries which have references to other states and
// only have this copied set referencing to the current state.
func (b *BeaconState) CopyAllTries() {
	b.lock.Lock()
	defer b.lock.Unlock()

	for fldIdx, fieldTrie := range b.stateFieldLeaves {
		if fieldTrie.FieldReference() != nil {
			fieldTrie.Lock()
			if fieldTrie.FieldReference().Refs() > 1 {
				fieldTrie.FieldReference().MinusRef()
				newTrie := fieldTrie.CopyTrie()
				b.stateFieldLeaves[fldIdx] = newTrie
			}
			fieldTrie.Unlock()
		}
	}
}

func (b *BeaconState) recomputeFieldTrie(index types.FieldIndex, elements interface{}) ([32]byte, error) {
	fTrie := b.stateFieldLeaves[index]
	fTrieMutex := fTrie.RWMutex
	// We can't lock the trie directly because the trie's variable gets reassigned,
	// and therefore we would call Unlock() on a different object.
	fTrieMutex.Lock()

	if fTrie.Empty() {
		err := b.resetFieldTrie(index, elements, fTrie.Length())
		if err != nil {
			fTrieMutex.Unlock()
			return [32]byte{}, err
		}
		// Reduce reference count as we are instantiating a new trie.
		fTrie.FieldReference().MinusRef()
		fTrieMutex.Unlock()
		return b.stateFieldLeaves[index].TrieRoot()
	}

	if fTrie.FieldReference().Refs() > 1 {
		fTrie.FieldReference().MinusRef()
		newTrie := fTrie.TransferTrie()
		b.stateFieldLeaves[index] = newTrie
		fTrie = newTrie
	}
	fTrieMutex.Unlock()

	// remove duplicate indexes
	b.dirtyIndices[index] = slice.SetUint64(b.dirtyIndices[index])
	// sort indexes again
	sort.Slice(b.dirtyIndices[index], func(i int, j int) bool {
		return b.dirtyIndices[index][i] < b.dirtyIndices[index][j]
	})
	root, err := fTrie.RecomputeTrie(b.dirtyIndices[index], elements)
	if err != nil {
		return [32]byte{}, err
	}
	b.dirtyIndices[index] = []uint64{}
	return root, nil
}

func (b *BeaconState) resetFieldTrie(index types.FieldIndex, elements interface{}, length uint64) error {
	fTrie, err := fieldtrie.NewFieldTrie(index, fieldMap[index], elements, length)
	if err != nil {
		return err
	}
	b.stateFieldLeaves[index] = fTrie
	b.dirtyIndices[index] = []uint64{}
	return nil
}

func finalizerCleanup(b *BeaconState) {
	b.lock.Lock()
	defer b.lock.Unlock()
	for field, v := range b.sharedFieldReferences {
		v.MinusRef()
		if b.stateFieldLeaves[field].FieldReference() != nil {
			b.stateFieldLeaves[field].FieldReference().MinusRef()
		}
	}
	for i := range b.dirtyFields {
		delete(b.dirtyFields, i)
	}
	for i := range b.rebuildTrie {
		delete(b.rebuildTrie, i)
	}
	for i := range b.dirtyIndices {
		delete(b.dirtyIndices, i)
	}
	for i := range b.sharedFieldReferences {
		delete(b.sharedFieldReferences, i)
	}
	for i := range b.stateFieldLeaves {
		delete(b.stateFieldLeaves, i)
	}

	if features.Get().EnableExperimentalState {
		if b.blockRootsMultiValue != nil {
			b.blockRootsMultiValue.Detach(b)
		}
		if b.stateRootsMultiValue != nil {
			b.stateRootsMultiValue.Detach(b)
		}
		if b.randaoMixesMultiValue != nil {
			b.randaoMixesMultiValue.Detach(b)
		}
		if b.balancesMultiValue != nil {
			b.balancesMultiValue.Detach(b)
		}
		if b.inactivityScoresMultiValue != nil {
			b.inactivityScoresMultiValue.Detach(b)
		}
		if b.validatorsMultiValue != nil {
			b.validatorsMultiValue.Detach(b)
		}
	}

	state.Count.Sub(1)
}

func (b *BeaconState) blockRootsRootSelector(field types.FieldIndex) ([32]byte, error) {
	if b.rebuildTrie[field] {
		if features.Get().EnableExperimentalState {
			err := b.resetFieldTrie(field, mvslice.MultiValueSliceComposite[[32]byte]{
				Identifiable:    b,
				MultiValueSlice: b.blockRootsMultiValue,
			}, fieldparams.BlockRootsLength)
			if err != nil {
				return [32]byte{}, err
			}
		} else {
			err := b.resetFieldTrie(field, b.blockRoots, fieldparams.BlockRootsLength)
			if err != nil {
				return [32]byte{}, err
			}
		}
		delete(b.rebuildTrie, field)
		return b.stateFieldLeaves[field].TrieRoot()
	}
	if features.Get().EnableExperimentalState {
		return b.recomputeFieldTrie(field, mvslice.MultiValueSliceComposite[[32]byte]{
			Identifiable:    b,
			MultiValueSlice: b.blockRootsMultiValue,
		})
	} else {
		return b.recomputeFieldTrie(field, b.blockRoots)
	}
}

func (b *BeaconState) stateRootsRootSelector(field types.FieldIndex) ([32]byte, error) {
	if b.rebuildTrie[field] {
		if features.Get().EnableExperimentalState {
			err := b.resetFieldTrie(field, mvslice.MultiValueSliceComposite[[32]byte]{
				Identifiable:    b,
				MultiValueSlice: b.stateRootsMultiValue,
			}, fieldparams.StateRootsLength)
			if err != nil {
				return [32]byte{}, err
			}
		} else {
			err := b.resetFieldTrie(field, b.stateRoots, fieldparams.StateRootsLength)
			if err != nil {
				return [32]byte{}, err
			}
		}
		delete(b.rebuildTrie, field)
		return b.stateFieldLeaves[field].TrieRoot()
	}
	if features.Get().EnableExperimentalState {
		return b.recomputeFieldTrie(field, mvslice.MultiValueSliceComposite[[32]byte]{
			Identifiable:    b,
			MultiValueSlice: b.stateRootsMultiValue,
		})
	} else {
		return b.recomputeFieldTrie(field, b.stateRoots)
	}
}

func (b *BeaconState) validatorsRootSelector(field types.FieldIndex) ([32]byte, error) {
	if b.rebuildTrie[field] {
		if features.Get().EnableExperimentalState {
			err := b.resetFieldTrie(field, mvslice.MultiValueSliceComposite[*ethpb.Validator]{
				Identifiable:    b,
				MultiValueSlice: b.validatorsMultiValue,
			}, fieldparams.ValidatorRegistryLimit)
			if err != nil {
				return [32]byte{}, err
			}
		} else {
			err := b.resetFieldTrie(field, b.validators, fieldparams.ValidatorRegistryLimit)
			if err != nil {
				return [32]byte{}, err
			}
		}
		delete(b.rebuildTrie, field)
		return b.stateFieldLeaves[field].TrieRoot()
	}
	if features.Get().EnableExperimentalState {
		return b.recomputeFieldTrie(field, mvslice.MultiValueSliceComposite[*ethpb.Validator]{
			Identifiable:    b,
			MultiValueSlice: b.validatorsMultiValue,
		})
	} else {
		return b.recomputeFieldTrie(field, b.validators)
	}
}

func (b *BeaconState) balancesRootSelector(field types.FieldIndex) ([32]byte, error) {
	if b.rebuildTrie[field] {
		if features.Get().EnableExperimentalState {
			err := b.resetFieldTrie(field, mvslice.MultiValueSliceComposite[uint64]{
				Identifiable:    b,
				MultiValueSlice: b.balancesMultiValue,
			}, stateutil.ValidatorLimitForBalancesChunks())
			if err != nil {
				return [32]byte{}, err
			}
		} else {
			err := b.resetFieldTrie(field, b.balances, stateutil.ValidatorLimitForBalancesChunks())
			if err != nil {
				return [32]byte{}, err
			}
		}
		delete(b.rebuildTrie, field)
		return b.stateFieldLeaves[field].TrieRoot()
	}
	if features.Get().EnableExperimentalState {
		return b.recomputeFieldTrie(field, mvslice.MultiValueSliceComposite[uint64]{
			Identifiable:    b,
			MultiValueSlice: b.balancesMultiValue,
		})
	} else {
		return b.recomputeFieldTrie(field, b.balances)
	}
}

func (b *BeaconState) randaoMixesRootSelector(field types.FieldIndex) ([32]byte, error) {
	if b.rebuildTrie[field] {
		if features.Get().EnableExperimentalState {
			err := b.resetFieldTrie(field, mvslice.MultiValueSliceComposite[[32]byte]{
				Identifiable:    b,
				MultiValueSlice: b.randaoMixesMultiValue,
			}, fieldparams.RandaoMixesLength)
			if err != nil {
				return [32]byte{}, err
			}
		} else {
			err := b.resetFieldTrie(field, b.randaoMixes, fieldparams.RandaoMixesLength)
			if err != nil {
				return [32]byte{}, err
			}
		}
		delete(b.rebuildTrie, field)
		return b.stateFieldLeaves[field].TrieRoot()
	}
	if features.Get().EnableExperimentalState {
		return b.recomputeFieldTrie(field, mvslice.MultiValueSliceComposite[[32]byte]{
			Identifiable:    b,
			MultiValueSlice: b.randaoMixesMultiValue,
		})
	} else {
		return b.recomputeFieldTrie(field, b.randaoMixes)
	}
}
