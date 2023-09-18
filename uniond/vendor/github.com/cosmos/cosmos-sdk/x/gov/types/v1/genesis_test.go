package v1_test

import (
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/stretchr/testify/require"
)

func TestEmptyGenesis(t *testing.T) {
	state1 := v1.GenesisState{}
	require.True(t, state1.Empty())

	state2 := v1.DefaultGenesisState()
	require.False(t, state2.Empty())
}

func TestValidateGenesis(t *testing.T) {
	params := v1.DefaultParams()

	testCases := []struct {
		name         string
		genesisState func() *v1.GenesisState
		expErr       bool
	}{
		{
			name: "valid",
			genesisState: func() *v1.GenesisState {
				return v1.NewGenesisState(v1.DefaultStartingProposalID, params)
			},
		},
		{
			name: "invalid StartingProposalId",
			genesisState: func() *v1.GenesisState {
				return v1.NewGenesisState(0, params)
			},
			expErr: true,
		},
		{
			name: "invalid min deposit",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.MinDeposit = sdk.Coins{{
					Denom:  sdk.DefaultBondDenom,
					Amount: sdk.NewInt(-100),
				}}

				return v1.NewGenesisState(0, params1)
			},
			expErr: true,
		},
		{
			name: "invalid max deposit period",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.MaxDepositPeriod = nil

				return v1.NewGenesisState(0, params1)
			},
			expErr: true,
		},
		{
			name: "invalid quorum",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.Quorum = "2"

				return v1.NewGenesisState(0, params1)
			},
			expErr: true,
		},
		{
			name: "invalid threshold",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.Threshold = "2"

				return v1.NewGenesisState(0, params1)
			},
			expErr: true,
		},
		{
			name: "invalid veto threshold",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.VetoThreshold = "2"

				return v1.NewGenesisState(0, params1)
			},
			expErr: true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			err := v1.ValidateGenesis(tc.genesisState())
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
