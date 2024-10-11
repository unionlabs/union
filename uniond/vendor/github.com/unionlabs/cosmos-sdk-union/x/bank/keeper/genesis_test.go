package keeper_test

import (
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

func (suite *KeeperTestSuite) TestExportGenesis() {
	ctx := suite.ctx

	expectedMetadata := suite.getTestMetadata()
	expectedBalances, expTotalSupply := suite.getTestBalancesAndSupply()

	// Adding genesis supply to the expTotalSupply
	genesisSupply, _, err := suite.bankKeeper.GetPaginatedTotalSupply(suite.ctx, &query.PageRequest{Limit: query.PaginationMaxLimit})
	suite.Require().NoError(err)
	expTotalSupply = expTotalSupply.Add(genesisSupply...)

	for i := range []int{1, 2} {
		suite.bankKeeper.SetDenomMetaData(ctx, expectedMetadata[i])
		accAddr, err1 := suite.authKeeper.AddressCodec().StringToBytes(expectedBalances[i].Address)
		if err1 != nil {
			panic(err1)
		}
		// set balances via mint and send
		suite.mockMintCoins(mintAcc)
		suite.
			Require().
			NoError(suite.bankKeeper.MintCoins(ctx, types.MintModuleName, expectedBalances[i].Coins))
		suite.mockSendCoinsFromModuleToAccount(mintAcc, accAddr)
		suite.
			Require().
			NoError(suite.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.MintModuleName, accAddr, expectedBalances[i].Coins))
	}

	suite.Require().NoError(suite.bankKeeper.SetParams(ctx, types.DefaultParams()))

	exportGenesis, err := suite.bankKeeper.ExportGenesis(ctx)
	suite.Require().NoError(err)

	suite.Require().Len(exportGenesis.Params.SendEnabled, 0) //nolint:staticcheck // we're testing the old way here
	suite.Require().Equal(types.DefaultParams().DefaultSendEnabled, exportGenesis.Params.DefaultSendEnabled)
	suite.Require().Equal(expTotalSupply, exportGenesis.Supply)
	suite.Require().Subset(exportGenesis.Balances, expectedBalances)
	suite.Require().Equal(expectedMetadata, exportGenesis.DenomMetadata)
}

func (suite *KeeperTestSuite) getTestBalancesAndSupply() ([]types.Balance, sdk.Coins) {
	addr1Balance := sdk.Coins{sdk.NewInt64Coin("testcoin3", 10)}
	addr2Balance := sdk.Coins{sdk.NewInt64Coin("testcoin1", 32), sdk.NewInt64Coin("testcoin2", 34)}

	totalSupply := addr1Balance.Add(addr2Balance...)

	return []types.Balance{
		{Address: "cosmos1f9xjhxm0plzrh9cskf4qee4pc2xwp0n0556gh0", Coins: addr2Balance},
		{Address: "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd", Coins: addr1Balance},
	}, totalSupply
}

func (suite *KeeperTestSuite) TestInitGenesis() {
	m := types.Metadata{Description: sdk.DefaultBondDenom, Base: sdk.DefaultBondDenom, Display: sdk.DefaultBondDenom}
	g := types.DefaultGenesisState()
	g.DenomMetadata = []types.Metadata{m}
	bk := suite.bankKeeper
	suite.Require().NoError(bk.InitGenesis(suite.ctx, g))

	m2, found := bk.GetDenomMetaData(suite.ctx, m.Base)
	suite.Require().True(found)
	suite.Require().Equal(m, m2)
}

func (suite *KeeperTestSuite) TestTotalSupply() {
	// Prepare some test data.
	defaultGenesis := types.DefaultGenesisState()
	balances := []types.Balance{
		{Coins: sdk.NewCoins(sdk.NewCoin("foocoin", sdkmath.NewInt(1))), Address: "cosmos1f9xjhxm0plzrh9cskf4qee4pc2xwp0n0556gh0"},
		{Coins: sdk.NewCoins(sdk.NewCoin("barcoin", sdkmath.NewInt(1))), Address: "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd"},
		{Coins: sdk.NewCoins(sdk.NewCoin("foocoin", sdkmath.NewInt(10)), sdk.NewCoin("barcoin", sdkmath.NewInt(20))), Address: "cosmos1m3h30wlvsf8llruxtpukdvsy0km2kum8g38c8q"},
	}
	totalSupply := sdk.NewCoins(sdk.NewCoin("foocoin", sdkmath.NewInt(11)), sdk.NewCoin("barcoin", sdkmath.NewInt(21)))

	genesisSupply, _, err := suite.bankKeeper.GetPaginatedTotalSupply(suite.ctx, &query.PageRequest{Limit: query.PaginationMaxLimit})
	suite.Require().NoError(err)

	testcases := []struct {
		name      string
		genesis   *types.GenesisState
		expSupply sdk.Coins
		expErrMsg string
	}{
		{
			"calculation NOT matching genesis Supply field",
			types.NewGenesisState(defaultGenesis.Params, balances, sdk.NewCoins(sdk.NewCoin("wrongcoin", sdkmath.NewInt(1))), defaultGenesis.DenomMetadata, defaultGenesis.SendEnabled),
			nil, "genesis supply is incorrect, expected 1wrongcoin, got 21barcoin,11foocoin",
		},
		{
			"calculation matches genesis Supply field",
			types.NewGenesisState(defaultGenesis.Params, balances, totalSupply, defaultGenesis.DenomMetadata, defaultGenesis.SendEnabled),
			totalSupply, "",
		},
		{
			"calculation is correct, empty genesis Supply field",
			types.NewGenesisState(defaultGenesis.Params, balances, nil, defaultGenesis.DenomMetadata, defaultGenesis.SendEnabled),
			totalSupply, "",
		},
	}

	for _, tc := range testcases {
		suite.Run(tc.name, func() {
			if tc.expErrMsg != "" {
				suite.Require().ErrorContains(suite.bankKeeper.InitGenesis(suite.ctx, tc.genesis), tc.expErrMsg)
			} else {
				suite.Require().NoError(suite.bankKeeper.InitGenesis(suite.ctx, tc.genesis))
				totalSupply, _, err := suite.bankKeeper.GetPaginatedTotalSupply(suite.ctx, &query.PageRequest{Limit: query.PaginationMaxLimit})
				suite.Require().NoError(err)

				// adding genesis supply to expected supply
				expected := tc.expSupply.Add(genesisSupply...)
				suite.Require().Equal(expected, totalSupply)
			}
		})
	}
}
