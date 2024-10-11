package beacon_api

import (
	"context"
	"errors"
	"net/http"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestProposeBeaconBlock_Error(t *testing.T) {
	testSuites := []struct {
		name                 string
		returnedError        error
		expectedErrorMessage string
	}{
		{
			name:                 "error 202",
			expectedErrorMessage: "block was successfully broadcast but failed validation",
			returnedError: &httputil.DefaultJsonError{
				Code:    http.StatusAccepted,
				Message: "202 error",
			},
		},
		{
			name:                 "error 500",
			expectedErrorMessage: "HTTP request unsuccessful (500: foo error)",
			returnedError: &httputil.DefaultJsonError{
				Code:    http.StatusInternalServerError,
				Message: "foo error",
			},
		},
		{
			name:                 "other error",
			expectedErrorMessage: "foo error",
			returnedError:        errors.New("foo error"),
		},
	}

	testCases := []struct {
		name             string
		consensusVersion string
		endpoint         string
		block            *ethpb.GenericSignedBeaconBlock
	}{
		{
			name:             "phase0",
			consensusVersion: "phase0",
			endpoint:         "/eth/v1/beacon/blocks",
			block: &ethpb.GenericSignedBeaconBlock{
				Block: generateSignedPhase0Block(),
			},
		},
		{
			name:             "altair",
			consensusVersion: "altair",
			endpoint:         "/eth/v1/beacon/blocks",
			block: &ethpb.GenericSignedBeaconBlock{
				Block: generateSignedAltairBlock(),
			},
		},
		{
			name:             "bellatrix",
			consensusVersion: "bellatrix",
			endpoint:         "/eth/v1/beacon/blocks",
			block: &ethpb.GenericSignedBeaconBlock{
				Block: generateSignedBellatrixBlock(),
			},
		},
		{
			name:             "blinded bellatrix",
			consensusVersion: "bellatrix",
			endpoint:         "/eth/v1/beacon/blinded_blocks",
			block: &ethpb.GenericSignedBeaconBlock{
				Block: generateSignedBlindedBellatrixBlock(),
			},
		},
		{
			name:             "blinded capella",
			consensusVersion: "capella",
			endpoint:         "/eth/v1/beacon/blinded_blocks",
			block: &ethpb.GenericSignedBeaconBlock{
				Block: generateSignedBlindedCapellaBlock(),
			},
		},
	}

	for _, testSuite := range testSuites {
		for _, testCase := range testCases {
			t.Run(testSuite.name+"/"+testCase.name, func(t *testing.T) {
				ctrl := gomock.NewController(t)
				defer ctrl.Finish()

				ctx := context.Background()
				jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

				headers := map[string]string{"Eth-Consensus-Version": testCase.consensusVersion}
				jsonRestHandler.EXPECT().Post(
					ctx,
					testCase.endpoint,
					headers,
					gomock.Any(),
					nil,
				).Return(
					testSuite.returnedError,
				).Times(1)

				validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
				_, err := validatorClient.proposeBeaconBlock(ctx, testCase.block)
				assert.ErrorContains(t, testSuite.expectedErrorMessage, err)
			})
		}
	}
}

func TestProposeBeaconBlock_UnsupportedBlockType(t *testing.T) {
	validatorClient := &beaconApiValidatorClient{}
	_, err := validatorClient.proposeBeaconBlock(context.Background(), &ethpb.GenericSignedBeaconBlock{})
	assert.ErrorContains(t, "unsupported block type", err)
}
