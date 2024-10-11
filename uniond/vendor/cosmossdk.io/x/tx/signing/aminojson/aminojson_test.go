package aminojson_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoregistry"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing/aminojson"
	"cosmossdk.io/x/tx/signing/testutil"
)

func TestAminoJsonSignMode(t *testing.T) {
	fee := &txv1beta1.Fee{
		Amount: []*basev1beta1.Coin{{Denom: "uatom", Amount: "1000"}},
	}
	handlerOptions := testutil.HandlerArgumentOptions{
		ChainID: "test-chain",
		Memo:    "sometestmemo",
		Msg: &bankv1beta1.MsgSend{
			FromAddress: "foo",
			ToAddress:   "bar",
			Amount:      []*basev1beta1.Coin{{Denom: "demon", Amount: "100"}},
		},
		AccNum:        1,
		AccSeq:        2,
		SignerAddress: "signerAddress",
		Fee:           fee,
	}

	testCases := []struct {
		name     string
		malleate func(opts testutil.HandlerArgumentOptions) testutil.HandlerArgumentOptions
		error    string
	}{
		{
			name: "happy path",
			malleate: func(opts testutil.HandlerArgumentOptions) testutil.HandlerArgumentOptions {
				return opts
			},
		},
		{
			name: "empty signer",
			malleate: func(opts testutil.HandlerArgumentOptions) testutil.HandlerArgumentOptions {
				opts.SignerAddress = ""
				return opts
			},
			error: "got empty address in SIGN_MODE_LEGACY_AMINO_JSON handler: invalid request",
		},
		{
			name: "nil fee",
			malleate: func(opts testutil.HandlerArgumentOptions) testutil.HandlerArgumentOptions {
				opts.Fee = nil
				return opts
			},
			error: "fee cannot be nil",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			opts := tc.malleate(handlerOptions)
			signerData, txData, err := testutil.MakeHandlerArguments(opts)
			require.NoError(t, err)

			handler := aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{})
			_, err = handler.GetSignBytes(context.Background(), signerData, txData)
			if tc.error != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.error)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestNewSignModeHandler(t *testing.T) {
	handler := aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{})
	require.NotNil(t, handler)
	aj := aminojson.NewEncoder(aminojson.EncoderOptions{})
	handler = aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{
		FileResolver: protoregistry.GlobalFiles,
		TypeResolver: protoregistry.GlobalTypes,
		Encoder:      &aj,
	})
	require.NotNil(t, handler)
}
