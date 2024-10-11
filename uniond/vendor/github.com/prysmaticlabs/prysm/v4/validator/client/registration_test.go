package client

import (
	"context"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSubmitValidatorRegistrations(t *testing.T) {
	_, m, validatorKey, finish := setup(t)
	defer finish()

	ctx := context.Background()
	validatorRegsBatchSize := 2
	require.NoError(t, nil, SubmitValidatorRegistrations(ctx, m.validatorClient, []*ethpb.SignedValidatorRegistrationV1{}, validatorRegsBatchSize))

	regs := [...]*ethpb.ValidatorRegistrationV1{
		{
			FeeRecipient: bytesutil.PadTo([]byte("fee"), 20),
			GasLimit:     123,
			Timestamp:    uint64(time.Now().Unix()),
			Pubkey:       validatorKey.PublicKey().Marshal(),
		},
		{
			FeeRecipient: bytesutil.PadTo([]byte("fee"), 20),
			GasLimit:     456,
			Timestamp:    uint64(time.Now().Unix()),
			Pubkey:       validatorKey.PublicKey().Marshal(),
		},
		{
			FeeRecipient: bytesutil.PadTo([]byte("fee"), 20),
			GasLimit:     789,
			Timestamp:    uint64(time.Now().Unix()),
			Pubkey:       validatorKey.PublicKey().Marshal(),
		},
	}

	gomock.InOrder(
		m.validatorClient.EXPECT().
			SubmitValidatorRegistrations(gomock.Any(), &ethpb.SignedValidatorRegistrationsV1{
				Messages: []*ethpb.SignedValidatorRegistrationV1{
					{
						Message:   regs[0],
						Signature: params.BeaconConfig().ZeroHash[:],
					},
					{
						Message:   regs[1],
						Signature: params.BeaconConfig().ZeroHash[:],
					},
				},
			}).
			Return(nil, nil),

		m.validatorClient.EXPECT().
			SubmitValidatorRegistrations(gomock.Any(), &ethpb.SignedValidatorRegistrationsV1{
				Messages: []*ethpb.SignedValidatorRegistrationV1{
					{
						Message:   regs[2],
						Signature: params.BeaconConfig().ZeroHash[:],
					},
				},
			}).
			Return(nil, nil),
	)

	require.NoError(t, nil, SubmitValidatorRegistrations(
		ctx, m.validatorClient,
		[]*ethpb.SignedValidatorRegistrationV1{
			{
				Message:   regs[0],
				Signature: params.BeaconConfig().ZeroHash[:],
			},
			{
				Message:   regs[1],
				Signature: params.BeaconConfig().ZeroHash[:],
			},
			{
				Message:   regs[2],
				Signature: params.BeaconConfig().ZeroHash[:],
			},
		},
		validatorRegsBatchSize,
	))
}

func TestSubmitValidatorRegistration_CantSign(t *testing.T) {
	_, m, validatorKey, finish := setup(t)
	defer finish()

	ctx := context.Background()
	validatorRegsBatchSize := 500
	reg := &ethpb.ValidatorRegistrationV1{
		FeeRecipient: bytesutil.PadTo([]byte("fee"), 20),
		GasLimit:     123456,
		Timestamp:    uint64(time.Now().Unix()),
		Pubkey:       validatorKey.PublicKey().Marshal(),
	}

	m.validatorClient.EXPECT().
		SubmitValidatorRegistrations(gomock.Any(), &ethpb.SignedValidatorRegistrationsV1{
			Messages: []*ethpb.SignedValidatorRegistrationV1{
				{Message: reg,
					Signature: params.BeaconConfig().ZeroHash[:]},
			},
		}).
		Return(nil, errors.New("could not sign"))
	require.ErrorContains(t, "could not sign", SubmitValidatorRegistrations(ctx, m.validatorClient, []*ethpb.SignedValidatorRegistrationV1{
		{Message: reg,
			Signature: params.BeaconConfig().ZeroHash[:]},
	}, validatorRegsBatchSize))
}

func Test_signValidatorRegistration(t *testing.T) {
	_, m, validatorKey, finish := setup(t)
	defer finish()

	ctx := context.Background()
	reg := &ethpb.ValidatorRegistrationV1{
		FeeRecipient: bytesutil.PadTo([]byte("fee"), 20),
		GasLimit:     123456,
		Timestamp:    uint64(time.Now().Unix()),
		Pubkey:       validatorKey.PublicKey().Marshal(),
	}
	_, err := signValidatorRegistration(ctx, m.signfunc, reg)
	require.NoError(t, err)

}

func TestValidator_SignValidatorRegistrationRequest(t *testing.T) {
	_, m, validatorKey, finish := setup(t)
	defer finish()
	ctx := context.Background()
	byteval, err := hexutil.Decode("0x878705ba3f8bc32fcf7f4caa1a35e72af65cf766")
	require.NoError(t, err)
	tests := []struct {
		name            string
		arg             *ethpb.ValidatorRegistrationV1
		validatorSetter func(t *testing.T) *validator
		isCached        bool
		err             string
	}{
		{
			name: " Happy Path cached",
			arg: &ethpb.ValidatorRegistrationV1{
				Pubkey:       validatorKey.PublicKey().Marshal(),
				FeeRecipient: make([]byte, fieldparams.FeeRecipientLength),
				GasLimit:     30000000,
				Timestamp:    uint64(time.Now().Unix()),
			},
			validatorSetter: func(t *testing.T) *validator {
				v := validator{
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					genesisTime:                  0,
				}
				v.signedValidatorRegistrations[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())] = &ethpb.SignedValidatorRegistrationV1{
					Message: &ethpb.ValidatorRegistrationV1{
						Pubkey:       validatorKey.PublicKey().Marshal(),
						GasLimit:     30000000,
						FeeRecipient: make([]byte, fieldparams.FeeRecipientLength),
						Timestamp:    uint64(time.Now().Unix()),
					},
					Signature: make([]byte, 0),
				}
				return &v
			},
			isCached: true,
		},
		{
			name: " Happy Path not cached gas updated",
			arg: &ethpb.ValidatorRegistrationV1{
				Pubkey:       validatorKey.PublicKey().Marshal(),
				FeeRecipient: make([]byte, fieldparams.FeeRecipientLength),
				GasLimit:     30000000,
				Timestamp:    uint64(time.Now().Unix()),
			},
			validatorSetter: func(t *testing.T) *validator {
				v := validator{
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					genesisTime:                  0,
				}
				v.signedValidatorRegistrations[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())] = &ethpb.SignedValidatorRegistrationV1{
					Message: &ethpb.ValidatorRegistrationV1{
						Pubkey:       validatorKey.PublicKey().Marshal(),
						GasLimit:     35000000,
						FeeRecipient: make([]byte, fieldparams.FeeRecipientLength),
						Timestamp:    uint64(time.Now().Unix() - 1),
					},
					Signature: make([]byte, 0),
				}
				return &v
			},
			isCached: false,
		},
		{
			name: " Happy Path not cached feerecipient updated",
			arg: &ethpb.ValidatorRegistrationV1{
				Pubkey:       validatorKey.PublicKey().Marshal(),
				FeeRecipient: byteval,
				GasLimit:     30000000,
				Timestamp:    uint64(time.Now().Unix()),
			},
			validatorSetter: func(t *testing.T) *validator {
				v := validator{
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					genesisTime:                  0,
				}
				v.signedValidatorRegistrations[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())] = &ethpb.SignedValidatorRegistrationV1{
					Message: &ethpb.ValidatorRegistrationV1{
						Pubkey:       validatorKey.PublicKey().Marshal(),
						GasLimit:     30000000,
						FeeRecipient: make([]byte, fieldparams.FeeRecipientLength),
						Timestamp:    uint64(time.Now().Unix() - 1),
					},
					Signature: make([]byte, 0),
				}
				return &v
			},
			isCached: false,
		},
		{
			name: " Happy Path not cached first Entry",
			arg: &ethpb.ValidatorRegistrationV1{
				Pubkey:       validatorKey.PublicKey().Marshal(),
				FeeRecipient: byteval,
				GasLimit:     30000000,
				Timestamp:    uint64(time.Now().Unix()),
			},
			validatorSetter: func(t *testing.T) *validator {
				v := validator{
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					genesisTime:                  0,
				}
				return &v
			},
			isCached: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			v := tt.validatorSetter(t)

			startingReq, ok := v.signedValidatorRegistrations[bytesutil.ToBytes48(tt.arg.Pubkey)]

			got, err := v.SignValidatorRegistrationRequest(ctx, m.signfunc, tt.arg)
			require.NoError(t, err)
			if tt.isCached {
				require.DeepEqual(t, got, v.signedValidatorRegistrations[bytesutil.ToBytes48(tt.arg.Pubkey)])
			} else {
				if ok {
					require.NotEqual(t, got.Message.Timestamp, startingReq.Message.Timestamp)
				}
				require.Equal(t, got.Message.Timestamp, tt.arg.Timestamp)
				require.Equal(t, got.Message.GasLimit, tt.arg.GasLimit)
				require.Equal(t, hexutil.Encode(got.Message.FeeRecipient), hexutil.Encode(tt.arg.FeeRecipient))
				require.DeepEqual(t, got, v.signedValidatorRegistrations[bytesutil.ToBytes48(tt.arg.Pubkey)])
			}
		})
	}
}

func TestChunkSignedValidatorRegistrationV1(t *testing.T) {
	tests := map[string]struct {
		regs      []*ethpb.SignedValidatorRegistrationV1
		chunkSize int
		expected  [][]*ethpb.SignedValidatorRegistrationV1
	}{
		"All buckets are full": {
			regs: []*ethpb.SignedValidatorRegistrationV1{
				{Signature: []byte("1")},
				{Signature: []byte("2")},
				{Signature: []byte("3")},
				{Signature: []byte("4")},
				{Signature: []byte("5")},
				{Signature: []byte("6")},
			},
			chunkSize: 3,
			expected: [][]*ethpb.SignedValidatorRegistrationV1{
				{
					{Signature: []byte("1")},
					{Signature: []byte("2")},
					{Signature: []byte("3")},
				},
				{
					{Signature: []byte("4")},
					{Signature: []byte("5")},
					{Signature: []byte("6")},
				},
			},
		},
		"Last bucket is not full": {
			regs: []*ethpb.SignedValidatorRegistrationV1{
				{Signature: []byte("1")},
				{Signature: []byte("2")},
				{Signature: []byte("3")},
				{Signature: []byte("4")},
				{Signature: []byte("5")},
				{Signature: []byte("6")},
				{Signature: []byte("7")},
			},
			chunkSize: 3,
			expected: [][]*ethpb.SignedValidatorRegistrationV1{
				{
					{Signature: []byte("1")},
					{Signature: []byte("2")},
					{Signature: []byte("3")},
				},
				{
					{Signature: []byte("4")},
					{Signature: []byte("5")},
					{Signature: []byte("6")},
				},
				{
					{Signature: []byte("7")},
				},
			},
		},
		"Not enough items": {
			regs: []*ethpb.SignedValidatorRegistrationV1{
				{Signature: []byte("1")},
				{Signature: []byte("2")},
				{Signature: []byte("3")},
			},
			chunkSize: 42,
			expected: [][]*ethpb.SignedValidatorRegistrationV1{
				{
					{Signature: []byte("1")},
					{Signature: []byte("2")},
					{Signature: []byte("3")},
				},
			},
		},
		"Null chunk size": {
			regs: []*ethpb.SignedValidatorRegistrationV1{
				{Signature: []byte("1")},
				{Signature: []byte("2")},
				{Signature: []byte("3")},
			},
			chunkSize: 0,
			expected: [][]*ethpb.SignedValidatorRegistrationV1{
				{
					{Signature: []byte("1")},
					{Signature: []byte("2")},
					{Signature: []byte("3")},
				},
			},
		},
		"Negative chunk size": {
			regs: []*ethpb.SignedValidatorRegistrationV1{
				{Signature: []byte("1")},
				{Signature: []byte("2")},
				{Signature: []byte("3")},
			},
			chunkSize: -1,
			expected: [][]*ethpb.SignedValidatorRegistrationV1{
				{
					{Signature: []byte("1")},
					{Signature: []byte("2")},
					{Signature: []byte("3")},
				},
			},
		},
	}

	for name, test := range tests {
		t.Run(name, func(t *testing.T) {
			require.DeepEqual(t, test.expected, chunkSignedValidatorRegistrationV1(test.regs, test.chunkSize))
		})
	}
}
