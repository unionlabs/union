package codec_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	authmodulev1 "cosmossdk.io/api/cosmos/auth/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/depinject"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/codec"
)

var _ address.Codec = (*customAddressCodec)(nil)

type customAddressCodec struct{}

func (c customAddressCodec) StringToBytes(text string) ([]byte, error) {
	return []byte(text), nil
}

func (c customAddressCodec) BytesToString(bz []byte) (string, error) {
	return string(bz), nil
}

func AuthConfig() *authmodulev1.Module { return &authmodulev1.Module{Bech32Prefix: "cosmos"} }

func TestProvideAddressCodec(t *testing.T) {
	var addrCodec address.Codec
	var valAddressCodec address.ValidatorAddressCodec
	var consAddressCodec address.ConsensusAddressCodec

	err := depinject.Inject(
		depinject.Provide(
			AuthConfig,
			codec.ProvideAddressCodec,
		),
		&addrCodec, &valAddressCodec, &consAddressCodec)
	require.NoError(t, err)
	require.NotNil(t, addrCodec)
	_, ok := addrCodec.(customAddressCodec)
	require.False(t, ok)
	require.NotNil(t, valAddressCodec)
	_, ok = valAddressCodec.(customAddressCodec)
	require.False(t, ok)
	require.NotNil(t, consAddressCodec)
	_, ok = consAddressCodec.(customAddressCodec)
	require.False(t, ok)

	// Set the address codec to the custom one
	err = depinject.Inject(
		depinject.Configs(
			depinject.Provide(AuthConfig, codec.ProvideAddressCodec),
			depinject.Supply(
				log.NewNopLogger(),
				func() address.Codec { return customAddressCodec{} },
				func() address.ValidatorAddressCodec { return customAddressCodec{} },
				func() address.ConsensusAddressCodec { return customAddressCodec{} },
			),
		),
		&addrCodec, &valAddressCodec, &consAddressCodec)
	require.NoError(t, err)
	require.NotNil(t, addrCodec)
	_, ok = addrCodec.(customAddressCodec)
	require.True(t, ok)
	require.NotNil(t, valAddressCodec)
	_, ok = valAddressCodec.(customAddressCodec)
	require.True(t, ok)
	require.NotNil(t, consAddressCodec)
	_, ok = consAddressCodec.(customAddressCodec)
	require.True(t, ok)
}
