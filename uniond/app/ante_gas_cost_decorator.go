package app

import (
	storetypes "cosmossdk.io/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type GasCostDecorator struct {
}

func NewGasCostDecorator() GasCostDecorator {
	return GasCostDecorator{}
}

func (gc GasCostDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (_ sdk.Context, err error) {
	// Updates write cost by 100x
	gasConfig := storetypes.KVGasConfig()
	gasConfig.WriteCostFlat = 20_000
	gasConfig.WriteCostPerByte = 3_000
	ctx = ctx.WithKVGasConfig(gasConfig)

	return next(ctx, tx, simulate)
}
