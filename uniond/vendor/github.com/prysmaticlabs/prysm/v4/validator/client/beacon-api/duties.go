package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

type dutiesProvider interface {
	GetAttesterDuties(ctx context.Context, epoch primitives.Epoch, validatorIndices []primitives.ValidatorIndex) ([]*validator.AttesterDuty, error)
	GetProposerDuties(ctx context.Context, epoch primitives.Epoch) ([]*validator.ProposerDuty, error)
	GetSyncDuties(ctx context.Context, epoch primitives.Epoch, validatorIndices []primitives.ValidatorIndex) ([]*validator.SyncCommitteeDuty, error)
	GetCommittees(ctx context.Context, epoch primitives.Epoch) ([]*shared.Committee, error)
}

type beaconApiDutiesProvider struct {
	jsonRestHandler JsonRestHandler
}

type committeeIndexSlotPair struct {
	committeeIndex primitives.CommitteeIndex
	slot           primitives.Slot
}

func (c beaconApiValidatorClient) getDuties(ctx context.Context, in *ethpb.DutiesRequest) (*ethpb.DutiesResponse, error) {
	all, err := c.multipleValidatorStatus(ctx, &ethpb.MultipleValidatorStatusRequest{PublicKeys: in.PublicKeys})
	if err != nil {
		return nil, errors.Wrap(err, "failed to get validator status")
	}
	known := &ethpb.MultipleValidatorStatusResponse{
		PublicKeys: make([][]byte, 0, len(all.PublicKeys)),
		Statuses:   make([]*ethpb.ValidatorStatusResponse, 0, len(all.Statuses)),
		Indices:    make([]primitives.ValidatorIndex, 0, len(all.Indices)),
	}
	for i, status := range all.Statuses {
		if status.Status != ethpb.ValidatorStatus_UNKNOWN_STATUS {
			known.PublicKeys = append(known.PublicKeys, all.PublicKeys[i])
			known.Statuses = append(known.Statuses, all.Statuses[i])
			known.Indices = append(known.Indices, all.Indices[i])
		}
	}

	// Sync committees are an Altair feature
	fetchSyncDuties := in.Epoch >= params.BeaconConfig().AltairForkEpoch

	currentEpochDuties, err := c.getDutiesForEpoch(ctx, in.Epoch, known, fetchSyncDuties)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get duties for current epoch `%d`", in.Epoch)
	}

	nextEpochDuties, err := c.getDutiesForEpoch(ctx, in.Epoch+1, known, fetchSyncDuties)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get duties for next epoch `%d`", in.Epoch+1)
	}

	return &ethpb.DutiesResponse{
		Duties:             currentEpochDuties,
		CurrentEpochDuties: currentEpochDuties,
		NextEpochDuties:    nextEpochDuties,
	}, nil
}

func (c beaconApiValidatorClient) getDutiesForEpoch(
	ctx context.Context,
	epoch primitives.Epoch,
	multipleValidatorStatus *ethpb.MultipleValidatorStatusResponse,
	fetchSyncDuties bool,
) ([]*ethpb.DutiesResponse_Duty, error) {
	attesterDuties, err := c.dutiesProvider.GetAttesterDuties(ctx, epoch, multipleValidatorStatus.Indices)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get attester duties for epoch `%d`", epoch)
	}

	var syncDuties []*validator.SyncCommitteeDuty
	if fetchSyncDuties {
		if syncDuties, err = c.dutiesProvider.GetSyncDuties(ctx, epoch, multipleValidatorStatus.Indices); err != nil {
			return nil, errors.Wrapf(err, "failed to get sync duties for epoch `%d`", epoch)
		}
	}

	var proposerDuties []*validator.ProposerDuty
	if proposerDuties, err = c.dutiesProvider.GetProposerDuties(ctx, epoch); err != nil {
		return nil, errors.Wrapf(err, "failed to get proposer duties for epoch `%d`", epoch)
	}

	committees, err := c.dutiesProvider.GetCommittees(ctx, epoch)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get committees for epoch `%d`", epoch)
	}

	// Mapping from a validator index to its attesting committee's index and slot
	attesterDutiesMapping := make(map[primitives.ValidatorIndex]committeeIndexSlotPair)
	for _, attesterDuty := range attesterDuties {
		validatorIndex, err := strconv.ParseUint(attesterDuty.ValidatorIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse attester validator index `%s`", attesterDuty.ValidatorIndex)
		}

		slot, err := strconv.ParseUint(attesterDuty.Slot, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse attester slot `%s`", attesterDuty.Slot)
		}

		committeeIndex, err := strconv.ParseUint(attesterDuty.CommitteeIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse attester committee index `%s`", attesterDuty.CommitteeIndex)
		}

		attesterDutiesMapping[primitives.ValidatorIndex(validatorIndex)] = committeeIndexSlotPair{
			slot:           primitives.Slot(slot),
			committeeIndex: primitives.CommitteeIndex(committeeIndex),
		}
	}

	// Mapping from a validator index to its proposal slot
	proposerDutySlots := make(map[primitives.ValidatorIndex][]primitives.Slot)
	for _, proposerDuty := range proposerDuties {
		validatorIndex, err := strconv.ParseUint(proposerDuty.ValidatorIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse proposer validator index `%s`", proposerDuty.ValidatorIndex)
		}

		slot, err := strconv.ParseUint(proposerDuty.Slot, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse proposer slot `%s`", proposerDuty.Slot)
		}

		proposerDutySlots[primitives.ValidatorIndex(validatorIndex)] = append(proposerDutySlots[primitives.ValidatorIndex(validatorIndex)], primitives.Slot(slot))
	}

	// Set containing all validator indices that are part of a sync committee for this epoch
	syncDutiesMapping := make(map[primitives.ValidatorIndex]bool)
	for _, syncDuty := range syncDuties {
		validatorIndex, err := strconv.ParseUint(syncDuty.ValidatorIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse sync validator index `%s`", syncDuty.ValidatorIndex)
		}

		syncDutiesMapping[primitives.ValidatorIndex(validatorIndex)] = true
	}

	// Mapping from the {committeeIndex, slot} to each of the committee's validator indices
	committeeMapping := make(map[committeeIndexSlotPair][]primitives.ValidatorIndex)
	for _, committee := range committees {
		committeeIndex, err := strconv.ParseUint(committee.Index, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse committee index `%s`", committee.Index)
		}

		slot, err := strconv.ParseUint(committee.Slot, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse slot `%s`", committee.Slot)
		}

		validatorIndices := make([]primitives.ValidatorIndex, len(committee.Validators))
		for index, validatorIndexString := range committee.Validators {
			validatorIndex, err := strconv.ParseUint(validatorIndexString, 10, 64)
			if err != nil {
				return nil, errors.Wrapf(err, "failed to parse committee validator index `%s`", validatorIndexString)
			}
			validatorIndices[index] = primitives.ValidatorIndex(validatorIndex)
		}

		key := committeeIndexSlotPair{
			committeeIndex: primitives.CommitteeIndex(committeeIndex),
			slot:           primitives.Slot(slot),
		}
		committeeMapping[key] = validatorIndices
	}

	duties := make([]*ethpb.DutiesResponse_Duty, len(multipleValidatorStatus.Statuses))
	for index, validatorStatus := range multipleValidatorStatus.Statuses {
		validatorIndex := multipleValidatorStatus.Indices[index]
		pubkey := multipleValidatorStatus.PublicKeys[index]

		var attesterSlot primitives.Slot
		var committeeIndex primitives.CommitteeIndex
		var committeeValidatorIndices []primitives.ValidatorIndex

		if committeeMappingKey, ok := attesterDutiesMapping[validatorIndex]; ok {
			committeeIndex = committeeMappingKey.committeeIndex
			attesterSlot = committeeMappingKey.slot

			if committeeValidatorIndices, ok = committeeMapping[committeeMappingKey]; !ok {
				return nil, errors.Errorf("failed to find validators for committee index `%d` and slot `%d`", committeeIndex, attesterSlot)
			}
		}

		duties[index] = &ethpb.DutiesResponse_Duty{
			Committee:       committeeValidatorIndices,
			CommitteeIndex:  committeeIndex,
			AttesterSlot:    attesterSlot,
			ProposerSlots:   proposerDutySlots[validatorIndex],
			PublicKey:       pubkey,
			Status:          validatorStatus.Status,
			ValidatorIndex:  validatorIndex,
			IsSyncCommittee: syncDutiesMapping[validatorIndex],
		}
	}

	return duties, nil
}

// GetCommittees retrieves the committees for the given epoch
func (c beaconApiDutiesProvider) GetCommittees(ctx context.Context, epoch primitives.Epoch) ([]*shared.Committee, error) {
	committeeParams := url.Values{}
	committeeParams.Add("epoch", strconv.FormatUint(uint64(epoch), 10))
	committeesRequest := buildURL("/eth/v1/beacon/states/head/committees", committeeParams)

	var stateCommittees beacon.GetCommitteesResponse
	if err := c.jsonRestHandler.Get(ctx, committeesRequest, &stateCommittees); err != nil {
		return nil, err
	}

	if stateCommittees.Data == nil {
		return nil, errors.New("state committees data is nil")
	}

	for index, committee := range stateCommittees.Data {
		if committee == nil {
			return nil, errors.Errorf("committee at index `%d` is nil", index)
		}
	}

	return stateCommittees.Data, nil
}

// GetAttesterDuties retrieves the attester duties for the given epoch and validatorIndices
func (c beaconApiDutiesProvider) GetAttesterDuties(ctx context.Context, epoch primitives.Epoch, validatorIndices []primitives.ValidatorIndex) ([]*validator.AttesterDuty, error) {
	jsonValidatorIndices := make([]string, len(validatorIndices))
	for index, validatorIndex := range validatorIndices {
		jsonValidatorIndices[index] = strconv.FormatUint(uint64(validatorIndex), 10)
	}

	validatorIndicesBytes, err := json.Marshal(jsonValidatorIndices)
	if err != nil {
		return nil, errors.Wrap(err, "failed to marshal validator indices")
	}

	attesterDuties := &validator.GetAttesterDutiesResponse{}
	if err = c.jsonRestHandler.Post(
		ctx,
		fmt.Sprintf("/eth/v1/validator/duties/attester/%d", epoch),
		nil,
		bytes.NewBuffer(validatorIndicesBytes),
		attesterDuties,
	); err != nil {
		return nil, err
	}

	for index, attesterDuty := range attesterDuties.Data {
		if attesterDuty == nil {
			return nil, errors.Errorf("attester duty at index `%d` is nil", index)
		}
	}

	return attesterDuties.Data, nil
}

// GetProposerDuties retrieves the proposer duties for the given epoch
func (c beaconApiDutiesProvider) GetProposerDuties(ctx context.Context, epoch primitives.Epoch) ([]*validator.ProposerDuty, error) {
	proposerDuties := validator.GetProposerDutiesResponse{}
	if err := c.jsonRestHandler.Get(ctx, fmt.Sprintf("/eth/v1/validator/duties/proposer/%d", epoch), &proposerDuties); err != nil {
		return nil, err
	}

	if proposerDuties.Data == nil {
		return nil, errors.New("proposer duties data is nil")
	}

	for index, proposerDuty := range proposerDuties.Data {
		if proposerDuty == nil {
			return nil, errors.Errorf("proposer duty at index `%d` is nil", index)
		}
	}

	return proposerDuties.Data, nil
}

// GetSyncDuties retrieves the sync committee duties for the given epoch and validatorIndices
func (c beaconApiDutiesProvider) GetSyncDuties(ctx context.Context, epoch primitives.Epoch, validatorIndices []primitives.ValidatorIndex) ([]*validator.SyncCommitteeDuty, error) {
	jsonValidatorIndices := make([]string, len(validatorIndices))
	for index, validatorIndex := range validatorIndices {
		jsonValidatorIndices[index] = strconv.FormatUint(uint64(validatorIndex), 10)
	}

	validatorIndicesBytes, err := json.Marshal(jsonValidatorIndices)
	if err != nil {
		return nil, errors.Wrap(err, "failed to marshal validator indices")
	}

	syncDuties := validator.GetSyncCommitteeDutiesResponse{}
	if err = c.jsonRestHandler.Post(
		ctx,
		fmt.Sprintf("/eth/v1/validator/duties/sync/%d", epoch),
		nil,
		bytes.NewBuffer(validatorIndicesBytes),
		&syncDuties,
	); err != nil {
		return nil, err
	}

	if syncDuties.Data == nil {
		return nil, errors.New("sync duties data is nil")
	}

	for index, syncDuty := range syncDuties.Data {
		if syncDuty == nil {
			return nil, errors.Errorf("sync duty at index `%d` is nil", index)
		}
	}

	return syncDuties.Data, nil
}
