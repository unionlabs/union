package cometbls

import (
	"bytes"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// VerifyClientMessage checks if the clientMessage is of type Header or Misbehaviour and verifies the message
func (cs *ClientState) VerifyClientMessage(
	ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore,
	clientMsg exported.ClientMessage,
) error {
	switch msg := clientMsg.(type) {
	case *Header:
		return cs.verifyHeader(ctx, clientStore, cdc, msg)
	case *Misbehaviour:
		return cs.verifyMisbehaviour(ctx, clientStore, cdc, msg)
	default:
		return clienttypes.ErrInvalidClientType
	}
}

// verifyHeader returns an error if:
// - the client or header provided are not parseable to tendermint types
// - the header is invalid
// - header height is less than or equal to the trusted header height
// - header revision is not equal to trusted header revision
// - header valset commit verification fails
// - header timestamp is past the trusting period in relation to the consensus state
// - header timestamp is less than or equal to the consensus state timestamp
func (cs *ClientState) verifyHeader(
	ctx sdk.Context, clientStore storetypes.KVStore, cdc codec.BinaryCodec,
	header *Header,
) error {
	// Retrieve trusted consensus states for each Header in misbehaviour
	consState, found := GetConsensusState(clientStore, cdc, header.TrustedHeight)
	if !found {
		return errorsmod.Wrapf(clienttypes.ErrConsensusStateNotFound, "could not get trusted consensus state from clientStore for Header at TrustedHeight: %s", header.TrustedHeight)
	}

	// UpdateClient only accepts updates with a header at the same revision
	// as the trusted consensus state
	if header.GetHeight().GetRevisionNumber() != header.TrustedHeight.RevisionNumber {
		return errorsmod.Wrapf(
			ErrInvalidHeaderHeight,
			"header height revision %d does not match trusted header revision %d",
			header.GetHeight().GetRevisionNumber(), header.TrustedHeight.RevisionNumber,
		)
	}

	if consState.GetTimestamp() > uint64(header.SignedHeader.Header.GetTime().UnixNano()) {
		return errorsmod.Wrapf(
			ErrInvalidHeaderTimestamp,
			"trusted header timestamp %d is greater than the new header timestamp %d",
			consState.GetTimestamp(), header.SignedHeader.Header.GetTime().UnixNano(),
		)
	}

	// assert header height is newer than consensus state
	if header.GetHeight().LTE(header.TrustedHeight) {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"header height ≤ consensus state height (%s ≤ %s)", header.GetHeight(), header.TrustedHeight,
		)
	}

	if header.GetTime().UnixNano() >= ctx.BlockHeader().Time.UnixNano()+int64(cs.MaxClockDrift) {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"header time >= max drift (%d >= currentTime + %d)", header.GetTime().UnixNano(), cs.MaxClockDrift,
		)
	}

	if header.SignedHeader.Header.Height == int64(header.TrustedHeight.RevisionHeight)+1 &&
		!bytes.Equal(header.SignedHeader.Header.ValidatorsHash, consState.NextValidatorsHash) {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"the validators hash %s doesn't match the trusted validators hash %s for an adjacent block", header.SignedHeader.Header.ValidatorsHash, consState.NextValidatorsHash,
		)
	}

	zkp, err := ParseZKP(header.ZeroKnowledgeProof)

	if err != nil {
		return err
	}

	return zkp.Verify(consState.NextValidatorsHash, LightHeader{
		ChainId:            header.SignedHeader.Header.ChainID,
		Height:             header.SignedHeader.Header.Height,
		Time:               header.GetTime(),
		ValidatorsHash:     header.SignedHeader.Header.ValidatorsHash,
		NextValidatorsHash: header.SignedHeader.Header.NextValidatorsHash,
		AppHash:            header.SignedHeader.Header.AppHash,
	})
}

// UpdateState may be used to either create a consensus state for:
// - a future height greater than the latest client state height
// - a past height that was skipped during bisection
// If we are updating to a past height, a consensus state is created for that height to be persisted in client store
// If we are updating to a future height, the consensus state is created and the client state is updated to reflect
// the new latest height
// A list containing the updated consensus height is returned.
// UpdateState must only be used to update within a single revision, thus header revision number and trusted height's revision
// number must be the same. To update to a new revision, use a separate upgrade path
// UpdateState will prune the oldest consensus state if it is expired.
// If the provided clientMsg is not of type of Header then the handler will noop and empty slice is returned.
func (cs ClientState) UpdateState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) []exported.Height {
	header, ok := clientMsg.(*Header)
	if !ok {
		// clientMsg is invalid Misbehaviour, no update necessary
		return []exported.Height{}
	}

	// performance: do not prune in checkTx
	// simulation must prune for accurate gas estimation
	if (!ctx.IsCheckTx() && !ctx.IsReCheckTx()) || ctx.ExecMode() == sdk.ExecModeSimulate {
		cs.pruneOldestConsensusState(ctx, cdc, clientStore)
	}

	// check for duplicate update
	if _, found := GetConsensusState(clientStore, cdc, header.GetHeight()); found {
		// perform no-op
		return []exported.Height{header.GetHeight()}
	}

	height := header.GetHeight().(clienttypes.Height)
	if height.GT(cs.LatestHeight) {
		cs.LatestHeight = height
	}

	consensusState := &ConsensusState{
		Timestamp:          uint64(header.GetTime().UnixNano()),
		Root:               commitmenttypes.NewMerkleRoot(header.SignedHeader.Header.GetAppHash()),
		NextValidatorsHash: header.SignedHeader.Header.NextValidatorsHash,
	}

	// set client state, consensus state and asssociated metadata
	setClientState(clientStore, cdc, &cs)
	setConsensusState(clientStore, cdc, consensusState, header.GetHeight())
	setConsensusMetadata(ctx, clientStore, header.GetHeight())

	return []exported.Height{height}
}

// pruneOldestConsensusState will retrieve the earliest consensus state for this clientID and check if it is expired. If it is,
// that consensus state will be pruned from store along with all associated metadata. This will prevent the client store from
// becoming bloated with expired consensus states that can no longer be used for updates and packet verification.
func (cs ClientState) pruneOldestConsensusState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore) {
	// Check the earliest consensus state to see if it is expired, if so then set the prune height
	// so that we can delete consensus state and all associated metadata.
	var (
		pruneHeight exported.Height
	)

	pruneCb := func(height exported.Height) bool {
		consState, found := GetConsensusState(clientStore, cdc, height)
		// this error should never occur
		if !found {
			panic(errorsmod.Wrapf(clienttypes.ErrConsensusStateNotFound, "failed to retrieve consensus state at height: %s", height))
		}

		if cs.IsExpired(consState.Timestamp, uint64(ctx.BlockTime().UnixNano())) {
			pruneHeight = height
		}

		return true
	}

	IterateConsensusStateAscending(clientStore, pruneCb)

	// if pruneHeight is set, delete consensus state and metadata
	if pruneHeight != nil {
		deleteConsensusState(clientStore, pruneHeight)
		deleteConsensusMetadata(clientStore, pruneHeight)
	}
}

// UpdateStateOnMisbehaviour updates state upon misbehaviour, freezing the ClientState. This method should only be called when misbehaviour is detected
// as it does not perform any misbehaviour checks.
func (cs ClientState) UpdateStateOnMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, _ exported.ClientMessage) {
	cs.FrozenHeight = FrozenHeight

	clientStore.Set(host.ClientStateKey(), clienttypes.MustMarshalClientState(cdc, &cs))
}
