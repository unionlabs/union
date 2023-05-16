package localhost

import (
	"bytes"

	errorsmod "cosmossdk.io/errors"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcerrors "github.com/cosmos/ibc-go/v7/internal/errors"
	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v7/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var _ exported.ClientState = (*ClientState)(nil)

// NewClientState creates a new 09-localhost ClientState instance.
func NewClientState(height clienttypes.Height) exported.ClientState {
	return &ClientState{
		LatestHeight: height,
	}
}

// ClientType returns the 09-localhost client type.
func (cs ClientState) ClientType() string {
	return exported.Localhost
}

// GetLatestHeight returns the 09-localhost client state latest height.
func (cs ClientState) GetLatestHeight() exported.Height {
	return cs.LatestHeight
}

// Status always returns Active. The 09-localhost status cannot be changed.
func (cs ClientState) Status(_ sdk.Context, _ sdk.KVStore, _ codec.BinaryCodec) exported.Status {
	return exported.Active
}

// Validate performs a basic validation of the client state fields.
func (cs ClientState) Validate() error {
	if cs.LatestHeight.RevisionHeight == 0 {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidHeight, "local revision height cannot be zero")
	}

	return nil
}

// ZeroCustomFields returns the same client state since there are no custom fields in the 09-localhost client state.
func (cs ClientState) ZeroCustomFields() exported.ClientState {
	return &cs
}

// Initialize ensures that initial consensus state for localhost is nil.
func (cs ClientState) Initialize(ctx sdk.Context, cdc codec.BinaryCodec, clientStore sdk.KVStore, consState exported.ConsensusState) error {
	if consState != nil {
		return errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "initial consensus state for localhost must be nil.")
	}

	clientState := ClientState{
		LatestHeight: clienttypes.GetSelfHeight(ctx),
	}

	clientStore.Set(host.ClientStateKey(), clienttypes.MustMarshalClientState(cdc, &clientState))

	return nil
}

// GetTimestampAtHeight returns the current block time retrieved from the application context. The localhost client does not store consensus states and thus
// cannot provide a timestamp for the provided height.
func (cs ClientState) GetTimestampAtHeight(ctx sdk.Context, _ sdk.KVStore, _ codec.BinaryCodec, _ exported.Height) (uint64, error) {
	return uint64(ctx.BlockTime().UnixNano()), nil
}

// VerifyMembership is a generic proof verification method which verifies the existence of a given key and value within the IBC store.
// The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
// The caller must provide the full IBC store.
func (cs ClientState) VerifyMembership(
	ctx sdk.Context,
	store sdk.KVStore,
	_ codec.BinaryCodec,
	_ exported.Height,
	_ uint64,
	_ uint64,
	proof []byte,
	path exported.Path,
	value []byte,
) error {
	// ensure the proof provided is the expected sentinel localhost client proof
	if !bytes.Equal(proof, SentinelProof) {
		return errorsmod.Wrapf(commitmenttypes.ErrInvalidProof, "expected %s, got %s", string(SentinelProof), string(proof))
	}

	merklePath, ok := path.(commitmenttypes.MerklePath)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", commitmenttypes.MerklePath{}, path)
	}

	if len(merklePath.GetKeyPath()) != 2 {
		return errorsmod.Wrapf(host.ErrInvalidPath, "path must be of length 2: %s", merklePath.GetKeyPath())
	}

	// The commitment prefix (eg: "ibc") is omitted when operating on the core IBC store
	bz := store.Get([]byte(merklePath.KeyPath[1]))
	if bz == nil {
		return errorsmod.Wrapf(clienttypes.ErrFailedMembershipVerification, "value not found for path %s", path)
	}

	if !bytes.Equal(bz, value) {
		return errorsmod.Wrapf(clienttypes.ErrFailedMembershipVerification, "value provided does not equal value stored at path: %s", path)
	}

	return nil
}

// VerifyNonMembership is a generic proof verification method which verifies the absence of a given CommitmentPath within the IBC store.
// The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
// The caller must provide the full IBC store.
func (cs ClientState) VerifyNonMembership(
	ctx sdk.Context,
	store sdk.KVStore,
	_ codec.BinaryCodec,
	_ exported.Height,
	_ uint64,
	_ uint64,
	proof []byte,
	path exported.Path,
) error {
	// ensure the proof provided is the expected sentinel localhost client proof
	if !bytes.Equal(proof, SentinelProof) {
		return errorsmod.Wrapf(commitmenttypes.ErrInvalidProof, "expected %s, got %s", string(SentinelProof), string(proof))
	}

	merklePath, ok := path.(commitmenttypes.MerklePath)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", commitmenttypes.MerklePath{}, path)
	}

	if len(merklePath.GetKeyPath()) != 2 {
		return errorsmod.Wrapf(host.ErrInvalidPath, "path must be of length 2: %s", merklePath.GetKeyPath())
	}

	// The commitment prefix (eg: "ibc") is omitted when operating on the core IBC store
	if store.Has([]byte(merklePath.KeyPath[1])) {
		return errorsmod.Wrapf(clienttypes.ErrFailedNonMembershipVerification, "value found for path %s", path)
	}

	return nil
}

// VerifyClientMessage is unsupported by the 09-localhost client type and returns an error.
func (cs ClientState) VerifyClientMessage(_ sdk.Context, _ codec.BinaryCodec, _ sdk.KVStore, _ exported.ClientMessage) error {
	return errorsmod.Wrap(clienttypes.ErrUpdateClientFailed, "client message verification is unsupported by the localhost client")
}

// CheckForMisbehaviour is unsupported by the 09-localhost client type and performs a no-op, returning false.
func (cs ClientState) CheckForMisbehaviour(_ sdk.Context, _ codec.BinaryCodec, _ sdk.KVStore, _ exported.ClientMessage) bool {
	return false
}

// UpdateStateOnMisbehaviour is unsupported by the 09-localhost client type and performs a no-op.
func (cs ClientState) UpdateStateOnMisbehaviour(_ sdk.Context, _ codec.BinaryCodec, _ sdk.KVStore, _ exported.ClientMessage) {
}

// UpdateState updates and stores as necessary any associated information for an IBC client, such as the ClientState and corresponding ConsensusState.
// Upon successful update, a list of consensus heights is returned. It assumes the ClientMessage has already been verified.
func (cs ClientState) UpdateState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore sdk.KVStore, _ exported.ClientMessage) []exported.Height {
	height := clienttypes.GetSelfHeight(ctx)
	cs.LatestHeight = height

	clientStore.Set(host.ClientStateKey(), clienttypes.MustMarshalClientState(cdc, &cs))

	return []exported.Height{height}
}

// ExportMetadata is a no-op for the 09-localhost client.
func (cs ClientState) ExportMetadata(_ sdk.KVStore) []exported.GenesisMetadata {
	return nil
}

// CheckSubstituteAndUpdateState returns an error. The localhost cannot be modified by
// proposals.
func (cs ClientState) CheckSubstituteAndUpdateState(_ sdk.Context, _ codec.BinaryCodec, _, _ sdk.KVStore, _ exported.ClientState) error {
	return errorsmod.Wrap(clienttypes.ErrUpdateClientFailed, "cannot update localhost client with a proposal")
}

// VerifyUpgradeAndUpdateState returns an error since localhost cannot be upgraded
func (cs ClientState) VerifyUpgradeAndUpdateState(
	_ sdk.Context,
	_ codec.BinaryCodec,
	_ sdk.KVStore,
	_ exported.ClientState,
	_ exported.ConsensusState,
	_,
	_ []byte,
) error {
	return errorsmod.Wrap(clienttypes.ErrInvalidUpgradeClient, "cannot upgrade localhost client")
}
