package accounts

import (
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	"github.com/urfave/cli/v2"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

const (
	passwordFileName = "password.txt"
	password         = "OhWOWthisisatest42!$"
)

func setupWalletAndPasswordsDir(t testing.TB) (string, string, string) {
	walletDir := filepath.Join(t.TempDir(), "wallet")
	passwordsDir := filepath.Join(t.TempDir(), "passwords")
	passwordFileDir := filepath.Join(t.TempDir(), "passwordFile")
	require.NoError(t, os.MkdirAll(passwordFileDir, params.BeaconIoConfig().ReadWriteExecutePermissions))
	passwordFilePath := filepath.Join(passwordFileDir, passwordFileName)
	require.NoError(t, os.WriteFile(passwordFilePath, []byte(password), os.ModePerm))
	return walletDir, passwordsDir, passwordFilePath
}

// Returns the fullPath to the newly created keystore file.
func createKeystore(t *testing.T, path string) (*keymanager.Keystore, string) {
	validatingKey, err := bls.RandKey()
	require.NoError(t, err)
	encryptor := keystorev4.New()
	cryptoFields, err := encryptor.Encrypt(validatingKey.Marshal(), password)
	require.NoError(t, err)
	id, err := uuid.NewRandom()
	require.NoError(t, err)
	keystoreFile := &keymanager.Keystore{
		Crypto:      cryptoFields,
		ID:          id.String(),
		Pubkey:      fmt.Sprintf("%x", validatingKey.PublicKey().Marshal()),
		Version:     encryptor.Version(),
		Description: encryptor.Name(),
	}
	encoded, err := json.MarshalIndent(keystoreFile, "", "\t")
	require.NoError(t, err)
	// Write the encoded keystore to disk with the timestamp appended
	createdAt := prysmTime.Now().Unix()
	fullPath := filepath.Join(path, fmt.Sprintf(local.KeystoreFileNameFormat, createdAt))
	require.NoError(t, os.WriteFile(fullPath, encoded, os.ModePerm))
	return keystoreFile, fullPath
}

type testWalletConfig struct {
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
	passwordsDir            string
	walletDir               string
}

func setupWalletCtx(
	tb testing.TB,
	cfg *testWalletConfig,
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

func TestDeleteAccounts_Noninteractive(t *testing.T) {
	walletDir, _, passwordFilePath := setupWalletAndPasswordsDir(t)
	// Write a directory where we will import keys from.
	keysDir := filepath.Join(t.TempDir(), "keysDir")
	require.NoError(t, os.MkdirAll(keysDir, os.ModePerm))

	// Create 3 keystore files in the keys directory we can then
	// import from in our wallet.
	k1, _ := createKeystore(t, keysDir)
	time.Sleep(time.Second)
	k2, _ := createKeystore(t, keysDir)
	time.Sleep(time.Second)
	k3, _ := createKeystore(t, keysDir)
	generatedPubKeys := []string{k1.Pubkey, k2.Pubkey, k3.Pubkey}
	// Only delete keys 0 and 1.
	deletePublicKeys := strings.Join(generatedPubKeys[0:2], ",")

	// We initialize a wallet with a local keymanager.
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		// Wallet configuration flags.
		walletDir:           walletDir,
		keymanagerKind:      keymanager.Local,
		walletPasswordFile:  passwordFilePath,
		accountPasswordFile: passwordFilePath,
		// Flags required for ImportAccounts to work.
		keysDir: keysDir,
		// Flags required for DeleteAccounts to work.
		deletePublicKeys: deletePublicKeys,
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

	// We attempt to import accounts.
	require.NoError(t, accountsImport(cliCtx))

	// We attempt to delete the accounts specified.
	require.NoError(t, accountsDelete(cliCtx))

	km, err := local.NewKeymanager(
		cliCtx.Context,
		&local.SetupConfig{
			Wallet:           w,
			ListenForChanges: false,
		},
	)
	require.NoError(t, err)
	remainingAccounts, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, len(remainingAccounts), 1)
	remainingPublicKey, err := hex.DecodeString(k3.Pubkey)
	require.NoError(t, err)
	assert.DeepEqual(t, remainingAccounts[0], bytesutil.ToBytes48(remainingPublicKey))
}
