package keeper_test

import (
	gocontext "context"
	"fmt"
	"time"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	vestingtypes "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	"github.com/cosmos/cosmos-sdk/x/bank/testutil"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
)

func (suite *KeeperTestSuite) TestQueryBalance() {
	ctx, queryClient := suite.ctx, suite.queryClient
	_, _, addr := testdata.KeyTestPubAddr()

	_, err := queryClient.Balance(gocontext.Background(), &types.QueryBalanceRequest{})
	suite.Require().Error(err)

	_, err = queryClient.Balance(gocontext.Background(), &types.QueryBalanceRequest{Address: addr.String()})
	suite.Require().Error(err)

	req := types.NewQueryBalanceRequest(addr, "0000")
	_, err = queryClient.Balance(gocontext.Background(), req)
	suite.Require().Error(err)

	req = types.NewQueryBalanceRequest(addr, fooDenom)
	res, err := queryClient.Balance(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balance.IsZero())

	origCoins := sdk.NewCoins(newFooCoin(50), newBarCoin(30))

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(suite.bankKeeper, ctx, addr, origCoins))

	res, err = queryClient.Balance(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balance.IsEqual(newFooCoin(50)))
}

func (suite *KeeperTestSuite) TestQueryAllBalances() {
	ctx, queryClient := suite.ctx, suite.queryClient
	_, _, addr := testdata.KeyTestPubAddr()
	_, err := queryClient.AllBalances(gocontext.Background(), &types.QueryAllBalancesRequest{})
	suite.Require().Error(err)

	pageReq := &query.PageRequest{
		Key:        nil,
		Limit:      1,
		CountTotal: false,
	}
	req := types.NewQueryAllBalancesRequest(addr, pageReq)
	res, err := queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balances.IsZero())

	fooCoins := newFooCoin(50)
	barCoins := newBarCoin(30)

	origCoins := sdk.NewCoins(fooCoins, barCoins)

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(suite.bankKeeper, ctx, addr, origCoins))

	res, err = queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Equal(res.Balances.Len(), 1)
	suite.NotNil(res.Pagination.NextKey)

	suite.T().Log("query second page with nextkey")
	pageReq = &query.PageRequest{
		Key:        res.Pagination.NextKey,
		Limit:      1,
		CountTotal: true,
	}
	req = types.NewQueryAllBalancesRequest(addr, pageReq)
	res, err = queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Equal(res.Balances.Len(), 1)
	suite.Nil(res.Pagination.NextKey)
}

func (suite *KeeperTestSuite) TestSpendableBalances() {
	ctx := suite.ctx
	_, _, addr := testdata.KeyTestPubAddr()
	ctx = ctx.WithBlockTime(time.Now())
	queryClient := suite.mockQueryClient(ctx)

	_, err := queryClient.SpendableBalances(sdk.WrapSDKContext(ctx), &types.QuerySpendableBalancesRequest{})
	suite.Require().Error(err)

	pageReq := &query.PageRequest{
		Key:        nil,
		Limit:      2,
		CountTotal: false,
	}
	req := types.NewQuerySpendableBalancesRequest(addr, pageReq)
	acc := authtypes.NewBaseAccountWithAddress(addr)

	suite.mockSpendableCoins(ctx, acc)
	res, err := queryClient.SpendableBalances(sdk.WrapSDKContext(ctx), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balances.IsZero())

	fooCoins := newFooCoin(50)
	barCoins := newBarCoin(30)

	origCoins := sdk.NewCoins(fooCoins, barCoins)
	vacc := vestingtypes.NewContinuousVestingAccount(
		acc,
		sdk.NewCoins(fooCoins),
		ctx.BlockTime().Unix(),
		ctx.BlockTime().Add(time.Hour).Unix(),
	)

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(suite.bankKeeper, suite.ctx, addr, origCoins))

	// move time forward for some tokens to vest
	ctx = ctx.WithBlockTime(ctx.BlockTime().Add(30 * time.Minute))
	queryClient = suite.mockQueryClient(ctx)

	suite.mockSpendableCoins(ctx, vacc)
	res, err = queryClient.SpendableBalances(sdk.WrapSDKContext(ctx), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Equal(2, res.Balances.Len())
	suite.Nil(res.Pagination.NextKey)
	suite.EqualValues(30, res.Balances[0].Amount.Int64())
	suite.EqualValues(25, res.Balances[1].Amount.Int64())
}

func (suite *KeeperTestSuite) TestSpendableBalanceByDenom() {
	ctx := suite.ctx
	_, _, addr := testdata.KeyTestPubAddr()
	ctx = ctx.WithBlockTime(time.Now())
	queryClient := suite.mockQueryClient(ctx)

	_, err := queryClient.SpendableBalanceByDenom(ctx, &types.QuerySpendableBalanceByDenomRequest{})
	suite.Require().Error(err)

	req := types.NewQuerySpendableBalanceByDenomRequest(addr, fooDenom)
	acc := authtypes.NewBaseAccountWithAddress(addr)

	suite.mockSpendableCoins(ctx, acc)
	res, err := queryClient.SpendableBalanceByDenom(ctx, req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balance.IsZero())

	fooCoins := newFooCoin(100)
	barCoins := newBarCoin(30)

	origCoins := sdk.NewCoins(fooCoins, barCoins)
	vacc := vestingtypes.NewContinuousVestingAccount(
		acc,
		sdk.NewCoins(fooCoins),
		ctx.BlockTime().Unix(),
		ctx.BlockTime().Add(time.Hour).Unix(),
	)

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(suite.bankKeeper, suite.ctx, addr, origCoins))

	// move time forward for half of the tokens to vest
	ctx = ctx.WithBlockTime(ctx.BlockTime().Add(30 * time.Minute))
	queryClient = suite.mockQueryClient(ctx)

	// check fooCoins first, it has some vested and some vesting
	suite.mockSpendableCoins(ctx, vacc)
	res, err = queryClient.SpendableBalanceByDenom(ctx, req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.EqualValues(50, res.Balance.Amount.Int64())

	// check barCoins, all of it is spendable
	req.Denom = barDenom
	suite.mockSpendableCoins(ctx, vacc)
	res, err = queryClient.SpendableBalanceByDenom(ctx, req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.EqualValues(30, res.Balance.Amount.Int64())
}

func (suite *KeeperTestSuite) TestQueryTotalSupply() {
	ctx, queryClient := suite.ctx, suite.queryClient
	res, err := queryClient.TotalSupply(gocontext.Background(), &types.QueryTotalSupplyRequest{})
	suite.Require().NoError(err)
	genesisSupply := res.Supply
	testCoins := sdk.NewCoins(sdk.NewInt64Coin("test", 400000000))

	suite.mockMintCoins(mintAcc)
	suite.
		Require().
		NoError(suite.bankKeeper.MintCoins(ctx, minttypes.ModuleName, testCoins))

	res, err = queryClient.TotalSupply(gocontext.Background(), &types.QueryTotalSupplyRequest{})
	suite.Require().NoError(err)
	suite.Require().NotNil(res)

	expectedTotalSupply := genesisSupply.Add(testCoins...)
	suite.Require().Equal(1, len(res.Supply))
	suite.Require().Equal(res.Supply, expectedTotalSupply)
}

func (suite *KeeperTestSuite) TestQueryTotalSupplyOf() {
	ctx, queryClient := suite.ctx, suite.queryClient

	test1Supply := sdk.NewInt64Coin("test1", 4000000)
	test2Supply := sdk.NewInt64Coin("test2", 700000000)
	expectedTotalSupply := sdk.NewCoins(test1Supply, test2Supply)

	suite.mockMintCoins(mintAcc)
	suite.
		Require().
		NoError(suite.bankKeeper.MintCoins(ctx, minttypes.ModuleName, expectedTotalSupply))

	_, err := queryClient.SupplyOf(gocontext.Background(), &types.QuerySupplyOfRequest{})
	suite.Require().Error(err)

	res, err := queryClient.SupplyOf(gocontext.Background(), &types.QuerySupplyOfRequest{Denom: test1Supply.Denom})
	suite.Require().NoError(err)
	suite.Require().NotNil(res)

	suite.Require().Equal(test1Supply, res.Amount)
}

func (suite *KeeperTestSuite) TestQueryParams() {
	res, err := suite.queryClient.Params(gocontext.Background(), &types.QueryParamsRequest{})
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Require().Equal(suite.bankKeeper.GetParams(suite.ctx), res.GetParams())
}

func (suite *KeeperTestSuite) QueryDenomsMetadataRequest() {
	var (
		req         *types.QueryDenomsMetadataRequest
		expMetadata = []types.Metadata{}
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty pagination",
			func() {
				req = &types.QueryDenomsMetadataRequest{}
			},
			true,
		},
		{
			"success, no results",
			func() {
				req = &types.QueryDenomsMetadataRequest{
					Pagination: &query.PageRequest{
						Limit:      3,
						CountTotal: true,
					},
				}
			},
			true,
		},
		{
			"success",
			func() {
				metadataAtom := types.Metadata{
					Description: "The native staking token of the Cosmos Hub.",
					DenomUnits: []*types.DenomUnit{
						{
							Denom:    "uatom",
							Exponent: 0,
							Aliases:  []string{"microatom"},
						},
						{
							Denom:    "atom",
							Exponent: 6,
							Aliases:  []string{"ATOM"},
						},
					},
					Base:    "uatom",
					Display: "atom",
				}

				metadataEth := types.Metadata{
					Description: "Ethereum native token",
					DenomUnits: []*types.DenomUnit{
						{
							Denom:    "wei",
							Exponent: 0,
						},
						{
							Denom:    "eth",
							Exponent: 18,
							Aliases:  []string{"ETH", "ether"},
						},
					},
					Base:    "wei",
					Display: "eth",
				}

				suite.bankKeeper.SetDenomMetaData(suite.ctx, metadataAtom)
				suite.bankKeeper.SetDenomMetaData(suite.ctx, metadataEth)
				expMetadata = []types.Metadata{metadataAtom, metadataEth}
				req = &types.QueryDenomsMetadataRequest{
					Pagination: &query.PageRequest{
						Limit:      7,
						CountTotal: true,
					},
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := sdk.WrapSDKContext(suite.ctx)

			res, err := suite.queryClient.DenomsMetadata(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(expMetadata, res.Metadatas)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) QueryDenomMetadataRequest() {
	var (
		req         *types.QueryDenomMetadataRequest
		expMetadata = types.Metadata{}
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty denom",
			func() {
				req = &types.QueryDenomMetadataRequest{}
			},
			false,
		},
		{
			"not found denom",
			func() {
				req = &types.QueryDenomMetadataRequest{
					Denom: "foo",
				}
			},
			false,
		},
		{
			"success",
			func() {
				expMetadata := types.Metadata{
					Description: "The native staking token of the Cosmos Hub.",
					DenomUnits: []*types.DenomUnit{
						{
							Denom:    "uatom",
							Exponent: 0,
							Aliases:  []string{"microatom"},
						},
						{
							Denom:    "atom",
							Exponent: 6,
							Aliases:  []string{"ATOM"},
						},
					},
					Base:    "uatom",
					Display: "atom",
				}

				suite.bankKeeper.SetDenomMetaData(suite.ctx, expMetadata)
				req = &types.QueryDenomMetadataRequest{
					Denom: expMetadata.Base,
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := sdk.WrapSDKContext(suite.ctx)

			res, err := suite.queryClient.DenomMetadata(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(expMetadata, res.Metadata)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCDenomOwners() {
	ctx := suite.ctx

	keeper := suite.bankKeeper

	suite.mockMintCoins(mintAcc)
	suite.Require().NoError(keeper.MintCoins(ctx, minttypes.ModuleName, initCoins))

	for i := 0; i < 10; i++ {
		addr := sdk.AccAddress([]byte(fmt.Sprintf("account-%d", i)))

		bal := sdk.NewCoins(sdk.NewCoin(
			sdk.DefaultBondDenom,
			sdk.TokensFromConsensusPower(initialPower/10, sdk.DefaultPowerReduction),
		))
		suite.mockSendCoinsFromModuleToAccount(mintAcc, addr)
		suite.Require().NoError(keeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, addr, bal))
	}

	testCases := map[string]struct {
		req      *types.QueryDenomOwnersRequest
		expPass  bool
		numAddrs int
		hasNext  bool
		total    uint64
	}{
		"empty request": {
			req:     &types.QueryDenomOwnersRequest{},
			expPass: false,
		},
		"invalid denom": {
			req: &types.QueryDenomOwnersRequest{
				Denom: "foo",
			},
			expPass:  true,
			numAddrs: 0,
			hasNext:  false,
			total:    0,
		},
		"valid request - page 1": {
			req: &types.QueryDenomOwnersRequest{
				Denom: sdk.DefaultBondDenom,
				Pagination: &query.PageRequest{
					Limit:      6,
					CountTotal: true,
				},
			},
			expPass:  true,
			numAddrs: 6,
			hasNext:  true,
			total:    10,
		},
		"valid request - page 2": {
			req: &types.QueryDenomOwnersRequest{
				Denom: sdk.DefaultBondDenom,
				Pagination: &query.PageRequest{
					Offset:     6,
					Limit:      10,
					CountTotal: true,
				},
			},
			expPass:  true,
			numAddrs: 4,
			hasNext:  false,
			total:    10,
		},
	}

	for name, tc := range testCases {
		suite.Run(name, func() {
			resp, err := suite.queryClient.DenomOwners(gocontext.Background(), tc.req)
			if tc.expPass {
				suite.NoError(err)
				suite.NotNil(resp)
				suite.Len(resp.DenomOwners, tc.numAddrs)
				suite.Equal(tc.total, resp.Pagination.Total)

				if tc.hasNext {
					suite.NotNil(resp.Pagination.NextKey)
				} else {
					suite.Nil(resp.Pagination.NextKey)
				}
			} else {
				suite.Require().Error(err)
			}
		})
	}

	suite.Require().True(true)
}

func (suite *KeeperTestSuite) TestQuerySendEnabled() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper

	bankKeeper.SetSendEnabled(ctx, "falsestcoin", false)
	bankKeeper.SetSendEnabled(ctx, "truestcoin", true)

	tests := []struct {
		name string
		req  *types.QuerySendEnabledRequest
		exp  *types.QuerySendEnabledResponse
	}{
		{
			name: "nil denoms list",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "falsestcoin", Enabled: false},
					{Denom: "truestcoin", Enabled: true},
				},
				Pagination: &query.PageResponse{
					NextKey: nil,
					Total:   2,
				},
			},
		},
		{
			name: "empty denoms list",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "falsestcoin", Enabled: false},
					{Denom: "truestcoin", Enabled: true},
				},
				Pagination: &query.PageResponse{
					NextKey: nil,
					Total:   2,
				},
			},
		},
		{
			name: "limit 1",
			req: &types.QuerySendEnabledRequest{
				Pagination: &query.PageRequest{
					Limit:      1,
					CountTotal: true,
				},
			},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "falsestcoin", Enabled: false},
				},
				Pagination: &query.PageResponse{
					NextKey: []byte("truestcoin"),
					Total:   2,
				},
			},
		},
		{
			name: "just truestcoin",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{"truestcoin"}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "truestcoin", Enabled: true},
				},
				Pagination: nil,
			},
		},
		{
			name: "just falsestcoin",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{"falsestcoin"}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "falsestcoin", Enabled: false},
				},
				Pagination: nil,
			},
		},
		{
			name: "just an unknown coin",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{"unknowniercoin"}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: nil,
				Pagination:  nil,
			},
		},
		{
			name: "both truestcoin falsestcoin",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{"truestcoin", "falsestcoin"}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "truestcoin", Enabled: true},
					{Denom: "falsestcoin", Enabled: false},
				},
				Pagination: nil,
			},
		},
		{
			name: "both truestcoin falsestcoin and an unknown",
			req:  &types.QuerySendEnabledRequest{Denoms: []string{"truestcoin", "falsestcoin", "unknownestcoin"}},
			exp: &types.QuerySendEnabledResponse{
				SendEnabled: []*types.SendEnabled{
					{Denom: "truestcoin", Enabled: true},
					{Denom: "falsestcoin", Enabled: false},
				},
				Pagination: nil,
			},
		},
	}

	for _, tc := range tests {
		suite.Run(tc.name, func() {
			resp, err := suite.queryClient.SendEnabled(gocontext.Background(), tc.req)
			suite.Require().NoError(err)
			if !suite.Assert().Equal(tc.exp, resp) {
				if !suite.Assert().Len(resp.SendEnabled, len(tc.exp.SendEnabled)) {
					for i := range tc.exp.SendEnabled {
						suite.Assert().Equal(tc.exp.SendEnabled[i].Denom, resp.SendEnabled[i].Denom, fmt.Sprintf("SendEnabled[%d].Denom", i))
						suite.Assert().Equal(tc.exp.SendEnabled[i].Enabled, resp.SendEnabled[i].Enabled, fmt.Sprintf("SendEnabled[%d].Enabled", i))
					}
				}
				if !suite.Assert().Equal(tc.exp.Pagination, resp.Pagination, "Pagination") && tc.exp.Pagination != nil && resp.Pagination != nil {
					suite.Assert().Equal(tc.exp.Pagination.NextKey, resp.Pagination.NextKey, "Pagination.NextKey")
					suite.Assert().Equal(tc.exp.Pagination.Total, resp.Pagination.Total, "Pagination.Total")
				}
			}
			suite.Require().Equal(tc.exp, resp)
		})
	}
}
