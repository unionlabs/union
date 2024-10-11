package keys

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func Test_runImportCmd(t *testing.T) {
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	testCases := []struct {
		name           string
		keyringBackend string
		userInput      string
		expectError    bool
	}{
		{
			name:           "test backend success",
			keyringBackend: keyring.BackendTest,
			// key armor passphrase
			userInput: "123456789\n",
		},
		{
			name:           "test backend fail with wrong armor pass",
			keyringBackend: keyring.BackendTest,
			userInput:      "987654321\n",
			expectError:    true,
		},
		{
			name:           "file backend success",
			keyringBackend: keyring.BackendFile,
			// key armor passphrase + keyring password x2
			userInput: "123456789\n12345678\n12345678\n",
		},
		{
			name:           "file backend fail with wrong armor pass",
			keyringBackend: keyring.BackendFile,
			userInput:      "987654321\n12345678\n12345678\n",
			expectError:    true,
		},
		{
			name:           "file backend fail with wrong keyring pass",
			keyringBackend: keyring.BackendFile,
			userInput:      "123465789\n12345678\n87654321\n",
			expectError:    true,
		},
		{
			name:           "file backend fail with no keyring pass",
			keyringBackend: keyring.BackendFile,
			userInput:      "123465789\n",
			expectError:    true,
		},
	}

	armoredKey := `-----BEGIN TENDERMINT PRIVATE KEY-----
salt: A790BB721D1C094260EA84F5E5B72289
kdf: bcrypt

HbP+c6JmeJy9JXe2rbbF1QtCX1gLqGcDQPBXiCtFvP7/8wTZtVOPj8vREzhZ9ElO
3P7YnrzPQThG0Q+ZnRSbl9MAS8uFAM4mqm5r/Ys=
=f3l4
-----END TENDERMINT PRIVATE KEY-----
`

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			cmd := ImportKeyCommand()
			cmd.Flags().AddFlagSet(Commands().PersistentFlags())
			mockIn := testutil.ApplyMockIODiscardOutErr(cmd)

			// Now add a temporary keybase
			kbHome := filepath.Join(t.TempDir(), fmt.Sprintf("kbhome-%s", tc.name))
			// Create dir, otherwise os.WriteFile will fail
			if _, err := os.Stat(kbHome); os.IsNotExist(err) {
				err = os.MkdirAll(kbHome, 0o700)
				require.NoError(t, err)
			}
			kb, err := keyring.New(sdk.KeyringServiceName(), tc.keyringBackend, kbHome, nil, cdc)
			require.NoError(t, err)

			clientCtx := client.Context{}.
				WithKeyringDir(kbHome).
				WithKeyring(kb).
				WithInput(mockIn).
				WithCodec(cdc)
			ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

			t.Cleanup(cleanupKeys(t, kb, "keyname1"))

			keyfile := filepath.Join(kbHome, "key.asc")
			require.NoError(t, os.WriteFile(keyfile, []byte(armoredKey), 0o600))

			defer func() {
				_ = os.RemoveAll(kbHome)
			}()

			mockIn.Reset(tc.userInput)
			cmd.SetArgs([]string{
				"keyname1", keyfile,
				fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, tc.keyringBackend),
			})

			err = cmd.ExecuteContext(ctx)
			if tc.expectError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func Test_runImportHexCmd(t *testing.T) {
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	testCases := []struct {
		name           string
		keyringBackend string
		hexKey         string
		keyType        string
		expectError    bool
	}{
		{
			name:           "test backend success",
			keyringBackend: keyring.BackendTest,
			hexKey:         "0xa3e57952e835ed30eea86a2993ac2a61c03e74f2085b3635bd94aa4d7ae0cfdf",
			keyType:        "secp256k1",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			cmd := ImportKeyHexCommand()
			cmd.Flags().AddFlagSet(Commands().PersistentFlags())
			mockIn := testutil.ApplyMockIODiscardOutErr(cmd)

			// Now add a temporary keybase
			kbHome := filepath.Join(t.TempDir(), fmt.Sprintf("kbhome-%s", tc.name))
			kb, err := keyring.New(sdk.KeyringServiceName(), tc.keyringBackend, kbHome, nil, cdc)
			require.NoError(t, err)

			clientCtx := client.Context{}.
				WithKeyringDir(kbHome).
				WithKeyring(kb).
				WithInput(mockIn).
				WithCodec(cdc)
			ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

			t.Cleanup(cleanupKeys(t, kb, "keyname1"))

			defer func() {
				_ = os.RemoveAll(kbHome)
			}()

			cmd.SetArgs([]string{
				"keyname1", tc.hexKey,
				fmt.Sprintf("--%s=%s", flags.FlagKeyType, tc.keyType),
				fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, tc.keyringBackend),
			})

			err = cmd.ExecuteContext(ctx)
			if tc.expectError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func Test_runImportCmdWithEmptyName(t *testing.T) {
	cmd := ImportKeyCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())
	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	// Now add a temporary keybase
	kbHome := t.TempDir()
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
	require.NoError(t, err)

	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithKeyring(kb).
		WithInput(mockIn).
		WithCodec(cdc)
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)
	cmd.SetArgs([]string{
		"", "fake-file",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})

	require.ErrorContains(t, cmd.ExecuteContext(ctx), "the provided name is invalid or empty after trimming whitespace")

	cmd = ImportKeyHexCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())
	testutil.ApplyMockIODiscardOutErr(cmd)
	cmd.SetArgs([]string{
		"", "fake-hex",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})

	require.ErrorContains(t, cmd.ExecuteContext(ctx), "the provided name is invalid or empty after trimming whitespace")
}
