package offchain

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	"cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/aminojson"
	"cosmossdk.io/x/tx/signing/direct"
	"cosmossdk.io/x/tx/signing/directaux"
	"cosmossdk.io/x/tx/signing/textual"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
)

const (
	addressCodecPrefix          = "cosmos"
	validatorAddressCodecPrefix = "cosmosvaloper"
	mnemonic                    = "have embark stumble card pistol fun gauge obtain forget oil awesome lottery unfold corn sure original exist siren pudding spread uphold dwarf goddess card"
)

func getCodec() codec.Codec {
	registry := testutil.CodecOptions{}.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)

	return codec.NewProtoCodec(registry)
}

func newGRPCCoinMetadataQueryFn(grpcConn grpc.ClientConnInterface) textual.CoinMetadataQueryFn {
	return func(ctx context.Context, denom string) (*bankv1beta1.Metadata, error) {
		bankQueryClient := bankv1beta1.NewQueryClient(grpcConn)
		res, err := bankQueryClient.DenomMetadata(ctx, &bankv1beta1.QueryDenomMetadataRequest{
			Denom: denom,
		})
		if err != nil {
			return nil, err
		}

		return res.Metadata, nil
	}
}

// testConfig fulfills client.TxConfig although SignModeHandler is the only method implemented.
type testConfig struct {
	handler *signing.HandlerMap
}

func (t testConfig) SignModeHandler() *signing.HandlerMap {
	return t.handler
}

func (t testConfig) TxEncoder() sdk.TxEncoder {
	return nil
}

func (t testConfig) TxDecoder() sdk.TxDecoder {
	return nil
}

func (t testConfig) TxJSONEncoder() sdk.TxEncoder {
	return nil
}

func (t testConfig) TxJSONDecoder() sdk.TxDecoder {
	return nil
}

func (t testConfig) MarshalSignatureJSON(v2s []signingtypes.SignatureV2) ([]byte, error) {
	return nil, nil
}

func (t testConfig) UnmarshalSignatureJSON(bytes []byte) ([]signingtypes.SignatureV2, error) {
	return nil, nil
}

func (t testConfig) NewTxBuilder() client.TxBuilder {
	return nil
}

func (t testConfig) WrapTxBuilder(s sdk.Tx) (client.TxBuilder, error) {
	return nil, nil
}

func (t testConfig) SigningContext() *signing.Context {
	return nil
}

func newTestConfig(t *testing.T) *testConfig {
	t.Helper()

	enabledSignModes := []signingtypes.SignMode{
		signingtypes.SignMode_SIGN_MODE_DIRECT,
		signingtypes.SignMode_SIGN_MODE_DIRECT_AUX,
		signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
		signingtypes.SignMode_SIGN_MODE_TEXTUAL,
	}

	var err error
	signingOptions := signing.Options{
		AddressCodec:          address.NewBech32Codec(addressCodecPrefix),
		ValidatorAddressCodec: address.NewBech32Codec(validatorAddressCodecPrefix),
	}
	signingContext, err := signing.NewContext(signingOptions)
	require.NoError(t, err)

	lenSignModes := len(enabledSignModes)
	handlers := make([]signing.SignModeHandler, lenSignModes)
	for i, m := range enabledSignModes {
		var err error
		switch m {
		case signingtypes.SignMode_SIGN_MODE_DIRECT:
			handlers[i] = &direct.SignModeHandler{}
		case signingtypes.SignMode_SIGN_MODE_DIRECT_AUX:
			handlers[i], err = directaux.NewSignModeHandler(directaux.SignModeHandlerOptions{
				TypeResolver:   signingOptions.TypeResolver,
				SignersContext: signingContext,
			})
			require.NoError(t, err)
		case signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON:
			handlers[i] = aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{
				FileResolver: signingOptions.FileResolver,
				TypeResolver: signingOptions.TypeResolver,
			})
		case signingtypes.SignMode_SIGN_MODE_TEXTUAL:
			handlers[i], err = textual.NewSignModeHandler(textual.SignModeOptions{
				CoinMetadataQuerier: newGRPCCoinMetadataQueryFn(client.Context{}),
				FileResolver:        signingOptions.FileResolver,
				TypeResolver:        signingOptions.TypeResolver,
			})
			require.NoError(t, err)
		}
	}

	handler := signing.NewHandlerMap(handlers...)
	return &testConfig{handler: handler}
}
