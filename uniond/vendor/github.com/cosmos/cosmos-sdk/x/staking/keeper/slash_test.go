package keeper_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/testutil"
)

// tests Jail, Unjail
func (s *KeeperTestSuite) TestRevocation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valAddr := sdk.ValAddress(PKs[0].Address().Bytes())
	consAddr := sdk.ConsAddress(PKs[0].Address())
	validator := testutil.NewValidator(s.T(), valAddr, PKs[0])

	// initial state
	keeper.SetValidator(ctx, validator)
	keeper.SetValidatorByConsAddr(ctx, validator)
	val, found := keeper.GetValidator(ctx, valAddr)
	require.True(found)
	require.False(val.IsJailed())

	// test jail
	keeper.Jail(ctx, consAddr)
	val, found = keeper.GetValidator(ctx, valAddr)
	require.True(found)
	require.True(val.IsJailed())

	// test unjail
	keeper.Unjail(ctx, consAddr)
	val, found = keeper.GetValidator(ctx, valAddr)
	require.True(found)
	require.False(val.IsJailed())
}

// tests Slash at a future height (must panic)
func (s *KeeperTestSuite) TestSlashAtFutureHeight() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	consAddr := sdk.ConsAddress(PKs[0].Address())
	validator := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[0].Address().Bytes()), PKs[0])
	keeper.SetValidator(ctx, validator)
	err := keeper.SetValidatorByConsAddr(ctx, validator)
	require.NoError(err)

	fraction := sdk.NewDecWithPrec(5, 1)
	require.Panics(func() { keeper.Slash(ctx, consAddr, 1, 10, fraction) })
}
