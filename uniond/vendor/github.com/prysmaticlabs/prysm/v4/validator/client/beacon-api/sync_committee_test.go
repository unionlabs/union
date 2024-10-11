package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/golang/protobuf/ptypes/empty"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestSubmitSyncMessage_Valid(t *testing.T) {
	const beaconBlockRoot = "0x719d4f66a5f25c35d93718821aacb342194391034b11cf0a5822cc249178a274"
	const signature = "0xb459ef852bd4e0cb96e6723d67cacc8215406dd9ba663f8874a083167ebf428b28b746431bdbc1820a25289377b2610881e52b3a05c3548c5e99c08c8a36342573be5962d7510c03dcba8ddfb8ae419e59d222ddcf31cc512e704ef2cc3cf8"

	decodedBeaconBlockRoot, err := hexutil.Decode(beaconBlockRoot)
	require.NoError(t, err)

	decodedSignature, err := hexutil.Decode(signature)
	require.NoError(t, err)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonSyncCommitteeMessage := &shared.SyncCommitteeMessage{
		Slot:            "42",
		BeaconBlockRoot: beaconBlockRoot,
		ValidatorIndex:  "12345",
		Signature:       signature,
	}

	marshalledJsonRegistrations, err := json.Marshal([]*shared.SyncCommitteeMessage{jsonSyncCommitteeMessage})
	require.NoError(t, err)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		context.Background(),
		"/eth/v1/beacon/pool/sync_committees",
		nil,
		bytes.NewBuffer(marshalledJsonRegistrations),
		nil,
	).Return(
		nil,
	).Times(1)

	protoSyncCommiteeMessage := ethpb.SyncCommitteeMessage{
		Slot:           primitives.Slot(42),
		BlockRoot:      decodedBeaconBlockRoot,
		ValidatorIndex: primitives.ValidatorIndex(12345),
		Signature:      decodedSignature,
	}

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	res, err := validatorClient.SubmitSyncMessage(context.Background(), &protoSyncCommiteeMessage)

	assert.DeepEqual(t, new(empty.Empty), res)
	require.NoError(t, err)
}

func TestSubmitSyncMessage_BadRequest(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		context.Background(),
		"/eth/v1/beacon/pool/sync_committees",
		nil,
		gomock.Any(),
		nil,
	).Return(
		errors.New("foo error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	_, err := validatorClient.SubmitSyncMessage(context.Background(), &ethpb.SyncCommitteeMessage{})
	assert.ErrorContains(t, "foo error", err)
}

func TestGetSyncMessageBlockRoot(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	const blockRoot = "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	tests := []struct {
		name                 string
		endpointError        error
		expectedErrorMessage string
		expectedResponse     beacon.BlockRootResponse
	}{
		{
			name: "valid request",
			expectedResponse: beacon.BlockRootResponse{
				Data: &beacon.BlockRoot{
					Root: blockRoot,
				},
			},
		},
		{
			name:                 "internal server error",
			expectedErrorMessage: "internal server error",
			endpointError:        errors.New("internal server error"),
		},
		{
			name: "execution optimistic",
			expectedResponse: beacon.BlockRootResponse{
				ExecutionOptimistic: true,
			},
			expectedErrorMessage: "the node is currently optimistic and cannot serve validators",
		},
		{
			name:                 "no data",
			expectedResponse:     beacon.BlockRootResponse{},
			expectedErrorMessage: "no data returned",
		},
		{
			name: "no root",
			expectedResponse: beacon.BlockRootResponse{
				Data: new(beacon.BlockRoot),
			},
			expectedErrorMessage: "no root returned",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ctx := context.Background()
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Get(
				ctx,
				"/eth/v1/beacon/blocks/head/root",
				&beacon.BlockRootResponse{},
			).SetArg(
				2,
				test.expectedResponse,
			).Return(
				test.endpointError,
			).Times(1)

			validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
			actualResponse, err := validatorClient.getSyncMessageBlockRoot(ctx)
			if test.expectedErrorMessage != "" {
				require.ErrorContains(t, test.expectedErrorMessage, err)
				return
			}

			require.NoError(t, err)

			expectedRootBytes, err := hexutil.Decode(test.expectedResponse.Data.Root)
			require.NoError(t, err)

			require.NoError(t, err)
			require.DeepEqual(t, expectedRootBytes, actualResponse.Root)
		})
	}
}

func TestGetSyncCommitteeContribution(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	const blockRoot = "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"

	request := &ethpb.SyncCommitteeContributionRequest{
		Slot:      primitives.Slot(1),
		PublicKey: nil,
		SubnetId:  1,
	}

	contributionJson := &shared.SyncCommitteeContribution{
		Slot:              "1",
		BeaconBlockRoot:   blockRoot,
		SubcommitteeIndex: "1",
		AggregationBits:   "0x01",
		Signature:         "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
	}

	tests := []struct {
		name           string
		contribution   validator.ProduceSyncCommitteeContributionResponse
		endpointErr    error
		expectedErrMsg string
	}{
		{
			name:         "valid request",
			contribution: validator.ProduceSyncCommitteeContributionResponse{Data: contributionJson},
		},
		{
			name:           "bad request",
			endpointErr:    errors.New("internal server error"),
			expectedErrMsg: "internal server error",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ctx := context.Background()
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Get(
				ctx,
				"/eth/v1/beacon/blocks/head/root",
				&beacon.BlockRootResponse{},
			).SetArg(
				2,
				beacon.BlockRootResponse{
					Data: &beacon.BlockRoot{
						Root: blockRoot,
					},
				},
			).Return(
				nil,
			).Times(1)

			jsonRestHandler.EXPECT().Get(
				ctx,
				fmt.Sprintf("/eth/v1/validator/sync_committee_contribution?beacon_block_root=%s&slot=%d&subcommittee_index=%d",
					blockRoot, uint64(request.Slot), request.SubnetId),
				&validator.ProduceSyncCommitteeContributionResponse{},
			).SetArg(
				2,
				test.contribution,
			).Return(
				test.endpointErr,
			).Times(1)

			validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
			actualResponse, err := validatorClient.getSyncCommitteeContribution(ctx, request)
			if test.expectedErrMsg != "" {
				require.ErrorContains(t, test.expectedErrMsg, err)
				return
			}
			require.NoError(t, err)

			expectedResponse, err := convertSyncContributionJsonToProto(test.contribution.Data)
			require.NoError(t, err)
			assert.DeepEqual(t, expectedResponse, actualResponse)
		})
	}
}

func TestGetSyncSubCommitteeIndex(t *testing.T) {
	const (
		pubkeyStr          = "0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13"
		syncDutiesEndpoint = "/eth/v1/validator/duties/sync"
		validatorsEndpoint = "/eth/v1/beacon/states/head/validators"
		validatorIndex     = "55293"
		slot               = primitives.Slot(123)
	)

	expectedResponse := &ethpb.SyncSubcommitteeIndexResponse{
		Indices: []primitives.CommitteeIndex{123, 456},
	}

	syncDuties := []*validator.SyncCommitteeDuty{
		{
			Pubkey:         hexutil.Encode([]byte{1}),
			ValidatorIndex: validatorIndex,
			ValidatorSyncCommitteeIndices: []string{
				"123",
				"456",
			},
		},
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	tests := []struct {
		name             string
		duties           []*validator.SyncCommitteeDuty
		validatorsErr    error
		dutiesErr        error
		expectedErrorMsg string
	}{
		{
			name:   "success",
			duties: syncDuties,
		},
		{
			name:             "no sync duties",
			duties:           []*validator.SyncCommitteeDuty{},
			expectedErrorMsg: fmt.Sprintf("no sync committee duty for the given slot %d", slot),
		},
		{
			name:             "duties endpoint error",
			dutiesErr:        errors.New("bad request"),
			expectedErrorMsg: "bad request",
		},
		{
			name:             "validator index endpoint error",
			validatorsErr:    errors.New("bad request"),
			expectedErrorMsg: "bad request",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ctx := context.Background()
			valsReq := &beacon.GetValidatorsRequest{
				Ids:      []string{pubkeyStr},
				Statuses: []string{},
			}
			valsReqBytes, err := json.Marshal(valsReq)
			require.NoError(t, err)
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Post(
				ctx,
				validatorsEndpoint,
				nil,
				bytes.NewBuffer(valsReqBytes),
				&beacon.GetValidatorsResponse{},
			).SetArg(
				4,
				beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index:  validatorIndex,
							Status: "active_ongoing",
							Validator: &beacon.Validator{
								Pubkey: stringPubKey,
							},
						},
					},
				},
			).Return(
				test.validatorsErr,
			).Times(1)

			validatorIndicesBytes, err := json.Marshal([]string{validatorIndex})
			require.NoError(t, err)

			var syncDutiesCalled int
			if test.validatorsErr == nil {
				syncDutiesCalled = 1
			}

			jsonRestHandler.EXPECT().Post(
				ctx,
				fmt.Sprintf("%s/%d", syncDutiesEndpoint, slots.ToEpoch(slot)),
				nil,
				bytes.NewBuffer(validatorIndicesBytes),
				&validator.GetSyncCommitteeDutiesResponse{},
			).SetArg(
				4,
				validator.GetSyncCommitteeDutiesResponse{
					Data: test.duties,
				},
			).Return(
				test.dutiesErr,
			).Times(syncDutiesCalled)

			pubkey, err := hexutil.Decode(pubkeyStr)
			require.NoError(t, err)

			validatorClient := &beaconApiValidatorClient{
				jsonRestHandler: jsonRestHandler,
				stateValidatorsProvider: beaconApiStateValidatorsProvider{
					jsonRestHandler: jsonRestHandler,
				},
				dutiesProvider: beaconApiDutiesProvider{
					jsonRestHandler: jsonRestHandler,
				},
			}
			actualResponse, err := validatorClient.getSyncSubcommitteeIndex(ctx, &ethpb.SyncSubcommitteeIndexRequest{
				PublicKey: pubkey,
				Slot:      slot,
			})
			if test.expectedErrorMsg == "" {
				require.NoError(t, err)
				assert.DeepEqual(t, expectedResponse, actualResponse)
			} else {
				require.ErrorContains(t, test.expectedErrorMsg, err)
			}
		})
	}
}
