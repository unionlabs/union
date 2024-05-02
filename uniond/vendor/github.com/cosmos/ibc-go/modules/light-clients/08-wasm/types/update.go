package types

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ClientState = (*ClientState)(nil)

// VerifyClientMessage must verify a ClientMessage. A ClientMessage could be a Header, Misbehaviour, or batch update.
// It must handle each type of ClientMessage appropriately. Calls to CheckForMisbehaviour, UpdateState, and UpdateStateOnMisbehaviour
// will assume that the content of the ClientMessage has been verified and can be trusted. An error should be returned
// if the ClientMessage fails to verify.
func (cs ClientState) VerifyClientMessage(ctx sdk.Context, _ codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) error {
	clientMessage, ok := clientMsg.(*ClientMessage)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected type: %T, got: %T", &ClientMessage{}, clientMsg)
	}

	payload := QueryMsg{
		VerifyClientMessage: &VerifyClientMessageMsg{ClientMessage: clientMessage.Data},
	}
	_, err := wasmQuery[EmptyResult](ctx, clientStore, &cs, payload)
	return err
}

// Client state and new consensus states are updated in the store by the contract
func (cs ClientState) UpdateState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) []exported.Height {
	clientMessage, ok := clientMsg.(*ClientMessage)
	if !ok {
		panic(fmt.Errorf("expected type %T, got %T", &ClientMessage{}, clientMsg))
	}

	payload := SudoMsg{
		UpdateState: &UpdateStateMsg{ClientMessage: clientMessage.Data},
	}

	result, err := wasmSudo[UpdateStateResult](ctx, cdc, clientStore, &cs, payload)
	if err != nil {
		panic(err)
	}

	heights := []exported.Height{}
	for _, height := range result.Heights {
		heights = append(heights, height)
	}

	return heights
}

// UpdateStateOnMisbehaviour should perform appropriate state changes on a client state given that misbehaviour has been detected and verified
// Client state is updated in the store by contract.
func (cs ClientState) UpdateStateOnMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) {
	clientMessage, ok := clientMsg.(*ClientMessage)
	if !ok {
		panic(fmt.Errorf("expected type %T, got %T", &ClientMessage{}, clientMsg))
	}

	payload := SudoMsg{
		UpdateStateOnMisbehaviour: &UpdateStateOnMisbehaviourMsg{ClientMessage: clientMessage.Data},
	}

	_, err := wasmSudo[EmptyResult](ctx, cdc, clientStore, &cs, payload)
	if err != nil {
		panic(err)
	}
}
