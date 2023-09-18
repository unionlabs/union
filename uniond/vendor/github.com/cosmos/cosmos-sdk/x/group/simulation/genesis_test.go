package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	sdkmath "cosmossdk.io/math"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/simulation"
	"github.com/cosmos/cosmos-sdk/x/group/testutil"
)

func TestRandomizedGenState(t *testing.T) {
	var cdc codec.Codec
	err := depinject.Inject(testutil.AppConfig, &cdc)
	require.NoError(t, err)

	s := rand.NewSource(1)
	r := rand.New(s)

	simState := module.SimulationState{
		AppParams:    make(simtypes.AppParams),
		Cdc:          cdc,
		Rand:         r,
		NumBonded:    3,
		Accounts:     simtypes.RandomAccounts(r, 3),
		InitialStake: sdkmath.NewInt(1000),
		GenState:     make(map[string]json.RawMessage),
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
