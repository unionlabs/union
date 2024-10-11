package keeper_test

import (
	"cosmossdk.io/x/feegrant"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	invalidGrantee = "invalid-grantee"
	invalidGranter = "invalid-granter"
)

func (suite *KeeperTestSuite) TestFeeAllowance() {
	testCases := []struct {
		name      string
		req       *feegrant.QueryAllowanceRequest
		expectErr bool
		preRun    func()
		postRun   func(_ *feegrant.QueryAllowanceResponse)
	}{
		{
			"nil request",
			nil,
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"fail: invalid granter",
			&feegrant.QueryAllowanceRequest{
				Granter: invalidGranter,
				Grantee: suite.encodedAddrs[0],
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"fail: invalid grantee",
			&feegrant.QueryAllowanceRequest{
				Granter: suite.encodedAddrs[0],
				Grantee: invalidGrantee,
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"fail: no grants",
			&feegrant.QueryAllowanceRequest{
				Granter: suite.encodedAddrs[0],
				Grantee: suite.encodedAddrs[1],
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"non existed grant",
			&feegrant.QueryAllowanceRequest{
				Granter: invalidGranter,
				Grantee: invalidGrantee,
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"valid query: expect single grant",
			&feegrant.QueryAllowanceRequest{
				Granter: suite.encodedAddrs[0],
				Grantee: suite.encodedAddrs[1],
			},
			false,
			func() {
				suite.grantFeeAllowance(suite.addrs[0], suite.addrs[1])
			},
			func(response *feegrant.QueryAllowanceResponse) {
				suite.Require().Equal(response.Allowance.Granter, suite.encodedAddrs[0])
				suite.Require().Equal(response.Allowance.Grantee, suite.encodedAddrs[1])
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			tc.preRun()
			resp, err := suite.feegrantKeeper.Allowance(suite.ctx, tc.req)
			if tc.expectErr {
				suite.Require().Error(err)
			} else {
				suite.Require().NoError(err)
				tc.postRun(resp)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestFeeAllowances() {
	testCases := []struct {
		name      string
		req       *feegrant.QueryAllowancesRequest
		expectErr bool
		preRun    func()
		postRun   func(_ *feegrant.QueryAllowancesResponse)
	}{
		{
			"nil request",
			nil,
			true,
			func() {},
			func(*feegrant.QueryAllowancesResponse) {},
		},
		{
			"fail: invalid grantee",
			&feegrant.QueryAllowancesRequest{
				Grantee: invalidGrantee,
			},
			true,
			func() {},
			func(*feegrant.QueryAllowancesResponse) {},
		},
		{
			"no grants",
			&feegrant.QueryAllowancesRequest{
				Grantee: suite.encodedAddrs[1],
			},
			false,
			func() {},
			func(resp *feegrant.QueryAllowancesResponse) {
				suite.Require().Equal(len(resp.Allowances), 0)
			},
		},
		{
			"valid query: expect single grant",
			&feegrant.QueryAllowancesRequest{
				Grantee: suite.encodedAddrs[1],
			},
			false,
			func() {
				suite.grantFeeAllowance(suite.addrs[0], suite.addrs[1])
			},
			func(resp *feegrant.QueryAllowancesResponse) {
				suite.Require().Equal(len(resp.Allowances), 1)
				suite.Require().Equal(resp.Allowances[0].Granter, suite.encodedAddrs[0])
				suite.Require().Equal(resp.Allowances[0].Grantee, suite.encodedAddrs[1])
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			tc.preRun()
			resp, err := suite.feegrantKeeper.Allowances(suite.ctx, tc.req)
			if tc.expectErr {
				suite.Require().Error(err)
			} else {
				suite.Require().NoError(err)
				tc.postRun(resp)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestFeeAllowancesByGranter() {
	testCases := []struct {
		name      string
		req       *feegrant.QueryAllowancesByGranterRequest
		expectErr bool
		preRun    func()
		postRun   func(_ *feegrant.QueryAllowancesByGranterResponse)
	}{
		{
			"nil request",
			nil,
			true,
			func() {},
			func(*feegrant.QueryAllowancesByGranterResponse) {},
		},
		{
			"fail: invalid grantee",
			&feegrant.QueryAllowancesByGranterRequest{
				Granter: invalidGrantee,
			},
			true,
			func() {},
			func(*feegrant.QueryAllowancesByGranterResponse) {},
		},
		{
			"no grants",
			&feegrant.QueryAllowancesByGranterRequest{
				Granter: suite.encodedAddrs[0],
			},
			false,
			func() {},
			func(resp *feegrant.QueryAllowancesByGranterResponse) {
				suite.Require().Equal(len(resp.Allowances), 0)
			},
		},
		{
			"valid query: expect single grant",
			&feegrant.QueryAllowancesByGranterRequest{
				Granter: suite.encodedAddrs[0],
			},
			false,
			func() {
				suite.grantFeeAllowance(suite.addrs[0], suite.addrs[1])

				// adding this allowance to check whether the pagination working fine.
				suite.grantFeeAllowance(suite.addrs[1], suite.addrs[2])
			},
			func(resp *feegrant.QueryAllowancesByGranterResponse) {
				suite.Require().Equal(len(resp.Allowances), 1)
				suite.Require().Equal(resp.Allowances[0].Granter, suite.encodedAddrs[0])
				suite.Require().Equal(resp.Allowances[0].Grantee, suite.encodedAddrs[1])
				suite.Require().Equal(resp.Pagination.Total, uint64(1))
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			tc.preRun()
			resp, err := suite.feegrantKeeper.AllowancesByGranter(suite.ctx, tc.req)
			if tc.expectErr {
				suite.Require().Error(err)
			} else {
				suite.Require().NoError(err)
				tc.postRun(resp)
			}
		})
	}
}

func (suite *KeeperTestSuite) grantFeeAllowance(granter, grantee sdk.AccAddress) {
	exp := suite.ctx.HeaderInfo().Time.AddDate(1, 0, 0)
	err := suite.feegrantKeeper.GrantAllowance(suite.ctx, granter, grantee, &feegrant.BasicAllowance{
		SpendLimit: sdk.NewCoins(sdk.NewInt64Coin("atom", 555)),
		Expiration: &exp,
	})
	suite.Require().NoError(err)
}
