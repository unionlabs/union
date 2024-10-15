package types

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ gogoprotoany.UnpackInterfacesMessage = (*QueryClientStateResponse)(nil)
	_ gogoprotoany.UnpackInterfacesMessage = (*QueryClientStatesResponse)(nil)
	_ gogoprotoany.UnpackInterfacesMessage = (*QueryConsensusStateResponse)(nil)
	_ gogoprotoany.UnpackInterfacesMessage = (*QueryConsensusStatesResponse)(nil)
)

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (qcsr QueryClientStatesResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	for _, cs := range qcsr.ClientStates {
		if err := cs.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}
	return nil
}

// NewQueryClientStateResponse creates a new QueryClientStateResponse instance.
func NewQueryClientStateResponse(
	clientStateAny *codectypes.Any, proof []byte, height Height,
) *QueryClientStateResponse {
	return &QueryClientStateResponse{
		ClientState: clientStateAny,
		Proof:       proof,
		ProofHeight: height,
	}
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (qcsr QueryClientStateResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return unpacker.UnpackAny(qcsr.ClientState, new(exported.ClientState))
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (qcsr QueryConsensusStatesResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	for _, cs := range qcsr.ConsensusStates {
		if err := cs.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}
	return nil
}

// NewQueryConsensusStateResponse creates a new QueryConsensusStateResponse instance.
func NewQueryConsensusStateResponse(
	consensusStateAny *codectypes.Any, proof []byte, height Height,
) *QueryConsensusStateResponse {
	return &QueryConsensusStateResponse{
		ConsensusState: consensusStateAny,
		Proof:          proof,
		ProofHeight:    height,
	}
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (qcsr QueryConsensusStateResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return unpacker.UnpackAny(qcsr.ConsensusState, new(exported.ConsensusState))
}
