package keys

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"testing"

	"github.com/cosmos/go-bip39"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func Test_runAddCmdBasic(t *testing.T) {
	cmd := AddKeyCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())

	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	kbHome := t.TempDir()

	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
	require.NoError(t, err)

	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithInput(mockIn).
		WithCodec(cdc).
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	t.Cleanup(func() {
		_ = kb.Delete("keyname1")
		_ = kb.Delete("keyname2")
	})

	// test empty name
	cmd.SetArgs([]string{
		"",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	mockIn.Reset("y\n")
	require.ErrorContains(t, cmd.ExecuteContext(ctx), "the provided name is invalid or empty after trimming whitespace")

	cmd.SetArgs([]string{
		"keyname1",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	mockIn.Reset("y\n")
	require.NoError(t, cmd.ExecuteContext(ctx))

	mockIn.Reset("N\n")
	require.Error(t, cmd.ExecuteContext(ctx))

	cmd.SetArgs([]string{
		"keyname2",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})

	require.NoError(t, cmd.ExecuteContext(ctx))
	require.Error(t, cmd.ExecuteContext(ctx))

	mockIn.Reset("y\n")
	require.NoError(t, cmd.ExecuteContext(ctx))

	cmd.SetArgs([]string{
		"keyname4",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})

	require.NoError(t, cmd.ExecuteContext(ctx))
	require.Error(t, cmd.ExecuteContext(ctx))

	cmd.SetArgs([]string{
		"keyname5",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=true", flags.FlagDryRun),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
	})

	require.NoError(t, cmd.ExecuteContext(ctx))

	// In recovery mode
	cmd.SetArgs([]string{
		"keyname6",
		fmt.Sprintf("--%s=true", flagRecover),
	})

	// use valid mnemonic and complete recovery key generation successfully
	mockIn.Reset("decide praise business actor peasant farm drastic weather extend front hurt later song give verb rhythm worry fun pond reform school tumble august one\n")
	require.NoError(t, cmd.ExecuteContext(ctx))

	// use invalid mnemonic and fail recovery key generation
	mockIn.Reset("invalid mnemonic\n")
	require.Error(t, cmd.ExecuteContext(ctx))

	// In interactive mode
	cmd.SetArgs([]string{
		"keyname7",
		"-i",
		fmt.Sprintf("--%s=false", flagRecover),
	})

	const password = "password1!"

	// set password default interactive key generation successfully
	mockIn.Reset("\n\n")
	require.NoError(t, cmd.ExecuteContext(ctx))

	// set password and complete interactive key generation successfully
	mockIn.Reset("\n" + password + "\n" + password + "\n")
	require.NoError(t, cmd.ExecuteContext(ctx))

	// passwords don't match and fail interactive key generation
	mockIn.Reset("\n" + password + "\n" + "fail" + "\n")
	require.Error(t, cmd.ExecuteContext(ctx))
}

func Test_runAddCmdMultisigDupKeys(t *testing.T) {
	cmd := AddKeyCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())

	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	kbHome := t.TempDir()

	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
	require.NoError(t, err)

	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithInput(mockIn).
		WithCodec(cdc).
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	t.Cleanup(func() {
		_ = kb.Delete("keyname1")
		_ = kb.Delete("keyname2")
		_ = kb.Delete("multisigname")
	})

	cmd.SetArgs([]string{
		"keyname1",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	require.NoError(t, cmd.ExecuteContext(ctx))

	cmd.SetArgs([]string{
		"keyname2",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	require.NoError(t, cmd.ExecuteContext(ctx))

	cmd.SetArgs([]string{
		"multisigname",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
		fmt.Sprintf("--%s=%s", flagMultisig, "keyname1,keyname2"),
		fmt.Sprintf("--%s=%s", flagMultiSigThreshold, "2"),
	})
	require.NoError(t, cmd.ExecuteContext(ctx))

	cmd.SetArgs([]string{
		"multisigname",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
		fmt.Sprintf("--%s=%s", flagMultisig, "keyname1,keyname1"),
		fmt.Sprintf("--%s=%s", flagMultiSigThreshold, "2"),
	})
	mockIn.Reset("y\n")
	require.Error(t, cmd.ExecuteContext(ctx))
	mockIn.Reset("y\n")
	require.EqualError(t, cmd.ExecuteContext(ctx), "duplicate multisig keys: keyname1")
}

func Test_runAddCmdDryRun(t *testing.T) {
	pubkey1 := `{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AtObiFVE4s+9+RX5SP8TN9r2mxpoaT4eGj9CJfK7VRzN"}`
	pubkey2 := `{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"A/se1vkqgdQ7VJQCM4mxN+L+ciGhnnJ4XYsQCRBMrdRi"}`
	b64Pubkey := "QWhnOHhpdXBJcGZ2UlR2ak5la1ExclROUThTOW96YjdHK2RYQmFLVjl4aUo="
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec

	testData := []struct {
		name  string
		args  []string
		added bool
	}{
		{
			name: "account is added",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "false"),
			},
			added: true,
		},
		{
			name: "account is not added with dry run",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "true"),
			},
			added: false,
		},
		{
			name: "multisig account is added",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "false"),
				fmt.Sprintf("--%s=%s", flagMultisig, "subkey"),
			},
			added: true,
		},
		{
			name: "multisig account is not added with dry run",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "true"),
				fmt.Sprintf("--%s=%s", flagMultisig, "subkey"),
			},
			added: false,
		},
		{
			name: "pubkey account is added",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "false"),
				fmt.Sprintf("--%s=%s", FlagPublicKey, pubkey1),
			},
			added: true,
		},
		{
			name: "pubkey account is not added with dry run",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "true"),
				fmt.Sprintf("--%s=%s", FlagPublicKey, pubkey2),
			},
			added: false,
		},
		{
			name: "base64 pubkey account is added",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "false"),
				fmt.Sprintf("--%s=%s", flagPubKeyBase64, b64Pubkey),
			},
			added: true,
		},
		{
			name: "base64 pubkey account is not added with dry run",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "true"),
				fmt.Sprintf("--%s=%s", flagPubKeyBase64, b64Pubkey),
			},
			added: false,
		},
	}
	for _, tt := range testData {
		t.Run(tt.name, func(t *testing.T) {
			cmd := AddKeyCommand()
			cmd.Flags().AddFlagSet(Commands().PersistentFlags())

			kbHome := t.TempDir()
			mockIn := testutil.ApplyMockIODiscardOutErr(cmd)

			kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
			require.NoError(t, err)

			clientCtx := client.Context{}.
				WithCodec(cdc).
				WithKeyringDir(kbHome).
				WithKeyring(kb).
				WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
				WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
				WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))
			ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

			path := sdk.GetFullBIP44Path()
			_, err = kb.NewAccount("subkey", testdata.TestMnemonic, "", path, hd.Secp256k1)
			require.NoError(t, err)

			t.Cleanup(func() {
				_ = kb.Delete("subkey")
			})

			b := bytes.NewBufferString("")
			cmd.SetOut(b)

			cmd.SetArgs(tt.args)
			require.NoError(t, cmd.ExecuteContext(ctx))

			if tt.added {
				_, err := kb.Key("testkey")
				require.NoError(t, err)

				out, err := io.ReadAll(b)
				require.NoError(t, err)
				require.Contains(t, string(out), "name: testkey")
			} else {
				_, err = kb.Key("testkey")
				require.Error(t, err)
				require.Equal(t, "testkey.info: key not found", err.Error())
			}
		})
	}
}

func TestAddRecoverFileBackend(t *testing.T) {
	cmd := AddKeyCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec

	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	kbHome := t.TempDir()

	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithInput(mockIn).
		WithCodec(cdc).
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	cmd.SetArgs([]string{
		"keyname1",
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendFile),
		fmt.Sprintf("--%s", flagRecover),
	})

	keyringPassword := "12345678"

	entropySeed, err := bip39.NewEntropy(mnemonicEntropySize)
	require.NoError(t, err)

	mnemonic, err := bip39.NewMnemonic(entropySeed)
	require.NoError(t, err)

	mockIn.Reset(fmt.Sprintf("%s\n%s\n%s\n", mnemonic, keyringPassword, keyringPassword))
	require.NoError(t, cmd.ExecuteContext(ctx))

	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendFile, kbHome, mockIn, cdc)
	require.NoError(t, err)

	t.Cleanup(func() {
		mockIn.Reset(fmt.Sprintf("%s\n%s\n", keyringPassword, keyringPassword))
		_ = kb.Delete("keyname1")
	})

	mockIn.Reset(fmt.Sprintf("%s\n%s\n", keyringPassword, keyringPassword))
	k, err := kb.Key("keyname1")
	require.NoError(t, err)
	require.Equal(t, "keyname1", k.Name)
}
