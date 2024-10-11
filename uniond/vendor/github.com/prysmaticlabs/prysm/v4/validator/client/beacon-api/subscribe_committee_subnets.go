package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"strconv"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func (c beaconApiValidatorClient) subscribeCommitteeSubnets(ctx context.Context, in *ethpb.CommitteeSubnetsSubscribeRequest, validatorIndices []primitives.ValidatorIndex) error {
	if in == nil {
		return errors.New("committee subnets subscribe request is nil")
	}

	if len(in.CommitteeIds) != len(in.Slots) || len(in.CommitteeIds) != len(in.IsAggregator) || len(in.CommitteeIds) != len(validatorIndices) {
		return errors.New("arrays `in.CommitteeIds`, `in.Slots`, `in.IsAggregator` and `validatorIndices` don't have the same length")
	}

	slotToCommitteesAtSlotMap := make(map[primitives.Slot]uint64)
	jsonCommitteeSubscriptions := make([]*shared.BeaconCommitteeSubscription, len(in.CommitteeIds))
	for index := range in.CommitteeIds {
		subscribeSlot := in.Slots[index]
		subscribeCommitteeId := in.CommitteeIds[index]
		subscribeIsAggregator := in.IsAggregator[index]
		subscribeValidatorIndex := validatorIndices[index]

		committeesAtSlot, foundSlot := slotToCommitteesAtSlotMap[subscribeSlot]
		if !foundSlot {
			// Lazily fetch the committeesAtSlot from the beacon node if they are not already in the map
			epoch := slots.ToEpoch(subscribeSlot)
			duties, err := c.dutiesProvider.GetAttesterDuties(ctx, epoch, validatorIndices)
			if err != nil {
				return errors.Wrapf(err, "failed to get duties for epoch `%d`", epoch)
			}

			for _, duty := range duties {
				dutySlot, err := strconv.ParseUint(duty.Slot, 10, 64)
				if err != nil {
					return errors.Wrapf(err, "failed to parse slot `%s`", duty.Slot)
				}

				committees, err := strconv.ParseUint(duty.CommitteesAtSlot, 10, 64)
				if err != nil {
					return errors.Wrapf(err, "failed to parse CommitteesAtSlot `%s`", duty.CommitteesAtSlot)
				}

				slotToCommitteesAtSlotMap[primitives.Slot(dutySlot)] = committees
			}

			// If the slot still isn't in the map, we either received bad data from the beacon node or the caller of this function gave us bad data
			if committeesAtSlot, foundSlot = slotToCommitteesAtSlotMap[subscribeSlot]; !foundSlot {
				return errors.Errorf("failed to get committees for slot `%d`", subscribeSlot)
			}
		}

		jsonCommitteeSubscriptions[index] = &shared.BeaconCommitteeSubscription{
			CommitteeIndex:   strconv.FormatUint(uint64(subscribeCommitteeId), 10),
			CommitteesAtSlot: strconv.FormatUint(committeesAtSlot, 10),
			Slot:             strconv.FormatUint(uint64(subscribeSlot), 10),
			IsAggregator:     subscribeIsAggregator,
			ValidatorIndex:   strconv.FormatUint(uint64(subscribeValidatorIndex), 10),
		}
	}

	committeeSubscriptionsBytes, err := json.Marshal(jsonCommitteeSubscriptions)
	if err != nil {
		return errors.Wrap(err, "failed to marshal committees subscriptions")
	}

	return c.jsonRestHandler.Post(
		ctx,
		"/eth/v1/validator/beacon_committee_subscriptions",
		nil,
		bytes.NewBuffer(committeeSubscriptionsBytes),
		nil,
	)
}
