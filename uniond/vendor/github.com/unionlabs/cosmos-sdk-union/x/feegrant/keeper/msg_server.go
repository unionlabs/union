package keeper

import (
	"context"
	"strings"

	"cosmossdk.io/core/event"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/feegrant"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

type msgServer struct {
	Keeper
}

// NewMsgServerImpl returns an implementation of the feegrant MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(k Keeper) feegrant.MsgServer {
	return &msgServer{
		Keeper: k,
	}
}

var _ feegrant.MsgServer = msgServer{}

// GrantAllowance grants an allowance from the granter's funds to be used by the grantee.
func (k msgServer) GrantAllowance(ctx context.Context, msg *feegrant.MsgGrantAllowance) (*feegrant.MsgGrantAllowanceResponse, error) {
	if strings.EqualFold(msg.Grantee, msg.Granter) {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, "cannot self-grant fee authorization")
	}

	grantee, err := k.authKeeper.AddressCodec().StringToBytes(msg.Grantee)
	if err != nil {
		return nil, err
	}

	granter, err := k.authKeeper.AddressCodec().StringToBytes(msg.Granter)
	if err != nil {
		return nil, err
	}

	if f, _ := k.GetAllowance(ctx, granter, grantee); f != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "fee allowance already exists")
	}

	allowance, err := msg.GetFeeAllowanceI()
	if err != nil {
		return nil, err
	}

	if err := allowance.ValidateBasic(); err != nil {
		return nil, err
	}

	err = k.Keeper.GrantAllowance(ctx, granter, grantee, allowance)
	if err != nil {
		return nil, err
	}

	return &feegrant.MsgGrantAllowanceResponse{}, nil
}

// RevokeAllowance revokes a fee allowance between a granter and grantee.
func (k msgServer) RevokeAllowance(ctx context.Context, msg *feegrant.MsgRevokeAllowance) (*feegrant.MsgRevokeAllowanceResponse, error) {
	if msg.Grantee == msg.Granter {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, "addresses must be different")
	}

	grantee, err := k.authKeeper.AddressCodec().StringToBytes(msg.Grantee)
	if err != nil {
		return nil, err
	}

	granter, err := k.authKeeper.AddressCodec().StringToBytes(msg.Granter)
	if err != nil {
		return nil, err
	}

	err = k.Keeper.revokeAllowance(ctx, granter, grantee)
	if err != nil {
		return nil, err
	}

	return &feegrant.MsgRevokeAllowanceResponse{}, nil
}

// PruneAllowances removes expired allowances from the store.
func (k msgServer) PruneAllowances(ctx context.Context, req *feegrant.MsgPruneAllowances) (*feegrant.MsgPruneAllowancesResponse, error) {
	// 75 is an arbitrary value, we can change it later if needed
	err := k.RemoveExpiredAllowances(ctx, 75)
	if err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		feegrant.EventTypePruneFeeGrant,
		event.NewAttribute(feegrant.AttributeKeyPruner, req.Pruner),
	); err != nil {
		return nil, err
	}

	return &feegrant.MsgPruneAllowancesResponse{}, nil
}
