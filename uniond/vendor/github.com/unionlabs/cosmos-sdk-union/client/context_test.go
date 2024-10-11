package client_test

import (
	"bytes"
	"encoding/json"
	"os"
	"testing"

	"github.com/spf13/viper"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestMain(m *testing.M) {
	viper.Set(flags.FlagKeyringBackend, keyring.BackendMemory)
	os.Exit(m.Run())
}

func TestContext_PrintProto(t *testing.T) {
	ctx := client.Context{}

	animal := &testdata.Dog{
		Size_: "big",
		Name:  "Spot",
	}
	anyAnimal, err := types.NewAnyWithValue(animal)
	require.NoError(t, err)
	hasAnimal := &testdata.HasAnimal{
		Animal: anyAnimal,
		X:      10,
	}

	// proto
	registry := testdata.NewTestInterfaceRegistry()
	ctx = ctx.WithCodec(codec.NewProtoCodec(registry))

	// json
	buf := &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = flags.OutputFormatJSON
	err = ctx.PrintProto(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`{"animal":{"@type":"/testpb.Dog","size":"big","name":"Spot"},"x":"10"}
`, buf.String())

	// yaml
	buf = &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = flags.OutputFormatText
	err = ctx.PrintProto(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`animal:
  '@type': /testpb.Dog
  name: Spot
  size: big
x: "10"
`, buf.String())
}

func TestContext_PrintRaw(t *testing.T) {
	ctx := client.Context{}
	hasAnimal := json.RawMessage(`{"animal":{"@type":"/testpb.Dog","size":"big","name":"Spot"},"x":"10"}`)

	// json
	buf := &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = flags.OutputFormatJSON
	err := ctx.PrintRaw(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`{"animal":{"@type":"/testpb.Dog","size":"big","name":"Spot"},"x":"10"}
`, buf.String())

	// yaml
	buf = &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = flags.OutputFormatText
	err = ctx.PrintRaw(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`animal:
  '@type': /testpb.Dog
  name: Spot
  size: big
x: "10"
`, buf.String())
}

func TestGetFromFields(t *testing.T) {
	cfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	path := hd.CreateHDPath(118, 0, 0).String()

	testCases := []struct {
		clientCtx   client.Context
		keyring     func() keyring.Keyring
		from        string
		expectedErr string
	}{
		{
			clientCtx: client.Context{}.WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).WithKeyringDefaultKeyName("alice"),
			keyring: func() keyring.Keyring {
				kb := keyring.NewInMemory(cfg.Codec)

				_, _, err := kb.NewMnemonic("alice", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)

				return kb
			},
			from: "",
		},
		{
			clientCtx: client.Context{}.WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				kb := keyring.NewInMemory(cfg.Codec)

				_, _, err := kb.NewMnemonic("alice", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)

				return kb
			},
			from: "alice",
		},
		{
			clientCtx: client.Context{}.WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				kb, err := keyring.New(t.Name(), keyring.BackendTest, t.TempDir(), nil, cfg.Codec)
				require.NoError(t, err)

				_, _, err = kb.NewMnemonic("alice", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)

				return kb
			},
			from: "alice",
		},
		{
			clientCtx: client.Context{}.WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:        "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
			expectedErr: "key with given address not found: key not found",
		},
		{
			clientCtx: client.Context{}.WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				kb, err := keyring.New(t.Name(), keyring.BackendTest, t.TempDir(), nil, cfg.Codec)
				require.NoError(t, err)
				return kb
			},
			from:        "alice",
			expectedErr: "alice.info: key not found",
		},
		{
			clientCtx: client.Context{}.WithSimulation(true).WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from: "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
		},
		{
			clientCtx: client.Context{}.WithSimulation(true).WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:        "alice",
			expectedErr: "a valid address must be provided in simulation mode",
		},
		{
			clientCtx: client.Context{}.WithGenerateOnly(true).WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from: "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
		},
		{
			clientCtx: client.Context{}.WithGenerateOnly(true).WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:        "alice",
			expectedErr: "alice.info: key not found",
		},
		{
			clientCtx: client.Context{}.WithGenerateOnly(true).WithAddressCodec(addresscodec.NewBech32Codec("cosmos")),
			keyring: func() keyring.Keyring {
				kb, err := keyring.New(t.Name(), keyring.BackendTest, t.TempDir(), nil, cfg.Codec)
				require.NoError(t, err)

				_, _, err = kb.NewMnemonic("alice", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)

				return kb
			},
			from: "alice",
		},
	}

	for _, tc := range testCases {
		_, _, _, err := client.GetFromFields(tc.clientCtx, tc.keyring(), tc.from)
		if tc.expectedErr == "" {
			require.NoError(t, err)
		} else {
			require.ErrorContains(t, err, tc.expectedErr)
		}
	}
}
