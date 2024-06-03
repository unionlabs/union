package bindings

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"

	bindingstypes "union/x/deferredack/bindings/types"
	deferredackkeeper "union/x/deferredack/keeper"
)

type QueryPlugin struct {
	bankKeeper        *bankkeeper.BaseKeeper
	deferredAckKeeper *deferredackkeeper.Keeper
}

// NewQueryPlugin returns a reference to a new QueryPlugin.
func NewQueryPlugin(b *bankkeeper.BaseKeeper, tfk *deferredackkeeper.Keeper) *QueryPlugin {
	return &QueryPlugin{
		bankKeeper:        b,
		deferredAckKeeper: tfk,
	}
}

func (qp QueryPlugin) GetParams(ctx sdk.Context) (*bindingstypes.ParamsResponse, error) {
	params := qp.deferredAckKeeper.GetParams(ctx)
	return &bindingstypes.ParamsResponse{
		Params: bindingstypes.Params{
			FeePercentage: params.FeePercentage,
		},
	}, nil
}
