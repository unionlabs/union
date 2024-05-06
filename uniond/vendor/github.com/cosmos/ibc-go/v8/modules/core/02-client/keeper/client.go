package keeper

import (
	metrics "github.com/hashicorp/go-metrics"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// CreateClient generates a new client identifier and isolated prefix store for the provided client state.
// The client state is responsible for setting any client-specific data in the store via the Initialize method.
// This includes the client state, initial consensus state and any associated metadata.
func (k Keeper) CreateClient(
	ctx sdk.Context, clientState exported.ClientState, consensusState exported.ConsensusState,
) (string, error) {
	if clientState.ClientType() == exported.Localhost {
		return "", errorsmod.Wrapf(types.ErrInvalidClientType, "cannot create client of type: %s", clientState.ClientType())
	}

	params := k.GetParams(ctx)
	if !params.IsAllowedClient(clientState.ClientType()) {
		return "", errorsmod.Wrapf(
			types.ErrInvalidClientType,
			"client state type %s is not registered in the allowlist", clientState.ClientType(),
		)
	}

	clientID := k.GenerateClientIdentifier(ctx, clientState.ClientType())
	clientStore := k.ClientStore(ctx, clientID)

	if err := clientState.Initialize(ctx, k.cdc, clientStore, consensusState); err != nil {
		return "", err
	}

	if status := k.GetClientStatus(ctx, clientState, clientID); status != exported.Active {
		return "", errorsmod.Wrapf(types.ErrClientNotActive, "cannot create client (%s) with status %s", clientID, status)
	}

	k.Logger(ctx).Info("client created at height", "client-id", clientID, "height", clientState.GetLatestHeight().String())

	defer telemetry.IncrCounterWithLabels(
		[]string{"ibc", "client", "create"},
		1,
		[]metrics.Label{telemetry.NewLabel(types.LabelClientType, clientState.ClientType())},
	)

	emitCreateClientEvent(ctx, clientID, clientState)

	return clientID, nil
}

// UpdateClient updates the consensus state and the state root from a provided header.
func (k Keeper) UpdateClient(ctx sdk.Context, clientID string, clientMsg exported.ClientMessage) error {
	clientState, found := k.GetClientState(ctx, clientID)
	if !found {
		return errorsmod.Wrapf(types.ErrClientNotFound, "cannot update client with ID %s", clientID)
	}

	clientStore := k.ClientStore(ctx, clientID)

	if status := k.GetClientStatus(ctx, clientState, clientID); status != exported.Active {
		return errorsmod.Wrapf(types.ErrClientNotActive, "cannot update client (%s) with status %s", clientID, status)
	}

	if err := clientState.VerifyClientMessage(ctx, k.cdc, clientStore, clientMsg); err != nil {
		return err
	}

	foundMisbehaviour := clientState.CheckForMisbehaviour(ctx, k.cdc, clientStore, clientMsg)
	if foundMisbehaviour {
		clientState.UpdateStateOnMisbehaviour(ctx, k.cdc, clientStore, clientMsg)

		k.Logger(ctx).Info("client frozen due to misbehaviour", "client-id", clientID)

		defer telemetry.IncrCounterWithLabels(
			[]string{"ibc", "client", "misbehaviour"},
			1,
			[]metrics.Label{
				telemetry.NewLabel(types.LabelClientType, clientState.ClientType()),
				telemetry.NewLabel(types.LabelClientID, clientID),
				telemetry.NewLabel(types.LabelMsgType, "update"),
			},
		)

		emitSubmitMisbehaviourEvent(ctx, clientID, clientState)

		return nil
	}

	consensusHeights := clientState.UpdateState(ctx, k.cdc, clientStore, clientMsg)

	k.Logger(ctx).Info("client state updated", "client-id", clientID, "heights", consensusHeights)

	defer telemetry.IncrCounterWithLabels(
		[]string{"ibc", "client", "update"},
		1,
		[]metrics.Label{
			telemetry.NewLabel(types.LabelClientType, clientState.ClientType()),
			telemetry.NewLabel(types.LabelClientID, clientID),
			telemetry.NewLabel(types.LabelUpdateType, "msg"),
		},
	)

	// emitting events in the keeper emits for both begin block and handler client updates
	emitUpdateClientEvent(ctx, clientID, clientState.ClientType(), consensusHeights, k.cdc, clientMsg)

	return nil
}

// UpgradeClient upgrades the client to a new client state if this new client was committed to
// by the old client at the specified upgrade height
func (k Keeper) UpgradeClient(ctx sdk.Context, clientID string, upgradedClient exported.ClientState, upgradedConsState exported.ConsensusState,
	upgradeClientProof, upgradeConsensusStateProof []byte,
) error {
	clientState, found := k.GetClientState(ctx, clientID)
	if !found {
		return errorsmod.Wrapf(types.ErrClientNotFound, "cannot update client with ID %s", clientID)
	}

	clientStore := k.ClientStore(ctx, clientID)

	if status := k.GetClientStatus(ctx, clientState, clientID); status != exported.Active {
		return errorsmod.Wrapf(types.ErrClientNotActive, "cannot upgrade client (%s) with status %s", clientID, status)
	}

	if err := clientState.VerifyUpgradeAndUpdateState(ctx, k.cdc, clientStore,
		upgradedClient, upgradedConsState, upgradeClientProof, upgradeConsensusStateProof,
	); err != nil {
		return errorsmod.Wrapf(err, "cannot upgrade client with ID %s", clientID)
	}

	k.Logger(ctx).Info("client state upgraded", "client-id", clientID, "height", upgradedClient.GetLatestHeight().String())

	defer telemetry.IncrCounterWithLabels(
		[]string{"ibc", "client", "upgrade"},
		1,
		[]metrics.Label{
			telemetry.NewLabel(types.LabelClientType, upgradedClient.ClientType()),
			telemetry.NewLabel(types.LabelClientID, clientID),
		},
	)

	emitUpgradeClientEvent(ctx, clientID, upgradedClient)

	return nil
}

// RecoverClient will retrieve the subject and substitute client.
// A callback will occur to the subject client state with the client
// prefixed store being provided for both the subject and the substitute client.
// The IBC client implementations are responsible for validating the parameters of the
// substitute (ensuring they match the subject's parameters) as well as copying
// the necessary consensus states from the substitute to the subject client
// store. The substitute must be Active and the subject must not be Active.
func (k Keeper) RecoverClient(ctx sdk.Context, subjectClientID, substituteClientID string) error {
	subjectClientState, found := k.GetClientState(ctx, subjectClientID)
	if !found {
		return errorsmod.Wrapf(types.ErrClientNotFound, "subject client with ID %s", subjectClientID)
	}

	subjectClientStore := k.ClientStore(ctx, subjectClientID)

	if status := k.GetClientStatus(ctx, subjectClientState, subjectClientID); status == exported.Active {
		return errorsmod.Wrapf(types.ErrInvalidRecoveryClient, "cannot recover %s subject client", exported.Active)
	}

	substituteClientState, found := k.GetClientState(ctx, substituteClientID)
	if !found {
		return errorsmod.Wrapf(types.ErrClientNotFound, "substitute client with ID %s", substituteClientID)
	}

	if subjectClientState.GetLatestHeight().GTE(substituteClientState.GetLatestHeight()) {
		return errorsmod.Wrapf(types.ErrInvalidHeight, "subject client state latest height is greater or equal to substitute client state latest height (%s >= %s)", subjectClientState.GetLatestHeight(), substituteClientState.GetLatestHeight())
	}

	substituteClientStore := k.ClientStore(ctx, substituteClientID)

	if status := k.GetClientStatus(ctx, substituteClientState, substituteClientID); status != exported.Active {
		return errorsmod.Wrapf(types.ErrClientNotActive, "substitute client is not %s, status is %s", exported.Active, status)
	}

	if err := subjectClientState.CheckSubstituteAndUpdateState(ctx, k.cdc, subjectClientStore, substituteClientStore, substituteClientState); err != nil {
		return errorsmod.Wrap(err, "failed to validate substitute client")
	}

	k.Logger(ctx).Info("client recovered", "client-id", subjectClientID)

	defer telemetry.IncrCounterWithLabels(
		[]string{"ibc", "client", "update"},
		1,
		[]metrics.Label{
			telemetry.NewLabel(types.LabelClientType, substituteClientState.ClientType()),
			telemetry.NewLabel(types.LabelClientID, subjectClientID),
			telemetry.NewLabel(types.LabelUpdateType, "recovery"),
		},
	)

	// emitting events in the keeper for recovering clients
	emitRecoverClientEvent(ctx, subjectClientID, substituteClientState.ClientType())

	return nil
}
