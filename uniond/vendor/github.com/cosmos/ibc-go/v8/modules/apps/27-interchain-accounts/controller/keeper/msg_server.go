package keeper

import (
	"context"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var _ types.MsgServer = (*msgServer)(nil)

type msgServer struct {
	*Keeper
}

// NewMsgServerImpl returns an implementation of the ICS27 MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper *Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

// RegisterInterchainAccount defines a rpc handler for MsgRegisterInterchainAccount
func (s msgServer) RegisterInterchainAccount(goCtx context.Context, msg *types.MsgRegisterInterchainAccount) (*types.MsgRegisterInterchainAccountResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	portID, err := icatypes.NewControllerPortID(msg.Owner)
	if err != nil {
		return nil, err
	}

	if s.IsMiddlewareEnabled(ctx, portID, msg.ConnectionId) && !s.IsActiveChannelClosed(ctx, msg.ConnectionId, portID) {
		return nil, errorsmod.Wrap(icatypes.ErrInvalidChannelFlow, "channel is already active or a handshake is in flight")
	}

	s.SetMiddlewareDisabled(ctx, portID, msg.ConnectionId)

	// use ORDER_ORDERED as default in case msg's ordering is NONE
	var order channeltypes.Order
	if msg.Ordering == channeltypes.NONE {
		order = channeltypes.ORDERED
	} else {
		order = msg.Ordering
	}

	channelID, err := s.registerInterchainAccount(ctx, msg.ConnectionId, portID, msg.Version, order)
	if err != nil {
		s.Logger(ctx).Error("error registering interchain account", "error", err.Error())
		return nil, err
	}

	s.Logger(ctx).Info("successfully registered interchain account", "channel-id", channelID)

	return &types.MsgRegisterInterchainAccountResponse{
		ChannelId: channelID,
		PortId:    portID,
	}, nil
}

// SendTx defines a rpc handler for MsgSendTx
func (s msgServer) SendTx(goCtx context.Context, msg *types.MsgSendTx) (*types.MsgSendTxResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	portID, err := icatypes.NewControllerPortID(msg.Owner)
	if err != nil {
		return nil, err
	}

	// the absolute timeout value is calculated using the controller chain block time + the relative timeout value
	// this assumes time synchrony to a certain degree between the controller and counterparty host chain
	absoluteTimeout := uint64(ctx.BlockTime().UnixNano()) + msg.RelativeTimeout
	seq, err := s.sendTx(ctx, msg.ConnectionId, portID, msg.PacketData, absoluteTimeout)
	if err != nil {
		return nil, err
	}

	return &types.MsgSendTxResponse{Sequence: seq}, nil
}

// UpdateParams defines an rpc handler method for MsgUpdateParams. Updates the ica/controller submodule's parameters.
func (k Keeper) UpdateParams(goCtx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if k.GetAuthority() != msg.Signer {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "expected %s, got %s", k.GetAuthority(), msg.Signer)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	k.SetParams(ctx, msg.Params)

	return &types.MsgUpdateParamsResponse{}, nil
}
