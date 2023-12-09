package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// EmitAcknowledgementEvent emits an event signalling a successful or failed acknowledgement and including the error
// details if any.
func EmitAcknowledgementEvent(ctx sdk.Context, packet exported.PacketI, ack exported.Acknowledgement, err error) {
	attributes := []sdk.Attribute{
		sdk.NewAttribute(sdk.AttributeKeyModule, icatypes.ModuleName),
		sdk.NewAttribute(icatypes.AttributeKeyControllerChannelID, packet.GetDestChannel()),
		sdk.NewAttribute(icatypes.AttributeKeyAckSuccess, fmt.Sprintf("%t", ack.Success())),
	}

	if err != nil {
		attributes = append(attributes, sdk.NewAttribute(icatypes.AttributeKeyAckError, err.Error()))
	}

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			icatypes.EventTypePacket,
			attributes...,
		),
	)
}
