package beacon_api

import (
	"context"
	"encoding/binary"
	"fmt"
	"strconv"

	"github.com/pkg/errors"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

type DoppelGangerInfo struct {
	validatorEpoch primitives.Epoch
	response       *ethpb.DoppelGangerResponse_ValidatorResponse
}

func (c *beaconApiValidatorClient) checkDoppelGanger(ctx context.Context, in *ethpb.DoppelGangerRequest) (*ethpb.DoppelGangerResponse, error) {
	// Check if there is any doppelganger validator for the last 2 epochs.
	// - Check if the beacon node is synced
	// - If we are in Phase0, we consider there is no doppelganger.
	// - If all validators we want to check doppelganger existence were live in local antislashing
	//   database for the last 2 epochs, we consider there is no doppelganger.
	//   This is typically the case when we reboot the validator client.
	// - If some validators we want to check doppelganger existence were NOT live
	//   in local antislashing for the last two epochs, then we check onchain if there is
	//   some liveness for these validators. If yes, we consider there is a doppelganger.

	// Check inputs are correct.
	if in == nil || in.ValidatorRequests == nil || len(in.ValidatorRequests) == 0 {
		return &ethpb.DoppelGangerResponse{
			Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{},
		}, nil
	}

	validatorRequests := in.ValidatorRequests

	// Prepare response.
	stringPubKeys := make([]string, len(validatorRequests))
	stringPubKeyToDoppelGangerInfo := make(map[string]DoppelGangerInfo, len(validatorRequests))

	for i, vr := range validatorRequests {
		if vr == nil {
			return nil, errors.New("validator request is nil")
		}

		pubKey := vr.PublicKey
		stringPubKey := hexutil.Encode(pubKey)
		stringPubKeys[i] = stringPubKey

		stringPubKeyToDoppelGangerInfo[stringPubKey] = DoppelGangerInfo{
			validatorEpoch: vr.Epoch,
			response: &ethpb.DoppelGangerResponse_ValidatorResponse{
				PublicKey:       pubKey,
				DuplicateExists: false,
			},
		}
	}

	// Check if the beacon node if synced.
	isSyncing, err := c.isSyncing(ctx)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get beacon node sync status")
	}

	if isSyncing {
		return nil, errors.New("beacon node not synced")
	}

	// Retrieve fork version -- Return early if we are in phase0.
	forkResponse, err := c.getFork(ctx)
	if err != nil || forkResponse == nil || forkResponse.Data == nil {
		return nil, errors.Wrapf(err, "failed to get fork")
	}

	forkVersionBytes, err := hexutil.Decode(forkResponse.Data.CurrentVersion)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode fork version")
	}

	forkVersion := binary.LittleEndian.Uint32(forkVersionBytes)

	if forkVersion == version.Phase0 {
		log.Info("Skipping doppelganger check for Phase 0")
		return buildResponse(stringPubKeys, stringPubKeyToDoppelGangerInfo), nil
	}

	// Retrieve current epoch.
	headers, err := c.getHeaders(ctx)
	if err != nil || headers == nil || headers.Data == nil || len(headers.Data) == 0 ||
		headers.Data[0].Header == nil || headers.Data[0].Header.Message == nil {
		return nil, errors.Wrapf(err, "failed to get headers")
	}

	headSlotUint64, err := strconv.ParseUint(headers.Data[0].Header.Message.Slot, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse head slot")
	}

	headSlot := primitives.Slot(headSlotUint64)
	currentEpoch := slots.ToEpoch(headSlot)

	// Extract input pubkeys we did not validate for the 2 last epochs.
	// If we detect onchain liveness for these keys during the 2 last epochs, a doppelganger may exist somewhere.
	var notRecentStringPubKeys []string

	for _, spk := range stringPubKeys {
		dph, ok := stringPubKeyToDoppelGangerInfo[spk]
		if !ok {
			return nil, errors.New("failed to retrieve doppelganger info from string public key")
		}

		if dph.validatorEpoch+2 < currentEpoch {
			notRecentStringPubKeys = append(notRecentStringPubKeys, spk)
		}
	}

	// If all provided keys are recent (aka `notRecentPubKeys` is empty) we return early
	// as we are unable to effectively determine if a doppelganger is active.
	if len(notRecentStringPubKeys) == 0 {
		return buildResponse(stringPubKeys, stringPubKeyToDoppelGangerInfo), nil
	}

	// Retrieve correspondence between validator pubkey and index.
	stateValidators, err := c.stateValidatorsProvider.GetStateValidators(ctx, notRecentStringPubKeys, nil, nil)
	if err != nil || stateValidators == nil || stateValidators.Data == nil {
		return nil, errors.Wrapf(err, "failed to get state validators")
	}

	validators := stateValidators.Data
	stringPubKeyToIndex := make(map[string]string, len(validators))
	indexes := make([]string, len(validators))

	for i, v := range validators {
		if v == nil {
			return nil, errors.New("validator container is nil")
		}

		index := v.Index

		if v.Validator == nil {
			return nil, errors.New("validator is nil")
		}

		stringPubKeyToIndex[v.Validator.Pubkey] = index
		indexes[i] = index
	}

	// Get validators liveness for the last epoch.
	// We request a state 1 epoch ago. We are guaranteed to have currentEpoch > 2
	// since we assume that we are not in phase0.
	previousEpoch := currentEpoch - 1

	indexToPreviousLiveness, err := c.getIndexToLiveness(ctx, previousEpoch, indexes)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get map from validator index to liveness for previous epoch %d", previousEpoch)
	}

	// Get validators liveness for the current epoch.
	indexToCurrentLiveness, err := c.getIndexToLiveness(ctx, currentEpoch, indexes)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get map from validator index to liveness for current epoch %d", currentEpoch)
	}

	// Set `DuplicateExists` to `true` if needed.
	for _, spk := range notRecentStringPubKeys {
		index, ok := stringPubKeyToIndex[spk]
		if !ok {
			// if !ok, the validator corresponding to `stringPubKey` does not exist onchain.
			continue
		}

		previousLiveness, ok := indexToPreviousLiveness[index]
		if !ok {
			return nil, fmt.Errorf("failed to retrieve liveness for previous epoch `%d` for validator index `%s`", previousEpoch, index)
		}

		if previousLiveness {
			log.WithField("pubkey", spk).WithField("epoch", previousEpoch).Warn("Doppelganger found")
		}

		currentLiveness, ok := indexToCurrentLiveness[index]
		if !ok {
			return nil, fmt.Errorf("failed to retrieve liveness for current epoch `%d` for validator index `%s`", currentEpoch, index)
		}

		if currentLiveness {
			log.WithField("pubkey", spk).WithField("epoch", currentEpoch).Warn("Doppelganger found")
		}

		globalLiveness := previousLiveness || currentLiveness

		if globalLiveness {
			stringPubKeyToDoppelGangerInfo[spk].response.DuplicateExists = true
		}
	}

	return buildResponse(stringPubKeys, stringPubKeyToDoppelGangerInfo), nil
}

func buildResponse(
	stringPubKeys []string,
	stringPubKeyToDoppelGangerHelper map[string]DoppelGangerInfo,
) *ethpb.DoppelGangerResponse {
	responses := make([]*ethpb.DoppelGangerResponse_ValidatorResponse, len(stringPubKeys))

	for i, spk := range stringPubKeys {
		responses[i] = stringPubKeyToDoppelGangerHelper[spk].response
	}

	return &ethpb.DoppelGangerResponse{
		Responses: responses,
	}
}

func (c *beaconApiValidatorClient) getIndexToLiveness(ctx context.Context, epoch primitives.Epoch, indexes []string) (map[string]bool, error) {
	livenessResponse, err := c.getLiveness(ctx, epoch, indexes)
	if err != nil || livenessResponse.Data == nil {
		return nil, errors.Wrapf(err, fmt.Sprintf("failed to get liveness for epoch %d", epoch))
	}

	indexToLiveness := make(map[string]bool, len(livenessResponse.Data))

	for _, liveness := range livenessResponse.Data {
		if liveness == nil {
			return nil, errors.New("liveness is nil")
		}

		indexToLiveness[liveness.Index] = liveness.IsLive
	}

	return indexToLiveness, nil
}
