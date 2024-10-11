package keeper

import (
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// UnmarshalClientState attempts to decode and return an ClientState object from
// raw encoded bytes.
func (k Keeper) UnmarshalClientState(bz []byte) (exported.ClientState, error) {
	return types.UnmarshalClientState(k.cdc, bz)
}

// MustUnmarshalClientState attempts to decode and return an ClientState object from
// raw encoded bytes. It panics on error.
func (k Keeper) MustUnmarshalClientState(bz []byte) exported.ClientState {
	return types.MustUnmarshalClientState(k.cdc, bz)
}

// UnmarshalConsensusState attempts to decode and return an ConsensusState object from
// raw encoded bytes.
func (k Keeper) UnmarshalConsensusState(bz []byte) (exported.ConsensusState, error) {
	return types.UnmarshalConsensusState(k.cdc, bz)
}

// MustUnmarshalConsensusState attempts to decode and return an ConsensusState object from
// raw encoded bytes. It panics on error.
func (k Keeper) MustUnmarshalConsensusState(bz []byte) exported.ConsensusState {
	return types.MustUnmarshalConsensusState(k.cdc, bz)
}

// MustMarshalClientState attempts to encode an ClientState object and returns the
// raw encoded bytes. It panics on error.
func (k Keeper) MustMarshalClientState(clientState exported.ClientState) []byte {
	return types.MustMarshalClientState(k.cdc, clientState)
}

// MustMarshalConsensusState attempts to encode an ConsensusState object and returns the
// raw encoded bytes. It panics on error.
func (k Keeper) MustMarshalConsensusState(consensusState exported.ConsensusState) []byte {
	return types.MustMarshalConsensusState(k.cdc, consensusState)
}
