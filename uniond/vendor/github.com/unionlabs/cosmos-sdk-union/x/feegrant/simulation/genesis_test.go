package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/module"
	"cosmossdk.io/x/feegrant/simulation"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	moduletypes "github.com/cosmos/cosmos-sdk/types/module"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestRandomizedGenState(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})
	s := rand.NewSource(1)
	r := rand.New(s)
	cdcOpts := codectestutil.CodecOptions{}
	accounts := simtypes.RandomAccounts(r, 3)

	simState := moduletypes.SimulationState{
		AppParams:      make(simtypes.AppParams),
		Cdc:            encCfg.Codec,
		AddressCodec:   cdcOpts.GetAddressCodec(),
		ValidatorCodec: cdcOpts.GetValidatorCodec(),
		Rand:           r,
		NumBonded:      3,
		Accounts:       accounts,
		InitialStake:   math.NewInt(1000),
		GenState:       make(map[string]json.RawMessage),
	}

	simulation.RandomizedGenState(&simState)
	var feegrantGenesis feegrant.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[feegrant.ModuleName], &feegrantGenesis)

	require.Len(t, feegrantGenesis.Allowances, len(accounts)-1)
}
