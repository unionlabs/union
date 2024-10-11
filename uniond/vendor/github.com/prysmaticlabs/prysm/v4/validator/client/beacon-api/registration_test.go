package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/golang/protobuf/ptypes/empty"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestRegistration_Valid(t *testing.T) {
	const feeRecipient1 = "0xca008b199c03a2a2f6bc2ed52d6404c4d8510b35"
	const pubKey1 = "0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13"
	const signature1 = "0xb459ef852bd4e0cb96e6723d67cacc8215406dd9ba663f8874a083167ebf428b28b746431bdbc1820a25289377b2610881e52b3a05c3548c5e99c08c8a36342573be5962d7510c03dcba8ddfb8ae419e59d222ddcf31cc512e704ef2cc3cf8"

	const feeRecipient2 = "0x8145d80111309e4621ed7632319664ac440b0198"
	const pubKey2 = "0x800015473bdc3a7f45ef8eb8abc598bc20021e55ad6e6ad1d745aaef9730dd2c28ec08bf42df18451de94dd4a6d24ec5"
	const signature2 = "0xc459ef852bd4e0cb96e6723d67cacc8215406dd9ba663f8874a083167ebf428b28b746431bdbc1820a25289377b2610881e52b3a05c3548c5e99c08c8a36342573be5962d7510c03dcba8ddfb8ae419e59d222ddcf31cc512e704ef2cc3cf8"

	const feeRecipient3 = "0x085f2adb1295821838910be402b3c8cdc118bd86"
	const pubKey3 = "0x80006dbd87090ce8d611ffb8d2c700901d7a07a73607e18c6dc5ff39e44dae317816387b61fa9008de3cbe07583c0358"
	const signature3 = "0xd459ef852bd4e0cb96e6723d67cacc8215406dd9ba663f8874a083167ebf428b28b746431bdbc1820a25289377b2610881e52b3a05c3548c5e99c08c8a36342573be5962d7510c03dcba8ddfb8ae419e59d222ddcf31cc512e704ef2cc3cf8"

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRegistrations := []*shared.SignedValidatorRegistration{
		{
			Message: &shared.ValidatorRegistration{
				FeeRecipient: feeRecipient1,
				GasLimit:     "100",
				Timestamp:    "1000",
				Pubkey:       pubKey1,
			},
			Signature: signature1,
		},
		{
			Message: &shared.ValidatorRegistration{
				FeeRecipient: feeRecipient2,
				GasLimit:     "200",
				Timestamp:    "2000",
				Pubkey:       pubKey2,
			},
			Signature: signature2,
		},
		{
			Message: &shared.ValidatorRegistration{
				FeeRecipient: feeRecipient3,
				GasLimit:     "300",
				Timestamp:    "3000",
				Pubkey:       pubKey3,
			},
			Signature: signature3,
		},
	}

	marshalledJsonRegistrations, err := json.Marshal(jsonRegistrations)
	require.NoError(t, err)

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		context.Background(),
		"/eth/v1/validator/register_validator",
		nil,
		bytes.NewBuffer(marshalledJsonRegistrations),
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

	decodedPubkey1, err := hexutil.Decode(pubKey1)
	require.NoError(t, err)
	decodedPubkey2, err := hexutil.Decode(pubKey2)
	require.NoError(t, err)
	decodedPubkey3, err := hexutil.Decode(pubKey3)
	require.NoError(t, err)

	decodedSignature1, err := hexutil.Decode(signature1)
	require.NoError(t, err)
	decodedSignature2, err := hexutil.Decode(signature2)
	require.NoError(t, err)
	decodedSignature3, err := hexutil.Decode(signature3)
	require.NoError(t, err)

	protoRegistrations := ethpb.SignedValidatorRegistrationsV1{
		Messages: []*ethpb.SignedValidatorRegistrationV1{
			{
				Message: &ethpb.ValidatorRegistrationV1{
					FeeRecipient: decodedFeeRecipient1,
					GasLimit:     100,
					Timestamp:    1000,
					Pubkey:       decodedPubkey1,
				},
				Signature: decodedSignature1,
			},
			{
				Message: &ethpb.ValidatorRegistrationV1{
					FeeRecipient: decodedFeeRecipient2,
					GasLimit:     200,
					Timestamp:    2000,
					Pubkey:       decodedPubkey2,
				},
				Signature: decodedSignature2,
			},
			{
				Message: &ethpb.ValidatorRegistrationV1{
					FeeRecipient: decodedFeeRecipient3,
					GasLimit:     300,
					Timestamp:    3000,
					Pubkey:       decodedPubkey3,
				},
				Signature: decodedSignature3,
			},
		},
	}

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	res, err := validatorClient.SubmitValidatorRegistrations(context.Background(), &protoRegistrations)

	assert.DeepEqual(t, new(empty.Empty), res)
	require.NoError(t, err)
}

func TestRegistration_BadRequest(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Post(
		context.Background(),
		"/eth/v1/validator/register_validator",
		nil,
		gomock.Any(),
		nil,
	).Return(
		errors.New("foo error"),
	).Times(1)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	_, err := validatorClient.SubmitValidatorRegistrations(context.Background(), &ethpb.SignedValidatorRegistrationsV1{})
	assert.ErrorContains(t, "foo error", err)
}
