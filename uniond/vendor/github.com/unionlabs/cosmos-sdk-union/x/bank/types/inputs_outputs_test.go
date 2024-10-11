package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestValidateInputOutputs(t *testing.T) {
	tests := []struct {
		name    string
		input   types.Input
		outputs []types.Output
		expErr  bool
	}{
		{
			"valid input and outputs",
			types.NewInput("cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t", sdk.Coins{sdk.NewInt64Coin("uatom", 1)}),
			[]types.Output{
				{
					Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
					Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
				},
			},
			false,
		},
		{
			"empty input",
			types.Input{},
			[]types.Output{
				{
					Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
					Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
				},
			},
			true,
		},
		{
			"input and output mismatch",
			types.NewInput("cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t", sdk.Coins{sdk.NewInt64Coin("uatom", 1)}),
			[]types.Output{
				{
					Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
					Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 2)},
				},
			},
			true,
		},
		{
			"invalid output",
			types.NewInput("cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t", sdk.Coins{sdk.NewInt64Coin("uatom", 1)}),
			[]types.Output{
				{
					Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
					Coins:   sdk.Coins{sdk.Coin{Denom: "", Amount: math.NewInt(1)}},
				},
			},
			true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := types.ValidateInputOutputs(tc.input, tc.outputs)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
