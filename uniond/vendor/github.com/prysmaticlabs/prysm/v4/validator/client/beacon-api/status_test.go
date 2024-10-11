package beacon_api

import (
	"context"
	"errors"
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	validator2 "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/validator"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestValidatorStatus_Nominal(t *testing.T) {
	const stringValidatorPubKey = "0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463"
	validatorPubKey, err := hexutil.Decode(stringValidatorPubKey)
	require.NoError(t, err)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)

	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		[]string{stringValidatorPubKey},
		nil,
		nil,
	).Return(
		&beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{
				{
					Index:  "35000",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey:          stringValidatorPubKey,
						ActivationEpoch: "56",
					},
				},
			},
		},
		nil,
	).Times(1)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: stateValidatorsProvider,
		prysmBeaconChainCLient: prysmBeaconChainClient{
			nodeClient: &beaconApiNodeClient{
				jsonRestHandler: jsonRestHandler,
			},
		},
	}

	// Expect node version endpoint call.
	var nodeVersionResponse node.GetVersionResponse
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/node/version",
		&nodeVersionResponse,
	).Return(
		iface.ErrNotSupported,
	).Times(1)

	actualValidatorStatusResponse, err := validatorClient.ValidatorStatus(
		ctx,
		&ethpb.ValidatorStatusRequest{
			PublicKey: validatorPubKey,
		},
	)

	expectedValidatorStatusResponse := ethpb.ValidatorStatusResponse{
		Status:          ethpb.ValidatorStatus_ACTIVE,
		ActivationEpoch: 56,
	}

	require.NoError(t, err)
	assert.DeepEqual(t, &expectedValidatorStatusResponse, actualValidatorStatusResponse)
}

func TestValidatorStatus_Error(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)

	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		gomock.Any(),
		nil,
		nil,
	).Return(
		&beacon.GetValidatorsResponse{},
		errors.New("a specific error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{stateValidatorsProvider: stateValidatorsProvider}

	_, err := validatorClient.ValidatorStatus(
		ctx,
		&ethpb.ValidatorStatusRequest{
			PublicKey: []byte{},
		},
	)

	require.ErrorContains(t, "failed to get validator status response", err)
}

func TestMultipleValidatorStatus_Nominal(t *testing.T) {
	stringValidatorsPubKey := []string{
		"0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13", // existing
		"0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463", // existing
	}

	ctx := context.Background()
	validatorsPubKey := make([][]byte, len(stringValidatorsPubKey))

	for i, stringValidatorPubKey := range stringValidatorsPubKey {
		validatorPubKey, err := hexutil.Decode(stringValidatorPubKey)
		require.NoError(t, err)
		validatorsPubKey[i] = validatorPubKey
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)

	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		stringValidatorsPubKey,
		[]primitives.ValidatorIndex{},
		nil,
	).Return(
		&beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{
				{
					Index:  "11111",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey:          "0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13",
						ActivationEpoch: "12",
					},
				},
				{
					Index:  "22222",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey:          "0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463",
						ActivationEpoch: "34",
					},
				},
			},
		},
		nil,
	).Times(1)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	// Expect node version endpoint call.
	var nodeVersionResponse node.GetVersionResponse
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/node/version",
		&nodeVersionResponse,
	).Return(
		iface.ErrNotSupported,
	).Times(1)

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: stateValidatorsProvider,
		prysmBeaconChainCLient: prysmBeaconChainClient{
			nodeClient: &beaconApiNodeClient{
				jsonRestHandler: jsonRestHandler,
			},
		},
	}

	expectedValidatorStatusResponse := ethpb.MultipleValidatorStatusResponse{
		PublicKeys: validatorsPubKey,
		Indices: []primitives.ValidatorIndex{
			11111,
			22222,
		},
		Statuses: []*ethpb.ValidatorStatusResponse{
			{
				Status:          ethpb.ValidatorStatus_ACTIVE,
				ActivationEpoch: 12,
			},
			{
				Status:          ethpb.ValidatorStatus_ACTIVE,
				ActivationEpoch: 34,
			},
		},
	}

	actualValidatorStatusResponse, err := validatorClient.MultipleValidatorStatus(
		ctx,
		&ethpb.MultipleValidatorStatusRequest{
			PublicKeys: validatorsPubKey,
		},
	)
	require.NoError(t, err)
	assert.DeepEqual(t, &expectedValidatorStatusResponse, actualValidatorStatusResponse)
}

func TestMultipleValidatorStatus_Error(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)

	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		gomock.Any(),
		[]primitives.ValidatorIndex{},
		nil,
	).Return(
		&beacon.GetValidatorsResponse{},
		errors.New("a specific error"),
	).Times(1)

	validatorClient := beaconApiValidatorClient{stateValidatorsProvider: stateValidatorsProvider}

	_, err := validatorClient.MultipleValidatorStatus(
		ctx,
		&ethpb.MultipleValidatorStatusRequest{
			PublicKeys: [][]byte{},
		},
	)
	require.ErrorContains(t, "failed to get validators status response", err)
}

func TestGetValidatorsStatusResponse_Nominal_SomeActiveValidators(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	stringValidatorsPubKey := []string{
		"0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13", // existing
		"0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463", // existing
		"0x80000e851c0f53c3246ff726d7ff7766661ca5e12a07c45c114d208d54f0f8233d4380b2e9aff759d69795d1df905526", // NOT existing
		"0x8000ab56b051f9d8f31c687528c6e91c9b98e4c3a241e752f9ccfbea7c5a7fbbd272bdf2c0a7e52ce7e0b57693df364c", // existing
		"0x8000b3e51de7e2e319b23a42d468dc8e63cd61daa5c4609cf2d800026d92706d1240414e155057bdc35e0574bba3ad80", // NOT existing
		"0x800010c20716ef4264a6d93b3873a008ece58fb9312ac2cc3b0ccc40aedb050f2038281e6a92242a35476af9903c7919", // existing
	}

	validatorsPubKey := make([][]byte, len(stringValidatorsPubKey))

	for i, stringValidatorPubKey := range stringValidatorsPubKey {
		validatorPubKey, err := hexutil.Decode(stringValidatorPubKey)
		require.NoError(t, err)
		validatorsPubKey[i] = validatorPubKey
	}

	validatorsIndex := []primitives.ValidatorIndex{
		12345, // NOT existing
		33333, // existing
	}

	extraStringValidatorKey := "0x80003eb1e78ffdea6c878026b7074f84aaa16536c8e1960a652e817c848e7ccb051087f837b7d2bb6773cd9705601ede"

	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)

	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		stringValidatorsPubKey,
		validatorsIndex,
		nil,
	).Return(
		&beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{
				{
					Index:  "11111",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey:          "0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13",
						ActivationEpoch: "12",
					},
				},
				{
					Index:  "22222",
					Status: "active_exiting",
					Validator: &beacon.Validator{
						Pubkey:          "0x800010c20716ef4264a6d93b3873a008ece58fb9312ac2cc3b0ccc40aedb050f2038281e6a92242a35476af9903c7919",
						ActivationEpoch: "34",
					},
				},
				{
					Index:  "33333",
					Status: "active_ongoing",
					Validator: &beacon.Validator{
						Pubkey:          extraStringValidatorKey,
						ActivationEpoch: "56",
					},
				},
				{
					Index:  "40000",
					Status: "pending_queued",
					Validator: &beacon.Validator{
						Pubkey:          "0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463",
						ActivationEpoch: fmt.Sprintf("%d", params.BeaconConfig().FarFutureEpoch),
					},
				},
				{
					Index:  "50000",
					Status: "pending_queued",
					Validator: &beacon.Validator{
						Pubkey:          "0x8000ab56b051f9d8f31c687528c6e91c9b98e4c3a241e752f9ccfbea7c5a7fbbd272bdf2c0a7e52ce7e0b57693df364c",
						ActivationEpoch: fmt.Sprintf("%d", params.BeaconConfig().FarFutureEpoch),
					},
				},
			},
		},
		nil,
	).Times(1)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	// Expect node version endpoint call.
	var nodeVersionResponse node.GetVersionResponse
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/node/version",
		&nodeVersionResponse,
	).Return(
		nil,
	).SetArg(
		2,
		node.GetVersionResponse{Data: &node.Version{Version: "prysm/v0.0.1"}},
	).Times(1)

	var validatorCountResponse validator2.CountResponse
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/beacon/states/head/validator_count?",
		&validatorCountResponse,
	).Return(
		nil,
	).SetArg(
		2,
		validator2.CountResponse{
			Data: []*validator2.Count{
				{
					Status: "active",
					Count:  "50001",
				},
				{
					Status: "pending",
					Count:  "11000",
				},
			},
		},
	).Times(1)

	wantedStringValidatorsPubkey := []string{
		"0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13", // existing
		"0x800010c20716ef4264a6d93b3873a008ece58fb9312ac2cc3b0ccc40aedb050f2038281e6a92242a35476af9903c7919", // existing,
		extraStringValidatorKey, // existing,
		"0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463", // existing,
		"0x8000ab56b051f9d8f31c687528c6e91c9b98e4c3a241e752f9ccfbea7c5a7fbbd272bdf2c0a7e52ce7e0b57693df364c", // existing
		"0x80000e851c0f53c3246ff726d7ff7766661ca5e12a07c45c114d208d54f0f8233d4380b2e9aff759d69795d1df905526", // NOT existing
		"0x8000b3e51de7e2e319b23a42d468dc8e63cd61daa5c4609cf2d800026d92706d1240414e155057bdc35e0574bba3ad80", // NOT existing
	}

	wantedValidatorsPubKey := make([][]byte, len(wantedStringValidatorsPubkey))
	for i, stringValidatorPubKey := range wantedStringValidatorsPubkey {
		validatorPubKey, err := hexutil.Decode(stringValidatorPubKey)
		require.NoError(t, err)

		wantedValidatorsPubKey[i] = validatorPubKey
	}

	wantedValidatorsIndex := []primitives.ValidatorIndex{
		11111,
		22222,
		33333,
		40000,
		50000,
		primitives.ValidatorIndex(^uint64(0)),
		primitives.ValidatorIndex(^uint64(0)),
	}

	wantedValidatorsStatusResponse := []*ethpb.ValidatorStatusResponse{
		{
			Status:          ethpb.ValidatorStatus_ACTIVE,
			ActivationEpoch: 12,
		},
		{
			Status:          ethpb.ValidatorStatus_EXITING,
			ActivationEpoch: 34,
		},
		{
			Status:          ethpb.ValidatorStatus_ACTIVE,
			ActivationEpoch: 56,
		},
		{
			Status:                    ethpb.ValidatorStatus_PENDING,
			ActivationEpoch:           params.BeaconConfig().FarFutureEpoch,
			PositionInActivationQueue: 1000,
		},
		{
			Status:                    ethpb.ValidatorStatus_PENDING,
			ActivationEpoch:           params.BeaconConfig().FarFutureEpoch,
			PositionInActivationQueue: 11000,
		},
		{
			Status:          ethpb.ValidatorStatus_UNKNOWN_STATUS,
			ActivationEpoch: params.BeaconConfig().FarFutureEpoch,
		},
		{
			Status:          ethpb.ValidatorStatus_UNKNOWN_STATUS,
			ActivationEpoch: params.BeaconConfig().FarFutureEpoch,
		},
	}

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: stateValidatorsProvider,
		prysmBeaconChainCLient: prysmBeaconChainClient{
			nodeClient: &beaconApiNodeClient{
				jsonRestHandler: jsonRestHandler,
			},
			jsonRestHandler: jsonRestHandler,
		},
	}
	actualValidatorsPubKey, actualValidatorsIndex, actualValidatorsStatusResponse, err := validatorClient.getValidatorsStatusResponse(ctx, validatorsPubKey, validatorsIndex)

	require.NoError(t, err)
	assert.DeepEqual(t, wantedValidatorsPubKey, actualValidatorsPubKey)
	assert.DeepEqual(t, wantedValidatorsIndex, actualValidatorsIndex)
	assert.DeepEqual(t, wantedValidatorsStatusResponse, actualValidatorsStatusResponse)
}

func TestGetValidatorsStatusResponse_Nominal_NoActiveValidators(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	const stringValidatorPubKey = "0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463"
	validatorPubKey, err := hexutil.Decode(stringValidatorPubKey)
	require.NoError(t, err)

	ctx := context.Background()
	stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)

	stateValidatorsProvider.EXPECT().GetStateValidators(
		ctx,
		[]string{stringValidatorPubKey},
		nil,
		nil,
	).Return(
		&beacon.GetValidatorsResponse{
			Data: []*beacon.ValidatorContainer{
				{
					Index:  "40000",
					Status: "pending_queued",
					Validator: &beacon.Validator{
						Pubkey:          "0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463",
						ActivationEpoch: fmt.Sprintf("%d", params.BeaconConfig().FarFutureEpoch),
					},
				},
			},
		},
		nil,
	).Times(1)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	// Expect node version endpoint call.
	var nodeVersionResponse node.GetVersionResponse
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/node/version",
		&nodeVersionResponse,
	).Return(
		iface.ErrNotSupported,
	).Times(1)

	wantedValidatorsPubKey := [][]byte{validatorPubKey}
	wantedValidatorsIndex := []primitives.ValidatorIndex{40000}
	wantedValidatorsStatusResponse := []*ethpb.ValidatorStatusResponse{
		{
			Status:          ethpb.ValidatorStatus_PENDING,
			ActivationEpoch: params.BeaconConfig().FarFutureEpoch,
		},
	}

	validatorClient := beaconApiValidatorClient{
		stateValidatorsProvider: stateValidatorsProvider,
		prysmBeaconChainCLient: prysmBeaconChainClient{
			nodeClient: &beaconApiNodeClient{
				jsonRestHandler: jsonRestHandler,
			},
			jsonRestHandler: jsonRestHandler,
		},
	}
	actualValidatorsPubKey, actualValidatorsIndex, actualValidatorsStatusResponse, err := validatorClient.getValidatorsStatusResponse(ctx, wantedValidatorsPubKey, nil)

	require.NoError(t, err)
	require.NoError(t, err)
	assert.DeepEqual(t, wantedValidatorsPubKey, actualValidatorsPubKey)
	assert.DeepEqual(t, wantedValidatorsIndex, actualValidatorsIndex)
	assert.DeepEqual(t, wantedValidatorsStatusResponse, actualValidatorsStatusResponse)
}

type getStateValidatorsInterface struct {
	// Inputs
	inputStringPubKeys []string
	inputIndexes       []primitives.ValidatorIndex
	inputStatuses      []string

	// Outputs
	outputStateValidatorsResponseJson *beacon.GetValidatorsResponse
	outputErr                         error
}

func TestValidatorStatusResponse_InvalidData(t *testing.T) {
	stringPubKey := "0x8000a6c975761b488bdb0dfba4ed37c0d97d6e6b968562ef5c84aa9a5dfb92d8e309195004e97709077723739bf04463"
	pubKey, err := hexutil.Decode(stringPubKey)
	require.NoError(t, err)

	testCases := []struct {
		name string

		// Inputs
		inputPubKeys                     [][]byte
		inputIndexes                     []primitives.ValidatorIndex
		inputGetStateValidatorsInterface getStateValidatorsInterface
		validatorCountCalled             int

		// Outputs
		outputErrMessage string
	}{
		{
			name:         "failed getStateValidators",
			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{
				inputStringPubKeys:                []string{stringPubKey},
				outputStateValidatorsResponseJson: &beacon.GetValidatorsResponse{},
				outputErr:                         errors.New("a specific error"),
			},
			outputErrMessage: "failed to get state validators",
		},
		{
			name: "failed to parse validator public key NotAPublicKey",

			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{
				inputStringPubKeys: []string{stringPubKey},
				inputIndexes:       nil,
				inputStatuses:      nil,

				outputStateValidatorsResponseJson: &beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index: "0",
							Validator: &beacon.Validator{
								Pubkey: "NotAPublicKey",
							},
						},
					},
				},
				outputErr: nil,
			},
			validatorCountCalled: 1,
			outputErrMessage:     "failed to parse validator public key",
		},
		{
			name: "failed to parse validator index NotAnIndex",

			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{

				inputStringPubKeys: []string{stringPubKey},
				inputIndexes:       nil,
				inputStatuses:      nil,

				outputStateValidatorsResponseJson: &beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index: "NotAnIndex",
							Validator: &beacon.Validator{
								Pubkey: stringPubKey,
							},
						},
					},
				},
				outputErr: nil,
			},
			validatorCountCalled: 1,
			outputErrMessage:     "failed to parse validator index",
		},
		{
			name: "invalid validator status",

			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{
				inputStringPubKeys: []string{stringPubKey},
				inputIndexes:       nil,
				inputStatuses:      nil,

				outputStateValidatorsResponseJson: &beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index:  "12345",
							Status: "NotAStatus",
							Validator: &beacon.Validator{
								Pubkey: stringPubKey,
							},
						},
					},
				},
				outputErr: nil,
			},
			validatorCountCalled: 1,
			outputErrMessage:     "invalid validator status NotAStatus",
		},
		{
			name: "failed to parse activation epoch",

			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{
				inputStringPubKeys: []string{stringPubKey},
				inputIndexes:       nil,
				inputStatuses:      nil,

				outputStateValidatorsResponseJson: &beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index:  "12345",
							Status: "active_ongoing",
							Validator: &beacon.Validator{
								Pubkey:          stringPubKey,
								ActivationEpoch: "NotAnEpoch",
							},
						},
					},
				},
				outputErr: nil,
			},
			validatorCountCalled: 1,
			outputErrMessage:     "failed to parse activation epoch NotAnEpoch",
		},
		{
			name: "failed to get state validators",

			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{
				inputStringPubKeys: []string{stringPubKey},
				inputIndexes:       nil,
				inputStatuses:      nil,

				outputStateValidatorsResponseJson: nil,
				outputErr:                         errors.New("a specific error"),
			},
			outputErrMessage: "failed to get state validators",
		},
		{
			name: "failed to parse last validator index",

			inputPubKeys: [][]byte{pubKey},
			inputIndexes: nil,
			inputGetStateValidatorsInterface: getStateValidatorsInterface{
				inputStringPubKeys: []string{stringPubKey},
				inputIndexes:       nil,
				inputStatuses:      nil,

				outputStateValidatorsResponseJson: &beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index: "NotAnIndex",
						},
					},
				},
				outputErr: nil,
			},
			validatorCountCalled: 1,
			outputErrMessage:     "failed to parse validator index NotAnIndex",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name,
			func(t *testing.T) {
				ctrl := gomock.NewController(t)
				defer ctrl.Finish()

				ctx := context.Background()
				stateValidatorsProvider := mock.NewMockStateValidatorsProvider(ctrl)
				stateValidatorsProvider.EXPECT().GetStateValidators(
					ctx,
					testCase.inputGetStateValidatorsInterface.inputStringPubKeys,
					testCase.inputGetStateValidatorsInterface.inputIndexes,
					testCase.inputGetStateValidatorsInterface.inputStatuses,
				).Return(
					testCase.inputGetStateValidatorsInterface.outputStateValidatorsResponseJson,
					testCase.inputGetStateValidatorsInterface.outputErr,
				).Times(1)

				jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

				// Expect node version endpoint call.
				var nodeVersionResponse node.GetVersionResponse
				jsonRestHandler.EXPECT().Get(
					ctx,
					"/eth/v1/node/version",
					&nodeVersionResponse,
				).Return(
					iface.ErrNotSupported,
				).Times(testCase.validatorCountCalled)

				validatorClient := beaconApiValidatorClient{
					stateValidatorsProvider: stateValidatorsProvider,
					prysmBeaconChainCLient: prysmBeaconChainClient{
						nodeClient: &beaconApiNodeClient{
							jsonRestHandler: jsonRestHandler,
						},
						jsonRestHandler: jsonRestHandler,
					},
				}

				_, _, _, err := validatorClient.getValidatorsStatusResponse(
					ctx,
					testCase.inputPubKeys,
					testCase.inputIndexes,
				)

				assert.ErrorContains(t, testCase.outputErrMessage, err)
			},
		)
	}
}
