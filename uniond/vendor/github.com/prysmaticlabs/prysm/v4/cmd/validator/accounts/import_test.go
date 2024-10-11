package accounts

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

func TestImport_Noninteractive(t *testing.T) {
	local.ResetCaches()
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
	assert.Equal(t, len(accNames), 0)

	// Create 2 keys.
	createKeystore(t, keysDir)
	time.Sleep(time.Second)
	createKeystore(t, keysDir)

	require.NoError(t, accountsImport(cliCtx))

	w, err = wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir:      walletDir,
		WalletPassword: password,
	})
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	keys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)

	assert.Equal(t, 2, len(keys))
}

// TestImport_DuplicateKeys is a regression test that ensures correction function if duplicate keys are being imported
func TestImport_DuplicateKeys(t *testing.T) {
	local.ResetCaches()
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

	// Create a key and then copy it to create a duplicate
	_, keystorePath := createKeystore(t, keysDir)
	time.Sleep(time.Second)
	input, err := os.ReadFile(keystorePath)
	require.NoError(t, err)
	keystorePath2 := filepath.Join(keysDir, "copyOfKeystore.json")
	err = os.WriteFile(keystorePath2, input, os.ModePerm)
	require.NoError(t, err)

	require.NoError(t, accountsImport(cliCtx))

	_, err = wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir:      walletDir,
		WalletPassword: password,
	})
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	keys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)

	// There should only be 1 account as the duplicate keystore was ignored
	assert.Equal(t, 1, len(keys))
}

func TestImport_Noninteractive_RandomName(t *testing.T) {
	local.ResetCaches()
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
	assert.Equal(t, len(accNames), 0)

	// Create 2 keys.
	createRandomNameKeystore(t, keysDir)
	time.Sleep(time.Second)
	createRandomNameKeystore(t, keysDir)

	require.NoError(t, accountsImport(cliCtx))

	w, err = wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir:      walletDir,
		WalletPassword: password,
	})
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	keys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)

	assert.Equal(t, 2, len(keys))
}

// Returns the fullPath to the newly created keystore file.
func createRandomNameKeystore(t *testing.T, path string) (*keymanager.Keystore, string) {
	validatingKey, err := bls.RandKey()
	require.NoError(t, err)
	encryptor := keystorev4.New()
	cryptoFields, err := encryptor.Encrypt(validatingKey.Marshal(), password)
	require.NoError(t, err)
	id, err := uuid.NewRandom()
	require.NoError(t, err)
	keystoreFile := &keymanager.Keystore{
		Crypto:  cryptoFields,
		ID:      id.String(),
		Pubkey:  fmt.Sprintf("%x", validatingKey.PublicKey().Marshal()),
		Version: encryptor.Version(),
		Name:    encryptor.Name(),
	}
	encoded, err := json.MarshalIndent(keystoreFile, "", "\t")
	require.NoError(t, err)
	// Write the encoded keystore to disk with the timestamp appended
	random, err := rand.Int(rand.Reader, big.NewInt(1000000))
	require.NoError(t, err)
	fullPath := filepath.Join(path, fmt.Sprintf("test-%d-keystore", random.Int64()))
	require.NoError(t, os.WriteFile(fullPath, encoded, os.ModePerm))
	return keystoreFile, fullPath
}

func TestImport_Noninteractive_Filepath(t *testing.T) {
	local.ResetCaches()
	walletDir, passwordsDir, passwordFilePath := setupWalletAndPasswordsDir(t)
	keysDir := filepath.Join(t.TempDir(), "keysDir")
	require.NoError(t, os.MkdirAll(keysDir, os.ModePerm))

	_, keystorePath := createKeystore(t, keysDir)
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:           walletDir,
		passwordsDir:        passwordsDir,
		keysDir:             keystorePath,
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
	assert.Equal(t, len(accNames), 0)

	require.NoError(t, accountsImport(cliCtx))

	w, err = wallet.OpenWallet(cliCtx.Context, &wallet.Config{
		WalletDir:      walletDir,
		WalletPassword: password,
	})
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	keys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)

	assert.Equal(t, 1, len(keys))
}
