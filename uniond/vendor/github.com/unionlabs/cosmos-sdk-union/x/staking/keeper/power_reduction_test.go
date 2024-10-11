package keeper_test

import (
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestTokensToConsensusPower() {
	s.Require().Equal(int64(0), s.stakingKeeper.TokensToConsensusPower(s.ctx, sdk.DefaultPowerReduction.Sub(sdkmath.NewInt(1))))
	s.Require().Equal(int64(1), s.stakingKeeper.TokensToConsensusPower(s.ctx, sdk.DefaultPowerReduction))
}

func (s *KeeperTestSuite) TestTokensFromConsensusPower() {
	s.Require().Equal(sdkmath.NewInt(0), s.stakingKeeper.TokensFromConsensusPower(s.ctx, 0))
	s.Require().Equal(sdk.DefaultPowerReduction, s.stakingKeeper.TokensFromConsensusPower(s.ctx, 1))
}
