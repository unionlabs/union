package types

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/codec"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// MustUnmarshalClientState attempts to decode and return an ClientState object from
// raw encoded bytes. It panics on error.
func MustUnmarshalClientState(cdc codec.BinaryCodec, bz []byte) exported.ClientState {
	clientState, err := UnmarshalClientState(cdc, bz)
	if err != nil {
		panic(fmt.Errorf("failed to decode client state: %w", err))
	}

	return clientState
}

// MustMarshalClientState attempts to encode an ClientState object and returns the
// raw encoded bytes. It panics on error.
func MustMarshalClientState(cdc codec.BinaryCodec, clientState exported.ClientState) []byte {
	bz, err := MarshalClientState(cdc, clientState)
	if err != nil {
		panic(fmt.Errorf("failed to encode client state: %w", err))
	}

	return bz
}

// MarshalClientState protobuf serializes an ClientState interface
func MarshalClientState(cdc codec.BinaryCodec, clientStateI exported.ClientState) ([]byte, error) {
	return cdc.MarshalInterface(clientStateI)
}

// UnmarshalClientState returns an ClientState interface from raw encoded clientState
// bytes of a Proto-based ClientState type. An error is returned upon decoding
// failure.
func UnmarshalClientState(cdc codec.BinaryCodec, bz []byte) (exported.ClientState, error) {
	var clientState exported.ClientState
	if err := cdc.UnmarshalInterface(bz, &clientState); err != nil {
		return nil, err
	}

	return clientState, nil
}

// MustUnmarshalConsensusState attempts to decode and return an ConsensusState object from
// raw encoded bytes. It panics on error.
func MustUnmarshalConsensusState(cdc codec.BinaryCodec, bz []byte) exported.ConsensusState {
	consensusState, err := UnmarshalConsensusState(cdc, bz)
	if err != nil {
		panic(fmt.Errorf("failed to decode consensus state: %w", err))
	}

	return consensusState
}

// MustMarshalConsensusState attempts to encode a ConsensusState object and returns the
// raw encoded bytes. It panics on error.
func MustMarshalConsensusState(cdc codec.BinaryCodec, consensusState exported.ConsensusState) []byte {
	bz, err := MarshalConsensusState(cdc, consensusState)
	if err != nil {
		panic(fmt.Errorf("failed to encode consensus state: %w", err))
	}

	return bz
}

// MarshalConsensusState protobuf serializes a ConsensusState interface
func MarshalConsensusState(cdc codec.BinaryCodec, cs exported.ConsensusState) ([]byte, error) {
	return cdc.MarshalInterface(cs)
}

// UnmarshalConsensusState returns a ConsensusState interface from raw encoded consensus state
// bytes of a Proto-based ConsensusState type. An error is returned upon decoding
// failure.
func UnmarshalConsensusState(cdc codec.BinaryCodec, bz []byte) (exported.ConsensusState, error) {
	var consensusState exported.ConsensusState
	if err := cdc.UnmarshalInterface(bz, &consensusState); err != nil {
		return nil, err
	}

	return consensusState, nil
}

// MarshalClientMessage protobuf serializes a ClientMessage interface
func MarshalClientMessage(cdc codec.BinaryCodec, clientMessage exported.ClientMessage) ([]byte, error) {
	return cdc.MarshalInterface(clientMessage)
}

// MustMarshalClientMessage attempts to encode a ClientMessage object and returns the
// raw encoded bytes. It panics on error.
func MustMarshalClientMessage(cdc codec.BinaryCodec, clientMessage exported.ClientMessage) []byte {
	bz, err := MarshalClientMessage(cdc, clientMessage)
	if err != nil {
		panic(fmt.Errorf("failed to encode ClientMessage: %w", err))
	}

	return bz
}

// UnmarshalClientMessage returns a ClientMessage interface from raw proto encoded header bytes.
// An error is returned upon decoding failure.
func UnmarshalClientMessage(cdc codec.BinaryCodec, bz []byte) (exported.ClientMessage, error) {
	var clientMessage exported.ClientMessage
	if err := cdc.UnmarshalInterface(bz, &clientMessage); err != nil {
		return nil, err
	}

	return clientMessage, nil
}
