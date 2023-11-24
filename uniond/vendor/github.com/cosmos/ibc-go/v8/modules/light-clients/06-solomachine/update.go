package solomachine

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// VerifyClientMessage introspects the provided ClientMessage and checks its validity
// A Solomachine Header is considered valid if the currently registered public key has signed over the new public key with the correct sequence
// A Solomachine Misbehaviour is considered valid if duplicate signatures of the current public key are found on two different messages at a given sequence
func (cs ClientState) VerifyClientMessage(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) error {
	switch msg := clientMsg.(type) {
	case *Header:
		return cs.verifyHeader(cdc, msg)
	case *Misbehaviour:
		return cs.verifyMisbehaviour(cdc, msg)
	default:
		return errorsmod.Wrapf(clienttypes.ErrInvalidClientType, "expected type of %T or %T, got type %T", Header{}, Misbehaviour{}, msg)
	}
}

func (cs ClientState) verifyHeader(cdc codec.BinaryCodec, header *Header) error {
	// assert update timestamp is not less than current consensus state timestamp
	if header.Timestamp < cs.ConsensusState.Timestamp {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"header timestamp is less than to the consensus state timestamp (%d < %d)", header.Timestamp, cs.ConsensusState.Timestamp,
		)
	}

	// assert currently registered public key signed over the new public key with correct sequence
	headerData := &HeaderData{
		NewPubKey:      header.NewPublicKey,
		NewDiversifier: header.NewDiversifier,
	}

	dataBz, err := cdc.Marshal(headerData)
	if err != nil {
		return err
	}

	signBytes := &SignBytes{
		Sequence:    cs.Sequence,
		Timestamp:   header.Timestamp,
		Diversifier: cs.ConsensusState.Diversifier,
		Path:        []byte(SentinelHeaderPath),
		Data:        dataBz,
	}

	data, err := cdc.Marshal(signBytes)
	if err != nil {
		return err
	}

	sigData, err := UnmarshalSignatureData(cdc, header.Signature)
	if err != nil {
		return err
	}

	publicKey, err := cs.ConsensusState.GetPubKey()
	if err != nil {
		return err
	}

	if err := VerifySignature(publicKey, data, sigData); err != nil {
		return errorsmod.Wrap(ErrInvalidHeader, err.Error())
	}

	return nil
}

// UpdateState updates the consensus state to the new public key and an incremented sequence.
// A list containing the updated consensus height is returned.
func (cs ClientState) UpdateState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) []exported.Height {
	smHeader, ok := clientMsg.(*Header)
	if !ok {
		panic(fmt.Errorf("unsupported ClientMessage: %T", clientMsg))
	}

	// create new solomachine ConsensusState
	consensusState := &ConsensusState{
		PublicKey:   smHeader.NewPublicKey,
		Diversifier: smHeader.NewDiversifier,
		Timestamp:   smHeader.Timestamp,
	}

	cs.Sequence++
	cs.ConsensusState = consensusState

	setClientState(clientStore, cdc, &cs)

	return []exported.Height{clienttypes.NewHeight(0, cs.Sequence)}
}

// UpdateStateOnMisbehaviour updates state upon misbehaviour. This method should only be called on misbehaviour
// as it does not perform any misbehaviour checks.
func (cs ClientState) UpdateStateOnMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, _ exported.ClientMessage) {
	cs.IsFrozen = true

	setClientState(clientStore, cdc, &cs)
}
