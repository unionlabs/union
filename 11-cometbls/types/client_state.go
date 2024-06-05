package types

import (
	"time"

	ics23 "github.com/cosmos/ics23/go"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ClientState = (*ClientState)(nil)

// NewClientState creates a new ClientState instance
func NewClientState(
	chainID string,
	trustingPeriod, ubdPeriod, maxClockDrift uint64,
	latestHeight clienttypes.Height, specs []*ics23.ProofSpec,
	upgradePath []string,
) *ClientState {
	return &ClientState{
		ChainId:         chainID,
		TrustingPeriod:  trustingPeriod,
		UnbondingPeriod: ubdPeriod,
		MaxClockDrift:   maxClockDrift,
		LatestHeight:    latestHeight,
		FrozenHeight:    clienttypes.ZeroHeight(),
	}
}

// GetChainID returns the chain-id
func (cs ClientState) GetChainID() string {
	return cs.ChainId
}

// ClientType is tendermint.
func (ClientState) ClientType() string {
	return "11-cometbls"
}

// GetLatestHeight returns latest block height.
func (cs ClientState) GetLatestHeight() exported.Height {
	return cs.LatestHeight
}

// GetTimestampAtHeight returns the timestamp in nanoseconds of the consensus state at the given height.
func (ClientState) GetTimestampAtHeight(
	ctx sdk.Context,
	clientStore storetypes.KVStore,
	cdc codec.BinaryCodec,
	height exported.Height,
) (uint64, error) {
	// get consensus state at height from clientStore to check for expiry
	consState, found := GetConsensusState(clientStore, cdc, height)
	if !found {
		return 0, errorsmod.Wrapf(clienttypes.ErrConsensusStateNotFound, "height (%s)", height)
	}
	return consState.Timestamp, nil
}

// Status returns the status of the tendermint client.
// The client may be:
// - Active: FrozenHeight is zero and client is not expired
// - Frozen: Frozen Height is not zero
// - Expired: the latest consensus state timestamp + trusting period <= current time
//
// A frozen client will become expired, so the Frozen status
// has higher precedence.
func (cs ClientState) Status(
	ctx sdk.Context,
	clientStore storetypes.KVStore,
	cdc codec.BinaryCodec,
) exported.Status {
	if !cs.FrozenHeight.IsZero() {
		return exported.Frozen
	}

	// get latest consensus state from clientStore to check for expiry
	consState, found := GetConsensusState(clientStore, cdc, cs.GetLatestHeight())
	if !found {
		// if the client state does not have an associated consensus state for its latest height
		// then it must be expired
		return exported.Expired
	}

	if cs.IsExpired(time.UnixMilli(int64(consState.Timestamp)), ctx.BlockTime()) {
		return exported.Expired
	}

	return exported.Active
}

// IsExpired returns whether or not the client has passed the trusting period since the last
// update (in which case no headers are considered valid).
func (cs ClientState) IsExpired(latestTimestamp, now time.Time) bool {
	return uint64(latestTimestamp.UnixMilli())+cs.TrustingPeriod > uint64(now.UnixMilli())
}

// ExportMetadata must export metadata stored within the clientStore for genesis export
func (cs ClientState) ExportMetadata(clientStore storetypes.KVStore) []exported.GenesisMetadata {
	return []exported.GenesisMetadata{}
}

// ZeroCustomFields zeroes out any client customizable fields in client state
// Ledger enforced fields are maintained while all custom fields are zero values
// Used to verify upgrades
func (cs ClientState) ZeroCustomFields() exported.ClientState {
	return &ClientState{}
}

// Initialize is called upon client creation, it allows the client to perform validation on the initial consensus state and set the
// client state, consensus state and any client-specific metadata necessary for correct light client operation in the provided client store.
func (cs ClientState) Initialize(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, consensusState exported.ConsensusState) error {
	return nil
}

// VerifyMembership is a generic proof verification method which verifies a proof of the existence of a value at a given CommitmentPath at the specified height.
// The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
func (cs ClientState) VerifyMembership(
	ctx sdk.Context,
	clientStore storetypes.KVStore,
	cdc codec.BinaryCodec,
	height exported.Height,
	delayTimePeriod uint64,
	delayBlockPeriod uint64,
	proof []byte,
	path exported.Path,
	value []byte,
) error {
	return nil
}

// VerifyNonMembership is a generic proof verification method which verifies the absence of a given CommitmentPath at a specified height.
// The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
func (cs ClientState) VerifyNonMembership(
	ctx sdk.Context,
	clientStore storetypes.KVStore,
	cdc codec.BinaryCodec,
	height exported.Height,
	delayTimePeriod uint64,
	delayBlockPeriod uint64,
	proof []byte,
	path exported.Path,
) error {
	return nil
}

// VerifyClientMessage must verify a ClientMessage. A ClientMessage could be a Header, Misbehaviour, or batch update.
// It must handle each type of ClientMessage appropriately. Calls to CheckForMisbehaviour, UpdateState, and UpdateStateOnMisbehaviour
// will assume that the content of the ClientMessage has been verified and can be trusted. An error should be returned
// if the ClientMessage fails to verify.
func (cs ClientState) VerifyClientMessage(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) error {
	return nil
}

// Checks for evidence of a misbehaviour in Header or Misbehaviour type. It assumes the ClientMessage
// has already been verified.
func (cs ClientState) CheckForMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) bool {
	return false
}

// UpdateStateOnMisbehaviour should perform appropriate state changes on a client state given that misbehaviour has been detected and verified
func (cs ClientState) UpdateStateOnMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) {

}

// UpdateState updates and stores as necessary any associated information for an IBC client, such as the ClientState and corresponding ConsensusState.
// Upon successful update, a list of consensus heights is returned. It assumes the ClientMessage has already been verified.
func (cs ClientState) UpdateState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) []exported.Height {
	return []exported.Height{}
}

// CheckSubstituteAndUpdateState must verify that the provided substitute may be used to update the subject client.
// The light client must set the updated client and consensus states within the clientStore for the subject client.
func (cs ClientState) CheckSubstituteAndUpdateState(ctx sdk.Context, cdc codec.BinaryCodec, subjectClientStore, substituteClientStore storetypes.KVStore, substituteClient exported.ClientState) error {
	return nil
}

// Upgrade functions
// NOTE: proof heights are not included as upgrade to a new revision is expected to pass only on the last
// height committed by the current revision. Clients are responsible for ensuring that the planned last
// height of the current revision is somehow encoded in the proof verification process.
// This is to ensure that no premature upgrades occur, since upgrade plans committed to by the counterparty
// may be cancelled or modified before the last planned height.
// If the upgrade is verified, the upgraded client and consensus states must be set in the client store.
func (cs ClientState) VerifyUpgradeAndUpdateState(
	ctx sdk.Context,
	cdc codec.BinaryCodec,
	store storetypes.KVStore,
	newClient exported.ClientState,
	newConsState exported.ConsensusState,
	upgradeClientProof,
	upgradeConsensusStateProof []byte,
) error {
	return nil
}

func (cs ClientState) Validate() error {
	return nil
}
