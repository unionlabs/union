package types

import (
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ConsensusState = (*ConsensusState)(nil)

// NewConsensusState creates a new ConsensusState instance.
func NewConsensusState(data []byte) *ConsensusState {
	return &ConsensusState{
		Data: data,
	}
}

// ClientType returns Wasm type.
func (ConsensusState) ClientType() string {
	return exported.Wasm
}

// GetTimestamp returns block time in nanoseconds of the header that created consensus state.
func (ConsensusState) GetTimestamp() uint64 {
	return 0
}

// ValidateBasic defines a basic validation for the wasm client consensus state.
func (cs ConsensusState) ValidateBasic() error {
	if len(cs.Data) == 0 {
		return errorsmod.Wrap(ErrInvalidData, "data cannot be empty")
	}

	return nil
}
