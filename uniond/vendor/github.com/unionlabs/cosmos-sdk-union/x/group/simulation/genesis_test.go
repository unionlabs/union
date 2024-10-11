package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/bank"
	"cosmossdk.io/x/group"
	groupmodule "cosmossdk.io/x/group/module"
	"cosmossdk.io/x/group/simulation"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/types/module"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestRandomizedGenState(t *testing.T) {
	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, groupmodule.AppModule{}, bank.AppModule{})
	cdc := encodingConfig.Codec
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
		Accounts:       simtypes.RandomAccounts(r, 3),
		InitialStake:   sdkmath.NewInt(1000),
		GenState:       make(map[string]json.RawMessage),
	}

	simulation.RandomizedGenState(&simState)
	var groupGenesis group.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[group.ModuleName], &groupGenesis)

	require.Equal(t, int(groupGenesis.GroupSeq), len(simState.Accounts))
	require.Len(t, groupGenesis.Groups, len(simState.Accounts))
	require.Len(t, groupGenesis.GroupMembers, len(simState.Accounts))
	require.Equal(t, int(groupGenesis.GroupPolicySeq), len(simState.Accounts))
	require.Len(t, groupGenesis.GroupPolicies, len(simState.Accounts))
	require.Equal(t, int(groupGenesis.ProposalSeq), len(simState.Accounts))
	require.Len(t, groupGenesis.Proposals, len(simState.Accounts))
	require.Len(t, groupGenesis.Votes, len(simState.Accounts))
}
