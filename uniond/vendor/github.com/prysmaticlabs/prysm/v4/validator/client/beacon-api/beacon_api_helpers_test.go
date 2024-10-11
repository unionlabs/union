package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/url"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestBeaconApiHelpers(t *testing.T) {
	tests := []struct {
		name  string
		input string
		valid bool
	}{
		{
			name:  "correct format",
			input: "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
			valid: true,
		},
		{
			name:  "root too small",
			input: "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f",
			valid: false,
		},
		{
			name:  "root too big",
			input: "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f22",
			valid: false,
		},
		{
			name:  "empty root",
			input: "",
			valid: false,
		},
		{
			name:  "no 0x prefix",
			input: "cf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
			valid: false,
		},
		{
			name:  "invalid characters",
			input: "0xzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
			valid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.valid, validRoot(tt.input))
		})
	}
}

func TestBeaconApiHelpers_TestUint64ToString(t *testing.T) {
	const expectedResult = "1234"
	const val = uint64(1234)

	assert.Equal(t, expectedResult, uint64ToString(val))
	assert.Equal(t, expectedResult, uint64ToString(primitives.Slot(val)))
	assert.Equal(t, expectedResult, uint64ToString(primitives.ValidatorIndex(val)))
	assert.Equal(t, expectedResult, uint64ToString(primitives.CommitteeIndex(val)))
	assert.Equal(t, expectedResult, uint64ToString(primitives.Epoch(val)))
}

func TestBuildURL_NoParams(t *testing.T) {
	wanted := "/aaa/bbb/ccc"
	actual := buildURL("/aaa/bbb/ccc")
	assert.Equal(t, wanted, actual)
}

func TestBuildURL_WithParams(t *testing.T) {
	params := url.Values{}
	params.Add("xxxx", "1")
	params.Add("yyyy", "2")
	params.Add("zzzz", "3")

	wanted := "/aaa/bbb/ccc?xxxx=1&yyyy=2&zzzz=3"
	actual := buildURL("/aaa/bbb/ccc", params)
	assert.Equal(t, wanted, actual)
}

const forkEndpoint = "/eth/v1/beacon/states/head/fork"

func TestGetFork_Nominal(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	stateForkResponseJson := beacon.GetStateForkResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	expected := beacon.GetStateForkResponse{
		Data: &shared.Fork{
			PreviousVersion: "0x1",
			CurrentVersion:  "0x2",
			Epoch:           "3",
		},
	}

	ctx := context.Background()

	jsonRestHandler.EXPECT().Get(
		ctx,
		forkEndpoint,
		&stateForkResponseJson,
	).Return(
		nil,
	).SetArg(
		2,
		expected,
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		jsonRestHandler: jsonRestHandler,
	}

	fork, err := validatorClient.getFork(ctx)
	require.NoError(t, err)
	assert.DeepEqual(t, &expected, fork)
}

func TestGetFork_Invalid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	ctx := context.Background()

	jsonRestHandler.EXPECT().Get(
		ctx,
		forkEndpoint,
		gomock.Any(),
	).Return(
		errors.New("custom error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		jsonRestHandler: jsonRestHandler,
	}

	_, err := validatorClient.getFork(ctx)
	require.ErrorContains(t, "custom error", err)
}

const headersEndpoint = "/eth/v1/beacon/headers"

func TestGetHeaders_Nominal(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	blockHeadersResponseJson := beacon.GetBlockHeadersResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	expected := beacon.GetBlockHeadersResponse{
		Data: []*shared.SignedBeaconBlockHeaderContainer{
			{
				Header: &shared.SignedBeaconBlockHeader{
					Message: &shared.BeaconBlockHeader{
						Slot: "42",
					},
				},
			},
		},
	}

	ctx := context.Background()

	jsonRestHandler.EXPECT().Get(
		ctx,
		headersEndpoint,
		&blockHeadersResponseJson,
	).Return(
		nil,
	).SetArg(
		2,
		expected,
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		jsonRestHandler: jsonRestHandler,
	}

	headers, err := validatorClient.getHeaders(ctx)
	require.NoError(t, err)
	assert.DeepEqual(t, &expected, headers)
}

func TestGetHeaders_Invalid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	ctx := context.Background()

	jsonRestHandler.EXPECT().Get(
		ctx,
		headersEndpoint,
		gomock.Any(),
	).Return(
		errors.New("custom error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		jsonRestHandler: jsonRestHandler,
	}

	_, err := validatorClient.getHeaders(ctx)
	require.ErrorContains(t, "custom error", err)
}

const livenessEndpoint = "/eth/v1/validator/liveness/42"

func TestGetLiveness_Nominal(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	livenessResponseJson := validator.GetLivenessResponse{}

	indexes := []string{"1", "2"}
	marshalledIndexes, err := json.Marshal(indexes)
	require.NoError(t, err)

	expected := validator.GetLivenessResponse{
		Data: []*validator.Liveness{
			{
				Index:  "1",
				IsLive: true,
			},
			{
				Index:  "2",
				IsLive: false,
			},
		},
	}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		livenessEndpoint,
		nil,
		bytes.NewBuffer(marshalledIndexes),
		&livenessResponseJson,
	).SetArg(
		4,
		expected,
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	liveness, err := validatorClient.getLiveness(ctx, 42, indexes)

	require.NoError(t, err)
	assert.DeepEqual(t, &expected, liveness)
}

func TestGetLiveness_Invalid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		livenessEndpoint,
		nil,
		gomock.Any(),
		gomock.Any(),
	).Return(
		errors.New("custom error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	_, err := validatorClient.getLiveness(ctx, 42, nil)

	require.ErrorContains(t, "custom error", err)
}

const syncingEnpoint = "/eth/v1/node/syncing"

func TestGetIsSyncing_Nominal(t *testing.T) {
	testCases := []struct {
		name      string
		isSyncing bool
	}{
		{
			name:      "Syncing",
			isSyncing: true,
		},
		{
			name:      "Not syncing",
			isSyncing: false,
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			syncingResponseJson := node.SyncStatusResponse{}
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

			expected := node.SyncStatusResponse{
				Data: &node.SyncStatusResponseData{
					IsSyncing: testCase.isSyncing,
				},
			}

			ctx := context.Background()

			jsonRestHandler.EXPECT().Get(
				ctx,
				syncingEnpoint,
				&syncingResponseJson,
			).Return(
				nil,
			).SetArg(
				2,
				expected,
			).Times(1)

			validatorClient := beaconApiValidatorClient{
				jsonRestHandler: jsonRestHandler,
			}

			isSyncing, err := validatorClient.isSyncing(ctx)
			require.NoError(t, err)
			assert.Equal(t, testCase.isSyncing, isSyncing)
		})
	}
}

func TestGetIsSyncing_Invalid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	syncingResponseJson := node.SyncStatusResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	ctx := context.Background()

	jsonRestHandler.EXPECT().Get(
		ctx,
		syncingEnpoint,
		&syncingResponseJson,
	).Return(
		errors.New("custom error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		jsonRestHandler: jsonRestHandler,
	}

	isSyncing, err := validatorClient.isSyncing(ctx)
	assert.Equal(t, true, isSyncing)
	assert.ErrorContains(t, "failed to get syncing status", err)
}
