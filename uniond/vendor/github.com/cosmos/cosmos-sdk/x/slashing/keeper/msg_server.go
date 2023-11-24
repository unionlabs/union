package keeper

import (
	"context"

	"cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

var _ types.MsgServer = msgServer{}

type msgServer struct {
	Keeper
}

// NewMsgServerImpl returns an implementation of the slashing MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

// UpdateParams implements MsgServer.UpdateParams method.
// It defines a method to update the x/slashing module parameters.
func (k msgServer) UpdateParams(goCtx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if k.authority != msg.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, msg.Authority)
	}

	if err := msg.Params.Validate(); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	if err := k.SetParams(ctx, msg.Params); err != nil {
		return nil, err
	}

	return &types.MsgUpdateParamsResponse{}, nil
}

// Unjail implements MsgServer.Unjail method.
// Validators must submit a transaction to unjail itself after
// having been jailed (and thus unbonded) for downtime
func (k msgServer) Unjail(goCtx context.Context, msg *types.MsgUnjail) (*types.MsgUnjailResponse, error) {
	valAddr, err := k.sk.ValidatorAddressCodec().StringToBytes(msg.ValidatorAddr)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("validator input address: %s", err)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	if err := k.Keeper.Unjail(ctx, valAddr); err != nil {
		return nil, err
	}

	return &types.MsgUnjailResponse{}, nil
}
