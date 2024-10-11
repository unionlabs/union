package keeper

import (
	"context"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var _ types.MsgServer = (*Keeper)(nil)

// RegisterPayee defines a rpc handler method for MsgRegisterPayee
// RegisterPayee is called by the relayer on each channelEnd and allows them to set an optional
// payee to which reverse and timeout relayer packet fees will be paid out. The payee should be registered on
// the source chain from which packets originate as this is where fee distribution takes place. This function may be
// called more than once by a relayer, in which case, the latest payee is always used.
func (k Keeper) RegisterPayee(goCtx context.Context, msg *types.MsgRegisterPayee) (*types.MsgRegisterPayeeResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	payee, err := sdk.AccAddressFromBech32(msg.Payee)
	if err != nil {
		return nil, err
	}

	if k.bankKeeper.BlockedAddr(payee) {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "%s is not authorized to be a payee", payee)
	}

	// only register payee address if the channel exists and is fee enabled
	if _, found := k.channelKeeper.GetChannel(ctx, msg.PortId, msg.ChannelId); !found {
		return nil, channeltypes.ErrChannelNotFound
	}

	if !k.IsFeeEnabled(ctx, msg.PortId, msg.ChannelId) {
		return nil, types.ErrFeeNotEnabled
	}

	k.SetPayeeAddress(ctx, msg.Relayer, msg.Payee, msg.ChannelId)

	k.Logger(ctx).Info("registering payee address for relayer", "relayer", msg.Relayer, "payee", msg.Payee, "channel", msg.ChannelId)

	emitRegisterPayeeEvent(ctx, msg.Relayer, msg.Payee, msg.ChannelId)

	return &types.MsgRegisterPayeeResponse{}, nil
}

// RegisterCounterpartyPayee defines a rpc handler method for MsgRegisterCounterpartyPayee
// RegisterCounterpartyPayee is called by the relayer on each channelEnd and allows them to specify the counterparty
// payee address before relaying. This ensures they will be properly compensated for forward relaying since
// the destination chain must include the registered counterparty payee address in the acknowledgement. This function
// may be called more than once by a relayer, in which case, the latest counterparty payee address is always used.
func (k Keeper) RegisterCounterpartyPayee(goCtx context.Context, msg *types.MsgRegisterCounterpartyPayee) (*types.MsgRegisterCounterpartyPayeeResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// only register counterparty payee if the channel exists and is fee enabled
	if _, found := k.channelKeeper.GetChannel(ctx, msg.PortId, msg.ChannelId); !found {
		return nil, channeltypes.ErrChannelNotFound
	}

	if !k.IsFeeEnabled(ctx, msg.PortId, msg.ChannelId) {
		return nil, types.ErrFeeNotEnabled
	}

	k.SetCounterpartyPayeeAddress(ctx, msg.Relayer, msg.CounterpartyPayee, msg.ChannelId)

	k.Logger(ctx).Info("registering counterparty payee for relayer", "relayer", msg.Relayer, "counterparty payee", msg.CounterpartyPayee, "channel", msg.ChannelId)

	emitRegisterCounterpartyPayeeEvent(ctx, msg.Relayer, msg.CounterpartyPayee, msg.ChannelId)

	return &types.MsgRegisterCounterpartyPayeeResponse{}, nil
}

// PayPacketFee defines a rpc handler method for MsgPayPacketFee
// PayPacketFee is an open callback that may be called by any module/user that wishes to escrow funds in order to relay the packet with the next sequence
func (k Keeper) PayPacketFee(goCtx context.Context, msg *types.MsgPayPacketFee) (*types.MsgPayPacketFeeResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	if !k.IsFeeEnabled(ctx, msg.SourcePortId, msg.SourceChannelId) {
		// users may not escrow fees on this channel. Must send packets without a fee message
		return nil, types.ErrFeeNotEnabled
	}

	if k.IsLocked(ctx) {
		return nil, types.ErrFeeModuleLocked
	}

	refundAcc, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return nil, err
	}

	if err := k.bankKeeper.IsSendEnabledCoins(ctx, msg.Fee.Total()...); err != nil {
		return nil, err
	}

	if k.bankKeeper.BlockedAddr(refundAcc) {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "%s is not allowed to escrow fees", refundAcc)
	}

	// get the next sequence
	sequence, found := k.GetNextSequenceSend(ctx, msg.SourcePortId, msg.SourceChannelId)
	if !found {
		return nil, channeltypes.ErrSequenceSendNotFound
	}

	packetID := channeltypes.NewPacketID(msg.SourcePortId, msg.SourceChannelId, sequence)
	packetFee := types.NewPacketFee(msg.Fee, msg.Signer, msg.Relayers)

	if err := k.escrowPacketFee(ctx, packetID, packetFee); err != nil {
		return nil, err
	}

	return &types.MsgPayPacketFeeResponse{}, nil
}

// PayPacketFeeAsync defines a rpc handler method for MsgPayPacketFeeAsync
// PayPacketFeeAsync is an open callback that may be called by any module/user that wishes to escrow funds in order to
// incentivize the relaying of a known packet. Only packets which have been sent and have not gone through the
// packet life cycle may be incentivized.
func (k Keeper) PayPacketFeeAsync(goCtx context.Context, msg *types.MsgPayPacketFeeAsync) (*types.MsgPayPacketFeeAsyncResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	if !k.IsFeeEnabled(ctx, msg.PacketId.PortId, msg.PacketId.ChannelId) {
		// users may not escrow fees on this channel. Must send packets without a fee message
		return nil, types.ErrFeeNotEnabled
	}

	if k.IsLocked(ctx) {
		return nil, types.ErrFeeModuleLocked
	}

	refundAcc, err := sdk.AccAddressFromBech32(msg.PacketFee.RefundAddress)
	if err != nil {
		return nil, err
	}

	if err := k.bankKeeper.IsSendEnabledCoins(ctx, msg.PacketFee.Fee.Total()...); err != nil {
		return nil, err
	}

	if k.bankKeeper.BlockedAddr(refundAcc) {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "%s is not allowed to escrow fees", refundAcc)
	}

	nextSeqSend, found := k.GetNextSequenceSend(ctx, msg.PacketId.PortId, msg.PacketId.ChannelId)
	if !found {
		return nil, errorsmod.Wrapf(channeltypes.ErrSequenceSendNotFound, "channel does not exist, portID: %s, channelID: %s", msg.PacketId.PortId, msg.PacketId.ChannelId)
	}

	// only allow incentivizing of packets which have been sent
	if msg.PacketId.Sequence >= nextSeqSend {
		return nil, channeltypes.ErrPacketNotSent
	}

	// only allow incentivizng of packets which have not completed the packet life cycle
	if bz := k.GetPacketCommitment(ctx, msg.PacketId.PortId, msg.PacketId.ChannelId, msg.PacketId.Sequence); len(bz) == 0 {
		return nil, errorsmod.Wrapf(channeltypes.ErrPacketCommitmentNotFound, "packet has already been acknowledged or timed out")
	}

	if err := k.escrowPacketFee(ctx, msg.PacketId, msg.PacketFee); err != nil {
		return nil, err
	}

	return &types.MsgPayPacketFeeAsyncResponse{}, nil
}
