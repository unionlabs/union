package client_test

import (
	"bytes"
	"encoding/json"
	"os"
	"strings"
	"testing"

	"github.com/spf13/viper"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
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
	any, err := types.NewAnyWithValue(animal)
	require.NoError(t, err)
	hasAnimal := &testdata.HasAnimal{
		Animal: any,
		X:      10,
	}

	// proto
	registry := testdata.NewTestInterfaceRegistry()
	ctx = ctx.WithCodec(codec.NewProtoCodec(registry))

	// json
	buf := &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = "json"
	err = ctx.PrintProto(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`{"animal":{"@type":"/testpb.Dog","size":"big","name":"Spot"},"x":"10"}
`, buf.String())

	// yaml
	buf = &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = "text"
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

func TestContext_PrintObjectLegacy(t *testing.T) {
	ctx := client.Context{}

	animal := &testdata.Dog{
		Size_: "big",
		Name:  "Spot",
	}
	any, err := types.NewAnyWithValue(animal)
	require.NoError(t, err)
	hasAnimal := &testdata.HasAnimal{
		Animal: any,
		X:      10,
	}

	// amino
	amino := testdata.NewTestAmino()
	ctx = ctx.WithLegacyAmino(&codec.LegacyAmino{Amino: amino})

	// json
	buf := &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = "json"
	err = ctx.PrintObjectLegacy(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`{"type":"testpb/HasAnimal","value":{"animal":{"type":"testpb/Dog","value":{"size":"big","name":"Spot"}},"x":"10"}}
`, buf.String())

	// yaml
	buf = &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = "text"
	err = ctx.PrintObjectLegacy(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`type: testpb/HasAnimal
value:
  animal:
    type: testpb/Dog
    value:
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
	ctx.OutputFormat = "json"
	err := ctx.PrintRaw(hasAnimal)
	require.NoError(t, err)
	require.Equal(t,
		`{"animal":{"@type":"/testpb.Dog","size":"big","name":"Spot"},"x":"10"}
`, buf.String())

	// yaml
	buf = &bytes.Buffer{}
	ctx = ctx.WithOutput(buf)
	ctx.OutputFormat = "text"
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
	cfg := testutil.MakeTestEncodingConfig()
	path := hd.CreateHDPath(118, 0, 0).String()

	testCases := []struct {
		clientCtx   client.Context
		keyring     func() keyring.Keyring
		from        string
		expectedErr string
	}{
		{
			keyring: func() keyring.Keyring {
				kb := keyring.NewInMemory(cfg.Codec)

				_, _, err := kb.NewMnemonic("alice", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)

				return kb
			},
			from: "alice",
		},
		{
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
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:        "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
			expectedErr: "key with address cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5 not found: key not found",
		},
		{
			keyring: func() keyring.Keyring {
				kb, err := keyring.New(t.Name(), keyring.BackendTest, t.TempDir(), nil, cfg.Codec)
				require.NoError(t, err)
				return kb
			},
			from:        "alice",
			expectedErr: "alice.info: key not found",
		},
		{
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:      "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
			clientCtx: client.Context{}.WithSimulation(true),
		},
		{
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:        "alice",
			clientCtx:   client.Context{}.WithSimulation(true),
			expectedErr: "a valid bech32 address must be provided in simulation mode",
		},
		{
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:      "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
			clientCtx: client.Context{}.WithGenerateOnly(true),
		},
		{
			keyring: func() keyring.Keyring {
				return keyring.NewInMemory(cfg.Codec)
			},
			from:        "alice",
			clientCtx:   client.Context{}.WithGenerateOnly(true),
			expectedErr: "alice.info: key not found",
		},
		{
			keyring: func() keyring.Keyring {
				kb, err := keyring.New(t.Name(), keyring.BackendTest, t.TempDir(), nil, cfg.Codec)
				require.NoError(t, err)

				_, _, err = kb.NewMnemonic("alice", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)

				return kb
			},
			clientCtx: client.Context{}.WithGenerateOnly(true),
			from:      "alice",
		},
	}

	for _, tc := range testCases {
		_, _, _, err := client.GetFromFields(tc.clientCtx, tc.keyring(), tc.from)
		if tc.expectedErr == "" {
			require.NoError(t, err)
		} else {
			require.True(t, strings.HasPrefix(err.Error(), tc.expectedErr))
		}
	}
}
