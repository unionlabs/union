package accounts

import (
	"archive/zip"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	constant "github.com/prysmaticlabs/prysm/v4/validator/testing"
)

func TestBackupAccounts_Noninteractive_Derived(t *testing.T) {
	walletDir, _, passwordFilePath := setupWalletAndPasswordsDir(t)
	// Specify the password locally to this file for convenience.
	password := "Pa$sW0rD0__Fo0xPr"
	require.NoError(t, os.WriteFile(passwordFilePath, []byte(password), os.ModePerm))

	// Write a directory where we will backup accounts to.
	backupDir := filepath.Join(t.TempDir(), "backupDir")
	require.NoError(t, os.MkdirAll(backupDir, params.BeaconIoConfig().ReadWriteExecutePermissions))

	// Write a password for the accounts we wish to backup to a file.
	backupPasswordFile := filepath.Join(backupDir, "backuppass.txt")
	err := os.WriteFile(
		backupPasswordFile,
		[]byte("Passw0rdz4938%%"),
		params.BeaconIoConfig().ReadWritePermissions,
	)
	require.NoError(t, err)

	// We initialize a wallet with a derived keymanager.
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		// Wallet configuration flags.
		walletDir:          walletDir,
		keymanagerKind:     keymanager.Derived,
		walletPasswordFile: passwordFilePath,
		// Flags required for BackupAccounts to work.
		backupPasswordFile: backupPasswordFile,
		backupDir:          backupDir,
	})
	opts := []accounts.Option{
		accounts.WithWalletDir(walletDir),
		accounts.WithKeymanagerType(keymanager.Derived),
		accounts.WithWalletPassword(password),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(cliCtx.Context)
	require.NoError(t, err)

	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	// Create 2 accounts
	derivedKM, ok := km.(*derived.Keymanager)
	require.Equal(t, true, ok)
	err = derivedKM.RecoverAccountsFromMnemonic(cliCtx.Context, constant.TestMnemonic, derived.DefaultMnemonicLanguage, "", 2)
	require.NoError(t, err)

	// Obtain the public keys of the accounts we created
	pubkeys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	var generatedPubKeys []string
	for _, pubkey := range pubkeys {
		encoded := make([]byte, hex.EncodedLen(len(pubkey)))
		hex.Encode(encoded, pubkey[:])
		generatedPubKeys = append(generatedPubKeys, string(encoded))
	}
	backupPublicKeys := strings.Join(generatedPubKeys, ",")

	// Recreate a cliCtx with the addition of these backup keys to be later used by the backup process
	cliCtx = setupWalletCtx(t, &testWalletConfig{
		// Wallet configuration flags.
		walletDir:          walletDir,
		keymanagerKind:     keymanager.Derived,
		walletPasswordFile: passwordFilePath,
		// Flags required for BackupAccounts to work.
		backupPublicKeys:   backupPublicKeys,
		backupPasswordFile: backupPasswordFile,
		backupDir:          backupDir,
	})

	// Next, we attempt to backup the accounts.
	require.NoError(t, accountsBackup(cliCtx))

	// We check a backup.zip file was created at the output path.
	zipFilePath := filepath.Join(backupDir, accounts.ArchiveFilename)
	assert.DeepEqual(t, true, file.Exists(zipFilePath))

	// We attempt to unzip the file and verify the keystores do match our accounts.
	f, err := os.Open(zipFilePath)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, f.Close())
	}()
	fi, err := f.Stat()
	require.NoError(t, err)
	r, err := zip.NewReader(f, fi.Size())
	require.NoError(t, err)

	// We check we have 2 keystore files in the unzipped results.
	require.DeepEqual(t, 2, len(r.File))
	unzippedPublicKeys := make([]string, 2)
	for i, unzipped := range r.File {
		ff, err := unzipped.Open()
		require.NoError(t, err)
		encodedBytes, err := io.ReadAll(ff)
		require.NoError(t, err)
		keystoreFile := &keymanager.Keystore{}
		require.NoError(t, json.Unmarshal(encodedBytes, keystoreFile))
		require.NoError(t, ff.Close())
		unzippedPublicKeys[i] = keystoreFile.Pubkey
	}
	sort.Strings(unzippedPublicKeys)
	sort.Strings(generatedPubKeys)
	assert.DeepEqual(t, unzippedPublicKeys, generatedPubKeys)
}

func TestBackupAccounts_Noninteractive_Imported(t *testing.T) {
	walletDir, _, passwordFilePath := setupWalletAndPasswordsDir(t)
	// Write a directory where we will import keys from.
	keysDir := filepath.Join(t.TempDir(), "keysDir")
	require.NoError(t, os.MkdirAll(keysDir, params.BeaconIoConfig().ReadWriteExecutePermissions))

	// Write a directory where we will backup accounts to.
	backupDir := filepath.Join(t.TempDir(), "backupDir")
	require.NoError(t, os.MkdirAll(backupDir, params.BeaconIoConfig().ReadWriteExecutePermissions))

	// Create 2 keystore files in the keys directory we can then
	// import from in our wallet.
	k1, _ := createKeystore(t, keysDir)
	time.Sleep(time.Second)
	k2, _ := createKeystore(t, keysDir)
	generatedPubKeys := []string{k1.Pubkey, k2.Pubkey}
	backupPublicKeys := strings.Join(generatedPubKeys, ",")

	// Write a password for the accounts we wish to backup to a file.
	backupPasswordFile := filepath.Join(backupDir, "backuppass.txt")
	err := os.WriteFile(
		backupPasswordFile,
		[]byte("Passw0rdz4938%%"),
		params.BeaconIoConfig().ReadWritePermissions,
	)
	require.NoError(t, err)

	// We initialize a wallet with a imported keymanager.
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		// Wallet configuration flags.
		walletDir:           walletDir,
		keymanagerKind:      keymanager.Local,
		walletPasswordFile:  passwordFilePath,
		accountPasswordFile: passwordFilePath,
		// Flags required for ImportAccounts to work.
		keysDir: keysDir,
		// Flags required for BackupAccounts to work.
		backupPublicKeys:   backupPublicKeys,
		backupPasswordFile: backupPasswordFile,
		backupDir:          backupDir,
	})
	opts := []accounts.Option{
		accounts.WithWalletDir(walletDir),
		accounts.WithKeymanagerType(keymanager.Local),
		accounts.WithWalletPassword(password),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	_, err = acc.WalletCreate(cliCtx.Context)
	require.NoError(t, err)

	// We attempt to import accounts we wrote to the keys directory
	// into our newly created wallet.
	require.NoError(t, accountsImport(cliCtx))

	// Next, we attempt to backup the accounts.
	require.NoError(t, accountsBackup(cliCtx))

	// We check a backup.zip file was created at the output path.
	zipFilePath := filepath.Join(backupDir, accounts.ArchiveFilename)
	assert.DeepEqual(t, true, file.Exists(zipFilePath))

	// We attempt to unzip the file and verify the keystores do match our accounts.
	f, err := os.Open(zipFilePath)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, f.Close())
	}()
	fi, err := f.Stat()
	require.NoError(t, err)
	r, err := zip.NewReader(f, fi.Size())
	require.NoError(t, err)

	// We check we have 2 keystore files in the unzipped results.
	require.DeepEqual(t, 2, len(r.File))
	unzippedPublicKeys := make([]string, 2)
	for i, unzipped := range r.File {
		ff, err := unzipped.Open()
		require.NoError(t, err)
		encodedBytes, err := io.ReadAll(ff)
		require.NoError(t, err)
		keystoreFile := &keymanager.Keystore{}
		require.NoError(t, json.Unmarshal(encodedBytes, keystoreFile))
		require.NoError(t, ff.Close())
		unzippedPublicKeys[i] = keystoreFile.Pubkey
	}
	sort.Strings(unzippedPublicKeys)
	sort.Strings(generatedPubKeys)
	assert.DeepEqual(t, unzippedPublicKeys, generatedPubKeys)
}
