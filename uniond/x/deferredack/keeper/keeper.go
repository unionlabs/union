package keeper

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channelkeeper "github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"

	"union/x/deferredack/types"
)

type (
	Keeper struct {
		cdc           codec.BinaryCodec
		storeKey      storetypes.StoreKey
		ics4Wrapper   porttypes.ICS4Wrapper
		channelKeeper channelkeeper.Keeper
	}
)

func NewKeeper(
	cdc codec.BinaryCodec,
	storeKey storetypes.StoreKey,
	ics4Wrapper porttypes.ICS4Wrapper,
	channelKeeper channelkeeper.Keeper,
) Keeper {
	return Keeper{
		cdc:           cdc,
		storeKey:      storeKey,
		ics4Wrapper:   ics4Wrapper,
		channelKeeper: channelKeeper,
	}
}

func (k *Keeper) WriteDeferredAck(ctx sdk.Context, deferredPacketInfo *types.DeferredPacketInfo, ack types.Acknowledgement) error {
	_, chanCap, err := k.channelKeeper.LookupModuleByChannel(ctx, deferredPacketInfo.RefundPortId, deferredPacketInfo.RefundChannelId)

	if err != nil {
		return errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	return k.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, channeltypes.Packet{
		Sequence:           deferredPacketInfo.Sequence,
		SourcePort:         deferredPacketInfo.PacketSrcPortId,
		SourceChannel:      deferredPacketInfo.PacketSrcChannelId,
		DestinationPort:    deferredPacketInfo.RefundPortId,
		DestinationChannel: deferredPacketInfo.RefundChannelId,
		Data:               deferredPacketInfo.PacketData,
		TimeoutHeight:      clienttypes.MustParseHeight(deferredPacketInfo.PacketTimeoutHeight),
		TimeoutTimestamp:   deferredPacketInfo.PacketTimeoutTimestamp,
	}, ack)
}

func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", types.ModuleName))
}
