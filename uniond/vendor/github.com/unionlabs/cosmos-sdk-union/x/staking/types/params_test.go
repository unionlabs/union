package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestParamsEqual(t *testing.T) {
	p1 := types.DefaultParams()
	p2 := types.DefaultParams()

	ok := p1.Equal(p2)
	require.True(t, ok)

	p2.UnbondingTime = 60 * 60 * 24 * 2
	p2.BondDenom = "soup"

	ok = p1.Equal(p2)
	require.False(t, ok)
}

func TestValidateParams(t *testing.T) {
	params := types.DefaultParams()

	coinZero := sdk.NewInt64Coin("stake", 0)

	// default params have no error
	require.NoError(t, params.Validate())

	// validate min commission rate
	params.MinCommissionRate = math.LegacyNewDec(-1)
	require.Error(t, params.Validate())

	params.MinCommissionRate = math.LegacyNewDec(2)
	require.Error(t, params.Validate())

	// reset params to default
	params = types.DefaultParams()

	// check keyRotationFee
	params.KeyRotationFee = coinZero
	require.Error(t, params.Validate())
}
