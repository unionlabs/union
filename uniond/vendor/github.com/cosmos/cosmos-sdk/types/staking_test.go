package types_test

import (
	"testing"

	"github.com/stretchr/testify/suite"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type stakingTestSuite struct {
	suite.Suite
}

func TestStakingTestSuite(t *testing.T) {
	suite.Run(t, new(stakingTestSuite))
}

func (s *stakingTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *stakingTestSuite) TestTokensToConsensusPower() {
	s.Require().Equal(int64(0), sdk.TokensToConsensusPower(sdk.NewInt(999_999), sdk.DefaultPowerReduction))
	s.Require().Equal(int64(1), sdk.TokensToConsensusPower(sdk.NewInt(1_000_000), sdk.DefaultPowerReduction))
}
