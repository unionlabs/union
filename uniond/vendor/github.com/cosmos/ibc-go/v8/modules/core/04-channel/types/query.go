package types

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ codectypes.UnpackInterfacesMessage = (*QueryChannelClientStateResponse)(nil)
	_ codectypes.UnpackInterfacesMessage = (*QueryChannelConsensusStateResponse)(nil)
)

// NewQueryChannelResponse creates a new QueryChannelResponse instance
func NewQueryChannelResponse(channel Channel, proof []byte, height clienttypes.Height) *QueryChannelResponse {
	return &QueryChannelResponse{
		Channel:     &channel,
		Proof:       proof,
		ProofHeight: height,
	}
}

// NewQueryChannelClientStateResponse creates a newQueryChannelClientStateResponse instance
func NewQueryChannelClientStateResponse(identifiedClientState clienttypes.IdentifiedClientState, proof []byte, height clienttypes.Height) *QueryChannelClientStateResponse {
	return &QueryChannelClientStateResponse{
		IdentifiedClientState: &identifiedClientState,
		Proof:                 proof,
		ProofHeight:           height,
	}
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (qccsr QueryChannelClientStateResponse) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return qccsr.IdentifiedClientState.UnpackInterfaces(unpacker)
}

// NewQueryChannelConsensusStateResponse creates a newQueryChannelConsensusStateResponse instance
func NewQueryChannelConsensusStateResponse(clientID string, anyConsensusState *codectypes.Any, consensusStateHeight exported.Height, proof []byte, height clienttypes.Height) *QueryChannelConsensusStateResponse {
	return &QueryChannelConsensusStateResponse{
		ConsensusState: anyConsensusState,
		ClientId:       clientID,
		Proof:          proof,
		ProofHeight:    height,
	}
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (qccsr QueryChannelConsensusStateResponse) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(qccsr.ConsensusState, new(exported.ConsensusState))
}

// NewQueryPacketCommitmentResponse creates a new QueryPacketCommitmentResponse instance
func NewQueryPacketCommitmentResponse(
	commitment []byte, proof []byte, height clienttypes.Height,
) *QueryPacketCommitmentResponse {
	return &QueryPacketCommitmentResponse{
		Commitment:  commitment,
		Proof:       proof,
		ProofHeight: height,
	}
}

// NewQueryPacketReceiptResponse creates a new QueryPacketReceiptResponse instance
func NewQueryPacketReceiptResponse(
	recvd bool, proof []byte, height clienttypes.Height,
) *QueryPacketReceiptResponse {
	return &QueryPacketReceiptResponse{
		Received:    recvd,
		Proof:       proof,
		ProofHeight: height,
	}
}

// NewQueryPacketAcknowledgementResponse creates a new QueryPacketAcknowledgementResponse instance
func NewQueryPacketAcknowledgementResponse(
	acknowledgement []byte, proof []byte, height clienttypes.Height,
) *QueryPacketAcknowledgementResponse {
	return &QueryPacketAcknowledgementResponse{
		Acknowledgement: acknowledgement,
		Proof:           proof,
		ProofHeight:     height,
	}
}

// NewQueryNextSequenceReceiveResponse creates a new QueryNextSequenceReceiveResponse instance
func NewQueryNextSequenceReceiveResponse(
	sequence uint64, proof []byte, height clienttypes.Height,
) *QueryNextSequenceReceiveResponse {
	return &QueryNextSequenceReceiveResponse{
		NextSequenceReceive: sequence,
		Proof:               proof,
		ProofHeight:         height,
	}
}

// NewQueryNextSequenceSendResponse creates a new QueryNextSequenceSendResponse instance
func NewQueryNextSequenceSendResponse(
	sequence uint64, proof []byte, height clienttypes.Height,
) *QueryNextSequenceSendResponse {
	return &QueryNextSequenceSendResponse{
		NextSequenceSend: sequence,
		Proof:            proof,
		ProofHeight:      height,
	}
}

// NewQueryUpgradeErrorResponse creates a new QueryUpgradeErrorResponse instance
func NewQueryUpgradeErrorResponse(errorReceipt ErrorReceipt, proof []byte, height clienttypes.Height) *QueryUpgradeErrorResponse {
	return &QueryUpgradeErrorResponse{
		ErrorReceipt: errorReceipt,
		Proof:        proof,
		ProofHeight:  height,
	}
}

// NewQueryUpgradeResponse creates a new QueryUpgradeResponse instance
func NewQueryUpgradeResponse(upgrade Upgrade, proof []byte, height clienttypes.Height) *QueryUpgradeResponse {
	return &QueryUpgradeResponse{
		Upgrade:     upgrade,
		Proof:       proof,
		ProofHeight: height,
	}
}
