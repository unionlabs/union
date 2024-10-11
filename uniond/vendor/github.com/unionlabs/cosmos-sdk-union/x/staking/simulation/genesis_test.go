package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/staking/simulation"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// TestRandomizedGenState tests the normal scenario of applying RandomizedGenState.
// Abonormal scenarios are not tested here.
func TestRandomizedGenState(t *testing.T) {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(interfaceRegistry)
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

	var stakingGenesis types.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[types.ModuleName], &stakingGenesis)

	require.Equal(t, uint32(207), stakingGenesis.Params.MaxValidators)
	require.Equal(t, uint32(7), stakingGenesis.Params.MaxEntries)
	require.Equal(t, uint32(0), stakingGenesis.Params.HistoricalEntries)
	require.Equal(t, "stake", stakingGenesis.Params.BondDenom)
	require.Equal(t, float64(238280), stakingGenesis.Params.UnbondingTime.Seconds())
	// check numbers of Delegations and Validators
	require.Len(t, stakingGenesis.Delegations, 3)
	require.Len(t, stakingGenesis.Validators, 3)
	// check Delegations
	require.Equal(t, "cosmos1tnh2q55v8wyygtt9srz5safamzdengsnqeycj3", stakingGenesis.Delegations[0].DelegatorAddress)
	require.Equal(t, "cosmosvaloper1tnh2q55v8wyygtt9srz5safamzdengsn9dsd7z", stakingGenesis.Delegations[0].ValidatorAddress)
	require.Equal(t, "1000.000000000000000000", stakingGenesis.Delegations[0].Shares.String())
	// check validators
	require.Equal(t, "cosmosvaloper1ghekyjucln7y67ntx7cf27m9dpuxxemnsvnaes", stakingGenesis.Validators[2].GetOperator())
	require.Equal(t, []byte{0xa, 0x20, 0x51, 0xde, 0xbd, 0xe8, 0xfa, 0xdf, 0x4e, 0xfc, 0x33, 0xa5, 0x16, 0x94, 0xf6, 0xee, 0xd3, 0x69, 0x7a, 0x7a, 0x1c, 0x2d, 0x50, 0xb6, 0x2, 0xf7, 0x16, 0x4e, 0x66, 0x9f, 0xff, 0x38, 0x91, 0x9b}, stakingGenesis.Validators[2].ConsensusPubkey.Value)
	require.Equal(t, false, stakingGenesis.Validators[2].Jailed)
	require.Equal(t, "BOND_STATUS_UNBONDED", stakingGenesis.Validators[2].Status.String())
	require.Equal(t, "1000", stakingGenesis.Validators[2].Tokens.String())
	require.Equal(t, "1000.000000000000000000", stakingGenesis.Validators[2].DelegatorShares.String())
	require.Equal(t, "0.760000000000000000", stakingGenesis.Validators[2].Commission.CommissionRates.Rate.String())
	require.Equal(t, "0.760000000000000000", stakingGenesis.Validators[2].Commission.CommissionRates.MaxRate.String())
	require.Equal(t, "0.312739151653465930", stakingGenesis.Validators[2].Commission.CommissionRates.MaxChangeRate.String())
	require.Equal(t, "1", stakingGenesis.Validators[2].MinSelfDelegation.String())
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
			}, "invalid memory address or nil pointer dereference"},
		{
			// panic => reason: numBonded != len(Accnounts)
			module.SimulationState{
				AppParams:    make(simtypes.AppParams),
				Cdc:          cdc,
				Rand:         r,
				NumBonded:    4,
				Accounts:     simtypes.RandomAccounts(r, 3),
				InitialStake: sdkmath.NewInt(1000),
				GenState:     make(map[string]json.RawMessage),
			}, "invalid memory address or nil pointer dereference",
		},
	}

	for _, tt := range tests {
		require.Panicsf(t, func() { simulation.RandomizedGenState(&tt.simState) }, tt.panicMsg)
	}
}
