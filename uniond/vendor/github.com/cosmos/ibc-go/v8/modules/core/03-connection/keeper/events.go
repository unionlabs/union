package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
)

// emitConnectionOpenInitEvent emits a connection open init event
func emitConnectionOpenInitEvent(ctx context.Context, connectionID string, clientID string, counterparty types.Counterparty) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeConnectionOpenInit,
			sdk.NewAttribute(types.AttributeKeyConnectionID, connectionID),
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyCounterpartyClientID, counterparty.ClientId),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitConnectionOpenTryEvent emits a connection open try event
func emitConnectionOpenTryEvent(ctx context.Context, connectionID string, clientID string, counterparty types.Counterparty) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeConnectionOpenTry,
			sdk.NewAttribute(types.AttributeKeyConnectionID, connectionID),
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyCounterpartyClientID, counterparty.ClientId),
			sdk.NewAttribute(types.AttributeKeyCounterpartyConnectionID, counterparty.ConnectionId),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitConnectionOpenAckEvent emits a connection open acknowledge event
func emitConnectionOpenAckEvent(ctx context.Context, connectionID string, connectionEnd types.ConnectionEnd) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeConnectionOpenAck,
			sdk.NewAttribute(types.AttributeKeyConnectionID, connectionID),
			sdk.NewAttribute(types.AttributeKeyClientID, connectionEnd.ClientId),
			sdk.NewAttribute(types.AttributeKeyCounterpartyClientID, connectionEnd.Counterparty.ClientId),
			sdk.NewAttribute(types.AttributeKeyCounterpartyConnectionID, connectionEnd.Counterparty.ConnectionId),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitConnectionOpenConfirmEvent emits a connection open confirm event
func emitConnectionOpenConfirmEvent(ctx context.Context, connectionID string, connectionEnd types.ConnectionEnd) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeConnectionOpenConfirm,
			sdk.NewAttribute(types.AttributeKeyConnectionID, connectionID),
			sdk.NewAttribute(types.AttributeKeyClientID, connectionEnd.ClientId),
			sdk.NewAttribute(types.AttributeKeyCounterpartyClientID, connectionEnd.Counterparty.ClientId),
			sdk.NewAttribute(types.AttributeKeyCounterpartyConnectionID, connectionEnd.Counterparty.ConnectionId),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}
