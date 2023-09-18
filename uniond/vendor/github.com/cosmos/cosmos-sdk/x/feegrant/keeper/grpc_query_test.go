package keeper_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
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
				Granter: "invalid_granter",
				Grantee: suite.addrs[0].String(),
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"fail: invalid grantee",
			&feegrant.QueryAllowanceRequest{
				Granter: suite.addrs[0].String(),
				Grantee: "invalid_grantee",
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"fail: no grants",
			&feegrant.QueryAllowanceRequest{
				Granter: suite.addrs[0].String(),
				Grantee: suite.addrs[1].String(),
			},
			true,
			func() {},
			func(*feegrant.QueryAllowanceResponse) {},
		},
		{
			"valid query: expect single grant",
			&feegrant.QueryAllowanceRequest{
				Granter: suite.addrs[0].String(),
				Grantee: suite.addrs[1].String(),
			},
			false,
			func() {
				suite.grantFeeAllowance(suite.addrs[0], suite.addrs[1])
			},
			func(response *feegrant.QueryAllowanceResponse) {
				suite.Require().Equal(response.Allowance.Granter, suite.addrs[0].String())
				suite.Require().Equal(response.Allowance.Grantee, suite.addrs[1].String())
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
				Grantee: "invalid_grantee",
			},
			true,
			func() {},
			func(*feegrant.QueryAllowancesResponse) {},
		},
		{
			"no grants",
			&feegrant.QueryAllowancesRequest{
				Grantee: suite.addrs[1].String(),
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
				Grantee: suite.addrs[1].String(),
			},
			false,
			func() {
				suite.grantFeeAllowance(suite.addrs[0], suite.addrs[1])
			},
			func(resp *feegrant.QueryAllowancesResponse) {
				suite.Require().Equal(len(resp.Allowances), 1)
				suite.Require().Equal(resp.Allowances[0].Granter, suite.addrs[0].String())
				suite.Require().Equal(resp.Allowances[0].Grantee, suite.addrs[1].String())
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
				Granter: "invalid_grantee",
			},
			true,
			func() {},
			func(*feegrant.QueryAllowancesByGranterResponse) {},
		},
		{
			"no grants",
			&feegrant.QueryAllowancesByGranterRequest{
				Granter: suite.addrs[0].String(),
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
				Granter: suite.addrs[0].String(),
			},
			false,
			func() {
				suite.grantFeeAllowance(suite.addrs[0], suite.addrs[1])

				// adding this allowance to check whether the pagination working fine.
				suite.grantFeeAllowance(suite.addrs[1], suite.addrs[2])
			},
			func(resp *feegrant.QueryAllowancesByGranterResponse) {
				suite.Require().Equal(len(resp.Allowances), 1)
				suite.Require().Equal(resp.Allowances[0].Granter, suite.addrs[0].String())
				suite.Require().Equal(resp.Allowances[0].Grantee, suite.addrs[1].String())
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
	exp := suite.ctx.BlockTime().AddDate(1, 0, 0)
	err := suite.feegrantKeeper.GrantAllowance(suite.ctx, granter, grantee, &feegrant.BasicAllowance{
		SpendLimit: sdk.NewCoins(sdk.NewInt64Coin("atom", 555)),
		Expiration: &exp,
	})
	suite.Require().NoError(err)
}
