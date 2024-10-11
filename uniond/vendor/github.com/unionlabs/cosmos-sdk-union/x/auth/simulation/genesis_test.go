package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// TestRandomizedGenState tests the normal scenario of applying RandomizedGenState.
// Abonormal scenarios are not tested here.
func TestRandomizedGenState(t *testing.T) {
	registry := codectypes.NewInterfaceRegistry()
	types.RegisterInterfaces(registry)
	cdc := codec.NewProtoCodec(registry)
	cdcOpts := testutil.CodecOptions{}
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

	simulation.RandomizedGenState(&simState, simulation.RandomGenesisAccounts)

	var authGenesis types.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[types.ModuleName], &authGenesis)

	require.Equal(t, uint64(0x8c), authGenesis.Params.GetMaxMemoCharacters())
	require.Equal(t, uint64(0x2b6), authGenesis.Params.GetSigVerifyCostED25519())
	require.Equal(t, uint64(0x1ff), authGenesis.Params.GetSigVerifyCostSecp256k1())
	require.Equal(t, uint64(9), authGenesis.Params.GetTxSigLimit())
	require.Equal(t, uint64(5), authGenesis.Params.GetTxSizeCostPerByte())

	genAccounts, err := types.UnpackAccounts(authGenesis.Accounts)
	require.NoError(t, err)
	require.Len(t, genAccounts, 3)
	require.Equal(t, "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", genAccounts[2].GetAddress().String())
	require.Equal(t, uint64(0), genAccounts[2].GetAccountNumber())
	require.Equal(t, uint64(0), genAccounts[2].GetSequence())
}
