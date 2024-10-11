package wallet

import (
	"context"
	"flag"
	"os"
	"path/filepath"
	"strconv"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	"github.com/urfave/cli/v2"
)

const (
	walletDirName    = "wallet"
	mnemonicFileName = "mnemonic.txt"
	mnemonic         = "garage car helmet trade salmon embrace market giant movie wet same champion dawn chair shield drill amazing panther accident puzzle garden mosquito kind arena"
)

type recoverCfgStruct struct {
	walletDir        string
	passwordFilePath string
	mnemonicFilePath string
	numAccounts      int64
}

func setupRecoverCfg(t *testing.T) *recoverCfgStruct {
	testDir := t.TempDir()
	walletDir := filepath.Join(testDir, walletDirName)
	passwordFilePath := filepath.Join(testDir, passwordFileName)
	require.NoError(t, os.WriteFile(passwordFilePath, []byte(password), os.ModePerm))
	mnemonicFilePath := filepath.Join(testDir, mnemonicFileName)
	require.NoError(t, os.WriteFile(mnemonicFilePath, []byte(mnemonic), os.ModePerm))

	return &recoverCfgStruct{
		walletDir:        walletDir,
		passwordFilePath: passwordFilePath,
		mnemonicFilePath: mnemonicFilePath,
	}
}

func createRecoverCliCtx(t *testing.T, cfg *recoverCfgStruct) *cli.Context {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.WalletDirFlag.Name, cfg.walletDir, "")
	set.String(flags.WalletPasswordFileFlag.Name, cfg.passwordFilePath, "")
	set.String(flags.KeymanagerKindFlag.Name, keymanager.Derived.String(), "")
	set.String(flags.MnemonicFileFlag.Name, cfg.mnemonicFilePath, "")
	set.String(flags.MnemonicLanguageFlag.Name, derived.DefaultMnemonicLanguage, "")
	set.Bool(flags.SkipMnemonic25thWordCheckFlag.Name, true, "")
	set.Int64(flags.NumAccountsFlag.Name, cfg.numAccounts, "")
	assert.NoError(t, set.Set(flags.SkipMnemonic25thWordCheckFlag.Name, "true"))
	assert.NoError(t, set.Set(flags.WalletDirFlag.Name, cfg.walletDir))
	assert.NoError(t, set.Set(flags.WalletPasswordFileFlag.Name, cfg.passwordFilePath))
	assert.NoError(t, set.Set(flags.KeymanagerKindFlag.Name, keymanager.Derived.String()))
	assert.NoError(t, set.Set(flags.MnemonicFileFlag.Name, cfg.mnemonicFilePath))
	assert.NoError(t, set.Set(flags.MnemonicLanguageFlag.Name, derived.DefaultMnemonicLanguage))
	assert.NoError(t, set.Set(flags.NumAccountsFlag.Name, strconv.Itoa(int(cfg.numAccounts))))
	return cli.NewContext(&app, set, nil)
}

func TestRecoverDerivedWallet(t *testing.T) {
	cfg := setupRecoverCfg(t)
	cfg.numAccounts = 4
	cliCtx := createRecoverCliCtx(t, cfg)
	require.NoError(t, walletRecover(cliCtx))

	ctx := context.Background()
	w, err := wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir:      cfg.walletDir,
		WalletPassword: password,
	})
	assert.NoError(t, err)

	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	derivedKM, ok := km.(*derived.Keymanager)
	if !ok {
		t.Fatal("not a derived keymanager")
	}
	names, err := derivedKM.ValidatingAccountNames(ctx)
	assert.NoError(t, err)
	require.Equal(t, len(names), int(cfg.numAccounts))
}

// TestRecoverDerivedWallet_OneAccount is a test for regression in cases where the number of accounts recovered is 1
func TestRecoverDerivedWallet_OneAccount(t *testing.T) {
	cfg := setupRecoverCfg(t)
	cfg.numAccounts = 1
	cliCtx := createRecoverCliCtx(t, cfg)
	require.NoError(t, walletRecover(cliCtx))

	_, err := wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir:      cfg.walletDir,
		WalletPassword: password,
	})
	assert.NoError(t, err)
}

func TestRecoverDerivedWallet_AlreadyExists(t *testing.T) {
	cfg := setupRecoverCfg(t)
	cfg.numAccounts = 4
	cliCtx := createRecoverCliCtx(t, cfg)
	require.NoError(t, walletRecover(cliCtx))

	// Trying to recover an HD wallet into a directory that already exists should give an error
	require.ErrorContains(t, "a wallet already exists at this location", walletRecover(cliCtx))
}
