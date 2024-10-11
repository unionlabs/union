package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

const stringPubKey = "0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13"

func getPubKeyAndReqBuffer(t *testing.T) ([]byte, *bytes.Buffer) {
	pubKey, err := hexutil.Decode(stringPubKey)
	require.NoError(t, err)
	req := beacon.GetValidatorsRequest{
		Ids:      []string{stringPubKey},
		Statuses: []string{},
	}
	reqBytes, err := json.Marshal(req)
	require.NoError(t, err)
	return pubKey, bytes.NewBuffer(reqBytes)
}

func TestIndex_Nominal(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	pubKey, reqBuffer := getPubKeyAndReqBuffer(t)
	ctx := context.Background()

	stateValidatorsResponseJson := beacon.GetValidatorsResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/states/head/validators",
		nil,
		reqBuffer,
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
						Pubkey: stringPubKey,
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

	validatorIndex, err := validatorClient.ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubKey,
		},
	)

	require.NoError(t, err)
	assert.Equal(t, primitives.ValidatorIndex(55293), validatorIndex.Index)
}

func TestIndex_UnexistingValidator(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	pubKey, reqBuffer := getPubKeyAndReqBuffer(t)
	ctx := context.Background()

	stateValidatorsResponseJson := beacon.GetValidatorsResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/states/head/validators",
		nil,
		reqBuffer,
		&stateValidatorsResponseJson,
	).Return(
		nil,
	).SetArg(
		4,
		beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{},
		},
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: beaconApiStateValidatorsProvider{
			jsonRestHandler: jsonRestHandler,
		},
	}

	_, err := validatorClient.ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubKey,
		},
	)

	wanted := "could not find validator index for public key `0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13`"
	assert.ErrorContains(t, wanted, err)
}

func TestIndex_BadIndexError(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	pubKey, reqBuffer := getPubKeyAndReqBuffer(t)
	ctx := context.Background()

	stateValidatorsResponseJson := beacon.GetValidatorsResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/states/head/validators",
		nil,
		reqBuffer,
		&stateValidatorsResponseJson,
	).Return(
		nil,
	).SetArg(
		4,
		beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{
				{
					Index:  "This is not an index",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey: stringPubKey,
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

	_, err := validatorClient.ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubKey,
		},
	)

	assert.ErrorContains(t, "failed to parse validator index", err)
}

func TestIndex_JsonResponseError(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	pubKey, reqBuffer := getPubKeyAndReqBuffer(t)
	ctx := context.Background()

	stateValidatorsResponseJson := beacon.GetValidatorsResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/states/head/validators",
		nil,
		reqBuffer,
		&stateValidatorsResponseJson,
	).Return(
		errors.New("some specific json error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: beaconApiStateValidatorsProvider{
			jsonRestHandler: jsonRestHandler,
		},
	}

	_, err := validatorClient.ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubKey,
		},
	)

	assert.ErrorContains(t, "failed to get state validator", err)
}
