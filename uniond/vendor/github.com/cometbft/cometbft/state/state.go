package state

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"time"

	"github.com/cosmos/gogoproto/proto"

	cmtstate "github.com/cometbft/cometbft/proto/tendermint/state"
	cmtversion "github.com/cometbft/cometbft/proto/tendermint/version"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"
)

// database keys
var (
	stateKey = []byte("stateKey")
)

//-----------------------------------------------------------------------------

// InitStateVersion sets the Consensus.Block and Software versions,
// but leaves the Consensus.App version blank.
// The Consensus.App version will be set during the Handshake, once
// we hear from the app what protocol version it is running.
var InitStateVersion = cmtstate.Version{
	Consensus: cmtversion.Consensus{
		Block: version.BlockProtocol,
		App:   0,
	},
	Software: version.TMCoreSemVer,
}

//-----------------------------------------------------------------------------

// State is a short description of the latest committed block of the consensus protocol.
// It keeps all information necessary to validate new blocks,
// including the last validator set and the consensus params.
// All fields are exposed so the struct can be easily serialized,
// but none of them should be mutated directly.
// Instead, use state.Copy() or state.NextState(...).
// NOTE: not goroutine-safe.
type State struct {
	Version cmtstate.Version

	// immutable
	ChainID       string
	InitialHeight int64 // should be 1, not 0, when starting from height 1

	// LastBlockHeight=0 at genesis (ie. block(H=0) does not exist)
	LastBlockHeight int64
	LastBlockID     types.BlockID
	LastBlockTime   time.Time

	// LastValidators is used to validate block.LastCommit.
	// Validators are persisted to the database separately every time they change,
	// so we can query for historical validator sets.
	// Note that if s.LastBlockHeight causes a valset change,
	// we set s.LastHeightValidatorsChanged = s.LastBlockHeight + 1 + 1
	// Extra +1 due to nextValSet delay.
	NextValidators              *types.ValidatorSet
	Validators                  *types.ValidatorSet
	LastValidators              *types.ValidatorSet
	LastHeightValidatorsChanged int64

	// Consensus parameters used for validating blocks.
	// Changes returned by FinalizeBlock and updated after Commit.
	ConsensusParams                  types.ConsensusParams
	LastHeightConsensusParamsChanged int64

	// Merkle root of the results from executing prev block
	LastResultsHash []byte

	// the latest AppHash we've received from calling abci.Commit()
	AppHash []byte
}

// Copy makes a copy of the State for mutating.
func (state State) Copy() State {

	return State{
		Version:       state.Version,
		ChainID:       state.ChainID,
		InitialHeight: state.InitialHeight,

		LastBlockHeight: state.LastBlockHeight,
		LastBlockID:     state.LastBlockID,
		LastBlockTime:   state.LastBlockTime,

		NextValidators:              state.NextValidators.Copy(),
		Validators:                  state.Validators.Copy(),
		LastValidators:              state.LastValidators.Copy(),
		LastHeightValidatorsChanged: state.LastHeightValidatorsChanged,

		ConsensusParams:                  state.ConsensusParams,
		LastHeightConsensusParamsChanged: state.LastHeightConsensusParamsChanged,

		AppHash: state.AppHash,

		LastResultsHash: state.LastResultsHash,
	}
}

// Equals returns true if the States are identical.
func (state State) Equals(state2 State) bool {
	sbz, s2bz := state.Bytes(), state2.Bytes()
	return bytes.Equal(sbz, s2bz)
}

// Bytes serializes the State using protobuf.
// It panics if either casting to protobuf or serialization fails.
func (state State) Bytes() []byte {
	sm, err := state.ToProto()
	if err != nil {
		panic(err)
	}
	bz, err := proto.Marshal(sm)
	if err != nil {
		panic(err)
	}
	return bz
}

// IsEmpty returns true if the State is equal to the empty State.
func (state State) IsEmpty() bool {
	return state.Validators == nil // XXX can't compare to Empty
}

// ToProto takes the local state type and returns the equivalent proto type
func (state *State) ToProto() (*cmtstate.State, error) {
	if state == nil {
		return nil, errors.New("state is nil")
	}

	sm := new(cmtstate.State)

	sm.Version = state.Version
	sm.ChainID = state.ChainID
	sm.InitialHeight = state.InitialHeight
	sm.LastBlockHeight = state.LastBlockHeight

	sm.LastBlockID = state.LastBlockID.ToProto()
	sm.LastBlockTime = state.LastBlockTime
	vals, err := state.Validators.ToProto()
	if err != nil {
		return nil, err
	}
	sm.Validators = vals

	nVals, err := state.NextValidators.ToProto()
	if err != nil {
		return nil, err
	}
	sm.NextValidators = nVals

	if state.LastBlockHeight >= 1 { // At Block 1 LastValidators is nil
		lVals, err := state.LastValidators.ToProto()
		if err != nil {
			return nil, err
		}
		sm.LastValidators = lVals
	}

	sm.LastHeightValidatorsChanged = state.LastHeightValidatorsChanged
	sm.ConsensusParams = state.ConsensusParams.ToProto()
	sm.LastHeightConsensusParamsChanged = state.LastHeightConsensusParamsChanged
	sm.LastResultsHash = state.LastResultsHash
	sm.AppHash = state.AppHash

	return sm, nil
}

// FromProto takes a state proto message & returns the local state type
func FromProto(pb *cmtstate.State) (*State, error) { //nolint:golint
	if pb == nil {
		return nil, errors.New("nil State")
	}

	state := new(State)

	state.Version = pb.Version
	state.ChainID = pb.ChainID
	state.InitialHeight = pb.InitialHeight

	bi, err := types.BlockIDFromProto(&pb.LastBlockID)
	if err != nil {
		return nil, err
	}
	state.LastBlockID = *bi
	state.LastBlockHeight = pb.LastBlockHeight
	state.LastBlockTime = pb.LastBlockTime

	vals, err := types.ValidatorSetFromProto(pb.Validators)
	if err != nil {
		return nil, err
	}
	state.Validators = vals

	nVals, err := types.ValidatorSetFromProto(pb.NextValidators)
	if err != nil {
		return nil, err
	}
	state.NextValidators = nVals

	if state.LastBlockHeight >= 1 { // At Block 1 LastValidators is nil
		lVals, err := types.ValidatorSetFromProto(pb.LastValidators)
		if err != nil {
			return nil, err
		}
		state.LastValidators = lVals
	} else {
		state.LastValidators = types.NewValidatorSet(nil)
	}

	state.LastHeightValidatorsChanged = pb.LastHeightValidatorsChanged
	state.ConsensusParams = types.ConsensusParamsFromProto(pb.ConsensusParams)
	state.LastHeightConsensusParamsChanged = pb.LastHeightConsensusParamsChanged
	state.LastResultsHash = pb.LastResultsHash
	state.AppHash = pb.AppHash

	return state, nil
}

//------------------------------------------------------------------------
// Create a block from the latest state

// MakeBlock builds a block from the current state with the given txs, commit,
// and evidence. Note it also takes a proposerAddress because the state does not
// track rounds, and hence does not know the correct proposer. TODO: fix this!
func (state State) MakeBlock(
	height int64,
	txs []types.Tx,
	lastCommit *types.Commit,
	evidence []types.Evidence,
	proposerAddress []byte,
) *types.Block {

	// Build base block with block data.
	block := types.MakeBlock(height, txs, lastCommit, evidence)

	// Set time.
	var timestamp time.Time
	if height == state.InitialHeight {
		timestamp = state.LastBlockTime // genesis time
	} else {
		timestamp = MedianTime(lastCommit, state.LastValidators)
	}

	// Fill rest of header with state data.
	block.Header.Populate(
		state.Version.Consensus, state.ChainID,
		timestamp, state.LastBlockID,
		state.Validators.Hash(), state.NextValidators.Hash(),
		state.ConsensusParams.Hash(), state.AppHash, state.LastResultsHash,
		proposerAddress,
	)

	return block
}

// MedianTime computes a median time for a given Commit (based on Timestamp field of votes messages) and the
// corresponding validator set. The computed time is always between timestamps of
// the votes sent by honest processes, i.e., a faulty processes can not arbitrarily increase or decrease the
// computed value.
func MedianTime(commit *types.Commit, validators *types.ValidatorSet) time.Time {
	weightedTimes := make([]*cmttime.WeightedTime, len(commit.Signatures))
	totalVotingPower := int64(0)

	for i, commitSig := range commit.Signatures {
		if commitSig.BlockIDFlag == types.BlockIDFlagAbsent {
			continue
		}
		_, validator := validators.GetByAddress(commitSig.ValidatorAddress)
		// If there's no condition, TestValidateBlockCommit panics; not needed normally.
		if validator != nil {
			totalVotingPower += validator.VotingPower
			weightedTimes[i] = cmttime.NewWeightedTime(commitSig.Timestamp, validator.VotingPower)
		}
	}

	return cmttime.WeightedMedian(weightedTimes, totalVotingPower)
}

//------------------------------------------------------------------------
// Genesis

// MakeGenesisStateFromFile reads and unmarshals state from the given
// file.
//
// Used during replay and in tests.
func MakeGenesisStateFromFile(genDocFile string) (State, error) {
	genDoc, err := MakeGenesisDocFromFile(genDocFile)
	if err != nil {
		return State{}, err
	}
	return MakeGenesisState(genDoc)
}

// MakeGenesisDocFromFile reads and unmarshals genesis doc from the given file.
func MakeGenesisDocFromFile(genDocFile string) (*types.GenesisDoc, error) {
	genDocJSON, err := os.ReadFile(genDocFile)
	if err != nil {
		return nil, fmt.Errorf("couldn't read GenesisDoc file: %v", err)
	}
	genDoc, err := types.GenesisDocFromJSON(genDocJSON)
	if err != nil {
		return nil, fmt.Errorf("error reading GenesisDoc: %v", err)
	}
	return genDoc, nil
}

// MakeGenesisState creates state from types.GenesisDoc.
func MakeGenesisState(genDoc *types.GenesisDoc) (State, error) {
	err := genDoc.ValidateAndComplete()
	if err != nil {
		return State{}, fmt.Errorf("error in genesis doc: %w", err)
	}

	var validatorSet, nextValidatorSet *types.ValidatorSet
	if genDoc.Validators == nil {
		validatorSet = types.NewValidatorSet(nil)
		nextValidatorSet = types.NewValidatorSet(nil)
	} else {
		validators := make([]*types.Validator, len(genDoc.Validators))
		for i, val := range genDoc.Validators {
			validators[i] = types.NewValidator(val.PubKey, val.Power)
		}
		validatorSet = types.NewValidatorSet(validators)
		nextValidatorSet = types.NewValidatorSet(validators).CopyIncrementProposerPriority(1)
	}

	return State{
		Version:       InitStateVersion,
		ChainID:       genDoc.ChainID,
		InitialHeight: genDoc.InitialHeight,

		LastBlockHeight: 0,
		LastBlockID:     types.BlockID{},
		LastBlockTime:   genDoc.GenesisTime,

		NextValidators:              nextValidatorSet,
		Validators:                  validatorSet,
		LastValidators:              types.NewValidatorSet(nil),
		LastHeightValidatorsChanged: genDoc.InitialHeight,

		ConsensusParams:                  *genDoc.ConsensusParams,
		LastHeightConsensusParamsChanged: genDoc.InitialHeight,

		AppHash: genDoc.AppHash,
	}, nil
}
