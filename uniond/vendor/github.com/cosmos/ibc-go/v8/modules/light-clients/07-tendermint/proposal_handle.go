package tendermint

import (
	"reflect"
	"time"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// CheckSubstituteAndUpdateState will try to update the client with the state of the
// substitute.
//
// AllowUpdateAfterMisbehaviour and AllowUpdateAfterExpiry have been deprecated.
// Please see ADR 026 for more information.
//
// The following must always be true:
//   - The substitute client is the same type as the subject client
//   - The subject and substitute client states match in all parameters (expect frozen height, latest height, and chain-id)
//
// In case 1) before updating the client, the client will be unfrozen by resetting
// the FrozenHeight to the zero Height.
func (cs ClientState) CheckSubstituteAndUpdateState(
	ctx sdk.Context, cdc codec.BinaryCodec, subjectClientStore,
	substituteClientStore storetypes.KVStore, substituteClient exported.ClientState,
) error {
	substituteClientState, ok := substituteClient.(*ClientState)
	if !ok {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "expected type %T, got %T", &ClientState{}, substituteClient)
	}

	if !IsMatchingClientState(cs, *substituteClientState) {
		return errorsmod.Wrap(clienttypes.ErrInvalidSubstitute, "subject client state does not match substitute client state")
	}

	if cs.Status(ctx, subjectClientStore, cdc) == exported.Frozen {
		// unfreeze the client
		cs.FrozenHeight = clienttypes.ZeroHeight()
	}

	// copy consensus states and processed time from substitute to subject
	// starting from initial height and ending on the latest height (inclusive)
	height := substituteClientState.GetLatestHeight()

	consensusState, found := GetConsensusState(substituteClientStore, cdc, height)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrConsensusStateNotFound, "unable to retrieve latest consensus state for substitute client")
	}

	setConsensusState(subjectClientStore, cdc, consensusState, height)

	// set metadata stored for the substitute consensus state
	processedHeight, found := GetProcessedHeight(substituteClientStore, height)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrUpdateClientFailed, "unable to retrieve processed height for substitute client latest height")
	}

	processedTime, found := GetProcessedTime(substituteClientStore, height)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrUpdateClientFailed, "unable to retrieve processed time for substitute client latest height")
	}

	setConsensusMetadataWithValues(subjectClientStore, height, processedHeight, processedTime)

	cs.LatestHeight = substituteClientState.LatestHeight
	cs.ChainId = substituteClientState.ChainId

	// set new trusting period based on the substitute client state
	cs.TrustingPeriod = substituteClientState.TrustingPeriod

	// no validation is necessary since the substitute is verified to be Active
	// in 02-client.
	setClientState(subjectClientStore, cdc, &cs)

	return nil
}

// IsMatchingClientState returns true if all the client state parameters match
// except for frozen height, latest height, trusting period, chain-id.
func IsMatchingClientState(subject, substitute ClientState) bool {
	// zero out parameters which do not need to match
	subject.LatestHeight = clienttypes.ZeroHeight()
	subject.FrozenHeight = clienttypes.ZeroHeight()
	subject.TrustingPeriod = time.Duration(0)
	substitute.LatestHeight = clienttypes.ZeroHeight()
	substitute.FrozenHeight = clienttypes.ZeroHeight()
	substitute.TrustingPeriod = time.Duration(0)
	subject.ChainId = ""
	substitute.ChainId = ""
	// sets both sets of flags to true as these flags have been DEPRECATED, see ADR-026 for more information
	subject.AllowUpdateAfterExpiry = true
	substitute.AllowUpdateAfterExpiry = true
	subject.AllowUpdateAfterMisbehaviour = true
	substitute.AllowUpdateAfterMisbehaviour = true

	return reflect.DeepEqual(subject, substitute)
}
