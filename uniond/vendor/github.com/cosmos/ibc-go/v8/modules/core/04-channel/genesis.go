package channel

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// InitGenesis initializes the ibc channel submodule's state from a provided genesis
// state.
func InitGenesis(ctx sdk.Context, k keeper.Keeper, gs types.GenesisState) {
	if err := gs.Params.Validate(); err != nil {
		panic(fmt.Sprintf("invalid ibc channel genesis state parameters: %v", err))
	}
	k.SetParams(ctx, gs.Params)
	for _, channel := range gs.Channels {
		ch := types.NewChannel(channel.State, channel.Ordering, channel.Counterparty, channel.ConnectionHops, channel.Version)
		k.SetChannel(ctx, channel.PortId, channel.ChannelId, ch)
	}
	for _, ack := range gs.Acknowledgements {
		k.SetPacketAcknowledgement(ctx, ack.PortId, ack.ChannelId, ack.Sequence, ack.Data)
	}
	for _, commitment := range gs.Commitments {
		k.SetPacketCommitment(ctx, commitment.PortId, commitment.ChannelId, commitment.Sequence, commitment.Data)
	}
	for _, receipt := range gs.Receipts {
		k.SetPacketReceipt(ctx, receipt.PortId, receipt.ChannelId, receipt.Sequence)
	}
	for _, ss := range gs.SendSequences {
		k.SetNextSequenceSend(ctx, ss.PortId, ss.ChannelId, ss.Sequence)
	}
	for _, rs := range gs.RecvSequences {
		k.SetNextSequenceRecv(ctx, rs.PortId, rs.ChannelId, rs.Sequence)
	}
	for _, as := range gs.AckSequences {
		k.SetNextSequenceAck(ctx, as.PortId, as.ChannelId, as.Sequence)
	}
	k.SetNextChannelSequence(ctx, gs.NextChannelSequence)
}

// ExportGenesis returns the ibc channel submodule's exported genesis.
func ExportGenesis(ctx sdk.Context, k keeper.Keeper) types.GenesisState {
	return types.GenesisState{
		Channels:            k.GetAllChannels(ctx),
		Acknowledgements:    k.GetAllPacketAcks(ctx),
		Commitments:         k.GetAllPacketCommitments(ctx),
		Receipts:            k.GetAllPacketReceipts(ctx),
		SendSequences:       k.GetAllPacketSendSeqs(ctx),
		RecvSequences:       k.GetAllPacketRecvSeqs(ctx),
		AckSequences:        k.GetAllPacketAckSeqs(ctx),
		NextChannelSequence: k.GetNextChannelSequence(ctx),
		Params:              k.GetParams(ctx),
	}
}
