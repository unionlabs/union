package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"strconv"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

const subscribeCommitteeSubnetsTestEndpoint = "/eth/v1/validator/beacon_committee_subscriptions"

func TestSubscribeCommitteeSubnets_Valid(t *testing.T) {
	subscribeSlots := []primitives.Slot{0, 1, 100}
	validatorIndices := []primitives.ValidatorIndex{2, 3, 4}
	committeesAtSlot := []uint64{5, 6, 7}
	isAggregator := []bool{false, true, false}
	committeeIndices := []primitives.CommitteeIndex{8, 9, 10}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonCommitteeSubscriptions := make([]*shared.BeaconCommitteeSubscription, len(subscribeSlots))
	for index := range jsonCommitteeSubscriptions {
		jsonCommitteeSubscriptions[index] = &shared.BeaconCommitteeSubscription{
			ValidatorIndex:   strconv.FormatUint(uint64(validatorIndices[index]), 10),
			CommitteeIndex:   strconv.FormatUint(uint64(committeeIndices[index]), 10),
			CommitteesAtSlot: strconv.FormatUint(committeesAtSlot[index], 10),
			Slot:             strconv.FormatUint(uint64(subscribeSlots[index]), 10),
			IsAggregator:     isAggregator[index],
		}
	}

	committeeSubscriptionsBytes, err := json.Marshal(jsonCommitteeSubscriptions)
	require.NoError(t, err)

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		subscribeCommitteeSubnetsTestEndpoint,
		nil,
		bytes.NewBuffer(committeeSubscriptionsBytes),
		nil,
	).Return(
		nil,
	).Times(1)

	duties := make([]*validator.AttesterDuty, len(subscribeSlots))
	for index := range duties {
		duties[index] = &validator.AttesterDuty{
			ValidatorIndex:   strconv.FormatUint(uint64(validatorIndices[index]), 10),
			CommitteeIndex:   strconv.FormatUint(uint64(committeeIndices[index]), 10),
			CommitteesAtSlot: strconv.FormatUint(committeesAtSlot[index], 10),
			Slot:             strconv.FormatUint(uint64(subscribeSlots[index]), 10),
		}
	}

	// Even though we have 3 distinct slots, the first 2 ones are in the same epoch so we should only send 2 requests to the beacon node
	dutiesProvider := mock.NewMockdutiesProvider(ctrl)
	dutiesProvider.EXPECT().GetAttesterDuties(
		ctx,
		slots.ToEpoch(subscribeSlots[0]),
		validatorIndices,
	).Return(
		[]*validator.AttesterDuty{
			{
				CommitteesAtSlot: strconv.FormatUint(committeesAtSlot[0], 10),
				Slot:             strconv.FormatUint(uint64(subscribeSlots[0]), 10),
			},
			{
				CommitteesAtSlot: strconv.FormatUint(committeesAtSlot[1], 10),
				Slot:             strconv.FormatUint(uint64(subscribeSlots[1]), 10),
			},
		},
		nil,
	).Times(1)

	dutiesProvider.EXPECT().GetAttesterDuties(
		ctx,
		slots.ToEpoch(subscribeSlots[2]),
		validatorIndices,
	).Return(
		[]*validator.AttesterDuty{
			{
				CommitteesAtSlot: strconv.FormatUint(committeesAtSlot[2], 10),
				Slot:             strconv.FormatUint(uint64(subscribeSlots[2]), 10),
			},
		},
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{
		jsonRestHandler: jsonRestHandler,
		dutiesProvider:  dutiesProvider,
	}
	err = validatorClient.subscribeCommitteeSubnets(
		ctx,
		&ethpb.CommitteeSubnetsSubscribeRequest{
			Slots:        subscribeSlots,
			CommitteeIds: committeeIndices,
			IsAggregator: isAggregator,
		},
		validatorIndices,
	)
	require.NoError(t, err)
}

func TestSubscribeCommitteeSubnets_Error(t *testing.T) {
	const arraySizeMismatchErrorMessage = "arrays `in.CommitteeIds`, `in.Slots`, `in.IsAggregator` and `validatorIndices` don't have the same length"

	testCases := []struct {
		name                    string
		subscribeRequest        *ethpb.CommitteeSubnetsSubscribeRequest
		validatorIndices        []primitives.ValidatorIndex
		attesterDuty            *validator.AttesterDuty
		dutiesError             error
		expectGetDutiesQuery    bool
		expectSubscribeRestCall bool
		expectedErrorMessage    string
	}{
		{
			name:                 "nil subscribe request",
			subscribeRequest:     nil,
			expectedErrorMessage: "committee subnets subscribe request is nil",
		},
		{
			name: "CommitteeIds size mismatch",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				CommitteeIds: []primitives.CommitteeIndex{1},
				Slots:        []primitives.Slot{1, 2},
				IsAggregator: []bool{false, true},
			},
			validatorIndices:     []primitives.ValidatorIndex{1, 2},
			expectedErrorMessage: arraySizeMismatchErrorMessage,
		},
		{
			name: "Slots size mismatch",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				CommitteeIds: []primitives.CommitteeIndex{1, 2},
				Slots:        []primitives.Slot{1},
				IsAggregator: []bool{false, true},
			},
			validatorIndices:     []primitives.ValidatorIndex{1, 2},
			expectedErrorMessage: arraySizeMismatchErrorMessage,
		},
		{
			name: "IsAggregator size mismatch",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				CommitteeIds: []primitives.CommitteeIndex{1, 2},
				Slots:        []primitives.Slot{1, 2},
				IsAggregator: []bool{false},
			},
			validatorIndices:     []primitives.ValidatorIndex{1, 2},
			expectedErrorMessage: arraySizeMismatchErrorMessage,
		},
		{
			name: "ValidatorIndices size mismatch",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				CommitteeIds: []primitives.CommitteeIndex{1, 2},
				Slots:        []primitives.Slot{1, 2},
				IsAggregator: []bool{false, true},
			},
			validatorIndices:     []primitives.ValidatorIndex{1},
			expectedErrorMessage: arraySizeMismatchErrorMessage,
		},
		{
			name: "bad duties query",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				Slots:        []primitives.Slot{1},
				CommitteeIds: []primitives.CommitteeIndex{2},
				IsAggregator: []bool{false},
			},
			validatorIndices:     []primitives.ValidatorIndex{3},
			dutiesError:          errors.New("foo error"),
			expectGetDutiesQuery: true,
			expectedErrorMessage: "failed to get duties for epoch `0`: foo error",
		},
		{
			name: "bad duty slot",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				Slots:        []primitives.Slot{1},
				CommitteeIds: []primitives.CommitteeIndex{2},
				IsAggregator: []bool{false},
			},
			validatorIndices: []primitives.ValidatorIndex{3},
			attesterDuty: &validator.AttesterDuty{
				Slot:             "foo",
				CommitteesAtSlot: "1",
			},
			expectGetDutiesQuery: true,
			expectedErrorMessage: "failed to parse slot `foo`",
		},
		{
			name: "bad duty committees at slot",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				Slots:        []primitives.Slot{1},
				CommitteeIds: []primitives.CommitteeIndex{2},
				IsAggregator: []bool{false},
			},
			validatorIndices: []primitives.ValidatorIndex{3},
			attesterDuty: &validator.AttesterDuty{
				Slot:             "1",
				CommitteesAtSlot: "foo",
			},
			expectGetDutiesQuery: true,
			expectedErrorMessage: "failed to parse CommitteesAtSlot `foo`",
		},
		{
			name: "missing slot in duties",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				Slots:        []primitives.Slot{1},
				CommitteeIds: []primitives.CommitteeIndex{2},
				IsAggregator: []bool{false},
			},
			validatorIndices: []primitives.ValidatorIndex{3},
			attesterDuty: &validator.AttesterDuty{
				Slot:             "2",
				CommitteesAtSlot: "3",
			},
			expectGetDutiesQuery: true,
			expectedErrorMessage: "failed to get committees for slot `1`",
		},
		{
			name: "bad POST request",
			subscribeRequest: &ethpb.CommitteeSubnetsSubscribeRequest{
				Slots:        []primitives.Slot{1},
				CommitteeIds: []primitives.CommitteeIndex{2},
				IsAggregator: []bool{false},
			},
			validatorIndices: []primitives.ValidatorIndex{3},
			attesterDuty: &validator.AttesterDuty{
				Slot:             "1",
				CommitteesAtSlot: "2",
			},
			expectGetDutiesQuery:    true,
			expectSubscribeRestCall: true,
			expectedErrorMessage:    "foo error",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			ctx := context.Background()

			dutiesProvider := mock.NewMockdutiesProvider(ctrl)
			if testCase.expectGetDutiesQuery {
				dutiesProvider.EXPECT().GetAttesterDuties(
					ctx,
					gomock.Any(),
					gomock.Any(),
				).Return(
					[]*validator.AttesterDuty{testCase.attesterDuty},
					testCase.dutiesError,
				).Times(1)
			}

			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			if testCase.expectSubscribeRestCall {
				jsonRestHandler.EXPECT().Post(
					ctx,
					subscribeCommitteeSubnetsTestEndpoint,
					gomock.Any(),
					gomock.Any(),
					gomock.Any(),
				).Return(
					errors.New("foo error"),
				).Times(1)
			}

			validatorClient := &beaconApiValidatorClient{
				jsonRestHandler: jsonRestHandler,
				dutiesProvider:  dutiesProvider,
			}
			err := validatorClient.subscribeCommitteeSubnets(ctx, testCase.subscribeRequest, testCase.validatorIndices)
			assert.ErrorContains(t, testCase.expectedErrorMessage, err)
		})
	}
}
