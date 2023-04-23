package keeper

import (
	"fmt"
	"strings"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	capabilitytypes "github.com/cosmos/cosmos-sdk/x/capability/types"

	icatypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"
)

// OnChanOpenInit performs basic validation of channel initialization.
// The channel order must be ORDERED, the counterparty port identifier
// must be the host chain representation as defined in the types package,
// the channel version must be equal to the version in the types package,
// there must not be an active channel for the specfied port identifier,
// and the interchain accounts module must be able to claim the channel
// capability.
func (k Keeper) OnChanOpenInit(
	ctx sdk.Context,
	order channeltypes.Order,
	connectionHops []string,
	portID string,
	channelID string,
	chanCap *capabilitytypes.Capability,
	counterparty channeltypes.Counterparty,
	version string,
) (string, error) {
	if order != channeltypes.ORDERED {
		return "", sdkerrors.Wrapf(channeltypes.ErrInvalidChannelOrdering, "expected %s channel, got %s", channeltypes.ORDERED, order)
	}

	if !strings.HasPrefix(portID, icatypes.ControllerPortPrefix) {
		return "", sdkerrors.Wrapf(icatypes.ErrInvalidControllerPort, "expected %s{owner-account-address}, got %s", icatypes.ControllerPortPrefix, portID)
	}

	if counterparty.PortId != icatypes.HostPortID {
		return "", sdkerrors.Wrapf(icatypes.ErrInvalidHostPort, "expected %s, got %s", icatypes.HostPortID, counterparty.PortId)
	}

	var metadata icatypes.Metadata
	if strings.TrimSpace(version) == "" {
		connection, err := k.channelKeeper.GetConnection(ctx, connectionHops[0])
		if err != nil {
			return "", err
		}

		metadata = icatypes.NewDefaultMetadata(connectionHops[0], connection.GetCounterparty().GetConnectionID())
	} else {
		if err := icatypes.ModuleCdc.UnmarshalJSON([]byte(version), &metadata); err != nil {
			return "", sdkerrors.Wrapf(icatypes.ErrUnknownDataType, "cannot unmarshal ICS-27 interchain accounts metadata")
		}
	}

	if err := icatypes.ValidateControllerMetadata(ctx, k.channelKeeper, connectionHops, metadata); err != nil {
		return "", err
	}

	activeChannelID, found := k.GetActiveChannelID(ctx, connectionHops[0], portID)
	if found {
		channel, found := k.channelKeeper.GetChannel(ctx, portID, activeChannelID)
		if !found {
			panic(fmt.Sprintf("active channel mapping set for %s but channel does not exist in channel store", activeChannelID))
		}

		if channel.State == channeltypes.OPEN {
			return "", sdkerrors.Wrapf(icatypes.ErrActiveChannelAlreadySet, "existing active channel %s for portID %s is already OPEN", activeChannelID, portID)
		}

		appVersion, found := k.GetAppVersion(ctx, portID, activeChannelID)
		if !found {
			panic(fmt.Sprintf("active channel mapping set for %s, but channel does not exist in channel store", activeChannelID))
		}

		if !icatypes.IsPreviousMetadataEqual(appVersion, metadata) {
			return "", sdkerrors.Wrap(icatypes.ErrInvalidVersion, "previous active channel metadata does not match provided version")
		}
	}

	return string(icatypes.ModuleCdc.MustMarshalJSON(&metadata)), nil
}

// OnChanOpenAck sets the active channel for the interchain account/owner pair
// and stores the associated interchain account address in state keyed by it's corresponding port identifier
func (k Keeper) OnChanOpenAck(
	ctx sdk.Context,
	portID,
	channelID string,
	counterpartyVersion string,
) error {
	if portID == icatypes.HostPortID {
		return sdkerrors.Wrapf(icatypes.ErrInvalidControllerPort, "portID cannot be host chain port ID: %s", icatypes.HostPortID)
	}

	if !strings.HasPrefix(portID, icatypes.ControllerPortPrefix) {
		return sdkerrors.Wrapf(icatypes.ErrInvalidControllerPort, "expected %s{owner-account-address}, got %s", icatypes.ControllerPortPrefix, portID)
	}

	var metadata icatypes.Metadata
	if err := icatypes.ModuleCdc.UnmarshalJSON([]byte(counterpartyVersion), &metadata); err != nil {
		return sdkerrors.Wrapf(icatypes.ErrUnknownDataType, "cannot unmarshal ICS-27 interchain accounts metadata")
	}

	if activeChannelID, found := k.GetOpenActiveChannel(ctx, metadata.ControllerConnectionId, portID); found {
		return sdkerrors.Wrapf(icatypes.ErrActiveChannelAlreadySet, "existing active channel %s for portID %s", activeChannelID, portID)
	}

	channel, found := k.channelKeeper.GetChannel(ctx, portID, channelID)
	if !found {
		return sdkerrors.Wrapf(channeltypes.ErrChannelNotFound, "failed to retrieve channel %s on port %s", channelID, portID)
	}

	if err := icatypes.ValidateControllerMetadata(ctx, k.channelKeeper, channel.ConnectionHops, metadata); err != nil {
		return err
	}

	if strings.TrimSpace(metadata.Address) == "" {
		return sdkerrors.Wrap(icatypes.ErrInvalidAccountAddress, "interchain account address cannot be empty")
	}

	k.SetActiveChannelID(ctx, metadata.ControllerConnectionId, portID, channelID)
	k.SetInterchainAccountAddress(ctx, metadata.ControllerConnectionId, portID, metadata.Address)

	return nil
}

// OnChanCloseConfirm removes the active channel stored in state
func (k Keeper) OnChanCloseConfirm(
	ctx sdk.Context,
	portID,
	channelID string,
) error {
	return nil
}
