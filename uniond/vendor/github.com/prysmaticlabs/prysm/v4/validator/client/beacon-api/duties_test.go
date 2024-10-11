package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"testing"

	validatormock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

const getAttesterDutiesTestEndpoint = "/eth/v1/validator/duties/attester"
const getProposerDutiesTestEndpoint = "/eth/v1/validator/duties/proposer"
const getSyncDutiesTestEndpoint = "/eth/v1/validator/duties/sync"
const getCommitteesTestEndpoint = "/eth/v1/beacon/states/head/committees"

func TestGetAttesterDuties_Valid(t *testing.T) {
	stringValidatorIndices := []string{"2", "9"}
	const epoch = primitives.Epoch(1)

	validatorIndicesBytes, err := json.Marshal(stringValidatorIndices)
	require.NoError(t, err)

	expectedAttesterDuties := validator.GetAttesterDutiesResponse{
		Data: []*validator.AttesterDuty{
			{
				Pubkey:                  hexutil.Encode([]byte{1}),
				ValidatorIndex:          "2",
				CommitteeIndex:          "3",
				CommitteeLength:         "4",
				CommitteesAtSlot:        "5",
				ValidatorCommitteeIndex: "6",
				Slot:                    "7",
			},
			{
				Pubkey:                  hexutil.Encode([]byte{8}),
				ValidatorIndex:          "9",
				CommitteeIndex:          "10",
				CommitteeLength:         "11",
				CommitteesAtSlot:        "12",
				ValidatorCommitteeIndex: "13",
				Slot:                    "14",
			},
		},
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	validatorIndices := []primitives.ValidatorIndex{2, 9}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getAttesterDutiesTestEndpoint, epoch),
		nil,
		bytes.NewBuffer(validatorIndicesBytes),
		&validator.GetAttesterDutiesResponse{},
	).Return(
		nil,
	).SetArg(
		4,
		expectedAttesterDuties,
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	attesterDuties, err := dutiesProvider.GetAttesterDuties(ctx, epoch, validatorIndices)
	require.NoError(t, err)
	assert.DeepEqual(t, expectedAttesterDuties.Data, attesterDuties)
}

func TestGetAttesterDuties_HttpError(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getAttesterDutiesTestEndpoint, epoch),
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		errors.New("foo error"),
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetAttesterDuties(ctx, epoch, nil)
	assert.ErrorContains(t, "foo error", err)
}

func TestGetAttesterDuties_NilAttesterDuty(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getAttesterDutiesTestEndpoint, epoch),
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		4,
		validator.GetAttesterDutiesResponse{
			Data: []*validator.AttesterDuty{nil},
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetAttesterDuties(ctx, epoch, nil)
	assert.ErrorContains(t, "attester duty at index `0` is nil", err)
}

func TestGetProposerDuties_Valid(t *testing.T) {
	const epoch = primitives.Epoch(1)

	expectedProposerDuties := validator.GetProposerDutiesResponse{
		Data: []*validator.ProposerDuty{
			{
				Pubkey:         hexutil.Encode([]byte{1}),
				ValidatorIndex: "2",
				Slot:           "3",
			},
			{
				Pubkey:         hexutil.Encode([]byte{4}),
				ValidatorIndex: "5",
				Slot:           "6",
			},
		},
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s/%d", getProposerDutiesTestEndpoint, epoch),
		&validator.GetProposerDutiesResponse{},
	).Return(
		nil,
	).SetArg(
		2,
		expectedProposerDuties,
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	proposerDuties, err := dutiesProvider.GetProposerDuties(ctx, epoch)
	require.NoError(t, err)
	assert.DeepEqual(t, expectedProposerDuties.Data, proposerDuties)
}

func TestGetProposerDuties_HttpError(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s/%d", getProposerDutiesTestEndpoint, epoch),
		gomock.Any(),
	).Return(
		errors.New("foo error"),
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetProposerDuties(ctx, epoch)
	assert.ErrorContains(t, "foo error", err)
}

func TestGetProposerDuties_NilData(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s/%d", getProposerDutiesTestEndpoint, epoch),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		2,
		validator.GetProposerDutiesResponse{
			Data: nil,
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetProposerDuties(ctx, epoch)
	assert.ErrorContains(t, "proposer duties data is nil", err)
}

func TestGetProposerDuties_NilProposerDuty(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s/%d", getProposerDutiesTestEndpoint, epoch),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		2,
		validator.GetProposerDutiesResponse{
			Data: []*validator.ProposerDuty{nil},
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetProposerDuties(ctx, epoch)
	assert.ErrorContains(t, "proposer duty at index `0` is nil", err)
}

func TestGetSyncDuties_Valid(t *testing.T) {
	stringValidatorIndices := []string{"2", "6"}
	const epoch = primitives.Epoch(1)

	validatorIndicesBytes, err := json.Marshal(stringValidatorIndices)
	require.NoError(t, err)

	expectedSyncDuties := validator.GetSyncCommitteeDutiesResponse{
		Data: []*validator.SyncCommitteeDuty{
			{
				Pubkey:         hexutil.Encode([]byte{1}),
				ValidatorIndex: "2",
				ValidatorSyncCommitteeIndices: []string{
					"3",
					"4",
				},
			},
			{
				Pubkey:         hexutil.Encode([]byte{5}),
				ValidatorIndex: "6",
				ValidatorSyncCommitteeIndices: []string{
					"7",
					"8",
				},
			},
		},
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	validatorIndices := []primitives.ValidatorIndex{2, 6}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getSyncDutiesTestEndpoint, epoch),
		nil,
		bytes.NewBuffer(validatorIndicesBytes),
		&validator.GetSyncCommitteeDutiesResponse{},
	).Return(
		nil,
	).SetArg(
		4,
		expectedSyncDuties,
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	syncDuties, err := dutiesProvider.GetSyncDuties(ctx, epoch, validatorIndices)
	require.NoError(t, err)
	assert.DeepEqual(t, expectedSyncDuties.Data, syncDuties)
}

func TestGetSyncDuties_HttpError(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getSyncDutiesTestEndpoint, epoch),
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		errors.New("foo error"),
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetSyncDuties(ctx, epoch, nil)
	assert.ErrorContains(t, "foo error", err)
}

func TestGetSyncDuties_NilData(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getSyncDutiesTestEndpoint, epoch),
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		4,
		validator.GetSyncCommitteeDutiesResponse{
			Data: nil,
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetSyncDuties(ctx, epoch, nil)
	assert.ErrorContains(t, "sync duties data is nil", err)
}

func TestGetSyncDuties_NilSyncDuty(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		fmt.Sprintf("%s/%d", getSyncDutiesTestEndpoint, epoch),
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		4,
		validator.GetSyncCommitteeDutiesResponse{
			Data: []*validator.SyncCommitteeDuty{nil},
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetSyncDuties(ctx, epoch, nil)
	assert.ErrorContains(t, "sync duty at index `0` is nil", err)
}

func TestGetCommittees_Valid(t *testing.T) {
	const epoch = primitives.Epoch(1)

	expectedCommittees := beacon.GetCommitteesResponse{
		Data: []*shared.Committee{
			{
				Index: "1",
				Slot:  "2",
				Validators: []string{
					"3",
					"4",
				},
			},
			{
				Index: "5",
				Slot:  "6",
				Validators: []string{
					"7",
					"8",
				},
			},
		},
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s?epoch=%d", getCommitteesTestEndpoint, epoch),
		&beacon.GetCommitteesResponse{},
	).Return(
		nil,
	).SetArg(
		2,
		expectedCommittees,
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	committees, err := dutiesProvider.GetCommittees(ctx, epoch)
	require.NoError(t, err)
	assert.DeepEqual(t, expectedCommittees.Data, committees)
}

func TestGetCommittees_HttpError(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s?epoch=%d", getCommitteesTestEndpoint, epoch),
		gomock.Any(),
	).Return(
		errors.New("foo error"),
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetCommittees(ctx, epoch)
	assert.ErrorContains(t, "foo error", err)
}

func TestGetCommittees_NilData(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s?epoch=%d", getCommitteesTestEndpoint, epoch),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		2,
		beacon.GetCommitteesResponse{
			Data: nil,
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetCommittees(ctx, epoch)
	assert.ErrorContains(t, "state committees data is nil", err)
}

func TestGetCommittees_NilCommittee(t *testing.T) {
	const epoch = primitives.Epoch(1)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("%s?epoch=%d", getCommitteesTestEndpoint, epoch),
		gomock.Any(),
	).Return(
		nil,
	).SetArg(
		2,
		beacon.GetCommitteesResponse{
			Data: []*shared.Committee{nil},
		},
	).Times(1)

	dutiesProvider := &beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler}
	_, err := dutiesProvider.GetCommittees(ctx, epoch)
	assert.ErrorContains(t, "committee at index `0` is nil", err)
}

func TestGetDutiesForEpoch_Error(t *testing.T) {
	const epoch = primitives.Epoch(1)
	pubkeys := [][]byte{{1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}, {10}, {11}, {12}}
	validatorIndices := []primitives.ValidatorIndex{13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24}
	committeeIndices := []primitives.CommitteeIndex{25, 26, 27}
	committeeSlots := []primitives.Slot{28, 29, 30}
	proposerSlots := []primitives.Slot{31, 32, 33, 34, 35, 36, 37, 38}

	testCases := []struct {
		name                     string
		expectedError            string
		generateAttesterDuties   func() []*validator.AttesterDuty
		fetchAttesterDutiesError error
		generateProposerDuties   func() []*validator.ProposerDuty
		fetchProposerDutiesError error
		generateSyncDuties       func() []*validator.SyncCommitteeDuty
		fetchSyncDutiesError     error
		generateCommittees       func() []*shared.Committee
		fetchCommitteesError     error
	}{
		{
			name:                     "get attester duties failed",
			expectedError:            "failed to get attester duties for epoch `1`: foo error",
			fetchAttesterDutiesError: errors.New("foo error"),
		},
		{
			name:                     "get proposer duties failed",
			expectedError:            "failed to get proposer duties for epoch `1`: foo error",
			fetchAttesterDutiesError: nil,
			fetchProposerDutiesError: errors.New("foo error"),
		},
		{
			name:                 "get sync duties failed",
			expectedError:        "failed to get sync duties for epoch `1`: foo error",
			fetchSyncDutiesError: errors.New("foo error"),
		},
		{
			name:                 "get committees failed",
			expectedError:        "failed to get committees for epoch `1`: foo error",
			fetchCommitteesError: errors.New("foo error"),
		},
		{
			name:          "bad attester validator index",
			expectedError: "failed to parse attester validator index `foo`",
			generateAttesterDuties: func() []*validator.AttesterDuty {
				attesterDuties := generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots)
				attesterDuties[0].ValidatorIndex = "foo"
				return attesterDuties
			},
		},
		{
			name:          "bad attester slot",
			expectedError: "failed to parse attester slot `foo`",
			generateAttesterDuties: func() []*validator.AttesterDuty {
				attesterDuties := generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots)
				attesterDuties[0].Slot = "foo"
				return attesterDuties
			},
		},
		{
			name:          "bad attester committee index",
			expectedError: "failed to parse attester committee index `foo`",
			generateAttesterDuties: func() []*validator.AttesterDuty {
				attesterDuties := generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots)
				attesterDuties[0].CommitteeIndex = "foo"
				return attesterDuties
			},
		},
		{
			name:          "bad proposer validator index",
			expectedError: "failed to parse proposer validator index `foo`",
			generateProposerDuties: func() []*validator.ProposerDuty {
				proposerDuties := generateValidProposerDuties(pubkeys, validatorIndices, proposerSlots)
				proposerDuties[0].ValidatorIndex = "foo"
				return proposerDuties
			},
		},
		{
			name:          "bad proposer slot",
			expectedError: "failed to parse proposer slot `foo`",
			generateProposerDuties: func() []*validator.ProposerDuty {
				proposerDuties := generateValidProposerDuties(pubkeys, validatorIndices, proposerSlots)
				proposerDuties[0].Slot = "foo"
				return proposerDuties
			},
		},
		{
			name:          "bad sync validator index",
			expectedError: "failed to parse sync validator index `foo`",
			generateSyncDuties: func() []*validator.SyncCommitteeDuty {
				syncDuties := generateValidSyncDuties(pubkeys, validatorIndices)
				syncDuties[0].ValidatorIndex = "foo"
				return syncDuties
			},
		},
		{
			name:          "bad committee index",
			expectedError: "failed to parse committee index `foo`",
			generateCommittees: func() []*shared.Committee {
				committees := generateValidCommittees(committeeIndices, committeeSlots, validatorIndices)
				committees[0].Index = "foo"
				return committees
			},
		},
		{
			name:          "bad committee slot",
			expectedError: "failed to parse slot `foo`",
			generateCommittees: func() []*shared.Committee {
				committees := generateValidCommittees(committeeIndices, committeeSlots, validatorIndices)
				committees[0].Slot = "foo"
				return committees
			},
		},
		{
			name:          "bad committee validator index",
			expectedError: "failed to parse committee validator index `foo`",
			generateCommittees: func() []*shared.Committee {
				committees := generateValidCommittees(committeeIndices, committeeSlots, validatorIndices)
				committees[0].Validators[0] = "foo"
				return committees
			},
		},
		{
			name:          "committee index and slot not found in committees mapping",
			expectedError: "failed to find validators for committee index `1` and slot `2`",
			generateAttesterDuties: func() []*validator.AttesterDuty {
				attesterDuties := generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots)
				attesterDuties[0].CommitteeIndex = "1"
				attesterDuties[0].Slot = "2"
				return attesterDuties
			},
			generateCommittees: func() []*shared.Committee {
				return []*shared.Committee{}
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			ctx := context.Background()

			var attesterDuties []*validator.AttesterDuty
			if testCase.generateAttesterDuties == nil {
				attesterDuties = generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots)
			} else {
				attesterDuties = testCase.generateAttesterDuties()
			}

			var proposerDuties []*validator.ProposerDuty
			if testCase.generateProposerDuties == nil {
				proposerDuties = generateValidProposerDuties(pubkeys, validatorIndices, proposerSlots)
			} else {
				proposerDuties = testCase.generateProposerDuties()
			}

			var syncDuties []*validator.SyncCommitteeDuty
			if testCase.generateSyncDuties == nil {
				syncDuties = generateValidSyncDuties(pubkeys, validatorIndices)
			} else {
				syncDuties = testCase.generateSyncDuties()
			}

			var committees []*shared.Committee
			if testCase.generateCommittees == nil {
				committees = generateValidCommittees(committeeIndices, committeeSlots, validatorIndices)
			} else {
				committees = testCase.generateCommittees()
			}

			dutiesProvider := mock.NewMockdutiesProvider(ctrl)
			dutiesProvider.EXPECT().GetAttesterDuties(
				ctx,
				epoch,
				gomock.Any(),
			).Return(
				attesterDuties,
				testCase.fetchAttesterDutiesError,
			).AnyTimes()

			dutiesProvider.EXPECT().GetProposerDuties(
				ctx,
				epoch,
			).Return(
				proposerDuties,
				testCase.fetchProposerDutiesError,
			).AnyTimes()

			dutiesProvider.EXPECT().GetSyncDuties(
				ctx,
				epoch,
				gomock.Any(),
			).Return(
				syncDuties,
				testCase.fetchSyncDutiesError,
			).AnyTimes()

			dutiesProvider.EXPECT().GetCommittees(
				ctx,
				epoch,
			).Return(
				committees,
				testCase.fetchCommitteesError,
			).AnyTimes()

			validatorClient := &beaconApiValidatorClient{dutiesProvider: dutiesProvider}
			_, err := validatorClient.getDutiesForEpoch(
				ctx,
				epoch,
				&ethpb.MultipleValidatorStatusResponse{
					PublicKeys: pubkeys,
					Indices:    validatorIndices,
					Statuses: []*ethpb.ValidatorStatusResponse{
						{Status: ethpb.ValidatorStatus_UNKNOWN_STATUS},
						{Status: ethpb.ValidatorStatus_DEPOSITED},
						{Status: ethpb.ValidatorStatus_PENDING},
						{Status: ethpb.ValidatorStatus_ACTIVE},
						{Status: ethpb.ValidatorStatus_EXITING},
						{Status: ethpb.ValidatorStatus_SLASHING},
						{Status: ethpb.ValidatorStatus_EXITED},
						{Status: ethpb.ValidatorStatus_INVALID},
						{Status: ethpb.ValidatorStatus_PARTIALLY_DEPOSITED},
						{Status: ethpb.ValidatorStatus_UNKNOWN_STATUS},
						{Status: ethpb.ValidatorStatus_DEPOSITED},
						{Status: ethpb.ValidatorStatus_PENDING},
					},
				},
				true,
			)
			assert.ErrorContains(t, testCase.expectedError, err)
		})
	}
}

func TestGetDutiesForEpoch_Valid(t *testing.T) {
	testCases := []struct {
		name            string
		fetchSyncDuties bool
	}{
		{
			name:            "fetch attester and proposer duties",
			fetchSyncDuties: false,
		},
		{
			name:            "fetch attester and sync and proposer duties",
			fetchSyncDuties: true,
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			const epoch = primitives.Epoch(1)
			pubkeys := [][]byte{{1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}, {10}, {11}, {12}}
			validatorIndices := []primitives.ValidatorIndex{13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24}
			committeeIndices := []primitives.CommitteeIndex{25, 26, 27}
			committeeSlots := []primitives.Slot{28, 29, 30}
			proposerSlots := []primitives.Slot{31, 32, 33, 34, 35, 36, 37, 38}

			statuses := []ethpb.ValidatorStatus{
				ethpb.ValidatorStatus_UNKNOWN_STATUS,
				ethpb.ValidatorStatus_DEPOSITED,
				ethpb.ValidatorStatus_PENDING,
				ethpb.ValidatorStatus_ACTIVE,
				ethpb.ValidatorStatus_EXITING,
				ethpb.ValidatorStatus_SLASHING,
				ethpb.ValidatorStatus_EXITED,
				ethpb.ValidatorStatus_INVALID,
				ethpb.ValidatorStatus_PARTIALLY_DEPOSITED,
				ethpb.ValidatorStatus_UNKNOWN_STATUS,
				ethpb.ValidatorStatus_DEPOSITED,
				ethpb.ValidatorStatus_PENDING,
			}

			multipleValidatorStatus := &ethpb.MultipleValidatorStatusResponse{
				PublicKeys: pubkeys,
				Indices:    validatorIndices,
				Statuses: []*ethpb.ValidatorStatusResponse{
					{Status: statuses[0]},
					{Status: statuses[1]},
					{Status: statuses[2]},
					{Status: statuses[3]},
					{Status: statuses[4]},
					{Status: statuses[5]},
					{Status: statuses[6]},
					{Status: statuses[7]},
					{Status: statuses[8]},
					{Status: statuses[9]},
					{Status: statuses[10]},
					{Status: statuses[11]},
				},
			}

			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			ctx := context.Background()

			dutiesProvider := mock.NewMockdutiesProvider(ctrl)
			dutiesProvider.EXPECT().GetCommittees(
				ctx,
				epoch,
			).Return(
				generateValidCommittees(committeeIndices, committeeSlots, validatorIndices),
				nil,
			).Times(1)

			dutiesProvider.EXPECT().GetAttesterDuties(
				ctx,
				epoch,
				multipleValidatorStatus.Indices,
			).Return(
				generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots),
				nil,
			).Times(1)

			dutiesProvider.EXPECT().GetProposerDuties(
				ctx,
				epoch,
			).Return(
				generateValidProposerDuties(pubkeys, validatorIndices, proposerSlots),
				nil,
			).Times(1)

			if testCase.fetchSyncDuties {
				dutiesProvider.EXPECT().GetSyncDuties(
					ctx,
					epoch,
					multipleValidatorStatus.Indices,
				).Return(
					generateValidSyncDuties(pubkeys, validatorIndices),
					nil,
				).Times(1)
			}

			var expectedProposerSlots1 []primitives.Slot
			var expectedProposerSlots2 []primitives.Slot
			var expectedProposerSlots3 []primitives.Slot
			var expectedProposerSlots4 []primitives.Slot

			expectedProposerSlots1 = []primitives.Slot{
				proposerSlots[0],
				proposerSlots[1],
			}

			expectedProposerSlots2 = []primitives.Slot{
				proposerSlots[2],
				proposerSlots[3],
			}

			expectedProposerSlots3 = []primitives.Slot{
				proposerSlots[4],
				proposerSlots[5],
			}

			expectedProposerSlots4 = []primitives.Slot{
				proposerSlots[6],
				proposerSlots[7],
			}

			expectedDuties := []*ethpb.DutiesResponse_Duty{
				{
					Committee: []primitives.ValidatorIndex{
						validatorIndices[0],
						validatorIndices[1],
					},
					CommitteeIndex: committeeIndices[0],
					AttesterSlot:   committeeSlots[0],
					PublicKey:      pubkeys[0],
					Status:         statuses[0],
					ValidatorIndex: validatorIndices[0],
				},
				{
					Committee: []primitives.ValidatorIndex{
						validatorIndices[0],
						validatorIndices[1],
					},
					CommitteeIndex: committeeIndices[0],
					AttesterSlot:   committeeSlots[0],
					PublicKey:      pubkeys[1],
					Status:         statuses[1],
					ValidatorIndex: validatorIndices[1],
				},
				{
					Committee: []primitives.ValidatorIndex{
						validatorIndices[2],
						validatorIndices[3],
					},
					CommitteeIndex: committeeIndices[1],
					AttesterSlot:   committeeSlots[1],
					PublicKey:      pubkeys[2],
					Status:         statuses[2],
					ValidatorIndex: validatorIndices[2],
				},
				{
					Committee: []primitives.ValidatorIndex{
						validatorIndices[2],
						validatorIndices[3],
					},
					CommitteeIndex: committeeIndices[1],
					AttesterSlot:   committeeSlots[1],
					PublicKey:      pubkeys[3],
					Status:         statuses[3],
					ValidatorIndex: validatorIndices[3],
				},
				{
					Committee: []primitives.ValidatorIndex{
						validatorIndices[4],
						validatorIndices[5],
					},
					CommitteeIndex: committeeIndices[2],
					AttesterSlot:   committeeSlots[2],
					PublicKey:      pubkeys[4],
					Status:         statuses[4],
					ValidatorIndex: validatorIndices[4],
					ProposerSlots:  expectedProposerSlots1,
				},
				{
					Committee: []primitives.ValidatorIndex{
						validatorIndices[4],
						validatorIndices[5],
					},
					CommitteeIndex:  committeeIndices[2],
					AttesterSlot:    committeeSlots[2],
					PublicKey:       pubkeys[5],
					Status:          statuses[5],
					ValidatorIndex:  validatorIndices[5],
					ProposerSlots:   expectedProposerSlots2,
					IsSyncCommittee: testCase.fetchSyncDuties,
				},
				{
					PublicKey:       pubkeys[6],
					Status:          statuses[6],
					ValidatorIndex:  validatorIndices[6],
					ProposerSlots:   expectedProposerSlots3,
					IsSyncCommittee: testCase.fetchSyncDuties,
				},
				{
					PublicKey:       pubkeys[7],
					Status:          statuses[7],
					ValidatorIndex:  validatorIndices[7],
					ProposerSlots:   expectedProposerSlots4,
					IsSyncCommittee: testCase.fetchSyncDuties,
				},
				{
					PublicKey:       pubkeys[8],
					Status:          statuses[8],
					ValidatorIndex:  validatorIndices[8],
					IsSyncCommittee: testCase.fetchSyncDuties,
				},
				{
					PublicKey:       pubkeys[9],
					Status:          statuses[9],
					ValidatorIndex:  validatorIndices[9],
					IsSyncCommittee: testCase.fetchSyncDuties,
				},
				{
					PublicKey:      pubkeys[10],
					Status:         statuses[10],
					ValidatorIndex: validatorIndices[10],
				},
				{
					PublicKey:      pubkeys[11],
					Status:         statuses[11],
					ValidatorIndex: validatorIndices[11],
				},
			}

			validatorClient := &beaconApiValidatorClient{dutiesProvider: dutiesProvider}
			duties, err := validatorClient.getDutiesForEpoch(
				ctx,
				epoch,
				multipleValidatorStatus,
				testCase.fetchSyncDuties,
			)
			require.NoError(t, err)
			assert.DeepEqual(t, expectedDuties, duties)
		})
	}
}

func TestGetDuties_Valid(t *testing.T) {
	testCases := []struct {
		name  string
		epoch primitives.Epoch
	}{
		{
			name:  "genesis epoch",
			epoch: params.BeaconConfig().GenesisEpoch,
		},
		{
			name:  "altair epoch",
			epoch: params.BeaconConfig().AltairForkEpoch,
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			statuses := []ethpb.ValidatorStatus{
				ethpb.ValidatorStatus_DEPOSITED,
				ethpb.ValidatorStatus_PENDING,
				ethpb.ValidatorStatus_ACTIVE,
				ethpb.ValidatorStatus_EXITING,
				ethpb.ValidatorStatus_SLASHING,
				ethpb.ValidatorStatus_EXITED,
				ethpb.ValidatorStatus_EXITED,
				ethpb.ValidatorStatus_EXITED,
				ethpb.ValidatorStatus_EXITED,
				ethpb.ValidatorStatus_DEPOSITED,
				ethpb.ValidatorStatus_PENDING,
				ethpb.ValidatorStatus_ACTIVE,
			}
			pubkeys := make([][]byte, len(statuses))
			validatorIndices := make([]primitives.ValidatorIndex, len(statuses))
			for i := range statuses {
				pubkeys[i] = []byte(strconv.Itoa(i))
				validatorIndices[i] = primitives.ValidatorIndex(i)
			}

			committeeIndices := []primitives.CommitteeIndex{25, 26, 27}
			committeeSlots := []primitives.Slot{28, 29, 30}
			proposerSlots := []primitives.Slot{31, 32, 33, 34, 35, 36, 37, 38}

			statusResps := make([]*ethpb.ValidatorStatusResponse, len(statuses))
			for i, s := range statuses {
				statusResps[i] = &ethpb.ValidatorStatusResponse{Status: s}
			}
			multipleValidatorStatus := &ethpb.MultipleValidatorStatusResponse{
				PublicKeys: pubkeys,
				Indices:    validatorIndices,
				Statuses:   statusResps,
			}

			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			ctx := context.Background()

			dutiesProvider := mock.NewMockdutiesProvider(ctrl)
			dutiesProvider.EXPECT().GetCommittees(
				ctx,
				testCase.epoch,
			).Return(
				generateValidCommittees(committeeIndices, committeeSlots, validatorIndices),
				nil,
			).Times(2)

			dutiesProvider.EXPECT().GetAttesterDuties(
				ctx,
				testCase.epoch,
				multipleValidatorStatus.Indices,
			).Return(
				generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots),
				nil,
			).Times(2)

			dutiesProvider.EXPECT().GetProposerDuties(
				ctx,
				testCase.epoch,
			).Return(
				generateValidProposerDuties(pubkeys, validatorIndices, proposerSlots),
				nil,
			).Times(2)

			fetchSyncDuties := testCase.epoch >= params.BeaconConfig().AltairForkEpoch
			if fetchSyncDuties {
				dutiesProvider.EXPECT().GetSyncDuties(
					ctx,
					testCase.epoch,
					multipleValidatorStatus.Indices,
				).Return(
					generateValidSyncDuties(pubkeys, validatorIndices),
					nil,
				).Times(2)
			}

			dutiesProvider.EXPECT().GetCommittees(
				ctx,
				testCase.epoch+1,
			).Return(
				reverseSlice(generateValidCommittees(committeeIndices, committeeSlots, validatorIndices)),
				nil,
			).Times(2)

			dutiesProvider.EXPECT().GetAttesterDuties(
				ctx,
				testCase.epoch+1,
				validatorIndices,
			).Return(
				reverseSlice(generateValidAttesterDuties(pubkeys, validatorIndices, committeeIndices, committeeSlots)),
				nil,
			).Times(2)

			dutiesProvider.EXPECT().GetProposerDuties(
				ctx,
				testCase.epoch+1,
			).Return(
				generateValidProposerDuties(pubkeys, validatorIndices, proposerSlots),
				nil,
			).Times(2)

			if fetchSyncDuties {
				dutiesProvider.EXPECT().GetSyncDuties(
					ctx,
					testCase.epoch+1,
					validatorIndices,
				).Return(
					reverseSlice(generateValidSyncDuties(pubkeys, validatorIndices)),
					nil,
				).Times(2)
			}

			stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)
			stateValidatorsProvider.EXPECT().GetStateValidators(
				ctx,
				gomock.Any(),
				gomock.Any(),
				gomock.Any(),
			).Return(
				&beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[0]), 10),
							Status: "pending_initialized",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[0]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[1]), 10),
							Status: "pending_queued",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[1]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[2]), 10),
							Status: "active_ongoing",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[2]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[3]), 10),
							Status: "active_exiting",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[3]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[4]), 10),
							Status: "active_slashed",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[4]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[5]), 10),
							Status: "exited_unslashed",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[5]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[6]), 10),
							Status: "exited_slashed",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[6]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[7]), 10),
							Status: "withdrawal_possible",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[7]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[8]), 10),
							Status: "withdrawal_done",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[8]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[9]), 10),
							Status: "pending_initialized",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[9]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[10]), 10),
							Status: "pending_queued",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[10]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
						{
							Index:  strconv.FormatUint(uint64(validatorIndices[11]), 10),
							Status: "active_ongoing",
							Validator: &beacon.Validator{
								Pubkey:          hexutil.Encode(pubkeys[11]),
								ActivationEpoch: strconv.FormatUint(uint64(testCase.epoch), 10),
							},
						},
					},
				},
				nil,
			).MinTimes(1)

			prysmBeaconChainClient := validatormock.NewMockPrysmBeaconChainClient(ctrl)
			prysmBeaconChainClient.EXPECT().GetValidatorCount(
				ctx,
				gomock.Any(),
				gomock.Any(),
			).Return(
				nil,
				iface.ErrNotSupported,
			).MinTimes(1)

			// Make sure that our values are equal to what would be returned by calling getDutiesForEpoch individually
			validatorClient := &beaconApiValidatorClient{
				dutiesProvider:          dutiesProvider,
				stateValidatorsProvider: stateValidatorsProvider,
				prysmBeaconChainCLient:  prysmBeaconChainClient,
			}

			expectedCurrentEpochDuties, err := validatorClient.getDutiesForEpoch(
				ctx,
				testCase.epoch,
				multipleValidatorStatus,
				fetchSyncDuties,
			)
			require.NoError(t, err)

			expectedNextEpochDuties, err := validatorClient.getDutiesForEpoch(
				ctx,
				testCase.epoch+1,
				multipleValidatorStatus,
				fetchSyncDuties,
			)
			require.NoError(t, err)

			expectedDuties := &ethpb.DutiesResponse{
				Duties:             expectedCurrentEpochDuties,
				CurrentEpochDuties: expectedCurrentEpochDuties,
				NextEpochDuties:    expectedNextEpochDuties,
			}

			duties, err := validatorClient.getDuties(ctx, &ethpb.DutiesRequest{
				Epoch:      testCase.epoch,
				PublicKeys: append(pubkeys, []byte("0xunknown")),
			})
			require.NoError(t, err)

			assert.DeepEqual(t, expectedDuties, duties)
		})
	}
}

func TestGetDuties_GetValidatorStatusFailed(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)
	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		nil,
		errors.New("foo error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{
		stateValidatorsProvider: stateValidatorsProvider,
	}

	_, err := validatorClient.getDuties(ctx, &ethpb.DutiesRequest{
		Epoch:      1,
		PublicKeys: [][]byte{},
	})
	assert.ErrorContains(t, "failed to get validator status", err)
	assert.ErrorContains(t, "foo error", err)
}

func TestGetDuties_GetDutiesForEpochFailed(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)
	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		&beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{},
		},
		nil,
	).Times(1)

	dutiesProvider := mock.NewMockdutiesProvider(ctrl)
	dutiesProvider.EXPECT().GetAttesterDuties(
		ctx,
		primitives.Epoch(1),
		gomock.Any(),
	).Return(
		nil,
		errors.New("foo error"),
	).Times(1)

	prysmBeaconChainClient := validatormock.NewMockPrysmBeaconChainClient(ctrl)
	prysmBeaconChainClient.EXPECT().GetValidatorCount(
		ctx,
		gomock.Any(),
		gomock.Any(),
	).Return(
		nil,
		iface.ErrNotSupported,
	).MinTimes(1)

	validatorClient := &beaconApiValidatorClient{
		stateValidatorsProvider: stateValidatorsProvider,
		dutiesProvider:          dutiesProvider,
		prysmBeaconChainCLient:  prysmBeaconChainClient,
	}

	_, err := validatorClient.getDuties(ctx, &ethpb.DutiesRequest{
		Epoch:      1,
		PublicKeys: [][]byte{},
	})
	assert.ErrorContains(t, "failed to get duties for current epoch `1`", err)
	assert.ErrorContains(t, "foo error", err)
}

func generateValidCommittees(committeeIndices []primitives.CommitteeIndex, slots []primitives.Slot, validatorIndices []primitives.ValidatorIndex) []*shared.Committee {
	return []*shared.Committee{
		{
			Index: strconv.FormatUint(uint64(committeeIndices[0]), 10),
			Slot:  strconv.FormatUint(uint64(slots[0]), 10),
			Validators: []string{
				strconv.FormatUint(uint64(validatorIndices[0]), 10),
				strconv.FormatUint(uint64(validatorIndices[1]), 10),
			},
		},
		{
			Index: strconv.FormatUint(uint64(committeeIndices[1]), 10),
			Slot:  strconv.FormatUint(uint64(slots[1]), 10),
			Validators: []string{
				strconv.FormatUint(uint64(validatorIndices[2]), 10),
				strconv.FormatUint(uint64(validatorIndices[3]), 10),
			},
		},
		{
			Index: strconv.FormatUint(uint64(committeeIndices[2]), 10),
			Slot:  strconv.FormatUint(uint64(slots[2]), 10),
			Validators: []string{
				strconv.FormatUint(uint64(validatorIndices[4]), 10),
				strconv.FormatUint(uint64(validatorIndices[5]), 10),
			},
		},
	}
}

func generateValidAttesterDuties(pubkeys [][]byte, validatorIndices []primitives.ValidatorIndex, committeeIndices []primitives.CommitteeIndex, slots []primitives.Slot) []*validator.AttesterDuty {
	return []*validator.AttesterDuty{
		{
			Pubkey:         hexutil.Encode(pubkeys[0]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[0]), 10),
			CommitteeIndex: strconv.FormatUint(uint64(committeeIndices[0]), 10),
			Slot:           strconv.FormatUint(uint64(slots[0]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[1]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[1]), 10),
			CommitteeIndex: strconv.FormatUint(uint64(committeeIndices[0]), 10),
			Slot:           strconv.FormatUint(uint64(slots[0]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[2]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[2]), 10),
			CommitteeIndex: strconv.FormatUint(uint64(committeeIndices[1]), 10),
			Slot:           strconv.FormatUint(uint64(slots[1]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[3]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[3]), 10),
			CommitteeIndex: strconv.FormatUint(uint64(committeeIndices[1]), 10),
			Slot:           strconv.FormatUint(uint64(slots[1]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[4]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[4]), 10),
			CommitteeIndex: strconv.FormatUint(uint64(committeeIndices[2]), 10),
			Slot:           strconv.FormatUint(uint64(slots[2]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[5]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[5]), 10),
			CommitteeIndex: strconv.FormatUint(uint64(committeeIndices[2]), 10),
			Slot:           strconv.FormatUint(uint64(slots[2]), 10),
		},
	}
}

func generateValidProposerDuties(pubkeys [][]byte, validatorIndices []primitives.ValidatorIndex, slots []primitives.Slot) []*validator.ProposerDuty {
	return []*validator.ProposerDuty{
		{
			Pubkey:         hexutil.Encode(pubkeys[4]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[4]), 10),
			Slot:           strconv.FormatUint(uint64(slots[0]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[4]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[4]), 10),
			Slot:           strconv.FormatUint(uint64(slots[1]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[5]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[5]), 10),
			Slot:           strconv.FormatUint(uint64(slots[2]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[5]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[5]), 10),
			Slot:           strconv.FormatUint(uint64(slots[3]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[6]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[6]), 10),
			Slot:           strconv.FormatUint(uint64(slots[4]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[6]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[6]), 10),
			Slot:           strconv.FormatUint(uint64(slots[5]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[7]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[7]), 10),
			Slot:           strconv.FormatUint(uint64(slots[6]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[7]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[7]), 10),
			Slot:           strconv.FormatUint(uint64(slots[7]), 10),
		},
	}
}

func generateValidSyncDuties(pubkeys [][]byte, validatorIndices []primitives.ValidatorIndex) []*validator.SyncCommitteeDuty {
	return []*validator.SyncCommitteeDuty{
		{
			Pubkey:         hexutil.Encode(pubkeys[5]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[5]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[6]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[6]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[7]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[7]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[8]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[8]), 10),
		},
		{
			Pubkey:         hexutil.Encode(pubkeys[9]),
			ValidatorIndex: strconv.FormatUint(uint64(validatorIndices[9]), 10),
		},
	}
}

// We will use a reverse function to easily make sure that the current epoch and next epoch data returned by getDutiesForEpoch
// are not the same
func reverseSlice[T interface{}](slice []T) []T {
	reversedSlice := make([]T, len(slice))
	for i := range slice {
		reversedSlice[len(reversedSlice)-1-i] = slice[i]
	}
	return reversedSlice
}
