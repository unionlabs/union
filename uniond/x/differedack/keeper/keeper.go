package keeper

import (
	"fmt"

	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"

	"union/x/differedack/types"
)

type (
	Keeper struct {
		cdc         codec.BinaryCodec
		storeKey    storetypes.StoreKey
		ics4Wrapper porttypes.ICS4Wrapper
	}
)

func NewKeeper(
	cdc codec.BinaryCodec,
	storeKey storetypes.StoreKey,
	ics4Wrapper porttypes.ICS4Wrapper,
) Keeper {
	return Keeper{
		cdc:         cdc,
		storeKey:    storeKey,
		ics4Wrapper: ics4Wrapper,
	}
}

func (k *Keeper) WriteDifferedAck(ctx sdk.Context, packet channeltypes.Packet, data transfertypes.FungibleTokenPacketData, differedPacketInfo *types.DifferedPacketInfo, ack channeltypes.Acknowledgement) error {
	return nil
}

func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", types.ModuleName))
}
