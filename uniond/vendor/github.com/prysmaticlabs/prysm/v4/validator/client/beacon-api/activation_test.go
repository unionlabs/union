package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestComputeWaitElements_LastRecvTimeZero(t *testing.T) {
	now := time.Date(2022, 1, 1, 0, 0, 0, 0, time.UTC)
	lastRecvTime := time.Time{}

	waitDuration, nextRecvTime := computeWaitElements(now, lastRecvTime)

	assert.Equal(t, time.Duration(0), waitDuration)
	assert.Equal(t, now, nextRecvTime)
}

func TestComputeWaitElements_LastRecvTimeNotZero(t *testing.T) {
	delay := 10
	now := time.Date(2022, 1, 1, 0, 0, delay, 0, time.UTC)
	lastRecvTime := time.Date(2022, 1, 1, 0, 0, 0, 0, time.UTC)
	secondsPerSlot := params.BeaconConfig().SecondsPerSlot

	waitDuration, nextRecvTime := computeWaitElements(now, lastRecvTime)

	assert.Equal(t, time.Duration(secondsPerSlot-uint64(delay))*time.Second, waitDuration)
	assert.Equal(t, time.Date(2022, 1, 1, 0, 0, int(secondsPerSlot), 0, time.UTC), nextRecvTime)
}

func TestComputeWaitElements_Longest(t *testing.T) {
	now := time.Date(2022, 1, 1, 0, 0, 20, 0, time.UTC)
	lastRecvTime := time.Date(2022, 1, 1, 0, 0, 0, 0, time.UTC)

	waitDuration, nextRecvTime := computeWaitElements(now, lastRecvTime)

	assert.Equal(t, 0*time.Second, waitDuration)
	assert.Equal(t, now, nextRecvTime)
}

func TestActivation_Nominal(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	stringPubKeys := []string{
		"0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13", // active_ongoing
		"0x80000e851c0f53c3246ff726d7ff7766661ca5e12a07c45c114d208d54f0f8233d4380b2e9aff759d69795d1df905526", // active_exiting
		"0x424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242", // does not exist
		"0x800015473bdc3a7f45ef8eb8abc598bc20021e55ad6e6ad1d745aaef9730dd2c28ec08bf42df18451de94dd4a6d24ec5", // exited_slashed
	}

	pubKeys := make([][]byte, len(stringPubKeys))
	for i, stringPubKey := range stringPubKeys {
		pubKey, err := hexutil.Decode(stringPubKey)
		require.NoError(t, err)

		pubKeys[i] = pubKey
	}

	wantedStatuses := []*ethpb.ValidatorActivationResponse_Status{
		{
			PublicKey: pubKeys[0],
			Index:     55293,
			Status: &ethpb.ValidatorStatusResponse{
				Status: ethpb.ValidatorStatus_ACTIVE,
			},
		},
		{
			PublicKey: pubKeys[1],
			Index:     11877,
			Status: &ethpb.ValidatorStatusResponse{
				Status: ethpb.ValidatorStatus_EXITING,
			},
		},
		{
			PublicKey: pubKeys[3],
			Index:     210439,
			Status: &ethpb.ValidatorStatusResponse{
				Status: ethpb.ValidatorStatus_EXITED,
			},
		},
		{
			PublicKey: pubKeys[2],
			Index:     18446744073709551615,
			Status: &ethpb.ValidatorStatusResponse{
				Status: ethpb.ValidatorStatus_UNKNOWN_STATUS,
			},
		},
	}

	stateValidatorsResponseJson := beacon.GetValidatorsResponse{}

	// Instantiate a cancellable context.
	ctx, cancel := context.WithCancel(context.Background())

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	req := &beacon.GetValidatorsRequest{
		Ids:      stringPubKeys,
		Statuses: []string{},
	}
	reqBytes, err := json.Marshal(req)
	require.NoError(t, err)

	// Get does not return any result for non existing key
	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/states/head/validators",
		nil,
		bytes.NewBuffer(reqBytes),
		&stateValidatorsResponseJson,
	).Return(
		nil,
	).SetArg(
		4,
		beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{
				{
					Index:  "55293",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey: stringPubKeys[0],
					},
				},
				{
					Index:  "11877",
					Status: "active_exiting",
					Validator: &beacon.Validator{
						Pubkey: stringPubKeys[1],
					},
				},
				{
					Index:  "210439",
					Status: "exited_slashed",
					Validator: &beacon.Validator{
						Pubkey: stringPubKeys[3],
					},
				},
			},
		},
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: beaconApiStateValidatorsProvider{
			jsonRestHandler: jsonRestHandler,
		},
	}

	waitForActivation, err := validatorClient.WaitForActivation(
		ctx,
		&ethpb.ValidatorActivationRequest{
			PublicKeys: pubKeys,
		},
	)
	assert.NoError(t, err)

	// This first call to `Recv` should return immediately
	resp, err := waitForActivation.Recv()
	require.NoError(t, err)
	assert.DeepEqual(t, wantedStatuses, resp.Statuses)

	// Cancel the context after 1 second
	go func(ctx context.Context) {
		time.Sleep(time.Second)
		cancel()
	}(ctx)

	// This second call to `Recv` should return after ~12 seconds, but is interrupted by the cancel
	_, err = waitForActivation.Recv()

	assert.ErrorContains(t, "context canceled", err)
}

func TestActivation_InvalidData(t *testing.T) {
	testCases := []struct {
		name                 string
		data                 []*beacon.ValidatorContainer
		expectedErrorMessage string
	}{
		{
			name: "bad validator public key",
			data: []*beacon.ValidatorContainer{
				{
					Index:  "55293",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey: "NotAPubKey",
					},
				},
			},
			expectedErrorMessage: "failed to parse validator public key",
		},
		{
			name: "bad validator index",
			data: []*beacon.ValidatorContainer{
				{
					Index:  "NotAnIndex",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey: stringPubKey,
					},
				},
			},
			expectedErrorMessage: "failed to parse validator index",
		},
		{
			name: "invalid validator status",
			data: []*beacon.ValidatorContainer{
				{
					Index:  "12345",
					Status: "NotAStatus",
					Validator: &beacon.Validator{
						Pubkey: stringPubKey,
					},
				},
			},
			expectedErrorMessage: "invalid validator status: NotAStatus",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name,
			func(t *testing.T) {
				ctrl := gomock.NewController(t)
				defer ctrl.Finish()

				ctx := context.Background()

				jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
				jsonRestHandler.EXPECT().Post(
					ctx,
					gomock.Any(),
					gomock.Any(),
					gomock.Any(),
					gomock.Any(),
				).Return(
					nil,
				).SetArg(
					4,
					beacon.GetValidatorsResponse{
						Data: testCase.data,
					},
				).Times(1)

				validatorClient := beaconApiValidatorClient{
					stateValidatorsProvider: beaconApiStateValidatorsProvider{
						jsonRestHandler: jsonRestHandler,
					},
				}

				waitForActivation, err := validatorClient.WaitForActivation(
					ctx,
					&ethpb.ValidatorActivationRequest{},
				)
				assert.NoError(t, err)

				_, err = waitForActivation.Recv()
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			},
		)
	}
}

func TestActivation_JsonResponseError(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).Return(
		errors.New("some specific json error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: beaconApiStateValidatorsProvider{
			jsonRestHandler: jsonRestHandler,
		},
	}

	waitForActivation, err := validatorClient.WaitForActivation(
		ctx,
		&ethpb.ValidatorActivationRequest{},
	)
	assert.NoError(t, err)

	_, err = waitForActivation.Recv()
	assert.ErrorContains(t, "failed to get state validators", err)
}
