package keeper

import (
	"context"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// RecvPacketReCheckTx applies replay protection ensuring that when relay messages are
// re-executed in ReCheckTx, we can appropriately filter out redundant relay transactions.
func (k *Keeper) RecvPacketReCheckTx(ctx context.Context, packet types.Packet) error {
	channel, found := k.GetChannel(ctx, packet.GetDestPort(), packet.GetDestChannel())
	if !found {
		return errorsmod.Wrap(types.ErrChannelNotFound, packet.GetDestChannel())
	}

	if err := k.applyReplayProtection(ctx, packet, channel); err != nil {
		return err
	}

	return nil
}
