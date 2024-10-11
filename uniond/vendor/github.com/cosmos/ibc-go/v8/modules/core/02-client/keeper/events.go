package keeper

import (
	"context"
	"encoding/hex"
	"fmt"
	"strconv"
	"strings"

	upgradetypes "cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// emitCreateClientEvent emits a create client event
func emitCreateClientEvent(ctx context.Context, clientID string, clientState exported.ClientState) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeCreateClient,
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyClientType, clientState.ClientType()),
			sdk.NewAttribute(types.AttributeKeyConsensusHeight, clientState.GetLatestHeight().String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitUpdateClientEvent emits an update client event
func emitUpdateClientEvent(ctx context.Context, clientID string, clientType string, consensusHeights []exported.Height, cdc codec.BinaryCodec, clientMsg exported.ClientMessage) {
	// Marshal the ClientMessage as an Any and encode the resulting bytes to hex.
	// This prevents the event value from containing invalid UTF-8 characters
	// which may cause data to be lost when JSON encoding/decoding.
	clientMsgStr := hex.EncodeToString(types.MustMarshalClientMessage(cdc, clientMsg))

	var consensusHeightAttr string
	if len(consensusHeights) != 0 {
		consensusHeightAttr = consensusHeights[0].String()
	}

	consensusHeightsAttr := make([]string, len(consensusHeights))
	for i, height := range consensusHeights {
		consensusHeightsAttr[i] = height.String()
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeUpdateClient,
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyClientType, clientType),
			// Deprecated: AttributeKeyConsensusHeight is deprecated and will be removed in a future release.
			// Please use AttributeKeyConsensusHeights instead.
			sdk.NewAttribute(types.AttributeKeyConsensusHeight, consensusHeightAttr),
			sdk.NewAttribute(types.AttributeKeyConsensusHeights, strings.Join(consensusHeightsAttr, ",")),
			sdk.NewAttribute(types.AttributeKeyHeader, clientMsgStr),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitUpgradeClientEvent emits an upgrade client event
func emitUpgradeClientEvent(ctx context.Context, clientID string, clientState exported.ClientState) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeUpgradeClient,
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyClientType, clientState.ClientType()),
			sdk.NewAttribute(types.AttributeKeyConsensusHeight, clientState.GetLatestHeight().String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitSubmitMisbehaviourEvent emits a client misbehaviour event
func emitSubmitMisbehaviourEvent(ctx context.Context, clientID string, clientState exported.ClientState) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeSubmitMisbehaviour,
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyClientType, clientState.ClientType()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitRecoverClientEvent emits a recover client event
func emitRecoverClientEvent(ctx context.Context, clientID, clientType string) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeRecoverClient,
			sdk.NewAttribute(types.AttributeKeySubjectClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyClientType, clientType),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitScheduleIBCSoftwareUpgradeEvent emits a schedule IBC software upgrade event
func emitScheduleIBCSoftwareUpgradeEvent(ctx context.Context, title string, height int64) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeScheduleIBCSoftwareUpgrade,
			sdk.NewAttribute(types.AttributeKeyUpgradePlanTitle, title),
			sdk.NewAttribute(types.AttributeKeyUpgradePlanHeight, fmt.Sprintf("%d", height)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitUpgradeChainEvent emits an upgrade chain event.
func EmitUpgradeChainEvent(ctx context.Context, height int64) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeUpgradeChain,
			sdk.NewAttribute(types.AttributeKeyUpgradePlanHeight, strconv.FormatInt(height, 10)),
			sdk.NewAttribute(types.AttributeKeyUpgradeStore, upgradetypes.StoreKey), // which store to query proof of consensus state from
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}
