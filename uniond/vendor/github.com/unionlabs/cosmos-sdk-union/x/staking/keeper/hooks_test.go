package keeper_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestHookAfterConsensusPubKeyUpdate() {
	stKeeper := s.stakingKeeper
	ctx := s.ctx
	require := s.Require()

	rotationFee := sdk.NewInt64Coin("stake", 1000000)
	err := stKeeper.Hooks().AfterConsensusPubKeyUpdate(ctx, PKs[0], PKs[1], rotationFee)
	require.NoError(err)
}
