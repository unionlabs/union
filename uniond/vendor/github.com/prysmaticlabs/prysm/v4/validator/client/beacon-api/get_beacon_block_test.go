package beacon_api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	test_helpers "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/test-helpers"
)

func TestGetBeaconBlock_RequestFailed(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		gomock.Any(),
		gomock.Any(),
	).Return(
		errors.New("foo error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	_, err := validatorClient.getBeaconBlock(ctx, 1, []byte{1}, []byte{2})
	assert.ErrorContains(t, "foo error", err)
}

func TestGetBeaconBlock_Error(t *testing.T) {
	testCases := []struct {
		name                 string
		beaconBlock          interface{}
		expectedErrorMessage string
		consensusVersion     string
		blinded              bool
		data                 json.RawMessage
	}{
		{
			name:                 "phase0 block decoding failed",
			expectedErrorMessage: "failed to decode phase0 block response json",
			consensusVersion:     "phase0",
			data:                 []byte{},
		},
		{
			name:                 "altair block decoding failed",
			expectedErrorMessage: "failed to decode altair block response json",
			consensusVersion:     "altair",
			data:                 []byte{},
		},
		{
			name:                 "bellatrix block decoding failed",
			expectedErrorMessage: "failed to decode bellatrix block response json",
			beaconBlock:          "foo",
			consensusVersion:     "bellatrix",
			blinded:              false,
			data:                 []byte{},
		},
		{
			name:                 "blinded bellatrix block decoding failed",
			expectedErrorMessage: "failed to decode bellatrix block response json",
			beaconBlock:          "foo",
			consensusVersion:     "bellatrix",
			blinded:              true,
			data:                 []byte{},
		},
		{
			name:                 "capella block decoding failed",
			expectedErrorMessage: "failed to decode capella block response json",
			beaconBlock:          "foo",
			consensusVersion:     "capella",
			blinded:              false,
			data:                 []byte{},
		},
		{
			name:                 "blinded capella block decoding failed",
			expectedErrorMessage: "failed to decode capella block response json",
			beaconBlock:          "foo",
			consensusVersion:     "capella",
			blinded:              true,
			data:                 []byte{},
		},
		{
			name:                 "deneb block decoding failed",
			expectedErrorMessage: "failed to decode deneb block response json",
			beaconBlock:          "foo",
			consensusVersion:     "deneb",
			blinded:              false,
			data:                 []byte{},
		},
		{
			name:                 "blinded deneb block decoding failed",
			expectedErrorMessage: "failed to decode deneb block response json",
			beaconBlock:          "foo",
			consensusVersion:     "deneb",
			blinded:              true,
			data:                 []byte{},
		},
		{
			name:                 "unsupported consensus version",
			expectedErrorMessage: "unsupported consensus version `foo`",
			consensusVersion:     "foo",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			ctx := context.Background()

			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
			jsonRestHandler.EXPECT().Get(
				ctx,
				gomock.Any(),
				&validator.ProduceBlockV3Response{},
			).SetArg(
				2,
				validator.ProduceBlockV3Response{
					Version: testCase.consensusVersion,
					Data:    testCase.data,
				},
			).Return(
				nil,
			).Times(1)

			validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
			_, err := validatorClient.getBeaconBlock(ctx, 1, []byte{1}, []byte{2})
			assert.ErrorContains(t, testCase.expectedErrorMessage, err)
		})
	}
}

func TestGetBeaconBlock_Phase0Valid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoPhase0BeaconBlock()
	block := test_helpers.GenerateJsonPhase0BeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}
	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version: "phase0",
			Data:    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_Phase0{
			Phase0: proto,
		},
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_AltairValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoAltairBeaconBlock()
	block := test_helpers.GenerateJsonAltairBeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version: "altair",
			Data:    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_Altair{
			Altair: proto,
		},
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_BellatrixValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoBellatrixBeaconBlock()
	block := test_helpers.GenerateJsonBellatrixBeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version:                 "bellatrix",
			ExecutionPayloadBlinded: false,
			Data:                    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_Bellatrix{
			Bellatrix: proto,
		},
		IsBlinded: false,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_BlindedBellatrixValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoBlindedBellatrixBeaconBlock()
	block := test_helpers.GenerateJsonBlindedBellatrixBeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version:                 "bellatrix",
			ExecutionPayloadBlinded: true,
			Data:                    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_BlindedBellatrix{
			BlindedBellatrix: proto,
		},
		IsBlinded: true,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_CapellaValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoCapellaBeaconBlock()
	block := test_helpers.GenerateJsonCapellaBeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version:                 "capella",
			ExecutionPayloadBlinded: false,
			Data:                    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_Capella{
			Capella: proto,
		},
		IsBlinded: false,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_BlindedCapellaValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoBlindedCapellaBeaconBlock()
	block := test_helpers.GenerateJsonBlindedCapellaBeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version:                 "capella",
			ExecutionPayloadBlinded: true,
			Data:                    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_BlindedCapella{
			BlindedCapella: proto,
		},
		IsBlinded: true,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_DenebValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoDenebBeaconBlockContents()
	block := test_helpers.GenerateJsonDenebBeaconBlockContents()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version:                 "deneb",
			ExecutionPayloadBlinded: false,
			Data:                    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_Deneb{
			Deneb: proto,
		},
		IsBlinded: false,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_BlindedDenebValid(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoBlindedDenebBeaconBlock()
	block := test_helpers.GenerateJsonBlindedDenebBeaconBlock()
	bytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).SetArg(
		2,
		validator.ProduceBlockV3Response{
			Version:                 "deneb",
			ExecutionPayloadBlinded: true,
			Data:                    bytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_BlindedDeneb{
			BlindedDeneb: proto,
		},
		IsBlinded: true,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_FallbackToBlindedBlock(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoBlindedDenebBeaconBlock()
	block := test_helpers.GenerateJsonBlindedDenebBeaconBlock()
	blockBytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).Return(
		&httputil.DefaultJsonError{Code: http.StatusNotFound},
	).Times(1)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v1/validator/blinded_blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&abstractProduceBlockResponseJson{},
	).SetArg(
		2,
		abstractProduceBlockResponseJson{
			Version: "deneb",
			Data:    blockBytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_BlindedDeneb{
			BlindedDeneb: proto,
		},
		IsBlinded: true,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}

func TestGetBeaconBlock_FallbackToFullBlock(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	proto := test_helpers.GenerateProtoDenebBeaconBlockContents()
	block := test_helpers.GenerateJsonDenebBeaconBlockContents()
	blockBytes, err := json.Marshal(block)
	require.NoError(t, err)

	const slot = primitives.Slot(1)
	randaoReveal := []byte{2}
	graffiti := []byte{3}

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v3/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&validator.ProduceBlockV3Response{},
	).Return(
		&httputil.DefaultJsonError{Code: http.StatusNotFound},
	).Times(1)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v1/validator/blinded_blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&abstractProduceBlockResponseJson{},
	).Return(
		&httputil.DefaultJsonError{Code: http.StatusInternalServerError},
	).Times(1)
	jsonRestHandler.EXPECT().Get(
		ctx,
		fmt.Sprintf("/eth/v2/validator/blocks/%d?graffiti=%s&randao_reveal=%s", slot, hexutil.Encode(graffiti), hexutil.Encode(randaoReveal)),
		&abstractProduceBlockResponseJson{},
	).SetArg(
		2,
		abstractProduceBlockResponseJson{
			Version: "deneb",
			Data:    blockBytes,
		},
	).Return(
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	beaconBlock, err := validatorClient.getBeaconBlock(ctx, slot, randaoReveal, graffiti)
	require.NoError(t, err)

	expectedBeaconBlock := &ethpb.GenericBeaconBlock{
		Block: &ethpb.GenericBeaconBlock_Deneb{
			Deneb: proto,
		},
		IsBlinded: false,
	}

	assert.DeepEqual(t, expectedBeaconBlock, beaconBlock)
}
