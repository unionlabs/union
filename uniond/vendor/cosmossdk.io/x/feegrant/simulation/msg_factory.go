package simulation

import (
	"context"

	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/keeper"

	"github.com/cosmos/cosmos-sdk/simsx"
)

func MsgGrantAllowanceFactory(k keeper.Keeper) simsx.SimMsgFactoryFn[*feegrant.MsgGrantAllowance] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *feegrant.MsgGrantAllowance) {
		granter := testData.AnyAccount(reporter, simsx.WithSpendableBalance())
		grantee := testData.AnyAccount(reporter, simsx.ExcludeAccounts(granter))
		if reporter.IsSkipped() {
			return nil, nil
		}
		if f, _ := k.GetAllowance(ctx, granter.Address, grantee.Address); f != nil {
			reporter.Skip("fee allowance exists")
			return nil, nil
		}

		coins := granter.LiquidBalance().RandSubsetCoins(reporter, simsx.WithSendEnabledCoins())
		oneYear := simsx.BlockTime(ctx).AddDate(1, 0, 0)
		msg, err := feegrant.NewMsgGrantAllowance(
			&feegrant.BasicAllowance{SpendLimit: coins, Expiration: &oneYear},
			granter.AddressBech32,
			grantee.AddressBech32,
		)
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}
		return []simsx.SimAccount{granter}, msg
	}
}

func MsgRevokeAllowanceFactory(k keeper.Keeper) simsx.SimMsgFactoryFn[*feegrant.MsgRevokeAllowance] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *feegrant.MsgRevokeAllowance) {
		var gotGrant *feegrant.Grant
		if err := k.IterateAllFeeAllowances(ctx, func(grant feegrant.Grant) bool {
			gotGrant = &grant
			return true
		}); err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}
		if gotGrant == nil {
			reporter.Skip("no grant found")
			return nil, nil
		}
		granter := testData.GetAccount(reporter, gotGrant.Granter)
		grantee := testData.GetAccount(reporter, gotGrant.Grantee)
		msg := feegrant.NewMsgRevokeAllowance(granter.AddressBech32, grantee.AddressBech32)
		return []simsx.SimAccount{granter}, &msg
	}
}
