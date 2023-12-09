package types

import (
	"bytes"
	"encoding/hex"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// CheckSubstituteAndUpdateState will verify that a substitute client state is valid and update the subject client state.
// Note that this method is used only for recovery and will not allow changes to the checksum.
func (cs ClientState) CheckSubstituteAndUpdateState(ctx sdk.Context, cdc codec.BinaryCodec, subjectClientStore, substituteClientStore storetypes.KVStore, substituteClient exported.ClientState) error {
	substituteClientState, ok := substituteClient.(*ClientState)
	if !ok {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidClient,
			"invalid substitute client state: expected type %T, got %T", &ClientState{}, substituteClient,
		)
	}

	// check that checksums of subject client state and substitute client state match
	// changing the checksum is only allowed through the migrate contract RPC endpoint
	if !bytes.Equal(cs.Checksum, substituteClientState.Checksum) {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "expected checksums to be equal: expected %s, got %s", hex.EncodeToString(cs.Checksum), hex.EncodeToString(substituteClientState.Checksum))
	}

	store := newMigrateClientWrappedStore(subjectClientStore, substituteClientStore)

	payload := SudoMsg{
		MigrateClientStore: &MigrateClientStoreMsg{},
	}

	_, err := wasmSudo[EmptyResult](ctx, cdc, store, &cs, payload)
	return err
}
