package wasm

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	internaltypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/types"
	wasmkeeper "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/keeper"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.LightClientModule = (*LightClientModule)(nil)

// LightClientModule implements the core IBC api.LightClientModule interface.
type LightClientModule struct {
	keeper        wasmkeeper.Keeper
	storeProvider exported.ClientStoreProvider
}

// NewLightClientModule creates and returns a new 08-wasm LightClientModule.
func NewLightClientModule(keeper wasmkeeper.Keeper) LightClientModule {
	return LightClientModule{
		keeper: keeper,
	}
}

// RegisterStoreProvider is called by core IBC when a LightClientModule is added to the router.
// It allows the LightClientModule to set a ClientStoreProvider which supplies isolated prefix client stores
// to IBC light client instances.
func (l *LightClientModule) RegisterStoreProvider(storeProvider exported.ClientStoreProvider) {
	l.storeProvider = storeProvider
}

// Initialize unmarshals the provided client and consensus states and performs basic validation. It sets the client
// state and consensus state in the client store.
// It also initializes the wasm contract for the client.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) Initialize(ctx sdk.Context, clientID string, clientStateBz, consensusStateBz []byte) error {
	var clientState types.ClientState
	if err := l.keeper.Codec().Unmarshal(clientStateBz, &clientState); err != nil {
		return err
	}

	if err := clientState.Validate(); err != nil {
		return err
	}

	var consensusState types.ConsensusState
	if err := l.keeper.Codec().Unmarshal(consensusStateBz, &consensusState); err != nil {
		return err
	}

	if err := consensusState.ValidateBasic(); err != nil {
		return err
	}

	clientStore := l.storeProvider.ClientStore(ctx, clientID)

	// Do not allow initialization of a client with a checksum that hasn't been previously stored via storeWasmCode.
	if !l.keeper.HasChecksum(ctx, clientState.Checksum) {
		return errorsmod.Wrapf(types.ErrInvalidChecksum, "checksum (%s) has not been previously stored", hex.EncodeToString(clientState.Checksum))
	}

	payload := types.InstantiateMessage{
		ClientState:    clientState.Data,
		ConsensusState: consensusState.Data,
		Checksum:       clientState.Checksum,
	}

	return l.keeper.WasmInstantiate(ctx, clientID, clientStore, &clientState, payload)
}

// VerifyClientMessage obtains the client state associated with the client identifier, it then must verify the ClientMessage.
// A ClientMessage could be a Header, Misbehaviour, or batch update.
// It must handle each type of ClientMessage appropriately. Calls to CheckForMisbehaviour, UpdateState, and UpdateStateOnMisbehaviour
// will assume that the content of the ClientMessage has been verified and can be trusted. An error should be returned
// if the ClientMessage fails to verify.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) VerifyClientMessage(ctx sdk.Context, clientID string, clientMsg exported.ClientMessage) error {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	clientMessage, ok := clientMsg.(*types.ClientMessage)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected type: %T, got: %T", &types.ClientMessage{}, clientMsg)
	}

	payload := types.QueryMsg{
		VerifyClientMessage: &types.VerifyClientMessageMsg{ClientMessage: clientMessage.Data},
	}
	_, err := l.keeper.WasmQuery(ctx, clientID, clientStore, clientState, payload)
	return err
}

// CheckForMisbehaviour obtains the client state associated with the client identifier, it detects misbehaviour in a submitted Header
// message and verifies the correctness of a submitted Misbehaviour ClientMessage.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) CheckForMisbehaviour(ctx sdk.Context, clientID string, clientMsg exported.ClientMessage) bool {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		panic(errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID))
	}

	clientMessage, ok := clientMsg.(*types.ClientMessage)
	if !ok {
		return false
	}

	payload := types.QueryMsg{
		CheckForMisbehaviour: &types.CheckForMisbehaviourMsg{ClientMessage: clientMessage.Data},
	}

	res, err := l.keeper.WasmQuery(ctx, clientID, clientStore, clientState, payload)
	if err != nil {
		return false
	}

	var result types.CheckForMisbehaviourResult
	if err := json.Unmarshal(res, &result); err != nil {
		return false
	}

	return result.FoundMisbehaviour
}

// UpdateStateOnMisbehaviour obtains the client state associated with the client identifier performs appropriate state changes on
// a client state given that misbehaviour has been detected and verified.
// Client state is updated in the store by the contract.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) UpdateStateOnMisbehaviour(ctx sdk.Context, clientID string, clientMsg exported.ClientMessage) {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		panic(errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID))
	}

	clientMessage, ok := clientMsg.(*types.ClientMessage)
	if !ok {
		panic(fmt.Errorf("expected type %T, got %T", &types.ClientMessage{}, clientMsg))
	}

	payload := types.SudoMsg{
		UpdateStateOnMisbehaviour: &types.UpdateStateOnMisbehaviourMsg{ClientMessage: clientMessage.Data},
	}

	_, err := l.keeper.WasmSudo(ctx, clientID, clientStore, clientState, payload)
	if err != nil {
		panic(err)
	}
}

// UpdateState obtains the client state associated with the client identifier and calls into the appropriate
// contract endpoint. Client state and new consensus states are updated in the store by the contract.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) UpdateState(ctx sdk.Context, clientID string, clientMsg exported.ClientMessage) []exported.Height {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		panic(errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID))
	}

	clientMessage, ok := clientMsg.(*types.ClientMessage)
	if !ok {
		panic(fmt.Errorf("expected type %T, got %T", &types.ClientMessage{}, clientMsg))
	}

	payload := types.SudoMsg{
		UpdateState: &types.UpdateStateMsg{ClientMessage: clientMessage.Data},
	}

	res, err := l.keeper.WasmSudo(ctx, clientID, clientStore, clientState, payload)
	if err != nil {
		panic(err)
	}

	var result types.UpdateStateResult
	if err := json.Unmarshal(res, &result); err != nil {
		panic(errorsmod.Wrap(types.ErrWasmInvalidResponseData, err.Error()))
	}

	heights := []exported.Height{}
	for _, height := range result.Heights {
		heights = append(heights, height)
	}

	return heights
}

// VerifyMembership obtains the client state associated with the client identifier and calls into the appropriate contract endpoint.
// VerifyMembership is a generic proof verification method which verifies a proof of the existence of a value at a given CommitmentPath at the specified height.
// The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
// If a zero proof height is passed in, it will fail to retrieve the associated consensus state.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) VerifyMembership(
	ctx sdk.Context,
	clientID string,
	height exported.Height,
	delayTimePeriod uint64,
	delayBlockPeriod uint64,
	proof []byte,
	path exported.Path,
	value []byte,
) error {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	proofHeight, ok := height.(clienttypes.Height)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", clienttypes.Height{}, height)
	}

	if clientState.LatestHeight.LT(height) {
		return errorsmod.Wrapf(
			ibcerrors.ErrInvalidHeight,
			"client state height < proof height (%d < %d), please ensure the client has been updated", clientState.LatestHeight, height,
		)
	}

	merklePath, ok := path.(commitmenttypes.MerklePath)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", commitmenttypes.MerklePath{}, path)
	}

	payload := types.SudoMsg{
		VerifyMembership: &types.VerifyMembershipMsg{
			Height:           proofHeight,
			DelayTimePeriod:  delayTimePeriod,
			DelayBlockPeriod: delayBlockPeriod,
			Proof:            proof,
			Path:             merklePath,
			Value:            value,
		},
	}
	_, err := l.keeper.WasmSudo(ctx, clientID, clientStore, clientState, payload)
	return err
}

// VerifyNonMembership obtains the client state associated with the client identifier and calls into the appropriate contract endpoint.
// VerifyNonMembership is a generic proof verification method which verifies the absence of a given CommitmentPath at a specified height.
// The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
// If a zero proof height is passed in, it will fail to retrieve the associated consensus state.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) VerifyNonMembership(
	ctx sdk.Context,
	clientID string,
	height exported.Height,
	delayTimePeriod uint64,
	delayBlockPeriod uint64,
	proof []byte,
	path exported.Path,
) error {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	proofHeight, ok := height.(clienttypes.Height)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", clienttypes.Height{}, height)
	}

	if clientState.LatestHeight.LT(height) {
		return errorsmod.Wrapf(
			ibcerrors.ErrInvalidHeight,
			"client state height < proof height (%d < %d), please ensure the client has been updated", clientState.LatestHeight, height,
		)
	}

	merklePath, ok := path.(commitmenttypes.MerklePath)
	if !ok {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", commitmenttypes.MerklePath{}, path)
	}

	payload := types.SudoMsg{
		VerifyNonMembership: &types.VerifyNonMembershipMsg{
			Height:           proofHeight,
			DelayTimePeriod:  delayTimePeriod,
			DelayBlockPeriod: delayBlockPeriod,
			Proof:            proof,
			Path:             merklePath,
		},
	}
	_, err := l.keeper.WasmSudo(ctx, clientID, clientStore, clientState, payload)
	return err
}

// Status obtains the client state associated with the client identifier and calls into the appropriate contract endpoint.
// It returns the status of the wasm client.
// The client may be:
// - Active: frozen height is zero and client is not expired
// - Frozen: frozen height is not zero
// - Expired: the latest consensus state timestamp + trusting period <= current time
// - Unauthorized: the client type is not registered as an allowed client type
//
// A frozen client will become expired, so the Frozen status
// has higher precedence.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) Status(ctx sdk.Context, clientID string) exported.Status {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return exported.Unknown
	}

	// Return unauthorized if the checksum hasn't been previously stored via storeWasmCode.
	if !l.keeper.HasChecksum(ctx, clientState.Checksum) {
		return exported.Unauthorized
	}

	payload := types.QueryMsg{Status: &types.StatusMsg{}}
	res, err := l.keeper.WasmQuery(ctx, clientID, clientStore, clientState, payload)
	if err != nil {
		return exported.Unknown
	}

	var result types.StatusResult
	if err := json.Unmarshal(res, &result); err != nil {
		return exported.Unknown
	}

	return exported.Status(result.Status)
}

// LatestHeight returns the latest height for the client state for the given client identifier.
// If no client is present for the provided client identifier a zero value height is returned.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) LatestHeight(ctx sdk.Context, clientID string) exported.Height {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return clienttypes.ZeroHeight()
	}

	return clientState.LatestHeight
}

// TimestampAtHeight obtains the client state associated with the client identifier and calls into the appropriate contract endpoint.
// It returns the timestamp in nanoseconds of the consensus state at the given height.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) TimestampAtHeight(ctx sdk.Context, clientID string, height exported.Height) (uint64, error) {
	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	cdc := l.keeper.Codec()

	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return 0, errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	timestampHeight, ok := height.(clienttypes.Height)
	if !ok {
		return 0, errorsmod.Wrapf(ibcerrors.ErrInvalidType, "expected %T, got %T", clienttypes.Height{}, height)
	}

	payload := types.QueryMsg{
		TimestampAtHeight: &types.TimestampAtHeightMsg{
			Height: timestampHeight,
		},
	}

	res, err := l.keeper.WasmQuery(ctx, clientID, clientStore, clientState, payload)
	if err != nil {
		return 0, errorsmod.Wrapf(err, "height (%s)", height)
	}

	var result types.TimestampAtHeightResult
	if err := json.Unmarshal(res, &result); err != nil {
		return 0, errorsmod.Wrapf(types.ErrWasmInvalidResponseData, "failed to unmarshal result of wasm query: %v", err)
	}

	return result.Timestamp, nil
}

// RecoverClient asserts that the substitute client is a wasm client. It obtains the client state associated with the
// subject client and calls into the appropriate contract endpoint.
// It will verify that a substitute client state is valid and update the subject client state.
// Note that this method is used only for recovery and will not allow changes to the checksum.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) RecoverClient(ctx sdk.Context, clientID, substituteClientID string) error {
	substituteClientType, _, err := clienttypes.ParseClientIdentifier(substituteClientID)
	if err != nil {
		return err
	}

	if substituteClientType != types.Wasm {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClientType, "expected: %s, got: %s", types.Wasm, substituteClientType)
	}

	cdc := l.keeper.Codec()

	subjectClientStore := l.storeProvider.ClientStore(ctx, clientID)
	subjectClientState, found := types.GetClientState(subjectClientStore, cdc)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	substituteClientStore := l.storeProvider.ClientStore(ctx, substituteClientID)
	substituteClientState, found := types.GetClientState(substituteClientStore, cdc)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, substituteClientID)
	}

	// check that checksums of subject client state and substitute client state match
	// changing the checksum is only allowed through the migrate contract RPC endpoint
	if !bytes.Equal(subjectClientState.Checksum, substituteClientState.Checksum) {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "expected checksums to be equal: expected %s, got %s", hex.EncodeToString(subjectClientState.Checksum), hex.EncodeToString(substituteClientState.Checksum))
	}

	store := internaltypes.NewClientRecoveryStore(subjectClientStore, substituteClientStore)

	payload := types.SudoMsg{
		MigrateClientStore: &types.MigrateClientStoreMsg{},
	}

	_, err = l.keeper.WasmSudo(ctx, clientID, store, subjectClientState, payload)
	return err
}

// VerifyUpgradeAndUpdateState obtains the client state associated with the client identifier and calls into the appropriate contract endpoint.
// The new client and consensus states will be unmarshaled and an error is returned if the new client state is not at a height greater
// than the existing client. On a successful verification, it expects the contract to update the new client state, consensus state, and any other client metadata.
//
// CONTRACT: clientID is validated in 02-client router, thus clientID is assumed here to have the format 08-wasm-{n}.
func (l LightClientModule) VerifyUpgradeAndUpdateState(
	ctx sdk.Context,
	clientID string,
	newClient []byte,
	newConsState []byte,
	upgradeClientProof,
	upgradeConsensusStateProof []byte,
) error {
	cdc := l.keeper.Codec()

	var newClientState types.ClientState
	if err := cdc.Unmarshal(newClient, &newClientState); err != nil {
		return errorsmod.Wrap(clienttypes.ErrInvalidClient, err.Error())
	}

	var newConsensusState types.ConsensusState
	if err := cdc.Unmarshal(newConsState, &newConsensusState); err != nil {
		return errorsmod.Wrap(clienttypes.ErrInvalidConsensus, err.Error())
	}

	clientStore := l.storeProvider.ClientStore(ctx, clientID)
	clientState, found := types.GetClientState(clientStore, cdc)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	// last height of current counterparty chain must be client's latest height
	lastHeight := clientState.LatestHeight
	if !newClientState.LatestHeight.GT(lastHeight) {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidHeight, "upgraded client height %s must be at greater than current client height %s", newClientState.LatestHeight, lastHeight)
	}

	payload := types.SudoMsg{
		VerifyUpgradeAndUpdateState: &types.VerifyUpgradeAndUpdateStateMsg{
			UpgradeClientState:         newClientState.Data,
			UpgradeConsensusState:      newConsensusState.Data,
			ProofUpgradeClient:         upgradeClientProof,
			ProofUpgradeConsensusState: upgradeConsensusStateProof,
		},
	}

	_, err := l.keeper.WasmSudo(ctx, clientID, clientStore, clientState, payload)
	return err
}
