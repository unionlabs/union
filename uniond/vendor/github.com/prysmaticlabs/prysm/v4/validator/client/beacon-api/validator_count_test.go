package beacon_api

import (
	"context"
	"errors"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	validator2 "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
)

func TestGetValidatorCount(t *testing.T) {
	const nodeVersion = "prysm/v0.0.1"

	testCases := []struct {
		name                        string
		versionEndpointError        error
		validatorCountEndpointError error
		versionResponse             node.GetVersionResponse
		validatorCountResponse      validator2.CountResponse
		validatorCountCalled        int
		expectedResponse            []iface.ValidatorCount
		expectedError               string
	}{
		{
			name: "success",
			versionResponse: node.GetVersionResponse{
				Data: &node.Version{Version: nodeVersion},
			},
			validatorCountResponse: validator2.CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*validator2.Count{
					{
						Status: "active",
						Count:  "10",
					},
				},
			},
			validatorCountCalled: 1,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active",
					Count:  10,
				},
			},
		},
		{
			name: "not supported beacon node",
			versionResponse: node.GetVersionResponse{
				Data: &node.Version{Version: "lighthouse/v0.0.1"},
			},
			expectedError: "endpoint not supported",
		},
		{
			name:                 "fails to get version",
			versionEndpointError: errors.New("foo error"),
			expectedError:        "failed to get node version",
		},
		{
			name: "fails to get validator count",
			versionResponse: node.GetVersionResponse{
				Data: &node.Version{Version: nodeVersion},
			},
			validatorCountEndpointError: errors.New("foo error"),
			validatorCountCalled:        1,
			expectedError:               "foo error",
		},
		{
			name: "nil validator count data",
			versionResponse: node.GetVersionResponse{
				Data: &node.Version{Version: nodeVersion},
			},
			validatorCountResponse: validator2.CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data:                nil,
			},
			validatorCountCalled: 1,
			expectedError:        "validator count data is nil",
		},
		{
			name: "invalid validator count",
			versionResponse: node.GetVersionResponse{
				Data: &node.Version{Version: nodeVersion},
			},
			validatorCountResponse: validator2.CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*validator2.Count{
					{
						Status: "active",
						Count:  "10",
					},
					{
						Status: "exited",
						Count:  "10",
					},
				},
			},
			validatorCountCalled: 1,
			expectedError:        "mismatch between validator count data and the number of statuses provided",
		},
	}

	for _, test := range testCases {
		t.Run(test.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			ctx := context.Background()
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

			// Expect node version endpoint call.
			var nodeVersionResponse node.GetVersionResponse
			jsonRestHandler.EXPECT().Get(
				ctx,
				"/eth/v1/node/version",
				&nodeVersionResponse,
			).Return(
				test.versionEndpointError,
			).SetArg(
				2,
				test.versionResponse,
			)

			var validatorCountResponse validator2.CountResponse
			jsonRestHandler.EXPECT().Get(
				ctx,
				"/eth/v1/beacon/states/head/validator_count?status=active",
				&validatorCountResponse,
			).Return(
				test.validatorCountEndpointError,
			).SetArg(
				2,
				test.validatorCountResponse,
			).Times(test.validatorCountCalled)

			// Type assertion.
			var client iface.PrysmBeaconChainClient = &prysmBeaconChainClient{
				nodeClient:      &beaconApiNodeClient{jsonRestHandler: jsonRestHandler},
				jsonRestHandler: jsonRestHandler,
			}

			countResponse, err := client.GetValidatorCount(ctx, "head", []validator.Status{validator.Active})

			if len(test.expectedResponse) == 0 {
				require.ErrorContains(t, test.expectedError, err)
			} else {
				require.NoError(t, err)
				require.DeepEqual(t, test.expectedResponse, countResponse)
			}
		})
	}

}
