package keeper_test

import (
	"fmt"

	"cosmossdk.io/x/params/types"
	"cosmossdk.io/x/params/types/proposal"
)

func (suite *KeeperTestSuite) TestGRPCQueryParams() {
	var (
		req      *proposal.QueryParamsRequest
		expValue string
		space    types.Subspace
	)
	key := []byte("key")

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &proposal.QueryParamsRequest{}
			},
			false,
		},
		{
			"invalid request with subspace not found",
			func() {
				req = &proposal.QueryParamsRequest{Subspace: "test"}
			},
			false,
		},
		{
			"invalid request with subspace and key not found",
			func() {
				req = &proposal.QueryParamsRequest{Subspace: "test", Key: "key"}
			},
			false,
		},
		{
			"success",
			func() {
				space = suite.paramsKeeper.Subspace("test").
					WithKeyTable(types.NewKeyTable(types.NewParamSetPair(key, paramJSON{}, validateNoOp)))
				req = &proposal.QueryParamsRequest{Subspace: "test", Key: "key"}
				expValue = ""
			},
			true,
		},
		{
			"update value success",
			func() {
				err := space.Update(suite.ctx, key, []byte(`{"param1":"10241024"}`))
				suite.Require().NoError(err)
				req = &proposal.QueryParamsRequest{Subspace: "test", Key: "key"}
				expValue = `{"param1":"10241024"}`
			},
			true,
		},
	}

	suite.SetupTest()

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			tc.malleate()

			res, err := suite.queryClient.Params(suite.ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(expValue, res.Param.Value)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(res)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQuerySubspaces() {
	// NOTE: Each subspace will not have any keys that we can check against
	// because InitGenesis has not been called during app construction.
	resp, err := suite.queryClient.Subspaces(suite.ctx, &proposal.QuerySubspacesRequest{})
	suite.Require().NoError(err)
	suite.Require().NotNil(resp)

	spaces := make([]string, len(resp.Subspaces))
	i := 0
	for _, ss := range resp.Subspaces {
		spaces[i] = ss.Subspace
		i++
	}

	// require the response contains a few subspaces we know exist
	suite.Require().Contains(spaces, "bank")
	suite.Require().Contains(spaces, "staking")
}
