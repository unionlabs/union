package keeper_test

import (
	"fmt"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

func (suite *KeeperTestSuite) TestGenesis() {
	getTrace := func(index uint) string {
		return fmt.Sprintf("transfer/channelToChain%d", index)
	}

	var (
		denomTraces           types.Traces
		escrows               sdk.Coins
		pathsAndEscrowAmounts = []struct {
			path   string
			escrow string
		}{
			{getTrace(0), "10"},
			{fmt.Sprintf("%s/%s", getTrace(1), getTrace(0)), "100000"},
			{fmt.Sprintf("%s/%s/%s", getTrace(2), getTrace(1), getTrace(0)), "10000000000"},
			{fmt.Sprintf("%s/%s/%s/%s", getTrace(3), getTrace(2), getTrace(1), getTrace(0)), "1000000000000000"},
			{fmt.Sprintf("%s/%s/%s/%s/%s", getTrace(4), getTrace(3), getTrace(2), getTrace(1), getTrace(0)), "100000000000000000000"},
		}
	)

	for _, pathAndEscrowAmount := range pathsAndEscrowAmounts {
		denomTrace := types.DenomTrace{
			BaseDenom: "uatom",
			Path:      pathAndEscrowAmount.path,
		}
		denomTraces = append(types.Traces{denomTrace}, denomTraces...)
		suite.chainA.GetSimApp().TransferKeeper.SetDenomTrace(suite.chainA.GetContext(), denomTrace)

		denom := denomTrace.IBCDenom()
		amount, ok := sdkmath.NewIntFromString(pathAndEscrowAmount.escrow)
		suite.Require().True(ok)
		escrows = append(sdk.NewCoins(sdk.NewCoin(denom, amount)), escrows...)
		suite.chainA.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainA.GetContext(), sdk.NewCoin(denom, amount))
	}

	genesis := suite.chainA.GetSimApp().TransferKeeper.ExportGenesis(suite.chainA.GetContext())

	suite.Require().Equal(types.PortID, genesis.PortId)
	suite.Require().Equal(denomTraces.Sort(), genesis.DenomTraces)
	suite.Require().Equal(escrows.Sort(), genesis.TotalEscrowed)

	suite.Require().NotPanics(func() {
		suite.chainA.GetSimApp().TransferKeeper.InitGenesis(suite.chainA.GetContext(), *genesis)
	})

	for _, denomTrace := range denomTraces {
		_, found := suite.chainA.GetSimApp().BankKeeper.GetDenomMetaData(suite.chainA.GetContext(), denomTrace.IBCDenom())
		suite.Require().True(found)
	}
}
