package keeper_test

import (
	"fmt"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktestutil "cosmossdk.io/x/bank/testutil"
	banktypes "cosmossdk.io/x/bank/types"

	transferkeeper "github.com/cosmos/ibc-go/v8/modules/apps/transfer/keeper"
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestMigratorMigrateParams() {
	testCases := []struct {
		msg            string
		malleate       func()
		expectedParams transfertypes.Params
	}{
		{
			"success: default params",
			func() {
				params := transfertypes.DefaultParams()
				subspace := suite.chainA.GetSimApp().GetSubspace(transfertypes.ModuleName)
				subspace.SetParamSet(suite.chainA.GetContext(), &params) // set params
			},
			transfertypes.DefaultParams(),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(fmt.Sprintf("case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate() // explicitly set params

			migrator := transferkeeper.NewMigrator(suite.chainA.GetSimApp().TransferKeeper)
			err := migrator.MigrateParams(suite.chainA.GetContext())
			suite.Require().NoError(err)

			params := suite.chainA.GetSimApp().TransferKeeper.GetParams(suite.chainA.GetContext())
			suite.Require().Equal(tc.expectedParams, params)
		})
	}
}

func (suite *KeeperTestSuite) TestMigratorMigrateTraces() {
	testCases := []struct {
		msg            string
		malleate       func()
		expectedTraces transfertypes.Traces
	}{
		{
			"success: two slashes in base denom",
			func() {
				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(
					suite.chainA.GetContext(),
					transfertypes.DenomTrace{
						BaseDenom: "pool/1", Path: "transfer/channel-0/gamm",
					})
			},
			transfertypes.Traces{
				{
					BaseDenom: "gamm/pool/1", Path: "transfer/channel-0",
				},
			},
		},
		{
			"success: one slash in base denom",
			func() {
				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(
					suite.chainA.GetContext(),
					transfertypes.DenomTrace{
						BaseDenom: "0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA", Path: "transfer/channel-149/erc",
					})
			},
			transfertypes.Traces{
				{
					BaseDenom: "erc/0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA", Path: "transfer/channel-149",
				},
			},
		},
		{
			"success: multiple slashes in a row in base denom",
			func() {
				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(
					suite.chainA.GetContext(),
					transfertypes.DenomTrace{
						BaseDenom: "1", Path: "transfer/channel-5/gamm//pool",
					})
			},
			transfertypes.Traces{
				{
					BaseDenom: "gamm//pool/1", Path: "transfer/channel-5",
				},
			},
		},
		{
			"success: multihop base denom",
			func() {
				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(
					suite.chainA.GetContext(),
					transfertypes.DenomTrace{
						BaseDenom: "transfer/channel-1/uatom", Path: "transfer/channel-0",
					})
			},
			transfertypes.Traces{
				{
					BaseDenom: "uatom", Path: "transfer/channel-0/transfer/channel-1",
				},
			},
		},
		{
			"success: non-standard port",
			func() {
				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(
					suite.chainA.GetContext(),
					transfertypes.DenomTrace{
						BaseDenom: "customport/channel-7/uatom", Path: "transfer/channel-0/transfer/channel-1",
					})
			},
			transfertypes.Traces{
				{
					BaseDenom: "uatom", Path: "transfer/channel-0/transfer/channel-1/customport/channel-7",
				},
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(fmt.Sprintf("case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate() // explicitly set up denom traces

			migrator := transferkeeper.NewMigrator(suite.chainA.GetSimApp().TransferKeeper)
			err := migrator.MigrateTraces(suite.chainA.GetContext())
			suite.Require().NoError(err)

			traces := suite.chainA.GetSimApp().TransferKeeper.GetAllDenomTraces(suite.chainA.GetContext())
			suite.Require().Equal(tc.expectedTraces, traces)
		})
	}
}

func (suite *KeeperTestSuite) TestMigratorMigrateTracesCorruptionDetection() {
	// IBCDenom() previously would return "customport/channel-0/uatom", but now should return ibc/{hash}
	corruptedDenomTrace := transfertypes.DenomTrace{
		BaseDenom: "customport/channel-0/uatom",
		Path:      "",
	}
	suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(suite.chainA.GetContext(), corruptedDenomTrace)

	migrator := transferkeeper.NewMigrator(suite.chainA.GetSimApp().TransferKeeper)
	suite.Panics(func() {
		migrator.MigrateTraces(suite.chainA.GetContext()) //nolint:errcheck // we shouldn't check the error here because we want to ensure that a panic occurs.
	})
}

func (suite *KeeperTestSuite) TestMigrateTotalEscrowForDenom() {
	var (
		path  *ibctesting.Path
		denom string
	)

	testCases := []struct {
		msg               string
		malleate          func()
		expectedEscrowAmt sdkmath.Int
	}{
		{
			"success: one native denom escrowed in one channel",
			func() {
				denom = sdk.DefaultBondDenom
				escrowAddress := transfertypes.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				coin := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))

				// funds the escrow account to have balance
				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrowAddress, sdk.NewCoins(coin)))
			},
			sdkmath.NewInt(100),
		},
		{
			"success: one native denom escrowed in two channels",
			func() {
				denom = sdk.DefaultBondDenom
				extraPath := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(extraPath)

				escrowAddress1 := transfertypes.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				escrowAddress2 := transfertypes.GetEscrowAddress(extraPath.EndpointA.ChannelConfig.PortID, extraPath.EndpointA.ChannelID)
				coin1 := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
				coin2 := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))

				// funds the escrow accounts to have balance
				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrowAddress1, sdk.NewCoins(coin1)))
				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrowAddress2, sdk.NewCoins(coin2)))
			},
			sdkmath.NewInt(200),
		},
		{
			"success: valid ibc denom escrowed in one channel",
			func() {
				escrowAddress := transfertypes.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				trace := transfertypes.ParseDenomTrace(transfertypes.GetPrefixedDenom(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, sdk.DefaultBondDenom))
				coin := sdk.NewCoin(trace.IBCDenom(), sdkmath.NewInt(100))
				denom = trace.IBCDenom()

				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(suite.chainA.GetContext(), trace)

				// funds the escrow account to have balance
				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrowAddress, sdk.NewCoins(coin)))
			},
			sdkmath.NewInt(100),
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			path = ibctesting.NewTransferPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			tc.malleate() // explicitly fund escrow account

			migrator := transferkeeper.NewMigrator(suite.chainA.GetSimApp().TransferKeeper)
			suite.Require().NoError(migrator.MigrateTotalEscrowForDenom(suite.chainA.GetContext()))

			// check that the migration set the expected amount for both native and IBC tokens
			amount := suite.chainA.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainA.GetContext(), denom)
			suite.Require().Equal(tc.expectedEscrowAmt, amount.Amount)
		})
	}
}

func (suite *KeeperTestSuite) TestMigratorMigrateMetadata() {
	var (
		denomTraces      []transfertypes.DenomTrace
		expectedMetadata []banktypes.Metadata
	)

	testCases := []struct {
		msg      string
		malleate func()
	}{
		{
			"success with one denom trace with one hop",
			func() {
				denomTraces = []transfertypes.DenomTrace{
					{
						BaseDenom: "foo",
						Path:      "transfer/channel-0",
					},
				}

				expectedMetadata = []banktypes.Metadata{
					{
						Description: "IBC token from transfer/channel-0/foo",
						DenomUnits: []*banktypes.DenomUnit{
							{
								Denom:    "foo",
								Exponent: 0,
							},
						},
						Base:    denomTraces[0].IBCDenom(), // ibc/EB7094899ACFB7A6F2A67DB084DEE2E9A83DEFAA5DEF92D9A9814FFD9FF673FA
						Display: "transfer/channel-0/foo",
						Name:    "transfer/channel-0/foo IBC token",
						Symbol:  "FOO",
					},
				}
			},
		},
		{
			"success with one denom trace with two hops",
			func() {
				denomTraces = []transfertypes.DenomTrace{
					{
						BaseDenom: "ubar",
						Path:      "transfer/channel-1/transfer/channel-2",
					},
				}

				expectedMetadata = []banktypes.Metadata{
					{
						Description: "IBC token from transfer/channel-1/transfer/channel-2/ubar",
						DenomUnits: []*banktypes.DenomUnit{
							{
								Denom:    "ubar",
								Exponent: 0,
							},
						},
						Base:    denomTraces[0].IBCDenom(), // ibc/8243B3EAA19BAB1DB3B0020B81C0C5A953E7B22C042CEE44E639A11A238BA57C
						Display: "transfer/channel-1/transfer/channel-2/ubar",
						Name:    "transfer/channel-1/transfer/channel-2/ubar IBC token",
						Symbol:  "UBAR",
					},
				}
			},
		},
		{
			"success with two denom traces with one hop",
			func() {
				denomTraces = []transfertypes.DenomTrace{
					{
						BaseDenom: "foo",
						Path:      "transfer/channel-0",
					},
					{
						BaseDenom: "bar",
						Path:      "transfer/channel-0",
					},
				}

				expectedMetadata = []banktypes.Metadata{
					{
						Description: "IBC token from transfer/channel-0/foo",
						DenomUnits: []*banktypes.DenomUnit{
							{
								Denom:    "foo",
								Exponent: 0,
							},
						},
						Base:    denomTraces[0].IBCDenom(), // ibc/EB7094899ACFB7A6F2A67DB084DEE2E9A83DEFAA5DEF92D9A9814FFD9FF673FA
						Display: "transfer/channel-0/foo",
						Name:    "transfer/channel-0/foo IBC token",
						Symbol:  "FOO",
					},
					{
						Description: "IBC token from transfer/channel-0/bar",
						DenomUnits: []*banktypes.DenomUnit{
							{
								Denom:    "bar",
								Exponent: 0,
							},
						},
						Base:    denomTraces[1].IBCDenom(), // ibc/E1530E21F1848B6C29C9E89256D43E294976897611A61741CACBA55BE21736F5
						Display: "transfer/channel-0/bar",
						Name:    "transfer/channel-0/bar IBC token",
						Symbol:  "BAR",
					},
				}
			},
		},
		{
			"success with two denom traces, metadata for one of them already exists",
			func() {
				denomTraces = []transfertypes.DenomTrace{
					{
						BaseDenom: "foo",
						Path:      "transfer/channel-0",
					},
					{
						BaseDenom: "bar",
						Path:      "transfer/channel-0",
					},
				}

				expectedMetadata = []banktypes.Metadata{
					{
						Description: "IBC token from transfer/channel-0/foo",
						DenomUnits: []*banktypes.DenomUnit{
							{
								Denom:    "foo",
								Exponent: 0,
							},
						},
						Base:    denomTraces[0].IBCDenom(), // ibc/EB7094899ACFB7A6F2A67DB084DEE2E9A83DEFAA5DEF92D9A9814FFD9FF673FA
						Display: "transfer/channel-0/foo",
						Name:    "transfer/channel-0/foo IBC token",
						Symbol:  "FOO",
					},
					{
						Description: "IBC token from transfer/channel-0/bar",
						DenomUnits: []*banktypes.DenomUnit{
							{
								Denom:    "bar",
								Exponent: 0,
							},
						},
						Base:    denomTraces[1].IBCDenom(), // ibc/E1530E21F1848B6C29C9E89256D43E294976897611A61741CACBA55BE21736F5
						Display: "transfer/channel-0/bar",
						Name:    "transfer/channel-0/bar IBC token",
						Symbol:  "BAR",
					},
				}

				// set metadata for one of the tokens, so that it exists already in state before doing the migration
				suite.chainA.GetSimApp().BankKeeper.SetDenomMetaData(suite.chainA.GetContext(), expectedMetadata[1])
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset
			ctx := suite.chainA.GetContext()

			tc.malleate()

			for _, denomTrace := range denomTraces {
				suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(ctx, denomTrace)
			}

			// run migration
			migrator := transferkeeper.NewMigrator(suite.chainA.GetSimApp().TransferKeeper)
			err := migrator.MigrateDenomMetadata(ctx)
			suite.Require().NoError(err)

			for _, expMetadata := range expectedMetadata {
				denomMetadata, found := suite.chainA.GetSimApp().BankKeeper.GetDenomMetaData(ctx, expMetadata.Base)
				suite.Require().True(found)
				suite.Require().Equal(expMetadata, denomMetadata)
			}
		})
	}
}
