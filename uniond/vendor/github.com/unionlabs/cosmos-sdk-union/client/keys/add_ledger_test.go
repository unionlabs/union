//go:build ledger || test_ledger_mock
// +build ledger test_ledger_mock

package keys

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func Test_runAddCmdLedgerWithCustomCoinType(t *testing.T) {
	config := sdk.GetConfig()

	bech32PrefixAccAddr := "terra"
	bech32PrefixAccPub := "terrapub"
	bech32PrefixValAddr := "terravaloper"
	bech32PrefixValPub := "terravaloperpub"
	bech32PrefixConsAddr := "terravalcons"
	bech32PrefixConsPub := "terravalconspub"

	config.SetBech32PrefixForAccount(bech32PrefixAccAddr, bech32PrefixAccPub)
	config.SetBech32PrefixForValidator(bech32PrefixValAddr, bech32PrefixValPub)
	config.SetBech32PrefixForConsensusNode(bech32PrefixConsAddr, bech32PrefixConsPub)

	// Prepare a keybase
	kbHome := t.TempDir()

	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithCodec(cdc).
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	cmd := AddKeyCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())
	cmd.SetArgs([]string{
		"keyname1",
		fmt.Sprintf("--%s=true", flags.FlagUseLedger),
		fmt.Sprintf("--%s=0", flagAccount),
		fmt.Sprintf("--%s=0", flagIndex),
		fmt.Sprintf("--%s=330", flagCoinType),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})

	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	mockIn.Reset("test1234\ntest1234\n")
	require.NoError(t, cmd.ExecuteContext(ctx))

	// Now check that it has been stored properly
	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
	require.NoError(t, err)
	require.NotNil(t, kb)
	t.Cleanup(func() {
		_ = kb.Delete("keyname1")
	})
	mockIn.Reset("test1234\n")
	key1, err := kb.Key("keyname1")
	require.NoError(t, err)
	require.NotNil(t, key1)

	require.Equal(t, "keyname1", key1.Name)
	require.Equal(t, keyring.TypeLedger, key1.GetType())
	pub, err := key1.GetPubKey()
	require.NoError(t, err)
	require.Equal(t,
		"PubKeySecp256k1{03028F0D5A9FD41600191CDEFDEA05E77A68DFBCE286241C0190805B9346667D07}",
		pub.String())

	config.SetBech32PrefixForAccount(sdk.Bech32PrefixAccAddr, sdk.Bech32PrefixAccPub)
	config.SetBech32PrefixForValidator(sdk.Bech32PrefixValAddr, sdk.Bech32PrefixValPub)
	config.SetBech32PrefixForConsensusNode(sdk.Bech32PrefixConsAddr, sdk.Bech32PrefixConsPub)
}

func Test_runAddCmdLedger(t *testing.T) {
	cmd := AddKeyCommand()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())

	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	kbHome := t.TempDir()
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec

	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithCodec(cdc).
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	cmd.SetArgs([]string{
		"keyname1",
		fmt.Sprintf("--%s=true", flags.FlagUseLedger),
		fmt.Sprintf("--%s=%s", flags.FlagOutput, flags.OutputFormatText),
		fmt.Sprintf("--%s=%s", flags.FlagKeyType, hd.Secp256k1Type),
		fmt.Sprintf("--%s=%d", flagCoinType, sdk.CoinType),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	mockIn.Reset("test1234\ntest1234\n")

	require.NoError(t, cmd.ExecuteContext(ctx))

	// Now check that it has been stored properly
	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
	require.NoError(t, err)

	// Now check that it has been stored properly
	require.NotNil(t, kb)
	t.Cleanup(func() {
		_ = kb.Delete("keyname1")
	})

	mockIn.Reset("test1234\n")
	key1, err := kb.Key("keyname1")
	require.NoError(t, err)
	require.NotNil(t, key1)

	require.Equal(t, "keyname1", key1.Name)
	pub, err := key1.GetPubKey()
	require.NoError(t, err)
	require.Equal(t,
		"PubKeySecp256k1{034FEF9CD7C4C63588D3B03FEB5281B9D232CBA34D6F3D71AEE59211FFBFE1FE87}",
		pub.String())
}

func Test_runAddCmdLedgerDryRun(t *testing.T) {
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	testData := []struct {
		name  string
		args  []string
		added bool
	}{
		{
			name: "ledger account is added",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "false"),
				fmt.Sprintf("--%s=%s", flags.FlagUseLedger, "true"),
			},
			added: true,
		},
		{
			name: "ledger account is not added with dry run",
			args: []string{
				"testkey",
				fmt.Sprintf("--%s=%s", flags.FlagDryRun, "true"),
				fmt.Sprintf("--%s=%s", flags.FlagUseLedger, "true"),
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
				WithKeyringDir(kbHome).
				WithKeyring(kb).
				WithCodec(cdc).
				WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
				WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
				WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))
			ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)
			b := bytes.NewBufferString("")
			cmd.SetOut(b)

			cmd.SetArgs(tt.args)
			require.NoError(t, cmd.ExecuteContext(ctx))

			if tt.added {
				_, err = kb.Key("testkey")
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
