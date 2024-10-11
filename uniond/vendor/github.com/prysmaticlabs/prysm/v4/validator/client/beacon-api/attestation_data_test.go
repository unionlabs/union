package beacon_api

import (
	"context"
	"errors"
	"fmt"
	"strconv"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestGetAttestationData_ValidAttestation(t *testing.T) {
	ctx := context.Background()
	expectedSlot := uint64(5)
	expectedCommitteeIndex := uint64(6)
	expectedBeaconBlockRoot := "0x0636045df9bdda3ab96592cf5389032c8ec3977f911e2b53509b348dfe164d4d"
	expectedSourceEpoch := uint64(7)
	expectedSourceRoot := "0xd4bcbdefc8156e85247681086e8050e5d2d5d1bf076a25f6decd99250f3a378d"
	expectedTargetEpoch := uint64(8)
	expectedTargetRoot := "0x246590e8e4c2a9bd13cc776ecc7025bc432219f076e80b27267b8fa0456dc821"

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	produceAttestationDataResponseJson := validator.GetAttestationDataResponse{}

	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v1/validator/attestation_data?committee_index=%d&slot=%d", expectedCommitteeIndex, expectedSlot),
		&produceAttestationDataResponseJson,
	).Return(
		nil,
	).SetArg(
		2,
		validator.GetAttestationDataResponse{
			Data: &shared.AttestationData{
				Slot:            strconv.FormatUint(expectedSlot, 10),
				CommitteeIndex:  strconv.FormatUint(expectedCommitteeIndex, 10),
				BeaconBlockRoot: expectedBeaconBlockRoot,
				Source: &shared.Checkpoint{
					Epoch: strconv.FormatUint(expectedSourceEpoch, 10),
					Root:  expectedSourceRoot,
				},
				Target: &shared.Checkpoint{
					Epoch: strconv.FormatUint(expectedTargetEpoch, 10),
					Root:  expectedTargetRoot,
				},
			},
		},
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	resp, err := validatorClient.getAttestationData(ctx, primitives.Slot(expectedSlot), primitives.CommitteeIndex(expectedCommitteeIndex))
	assert.NoError(t, err)

	require.NotNil(t, resp)
	assert.Equal(t, expectedBeaconBlockRoot, hexutil.Encode(resp.BeaconBlockRoot))
	assert.Equal(t, expectedCommitteeIndex, uint64(resp.CommitteeIndex))
	assert.Equal(t, expectedSlot, uint64(resp.Slot))

	require.NotNil(t, resp.Source)
	assert.Equal(t, expectedSourceEpoch, uint64(resp.Source.Epoch))
	assert.Equal(t, expectedSourceRoot, hexutil.Encode(resp.Source.Root))

	require.NotNil(t, resp.Target)
	assert.Equal(t, expectedTargetEpoch, uint64(resp.Target.Epoch))
	assert.Equal(t, expectedTargetRoot, hexutil.Encode(resp.Target.Root))
}

func TestGetAttestationData_InvalidData(t *testing.T) {
	ctx := context.Background()

	testCases := []struct {
		name                 string
		generateData         func() validator.GetAttestationDataResponse
		expectedErrorMessage string
	}{
		{
			name: "nil attestation data",
			generateData: func() validator.GetAttestationDataResponse {
				return validator.GetAttestationDataResponse{
					Data: nil,
				}
			},
			expectedErrorMessage: "attestation data is nil",
		},
		{
			name: "invalid committee index",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.CommitteeIndex = "foo"
				return attestation
			},
			expectedErrorMessage: "failed to parse attestation committee index: foo",
		},
		{
			name: "invalid block root",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.BeaconBlockRoot = "foo"
				return attestation
			},
			expectedErrorMessage: "invalid beacon block root: foo",
		},
		{
			name: "invalid slot",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Slot = "foo"
				return attestation
			},
			expectedErrorMessage: "failed to parse attestation slot: foo",
		},
		{
			name: "nil source",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Source = nil
				return attestation
			},
			expectedErrorMessage: "attestation source is nil",
		},
		{
			name: "invalid source epoch",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Source.Epoch = "foo"
				return attestation
			},
			expectedErrorMessage: "failed to parse attestation source epoch: foo",
		},
		{
			name: "invalid source root",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Source.Root = "foo"
				return attestation
			},
			expectedErrorMessage: "invalid attestation source root: foo",
		},
		{
			name: "nil target",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Target = nil
				return attestation
			},
			expectedErrorMessage: "attestation target is nil",
		},
		{
			name: "invalid target epoch",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Target.Epoch = "foo"
				return attestation
			},
			expectedErrorMessage: "failed to parse attestation target epoch: foo",
		},
		{
			name: "invalid target root",
			generateData: func() validator.GetAttestationDataResponse {
				attestation := generateValidAttestation(1, 2)
				attestation.Data.Target.Root = "foo"
				return attestation
			},
			expectedErrorMessage: "invalid attestation target root: foo",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			produceAttestationDataResponseJson := validator.GetAttestationDataResponse{}
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Get(
				ctx,
				"/eth/v1/validator/attestation_data?committee_index=2&slot=1",
				&produceAttestationDataResponseJson,
			).Return(
				nil,
			).SetArg(
				2,
				testCase.generateData(),
			).Times(1)

			validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
			_, err := validatorClient.getAttestationData(ctx, 1, 2)
			assert.ErrorContains(t, testCase.expectedErrorMessage, err)
		})
	}
}

func TestGetAttestationData_JsonResponseError(t *testing.T) {
	const slot = primitives.Slot(1)
	const committeeIndex = primitives.CommitteeIndex(2)

	ctx := context.Background()

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	produceAttestationDataResponseJson := validator.GetAttestationDataResponse{}
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v1/validator/attestation_data?committee_index=%d&slot=%d", committeeIndex, slot),
		&produceAttestationDataResponseJson,
	).Return(
		errors.New("some specific json response error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	_, err := validatorClient.getAttestationData(ctx, slot, committeeIndex)
	assert.ErrorContains(t, "some specific json response error", err)
}

func generateValidAttestation(slot uint64, committeeIndex uint64) validator.GetAttestationDataResponse {
	return validator.GetAttestationDataResponse{
		Data: &shared.AttestationData{
			Slot:            strconv.FormatUint(slot, 10),
			CommitteeIndex:  strconv.FormatUint(committeeIndex, 10),
			BeaconBlockRoot: "0x5ecf3bff35e39d5f75476d42950d549f81fa93038c46b6652ae89ae1f7ad834f",
			Source: &shared.Checkpoint{
				Epoch: "3",
				Root:  "0x9023c9e64f23c1d451d5073c641f5f69597c2ad7d82f6f16e67d703e0ce5db8b",
			},
			Target: &shared.Checkpoint{
				Epoch: "4",
				Root:  "0xb154d46803b15b458ca822466547b054bc124338c6ee1d9c433dcde8c4457cca",
			},
		},
	}
}
