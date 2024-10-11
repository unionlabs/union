package beacon_api

import (
	"context"
	"errors"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/config"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestGetGenesis(t *testing.T) {
	testCases := []struct {
		name                    string
		genesisResponse         *beacon.Genesis
		genesisError            error
		depositContractResponse config.GetDepositContractResponse
		depositContractError    error
		queriesDepositContract  bool
		expectedResponse        *ethpb.Genesis
		expectedError           string
	}{
		{
			name:          "fails to get genesis",
			genesisError:  errors.New("foo error"),
			expectedError: "failed to get genesis: foo error",
		},
		{
			name: "fails to decode genesis validator root",
			genesisResponse: &beacon.Genesis{
				GenesisTime:           "1",
				GenesisValidatorsRoot: "foo",
			},
			expectedError: "failed to decode genesis validator root `foo`",
		},
		{
			name: "fails to parse genesis time",
			genesisResponse: &beacon.Genesis{
				GenesisTime:           "foo",
				GenesisValidatorsRoot: hexutil.Encode([]byte{1}),
			},
			expectedError: "failed to parse genesis time `foo`",
		},
		{
			name: "fails to query contract information",
			genesisResponse: &beacon.Genesis{
				GenesisTime:           "1",
				GenesisValidatorsRoot: hexutil.Encode([]byte{2}),
			},
			depositContractError:   errors.New("foo error"),
			queriesDepositContract: true,
			expectedError:          "foo error",
		},
		{
			name: "fails to read nil deposit contract data",
			genesisResponse: &beacon.Genesis{
				GenesisTime:           "1",
				GenesisValidatorsRoot: hexutil.Encode([]byte{2}),
			},
			queriesDepositContract: true,
			depositContractResponse: config.GetDepositContractResponse{
				Data: nil,
			},
			expectedError: "deposit contract data is nil",
		},
		{
			name: "fails to decode deposit contract address",
			genesisResponse: &beacon.Genesis{
				GenesisTime:           "1",
				GenesisValidatorsRoot: hexutil.Encode([]byte{2}),
			},
			queriesDepositContract: true,
			depositContractResponse: config.GetDepositContractResponse{
				Data: &config.DepositContractData{
					Address: "foo",
				},
			},
			expectedError: "failed to decode deposit contract address `foo`",
		},
		{
			name: "successfully retrieves genesis info",
			genesisResponse: &beacon.Genesis{
				GenesisTime:           "654812",
				GenesisValidatorsRoot: hexutil.Encode([]byte{2}),
			},
			queriesDepositContract: true,
			depositContractResponse: config.GetDepositContractResponse{
				Data: &config.DepositContractData{
					Address: hexutil.Encode([]byte{3}),
				},
			},
			expectedResponse: &ethpb.Genesis{
				GenesisTime: &timestamppb.Timestamp{
					Seconds: 654812,
				},
				DepositContractAddress: []byte{3},
				GenesisValidatorsRoot:  []byte{2},
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()
			ctx := context.Background()

			genesisProvider := mock.NewMockGenesisProvider(ctrl)
			genesisProvider.EXPECT().GetGenesis(
				ctx,
			).Return(
				testCase.genesisResponse,
				testCase.genesisError,
			)

			depositContractJson := config.GetDepositContractResponse{}
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

			if testCase.queriesDepositContract {
				jsonRestHandler.EXPECT().Get(
					ctx,
					"/eth/v1/config/deposit_contract",
					&depositContractJson,
				).Return(
					testCase.depositContractError,
				).SetArg(
					2,
					testCase.depositContractResponse,
				)
			}

			nodeClient := &beaconApiNodeClient{
				genesisProvider: genesisProvider,
				jsonRestHandler: jsonRestHandler,
			}
			response, err := nodeClient.GetGenesis(ctx, &emptypb.Empty{})

			if testCase.expectedResponse == nil {
				assert.ErrorContains(t, testCase.expectedError, err)
			} else {
				assert.DeepEqual(t, testCase.expectedResponse, response)
			}
		})
	}
}

func TestGetSyncStatus(t *testing.T) {
	const syncingEndpoint = "/eth/v1/node/syncing"

	testCases := []struct {
		name                 string
		restEndpointResponse node.SyncStatusResponse
		restEndpointError    error
		expectedResponse     *ethpb.SyncStatus
		expectedError        string
	}{
		{
			name:              "fails to query REST endpoint",
			restEndpointError: errors.New("foo error"),
			expectedError:     "foo error",
		},
		{
			name:                 "returns nil syncing data",
			restEndpointResponse: node.SyncStatusResponse{Data: nil},
			expectedError:        "syncing data is nil",
		},
		{
			name: "returns false syncing status",
			restEndpointResponse: node.SyncStatusResponse{
				Data: &node.SyncStatusResponseData{
					IsSyncing: false,
				},
			},
			expectedResponse: &ethpb.SyncStatus{
				Syncing: false,
			},
		},
		{
			name: "returns true syncing status",
			restEndpointResponse: node.SyncStatusResponse{
				Data: &node.SyncStatusResponseData{
					IsSyncing: true,
				},
			},
			expectedResponse: &ethpb.SyncStatus{
				Syncing: true,
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()
			ctx := context.Background()

			syncingResponse := node.SyncStatusResponse{}
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Get(
				ctx,
				syncingEndpoint,
				&syncingResponse,
			).Return(
				testCase.restEndpointError,
			).SetArg(
				2,
				testCase.restEndpointResponse,
			)

			nodeClient := &beaconApiNodeClient{jsonRestHandler: jsonRestHandler}
			syncStatus, err := nodeClient.GetSyncStatus(ctx, &emptypb.Empty{})

			if testCase.expectedResponse == nil {
				assert.ErrorContains(t, testCase.expectedError, err)
			} else {
				assert.DeepEqual(t, testCase.expectedResponse, syncStatus)
			}
		})
	}
}

func TestGetVersion(t *testing.T) {
	const versionEndpoint = "/eth/v1/node/version"

	testCases := []struct {
		name                 string
		restEndpointResponse node.GetVersionResponse
		restEndpointError    error
		expectedResponse     *ethpb.Version
		expectedError        string
	}{
		{
			name:              "fails to query REST endpoint",
			restEndpointError: errors.New("foo error"),
			expectedError:     "foo error",
		},
		{
			name:                 "returns nil version data",
			restEndpointResponse: node.GetVersionResponse{Data: nil},
			expectedError:        "empty version response",
		},
		{
			name: "returns proper version response",
			restEndpointResponse: node.GetVersionResponse{
				Data: &node.Version{
					Version: "prysm/local",
				},
			},
			expectedResponse: &ethpb.Version{
				Version: "prysm/local",
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()
			ctx := context.Background()

			var versionResponse node.GetVersionResponse
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Get(
				ctx,
				versionEndpoint,
				&versionResponse,
			).Return(
				testCase.restEndpointError,
			).SetArg(
				2,
				testCase.restEndpointResponse,
			)

			nodeClient := &beaconApiNodeClient{jsonRestHandler: jsonRestHandler}
			version, err := nodeClient.GetVersion(ctx, &emptypb.Empty{})

			if testCase.expectedResponse == nil {
				assert.ErrorContains(t, testCase.expectedError, err)
			} else {
				assert.DeepEqual(t, testCase.expectedResponse, version)
			}
		})
	}
}
