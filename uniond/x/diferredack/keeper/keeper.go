package keeper

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channelkeeper "github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"

	"union/x/diferredack/types"
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

func (k *Keeper) WriteDiferredAck(ctx sdk.Context, diferredPacketInfo *types.DiferredPacketInfo, ack channeltypes.Acknowledgement) error {
	_, chanCap, err := k.channelKeeper.LookupModuleByChannel(ctx, diferredPacketInfo.RefundPortId, diferredPacketInfo.RefundChannelId)

	if err != nil {
		return errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	return k.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, channeltypes.Packet{
		Sequence:           diferredPacketInfo.Sequence,
		SourcePort:         diferredPacketInfo.PacketSrcPortId,
		SourceChannel:      diferredPacketInfo.PacketSrcChannelId,
		DestinationPort:    diferredPacketInfo.RefundPortId,
		DestinationChannel: diferredPacketInfo.RefundChannelId,
		Data:               diferredPacketInfo.PacketData,
		TimeoutHeight:      clienttypes.MustParseHeight(diferredPacketInfo.PacketTimeoutHeight),
		TimeoutTimestamp:   diferredPacketInfo.PacketTimeoutTimestamp,
	}, ack)
}

func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", types.ModuleName))
}
