package keeper

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channelkeeper "github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"

	"union/x/differedack/types"
)

type (
	Keeper struct {
		cdc           codec.BinaryCodec
		storeKey      storetypes.StoreKey
		paramSpace    paramtypes.Subspace
		ics4Wrapper   porttypes.ICS4Wrapper
		channelKeeper channelkeeper.Keeper
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
	_, chanCap, err := k.channelKeeper.LookupModuleByChannel(ctx, differedPacketInfo.RefundPortId, differedPacketInfo.RefundChannelId)

	if err != nil {
		return errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	return k.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, channeltypes.Packet{
		Sequence:           differedPacketInfo.RefundSequence,
		SourcePort:         differedPacketInfo.PacketSrcPortId,
		SourceChannel:      differedPacketInfo.PacketSrcChannelId,
		DestinationPort:    differedPacketInfo.RefundPortId,
		DestinationChannel: differedPacketInfo.RefundChannelId,
		Data:               differedPacketInfo.PacketData,
		TimeoutHeight:      clienttypes.MustParseHeight(differedPacketInfo.PacketTimeoutHeight),
		TimeoutTimestamp:   differedPacketInfo.PacketTimeoutTimestamp,
	}, ack)
}

func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", types.ModuleName))
}
