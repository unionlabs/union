package types

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"

	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
)

type (
	checkSubstituteAndUpdateStateInnerPayload struct{}
	checkSubstituteAndUpdateStatePayload      struct {
		CheckSubstituteAndUpdateState checkSubstituteAndUpdateStateInnerPayload `json:"check_substitute_and_update_state"`
	}
)

// CheckSubstituteAndUpdateState will try to update the client with the state of the
// substitute.
//
// The following must always be true:
//   - The substitute client is the same type as the subject client
//   - The subject and substitute client states match in all parameters (expect frozen height, latest height, and chain-id)
//
// In case 1) before updating the client, the client will be unfrozen by resetting
// the FrozenHeight to the zero Height.
func (cs ClientState) CheckSubstituteAndUpdateState(
	ctx sdk.Context,
	_ codec.BinaryCodec,
	subjectClientStore, substituteClientStore sdk.KVStore,
	substituteClient exported.ClientState,
) error {
	var (
		SubjectPrefix    = []byte("subject/")
		SubstitutePrefix = []byte("substitute/")
	)

	_, ok := substituteClient.(*ClientState)
	if !ok {
		return sdkerrors.Wrapf(
			clienttypes.ErrInvalidClient,
			fmt.Sprintf("invalid substitute client state. expected type %T, got %T", &ClientState{}, substituteClient),
		)
	}

	store := newWrappedStore(subjectClientStore, substituteClientStore, SubjectPrefix, SubstitutePrefix)

	payload := checkSubstituteAndUpdateStatePayload{
		CheckSubstituteAndUpdateState: checkSubstituteAndUpdateStateInnerPayload{},
	}

	_, err := call[contractResult](ctx, store, &cs, payload)
	return err
}
