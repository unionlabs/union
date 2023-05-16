package keeper

import (
	errorsmod "cosmossdk.io/errors"
	metrics "github.com/armon/go-metrics"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
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
	proofUpgradeClient, proofUpgradeConsState []byte,
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
		upgradedClient, upgradedConsState, proofUpgradeClient, proofUpgradeConsState,
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
