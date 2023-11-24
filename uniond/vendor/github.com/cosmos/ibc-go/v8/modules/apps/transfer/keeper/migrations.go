package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{
		keeper: keeper,
	}
}

// MigrateParams migrates the transfer module's parameters from the x/params to self store.
func (m Migrator) MigrateParams(ctx sdk.Context) error {
	var params types.Params
	m.keeper.legacySubspace.GetParamSet(ctx, &params)

	m.keeper.SetParams(ctx, params)
	m.keeper.Logger(ctx).Info("successfully migrated transfer app self-manage params")
	return nil
}

// MigrateTraces migrates the DenomTraces to the correct format, accounting for slashes in the BaseDenom.
func (m Migrator) MigrateTraces(ctx sdk.Context) error {
	// list of traces that must replace the old traces in store
	var newTraces []types.DenomTrace
	m.keeper.IterateDenomTraces(ctx,
		func(dt types.DenomTrace) (stop bool) {
			// check if the new way of splitting FullDenom
			// is the same as the current DenomTrace.
			// If it isn't then store the new DenomTrace in the list of new traces.
			newTrace := types.ParseDenomTrace(dt.GetFullDenomPath())
			err := newTrace.Validate()
			if err != nil {
				panic(err)
			}

			if dt.IBCDenom() != newTrace.IBCDenom() {
				// The new form of parsing will result in a token denomination change.
				// A bank migration is required. A panic should occur to prevent the
				// chain from using corrupted state.
				panic(fmt.Errorf("migration will result in corrupted state. Previous IBC token (%s) requires a bank migration. Expected denom trace (%s)", dt, newTrace))
			}

			if !equalTraces(newTrace, dt) {
				newTraces = append(newTraces, newTrace)
			}

			return false
		})

	// replace the outdated traces with the new trace information
	for _, nt := range newTraces {
		m.keeper.SetDenomTrace(ctx, nt)
	}
	return nil
}

// MigrateDenomMetadata sets token metadata for all the IBC denom traces
func (m Migrator) MigrateDenomMetadata(ctx sdk.Context) error {
	m.keeper.IterateDenomTraces(ctx,
		func(dt types.DenomTrace) (stop bool) {
			// check if the metadata for the given denom trace does not already exist
			if !m.keeper.bankKeeper.HasDenomMetaData(ctx, dt.IBCDenom()) {
				m.keeper.setDenomMetadata(ctx, dt)
			}
			return false
		})

	m.keeper.Logger(ctx).Info("successfully added metadata to IBC voucher denominations")
	return nil
}

// MigrateTotalEscrowForDenom migrates the total amount of source chain tokens in escrow.
func (m Migrator) MigrateTotalEscrowForDenom(ctx sdk.Context) error {
	var totalEscrowed sdk.Coins
	portID := m.keeper.GetPort(ctx)

	transferChannels := m.keeper.channelKeeper.GetAllChannelsWithPortPrefix(ctx, portID)
	for _, channel := range transferChannels {
		escrowAddress := types.GetEscrowAddress(portID, channel.ChannelId)
		escrowBalances := m.keeper.bankKeeper.GetAllBalances(ctx, escrowAddress)

		totalEscrowed = totalEscrowed.Add(escrowBalances...)
	}

	for _, totalEscrow := range totalEscrowed {
		m.keeper.SetTotalEscrowForDenom(ctx, totalEscrow)
	}

	m.keeper.Logger(ctx).Info("successfully set total escrow for %d denominations", totalEscrowed.Len())
	return nil
}

func equalTraces(dtA, dtB types.DenomTrace) bool {
	return dtA.BaseDenom == dtB.BaseDenom && dtA.Path == dtB.Path
}
