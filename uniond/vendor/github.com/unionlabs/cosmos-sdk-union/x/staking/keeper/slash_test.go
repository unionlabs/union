package keeper_test

import (
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/staking/testutil"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// tests Jail, Unjail
func (s *KeeperTestSuite) TestRevocation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valAddr := sdk.ValAddress(PKs[0].Address().Bytes())
	consAddr := sdk.ConsAddress(PKs[0].Address())
	validator := testutil.NewValidator(s.T(), valAddr, PKs[0])

	// initial state
	require.NoError(keeper.SetValidator(ctx, validator))
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))
	val, err := keeper.GetValidator(ctx, valAddr)
	require.NoError(err)
	require.False(val.IsJailed())

	// test jail
	require.NoError(keeper.Jail(ctx, consAddr))
	val, err = keeper.GetValidator(ctx, valAddr)
	require.NoError(err)
	require.True(val.IsJailed())

	// test unjail
	require.NoError(keeper.Unjail(ctx, consAddr))
	val, err = keeper.GetValidator(ctx, valAddr)
	require.NoError(err)
	require.False(val.IsJailed())
}

// tests Slash at a future height (must error)
func (s *KeeperTestSuite) TestSlashAtFutureHeight() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	consAddr := sdk.ConsAddress(PKs[0].Address())
	validator := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[0].Address().Bytes()), PKs[0])
	require.NoError(keeper.SetValidator(ctx, validator))
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	fraction := sdkmath.LegacyNewDecWithPrec(5, 1)
	_, err := keeper.Slash(ctx, consAddr, 1, 10, fraction)
	require.Error(err)
}
