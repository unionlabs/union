package simulation

import (
	"context"

	"cosmossdk.io/x/protocolpool/types"

	"github.com/cosmos/cosmos-sdk/simsx"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func MsgFundCommunityPoolFactory() simsx.SimMsgFactoryFn[*types.MsgFundCommunityPool] {
	return func(_ context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgFundCommunityPool) {
		funder := testData.AnyAccount(reporter, simsx.WithSpendableBalance())
		fundAmount := funder.LiquidBalance().RandSubsetCoins(reporter)
		msg := types.NewMsgFundCommunityPool(fundAmount, funder.AddressBech32)
		return []simsx.SimAccount{funder}, msg
	}
}

// MsgCommunityPoolSpendFactory creates a gov proposal to send tokens from the community pool to a random account
func MsgCommunityPoolSpendFactory() simsx.SimMsgFactoryFn[*types.MsgCommunityPoolSpend] {
	return func(_ context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgCommunityPoolSpend) {
		return nil, &types.MsgCommunityPoolSpend{
			Authority: testData.ModuleAccountAddress(reporter, "gov"),
			Recipient: testData.AnyAccount(reporter).AddressBech32,
			Amount:    must(sdk.ParseCoinsNormalized("100stake,2testtoken")),
		}
	}
}

func must[T any](r T, err error) T {
	if err != nil {
		panic(err)
	}
	return r
}
