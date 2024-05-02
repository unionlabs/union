package keeper

import (
	"fmt"
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// OnChanOpenTry performs basic validation of the ICA channel
// and registers a new interchain account (if it doesn't exist).
// The version returned will include the registered interchain
// account address.
func (k Keeper) OnChanOpenTry(
	ctx sdk.Context,
	order channeltypes.Order,
	connectionHops []string,
	portID,
	channelID string,
	chanCap *capabilitytypes.Capability,
	counterparty channeltypes.Counterparty,
	counterpartyVersion string,
) (string, error) {
	if portID != icatypes.HostPortID {
		return "", errorsmod.Wrapf(icatypes.ErrInvalidHostPort, "expected %s, got %s", icatypes.HostPortID, portID)
	}

	metadata, err := icatypes.MetadataFromVersion(counterpartyVersion)
	if err != nil {
		return "", err
	}

	if err = icatypes.ValidateHostMetadata(ctx, k.channelKeeper, connectionHops, metadata); err != nil {
		return "", err
	}

	activeChannelID, found := k.GetActiveChannelID(ctx, connectionHops[0], counterparty.PortId)
	if found {
		channel, found := k.channelKeeper.GetChannel(ctx, portID, activeChannelID)
		if !found {
			panic(fmt.Errorf("active channel mapping set for %s but channel does not exist in channel store", activeChannelID))
		}

		if channel.State != channeltypes.CLOSED {
			return "", errorsmod.Wrapf(icatypes.ErrActiveChannelAlreadySet, "existing active channel %s for portID %s must be %s", activeChannelID, portID, channeltypes.CLOSED)
		}

		// if a channel is being reopened, we allow the controller to propose new fields
		// which are not exactly the same as the previous. The provided address will
		// be overwritten with the correct one before the metadata is returned.
	}

	// On the host chain the capability may only be claimed during the OnChanOpenTry
	// The capability being claimed in OpenInit is for a controller chain (the port is different)
	if err = k.ClaimCapability(ctx, chanCap, host.ChannelCapabilityPath(portID, channelID)); err != nil {
		return "", errorsmod.Wrapf(err, "failed to claim capability for channel %s on port %s", channelID, portID)
	}

	var accAddress sdk.AccAddress

	interchainAccAddr, found := k.GetInterchainAccountAddress(ctx, metadata.HostConnectionId, counterparty.PortId)
	if found {
		// reopening an interchain account
		k.Logger(ctx).Info("reopening existing interchain account", "address", interchainAccAddr)
		accAddress = sdk.MustAccAddressFromBech32(interchainAccAddr)
		if _, ok := k.accountKeeper.GetAccount(ctx, accAddress).(*icatypes.InterchainAccount); !ok {
			return "", errorsmod.Wrapf(icatypes.ErrInvalidAccountReopening, "existing account address %s, does not have interchain account type", accAddress)
		}

	} else {
		accAddress, err = k.createInterchainAccount(ctx, metadata.HostConnectionId, counterparty.PortId)
		if err != nil {
			return "", err
		}
		k.Logger(ctx).Info("successfully created new interchain account", "host-connection-id", metadata.HostConnectionId, "port-id", counterparty.PortId, "address", accAddress)
	}

	metadata.Address = accAddress.String()
	versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
	if err != nil {
		return "", err
	}

	return string(versionBytes), nil
}

// OnChanOpenConfirm completes the handshake process by setting the active channel in state on the host chain
func (k Keeper) OnChanOpenConfirm(
	ctx sdk.Context,
	portID,
	channelID string,
) error {
	channel, found := k.channelKeeper.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(channeltypes.ErrChannelNotFound, "failed to retrieve channel %s on port %s", channelID, portID)
	}

	// It is assumed the controller chain will not allow multiple active channels to be created for the same connectionID/portID
	// If the controller chain does allow multiple active channels to be created for the same connectionID/portID,
	// disallowing overwriting the current active channel guarantees the channel can no longer be used as the controller
	// and host will disagree on what the currently active channel is
	k.SetActiveChannelID(ctx, channel.ConnectionHops[0], channel.Counterparty.PortId, channelID)

	return nil
}

// OnChanCloseConfirm removes the active channel stored in state
func (Keeper) OnChanCloseConfirm(
	ctx sdk.Context,
	portID,
	channelID string,
) error {
	return nil
}

// OnChanUpgradeTry performs the upgrade try step of the channel upgrade handshake.
// The upgrade try callback must verify the proposed changes to the order, connectionHops, and version.
// Within the version we have the tx type, encoding, interchain account address, host/controller connectionID's
// and the ICS27 protocol version.
//
// The following may be changed:
// - tx type (must be supported)
// - encoding (must be supported)
// - order
//
// The following may not be changed:
// - connectionHops (and subsequently host/controller connectionIDs)
// - interchain account address
// - ICS27 protocol version
func (k Keeper) OnChanUpgradeTry(ctx sdk.Context, portID, channelID string, proposedOrder channeltypes.Order, proposedConnectionHops []string, counterpartyVersion string) (string, error) {
	if portID != icatypes.HostPortID {
		return "", errorsmod.Wrapf(porttypes.ErrInvalidPort, "expected %s, got %s", icatypes.HostPortID, portID)
	}

	// verify connection hops has not changed
	connectionID, err := k.getConnectionID(ctx, portID, channelID)
	if err != nil {
		return "", err
	}

	if len(proposedConnectionHops) != 1 || proposedConnectionHops[0] != connectionID {
		return "", errorsmod.Wrapf(channeltypes.ErrInvalidUpgrade, "expected connection hops %s, got %s", []string{connectionID}, proposedConnectionHops)
	}

	if strings.TrimSpace(counterpartyVersion) == "" {
		return "", errorsmod.Wrap(channeltypes.ErrInvalidChannelVersion, "counterparty version cannot be empty")
	}

	proposedCounterpartyMetadata, err := icatypes.MetadataFromVersion(counterpartyVersion)
	if err != nil {
		return "", err
	}

	currentMetadata, err := k.getAppMetadata(ctx, portID, channelID)
	if err != nil {
		return "", err
	}

	// ValidateHostMetadata will ensure the ICS27 protocol version has not changed and that the
	// tx type and encoding are supported. It also validates the connection params against the counterparty metadata.
	if err := icatypes.ValidateHostMetadata(ctx, k.channelKeeper, proposedConnectionHops, proposedCounterpartyMetadata); err != nil {
		return "", errorsmod.Wrap(err, "invalid metadata")
	}

	// the interchain account address on the host chain
	// must remain the same after the upgrade.
	if currentMetadata.Address != proposedCounterpartyMetadata.Address {
		return "", errorsmod.Wrap(icatypes.ErrInvalidAccountAddress, "interchain account address cannot be changed")
	}

	// these explicit checks on the controller connection identifier should be unreachable
	if currentMetadata.ControllerConnectionId != proposedCounterpartyMetadata.ControllerConnectionId {
		return "", errorsmod.Wrap(connectiontypes.ErrInvalidConnection, "proposed controller connection ID must not change")
	}

	// these explicit checks on the host connection identifier should be unreachable
	if currentMetadata.HostConnectionId != proposedConnectionHops[0] {
		return "", errorsmod.Wrap(connectiontypes.ErrInvalidConnectionIdentifier, "proposed connection hop must not change")
	}

	return counterpartyVersion, nil
}
