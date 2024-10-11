package keeper_test

import (
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// TestMigrateDefaultParams tests the migration for the channel params
func (suite *KeeperTestSuite) TestMigrateDefaultParams() {
	testCases := []struct {
		name           string
		expectedParams channeltypes.Params
	}{
		{
			"success: default params",
			channeltypes.DefaultParams(),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			ctx := suite.chainA.GetContext()
			migrator := keeper.NewMigrator(suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper)
			err := migrator.MigrateParams(ctx)
			suite.Require().NoError(err)

			params := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetParams(ctx)
			suite.Require().Equal(tc.expectedParams, params)
		})
	}
}
