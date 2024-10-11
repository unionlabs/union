package accounts

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	"github.com/prysmaticlabs/prysm/v4/validator/node"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

func TestWalletWithKeymanager(t *testing.T) {
	logHook := test.NewGlobal()
	walletDir, passwordsDir, passwordFilePath := setupWalletAndPasswordsDir(t)
	keysDir := filepath.Join(t.TempDir(), "keysDir")
	require.NoError(t, os.MkdirAll(keysDir, os.ModePerm))

	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:           walletDir,
		passwordsDir:        passwordsDir,
		keysDir:             keysDir,
		keymanagerKind:      keymanager.Local,
		walletPasswordFile:  passwordFilePath,
		accountPasswordFile: passwordFilePath,
	})

	opts := []accounts.Option{
		accounts.WithWalletDir(walletDir),
		accounts.WithKeymanagerType(keymanager.Local),
		accounts.WithWalletPassword(password),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(cliCtx.Context)
	require.NoError(t, err)
	newKm, err := local.NewKeymanager(
		cliCtx.Context,
		&local.SetupConfig{
			Wallet:           w,
			ListenForChanges: false,
		},
	)
	require.NoError(t, err)

	// Make sure there are no accounts at the start.
	accNames, err := newKm.ValidatingAccountNames()
	require.NoError(t, err)
	require.Equal(t, len(accNames), 0)

	// Create 2 keys.
	createKeystore(t, keysDir)
	time.Sleep(time.Second)
	createKeystore(t, keysDir)
	require.NoError(t, accountsImport(cliCtx))

	w, k, err := walletWithKeymanager(cliCtx)
	require.NoError(t, err)
	keys, err := k.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, len(keys), 2)
	require.Equal(t, w.KeymanagerKind(), keymanager.Local)

	assert.LogsContain(t, logHook, fmt.Sprintf("Imported accounts"))
	assert.LogsContain(t, logHook, hexutil.Encode(keys[0][:])[2:])
	assert.LogsContain(t, logHook, hexutil.Encode(keys[1][:])[2:])
}

func TestWalletWithKeymanager_web3signer(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.Web3SignerURLFlag.Name, "http://localhost:12345", "web3signer")
	c := &cli.StringSliceFlag{
		Name: "validators-external-signer-public-keys",
	}
	err := c.Apply(set)
	require.NoError(t, err)
	require.NoError(t, set.Set(flags.Web3SignerURLFlag.Name, "http://localhost:12345"))
	require.NoError(t, set.Set(flags.Web3SignerPublicValidatorKeysFlag.Name, "0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820"))
	ctx := cli.NewContext(&app, set, nil)
	bytes, err := hexutil.Decode("0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2")
	require.NoError(t, err)
	config, err := node.Web3SignerConfig(ctx)
	require.NoError(t, err)
	config.GenesisValidatorsRoot = bytes
	w, k, err := walletWithWeb3SignerKeymanager(ctx, config)
	require.NoError(t, err)
	keys, err := k.FetchValidatingPublicKeys(ctx.Context)
	require.NoError(t, err)
	require.Equal(t, len(keys), 1)
	require.Equal(t, w.KeymanagerKind(), keymanager.Web3Signer)
}
