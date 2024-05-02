package types

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// WasmConsensusHost implements the 02-client types.ConsensusHost interface.
type WasmConsensusHost struct {
	cdc      codec.BinaryCodec
	delegate clienttypes.ConsensusHost
}

var _ clienttypes.ConsensusHost = (*WasmConsensusHost)(nil)

// NewWasmConsensusHost creates and returns a new ConsensusHost for wasm wrapped consensus client state and consensus state self validation.
func NewWasmConsensusHost(cdc codec.BinaryCodec, delegate clienttypes.ConsensusHost) (*WasmConsensusHost, error) {
	if cdc == nil {
		return nil, fmt.Errorf("wasm consensus host codec is nil")
	}

	if delegate == nil {
		return nil, fmt.Errorf("wasm delegate consensus host is nil")
	}

	return &WasmConsensusHost{
		cdc:      cdc,
		delegate: delegate,
	}, nil
}

// GetSelfConsensusState implements the 02-client types.ConsensusHost interface.
func (w *WasmConsensusHost) GetSelfConsensusState(ctx sdk.Context, height exported.Height) (exported.ConsensusState, error) {
	consensusState, err := w.delegate.GetSelfConsensusState(ctx, height)
	if err != nil {
		return nil, err
	}

	// encode consensusState to wasm.ConsensusState.Data
	bz, err := w.cdc.MarshalInterface(consensusState)
	if err != nil {
		return nil, err
	}

	wasmConsensusState := &ConsensusState{
		Data: bz,
	}

	return wasmConsensusState, nil
}

// ValidateSelfClient implements the 02-client types.ConsensusHost interface.
func (w *WasmConsensusHost) ValidateSelfClient(ctx sdk.Context, clientState exported.ClientState) error {
	wasmClientState, ok := clientState.(*ClientState)
	if !ok {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "client must be a wasm client, expected: %T, got: %T", ClientState{}, wasmClientState)
	}

	if wasmClientState.Data == nil {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "wasm client state data is nil")
	}

	// unmarshal the wasmClientState bytes into the ClientState interface and call self validation
	var unwrappedClientState exported.ClientState
	if err := w.cdc.UnmarshalInterface(wasmClientState.Data, &unwrappedClientState); err != nil {
		return err
	}

	return w.delegate.ValidateSelfClient(ctx, unwrappedClientState)
}
