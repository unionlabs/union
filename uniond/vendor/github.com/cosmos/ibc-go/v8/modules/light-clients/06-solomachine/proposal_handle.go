package solomachine

import (
	"reflect"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// CheckSubstituteAndUpdateState verifies that the subject is allowed to be updated by
// a governance proposal and that the substitute client is a solo machine.
// It will update the consensus state to the substitute's consensus state and
// the sequence to the substitute's current sequence. An error is returned if
// the client has been disallowed to be updated by a governance proposal,
// the substitute is not a solo machine, or the current public key equals
// the new public key.
func (cs ClientState) CheckSubstituteAndUpdateState(
	ctx sdk.Context, cdc codec.BinaryCodec, subjectClientStore,
	_ storetypes.KVStore, substituteClient exported.ClientState,
) error {
	substituteClientState, ok := substituteClient.(*ClientState)
	if !ok {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClientType, "substitute client state type %T, expected  %T", substituteClient, &ClientState{})
	}

	subjectPublicKey, err := cs.ConsensusState.GetPubKey()
	if err != nil {
		return errorsmod.Wrap(err, "failed to get consensus public key")
	}

	substitutePublicKey, err := substituteClientState.ConsensusState.GetPubKey()
	if err != nil {
		return errorsmod.Wrap(err, "failed to get substitute client public key")
	}

	if reflect.DeepEqual(subjectPublicKey, substitutePublicKey) {
		return errorsmod.Wrapf(clienttypes.ErrInvalidHeader, "subject and substitute have the same public key")
	}

	// update to substitute parameters
	cs.Sequence = substituteClientState.Sequence
	cs.ConsensusState = substituteClientState.ConsensusState
	cs.IsFrozen = false

	setClientState(subjectClientStore, cdc, &cs)

	return nil
}
