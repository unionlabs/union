package simulation

import (
	"context"
	"encoding/json"
	"math/rand"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/types"

	"cosmossdk.io/core/address"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

const (
	// Minimum time per block
	minTimePerBlock int64 = 10000 / 2

	// Maximum time per block
	maxTimePerBlock int64 = 10000
)

// TODO: explain transitional matrix usage
var (
	// Currently there are 3 different liveness types,
	// fully online, spotty connection, offline.
	defaultLivenessTransitionMatrix, _ = CreateTransitionMatrix([][]int{
		{90, 20, 1},
		{10, 50, 5},
		{0, 10, 1000},
	})

	// 3 states: rand in range [0, 4*provided blocksize],
	// rand in range [0, 2 * provided blocksize], 0
	defaultBlockSizeTransitionMatrix, _ = CreateTransitionMatrix([][]int{
		{85, 5, 0},
		{15, 92, 1},
		{0, 3, 99},
	})
)

// Params define the parameters necessary for running the simulations
type Params struct {
	pastEvidenceFraction      float64
	numKeys                   int
	evidenceFraction          float64
	initialLivenessWeightings []int
	livenessTransitionMatrix  simulation.TransitionMatrix
	blockSizeTransitionMatrix simulation.TransitionMatrix
}

func (p Params) PastEvidenceFraction() float64 {
	return p.pastEvidenceFraction
}

func (p Params) NumKeys() int {
	return p.numKeys
}

func (p Params) EvidenceFraction() float64 {
	return p.evidenceFraction
}

func (p Params) InitialLivenessWeightings() []int {
	return p.initialLivenessWeightings
}

func (p Params) LivenessTransitionMatrix() simulation.TransitionMatrix {
	return p.livenessTransitionMatrix
}

func (p Params) BlockSizeTransitionMatrix() simulation.TransitionMatrix {
	return p.blockSizeTransitionMatrix
}

// RandomParams returns random simulation parameters
func RandomParams(r *rand.Rand) Params {
	return Params{
		pastEvidenceFraction:      r.Float64(),
		numKeys:                   simulation.RandIntBetween(r, 2, 2500), // number of accounts created for the simulation
		evidenceFraction:          r.Float64(),
		initialLivenessWeightings: []int{simulation.RandIntBetween(r, 1, 80), r.Intn(10), r.Intn(10)},
		livenessTransitionMatrix:  defaultLivenessTransitionMatrix,
		blockSizeTransitionMatrix: defaultBlockSizeTransitionMatrix,
	}
}

// Legacy param change proposals

// LegacyParamChange defines the object used for simulating parameter change proposals
type LegacyParamChange struct {
	subspace string
	key      string
	simValue simulation.SimValFn
}

func (spc LegacyParamChange) Subspace() string {
	return spc.subspace
}

func (spc LegacyParamChange) Key() string {
	return spc.key
}

func (spc LegacyParamChange) SimValue() simulation.SimValFn {
	return spc.simValue
}

// ComposedKey creates a new composed key for the legacy param change proposal
func (spc LegacyParamChange) ComposedKey() string {
	return spc.Subspace() + "/" + spc.Key()
}

// NewSimLegacyParamChange creates a new LegacyParamChange instance
func NewSimLegacyParamChange(subspace, key string, simVal simulation.SimValFn) simulation.LegacyParamChange {
	return LegacyParamChange{
		subspace: subspace,
		key:      key,
		simValue: simVal,
	}
}

// Proposal Msgs

// WeightedProposalMsg defines a common struct for proposal msgs defined by external modules (i.e outside gov)
type WeightedProposalMsg struct {
	appParamsKey    string                     // key used to retrieve the value of the weight from the simulation application params
	defaultWeight   int                        // default weight
	msgSimulatorFnX simulation.MsgSimulatorFnX // msg simulator function
}

// Deprecated: use NewWeightedProposalMsgX instead
func NewWeightedProposalMsg(appParamsKey string, defaultWeight int, msgSimulatorFn simulation.MsgSimulatorFn) simulation.WeightedProposalMsg {
	return NewWeightedProposalMsgX(appParamsKey, defaultWeight, func(_ context.Context, r *rand.Rand, accs []simulation.Account, cdc address.Codec) (sdk.Msg, error) {
		return msgSimulatorFn(r, accs, cdc)
	})
}

func NewWeightedProposalMsgX(appParamsKey string, defaultWeight int, msgSimulatorFn simulation.MsgSimulatorFnX) simulation.WeightedProposalMsg {
	return &WeightedProposalMsg{appParamsKey: appParamsKey, defaultWeight: defaultWeight, msgSimulatorFnX: msgSimulatorFn}
}

func (w WeightedProposalMsg) AppParamsKey() string {
	return w.appParamsKey
}

func (w WeightedProposalMsg) DefaultWeight() int {
	return w.defaultWeight
}

func (w WeightedProposalMsg) MsgSimulatorFn() simulation.MsgSimulatorFnX {
	return w.msgSimulatorFnX
}

// Legacy Proposal Content

// WeightedProposalContent defines a common struct for proposal content defined by external modules (i.e outside gov)
//
//nolint:staticcheck // used for legacy testing
type WeightedProposalContent struct {
	appParamsKey       string                        // key used to retrieve the value of the weight from the simulation application params
	defaultWeight      int                           // default weight
	contentSimulatorFn simulation.ContentSimulatorFn // content simulator function
}

func NewWeightedProposalContent(appParamsKey string, defaultWeight int, contentSimulatorFn simulation.ContentSimulatorFn) simulation.WeightedProposalContent { //nolint:staticcheck // used for legacy testing
	return &WeightedProposalContent{appParamsKey: appParamsKey, defaultWeight: defaultWeight, contentSimulatorFn: contentSimulatorFn}
}

func (w WeightedProposalContent) AppParamsKey() string {
	return w.appParamsKey
}

func (w WeightedProposalContent) DefaultWeight() int {
	return w.defaultWeight
}

func (w WeightedProposalContent) ContentSimulatorFn() simulation.ContentSimulatorFn { //nolint:staticcheck // used for legacy testing
	return w.contentSimulatorFn
}

// Consensus Params

// RandomConsensusParams returns random simulation consensus parameters, it extracts the Evidence from the Staking genesis state.
func RandomConsensusParams(r *rand.Rand, appState json.RawMessage, cdc codec.JSONCodec, maxGas int64) *cmtproto.ConsensusParams {
	var genesisState map[string]json.RawMessage
	err := json.Unmarshal(appState, &genesisState)
	if err != nil {
		panic(err)
	}

	stakingGenesisState := stakingtypes.GetGenesisStateFromAppState(cdc, genesisState)
	consensusParams := &cmtproto.ConsensusParams{
		Block: &cmtproto.BlockParams{
			MaxBytes: int64(simulation.RandIntBetween(r, 20000000, 30000000)),
			MaxGas:   maxGas,
		},
		Validator: &cmtproto.ValidatorParams{
			PubKeyTypes: []string{types.ABCIPubKeyTypeEd25519},
		},
		Evidence: &cmtproto.EvidenceParams{
			MaxAgeNumBlocks: int64(stakingGenesisState.Params.UnbondingTime / AverageBlockTime),
			MaxAgeDuration:  stakingGenesisState.Params.UnbondingTime,
		},
	}
	return consensusParams
}
