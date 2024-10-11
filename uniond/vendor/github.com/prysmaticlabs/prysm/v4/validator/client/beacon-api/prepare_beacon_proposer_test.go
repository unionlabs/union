package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

const prepareBeaconProposerTestEndpoint = "/eth/v1/validator/prepare_beacon_proposer"

func TestPrepareBeaconProposer_Valid(t *testing.T) {
	const feeRecipient1 = "0xca008b199c03a2a2f6bc2ed52d6404c4d8510b35"
	const feeRecipient2 = "0x8145d80111309e4621ed7632319664ac440b0198"
	const feeRecipient3 = "0x085f2adb1295821838910be402b3c8cdc118bd86"

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRecipients := []*shared.FeeRecipient{
		{
			ValidatorIndex: "1",
			FeeRecipient:   feeRecipient1,
		},
		{
			ValidatorIndex: "2",
			FeeRecipient:   feeRecipient2,
		},
		{
			ValidatorIndex: "3",
			FeeRecipient:   feeRecipient3,
		},
	}

	marshalledJsonRecipients, err := json.Marshal(jsonRecipients)
	require.NoError(t, err)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		prepareBeaconProposerTestEndpoint,
		nil,
		bytes.NewBuffer(marshalledJsonRecipients),
		nil,
	).Return(
		nil,
	).Times(1)

	decodedFeeRecipient1, err := hexutil.Decode(feeRecipient1)
	require.NoError(t, err)
	decodedFeeRecipient2, err := hexutil.Decode(feeRecipient2)
	require.NoError(t, err)
	decodedFeeRecipient3, err := hexutil.Decode(feeRecipient3)
	require.NoError(t, err)

	protoRecipients := []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
		{
			ValidatorIndex: 1,
			FeeRecipient:   decodedFeeRecipient1,
		},
		{
			ValidatorIndex: 2,
			FeeRecipient:   decodedFeeRecipient2,
		},
		{
			ValidatorIndex: 3,
			FeeRecipient:   decodedFeeRecipient3,
		},
	}

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	err = validatorClient.prepareBeaconProposer(ctx, protoRecipients)
	require.NoError(t, err)
}

func TestPrepareBeaconProposer_BadRequest(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		ctx,
		prepareBeaconProposerTestEndpoint,
		nil,
		gomock.Any(),
		nil,
	).Return(
		errors.New("foo error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	err := validatorClient.prepareBeaconProposer(ctx, nil)
	assert.ErrorContains(t, "foo error", err)
}
