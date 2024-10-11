package keeper_test

import (
	gocontext "context"
	"fmt"
	"time"

	v1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	"cosmossdk.io/core/header"
	"cosmossdk.io/x/bank/testutil"
	"cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	vestingtypes "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
)

func (suite *KeeperTestSuite) TestQueryBalance() {
	ctx, queryClient := suite.ctx, suite.queryClient
	_, _, addr := testdata.KeyTestPubAddr()

	addrStr, err := suite.authKeeper.AddressCodec().BytesToString(addr)
	suite.Require().NoError(err)

	origCoins := sdk.NewCoins(newBarCoin(30))
	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(ctx, suite.bankKeeper, addr, origCoins))

	testCases := []struct {
		name         string
		req          *types.QueryBalanceRequest
		expectErrMsg string
		postFn       func(res *types.QueryBalanceResponse)
	}{
		{
			"empty request",
			&types.QueryBalanceRequest{},
			"invalid denom",
			nil,
		},
		{
			"invalid denom",
			types.NewQueryBalanceRequest(addrStr, "0000"),
			"invalid denom",
			nil,
		},
		{
			"empty address",
			types.NewQueryBalanceRequest("", barDenom),
			"empty address string is not allowed",
			nil,
		},
		{
			"invalid address",
			&types.QueryBalanceRequest{Address: "foo", Denom: barDenom},
			"invalid address",
			nil,
		},
		{
			"query missing denom",
			&types.QueryBalanceRequest{Address: addrStr},
			"invalid denom",
			nil,
		},
		{
			"valid query empty result",
			types.NewQueryBalanceRequest(addrStr, fooDenom),
			"",
			func(res *types.QueryBalanceResponse) {
				suite.True(res.Balance.IsZero())
			},
		},
		{
			"valid query",
			types.NewQueryBalanceRequest(addrStr, barDenom),
			"",
			func(res *types.QueryBalanceResponse) {
				suite.True(res.Balance.IsEqual(newBarCoin(30)))
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			res, err := queryClient.Balance(gocontext.Background(), tc.req)
			if tc.expectErrMsg == "" {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
			} else {
				suite.Require().ErrorContains(err, tc.expectErrMsg)
			}

			if tc.postFn != nil {
				tc.postFn(res)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryAllBalances() {
	ctx, queryClient := suite.ctx, suite.queryClient
	_, _, addr := testdata.KeyTestPubAddr()
	_, err := queryClient.AllBalances(gocontext.Background(), &types.QueryAllBalancesRequest{})
	suite.Require().Error(err)

	addrStr, err := suite.authKeeper.AddressCodec().BytesToString(addr)
	suite.Require().NoError(err)

	pageReq := &query.PageRequest{
		Key:        nil,
		Limit:      1,
		CountTotal: false,
	}
	req := types.NewQueryAllBalancesRequest(addrStr, pageReq, false)
	res, err := queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balances.IsZero())

	fooCoins := newFooCoin(50)
	barCoins := newBarCoin(30)
	ibcCoins := newIbcCoin(20)

	origCoins := sdk.NewCoins(fooCoins, barCoins, ibcCoins)

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(ctx, suite.bankKeeper, addr, origCoins))

	addIBCMetadata(ctx, suite.bankKeeper)

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
	req = types.NewQueryAllBalancesRequest(addrStr, pageReq, false)
	res, err = queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Equal(res.Balances.Len(), 1)
	suite.NotNil(res.Pagination.NextKey)

	pageThree := res.Pagination.NextKey

	suite.T().Log("query third page with nextkey")
	pageReq = &query.PageRequest{
		Key:        pageThree,
		Limit:      1,
		CountTotal: true,
	}
	req = types.NewQueryAllBalancesRequest(addrStr, pageReq, false)
	res, err = queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Equal(res.Balances.Len(), 1)
	suite.Equal(res.Balances[0].Denom, ibcCoins.Denom)

	suite.T().Log("query third page with nextkey and resolve ibc denom")
	pageReq = &query.PageRequest{
		Key:        pageThree,
		Limit:      1,
		CountTotal: true,
	}
	req = types.NewQueryAllBalancesRequest(addrStr, pageReq, true)
	res, err = queryClient.AllBalances(gocontext.Background(), req)
	suite.Require().NoError(err)
	suite.Equal(res.Balances.Len(), 1)
	suite.Equal(res.Balances[0].Denom, ibcPath+"/"+ibcBaseDenom)
	suite.Nil(res.Pagination.NextKey)
}

func (suite *KeeperTestSuite) TestSpendableBalances() {
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := suite.authKeeper.AddressCodec().BytesToString(addr)
	suite.Require().NoError(err)

	ctx := sdk.UnwrapSDKContext(suite.ctx)
	ctx = ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	queryClient := suite.mockQueryClient(ctx)

	_, err = queryClient.SpendableBalances(ctx, &types.QuerySpendableBalancesRequest{})
	suite.Require().Error(err)

	pageReq := &query.PageRequest{
		Key:        nil,
		Limit:      2,
		CountTotal: false,
	}
	req := types.NewQuerySpendableBalancesRequest(addrStr, pageReq)
	acc := authtypes.NewBaseAccountWithAddress(addr)

	suite.mockSpendableCoins(ctx, acc)
	res, err := queryClient.SpendableBalances(ctx, req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balances.IsZero())

	fooCoins := newFooCoin(50)
	barCoins := newBarCoin(30)

	origCoins := sdk.NewCoins(fooCoins, barCoins)
	vacc, err := vestingtypes.NewContinuousVestingAccount(
		acc,
		sdk.NewCoins(fooCoins),
		ctx.HeaderInfo().Time.Unix(),
		ctx.HeaderInfo().Time.Add(time.Hour).Unix(),
	)
	suite.Require().NoError(err)

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(suite.ctx, suite.bankKeeper, addr, origCoins))

	// move time forward for some tokens to vest
	ctx = ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(30 * time.Minute)})
	queryClient = suite.mockQueryClient(ctx)

	suite.mockSpendableCoins(ctx, vacc)
	res, err = queryClient.SpendableBalances(ctx, req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Equal(2, res.Balances.Len())
	suite.Nil(res.Pagination.NextKey)
	suite.EqualValues(30, res.Balances[0].Amount.Int64())
	suite.EqualValues(25, res.Balances[1].Amount.Int64())
}

func (suite *KeeperTestSuite) TestSpendableBalanceByDenom() {
	_, _, addr := testdata.KeyTestPubAddr()

	ctx := sdk.UnwrapSDKContext(suite.ctx)
	ctx = ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	queryClient := suite.mockQueryClient(ctx)

	_, err := queryClient.SpendableBalanceByDenom(ctx, &types.QuerySpendableBalanceByDenomRequest{})
	suite.Require().Error(err)

	addrStr, err := suite.authKeeper.AddressCodec().BytesToString(addr)
	suite.Require().NoError(err)

	req := types.NewQuerySpendableBalanceByDenomRequest(addrStr, fooDenom)
	acc := authtypes.NewBaseAccountWithAddress(addr)

	suite.mockSpendableCoins(ctx, acc)
	res, err := queryClient.SpendableBalanceByDenom(ctx, req)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.True(res.Balance.IsZero())

	fooCoins := newFooCoin(100)
	barCoins := newBarCoin(30)

	origCoins := sdk.NewCoins(fooCoins, barCoins)
	vacc, err := vestingtypes.NewContinuousVestingAccount(
		acc,
		sdk.NewCoins(fooCoins),
		ctx.HeaderInfo().Time.Unix(),
		ctx.HeaderInfo().Time.Add(time.Hour).Unix(),
	)
	suite.Require().NoError(err)

	suite.mockFundAccount(addr)
	suite.Require().NoError(testutil.FundAccount(suite.ctx, suite.bankKeeper, addr, origCoins))

	// move time forward for half of the tokens to vest
	ctx = ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(30 * time.Minute)})
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
	suite.Require().NotNil(res)
	genesisSupply := res.Supply
	testCoins := sdk.NewCoins(sdk.NewInt64Coin("test", 400000000))

	suite.mockMintCoins(mintAcc)
	suite.Require().NoError(suite.bankKeeper.MintCoins(ctx, types.MintModuleName, testCoins))

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
	suite.Require().NoError(suite.bankKeeper.MintCoins(ctx, types.MintModuleName, expectedTotalSupply))

	_, err := queryClient.SupplyOf(gocontext.Background(), &types.QuerySupplyOfRequest{})
	suite.Require().Error(err)

	res, err := queryClient.SupplyOf(gocontext.Background(), &types.QuerySupplyOfRequest{Denom: test1Supply.Denom})
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Require().Equal(test1Supply, res.Amount)

	// total supply bogus denom
	res, err = queryClient.SupplyOf(gocontext.Background(), &types.QuerySupplyOfRequest{Denom: "bogus"})
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Require().Equal(sdk.NewInt64Coin("bogus", 0), res.Amount)
}

func (suite *KeeperTestSuite) TestQueryParams() {
	res, err := suite.queryClient.Params(gocontext.Background(), &types.QueryParamsRequest{})
	suite.Require().NoError(err)
	suite.Require().NotNil(res)
	suite.Require().Equal(suite.bankKeeper.GetParams(suite.ctx), res.GetParams())
}

func (suite *KeeperTestSuite) TestQueryDenomsMetadata() {
	var (
		req         *types.QueryDenomsMetadataRequest
		expMetadata = []types.Metadata(nil)
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
			ctx := suite.ctx

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

func (suite *KeeperTestSuite) TestQueryDenomMetadata() {
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
				expMetadata = types.Metadata{
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
			ctx := suite.ctx

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

func (suite *KeeperTestSuite) TestQueryDenomMetadataByQueryStringRequest() {
	var (
		req         *types.QueryDenomMetadataByQueryStringRequest
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
				req = &types.QueryDenomMetadataByQueryStringRequest{}
			},
			false,
		},
		{
			"not found denom",
			func() {
				req = &types.QueryDenomMetadataByQueryStringRequest{
					Denom: "foo",
				}
			},
			false,
		},
		{
			"success",
			func() {
				expMetadata = types.Metadata{
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
				req = &types.QueryDenomMetadataByQueryStringRequest{
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
			ctx := suite.ctx

			res, err := suite.queryClient.DenomMetadataByQueryString(ctx, req)

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

func (suite *KeeperTestSuite) TestGRPCDenomMetadataV2() {
	var (
		req      *v1beta1.QueryDenomMetadataRequest
		metadata = types.Metadata{
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
		expMetadata = &v1beta1.Metadata{}
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty denom",
			func() {
				req = &v1beta1.QueryDenomMetadataRequest{}
			},
			false,
		},
		{
			"not found denom",
			func() {
				req = &v1beta1.QueryDenomMetadataRequest{
					Denom: "foo",
				}
			},
			false,
		},
		{
			"success",
			func() {
				expMetadata = &v1beta1.Metadata{
					Description: metadata.Description,
					DenomUnits: []*v1beta1.DenomUnit{
						{
							Denom:    metadata.DenomUnits[0].Denom,
							Exponent: metadata.DenomUnits[0].Exponent,
							Aliases:  metadata.DenomUnits[0].Aliases,
						},
						{
							Denom:    metadata.DenomUnits[1].Denom,
							Exponent: metadata.DenomUnits[1].Exponent,
							Aliases:  metadata.DenomUnits[1].Aliases,
						},
					},
					Base:    metadata.Base,
					Display: metadata.Display,
				}

				suite.bankKeeper.SetDenomMetaData(suite.ctx, metadata)
				req = &v1beta1.QueryDenomMetadataRequest{
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
			ctx := suite.ctx

			res, err := suite.bankKeeper.DenomMetadataV2(ctx, req)

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
	suite.Require().NoError(keeper.MintCoins(ctx, types.MintModuleName, initCoins))

	for i := 0; i < 10; i++ {
		addr := sdk.AccAddress(fmt.Sprintf("account-%d", i))

		bal := sdk.NewCoins(sdk.NewCoin(
			sdk.DefaultBondDenom,
			sdk.TokensFromConsensusPower(initialPower/10, sdk.DefaultPowerReduction),
		))
		suite.mockSendCoinsFromModuleToAccount(mintAcc, addr)
		suite.Require().NoError(keeper.SendCoinsFromModuleToAccount(ctx, types.MintModuleName, addr, bal))
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

func (suite *KeeperTestSuite) TestGRPCDenomOwnersByQuery() {
	ctx := suite.ctx

	keeper := suite.bankKeeper

	suite.mockMintCoins(mintAcc)
	suite.Require().NoError(keeper.MintCoins(ctx, types.MintModuleName, initCoins))
	denom := "ibc/123123213123"
	newCoins := sdk.NewCoins(sdk.NewCoin(denom, initTokens))
	suite.mockMintCoins(mintAcc)
	suite.Require().NoError(keeper.MintCoins(ctx, types.MintModuleName, newCoins))

	for i := 0; i < 10; i++ {
		addr := sdk.AccAddress(fmt.Sprintf("account-%d", i))

		bal := sdk.NewCoins(sdk.NewCoin(
			sdk.DefaultBondDenom,
			sdk.TokensFromConsensusPower(initialPower/10, sdk.DefaultPowerReduction),
		))
		suite.mockSendCoinsFromModuleToAccount(mintAcc, addr)
		suite.Require().NoError(keeper.SendCoinsFromModuleToAccount(ctx, types.MintModuleName, addr, bal))
	}

	testCases := map[string]struct {
		req      *types.QueryDenomOwnersByQueryRequest
		expPass  bool
		numAddrs int
		hasNext  bool
		total    uint64
	}{
		"empty request": {
			req:     &types.QueryDenomOwnersByQueryRequest{},
			expPass: false,
		},
		"invalid denom": {
			req: &types.QueryDenomOwnersByQueryRequest{
				Denom: "foo",
			},
			expPass:  true,
			numAddrs: 0,
			hasNext:  false,
			total:    0,
		},
		"valid request - page 1": {
			req: &types.QueryDenomOwnersByQueryRequest{
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
			req: &types.QueryDenomOwnersByQueryRequest{
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
		"valid request for query": {
			req: &types.QueryDenomOwnersByQueryRequest{
				Denom: denom,
				Pagination: &query.PageRequest{
					Limit:      6,
					CountTotal: true,
				},
			},
			expPass:  true,
			numAddrs: 1,
			hasNext:  false,
			total:    1,
		},
	}

	for name, tc := range testCases {
		suite.Run(name, func() {
			resp, err := suite.queryClient.DenomOwnersByQuery(gocontext.Background(), tc.req)
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
