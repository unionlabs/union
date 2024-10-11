package keeper_test

import (
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestQueryInterchainAccount() {
	var req *types.QueryInterchainAccountRequest

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"empty request",
			func() {
				req = nil
			},
			false,
		},
		{
			"empty owner address",
			func() {
				req.Owner = ""
			},
			false,
		},
		{
			"invalid connection, account address not found",
			func() {
				req.ConnectionId = "invalid-connection-id"
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			path := NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, ibctesting.TestAccAddress)
			suite.Require().NoError(err)

			req = &types.QueryInterchainAccountRequest{
				ConnectionId: ibctesting.FirstConnectionID,
				Owner:        ibctesting.TestAccAddress,
			}

			tc.malleate()

			res, err := suite.chainA.GetSimApp().ICAControllerKeeper.InterchainAccount(suite.chainA.GetContext(), req)

			if tc.expPass {
				expAddress, exists := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(exists)

				suite.Require().NoError(err)
				suite.Require().Equal(expAddress, res.Address)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryParams() {
	ctx := suite.chainA.GetContext()
	expParams := types.DefaultParams()
	res, _ := suite.chainA.GetSimApp().ICAControllerKeeper.Params(ctx, &types.QueryParamsRequest{})
	suite.Require().Equal(&expParams, res.Params)
}
