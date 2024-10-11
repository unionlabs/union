package signing_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	"cosmossdk.io/x/tx/signing"
)

var (
	_ signing.SignModeHandler = directHandler{}
	_ signing.SignModeHandler = aminoJSONHandler{}
)

type directHandler struct{}

func (s directHandler) Mode() signingv1beta1.SignMode {
	return signingv1beta1.SignMode_SIGN_MODE_DIRECT
}

func (s directHandler) GetSignBytes(_ context.Context, _ signing.SignerData, _ signing.TxData) ([]byte, error) {
	panic("not implemented")
}

type aminoJSONHandler struct{}

func (s aminoJSONHandler) Mode() signingv1beta1.SignMode {
	return signingv1beta1.SignMode_SIGN_MODE_LEGACY_AMINO_JSON
}

func (s aminoJSONHandler) GetSignBytes(_ context.Context, _ signing.SignerData, _ signing.TxData) ([]byte, error) {
	panic("not implemented")
}

func TestNewHandlerMap(t *testing.T) {
	require.PanicsWithValue(t, "nil handler", func() {
		signing.NewHandlerMap(nil)
	})
	require.PanicsWithValue(t, "no handlers", func() {
		signing.NewHandlerMap()
	})
	dh := directHandler{}
	ah := aminoJSONHandler{}
	handlerMap := signing.NewHandlerMap(dh, aminoJSONHandler{})
	require.Equal(t, dh.Mode(), handlerMap.DefaultMode())
	require.NotEqual(t, ah.Mode(), handlerMap.DefaultMode())
}
