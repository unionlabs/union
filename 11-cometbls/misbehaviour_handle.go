package cometbls

import (
	"reflect"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// CheckForMisbehaviour detects duplicate height misbehaviour and BFT time violation misbehaviour
// in a submitted Header message and verifies the correctness of a submitted Misbehaviour ClientMessage
func (cs ClientState) CheckForMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, msg exported.ClientMessage) bool {
	switch msg := msg.(type) {
	case *Header:
		tmHeader := msg
		consState := tmHeader.ConsensusState()

		// Check if the Client store already has a consensus state for the header's height
		// If the consensus state exists, and it matches the header then we return early
		// since header has already been submitted in a previous UpdateClient.
		if existingConsState, found := GetConsensusState(clientStore, cdc, tmHeader.GetHeight()); found {
			// This header has already been submitted and the necessary state is already stored
			// in client store, thus we can return early without further validation.
			if reflect.DeepEqual(existingConsState, tmHeader.ConsensusState()) { //nolint:gosimple
				return false
			}

			// A consensus state already exists for this height, but it does not match the provided header.
			// The assumption is that Header has already been validated. Thus we can return true as misbehaviour is present
			return true
		}

		// Check that consensus state timestamps are monotonic
		prevCons, prevOk := GetPreviousConsensusState(clientStore, cdc, tmHeader.GetHeight())
		nextCons, nextOk := GetNextConsensusState(clientStore, cdc, tmHeader.GetHeight())
		// if previous consensus state exists, check consensus state time is greater than previous consensus state time
		// if previous consensus state is not before current consensus state return true
		if prevOk && prevCons.Timestamp >= consState.Timestamp {
			return true
		}
		// if next consensus state exists, check consensus state time is less than next consensus state time
		// if next consensus state is not after current consensus state return true
		if nextOk && nextCons.Timestamp <= consState.Timestamp {
			return true
		}
	case *Misbehaviour:
		// we don't do anything here since we already verified any possible misbehaviour in `verifyMisbehaviour`
		return true
	}

	return false
}

// verifyMisbehaviour determines whether or not two conflicting
// headers at the same height would have convinced the light client.
//
// NOTE: consensusState1 is the trusted consensus state that corresponds to the TrustedHeight
// of misbehaviour.Header_1
// Similarly, consensusState2 is the trusted consensus state that corresponds
// to misbehaviour.Header_2
// Misbehaviour sets frozen height to {0, 1} since it is only used as a boolean value (zero or non-zero).
func (cs *ClientState) verifyMisbehaviour(ctx sdk.Context, clientStore storetypes.KVStore, cdc codec.BinaryCodec, misbehaviour *Misbehaviour) error {
	// Regardless of the type of misbehaviour, ensure that both headers are valid and would have been accepted by light-client

	if err := cs.verifyHeader(ctx, clientStore, cdc, misbehaviour.Header_1); err != nil {
		return errorsmod.Wrap(err, "verifying Header_1 in Misbehaviour failed")
	}

	if err := cs.verifyHeader(ctx, clientStore, cdc, misbehaviour.Header_2); err != nil {
		return errorsmod.Wrap(err, "verifying Header_2 in Misbehaviour failed")
	}

	if misbehaviour.Header_1.TrustedHeight == misbehaviour.Header_2.TrustedHeight {
		if reflect.DeepEqual(misbehaviour.Header_1.SignedHeader, misbehaviour.Header_2.SignedHeader) {
			return errorsmod.Wrap(clienttypes.ErrInvalidMisbehaviour, "headers are the same")
		}
	} else {
		if misbehaviour.Header_1.SignedHeader.Time.After(misbehaviour.Header_2.SignedHeader.Time) {
			return errorsmod.Wrap(clienttypes.ErrInvalidMisbehaviour, "headers are in the correct order")
		}
	}

	return nil
}
