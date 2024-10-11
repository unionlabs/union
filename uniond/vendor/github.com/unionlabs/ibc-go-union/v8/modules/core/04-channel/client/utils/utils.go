package utils

import (
	"context"
	"encoding/binary"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"

	clientutils "github.com/cosmos/ibc-go/v8/modules/core/02-client/client/utils"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/client"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// QueryChannel returns a channel end.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryChannel(
	clientCtx client.Context, portID, channelID string, prove bool,
) (*types.QueryChannelResponse, error) {
	if prove {
		return queryChannelABCI(clientCtx, portID, channelID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryChannelRequest{
		PortId:    portID,
		ChannelId: channelID,
	}

	return queryClient.Channel(context.Background(), req)
}

func queryChannelABCI(clientCtx client.Context, portID, channelID string) (*types.QueryChannelResponse, error) {
	key := host.ChannelKey(portID, channelID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if channel exists
	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrChannelNotFound, "portID (%s), channelID (%s)", portID, channelID)
	}

	cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

	var channel types.Channel
	if err := cdc.Unmarshal(value, &channel); err != nil {
		return nil, err
	}

	return types.NewQueryChannelResponse(channel, proofBz, proofHeight), nil
}

// QueryChannelClientState returns the ClientState of a channel end. If
// prove is true, it performs an ABCI store query in order to retrieve the
// merkle proof. Otherwise, it uses the gRPC query client.
func QueryChannelClientState(
	clientCtx client.Context, portID, channelID string, prove bool,
) (*types.QueryChannelClientStateResponse, error) {
	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryChannelClientStateRequest{
		PortId:    portID,
		ChannelId: channelID,
	}

	res, err := queryClient.ChannelClientState(context.Background(), req)
	if err != nil {
		return nil, err
	}

	if prove {
		clientStateRes, err := clientutils.QueryClientStateABCI(clientCtx, res.IdentifiedClientState.ClientId)
		if err != nil {
			return nil, err
		}

		// use client state returned from ABCI query in case query height differs
		identifiedClientState := clienttypes.IdentifiedClientState{
			ClientId:    res.IdentifiedClientState.ClientId,
			ClientState: clientStateRes.ClientState,
		}
		res = types.NewQueryChannelClientStateResponse(identifiedClientState, clientStateRes.Proof, clientStateRes.ProofHeight)
	}

	return res, nil
}

// QueryChannelConsensusState returns the ConsensusState of a channel end. If
// prove is true, it performs an ABCI store query in order to retrieve the
// merkle proof. Otherwise, it uses the gRPC query client.
func QueryChannelConsensusState(
	clientCtx client.Context, portID, channelID string, height clienttypes.Height, prove bool,
) (*types.QueryChannelConsensusStateResponse, error) {
	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryChannelConsensusStateRequest{
		PortId:         portID,
		ChannelId:      channelID,
		RevisionNumber: height.RevisionNumber,
		RevisionHeight: height.RevisionHeight,
	}

	res, err := queryClient.ChannelConsensusState(context.Background(), req)
	if err != nil {
		return nil, err
	}

	if prove {
		consensusStateRes, err := clientutils.QueryConsensusStateABCI(clientCtx, res.ClientId, height)
		if err != nil {
			return nil, err
		}

		res = types.NewQueryChannelConsensusStateResponse(res.ClientId, consensusStateRes.ConsensusState, height, consensusStateRes.Proof, consensusStateRes.ProofHeight)
	}

	return res, nil
}

// QueryLatestConsensusState uses the channel Querier to return the
// latest ConsensusState given the source port ID and source channel ID.
func QueryLatestConsensusState(
	clientCtx client.Context, portID, channelID string,
) (exported.ConsensusState, clienttypes.Height, clienttypes.Height, error) {
	clientRes, err := QueryChannelClientState(clientCtx, portID, channelID, false)
	if err != nil {
		return nil, clienttypes.Height{}, clienttypes.Height{}, err
	}

	var clientState exported.ClientState
	if err := clientCtx.InterfaceRegistry.UnpackAny(clientRes.IdentifiedClientState.ClientState, &clientState); err != nil {
		return nil, clienttypes.Height{}, clienttypes.Height{}, err
	}

	clientHeight, ok := clientState.GetLatestHeight().(clienttypes.Height)
	if !ok {
		return nil, clienttypes.Height{}, clienttypes.Height{}, errorsmod.Wrapf(ibcerrors.ErrInvalidHeight, "invalid height type. expected type: %T, got: %T",
			clienttypes.Height{}, clientHeight)
	}
	res, err := QueryChannelConsensusState(clientCtx, portID, channelID, clientHeight, false)
	if err != nil {
		return nil, clienttypes.Height{}, clienttypes.Height{}, err
	}

	var consensusState exported.ConsensusState
	if err := clientCtx.InterfaceRegistry.UnpackAny(res.ConsensusState, &consensusState); err != nil {
		return nil, clienttypes.Height{}, clienttypes.Height{}, err
	}

	return consensusState, clientHeight, res.ProofHeight, nil
}

// QueryNextSequenceReceive returns the next sequence receive.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryNextSequenceReceive(
	clientCtx client.Context, portID, channelID string, prove bool,
) (*types.QueryNextSequenceReceiveResponse, error) {
	if prove {
		return queryNextSequenceRecvABCI(clientCtx, portID, channelID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryNextSequenceReceiveRequest{
		PortId:    portID,
		ChannelId: channelID,
	}

	return queryClient.NextSequenceReceive(context.Background(), req)
}

func queryNextSequenceRecvABCI(clientCtx client.Context, portID, channelID string) (*types.QueryNextSequenceReceiveResponse, error) {
	key := host.NextSequenceRecvKey(portID, channelID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if next sequence receive exists
	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrChannelNotFound, "portID (%s), channelID (%s)", portID, channelID)
	}

	sequence := binary.BigEndian.Uint64(value)

	return types.NewQueryNextSequenceReceiveResponse(sequence, proofBz, proofHeight), nil
}

// QueryNextSequenceSend returns the next sequence send.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryNextSequenceSend(
	clientCtx client.Context, portID, channelID string, prove bool,
) (*types.QueryNextSequenceSendResponse, error) {
	if prove {
		return queryNextSequenceSendABCI(clientCtx, portID, channelID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryNextSequenceSendRequest{
		PortId:    portID,
		ChannelId: channelID,
	}
	return queryClient.NextSequenceSend(context.Background(), req)
}

func queryNextSequenceSendABCI(clientCtx client.Context, portID, channelID string) (*types.QueryNextSequenceSendResponse, error) {
	key := host.NextSequenceSendKey(portID, channelID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if next sequence send exists
	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrChannelNotFound, "portID (%s), channelID (%s)", portID, channelID)
	}

	sequence := binary.BigEndian.Uint64(value)

	return types.NewQueryNextSequenceSendResponse(sequence, proofBz, proofHeight), nil
}

// QueryUpgradeError returns the upgrade error.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryUpgradeError(
	clientCtx client.Context, portID, channelID string, prove bool,
) (*types.QueryUpgradeErrorResponse, error) {
	if prove {
		return queryUpgradeErrorABCI(clientCtx, portID, channelID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryUpgradeErrorRequest{
		PortId:    portID,
		ChannelId: channelID,
	}

	return queryClient.UpgradeError(context.Background(), req)
}

// QueryUpgrade returns the upgrade.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryUpgrade(
	clientCtx client.Context, portID, channelID string, prove bool,
) (*types.QueryUpgradeResponse, error) {
	if prove {
		return queryUpgradeABCI(clientCtx, portID, channelID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryUpgradeRequest{
		PortId:    portID,
		ChannelId: channelID,
	}

	return queryClient.Upgrade(context.Background(), req)
}

// queryUpgradeErrorABCI queries the upgrade error from the store.
func queryUpgradeErrorABCI(clientCtx client.Context, portID, channelID string) (*types.QueryUpgradeErrorResponse, error) {
	key := host.ChannelUpgradeErrorKey(portID, channelID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if upgrade error exists
	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrUpgradeErrorNotFound, "portID (%s), channelID (%s)", portID, channelID)
	}

	cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

	var receipt types.ErrorReceipt
	if err := cdc.Unmarshal(value, &receipt); err != nil {
		return nil, err
	}

	return types.NewQueryUpgradeErrorResponse(receipt, proofBz, proofHeight), nil
}

// queryUpgradeABCI queries the upgrade from the store.
func queryUpgradeABCI(clientCtx client.Context, portID, channelID string) (*types.QueryUpgradeResponse, error) {
	key := host.ChannelUpgradeKey(portID, channelID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if upgrade exists
	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrUpgradeErrorNotFound, "portID (%s), channelID (%s)", portID, channelID)
	}

	cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

	var upgrade types.Upgrade
	if err := cdc.Unmarshal(value, &upgrade); err != nil {
		return nil, err
	}

	return types.NewQueryUpgradeResponse(upgrade, proofBz, proofHeight), nil
}

// QueryPacketCommitment returns a packet commitment.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryPacketCommitment(
	clientCtx client.Context, portID, channelID string,
	sequence uint64, prove bool,
) (*types.QueryPacketCommitmentResponse, error) {
	if prove {
		return queryPacketCommitmentABCI(clientCtx, portID, channelID, sequence)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryPacketCommitmentRequest{
		PortId:    portID,
		ChannelId: channelID,
		Sequence:  sequence,
	}

	return queryClient.PacketCommitment(context.Background(), req)
}

func queryPacketCommitmentABCI(
	clientCtx client.Context, portID, channelID string, sequence uint64,
) (*types.QueryPacketCommitmentResponse, error) {
	key := host.PacketCommitmentKey(portID, channelID, sequence)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if packet commitment exists
	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrPacketCommitmentNotFound, "portID (%s), channelID (%s), sequence (%d)", portID, channelID, sequence)
	}

	return types.NewQueryPacketCommitmentResponse(value, proofBz, proofHeight), nil
}

// QueryPacketReceipt returns data about a packet receipt.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryPacketReceipt(
	clientCtx client.Context, portID, channelID string,
	sequence uint64, prove bool,
) (*types.QueryPacketReceiptResponse, error) {
	if prove {
		return queryPacketReceiptABCI(clientCtx, portID, channelID, sequence)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryPacketReceiptRequest{
		PortId:    portID,
		ChannelId: channelID,
		Sequence:  sequence,
	}

	return queryClient.PacketReceipt(context.Background(), req)
}

func queryPacketReceiptABCI(
	clientCtx client.Context, portID, channelID string, sequence uint64,
) (*types.QueryPacketReceiptResponse, error) {
	key := host.PacketReceiptKey(portID, channelID, sequence)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	return types.NewQueryPacketReceiptResponse(value != nil, proofBz, proofHeight), nil
}

// QueryPacketAcknowledgement returns the data about a packet acknowledgement.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client
func QueryPacketAcknowledgement(clientCtx client.Context, portID, channelID string, sequence uint64, prove bool) (*types.QueryPacketAcknowledgementResponse, error) {
	if prove {
		return queryPacketAcknowledgementABCI(clientCtx, portID, channelID, sequence)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryPacketAcknowledgementRequest{
		PortId:    portID,
		ChannelId: channelID,
		Sequence:  sequence,
	}

	return queryClient.PacketAcknowledgement(context.Background(), req)
}

func queryPacketAcknowledgementABCI(clientCtx client.Context, portID, channelID string, sequence uint64) (*types.QueryPacketAcknowledgementResponse, error) {
	key := host.PacketAcknowledgementKey(portID, channelID, sequence)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	if len(value) == 0 {
		return nil, errorsmod.Wrapf(types.ErrInvalidAcknowledgement, "portID (%s), channelID (%s), sequence (%d)", portID, channelID, sequence)
	}

	return types.NewQueryPacketAcknowledgementResponse(value, proofBz, proofHeight), nil
}
