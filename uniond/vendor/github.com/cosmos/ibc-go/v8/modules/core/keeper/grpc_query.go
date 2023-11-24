package keeper

import (
	"context"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// ClientState implements the IBC QueryServer interface
func (k Keeper) ClientState(c context.Context, req *clienttypes.QueryClientStateRequest) (*clienttypes.QueryClientStateResponse, error) {
	return k.ClientKeeper.ClientState(c, req)
}

// ClientStates implements the IBC QueryServer interface
func (k Keeper) ClientStates(c context.Context, req *clienttypes.QueryClientStatesRequest) (*clienttypes.QueryClientStatesResponse, error) {
	return k.ClientKeeper.ClientStates(c, req)
}

// ConsensusState implements the IBC QueryServer interface
func (k Keeper) ConsensusState(c context.Context, req *clienttypes.QueryConsensusStateRequest) (*clienttypes.QueryConsensusStateResponse, error) {
	return k.ClientKeeper.ConsensusState(c, req)
}

// ConsensusStates implements the IBC QueryServer interface
func (k Keeper) ConsensusStates(c context.Context, req *clienttypes.QueryConsensusStatesRequest) (*clienttypes.QueryConsensusStatesResponse, error) {
	return k.ClientKeeper.ConsensusStates(c, req)
}

// ConsensusStateHeights implements the IBC QueryServer interface
func (k Keeper) ConsensusStateHeights(c context.Context, req *clienttypes.QueryConsensusStateHeightsRequest) (*clienttypes.QueryConsensusStateHeightsResponse, error) {
	return k.ClientKeeper.ConsensusStateHeights(c, req)
}

// ClientStatus implements the IBC QueryServer interface
func (k Keeper) ClientStatus(c context.Context, req *clienttypes.QueryClientStatusRequest) (*clienttypes.QueryClientStatusResponse, error) {
	return k.ClientKeeper.ClientStatus(c, req)
}

// ClientParams implements the IBC QueryServer interface
func (k Keeper) ClientParams(c context.Context, req *clienttypes.QueryClientParamsRequest) (*clienttypes.QueryClientParamsResponse, error) {
	return k.ClientKeeper.ClientParams(c, req)
}

// UpgradedClientState implements the IBC QueryServer interface
func (k Keeper) UpgradedClientState(c context.Context, req *clienttypes.QueryUpgradedClientStateRequest) (*clienttypes.QueryUpgradedClientStateResponse, error) {
	return k.ClientKeeper.UpgradedClientState(c, req)
}

// UpgradedConsensusState implements the IBC QueryServer interface
func (k Keeper) UpgradedConsensusState(c context.Context, req *clienttypes.QueryUpgradedConsensusStateRequest) (*clienttypes.QueryUpgradedConsensusStateResponse, error) {
	return k.ClientKeeper.UpgradedConsensusState(c, req)
}

// Connection implements the IBC QueryServer interface
func (k Keeper) Connection(c context.Context, req *connectiontypes.QueryConnectionRequest) (*connectiontypes.QueryConnectionResponse, error) {
	return k.ConnectionKeeper.Connection(c, req)
}

// Connections implements the IBC QueryServer interface
func (k Keeper) Connections(c context.Context, req *connectiontypes.QueryConnectionsRequest) (*connectiontypes.QueryConnectionsResponse, error) {
	return k.ConnectionKeeper.Connections(c, req)
}

// ClientConnections implements the IBC QueryServer interface
func (k Keeper) ClientConnections(c context.Context, req *connectiontypes.QueryClientConnectionsRequest) (*connectiontypes.QueryClientConnectionsResponse, error) {
	return k.ConnectionKeeper.ClientConnections(c, req)
}

// ConnectionClientState implements the IBC QueryServer interface
func (k Keeper) ConnectionClientState(c context.Context, req *connectiontypes.QueryConnectionClientStateRequest) (*connectiontypes.QueryConnectionClientStateResponse, error) {
	return k.ConnectionKeeper.ConnectionClientState(c, req)
}

// ConnectionConsensusState implements the IBC QueryServer interface
func (k Keeper) ConnectionConsensusState(c context.Context, req *connectiontypes.QueryConnectionConsensusStateRequest) (*connectiontypes.QueryConnectionConsensusStateResponse, error) {
	return k.ConnectionKeeper.ConnectionConsensusState(c, req)
}

// ConnectionParams implements the IBC QueryServer interface
func (k Keeper) ConnectionParams(c context.Context, req *connectiontypes.QueryConnectionParamsRequest) (*connectiontypes.QueryConnectionParamsResponse, error) {
	return k.ConnectionKeeper.ConnectionParams(c, req)
}

// Channel implements the IBC QueryServer interface
func (k Keeper) Channel(c context.Context, req *channeltypes.QueryChannelRequest) (*channeltypes.QueryChannelResponse, error) {
	return k.ChannelKeeper.Channel(c, req)
}

// Channels implements the IBC QueryServer interface
func (k Keeper) Channels(c context.Context, req *channeltypes.QueryChannelsRequest) (*channeltypes.QueryChannelsResponse, error) {
	return k.ChannelKeeper.Channels(c, req)
}

// ConnectionChannels implements the IBC QueryServer interface
func (k Keeper) ConnectionChannels(c context.Context, req *channeltypes.QueryConnectionChannelsRequest) (*channeltypes.QueryConnectionChannelsResponse, error) {
	return k.ChannelKeeper.ConnectionChannels(c, req)
}

// ChannelClientState implements the IBC QueryServer interface
func (k Keeper) ChannelClientState(c context.Context, req *channeltypes.QueryChannelClientStateRequest) (*channeltypes.QueryChannelClientStateResponse, error) {
	return k.ChannelKeeper.ChannelClientState(c, req)
}

// ChannelConsensusState implements the IBC QueryServer interface
func (k Keeper) ChannelConsensusState(c context.Context, req *channeltypes.QueryChannelConsensusStateRequest) (*channeltypes.QueryChannelConsensusStateResponse, error) {
	return k.ChannelKeeper.ChannelConsensusState(c, req)
}

// PacketCommitment implements the IBC QueryServer interface
func (k Keeper) PacketCommitment(c context.Context, req *channeltypes.QueryPacketCommitmentRequest) (*channeltypes.QueryPacketCommitmentResponse, error) {
	return k.ChannelKeeper.PacketCommitment(c, req)
}

// PacketCommitments implements the IBC QueryServer interface
func (k Keeper) PacketCommitments(c context.Context, req *channeltypes.QueryPacketCommitmentsRequest) (*channeltypes.QueryPacketCommitmentsResponse, error) {
	return k.ChannelKeeper.PacketCommitments(c, req)
}

// PacketReceipt implements the IBC QueryServer interface
func (k Keeper) PacketReceipt(c context.Context, req *channeltypes.QueryPacketReceiptRequest) (*channeltypes.QueryPacketReceiptResponse, error) {
	return k.ChannelKeeper.PacketReceipt(c, req)
}

// PacketAcknowledgement implements the IBC QueryServer interface
func (k Keeper) PacketAcknowledgement(c context.Context, req *channeltypes.QueryPacketAcknowledgementRequest) (*channeltypes.QueryPacketAcknowledgementResponse, error) {
	return k.ChannelKeeper.PacketAcknowledgement(c, req)
}

// PacketAcknowledgements implements the IBC QueryServer interface
func (k Keeper) PacketAcknowledgements(c context.Context, req *channeltypes.QueryPacketAcknowledgementsRequest) (*channeltypes.QueryPacketAcknowledgementsResponse, error) {
	return k.ChannelKeeper.PacketAcknowledgements(c, req)
}

// UnreceivedPackets implements the IBC QueryServer interface
func (k Keeper) UnreceivedPackets(c context.Context, req *channeltypes.QueryUnreceivedPacketsRequest) (*channeltypes.QueryUnreceivedPacketsResponse, error) {
	return k.ChannelKeeper.UnreceivedPackets(c, req)
}

// UnreceivedAcks implements the IBC QueryServer interface
func (k Keeper) UnreceivedAcks(c context.Context, req *channeltypes.QueryUnreceivedAcksRequest) (*channeltypes.QueryUnreceivedAcksResponse, error) {
	return k.ChannelKeeper.UnreceivedAcks(c, req)
}

// NextSequenceReceive implements the IBC QueryServer interface
func (k Keeper) NextSequenceReceive(c context.Context, req *channeltypes.QueryNextSequenceReceiveRequest) (*channeltypes.QueryNextSequenceReceiveResponse, error) {
	return k.ChannelKeeper.NextSequenceReceive(c, req)
}

// NextSequenceSend implements the IBC QueryServer interface
func (k Keeper) NextSequenceSend(c context.Context, req *channeltypes.QueryNextSequenceSendRequest) (*channeltypes.QueryNextSequenceSendResponse, error) {
	return k.ChannelKeeper.NextSequenceSend(c, req)
}
