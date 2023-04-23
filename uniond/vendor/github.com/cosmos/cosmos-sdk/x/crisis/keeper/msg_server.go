package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/crisis/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
)

var _ types.MsgServer = &Keeper{}

// VerifyInvariant implements MsgServer.VerifyInvariant method.
// It defines a method to verify a particular invariant.
func (k *Keeper) VerifyInvariant(goCtx context.Context, msg *types.MsgVerifyInvariant) (*types.MsgVerifyInvariantResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	constantFee := sdk.NewCoins(k.GetConstantFee(ctx))

	sender, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, err
	}
	if err := k.SendCoinsFromAccountToFeeCollector(ctx, sender, constantFee); err != nil {
		return nil, err
	}

	// use a cached context to avoid gas costs during invariants
	cacheCtx, _ := ctx.CacheContext()

	found := false
	msgFullRoute := msg.FullInvariantRoute()

	var res string
	var stop bool
	for _, invarRoute := range k.Routes() {
		if invarRoute.FullRoute() == msgFullRoute {
			res, stop = invarRoute.Invar(cacheCtx)
			found = true

			break
		}
	}

	if !found {
		return nil, types.ErrUnknownInvariant
	}

	if stop {
		// Currently, because the chain halts here, this transaction will never be included in the
		// blockchain thus the constant fee will have never been deducted. Thus no refund is required.

		// TODO replace with circuit breaker
		panic(res)
	}

	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeInvariant,
			sdk.NewAttribute(types.AttributeKeyRoute, msg.InvariantRoute),
		),
	})

	return &types.MsgVerifyInvariantResponse{}, nil
}

// UpdateParams implements MsgServer.UpdateParams method.
// It defines a method to update the x/crisis module parameters.
func (k *Keeper) UpdateParams(goCtx context.Context, req *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if k.authority != req.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, req.Authority)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	if err := k.SetConstantFee(ctx, req.ConstantFee); err != nil {
		return nil, err
	}

	return &types.MsgUpdateParamsResponse{}, nil
}
