package wallet

import (
	"flag"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

const (
	passwordFileName = "password.txt"
	password         = "OhWOWthisisatest42!$"
)

// `cmd/validator/accounts/delete_test.go`. https://pastebin.com/2n2VB7Ez is
// the error I couldn't get around.
func SetupWalletAndPasswordsDir(t testing.TB) (string, string, string) {
	walletDir := filepath.Join(t.TempDir(), "wallet")
	passwordsDir := filepath.Join(t.TempDir(), "passwords")
	passwordFileDir := filepath.Join(t.TempDir(), "passwordFile")
	require.NoError(t, os.MkdirAll(passwordFileDir, params.BeaconIoConfig().ReadWriteExecutePermissions))
	passwordFilePath := filepath.Join(passwordFileDir, passwordFileName)
	require.NoError(t, os.WriteFile(passwordFilePath, []byte(password), os.ModePerm))
	return walletDir, passwordsDir, passwordFilePath
}

type TestWalletConfig struct {
	exitAll                 bool
	skipDepositConfirm      bool
	keymanagerKind          keymanager.Kind
	numAccounts             int64
	grpcHeaders             string
	privateKeyFile          string
	accountPasswordFile     string
	walletPasswordFile      string
	backupPasswordFile      string
	backupPublicKeys        string
	voluntaryExitPublicKeys string
	deletePublicKeys        string
	keysDir                 string
	backupDir               string
	walletDir               string
	passwordsDir            string
}

func SetupWalletCtx(
	tb testing.TB,
	cfg *TestWalletConfig,
) *cli.Context {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.WalletDirFlag.Name, cfg.walletDir, "")
	set.String(flags.KeysDirFlag.Name, cfg.keysDir, "")
	set.String(flags.KeymanagerKindFlag.Name, cfg.keymanagerKind.String(), "")
	set.String(flags.DeletePublicKeysFlag.Name, cfg.deletePublicKeys, "")
	set.String(flags.VoluntaryExitPublicKeysFlag.Name, cfg.voluntaryExitPublicKeys, "")
	set.String(flags.BackupDirFlag.Name, cfg.backupDir, "")
	set.String(flags.BackupPasswordFile.Name, cfg.backupPasswordFile, "")
	set.String(flags.BackupPublicKeysFlag.Name, cfg.backupPublicKeys, "")
	set.String(flags.WalletPasswordFileFlag.Name, cfg.walletPasswordFile, "")
	set.String(flags.AccountPasswordFileFlag.Name, cfg.accountPasswordFile, "")
	set.Int64(flags.NumAccountsFlag.Name, cfg.numAccounts, "")
	set.Bool(flags.SkipDepositConfirmationFlag.Name, cfg.skipDepositConfirm, "")
	set.Bool(flags.SkipMnemonic25thWordCheckFlag.Name, true, "")
	set.Bool(flags.ExitAllFlag.Name, cfg.exitAll, "")
	set.String(flags.GrpcHeadersFlag.Name, cfg.grpcHeaders, "")

	if cfg.privateKeyFile != "" {
		set.String(flags.ImportPrivateKeyFileFlag.Name, cfg.privateKeyFile, "")
		assert.NoError(tb, set.Set(flags.ImportPrivateKeyFileFlag.Name, cfg.privateKeyFile))
	}
	assert.NoError(tb, set.Set(flags.WalletDirFlag.Name, cfg.walletDir))
	assert.NoError(tb, set.Set(flags.SkipMnemonic25thWordCheckFlag.Name, "true"))
	assert.NoError(tb, set.Set(flags.KeysDirFlag.Name, cfg.keysDir))
	assert.NoError(tb, set.Set(flags.KeymanagerKindFlag.Name, cfg.keymanagerKind.String()))
	assert.NoError(tb, set.Set(flags.DeletePublicKeysFlag.Name, cfg.deletePublicKeys))
	assert.NoError(tb, set.Set(flags.VoluntaryExitPublicKeysFlag.Name, cfg.voluntaryExitPublicKeys))
	assert.NoError(tb, set.Set(flags.BackupDirFlag.Name, cfg.backupDir))
	assert.NoError(tb, set.Set(flags.BackupPublicKeysFlag.Name, cfg.backupPublicKeys))
	assert.NoError(tb, set.Set(flags.BackupPasswordFile.Name, cfg.backupPasswordFile))
	assert.NoError(tb, set.Set(flags.WalletPasswordFileFlag.Name, cfg.walletPasswordFile))
	assert.NoError(tb, set.Set(flags.AccountPasswordFileFlag.Name, cfg.accountPasswordFile))
	assert.NoError(tb, set.Set(flags.NumAccountsFlag.Name, strconv.Itoa(int(cfg.numAccounts))))
	assert.NoError(tb, set.Set(flags.SkipDepositConfirmationFlag.Name, strconv.FormatBool(cfg.skipDepositConfirm)))
	assert.NoError(tb, set.Set(flags.ExitAllFlag.Name, strconv.FormatBool(cfg.exitAll)))
	assert.NoError(tb, set.Set(flags.GrpcHeadersFlag.Name, cfg.grpcHeaders))
	return cli.NewContext(&app, set, nil)
}

func init() {
	logrus.SetLevel(logrus.DebugLevel)
	logrus.SetOutput(io.Discard)
}

func TestCreateOrOpenWallet(t *testing.T) {
	hook := logTest.NewGlobal()
	walletDir, passwordsDir, walletPasswordFile := SetupWalletAndPasswordsDir(t)
	cliCtx := SetupWalletCtx(t, &TestWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		keymanagerKind:     keymanager.Local,
		walletPasswordFile: walletPasswordFile,
	})

	createdWallet, err := wallet.OpenWalletOrElseCli(cliCtx, wallet.OpenOrCreateNewWallet)
	require.NoError(t, err)
	require.LogsContain(t, hook, "Successfully created new wallet")

	openedWallet, err := wallet.OpenWalletOrElseCli(cliCtx, wallet.OpenOrCreateNewWallet)
	require.NoError(t, err)
	assert.Equal(t, createdWallet.KeymanagerKind(), openedWallet.KeymanagerKind())
	assert.Equal(t, createdWallet.AccountsDir(), openedWallet.AccountsDir())
}

func TestCreateWallet_Local(t *testing.T) {
	walletDir, passwordsDir, walletPasswordFile := SetupWalletAndPasswordsDir(t)
	cliCtx := SetupWalletCtx(t, &TestWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		keymanagerKind:     keymanager.Local,
		walletPasswordFile: walletPasswordFile,
	})

	// We attempt to create the wallet.
	_, err := CreateAndSaveWalletCli(cliCtx)
	require.NoError(t, err)

	// We attempt to open the newly created wallet.
	w, err := wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir: walletDir,
	})
	assert.NoError(t, err)
	_, err = w.ReadFileAtPath(cliCtx.Context, local.AccountsPath, local.AccountsKeystoreFileName)
	require.NoError(t, err)
}

func TestCreateWallet_Derived(t *testing.T) {
	walletDir, passwordsDir, passwordFile := SetupWalletAndPasswordsDir(t)
	cliCtx := SetupWalletCtx(t, &TestWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		walletPasswordFile: passwordFile,
		keymanagerKind:     keymanager.Derived,
		numAccounts:        1,
	})

	// We attempt to create the wallet.
	_, err := CreateAndSaveWalletCli(cliCtx)
	require.NoError(t, err)

	// We attempt to open the newly created wallet.
	_, err = wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir: walletDir,
	})
	assert.NoError(t, err)
}

// TestCreateWallet_WalletAlreadyExists checks for expected error if trying to create a wallet when there is one already.
func TestCreateWallet_WalletAlreadyExists(t *testing.T) {
	walletDir, passwordsDir, passwordFile := SetupWalletAndPasswordsDir(t)
	cliCtx := SetupWalletCtx(t, &TestWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		walletPasswordFile: passwordFile,
		keymanagerKind:     keymanager.Derived,
		numAccounts:        1,
	})

	// We attempt to create the wallet.
	_, err := CreateAndSaveWalletCli(cliCtx)
	require.NoError(t, err)

	// We attempt to create another wallet of the same type at the same location. We expect an error.
	_, err = CreateAndSaveWalletCli(cliCtx)
	require.ErrorContains(t, "already exists", err)

	cliCtx = SetupWalletCtx(t, &TestWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		walletPasswordFile: passwordFile,
		keymanagerKind:     keymanager.Local,
	})

	// We attempt to create another wallet of different type at the same location. We expect an error.
	_, err = CreateAndSaveWalletCli(cliCtx)
	require.ErrorContains(t, "already exists", err)
}

func TestInputKeymanagerKind(t *testing.T) {
	tests := []struct {
		name    string
		args    string
		want    keymanager.Kind
		wantErr bool
	}{
		{
			name:    "local returns local kind",
			args:    "local",
			want:    keymanager.Local,
			wantErr: false,
		},
		{
			name:    "direct returns local kind",
			args:    "direct",
			want:    keymanager.Local,
			wantErr: false,
		},
		{
			name:    "imported returns local kind",
			args:    "imported",
			want:    keymanager.Local,
			wantErr: false,
		},
		{
			name:    "derived returns derived kind",
			args:    "derived",
			want:    keymanager.Derived,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			app := cli.App{}
			set := flag.NewFlagSet("test", 0)
			set.String(flags.KeymanagerKindFlag.Name, tt.args, "")
			assert.NoError(t, set.Set(flags.KeymanagerKindFlag.Name, tt.args))
			cliCtx := cli.NewContext(&app, set, nil)
			got, err := inputKeymanagerKind(cliCtx)
			assert.NoError(t, err)
			assert.Equal(t, tt.want.String(), got.String())
		})
	}
}
