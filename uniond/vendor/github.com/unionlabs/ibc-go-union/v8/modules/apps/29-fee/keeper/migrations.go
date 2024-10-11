package keeper

import (
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
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

// Migrate1to2 migrates ibc-fee module from ConsensusVersion 1 to 2
// by refunding leftover fees to the refund address.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	store := m.keeper.storeService.OpenKVStore(ctx)
	iterator := storetypes.KVStorePrefixIterator(runtime.KVStoreAdapter(store), []byte(types.FeesInEscrowPrefix))
	defer coretypes.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	for ; iterator.Valid(); iterator.Next() {
		feesInEscrow := m.keeper.MustUnmarshalFees(iterator.Value())

		for _, packetFee := range feesInEscrow.PacketFees {
			refundCoins := legacyTotal(packetFee.Fee).Sub(packetFee.Fee.Total()...)

			refundAddr, err := sdk.AccAddressFromBech32(packetFee.RefundAddress)
			if err != nil {
				return err
			}

			m.keeper.distributeFee(ctx, refundAddr, refundAddr, refundCoins)
		}
	}

	return nil
}

// legacyTotal returns the legacy total amount for a given Fee
// The total amount is the RecvFee + AckFee + TimeoutFee
func legacyTotal(f types.Fee) sdk.Coins {
	return f.RecvFee.Add(f.AckFee...).Add(f.TimeoutFee...)
}
