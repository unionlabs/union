package keeper_test

import (
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// TestMigrateParams tests that the params for the connection are properly migrated
func (suite *KeeperTestSuite) TestMigrateParams() {
	testCases := []struct {
		name           string
		malleate       func()
		expectedParams types.Params
	}{
		{
			"success: default params",
			func() {
				params := types.DefaultParams()
				subspace := suite.chainA.GetSimApp().GetSubspace(ibcexported.ModuleName)
				subspace.SetParamSet(suite.chainA.GetContext(), &params)
			},
			types.DefaultParams(),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			tc.malleate()

			ctx := suite.chainA.GetContext()
			migrator := keeper.NewMigrator(suite.chainA.GetSimApp().IBCKeeper.ConnectionKeeper)
			err := migrator.MigrateParams(ctx)
			suite.Require().NoError(err)

			params := suite.chainA.GetSimApp().IBCKeeper.ConnectionKeeper.GetParams(ctx)
			suite.Require().Equal(tc.expectedParams, params)
		})
	}
}
