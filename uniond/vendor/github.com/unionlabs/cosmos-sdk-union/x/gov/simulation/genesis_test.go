package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/gov/simulation"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// TestRandomizedGenState tests the normal scenario of applying RandomizedGenState.
// Abnormal scenarios are not tested here.
func TestRandomizedGenState(t *testing.T) {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)
	cdcOpts := codectestutil.CodecOptions{}

	s := rand.NewSource(1)
	r := rand.New(s)

	simState := module.SimulationState{
		AppParams:      make(simtypes.AppParams),
		Cdc:            cdc,
		AddressCodec:   cdcOpts.GetAddressCodec(),
		ValidatorCodec: cdcOpts.GetValidatorCodec(),
		Rand:           r,
		NumBonded:      3,
		BondDenom:      sdk.DefaultBondDenom,
		Accounts:       simtypes.RandomAccounts(r, 3),
		InitialStake:   sdkmath.NewInt(1000),
		GenState:       make(map[string]json.RawMessage),
	}

	simulation.RandomizedGenState(&simState)

	var govGenesis v1.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[types.ModuleName], &govGenesis)

	const (
		tallyQuorum             = "0.387000000000000000"
		tallyYesQuorum          = "0.449000000000000000"
		tallyExpeditedQuorum    = "0.457000000000000000"
		tallyThreshold          = "0.479000000000000000"
		tallyExpeditedThreshold = "0.545000000000000000"
		tallyVetoThreshold      = "0.280000000000000000"
		minInitialDepositDec    = "0.880000000000000000"
		proposalCancelMaxPeriod = "0.110000000000000000"
	)

	require.Equal(t, "272stake", govGenesis.Params.MinDeposit[0].String())
	require.Equal(t, "800stake", govGenesis.Params.ExpeditedMinDeposit[0].String())
	require.Equal(t, "41h11m36s", govGenesis.Params.MaxDepositPeriod.String())
	require.Equal(t, float64(291928), govGenesis.Params.VotingPeriod.Seconds())
	require.Equal(t, float64(33502), govGenesis.Params.ExpeditedVotingPeriod.Seconds())
	require.Equal(t, tallyQuorum, govGenesis.Params.Quorum)
	require.Equal(t, tallyYesQuorum, govGenesis.Params.YesQuorum)
	require.Equal(t, tallyExpeditedQuorum, govGenesis.Params.ExpeditedQuorum)
	require.Equal(t, tallyThreshold, govGenesis.Params.Threshold)
	require.Equal(t, tallyExpeditedThreshold, govGenesis.Params.ExpeditedThreshold)
	require.Equal(t, tallyVetoThreshold, govGenesis.Params.VetoThreshold)
	require.Equal(t, proposalCancelMaxPeriod, govGenesis.Params.ProposalCancelMaxPeriod)
	require.Equal(t, uint64(0x28), govGenesis.StartingProposalId)
	require.Equal(t, []*v1.Deposit{}, govGenesis.Deposits)
	require.Equal(t, []*v1.Vote{}, govGenesis.Votes)
	require.Equal(t, []*v1.Proposal{}, govGenesis.Proposals)
}

// TestRandomizedGenState1 tests abnormal scenarios of applying RandomizedGenState.
func TestRandomizedGenState1(t *testing.T) {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	s := rand.NewSource(1)
	r := rand.New(s)
	// all these tests will panic
	tests := []struct {
		simState module.SimulationState
		panicMsg string
	}{
		{ // panic => reason: incomplete initialization of the simState
			module.SimulationState{}, "invalid memory address or nil pointer dereference"},
		{ // panic => reason: incomplete initialization of the simState
			module.SimulationState{
				AppParams: make(simtypes.AppParams),
				Cdc:       cdc,
				Rand:      r,
			}, "assignment to entry in nil map"},
	}

	for _, tt := range tests {
		require.Panicsf(t, func() { simulation.RandomizedGenState(&tt.simState) }, tt.panicMsg)
	}
}
