package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	capabilitytypes "github.com/cosmos/cosmos-sdk/x/capability/types"

	controllertypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/controller/types"
	icatypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper *Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper *Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// AssertChannelCapabilityMigrations checks that all channel capabilities generated using the interchain accounts controller port prefix
// are owned by the controller submodule and ibc.
func (m Migrator) AssertChannelCapabilityMigrations(ctx sdk.Context) error {
	if m.keeper != nil {
		logger := m.keeper.Logger(ctx)
		filteredChannels := m.keeper.channelKeeper.GetAllChannelsWithPortPrefix(ctx, icatypes.ControllerPortPrefix)
		for _, ch := range filteredChannels {
			name := host.ChannelCapabilityPath(ch.PortId, ch.ChannelId)
			capability, found := m.keeper.scopedKeeper.GetCapability(ctx, name)
			if !found {
				logger.Error(fmt.Sprintf("failed to find capability: %s", name))
				return sdkerrors.Wrapf(capabilitytypes.ErrCapabilityNotFound, "failed to find capability: %s", name)
			}

			isAuthenticated := m.keeper.scopedKeeper.AuthenticateCapability(ctx, capability, name)
			if !isAuthenticated {
				logger.Error(fmt.Sprintf("expected capability owner: %s", controllertypes.SubModuleName))
				return sdkerrors.Wrapf(capabilitytypes.ErrCapabilityNotOwned, "expected capability owner: %s", controllertypes.SubModuleName)
			}

			m.keeper.SetMiddlewareEnabled(ctx, ch.PortId, ch.ConnectionHops[0])
			logger.Info("successfully migrated channel capability", "name", name)
		}
	}
	return nil
}
