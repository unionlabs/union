package types

import (
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

type (
	checkForMisbehaviourInnerPayload struct {
		ClientMessage clientMessage `json:"client_message"`
	}
	checkForMisbehaviourPayload struct {
		CheckForMisbehaviour checkForMisbehaviourInnerPayload `json:"check_for_misbehaviour"`
	}
)

func (cs ClientState) CheckForMisbehaviour(ctx sdk.Context, _ codec.BinaryCodec, clientStore sdk.KVStore, msg exported.ClientMessage) bool {
	clientMsgConcrete := clientMessage{
		Header:       nil,
		Misbehaviour: nil,
	}
	switch clientMsg := msg.(type) {
	case *Header:
		clientMsgConcrete.Header = clientMsg
	case *Misbehaviour:
		clientMsgConcrete.Misbehaviour = clientMsg
	}

	if clientMsgConcrete.Header == nil && clientMsgConcrete.Misbehaviour == nil {
		return false
	}

	inner := checkForMisbehaviourInnerPayload{
		ClientMessage: clientMsgConcrete,
	}
	payload := checkForMisbehaviourPayload{
		CheckForMisbehaviour: inner,
	}

	result, err := call[contractResult](ctx, clientStore, &cs, payload)
	if err != nil {
		panic(err)
	}

	return result.FoundMisbehaviour
}
