package bindings

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"

	bindingstypes "union/x/diferredack/bindings/types"
	diferredackkeeper "union/x/diferredack/keeper"
)

type QueryPlugin struct {
	bankKeeper        *bankkeeper.BaseKeeper
	diferredAckKeeper *diferredackkeeper.Keeper
}

// NewQueryPlugin returns a reference to a new QueryPlugin.
func NewQueryPlugin(b *bankkeeper.BaseKeeper, tfk *diferredackkeeper.Keeper) *QueryPlugin {
	return &QueryPlugin{
		bankKeeper:        b,
		diferredAckKeeper: tfk,
	}
}

func (qp QueryPlugin) GetParams(ctx sdk.Context) (*bindingstypes.ParamsResponse, error) {
	params := qp.diferredAckKeeper.GetParams(ctx)
	return &bindingstypes.ParamsResponse{
		Params: bindingstypes.Params{
			FeePercentage: params.FeePercentage,
		},
	}, nil
}
