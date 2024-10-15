package keeper

import (
	"context"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// emitIncentivizedPacketEvent emits an event containing information on the total amount of fees incentivizing
// a specific packet. It should be emitted on every fee escrowed for the given packetID.
func emitIncentivizedPacketEvent(ctx context.Context, packetID channeltypes.PacketId, packetFees types.PacketFees) {
	var (
		totalRecvFees    sdk.Coins
		totalAckFees     sdk.Coins
		totalTimeoutFees sdk.Coins
	)

	for _, fee := range packetFees.PacketFees {
		// only emit total fees for packet fees which allow any relayer to relay
		if fee.Relayers == nil {
			totalRecvFees = totalRecvFees.Add(fee.Fee.RecvFee...)
			totalAckFees = totalAckFees.Add(fee.Fee.AckFee...)
			totalTimeoutFees = totalTimeoutFees.Add(fee.Fee.TimeoutFee...)
		}
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeIncentivizedPacket,
			sdk.NewAttribute(channeltypes.AttributeKeyPortID, packetID.PortId),
			sdk.NewAttribute(channeltypes.AttributeKeyChannelID, packetID.ChannelId),
			sdk.NewAttribute(channeltypes.AttributeKeySequence, fmt.Sprint(packetID.Sequence)),
			sdk.NewAttribute(types.AttributeKeyRecvFee, totalRecvFees.String()),
			sdk.NewAttribute(types.AttributeKeyAckFee, totalAckFees.String()),
			sdk.NewAttribute(types.AttributeKeyTimeoutFee, totalTimeoutFees.String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.ModuleName),
		),
	})
}

// emitRegisterPayeeEvent emits an event containing information of a registered payee for a relayer on a particular channel
func emitRegisterPayeeEvent(ctx context.Context, relayer, payee, channelID string) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeRegisterPayee,
			sdk.NewAttribute(types.AttributeKeyRelayer, relayer),
			sdk.NewAttribute(types.AttributeKeyPayee, payee),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.ModuleName),
		),
	})
}

// emitRegisterCounterpartyPayeeEvent emits an event containing information of a registered counterparty payee for a relayer on a particular channel
func emitRegisterCounterpartyPayeeEvent(ctx context.Context, relayer, counterpartyPayee, channelID string) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeRegisterCounterpartyPayee,
			sdk.NewAttribute(types.AttributeKeyRelayer, relayer),
			sdk.NewAttribute(types.AttributeKeyCounterpartyPayee, counterpartyPayee),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.ModuleName),
		),
	})
}

// emitDistributeFeeEvent emits an event containing a distribution fee and receiver address
func emitDistributeFeeEvent(ctx context.Context, receiver string, fee sdk.Coins) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeDistributeFee,
			sdk.NewAttribute(types.AttributeKeyReceiver, receiver),
			sdk.NewAttribute(types.AttributeKeyFee, fee.String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.ModuleName),
		),
	})
}
